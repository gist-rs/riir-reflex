//! riir-train Issue 576 T3 — the Arm-B TEACHER pass: laya class
//! probabilities over the TRAIN rows of the six Arm-A suites, dumped as
//! frozen data for the distillation student (`../riir-train`
//! `instinct_specialist::load_teacher_dump` / `train_arm_b`).
//!
//! Early-exit mode (`harness --distill`), like `--e0`: no eval lane runs,
//! no test row is touched. Per suite:
//!
//! 1. the train envelope goes through the SAME builder the eval lane uses
//!    ([`super::load_rows`] → `stratified_split` → `spec.build`), so the
//!    rendered states/questions are byte-consistent with the arena's own
//!    laya lane;
//! 2. every case is forwarded through `RiirAgent::system_one` (the
//!    G5-proven lane);
//! 3. the answer's probabilities are mapped into the STUDENT-SIDE label
//!    universe (first appearance over the surviving train docs — the same
//!    law as riir-train's `label_universe` over the same survivor
//!    sequence), with a per-row gold pin: the presented key at the gold
//!    position must equal the student-side label string (Name suites:
//!    massive/banking77 — direct equality; FixedInt suites: ag_news /
//!    emotion / sst5 / xnli — presented position i ↔ int label i, the
//!    builder's ClassLabel-order invariant).
//!
//! massive_intent_en presents a 20-key PER-ROW option subset; its dump
//! carries the full-universe target vector with 0 for un-presented labels
//! (true zeros: the gold is always presented, so an un-presented label is
//! genuinely not this row's class — the one-vs-all student reads per-class
//! targets, not a distribution).
//!
//! Output: `<out>/<suite>_teacher.bin` (magic `RIDT`, little-endian) +
//! a `.blake3` sidecar. Weights/probabilities are DATA on disk, never
//! committed (`.raw/` is gitignored).

use std::path::Path;

use super::{
    case_questions, git_sha, hostname, iso8601_utc, laya_checkpoints_for, load_laya_agent,
    load_rows, percentile_us, RunOptions, SuiteSpec, SUITES,
};
use crate::harness::suites::{stratified_split, train_docs};

pub const TEACHER_MAGIC: &[u8; 4] = b"RIDT";
pub const TEACHER_VERSION: u32 = 1;

/// The T3 suite set — the six Arm-A suites (riir-train Issue 576 T2).
/// Everything else registered in [`SUITES`] is a named loud skip.
const T3_SUITES: [&str; 6] = [
    "ag_news",
    "emotion",
    "sst5",
    "massive_intent_en",
    "banking77",
    "xnli_en",
];

/// How a suite's presented option keys map into the student label universe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMap {
    /// Presented keys ARE the student label strings (massive: label_text
    /// verbatim; banking77: the mteb builder's sorted stripped label_text).
    Name,
    /// Presented position i ↔ int label i (the fixed-option suites: the
    /// builder's ClassLabel-order invariant; student labels are int
    /// strings). The per-row gold pin verifies it on real data.
    FixedInt,
}

impl KeyMap {
    /// `None` = not a T3 teacher suite (named loud skip upstream).
    #[must_use]
    pub fn for_suite(suite: &str) -> Option<Self> {
        match suite {
            "massive_intent_en" | "banking77" => Some(Self::Name),
            "ag_news" | "emotion" | "sst5" | "xnli_en" => Some(Self::FixedInt),
            _ => None,
        }
    }
}

fn t3_skip_reason(suite: &str) -> &'static str {
    match suite {
        "typed_decisions" => {
            "not a T3 teacher suite — Bench 051 measured its NB tables never arm \
             (options are state-field values, route terms never activate); \
             distillation there waits for the option-conditioned scorer \
             (riir-instinct Issue 005 E0 note)"
        }
        "prompt_injections" => {
            "not a T3 teacher suite — noul 2-class, outside Arm A's six (576 T2 scope)"
        }
        _ => "not a T3 teacher suite",
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DistillSuite {
    pub name: String,
    pub checkpoint: &'static str,
    pub n_rows: usize,
    pub n_classes: usize,
    /// Teacher argmax vs gold over the train rows — a DATA-QUALITY sanity
    /// stat for the distillation join, never a published lane number (the
    /// G5 gate governs those).
    pub teacher_accuracy: f64,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub seconds: f64,
    pub path: String,
    /// First 16 hex of the file's BLAKE3 (the sidecar carries the full one).
    pub blake3: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DistillMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub host: String,
    pub device: String,
    pub datasets_dir: String,
    pub limit: usize,
    pub row_rule: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct DistillOutput {
    pub meta: DistillMeta,
    pub suites: Vec<DistillSuite>,
    pub skipped: Vec<String>,
}

/// Argmax over presented (key, prob) pairs, ties to the LOWEST position
/// (the engine argmax law).
fn argmax_lowest_pos(probabilities: &[(String, f64)]) -> usize {
    let mut best = 0usize;
    for (i, (_, p)) in probabilities.iter().enumerate().skip(1) {
        if *p > probabilities[best].1 {
            best = i;
        }
    }
    best
}

/// Map one answer's presented probabilities into the student universe.
/// Returns (gold class, teacher-pick class, q over `classes`).
fn map_targets(
    keymap: KeyMap,
    doc_label: &str,
    gold_pos: usize,
    probabilities: &[(String, f64)],
    classes: &[String],
) -> Result<(usize, usize, Vec<f32>), String> {
    let class_of =
        |s: &str| classes.iter().position(|c| c == s).ok_or_else(|| format!("class {s:?} absent"));
    let mut q = vec![0.0f32; classes.len()];
    let pick_pos = argmax_lowest_pos(probabilities);
    match keymap {
        KeyMap::FixedInt => {
            if probabilities.len() != classes.len() {
                return Err(format!(
                    "presented {} option(s) vs {} class(es) — fixed suites present the \
                     full universe",
                    probabilities.len(),
                    classes.len()
                ));
            }
            if doc_label != gold_pos.to_string() {
                return Err(format!(
                    "gold pin failed: doc label {doc_label:?} != presented gold position \
                     {gold_pos} — builder/dataset order drift"
                ));
            }
            for (i, (_, p)) in probabilities.iter().enumerate() {
                let ci = class_of(&i.to_string())?;
                q[ci] = *p as f32;
            }
            let pick = class_of(&pick_pos.to_string())?;
            Ok((class_of(doc_label)?, pick, q))
        }
        KeyMap::Name => {
            for (k, p) in probabilities {
                let ci = class_of(k)?;
                q[ci] = *p as f32;
            }
            let gold_key = probabilities
                .get(gold_pos)
                .map(|(k, _)| k.as_str())
                .ok_or_else(|| format!("gold position {gold_pos} out of range"))?;
            if gold_key != doc_label {
                return Err(format!(
                    "gold pin failed: presented gold key {gold_key:?} != doc label \
                     {doc_label:?} — join drift"
                ));
            }
            let pick = class_of(&probabilities[pick_pos].0)?;
            Ok((class_of(doc_label)?, pick, q))
        }
    }
}

/// Distill one suite's train rows through the laya lane.
fn distill_suite(
    spec: &SuiteSpec,
    dir: &Path,
    out_dir: &Path,
    limit: usize,
    device: &str,
) -> Result<DistillSuite, String> {
    use crate::laya::config::Checkpoint;

    let t_start = std::time::Instant::now();
    let suite_dir = dir.join(spec.name);
    let train_rows = load_rows(&suite_dir, "train")?;
    // `limit` = a label-stratified cap (validation runs); 0 = the whole
    // split (identity split, the T3 protocol posture).
    let split = stratified_split(&train_rows, spec.name, limit);
    let docs = train_docs(&split.front, spec.name);
    let built = (spec.build)(&split.front, 0);
    if docs.len() != built.cases.len() {
        return Err(format!(
            "row/case join broke: {} doc(s) vs {} case(s) — survivor predicate drift",
            docs.len(),
            built.cases.len()
        ));
    }
    if docs.is_empty() {
        return Err("no train rows survived the label rule".into());
    }
    // Student-side label universe: first appearance over the surviving docs
    // — the same law as riir-train's label_universe over the same sequence,
    // so the student's join check is exact equality (order included).
    let mut classes: Vec<String> = Vec::with_capacity(16);
    for d in &docs {
        if !classes.iter().any(|c| c == &d.label) {
            classes.push(d.label.clone());
        }
    }
    let n_classes = classes.len();

    let keymap = KeyMap::for_suite(spec.name)
        .ok_or_else(|| format!("{}: no key mapping (not a T3 suite)", spec.name))?;
    let ckpt = laya_checkpoints_for(spec.name)
        .first()
        .copied()
        .ok_or_else(|| format!("{}: no laya checkpoint", spec.name))?;
    let ck = match ckpt {
        "english" => Checkpoint::English,
        "multilingual" => Checkpoint::Multilingual,
        "typed" => Checkpoint::TypedDecisions,
        other => return Err(format!("unknown checkpoint {other}")),
    };
    let agent = load_laya_agent(ckpt, ck)?;

    let header = serde_json::json!({
        "suite": spec.name,
        "checkpoint": ckpt,
        "device": device,
        "git_sha": git_sha(),
        "datasets_dir": dir.display().to_string(),
        "row_rule": "student rows = reflex train_docs over the sorted train-*.json pages; \
                     case i ↔ doc i (same survivor predicate, same order); the student \
                     re-verifies per-row gold labels against its own rows",
        "classes": classes,
        "n_rows": docs.len(),
        "limit": limit,
        "created_utc": iso8601_utc(),
    });
    let header_bytes = serde_json::to_string(&header).expect("header json").into_bytes();

    let mut out_bytes: Vec<u8> = Vec::with_capacity(1 << 20);
    out_bytes.extend_from_slice(TEACHER_MAGIC);
    out_bytes.extend_from_slice(&TEACHER_VERSION.to_le_bytes());
    out_bytes.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    out_bytes.extend_from_slice(&header_bytes);

    // One discarded warmup forward: compiles the Metal pipelines so the
    // per-row latency stat is not cold-skewed (the run_laya_checkpoint
    // pre-ramp law, in the data-pass posture).
    if let Some(case) = built.cases.first() {
        agent
            .system_one(&case.state, &case_questions(case))
            .map_err(|e| format!("laya warmup ({ckpt}): {e}"))?;
        eprintln!("  [distill {}] warmup done ({ckpt})", spec.name);
    }

    let mut durs_ms: Vec<u64> = Vec::with_capacity(docs.len());
    let mut hits = 0usize;
    for (doc, case) in docs.iter().zip(built.cases.iter()) {
        let questions = case_questions(case);
        let t0 = std::time::Instant::now();
        let answers = agent
            .system_one(&case.state, &questions)
            .map_err(|e| format!("laya forward ({}): {e}", case.id))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        let a = answers
            .first()
            .ok_or_else(|| format!("{}: no answer for its single question", case.id))?;
        let (gold_c, pick_c, q) = map_targets(
            keymap,
            &doc.label,
            case.gold[0].idx,
            &a.probabilities,
            &classes,
        )
        .map_err(|e| format!("{}: {e}", case.id))?;
        hits += usize::from(pick_c == gold_c);
        out_bytes.extend_from_slice(&(gold_c as u32).to_le_bytes());
        out_bytes.extend_from_slice(&(pick_c as u32).to_le_bytes());
        for v in &q {
            out_bytes.extend_from_slice(&v.to_le_bytes());
        }
        if durs_ms.len().is_multiple_of(2000) {
            eprintln!(
                "  [distill {}] {}/{} rows · teacher acc so far {:.3}",
                spec.name,
                durs_ms.len(),
                docs.len(),
                hits as f64 / durs_ms.len() as f64
            );
        }
    }

    let (p50, p99, _support) = percentile_us(&durs_ms);
    let teacher_accuracy = hits as f64 / docs.len() as f64;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
    let path = out_dir.join(format!("{}_teacher.bin", spec.name));
    std::fs::write(&path, &out_bytes)
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    let seal = blake3::hash(&out_bytes);
    let sidecar = out_dir.join(format!("{}_teacher.bin.blake3", spec.name));
    std::fs::write(&sidecar, format!("{}\n", seal.to_hex()))
        .map_err(|e| format!("write {}: {e}", sidecar.display()))?;

    eprintln!(
        "  [distill {}] done: {} rows · {} classes · teacher acc {:.4} · p50 {} ms · {:.1}s",
        spec.name,
        docs.len(),
        n_classes,
        teacher_accuracy,
        p50,
        t_start.elapsed().as_secs_f64()
    );
    Ok(DistillSuite {
        name: spec.name.to_string(),
        checkpoint: ckpt,
        n_rows: docs.len(),
        n_classes,
        teacher_accuracy,
        latency_p50_ms: p50 as f64,
        latency_p99_ms: p99 as f64,
        seconds: t_start.elapsed().as_secs_f64(),
        path: path.display().to_string(),
        blake3: seal.to_hex()[..16].to_string(),
    })
}

/// Run the teacher pass over the requested suites (empty = all six T3
/// suites). Early-exit: no eval lane, no test row. A suite that fails is a
/// named loud skip — the mode still exits Ok for the suites that ran (the
/// caller prints the skips; the record decides whether they matter).
pub fn run_distill(
    opts: &RunOptions,
    out_dir: &Path,
    limit: usize,
) -> Result<DistillOutput, String> {
    let device = std::env::var("LAYA_DEVICE").unwrap_or_else(|_| "default".into());
    let mut suites = Vec::new();
    let mut skipped = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
            continue;
        }
        if spec.synthetic.is_some() {
            skipped.push(format!("{}: synthetic family — no dataset rows", spec.name));
            continue;
        }
        if !T3_SUITES.contains(&spec.name) {
            skipped.push(format!("{}: {}", spec.name, t3_skip_reason(spec.name)));
            continue;
        }
        match distill_suite(spec, &opts.datasets_dir, out_dir, limit, &device) {
            Ok(s) => suites.push(s),
            Err(e) => skipped.push(format!("{}: {e}", spec.name)),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "distill: no suite ran ({} skip/failure line(s), first: {})",
            skipped.len(),
            skipped.first().map_or("none", String::as_str)
        ));
    }
    Ok(DistillOutput {
        meta: DistillMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            host: hostname(),
            device,
            datasets_dir: opts.datasets_dir.display().to_string(),
            limit,
            row_rule: "train rows only (the no-cheat law, riir-reflex Issue 038); the test \
                       split stays reserved for the arena's single read",
        },
        suites,
        skipped,
    })
}

/// The markdown rendering (the record's table block).
#[must_use]
pub fn render_distill_markdown(out: &DistillOutput) -> String {
    let mut s = String::new();
    s.push_str("# Distill teacher pass — laya probabilities over train rows (riir-train Issue 576 T3)\n\n");
    s.push_str(&format!(
        "- date {} · sha {} · host {} · device {}\n- datasets {} · limit {}\n- rows: {}\n",
        out.meta.date_utc,
        out.meta.git_sha,
        out.meta.host,
        out.meta.device,
        out.meta.datasets_dir,
        out.meta.limit,
        out.meta.row_rule
    ));
    s.push_str("\n| suite | ckpt | rows | classes | teacher acc | p50 ms | p99 ms | s | blake3 |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for v in &out.suites {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {:.4} | {:.1} | {:.1} | {:.1} | {} |\n",
            v.name,
            v.checkpoint,
            v.n_rows,
            v.n_classes,
            v.teacher_accuracy,
            v.latency_p50_ms,
            v.latency_p99_ms,
            v.seconds,
            v.blake3
        ));
    }
    if !out.skipped.is_empty() {
        s.push_str("\n## Skipped / failed (loud absences)\n\n");
        for e in &out.skipped {
            s.push_str(&format!("- {e}\n"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probs(pairs: &[(&str, f64)]) -> Vec<(String, f64)> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    #[test]
    fn keymap_dispatches_the_t3_suites() {
        assert_eq!(KeyMap::for_suite("massive_intent_en"), Some(KeyMap::Name));
        assert_eq!(KeyMap::for_suite("banking77"), Some(KeyMap::Name));
        for s in ["ag_news", "emotion", "sst5", "xnli_en"] {
            assert_eq!(KeyMap::for_suite(s), Some(KeyMap::FixedInt), "{s}");
        }
        assert_eq!(KeyMap::for_suite("typed_decisions"), None);
        assert_eq!(KeyMap::for_suite("prompt_injections"), None);
        assert_eq!(T3_SUITES.len(), 6);
    }

    #[test]
    fn fixed_int_maps_by_position_into_first_appearance_order() {
        // Student universe arrived at label "1" first: classes = [1, 0, 2].
        let classes = vec!["1".to_string(), "0".to_string(), "2".to_string()];
        let p = probs(&[("a", 0.1), ("b", 0.6), ("c", 0.3)]);
        let (gold, pick, q) =
            map_targets(KeyMap::FixedInt, "2", 2, &p, &classes).expect("map");
        assert_eq!(gold, 2, "doc label \"2\" is classes[2]");
        assert_eq!(pick, 0, "argmax at position 1 → int label \"1\" = classes[0]");
        // position 0 (p .1) → class "0" → slot 1; position 1 (.6) → "1" → 0; 2 (.3) → "2" → 2.
        assert_eq!(q, vec![0.6, 0.1, 0.3]);
    }

    #[test]
    fn name_maps_by_key_and_pins_gold() {
        let classes: Vec<String> = ["arrange_flight", "check_weather", "send_email"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let p = probs(&[("check_weather", 0.2), ("send_email", 0.7), ("arrange_flight", 0.1)]);
        let (gold, pick, q) =
            map_targets(KeyMap::Name, "send_email", 1, &p, &classes).expect("map");
        assert_eq!(gold, 2);
        assert_eq!(pick, 2);
        assert_eq!(q, vec![0.1, 0.2, 0.7]);
    }

    #[test]
    fn gold_pin_refuses_drift_in_both_directions() {
        let classes: Vec<String> = ["0", "1", "2"].iter().map(|s| (*s).to_string()).collect();
        let p = probs(&[("a", 0.5), ("b", 0.3), ("c", 0.2)]);
        // FixedInt: doc label disagrees with the presented gold position.
        assert!(map_targets(KeyMap::FixedInt, "1", 2, &p, &classes).is_err());
        // Name: the presented key at gold is not the doc label.
        let pn = probs(&[("x", 0.5), ("y", 0.5)]);
        let cn: Vec<String> = vec!["x".to_string(), "y".to_string()];
        assert!(map_targets(KeyMap::Name, "y", 0, &pn, &cn).is_err());
        // FixedInt width mismatch (presented ≠ classes).
        let p2 = probs(&[("a", 1.0)]);
        assert!(map_targets(KeyMap::FixedInt, "0", 0, &p2, &classes).is_err());
    }

    #[test]
    fn argmax_ties_to_the_lowest_position() {
        let p = probs(&[("a", 0.4), ("b", 0.4), ("c", 0.2)]);
        assert_eq!(argmax_lowest_pos(&p), 0);
        let p2 = probs(&[("a", 0.1), ("b", 0.4), ("c", 0.4)]);
        assert_eq!(argmax_lowest_pos(&p2), 1);
    }
}
