//! riir-train Issue 576 T3 — the Arm-B TEACHER pass, generalized by
//! riir-train Plan 426 T1: class probabilities over the TRAIN rows of the
//! six Arm-A suites from a SELECTED teacher (`--distill-teacher`, default
//! `laya`), dumped as frozen data for the distillation student
//! (`../riir-train` `instinct_specialist::load_teacher_dump` /
//! `train_arm_b`).
//!
//! Teachers:
//!
//! * `laya` (default) — the G5-proven `RiirAgent::system_one` lane
//!   (feature `laya-riir`); needs the checkpoints locally.
//! * `openthai` — the OpenThai comparison lane (`src/lanes/openthai`, the
//!   Bench 074 wire; their `openthai_systemone` service over loopback
//!   HTTP). No feature needed; the loud refusal without their server is
//!   the agentjev law. The strongest measured massive teacher (0.9200,
//!   Bench 074/075) — Plan 426's V2-qualified distill source.
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
//!    universe (the distinct surviving-doc labels in SORTED order — the
//!    same law as riir-train's `label_universe`), with a per-row gold pin:
//!    the presented key at the gold position must equal the student-side
//!    label string (Name suites: massive/banking77 — direct equality;
//!    FixedInt suites: ag_news / emotion / sst5 / xnli — presented
//!    position i ↔ int label i, the builder's ClassLabel-order invariant).
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

#[cfg(feature = "laya-riir")]
use super::case_questions;
use super::{git_sha, iso8601_utc, load_rows, percentile_us, RunOptions, SuiteSpec, SUITES};
use crate::harness::runner::hostname_refusing_unknown;
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

/// The selected distill teacher (Plan 426 T1's seam): everything the
/// per-row loop needs, constructed once per run. The forward returns
/// LABEL-KEYED probabilities — exactly what [`map_targets`] consumes —
/// plus the row's wall milliseconds. `pub(crate)`: the ensemble-gate
/// lane (Plan 426 T2) runs BOTH teachers over one shared case set
/// through this same seam — the fusion can never see teachers answered
/// by different code paths.
pub(crate) enum TeacherForward {
    /// The G5-proven local lane (feature `laya-riir`). The keyed
    /// probabilities come straight off the answer. Boxed: RiirAgent is
    /// orders of magnitude larger than the openthai variant's lane handle
    /// (the large_enum_variant law).
    #[cfg(feature = "laya-riir")]
    Laya {
        agent: Box<crate::laya::riir::RiirAgent>,
        provenance: &'static str,
    },
    /// The OpenThai loopback service (the Bench 074 wire, ungated). The
    /// keyed probabilities are built from their positional answer over
    /// the question's presented keys.
    Openthai { lane: crate::lanes::openthai::OpenThaiLane, provenance: String },
}

impl TeacherForward {
    /// The teacher's name — the header/record provenance token.
    pub(crate) fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { .. } => "laya",
            Self::Openthai { .. } => "openthai",
        }
    }

    /// Checkpoint (laya) or model id (openthai) — the record's
    /// `checkpoint` field, the teacher-side provenance.
    pub(crate) fn provenance(&self) -> &str {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { provenance, .. } => provenance,
            Self::Openthai { provenance, .. } => provenance,
        }
    }

    /// One discarded forward before the loop, so the latency stat is not
    /// cold-skewed (laya's pipeline-compile pre-ramp; openthai's server
    /// cold path) — and, for openthai, the final proof their server is
    /// actually answering plus the model-id provenance capture (the
    /// agentjev law: refuse loud, never half-run).
    pub(crate) fn warmup(&mut self, case: &super::SuiteCase) -> Result<(), String> {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { agent, .. } => {
                agent
                    .system_one(&case.state, &case_questions(case))
                    .map(|_| ())
                    .map_err(|e| format!("laya warmup: {e}"))
            }
            Self::Openthai { lane, provenance } => {
                lane.health()?;
                let raw = lane
                    .decide_raw(case)
                    .map_err(|e| format!("openthai warmup: {e}"))?;
                let parsed: serde_json::Value = serde_json::from_str(&raw)
                    .map_err(|e| format!("openthai warmup parse: {e}"))?;
                *provenance = format!("openthai:{}", crate::lanes::openthai::OpenThaiLane::model_of(&parsed));
                Ok(())
            }
        }
    }

    /// One row's keyed probabilities + wall ms. The FIRST question's
    /// answer (the distill join is one gold per row — the single-question
    /// contract the T3 suites speak).
    pub(crate) fn forward(&self, case: &super::SuiteCase) -> Result<(Vec<(String, f64)>, u64), String> {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { agent, .. } => {
                let questions = case_questions(case);
                let t0 = std::time::Instant::now();
                let answers = agent
                    .system_one(&case.state, &questions)
                    .map_err(|e| format!("laya forward ({}): {e}", case.id))?;
                let ms = t0.elapsed().as_millis() as u64;
                let a = answers
                    .first()
                    .ok_or_else(|| format!("{}: no answer for its single question", case.id))?;
                Ok((a.probabilities.clone(), ms))
            }
            Self::Openthai { lane, .. } => {
                let (outcome, client_ms) = lane
                    .decide(case)
                    .map_err(|e| format!("openthai forward ({}): {e}", case.id))?;
                let (probs, _pick, _conf) = outcome.answers.first().ok_or_else(|| {
                    format!("{}: no answer for its single question", case.id)
                })?;
                let q = case.questions.first().ok_or_else(|| {
                    format!("{}: no questions for its single gold", case.id)
                })?;
                let keyed = openthai_keyed_probs(q, probs)
                    .map_err(|e| format!("{}: {e}", case.id))?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
        }
    }
}

/// Build the label-keyed probability vector one openthai answer speaks,
/// over the question's PRESENTED keys in presentation order (the same
/// iteration [`crate::lanes::openthai::map_answers`] reads positionally):
/// choice → the criteria object's keys; score → the level indices
/// ("0".."k-1", the FixedInt mapping's own spelling); noul → their
/// p(yes) reads as [p_no, p_yes] over the pair, keyed "0"/"1" (the gold
/// idx space). The keys are exactly what [`map_targets`] matches on:
/// Name suites match them to the student labels, FixedInt suites ignore
/// the key string and read the position.
pub(crate) fn openthai_keyed_probs(
    q: &crate::harness::suites::SuiteQuestion,
    probs: &[f64],
) -> Result<Vec<(String, f64)>, String> {
    use crate::harness::suites::QKind;
    let keys: Vec<String> = match q.kind {
        QKind::Noul => (0..2).map(|i| i.to_string()).collect(),
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            (0..n).map(|i| i.to_string()).collect()
        }
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .ok_or_else(|| "choice criteria must be an object".to_string())?,
    };
    if keys.len() != probs.len() {
        return Err(format!(
            "their answer carries {} probabilities vs {} presented keys — wire drift",
            probs.len(),
            keys.len()
        ));
    }
    Ok(keys.into_iter().zip(probs.iter().copied()).collect())
}

/// Construct the selected teacher for one distill run (Plan 426 T1).
/// `laya` needs the checkpoints + the `laya-riir` feature; `openthai`
/// health-checks their loopback service here — a server that is down is
/// a LOUD refusal naming the env (the agentjev law), never a half-run.
pub(crate) fn construct_teacher(name: &str, spec: &SuiteSpec) -> Result<TeacherForward, String> {
    match name {
        "openthai" => {
            let lane = crate::lanes::openthai::OpenThaiLane::default();
            lane.health()?;
            Ok(TeacherForward::Openthai {
                lane,
                // Filled by the warmup's decide (the model id is read
                // off their response — the lane's own law).
                provenance: "openthai:?".to_string(),
            })
        }
        "laya" => {
            #[cfg(feature = "laya-riir")]
            {
                let ckpt = super::laya_checkpoints_for(spec.name)
                    .first()
                    .copied()
                    .ok_or_else(|| format!("{}: no laya checkpoint", spec.name))?;
                let ck = match ckpt {
                    "english" => crate::laya::config::Checkpoint::English,
                    "multilingual" => crate::laya::config::Checkpoint::Multilingual,
                    "typed" => crate::laya::config::Checkpoint::TypedDecisions,
                    other => return Err(format!("unknown checkpoint {other}")),
                };
                let agent = Box::new(super::load_laya_agent(ckpt, ck)?);
                Ok(TeacherForward::Laya { agent, provenance: ckpt })
            }
            #[cfg(not(feature = "laya-riir"))]
            {
                let _ = spec;
                Err(
                    "--distill-teacher laya needs the `laya-riir` feature — rebuild: cargo \
                     build --release --features laya-riir --bin harness (macOS: \
                     laya-riir-metal)"
                        .to_string(),
                )
            }
        }
        other => Err(format!(
            "unknown --distill-teacher {other:?} (the seam knows 'laya' and 'openthai')"
        )),
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
    /// The teacher-side provenance: the laya checkpoint name, or the
    /// openthai model id read off their live response (Plan 426 T1).
    pub checkpoint: String,
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
/// (the engine argmax law). `pub(crate)`: the synth lane's veto (Plan 426
/// T5) accepts a candidate iff the teacher's argmax key equals the gold
/// label — the same tie law.
pub(crate) fn argmax_lowest_pos(probabilities: &[(String, f64)]) -> usize {
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

/// Distill one suite's train rows through the selected teacher
/// (Plan 426 T1: `--distill-teacher`, default laya — the 576 T3 posture).
fn distill_suite(
    spec: &SuiteSpec,
    dir: &Path,
    out_dir: &Path,
    limit: usize,
    teacher_name: &str,
) -> Result<DistillSuite, String> {
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
    // Student-side label universe: the distinct surviving-doc labels in
    // SORTED order — riir-train's `label_universe` law ("stable domain
    // order; the artifact carries the names"), which the student asserts
    // as exact equality (order included).
    let mut classes: Vec<String> = Vec::with_capacity(16);
    for d in &docs {
        if !classes.iter().any(|c| c == &d.label) {
            classes.push(d.label.clone());
        }
    }
    classes.sort();
    let n_classes = classes.len();

    let keymap = KeyMap::for_suite(spec.name)
        .ok_or_else(|| format!("{}: no key mapping (not a T3 suite)", spec.name))?;
    let mut teacher = construct_teacher(teacher_name, spec)?;

    let header = serde_json::json!({
        "suite": spec.name,
        "teacher": teacher.name(),
        "checkpoint": teacher.provenance(),
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

    // One discarded warmup forward: compiles the laya Metal pipelines /
    // warms their server's cold path so the per-row latency stat is not
    // cold-skewed (the run_laya_checkpoint pre-ramp law, in the data-pass
    // posture). For openthai this is also the model-id capture + the
    // final loud proof their server answers.
    if let Some(case) = built.cases.first() {
        teacher.warmup(case)?;
        eprintln!(
            "  [distill {}] warmup done ({}/{})",
            spec.name,
            teacher.name(),
            teacher.provenance()
        );
    }

    let mut durs_ms: Vec<u64> = Vec::with_capacity(docs.len());
    let mut hits = 0usize;
    for (doc, case) in docs.iter().zip(built.cases.iter()) {
        let t0 = std::time::Instant::now();
        let (probabilities, ms) = teacher.forward(case)?;
        durs_ms.push(if ms > 0 { ms } else { t0.elapsed().as_millis() as u64 });
        let (gold_c, pick_c, q) = map_targets(
            keymap,
            &doc.label,
            case.gold[0].idx,
            &probabilities,
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
        checkpoint: teacher.provenance().to_string(),
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
    teacher_name: &str,
) -> Result<DistillOutput, String> {
    let device_env = std::env::var("LAYA_DEVICE").unwrap_or_else(|_| "default".into());
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
        match distill_suite(spec, &opts.datasets_dir, out_dir, limit, teacher_name) {
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
            host: hostname_refusing_unknown(),
            device: format!("teacher={teacher_name}; laya_device={device_env}"),
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
    s.push_str("# Distill teacher pass — teacher probabilities over train rows (riir-train Issue 576 T3 / Plan 426 T1)\n\n");
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
    fn fixed_int_maps_by_position_into_the_student_order() {
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

    // ── Plan 426 T1: the openthai keying seam ─────────────────────────

    fn choice_question(criteria: serde_json::Value) -> crate::harness::suites::SuiteQuestion {
        crate::harness::suites::SuiteQuestion {
            qid: "q".into(),
            kind: crate::harness::suites::QKind::Choice,
            instructions: String::new(),
            criteria,
        }
    }

    #[test]
    fn openthai_keying_reads_the_criteria_key_order() {
        let q = choice_question(serde_json::json!({"b_key": 1, "a_key": 2, "c_key": 3}));
        let keyed = openthai_keyed_probs(&q, &[0.2, 0.7, 0.1]).expect("keyed");
        let keys: Vec<&str> = keyed.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["b_key", "a_key", "c_key"], "the criteria object's own key order (the same iteration map_answers reads)");
        assert_eq!(keyed[1].1, 0.7);
    }

    #[test]
    fn openthai_keying_scores_and_noul_speak_the_fixed_int_spelling() {
        let mut q = choice_question(serde_json::json!(["l0", "l1", "l2"]));
        q.kind = crate::harness::suites::QKind::Score;
        let keyed = openthai_keyed_probs(&q, &[0.1, 0.6, 0.3]).expect("keyed");
        assert_eq!(
            keyed.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
            vec!["0", "1", "2"],
            "the level-index spelling map_targets' FixedInt arm reads"
        );
        q.kind = crate::harness::suites::QKind::Noul;
        q.criteria = serde_json::Value::Null;
        let keyed = openthai_keyed_probs(&q, &[0.9, 0.1]).expect("keyed");
        assert_eq!(
            keyed.iter().map(|(k, v)| (k.as_str(), *v)).collect::<Vec<_>>(),
            vec![("0", 0.9), ("1", 0.1)],
            "their p(yes) arrives as [p_no, p_yes] — the gold idx space"
        );
    }

    #[test]
    fn openthai_keying_refuses_width_drift() {
        let q = choice_question(serde_json::json!({"a": 1, "b": 2}));
        assert!(openthai_keyed_probs(&q, &[0.5, 0.3, 0.2]).is_err());
        // A choice question without an object has no keys to speak.
        let mut bad = choice_question(serde_json::json!(["x"]));
        bad.kind = crate::harness::suites::QKind::Choice;
        assert!(openthai_keyed_probs(&bad, &[1.0]).is_err());
    }
}
