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
//!    predicate, same order) and the classes array (exact equality under
//!    the bins' own `classes_spelling` maps — the 623 StudentUniverse
//!    canonicalization, Bench 160 finding (a): post-fix bins spell classes
//!    in the student universe, pre-fix bins in the corpus spelling; both
//!    sides are brought to the CANONICAL sorted-corpus space — names
//!    inverted through the map, gold/pick/q permuted when the source
//!    carried the student order), and that shared space is what every
//!    routed artifact emits;
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
//! 7. the 640-T4 CROSS-VALIDATION gate (verdict round 1): the gate g is
//!    computed from the SAME rows the routed accuracy is measured on —
//!    in-sample by construction, and the optimism grows with every added
//!    column (the mammoth/elephant extension's exact hazard). Every blend
//!    suite therefore also carries a leave-one-row-out routed accuracy
//!    (`routed_accuracy_cv`): each row's pick uses a gate computed with that
//!    row dropped from BOTH accuracy sums — exact, deterministic, the pick
//!    never sees its own row. The emitted artifact keeps the full-data gate
//!    (the deployable form); the CV column is the honest estimate beside it.
//!
//! Honest-read caveats carried in the manifest: every accuracy here is a
//! TRAIN-row number (the captures' no-cheat law) — the routed artifact's
//! verdict is T4's single frozen read; nothing here selects on it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use katgpt_core::exact_sigmoid_f64;

use super::distill::{TeacherBinWriter, read_teacher_bin};
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
    /// The 640-T4 cross-validation read (verdict round 1): leave-one-row-out
    /// routed accuracy — each row scored under a gate that never saw it.
    /// `None` = passthrough, or a sub-2-row suite (no held-out fold exists).
    pub routed_accuracy_cv: Option<f64>,
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

/// Substrate delegation (the Issue-014 engine.rs precedent, f64 twin; found
/// by the 2026-10-15 substrate-first drift audit, katgpt-rs `bbc28a091`):
/// the two-branch libm reference in katgpt-core replaces the local
/// single-branch body. Bit-identical for gate args >= 0 (identical
/// expression); bounded ULPs below (algebraically identical forms, different
/// fp roundings — pinned by the frozen-legacy test). The routed artifact of
/// record (bench 144) was produced with the legacy body; this is an offline
/// instrument under the frozen-read law, so no live gate pins those bits
/// (the det gate is same-binary by construction).
fn sigmoid(x: f64) -> f64 {
    exact_sigmoid_f64(x)
}

/// The 640-T4 cross-validation gate (verdict round 1): the in-sample oracle
/// reads optimistic because the gate g is fitted on the rows it is scored
/// on. Leave-one-row-out: row r's pick uses `g_r` computed with row r
/// dropped from BOTH accuracy sums — `((hits_a−hit_r) − (hits_b−hit_r)) /
/// (n−1)`, O(1) per row, no RNG, bit-reproducible. `None` below 2 rows (no
/// held-out fold exists).
fn routed_accuracy_loo_cv(
    col_a: &super::distill::TeacherColumn,
    col_b: &super::distill::TeacherColumn,
    hits_a: usize,
    hits_b: usize,
) -> Option<f64> {
    let n = col_a.gold.len();
    if n < 2 {
        return None;
    }
    let denom = (n - 1) as f64 * ROUTER_TEMPERATURE;
    let mut cv_hits = 0usize;
    for r in 0..n {
        let hit_a = usize::from(col_a.pick[r] == col_a.gold[r]);
        let hit_b = usize::from(col_b.pick[r] == col_b.gold[r]);
        let gap = (hits_a - hit_a) as f64 - (hits_b - hit_b) as f64;
        let g = sigmoid(gap / denom);
        let q_route: Vec<f32> = col_a.q[r]
            .iter()
            .zip(col_b.q[r].iter())
            .map(|(&a, &b)| (g * f64::from(a) + (1.0 - g) * f64::from(b)) as f32)
            .collect();
        cv_hits += usize::from(argmax_lowest_pos(&q_route) == col_a.gold[r] as usize);
    }
    Some(cv_hits as f64 / n as f64)
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

fn load_source(
    dir: &Path,
    suite: &str,
    path: &Path,
    seal: String,
) -> Result<(RouterSource, super::distill::TeacherColumn), String> {
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

/// The 623 StudentUniverse canonicalizer (Bench 160 finding (a)): a post-fix
/// capture carries `classes_spelling: {law, map: corpus→student}` with
/// STUDENT-spelled `classes`; a pre-fix capture carries corpus-spelled
/// `classes` with the field absent (or the identity string). Returns the
/// CANONICAL (corpus) spelling: the map is inverted, never the data — the
/// row indices are shared across the eras, only the names differ. A
/// malformed map (non-injective, or missing a class) is a loud refusal, a
/// suite skip — never a silent pass.
fn canonical_classes(header: &serde_json::Value, label: &str) -> Result<Vec<String>, String> {
    let classes: Vec<String> = header
        .get("classes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{label}: header has no classes array"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{label}: non-string class entry"))
        })
        .collect::<Result<_, _>>()?;
    match header.get("classes_spelling") {
        None | Some(serde_json::Value::String(_)) => Ok(classes),
        Some(serde_json::Value::Object(obj)) => {
            let map = obj
                .get("map")
                .and_then(|v| v.as_object())
                .ok_or_else(|| format!("{label}: classes_spelling object without a map"))?;
            // Invert corpus→student into student→corpus; injectivity is the
            // law (the bench-160 gold-verified maps are bijective).
            let mut inv: BTreeMap<&str, &str> = BTreeMap::new();
            for (corpus, student) in map {
                let s = student.as_str().ok_or_else(|| {
                    format!("{label}: non-string classes_spelling value for {corpus}")
                })?;
                if inv.insert(s, corpus.as_str()).is_some() {
                    return Err(format!(
                        "{label}: classes_spelling map not injective (duplicate student spelling {s})"
                    ));
                }
            }
            classes
                .iter()
                .map(|c| {
                    inv.get(c.as_str()).copied().map(str::to_string).ok_or_else(|| {
                        format!(
                            "{label}: class {c} missing from the classes_spelling map — refusing the join"
                        )
                    })
                })
                .collect()
        }
        Some(_) => Err(format!("{label}: classes_spelling of unsupported shape")),
    }
}

/// Bring a loaded column into the CANONICAL index space: canonical class
/// names (the inverted `classes_spelling` map where present) in SORTED
/// order — the distill corpus law (`classes.sort()`), which is exactly the
/// native space of every pre-623 capture. When the source order differs
/// (the StudentUniverse permutation), gold/pick/q are permuted into the
/// canonical space; a source already in canonical order passes through
/// bit-identical. Returns (canonical names, canonical-space column).
fn canonicalize_column(
    col: super::distill::TeacherColumn,
    label: &str,
) -> Result<(Vec<String>, super::distill::TeacherColumn), String> {
    let canon = canonical_classes(&col.header, label)?;
    let mut sorted: Vec<String> = canon.clone();
    sorted.sort();
    if sorted == canon {
        return Ok((canon, col));
    }
    // src_of[i] = the source index carrying canonical class i;
    // canon_of[s] = the canonical index of source class s.
    let src_of: Vec<usize> = sorted
        .iter()
        .map(|name| {
            canon
                .iter()
                .position(|c| c == name)
                .ok_or_else(|| format!("{label}: canonical class {name} vanished during ordering"))
        })
        .collect::<Result<_, _>>()?;
    let mut canon_of = vec![0usize; canon.len()];
    for (ci, &si) in src_of.iter().enumerate() {
        canon_of[si] = ci;
    }
    let q = col
        .q
        .iter()
        .map(|row| src_of.iter().map(|&s| row[s]).collect::<Vec<f32>>())
        .collect();
    let gold = col
        .gold
        .iter()
        .map(|&g| canon_of[g as usize] as u32)
        .collect();
    let pick = col
        .pick
        .iter()
        .map(|&p| canon_of[p as usize] as u32)
        .collect();
    Ok((
        sorted,
        super::distill::TeacherColumn {
            header: col.header,
            gold,
            pick,
            q,
        },
    ))
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
                routed_accuracy_cv: None,
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

        // JOIN VERIFICATION — both columns brought to the canonical (sorted
        // corpus) index space under each bin's own `classes_spelling` map
        // (the 623 canonicalization: names inverted, gold/pick/q permuted;
        // a source already canonical passes through untouched) + per-row
        // gold pins in that shared space.
        let (classes_a, col_a) =
            match canonicalize_column(col_a, &format!("{suite} @ {}", path_a.display())) {
                Ok(x) => x,
                Err(e) => {
                    skipped.push(format!("{suite}: {e}"));
                    continue;
                }
            };
        let (classes_b, col_b) =
            match canonicalize_column(col_b, &format!("{suite} @ {}", path_b.display())) {
                Ok(x) => x,
                Err(e) => {
                    skipped.push(format!("{suite}: {e}"));
                    continue;
                }
            };
        if classes_a != classes_b {
            skipped.push(format!(
                "{suite}: classes universes differ across captures (canonical spelling) — \
                 refusing the join"
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
        if col_a
            .gold
            .iter()
            .zip(col_b.gold.iter())
            .any(|(g1, g2)| g1 != g2)
        {
            skipped.push(format!(
                "{suite}: per-row gold pins disagree — the captures are not row-aligned; \
                 refusing the join"
            ));
            continue;
        }

        // Gate + blend.
        let gate = sigmoid((src_a.accuracy - src_b.accuracy) / ROUTER_TEMPERATURE);
        let n_classes = col_a.q.first().map_or(0, Vec::len);
        let hits = |col: &super::distill::TeacherColumn| {
            col.pick
                .iter()
                .zip(col.gold.iter())
                .filter(|(p, g)| p == g)
                .count()
        };
        let routed_accuracy_cv = routed_accuracy_loo_cv(&col_a, &col_b, hits(&col_a), hits(&col_b));
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
                "classes_canonical": "the emitted `classes` are the CANONICAL sorted corpus \
                                     spelling (the 623 StudentUniverse maps applied at join \
                                     time; gold/pick/q permuted into that space when the \
                                     source carried the student order)",
                "classes_spelling_sources": {
                    "a": col_a.header.get("classes_spelling").cloned().unwrap_or(serde_json::Value::Null),
                    "b": col_b.header.get("classes_spelling").cloned().unwrap_or(serde_json::Value::Null),
                },
                "routed_accuracy_cv": routed_accuracy_cv,
                "cv_law": "leave-one-row-out (the 640-T4 gate): each row scored under a gate \
                           that never saw it; the artifact keeps the full-data gate",
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
            routed_accuracy_cv,
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
    s.push_str(
        "\n| suite | sources (acc) | gate g | routed acc | cv (loo) | mode | classes | routed blake3 |\n",
    );
    s.push_str("|---|---|---|---|---|---|---|---|\n");
    for v in &report.suites {
        let srcs = v
            .sources
            .iter()
            .map(|src| format!("{} {:.4}", src.teacher, src.accuracy))
            .collect::<Vec<_>>()
            .join(" ⊕ ");
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            v.name,
            srcs,
            v.gate
                .map(|g| format!("{g:.4}"))
                .unwrap_or_else(|| "—".into()),
            v.routed_accuracy
                .map(|a| format!("{a:.4}"))
                .unwrap_or_else(|| "—".into()),
            v.routed_accuracy_cv
                .map(|a| format!("{a:.4}"))
                .unwrap_or_else(|| "—".into()),
            if v.gate.is_some() {
                "blend"
            } else {
                "passthrough"
            },
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
    use crate::harness::runner::distill::{TeacherBinWriter, read_teacher_bin};

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

    /// Writes a raw bin under `<dir>/<suite>_teacher.bin` (run_teacher_router
    /// discovers by filename, so the fixture must hit the disk).
    fn write_bin(dir: &Path, suite: &str, bytes: &[u8]) {
        std::fs::create_dir_all(dir).expect("mkdir");
        std::fs::write(dir.join(format!("{suite}_teacher.bin")), bytes).expect("write bin");
    }

    fn spelling_header(classes: &[&str], map: Option<serde_json::Value>) -> serde_json::Value {
        let mut h = serde_json::json!({
            "suite": "emo",
            "teacher": "t",
            "checkpoint": "t@test",
            "classes": classes,
            "n_rows": 2,
        });
        if let Some(m) = map {
            h["classes_spelling"] = serde_json::json!({ "law": "test", "map": m });
        }
        h
    }

    fn write_headered_bin(
        dir: &Path,
        suite: &str,
        header: serde_json::Value,
        gold: &[u32],
        q_rows: &[Vec<f32>],
    ) {
        let mut w = TeacherBinWriter::new(&header).expect("writer");
        for (g, q) in gold.iter().zip(q_rows.iter()) {
            w.push_row(*g, argmax_lowest_pos(q) as u32, q);
        }
        write_bin(dir, suite, &w.into_bytes());
    }

    #[test]
    fn classes_spelling_remap_joins_and_emits_canonical() {
        let root =
            std::env::temp_dir().join(format!("reflex_router_spell_ok_{}", std::process::id()));
        let dir_a = root.join("a");
        let dir_b = root.join("b");
        let out = root.join("out");
        // Corpus-spelled pre-623 column: no classes_spelling field (corpus
        // names arrive SORTED — the distill corpus law).
        write_headered_bin(
            &dir_a,
            "emo",
            spelling_header(&["anger", "joy"], None),
            &[0, 1],
            &[vec![0.9, 0.1], vec![0.3, 0.7]],
        );
        // Student-spelled post-fix column: classes ["0","1"] + the injective map.
        write_headered_bin(
            &dir_b,
            "emo",
            spelling_header(
                &["0", "1"],
                Some(serde_json::json!({ "anger": "0", "joy": "1" })),
            ),
            &[0, 1],
            &[vec![0.8, 0.2], vec![0.4, 0.6]],
        );
        let report = run_teacher_router(&[dir_a.clone(), dir_b.clone()], &out).expect("router run");
        assert!(
            report.skipped.is_empty(),
            "unexpected skips: {:?}",
            report.skipped
        );
        assert_eq!(report.suites.len(), 1);
        assert!(
            report.suites[0].gate.is_some(),
            "remapped join must route, not refuse"
        );
        // The routed artifact emits the CANONICAL (corpus) spelling + the
        // per-source provenance maps.
        let routed = std::fs::read(out.join("emo_teacher.bin")).expect("routed bin");
        let col = read_teacher_bin(&routed).expect("read routed");
        assert_eq!(
            col.header
                .get("classes")
                .and_then(|v| v.as_array())
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["anger", "joy"]
        );
        let spell = col
            .header
            .pointer("/router/classes_spelling_sources")
            .expect("spelling provenance");
        assert!(
            spell.get("a").unwrap().is_null(),
            "pre-623 side records null"
        );
        assert!(spell.get("b").unwrap().get("map").is_some());
        assert_eq!(col.gold, vec![0, 1]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn classes_spelling_malformed_map_refuses_loud() {
        let root =
            std::env::temp_dir().join(format!("reflex_router_spell_bad_{}", std::process::id()));
        let dir_a = root.join("a");
        let dir_b = root.join("b");
        let out = root.join("out");
        write_headered_bin(
            &dir_a,
            "emo",
            spelling_header(&["sad", "joy"], None),
            &[0, 1],
            &[vec![0.9, 0.1], vec![0.3, 0.7]],
        );
        // The map is missing "joy" — the student class "1" has no corpus name,
        // so the join cannot be certified; refuse the suite, never guess.
        write_headered_bin(
            &dir_b,
            "emo",
            spelling_header(&["0", "1"], Some(serde_json::json!({ "sad": "0" }))),
            &[0, 1],
            &[vec![0.8, 0.2], vec![0.4, 0.6]],
        );
        let report = run_teacher_router(&[dir_a, dir_b], &out).expect("router run");
        assert!(
            report.suites.is_empty(),
            "a lying map must refuse the suite"
        );
        assert_eq!(report.skipped.len(), 1);
        assert!(report.skipped[0].contains("missing from the classes_spelling map"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn classes_spelling_student_order_is_permuted_into_canonical_space() {
        // The xnli shape: corpus (sorted) = [contradiction, entailment,
        // neutral]; student = ["0","1","2"] with entailment=0, neutral=1,
        // contradiction=2 — a real permutation, so the DATA must move, not
        // just the names.
        let root =
            std::env::temp_dir().join(format!("reflex_router_spell_perm_{}", std::process::id()));
        let dir_a = root.join("a");
        let dir_b = root.join("b");
        let out = root.join("out");
        // Pre-623 corpus space (already canonical): row0 gold=entailment(1),
        // row1 gold=contradiction(0).
        write_headered_bin(
            &dir_a,
            "x",
            spelling_header(&["contradiction", "entailment", "neutral"], None),
            &[1, 0],
            &[vec![0.1, 0.8, 0.1], vec![0.7, 0.2, 0.1]],
        );
        // Post-fix student space: row0 gold=0 (entailment), row1 gold=2
        // (contradiction); q strong on the same semantic classes.
        write_headered_bin(
            &dir_b,
            "x",
            spelling_header(
                &["0", "1", "2"],
                Some(
                    serde_json::json!({ "entailment": "0", "neutral": "1", "contradiction": "2" }),
                ),
            ),
            &[0, 2],
            &[vec![0.6, 0.2, 0.2], vec![0.2, 0.2, 0.6]],
        );
        let report = run_teacher_router(&[dir_a, dir_b], &out).expect("router run");
        assert!(
            report.skipped.is_empty(),
            "unexpected skips: {:?}",
            report.skipped
        );
        assert_eq!(report.suites.len(), 1);
        let s = &report.suites[0];
        assert!(s.gate.is_some());
        // Both columns are perfect after permutation -> routed acc 1.0.
        assert_eq!(s.routed_accuracy, Some(1.0));
        let routed = std::fs::read(out.join("x_teacher.bin")).expect("routed bin");
        let col = read_teacher_bin(&routed).expect("read routed");
        // Emitted golds are CANONICAL-space: entailment=1, contradiction=0.
        assert_eq!(col.gold, vec![1, 0]);
        // Emitted classes are the sorted corpus names.
        assert_eq!(
            col.header
                .get("classes")
                .and_then(|v| v.as_array())
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["contradiction", "entailment", "neutral"]
        );
        // Row 0's routed q sits in canonical space: strong at entailment(1),
        // the 0.5-blend of a=[0.1,0.8,0.1] and perm(b)=[0.2,0.6,0.2].
        let q0 = &col.q[0];
        assert!((q0[1] - 0.7).abs() < 1e-6, "q0 = {q0:?}");
        assert!(
            (q0[0] - 0.15).abs() < 1e-6 && (q0[2] - 0.15).abs() < 1e-6,
            "q0 = {q0:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
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

    fn hits_of(col: &crate::harness::runner::distill::TeacherColumn) -> usize {
        col.pick
            .iter()
            .zip(col.gold.iter())
            .filter(|(p, g)| p == g)
            .count()
    }

    #[test]
    fn loo_cv_on_a_complement_fixture_reads_zero_where_insample_reads_half() {
        // The 640-T4 optimism class, at the smallest shape that shows it:
        // a wins row 0 and loses row 1, b the exact complement. The in-sample
        // gate is 0.5, the blends tie, and the lowest-position tie law hands
        // row 0's hit to the routed column — 0.5 read on rows the honest
        // estimate cannot keep. LOO routes each row AWAY from the teacher
        // that got it (the gate's own success on a row makes that row look
        // better for the OTHER column once the row is dropped), so the CV
        // read is 0.0 — the estimator is honest in both directions.
        let gold = [0u32, 1];
        let qa = [vec![1.0f32, 0.0], vec![1.0, 0.0]];
        let qb = [vec![0.0f32, 1.0], vec![0.0, 1.0]];
        let col_a = read_teacher_bin(&tiny_bin(&gold, &qa, "a", &["x", "y"])).expect("a");
        let col_b = read_teacher_bin(&tiny_bin(&gold, &qb, "b", &["x", "y"])).expect("b");
        assert_eq!((hits_of(&col_a), hits_of(&col_b)), (1, 1));
        // In-sample: g = sigmoid(0) = 0.5; both blends tie; lowest position
        // wins row 0 (gold) and loses row 1.
        let g = sigmoid(0.0);
        let in_sample: Vec<usize> = (0..2)
            .map(|r| {
                let q: Vec<f32> = col_a.q[r]
                    .iter()
                    .zip(col_b.q[r].iter())
                    .map(|(&a, &b)| (g * f64::from(a) + (1.0 - g) * f64::from(b)) as f32)
                    .collect();
                argmax_lowest_pos(&q)
            })
            .collect();
        assert_eq!(in_sample, vec![0, 0]);
        assert_eq!(
            in_sample
                .iter()
                .zip(gold.iter())
                .filter(|(p, g)| **p == **g as usize)
                .count(),
            1
        );
        // The honest read: 0.0.
        assert_eq!(
            routed_accuracy_loo_cv(&col_a, &col_b, hits_of(&col_a), hits_of(&col_b)),
            Some(0.0)
        );
    }

    #[test]
    fn loo_cv_equals_insample_when_one_column_dominates() {
        // No selection optimism where there is no selection: a dominates
        // every row, so every LOO gate still routes to a and the CV read
        // equals the in-sample read exactly.
        let gold = [0u32, 1, 0];
        let qa = [vec![1.0f32, 0.0], vec![0.0, 1.0], vec![1.0, 0.0]];
        let qb = [vec![0.0f32, 1.0], vec![1.0, 0.0], vec![0.0, 1.0]];
        let col_a = read_teacher_bin(&tiny_bin(&gold, &qa, "a", &["x", "y"])).expect("a");
        let col_b = read_teacher_bin(&tiny_bin(&gold, &qb, "b", &["x", "y"])).expect("b");
        assert_eq!(hits_of(&col_a), 3);
        assert_eq!(hits_of(&col_b), 0);
        assert_eq!(
            routed_accuracy_loo_cv(&col_a, &col_b, hits_of(&col_a), hits_of(&col_b)),
            Some(1.0)
        );
    }

    #[test]
    fn loo_cv_needs_two_rows() {
        let gold = [0u32];
        let qa = [vec![1.0f32, 0.0]];
        let qb = [vec![0.0f32, 1.0]];
        let col_a = read_teacher_bin(&tiny_bin(&gold, &qa, "a", &["x", "y"])).expect("a");
        let col_b = read_teacher_bin(&tiny_bin(&gold, &qb, "b", &["x", "y"])).expect("b");
        assert_eq!(routed_accuracy_loo_cv(&col_a, &col_b, 1, 0), None);
    }

    #[test]
    fn sigmoid_delegation_matches_frozen_legacy_body() {
        // Issue-014 / katgpt-rs Issue-870 precedent: router.rs carried a local
        // single-branch f64 sigmoid (`1.0 / (1.0 + (-x).exp())`). The
        // delegation to `exact_sigmoid_f64` must not move gate outputs on the
        // reachable domain — the gate arg `(acc_a - acc_b) / T` with
        // accuracies in [0, 1] and T = 0.01 implies |x| <= 100. This sweep
        // covers [-120, 120] (domain plus margin; all finite-exp territory
        // for both forms in f64). The legacy body is frozen HERE so any
        // future substrate form drift reds this pin.
        let legacy = |x: f64| 1.0 / (1.0 + (-x).exp());
        let mut max_ulps = 0i64;
        let mut max_ulps_at = 0.0f64;
        for i in 0..=24_000i32 {
            let x = -120.0f64 + (i as f64) * 0.01;
            let got = exact_sigmoid_f64(x);
            let want = legacy(x);
            if x >= 0.0 {
                assert_eq!(got.to_bits(), want.to_bits(), "x={x} must be bit-identical");
            } else {
                let ulps = (got.to_bits() as i64 - want.to_bits() as i64).abs();
                if ulps > max_ulps {
                    max_ulps = ulps;
                    max_ulps_at = x;
                }
            }
        }
        // Edge anchors: the exact reachable bounds plus the documented
        // half-gap logits from `gate_math_is_sigmoid_never_softmax`.
        for &x in &[
            -100.0f64,
            -0.005 / ROUTER_TEMPERATURE,
            0.0,
            0.005 / ROUTER_TEMPERATURE,
            100.0,
        ] {
            let got = exact_sigmoid_f64(x);
            let want = legacy(x);
            if x >= 0.0 {
                assert_eq!(got.to_bits(), want.to_bits(), "x={x} must be bit-identical");
            } else {
                let ulps = (got.to_bits() as i64 - want.to_bits() as i64).abs();
                if ulps > max_ulps {
                    max_ulps = ulps;
                    max_ulps_at = x;
                }
            }
        }
        assert!(
            max_ulps <= 3,
            "delegation drifted {max_ulps} ULPs from the frozen legacy body at x={max_ulps_at}"
        );
    }
}
