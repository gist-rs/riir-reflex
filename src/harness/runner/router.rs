//! The T2.5 MODELLESS teacher router (riir-train Issue 629 T2.5) — routes
//! over CAPTURED teacher columns (RIDT bins over TRAIN rows), never over
//! live forwards and never over a test row. The modelless-first mandate:
//! before any fine-tune (T3), a sigmoid-gated per-suite pick between the
//! captured teacher columns may already clear the bar — the fine-tune then
//! has to beat the ROUTER, not the raw teacher.
//!
//! Law (the issue, verdict round 1): "a sigmoid-gated, per-benchmark-family
//! router that picks the teacher's column per family (or blends under a
//! sigmoid gate — never softmax)". Per suite:
//!
//! 1. load each source capture's RIDT column;
//! 2. verify the CROSS-CAPTURE JOIN by the per-row gold pins (every row's
//!    gold must agree across captures — same datasets dir, same survivor
//!    predicate, same order) and the classes array (exact equality);
//! 3. recompute each column's train-row accuracy from the bins (never
//!    trust the manifest's number for a gate input);
//! 4. gate `g = sigmoid((acc_a − acc_b) / T)` with `T = 0.01` (a fixed,
//!    documented temperature — differences under ~1 pp blend near 50/50,
//!    gaps ≥ 3 pp approach a hard pick; sigmoid, never softmax);
//! 5. blend per row `q_route = g·q_a + (1−g)·q_b`, pick = argmax at the
//!    LOWEST position (the map_targets tie law);
//! 6. emit the routed RIDT column (the same byte layout — a drop-in
//!    teacher artifact for T3's student / T4's frozen read) + the router
//!    manifest. Single-source suites pass through BYTE-IDENTICAL (copied
//!    with their sidecar; the source provenance header is preserved).
//!
//! Honest-read caveats carried in the manifest: every accuracy here is a
//! TRAIN-row number (the captures' no-cheat law) — the routed artifact's
//! verdict is T4's single frozen read; nothing here selects on it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::distill::{read_teacher_bin, TeacherBinWriter};
use super::{git_sha, iso8601_utc};
use crate::harness::runner::hostname_refusing_unknown;

/// The gate temperature (fixed, no fitted degrees of freedom): a 1 pp
/// accuracy gap moves the gate one logit; sigmoid, never softmax.
pub const ROUTER_TEMPERATURE: f64 = 0.01;

#[derive(Debug, serde::Serialize)]
pub struct RouterSource {
    pub dir: String,
    pub bin_path: PathBuf,
    pub teacher: String,
    pub checkpoint: String,
    pub blake3: String,
    pub n_rows: usize,
    pub accuracy: f64,
    pub mean_max_prob: f64,
    pub mean_margin: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct RouterSuite {
    pub name: String,
    pub sources: Vec<RouterSource>,
    /// `Some(g)` = routed blend; `None` = passthrough (single source).
    pub gate: Option<f64>,
    pub routed_accuracy: Option<f64>,
    pub routed_blake3: Option<String>,
    pub n_classes: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct RouterReport {
    pub inputs: Vec<String>,
    pub temperature: f64,
    pub suites: Vec<RouterSuite>,
    pub skipped: Vec<String>,
}

fn argmax_lowest_pos(q: &[f32]) -> usize {
    let mut best = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    for (i, &v) in q.iter().enumerate() {
        if v > best_v {
            best_v = v;
            best = i;
        }
    }
    best
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// One source dir's discovered columns: `<suite>_teacher.bin` → (path, blake3).
fn discover_bins(dir: &Path) -> Result<BTreeMap<String, (PathBuf, String)>, String> {
    let mut out = BTreeMap::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with("_teacher.bin") {
            continue;
        }
        let Some(suite) = name.strip_suffix("_teacher.bin") else {
            continue;
        };
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let seal = blake3::hash(&bytes).to_hex()[..16].to_string();
        out.insert(suite.to_string(), (path, seal));
    }
    if out.is_empty() {
        return Err(format!(
            "{}: no *_teacher.bin captures found (pass the CAPTURE dirs, not the bin files)",
            dir.display()
        ));
    }
    Ok(out)
}

fn load_source(dir: &Path, suite: &str, path: &Path, seal: String) -> Result<(RouterSource, super::distill::TeacherColumn), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let col = read_teacher_bin(&bytes).map_err(|e| format!("{} ({suite}): {e}", path.display()))?;
    let n_rows = col.gold.len();
    if n_rows == 0 {
        return Err(format!("{} ({suite}): zero rows", path.display()));
    }
    let mut hits = 0usize;
    let mut max_prob_sum = 0.0f64;
    let mut margin_sum = 0.0f64;
    for (i, q) in col.q.iter().enumerate() {
        hits += usize::from(col.pick[i] == col.gold[i]);
        let mut sorted = q.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        max_prob_sum += f64::from(sorted.first().copied().unwrap_or(0.0));
        if sorted.len() >= 2 {
            margin_sum += f64::from(sorted[0] - sorted[1]);
        }
    }
    let teacher = col
        .header
        .get("teacher")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let checkpoint = col
        .header
        .get("checkpoint")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let src = RouterSource {
        dir: dir.display().to_string(),
        bin_path: path.to_path_buf(),
        teacher,
        checkpoint,
        blake3: seal,
        n_rows,
        accuracy: hits as f64 / n_rows as f64,
        mean_max_prob: max_prob_sum / n_rows as f64,
        mean_margin: margin_sum / n_rows as f64,
    };
    Ok((src, col))
}

/// Route over the given capture dirs (>= 2 for any routing to happen; a
/// single dir is a legal passthrough-only invocation, refused by the CLI).
pub fn run_teacher_router(input_dirs: &[PathBuf], out_dir: &Path) -> Result<RouterReport, String> {
    if input_dirs.is_empty() {
        return Err("teacher-router: no input capture dirs".into());
    }
    // suite -> (dir index, path, seal) — one column per dir max.
    let mut by_suite: BTreeMap<String, Vec<(usize, PathBuf, String)>> = BTreeMap::new();
    for (i, dir) in input_dirs.iter().enumerate() {
        for (suite, (path, seal)) in discover_bins(dir)? {
            by_suite.entry(suite).or_default().push((i, path, seal));
        }
    }

    let mut suites = Vec::new();
    let mut skipped = Vec::new();
    for (suite, columns) in &by_suite {
        let dirs_with: Vec<usize> = {
            let mut d: Vec<usize> = columns.iter().map(|(i, _, _)| *i).collect();
            d.sort_unstable();
            d.dedup();
            d
        };
        if columns.len() > 2 {
            skipped.push(format!(
                "{suite}: {} columns from {} dir(s) — the router is pairwise; pass exactly the \
                 two dirs that own this suite",
                columns.len(),
                dirs_with.len()
            ));
            continue;
        }
        if columns.len() == 1 {
            // Passthrough: byte-copy the bin + sidecar; provenance rides the
            // ORIGINAL header (no re-encode drift).
            let (_, path, seal) = &columns[0];
            let (src, col) = load_source(&input_dirs[columns[0].0], suite, path, seal.clone())?;
            std::fs::create_dir_all(out_dir)
                .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
            let dest = out_dir.join(format!("{suite}_teacher.bin"));
            let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            std::fs::write(&dest, &bytes).map_err(|e| format!("write {}: {e}", dest.display()))?;
            let sidecar_src = path.with_extension("bin.blake3");
            if sidecar_src.exists()
                && let Ok(side) = std::fs::read_to_string(&sidecar_src)
            {
                let _ = std::fs::write(
                    dest.with_extension("bin.blake3"),
                    side.trim_end().to_string() + "\n",
                );
            }
            let n_classes = col
                .header
                .get("classes")
                .and_then(|v| v.as_array())
                .map_or(0, |a| a.len());
            drop(col);
            suites.push(RouterSuite {
                name: suite.clone(),
                sources: vec![src],
                gate: None,
                routed_accuracy: None,
                routed_blake3: Some(seal.clone()),
                n_classes,
            });
            continue;
        }
        // Two columns (two distinct dirs — same-dir duplicates are impossible:
        // one bin per suite per dir).
        let (i_a, path_a, seal_a) = &columns[0];
        let (i_b, path_b, seal_b) = &columns[1];
        let (src_a, col_a) = load_source(&input_dirs[*i_a], suite, path_a, seal_a.clone())?;
        let (src_b, col_b) = load_source(&input_dirs[*i_b], suite, path_b, seal_b.clone())?;

        // JOIN VERIFICATION — classes equality + per-row gold pins.
        let classes_a = col_a.header.get("classes").cloned().unwrap_or(serde_json::Value::Null);
        let classes_b = col_b.header.get("classes").cloned().unwrap_or(serde_json::Value::Null);
        if classes_a != classes_b {
            skipped.push(format!(
                "{suite}: classes universes differ across captures — refusing the join"
            ));
            continue;
        }
        if col_a.gold.len() != col_b.gold.len() {
            skipped.push(format!(
                "{suite}: row counts differ ({} vs {}) — different datasets posture or \
                 survivor predicate; refusing the join",
                col_a.gold.len(),
                col_b.gold.len()
            ));
            continue;
        }
        if col_a.gold.iter().zip(col_b.gold.iter()).any(|(g1, g2)| g1 != g2) {
            skipped.push(format!(
                "{suite}: per-row gold pins disagree — the captures are not row-aligned; \
                 refusing the join"
            ));
            continue;
        }

        // Gate + blend.
        let gate = sigmoid((src_a.accuracy - src_b.accuracy) / ROUTER_TEMPERATURE);
        let n_classes = col_a.q.first().map_or(0, Vec::len);
        let datasets_dir = col_a
            .header
            .get("datasets_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let classes = classes_a;
        let n_rows = col_a.gold.len();
        let header = serde_json::json!({
            "suite": suite,
            "teacher": "router",
            "checkpoint": format!(
                "g={gate:.4};a={}@{};b={}@{}",
                src_a.teacher, src_a.checkpoint, src_b.teacher, src_b.checkpoint
            ),
            "git_sha": git_sha(),
            "datasets_dir": datasets_dir,
            "row_rule": "router blend over row-aligned captured columns (train rows only, the \
                         no-cheat law); gold pins verified across sources at route time",
            "classes": classes,
            "n_rows": n_rows,
            "created_utc": iso8601_utc(),
            "router": {
                "gate": gate,
                "temperature": ROUTER_TEMPERATURE,
                "law": "g = sigmoid((acc_a - acc_b) / T) per suite; q_route = g*q_a + (1-g)*q_b; \
                        sigmoid, never softmax",
                "sources": [
                    { "dir": src_a.dir, "teacher": src_a.teacher, "checkpoint": src_a.checkpoint,
                      "blake3": src_a.blake3, "train_accuracy": src_a.accuracy },
                    { "dir": src_b.dir, "teacher": src_b.teacher, "checkpoint": src_b.checkpoint,
                      "blake3": src_b.blake3, "train_accuracy": src_b.accuracy },
                ],
            },
        });
        let mut writer = TeacherBinWriter::new(&header)?;
        let mut routed_hits = 0usize;
        for r in 0..n_rows {
            let qa = &col_a.q[r];
            let qb = &col_b.q[r];
            let q_route: Vec<f32> = qa
                .iter()
                .zip(qb.iter())
                .map(|(&a, &b)| (gate * f64::from(a) + (1.0 - gate) * f64::from(b)) as f32)
                .collect();
            let pick = argmax_lowest_pos(&q_route);
            let gold = col_a.gold[r] as usize;
            routed_hits += usize::from(pick == gold);
            writer.push_row(gold as u32, pick as u32, &q_route);
        }
        let out_bytes = writer.into_bytes();
        std::fs::create_dir_all(out_dir)
            .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
        let dest = out_dir.join(format!("{suite}_teacher.bin"));
        std::fs::write(&dest, &out_bytes).map_err(|e| format!("write {}: {e}", dest.display()))?;
        let seal = blake3::hash(&out_bytes).to_hex();
        std::fs::write(dest.with_extension("bin.blake3"), format!("{seal}\n"))
            .map_err(|e| format!("write sidecar: {e}"))?;
        suites.push(RouterSuite {
            name: suite.clone(),
            sources: vec![src_a, src_b],
            gate: Some(gate),
            routed_accuracy: Some(routed_hits as f64 / n_rows as f64),
            routed_blake3: Some(seal.as_str()[..16].to_string()),
            n_classes,
        });
    }

    Ok(RouterReport {
        inputs: input_dirs.iter().map(|p| p.display().to_string()).collect(),
        temperature: ROUTER_TEMPERATURE,
        suites,
        skipped,
    })
}

/// The markdown rendering (the record's table block).
#[must_use]
pub fn render_router_markdown(report: &RouterReport) -> String {
    let mut s = String::new();
    s.push_str("# Teacher router — sigmoid-gated per-suite blend over captured columns (riir-train Issue 629 T2.5)\n\n");
    s.push_str(&format!(
        "- date {} · sha {} · host {}\n- inputs {}\n- temperature {} (sigmoid, never softmax; g = sigmoid((acc_a-acc_b)/T))\n- law: train-row captures only — the routed artifact's verdict is T4's single frozen read; nothing here selects on it\n",
        iso8601_utc(),
        git_sha().unwrap_or_else(|| "unknown".into()),
        hostname_refusing_unknown(),
        report.inputs.join(", "),
        report.temperature,
    ));
    s.push_str("\n| suite | sources (acc) | gate g | routed acc | mode | classes | routed blake3 |\n");
    s.push_str("|---|---|---|---|---|---|---|\n");
    for v in &report.suites {
        let srcs = v
            .sources
            .iter()
            .map(|src| format!("{} {:.4}", src.teacher, src.accuracy))
            .collect::<Vec<_>>()
            .join(" ⊕ ");
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            v.name,
            srcs,
            v.gate.map(|g| format!("{g:.4}")).unwrap_or_else(|| "—".into()),
            v.routed_accuracy
                .map(|a| format!("{a:.4}"))
                .unwrap_or_else(|| "—".into()),
            if v.gate.is_some() { "blend" } else { "passthrough" },
            v.n_classes,
            v.routed_blake3.clone().unwrap_or_else(|| "—".into()),
        ));
    }
    if !report.skipped.is_empty() {
        s.push_str("\n## Skipped\n\n");
        for line in &report.skipped {
            s.push_str(&format!("- {line}\n"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::runner::distill::{read_teacher_bin, TeacherBinWriter};

    fn tiny_bin(gold: &[u32], q_rows: &[Vec<f32>], teacher: &str, classes: &[&str]) -> Vec<u8> {
        let header = serde_json::json!({
            "suite": "t",
            "teacher": teacher,
            "checkpoint": format!("{teacher}@test"),
            "classes": classes,
            "n_rows": gold.len(),
        });
        let mut w = TeacherBinWriter::new(&header).expect("writer");
        for (g, q) in gold.iter().zip(q_rows.iter()) {
            w.push_row(*g, argmax_lowest_pos(q) as u32, q);
        }
        w.into_bytes()
    }

    #[test]
    fn writer_reader_roundtrip() {
        let classes = ["a", "b", "c"];
        let gold = [0u32, 2, 1];
        let q = vec![
            vec![0.7f32, 0.2, 0.1],
            vec![0.1, 0.2, 0.7],
            vec![0.2, 0.6, 0.2],
        ];
        let bytes = tiny_bin(&gold, &q, "x", &classes);
        let col = read_teacher_bin(&bytes).expect("read");
        assert_eq!(col.gold, gold.to_vec());
        assert_eq!(col.pick, vec![0, 2, 1]);
        assert_eq!(col.q, q);
        assert_eq!(
            col.header.get("teacher").and_then(|v| v.as_str()),
            Some("x")
        );
    }

    #[test]
    fn reader_refuses_truncation_bad_magic_and_lying_headers() {
        assert!(read_teacher_bin(b"NOPE").is_err());
        let bytes = tiny_bin(&[0], &[vec![1.0, 0.0]], "x", &["a", "b"]);
        assert!(read_teacher_bin(&bytes[..bytes.len() - 1]).is_err());
        // A header claiming n_rows=2 with one row's body -> refused.
        let hdr = serde_json::json!({
            "suite": "t", "teacher": "x", "classes": ["a", "b"], "n_rows": 2
        });
        let hdr_bytes = serde_json::to_vec(&hdr).unwrap();
        let mut lying = Vec::new();
        lying.extend_from_slice(b"RIDT");
        lying.extend_from_slice(&1u32.to_le_bytes());
        lying.extend_from_slice(&(hdr_bytes.len() as u32).to_le_bytes());
        lying.extend_from_slice(&hdr_bytes);
        lying.extend_from_slice(&0u32.to_le_bytes());
        lying.extend_from_slice(&0u32.to_le_bytes());
        lying.extend_from_slice(&1.0f32.to_le_bytes());
        lying.extend_from_slice(&0.0f32.to_le_bytes());
        assert!(read_teacher_bin(&lying).is_err());
    }

    #[test]
    fn gate_math_is_sigmoid_never_softmax() {
        // Equal accuracies -> 0.5 exactly.
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-12);
        // A 3 pp gap at T=0.01 -> sigmoid(3) ≈ 0.9526 (a near-hard pick,
        // still never exactly 1 — softmax-free by construction).
        let g = sigmoid(3.0);
        assert!((g - 0.952_574_1).abs() < 1e-6);
        assert!(g < 1.0);
        // Monotone in the gap.
        assert!(sigmoid(0.005 / ROUTER_TEMPERATURE) > 0.5);
        assert!(sigmoid(-0.005 / ROUTER_TEMPERATURE) < 0.5);
    }

    #[test]
    fn argmax_takes_lowest_position_on_ties() {
        assert_eq!(argmax_lowest_pos(&[0.3, 0.5, 0.5]), 1);
        assert_eq!(argmax_lowest_pos(&[0.5, 0.5, 0.5]), 0);
        assert_eq!(argmax_lowest_pos(&[0.1, 0.2, 0.9]), 2);
    }
}
