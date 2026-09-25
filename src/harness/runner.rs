//! The Plan 603 T1.5 harness runner: builds each suite from the fetched
//! datasets-server row files, runs BOTH lanes over byte-identical questions,
//! and emits the honest per-task tables (`results.json` + `TABLES.md`).
//!
//! Lane contracts:
//! - **modelless** — [`crate::engine::DecisionEngine`], one domain per label
//!   (corpus-is-the-model: train-split texts of that label, capped per
//!   label), sigmoid-gate calibration fit on a train-tail calibration slice
//!   via `observe`, fused abstain at the birth-measured thresholds.
//! - **laya** (feature `laya-riir`) — [`crate::laya::riir::RiirAgent`], the
//!   G5-parity-gated forward (the riir-owned backend since the candle
//!   removal, `.issues/006`); hard metrics read the same answer
//!   envelope the reference benches read (rounded-4 probabilities, max-prob
//!   confidence per protocols §5.1).
//!
//! Honesty rules enforced here (plan caveats 3/4):
//! - every number in the tables comes from a run of THIS binary — nothing
//!   hand-typed;
//! - the bit-identity determinism CLAIM is scoped to the modelless lane
//!   (caveat 3); the laya lane gets the same observed repeat check and it is
//!   REPORTED, but the published claim stays the modelless one;
//! - losses are printed, never hidden (the zero-shot-breadth caveat).
//!
//! Protocol divergences from the laya reference (documented, deliberate):
//! - option sampling for `massive_intent_en` uses SplitMix64 (suites.rs), not
//!   CPython MT19937 — the comparison integrity that matters is that BOTH
//!   lanes here answer byte-identical questions, which this runner
//!   guarantees by construction (one Suite, two renderers);
//! - banking77 uses the `mteb/banking77` mirror (the reference's own
//!   bench_apps variant) because `PolyAI/banking77` is script-based and
//!   unservable (dataset_manifest.md Gaps);
//! - the engine-side context is state + prompt only (wire `criteria` stays
//!   None) — matching the modelless lane's serving path.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;

use crate::embed::EMBED_DIM;
use crate::engine::{
    DecisionEngine, EngineConfig, ExpertSpec, FusedGateRecommendation, GateObservation, Posture,
    Scratch, recommend_fused_gate,
};
use crate::harness::families::SynthData;
use crate::harness::latency::{LatencyExtremes, fmt_p99_cell};
use crate::harness::metrics::{
    CalibrationPair, ConfusionRow, G1Verdict, HardMetrics, conformal_naive_floor, confusion_top,
    ece_of, g1_verdict_of, hard_metrics, score_metrics, soft_metrics, subset_accuracy,
};
use crate::harness::pair_heads::{
    ArmedPair, MAX_ARMED_PAIRS, MIN_CAL_SUPPORT, PairHead, PairHeadAb, PairSubsetRow, select_pairs,
};
use crate::harness::suites::{
    QKind, Suite, SuiteCase, TrainDoc, build_ag_news, build_banking77_mteb, build_emotion,
    build_massive_intent_en, build_prompt_injections, build_sst5, build_typed_decisions,
    build_xnli_en, stratified_selection_slices, train_docs,
};
use crate::pyjson::serialize_state;

/// Where the fetch layer leaves the row files.
pub const DEFAULT_DATASETS_DIR: &str = ".raw/datasets";

/// The SplitMix64 option seed for MASSIVE (any fixed value; reproducibility
/// is what matters, not the value).
pub const MASSIVE_OPTION_SEED: u64 = 0x0603_2026_0922;

// ── suite registry ──────────────────────────────────────────────────────

struct SuiteSpec {
    name: &'static str,
    /// Test-row eval cap (0 = all rows on disk).
    test_cap: usize,
    /// Calibration-slice cap (train rows reused as cases; 0 = none).
    cal_cap: usize,
    corpus_cap_per_label: usize,
    build: fn(&Value, usize) -> Suite,
    /// Some(build) = in-process synthetic suite (Issue 004): no dataset
    /// files — `prepare` never touches `dir`, the builder self-splits
    /// corpus/cal/eval, and the caps above are ignored.
    synthetic: Option<fn() -> SynthData>,
    /// false = LLM-lane ONLY (Issue 004 T3): the modelless lane has no KV
    /// cache, so it has no honest answer for the family — the runner skips
    /// it LOUDLY (an absence line, never a silent zero) when `laya` is off.
    modelless_lane: bool,
}

/// Per-suite registry. Domain counts are pinned to the FETCHED test rows
/// (dataset_manifest.md); the runner asserts them at build time so a
/// silently different fetch fails loud instead of mis-arming the engine.
const SUITES: &[SuiteSpec] = &[
    SuiteSpec {
        name: "typed_decisions",
        test_cap: 0, // all 400 — no sampling (protocols §3.1)
        cal_cap: 100,
        corpus_cap_per_label: 48,
        build: build_typed_decisions,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "ag_news",
        test_cap: 400,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        build: build_ag_news,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "emotion",
        test_cap: 400,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        build: build_emotion,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "sst5",
        test_cap: 600,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        build: build_sst5,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "prompt_injections",
        test_cap: 0, // all 116
        cal_cap: 100,
        corpus_cap_per_label: 64,
        build: build_prompt_injections,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "xnli_en",
        test_cap: 300,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        build: build_xnli_en,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "massive_intent_en",
        test_cap: 300,
        cal_cap: 200,
        corpus_cap_per_label: 48,
        build: |v, n| build_massive_intent_en(v, n, MASSIVE_OPTION_SEED),
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "banking77",
        test_cap: 500,
        cal_cap: 200,
        corpus_cap_per_label: 40,
        build: build_banking77_mteb,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "code_fixtures",
        test_cap: 0,
        cal_cap: 0, // generated with its own gold-programmatic cal slice
        corpus_cap_per_label: usize::MAX, // the builder fixes its own corpus
        build: build_code_fixtures, // generated in-process; rows_file unused
        synthetic: None, // legacy in-process path (prepare branch below)
        modelless_lane: true,
    },
    // ── Issue 004 (Research 579): the six harness decision-point families ──
    // In-process synthetic suites (no datasets); five modelless by default,
    // cache_reuse LLM-lane only (loud skip without `laya`).
    SuiteSpec {
        name: "harness_visibility",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        build: synthetic_build_unused,
        synthetic: Some(families_synth_visibility),
        modelless_lane: true,
    },
    SuiteSpec {
        name: "harness_permissions",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        build: synthetic_build_unused,
        synthetic: Some(families_synth_permissions),
        modelless_lane: true,
    },
    SuiteSpec {
        name: "harness_tool_fit",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        build: synthetic_build_unused,
        synthetic: Some(families_synth_tool_fit),
        modelless_lane: true,
    },
    SuiteSpec {
        name: "harness_routing",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        build: synthetic_build_unused,
        synthetic: Some(families_synth_routing),
        modelless_lane: true,
    },
    SuiteSpec {
        name: "harness_sensitivity",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        build: synthetic_build_unused,
        synthetic: Some(families_synth_sensitivity),
        modelless_lane: true,
    },
    SuiteSpec {
        name: "harness_cache_reuse",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        build: synthetic_build_unused,
        // Issue 004 T3: the modelless lane has no KV cache — the family is
        // answered by the `laya` lane only, and the default build reports a
        // loud SKIPPED absence instead of a fake answer.
        modelless_lane: false,
        synthetic: Some(families_synth_cache_reuse),
    },
];

/// Never called — synthetic suites build via their `synthetic` fn; this
/// placeholder keeps the `build` field total.
fn synthetic_build_unused(_rows: &Value, _max_rows: usize) -> Suite {
    unreachable!("synthetic suites build via SuiteSpec::synthetic")
}

fn families_synth_visibility() -> SynthData {
    crate::harness::families::synth_visibility()
}

fn families_synth_permissions() -> SynthData {
    crate::harness::families::synth_permissions()
}

fn families_synth_tool_fit() -> SynthData {
    crate::harness::families::synth_tool_fit()
}

fn families_synth_routing() -> SynthData {
    crate::harness::families::synth_routing()
}

fn families_synth_sensitivity() -> SynthData {
    crate::harness::families::synth_sensitivity()
}

fn families_synth_cache_reuse() -> SynthData {
    crate::harness::families::synth_cache_reuse()
}

/// Checkpoints per suite (laya lane): the specialist `typed` model answers
/// the typed-decisions headline alongside the two base checkpoints; every
/// other suite is the general `english` checkpoint (the reference's own
/// layout — T4 ran base checkpoints on the English suites and `typed` on
/// typed-decisions).
fn laya_checkpoints_for(suite: &str) -> &'static [&'static str] {
    if suite == "typed_decisions" {
        &["typed", "english", "multilingual"]
    } else {
        &["english"]
    }
}

// ── code fixtures (our own suite — no network, programmatic gold) ───────

/// The modules the code fixtures draw from (fixed order IS the option
/// order). Real code spans from THIS repo's own sources; gold labels are
/// programmatic (which module / is-pub), never hand-labeled.
const CODE_MODULES: &[&str] = &[
    "embed.rs",
    "engine.rs",
    "readout.rs",
    "serve.rs",
    "harness/metrics.rs",
    "harness/suites.rs",
    "laya/agent.rs",
    "laya/router.rs",
];

const CODE_MODULE_LABELS: &[&str] = &[
    "embed",
    "engine",
    "readout",
    "serve",
    "harness::metrics",
    "harness::suites",
    "laya::agent",
    "laya::router",
];

struct CodeFn {
    src: String,
    is_pub: bool,
}

/// Extract top-level `fn` decls from one source file: returns (decl line,
/// body text up to the closing brace at column 0 or a 80-line cap).
fn extract_fns(path: &Path) -> Vec<CodeFn> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut fns = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let is_pub = trimmed.starts_with("pub fn ");
        let is_fn = trimmed.starts_with("fn ");
        if is_pub || is_fn {
            let start = i;
            let mut body = String::new();
            while i < lines.len() && i - start < 80 {
                body.push_str(lines[i]);
                body.push('\n');
                if i > start && lines[i].starts_with('}') {
                    break;
                }
                i += 1;
            }
            fns.push(CodeFn { src: body, is_pub });
        }
        i += 1;
    }
    fns
}

/// The deterministic code-fixture split: per module, fns[0..2] = eval cases,
/// fns[2..10] = calibration cases, fns[10..16] = corpus docs (with graceful
/// fallbacks for small modules; every module yields ≥1 eval case).
fn code_fn_slices() -> Vec<(usize /*module idx*/, Vec<CodeFn>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    CODE_MODULES
        .iter()
        .enumerate()
        .map(|(mi, m)| (mi, extract_fns(&root.join(m))))
        .collect()
}

fn code_case(mi: usize, f: &CodeFn, id: &str) -> SuiteCase {
    let mut crit = serde_json::Map::new();
    for label in CODE_MODULE_LABELS {
        crit.insert((*label).to_string(), Value::Null);
    }
    SuiteCase {
        id: id.to_string(),
        state: Value::String(f.src.clone()),
        questions: vec![
            crate::harness::suites::SuiteQuestion {
                qid: "module".into(),
                kind: QKind::Choice,
                instructions: "Which module of the riir-reflex crate is this Rust function from?"
                    .into(),
                criteria: Value::Object(crit),
            },
            crate::harness::suites::SuiteQuestion {
                qid: "is_pub".into(),
                kind: QKind::Noul,
                instructions: "Is this Rust function declared `pub`?".into(),
                criteria: Value::Null,
            },
        ],
        gold: vec![
            crate::harness::suites::GoldAnswer {
                idx: mi,
                soft: vec![0.0; CODE_MODULE_LABELS.len()],
                gold_score: None,
            },
            crate::harness::suites::GoldAnswer {
                idx: usize::from(f.is_pub),
                soft: vec![0.0; 2],
                gold_score: None,
            },
        ],
    }
}

fn build_code_fixtures(_rows: &Value, _max_rows: usize) -> Suite {
    let mut cases = Vec::new();
    for (mi, fns) in code_fn_slices() {
        if fns.is_empty() {
            continue;
        }
        cases.push(code_case(
            mi,
            &fns[0],
            &format!("code:{}:0", CODE_MODULES[mi]),
        ));
        if fns.len() > 1 {
            cases.push(code_case(
                mi,
                &fns[1],
                &format!("code:{}:1", CODE_MODULES[mi]),
            ));
        }
    }
    Suite {
        name: "code_fixtures",
        cases,
        option_counts_note: "8 modules × up to 2 real fn spans from this repo's own sources; \
                             gold programmatic (module, is_pub)",
    }
}

/// The code-fixture calibration cases (gold-programmatic, same questions).
pub fn code_fixtures_cal_cases() -> Vec<SuiteCase> {
    let mut cases = Vec::new();
    for (mi, fns) in code_fn_slices() {
        for (k, f) in fns.iter().enumerate().skip(2).take(8) {
            cases.push(code_case(
                mi,
                f,
                &format!("codecal:{}:{k}", CODE_MODULES[mi]),
            ));
        }
    }
    cases
}

/// The code-fixture corpora: per module, fns[10..16] as docs — STRICTLY
/// after the eval (fns[0..2]) and calibration (fns[2..10]) slices, so no
/// cal/eval fn ever scores against itself in its own corpus (the
/// self-inclusion leak measured on the first run). Modules with fewer fns
/// contribute what remains; empty corpora take the self-doc fallback.
pub fn code_fixtures_docs() -> Vec<TrainDoc> {
    let mut docs = Vec::new();
    for (mi, fns) in code_fn_slices() {
        for f in fns.iter().skip(10).take(6) {
            docs.push(TrainDoc {
                label: CODE_MODULE_LABELS[mi].to_string(),
                text: f.src.clone(),
            });
        }
    }
    docs
}

/// Load one dataset suite's RAW envelopes (test + train) — the save-corpus
/// path's source of truth (Issue 007 P1: one corpus = ONE digest-pinned kv
/// row). Mirrors what `prepare` feeds the suite builders, byte-for-byte.
pub fn load_suite_envelope(
    datasets_dir: &Path,
    suite_name: &str,
) -> Result<serde_json::Value, String> {
    let spec = SUITES
        .iter()
        .find(|s| s.name == suite_name)
        .ok_or_else(|| format!("unknown suite {suite_name}"))?;
    if spec.synthetic.is_some() {
        return Err(format!(
            "suite {suite_name} is compiled-in (synthetic), not a dataset corpus — \
             nothing to store"
        ));
    }
    let suite_dir = datasets_dir.join(spec.name);
    let test = load_rows(&suite_dir, "test")?;
    let train = load_rows(&suite_dir, "train").map_err(|e| format!("suite {}: {e}", spec.name))?;
    Ok(serde_json::json!({
        "suite": spec.name,
        "test_rows": test,
        "train_rows": train,
    }))
}

// ── result types (serde — results.json) ─────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SelectiveMetrics {
    pub abstain_rate: f64,
    /// Forced accuracy among the NOT-abstained questions.
    pub selective_accuracy: f64,
    pub selective_n: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaneResult {
    pub lane: &'static str,
    /// Model/checkpoint name (modelless lane: "modelless").
    pub model: String,
    /// Hard metrics over ALL questions, forced (abstain ignored — argmax of
    /// the full probability vector; protocols §5.1 hard_metrics).
    pub hard: HardMetrics,
    /// ECE over the lane's own reported confidence readout (modelless: the
    /// engine's readout confidence; laya: its entropy `confidence` field).
    pub readout_ece: Option<f64>,
    /// Abstain behavior (modelless only — laya cannot abstain; the
    /// Research-562 flaw the wire fixes with first-class abstention).
    pub raw_abstain: Option<SelectiveMetrics>,
    pub calibrated_abstain: Option<SelectiveMetrics>,
    /// G1 surfaces (modelless only): ECE over the readout confidence raw vs
    /// sigmoid-gate-calibrated vs the conformal-naive floor.
    pub readout_ece_raw: Option<f64>,
    pub readout_ece_calibrated: Option<f64>,
    pub floor_ece: Option<f64>,
    pub g1_pass: Option<bool>,
    /// The G1 verdict spelled out (`pass`/`fail`/`no_claim`): `no_claim` is
    /// the calibrator-never-fitted state the `g1_pass` None projection hides,
    /// surfaced so the tables can print NO CLAIM instead of FAIL.
    pub g1_verdict: Option<G1Verdict>,
    /// Per-question-type hard metrics (typed_decisions only).
    pub by_question_type: Option<BTreeMap<String, HardMetrics>>,
    /// Soft-distribution metrics (typed_decisions only).
    pub soft_acc: Option<f64>,
    pub brier_soft: Option<f64>,
    /// Score metrics (typed_decisions score questions / sst5).
    pub score_mae: Option<f64>,
    pub within_1: Option<f64>,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub latency_tail_support: usize,
    /// First / max / argmax-case of the per-case samples (Issue 020 T8) —
    /// the p99 is the MAX whenever tail support is 1, and this says which
    /// case it was (case 0 = the cold-start reading). `None` on an empty lane.
    pub latency_extremes: Option<LatencyExtremes>,
    /// Repeat-run bit-identity check (first 10 cases answered twice).
    pub determinism_ok: Option<bool>,
    pub seconds: f64,
    pub n_cases: usize,
    pub n_questions: usize,
    /// The fitted fused-gate thresholds (cal-slice 30th percentile — the
    /// T1.6 arena posture ρ=30%). Default birth constants when the cal
    /// slice is too small to fit.
    pub score_threshold: f32,
    pub distance_threshold: f32,
    /// The fit-time recommendation the thresholds came from (Issue 009
    /// T2/T3 — jimothy's `thresholdRecommendation` shape): posture + per-
    /// axis support and accuracy disclosure; per-axis `null` on thin
    /// support. `None` on lanes that fit no gates (laya).
    pub threshold_recommendation: Option<FusedGateRecommendation>,
    /// The modelless lane's effective per-label corpus cap and how it was
    /// chosen — always disclosed so a table never reads against an unknown
    /// corpus posture (None = the lane has no corpus: the laya lanes).
    pub corpus_cap: Option<CorpusCapInfo>,
    /// Top-K categorical (gold → pred) confusion pairs, forced raw eval
    /// (modelless only; Issue 013 lever-3 probe instrumentation — report-
    /// only, never a gate). None on the laya lanes.
    pub confusion: Option<Vec<ConfusionRow>>,
    /// The lever-3 pair-head A/B records, `(top2-gated, pred-anchored)`
    /// (present only when the `--pair-head-ab` arm ran). None otherwise,
    /// and on the laya lanes.
    pub pair_head_ab: Option<(PairHeadAb, PairHeadAb)>,
    /// Issue 024 T3: hard accuracy over the eval rows NOT leak-flagged —
    /// the same forced-row walk `hard` reads, restricted to the unflagged
    /// cases. None = feature off / suite out of scope / every row flagged
    /// (an absence, never a fabricated rate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_deleaked: Option<f64>,
}

/// One cal-slice cap-selection candidate reading (Issue 013 lever-1
/// protocol plumb).
#[derive(Debug, Clone, Serialize)]
pub struct CapCandidate {
    pub cap: usize,
    /// Forced accuracy over the cal slice with the engine built at this
    /// corpus cap — the ONLY quantity selection reads (test is reported
    /// once, at the selected cap, never during selection).
    pub cal_acc: f64,
}

/// How the modelless lane's per-label corpus cap was chosen.
#[derive(Debug, Clone, Serialize)]
pub struct CorpusCapInfo {
    pub effective: usize,
    /// `registry` | `--corpus-cap override` | `cal-slice selection` |
    /// `registry (selection n/a: self-corpora)`.
    pub source: String,
    /// Some = the cal-slice selection table, candidates in ascending cap
    /// order (the registry default is always a candidate). `None` when no
    /// selection ran.
    pub selection: Option<Vec<CapCandidate>>,
}

/// Top-K cap for the confusion readout (the probe table's size).
pub const CONFUSION_TOP: usize = 12;

/// Categorical confusion over one lane's forced raw eval: Choice + Score
/// questions only — Noul is the binary `[no, yes]` wire (its "confusion" is
/// the flip, not a mineable pair). Option keys come from the same criteria
/// `engine_request` feeds the engine, so a pair names exactly the options
/// the engine scored.
#[must_use]
fn confusion_rows(eval: &Eval, cases: &[SuiteCase], top: usize) -> Vec<ConfusionRow> {
    let mut mispairs: Vec<(String, String)> = Vec::new();
    for (ci, case) in cases.iter().enumerate() {
        for (qi, q) in case.questions.iter().enumerate() {
            if q.kind == QKind::Noul {
                continue;
            }
            let keys: Vec<String> = match q.kind {
                QKind::Choice => q
                    .criteria
                    .as_object()
                    .map(|m| m.keys().cloned().collect())
                    .unwrap_or_default(),
                QKind::Score => q
                    .criteria
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                QKind::Noul => continue,
            };
            let gold = case.gold[qi].idx;
            let pick = eval.picks[ci][qi];
            if gold == pick {
                continue;
            }
            let key = |i: usize| keys.get(i).cloned().unwrap_or_else(|| format!("#{i}"));
            mispairs.push((key(gold), key(pick)));
        }
    }
    confusion_top(&mispairs, top)
}

// ── Issue 013 lever-3 pair-head A/B (report-only arm) ─────────────────

/// Display keys for the suite's option universe: the first non-Noul
/// question's criteria keys/items, falling back to the engine domain label.
/// Display-only (the record names pairs the way the questions did).
fn option_display_labels(suite: &Suite, labels: &[String]) -> Vec<String> {
    let mut out: Vec<String> = labels.to_vec();
    let Some(first_q) = suite
        .cases
        .iter()
        .flat_map(|case| case.questions.iter())
        .find(|q| q.kind != QKind::Noul)
    else {
        return out;
    };
    match first_q.kind {
        QKind::Choice => {
            if let Some(m) = first_q.criteria.as_object() {
                for (i, k) in m.keys().enumerate() {
                    if i < out.len() {
                        out[i] = k.clone();
                    }
                }
            }
        }
        QKind::Score => {
            if let Some(a) = first_q.criteria.as_array() {
                for (i, v) in a.iter().enumerate() {
                    if i < out.len() {
                        out[i] = match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                    }
                }
            }
        }
        QKind::Noul => {}
    }
    out
}

/// Which questions a pair head fires on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PairGate {
    /// The engine's own top-2 IS the armed pair — the canonical form. It
    /// can never see an error whose gold ranked ≥ 3 (the measured black-
    /// hole class).
    Top2,
    /// The engine's PICK is in an armed pair — the pred-anchored variant.
    /// It sees the black-hole errors directly; overlapping pairs resolve
    /// to the first armed pair (cal-support order, deterministic).
    PredAnchored,
}

/// The pair-head A/B pass (Issue 013 lever 3): arm the top confusion pairs
/// from the CAL slice's mispredictions, fit a diagonal-LDA head per pair
/// from the pair's corpus docs (the same capped docs the engine's domains
/// consume), then re-decide questions under BOTH firing gates. Forced
/// picks only; Choice + Score questions (Noul is the binary wire).
/// Report-only: the result is the record, never a gate, and nothing
/// outside an armed pair's subset can move. Returns
/// `(top2-gated, pred-anchored)`.
#[allow(clippy::too_many_arguments)]
fn pair_head_ab_pass(
    raw_eval: &Eval,
    suite: &Suite,
    state_strs: &[String],
    cal_eval: &Eval,
    cal_cases: &[SuiteCase],
    train: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
    embedder: &crate::embed::Embedder,
) -> Result<(PairHeadAb, PairHeadAb), String> {
    // 1. Arm: cal-slice mispredictions in INDEX space (the display-key
    // confusion readout is for the record; selection needs class indices).
    let mut cal_mispairs: Vec<(usize, usize)> = Vec::new();
    for (ci, case) in cal_cases.iter().enumerate() {
        for (qi, q) in case.questions.iter().enumerate() {
            if q.kind == QKind::Noul {
                continue;
            }
            let gold = case.gold[qi].idx;
            let pick = cal_eval.picks[ci][qi];
            if gold != pick {
                cal_mispairs.push((gold, pick));
            }
        }
    }
    let armed = select_pairs(&cal_mispairs, MAX_ARMED_PAIRS, MIN_CAL_SUPPORT);

    // 2. Fit: per-class corpus doc embeddings (same capped train docs the
    // engine's domains consume, same order).
    let display = option_display_labels(suite, labels);
    let mut heads: Vec<PairHead<EMBED_DIM>> = Vec::with_capacity(armed.len());
    if !armed.is_empty() {
        let docs_for = |li: usize| -> Result<Vec<[f32; EMBED_DIM]>, String> {
            let texts: Vec<&String> = train
                .iter()
                .filter(|d| d.label == labels[li])
                .map(|d| &d.text)
                .take(cap_per_label)
                .collect();
            let mut vs = Vec::with_capacity(texts.len());
            for t in texts {
                let mut v = [0.0f32; EMBED_DIM];
                embedder.embed_into(t.as_bytes(), &mut v);
                vs.push(v);
            }
            Ok(vs)
        };
        for arm in &armed {
            let da = docs_for(arm.a)?;
            let db = docs_for(arm.b)?;
            match PairHead::fit(arm.a, arm.b, &da, &db) {
                Some(h) => heads.push(h),
                // A class with no corpus docs cannot arm (the engine's
                // self-doc fallback covers scoring, not a fitted mean).
                None => continue,
            }
        }
    }

    // 3. Apply under both gates: re-decide the fired subsets from the
    // state embedding (embed once per case, reused by both walks).
    let mut state_embeds: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(suite.cases.len());
    for s in state_strs {
        let mut v = [0.0f32; EMBED_DIM];
        embedder.embed_into(s.as_bytes(), &mut v);
        state_embeds.push(v);
    }
    let top2_ab = pair_head_walk(
        PairGate::Top2,
        &heads,
        &armed,
        &display,
        suite,
        raw_eval,
        &state_embeds,
    );
    let pred_ab = pair_head_walk(
        PairGate::PredAnchored,
        &heads,
        &armed,
        &display,
        suite,
        raw_eval,
        &state_embeds,
    );
    Ok((top2_ab, pred_ab))
}

/// One gated A/B walk over the test cases with pre-fitted heads.
#[allow(clippy::too_many_arguments)]
fn pair_head_walk(
    gate: PairGate,
    heads: &[PairHead<EMBED_DIM>],
    armed: &[ArmedPair],
    display: &[String],
    suite: &Suite,
    raw_eval: &Eval,
    state_embeds: &[[f32; EMBED_DIM]],
) -> PairHeadAb {
    let mut per_pair: Vec<PairSubsetRow> = armed
        .iter()
        .map(|arm| PairSubsetRow {
            a: arm.a,
            b: arm.b,
            a_label: display.get(arm.a).cloned().unwrap_or_default(),
            b_label: display.get(arm.b).cloned().unwrap_or_default(),
            n: 0,
            baseline_correct: 0,
            head_correct: 0,
            n_gold_in_pair: 0,
            baseline_correct_in_pair: 0,
            head_correct_in_pair: 0,
        })
        .collect();
    let mut baseline_correct_all = 0usize;
    let mut head_correct_all = 0usize;
    let mut counted = 0usize;
    let mut overrides = 0usize;
    let mut base_on_subset = 0usize;
    let mut head_on_subset = 0usize;
    let mut subset_in_pair = 0usize;
    let mut base_in_pair = 0usize;
    let mut head_in_pair = 0usize;
    for (ci, case) in suite.cases.iter().enumerate() {
        for (qi, q) in case.questions.iter().enumerate() {
            if q.kind == QKind::Noul {
                continue;
            }
            let gold = case.gold[qi].idx;
            let pick = raw_eval.picks[ci][qi];
            let probs = &raw_eval.probs[ci][qi];
            let base_ok = gold == pick;
            baseline_correct_all += usize::from(base_ok);
            counted += 1;
            // Which armed pair (if any) fires, and which pair indices the
            // record attributes the question to.
            let fired = match gate {
                PairGate::Top2 => {
                    let Some((i1, i2)) = top2(probs) else {
                        head_correct_all += usize::from(base_ok);
                        continue;
                    };
                    // Attribute canonically (h.a < h.b) — the top-2 may
                    // arrive in either order.
                    heads
                        .iter()
                        .find(|h| (h.a == i1 && h.b == i2) || (h.a == i2 && h.b == i1))
                        .map(|h| (h.a, h.b, h))
                }
                PairGate::PredAnchored => heads
                    .iter()
                    .find(|h| pick == h.a || pick == h.b)
                    .map(|h| (h.a, h.b, h)),
            };
            let Some((pi_a, pi_b, head)) = fired else {
                head_correct_all += usize::from(base_ok);
                continue;
            };
            let pi = per_pair
                .iter_mut()
                .find(|r| r.a == pi_a && r.b == pi_b)
                .expect("fired head came from the armed set");
            pi.n += 1;
            pi.baseline_correct += usize::from(base_ok);
            base_on_subset += usize::from(base_ok);
            overrides += 1;
            let gold_in_pair = gold == pi_a || gold == pi_b;
            if gold_in_pair {
                pi.n_gold_in_pair += 1;
                subset_in_pair += 1;
            }
            pi.baseline_correct_in_pair += usize::from(base_ok && gold_in_pair);
            base_in_pair += usize::from(base_ok && gold_in_pair);
            let hp = head.pick(&state_embeds[ci]);
            let head_ok = gold == hp;
            pi.head_correct += usize::from(head_ok);
            pi.head_correct_in_pair += usize::from(head_ok && gold_in_pair);
            head_on_subset += usize::from(head_ok);
            head_in_pair += usize::from(head_ok && gold_in_pair);
            head_correct_all += usize::from(head_ok);
        }
    }
    let f = |n: usize| n as f64 / counted.max(1) as f64;
    PairHeadAb {
        armed: armed.to_vec(),
        baseline_acc: f(baseline_correct_all),
        head_acc: f(head_correct_all),
        n_counted: counted,
        n_overrides: overrides,
        subset_n: overrides,
        baseline_correct_on_subset: base_on_subset,
        head_correct_on_subset: head_on_subset,
        subset_gold_in_pair: subset_in_pair,
        baseline_correct_in_pair: base_in_pair,
        head_correct_in_pair: head_in_pair,
        per_pair,
    }
}

/// Top-2 indices by first-max-wins (ties keep the earlier index), or `None`
/// under two options.
#[must_use]
fn top2(probs: &[f64]) -> Option<(usize, usize)> {
    if probs.len() < 2 {
        return None;
    }
    let mut best = if probs[1] > probs[0] { (1, 0) } else { (0, 1) };
    for i in 2..probs.len() {
        let p = probs[i];
        if p > probs[best.0] {
            best = (i, best.0);
        } else if p > probs[best.1] {
            best.1 = i;
        }
    }
    Some(best)
}

/// The default cap-selection candidate ladder for `--cal-select-cap`
/// (the Bench 003 sweep grid; the registry default joins the candidate set
/// automatically at run time).
pub const DEFAULT_CAL_SELECT_CAPS: &[usize] = &[8, 16, 32, 64, 128, 256, 512];

/// Parse the `--cal-select-cap` value: `None` (bare flag) = the default
/// ladder; `Some(list)` = a comma-separated candidate list. Zero is not a
/// corpus cap (take(0) would build every domain on its self-doc fallback).
pub fn parse_cal_select_caps(arg: Option<&str>) -> Result<Vec<usize>, String> {
    match arg {
        None => Ok(DEFAULT_CAL_SELECT_CAPS.to_vec()),
        Some(s) => s
            .split(',')
            .map(|p| {
                let cap = p
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| format!("--cal-select-cap: bad candidate {p:?}"))?;
                if cap == 0 {
                    return Err("--cal-select-cap: 0 is not a corpus cap".to_string());
                }
                Ok(cap)
            })
            .collect(),
    }
}

/// The selection law over the cal readings: argmax cal accuracy; exact
/// ties prefer the registry default, then the smallest candidate. The
/// readings are bit-reproducible (fixed fixtures, no RNG), so ties are
/// exact and the law is deterministic.
#[must_use]
fn select_cap(cands: &[CapCandidate], registry_default: usize) -> usize {
    let Some(best) = cands.first() else {
        return registry_default;
    };
    let mut best = best;
    for c in &cands[1..] {
        let wins = c.cal_acc > best.cal_acc
            || (c.cal_acc == best.cal_acc
                && (c.cap == registry_default
                    || (best.cap != registry_default && c.cap < best.cap)));
        if wins {
            best = c;
        }
    }
    best.cap
}

#[derive(Debug, Clone, Serialize)]
pub struct SuiteResult {
    pub name: String,
    pub n_cases: usize,
    pub n_questions: usize,
    /// None = the family has NO modelless lane (Issue 004 T3: LLM-only —
    /// an honest absence, never a fabricated row).
    pub modelless: Option<LaneResult>,
    /// checkpoint name → result (empty when compiled without `laya` or the
    /// lane was disabled — the table prints the honest absence).
    pub laya: BTreeMap<String, LaneResult>,
    /// Issue 019 T3 (`.issues/027`): the CLM comparison lane's row — the
    /// external Contrastive-LM reference over `/v1/systemone`, measured
    /// client-side on this box. Absent (never a fabricated row) when the
    /// lane did not run (flag off / feature off / server unreachable —
    /// the absences section names it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clm: Option<LaneResult>,
    /// Issue 029: the GLiNER comparison lane's row — fastino/GLiNER2.5-Decide
    /// (Apache-2.0, not affiliated) as a JSONL subprocess oracle over THEIR
    /// gliner2 package (`scripts/gliner_lane.py`, the laya-python protocol).
    /// Absent (never a fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gliner: Option<LaneResult>,
    /// Issue 025 amendment 4 / `.issues/027`: the AgentJev comparison lane's
    /// row — their `jev_service` (Apache-2.0, not affiliated) served on
    /// loopback, measured over HTTP. The missing data point this lane owns:
    /// AgentJev's GOLD-LABEL accuracy on our typed_decisions split (their
    /// published 79.25% is teacher-argmax agreement — a different protocol).
    /// Absent (never a fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agentjev: Option<LaneResult>,
    /// Issue 024 T3: the corpus∪cal → eval near-duplicate leak scan over
    /// the slices THIS run served. Absent (not a zero rate) when the
    /// `slice_leak` feature is off or the suite is out of scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leak: Option<SuiteLeak>,
}

/// Issue 024 T3 — the per-suite leak block (results.json, additive).
/// Reference side = the train split the run's corpora/calibration draw
/// from; query side = the built eval cases. Counts are ROWS, never a
/// rate — rates belong to the site layer (T4). Semantics are the probe's
/// (`scripts/slice_leak_probe.py`): EXACT over every eval row, the NEAR
/// scan over the first `near_cap` eval rows — so these counts sit at or
/// below the probe's train-vs-test bound by construction (the registry
/// test caps can only shrink the query side).
#[derive(Debug, Clone, Serialize)]
pub struct SuiteLeak {
    /// The NEAR-twin Jaccard threshold (`LeakParams::default` → 0.8).
    pub threshold: f64,
    pub n_reference: usize,
    pub n_eval: usize,
    pub exact: usize,
    pub near: usize,
}

#[derive(Debug, Serialize)]
pub struct RunMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub host: String,
    pub profile: String,
    pub laya_feature: bool,
    /// 0 = uncapped (protocol default). >0 = the laya lane was question-capped
    /// per checkpoint — a PARTIAL run; disclosed in the table header so a
    /// capped laya `n` is never read against a full-N modelless `n`.
    pub laya_max_questions: usize,
    pub datasets_dir: String,
    /// The run's corpus-cap posture: registry defaults, an explicit
    /// override, or the cal-slice selection protocol (Issue 013 lever 1).
    pub corpus_cap_mode: String,
    pub corpus_protocol: String,
    pub calibration_protocol: String,
    pub floor_definition: String,
    pub determinism_scoping: String,
    /// The riir lane's device posture for this run (honest latency reading:
    /// the same build answers differently on cpu vs metal).
    pub laya_device: String,
    /// Whether the laya-python (torch reference) oracle lane ran, and its
    /// measurement caveats when it did.
    pub laya_python_lane: String,
    /// Whether the CLM comparison lane ran (Issue 019 T3 / .issues/027),
    /// and its serving posture when it did (the external reference over
    /// `/v1/systemone` — comparison lane, never a product lane).
    pub clm_lane: String,
    /// Whether the GLiNER comparison lane ran (Issue 029), and its serving
    /// posture when it did (fastino/GLiNER2.5-Decide over their gliner2
    /// package as a JSONL subprocess — comparison lane, never a product
    /// lane).
    pub gliner_lane: String,
    /// Whether the AgentJev comparison lane ran (Issue 025 amendment 4 /
    /// `.issues/027`), and its serving posture when it did (their
    /// `jev_service` on loopback, measured over HTTP — the MEASURE-vs-SERVE
    /// split; comparison lane, never a product lane).
    pub agentjev_lane: String,
    /// Power source / power mode / load / swap at run start AND end, with a
    /// latency-quotable verdict (Issue 021 T7) — the axis every Issue 020
    /// A/B was missing.
    pub box_state: super::box_state::BoxStateSpan,
    pub divergences: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RunOutput {
    pub meta: RunMeta,
    pub suites: Vec<SuiteResult>,
}

// ── data loading ────────────────────────────────────────────────────────

/// Concatenate `.rows[].row` across every `test-*.json` (or `train-*.json`)
/// page file, in sorted filename order, into one synthetic envelope.
fn load_rows(dir: &Path, split: &str) -> Result<Value, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&format!("{split}-")) && n.ends_with(".json"))
        })
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("no {split}-*.json files under {}", dir.display()));
    }
    let mut rows: Vec<Value> = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("read {}: {e}", f.display()))?;
        let v: Value =
            serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", f.display()))?;
        let page = v
            .get("rows")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{}: no rows array", f.display()))?;
        for r in page {
            if let Some(row) = r.get("row") {
                rows.push(row.clone());
            }
        }
    }
    // Re-wrap in the datasets-server ENVELOPE shape the suite builders
    // consume (`{"rows": [{"row_idx": N, "row": {...}}]}`) — the builders
    // read `.rows[i].row`, never bare rows.
    let wrapped: Vec<Value> = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| serde_json::json!({ "row_idx": i, "row": row }))
        .collect();
    Ok(serde_json::json!({ "rows": wrapped }))
}

// ── engine construction ─────────────────────────────────────────────────

/// Build the per-suite engine: one domain per label, corpus = capped train
/// docs of that label (self-doc fallback so an option with no train rows
/// still has a corpus — covered by `corpus_protocol`, never an empty
/// domain).
fn build_engine<const N: usize>(
    suite: &str,
    train: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
    cfg: EngineConfig,
) -> Result<DecisionEngine<N, EMBED_DIM>, String> {
    assert_eq!(
        labels.len(),
        N,
        "suite {suite}: {N} domains armed but the option universe has {} labels",
        labels.len()
    );
    let mut specs: Vec<ExpertSpec> = Vec::with_capacity(N);
    for label in labels {
        let mut docs: Vec<String> = train
            .iter()
            .filter(|d| d.label == *label)
            .map(|d| d.text.clone())
            .take(cap_per_label)
            .collect();
        if docs.is_empty() {
            // Self-doc fallback: the label text itself is the corpus (a
            // corpus-is-the-model engine always has SOMETHING to score
            // against; without this, out-of-train options would be
            // unscorable). Reported in the run meta via corpus_protocol.
            docs.push(label.clone());
        }
        specs.push(ExpertSpec::new(label.as_str(), &docs));
    }
    DecisionEngine::<N, EMBED_DIM>::build_specs(specs, cfg)
        .map_err(|e| format!("engine build ({suite}): {e}"))
}

/// The engine wire request for one case: state serialized with the
/// Python-JSON law (the SAME bytes the laya lane sequences), prompt =
/// instructions, options from the criteria structure, criteria text None
/// (the modelless context is state + prompt, its serving path).
fn engine_request(
    case: &SuiteCase,
    state_str: &str,
) -> Result<katgpt_core::decision_wire::DecisionRequest, String> {
    use katgpt_core::decision_wire::Question;
    let mut questions = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let q = match q.kind {
            QKind::Choice => {
                let keys: Vec<String> = q
                    .criteria
                    .as_object()
                    .map(|m| m.keys().cloned().collect())
                    .ok_or_else(|| format!("case {}: choice criteria must be an object", q.qid))?;
                Question::choice(q.qid.as_str(), q.instructions.as_str(), keys, None)
            }
            QKind::Score => {
                let levels: Vec<String> = q
                    .criteria
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect()
                    })
                    .ok_or_else(|| format!("case {}: score criteria must be an array", q.qid))?;
                Question::score(q.qid.as_str(), q.instructions.as_str(), levels)
            }
            QKind::Noul => Question::noul(q.qid.as_str(), q.instructions.as_str()),
        };
        questions.push(q);
    }
    Ok(katgpt_core::decision_wire::DecisionRequest {
        state: state_str.to_string(),
        questions,
    })
}

// ── evaluation record ───────────────────────────────────────────────────

/// A lane evaluation over one case set: per case, per question probabilities
/// in LABEL space ([p_no, p_yes] for noul — gold idx 1 = true), forced picks
/// in label space, readout confidences, abstain flags.
struct Eval {
    probs: Vec<Vec<Vec<f64>>>,
    picks: Vec<Vec<usize>>,
    confs: Vec<Vec<f64>>,
    abstained: Vec<Vec<bool>>,
}

impl Eval {
    fn n_questions(&self) -> usize {
        self.probs.iter().map(|c| c.len()).sum()
    }

    // The cases are `AsRef<SuiteCase>` so the SAME methods serve the
    // modelless lane (a full `&[SuiteCase]`) and the laya ANE lane (the
    // SERVED subset as `&[&SuiteCase]` — bucket-skipped cases contribute
    // no vectors, so the walk must stay index-aligned with them).
    fn forced_rows(&self, cases: &[impl AsRef<SuiteCase>]) -> Vec<(usize, Vec<f64>)> {
        let mut rows = Vec::new();
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, _q) in case.questions.iter().enumerate() {
                rows.push((case.gold[qi].idx, self.probs[ci][qi].clone()));
            }
        }
        rows
    }

    fn readout_pairs(&self, cases: &[impl AsRef<SuiteCase>]) -> Vec<(f64, bool)> {
        let mut pairs = Vec::new();
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, _q) in case.questions.iter().enumerate() {
                pairs.push((self.confs[ci][qi], self.picks[ci][qi] == case.gold[qi].idx));
            }
        }
        pairs
    }

    fn selective(&self, cases: &[impl AsRef<SuiteCase>]) -> SelectiveMetrics {
        let mut n = 0usize;
        let mut correct = 0usize;
        let mut total = 0usize;
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, _q) in case.questions.iter().enumerate() {
                total += 1;
                if !self.abstained[ci][qi] {
                    n += 1;
                    if self.picks[ci][qi] == case.gold[qi].idx {
                        correct += 1;
                    }
                }
            }
        }
        SelectiveMetrics {
            abstain_rate: if total > 0 {
                (total - n) as f64 / total as f64
            } else {
                0.0
            },
            selective_accuracy: if n > 0 {
                correct as f64 / n as f64
            } else {
                0.0
            },
            selective_n: n,
        }
    }
}

struct Latency {
    p50_ms: f64,
    p99_ms: f64,
    tail_support: usize,
    extremes: Option<LatencyExtremes>,
    determinism_ok: Option<bool>,
}

/// Nearest-rank percentiles over µs samples; returns (p50, p99, tail support
/// of p99) — the tail-support figure prints beside p99 in the tables (the
/// repo-family percentile law).
fn percentile_us(durs: &[u64]) -> (u64, u64, usize) {
    let mut d = durs.to_vec();
    d.sort_unstable();
    let n = d.len();
    if n == 0 {
        return (0, 0, 0);
    }
    let p50 = d[n / 2];
    let idx99 = (n * 99).div_ceil(100) - 1;
    (p50, d[idx99], n - idx99)
}

fn argmax(p: &[f64]) -> usize {
    let mut best = 0usize;
    for (i, v) in p.iter().enumerate() {
        if *v > p[best] {
            best = i;
        }
    }
    best
}

// ── the modelless lane ──────────────────────────────────────────────────

fn eval_engine<const N: usize>(
    engine: &mut DecisionEngine<N, EMBED_DIM>,
    cases: &[SuiteCase],
    state_strs: &[String],
    check_determinism: bool,
) -> Result<(Eval, Latency), String> {
    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut abstained = Vec::with_capacity(cases.len());
    let max_q = cases.iter().map(|c| c.questions.len()).max().unwrap_or(1);
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    sc.prepare(max_q);
    let mut durs_us: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = if check_determinism { Some(true) } else { None };

    for (ci, case) in cases.iter().enumerate() {
        let req = engine_request(case, &state_strs[ci])?;
        let t0 = Instant::now();
        let resp = engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("engine decide ({}, case {ci}): {e}", case.id))?;
        durs_us.push(t0.elapsed().as_micros() as u64);

        if check_determinism && ci < 10 {
            let resp2 = engine
                .decide_with(&req, &mut sc)
                .map_err(|e| format!("engine determinism rerun ({}, case {ci}): {e}", case.id))?;
            let j1 = serde_json::to_string(&resp.answers).unwrap_or_default();
            let j2 = serde_json::to_string(&resp2.answers).unwrap_or_default();
            if j1 != j2 {
                *determinism_ok.get_or_insert(true) = false;
            }
            drop(resp2);
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        let mut cabst = Vec::with_capacity(case.questions.len());
        for (q, ans) in case.questions.iter().zip(resp.answers.iter()) {
            // Engine internal noul order is [yes, no] (wire p_yes only);
            // flip to [no, yes] to match the gold-index convention.
            let p: Vec<f64> = if q.kind == QKind::Noul {
                let p_yes = f64::from(ans.probabilities[0]);
                vec![1.0 - p_yes, p_yes]
            } else {
                ans.probabilities.iter().map(|p| f64::from(*p)).collect()
            };
            let pick = match &ans.outcome {
                Some(katgpt_core::decision_wire::Outcome::Choice { index }) => *index as usize,
                Some(katgpt_core::decision_wire::Outcome::Score { level }) => *level as usize,
                Some(katgpt_core::decision_wire::Outcome::Noul { yes }) => usize::from(*yes),
                None => argmax(&p), // abstained — forced pick for the metrics
            };
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(f64::from(ans.confidence));
            cabst.push(ans.outcome.is_none());
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
        abstained.push(cabst);
    }
    let (p50, p99, support) = percentile_us(&durs_us);
    Ok((
        Eval {
            probs,
            picks,
            confs,
            abstained,
        },
        Latency {
            p50_ms: p50 as f64 / 1000.0,
            p99_ms: p99 as f64 / 1000.0,
            tail_support: support,
            extremes: LatencyExtremes::of(&durs_us, 1000.0),
            determinism_ok,
        },
    ))
}

// ── per-suite modelless result ──────────────────────────────────────────

/// Gold answer per question for a case, restricted to what the lane tables
/// read (idx / soft / score).
struct Gold<'a> {
    idx: usize,
    soft: &'a [f64],
    score: Option<f64>,
}

/// Inputs for one modelless lane run (bundled to keep the runner's
/// argument list short; all borrowed).
struct ModellessInput<'a> {
    spec: &'a SuiteSpec,
    suite: &'a Suite,
    train: &'a [TrainDoc],
    state_strs: &'a [String],
    cal_cases: &'a [SuiteCase],
    cal_state_strs: &'a [String],
    labels: &'a [String],
    want_by_type: bool,
    /// The EFFECTIVE per-label corpus cap (registry default or the
    /// --corpus-cap override — Issue 013 lever 1's instrument).
    corpus_cap_per_label: usize,
    /// The cap's base source when no cal-slice selection ran
    /// ("registry" or "--corpus-cap override").
    cap_source_base: &'static str,
    /// Cal-slice cap-selection candidates (empty = off — run() enforces
    /// mutual exclusion with the override).
    cal_select_caps: &'a [usize],
    /// The suite's raw train envelope (the selection stratification reads
    /// the rows it buckets; `train` alone has lost the row structure).
    train_rows: &'a Value,
    /// Run the Issue-013 lever-3 pair-head A/B arm (report-only; pairs are
    /// armed from CAL-slice confusion, heads fitted from corpus docs).
    pair_head_ab: bool,
    /// Issue 024 T3: one leak flag per eval case (true = the case has an
    /// exact/near twin in the corpus∪cal side — drop from acc_deleaked).
    /// None = the `slice_leak` feature is off or the suite is out of scope.
    leak_flags: Option<&'a [bool]>,
}

/// The cal-slice cap-selection measurement (Issue 013 lever-1 protocol
/// plumb): a label-STRATIFIED selection slice over the pool region (the
/// registry cal slice is label-clustered — the mirrors store rows
/// label-grouped — and cal accuracy on it reads chance-level, which cannot
/// rank caps), each candidate measured on corpora that EXCLUDE the
/// selection docs by content (a selection case must never score against
/// its own text). The suite's own row is NOT affected: it runs the full
/// registry pool at the selected cap, exactly the published posture.
struct CapSelection {
    selected: usize,
    rows: Vec<CapCandidate>,
}

fn build_selection_measurement<const N: usize>(
    inp: &ModellessInput<'_>,
) -> Result<CapSelection, String> {
    let spec = inp.spec;
    let pool_from = spec.cal_cap.min(inp.train.len());
    let slices = stratified_selection_slices(
        inp.train_rows,
        spec.name,
        inp.labels,
        pool_from,
        spec.cal_cap,
    );
    if slices.n_picks == 0 {
        return Err(format!(
            "{}: cap selection produced an empty stratified slice — the train rows \
             beyond the cal prefix carry none of the engine's labels",
            spec.name
        ));
    }
    // The selection cases: the suite's own builder over the permuted
    // envelope (universe from ALL rows — identical option set to the
    // registry cal slice), max_rows = the front length.
    let sel_suite = (spec.build)(&slices.permuted, slices.n_picks);
    let sel_state_strs: Vec<String> = sel_suite
        .cases
        .iter()
        .map(|c| serialize_state(&c.state))
        .collect();
    // The selection docs (same train_docs rule) drive the content
    // exclusion: pool minus these texts.
    let sel_docs = train_docs(&slices.front, spec.name);
    let excluded: HashSet<&str> = sel_docs.iter().map(|d| d.text.as_str()).collect();
    let pool: Vec<TrainDoc> = inp.train[pool_from..]
        .iter()
        .filter(|d| !excluded.contains(d.text.as_str()))
        .cloned()
        .collect();
    eprintln!(
        "    cap-selection: stratified slice {} case(s) over {} label(s); selection corpora \
         pool {} → {} doc(s) (selection docs excluded — the suite's own row below runs the \
         FULL registry pool)",
        sel_suite.cases.len(),
        inp.labels.len(),
        inp.train.len() - pool_from,
        pool.len()
    );

    let mut cands = inp.cal_select_caps.to_vec();
    cands.push(spec.corpus_cap_per_label);
    cands.sort_unstable();
    cands.dedup();
    let mut rows = Vec::with_capacity(cands.len());
    for &cap in &cands {
        let mut engine =
            build_engine::<N>(spec.name, &pool, inp.labels, cap, EngineConfig::default())?;
        let (ev, _) = eval_engine(&mut engine, &sel_suite.cases, &sel_state_strs, false)?;
        let cal_acc = hard_metrics(&ev.forced_rows(&sel_suite.cases)).accuracy;
        rows.push(CapCandidate { cap, cal_acc });
        eprintln!("    cap-selection: cap {cap} → sel-slice acc {cal_acc:.4}");
    }
    let selected = select_cap(&rows, spec.corpus_cap_per_label);
    eprintln!(
        "    cap-selection: selected {selected} (registry default {}) — the test row below \
         is the single test read, at the FULL registry pool",
        spec.corpus_cap_per_label
    );
    Ok(CapSelection { selected, rows })
}

#[allow(clippy::too_many_lines)]
fn run_modelless<const N: usize>(inp: &ModellessInput<'_>) -> Result<LaneResult, String> {
    let spec = inp.spec;
    let suite = inp.suite;
    let train = inp.train;
    let state_strs = inp.state_strs;
    let cal_cases = inp.cal_cases;
    let cal_state_strs = inp.cal_state_strs;
    let labels = inp.labels;
    let want_by_type = inp.want_by_type;
    let t_start = Instant::now();
    let default_cfg = EngineConfig::default();

    // ── Cal-slice cap selection (Issue 013 lever-1 protocol plumb). The
    // cap is picked ONLY on the stratified selection slice — the test
    // split is read once, at the selected cap, further down — closing the
    // protocol hole that deferred the banking77 promotion (a cap picked on
    // test would be a test-set-selected hyperparameter). Self-corpora
    // suites (synthetic families, code_fixtures) ignore the caps by
    // construction; the source string below discloses the n/a instead of
    // silently running or silently dropping the ask.
    let selection = if inp.cal_select_caps.is_empty()
        || spec.synthetic.is_some()
        || spec.corpus_cap_per_label == usize::MAX
    {
        None
    } else if cal_cases.is_empty() {
        return Err("cal-slice cap selection needs a cal slice (suite cal_cap > 0)".to_string());
    } else {
        Some(build_selection_measurement::<N>(inp)?)
    };
    let effective_cap = selection
        .as_ref()
        .map_or(inp.corpus_cap_per_label, |s| s.selected);

    // Corpus pool = the train docs AFTER the calibration slice — the cal
    // cases must not be members of their own reference corpora (a cal case
    // scoring cos 1.0 against ITSELF inflates every cal-slice quantile and
    // over-arms the gates on test — measured: 64→100% abstain drift on
    // ag_news before this split). The code_fixtures suite self-splits
    // (spec.cal_cap = 0 → the pool is its whole generated corpus, which is
    // already disjoint by construction).

    // ── Fused-gate threshold fitting (the engine's own law: thresholds from
    // measured geometry, never magic numbers). The birth constants (0.35 /
    // 0.5) were measured on the birth corpus and do NOT transfer to these
    // suites — measured: with the defaults the distance gate abstains 100%
    // on several suites. Fit BOTH thresholds at the cal-slice 30th
    // percentile (the T1.6 arena posture, target abstain rate ρ = 30%):
    // 30% of in-corpus-distribution questions abstain, the rest pass. The
    // fitted values ride the result row.
    let (score_threshold, distance_threshold, threshold_recommendation) = {
        let corpus_pool: &[TrainDoc] = train.get(spec.cal_cap.min(train.len())..).unwrap_or(train);
        let mut probe = build_engine::<N>(
            spec.name,
            corpus_pool,
            labels,
            effective_cap,
            default_cfg.clone(),
        )?;
        let mut sc: Scratch<EMBED_DIM> = Scratch::new();
        let max_cal_q = cal_cases
            .iter()
            .map(|c| c.questions.len())
            .max()
            .unwrap_or(1);
        sc.prepare(max_cal_q);
        // Labeled cal-slice observations for the threshold-recommendation
        // surface (Issue 009): score axis = the engine readout confidence,
        // distance axis = the corpus-distance gate's abstain confidence;
        // correct = the probe's own pick vs gold on the case's LAST
        // question (the eval_engine convention, incl. the Noul [no, yes]
        // wire flip). The percentile posture selects on score only — the
        // label feeds the accuracy DISCLOSURE, never the threshold, so
        // the T2 migration stays byte-identical.
        let mut score_obs: Vec<GateObservation> = Vec::new();
        let mut distance_obs: Vec<GateObservation> = Vec::new();
        for (ci, case) in cal_cases.iter().enumerate() {
            let req = engine_request(case, &cal_state_strs[ci])?;
            probe
                .solve_into(&req, &mut sc)
                .map_err(|e| format!("cal probe ({}, case {ci}): {e}", case.id))?;
            // The gate q survives only for the LAST question of a solve —
            // sample the last slot per case (n = #cal cases, enough for a
            // quantile at every suite's cal_cap).
            if let (Some(slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) {
                let pick = match case.questions.last().map(|q| q.kind) {
                    // Engine internal noul order is [yes, no]; gold uses
                    // the wire [no, yes] convention (eval_engine's flip).
                    Some(QKind::Noul) => (1 - slot.pick) as usize,
                    _ => slot.pick as usize,
                };
                let correct = case.gold.last().is_some_and(|g| g.idx == pick);
                score_obs.push(GateObservation {
                    score: slot.confidence,
                    correct,
                });
                distance_obs.push(GateObservation {
                    score: probe.gate(*dom).abstain_confidence(&sc.q),
                    correct,
                });
            }
        }
        // The T1.6 arena posture through the engine surface (Issue 009
        // T2): both axes at ρ = 30%. Thin support falls back to the birth
        // constants — THIN_SUPPORT_FLOOR == 16 IS the old
        // `confs.len() < 16` arm, so the fallback semantics are
        // identical, and the percentile law itself is pinned bit-identical
        // to the runner's old `quantile` by the migration gate.
        let rec =
            recommend_fused_gate(&score_obs, &distance_obs, Posture::Percentile { rho: 0.30 });
        (
            rec.score
                .map_or(default_cfg.score_threshold, |r| r.threshold),
            rec.distance
                .map_or(default_cfg.distance_threshold, |r| r.threshold),
            rec,
        )
    };
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };

    // Corpus corpora: eval-cap docs per label, from the SAME pool the
    // thresholds were fitted on (post-cal-slice train docs).
    let corpus_pool: &[TrainDoc] = train.get(spec.cal_cap.min(train.len())..).unwrap_or(train);
    let mut raw_engine =
        build_engine::<N>(spec.name, corpus_pool, labels, effective_cap, cfg.clone())?;

    // Calibration pairs from a RAW (uncalibrated) engine over the cal slice.
    let mut cal_engine =
        build_engine::<N>(spec.name, corpus_pool, labels, effective_cap, cfg.clone())?;
    let cal_cases: Vec<SuiteCase> = if cal_cases.is_empty() {
        Vec::new()
    } else {
        cal_cases.to_vec()
    };
    let (cal_eval, _) = if cal_cases.is_empty() {
        // code_fixtures carries its own cal slice; empty here means no cal.
        (
            Eval {
                probs: vec![],
                picks: vec![],
                confs: vec![],
                abstained: vec![],
            },
            Latency {
                p50_ms: 0.0,
                p99_ms: 0.0,
                tail_support: 0,
                extremes: None,
                determinism_ok: None,
            },
        )
    } else {
        eval_engine(&mut cal_engine, &cal_cases, cal_state_strs, false)?
    };
    let mut cal_pairs: Vec<CalibrationPair> = Vec::new();
    for (ci, case) in cal_cases.iter().enumerate() {
        for (qi, _q) in case.questions.iter().enumerate() {
            cal_pairs.push(CalibrationPair {
                conf: cal_eval.confs[ci][qi],
                correct: cal_eval.picks[ci][qi] == case.gold[qi].idx,
            });
        }
    }

    // RAW eval over the test cases (uncalibrated readout + raw abstain).
    let (raw_eval, lat) = eval_engine(&mut raw_engine, &suite.cases, state_strs, true)?;

    // CALIBRATED engine: fit on the cal pairs, then re-eval the test cases.
    let mut fitted = build_engine::<N>(spec.name, corpus_pool, labels, effective_cap, cfg)?;
    let mut moved = false;
    for p in &cal_pairs {
        moved |= fitted.observe(p.conf as f32, p.correct);
    }
    let (cal_eval_test, _) = eval_engine(&mut fitted, &suite.cases, state_strs, false)?;

    // ── metrics assembly (label space; noul [no, yes]) ──
    let forced = raw_eval.forced_rows(&suite.cases);
    let hard = hard_metrics(&forced);
    // Issue 024 T3: the same walk restricted to the unflagged cases. None
    // (feature off / out of scope) stays an absence in results.json.
    let acc_deleaked = inp.leak_flags.and_then(|keep| {
        let per_case: Vec<usize> = suite.cases.iter().map(|c| c.questions.len()).collect();
        subset_accuracy(&forced, keep, &per_case)
    });
    let readout_pairs_raw = raw_eval.readout_pairs(&suite.cases);
    let readout_pairs_cal = cal_eval_test.readout_pairs(&suite.cases);
    let readout_ece_raw = ece_of(&readout_pairs_raw);
    let readout_ece_cal = ece_of(&readout_pairs_cal);
    let raw_abstain = raw_eval.selective(&suite.cases);
    let calibrated_abstain = cal_eval_test.selective(&suite.cases);

    // The conformal-naive floor: split-conformal recalibration of the raw
    // readout confidences (G1's Report-the-Floor comparison).
    let test_confs: Vec<f64> = readout_pairs_raw.iter().map(|(c, _)| *c).collect();
    let floored: Vec<f64> = conformal_naive_floor(&cal_pairs, &test_confs);
    let floor_pairs: Vec<(f64, bool)> = floored
        .into_iter()
        .zip(readout_pairs_raw.iter().map(|(_, ok)| *ok))
        .collect();
    let floor_ece = ece_of(&floor_pairs);

    // The verdict is derived in metrics (pure, known-answer-tested): the
    // calibrator-never-fitted state is NO CLAIM — the meta's
    // calibration_protocol line has promised exactly that since the lane
    // landed; this makes the code keep the promise.
    let (g1_pass, g1_verdict) = g1_verdict_of(
        moved,
        cal_pairs.len(),
        readout_ece_cal,
        readout_ece_raw,
        floor_ece,
    );

    // Optional per-type + soft/score metrics.
    let mut by_question_type: Option<BTreeMap<String, HardMetrics>> = None;
    let mut soft_acc: Option<f64> = None;
    let mut brier_soft: Option<f64> = None;
    let mut score_mae: Option<f64> = None;
    let mut within_1: Option<f64> = None;

    if want_by_type {
        let mut buckets: BTreeMap<String, Vec<(usize, Vec<f64>)>> = BTreeMap::new();
        let mut soft_sum = 0.0f64;
        let mut soft_n = 0usize;
        let mut mae_sum = 0.0f64;
        let mut w1_sum = 0.0f64;
        let mut score_n = 0usize;
        for (ci, case) in suite.cases.iter().enumerate() {
            for (qi, q) in case.questions.iter().enumerate() {
                let g = Gold {
                    idx: case.gold[qi].idx,
                    soft: &case.gold[qi].soft,
                    score: case.gold[qi].gold_score,
                };
                let probs = &raw_eval.probs[ci][qi];
                buckets
                    .entry(q.kind.as_str().to_string())
                    .or_default()
                    .push((g.idx, probs.clone()));
                if let Some(sm) = soft_metrics(probs, g.soft) {
                    soft_sum += sm.soft_acc;
                    soft_n += 1;
                }
                if let Some(gs) = g.score {
                    let sm = score_metrics(probs, gs);
                    mae_sum += sm.mae;
                    w1_sum += sm.within_1;
                    score_n += 1;
                }
            }
        }
        by_question_type = Some(
            buckets
                .into_iter()
                .map(|(k, rows)| (k, hard_metrics(&rows)))
                .collect(),
        );
        if soft_n > 0 {
            soft_acc = Some(soft_sum / soft_n as f64);
        }
        if score_n > 0 {
            score_mae = Some(mae_sum / score_n as f64);
            within_1 = Some(w1_sum / score_n as f64);
        }
    } else {
        // sst5: score metrics over its single score question per case.
        let mut mae_sum = 0.0f64;
        let mut w1_sum = 0.0f64;
        let mut score_n = 0usize;
        for (ci, case) in suite.cases.iter().enumerate() {
            for (qi, q) in case.questions.iter().enumerate() {
                if q.kind == QKind::Score
                    && let Some(gs) = case.gold[qi].gold_score
                {
                    let sm = score_metrics(&raw_eval.probs[ci][qi], gs);
                    mae_sum += sm.mae;
                    w1_sum += sm.within_1;
                    score_n += 1;
                }
            }
        }
        if score_n > 0 {
            score_mae = Some(mae_sum / score_n as f64);
            within_1 = Some(w1_sum / score_n as f64);
        }
    }

    // Brier soft for typed_decisions: mean of per-question brier_soft.
    if spec.name == "typed_decisions" {
        let mut bsum = 0.0f64;
        let mut bn = 0usize;
        for (ci, case) in suite.cases.iter().enumerate() {
            for (qi, _q) in case.questions.iter().enumerate() {
                if let Some(sm) = soft_metrics(&raw_eval.probs[ci][qi], &case.gold[qi].soft) {
                    bsum += sm.brier_soft;
                    bn += 1;
                }
            }
        }
        if bn > 0 {
            brier_soft = Some(bsum / bn as f64);
        }
        if soft_acc.is_none() {
            let (mut s, mut n) = (0.0f64, 0usize);
            for (ci, case) in suite.cases.iter().enumerate() {
                for (qi, _q) in case.questions.iter().enumerate() {
                    if let Some(sm) = soft_metrics(&raw_eval.probs[ci][qi], &case.gold[qi].soft) {
                        s += sm.soft_acc;
                        n += 1;
                    }
                }
            }
            if n > 0 {
                soft_acc = Some(s / n as f64);
            }
        }
    }

    let cap_source = if selection.is_some() {
        "cal-slice selection (stratified slice; corpora exclude the selection docs)"
    } else if spec.synthetic.is_some() || spec.corpus_cap_per_label == usize::MAX {
        "registry (selection n/a: self-corpora)"
    } else {
        inp.cap_source_base
    };

    Ok(LaneResult {
        lane: "modelless",
        model: "modelless".to_string(),
        hard,
        readout_ece: Some(readout_ece_cal),
        raw_abstain: Some(raw_abstain),
        calibrated_abstain: Some(calibrated_abstain),
        readout_ece_raw: Some(readout_ece_raw),
        readout_ece_calibrated: Some(readout_ece_cal),
        floor_ece: Some(floor_ece),
        g1_pass,
        g1_verdict: Some(g1_verdict),
        by_question_type,
        soft_acc,
        brier_soft,
        score_mae,
        within_1,
        latency_p50_ms: lat.p50_ms,
        latency_p99_ms: lat.p99_ms,
        latency_tail_support: lat.tail_support,
        acc_deleaked,
        latency_extremes: lat.extremes,
        determinism_ok: lat.determinism_ok,
        seconds: t_start.elapsed().as_secs_f64(),
        n_cases: suite.cases.len(),
        n_questions: raw_eval.n_questions(),
        score_threshold,
        distance_threshold,
        threshold_recommendation: Some(threshold_recommendation),
        corpus_cap: Some(CorpusCapInfo {
            effective: effective_cap,
            source: cap_source.to_string(),
            selection: selection.map(|s| s.rows),
        }),
        confusion: Some(confusion_rows(&raw_eval, &suite.cases, CONFUSION_TOP)),
        pair_head_ab: if inp.pair_head_ab {
            let (top2_ab, pred_ab) = pair_head_ab_pass(
                &raw_eval,
                suite,
                state_strs,
                &cal_eval,
                &cal_cases,
                train,
                labels,
                effective_cap,
                &crate::embed::Embedder,
            )?;
            Some((top2_ab, pred_ab))
        } else {
            None
        },
    })
}

// ── the laya lane (the riir-owned backend since .issues/006 — the candle
// reference lane was deleted; `RiirAgent` serves the same envelope surface
// `system_one`, G5-proven against the same frozen captures) ────────────────

/// The checkpoint's agent at the ENV-selected posture. `LAYA_DEVICE=ane`
/// (feature `laya-riir-ane`) routes to `RiirAgent::load_ane` with the
/// artifact root from `LAYA_ANE_ARTIFACTS_DIR` (else `assets/ane/`) — the
/// HARNESS is the explicit consumer choosing the constructor from the env
/// (the substrate's own plain loader still refuses env-only ANE; an env
/// value can never silently demote an explicitly requested lane). A
/// missing artifact tree errors LOUD naming the remedy, never a CPU
/// number wearing an ANE label.
/// `target_os = "macos"` — the SAME scope the substrate gates
/// `RiirAgent::load_ane` with (Core ML artifacts); elsewhere this variant
/// compiles to nothing and the plain fallback below serves (its substrate
/// loader refuses an env `ane` LOUD — an ANE number can never silently
/// wear a CPU label either way).
#[cfg(all(
    feature = "laya-riir",
    feature = "laya-riir-ane",
    target_os = "macos"
))]
fn load_laya_agent(
    ckpt: &str,
    ck: crate::laya::config::Checkpoint,
) -> Result<crate::laya::riir::RiirAgent, String> {
    use crate::laya::riir::agent::DeviceKind;
    use crate::laya::weights::weights_root;
    use std::path::PathBuf;
    if DeviceKind::from_env().map_err(|e| e.to_string())? == DeviceKind::Ane {
        let ane_root = std::env::var_os("LAYA_ANE_ARTIFACTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("assets/ane"));
        let manifest = ane_root.join("manifest.json");
        if !manifest.exists() {
            return Err(format!(
                "LAYA_DEVICE=ane: no manifest at {} — run scripts/ane_convert.py first \
                 (artifacts are local-only) or set LAYA_ANE_ARTIFACTS_DIR",
                manifest.display()
            ));
        }
        return crate::laya::riir::RiirAgent::load_ane(&weights_root(), ck, &ane_root, &manifest)
            .map_err(|e| format!("laya ane load ({ckpt}): {e}"));
        }
        crate::laya::riir::RiirAgent::load(&weights_root(), ck)
            .map_err(|e| format!("laya load ({ckpt}): {e}"))
    }

/// The no-ANE-build form: plain load (an env `ane` value is refused loud
/// by the substrate — fail loud, never a silent fallback). Also the form
/// on a non-macOS box with the `laya-riir-ane` feature ON: the substrate's
/// `load_ane` is macOS-scoped (Core ML), so the plain loader serves and
/// refuses the env selection loud — the same fail-loud law.
#[cfg(all(
    feature = "laya-riir",
    not(all(feature = "laya-riir-ane", target_os = "macos"))
))]
fn load_laya_agent(
    ckpt: &str,
    ck: crate::laya::config::Checkpoint,
) -> Result<crate::laya::riir::RiirAgent, String> {
    use crate::laya::riir::RiirAgent;
    use crate::laya::weights::weights_root;
    RiirAgent::load(&weights_root(), ck).map_err(|e| format!("laya load ({ckpt}): {e}"))
}

#[cfg(feature = "laya-riir")]
fn run_laya_checkpoint(
    suite: &Suite,
    ckpt: &'static str,
    laya_max_questions: usize,
    // Issue 024 T3: one leak flag per suite.cases entry (the caller's full
    // eval scan). None = feature off / suite out of scope.
    leak_flags: Option<&[bool]>,
) -> Result<(LaneResult, Vec<String>), String> {
    use crate::laya::config::Checkpoint;

    let t_start = Instant::now();
    let ck = match ckpt {
        "english" => Checkpoint::English,
        "multilingual" => Checkpoint::Multilingual,
        "typed" => Checkpoint::TypedDecisions,
        other => return Err(format!("unknown checkpoint {other}")),
    };
    let agent = load_laya_agent(ckpt, ck)?;

    // Cap the eval cases when asked (runtime trim; documented in the table
    // when used).
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // GPU pre-ramp (Issue 020): the first measured case after a fresh load
    // pays the Metal pipeline compile (the python oracle pays its MPS graph
    // compile the same way), which is the recorded cold-first-suite
    // signature — the process's first suite inflated across runs. One
    // unmeasured warmup case per (suite, checkpoint) load compiles the
    // exact pipelines the timed loop then uses. `LAYA_HARNESS_NO_WARMUP=1`
    // restores the cold posture (the measurement, not the default).
    if laya_gpu_preramp_enabled()
        && let Some(first) = cases.first()
    {
        let warm_q = case_questions(first);
        match agent.system_one(&first.state, &warm_q) {
            Ok(_) => eprintln!("  [laya {ckpt}] gpu pre-ramp: 1 unmeasured warmup case"),
            Err(crate::laya::LayaError::Bucket { .. }) => {}
            Err(e) => return Err(format!("laya warmup ({ckpt}): {e}")),
        }
    }

    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // The ANE lane's bucket refusals are a coverage LIMIT, not a compute
    // failure: the case is skipped LOUDLY (named + counted into the suite's
    // disclosure) and the lane continues with the servable cases. Real
    // failures still abort the lane (the ?-path below is untouched).
    // `served_cases` is the slice the metrics tail walks — Eval's vectors
    // are per-SERVED-case, so the assembler must never see the full set.
    let mut served_cases: Vec<&SuiteCase> = Vec::with_capacity(cases.len());
    let mut served_orig: Vec<usize> = Vec::with_capacity(cases.len());
    let mut bucket_skipped_cases: Vec<&str> = Vec::new();

    for (ci, case) in cases.iter().enumerate() {
        let questions = case_questions(case);
        let t0 = Instant::now();
        let answers = match agent.system_one(&case.state, &questions) {
            Ok(a) => a,
            Err(crate::laya::LayaError::Bucket { seq, max, .. }) => {
                bucket_skipped_cases.push(&case.id);
                eprintln!(
                    "  [laya {ckpt}] case {} skipped: seq {seq} > max ANE bucket {max} \
                     (named + counted, not a failure)",
                    case.id
                );
                continue;
            }
            Err(e) => return Err(format!("laya forward ({}, case {ci}): {e}", case.id)),
        };
        durs_ms.push(t0.elapsed().as_millis() as u64);

        if ci < 10 {
            let answers2 = agent
                .system_one(&case.state, &questions)
                .map_err(|e| format!("laya determinism rerun (case {ci}): {e}"))?;
            let render = |as_: &[crate::laya::types::Answer]| {
                as_.iter()
                    .map(|a| {
                        format!(
                            "{}|{}|{:.6}|{:.6}|{:.6}",
                            a.choice.as_deref().unwrap_or(""),
                            a.score.unwrap_or(-1.0),
                            a.noul.unwrap_or(-1.0),
                            a.confidence,
                            a.act_probability
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";")
            };
            if render(&answers) != render(&answers2) {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(questions.len());
        let mut cpicks = Vec::with_capacity(questions.len());
        let mut cconfs = Vec::with_capacity(questions.len());
        for (q, ans) in case.questions.iter().zip(answers.iter()) {
            let (p, pick) = match q.kind {
                QKind::Choice => {
                    let keys: Vec<String> = q
                        .criteria
                        .as_object()
                        .map(|m| m.keys().cloned().collect())
                        .unwrap_or_default();
                    let probs: Vec<f64> = ans.probabilities.iter().map(|(_, v)| *v).collect();
                    let idx = ans
                        .choice
                        .as_ref()
                        .and_then(|c| keys.iter().position(|k| k == c))
                        .unwrap_or_else(|| {
                            probs
                                .iter()
                                .enumerate()
                                .max_by(|a, b| a.1.total_cmp(b.1))
                                .map_or(0, |(i, _)| i)
                        });
                    (probs, idx)
                }
                QKind::Score => {
                    let probs: Vec<f64> = ans.probabilities.iter().map(|(_, v)| *v).collect();
                    let pick = probs
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .map_or(0, |(i, _)| i);
                    (probs, pick)
                }
                QKind::Noul => {
                    let p_true = ans.noul.unwrap_or(0.5);
                    (vec![1.0 - p_true, p_true], usize::from(p_true >= 0.5))
                }
            };
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(ans.confidence);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
        served_cases.push(case);
        // The case walk is over `cases` — a PREFIX of suite.cases when the
        // question cap trimmed it — so the enumerate index IS the
        // suite.cases index the leak flags are keyed on.
        served_orig.push(ci);
    }

    // Every case over the bucket limit = the suite has NO servable cases:
    // an honest ABSENCE (the caller's errors list), never an empty metrics
    // row (the metrics tail indexes per-served-case vectors). Only the ANE
    // lane can produce skips, so the bucket-max read is gated on the SAME
    // cfg the agent's `ane_bucket_max()` carries (macos + feature) — a
    // narrower gate here would fail to compile on the non-macos ane arm.
    if probs.is_empty() {
        #[cfg(all(target_os = "macos", feature = "laya-riir-ane"))]
        let max_bucket = agent.ane_bucket_max().unwrap_or(0);
        #[cfg(not(all(target_os = "macos", feature = "laya-riir-ane")))]
        let max_bucket = 0;
        return Err(format!(
            "all {} case(s) exceed the ANE buckets (max {}) — no servable cases; \
             this device row is absent for this suite (named, not fabricated)",
            cases.len(), max_bucket,
        ));
    }

    // Latency percentiles are computed ONCE in the shared tail
    // (assemble_laya_lane_result) — both lanes' metrics must not diverge.
    // The leak flags are remapped through the bucket skips so the served
    // slice and the keep-mask stay index-aligned (subset_accuracy asserts
    // exactly that).
    let served_flags: Option<Vec<bool>> = leak_flags
        .map(|f| served_orig.iter().map(|&i| f[i]).collect());
    let result = assemble_laya_lane_result(
        "laya-riir",
        ckpt,
        suite.name,
        served_cases.as_slice(),
        served_flags.as_deref(),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        t_start.elapsed().as_secs_f64(),
    );
    // The bucket-skip disclosure rides back to the caller (who pushes it
    // into the absences section — the named-not-silent channel). `n_cases`
    // / `n_questions` already count the SERVED set only, so the served-vs-
    // corpus delta is visible in the table row itself.
    let notes = if bucket_skipped_cases.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "{} (laya/{ckpt}): {} case(s) skipped over the ANE bucket limit — {:?} \
             (a coverage limit, not a failure; the served row is the smaller set)",
            suite.name,
            bucket_skipped_cases.len(),
            bucket_skipped_cases
        )]
    };
    Ok((result, notes))
}

/// Shared tail of BOTH laya backends (the in-process riir backend and the
/// python subprocess oracle): the answer vectors → metrics, the optional
/// typed-decisions extras, and the [`LaneResult`] envelope. The two lanes
/// differ only in HOW answers are produced — the metrics must not.
#[allow(clippy::too_many_arguments)]
fn assemble_laya_lane_result(
    lane: &'static str,
    model: &str,
    suite_name: &str,
    cases: &[impl AsRef<SuiteCase>],
    // Issue 024 T3: one leak flag per SERVED case (aligned with `cases` —
    // the caller maps the full-suite flags through bucket skips / trims).
    // None = feature off / suite out of scope.
    leak_flags: Option<&[bool]>,
    probs: Vec<Vec<Vec<f64>>>,
    picks: Vec<Vec<usize>>,
    confs: Vec<Vec<f64>>,
    durs_ms: Vec<u64>,
    determinism_ok: Option<bool>,
    seconds: f64,
) -> LaneResult {
    let ev = Eval {
        probs,
        picks,
        confs,
        // laya cannot abstain (Research-562 flaw) — the selective metrics
        // report None rather than a fake zero; this field stays empty.
        abstained: Vec::new(),
    };
    let forced = ev.forced_rows(cases);
    let hard = hard_metrics(&forced);
    let acc_deleaked = leak_flags.and_then(|keep| {
        let per_case: Vec<usize> = cases.iter().map(|c| c.as_ref().questions.len()).collect();
        subset_accuracy(&forced, keep, &per_case)
    });
    let readout: Vec<(f64, bool)> = {
        let mut pairs = Vec::new();
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, _q) in case.questions.iter().enumerate() {
                pairs.push((ev.confs[ci][qi], ev.picks[ci][qi] == case.gold[qi].idx));
            }
        }
        pairs
    };

    // Optional extras for typed_decisions.
    let mut by_question_type: Option<BTreeMap<String, HardMetrics>> = None;
    let mut soft_acc: Option<f64> = None;
    let mut brier_soft: Option<f64> = None;
    let mut score_mae: Option<f64> = None;
    let mut within_1: Option<f64> = None;
    if suite_name == "typed_decisions" {
        let mut buckets: BTreeMap<String, Vec<(usize, Vec<f64>)>> = BTreeMap::new();
        let (mut ssum, mut sn, mut bsum) = (0.0f64, 0usize, 0.0f64);
        let (mut msum, mut wsum, mut mn) = (0.0f64, 0.0f64, 0usize);
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, q) in case.questions.iter().enumerate() {
                buckets
                    .entry(q.kind.as_str().to_string())
                    .or_default()
                    .push((case.gold[qi].idx, ev.probs[ci][qi].clone()));
                if let Some(sm) = soft_metrics(&ev.probs[ci][qi], &case.gold[qi].soft) {
                    ssum += sm.soft_acc;
                    bsum += sm.brier_soft;
                    sn += 1;
                }
                if let Some(gs) = case.gold[qi].gold_score {
                    let sm = score_metrics(&ev.probs[ci][qi], gs);
                    msum += sm.mae;
                    wsum += sm.within_1;
                    mn += 1;
                }
            }
        }
        by_question_type = Some(
            buckets
                .into_iter()
                .map(|(k, rows)| (k, hard_metrics(&rows)))
                .collect(),
        );
        if sn > 0 {
            soft_acc = Some(ssum / sn as f64);
            brier_soft = Some(bsum / sn as f64);
        }
        if mn > 0 {
            score_mae = Some(msum / mn as f64);
            within_1 = Some(wsum / mn as f64);
        }
    }

    let (p50, p99, support) = percentile_us(&durs_ms);
    LaneResult {
        lane,
        model: model.to_string(),
        hard,
        readout_ece: Some(ece_of(&readout)),
        raw_abstain: None,
        calibrated_abstain: None,
        readout_ece_raw: None,
        readout_ece_calibrated: None,
        floor_ece: None,
        g1_pass: None,
        g1_verdict: None,
        by_question_type,
        soft_acc,
        brier_soft,
        score_mae,
        within_1,
        latency_p50_ms: p50 as f64,
        latency_p99_ms: p99 as f64,
        latency_tail_support: support,
        latency_extremes: LatencyExtremes::of(&durs_ms, 1.0),
        determinism_ok,
        seconds,
        n_cases: cases.len(),
        n_questions: ev.n_questions(),
        acc_deleaked,
        score_threshold: f32::NAN, // laya exposes no abstain knob — N/A
        distance_threshold: f32::NAN,
        threshold_recommendation: None,
        corpus_cap: None, // the laya lanes have no corpus
        confusion: None,  // the pair probe is the modelless lane's instrument
        pair_head_ab: None,
    }
}

/// The laya-python lane: the ORIGINAL torch reference driven as a JSONL
/// subprocess oracle (measurement-only — the product lane stays the riir
/// backend; the owner's "no Python anywhere" directive governs the shipped
/// binary, not the bench reference, per the probe_orig_laya_latency
/// precedent). SAME cases, SAME answer mapping, SAME metrics tail as the
/// riir lane ([`assemble_laya_lane_result`]) — the only differences are the
/// forward's executor and the reference's own rounded-4 probabilities.
/// Latency is the subprocess round-trip per case (IPC included) and is
/// disclosed as such in the run meta.
fn run_laya_python_checkpoint(
    suite: &Suite,
    ckpt: &str,
    laya_max_questions: usize,
    // Issue 024 T3: one leak flag per suite.cases entry. The served set is
    // a PREFIX of suite.cases (the trim law), so the flags stay aligned.
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{ChildStdin, Command, Stdio};

    let script = std::env::var("LAYA_PYTHON_LANE_SCRIPT")
        .unwrap_or_else(|_| "scripts/laya_python_lane.py".to_string());
    if !Path::new(&script).is_file() {
        return Err(format!(
            "oracle script not found at {script} — run from the repo root or set \
             LAYA_PYTHON_LANE_SCRIPT (the lane is opt-in measurement tooling)"
        ));
    }
    let python = std::env::var("LAYA_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let device = std::env::var("LAYA_PY_DEVICE").unwrap_or_else(|_| "mps".to_string());

    let t_start = Instant::now();
    let mut child = Command::new(&python)
        .arg(&script)
        .arg(ckpt)
        .arg(&device)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // stderr inherits: the reference's load warnings stay visible.
        .spawn()
        .map_err(|e| format!("spawn {python} {script} ({ckpt}): {e}"))?;
    let mut stdin: ChildStdin = child
        .stdin
        .take()
        .ok_or_else(|| "oracle stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "oracle stdout unavailable".to_string())?;
    let mut reader = BufReader::new(stdout);

    let send_case = |stdin: &mut ChildStdin, case: &SuiteCase| -> Result<(), String> {
        let mut qs = Vec::with_capacity(case.questions.len());
        for q in &case.questions {
            let mut def = serde_json::Map::new();
            def.insert("type".into(), Value::String(q.kind.as_str().into()));
            def.insert("instructions".into(), Value::String(q.instructions.clone()));
            if !q.criteria.is_null() {
                def.insert("criteria".into(), q.criteria.clone());
            }
            qs.push(serde_json::json!({"qid": q.qid, "def": Value::Object(def)}));
        }
        let line = serde_json::json!({"state": case.state, "questions": qs});
        writeln!(stdin, "{line}").map_err(|e| format!("oracle stdin ({ckpt}): {e}"))?;
        stdin
            .flush()
            .map_err(|e| format!("oracle stdin flush ({ckpt}): {e}"))
    };

    let read_line = |reader: &mut BufReader<std::process::ChildStdout>,
                     buf: &mut String|
     -> Result<(), String> {
        buf.clear();
        let n = reader
            .read_line(buf)
            .map_err(|e| format!("oracle stdout ({ckpt}): {e}"))?;
        if n == 0 {
            return Err(format!(
                "oracle stream ended early ({ckpt}) — the reference process died; \
                 its stderr above names the cause"
            ));
        }
        Ok(())
    };

    // Handshake: one line once the checkpoint is loaded.
    let mut line = String::new();
    read_line(&mut reader, &mut line)?;
    let ready: Value = serde_json::from_str(line.trim())
        .map_err(|e| format!("oracle handshake ({ckpt}): {e} (got: {})", line.trim()))?;
    if ready.get("ready") != Some(&Value::Bool(true)) {
        return Err(format!(
            "oracle handshake ({ckpt}): expected {{\"ready\":true}}, got {line}"
        ));
    }

    // Cap the eval cases when asked — the SAME trim law as the riir lane.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // GPU pre-ramp (Issue 020), the oracle half: the first measured case
    // pays the reference's MPS graph compile — the same cold-first-suite
    // signature the riir lane's warmup addresses. One unmeasured case per
    // (suite, checkpoint) subprocess; `LAYA_HARNESS_NO_WARMUP=1` restores
    // the cold posture.
    if laya_gpu_preramp_enabled()
        && let Some(first) = cases.first()
    {
        send_case(&mut stdin, first)?;
        let mut line = String::new();
        read_line(&mut reader, &mut line)?;
        eprintln!("  [laya-python {ckpt}] gpu pre-ramp: 1 unmeasured warmup case");
    }

    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        send_case(&mut stdin, case)?;
        let mut line = String::new();
        read_line(&mut reader, &mut line)?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        let resp: Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("oracle response ({ckpt}, case {ci}): {e}"))?;

        if ci < 10 {
            send_case(&mut stdin, case)?;
            let mut line2 = String::new();
            read_line(&mut reader, &mut line2)?;
            if line != line2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        for q in case.questions.iter() {
            let answers = resp
                .get("answers")
                .and_then(|a| a.as_object())
                .ok_or_else(|| format!("oracle response ({ckpt}, case {ci}): no answers object"))?;
            let a = answers.get(&q.qid).ok_or_else(|| {
                format!("oracle response ({ckpt}, case {ci}): missing qid {}", q.qid)
            })?;
            let (p, pick, conf) = parse_python_answer(q, a)
                .map_err(|e| format!("oracle response ({ckpt}, case {ci}): {e}"))?;
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    drop(stdin);
    let _ = child.wait();

    Ok(assemble_laya_lane_result(
        "laya-python",
        ckpt,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        t_start.elapsed().as_secs_f64(),
    ))
}

/// Map one oracle answer object (`{"p": [...], "conf": c, "choice"?,
/// "noul"?}`) to the SAME (probs, pick, confidence) triple the riir lane's
/// answer mapping produces — the metrics must not see a different shape.
pub fn parse_python_answer(
    q: &crate::harness::suites::SuiteQuestion,
    a: &Value,
) -> Result<(Vec<f64>, usize, f64), String> {
    let p: Vec<f64> = a
        .get("p")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| format!("qid {}: missing/invalid p", q.qid))?;
    let conf = a
        .get("conf")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("qid {}: missing/invalid conf", q.qid))?;
    let pick = match q.kind {
        QKind::Choice => {
            let keys: Vec<String> = q
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            a.get("choice")
                .and_then(Value::as_str)
                .and_then(|c| keys.iter().position(|k| k == c))
                .unwrap_or_else(|| {
                    p.iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .map_or(0, |(i, _)| i)
                })
        }
        QKind::Score => p
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i),
        QKind::Noul => {
            if p.len() != 2 {
                return Err(format!("qid {}: noul p must be [1-n, n]", q.qid));
            }
            usize::from(p[1] >= 0.5)
        }
    };
    Ok((p, pick, conf))
}

#[cfg(not(feature = "laya-riir"))]
fn run_laya_checkpoint(
    _suite: &Suite,
    _ckpt: &str,
    _laya_max_questions: usize,
    _leak_flags: Option<&[bool]>,
) -> Result<(LaneResult, Vec<String>), String> {
    Err("laya-riir feature off".to_string())
}

/// GPU pre-ramp switch (Issue 020): default ON; `LAYA_HARNESS_NO_WARMUP=1`
/// is the explicit cold-posture opt-out (only the literal truthy spelling
/// disables — a typo must not silently restore the cold lane).
fn laya_gpu_preramp_enabled() -> bool {
    std::env::var("LAYA_HARNESS_NO_WARMUP").ok().as_deref() != Some("1")
}

/// The wire form of one suite case's questions, shared by the timed loop
/// and the pre-ramp warmup (identical construction, one home).
fn case_questions(case: &SuiteCase) -> Vec<(String, Value)> {
    let mut questions: Vec<(String, Value)> = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let mut def = serde_json::Map::new();
        def.insert("type".into(), Value::String(q.kind.as_str().into()));
        def.insert("instructions".into(), Value::String(q.instructions.clone()));
        if !q.criteria.is_null() {
            def.insert("criteria".into(), q.criteria.clone());
        }
        questions.push((q.qid.clone(), Value::Object(def)));
    }
    questions
}

// ── the CLM comparison lane (Issue 019 T3 / .issues/027) ─────────────
//
// PARITY LAW (what reaches THEIR encoder, byte-for-byte their reference
// protocol — `src/lanes/clm.rs` pins the law copies):
//   * state  = their `to_text(state)` prose (our copy is byte-pinned, so
//     the pre-rendered string IS what their server would render from the
//     raw object — `to_text(str)` is identity on their side);
//   * choice candidates = the criterion DESCRIPTION when one is given,
//     else the key (their `candidates` law: "a candidate reaches the
//     encoder exactly as the caller wrote it");
//   * score candidates  = the rubric level texts;
//   * noul candidates   = their default law (per-side descriptions are
//     inexpressible on our wire — the documented adapter divergence).
// Each lane applies its OWN pinned rendering law over the same case
// content (the laya lane prefixes `key: desc`; CLM embeds the bare
// description — the systems are compared, not their prompt formats).

/// The `/v1/systemone` request for one harness case, built under THEIR
/// rendering law (see the parity note above). The question ORDER and the
/// option ORDER are the harness criteria's insertion order — the same
/// label order every lane scores against.
#[cfg(feature = "clm-lane")]
fn clm_request(case: &SuiteCase) -> Result<katgpt_core::decision_wire::DecisionRequest, String> {
    use crate::lanes::clm;
    use katgpt_core::decision_wire::Question;
    let mut questions = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let question = match q.kind {
            QKind::Choice => {
                let obj = q.criteria.as_object().ok_or_else(|| {
                    format!("case {}: choice criteria must be an object", q.qid)
                })?;
                // THEIR candidates law: the description when present, else
                // the key (their `crit[k] in (None, "")` fallback).
                let options: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| match v {
                        Value::Null => k.clone(),
                        Value::String(s) if s.is_empty() => k.clone(),
                        Value::String(s) => s.clone(),
                        other => clm::to_text(other),
                    })
                    .collect();
                Question::choice(q.qid.as_str(), q.instructions.as_str(), options, None)
            }
            QKind::Score => {
                let levels: Vec<String> = q
                    .criteria
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect()
                    })
                    .ok_or_else(|| {
                        format!("case {}: score criteria must be an array", q.qid)
                    })?;
                Question::score(q.qid.as_str(), q.instructions.as_str(), levels)
            }
            QKind::Noul => Question::noul(q.qid.as_str(), q.instructions.as_str()),
        };
        questions.push(question);
    }
    Ok(katgpt_core::decision_wire::DecisionRequest {
        state: clm::to_text(&case.state),
        questions,
    })
}

/// One rendered answer row for the observed-repeat determinism check —
/// the SAME law as the laya lane's `render` (6-decimal probabilities,
/// pick, confidence), so a nondeterministic server shows up identically
/// in both lanes' `det` columns.
#[cfg(feature = "clm-lane")]
fn clm_repeat_render(
    answers: &[katgpt_core::decision_wire::Answer],
) -> String {
    answers
        .iter()
        .map(|a| {
            format!(
                "{:?}|{:?}|{:.6}",
                a.outcome, a.probabilities, a.confidence
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Run the CLM comparison lane over one suite: every case → one
/// `/v1/systemone` round trip, latency = CLIENT round trip (the
/// cross-lane consistency law — the same wall-clock every other lane's
/// p50 reports), `usage.input_tokens` summed for the run log.
#[cfg(feature = "clm-lane")]
fn run_clm_lane(
    suite: &Suite,
    leak_flags: Option<&[bool]>,
) -> Result<(LaneResult, u64), String> {
    use crate::lanes::clm::{ClmLane, DEFAULT_MODEL};
    use katgpt_core::decision_wire::Outcome;

    let t_start = Instant::now();
    let lane = ClmLane::default();
    let cases = &suite.cases;
    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    let mut input_tokens: u64 = 0;

    // WARMUP (measured 2026-09-25, the determinism pin's cold-start
    // finding): the very FIRST request after a clm-serve boot answers
    // correctly but reports usage.input_tokens = 0 — a server-side
    // first-request accounting quirk. One throwaway FIXED request (never
    // a case's — no cache pollution of measured latencies) absorbs the
    // cold path before the first measured case.
    {
        use katgpt_core::decision_wire::Question;
        let warm = katgpt_core::decision_wire::DecisionRequest {
            state: "warmup: the lane's cold-path probe (discarded; not a \
                    measured case)"
                .to_string(),
            questions: vec![Question::noul("warm", "Is this the warmup?")],
        };
        if let Err(e) = lane.decide(&warm) {
            return Err(format!("clm warmup: {e} — is their stack serving at \
                 CLM_SERVE_URL? (scripts/clm_serve_4090.sh status)"));
        }
    }

    for (ci, case) in cases.iter().enumerate() {
        let req = clm_request(case)?;
        let t0 = Instant::now();
        let (resp, usage) = lane
            .decide(&req)
            .map_err(|e| format!("clm round trip ({}): {e}", case.id))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        input_tokens += usage.input_tokens;

        // Observed-repeat check, first 10 cases (the laya lane's law): a
        // lane that cannot repeat byte-identically flags its det column.
        if ci < 10 {
            let (resp2, _) = lane
                .decide(&req)
                .map_err(|e| format!("clm determinism rerun ({}): {e}", case.id))?;
            if clm_repeat_render(&resp.answers) != clm_repeat_render(&resp2.answers) {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        for (q, ans) in case.questions.iter().zip(resp.answers.iter()) {
            let (p, pick) = match q.kind {
                // `map_response` returns probabilities in OUR option order
                // — the criteria insertion order — which IS the label
                // order the gold indexes score against.
                QKind::Choice => {
                    let idx = match ans.outcome.as_ref() {
                        Some(Outcome::Choice { index }) => *index as usize,
                        _ => 0,
                    };
                    (
                        ans.probabilities.iter().map(|p| f64::from(*p)).collect(),
                        idx,
                    )
                }
                QKind::Score => {
                    let lvl = match ans.outcome.as_ref() {
                        Some(Outcome::Score { level }) => *level as usize,
                        _ => 0,
                    };
                    (
                        ans.probabilities.iter().map(|p| f64::from(*p)).collect(),
                        lvl,
                    )
                }
                // LABEL space: [p_no, p_yes], gold idx 1 = true (the same
                // convention every lane's noul row reports).
                QKind::Noul => {
                    let p_yes = f64::from(ans.probabilities.first().copied().unwrap_or(0.5));
                    (vec![1.0 - p_yes, p_yes], usize::from(p_yes >= 0.5))
                }
            };
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(f64::from(ans.confidence));
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }

    let result = assemble_laya_lane_result(
        "clm",
        DEFAULT_MODEL,
        suite.name,
        cases,
        leak_flags,
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        t_start.elapsed().as_secs_f64(),
    );
    Ok((result, input_tokens))
}

/// The feature-off stub: an explicit `--clm` is a LOUD refusal naming the
/// rebuild, never a silent absence (the build-stamp law).
#[cfg(not(feature = "clm-lane"))]
fn run_clm_lane(
    _suite: &Suite,
    _leak_flags: Option<&[bool]>,
) -> Result<(LaneResult, u64), String> {
    Err("clm-lane feature off — rebuild with --features clm-lane to measure the CLM reference".to_string())
}

/// The GLiNER comparison lane (Issue 029): fastino/GLiNER2.5-Decide driven
/// as a JSONL subprocess oracle over THEIR gliner2 package
/// (`scripts/gliner_lane.py`) — the laya-python lane's protocol and answer
/// mapping (SAME cases, SAME metrics tail via [`assemble_laya_lane_result`];
/// the only differences are the forward's executor and their probability
/// readout). Latency is the subprocess round-trip per case (IPC included),
/// the same measurement law as the laya-python oracle — the honest
/// cross-lane comparison for it is laya-python, NOT the in-process riir
/// lane; the table's posture line says so.
///
/// Env: `GLINER_LANE_SCRIPT` (default `scripts/gliner_lane.py`) ·
/// `GLINER_PYTHON` (default `python3`; the venv needs gliner2 + torch +
/// transformers + peft + accelerate — gliner2 declares none of them) ·
/// `GLINER_PY_DEVICE` (default `cuda`) · `GLINER_MODEL` (default
/// `fastino/GLiNER2.5-Decide`). The handshake advertises the loaded model
/// id and device; the lane stamps THOSE into the row, never a hardcoded
/// name (an env override shows up as itself).
fn run_gliner_lane(
    suite: &Suite,
    laya_max_questions: usize,
    // Issue 024 T3: one leak flag per suite.cases entry; the served set is
    // a PREFIX (the trim law), so the flags stay aligned.
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{ChildStdin, Command, Stdio};

    let script = std::env::var("GLINER_LANE_SCRIPT")
        .unwrap_or_else(|_| "scripts/gliner_lane.py".to_string());
    if !Path::new(&script).is_file() {
        return Err(format!(
            "gliner lane script not found at {script} — run from the repo root or set \
             GLINER_LANE_SCRIPT (the lane is opt-in measurement tooling; the venv \
             needs gliner2 + torch + transformers + peft + accelerate)"
        ));
    }
    let python = std::env::var("GLINER_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let device = std::env::var("GLINER_PY_DEVICE").unwrap_or_else(|_| "cuda".to_string());

    let t_start = Instant::now();
    let mut child = Command::new(&python)
        .arg(&script)
        .arg(&device)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // stderr inherits: their loader's warnings stay visible.
        .spawn()
        .map_err(|e| format!("spawn {python} {script}: {e}"))?;
    let mut stdin: ChildStdin = child
        .stdin
        .take()
        .ok_or_else(|| "gliner oracle stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "gliner oracle stdout unavailable".to_string())?;
    let mut reader = BufReader::new(stdout);

    // The send/read helpers share the laya-python lane's shapes verbatim
    // (the wire is the point of the protocol).
    let send_case = |stdin: &mut ChildStdin, case: &SuiteCase| -> Result<(), String> {
        let questions = case_questions(case);
        let qs: Vec<Value> = questions
            .iter()
            .map(|(qid, def)| serde_json::json!({"qid": qid, "def": def}))
            .collect();
        let line = serde_json::json!({"state": case.state, "questions": qs});
        writeln!(stdin, "{line}").map_err(|e| format!("gliner stdin: {e}"))?;
        stdin.flush().map_err(|e| format!("gliner stdin flush: {e}"))
    };
    let read_line = |reader: &mut BufReader<std::process::ChildStdout>,
                     buf: &mut String|
     -> Result<(), String> {
        buf.clear();
        let n = reader
            .read_line(buf)
            .map_err(|e| format!("gliner stdout: {e}"))?;
        if n == 0 {
            return Err(
                "gliner oracle stream ended early — the subprocess died; its \
                 stderr above names the cause"
                    .to_string(),
            );
        }
        Ok(())
    };

    // Handshake: one ready line once the checkpoint is loaded; the
    // advertised model id is stamped into the row (never hardcoded).
    let mut line = String::new();
    read_line(&mut reader, &mut line)?;
    let ready: Value = serde_json::from_str(line.trim())
        .map_err(|e| format!("gliner handshake: {e} (got: {})", line.trim()))?;
    if ready.get("ready") != Some(&Value::Bool(true)) {
        return Err(format!(
            "gliner handshake: expected {{\"ready\":true}}, got {line}"
        ));
    }
    let model = ready
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("fastino/GLiNER2.5-Decide")
        .to_string();
    let advertised_device = ready
        .get("device")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string();
    eprintln!("    [gliner] oracle up: {model} on {advertised_device}");

    // The trim law: the SAME question cap as the laya lanes, so a capped
    // run never compares a full-N gliner row against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // GPU pre-ramp (Issue 020), the same law as both laya lanes: one
    // unmeasured warmup case absorbs the CUDA graph/kernel-compile cold
    // path before the first measured case. `LAYA_HARNESS_NO_WARMUP=1`
    // restores the cold posture for every GPU lane at once.
    if laya_gpu_preramp_enabled()
        && let Some(first) = cases.first()
    {
        send_case(&mut stdin, first)?;
        let mut warm = String::new();
        read_line(&mut reader, &mut warm)?;
        eprintln!("  [gliner] gpu pre-ramp: 1 unmeasured warmup case");
    }

    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        send_case(&mut stdin, case)?;
        let mut line = String::new();
        read_line(&mut reader, &mut line)?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        let resp: Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("gliner response (case {ci}): {e}"))?;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column.
        if ci < 10 {
            send_case(&mut stdin, case)?;
            let mut line2 = String::new();
            read_line(&mut reader, &mut line2)?;
            if line != line2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        for q in case.questions.iter() {
            let answers = resp
                .get("answers")
                .and_then(|a| a.as_object())
                .ok_or_else(|| format!("gliner response (case {ci}): no answers object"))?;
            let a = answers.get(&q.qid).ok_or_else(|| {
                format!("gliner response (case {ci}): missing qid {}", q.qid)
            })?;
            let (p, pick, conf) = parse_python_answer(q, a)
                .map_err(|e| format!("gliner response (case {ci}): {e}"))?;
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    drop(stdin);
    let _ = child.wait();

    Ok(assemble_laya_lane_result(
        "gliner",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The AgentJev comparison lane (Issue 025 amendment 4 / `.issues/027`):
/// their `jev_service` served on loopback, measured over HTTP — the
/// MEASURE-vs-SERVE split (their Apache-2.0 Python service runs on the
/// 4090, our Rust harness measures; the riir-infer port stays DEFERRED in
/// `.issues/025`). Same cases, their contract (`src/lanes/agentjev.rs`
/// pins the mapping), the same metrics tail via
/// [`assemble_laya_lane_result`]. Latency = client round-trip per case
/// (their `usage.wall_ms` recorded beside it — the 025 amendment-4 law).
///
/// Env: `AGENTJEV_SERVE_URL` (default `http://127.0.0.1:8149`). The
/// `/api/info` handshake advertises the loaded checkpoint + device; the
/// lane stamps THOSE into the row, never a hardcoded name.
fn run_agentjev_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::agentjev::AgentJevLane;

    let t_start = Instant::now();
    let lane = AgentJevLane::default();
    let (model, device) = lane.info()?;
    eprintln!("    [agentjev] service up: {model} on {device}");

    // WARMUP (the clm lane's cold-start law): one FIXED throwaway request
    // (never a case's — no cache pollution of measured latencies) absorbs
    // the service's cold path before the first measured case.
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm)
            .map_err(|e| format!("agentjev warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N agentjev row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    let mut server_wall_ms: f64 = 0.0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, wall_ms) = lane
            .decide(case)
            .map_err(|e| format!("agentjev lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        if let Some(w) = wall_ms {
            server_wall_ms += w;
        }

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column.
        if ci < 10 {
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("agentjev determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("agentjev determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [agentjev] server-side wall sum: {server_wall_ms:.0} ms");

    Ok(assemble_laya_lane_result(
        "agentjev",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        t_start.elapsed().as_secs_f64(),
    ))
}

// ── prepared suite + run ────────────────────────────────────────────────

struct Prepared {
    suite: Suite,
    train: Vec<TrainDoc>,
    state_strs: Vec<String>,
    cal_cases: Vec<SuiteCase>,
    cal_state_strs: Vec<String>,
    labels: Vec<String>,
    /// The raw train envelope (dataset suites) — the cap-selection
    /// stratification's input. Null on the synthetic/code paths (those
    /// suites are selection-ineligible).
    train_rows: Value,
}

fn prepare(spec: &SuiteSpec, dir: &Path) -> Result<Prepared, String> {
    if let Some(synth) = spec.synthetic {
        if !spec.modelless_lane && !cfg!(feature = "laya-riir") && !cfg!(feature = "clm-lane") {
            return Err(format!(
                "{}: SKIPPED \u{2014} LLM-lane only (Issue 004 T3): the modelless lane has \
                 no KV cache, so a modelless answer here would be a fake task; \
                 compile with --features laya-riir (or clm-lane) to run this family",
                spec.name
            ));
        }
        let d = synth();
        let state_strs = d
            .suite
            .cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        let cal_state_strs = d
            .cal_cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        return Ok(Prepared {
            labels: d.labels,
            suite: d.suite,
            train: d.docs,
            state_strs,
            cal_cases: d.cal_cases,
            cal_state_strs,
            train_rows: Value::Null,
        });
    }

    if spec.name == "code_fixtures" {
        let suite = build_code_fixtures(&Value::Null, 0);
        let state_strs = suite
            .cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        let cal_cases = code_fixtures_cal_cases();
        let cal_state_strs = cal_cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        return Ok(Prepared {
            labels: CODE_MODULE_LABELS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            suite,
            train: code_fixtures_docs(),
            state_strs,
            cal_cases,
            cal_state_strs,
            train_rows: Value::Null,
        });
    }

    let suite_dir = dir.join(spec.name);
    let test_rows = load_rows(&suite_dir, "test")?;
    let suite = (spec.build)(&test_rows, spec.test_cap);
    let train_rows =
        load_rows(&suite_dir, "train").map_err(|e| format!("suite {}: {e}", spec.name))?;
    let train = train_docs(&train_rows, spec.name);
    if train.is_empty() {
        return Err(format!(
            "suite {}: the train split produced no corpus docs — the modelless \
             lane cannot be built honestly without its corpus; fetch train rows \
             first (scripts/fetch_datasets.sh)",
            spec.name
        ));
    }

    // Engine domain labels. These must equal the TRAIN-DOC labels so the
    // per-domain corpora bind (routing routes over these domains; corpora
    // are the train docs of each label). Per suite:
    // - classification suites (ag_news / emotion / sst5 / xnli / prompt):
    //   the integer class labels the train docs carry (option index i
    //   ↔ class label i — asserted against the option-key union below);
    // - typed_decisions: the WORKFLOW names (train docs carry them);
    // - derived-universe suites (massive / banking77): the union of option
    //   keys over the TEST cases (train docs carry label_text);
    // - code_fixtures: the fixed module labels (already set above).
    let option_key_union = {
        let mut u: Vec<String> = Vec::new();
        for case in &suite.cases {
            for q in &case.questions {
                match q.kind {
                    QKind::Choice => {
                        if let Some(m) = q.criteria.as_object() {
                            for k in m.keys() {
                                if !u.contains(k) {
                                    u.push(k.clone());
                                }
                            }
                        }
                    }
                    QKind::Score => {
                        if let Some(a) = q.criteria.as_array() {
                            for i in 0..a.len() {
                                let s = i.to_string();
                                if !u.contains(&s) {
                                    u.push(s);
                                }
                            }
                        }
                    }
                    QKind::Noul => {}
                }
            }
        }
        u
    };
    let labels: Vec<String> = match spec.name {
        "ag_news" => (0..4).map(|i| i.to_string()).collect(),
        "emotion" => (0..6).map(|i| i.to_string()).collect(),
        "sst5" => (0..5).map(|i| i.to_string()).collect(),
        "xnli_en" => (0..3).map(|i| i.to_string()).collect(),
        "prompt_injections" => (0..2).map(|i| i.to_string()).collect(),
        "typed_decisions" => {
            // The workflow names from the TEST case ids (id =
            // "<workflow>:<row_idx>"), sorted for a stable domain order.
            let mut w: Vec<String> = suite
                .cases
                .iter()
                .filter_map(|c| c.id.split(':').next().map(str::to_string))
                .collect();
            w.sort();
            w.dedup();
            w
        }
        _ => option_key_union.clone(),
    };
    // For fixed-criteria classification suites the engine domains must be
    // exactly as numerous as the option keys (pick index ↔ domain index).
    // Noul-only suites present no option keys — exempt (their classes ride
    // the train-doc labels).
    if matches!(spec.name, "ag_news" | "emotion" | "sst5" | "xnli_en") {
        assert_eq!(
            labels.len(),
            option_key_union.len(),
            "suite {}: {} class labels vs {} option keys — the fetch/protocol \
             and the engine arming disagree",
            spec.name,
            labels.len(),
            option_key_union.len()
        );
    }
    if spec.name == "banking77" {
        assert_eq!(
            option_key_union.len(),
            77,
            "banking77: the TEST suite presents {} option keys (need all 77 — \
             fetch the full test split for the option universe)",
            option_key_union.len()
        );
    }

    // Calibration cases: the SAME builder over the train rows (identical
    // question shapes; gold from the train labels).
    let (cal_cases, cal_state_strs) = if spec.cal_cap > 0 {
        let cal_suite = (spec.build)(&train_rows, spec.cal_cap);
        let strs = cal_suite
            .cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        (cal_suite.cases, strs)
    } else {
        (Vec::new(), Vec::new())
    };

    let state_strs = suite
        .cases
        .iter()
        .map(|c| serialize_state(&c.state))
        .collect();
    Ok(Prepared {
        suite,
        train,
        state_strs,
        cal_cases,
        cal_state_strs,
        labels,
        train_rows,
    })
}

/// Gate/CLI accessor: `(is_synthetic, modelless_lane)` for a suite name.
#[must_use]
pub fn suite_lanes(name: &str) -> Option<(bool, bool)> {
    SUITES
        .iter()
        .find(|s| s.name == name)
        .map(|s| (s.synthetic.is_some(), s.modelless_lane))
}

/// Options for one harness run.
#[derive(Debug, Clone)]
pub struct RunOptions {
    pub datasets_dir: PathBuf,
    /// Suite names; empty = all registered.
    pub suites: Vec<String>,
    /// Cap laya-lane questions per suite (0 = the protocol default).
    pub laya_max_questions: usize,
    /// Skip the laya lane entirely (modelless-only run).
    pub skip_laya: bool,
    /// Also run the laya-PYTHON lane — the ORIGINAL torch reference as a
    /// subprocess oracle (measurement-only; opt-in, off by default).
    pub laya_python: bool,
    /// Also run the GLiNER comparison lane — fastino/GLiNER2.5-Decide over
    /// their gliner2 package as a JSONL subprocess oracle (Issue 029;
    /// measurement-only, off by default).
    pub gliner: bool,
    /// Also run the AgentJev comparison lane (Issue 025 amendment 4 /
    /// `.issues/027`): their `jev_service` answered over HTTP
    /// (`AGENTJEV_SERVE_URL`, default `http://127.0.0.1:8149`) — their
    /// stack serves, our Rust measures (comparison lane, never a product
    /// lane). An unreachable server is a LOUD error, never a silent skip.
    /// Default off.
    pub agentjev: bool,
    /// Override every dataset suite's per-label corpus cap (0 = the
    /// registry defaults). MEASUREMENT-ONLY — the acc-vs-cap lever sweep
    /// (Issue 013 lever 1); a published table must state the override or
    /// read against the registry posture, never mix the two. Mutually
    /// exclusive with [`RunOptions::cal_select_caps`] (one pins the cap,
    /// the other selects it).
    pub corpus_cap_override: usize,
    /// Cal-slice cap-selection candidates (Issue 013 lever-1 protocol
    /// plumb). Empty = off. Non-empty = for every eligible dataset suite,
    /// accuracy is measured at each candidate (plus the registry default)
    /// on a label-STRATIFIED selection slice drawn from the pool region —
    /// the registry cal slice itself is label-clustered (the mirrors store
    /// rows label-grouped) and cannot rank caps — the argmax cap is
    /// selected on that slice ONLY, and the test split is read once at the
    /// selected cap. The selection table rides the suite's result row.
    /// Mutually exclusive with [`RunOptions::corpus_cap_override`].
    pub cal_select_caps: Vec<usize>,
    /// Run the Issue-013 lever-3 pair-head A/B arm: per suite, arm the
    /// top confusion pairs from CAL-slice mispredictions, fit diagonal-LDA
    /// heads from the pair's corpus docs, and re-decide questions whose
    /// engine top-2 matches an armed pair. Report-only (a result-row
    /// record, never a gate); the test split is read once. Default off.
    pub pair_head_ab: bool,
    /// Also run the CLM comparison lane (Issue 019 T3 / `.issues/027`):
    /// the external Contrastive-LM reference answered over HTTP
    /// (`clm-serve` at `CLM_SERVE_URL`, default `http://127.0.0.1:8700`)
    /// — their stack serves, our Rust measures (comparison lane, never a
    /// product lane). Needs the `clm-lane` feature; an explicit flag
    /// without it, or an unreachable server, is a LOUD error, never a
    /// silent skip. Default off.
    pub clm: bool,
}

/// Run the harness. Suite-level failures (missing datasets, engine build
/// errors) are REPORTED in the returned output as errors — never silently
/// dropped — and the run continues with the remaining suites.
pub fn run(opts: &RunOptions) -> Result<(RunOutput, Vec<String>), String> {
    let box_start = super::box_state::capture();
    if opts.corpus_cap_override != 0 && !opts.cal_select_caps.is_empty() {
        return Err(
            "--corpus-cap and --cal-select-cap are mutually exclusive: one pins the cap, \
             the other selects it on the cal slice"
                .to_string(),
        );
    }
    let mut results: Vec<SuiteResult> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let laya_feature = cfg!(feature = "laya-riir");

    for spec in SUITES {
        if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
            continue;
        }
        eprintln!("═══ suite {} ═══", spec.name);
        let prepared = match prepare(spec, &opts.datasets_dir) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!("{}: {e}", spec.name));
                continue;
            }
        };
        let n_questions: usize = prepared.suite.cases.iter().map(|c| c.questions.len()).sum();
        eprintln!(
            "    {} cases / {} questions · {} domains",
            prepared.suite.cases.len(),
            n_questions,
            prepared.labels.len()
        );

        // Issue 024 T3: the corpus∪cal → eval near-duplicate scan over the
        // slices THIS run serves (feature `slice_leak`). Reference side =
        // the train split (the corpora draw from train[cal_cap..], the
        // calibration slice IS train[..cal_cap] — the union is the whole
        // split, the probe's exact reference); query side = the built eval
        // cases, whose raw dataset text is recovered by the same rule the
        // corpus side uses. Everything stays None when the feature is off
        // or the suite is out of scope — G3 keeps results.json identical.
        #[cfg(all(feature = "slice_leak", not(target_arch = "wasm32")))]
        let (leak_block, leak_flags): (Option<SuiteLeak>, Option<Vec<bool>>) = {
            let in_scope = spec.synthetic.is_none()
                && spec.name != "typed_decisions"
                && spec.name != "code_fixtures";
            if !in_scope {
                (None, None)
            } else {
                let queries: Option<Vec<String>> = prepared
                    .suite
                    .cases
                    .iter()
                    .map(|c| super::slice_leak::eval_case_text(spec.name, c))
                    .collect();
                match queries {
                    Some(queries) => {
                        let refs: Vec<&str> =
                            prepared.train.iter().map(|d| d.text.as_str()).collect();
                        match super::slice_leak::scan_eval(
                            &refs,
                            &queries,
                            super::slice_leak::LeakParams::default(),
                        ) {
                            Some(scan) => {
                                eprintln!(
                                    "    leak: exact {} / near {} over {} eval rows \
                                     ({} reference rows, J >= {:.2})",
                                    scan.exact,
                                    scan.near,
                                    scan.n_eval,
                                    scan.n_reference,
                                    super::slice_leak::LeakParams::default().near_j
                                );
                                (
                                    Some(SuiteLeak {
                                        threshold: super::slice_leak::LeakParams::default().near_j,
                                        n_reference: scan.n_reference,
                                        n_eval: scan.n_eval,
                                        exact: scan.exact,
                                        near: scan.near,
                                    }),
                                    Some(scan.flags),
                                )
                            }
                            None => (None, None),
                        }
                    }
                    None => {
                        // A builder key drifted: the scan would silently
                        // compare against a shorter query set — surfaced,
                        // never read as clean.
                        errors.push(format!(
                            "{}: leak report SKIPPED — an eval case's dataset text was \
                             not recoverable from its built state (state-shape drift)",
                            spec.name
                        ));
                        (None, None)
                    }
                }
            }
        };
        #[cfg(any(
            not(feature = "slice_leak"),
            target_arch = "wasm32"
        ))]
        let (leak_block, leak_flags): (Option<SuiteLeak>, Option<Vec<bool>>) = (None, None);
        let leak_flags_ref: Option<&[bool]> = leak_flags.as_deref();

        // Modelless lane (const-generic dispatch over the domain count).
        // Skipped entirely for LLM-only families (Issue 004 T3) — an honest
        // None, never a fabricated row.
        let modelless = if spec.modelless_lane {
            let inp = ModellessInput {
                spec,
                suite: &prepared.suite,
                train: &prepared.train,
                state_strs: &prepared.state_strs,
                cal_cases: &prepared.cal_cases,
                cal_state_strs: &prepared.cal_state_strs,
                labels: &prepared.labels,
                want_by_type: spec.name == "typed_decisions",
                corpus_cap_per_label: if opts.corpus_cap_override != 0 {
                    opts.corpus_cap_override
                } else {
                    spec.corpus_cap_per_label
                },
                cap_source_base: if opts.corpus_cap_override != 0 {
                    "--corpus-cap override"
                } else {
                    "registry"
                },
                cal_select_caps: &opts.cal_select_caps,
                train_rows: &prepared.train_rows,
                pair_head_ab: opts.pair_head_ab,
                leak_flags: leak_flags_ref,
            };
            macro_rules! dispatch {
                ($n:literal) => {
                    run_modelless::<$n>(&inp)
                };
            }
            let modelless = match prepared.labels.len() {
                2 => dispatch!(2),
                3 => dispatch!(3),
                4 => dispatch!(4),
                5 => dispatch!(5),
                6 => dispatch!(6),
                8 => dispatch!(8),
                59 => dispatch!(59),
                77 => dispatch!(77),
                n => {
                    errors.push(format!(
                        "{}: no engine instantiation for {} domains — extend the \
                         dispatch table in runner.rs",
                        spec.name, n
                    ));
                    continue;
                }
            };
            match modelless {
                Ok(r) => {
                    eprintln!(
                        "    modelless: acc {:.4} · ece(maxp) {:.4} · p50 {:.3} ms · {} s",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0
                    );
                    Some(r)
                }
                Err(e) => {
                    errors.push(format!("{} (modelless): {e}", spec.name));
                    continue;
                }
            }
        } else {
            eprintln!(
                "    modelless: SKIPPED — LLM-lane only (Issue 004 T3: no KV \
                 cache in the modelless lane)"
            );
            None
        };

        // Laya lane.
        let mut laya_results = BTreeMap::new();
        if !opts.skip_laya {
            for ck in laya_checkpoints_for(spec.name) {
                eprintln!("    laya[{ck}]: running…");
                match run_laya_checkpoint(&prepared.suite, ck, opts.laya_max_questions, leak_flags_ref) {
                    Ok((r, notes)) => {
                        eprintln!(
                            "    laya[{ck}]: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                            r.hard.accuracy,
                            r.hard.ece,
                            r.latency_p50_ms,
                            (r.seconds * 10.0).round() / 10.0
                        );
                        // The ANE bucket-skip disclosure — named cases,
                        // never a silently smaller served set.
                        errors.extend(notes);
                        laya_results.insert((*ck).to_string(), r);
                    }
                    Err(e) => {
                        // Honest absence: weights missing / feature off —
                        // recorded, never silently skipped.
                        errors.push(format!("{} (laya/{ck}): {e}", spec.name));
                    }
                }
            }
        }

        // Laya-python lane (opt-in measurement oracle): the ORIGINAL torch
        // reference answering the SAME cases as a subprocess. Keyed `py/`
        // so both backends can appear side by side in the tables.
        if opts.laya_python {
            for ck in laya_checkpoints_for(spec.name) {
                eprintln!("    laya-python[{ck}]: running…");
                match run_laya_python_checkpoint(&prepared.suite, ck, opts.laya_max_questions, leak_flags_ref) {
                    Ok(r) => {
                        eprintln!(
                            "    laya-python[{ck}]: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                            r.hard.accuracy,
                            r.hard.ece,
                            r.latency_p50_ms,
                            (r.seconds * 10.0).round() / 10.0
                        );
                        laya_results.insert(format!("py/{ck}"), r);
                    }
                    Err(e) => {
                        // Loud absence, never a silent skip (the same law
                        // as the riir lane's errors).
                        errors.push(format!("{} (laya-python/{ck}): {e}", spec.name));
                    }
                }
            }
        }

        // CLM comparison lane (Issue 019 T3 / `.issues/027`): the external
        // Contrastive-LM reference over `/v1/systemone` — their stack
        // serves, our Rust measures. Same cases, their rendering law, the
        // same metrics tail.
        let clm_result = if opts.clm {
            eprintln!("    clm: running…");
            match run_clm_lane(&prepared.suite, leak_flags_ref) {
                Ok((r, input_tokens)) => {
                    eprintln!(
                        "    clm: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s · {} input tokens",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0,
                        input_tokens
                    );
                    Some(r)
                }
                Err(e) => {
                    // Loud absence, never a silent skip (the same law as
                    // the laya lanes' errors).
                    errors.push(format!("{} (clm): {e}", spec.name));
                    None
                }
            }
        } else {
            None
        };

        // GLiNER comparison lane (Issue 029): fastino/GLiNER2.5-Decide as a
        // JSONL subprocess oracle over their gliner2 package — same cases,
        // the lane's own probability readout, the same metrics tail.
        let gliner_result = if opts.gliner {
            eprintln!("    gliner: running…");
            match run_gliner_lane(&prepared.suite, opts.laya_max_questions, leak_flags_ref) {
                Ok(r) => {
                    eprintln!(
                        "    gliner: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0
                    );
                    Some(r)
                }
                Err(e) => {
                    errors.push(format!("{} (gliner): {e}", spec.name));
                    None
                }
            }
        } else {
            None
        };

        // AgentJev comparison lane (Issue 025 amendment 4 / `.issues/027`):
        // their jev_service served on loopback, measured over HTTP — same
        // cases, their contract, the same metrics tail.
        let agentjev_result = if opts.agentjev {
            eprintln!("    agentjev: running…");
            match run_agentjev_lane(&prepared.suite, opts.laya_max_questions, leak_flags_ref) {
                Ok(r) => {
                    eprintln!(
                        "    agentjev: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0
                    );
                    Some(r)
                }
                Err(e) => {
                    errors.push(format!("{} (agentjev): {e}", spec.name));
                    None
                }
            }
        } else {
            None
        };

        results.push(SuiteResult {
            name: spec.name.to_string(),
            n_cases: prepared.suite.cases.len(),
            n_questions,
            modelless,
            laya: laya_results,
            clm: clm_result,
            gliner: gliner_result,
            agentjev: agentjev_result,
            leak: leak_block,
        });
    }

    let box_state = super::box_state::BoxStateSpan {
        start: box_start,
        end: super::box_state::capture(),
    };
    let meta = RunMeta {
        box_state,
        date_utc: iso8601_utc(),
        git_sha: git_sha().unwrap_or_else(|| "unknown".to_string()),
        host: hostname(),
        profile: if cfg!(debug_assertions) {
            "dev".to_string()
        } else {
            "release".to_string()
        },
        laya_feature,
        laya_max_questions: opts.laya_max_questions,
        datasets_dir: opts.datasets_dir.display().to_string(),
        corpus_cap_mode: if opts.corpus_cap_override != 0 {
            format!(
                "override {} (measurement-only --corpus-cap; every dataset suite) — \
                 MEASUREMENT POSTURE, not the registry posture",
                opts.corpus_cap_override
            )
        } else if !opts.cal_select_caps.is_empty() {
            "stratified cap selection: accuracy per candidate cap (+ the registry default) on \
             a label-stratified slice of the pool region, argmax picked on that slice ONLY, \
             test read once at the selected cap (Issue 013 lever-1 protocol)"
                .to_string()
        } else {
            "registry defaults".to_string()
        },
        corpus_protocol: "one engine domain per label; corpus = train-split docs of that \
                          label, capped per label (registry), self-doc fallback for \
                          labels absent from the fetched train rows"
            .to_string(),
        calibration_protocol: "sigmoid-gate fit on a train-tail calibration slice (same \
                               builder over the train rows); fused-gate thresholds fitted \
                               per suite at the cal-slice 30th percentile (the T1.6 arena \
                               posture rho=30%; the birth constants do not transfer — \
                               measured: 100% abstain on several suites at defaults); \
                               raw-vs-calibrated readout ECE compared per the G1 gate; \
                               no calibration claim when the calibrator never moved"
            .to_string(),
        floor_definition: "conformal-naive floor = split-conformal recalibration of the raw \
                           readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — \
                           the exchangeability-valid baseline (G1 fails unless the \
                           calibrated ECE beats it)"
            .to_string(),
        determinism_scoping: "the bit-identity claim is the MODELLESS lane's (plan caveat 3); \
                              the laya lane gets the same observed repeat check and it is \
                              reported, not claimed"
            .to_string(),
        laya_device: {
            #[cfg(feature = "laya-riir")]
            {
                use crate::laya::riir::agent::DeviceKind;
                match DeviceKind::from_env() {
                    Ok(DeviceKind::Metal) => {
                        "metal (LAYA_DEVICE or the build's macOS default)".to_string()
                    }
                    Ok(DeviceKind::Cuda) => {
                        "cuda (LAYA_DEVICE or the build's non-macOS CUDA default)".to_string()
                    }
                    Ok(DeviceKind::Cpu) => {
                        "cpu (LAYA_DEVICE or the no-backend default)".to_string()
                    }
                    Ok(DeviceKind::Ane) => {
                        // This harness interprets LAYA_DEVICE=ane itself: it
                        // routes the checkpoint loads to RiirAgent::load_ane
                        // (artifacts: LAYA_ANE_ARTIFACTS_DIR else assets/ane/).
                        // The substrate's plain loader still refuses env-only
                        // ANE — only an explicit consumer choice selects the
                        // lane, never a silent fallback.
                        "ane (LAYA_DEVICE=ane → load_ane; whole-graph CoreML encoder, Plan 002)"
                            .to_string()
                    }
                    Err(e) => format!("unknown ({e})"),
                }
            }
            #[cfg(not(feature = "laya-riir"))]
            {
                "n/a (compiled without laya-riir)".to_string()
            }
        },
        laya_python_lane: if opts.laya_python {
            "on — the ORIGINAL torch reference as a JSONL subprocess oracle: same cases, \
             the reference's own rounded-4 probabilities, latency = subprocess round-trip \
             (IPC included)"
                .to_string()
        } else {
            "off (pass --laya-python to add the reference lane)".to_string()
        },
        clm_lane: if opts.clm {
            "on — the external Contrastive-LM reference over /v1/systemone \
             (clm-serve; comparison lane, Apache-2.0, not affiliated): same cases, \
             their rendering law, latency = client round-trip; determinism = the \
             observed-repeat check"
                .to_string()
        } else {
            "off (pass --clm to add the comparison lane; .issues/027)".to_string()
        },
        gliner_lane: if opts.gliner {
            "on — fastino/GLiNER2.5-Decide over their gliner2 package as a JSONL \
             subprocess oracle (comparison lane, Apache-2.0, not affiliated): same \
             cases, their per-label probability readout (softmax for single-label \
             heads; conf = top-label probability), their fp32 default posture; \
             latency = subprocess round-trip (IPC included) — the honest \
             cross-lane latency comparison is laya-python (the same IPC law), \
             never the in-process riir lane; determinism = the observed-repeat \
             check; .issues/029"
                .to_string()
        } else {
            "off (pass --gliner to add the comparison lane; needs the gliner2 \
             venv — .issues/029)"
                .to_string()
        },
        agentjev_lane: if opts.agentjev {
            "on — their jev_service (malevrigns/agent-jev @ a965ca8f, Apache-2.0, \
             not affiliated) served on loopback, measured over HTTP (comparison \
             lane, never a product lane): same cases, their decision.v1 contract, \
             gold-label scoring (their published 79.25% is teacher-argmax \
             agreement — this lane's row is the protocol-honest split); latency = \
             client round-trip, their usage.wall_ms recorded beside it; \
             determinism = the observed-repeat check; .issues/025 + .issues/027"
                .to_string()
        } else {
            "off (pass --agentjev to add the comparison lane; needs their \
             jev_service on AGENTJEV_SERVE_URL — .issues/025)"
                .to_string()
        },
        divergences: vec![
            "massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — \
             both lanes see byte-identical questions, which is the integrity that matters"
                .to_string(),
            "banking77: mteb/banking77 mirror (the reference's own bench_apps variant); \
             PolyAI/banking77 is script-based and unservable"
                .to_string(),
            "engine context = state + prompt (wire criteria None) — the modelless \
             serving path"
                .to_string(),
            "harness families (Issue 004, Research 579): in-process synthetic \
             fixtures with programmatic gold; harness_cache_reuse is LLM-lane \
             only (the modelless lane has no KV cache) and reports a loud \
             SKIPPED absence without the laya-riir feature"
                .to_string(),
        ],
    };

    Ok((
        RunOutput {
            meta,
            suites: results,
        },
        errors,
    ))
}

// ── table rendering ─────────────────────────────────────────────────────

fn fmt4(x: f64) -> String {
    format!("{x:.4}")
}

fn fmt_opt(x: Option<f64>) -> String {
    x.map_or("—".to_string(), fmt4)
}

/// Render the run as markdown (`TABLES.md`).
#[must_use]
/// The gate-fit table cell (Issue 009 T2/T3): the fitted pair + the
/// support disclosure, `thin → defaults` when the cal slice was too small,
/// `—` on lanes that fit no gates.
fn fmt_gate_fit(r: &LaneResult) -> String {
    let Some(rec) = &r.threshold_recommendation else {
        return "—".to_string();
    };
    match (&rec.score, &rec.distance) {
        (Some(s), Some(d)) => format!(
            "s {} / d {} (n {})",
            fmt4(f64::from(s.threshold)),
            fmt4(f64::from(d.threshold)),
            s.support.n
        ),
        _ => "thin → defaults".to_string(),
    }
}

pub fn render_markdown(out: &RunOutput, errors: &[String]) -> String {
    let mut s = String::new();
    s.push_str("# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)\n\n");
    s.push_str(&format!(
        "- run: `{}` on `{}` ({}) · profile {} · laya feature {}\n",
        out.meta.git_sha, out.meta.host, out.meta.date_utc, out.meta.profile, out.meta.laya_feature
    ));
    if out.meta.laya_feature {
        s.push_str(
            "- laya posture: PRE-MOVE BASELINE — `src/laya` is scheduled to move to the \
             riir-infer repo (008 T4); the laya columns pin the pre-move tree at the sha above\n",
        );
        if out.meta.laya_max_questions > 0 {
            s.push_str(&format!(
                "- laya cap: {} questions per checkpoint — PARTIAL run; the laya `n` columns \
                 reflect the cap and are NOT comparable row-wise against the modelless `n` columns\n",
                out.meta.laya_max_questions
            ));
        }
    }
    s.push_str(&super::box_state::render_line(&out.meta.box_state));
    s.push_str(&format!(
        "- laya device posture: {}\n",
        out.meta.laya_device
    ));
    s.push_str(&format!(
        "- laya-python lane: {}\n",
        out.meta.laya_python_lane
    ));
    s.push_str(&format!("- clm lane: {}\n", out.meta.clm_lane));
    s.push_str(&format!("- gliner lane: {}\n", out.meta.gliner_lane));
    s.push_str(&format!(
        "- corpus cap posture: {}\n",
        out.meta.corpus_cap_mode
    ));
    s.push_str(&format!(
        "- corpus: {} \n- calibration: {}\n- floor: {}\n- determinism: {}\n",
        out.meta.corpus_protocol,
        out.meta.calibration_protocol,
        out.meta.floor_definition,
        out.meta.determinism_scoping
    ));
    for d in &out.meta.divergences {
        s.push_str(&format!("- divergence: {d}\n"));
    }
    s.push('\n');

    if !errors.is_empty() {
        s.push_str("## Absences / errors (honest — never silently dropped)\n\n");
        for e in errors {
            s.push_str(&format!("- {e}\n"));
        }
        s.push('\n');
    }

    for suite in &out.suites {
        s.push_str(&format!(
            "## {} — {} cases / {} questions\n\n",
            suite.name, suite.n_cases, suite.n_questions
        ));
        // The corpus-cap disclosure + the cal-slice selection table when
        // selection ran (Issue 013 lever-1 protocol): the selection reads
        // cal only; the suite's row below IS the single test read.
        if let Some(m) = &suite.modelless
            && let Some(cap) = &m.corpus_cap
        {
            s.push_str(&format!(
                "**corpus cap:** {} ({})\n\n",
                cap.effective, cap.source
            ));
            if let Some(rows) = &cap.selection {
                s.push_str("| cap | cal acc |\n|---|---|\n");
                for c in rows {
                    let mark = if c.cap == cap.effective {
                        " ← selected"
                    } else {
                        ""
                    };
                    s.push_str(&format!("| {}{} | {} |\n", c.cap, mark, fmt4(c.cal_acc)));
                }
                s.push('\n');
            }
        }
        let Some(m) = &suite.modelless else {
            if suite.laya.is_empty() {
                s.push_str(
                    "> modelless lane: SKIPPED — LLM-lane only (Issue 004 T3: the \
                     modelless lane has no KV cache; compile with `--features laya-riir`).\n\n",
                );
            } else {
                s.push_str(
                    "> modelless lane: SKIPPED — LLM-lane only (the modelless lane has no \
                     KV cache, so it has no honest answer for this family); the laya lane \
                     answered below.\n\n",
                );
            }
            if !suite.laya.is_empty() || suite.clm.is_some() || suite.gliner.is_some() {
                s.push_str("| lane · model | n | acc | ECE(maxp) | readout-ECE | p50 | p99 (support) | det |\n");
                s.push_str("|---|---|---|---|---|---|---|---|\n");
                for r in suite
                    .laya
                    .values()
                    .chain(suite.clm.iter())
                    .chain(suite.gliner.iter())
                {
                    s.push_str(&format!(
                        "| {} · {} | {} | {} | {} | {} | {:.1} ms | {} | {} |\n",
                        r.lane,
                        r.model,
                        r.hard.n,
                        fmt4(r.hard.accuracy),
                        fmt4(r.hard.ece),
                        fmt_opt(r.readout_ece),
                        r.latency_p50_ms,
                        fmt_p99_cell(
                            r.latency_p99_ms,
                            r.latency_tail_support,
                            r.latency_extremes.as_ref(),
                            1
                        ),
                        r.determinism_ok
                            .map_or("—", |ok| if ok { "✓" } else { "✗" }),
                    ));
                }
            } else {
                s.push_str("(no lane produced a result — see the absences section)\n");
            }
            s.push('\n');
            continue;
        };
        s.push_str(
            "| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |\n",
        );
        s.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
        s.push_str(&format!(
            "| modelless | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2}/{:.2} | {} | {:.3} ms | {} | {} | {} |\n",
            m.hard.n,
            fmt4(m.hard.accuracy),
            fmt4(m.hard.macro_f1),
            fmt4(m.hard.ece),
            fmt4(m.hard.brier),
            fmt4(m.hard.nll),
            fmt4(m.hard.aurc),
            fmt4(m.hard.acc_at_50_coverage),
            fmt_opt(m.readout_ece),
            m.raw_abstain.as_ref().map_or(0.0, |a| a.abstain_rate),
            m.calibrated_abstain.as_ref().map_or(0.0, |a| a.abstain_rate),
            m.calibrated_abstain
                .as_ref()
                .map_or("—".to_string(), |a| fmt4(a.selective_accuracy)),
            m.latency_p50_ms,
            fmt_p99_cell(
                m.latency_p99_ms,
                m.latency_tail_support,
                m.latency_extremes.as_ref(),
                3
            ),
            m.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            fmt_gate_fit(m),
        ));
        for r in suite.laya.values() {
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.clm {
            // Same shape as a laya row — the CLM reference is a comparison
            // lane with the same metrics surface (no abstain, no gates).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.gliner {
            // Same shape as the clm row — the GLiNER reference is a
            // comparison lane with the same metrics surface (no abstain,
            // no gates; latency = subprocess round-trip, the laya-python
            // measurement law).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.agentjev {
            // Same shape as the gliner row — the AgentJev reference is a
            // comparison lane with the same metrics surface (no abstain,
            // no gates; latency = client round-trip over HTTP).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if suite.laya.is_empty() {
            s.push_str("| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |\n");
        }

        // G1 table (modelless only).
        if let (Some(raw), Some(cal), Some(floor), Some(verdict)) = (
            m.readout_ece_raw,
            m.readout_ece_calibrated,
            m.floor_ece,
            m.g1_verdict,
        ) {
            let (word, why) = match verdict {
                G1Verdict::Pass => ("PASS", "beats both the uncalibrated output AND the floor"),
                G1Verdict::Fail => ("FAIL", "does not beat both"),
                G1Verdict::NoClaim => (
                    "NO CLAIM",
                    "the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail",
                ),
            };
            s.push_str(&format!(
                "\n**G1 (modelless readout ECE):** raw {} · calibrated {} · conformal-naive floor {} → **{}** ({})\n",
                fmt4(raw),
                fmt4(cal),
                fmt4(floor),
                word,
                why
            ));
        }

        // typed_decisions extras.
        if suite.name == "typed_decisions" {
            if let Some(m) = suite.modelless.as_ref() {
                s.push_str(&format!(
                    "\n**typed-decisions extras (modelless):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n\n",
                    fmt_opt(m.soft_acc),
                    fmt_opt(m.brier_soft),
                    fmt_opt(m.score_mae),
                    fmt_opt(m.within_1)
                ));
                if let Some(by) = &m.by_question_type {
                    s.push_str("| model | type | n | acc | ECE(maxp) | mean conf |\n|---|---|---|---|---|---|\n");
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| modelless | {t} | {} | {} | {} | {} |\n",
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                    for r in suite.laya.values() {
                        if let Some(by) = &r.by_question_type {
                            for (t, h) in by {
                                s.push_str(&format!(
                                    "| {}·{} | {t} | {} | {} | {} | {} |\n",
                                    r.lane,
                                    r.model,
                                    h.n,
                                    fmt4(h.accuracy),
                                    fmt4(h.ece),
                                    fmt4(h.mean_confidence)
                                ));
                            }
                        }
                    }
                }
            }
            for r in suite.laya.values() {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
            }
            if let Some(r) = &suite.clm {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
                if let Some(by) = &r.by_question_type {
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| {}·{} | {t} | {} | {} | {} | {} |\n",
                            r.lane,
                            r.model,
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                }
            }
            if let Some(r) = &suite.gliner {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
                if let Some(by) = &r.by_question_type {
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| {}·{} | {t} | {} | {} | {} | {} |\n",
                            r.lane,
                            r.model,
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                }
            }
        }
        if suite.name == "sst5"
            && let Some(m) = suite.modelless.as_ref()
        {
            s.push_str(&format!(
                "\n**sst5 score metrics (modelless):** MAE {} · within_1 {}\n",
                fmt_opt(m.score_mae),
                fmt_opt(m.within_1)
            ));
        }
        s.push('\n');
    }
    // Issue-025 landscape: PUBLISHED vendor rows, quoted as published —
    // never measured by this harness. Pin: malevrigns/agent-jev @
    // a965ca8ff06ccabc0c796dca5447b55cc2069cee (2026-09-23, Apache-2.0);
    // numbers verified against their README results table +
    // typed_decisions/agentjev_v1_report.json (/trained/*; the same file
    // carries the phase-4 pre-run baseline). Static by design — this file
    // is regenerated wholesale, so the section lives here, not in a
    // hand-edited copy.
    s.push_str(
r#"## Landscape — published specialist rows (NOT measured by this harness)

External "System One" typed-decision models on the same 400-case /
2000-question Typed Decisions official test split, quoted AS PUBLISHED
(issue 025; pin `malevrigns/agent-jev` @ `a965ca8f`, Apache-2.0). Their
protocol differs from ours — the footnotes are part of the row; no number
here is comparable without them.

| lane · model | source | acc | bool·noul / choice / score | p50 case |
|---|---|---|---|---|
| AgentJev-0.6B (598M, Qwen3-0.6B backbone) | published (their run) | **0.7925** | 88.83 / 75.33 / 75.00 | ~60–70 ms, their cuda box |
| reflex · agentjev (their service, gold-label) | **MEASURED** (bench 039, 4090) | **0.7715** | — | 88 ms (loopback HTTP) |
| Laya (published checkpoint, 421M ModernBERT) | their table — card-copied, not re-scored | 0.7700 | — | 41.53 ms (their box) |
| TypeSafe Jev 1.13.0 | their table — zero-shot generalist | 0.727 | — | — |
| reflex · laya-riir·typed | MEASURED — the typed_decisions table above | 0.7445 (baseline `aa37823`) | 78.50 / 73.33 / 72.25 | 1312 ms (m3 metal, that baseline) |
| reflex · modelless | MEASURED — the typed_decisions table above | 0.3190 (baseline `aa37823`) | 53.17 / 18.67 / 25.87 | 0.472 ms |

Footnotes: (1) their accuracy is agreement with the public TEACHER argmax;
ours is gold-label under the standard harness protocol — different
references of truth. (1b) the MEASURED agentjev row (bench 039, Issue 025
amendment 4) closes that gap for AgentJev: **0.7715 gold-label** on this
harness's split (teacher-argmax 0.7925 → gold −2.1pt) — the published
ranking SURVIVES the protocol change (+2.7pt over our measured laya-typed
0.7445; their teacher-protocol gap was +2.25). (2) their run held out 120 dev + 120 cal cases and
selected the step-600 checkpoint on dev soft-CE before opening test; ours
fits no per-benchmark head. (3) their wide-load figure (shared-prefix
298.91 ms vs unshared 609.65 ms at 66 paths / 33,547 tokens, backbone
token-ops 33,547 → 2,551 = 92.4% reduction, max prob delta 5.08e-4) is
their box and their load — not re-measured here. (4) on SHORT inputs their
own table reads Laya faster (41.53 ms vs ~60–70 ms p50/case); AgentJev's
latency win is wide candidate loads only. (5) the bool·noul column maps
their boolean primitive to our noul primitive — the closest analogue, not
a wire match; neither of their lanes carries an abstention primitive (the
wire's first-class abstention is ours alone).

"#,
    );
    // Issue-029 landscape row: fastino's fast-decisions suite — PUBLISHED
    // vendor rows, quoted as published; this harness does NOT run their
    // suite (the issue-029 non-goal: our 15 suites are the protocol).
    // Numbers verified against the fastino/GLiNER2.5-Decide model card +
    // the fastino/fast-decisions dataset card (fetched 2026-09-25,
    // Apache-2.0). The scored split is HELD OUT (their public repo ships
    // only the 100/domain dev split, with an explicit do-not-score note)
    // — so the row is necessarily quoted, never re-runnable here. Static
    // by design, the same law as the block above.
    s.push_str(
r#"## Landscape — fast-decisions (vendor suite; NOT measured by this harness)

fastino's `fast-decisions` suite — 17 English operational-decision
domains (commerce support intent/topic, ticket routing, product feedback,
banking intent, document type, review sentiment, assistant handoff,
email triage, clinic request, travel request, news topic, paper field,
sports recap, restaurant review, benefits request, screen tags), 300
held-out test examples per domain, exact-match accuracy, the same text
and candidate labels for every model. Quoted AS PUBLISHED (issue 029).

| model | avg exact-match |
|---|---|
| GLiNER2.5-Decide (340M, DeBERTa-v3-large) | **60.2%** |
| GLiNER2.5-Decide-1B (their dataset card: "GLiNER2 XL (1B)") | 59.6% |
| JevK5 | 57.6% |
| GLiNER2.5-multi-Decide (287M) | 56.7% |
| SemIf (Qwen3.5-4B) | 56.4% |
| GLiFormer large-v1 | 49.0% |
| Laya Router | 46.6% |

Footnotes: (1) the scored split is private — their public repo carries
only the 100/domain development split with an explicit "do not report a
score computed on the files in this repo" note, so no row here can be
re-scored outside fastino. (2) their metric is per-head exact match
(single-label string equality; multi-label heads compared as sets),
averaged over the 17 domains. (3) their "Laya Router" row names no
checkpoint variant — neither the base router nor the typed specialist;
our measured laya-riir cells are the port checkpoints under OUR protocol,
so the 60.2-vs-46.6 gap is their-suite/their-checkpoint/their-protocol.
(4) our measured comparison lives in the 15-suite tables (bench 037,
`.benchmarks/037_gliner_lane_4090`): the DIRECTION is confirmed on
decision-style suites — gliner beats the laya base checkpoint 9/15
(banking77 +20.8pt, massive_intent +7.3pt, all five harness families) —
and honestly refuted on classic NLU (ag_news −24.8pt, xnli_en −38.3pt;
their card's own "not a general-purpose model" framing). The
typed_decisions headline stays laya's: the `typed` specialist 0.7445 vs
gliner 0.5280.

"#,
    );
    s
}

// ── misc ────────────────────────────────────────────────────────────────

fn iso8601_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, mi, se) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // civil-from-days (Howard Hinnant's algorithm).
    let z = i64::try_from(days).unwrap_or(0) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{se:02}Z")
}

fn git_sha() -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn hostname() -> String {
    let env_override = std::env::var("REFLEX_BENCH_HOST").ok();
    let uname = std::process::Command::new("uname")
        .arg("-n")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    host_label(env_override.as_deref(), uname.as_deref())
}

/// Resolve the run's host label (Issue 018 T5). `REFLEX_BENCH_HOST`
/// overrides the machine's `uname -n`: the bench-site merge keys rows by
/// host, and a uname name is not self-describing there (`shikuwa` says
/// nothing about silicon or platform; `4090-windows` does). An empty or
/// whitespace-only override is IGNORED (falls back to uname), never an
/// empty label — a blank host row would silently corrupt the merge key.
fn host_label(env_override: Option<&str>, uname: Option<&str>) -> String {
    match env_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(h) => h.to_string(),
        None => match uname.map(str::trim).filter(|s| !s.is_empty()) {
            Some(n) => n.to_string(),
            None => "unknown".to_string(),
        },
    }
}

#[cfg(test)]
mod host_label_tests {
    //! Issue 018 T5 — the host-label override law. Pure over its inputs so
    //! the precedence (override > uname > "unknown") and the blank-guard
    //! are pinned without touching process env.

    use super::host_label;

    #[test]
    fn override_wins_over_uname() {
        assert_eq!(host_label(Some("4090-windows"), Some("shikuwa")), "4090-windows");
    }

    #[test]
    fn blank_override_falls_back_to_uname() {
        assert_eq!(host_label(Some("   "), Some("shikuwa")), "shikuwa");
        assert_eq!(host_label(Some(""), Some("shikuwa")), "shikuwa");
    }

    #[test]
    fn no_inputs_is_unknown_never_empty() {
        assert_eq!(host_label(None, None), "unknown");
        assert_eq!(host_label(None, Some("  ")), "unknown");
    }
}

#[cfg(test)]
mod threshold_migration_tests {
    //! Issue 009 T2 — the byte-identical migration arm. The runner's old
    //! inline `quantile` law is the frozen oracle here (deleted from the
    //! production path by the migration); the surface's Percentile posture
    //! and thin-support fallback must reproduce it exactly.

    use super::*;
    use crate::engine::THIN_SUPPORT_FLOOR;

    /// The runner's pre-T2 `quantile`, frozen verbatim (the migration
    /// oracle — T1's `percentile_parity_with_the_harness_quantile_law`
    /// gate pins the same law from the engine side).
    fn old_quantile_law(sample: &[f32], q: f64) -> f32 {
        let mut s = sample.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        if s.is_empty() {
            return 0.0;
        }
        let idx = ((s.len() as f64) * q) as usize;
        s[idx.min(s.len() - 1)]
    }

    fn obs(scores: &[f32]) -> Vec<GateObservation> {
        scores
            .iter()
            .map(|&s| GateObservation {
                score: s,
                correct: true,
            })
            .collect()
    }

    #[test]
    fn percentile_surface_matches_the_frozen_runner_law() {
        let fixtures: Vec<Vec<f32>> = vec![
            (0..16).map(|i| 0.01 + i as f32 * 0.06).collect(), // n=16 boundary
            (0..64).map(|i| 0.02 + i as f32 * 0.015).collect(),
            vec![0.5f32; 40], // all-ties
            {
                let mut v = (0..32).map(|i| i as f32 * 0.03).collect::<Vec<_>>();
                v[7] = f32::NAN; // NaN sorts above +inf under total_cmp
                v
            },
        ];
        for scores in fixtures {
            let o = obs(&scores);
            let rec = recommend_fused_gate(&o, &o, Posture::Percentile { rho: 0.30 });
            let want = old_quantile_law(&scores, 0.30);
            assert_eq!(
                rec.score.unwrap().threshold.to_bits(),
                want.to_bits(),
                "score axis, n={}",
                scores.len()
            );
            assert_eq!(
                rec.distance.unwrap().threshold.to_bits(),
                want.to_bits(),
                "distance axis, n={}",
                scores.len()
            );
        }
    }

    #[test]
    fn thin_support_falls_back_to_the_birth_constants() {
        // n = THIN_SUPPORT_FLOOR - 1 == the old `confs.len() < 16` arm.
        let scores: Vec<f32> = (0..(THIN_SUPPORT_FLOOR - 1) as i32)
            .map(|i| 0.05 + i as f32 * 0.05)
            .collect();
        let o = obs(&scores);
        let rec = recommend_fused_gate(&o, &o, Posture::Percentile { rho: 0.30 });
        assert!(rec.score.is_none() && rec.distance.is_none());
        let defaults = EngineConfig::default();
        assert_eq!(
            rec.score
                .map_or(defaults.score_threshold, |r| r.threshold)
                .to_bits(),
            defaults.score_threshold.to_bits()
        );
        assert_eq!(
            rec.distance
                .map_or(defaults.distance_threshold, |r| r.threshold)
                .to_bits(),
            defaults.distance_threshold.to_bits()
        );
    }

    #[test]
    fn n_equal_floor_fits() {
        let scores: Vec<f32> = (0..THIN_SUPPORT_FLOOR as i32)
            .map(|i| 0.05 + i as f32 * 0.05)
            .collect();
        let rec = recommend_fused_gate(
            &obs(&scores),
            &obs(&scores),
            Posture::Percentile { rho: 0.30 },
        );
        assert!(rec.score.is_some() && rec.distance.is_some());
    }
}

#[cfg(test)]
mod cap_selection_tests {
    //! Issue 013 lever-1 protocol plumb — the selection law + the CLI parse.

    use super::{CapCandidate, DEFAULT_CAL_SELECT_CAPS, parse_cal_select_caps, select_cap};

    fn cands(pairs: &[(usize, f64)]) -> Vec<CapCandidate> {
        pairs
            .iter()
            .map(|&(cap, cal_acc)| CapCandidate { cap, cal_acc })
            .collect()
    }

    #[test]
    fn argmax_wins() {
        let c = cands(&[(8, 0.30), (16, 0.45), (32, 0.41)]);
        assert_eq!(select_cap(&c, 16), 16);
        let c = cands(&[(8, 0.30), (16, 0.41), (32, 0.45)]);
        assert_eq!(select_cap(&c, 16), 32);
    }

    #[test]
    fn exact_tie_prefers_the_registry_default() {
        // Default tied with a smaller and a larger candidate: default wins.
        let c = cands(&[(8, 0.40), (40, 0.40), (128, 0.40)]);
        assert_eq!(select_cap(&c, 40), 40);
    }

    #[test]
    fn exact_tie_without_the_default_prefers_the_smallest() {
        let c = cands(&[(64, 0.40), (128, 0.40), (256, 0.40)]);
        assert_eq!(select_cap(&c, 40), 64);
    }

    #[test]
    fn default_does_not_beat_a_strictly_better_candidate() {
        let c = cands(&[(40, 0.40), (128, 0.50)]);
        assert_eq!(select_cap(&c, 40), 128);
    }

    #[test]
    fn empty_candidates_fall_back_to_the_registry_default() {
        assert_eq!(select_cap(&[], 64), 64);
    }

    #[test]
    fn bare_flag_parses_the_default_ladder() {
        assert_eq!(
            parse_cal_select_caps(None).unwrap(),
            DEFAULT_CAL_SELECT_CAPS.to_vec()
        );
    }

    #[test]
    fn list_parses_and_trims() {
        assert_eq!(
            parse_cal_select_caps(Some(" 8 , 128 ")).unwrap(),
            vec![8, 128]
        );
    }

    #[test]
    fn bad_and_zero_candidates_refuse() {
        assert!(parse_cal_select_caps(Some("8,x")).is_err());
        assert!(parse_cal_select_caps(Some("0")).is_err());
        assert!(parse_cal_select_caps(Some("")).is_err());
    }
}

#[cfg(all(test, feature = "clm-lane"))]
mod clm_request_tests {
    //! Issue 019 T3 — the CLM lane's harness-side request builder applies
    //! THEIR rendering law over OUR case shape: the state prose via the
    //! byte-pinned `to_text`, choice candidates = the criterion
    //! DESCRIPTIONS (their `candidates` law), score candidates = the
    //! rubric levels, noul = no options.
    use super::*;
    use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
    use serde_json::json;

    fn case(state: Value, questions: Vec<SuiteQuestion>) -> SuiteCase {
        let n = questions.len();
        SuiteCase {
            id: "t".to_string(),
            state,
            questions,
            gold: vec![crate::harness::suites::GoldAnswer {
                idx: 0,
                soft: vec![],
                gold_score: None,
            }; n],
        }
    }

    #[test]
    fn choice_options_are_the_descriptions_in_label_order() {
        let mut crit = serde_json::Map::new();
        crit.insert("world".to_string(), Value::String("world news".into()));
        crit.insert("sports".to_string(), Value::Null); // no desc → the key
        let q = SuiteQuestion {
            qid: "topic".to_string(),
            kind: QKind::Choice,
            instructions: "What is the topic?".to_string(),
            criteria: Value::Object(crit),
        };
        let req = clm_request(&case(json!({ "article": "x" }), vec![q])).unwrap();
        assert_eq!(req.state, "article: x"); // their to_text prose law
        assert_eq!(req.questions.len(), 1);
        assert_eq!(req.questions[0].options, vec!["world news", "sports"]); // descs, keys-as-fallback
        assert_eq!(req.questions[0].kind.as_str(), "choice");
    }

    #[test]
    fn score_options_are_the_rubric_levels() {
        let q = SuiteQuestion {
            qid: "sentiment".to_string(),
            kind: QKind::Score,
            instructions: "How positive?".to_string(),
            criteria: json!(["negative", "neutral", "positive"]),
        };
        let req = clm_request(&case(json!("plain state"), vec![q])).unwrap();
        // A bare-string state passes through their to_text unchanged.
        assert_eq!(req.state, "plain state");
        assert_eq!(req.questions[0].options, vec!["negative", "neutral", "positive"]);
    }

    #[test]
    fn noul_carries_no_options() {
        let q = SuiteQuestion {
            qid: "holds".to_string(),
            kind: QKind::Noul,
            instructions: "Is it true?".to_string(),
            criteria: Value::Null,
        };
        let req = clm_request(&case(json!(1), vec![q])).unwrap();
        assert!(req.questions[0].options.is_empty());
    }
}
