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
    AbstainCause, DecisionEngine, EngineConfig, ExpertSpec, FusedGateRecommendation,
    GateObservation, Posture, Scratch, recommend_fused_gate,
};
use crate::harness::families::SynthData;
use crate::harness::latency::{LatencyExtremes, fmt_p99_cell};
use crate::harness::metrics::{
    CalibrationPair, ConfusionRow, G1Verdict, HardMetrics, brier_of, conformal_naive_floor,
    confusion_top, ece_of, g1_verdict_of, hard_metrics, score_metrics, soft_metrics,
    subset_accuracy,
};
use crate::harness::metrics::{chance_corrected_skill, chance_majority};
use crate::harness::pair_heads::{
    ArmedPair, MAX_ARMED_PAIRS, MIN_CAL_SUPPORT, PairHead, PairHeadAb, PairSubsetRow, select_pairs,
};
#[cfg(feature = "option_cond")]
use crate::harness::suites::typed_gold_events;
use crate::harness::suites::{
    QKind, Suite, SuiteCase, SuiteQuestion, TrainDoc, build_ag_news, build_anli_jdi,
    build_banking77_mteb, build_emotion, build_hellaswag_jdi, build_massive_intent_en,
    build_mmlupro_jdi, build_prompt_injections, build_s1mb_choice, build_s1mb_noul,
    build_s1mb_score, build_sst5, build_thai_sib200, build_thai_wisesight, build_typed_decisions,
    build_wanli_en, build_winogrande_jdi, build_xnli_en, stratified_selection_slices,
    stratified_split, train_docs, train_row_label,
};
use crate::pyjson::serialize_state;
use crate::readout::ReadoutMode;
use katgpt_core::sigmoid_calibration::SigmoidGateCalibrator;

/// Issue 038: the count-table lever's selection + the separate transductive column.
mod nb_lane;
// The genome internals are consumed only behind `nb_scope`; the mod itself
// stays compiled in every posture (the `GenomeSelection` type is named
// unconditionally by the report field + the flag-off None binding).
#[cfg_attr(not(feature = "nb_scope"), allow(dead_code))]
mod genome_lane;
#[cfg(feature = "option_cond")]
mod oc_lane;
#[cfg(feature = "nb_ridge")]
mod ridge_lane;
// Issue 044 T3: the modelless NLI pair-feature head A/B (report-only;
// the mod is compiled in every posture — the record type is named
// unconditionally by the report field + the flag-off None binding).
mod nli_lane;
// Issue 047 M1: the xnli VALIDATION-slice reopen lane (pre-registered by
// .plans/006_nli_m1_reopen.md; same compile posture as nli_lane).
mod nli_m1;
pub use nb_lane::{NbCandidate, NbSelection, TRANSDUCTIVE_PROTOCOL, TransductiveReport};
pub use nli_lane::NliFeatureAb;
pub use nli_m1::M1Result;
#[cfg(feature = "option_cond")]
pub use oc_lane::{OcCandidate, OcSelection};
#[cfg(feature = "nb_ridge")]
pub use ridge_lane::{RidgeCandidate, RidgeSelection};

/// Issue 005 (riir-instinct) E0 — the count-table evidence-density
/// measurement (T1, reflex-only). Lives beside [`nb_lane`] as a child
/// module so it reuses [`prepare`], [`selection_slice`] and the suite
/// registry without publishing them.
#[cfg(feature = "nb_scope")]
mod e0;
#[cfg(feature = "nb_scope")]
pub use e0::{E0Meta, E0Output, E0Suite, E0ViewStats, render_e0_markdown, run_e0};

mod distill;
pub use distill::{DistillMeta, DistillOutput, DistillSuite, render_distill_markdown, run_distill};

mod router;
pub use router::{RouterReport, RouterSuite, render_router_markdown, run_teacher_router};

mod ensemble;
pub use ensemble::{
    EnsembleGateMeta, EnsembleGateOutput, EnsembleGateSuite, render_ensemble_gate_markdown,
    run_ensemble_gate,
};

mod synth;
pub use synth::{
    SynthArtifactMeta, SynthOptions, SynthOutput, load_synth_corpus, render_synth_markdown,
    run_synth_corpus, run_synth_plan,
};

mod corpus_ab;
pub use corpus_ab::{CorpusAbOutput, render_corpus_ab_markdown, run_corpus_ab};

// Issue 077 — the option-PERMUTATION spread probe (lane-agnostic over the
// `ChoiceOracle` seam; the pure core is `crate::harness::permutation`).
mod perm_probe;
pub use crate::harness::permutation::SpreadVerdict;
pub use perm_probe::{
    CaseRow, DEFAULT_PERM_K, DEFAULT_PERM_MAX_CASES, PermLaneReport, PermProbeMeta,
    PermProbeOptions, PermProbeOutput, PermSuiteReport, render_perm_probe_markdown, run_perm_probe,
};

// Issue 090 A1–A3 — the decoy-augmentation / null-stability probe: the
// pure core (`harness::null_stability`) + this runner half behind the
// `NullOracle` seam (modelless control + opt-in comparison cells).
mod null_probe;
pub use crate::harness::null_stability::{DEFAULT_NULL_K, DEFAULT_NULL_MAX_CASES, NullVerdict};
pub use null_probe::{
    NullLaneReport, NullProbeMeta, NullProbeOptions, NullProbeOutput, NullSuiteReport,
    render_null_probe_markdown, run_null_probe,
};

// Issue 094 T2 — the residual-stability depth probe (riir-infer Issue 1106's
// consumer): report-only, CPU-budgeted, behind `twt_profile`.
#[cfg(feature = "twt_profile")]
mod depth_probe;
#[cfg(feature = "twt_profile")]
pub use depth_probe::{
    DepthProbeOptions, DepthProbeOutput, DepthRow, DepthSuiteRow, render_depth_probe_markdown,
    run_depth_probe,
};
// Issue 094 T3 — the clef HEADFUL confidence-candidate probe: the local
// ANE lane's last-position vocabulary entropy against BOTH floors
// (conformal + modal). Report-only, macOS + `clef-ane` only.
#[cfg(all(feature = "clef-ane", target_os = "macos"))]
mod clef_conf_probe;
#[cfg(all(feature = "clef-ane", target_os = "macos"))]
pub use clef_conf_probe::{
    ClefConfProbeOptions, ClefConfProbeOutput, ClefConfRow, ClefConfSuiteRow,
    render_clef_conf_probe_markdown, run_clef_conf_probe,
};

// Issue 090 C1 — the rubric-covariance + route-agreement probe: the pure
// core (`harness::rubric`) + this runner half (modelless control; the
// wire surface renders through the serve edge's own translate → decide →
// map_answers path).
mod rubric_probe;
pub use rubric_probe::{
    DEFAULT_RUBRIC_MAX_CASES, DEFAULT_RUBRIC_SUITES, RubricProbeOptions, RubricProbeOutput,
    RubricSuiteReport, RubricVerdict, render_rubric_markdown, run_rubric_probe,
};

mod density_pilot;
mod echo_gates;
pub use density_pilot::{
    DensityPilotMeta, DensityPilotOutput, DensityPilotSuite, DensityRung,
    render_density_pilot_markdown, run_density_pilot,
};
mod gate_drift;
pub use gate_drift::{
    DEFAULT_MAX_IN_PER_SUITE, DEFAULT_MAX_OFF_PER_SOURCE, GATE_DRIFT_FAMILY,
    GATE_DRIFT_OFF_SOURCES, GateDriftMeta, GateDriftOptions, GateDriftOutput, GateDriftSizeCell,
    render_gate_drift_markdown, run_gate_drift,
};

// Issue 098 T1: the comparison-lane bodies carved out of this file - the
// laya/CLM/jsonl-oracle/GLiNER/Bekko/AgentJev/clef lanes in `lanes_wire`,
// the hosted HTTP lanes + the TeacherWireLane family in `lanes_hosted`.
// Call sites below keep the bare names via the glob re-imports; the two
// explicit re-exports hold the external paths (`parse_python_answer` for
// tests/harness_units.rs, `JsonlOracle` for distill's `super::` paths).
mod lanes_hosted;
mod lanes_wire;
use self::lanes_hosted::*;
pub(crate) use self::lanes_wire::JsonlOracle;
pub use self::lanes_wire::parse_python_answer;
mod report;
pub use self::report::render_markdown;

// Issue 098 T3b: the run-loop phase fns (validation → per-suite
// orchestration → meta) carved out of this file; the re-export keeps
// the public `runner::run` path unchanged.
mod run_phases;
pub use self::run_phases::run;

use self::lanes_wire::*;

/// Where the fetch layer leaves the row files.
pub const DEFAULT_DATASETS_DIR: &str = ".raw/datasets";

/// The SplitMix64 option seed for MASSIVE (any fixed value; reproducibility
/// is what matters, not the value).
pub const MASSIVE_OPTION_SEED: u64 = 0x0603_2026_0922;

// ── suite registry ──────────────────────────────────────────────────────

struct SuiteSpec {
    name: &'static str,
    /// The suite's data home on disk (defaults to `name`; the Issue-047
    /// validation suite shares `xnli_en`'s dir, whose manifest digests
    /// stay keyed there).
    dataset_dir: &'static str,
    /// Test-row eval cap (0 = all rows on disk).
    test_cap: usize,
    /// Calibration-slice cap (train rows reused as cases; 0 = none).
    cal_cap: usize,
    corpus_cap_per_label: usize,
    /// The eval split's file name on disk ("test" everywhere except the
    /// Issue-047 validation-confirmation suite, which reads `validation`
    /// and never touches the spent test split — R1).
    eval_split: &'static str,
    /// Runs ONLY when named in `--suites` — never a default-run member
    /// (a 2490-case confirmation slice would tax every default run, and
    /// its laya seat would be meaningless spend).
    named_only: bool,
    /// false = the suite arms its engine over a universe that DELIBERATELY
    /// exceeds the gold space (Plan 010's s1mb variable-option breadth:
    /// long-tail keys are presented, rarely gold) — the slice-integrity
    /// audit's label-coverage law is scoped OFF while its identity checks
    /// stay ON. Starved labels ride the Issue-039 fallback, disclosed.
    coverage_audit: bool,
    build: fn(&Value, usize) -> Suite,
    /// Some(build) = in-process synthetic suite (Issue 004): no dataset
    /// files — `prepare` never touches `dir`, the builder self-splits
    /// corpus/cal/eval, and the caps above are ignored.
    synthetic: Option<fn() -> SynthData>,
    /// false = the modelless lane has no honest answer for the suite —
    /// the runner skips it LOUDLY (an absence line, never a silent zero)
    /// when no LLM lane is compiled. All six harness families are true
    /// since Issue 045; the flag stays as the declared-inability
    /// machinery for any future suite that needs it.
    modelless_lane: bool,
}

// Plan 010's shared S1MB registry block (three suites by decision kind, the
// generic build_s1mb_* builders; the caps' shape: cal 200 like the big
// suites, corpus 16/label — the choice suite's option-key union is hundreds
// wide, so per-label caps bound the corpus without starving frequent keys;
// noul's two labels × 64 = 128, its train half carries 6,182). All three
// are `named_only` — 13k test questions must never tax a default run.
macro_rules! s1mb_spec {
    ($name:literal, $corpus_cap:literal, $build:path) => {
        SuiteSpec {
            name: $name,
            dataset_dir: $name,
            test_cap: 0, // the split IS the protocol half (converter-written)
            cal_cap: 200,
            corpus_cap_per_label: $corpus_cap,
            eval_split: "test",
            named_only: true,
            coverage_audit: false, // the presented universe exceeds the gold space (breadth suite)
            build: $build,
            synthetic: None,
            modelless_lane: true,
        }
    };
}

/// Per-suite registry. Domain counts are pinned to the FETCHED test rows
/// (dataset_manifest.md); the runner asserts them at build time so a
/// silently different fetch fails loud instead of mis-arming the engine.
const SUITES: &[SuiteSpec] = &[
    // Plan 010: the S1MB lane (named-only; s1mb_fetch_convert.py writes the
    // splits). First so the trio reads together.
    s1mb_spec!("s1mb_choice", 16, build_s1mb_choice),
    s1mb_spec!("s1mb_noul", 64, build_s1mb_noul),
    s1mb_spec!("s1mb_score", 16, build_s1mb_score),
    SuiteSpec {
        name: "typed_decisions",
        dataset_dir: "typed_decisions",
        test_cap: 0, // all 400 — no sampling (protocols §3.1)
        cal_cap: 100,
        corpus_cap_per_label: 48,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_typed_decisions,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "ag_news",
        dataset_dir: "ag_news",
        test_cap: 400,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_ag_news,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "emotion",
        dataset_dir: "emotion",
        test_cap: 400,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_emotion,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "sst5",
        dataset_dir: "sst5",
        test_cap: 600,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_sst5,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "prompt_injections",
        dataset_dir: "prompt_injections",
        test_cap: 0, // all 116
        cal_cap: 100,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_prompt_injections,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "xnli_en",
        dataset_dir: "xnli_en",
        test_cap: 300,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_xnli_en,
        synthetic: None,
        modelless_lane: true,
    },
    // Issue 047 R1: the M1 reopen lane's confirmation surface — the FULL
    // xnli_en VALIDATION split (~2490 rows), read once, pre-registered by
    // .plans/006_nli_m1_reopen.md. `named_only` keeps it out of default
    // runs; the spent 300-item test split is never loaded (eval_split).
    SuiteSpec {
        name: "xnli_en_val",
        dataset_dir: "xnli_en",
        test_cap: 0, // all validation rows — no sampling
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "validation",
        named_only: true,
        coverage_audit: true,
        build: build_xnli_en,
        synthetic: None,
        modelless_lane: true,
    },
    // Issue 080 — the ESC re-source NLI lane (rethink Issue 024 / Research
    // 002): WANLI re-enters the xnli_en niche under a clean CC BY grant.
    // `named_only` (the thai_* precedent — an opt-in via `--suites`, never a
    // default-run member); the eval cap rides the stratified round-robin
    // over the WHOLE 5,000-row test split (the banking77 posture), the pool
    // floor sits on the TRAIN_CAP=20000 pull (slice_guard 16_000).
    SuiteSpec {
        name: "wanli_en",
        dataset_dir: "wanli_en",
        test_cap: 300,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: true,
        coverage_audit: true,
        build: build_wanli_en,
        synthetic: None,
        modelless_lane: true,
    },
    // Plan 012 Phase A (the JDI panel tranche 1; reflex issue 087 closed
    // 2026-10-14 → HISTORY.md): the four
    // choice-shaped public board rows whose official splits ARE the board's
    // `cases` counts (1267 / 10042 / 3200 / 12032). `named_only` (the
    // wanli/thai precedent — an opt-in via `--suites`, never a default-run
    // member; Phase B runs them explicitly). Option keys are POSITIONS with
    // the texts riding the criteria descriptions (the xnli pattern), except
    // anli_jdi which keeps the real XNLI_KEYS class labels (macro-F1 clean —
    // and macro-F1 IS the board's metric there). MMLU-Pro's cal pool is the
    // 70-row validation split (no train split exists), so its cal_cap stays
    // UNDER 70 — a 200-row cal front would identity-split the pool to EMPTY
    // and the corpus guard would refuse the suite (the pool-after-cal law).
    SuiteSpec {
        name: "winogrande_jdi",
        dataset_dir: "winogrande_jdi",
        test_cap: 0, // all 1267 validation rows — the board's population
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "validation",
        named_only: true,
        coverage_audit: true,
        build: build_winogrande_jdi,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "hellaswag_jdi",
        dataset_dir: "hellaswag_jdi",
        test_cap: 2000, // the B5-disclosed cap; laya-full would be ~10 min
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "validation",
        named_only: true,
        coverage_audit: true,
        build: build_hellaswag_jdi,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "anli_jdi",
        dataset_dir: "anli_jdi",
        test_cap: 0, // all 3200 rows of the r1+r2+r3 union — the board's population
        cal_cap: 200,
        corpus_cap_per_label: 64,
        // The comma-list: load_rows concatenates test_r1 → test_r2 → test_r3
        // in the order given (the board's concatenation order).
        eval_split: "test_r1,test_r2,test_r3",
        named_only: true,
        coverage_audit: true,
        build: build_anli_jdi,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "mmlupro_jdi",
        dataset_dir: "mmlupro_jdi",
        test_cap: 2000,
        // 32, not 200: the corpus/cal pool is the 70-row validation split
        // (no train split exists) — a 200-row cal front identity-splits the
        // pool to empty and the corpus guard refuses (see the block comment
        // above). The starved corpus is the suite's disclosed by-construction
        // condition.
        cal_cap: 32,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: true,
        coverage_audit: true,
        build: build_mmlupro_jdi,
        synthetic: None,
        modelless_lane: true,
    },
    // Plan 003 T3.2 — the Thai probe lanes (opt-in via `--suites thai_*`,
    // G-ISO-2: never a default-run member; G-ISO-1/3/4 untouched — the
    // suites add zero runtime-code surface, the builders are pure harness
    // code reachable only from the `harness` bin).
    SuiteSpec {
        name: "thai_wisesight",
        dataset_dir: "thai_wisesight",
        test_cap: 400,
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: true,
        coverage_audit: true,
        build: build_thai_wisesight,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "thai_sib200",
        dataset_dir: "thai_sib200",
        test_cap: 0, // all 204 rows — the whole-set probe
        cal_cap: 200,
        corpus_cap_per_label: 64,
        eval_split: "test",
        named_only: true,
        coverage_audit: true,
        build: build_thai_sib200,
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "massive_intent_en",
        dataset_dir: "massive_intent_en",
        test_cap: 300,
        cal_cap: 200,
        corpus_cap_per_label: 48,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: |v, n| build_massive_intent_en(v, n, MASSIVE_OPTION_SEED),
        synthetic: None,
        modelless_lane: true,
    },
    SuiteSpec {
        name: "banking77",
        dataset_dir: "banking77",
        test_cap: 500,
        cal_cap: 200,
        corpus_cap_per_label: 40,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: build_banking77_mteb,
        synthetic: None,
        modelless_lane: true,
    },
    // ── RETIRED (owner call, 2026-10-02): the six Issue-004 harness
    // decision-point families (`harness_visibility` · `harness_permissions`
    // · `harness_tool_fit` · `harness_routing` · `harness_sensitivity` ·
    // `harness_cache_reuse`) are REMOVED — home-made synthetic evals the
    // modelless engine reads at chance on at the honest wide-eval
    // populations. Nobody benches them anymore; the final readings live in
    // git history and the instinct/reflex-site records.
    // ── RETIRED (owner call, 2026-10-11): `code_fixtures` — the frozen
    // in-process population harvested from this repo's own fn spans — is
    // REMOVED the same way (the suite read off-chance on the modelless
    // lane and carried the public board's noisiest comparison cell).
    // Issue 061: the code-defect family (perch-derived taxonomy, original
    // Rust fixtures; choice over clean + five defect classes).
    SuiteSpec {
        name: "semantic_defects",
        dataset_dir: "semantic_defects",
        test_cap: 0,
        cal_cap: 0,
        corpus_cap_per_label: usize::MAX,
        eval_split: "test",
        named_only: false,
        coverage_audit: true,
        build: synthetic_build_unused,
        synthetic: Some(families_synth_semantic_defects),
        modelless_lane: true,
    },
];

/// Never called — synthetic suites build via their `synthetic` fn; this
/// placeholder keeps the `build` field total.
fn synthetic_build_unused(_rows: &Value, _max_rows: usize) -> Suite {
    unreachable!("synthetic suites build via SuiteSpec::synthetic")
}

/// The synthetic (in-process fixture) suite names, derived from the
/// registry — never hand-typed. The fixture-fleet hygiene audit's
/// population: a new synthetic suite joins it by EXISTING, and the audit
/// reds until that suite carries hygiene facts (the 12th-instance wall —
/// every fixture set before the families was authored ungated and
/// measured its overlap only after the inflation was published).
pub fn synthetic_suite_names() -> Vec<&'static str> {
    SUITES
        .iter()
        .filter(|s| s.synthetic.is_some())
        .map(|s| s.name)
        .collect()
}

fn families_synth_semantic_defects() -> SynthData {
    crate::harness::families::synth_semantic_defects()
}

/// Checkpoints per suite (laya lane): the specialist `typed` model answers
/// the typed-decisions headline alongside the two base checkpoints; every
/// other suite is the general `english` checkpoint (the reference's own
/// layout — T4 ran base checkpoints on the English suites and `typed` on
/// typed-decisions) — EXCEPT the Thai probe suites (Plan 003 T3.3): the
/// board names laya-MULTILINGUAL, the one checkpoint that answers Thai
/// script (the frozen `ml-thai` G5 row; english collapses per
/// `ml-thai-collapse`, so an english column there measures the collapse,
/// not the capability).
fn laya_checkpoints_for(suite: &str) -> &'static [&'static str] {
    if suite == "typed_decisions" {
        &["typed", "english", "multilingual"]
    } else if suite.starts_with("thai_") {
        &["multilingual"]
    } else {
        &["english"]
    }
}

// ── RETIRED (owner call, 2026-10-11): the code_fixtures builders
// (`code_module_labels`, `code_case`, `build_code_fixtures`,
// `code_fixtures_cal_cases`, `code_fixtures_docs`) and the frozen
// population they read (`crate::harness::code_frozen`) are REMOVED with
// the suite. The measured history: Bench 001 addenda + 044 (the frozen
// population arc, incl. the commit-relative harvest lesson) + 087 (the
// per-question-shape PAW programs) + 116.

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
    let suite_dir = datasets_dir.join(spec.dataset_dir);
    let eval = load_rows(&suite_dir, spec.eval_split)?;
    let train = load_rows(&suite_dir, "train").map_err(|e| format!("suite {}: {e}", spec.name))?;
    Ok(serde_json::json!({
        "suite": spec.name,
        "test_rows": eval,
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

/// Issue 060: per-suite abstain-cause counts over the CLOSED taxonomy
/// (score gate / distance gate / density gate / grammar-invalid).
/// Computed from the CALIBRATED gate's per-question causes — the gate
/// behind the published `calibrated_abstain` rate — so the shares sum to
/// that metric's abstaining-question count. `grammar_invalid` is the
/// reserved serve-lane arm (the game-heads fall-through): zero on every
/// harness run (each wire question is grammar-valid by contract),
/// carried so the taxonomy's wire shape is fixed. `density_gate`
/// (Issue 066) counts the density half's marginal arm — nonzero only on
/// a `--density-gate` armed read, and SERIALIZED ONLY THEN (skip at
/// zero): every shipped row's wire shape stays the three-key closed
/// taxonomy byte-identically (the Issue-060 law — a fourth key would be
/// a wire change, so the fourth key rides only the arm that produces
/// it). Cells that predate a field publish as `None` (unattributed —
/// the page renders "not recorded", never guessed).
#[derive(Debug, Clone, Default, Serialize)]
pub struct AbstainCauses {
    pub score_gate: usize,
    pub distance_gate: usize,
    /// Issue 066 — see the struct docs for the skip-at-zero wire law.
    #[serde(skip_serializing_if = "is_zero")]
    pub density_gate: usize,
    pub grammar_invalid: usize,
}

/// `true` when `n == 0` — the `density_gate` key's serde skip predicate.
fn is_zero(n: &usize) -> bool {
    *n == 0
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
    /// Issue 073 T4: Brier over the same (confidence, correctness) pairs
    /// `readout_ece` reads — the second calibration axis for lanes whose
    /// vendor confidence has a calibration CLAIM to test (Drex's card
    /// disclaims what their homepage markets). Populated by the shared
    /// assembler for every lane that carries a readout; None on lanes
    /// assembled without one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readout_brier: Option<f64>,
    /// Abstain behavior (modelless only — laya cannot abstain; the
    /// Research-562 flaw the wire fixes with first-class abstention).
    pub raw_abstain: Option<SelectiveMetrics>,
    pub calibrated_abstain: Option<SelectiveMetrics>,
    /// Issue 060: the calibrated gate's abstain-cause shares (the closed
    /// taxonomy) — the WHY behind `calibrated_abstain`'s rate. `None` when
    /// the lane cannot abstain (laya) or the cell predates the field.
    pub abstain_causes: Option<AbstainCauses>,
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
    /// Issue 076 T2 (from Bench 125's instrumentation): the comparison
    /// lanes' confidence-vs-correctness readout — ECE/Brier of THEIR wire
    /// `confidence` fields per kind (choice + score; the wire carries no
    /// noul confidence — the null cell is the disclosure) beside the
    /// split-half conformal-naive floor companion. Computed through
    /// `harness::metrics` only (`assemble_drex_conf_readout` — the ONE
    /// shared math shape; the drex-named field is the historical spelling
    /// and carries the d1 lane's cells too, Issue 078). `None` for every
    /// other lane — never a fabricated row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drex_conf_readout: Option<crate::lanes::drex::DrexConfReadout>,
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
    /// How many cases the repeat-run check re-ran (the verdict's support —
    /// plan 001 task 9's repeat-count ask; `None` when the check did not run).
    pub determinism_n: Option<usize>,
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
    /// The cal-selected fitted-head posture (issue 030 lever 4) — per
    /// candidate cal accuracy + the selected scale. None = no selection
    /// ran (the flag was off, the suite is ineligible, or the lane has no
    /// cal slice / no corpus).
    pub head_selection: Option<HeadScaleSelection>,
    /// The cal-selected count-table posture (issue 038, `--nb-select`).
    /// None = no selection ran (flag off / ineligible suite / laya lanes).
    pub nb_selection: Option<NbSelection>,
    /// The cal-selected option-conditioned posture (issue 038 T7b,
    /// `--oc-select`). None = no selection ran (flag off / ineligible
    /// suite / the feature is compiled out).
    #[cfg(feature = "option_cond")]
    pub oc_selection: Option<oc_lane::OcSelection>,
    /// The cal-selected ridge-readout posture (issue 038 T7a,
    /// `--ridge-select`). None = no selection ran.
    #[cfg(feature = "nb_ridge")]
    pub ridge_selection: Option<ridge_lane::RidgeSelection>,
    /// The cal-selected JOINT blend genome (issue 038 T5,
    /// `--genome-select`): the composed posture's scale coordinates
    /// refined together on the same selection slice. None = no genome
    /// selection ran (flag off / ineligible suite).
    pub genome_selection: Option<genome_lane::GenomeSelection>,
    /// The TRANSDUCTIVE column (issue 038): a different protocol, published
    /// beside `hard.accuracy`, never inside it. None unless the count
    /// tables are armed on this row.
    pub transductive: Option<TransductiveReport>,
    /// Top-K categorical (gold → pred) confusion pairs, forced raw eval
    /// (modelless only; Issue 013 lever-3 probe instrumentation — report-
    /// only, never a gate). None on the laya lanes.
    pub confusion: Option<Vec<ConfusionRow>>,
    /// The lever-3 pair-head A/B records, `(top2-gated, pred-anchored)`
    /// (present only when the `--pair-head-ab` arm ran). None otherwise,
    /// and on the laya lanes.
    pub pair_head_ab: Option<(PairHeadAb, PairHeadAb)>,
    /// Issue 044 T3: the modelless NLI pair-feature head A/B record
    /// (report-only, `--nli-feature-ab`; xnli-shaped suites only). None
    /// otherwise, and on the laya lanes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nli_feature_ab: Option<nli_lane::NliFeatureAb>,
    /// Issue 047 M1: the xnli VALIDATION-slice reopen record
    /// (`--nli-m1`; pre-registered by .plans/006_nli_m1_reopen.md —
    /// per-item pick log included, R5). None otherwise, and on the laya
    /// lanes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nli_m1: Option<nli_m1::M1Result>,
    /// Issue 055 / Plan 008 T5 (`--mc-ab`): the seeded-MC distributional-layer
    /// A/B record — rejection ranking (u_pair vs the fused-gate confidence
    /// at matched coverage) + the histogram's decision rules (majority /
    /// mean / LCB-λ), knobs selected cal-side. Report-only: the served pick
    /// stays the legacy pipeline's. None = the arm did not run (flag off /
    /// the laya lanes / the feature compiled out).
    #[cfg(feature = "mc_ensemble")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc_ab: Option<super::mc_ab::McAbRecord>,
    /// Issue 066 (`--density-gate`, `density_gate` feature): the fused-
    /// abstain DENSITY half's paired A/B record — the density-armed gate
    /// vs the shipped calibrated gate over ONE frozen test read, with the
    /// marginal-slice decomposition (what the new axis removed) and the
    /// two-proportion LCB95 on Δ selective accuracy. Report-only: the
    /// published row stays the shipped posture's; adoption is a separate
    /// flip gated on the paired verdict. None = the arm did not run (flag
    /// off / the feature compiled out / the laya lanes).
    #[cfg(feature = "density_gate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density_ab: Option<super::density_ab::DensityAbRecord>,
    /// Plan 627 / Issue 090 C2: the `--set-refine-ab` paired A/B record —
    /// forced-pick accuracy both arms, flip decomposition (to-correct /
    /// to-wrong), Δ accuracy + one-sided-95% paired LCB. Report-only: the
    /// published row stays the shipped posture's; adoption is the
    /// plan-627 GOAT verdict, never a default change riding a measurement
    /// pass. None = the arm did not run (flag off / feature compiled out /
    /// the laya lanes).
    #[cfg(feature = "set_refine")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_refine_ab: Option<super::set_refine_ab::SetRefineAbRecord>,
    /// Issue 024 T3: hard accuracy over the eval rows NOT leak-flagged —
    /// the same forced-row walk `hard` reads, restricted to the unflagged
    /// cases. None = feature off / suite out of scope / every row flagged
    /// (an absence, never a fabricated rate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_deleaked: Option<f64>,
    /// Plan 011 §B3 — the JDI chance level (majority gold-class share) of
    /// the slice this row evaluated, and the chance-corrected skill
    /// `(hard.accuracy − chance)/(1 − chance)` clipped `[0,1]`. Reference
    /// columns for the JDI crosswalk (NEVER folded into `hard.accuracy`);
    /// None on rows without forced picks. See [`chance_majority`] /
    /// [`chance_corrected_skill`] in `super::metrics`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jdi_chance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jdi_skill: Option<f64>,
    /// Issue 039 T4: the confidence-readout CANDIDATE table on the cal
    /// slice — REPORT-ONLY (the arming lever was demoted at Bench 052; the
    /// shipped Dispatch law runs everywhere). None = thin cal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readout_report: Option<ReadoutReport>,
    /// Issue 039 T3: option labels with NO train docs in the corpus pool —
    /// they score against their self-doc fallback (the label text alone).
    /// Empty = the train pull covers the whole label universe (the guard is
    /// silent). Laya lanes never carry one (they read no train rows).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub corpus_fallbacks: Vec<String>,
    /// Issue 057: the corpus identity of the run that produced this lane's
    /// numbers — the pool the timed engine actually consumed (capped
    /// drafter docs per label, self-doc fallback included, plus the
    /// uncapped count-table sets when the posture arms them). This is the
    /// axis `corpus_cap` cannot see (the typed 800→1200 lift left the cap
    /// echo byte-identical); the publish layer's LANE-CARRY compares it
    /// per cell to decide whether incumbent timing still describes this
    /// lane. None on the laya lanes (they read no train rows). See
    /// [`corpus_digest`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corpus_digest: Option<String>,
    /// Issue 072 (`--dump-items`, rethink 028 T1): the per-item outcome
    /// records of this lane — ALWAYS built in memory (the same walk the
    /// metrics read), NEVER serialized into `results.json`; the CLI flag
    /// persists them to `<out>/items/<suite>.jsonl`. Join key across
    /// lanes: `(case_id, q_idx)`.
    #[serde(skip)]
    pub items: Vec<super::item_dump::ItemOutcome>,
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

/// The lane-pairing population identity (riir-reflex Issue 040): a stable
/// tag over the SERVED test cases of one suite — the pair key that makes
/// "is lane A comparable to lane B?" decidable at the publish layer, where
/// lanes arrive from different runs (the live-as-primary merge refreshes
/// lane classes one at a time by design).
///
/// FNV-1a 64 on purpose: `pub mod harness` is ungated, so this module must
/// compile at `--no-default-features`, where blake3 is not in the tree.
/// This is an identity tag for pairing, not a security hash — the property
/// that matters is SPEC-PINNED determinism: byte-identical served cases
/// (same questions, same order, same gold) hash identically on every host,
/// every feature set, every toolchain version.
///
/// Canonical payload: suite name, a separator, then `serde_json` of the
/// case vector. serde_json Map is BTreeMap-backed (sorted keys) and the
/// case/question fields are plain structs — the encoding is deterministic.
fn cases_digest(name: &str, cases: &[crate::harness::suites::SuiteCase]) -> String {
    fn fold(h: &mut u64, bytes: &[u8]) {
        for &b in bytes {
            *h ^= u64::from(b);
            // Issue 088: the multiplier below is NOT the FNV-1a 64 prime —
            // it is 2³⁶+435, a digit transposition of the true
            // `0x0100_0000_01b3`. The `fnv1a64-` prefix is the wire name
            // every recorded digest carries; the fold is deterministic and
            // pin-stable, but canonical-fnv1a64 replication will not match.
            // See the house-fold regression vectors in `suites.rs`.
            *h = h.wrapping_mul(0x0010_0000_01b3);
        }
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    fold(&mut h, name.as_bytes());
    fold(&mut h, &[0]);
    match serde_json::to_vec(cases) {
        Ok(bytes) => fold(&mut h, &bytes),
        // A case vector that cannot serialize (NaN/Inf gold scores) still
        // needs a STABLE identity — hash the reason, never a fresh random.
        Err(e) => fold(&mut h, format!("serde-error:{e}").as_bytes()),
    }
    format!("fnv1a64-{h:016x}")
}

/// The corpus identity of one modelless lane run (Issue 057): what the
/// timed engine actually consumed — the capped drafter docs per label
/// ([`per_label_corpus_docs`], self-doc fallback included) plus, when the
/// posture arms the count tables, the uncapped per-label table sets. This
/// is the axis the corpus-cap echo could not see (the typed 800→1200 lift
/// left `corpus_cap` byte-identical while ~50%'ing the tables): a pool or
/// cap change that moves ANY consumed doc moves this digest.
///
/// FNV-1a 64 on purpose — same law as [`cases_digest`]: `pub mod harness`
/// is ungated, so blake3 is not in the tree at `--no-default-features`;
/// this is an identity tag for pairing, not a security hash. Deterministic
/// across hosts/features/toolchains: the pool construction is
/// manifest-pinned and the fold order is the engine's own label/doc order.
/// ⚠ same Issue-088 caveat as [`cases_digest`]: the `fnv1a64-` prefix is
/// the recorded wire name, the multiplier is the house one (2³⁶+435), not
/// the canonical FNV-1a 64 prime.
fn corpus_digest(
    suite: &str,
    labels: &[String],
    drafter_docs: &[Vec<String>],
    nb_sets: Option<&[Vec<String>]>,
) -> String {
    fn fold(h: &mut u64, bytes: &[u8]) {
        for &b in bytes {
            *h ^= u64::from(b);
            *h = h.wrapping_mul(0x0010_0000_01b3);
        }
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    // Shape tag: a future change to WHAT this digest covers must not
    // silently alias an older shape's values.
    fold(&mut h, b"corpus-v1");
    fold(&mut h, suite.as_bytes());
    fold(&mut h, &[0]);
    for (idx, label) in labels.iter().enumerate() {
        fold(&mut h, label.as_bytes());
        fold(&mut h, &[1]);
        for t in &drafter_docs[idx] {
            fold(&mut h, t.as_bytes());
            fold(&mut h, &[2]);
        }
        fold(&mut h, &[3]);
        if let Some(sets) = nb_sets {
            for t in &sets[idx] {
                fold(&mut h, t.as_bytes());
                fold(&mut h, &[4]);
            }
        }
        fold(&mut h, &[5]);
    }
    format!("fnv1a64-{h:016x}")
}

#[derive(Debug, Clone, Serialize)]
pub struct SuiteResult {
    pub name: String,
    pub n_cases: usize,
    pub n_questions: usize,
    /// Population identity of the served test cases (Issue 040 T1): the
    /// pair key the publish layer uses to refuse comparing lanes that
    /// answered different question sets. See [`cases_digest`].
    pub cases_digest: String,
    /// None = the family has NO modelless lane (Issue 004 T3: LLM-only —
    /// an honest absence, never a fabricated row).
    pub modelless: Option<LaneResult>,
    /// checkpoint name → result (empty when compiled without `laya` or the
    /// lane was disabled — the table prints the honest absence).
    pub laya: BTreeMap<String, LaneResult>,
    /// Issue 038 T4′ — the cascade lane per escalator checkpoint
    /// (`--cascade`): the modelless answers stand; the calibrated fused
    /// gate's abstains escalate to the checkpoint. Empty (never a
    /// fabricated row) when the flag is off / no modelless lane / no laya
    /// row served the suite.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub cascade: BTreeMap<String, super::cascade::CascadeReport>,
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
    /// Bench 103 (owner call 2026-10-01): the Bekko comparison lane's row —
    /// hotchpotch/bekko-system-one-v0 (17M/68M/400M shared-prefix encoders,
    /// no license assigned yet) as a JSONL subprocess oracle over THEIR
    /// `BekkoSentenceTransformer` runtime (`scripts/bekko_lane.py`, the same
    /// laya-python protocol). Absent (never a fabricated row) when the lane
    /// did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bekko: Option<LaneResult>,
    /// Issue 025 amendment 4 / `.issues/027`: the AgentJev comparison lane's
    /// row — their `jev_service` (Apache-2.0, not affiliated) served on
    /// loopback, measured over HTTP. The missing data point this lane owns:
    /// AgentJev's GOLD-LABEL accuracy on our typed_decisions split (their
    /// published 79.25% is teacher-argmax agreement — a different protocol).
    /// Absent (never a fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agentjev: Option<LaneResult>,
    /// Plan 011 Phase A: the Cloudflare Clef comparison lane's row —
    /// `Cloudflare/clef` (+ `clef-flash`, Apache-2.0, not affiliated)
    /// served on Workers AI behind an operator-run loopback TLS forwarder
    /// (plan 011 A0; the extra hop is part of the row's latency claim and
    /// is disclosed in `RunMeta::clef_lane`). Jev-shaped per the vendor;
    /// the wire fixture (plan 011 A2.5) is the compatibility evidence.
    /// Absent (never a fabricated row) when the lane did not run — the
    /// no-creds refusal at lane construction is the gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clef: Option<LaneResult>,
    /// Plan 003 Phase 2: the OpenThai comparison lane's row — their
    /// `openthai_systemone` (Apache-2.0, not affiliated) served on
    /// loopback, measured over HTTP; the family's first Thai+English
    /// specialist. Absent (never a fabricated row) when the lane did not
    /// run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openthai: Option<LaneResult>,
    /// Issue 073 (from `.research/008`): the Drex DLM comparison lane's
    /// row — Nace.AI's open-weights 8B diffusion-LM decision model (CC
    /// BY-NC weights: MEASUREMENT ONLY, never a product lane, never a
    /// distill teacher), their server on loopback, measured over HTTP on
    /// the same TypeSafe `/v1/systemone` wire. Absent (never a fabricated
    /// row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drex: Option<LaneResult>,
    /// Issue 078 (from `.research/009`): the LiquidAI d1 comparison
    /// lane's row — d1-3B (`LiquidAI/d1-3B`, LFM2.5-VL-3B backbone;
    /// license `other`/lfm1.0: MEASUREMENT ONLY posture), our reference
    /// stdlib server over their in-repo `D1Model` on loopback, measured
    /// over HTTP on their official `/decisions/v1/systemone` wire. Absent
    /// (never a fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d1: Option<LaneResult>,
    /// Issue 082: the pplx-decider comparison lane's row — Perplexity's
    /// `pplx-decider-v1.1-27b` (Apache-2.0, UNGATED — the JDI 2026-10-07
    /// #1; teacher-eligible unlike drex/d1) served through their autojev
    /// server on the TypeSafe `/v1/systemone` wire, measured over HTTP.
    /// Absent (never a fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pplx: Option<LaneResult>,
    /// riir-train 629 T2: the vLLM-SR d3 (27B) comparison lane's row — the
    /// JDI board's d3 entrant (Apache-2.0, UNGATED — teacher-eligible;
    /// tag v3.1.0) served through their `d3_server.py` on the SAME
    /// TypeSafe `/v1/systemone` wire the pplx lane speaks (the d3 server
    /// is adapted FROM pplx's autojev server; `src/lanes/d3.rs`), measured
    /// over HTTP. The JDI-panel scoring read (the per-family spot check
    /// vs the board's published per-benchmark numbers — the lossy-surface
    /// law — and the B3 arm's public-panel column). Absent (never a
    /// fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d3: Option<LaneResult>,
    /// reflex Issue 090 B1 (feeding riir-train 632): the Jeva comparison
    /// lane's row — `LIQIIIII/Jeva-4B` (arXiv:2610.18773; Qwen3.5-4B
    /// backbone + 13.37M decision head, the first decision-native PAPER
    /// model; weights license UNDECLARED → measurement + teacher-capture
    /// posture, adoption fail-closed — 632 T0) served by OUR reference
    /// stdlib server (`scripts/jeva_server.py`) over their in-repo
    /// `jeva.Jeva`, measured over HTTP on our split. Absent (never a
    /// fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jeva: Option<LaneResult>,
    /// reflex Issue 095 (feeding riir-train 635 T3): the StartLux-Decision
    /// v1.1 comparison lane's row — board #1 (public_skill 72.88 on the
    /// pinned snapshot; Qwen3.8-27B backbone; weights CC BY-NC 4.0 →
    /// DECLARED-NC RESTRICTED posture, capture-for-measurement +
    /// teacher-capture, adoption fail-closed — 635 T0) served by THEIR
    /// Apache-2.0 engine `startlux-decide` over llama.cpp on the SAME
    /// `/v1/systemone` wire the pplx/d3 lanes speak, measured over HTTP.
    /// Absent (never a fabricated row) when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startlux: Option<LaneResult>,
    /// rethink Issue 032 T10 / Research 012: the mammoth/elephant HOSTED
    /// comparison lane's row — Base Labs' mammoth-1 (the JDI 0.4.1 board-#1
    /// public print 75.34, MIT) or elephant-1 (72.24) on Baseten Model APIs
    /// behind the operator-run loopback TLS forwarder (the clef A0 law; the
    /// hop is part of the row's latency claim), on the SAME `/v1/systemone`
    /// wire. MEASUREMENT-ONLY: no RIDT arm — API outputs never enter a
    /// capture artifact (the T10.2 law). Absent when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mammoth: Option<LaneResult>,
    /// rethink Issue 032 T10.2 / Research 012: the elephant-1 SELF-HOSTED
    /// comparison lane's row — the LOCAL-FIRST law's measurement posture
    /// (our reference `scripts/elephant_server.py` over their qev runtime;
    /// bf16, the block-bidirectional mask armed from the checkpoint).
    /// Capture-CAPABLE (self-hosted weights — the `--distill-teacher
    /// elephant` RIDT arm is the same serve), but this row is the test-split
    /// measurement. Absent when the lane did not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elephant: Option<LaneResult>,
    /// rethink Issue 038 / Research 013: the OpenDecider-nano SELF-HOSTED
    /// comparison lane's row — the third encoder-candidate's P0 admission
    /// rig (Apache-2.0, ~400M Ettin-encoder-400m + MLP decision head) via
    /// their `opendecider serve` on the SAME `/v1/systemone` wire. The P0
    /// gate is pre-registered in 038 (bar 0.7825, one frozen read; the
    /// selection-on-test caveat lives there). Absent when the lane did not
    /// run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opendecider: Option<LaneResult>,
    /// Issue 033: the PAW comparison lane's row (ProgramAsWeights, not
    /// affiliated) — free-text answers under the exact-match law, refusals
    /// counted, no probability surface (`src/lanes/paw.rs`). Absent when
    /// the lane did not run or the suite has no committed spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paw: Option<crate::lanes::paw::PawLaneResult>,
    /// Issue 033 Posture B: the same programs answered through their
    /// LOCAL llama.cpp runtime (`src/lanes/paw_local.rs`) — the
    /// runtime-posture twin of `paw`, determinism-comparable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paw_local: Option<crate::lanes::paw::PawLaneResult>,
    /// Issue 024 T3: the corpus∪cal → eval near-duplicate leak scan over
    /// the slices THIS run served. Absent (not a zero rate) when the
    /// `slice_leak` feature is off or the suite is out of scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leak: Option<SuiteLeak>,
    /// Issue 058: the slice-integrity audit — structural verdict
    /// (`violations` empty = the hard assertions held), per-slice label
    /// starvation disclosures, and the RAW slice-identity digests (test /
    /// cal / pool). The pool digest is the axis that catches a silently
    /// moved/truncated train pull at the same cap — the Bench-076 class.
    /// Absent on the synthetic/code paths (authored slices, not composed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slices: Option<super::slice_guard::SliceFacts>,
    /// Issue 094 T1 — the input-keyed modal-answer floor over this suite's
    /// question population (the leave-one-out anti-mimicry census; the
    /// `render_markdown` block quotes every lane row against it). `None`
    /// only when a suite carries no questions (never observed on a real
    /// suite — an honest absence, never a fabricated row).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modal_floor: Option<super::modal_floor::ModalFloorRow>,
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
    /// Issue 039 T2: the test/cal sampling law (stratified vs first-N).
    pub sampling_protocol: String,
    /// Issue 039 T4: the confidence-readout posture (shipped Dispatch law,
    /// or cal-side selection armed per suite past the margin).
    pub readout_posture: String,
    pub floor_definition: String,
    /// Issue 094 T1: the input-keyed modal-answer floor's construction
    /// (the anti-mimicry axis beside the conformal floor above).
    pub modal_floor_definition: String,
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
    /// Whether the Bekko comparison lane ran (Bench 103), and its serving
    /// posture when it did (hotchpotch/bekko-system-one-v0 over their
    /// `BekkoSentenceTransformer` runtime as a JSONL subprocess — comparison
    /// lane, never a product lane; MIT — verified 2026-10-02).
    pub bekko_lane: String,
    /// Whether the AgentJev comparison lane ran (Issue 025 amendment 4 /
    /// `.issues/027`), and its serving posture when it did (their
    /// `jev_service` on loopback, measured over HTTP — the MEASURE-vs-SERVE
    /// split; comparison lane, never a product lane).
    pub agentjev_lane: String,
    /// Whether the Clef comparison lane ran (plan 011 Phase A), and its
    /// serving posture when it did — including the loopback TLS-forwarder
    /// hop disclosure (plan 011 A0: the hop is part of the row's latency
    /// claim) and the env contract (never a committed credential).
    pub clef_lane: String,
    /// Whether the OpenThai comparison lane ran (Plan 003 Phase 2), and
    /// its serving posture when it did (their `openthai_systemone` on
    /// loopback, measured over HTTP — comparison lane, never a product
    /// lane).
    pub openthai_lane: String,
    /// Whether the Drex DLM comparison lane ran (Issue 073), and its
    /// serving posture when it did (their serve.py / llama.cpp `edlm`
    /// fork on loopback — the reply's own `model` field + DREX_SERVE_URL
    /// disclose which; CC BY-NC weights: measurement only).
    pub drex_lane: String,
    /// Whether the LiquidAI d1 comparison lane ran (Issue 078), and its
    /// serving posture when it did (our reference stdlib server over
    /// their in-repo `D1Model` on loopback — the served dtype + the
    /// reply's own `model` field + D1_SERVE_URL disclose which; lfm1.0
    /// license: measurement only).
    pub d1_lane: String,
    /// Whether the pplx-decider comparison lane ran (Issue 082), and its
    /// serving posture when it did (their autojev server — the model id +
    /// checkpoint basename from /health, PPLX_DEVICE the posture stamp; a
    /// local-serve read, the hosted posture owner-gated and never pooled).
    pub pplx_lane: String,
    /// Whether the vLLM-SR d3 comparison lane ran (riir-train 629 T2), and
    /// its serving posture when it did (their d3_server.py — the model id
    /// from /health, serve version-true with `--name d3-v3.1.0`;
    /// `D3_DEVICE` the quant-posture disclosure stamp, e.g. `cuda:0+nf4`
    /// — the lossy-surface law: the posture is PINNED and quoted beside
    /// every number).
    pub d3_lane: String,
    /// Whether the Jeva comparison lane ran (reflex Issue 090 B1), and
    /// its serving posture when it did (our `scripts/jeva_server.py` —
    /// the model id + RESOLVED dtype + allow_null posture from /health;
    /// weights license UNDECLARED, the 632-T0 restricted posture).
    pub jeva_lane: String,
    /// Whether the StartLux comparison lane ran (reflex Issue 095, feeding
    /// riir-train 635 T3), and its serving posture when it did (their
    /// `startlux-decide` engine over llama.cpp — the bidirectional Instant
    /// posture gate at health; the quant stamp via STARTLUX_DEVICE is the
    /// lossy-surface disclosure; weights CC BY-NC 4.0, the 635-T0
    /// DECLARED-NC RESTRICTED posture).
    pub startlux_lane: String,
    /// Whether the mammoth/elephant hosted comparison lane ran (rethink
    /// Issue 032 T10), and its serving posture when it did (Baseten Model
    /// APIs behind the operator-run loopback TLS forwarder — the hop
    /// disclosed; MIT, MEASUREMENT-ONLY, no RIDT arm).
    pub mammoth_lane: String,
    /// Whether the elephant-1 self-hosted comparison lane ran (rethink
    /// Issue 032 T10.2 — the LOCAL-FIRST law), and its serving posture when
    /// it did (our reference server over their qev runtime; the resolved
    /// engine dtype/attention/fork_mode ride the /health provenance — a
    /// causal serve can never hide, the 635 lesson).
    pub elephant_lane: String,
    /// The opendecider lane's posture disclosure (rethink Issue 038).
    pub opendecider_lane: String,
    /// Whether the PAW comparison lane ran (Issue 033), and its posture
    /// (hosted anonymous / authenticated) when it did.
    pub paw_lane: String,
    /// Whether the PAW Posture B lane ran (Issue 033), and its serving
    /// posture when it did (their local llama.cpp runtime over a Python
    /// subprocess — the runtime-posture twin of `paw_lane`).
    pub paw_local_lane: String,
    /// Power source / power mode / load / swap at run start AND end, with a
    /// latency-quotable verdict (Issue 021 T7) — the axis every Issue 020
    /// A/B was missing.
    pub box_state: super::box_state::BoxStateSpan,
    /// The modelless lane's fitted-head posture (issue 030 lever 4): OFF
    /// at the published baseline, or ON naming the scale — a run at a
    /// non-zero `head_scale` is a different engine posture, disclosed here
    /// so a table is never read against the wrong engine.
    pub head_posture: String,
    /// Issue 038 count-table posture + the transductive-column protocol.
    pub nb_posture: String,
    /// Issue 038 T7b option-conditioned posture (when `--oc-select` ran).
    #[cfg(feature = "option_cond")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub oc_posture: String,
    /// Issue 038 T7a ridge-readout posture (when `--ridge-select` ran).
    #[cfg(feature = "nb_ridge")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ridge_posture: String,
    pub divergences: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RunOutput {
    pub meta: RunMeta,
    pub suites: Vec<SuiteResult>,
}

// ── data loading ────────────────────────────────────────────────────────

/// Concatenate `.rows[].row` across every `<split>-*.json` (or a
/// comma-LIST of splits — Plan 012's `anli_jdi` reads the three-way
/// `test_r1,test_r2,test_r3` union) page file(s), per component in sorted
/// filename order, concatenated component-by-component in the order given,
/// into one synthetic envelope. Every existing caller passes a single split
/// (no comma) and is unchanged.
fn load_rows(dir: &Path, split: &str) -> Result<Value, String> {
    let mut rows: Vec<Value> = Vec::new();
    if split.contains(',') {
        for part in split.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let component =
                load_rows_split(dir, part).map_err(|e| format!("split list [{split}]: {e}"))?;
            rows.extend(component);
        }
    } else {
        rows = load_rows_split(dir, split)?;
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

/// The per-split half of [`load_rows`]: every `{split}-*.json` page under
/// `dir`, sorted, concatenated into bare rows.
fn load_rows_split(dir: &Path, split: &str) -> Result<Vec<Value>, String> {
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
    Ok(rows)
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
) -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String> {
    build_engine_with::<N>(suite, train, labels, cap_per_label, cfg, &[])
}

/// Issue 038's count-table corpus rule, extracted for [`e0`]: every pool
/// doc of the label, UNcapped (table scoring cost is independent of how
/// many docs built it), plus `extra_nb`; a starved label falls back to its
/// own label text — exactly [`build_engine_with`]'s per-label `nb_docs`.
#[cfg(feature = "nb_scope")]
fn nb_doc_sets(train: &[TrainDoc], labels: &[String], extra_nb: &[TrainDoc]) -> Vec<Vec<String>> {
    labels
        .iter()
        .map(|label| {
            let mut nb_docs: Vec<String> = train
                .iter()
                .chain(extra_nb.iter())
                .filter(|d| d.label == *label)
                .map(|d| d.text.clone())
                .collect();
            if nb_docs.is_empty() {
                nb_docs.push(label.clone());
            }
            nb_docs
        })
        .collect()
}

/// The seat-side [`nb_doc_sets`] wrapper: `None` when the config does not
/// arm the count tables OR the `nb_scope` feature is compiled out (the
/// seat's extended build must compile at every feature posture).
#[allow(unused_variables)]
fn nb_sets_for(
    nb_scale: f64,
    train: &[TrainDoc],
    labels: &[String],
    extra: &[TrainDoc],
) -> Option<Vec<Vec<String>>> {
    #[cfg(feature = "nb_scope")]
    {
        (nb_scale > 0.0).then(|| nb_doc_sets(train, labels, extra))
    }
    #[cfg(not(feature = "nb_scope"))]
    {
        None
    }
}

/// [`build_engine`] plus issue 038's count-table corpus: when the config
/// arms `nb_scale`, each label's tables read EVERY pool doc of that label
/// (uncapped — table scoring cost is independent of how many docs built
/// it) plus `extra_nb` (the transductive column's pseudo-labelled docs;
/// empty on every honest build). The drafter corpus stays capped.
fn build_engine_with<const N: usize>(
    suite: &str,
    train: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
    cfg: EngineConfig,
    extra_nb: &[TrainDoc],
) -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String> {
    #[cfg(not(feature = "nb_scope"))]
    let _ = extra_nb;
    #[cfg(feature = "nb_scope")]
    let nb_sets = (cfg.nb_scale > 0.0).then(|| nb_doc_sets(train, labels, extra_nb));
    #[cfg(not(feature = "nb_scope"))]
    let nb_sets = None;
    let (specs, fallback_labels) = specs_from_pool(train, labels, cap_per_label, nb_sets);
    DecisionEngine::<N, EMBED_DIM>::build_specs(specs, cfg)
        .map(|e| (e, fallback_labels))
        .map_err(|e| format!("engine build ({suite}): {e}"))
}

/// The timed drafter corpus, per label (Issue 057): exactly the docs
/// [`specs_from_pool`] hands each expert — the pool's label docs, CAPPED,
/// with the Issue-039 self-doc fallback for a starved label (the fallback
/// labels are returned so the caller decides: disclose loud on main builds,
/// ignore on selection slices where starvation is by construction). One
/// home for the construction so the lane's `corpus_digest` provably hashes
/// what the engine consumes, never a parallel reading of it.
fn per_label_corpus_docs(
    train: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
) -> (Vec<Vec<String>>, Vec<String>) {
    let mut per_label: Vec<Vec<String>> = Vec::with_capacity(labels.len());
    let mut fallback_labels: Vec<String> = Vec::new();
    for label in labels {
        let mut docs: Vec<String> = train
            .iter()
            .filter(|d| d.label == *label)
            .map(|d| d.text.clone())
            .take(cap_per_label)
            .collect();
        if docs.is_empty() {
            fallback_labels.push(label.clone());
            docs.push(label.clone());
        }
        per_label.push(docs);
    }
    (per_label, fallback_labels)
}

/// The shared per-label spec construction (domain corpora + the wider
/// count-table sets) + the Issue-039 self-doc fallback disclosure. One body
/// for both the plain and the option-conditioned build paths.
fn specs_from_pool(
    train: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
    nb_sets: Option<Vec<Vec<String>>>,
) -> (Vec<ExpertSpec>, Vec<String>) {
    let (drafter_docs, fallback_labels) = per_label_corpus_docs(train, labels, cap_per_label);
    let mut specs: Vec<ExpertSpec> = Vec::with_capacity(labels.len());
    for (idx, label) in labels.iter().enumerate() {
        #[cfg(not(feature = "nb_scope"))]
        let _ = (idx, &nb_sets);
        #[allow(unused_mut)]
        let mut spec = ExpertSpec::new(label.as_str(), &drafter_docs[idx]);
        #[cfg(feature = "nb_scope")]
        if let Some(sets) = nb_sets.as_ref() {
            spec = spec.with_nb_docs(sets[idx].clone());
        }
        #[cfg(not(feature = "nb_scope"))]
        let _ = &mut spec;
        specs.push(spec);
    }
    (specs, fallback_labels)
}

/// Per-label specs for a CORPUS-EXTENDED build (Plan 426 T5's synth law,
/// extracted from corpus_ab's arm B so the seat serves the same
/// construction it measured): the gold pool capped, then `extra` beyond
/// the cap per label (gold first, synth after — the drafter corpus order
/// the V5 read measured); the count-table sets read train ∪ extra
/// uncapped ([`nb_doc_sets`]). A label starved of BOTH falls back to its
/// own label text (the Issue-039 disclosure). Returns (specs, fallback
/// labels, extra docs actually used).
pub(crate) fn specs_corpus_extended(
    train: &[TrainDoc],
    extra: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
    extra_cap: usize,
    nb_sets: Option<&[Vec<String>]>,
) -> (Vec<ExpertSpec>, Vec<String>, usize) {
    let mut specs: Vec<ExpertSpec> = Vec::with_capacity(labels.len());
    let mut fallback_labels = Vec::new();
    let mut extra_used = 0usize;
    for (idx, label) in labels.iter().enumerate() {
        #[cfg(not(feature = "nb_scope"))]
        let _ = idx;
        let mut docs: Vec<String> = train
            .iter()
            .filter(|d| d.label == *label)
            .map(|d| d.text.clone())
            .take(cap_per_label)
            .collect();
        for d in extra.iter().filter(|d| d.label == *label).take(extra_cap) {
            docs.push(d.text.clone());
            extra_used += 1;
        }
        if docs.is_empty() {
            fallback_labels.push(label.clone());
            docs.push(label.clone());
        }
        #[allow(unused_mut)]
        let mut spec = ExpertSpec::new(label.as_str(), &docs);
        #[cfg(feature = "nb_scope")]
        if let Some(sets) = nb_sets {
            spec = spec.with_nb_docs(sets[idx].clone());
        }
        #[cfg(not(feature = "nb_scope"))]
        let _ = &mut spec;
        specs.push(spec);
    }
    (specs, fallback_labels, extra_used)
}

/// [`build_engine_with`] plus the option-conditioned events (issue 038
/// T7b): non-empty events route the build through `build_specs_oc`, which
/// fits the (qid, option) tables when `oc_scale > 0` (and refuses an armed
/// scale with empty events — the fail-closed law).
#[cfg(feature = "option_cond")]
#[allow(clippy::too_many_arguments)]
fn build_engine_oc_with<const N: usize>(
    suite: &str,
    train: &[TrainDoc],
    labels: &[String],
    cap_per_label: usize,
    cfg: EngineConfig,
    extra_nb: &[TrainDoc],
    oc_events: &[crate::option_cond::OcEvent],
) -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String> {
    let nb_sets = (cfg.nb_scale > 0.0).then(|| nb_doc_sets(train, labels, extra_nb));
    let (specs, fallback_labels) = specs_from_pool(train, labels, cap_per_label, nb_sets);
    DecisionEngine::<N, EMBED_DIM>::build_specs_oc(specs, cfg, oc_events)
        .map(|e| (e, fallback_labels))
        .map_err(|e| format!("engine build ({suite}): {e}"))
}

/// The engine wire request for one case: state serialized with the
/// Python-JSON law (the SAME bytes the laya lane sequences), prompt =
/// instructions, options from the criteria structure, criteria text None
/// (the modelless context is state + prompt, its serving path).
pub fn engine_request(
    case: &SuiteCase,
    state_str: &str,
) -> Result<katgpt_core::decision_wire::DecisionRequest, String> {
    let mut questions = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        questions.push(fresh_question(q)?);
    }
    Ok(katgpt_core::decision_wire::DecisionRequest {
        state: state_str.to_string(),
        questions,
    })
}

/// The scratch refill form of [`engine_request`] (reflex issue 070, lead
/// 2): writes the same request into a caller-owned buffer, reusing every
/// `String`/`Vec` capacity it already holds. A slot whose KIND changes
/// between requests is rebuilt fresh (the shape-change path — one
/// allocation, then stable for the suite's request shape).
fn engine_request_into(
    case: &SuiteCase,
    state_str: &str,
    req: &mut katgpt_core::decision_wire::DecisionRequest,
) -> Result<(), String> {
    req.state.clear();
    req.state.push_str(state_str);
    for (slot, q) in req.questions.iter_mut().zip(case.questions.iter()) {
        refill_question_slot(slot, q)?;
    }
    req.questions.truncate(case.questions.len());
    for q in &case.questions[req.questions.len()..] {
        req.questions.push(fresh_question(q)?);
    }
    Ok(())
}

/// The fresh-build half of [`engine_request`] — the owned path's
/// constructor and [`engine_request_into`]'s shape-change path.
fn fresh_question(q: &SuiteQuestion) -> Result<katgpt_core::decision_wire::Question, String> {
    use katgpt_core::decision_wire::Question;
    Ok(match q.kind {
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
    })
}

/// Same-kind in-place refill of one question slot (reflex issue 070,
/// lead 2): strings clear+rewrite (capacity kept), the option list
/// refills per slot and only grows/shrinks when the request shape
/// changes. The wire bytes match [`fresh_question`]'s construction
/// exactly — same map iteration order, same value spellings.
fn refill_question_slot(
    slot: &mut katgpt_core::decision_wire::Question,
    q: &SuiteQuestion,
) -> Result<(), String> {
    use katgpt_core::decision_wire::QuestionKind;
    let kind = match q.kind {
        QKind::Choice => QuestionKind::Choice,
        QKind::Score => QuestionKind::Score,
        QKind::Noul => QuestionKind::Noul,
    };
    if slot.kind != kind {
        *slot = fresh_question(q)?;
        return Ok(());
    }
    refill_string(&mut slot.id, &q.qid);
    refill_string(&mut slot.prompt, &q.instructions);
    slot.criteria = None;
    match q.kind {
        QKind::Choice => {
            let obj = q
                .criteria
                .as_object()
                .ok_or_else(|| format!("case {}: choice criteria must be an object", q.qid))?;
            refill_strings(&mut slot.options, obj.keys().map(String::as_str));
        }
        QKind::Score => {
            let arr = q
                .criteria
                .as_array()
                .ok_or_else(|| format!("case {}: score criteria must be an array", q.qid))?;
            for (i, v) in arr.iter().enumerate() {
                let spelled = match v {
                    Value::String(s) => {
                        if i < slot.options.len() {
                            refill_string(&mut slot.options[i], s);
                            continue;
                        }
                        s.clone()
                    }
                    other => other.to_string(),
                };
                if i < slot.options.len() {
                    refill_string(&mut slot.options[i], &spelled);
                } else {
                    slot.options.push(spelled);
                }
            }
            slot.options.truncate(arr.len());
        }
        QKind::Noul => {} // options MUST stay empty (the wire law)
    }
    Ok(())
}

fn refill_string(dst: &mut String, src: &str) {
    dst.clear();
    dst.push_str(src);
}

fn refill_strings<'a>(dst: &mut Vec<String>, src: impl Iterator<Item = &'a str>) {
    let mut n = 0usize;
    for s in src {
        if n < dst.len() {
            refill_string(&mut dst[n], s);
        } else {
            dst.push(s.to_string());
        }
        n += 1;
    }
    dst.truncate(n);
}

// ── evaluation record ───────────────────────────────────────────────────

/// A lane evaluation over one case set: per case, per question probabilities
/// in LABEL space ([p_no, p_yes] for noul — gold idx 1 = true), forced picks
/// in label space, readout confidences, abstain flags.
pub(crate) struct Eval {
    probs: Vec<Vec<Vec<f64>>>,
    pub(crate) picks: Vec<Vec<usize>>,
    pub(crate) confs: Vec<Vec<f64>>,
    pub(crate) abstained: Vec<Vec<bool>>,
    /// Issue 060: the fused gate's per-question abstain CAUSE (the engine's
    /// own classification — `Answered` when it did not). Captured from the
    /// caller-owned scratch right after `decide_with` (the slots survive the
    /// call and hold exactly the last request's verdicts), so the wire
    /// stays untouched.
    causes: Vec<Vec<AbstainCause>>,
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
        rows.shrink_to_fit();
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
        pairs.shrink_to_fit();
        pairs
    }

    /// Issue 060: per-cause counts over the closed taxonomy. Score-gate
    /// precedence (the engine's classification): `distance_gate` counts
    /// only the marginal cause (score threshold passed, distance gate
    /// fired), so the shares sum to the abstaining-question total — the
    /// same denominator `selective`'s `abstain_rate` uses over the same
    /// cases.
    pub(crate) fn abstain_causes(&self) -> AbstainCauses {
        let mut out = AbstainCauses::default();
        for ccase in &self.causes {
            for cause in ccase {
                match cause {
                    AbstainCause::ScoreGate => out.score_gate += 1,
                    AbstainCause::DistanceGate => out.distance_gate += 1,
                    // Issue 066: the density half's marginal arm — nonzero
                    // only on an armed `--density-gate` run (the engine
                    // emits it only behind the feature + knob).
                    AbstainCause::DensityGate => out.density_gate += 1,
                    // The engine never emits the serve-lane arm; the key is
                    // the reserved third of the closed taxonomy.
                    AbstainCause::GrammarInvalid => out.grammar_invalid += 1,
                    AbstainCause::Answered => {}
                }
            }
        }
        out
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

pub(crate) struct Latency {
    p50_ms: f64,
    p99_ms: f64,
    tail_support: usize,
    extremes: Option<LatencyExtremes>,
    determinism_ok: Option<bool>,
    /// Issue 060 rider (plan 001 task 9's deferred ask): HOW MANY cases the
    /// repeat-run check actually re-ran — the `determinism_ok` verdict's
    /// support. `None` when the check did not run (the flag's own None).
    determinism_n: Option<usize>,
    /// The raw per-case wall times (µs) — the seat's consumers compose
    /// H1's total from these ([`seat::SeatEval::durs_us`]).
    durs_us: Vec<u64>,
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

/// Label-space probabilities + the forced pick for one wire answer — the
/// ONE copy of the flip/argmax law (reflex issue 070 lead 2): the
/// engine's internal noul order is [p_yes] (wire p_yes only), flipped to
/// [no, yes] to match the gold-index convention; an abstained answer's
/// pick is the argmax (the metrics' forced pick). Writes `out` in place
/// (clear + extend — the scratch path reuses the capacity) and returns
/// the pick.
fn answer_probs_pick(
    kind: QKind,
    ans: &katgpt_core::decision_wire::Answer,
    out: &mut Vec<f64>,
) -> usize {
    out.clear();
    if kind == QKind::Noul {
        let p_yes = f64::from(ans.probabilities[0]);
        out.push(1.0 - p_yes);
        out.push(p_yes);
    } else {
        out.extend(ans.probabilities.iter().map(|p| f64::from(*p)));
    }
    match &ans.outcome {
        Some(katgpt_core::decision_wire::Outcome::Choice { index }) => *index as usize,
        Some(katgpt_core::decision_wire::Outcome::Score { level }) => *level as usize,
        Some(katgpt_core::decision_wire::Outcome::Noul { yes }) => usize::from(*yes),
        None => argmax(out),
    }
}

// ── the modelless lane ──────────────────────────────────────────────────

pub(crate) fn eval_engine<const N: usize>(
    engine: &mut DecisionEngine<N, EMBED_DIM>,
    cases: &[SuiteCase],
    state_strs: &[String],
    check_determinism: bool,
) -> Result<(Eval, Latency), String> {
    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut abstained = Vec::with_capacity(cases.len());
    let mut causes = Vec::with_capacity(cases.len());
    let max_q = cases.iter().map(|c| c.questions.len()).max().unwrap_or(1);
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    sc.prepare(max_q);
    let mut durs_us: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = if check_determinism { Some(true) } else { None };
    let mut determinism_n: usize = 0;

    for (ci, case) in cases.iter().enumerate() {
        let req = engine_request(case, &state_strs[ci])?;
        let t0 = Instant::now();
        let resp = engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("engine decide ({}, case {ci}): {e}", case.id))?;
        durs_us.push(t0.elapsed().as_micros() as u64);

        if check_determinism && ci < 10 {
            determinism_n += 1;
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
        // Issue 060: capture the per-question causes from the caller-owned
        // scratch RIGHT AFTER the first decide — the slots hold exactly this
        // request's verdicts until the next solve resets them (the determinism
        // rerun below reuses the same scratch).
        let ccauses: Vec<AbstainCause> = sc.slots.iter().map(|s| s.cause).collect();
        for (q, ans) in case.questions.iter().zip(resp.answers.iter()) {
            let mut p = Vec::new();
            let pick = answer_probs_pick(q.kind, ans, &mut p);
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(f64::from(ans.confidence));
            cabst.push(ans.outcome.is_none());
        }
        // The wire's `outcome.is_none()` and the engine's cause must agree
        // by construction — assert the join so a future gate that abstains
        // without a cause cannot publish a silent mismatch.
        debug_assert_eq!(cabst.len(), ccauses.len());
        for (w, c) in cabst.iter().zip(ccauses.iter()) {
            debug_assert_eq!(*w, c.abstained());
        }
        causes.push(ccauses);
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
            causes,
        },
        Latency {
            p50_ms: p50 as f64 / 1000.0,
            p99_ms: p99 as f64 / 1000.0,
            tail_support: support,
            extremes: LatencyExtremes::of(&durs_us, 1000.0),
            determinism_ok,
            determinism_n: if check_determinism {
                Some(determinism_n)
            } else {
                None
            },
            durs_us,
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
    /// Fitted per-label head scale (issue 030 lever 4; 0.0 = OFF — the
    /// published baseline posture). Plumbs into every modelless-lane engine
    /// build, threshold probe included, so the whole suite runs at ONE
    /// posture.
    head_scale: f32,
    /// Cal-slice head-scale selection (ties → 0). When true, `head_scale`
    /// is the FALLBACK for ineligible suites (synthetic families).
    head_select: bool,
    /// Cal-slice count-table selection (issue 038, `--nb-select`).
    #[cfg_attr(not(feature = "nb_scope"), allow(dead_code))]
    nb_select: bool,
    /// Cal-slice option-conditioned selection (issue 038 T7b, `--oc-select`).
    #[cfg_attr(not(feature = "option_cond"), allow(dead_code))]
    oc_select: bool,
    /// Cal-slice ridge-readout selection (issue 038 T7a, `--ridge-select`).
    #[cfg_attr(not(feature = "nb_ridge"), allow(dead_code))]
    ridge_select: bool,
    /// Cal-slice JOINT blend-genome selection (issue 038 T5,
    /// `--genome-select`): refines the composed greedy posture's scale
    /// coordinates together on the same selection slice.
    #[cfg_attr(not(feature = "nb_scope"), allow(dead_code))]
    genome_select: bool,
    /// The genome walk's acceptance-vs-seed bar (fraction, 0.05 = the
    /// house arming bar). `<= 0` resolves to the house bar (a 0 margin
    /// would accept any +1-question wiggle — never a default).
    #[cfg_attr(not(feature = "nb_scope"), allow(dead_code))]
    genome_accept_margin: f64,
    /// The cap's base source when no cal-slice selection ran
    /// ("registry" or "--corpus-cap override").
    cap_source_base: &'static str,
    /// Cal-slice cap-selection candidates (empty = off — run() enforces
    /// mutual exclusion with the override).
    cal_select_caps: &'a [usize],
    /// The suite's corpus-pool envelope (dataset suites: the train rows
    /// MINUS the stratified cal front — the selection stratification reads
    /// the rows it buckets; `train` alone has lost the row structure).
    /// Null on the synthetic/code paths (those suites are
    /// selection-ineligible).
    pool_rows: &'a Value,
    /// Run the Issue-013 lever-3 pair-head A/B arm (report-only; pairs are
    /// armed from CAL-slice confusion, heads fitted from corpus docs).
    pair_head_ab: bool,
    /// Issue 044 T3: run the NLI pair-feature head A/B (report-only;
    /// premise/hypothesis-shaped suites only — a loud skip elsewhere).
    nli_feature_ab: bool,
    /// Issue 047 M1: run the xnli VALIDATION-slice reopen lane
    /// (`--nli-m1`; pre-registered by .plans/006_nli_m1_reopen.md — the
    /// pass itself loud-skips any suite but `xnli_en_val`, R1's guard).
    nli_m1: bool,
    /// Issue 055 / Plan 008 T5: run the `--mc-ab` distributional-layer A/B
    /// (report-only; the seeded-MC wrapper over the fitted engine). None =
    /// the arm is off.
    #[cfg(feature = "mc_ensemble")]
    pub mc_ab: Option<&'a super::mc_ab::McAbConfig>,
    /// Issue 066: run the `--density-gate` fused-abstain density-half A/B
    /// (report-only; the density-armed gate vs the shipped calibrated
    /// gate over one frozen read).
    #[cfg_attr(not(feature = "density_gate"), allow(dead_code))]
    pub density_gate: bool,
    /// Plan 627 / Issue 090 C2: run the `--set-refine-ab` paired A/B
    /// (report-only; the refined engine vs the shipped posture over one
    /// frozen read). `Some((gamma, tau))` = armed with the CLI knobs.
    /// None = the arm is off / the feature compiled out.
    #[cfg_attr(not(feature = "set_refine"), allow(dead_code))]
    pub set_refine: Option<(f32, f32)>,
    /// Issue 024 T3: one leak flag per eval case (true = the case has an
    /// exact/near twin in the corpus∪cal side — drop from acc_deleaked).
    /// None = the `slice_leak` feature is off or the suite is out of scope.
    leak_flags: Option<&'a [bool]>,
    /// Issue 042 lever 3: also produce the worthiness probe's modelless
    /// half — the deployed calibrated gate over the suite's CAL slice.
    pub cascade_worthiness: bool,
    /// Issue 042 lever 1 (`--gate-fit-selection`): fit the fused-gate
    /// thresholds on the STRATIFIED selection slice instead of the
    /// train-tail cal slice. Dataset suites only — the synthetic/code
    /// paths carry no pool envelope and keep the cal-slice fit.
    pub gate_fit_selection: bool,
    /// Issue 042 lever 2 (`--gate-distance-only`): disable the fused
    /// gate's score axis (threshold 0.0) — abstain/escalation runs on the
    /// corpus-distance axis alone, at its fitted ρ=30% threshold.
    pub gate_distance_only: bool,
    /// Issue 056 direction 0 — PROMOTED default-on 2026-09-30 (verdict
    /// round 2): fit the score-axis threshold on the CALIBRATED scale the
    /// deployed gate applies (the probe's own per-question pairs reconstruct
    /// the deployed calibrator exactly). The fit-on-raw / apply-on-calibrated
    /// mismatch was a units bug — at a monotone fit the percentile-coherent
    /// threshold reproduces the raw gate's fitted target exactly, so it can
    /// never lose to the pre-calibration baseline. `--no-gate-fit-calibrated`
    /// restores the old posture for measurement.
    pub gate_fit_calibrated: bool,
    /// Issue 079: the legacy `k == N` index-alignment route binding.
    /// `true` (the default) = the incumbent shipped behavior,
    /// byte-identical; `false` (`--no-kn-route`) = the content-bound
    /// posture — every modelless-lane engine build (fitted, selection,
    /// transductive) arms route terms only via by-name resolution.
    pub kn_route: bool,
    /// Issue 079: the drafter-only correction mode for the questions that
    /// go drafter-only when the legacy binding is off (typed's short
    /// option keys). Default `Off` = the shipped scores, byte-identical;
    /// `--drafter-fix <mode>` arms one of the [`DrafterFix`] candidates
    /// (issue 036 T2) so the content-bound posture is measured TUNED,
    /// not at an arbitrary drafter posture.
    pub drafter_fix: crate::engine::DrafterFix,
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
    // Issue 039 T2: the selection draws from the POOL envelope (train rows
    // minus the stratified cal front), pool_from = 0 — under the stratified
    // cal slice the positional `cal_cap..` cut no longer separates cal from
    // pool, but the complement envelope does, by construction.
    let slices = stratified_selection_slices(inp.pool_rows, spec.name, inp.labels, 0, spec.cal_cap);
    if slices.n_picks == 0 {
        return Err(format!(
            "{}: cap selection produced an empty stratified slice — the pool rows \
             carry none of the engine's labels",
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
    let pool: Vec<TrainDoc> = inp
        .train
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
        inp.train.len(),
        pool.len()
    );

    let mut cands = inp.cal_select_caps.to_vec();
    cands.push(spec.corpus_cap_per_label);
    cands.sort_unstable();
    cands.dedup();
    let mut rows = Vec::with_capacity(cands.len());
    for &cap in &cands {
        let (mut engine, _) = build_engine::<N>(
            spec.name,
            &pool,
            inp.labels,
            cap,
            EngineConfig {
                head_scale: inp.head_scale,
                legacy_kn_route: inp.kn_route,
                drafter_fix: inp.drafter_fix,
                ..EngineConfig::default()
            },
        )?;
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

/// Fitted-head scale ladder (issue 030 lever 4). Geometric ×2 up to the
/// fitted-model-verbatim 1.0 — and NO FURTHER, on a measured structural
/// reason: the blend term is `0.5 + s·(σ(logit) − 0.5)`, so s > 1
/// over-weights the model beyond its own calibrated confidence, and the
/// maxp readout saturates (measured: banking77 at s=2 gains accuracy but
/// its calibrated ECE 0.498 fails the conformal floor 0.464 — the G1
/// gate — while s=1 keeps ≈all the accuracy win). Deeper tuning goes
/// through the cal-slice selection below, never through test reads.
const HEAD_SCALE_LADDER: [f32; 4] = [0.0, 0.25, 0.5, 1.0];

/// The promotion bar for the selection (measured, not tuned): a candidate
/// wins only if it clears the scale-0 baseline by this margin on the
/// selection slice. At the slice's n (≈200), 2σ of a proportion near 0.5
/// is ≈5–6 pt — inside the margin. Without it the n=200 noise flips
/// small/neutral suites both ways (measured: emotion's sel slice read
/// +2.5 pt at scale 1 while the test split read −5.5 pt). A selection
/// that only moves on strong evidence cannot lose a suite to noise —
/// and a suite left at 0 keeps the published baseline BIT-IDENTICALLY.
const HEAD_SELECT_MARGIN: f64 = 0.05;

/// One head-scale candidate's cal-slice accuracy.
#[derive(Debug, Clone, Serialize)]
pub struct HeadScaleCandidate {
    pub scale: f32,
    pub cal_acc: f64,
}

/// The cal-selected head posture for a suite (issue 030 lever 4, the
/// Issue-013 lever-1 protocol shape): accuracy per candidate on the CAL
/// slice, argmax picked there ONLY, ties → 0 (off — the conservative
/// winner), test read once at the selected posture. Disclosed per row so
/// a table never reads against an unknown head posture.
#[derive(Debug, Clone, Serialize)]
pub struct HeadScaleSelection {
    pub selected: f32,
    pub candidates: Vec<HeadScaleCandidate>,
}

/// The stratified selection slice every cal-side selection shares (cap,
/// head scale, issue-038 nb posture): the suite's own builder over the
/// label-stratified permuted envelope, and the corpus pool MINUS the
/// selection docs by content (a selection case must never score against
/// its own text). One instrument, three consumers.
struct SelSlice {
    cases: Vec<SuiteCase>,
    state_strs: Vec<String>,
    pool: Vec<TrainDoc>,
}

/// The option-conditioned gold events for a corpus pool (issue 038 T7b):
/// the suite's train-row events filtered to the pool's own texts, so a
/// selection slice (or a cal-front complement) never scores against its
/// own text — the shared selection-slice law, applied at the event level.
#[cfg(feature = "option_cond")]
fn oc_events_for(pool_rows: &Value, pool: &[TrainDoc]) -> Vec<crate::option_cond::OcEvent> {
    let texts: std::collections::HashSet<&str> = pool.iter().map(|d| d.text.as_str()).collect();
    typed_gold_events(pool_rows)
        .into_iter()
        .filter(|(_, _, state)| texts.contains(state.as_str()))
        .map(|(qid, option, doc)| crate::option_cond::OcEvent { qid, option, doc })
        .collect()
}

fn selection_slice(inp: &ModellessInput<'_>, tag: &str) -> Result<SelSlice, String> {
    let spec = inp.spec;
    // Synthetic suites carry no pool rows (Issue 045): the labelled
    // selection slice IS the authored cal front — the same labelled data
    // the fused-gate thresholds fit on, never the test cases — and the
    // corpus pool is the family's own docs minus any text the cal front
    // carries (the same self-inclusion exclusion the dataset path
    // enforces; the families' slices are disjoint by construction, the
    // gate asserts it).
    if inp.pool_rows.is_null() {
        if inp.cal_cases.is_empty() {
            return Err(format!(
                "{}: {tag} selection on a synthetic suite with no cal front",
                spec.name
            ));
        }
        let excluded: HashSet<&str> = inp
            .cal_cases
            .iter()
            .filter_map(|c| c.state.as_str())
            .collect();
        let pool: Vec<TrainDoc> = inp
            .train
            .iter()
            .filter(|d| !excluded.contains(d.text.as_str()))
            .cloned()
            .collect();
        eprintln!(
            "    {tag}: synthetic cal front {} case(s); corpus pool {} → {} doc(s)",
            inp.cal_cases.len(),
            inp.train.len(),
            pool.len()
        );
        return Ok(SelSlice {
            cases: inp.cal_cases.to_vec(),
            state_strs: inp.cal_state_strs.to_vec(),
            pool,
        });
    }
    // Issue 039 T2: pool-envelope input, pool_from = 0 (see
    // build_selection_measurement).
    let slices = stratified_selection_slices(inp.pool_rows, spec.name, inp.labels, 0, spec.cal_cap);
    if slices.n_picks == 0 {
        return Err(format!(
            "{}: {tag} selection produced an empty stratified slice",
            spec.name
        ));
    }
    let sel_suite = (spec.build)(&slices.permuted, slices.n_picks);
    let state_strs: Vec<String> = sel_suite
        .cases
        .iter()
        .map(|c| serialize_state(&c.state))
        .collect();
    let sel_docs = train_docs(&slices.front, spec.name);
    let excluded: HashSet<&str> = sel_docs.iter().map(|d| d.text.as_str()).collect();
    let pool: Vec<TrainDoc> = inp
        .train
        .iter()
        .filter(|d| !excluded.contains(d.text.as_str()))
        .cloned()
        .collect();
    eprintln!(
        "    {tag}: stratified slice {} case(s); corpora pool {} → {} doc(s)",
        sel_suite.cases.len(),
        inp.train.len(),
        pool.len()
    );
    Ok(SelSlice {
        cases: sel_suite.cases,
        state_strs,
        pool,
    })
}

/// Cal-slice head-scale selection, on the STRATIFIED selection slice —
/// the same instrument the cap selection uses. Measured reason this is
/// NOT the raw cal cases: the registry cal prefix is label-clustered in
/// the mirrors, and on banking77 the clustered slice REVERSED the scale
/// signal (cal accs fell with scale while the stratified-instrument's
/// sibling axis and the test split both rise) — the Bench-004/Issue-023
/// class, re-measured on this axis. Forced accuracy: thresholds are
/// maxed so the abstain gate cannot confound the scale signal (the gate
/// is re-fitted AFTER, at the selected posture, by the existing
/// threshold fit).
fn build_head_scale_selection<const N: usize>(
    inp: &ModellessInput<'_>,
    effective_cap: usize,
) -> Result<HeadScaleSelection, String> {
    let spec = inp.spec;
    let SelSlice {
        cases: sel_cases,
        state_strs: sel_state_strs,
        pool,
    } = selection_slice(inp, "head-select")?;
    let mut rows = Vec::with_capacity(HEAD_SCALE_LADDER.len());
    for &scale in &HEAD_SCALE_LADDER {
        let (mut engine, _) = build_engine::<N>(
            spec.name,
            &pool,
            inp.labels,
            effective_cap,
            EngineConfig {
                head_scale: scale,
                legacy_kn_route: inp.kn_route,
                drafter_fix: inp.drafter_fix,
                // Forced: never abstain (conf ≤ 1 < threshold).
                score_threshold: 2.0,
                distance_threshold: 2.0,
                ..EngineConfig::default()
            },
        )?;
        let (ev, _) = eval_engine(&mut engine, &sel_cases, &sel_state_strs, false)?;
        let cal_acc = hard_metrics(&ev.forced_rows(&sel_cases)).accuracy;
        rows.push(HeadScaleCandidate { scale, cal_acc });
        eprintln!("    head-select: scale {scale} → sel-slice acc {cal_acc:.4}");
    }
    // Promotion-bar argmax: candidates compete only if they clear the
    // scale-0 baseline by HEAD_SELECT_MARGIN; ties among clearing
    // candidates go to the higher accuracy, then the earlier (weaker)
    // scale. Nothing clears → 0 (off), the published baseline posture.
    let base = rows[0].cal_acc;
    let mut selected = rows[0].scale;
    let mut best_acc = base;
    for r in &rows[1..] {
        if r.cal_acc >= base + HEAD_SELECT_MARGIN && r.cal_acc > best_acc {
            best_acc = r.cal_acc;
            selected = r.scale;
        }
    }
    eprintln!(
        "    head-select: selected {selected} — the suite's row below is the single \
         test read, at this posture"
    );
    Ok(HeadScaleSelection {
        selected,
        candidates: rows,
    })
}

/// Issue 039 T4: the per-suite confidence-readout CANDIDATE table —
/// REPORT-ONLY by measured verdict. The original lever (arm the best
/// candidate per suite, selected on in-sample calibrated ECE over the cal
/// slice) was DEMOTED at Bench 052: on the wide suites it targets
/// (banking77 77-way, massive 60-way) the shipped Dispatch readout IS
/// max_prob (the wide arm — candidates coincide, cal ECE identical) and
/// inv_entropy measured WORSE (massive 0.5519 vs 0.4618, banking77 0.8070
/// vs 0.7860), so no functional beats the shipped law where the gap was;
/// where a candidate DID win on cal (the narrow suites, entropy→maxp),
/// arming it overfit the cal slice and regressed emotion's test G1 to a
/// FAIL. The wide-label G1 failures were closed by T2's stratified cal
/// slice instead. The report stays as instrumentation (what the candidate
/// functionals read on cal), and [`crate::engine::EngineConfig::readout`]
/// remains the opt-in knob (never armed by the harness).
#[derive(Debug, Clone, Serialize)]
pub struct ReadoutReport {
    /// The candidate the margin rule WOULD arm (`ReadoutMode::as_str`) —
    /// recorded, never applied (the demotion above).
    pub best_on_cal: String,
    /// In-sample calibrated ECE per candidate on the cal slice.
    pub candidates: Vec<ReadoutCandidate>,
}

/// One readout candidate's cal-slice reading.
#[derive(Debug, Clone, Serialize)]
pub struct ReadoutCandidate {
    pub mode: String,
    pub cal_ece: f64,
}

/// A candidate must beat the shipped Dispatch law by at least this cal-ECE
/// margin to be selected — a smaller delta is cal-slice noise, and arming a
/// non-default readout on noise would be selection without evidence.
const READOUT_SELECT_MARGIN: f64 = 0.005;

/// Below this many cal pairs the selection signal is too thin to rank
/// readouts — keep the shipped Dispatch law and record no selection (the
/// synthetic families' tiny cal slices; the dataset suites all clear it).
const READOUT_MIN_CAL_PAIRS: usize = 32;

/// Cal-slice readout CANDIDATE report (Issue 039 T4, REPORT-ONLY — the
/// arming lever was demoted, see [`ReadoutReport`]): rank the candidate
/// confidence readouts by IN-SAMPLE CALIBRATED ECE on the cal slice — fit
/// the sigmoid-gate calibrator on the candidate's own pairs, apply it to
/// the same pairs, take the ECE. The answer distribution and the forced
/// pick are readout-INDEPENDENT (the readout runs after the argmax), so
/// ONE cal evaluation serves every candidate: the probs + picks are
/// recovered from that eval and only the confidence scalar is recomputed
/// per mode. `None` = thin cal (below [`READOUT_MIN_CAL_PAIRS`]) — nothing
/// to report.
#[allow(clippy::too_many_arguments)]
fn readout_report_on_cal<const N: usize>(
    suite: &str,
    cal_cases: &[SuiteCase],
    cal_state_strs: &[String],
    corpus_pool: &[TrainDoc],
    labels: &[String],
    cap: usize,
    cfg: &EngineConfig,
    #[cfg(feature = "option_cond")] oc_events: Option<&[crate::option_cond::OcEvent]>,
    #[cfg(not(feature = "option_cond"))] _oc_events: (),
) -> Result<Option<ReadoutReport>, String> {
    let n_pairs: usize = cal_cases.iter().map(|c| c.questions.len()).sum();
    if n_pairs < READOUT_MIN_CAL_PAIRS {
        return Ok(None);
    }
    let candidates = [
        ReadoutMode::Dispatch,
        ReadoutMode::MaxProb,
        ReadoutMode::InvEntropy,
    ];
    // Probs + picks are mode-independent — one cal eval. The engine is
    // built at the caller's posture (heads/nb resolved); its own readout
    // field is irrelevant here (only probs/picks are read back).
    #[cfg(feature = "option_cond")]
    let (mut engine, _) = match oc_events {
        Some(ev) => {
            build_engine_oc_with::<N>(suite, corpus_pool, labels, cap, cfg.clone(), &[], ev)?
        }
        None => build_engine::<N>(suite, corpus_pool, labels, cap, cfg.clone())?,
    };
    #[cfg(not(feature = "option_cond"))]
    let (mut engine, _) = build_engine::<N>(suite, corpus_pool, labels, cap, cfg.clone())?;
    let (ev, _) = eval_engine(&mut engine, cal_cases, cal_state_strs, false)?;
    let mut rows = Vec::with_capacity(candidates.len());
    for &mode in &candidates {
        // The engine's readout ran on the INTERNAL distribution; the eval
        // stores label-space probs (noul flipped to [no, yes] and p_no
        // re-derived as 1 − p_yes). Recover the internal vector: exact for
        // choice/score (f64::from of the engine's f32, lossless both ways);
        // for noul the second element carries a 1 − p_yes f64 rounding
        // wobble (~1e-8) against the engine's own p_no — a selection-signal
        // wobble only, never a test-readout path.
        let mut cal = SigmoidGateCalibrator::new(cfg.cal_capacity, cfg.cal_min_obs);
        let mut raw_pairs: Vec<(f32, bool)> = Vec::with_capacity(n_pairs);
        for (ci, case) in cal_cases.iter().enumerate() {
            for (qi, q) in case.questions.iter().enumerate() {
                let p = &ev.probs[ci][qi];
                let internal: Vec<f32> = if q.kind == QKind::Noul {
                    let p_yes = p[1] as f32;
                    vec![p_yes, 1.0 - p_yes]
                } else {
                    p.iter().map(|&x| x as f32).collect()
                };
                let raw = crate::readout::confidence_with(mode, &internal);
                let correct = ev.picks[ci][qi] == case.gold[qi].idx;
                cal.observe(raw, correct);
                raw_pairs.push((raw, correct));
            }
        }
        let calibrated: Vec<(f64, bool)> = raw_pairs
            .iter()
            .map(|&(raw, ok)| (f64::from(cal.apply(raw)), ok))
            .collect();
        rows.push(ReadoutCandidate {
            mode: mode.as_str().to_string(),
            cal_ece: ece_of(&calibrated),
        });
    }
    // The margin rule's pick, RECORDED ONLY (never armed — the Bench-052
    // demotion: the winner overfits the cal slice on the narrow suites and
    // coincides with Dispatch on the wide ones).
    let dispatch_ece = rows[0].cal_ece;
    let mut best_on_cal = ReadoutMode::Dispatch;
    for (&mode, row) in candidates.iter().zip(rows.iter()).skip(1) {
        if row.cal_ece + READOUT_SELECT_MARGIN <= dispatch_ece {
            best_on_cal = mode;
            break;
        }
    }
    eprintln!(
        "    readout-report: best-on-cal {} ({} vs dispatch {:.4}; margin {READOUT_SELECT_MARGIN}; \
         REPORT-ONLY — the arming lever is demoted, Bench 052)",
        best_on_cal.as_str(),
        rows.iter()
            .map(|r| format!("{} {:.4}", r.mode, r.cal_ece))
            .collect::<Vec<_>>()
            .join(", "),
        dispatch_ece
    );
    Ok(Some(ReadoutReport {
        best_on_cal: best_on_cal.as_str().to_string(),
        candidates: rows,
    }))
}

/// The posture prologue of [`run_modelless`], extracted so the seat ([`seat`])
/// can fit the SAME deployed posture for a downstream lane — one source of
/// truth for the selections, never a re-derivation. Byte-identical to the
/// inline prologue it replaced (same operations, same order; the only
/// reordering is the pure report-only readout report, which now runs after
/// the fused-gate fit in [`run_modelless`]).
struct FittedPosture {
    effective_cap: usize,
    selection: Option<CapSelection>,
    head_selected: Option<HeadScaleSelection>,
    nb_selected: Option<NbSelection>,
    #[cfg(feature = "option_cond")]
    oc_selected: Option<oc_lane::OcSelection>,
    #[cfg(feature = "nb_ridge")]
    ridge_selected: Option<ridge_lane::RidgeSelection>,
    genome_selected: Option<genome_lane::GenomeSelection>,
    default_cfg: EngineConfig,
    score_threshold: f32,
    distance_threshold: f32,
    threshold_recommendation: FusedGateRecommendation,
}

/// Issue 042 levers — resolve the fitted recommendation into the deployed
/// thresholds (pure). Default: both axes' fitted values, the birth
/// constants as the thin-support fallback (the shipped law, unchanged).
/// Lever 2 (`distance_only`) pins the score threshold at 0.0 — no
/// calibrated confidence can fall below it, so the fused gate runs on the
/// corpus-distance axis alone (the transfer-stable half, Bench 061); the
/// distance threshold keeps its fitted value and the gate still abstains
/// ρ≈30% on the fit surface, now entirely through that axis.
fn resolve_gate_thresholds(
    rec: &FusedGateRecommendation,
    fallback: &EngineConfig,
    distance_only: bool,
) -> (f32, f32) {
    let score = rec
        .score
        .as_ref()
        .map_or(fallback.score_threshold, |r| r.threshold);
    let distance = rec
        .distance
        .as_ref()
        .map_or(fallback.distance_threshold, |r| r.threshold);
    if distance_only {
        (0.0, distance)
    } else {
        (score, distance)
    }
}

#[cfg(test)]
mod gate_lever_tests {
    //! Issue 042 levers — the threshold-resolution law (the pure half of
    //! the lever surface; the slice-selection half is structural and rides
    //! the acceptance lane run).

    use super::resolve_gate_thresholds;
    use crate::engine::{
        EngineConfig, GateObservation, Posture, THIN_SUPPORT_FLOOR, recommend_fused_gate,
    };

    fn obs(n: usize, base: f32) -> Vec<GateObservation> {
        (0..n)
            .map(|i| GateObservation {
                score: base + i as f32 * 0.01,
                correct: i % 2 == 0,
            })
            .collect()
    }

    fn fitted_rec() -> crate::engine::FusedGateRecommendation {
        let rec = recommend_fused_gate(
            &obs(40, 0.1),
            &obs(40, 0.2),
            Posture::Percentile { rho: 0.30 },
        );
        assert!(rec.score.is_some() && rec.distance.is_some());
        rec
    }

    #[test]
    fn lever_off_keeps_the_shipped_law() {
        let rec = fitted_rec();
        let fallback = EngineConfig::default();
        let (s, d) = resolve_gate_thresholds(&rec, &fallback, false);
        assert_eq!(s, rec.score.as_ref().unwrap().threshold);
        assert_eq!(d, rec.distance.as_ref().unwrap().threshold);
    }

    #[test]
    fn distance_only_pins_the_score_axis_to_zero() {
        let rec = fitted_rec();
        let fallback = EngineConfig::default();
        let (s, d) = resolve_gate_thresholds(&rec, &fallback, true);
        // The score axis can never abstain; the distance axis keeps its
        // fitted value (the gate still abstains ρ≈30% on the fit surface).
        assert_eq!(s, 0.0);
        assert_eq!(d, rec.distance.as_ref().unwrap().threshold);
    }

    #[test]
    fn distance_only_overrides_even_the_thin_support_fallback() {
        let thin = obs(THIN_SUPPORT_FLOOR - 1, 0.1);
        let rec = recommend_fused_gate(&thin, &thin, Posture::Percentile { rho: 0.30 });
        assert!(rec.score.is_none() && rec.distance.is_none());
        let fallback = EngineConfig::default();
        let (s_off, d_off) = resolve_gate_thresholds(&rec, &fallback, false);
        assert_eq!(s_off, fallback.score_threshold);
        assert_eq!(d_off, fallback.distance_threshold);
        let (s_on, d_on) = resolve_gate_thresholds(&rec, &fallback, true);
        assert_eq!(s_on, 0.0);
        assert_eq!(d_on, fallback.distance_threshold);
    }
}

fn fit_posture_inner<const N: usize>(inp: &ModellessInput<'_>) -> Result<FittedPosture, String> {
    let spec = inp.spec;
    // ── Cal-slice cap selection (Issue 013 lever-1 protocol plumb). The
    // cap is picked ONLY on the stratified selection slice — the test
    // split is read once, at the selected cap, further down — closing the
    // protocol hole that deferred the banking77 promotion (a cap picked on
    // test would be a test-set-selected hyperparameter). Self-corpora
    // suites (the synthetic family; the fixed-corpus registry mark below)
    // ignore the caps by construction; the source string below discloses
    // the n/a instead of silently running or silently dropping the ask.
    let selection = if inp.cal_select_caps.is_empty()
        || spec.synthetic.is_some()
        || spec.corpus_cap_per_label == usize::MAX
    {
        None
    } else if inp.cal_cases.is_empty() {
        return Err("cal-slice cap selection needs a cal slice (suite cal_cap > 0)".to_string());
    } else {
        Some(build_selection_measurement::<N>(inp)?)
    };
    let effective_cap = selection
        .as_ref()
        .map_or(inp.corpus_cap_per_label, |s| s.selected);

    // ── Fitted-head scale selection (issue 030 lever 4, the Issue-013
    // lever-1 protocol shape): the blend scale is picked ONLY on the
    // STRATIFIED selection slice (forced accuracy, ties → 0 = off), and
    // the test split is read ONCE at the selected posture. Runs at the
    // selected cap; eligible = the dataset suites (the cap-selection
    // eligibility) — synthetic suites keep the default CLI posture for
    // HEADS: their baseline rows are harness sanity pins.
    let head_selected =
        if inp.head_select && spec.synthetic.is_none() && spec.corpus_cap_per_label != usize::MAX {
            Some(build_head_scale_selection::<N>(inp, effective_cap)?)
        } else {
            None
        };
    let selected_scale = head_selected
        .as_ref()
        .map_or(inp.head_scale, |s| s.selected);
    // ── Count-table posture selection (issue 038 T1): same slice, same
    // promotion bar, at the selected cap + head scale; test read once.
    // Eligible = the dataset suites, and — since Issue 045's synthetic
    // selection fallback — synthetic suites whose cal front can carry
    // the selection (semantic_defects remains; the six Issue-004
    // families that carried the noul polarity lever were retired
    // 2026-10-02). `corpus_cap_per_label == usize::MAX` marks the
    // registry-fixed-corpus family suite; the synthetic eligibility rides
    // the synthetic flag alone now.
    #[cfg(feature = "nb_scope")]
    let nb_selected = if inp.nb_select
        && (spec.synthetic.is_some()
            || (spec.synthetic.is_none() && spec.corpus_cap_per_label != usize::MAX))
    {
        Some(nb_lane::build_nb_selection::<N>(
            inp,
            effective_cap,
            selected_scale,
        )?)
    } else {
        None
    };
    #[cfg(not(feature = "nb_scope"))]
    let nb_selected: Option<NbSelection> = None;
    #[allow(unused_mut)]
    let mut default_cfg = EngineConfig {
        head_scale: selected_scale,
        legacy_kn_route: inp.kn_route,
        drafter_fix: inp.drafter_fix,
        ..EngineConfig::default()
    };
    #[cfg(feature = "nb_scope")]
    if let Some(sel) = &nb_selected {
        default_cfg.nb_scale = sel.selected_scale;
        default_cfg.nb_alpha = nb_lane::alpha_of(sel.selected_alpha);
        default_cfg.nb_noul_domain = sel.selected_noul_domain;
        default_cfg.nb_view = nb_lane::view_of(sel.selected_view);
    }
    #[cfg(not(feature = "nb_scope"))]
    let _ = &mut default_cfg;
    // ── Option-conditioned selection (issue 038 T7b): same slice, same
    // promotion bar, at the selected cap + head + nb posture; test read
    // once. Events filter to the corpus pool's texts (the selection cases
    // never score against their own text).
    #[cfg(feature = "option_cond")]
    let oc_selected =
        if inp.oc_select && spec.synthetic.is_none() && spec.corpus_cap_per_label != usize::MAX {
            // Eligibility: the suite must carry per-question gold events in its
            // pool at all (typed_decisions is the only one today). A suite
            // without them DECLINES the lever loudly — an error would abort
            // every other suite's lane on a whole-workspace run.
            if oc_events_for(inp.pool_rows, inp.train).is_empty() {
                eprintln!(
                    "    oc-select: DECLINED — the suite's train rows carry no per-question gold \
                 events to condition on (baseline posture holds, byte-identical)"
                );
                None
            } else {
                Some(oc_lane::build_oc_selection::<N>(
                    inp,
                    effective_cap,
                    selected_scale,
                    &default_cfg,
                    |pool| oc_events_for(inp.pool_rows, pool),
                )?)
            }
        } else {
            None
        };
    #[cfg(feature = "option_cond")]
    if let Some(sel) = &oc_selected {
        default_cfg.oc_scale = sel.selected_scale;
    }
    #[cfg(not(feature = "option_cond"))]
    let _ = inp.oc_select;
    // ── Ridge selection (issue 038 T7a): last posture axis (it rides the
    // selected cap/head/nb/oc), same slice, same promotion bar. SHORT
    // ladder — every candidate pays a full O(k³)-per-class fit.
    #[cfg(feature = "nb_ridge")]
    let ridge_selected = if inp.ridge_select
        && spec.synthetic.is_none()
        && spec.corpus_cap_per_label != usize::MAX
    {
        Some(ridge_lane::build_ridge_selection::<N>(
            inp,
            effective_cap,
            &default_cfg,
            |pool| oc_events_for(inp.pool_rows, pool),
        )?)
    } else {
        None
    };
    #[cfg(feature = "nb_ridge")]
    if let Some(sel) = &ridge_selected {
        default_cfg.ridge_scale = sel.selected_scale;
        default_cfg.ridge_lambda = sel.selected_lambda;
    }
    #[cfg(not(feature = "nb_ridge"))]
    let _ = inp.ridge_select;
    // ── Joint blend-genome selection (issue 038 T5): the LAST posture
    // axis — it seeds from the composed greedy posture above and refines
    // the scale coordinates TOGETHER on the same selection slice, closing
    // the greedy-order interaction gap the one-at-a-time ladders leave.
    // The walk end must clear the seed by the promotion bar or the
    // posture is HELD (default_cfg untouched → byte-identical final
    // build). Runs BEFORE the fused-gate fit so the fitted thresholds
    // reflect whatever posture the genome settled on.
    #[cfg(feature = "nb_scope")]
    let genome_selected =
        if inp.genome_select && spec.synthetic.is_none() && spec.corpus_cap_per_label != usize::MAX
        {
            let g = genome_lane::build_genome_selection::<N>(
                inp,
                effective_cap,
                &default_cfg,
                if inp.genome_accept_margin > 0.0 {
                    inp.genome_accept_margin
                } else {
                    HEAD_SELECT_MARGIN
                },
                #[cfg(feature = "option_cond")]
                |pool| oc_events_for(inp.pool_rows, pool),
                #[cfg(not(feature = "option_cond"))]
                (),
            )?;
            default_cfg.route_scale = g.selected.route_scale;
            default_cfg.head_scale = g.selected.head_scale;
            default_cfg.nb_scale = g.selected.nb_scale;
            default_cfg.nb_alpha = nb_lane::alpha_of(g.selected.nb_alpha);
            default_cfg.nb_view = nb_lane::view_of(g.selected.nb_view);
            #[cfg(feature = "option_cond")]
            {
                default_cfg.oc_scale = g.selected.oc_scale;
            }
            #[cfg(feature = "nb_ridge")]
            {
                default_cfg.ridge_scale = g.selected.ridge_scale;
            }
            Some(g)
        } else {
            None
        };
    #[cfg(not(feature = "nb_scope"))]
    let genome_selected: Option<genome_lane::GenomeSelection> = None;
    // The full-pool events (corpus pool = the complement of the cal front),
    // needed by EVERY build made at the selected posture (the threshold
    // probe here, the three run_modelless builds, the readout report and
    // the transductive pass). None when the lever never arms — every build
    // path stays the plain one.
    #[cfg(feature = "option_cond")]
    let oc_events_full: Option<Vec<crate::option_cond::OcEvent>> =
        (default_cfg.oc_scale > 0.0).then(|| oc_events_for(inp.pool_rows, inp.train));
    #[cfg(not(feature = "option_cond"))]
    let oc_events_full: Option<std::convert::Infallible> = None;
    let _ = &oc_events_full;

    // ── Fused-gate threshold fitting (the engine's own law: thresholds from
    // measured geometry, never magic numbers). The birth constants (0.35 /
    // 0.5) were measured on the birth corpus and do NOT transfer to these
    // suites — measured: with the defaults the distance gate abstains 100%
    // on several suites. Fit BOTH thresholds at the cal-slice 30th
    // percentile (the T1.6 arena posture, target abstain rate ρ = 30%):
    // 30% of in-corpus-distribution questions abstain, the rest pass. The
    // fitted values ride the result row.
    //
    // Issue 042 levers 1–2 (opt-in): lever 1 (`--gate-fit-selection`)
    // moves the observation surface to the STRATIFIED selection slice —
    // the same held-out instrument the cap/head/nb/genome selections
    // share — with the probe corpus excluding the fit docs (a fit case
    // never scores against its own text, so the observed confidence
    // geometry matches the test side's out-of-corpus rows; the cal-slice
    // fit is in-sample for the CALIBRATOR, which is fitted on that same
    // slice — Bench 061's misplacement hypothesis). Lever 2
    // (`--gate-distance-only`) pins the score threshold at 0.0 after the
    // fit: the fused gate degenerates to the corpus-distance half — the
    // axis that held ~31–32% test abstain in every measured posture while
    // the score axis read 90–99%.
    let gate_fit_slice = if inp.gate_fit_selection
        && spec.synthetic.is_none()
        && spec.corpus_cap_per_label != usize::MAX
    {
        Some(selection_slice(inp, "gate-fit")?)
    } else {
        None
    };
    let (score_threshold, distance_threshold, threshold_recommendation) = {
        let fit_pool_storage: Vec<TrainDoc>;
        let (fit_cases, fit_state_strs, corpus_pool): (&[SuiteCase], &[String], &[TrainDoc]) =
            match &gate_fit_slice {
                Some(sel) => {
                    fit_pool_storage = sel.pool.clone();
                    (&sel.cases, &sel.state_strs, &fit_pool_storage)
                }
                // Issue 039 T2: `train` IS the corpus pool now (the
                // complement of the stratified cal front — cal rows
                // excluded by construction).
                None => (inp.cal_cases, inp.cal_state_strs, inp.train),
            };
        #[cfg(feature = "option_cond")]
        let (mut probe, _) = {
            // Lever 1 swaps the fit corpus, so the option-conditioned
            // event table is rebuilt for THAT pool (oc_events_full stays
            // the deployed builds' table over the full train pool).
            let sel_events: Option<Vec<crate::option_cond::OcEvent>> =
                match (&gate_fit_slice, oc_events_full.as_ref()) {
                    (Some(sel), Some(_)) => Some(oc_events_for(inp.pool_rows, &sel.pool)),
                    _ => None,
                };
            match sel_events.as_ref().or(oc_events_full.as_ref()) {
                Some(ev) => build_engine_oc_with::<N>(
                    spec.name,
                    corpus_pool,
                    inp.labels,
                    effective_cap,
                    default_cfg.clone(),
                    &[],
                    ev,
                )?,
                None => build_engine::<N>(
                    spec.name,
                    corpus_pool,
                    inp.labels,
                    effective_cap,
                    default_cfg.clone(),
                )?,
            }
        };
        #[cfg(not(feature = "option_cond"))]
        let (mut probe, _) = build_engine::<N>(
            spec.name,
            corpus_pool,
            inp.labels,
            effective_cap,
            default_cfg.clone(),
        )?;
        let mut sc: Scratch<EMBED_DIM> = Scratch::new();
        let max_fit_q = fit_cases
            .iter()
            .map(|c| c.questions.len())
            .max()
            .unwrap_or(1);
        sc.prepare(max_fit_q);
        // Labeled fit-slice observations for the threshold-recommendation
        // surface (Issue 009): score axis = the engine readout confidence,
        // distance axis = the corpus-distance gate's abstain confidence;
        // correct = the probe's own pick vs gold on the case's LAST
        // question (the eval_engine convention, incl. the Noul [no, yes]
        // wire flip). The percentile posture selects on score only — the
        // label feeds the accuracy DISCLOSURE, never the threshold, so
        // the T2 migration stays byte-identical.
        let mut score_obs: Vec<GateObservation> = Vec::new();
        let mut distance_obs: Vec<GateObservation> = Vec::new();
        // Issue 056 direction 0: per-question (readout conf, correct) pairs
        // over the SAME fit slice the probe solves — in the default posture
        // these are byte-identical to the deployed calibrator's `cal_pairs`
        // (same build, same requests), so the lever below reconstructs the
        // EXACT deployed fit.
        let mut fit_pairs: Vec<(f32, bool)> = Vec::new();
        for (ci, case) in fit_cases.iter().enumerate() {
            let req = engine_request(case, &fit_state_strs[ci])?;
            probe
                .solve_into(&req, &mut sc)
                .map_err(|e| format!("cal probe ({}, case {ci}): {e}", case.id))?;
            for (qi, slot) in sc.slots.iter().enumerate() {
                let pick = match case.questions.get(qi).map(|q| q.kind) {
                    // Engine internal noul order is [yes, no]; gold uses
                    // the wire [no, yes] convention (eval_engine's flip).
                    Some(QKind::Noul) => (1 - slot.pick) as usize,
                    _ => slot.pick as usize,
                };
                let correct = case.gold.get(qi).is_some_and(|g| g.idx == pick);
                fit_pairs.push((slot.confidence, correct));
            }
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
        // Issue 056 direction 0 (`--gate-fit-calibrated`): fit the score
        // axis threshold on the CALIBRATED scale the deployed gate applies.
        // The probe's own per-question pairs reconstruct the deployed
        // calibrator exactly (same evidence window, same capacity /
        // min-obs), so the ρ=30 percentile reads the same scale `solve`
        // compares against — closing the fit-on-raw / apply-on-calibrated
        // mismatch that calibration saturation turned into 90–99% abstain
        // (saturate-at-0 suites) or a disarmed score axis (saturate-at-1,
        // banking77). Honest limit: on a SATURATED fit the calibrated
        // scale carries no information, so the threshold degenerates with
        // it — this lever makes the gate HONEST, the substrate saturation
        // guard (Issue 056 repair 2) restores the information.
        if inp.gate_fit_calibrated {
            let mut cal = katgpt_core::sigmoid_calibration::SigmoidGateCalibrator::new(
                default_cfg.cal_capacity,
                default_cfg.cal_min_obs,
            );
            for (c, ok) in &fit_pairs {
                cal.observe(*c, *ok);
            }
            cal.refit();
            for obs in &mut score_obs {
                obs.score = cal.apply(obs.score);
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
        let (score_threshold, distance_threshold) =
            resolve_gate_thresholds(&rec, &default_cfg, inp.gate_distance_only);
        (score_threshold, distance_threshold, rec)
    };

    Ok(FittedPosture {
        effective_cap,
        selection,
        head_selected,
        nb_selected,
        #[cfg(feature = "option_cond")]
        oc_selected,
        #[cfg(feature = "nb_ridge")]
        ridge_selected,
        genome_selected,
        default_cfg,
        score_threshold,
        distance_threshold,
        threshold_recommendation,
    })
}

#[allow(clippy::too_many_lines)]
fn run_modelless<const N: usize>(
    inp: &ModellessInput<'_>,
) -> Result<(LaneResult, super::cascade::ModellessQuestions), String> {
    let spec = inp.spec;
    let suite = inp.suite;
    let train = inp.train;
    let state_strs = inp.state_strs;
    let cal_cases = inp.cal_cases;
    let cal_state_strs = inp.cal_state_strs;
    let labels = inp.labels;
    let want_by_type = inp.want_by_type;
    let t_start = Instant::now();

    let FittedPosture {
        effective_cap,
        selection,
        head_selected,
        nb_selected,
        #[cfg(feature = "option_cond")]
        oc_selected,
        #[cfg(feature = "nb_ridge")]
        ridge_selected,
        genome_selected,
        default_cfg,
        score_threshold,
        distance_threshold,
        threshold_recommendation,
    } = fit_posture_inner::<N>(inp)?;

    // The event-aware build path (issue 038 T7b): when the lever armed,
    // every engine built from here on (readout report included) carries the
    // (qid, option) tables. None → the plain path, byte-identical.
    #[cfg(feature = "option_cond")]
    let oc_events_full: Option<Vec<crate::option_cond::OcEvent>> =
        (default_cfg.oc_scale > 0.0).then(|| oc_events_for(inp.pool_rows, train));
    #[cfg(not(feature = "option_cond"))]
    let oc_events_full: Option<std::convert::Infallible> = None;

    // ── Confidence-readout CANDIDATE report (Issue 039 T4, REPORT-ONLY —
    // the arming lever was DEMOTED at Bench 052: on the wide suites it
    // targets, Dispatch IS max_prob and inv_entropy measured worse; where a
    // candidate won on cal, arming it overfit and regressed emotion's test
    // G1). The wide-label G1 failures were closed by T2's stratified cal
    // slice. The table is recorded for instrumentation; the shipped
    // Dispatch law runs everywhere. (Report-only and pure: it runs after
    // the fused-gate fit now that the posture prologue is shared with the
    // seat — no shared state, identical outputs.)
    let readout_report = if cal_cases.is_empty() {
        None
    } else {
        readout_report_on_cal::<N>(
            spec.name,
            cal_cases,
            cal_state_strs,
            train,
            labels,
            effective_cap,
            &default_cfg,
            #[cfg(feature = "option_cond")]
            oc_events_full.as_deref(),
            #[cfg(not(feature = "option_cond"))]
            (),
        )?
    };
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };

    // The event-aware build path (issue 038 T7b): when the lever armed,
    // every engine built from here on carries the (qid, option) tables.
    // None → the plain path, byte-identical.
    let corpus_pool: &[TrainDoc] = train;
    let build_at_posture =
        |cfg: EngineConfig| -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String> {
            #[cfg(feature = "option_cond")]
            if let Some(ev) = oc_events_full.as_ref() {
                return build_engine_oc_with::<N>(
                    spec.name,
                    corpus_pool,
                    labels,
                    effective_cap,
                    cfg,
                    &[],
                    ev,
                );
            }
            #[cfg(not(feature = "option_cond"))]
            let _ = &oc_events_full;
            build_engine::<N>(spec.name, corpus_pool, labels, effective_cap, cfg)
        };

    // Corpus corpora: eval-cap docs per label, from the SAME pool the
    // thresholds were fitted on (the complement of the stratified cal
    // front — cal rows excluded by construction, Issue 039 T2).
    let (mut raw_engine, corpus_fallbacks) = build_at_posture(cfg.clone())?;
    if !corpus_fallbacks.is_empty() {
        eprintln!(
            "  [issue-039 corpus guard] {}: {} option label(s) with NO train docs in the \
             corpus pool → self-doc fallback (scores nearly uninformative for them): {}",
            spec.name,
            corpus_fallbacks.len(),
            corpus_fallbacks.join(", ")
        );
    }

    // Calibration pairs from a RAW (uncalibrated) engine over the cal slice.
    let (mut cal_engine, _) = build_at_posture(cfg.clone())?;
    let cal_cases: Vec<SuiteCase> = if cal_cases.is_empty() {
        Vec::new()
    } else {
        cal_cases.to_vec()
    };
    let (cal_eval, _) = if cal_cases.is_empty() {
        // Synthetic suites carry their own cal slice; empty here means no
        // cal.
        (
            Eval {
                probs: vec![],
                picks: vec![],
                confs: vec![],
                abstained: vec![],
                causes: vec![],
            },
            Latency {
                p50_ms: 0.0,
                p99_ms: 0.0,
                tail_support: 0,
                extremes: None,
                determinism_ok: None,
                determinism_n: None,
                durs_us: Vec::new(),
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
    // Issue 056 repair 2/3 instrument: dump the EXACT cal-window pairs the
    // deployed calibrator fit on (the `RIIR_DEBUG_CAL_RANK` precedent —
    // stderr, opt-in, never a metric). These are the bytes the katgpt-rs
    // saturation-guard fixtures replay (reflex Issue 056 → katgpt-rs
    // Issue 909): same pairs → same FIFO window → same fitted params.
    if std::env::var_os("RIIR_DEBUG_CAL_WINDOW").is_some() {
        eprintln!("[cal-window] {} n={}", spec.name, cal_pairs.len());
        for p in &cal_pairs {
            eprintln!("[cal-window] {:e} {}", p.conf, u8::from(p.correct));
        }
    }

    // RAW eval over the test cases (uncalibrated readout + raw abstain).
    let (raw_eval, lat) = eval_engine(&mut raw_engine, &suite.cases, state_strs, true)?;

    // CALIBRATED engine: fit on the cal pairs, then re-eval the test cases.
    // (The calibrator window config is read here — `cfg` moves into
    // `build_at_posture` — so the nli G1 posture can reuse the SAME
    // calibrator family below.)
    let (nli_cal_capacity, nli_cal_min_obs) = (cfg.cal_capacity, cfg.cal_min_obs);
    let fitted_cfg_for_transductive = cfg.clone();
    let (mut fitted, _) = build_at_posture(cfg)?;
    let mut moved = false;
    for p in &cal_pairs {
        moved |= fitted.observe(p.conf as f32, p.correct);
    }
    let (cal_eval_test, _) = eval_engine(&mut fitted, &suite.cases, state_strs, false)?;

    // Issue 042 lever 3: the worthiness probe's modelless half — the SAME
    // calibrated gate over the suite's CAL slice (the deployed abstain rule
    // measured in-distribution on the selection slice; never a test read).
    // None when the lever is off or the suite has no cal slice — the probe
    // then reports unprobed and the pre-lever behavior (armed) holds.
    let cal_gate = if inp.cascade_worthiness && !cal_cases.is_empty() {
        let (cal_fitted_eval, _) = eval_engine(&mut fitted, &cal_cases, cal_state_strs, false)?;
        Some(super::cascade::CalGateRecords {
            picks: cal_fitted_eval.picks,
            abstained: cal_fitted_eval.abstained,
        })
    } else {
        None
    };

    // Issue 072 (`--dump-items`): the per-item outcome records — the same
    // walk the metrics read, materialized once here so the CLI can persist
    // them without re-running anything. Join key: (case_id, q_idx).
    let items = {
        let mut out = Vec::with_capacity(raw_eval.n_questions());
        for (ci, case) in suite.cases.iter().enumerate() {
            for (qi, q) in case.questions.iter().enumerate() {
                let cause_cal = cal_eval_test.causes[ci][qi];
                out.push(super::item_dump::ItemOutcome {
                    lane: "modelless",
                    model: "modelless".to_string(),
                    case_id: case.id.clone(),
                    case_idx: ci,
                    q_idx: qi,
                    kind: q.kind.as_str().to_string(),
                    gold: case.gold[qi].idx,
                    pick: raw_eval.picks[ci][qi],
                    ok: raw_eval.picks[ci][qi] == case.gold[qi].idx,
                    conf: raw_eval.confs[ci][qi],
                    conf_cal: Some(cal_eval_test.confs[ci][qi]),
                    abstain_raw: Some(raw_eval.abstained[ci][qi]),
                    abstain_cal: Some(cal_eval_test.abstained[ci][qi]),
                    cause_cal: cause_cal
                        .abstained()
                        .then(|| cause_cal.as_str().to_string()),
                });
            }
        }
        out
    };

    // Issue 038 T4′: the cascade lane composes THIS lane's answers with the
    // escalator's — the forced picks that define `hard` above plus the
    // calibrated gate's abstain flags (the shipped posture). Cheap clones
    // (label indices + bools), produced unconditionally so the lane's own
    // surface never depends on the consumer's flag.
    let modelless_questions = super::cascade::ModellessQuestions {
        picks: raw_eval.picks.clone(),
        abstained: cal_eval_test.abstained.clone(),
        cal_gate,
    };

    // ── metrics assembly (label space; noul [no, yes]) ──
    let forced = raw_eval.forced_rows(&suite.cases);
    let hard = hard_metrics(&forced);
    // Plan 011 §B3: the JDI reference columns from the same forced rows.
    let jdi_chance = chance_majority(&forced.iter().map(|(g, _)| *g).collect::<Vec<usize>>());
    let jdi_skill = chance_corrected_skill(hard.accuracy, jdi_chance);
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
    // Issue 073 T4 field: the Brier twin of the readout ECE, on the same
    // calibrated pairs (report-only — the calibration axis beside the
    // G1/ECE one).
    let readout_brier_cal = brier_of(&readout_pairs_cal);
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

    // ── The transductive column (issue 038): after the honest test read,
    // from the honest RAW eval's own forced picks — never gold.
    let transductive = nb_lane::transductive_pass::<N>(
        inp,
        &raw_eval,
        hard.accuracy,
        &fitted_cfg_for_transductive,
        corpus_pool,
        effective_cap,
        #[cfg(feature = "option_cond")]
        oc_events_full.as_deref(),
        #[cfg(not(feature = "option_cond"))]
        (),
    )?;

    let cap_source = if selection.is_some() {
        "cal-slice selection (stratified slice; corpora exclude the selection docs)"
    } else if spec.synthetic.is_some() || spec.corpus_cap_per_label == usize::MAX {
        "registry (selection n/a: self-corpora)"
    } else {
        inp.cap_source_base
    };

    // Issue 057: the digest covers exactly the timed engine's corpus — the
    // capped drafter docs + the count-table sets when the final posture
    // arms them (`fitted_cfg_for_transductive` is the same cfg every
    // `build_at_posture` call ran; the timed engines carry no extra_nb).
    // ⚠ coupling, not proof: this RE-DERIVES the docs the engines consume
    // (same inputs, deterministic construction — identical today). The
    // guard is the extraction: `specs_from_pool` itself calls
    // [`per_label_corpus_docs`], so an engine-call-site change (a
    // different cap or a non-empty extra_nb) cannot compile without
    // touching this site — but a future divergence here would make the
    // digest describe a corpus that was never timed (verdict round 2's
    // note; the fix then is passing one `drafter_docs` to BOTH sides).
    let corpus_digest = {
        let (drafter_docs, _) = per_label_corpus_docs(train, labels, effective_cap);
        let nb_sets = nb_sets_for(
            f64::from(fitted_cfg_for_transductive.nb_scale),
            train,
            labels,
            &[],
        );
        corpus_digest(spec.name, labels, &drafter_docs, nb_sets.as_deref())
    };
    Ok((
        LaneResult {
            lane: "modelless",
            model: "modelless".to_string(),
            hard,
            readout_ece: Some(readout_ece_cal),
            // Issue 073 T4 field, modelless lane: the readout Brier twin of
            // readout_ece — computed on the same calibrated readout pairs the
            // G1 tail reads. Report-only; never a gate.
            readout_brier: Some(readout_brier_cal),
            raw_abstain: Some(raw_abstain),
            calibrated_abstain: Some(calibrated_abstain),
            // Issue 060: the CALIBRATED gate's causes — the why behind the
            // published abstain rate (the raw engine's own gate runs un-fitted
            // thresholds; its causes are not the published story).
            abstain_causes: Some(cal_eval_test.abstain_causes()),
            readout_ece_raw: Some(readout_ece_raw),
            readout_ece_calibrated: Some(readout_ece_cal),
            floor_ece: Some(floor_ece),
            g1_pass,
            g1_verdict: Some(g1_verdict),
            drex_conf_readout: None,
            by_question_type,
            soft_acc,
            brier_soft,
            score_mae,
            within_1,
            latency_p50_ms: lat.p50_ms,
            latency_p99_ms: lat.p99_ms,
            latency_tail_support: lat.tail_support,
            acc_deleaked,
            jdi_chance: Some(jdi_chance),
            jdi_skill: Some(jdi_skill),
            readout_report,
            corpus_fallbacks,
            latency_extremes: lat.extremes,
            determinism_ok: lat.determinism_ok,
            determinism_n: lat.determinism_n,
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
            corpus_digest: Some(corpus_digest),
            head_selection: head_selected,
            nb_selection: nb_selected,
            #[cfg(feature = "option_cond")]
            oc_selection: oc_selected,
            #[cfg(feature = "nb_ridge")]
            ridge_selection: ridge_selected,
            genome_selection: genome_selected,
            transductive,
            confusion: Some(confusion_rows(&raw_eval, &suite.cases, CONFUSION_TOP)),
            items,
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
            nli_feature_ab: if inp.nli_feature_ab {
                nli_lane::nli_feature_ab_pass(
                    &raw_eval,
                    suite,
                    &cal_eval,
                    &cal_cases,
                    Some(nli_lane::G1Inputs {
                        engine_calibrated_test_ece: readout_ece_cal,
                        conformal_floor_test: floor_ece,
                        cal_capacity: nli_cal_capacity,
                        cal_min_obs: nli_cal_min_obs,
                    }),
                )?
            } else {
                None
            },
            nli_m1: if inp.nli_m1 {
                // Issue 047 M1: the fit pool (cal front + corpus-excluded
                // train rest, premise-guarded) is built at the lane's own
                // corpus law, then read by BOTH deployed engines — the pass
                // itself never touches an engine.
                let (pool_cases, n_premise_excluded) =
                    nli_m1::build_pool_cases(&cal_cases, inp.pool_rows, effective_cap)?;
                let pool_state_strs: Vec<String> = pool_cases
                    .iter()
                    .map(|c| serialize_state(&c.state))
                    .collect();
                let (pool_raw, _) =
                    eval_engine(&mut raw_engine, &pool_cases, &pool_state_strs, false)?;
                let (pool_fitted, _) =
                    eval_engine(&mut fitted, &pool_cases, &pool_state_strs, false)?;
                nli_m1::nli_m1_pass(&nli_m1::M1Inputs {
                    spec_name: spec.name,
                    suite,
                    raw_eval: &raw_eval,
                    cal_eval_test: &cal_eval_test,
                    floor_pairs: &floor_pairs,
                    floor_ece,
                    pool_cases: &pool_cases,
                    pool_raw: &pool_raw,
                    pool_fitted: &pool_fitted,
                    n_premise_excluded,
                    cal_capacity: nli_cal_capacity,
                    cal_min_obs: nli_cal_min_obs,
                })?
            } else {
                None
            },
            #[cfg(feature = "mc_ensemble")]
            mc_ab: if let Some(mcc) = inp.mc_ab {
                Some(super::mc_ab::mc_ab_pass(
                    &mut fitted,
                    suite,
                    state_strs,
                    &cal_cases,
                    cal_state_strs,
                    super::mc_ab::McBaseline {
                        confs_calibrated: &cal_eval_test.confs,
                        confs_raw: &raw_eval.confs,
                        picks: &raw_eval.picks,
                    },
                    mcc,
                )?)
            } else {
                None
            },
            // Issue 066 (`--density-gate`): the density half's paired A/B —
            // report-only, over the SAME frozen test read the row publishes
            // (the shipped arm IS this row's calibrated gate; the density arm
            // rebuilds at the same posture + knob, calibrator re-fit on the
            // same cal pairs).
            #[cfg(feature = "density_gate")]
            density_ab: if inp.density_gate {
                let shipped = super::density_ab::ShippedArm {
                    abstained: &cal_eval_test.abstained,
                    picks: &cal_eval_test.picks,
                    confs: &cal_eval_test.confs,
                };
                let build = |dcfg: EngineConfig| build_at_posture(dcfg);
                Some(super::density_ab::density_ab_pass::<N>(
                    spec.name,
                    &build,
                    &fitted_cfg_for_transductive,
                    &cal_pairs,
                    &cal_cases,
                    cal_state_strs,
                    &suite.cases,
                    state_strs,
                    shipped,
                )?)
            } else {
                None
            },
            // Plan 627 / Issue 090 C2 (`--set-refine-ab`): the set-conditioned
            // state refinement's paired A/B — report-only, over the SAME
            // frozen test read the row publishes (the shipped arm IS this
            // row's posture; the refined arm rebuilds at the same posture +
            // the knob, calibrator re-fit on the same cal pairs).
            #[cfg(feature = "set_refine")]
            set_refine_ab: if let Some((gamma, tau)) = inp.set_refine {
                let build = |rcfg: EngineConfig| build_at_posture(rcfg);
                Some(super::set_refine_ab::set_refine_ab_pass::<N>(
                    spec.name,
                    &build,
                    &fitted_cfg_for_transductive,
                    gamma,
                    tau,
                    &cal_pairs,
                    &suite.cases,
                    state_strs,
                    &cal_eval_test.picks,
                    &cal_eval_test.abstained,
                    &cal_eval_test.confs,
                )?)
            } else {
                None
            },
        },
        modelless_questions,
    ))
}

/// The wire form of one suite case's questions, shared by the timed loop
/// and the pre-ramp warmup (identical construction, one home). `pub` for
/// ONE external consumer: riir-instinct's arena encoder arm (instinct
/// issue 014 C1) renders the identical question wire for its encode —
/// the law stays at this one home rather than a law copy that can drift.
pub fn case_questions(case: &SuiteCase) -> Vec<(String, Value)> {
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
    questions.shrink_to_fit();
    questions
}

// ── prepared suite + run ────────────────────────────────────────────────

struct Prepared {
    suite: Suite,
    train: Vec<TrainDoc>,
    state_strs: Vec<String>,
    cal_cases: Vec<SuiteCase>,
    cal_state_strs: Vec<String>,
    labels: Vec<String>,
    /// The corpus-pool envelope (dataset suites: train minus the stratified
    /// cal front) — the selection stratification's input. Null on the
    /// synthetic/code paths (those suites are selection-ineligible).
    pool_rows: Value,
    /// Issue 058: the slice-integrity audit + identity digests. None on the
    /// synthetic/code paths (their slices are authored, not composed).
    slices: Option<super::slice_guard::SliceFacts>,
}

fn prepare(spec: &SuiteSpec, dir: &Path) -> Result<Prepared, String> {
    if let Some(synth) = spec.synthetic {
        if !spec.modelless_lane && !cfg!(feature = "laya-riir") && !cfg!(feature = "clm-lane") {
            // Unreachable over the current registry (every suite is
            // modelless since Issue 045) — the declared-inability skip is
            // kept for any future `modelless_lane: false` suite.
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
            pool_rows: Value::Null,
            slices: None,
        });
    }

    let suite_dir = dir.join(spec.dataset_dir);
    let eval_rows = load_rows(&suite_dir, spec.eval_split)?;
    // Issue 039 T2: the test sample is a label-STRATIFIED round-robin over
    // the whole split, not the first-N prefix — on label-sorted mirrors the
    // prefix is a label fraction (banking77's 500 first rows spanned 13 of
    // 77 labels; massive's 300 spanned 30 of 60), so every lane scored a
    // non-representative slice while the questions still offered every
    // label. Budget 0 (all rows) is the identity split, so uncapped suites
    // never move.
    let eval_split = stratified_split(&eval_rows, spec.name, spec.test_cap);
    let suite = (spec.build)(&eval_split.front, 0);
    let train_rows =
        load_rows(&suite_dir, "train").map_err(|e| format!("suite {}: {e}", spec.name))?;
    // The cal slice is stratified by the SAME law, and the corpus pool is
    // the split's REST (cal rows excluded by construction, not position —
    // the positional `train[cal_cap..]` cut was only correct while the cal
    // slice was the first-N prefix; on a label-clustered train mirror it
    // also ORPHANED every label whose whole block sat inside the prefix).
    let cal_split = stratified_split(&train_rows, spec.name, spec.cal_cap);
    let train = train_docs(&cal_split.rest, spec.name);
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
    //   keys over the TEST cases (train docs carry label_text).
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
        // Plan 010 S1MB noul: a 2-label suite ("false"/"true" — the gold
        // key strings the corpus docs carry); the option-key union is empty
        // (noul presents no keys), so the labels ride the train-doc
        // vocabulary exactly like the exempt branch below describes.
        "s1mb_noul" => vec!["false".to_string(), "true".to_string()],
        "s1mb_choice" | "s1mb_score" => option_key_union.clone(),
        // Issue 077: the fixed-criteria classification suites name their
        // engine domains by the OPTION KEYS (content binding for the
        // by-name option→domain resolution). train_row_label maps the
        // rows' int class labels through the SAME key consts, so corpora,
        // stratification, and the slice audit all read one spelling. The
        // count guards below pin the fetch (the old int-label vectors
        // hardcoded these counts; the union now carries them).
        "ag_news" | "emotion" | "sst5" | "xnli_en" | "xnli_en_val" | "thai_wisesight"
        | "wanli_en" => option_key_union.clone(),
        // Noul-only: no option keys — the classes ride the train-doc
        // integer labels (the exemption the slice audit documents).
        "prompt_injections" => vec!["0".to_string(), "1".to_string()],
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
    // the train-doc labels). Issue 077: the labels ARE the option-key union
    // now (content binding), so the fetch guard is the EXPECTED KEY SET —
    // an exact membership pin, not a bare count (a swapped key would
    // otherwise silently rebind every corpus).
    {
        let expected: Option<Vec<&str>> = match spec.name {
            "ag_news" => Some(crate::harness::suites::AG_NEWS_KEYS.to_vec()),
            "emotion" => Some(crate::harness::suites::EMOTION_KEYS.to_vec()),
            "sst5" => Some(vec!["0", "1", "2", "3", "4"]),
            "xnli_en" | "xnli_en_val" => Some(crate::harness::suites::XNLI_KEYS.to_vec()),
            // Issue 080: wanli_en shares the xnli key consts — the gold
            // string maps through the SAME order, so the expected-key pin is
            // the same set.
            "wanli_en" => Some(crate::harness::suites::XNLI_KEYS.to_vec()),
            // Plan 012: the JDI panel — anli shares the xnli ClassLabel
            // order; winogrande/hellaswag are fixed positional wires with
            // their own consts.
            "anli_jdi" => Some(crate::harness::suites::XNLI_KEYS.to_vec()),
            "winogrande_jdi" => Some(crate::harness::suites::WINOGRANDE_JDI_KEYS.to_vec()),
            "hellaswag_jdi" => Some(crate::harness::suites::HELLASWAG_JDI_KEYS.to_vec()),
            "thai_wisesight" => Some(crate::harness::suites::WISESIGHT_KEYS.to_vec()),
            "prompt_injections" => Some(vec!["0", "1"]),
            _ => None,
        };
        if let Some(expected) = expected {
            let spelled: Vec<&str> = labels.iter().map(String::as_str).collect();
            assert_eq!(
                spelled, expected,
                "suite {}: the option-key universe drifted from the fetch law — \\
                 the engine domains would silently rebind (issue 077's content \\
                 binding keys corpora to option CONTENT, so the set is pinned)",
                spec.name
            );
        }
        // Plan 012: mmlupro_jdi's union is DATA-DERIVED per eval slice (the
        // letters the presented cases' option counts fill), so the pin is
        // MEMBERSHIP in the A..J letter convention, not equality.
        if spec.name == "mmlupro_jdi" {
            assert!(
                labels
                    .iter()
                    .all(|l| crate::harness::suites::MMLUPRO_JDI_LETTERS.contains(&l.as_str())),
                "mmlupro_jdi: the option-key union drifted off the A..J letter \\
                 convention — {labels:?}"
            );
        }
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
    // Plan 003: the whole 204-row Thai test split must present the full
    // 7-topic universe — a partial fetch would bind empty corpora to the
    // missing domains (the banking77 guard, one suite over).
    if spec.name == "thai_sib200" {
        assert_eq!(
            option_key_union.len(),
            7,
            "thai_sib200: the TEST suite presents {} option keys (need all 7 — \
             fetch the full test split)",
            option_key_union.len()
        );
    }

    // Calibration cases: the SAME builder over the stratified cal front
    // (identical question shapes; gold from the train labels).
    let (cal_cases, cal_state_strs) = if spec.cal_cap > 0 {
        let cal_suite = (spec.build)(&cal_split.front, 0);
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

    // Issue 058 — the slice-integrity audit (the three-incident class:
    // Issue 023's misaligned cal read, Issue 039 T2's positional pool cut,
    // Bench 076's silent datasets-dir move). The audit is content-based —
    // raw row identity, not envelope positions — and hard-refuses the suite
    // on any overlap or an unrepresentative test sample; the slice digests
    // ride into results.json either way so a board diff can see "same
    // pool?" mechanically (the axis the 076 publish sailed through on).
    let mut slices = super::slice_guard::audit_envelopes(
        spec.name,
        &labels,
        &eval_split.front,
        &cal_split.front,
        &cal_split.rest,
        spec.test_cap,
        &|row| train_row_label(spec.name, row),
        spec.coverage_audit,
    );
    if !slices.is_clean() {
        // Issue 058 (verdict route 1): a frozen pool whose published
        // records were measured on its dirty bytes is accepted UNDER ITS
        // EXACT membership pin — same triple = same bytes, so the pin can
        // never admit a different defect; a repair or re-pull moves the
        // triple and the refusal returns until the pin is deleted in the
        // same change. The acceptance is loud at the run site and recorded
        // in results.json (`acknowledged`).
        match slices.known_dirty_acknowledgement(spec.name) {
            Some(pin) => {
                eprintln!(
                    "    ⛔ KNOWN-DIRTY SLICE (pinned) — proceeding under acknowledgement:\n    {}\n    pin reason: {}",
                    slices.summary_line(),
                    pin.reason
                );
                slices.acknowledged = true;
            }
            None => {
                return Err(format!(
                    "suite {}: slice-integrity refusal — {}; slice state: {}",
                    spec.name,
                    slices.violations.join("; "),
                    slices.summary_line()
                ));
            }
        }
    }

    Ok(Prepared {
        suite,
        train,
        state_strs,
        cal_cases,
        cal_state_strs,
        labels,
        pool_rows: cal_split.rest,
        slices: Some(slices),
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

/// The seat a downstream hybrid/instinct lane plugs into (riir-instinct
/// Issue 003 T3): the harness's per-suite preparation, the DEPLOYED
/// modelless posture, and the per-question eval, exposed so the consumer
/// runs byte-identical questions on the SAME engine posture without
/// re-deriving any of it. The dep edge is ONE-WAY — reflex never depends
/// on the consumer (the lane crate lives in the consumer repo); the seat
/// is the whole public surface the lane needs.
pub mod seat {
    use super::*;

    /// A prepared dataset suite: the byte-identical question sets, the
    /// corpus pool (train docs minus the stratified cal front), the cal
    /// front, and the engine label universe. `pool_rows` feeds the
    /// posture selections' stratification ([`fit_posture`] consumes it
    /// internally; callers rarely name it).
    pub struct Seat {
        pub suite: Suite,
        pub train: Vec<TrainDoc>,
        pub state_strs: Vec<String>,
        pub cal_cases: Vec<SuiteCase>,
        pub cal_state_strs: Vec<String>,
        pub labels: Vec<String>,
        pub pool_rows: Value,
        /// Issue 049 T1's posture-fork defence: true when the seat came
        /// from an in-process synthetic build (`semantic_defects`) rather
        /// than dataset files. The consumer cannot mistake it for a
        /// dataset seat — a synthetic seat carries authored fixtures with
        /// programmatic gold, and its published row must disclose the
        /// posture (never render as a dataset row).
        pub synthetic: bool,
        /// The seated synth corpus (Plan 426 T5's V5-PASS lever, seated at
        /// Plan 426 T6): blake3-verified at [`prepare_seat_with_synth`],
        /// in-universe filtered, applied at [`build_seat_engine`] BUILD time
        /// — the posture ([`fit_posture`]) stays gold-pool-fit, so the
        /// corpus is the only difference from the plain seat (the V5
        /// construction). `None` = byte-identical to the pre-overlay seat.
        pub synth: Option<SynthSeatCorpus>,
    }

    /// A verified synth corpus seated on a suite: the sidecar-checked rows
    /// (integrity lives in [`load_synth_corpus`]), in-universe filtered
    /// (out-of-universe rows are dropped COUNTED, never silent), plus the
    /// provenance a served receipt must disclose.
    pub struct SynthSeatCorpus {
        pub docs: Vec<TrainDoc>,
        pub extra_cap: usize,
        /// The artifact path + its sidecar-verified BLAKE3 — the corpus
        /// identity (Issue 057's axis): a seat whose corpus differs must be
        /// able to SAY so.
        pub path: String,
        pub digest_hex: String,
        pub rows_total: usize,
        pub rows_dropped: usize,
    }

    /// Prepare a suite by name. `semantic_defects` seats through its
    /// in-process build (Issue 049 T1 — the seat marks it `synthetic`, so
    /// the posture fork the old blanket refusal feared is now explicit;
    /// the six Issue-004 families and `code_fixtures` that used to seat
    /// here were retired 2026-10-02 / 2026-10-11); dataset suites are
    /// unchanged.
    pub fn prepare_seat(name: &str, dir: &Path) -> Result<Seat, String> {
        let spec = SUITES
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("seat: unknown suite {name}"))? as &SuiteSpec;
        let synthetic = spec.synthetic.is_some();
        if synthetic && !spec.modelless_lane {
            // Unreachable over the current registry (Issue 045 made every
            // synthetic suite modelless) — the declared-inability refusal
            // is kept for any future `modelless_lane: false` suite: the
            // seat's only posture is the modelless one, and a suite the
            // modelless lane cannot answer refuses rather than forks.
            return Err(format!(
                "seat: {name} refuses a modelless seat (no honest modelless answer \
                 for this suite); compile laya-lane consumers answer it through \
                 the laya lane instead"
            ));
        }
        let p = prepare(spec, dir)?;
        Ok(Seat {
            suite: p.suite,
            train: p.train,
            state_strs: p.state_strs,
            cal_cases: p.cal_cases,
            cal_state_strs: p.cal_state_strs,
            labels: p.labels,
            pool_rows: p.pool_rows,
            synthetic,
            synth: None,
        })
    }

    /// [`prepare_seat`] with the suite's synth corpus seated (Plan 426
    /// T6's serve posture): the artifact is blake3-verified + its suite
    /// header checked against the seat's name (a mismatch is a loud
    /// refusal — a corpus is never silently swapped across suites), and
    /// rows outside the engine's label universe are dropped COUNTED.
    pub fn prepare_seat_with_synth(
        name: &str,
        dir: &Path,
        synth_path: &Path,
        extra_cap: usize,
    ) -> Result<Seat, String> {
        let mut seat = prepare_seat(name, dir)?;
        let (meta, docs, digest_hex) = load_synth_corpus(synth_path)?;
        if meta.suite != name {
            return Err(format!(
                "synth corpus {} carries suite {:?}, not {name:?} — refusing the \\\n                 cross-suite seat",
                synth_path.display(),
                meta.suite
            ));
        }
        let rows_total = docs.len();
        let docs: Vec<TrainDoc> = docs
            .into_iter()
            .filter(|d| seat.labels.iter().any(|l| l == &d.label))
            .collect();
        let rows_dropped = rows_total - docs.len();
        if rows_dropped > 0 {
            eprintln!(
                "  [seat-synth {name}] {} synth row(s) outside the engine's {}-label \\\n                 universe — dropped (counted, never silent)",
                rows_dropped,
                seat.labels.len()
            );
        }
        seat.synth = Some(SynthSeatCorpus {
            docs,
            extra_cap,
            path: synth_path.display().to_string(),
            digest_hex,
            rows_total,
            rows_dropped,
        });
        Ok(seat)
    }

    impl Seat {
        /// Population identity of this seat's test cases — the SAME law the
        /// harness's `SuiteResult.cases_digest` uses ([`cases_digest`]:
        /// fnv1a64 over the suite name + the canonical serde of the case
        /// vector). Exposed so a consumer's frozen record can pin
        /// case-identity against any reflex-side doc (the plan-011 C2
        /// case-identity pin: a crosswalk cell publishes only on digest
        /// equality — recomputing the law downstream would drift, so the
        /// consumer calls THIS).
        pub fn cases_digest(&self) -> String {
            cases_digest(self.suite.name, &self.suite.cases)
        }
    }

    /// Posture-selection knobs (the deployed registry posture: selections
    /// on, cap candidates from the CLI convention). Empty
    /// `cal_select_caps` = the selection is off and the registry default
    /// cap holds.
    #[derive(Debug, Clone, Default)]
    pub struct PostureKnobs {
        pub head_select: bool,
        pub nb_select: bool,
        /// Option-conditioned selection (issue 038 T7b). Needs the
        /// `option_cond` feature; a knob set without it is a silent no-op
        /// at the seat level — the HARNESS level refuses loud (the CLI
        /// flag), so seat callers on a feature-less build get the baseline
        /// posture by construction.
        #[cfg(feature = "option_cond")]
        pub oc_select: bool,
        #[cfg(feature = "nb_ridge")]
        pub ridge_select: bool,
        /// Joint blend-genome selection (issue 038 T5). Needs `nb_scope`
        /// the same way — silent no-op at the seat level, loud at the CLI.
        pub genome_select: bool,
        /// The genome acceptance bar; `<= 0` = the house arming bar.
        pub genome_accept_margin: f64,
        pub cal_select_caps: Vec<usize>,
        /// The fused-gate RATE levers (issue 042 levers 1–2, Benches
        /// 063/066): fit the gate thresholds on the stratified selection
        /// slice (`gate_fit_selection`) and disable the score axis
        /// (`gate_distance_only`). DEFAULT OFF — the seat's shipped face
        /// is the T1.6 cal-slice fused fit, byte-identical for every
        /// existing caller. Rethink's ESC lane (Rethink Issue 017 T1)
        /// arms these: the shipped gate abstains ~100% at the deployed
        /// postures (the measured Bench-066 pathology — H1 escalation
        /// 1.000 in the arena records), so a SELECTIVE escalation set
        /// needs the rate-axis posture.
        pub gate_fit_selection: bool,
        pub gate_distance_only: bool,
        /// The drafter-only correction mode (issue 079's tuning axis — the
        /// seat half of the harness `--drafter-fix` flag). DEFAULT `Off` =
        /// the shipped scores, byte-identical for every existing caller.
        /// The PUBLISHED face arms `PerByte` (the board posture: Bench 131
        /// cell 2 + the site republish `d3aeeae` — typed the only mover,
        /// every other suite digit-identical), so the seat callers that
        /// follow the published posture arm it. The correction engages
        /// only on the drafter-only path, so it is inert wherever
        /// by-name/route resolution arms.
        pub drafter_fix: crate::engine::DrafterFix,
    }

    /// The deployed modelless posture for one suite: the effective corpus
    /// cap, the FULL engine config (selected head/nb scales + the fitted
    /// fused-gate thresholds), and the selection rows for disclosure.
    pub struct SeatPosture {
        pub effective_cap: usize,
        pub cfg: EngineConfig,
        pub score_threshold: f32,
        pub distance_threshold: f32,
        pub threshold_recommendation: FusedGateRecommendation,
        pub head_selection: Option<HeadScaleSelection>,
        pub nb_selection: Option<NbSelection>,
        #[cfg(feature = "option_cond")]
        pub oc_selection: Option<super::oc_lane::OcSelection>,
        #[cfg(feature = "nb_ridge")]
        pub ridge_selection: Option<super::ridge_lane::RidgeSelection>,
        pub genome_selection: Option<genome_lane::GenomeSelection>,
        pub cap_candidates: Option<Vec<CapCandidate>>,
    }

    /// Fit the suite's deployed posture — the same [`FittedPosture`]
    /// prologue [`run_modelless`] uses, through the same code (one source
    /// of truth; the seat adds no selection of its own).
    pub fn fit_posture<const N: usize>(
        name: &str,
        s: &Seat,
        knobs: &PostureKnobs,
    ) -> Result<SeatPosture, String> {
        let spec = SUITES
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("seat: unknown suite {name}"))?;
        let inp = ModellessInput {
            spec,
            cascade_worthiness: false,
            // The seat keeps the SHIPPED posture by default (the arena's
            // published face is the T1.6 cal-slice fused fit, never a
            // lever arm) — but the rate levers are CALLER-CHOICE now
            // (Rethink Issue 017 T1's ESC gate; the defaults preserve
            // every existing caller byte-identically). The Issue-056
            // scale-coherence fit stopped being a lever at its
            // 2026-09-30 promotion (verdict round 2: the fit-on-raw /
            // apply-on-calibrated mismatch is a units bug). The seat
            // carries the fix like every other deployed surface.
            gate_fit_selection: knobs.gate_fit_selection,
            gate_distance_only: knobs.gate_distance_only,
            gate_fit_calibrated: true,
            // Issue 079: the seat keeps the engine's INCUMBENT route law
            // (`kn_route: true` — the index-anchored arm, `4ad25df`; the
            // arena's selection surface is the instinct manifest, not a
            // reflex CLI flag), while the drafter-only correction mode is
            // CALLER-CHOICE — the published face arms PerByte (Bench 131
            // cell 2 + the `d3aeeae` site republish), so seat callers that
            // follow the published posture can arm it; `Off` (the knob's
            // default) keeps every other caller byte-identical.
            kn_route: true,
            drafter_fix: knobs.drafter_fix,
            suite: &s.suite,
            train: &s.train,
            state_strs: &s.state_strs,
            cal_cases: &s.cal_cases,
            cal_state_strs: &s.cal_state_strs,
            labels: &s.labels,
            want_by_type: false,
            corpus_cap_per_label: spec.corpus_cap_per_label,
            head_scale: 0.0,
            head_select: knobs.head_select,
            nb_select: knobs.nb_select,
            #[cfg(feature = "option_cond")]
            oc_select: knobs.oc_select,
            #[cfg(not(feature = "option_cond"))]
            oc_select: false,
            #[cfg(feature = "nb_ridge")]
            ridge_select: knobs.ridge_select,
            #[cfg(not(feature = "nb_ridge"))]
            ridge_select: false,
            genome_select: knobs.genome_select,
            genome_accept_margin: if knobs.genome_accept_margin > 0.0 {
                knobs.genome_accept_margin
            } else {
                HEAD_SELECT_MARGIN
            },
            cap_source_base: "registry",
            cal_select_caps: &knobs.cal_select_caps,
            pool_rows: &s.pool_rows,
            pair_head_ab: false,
            nli_feature_ab: false,
            nli_m1: false,
            #[cfg(feature = "mc_ensemble")]
            mc_ab: None,
            density_gate: false,
            set_refine: None,
            leak_flags: None,
        };
        let fp = super::fit_posture_inner::<N>(&inp)?;
        Ok(SeatPosture {
            effective_cap: fp.effective_cap,
            cfg: EngineConfig {
                score_threshold: fp.score_threshold,
                distance_threshold: fp.distance_threshold,
                ..fp.default_cfg
            },
            score_threshold: fp.score_threshold,
            distance_threshold: fp.distance_threshold,
            threshold_recommendation: fp.threshold_recommendation,
            head_selection: fp.head_selected,
            nb_selection: fp.nb_selected,
            #[cfg(feature = "option_cond")]
            oc_selection: fp.oc_selected,
            #[cfg(feature = "nb_ridge")]
            ridge_selection: fp.ridge_selected,
            genome_selection: fp.genome_selected,
            cap_candidates: fp.selection.map(|c| c.rows),
        })
    }

    /// Build the suite's engine at a seat posture (the caller owns the
    /// config; [`fit_posture`] produced it). Returns the engine + the
    /// self-doc fallback labels (the Issue-039 disclosure).
    ///
    /// A seated synth corpus ([`Self::prepare_seat_with_synth`]) extends
    /// the build — gold cap first, synth beyond it, in BOTH the drafter
    /// corpus and the count tables (the V5 arm-B construction, one body
    /// with the corpus-ab lane via [`specs_corpus_extended`]). An armed
    /// option-conditioned posture refuses: the (qid, option) tables are
    /// fit from gold events the synth rows do not carry, so an extended
    /// build cannot honestly arm them (the corpus-ab lane's law).
    pub fn build_seat_engine<const N: usize>(
        name: &str,
        s: &Seat,
        cap: usize,
        cfg: EngineConfig,
    ) -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String> {
        // An armed option-conditioned posture (issue 038 T7b) needs the
        // (qid, option) events — derived from the same pool rows the
        // harness feeds, filtered to the seat's own corpus pool.
        #[cfg(feature = "option_cond")]
        if cfg.oc_scale > 0.0 {
            if s.synth.is_some() {
                return Err(format!(
                    "seat-synth {name}: the posture arms the option-conditioned tables — \\\n                     the +synth corpus cannot carry them; refuse rather than mis-build"
                ));
            }
            let events = super::oc_events_for(&s.pool_rows, &s.train);
            return super::build_engine_oc_with::<N>(
                name,
                &s.train,
                &s.labels,
                cap,
                cfg,
                &[],
                &events,
            );
        }
        #[cfg(not(feature = "option_cond"))]
        let _ = name;
        if let Some(synth) = &s.synth {
            let nb_sets =
                super::nb_sets_for(f64::from(cfg.nb_scale), &s.train, &s.labels, &synth.docs);
            let (specs, fallbacks, used) = super::specs_corpus_extended(
                &s.train,
                &synth.docs,
                &s.labels,
                cap,
                synth.extra_cap,
                nb_sets.as_deref(),
            );
            let _ = used;
            return DecisionEngine::<N, EMBED_DIM>::build_specs(specs, cfg)
                .map(|e| (e, fallbacks))
                .map_err(|e| format!("engine build ({name} +synth): {e}"));
        }
        super::build_engine::<N>(name, &s.train, &s.labels, cap, cfg)
    }

    /// One question's seat answer (label space; noul is [no, yes]).
    pub struct QuestionOut {
        pub probs: Vec<f64>,
        pub pick: usize,
        pub conf: f64,
        pub abstained: bool,
    }

    /// A seat evaluation over one case set: per-case answers + per-case
    /// latency percentiles (µs, nearest-rank; tail support disclosed).
    /// `durs_us` carries the PER-CASE wall times (the hybrid's H1 total
    /// is reflex + decision, so the consumer needs the vector, not just
    /// the percentiles).
    pub struct SeatEval {
        pub cases: Vec<Vec<QuestionOut>>,
        pub durs_us: Vec<u64>,
        pub p50_us: u64,
        pub p99_us: u64,
        pub tail_support: usize,
    }

    /// Evaluate a seat engine over a case set (the harness's own eval
    /// path, determinism probe off — the caller re-runs if it wants it).
    pub fn eval_seat<const N: usize>(
        engine: &mut DecisionEngine<N, EMBED_DIM>,
        cases: &[SuiteCase],
        state_strs: &[String],
    ) -> Result<SeatEval, String> {
        let (ev, lat) = super::eval_engine(engine, cases, state_strs, false)?;
        Ok(SeatEval {
            cases: ev
                .probs
                .iter()
                .zip(ev.picks.iter())
                .zip(ev.confs.iter())
                .zip(ev.abstained.iter())
                .map(|(((probs, picks), confs), abst)| {
                    probs
                        .iter()
                        .zip(picks.iter())
                        .zip(confs.iter())
                        .zip(abst.iter())
                        .map(|(((p, k), c), a)| QuestionOut {
                            probs: p.clone(),
                            pick: *k,
                            conf: *c,
                            abstained: *a,
                        })
                        .collect()
                })
                .collect(),
            durs_us: lat.durs_us.clone(),
            p50_us: (lat.p50_ms * 1000.0).round() as u64,
            p99_us: (lat.p99_ms * 1000.0).round() as u64,
            tail_support: lat.tail_support,
        })
    }

    /// The abstain-flags face of [`eval_seat`] (reflex issue 070, lead 1):
    /// the SAME eval path with no `SeatEval` copy layer — for consumers
    /// that read only the gate's abstain flags (the ESC gate leg). The
    /// flags are `eval_seat`'s own `abstained` field verbatim
    /// (case-major, question-minor); only the dropped probs/pick/conf
    /// copies and the duration vector differ.
    ///
    /// Serve consumers should prefer [`eval_case_into`] (lead 2) — the
    /// scratch-refill face that also reuses the engine scratch, the wire
    /// request and the result vecs across requests.
    pub fn eval_seat_abstained<const N: usize>(
        engine: &mut DecisionEngine<N, EMBED_DIM>,
        cases: &[SuiteCase],
        state_strs: &[String],
    ) -> Result<Vec<Vec<bool>>, String> {
        let (ev, _lat) = super::eval_engine(engine, cases, state_strs, false)?;
        Ok(ev.abstained)
    }

    /// The eval path's reusable frame (reflex issue 070, lead 2): the
    /// engine solve scratch, the wire request, and the per-question
    /// answers in ONE caller-owned buffer. A serve consumer holds it
    /// across requests — [`eval_case_into`] refills every `String`/`Vec`
    /// in place (keep-when-equal shape), so a warm frame pays zero
    /// eval-path allocation per request. The result fields hold the LAST
    /// eval's answers and stay readable until the next call.
    ///
    /// The take/replace idiom keeps the frame out of `&mut self` methods
    /// while its answers are read:
    ///
    /// ```ignore
    /// let mut frame = std::mem::take(&mut self.eval_frame);
    /// eval_case_into(&mut self.engine, case, state, &mut frame)?;
    /// // … read frame.picks / frame.abstained …
    /// self.eval_frame = frame;
    /// ```
    pub struct CaseEvalScratch {
        sc: Scratch<EMBED_DIM>,
        req: katgpt_core::decision_wire::DecisionRequest,
        /// Per-question label-space probabilities (noul is [no, yes]).
        pub probs: Vec<Vec<f64>>,
        /// Per-question picks (forced argmax when abstained).
        pub picks: Vec<usize>,
        pub confs: Vec<f64>,
        pub abstained: Vec<bool>,
    }

    impl CaseEvalScratch {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for CaseEvalScratch {
        fn default() -> Self {
            Self {
                sc: Scratch::new(),
                req: katgpt_core::decision_wire::DecisionRequest {
                    state: String::new(),
                    questions: Vec::new(),
                },
                probs: Vec::new(),
                picks: Vec::new(),
                confs: Vec::new(),
                abstained: Vec::new(),
            }
        }
    }

    /// The scratch refill face of the eval path (reflex issue 070, lead
    /// 2): evaluates ONE case into the frame — the same request
    /// construction, noul flip, forced pick and confidence reads as
    /// [`eval_seat`] — minus every per-call allocation the frame can
    /// reuse. The determinism probe and the latency tail stay the arena
    /// path's job (the caller times the request itself).
    ///
    /// Byte-parity contract: the answers equal [`eval_seat`]'s over the
    /// one-case slice — the frozen-picks replays (instinct serve gates,
    /// rethink esc gates) assert it.
    pub fn eval_case_into<const N: usize>(
        engine: &mut DecisionEngine<N, EMBED_DIM>,
        case: &SuiteCase,
        state_str: &str,
        frame: &mut CaseEvalScratch,
    ) -> Result<(), String> {
        super::engine_request_into(case, state_str, &mut frame.req)?;
        frame.sc.prepare(case.questions.len().max(1));
        frame.picks.clear();
        frame.confs.clear();
        frame.abstained.clear();
        for slot in &mut frame.probs {
            slot.clear();
        }

        let resp = engine
            .decide_with(&frame.req, &mut frame.sc)
            .map_err(|e| format!("engine decide ({}): {e}", case.id))?;

        for (qi, (q, ans)) in case.questions.iter().zip(resp.answers.iter()).enumerate() {
            if frame.probs.len() <= qi {
                frame.probs.push(Vec::new());
            }
            let slot = &mut frame.probs[qi];
            let pick = super::answer_probs_pick(q.kind, ans, slot);
            frame.picks.push(pick);
            frame.confs.push(f64::from(ans.confidence));
            frame.abstained.push(ans.outcome.is_none());
        }
        // A longer PREVIOUS request leaves stale tail slots (the request
        // shape changed) — drop them so the next reader never sees them.
        frame.probs.truncate(frame.abstained.len());

        // The wire's `outcome.is_none()` and the engine's cause must agree
        // by construction (issue 060) — asserted against the frame's own
        // solve scratch, no Vec<AbstainCause> copy.
        debug_assert_eq!(frame.abstained.len(), frame.sc.slots.len());
        for (w, c) in frame.abstained.iter().zip(frame.sc.slots.iter()) {
            debug_assert_eq!(*w, c.cause.abstained());
        }
        Ok(())
    }

    /// The laya escalation face of G2 (cfg `laya-riir`): per-question
    /// wall latency of the pinned english checkpoint over the given
    /// cases (one discarded warmup forward first — the pre-ramp law),
    /// µs per case. LATENCY ONLY — the accuracy face of laya is the
    /// published harness row, never re-derived here.
    #[cfg(feature = "laya-riir")]
    pub fn laya_escalation_latency_us(cases: &[SuiteCase]) -> Result<Vec<u64>, String> {
        use crate::laya::config::Checkpoint;
        let agent = super::load_laya_agent("english", Checkpoint::English)?;
        if let Some(case) = cases.first() {
            agent
                .system_one(&case.state, &super::case_questions(case))
                .map_err(|e| format!("laya warmup: {e}"))?;
        }
        let mut durs = Vec::with_capacity(cases.len());
        for case in cases {
            let t0 = std::time::Instant::now();
            agent
                .system_one(&case.state, &super::case_questions(case))
                .map_err(|e| format!("laya forward ({}): {e}", case.id))?;
            durs.push(t0.elapsed().as_micros() as u64);
        }
        Ok(durs)
    }
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
    /// Issue 038 T4′: also compose the cascade lane — the modelless
    /// answers stand, the calibrated fused gate's abstains escalate to
    /// each served riir-laya checkpoint. Publishes accuracy AND the
    /// escalation rate per suite (the rate is the latency claim). Needs
    /// the laya lane (mutually exclusive with `skip_laya`) and the
    /// `laya-riir` feature; needs a modelless lane (LLM-only families
    /// get an honest absence).
    pub cascade: bool,
    /// Issue 042 lever 3: gate the cascade's escalation per suite by a
    /// cal-slice worthiness probe — the escalator answers the cal questions
    /// the calibrated gate abstained on, and the suite's escalation stays
    /// armed only where it reads ≥ the forced modelless picks on that probe
    /// set (delta ≥ the margin). A negative probe disarms the suite (every
    /// abstain stands as the modelless forced pick, disclosed in the row);
    /// missing cal records / thin support stay armed with a named reason.
    /// Needs `cascade`. Default off = the landed T4′ lane, byte-identical.
    pub cascade_worthiness: bool,
    /// The worthiness arm bar: the probe disarms when
    /// `(escalator − modelless accuracy on the cal probe set) <` this
    /// value. Default 0.16 (issue 042 T3 (a), promoted per the issue's
    /// own trigger — a second independent lane run reproduced 10/10 at
    /// 0.16 byte-identically; the bar sits in the measured (0.150, 0.288]
    /// gap that separated every probe family, Benches 063 + 066). 0.0
    /// restores arm-at-parity.
    pub cascade_worthiness_margin: f64,
    /// Issue 046 lever 4 (`--cascade-worthiness-lcb <F>`): the
    /// support-aware arm leg — when set, a suite also arms where the
    /// cal-slice probe delta's one-sided-95% lower confidence bound ≥
    /// the floor (the Bench-063 fused family: ag_news LCB +0.0854 at
    /// n 190 arms; massive +0.0064 at n 60 and sst5 −0.0059 do not —
    /// support separates what the point-estimate margin provably
    /// cannot). Additive to `cascade_worthiness_margin`; `None` = off,
    /// byte-identical with the lever-3 lane.
    pub cascade_worthiness_lcb: Option<f64>,
    /// Issue 042 lever 1 (`--gate-fit-selection`): fit the fused-gate
    /// thresholds on the STRATIFIED selection slice (the shared held-out
    /// instrument; the probe corpus excludes the fit docs) instead of the
    /// train-tail cal slice. Dataset suites only (the synthetic/code paths
    /// keep the cal-slice fit). Default off = the shipped T1.6 posture,
    /// byte-identical.
    pub gate_fit_selection: bool,
    /// Issue 042 lever 2 (`--gate-distance-only`): disable the fused
    /// gate's score axis (threshold 0.0) — abstain/escalation runs on the
    /// corpus-distance axis alone, at its fitted ρ=30% threshold. The
    /// transfer-stable-axis lever (Bench 061: the score axis read 90–99%
    /// test abstain at the armed postures while the distance axis held
    /// ~31–32% in every posture). Default off = the shipped fused gate.
    pub gate_distance_only: bool,
    /// Issue 056 direction 0 — PROMOTED default-on 2026-09-30 (verdict
    /// round 2, the units-bug adjudication): fit the score-axis threshold on
    /// the CALIBRATED scale the deployed gate applies — closing the
    /// fit-on-raw / apply-on-calibrated mismatch that saturation turned into
    /// 90–99% abstention or a disarmed score axis. At a monotone fit the
    /// percentile-coherent threshold reproduces the raw gate's fitted target
    /// exactly (Bench 095). `--no-gate-fit-calibrated` restores the old
    /// posture.
    pub gate_fit_calibrated: bool,
    /// Issue 079: the legacy `k == N` index-alignment route binding.
    /// Default on = the incumbent shipped behavior, byte-identical;
    /// `--no-kn-route` runs the content-bound posture (route terms arm
    /// only via by-name resolution) so both sides of the bench-128
    /// position-binding finding are measurable in one binary.
    pub kn_route: bool,
    /// Issue 079: the drafter-only correction mode (see
    /// [`ModellessInput::drafter_fix`]). Default `Off`.
    pub drafter_fix: crate::engine::DrafterFix,
    /// Also run the laya-PYTHON lane — the ORIGINAL torch reference as a
    /// subprocess oracle (measurement-only; opt-in, off by default).
    pub laya_python: bool,
    /// Also run the GLiNER comparison lane — fastino/GLiNER2.5-Decide over
    /// their gliner2 package as a JSONL subprocess oracle (Issue 029;
    /// measurement-only, off by default).
    pub gliner: bool,
    /// Also run the Bekko comparison lane — hotchpotch/bekko-system-one-v0
    /// over their BekkoSentenceTransformer runtime as a JSONL subprocess
    /// oracle (Bench 103; measurement-only, off by default; MIT — verified
    /// 2026-10-02).
    pub bekko: bool,
    /// Also run the AgentJev comparison lane (Issue 025 amendment 4 /
    /// `.issues/027`): their `jev_service` answered over HTTP
    /// (`AGENTJEV_SERVE_URL`, default `http://127.0.0.1:8149`) — their
    /// stack serves, our Rust measures (comparison lane, never a product
    /// lane). An unreachable server is a LOUD error, never a silent skip.
    /// Default off.
    pub agentjev: bool,
    /// Also run the Cloudflare Clef comparison lane (plan 011 Phase A):
    /// their Workers-AI-hosted decision models behind an operator-run
    /// loopback TLS forwarder (`CLEF_SERVE_URL`, default
    /// `http://127.0.0.1:8791`) — the agentjev shape; missing creds
    /// (`CLEF_ACCOUNT_ID`/`CLEF_RUN_PATH`) refuse at lane construction,
    /// and the spend ceiling `CLEF_SMOKE_MAX_CASES` (default 50) bounds
    /// undisclosed pricing. Default off.
    pub clef: bool,
    /// Also run the clef lane in its LOCAL in-process ANE arm (plan 619
    /// T3.1): clef-text-0.6b on the Neural Engine through the
    /// `riir-infer-clef` substrate (feature `clef-ane`, macOS). The
    /// `--clef-local` flag sets this AND implies the lane runs (the arm
    /// is selected by `CLEF_LOCAL=1` inside `ClefArm::from_env`, so the
    /// runner loop stays shared). Default off.
    pub clef_local: bool,
    /// Also run the OpenThai comparison lane (Plan 003 Phase 2): their
    /// `openthai_systemone` service answered over HTTP
    /// (`OPENTHAI_SERVE_URL`, default `http://127.0.0.1:8000`) — the
    /// agentjev shape, the first Thai+English specialist in the family.
    /// An unreachable server is a LOUD error, never a silent skip.
    /// Default off.
    pub openthai: bool,
    /// Also run the Drex DLM comparison lane (Issue 073, from
    /// `.research/008`): Nace.AI's open-weights 8B diffusion-LM decision
    /// model over the TypeSafe `/v1/systemone` wire at `DREX_SERVE_URL`.
    /// CC BY-NC weights: measurement only, never a product lane, never a
    /// distill teacher. An unreachable server is a LOUD error, never a
    /// silent skip. Default off.
    pub drex: bool,
    /// Also run the LiquidAI d1 comparison lane (Issue 078, from
    /// `.research/009`): d1-3B over their official
    /// `/decisions/v1/systemone` wire at `D1_SERVE_URL` (our reference
    /// stdlib server over their in-repo `D1Model`). License
    /// `other`/lfm1.0: measurement only, never a product lane. An
    /// unreachable server is a LOUD error, never a silent skip. Default
    /// off.
    pub d1: bool,
    /// Also run the pplx-decider comparison lane (Issue 082): Perplexity's
    /// `pplx-decider-v1.1-27b` (Apache-2.0, UNGATED — the JDI 2026-10-07
    /// #1; teacher-eligible unlike drex/d1) served through their autojev
    /// server over the TypeSafe `/v1/systemone` wire at `PPLX_SERVE_URL`
    /// (default `http://127.0.0.1:8793`; serve their side with
    /// `PORT=8793`). Local-serve posture only (the hosted posture is
    /// owner-gated and never pooled — the clef law); the read decides
    /// teacher candidacy (riir-train 623). An unreachable server is a
    /// LOUD error, never a silent skip. Default off.
    pub pplx: bool,
    /// Also run the vLLM-SR d3 comparison lane (riir-train 629 T2): the
    /// JDI board's `vLLM-SR d3 (27B)` (Apache-2.0, UNGATED —
    /// teacher-eligible; tag v3.1.0, Qwen3.8-27B backbone + 256-code
    /// readout, the pplx head class) served through their `d3_server.py`
    /// on the same `/v1/systemone` wire at `D3_SERVE_URL` (default
    /// `http://127.0.0.1:8794`; serve their side with `--port 8794`).
    /// The JDI-panel scoring read: the per-family spot check vs the
    /// board's published numbers gates the teacher captures (the
    /// lossy-surface law — an unreachable server is a LOUD error, never
    /// a silent skip). Default off.
    pub d3: bool,
    /// Also run the Jeva comparison lane (Issue 090 B1, feeding
    /// riir-train 632): `LIQIIIII/Jeva-4B` (arXiv:2610.18773; weights
    /// license UNDECLARED — measurement + teacher-capture posture,
    /// adoption fail-closed per 632 T0) served by OUR reference server
    /// `scripts/jeva_server.py` at `JEVA_SERVE_URL` (default
    /// `http://127.0.0.1:8095`). An unreachable server is a LOUD error,
    /// never a silent skip. Default off.
    pub jeva: bool,
    /// Also run the StartLux-Decision comparison lane (Issue 095, feeding
    /// riir-train 635 T3): board #1 (`startlux-models/StartLux-Decision-
    /// v1.1-27B-Instant`; weights CC BY-NC 4.0 → DECLARED-NC RESTRICTED —
    /// capture-for-measurement + teacher-capture, adoption fail-closed per
    /// 635 T0) served by THEIR Apache-2.0 engine `startlux-decide` over
    /// llama.cpp on the same `/v1/systemone` wire as pplx/d3 at
    /// `STARTLUX_SERVE_URL` (default `http://127.0.0.1:8090`). The health
    /// handshake GATES the bidirectional Instant posture (a causal/off
    /// serve misreads the checkpoint — refusal names the boot line). An
    /// unreachable server is a LOUD error, never a silent skip. Default
    /// off.
    pub startlux: bool,
    /// Also run the mammoth/elephant hosted comparison lane (rethink Issue
    /// 032 T10 / Research 012): Base Labs' mammoth-1 (the JDI 0.4.1
    /// board-#1, MIT) or elephant-1 on Baseten Model APIs — reached ONLY
    /// through the operator-run loopback TLS forwarder (the clef A0 law) at
    /// `MAMMOTH_SERVE_URL` (default `http://127.0.0.1:8793`) with
    /// `MAMMOTH_API_TOKEN` set (or `MAMMOTH_FORWARDER_AUTH=1`). The
    /// handshake is ONE minimal noul probe; the spend ceiling defaults to
    /// 50 cases (free week ≠ unmetered); MEASUREMENT-ONLY — no RIDT arm (the
    /// T10.2 law: API outputs never enter a capture artifact). An
    /// unreachable/unused-cred lane is a LOUD error, never a silent skip.
    /// Default off.
    pub mammoth: bool,
    /// Also run the elephant-1 SELF-HOSTED comparison lane (rethink Issue
    /// 032 T10.2 / Research 012 — the LOCAL-FIRST law): Base Labs'
    /// elephant-1 (9B, JDI 0.4.1 board #4 public print 72.24, MIT) served
    /// by OUR reference `scripts/elephant_server.py` over their qev runtime
    /// at `ELEPHANT_SERVE_URL` (default `http://127.0.0.1:8096`). The
    /// spend ceiling defaults to 50 cases (`ELEPHANT_ALLOW_UNCAPPED=1`
    /// records the local zero-marginal-cost posture — the panel and the
    /// capture passes ride it). An unreachable server is a LOUD error,
    /// never a silent skip. Default off.
    pub elephant: bool,
    /// Also run the OpenDecider-nano self-hosted comparison lane (rethink
    /// Issue 038 / Research 013 — the third encoder-candidate's P0
    /// admission rig): their `opendecider serve` (pip
    /// `opendecider[serve]`, Jev-compatible `/v1/systemone`) at
    /// `OPENDECIDER_SERVE_URL` (default `http://127.0.0.1:8000`), 0.8 GB
    /// weights, CPU-runnable. The handshake is ONE minimal noul probe; the
    /// spend ceiling defaults to 50 cases (`OPENDECIDER_ALLOW_UNCAPPED=1`
    /// records the local zero-marginal-cost posture — the P0's 400-case
    /// read rides it). The wire-smoke runs on TRAIN rows only (the 038
    /// gate). Default off.
    pub opendecider: bool,
    /// Also run the PAW comparison lane (Issue 033): ProgramAsWeights
    /// compiled per specced suite, answered over their hosted REST via a
    /// `curl` subprocess (`src/lanes/paw.rs`). Default off.
    pub paw: bool,
    /// Also run the PAW Posture B lane (Issue 033): the SAME compiled
    /// programs answered through their LOCAL llama.cpp runtime as a Python
    /// subprocess oracle (`src/lanes/paw_local.rs`, the gliner-lane
    /// shape). Never compiles — reads the hosted lane's cache. Default
    /// off.
    pub paw_local: bool,
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
    /// Issue 044 T3: run the modelless NLI pair-feature head A/B
    /// (`--nli-feature-ab`): closed-form diagonal-LDA over lexical
    /// premise/hypothesis pair features, fitted on the CAL slice, one test
    /// read under head-alone + two blend postures + the G1-constrained
    /// blend posture (Bench 069 protocol: λ selected cal-side under the
    /// pre-registered calibration constraint, both G1 triples at the
    /// read). Report-only
    /// (a result-row record, never a gate); premise/hypothesis-shaped
    /// suites only — a loud skip elsewhere. Default off.
    pub nli_feature_ab: bool,
    /// Issue 047 M1 (`--nli-m1`): run the xnli VALIDATION-slice reopen
    /// lane — pick/confidence separation with an agreement gate, the
    /// pre-registered posture of `.plans/006_nli_m1_reopen.md` (R1–R6
    /// binding; the pass reads `xnli_en_val` ONLY and loud-skips
    /// everything else, so the spent test split is structurally
    /// unreachable). Default off.
    pub nli_m1: bool,
    /// Issue 055 / Plan 008 T5 (`--mc-ab`; needs the `mc_ensemble` feature —
    /// without it the flag refuses as unknown, loud): run the seeded-MC
    /// distributional-layer A/B on every modelless suite — rejection
    /// ranking (u_pair vs the fused-gate confidence at matched coverage) +
    /// the histogram's decision rules (majority / mean / LCB-λ), knobs
    /// selected cal-side. Report-only; the test split is read once. Default
    /// off.
    #[cfg(feature = "mc_ensemble")]
    pub mc_ab: bool,
    /// `--mc-samples N` — total MC samples per case INCLUDING the
    /// unperturbed legacy run (default 8 = the G2 gate's N; 0 → default).
    #[cfg(feature = "mc_ensemble")]
    pub mc_samples: usize,
    /// `--mc-p-drop P` — pin the dropout probability, skipping the cal-side
    /// sweep (the selection table then carries the pinned row only). Must
    /// lie in (0, 0.5].
    #[cfg(feature = "mc_ensemble")]
    pub mc_p_drop: Option<f32>,
    /// `--mc-lambda L` — pin the LCB λ, skipping the cal-side λ sweep. ≥ 0.
    #[cfg(feature = "mc_ensemble")]
    pub mc_lambda: Option<f32>,
    /// Plan 621 Phase 5 T5.1 (`--probe-delta-ab`; needs the
    /// `probe_delta_ab` feature — without it the flag refuses loud naming
    /// the rebuild): arm the paired counterfactual ΔV certification on the
    /// `--corpus-ab` lane — katgpt-core `state_probe` over the discordant
    /// pairs (the sign test at ε = 0.5 + the adaptive-N disclosure).
    /// Report-only; the V5 paired-LB95 gate law is unchanged. Default off.
    #[cfg(feature = "probe_delta_ab")]
    pub probe_delta_ab: bool,
    /// Plan 627 / Issue 090 C2 (`--set-refine-ab`; needs the `set_refine`
    /// feature — without it the flag refuses loud naming the rebuild):
    /// run the set-conditioned state refinement's paired A/B on every
    /// modelless suite — the refined engine vs the shipped posture, one
    /// frozen read, flip decomposition + Δ accuracy with a paired LCB95.
    /// Report-only. Default off.
    #[cfg(feature = "set_refine")]
    pub set_refine_ab: bool,
    /// `--set-refine-gamma F` — pin the refinement gate scale (0/negative
    /// → the 1.0 default; the kernel-side working envelope measured at
    /// katgpt-rs Plan 627 T1 is γ 0.8–1.5).
    #[cfg(feature = "set_refine")]
    pub set_refine_gamma: f32,
    /// `--set-refine-tau F` — pin the refinement gate temperature (0/
    /// negative → the 0.5 default; envelope τ 0.4–0.7).
    #[cfg(feature = "set_refine")]
    pub set_refine_tau: f32,
    /// Fitted per-label head blend scale for the modelless lane (issue 030
    /// lever 4; `--head-scale`). 0.0 = OFF — the byte-identical pre-head
    /// posture and the published baseline. MEASUREMENT-ONLY knob: a run at
    /// a non-zero scale is a different engine posture and MUST be
    /// disclosed (`RunMeta::head_posture`) — never silently mixed into a
    /// published table. Mutually exclusive with [`RunOptions::head_select`]
    /// (one pins the scale, the other selects it).
    pub head_scale: f32,
    /// Cal-slice head-scale selection (issue 030 lever 4; `--head-select`):
    /// per eligible dataset suite, accuracy per ladder candidate (0 / 0.5 /
    /// 1 / 2) on the CAL slice (forced, ties → 0 = off), argmax picked
    /// there ONLY, test read once at the selected posture — the same
    /// protocol the cap selection and the threshold fit follow. Per-row
    /// candidates + selection ride the result. Mutually exclusive with a
    /// non-zero [`RunOptions::head_scale`].
    pub head_select: bool,
    /// Cal-slice count-table selection (issue 038 T1; `--nb-select`): per
    /// eligible dataset suite, (scale × α) candidates scored on the
    /// stratified selection slice under the heads' promotion bar, test read
    /// once; also emits the SEPARATE transductive column wherever the
    /// tables end up armed. Needs the `nb_scope` feature (a loud error
    /// without it, never a silent no-op).
    pub nb_select: bool,
    /// Cal-slice option-conditioned selection (issue 038 T7b;
    /// `--oc-select`): per eligible dataset suite, the (qid, option) table
    /// scale selected on the stratified selection slice under the heads'
    /// promotion bar, test read once. Needs the `option_cond` feature (a
    /// loud error without it, never a silent no-op). Default off.
    pub oc_select: bool,
    /// Cal-slice ridge-readout selection (issue 038 T7a; `--ridge-select`):
    /// the NBSVM closed-form ridge readout scale, selected on the same
    /// slice under the same promotion bar; λ fixed at the probe's 10.0
    /// (a λ ladder multiplies the O(k³)-per-class fit cost). Needs the
    /// `nb_ridge` feature (a loud error without it). Default off.
    pub ridge_select: bool,
    /// Cal-slice JOINT blend-genome selection (issue 038 T5;
    /// `--genome-select`): seeds from the composed greedy posture (cap →
    /// head → nb → oc → ridge) and refines the scoring-time scale
    /// coordinates `{route, head, nb(+α,+view), oc, ridge}` TOGETHER on
    /// the stratified selection slice — coordinate descent, moves gated
    /// by a noise-floor margin, the walk end gated by the same 5 pt
    /// promotion bar as the per-lever lanes; test read once at the walk
    /// end. Needs the `nb_scope` feature (a loud error without it).
    /// Default off.
    pub genome_select: bool,
    /// The genome walk's acceptance-vs-seed bar (issue 038 T5;
    /// `--genome-accept-margin`). `<= 0` resolves to the house arming bar
    /// (a 0 margin would accept any +1-question wiggle — never a default).
    /// Default 0.05.
    pub genome_accept_margin: f64,
    /// Issue 066: run the fused-abstain DENSITY-half A/B (report-only;
    /// `--density-gate`). Needs the `density_gate` feature — an explicit
    /// flag without it is a LOUD error, never a silent skip. Default off
    /// = the shipped fused gate, byte-identical.
    pub density_gate: bool,
    /// Also run the CLM comparison lane (Issue 019 T3 / `.issues/027`):
    /// the external Contrastive-LM reference answered over HTTP
    /// (`clm-serve` at `CLM_SERVE_URL`, default `http://127.0.0.1:8700`)
    /// — their stack serves, our Rust measures (comparison lane, never a
    /// product lane). Needs the `clm-lane` feature; an explicit flag
    /// without it, or an unreachable server, is a LOUD error, never a
    /// silent skip. Default off.
    pub clm: bool,
    /// Issue 072 (`--dump-items`; rethink 028 T1's instrument): persist the
    /// per-item outcome records every lane already materialized in memory
    /// to `<out>/items/<suite>.jsonl` (one JSONL line per answered
    /// (one JSONL line per answered
    /// question; join key `(case_id, q_idx)`). Measurement-only: no lane
    /// behavior changes, `results.json` stays byte-identical. Default off
    /// = no file written (the records are still built — the same walk the
    /// metrics read — and dropped).
    pub dump_items: bool,
}

impl RunOptions {
    /// The no-flags CLI posture: every optional comparison lane, probe,
    /// and selection lever OFF; the promoted engine defaults ON
    /// (`gate_fit_calibrated`, `kn_route`); measurement margins at their
    /// house values. THE single struct-literal home for `RunOptions` —
    /// `harness_main` resolves its no-flags base through this constructor,
    /// and out-of-repo consumers (instinct's A0 drift pin) build on it
    /// instead of enumerating fields, so a lane field added upstream is
    /// off-by-construction downstream and never an E0063 break there
    /// (instinct issue 023, the pplx → d3 → jeva lineage). Posture deltas
    /// are the caller's explicit overrides on the returned value.
    pub fn lanes_off(datasets_dir: PathBuf, suites: Vec<String>) -> Self {
        Self {
            datasets_dir,
            suites,
            laya_max_questions: 0,
            skip_laya: false,
            dump_items: false,
            cascade: false,
            cascade_worthiness: false,
            // Issue 042 T3 (a), promoted per the issue's own trigger (a second
            // independent lane run reproduced 10/10 at 0.16 — byte-identical
            // probe deltas, recorded in the 042 HISTORY row): the measured
            // arm bar sits in the (0.150, 0.288] gap that separated every
            // probe family measured (Bench 063 fused, Bench 066 distance +
            // combined). An explicit --cascade-worthiness-margin overrides.
            cascade_worthiness_margin: 0.16,
            // Issue 046 lever 4: the support-aware arm leg — off by default
            // (byte-identical with the lever-3 lane).
            cascade_worthiness_lcb: None,
            gate_fit_selection: false,
            gate_distance_only: false,
            // Issue 056 direction 0 — PROMOTED default-on 2026-09-30 (verdict
            // round 2, the units-bug adjudication): the gate-fit probe observes
            // on the SAME scale the deployed gate applies. At a monotone fit the
            // percentile-coherent threshold reproduces the raw gate's fitted
            // target exactly, so this can never lose to the pre-calibration
            // baseline. `--no-gate-fit-calibrated` restores the old
            // fit-on-raw/apply-on-calibrated posture for measurement.
            gate_fit_calibrated: true,
            // Issue 079 — the legacy `k == N` index-alignment route binding is
            // the incumbent default; `--no-kn-route` runs the content-bound
            // posture (route terms arm only via by-name resolution) so both
            // sides of the bench-128 position-binding finding are measurable
            // in one binary.
            kn_route: true,
            // Issue 079 — the drafter-only corrections are OFF by default (the
            // shipped scores, byte-identical); `--drafter-fix <mode>` arms one
            // for the content-bound posture's tuning axis.
            drafter_fix: crate::engine::DrafterFix::Off,
            laya_python: false,
            clm: false,
            gliner: false,
            bekko: false,
            agentjev: false,
            clef: false,
            openthai: false,
            drex: false,
            d1: false,
            pplx: false,
            d3: false,
            jeva: false,
            startlux: false,
            mammoth: false,
            elephant: false,
            opendecider: false,
            paw: false,
            paw_local: false,
            clef_local: false,
            corpus_cap_override: 0,
            cal_select_caps: Vec::new(),
            pair_head_ab: false,
            nli_feature_ab: false,
            nli_m1: false,
            #[cfg(feature = "mc_ensemble")]
            mc_ab: false,
            #[cfg(feature = "mc_ensemble")]
            mc_samples: 0,
            #[cfg(feature = "mc_ensemble")]
            mc_p_drop: None,
            #[cfg(feature = "mc_ensemble")]
            mc_lambda: None,
            head_scale: 0.0,
            head_select: false,
            nb_select: false,
            oc_select: false,
            ridge_select: false,
            genome_select: false,
            genome_accept_margin: 0.05,
            density_gate: false,
            #[cfg(feature = "probe_delta_ab")]
            probe_delta_ab: false,
            #[cfg(feature = "set_refine")]
            set_refine_ab: false,
            #[cfg(feature = "set_refine")]
            set_refine_gamma: 0.0,
            #[cfg(feature = "set_refine")]
            set_refine_tau: 0.0,
        }
    }
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

/// Bench 082 — the runner-side host-label REFUSAL. A row whose host
/// would be `unknown` is a phantom-host row: the bench-site merge keys
/// rows by host, and `publish_bench.py` refuses `unknown` at load (the
/// relabel incident — reflex-site `74b49e4`), so the row could never
/// publish — the failure belongs HERE, at row birth, naming the exact
/// remedy. The 4090 measured case: `uname -n` does not resolve in the
/// probe there, so an unset label used to land as `unknown` and needed
/// a hand relabel. Only the harness META path calls this — the pure
/// `host_label` law (precedence + the blank-guard) is unchanged and
/// stays unit-pinned.
fn hostname_refusing_unknown() -> String {
    let label = hostname();
    if label == PHANTOM_HOST_SENTINEL {
        eprintln!(
            "REFUSING to mint a bench row with host '{}' — set \
             REFLEX_BENCH_HOST=<descriptive-label> (e.g. 4090-windows) and re-run; \
             the bench-site merge keys rows by host and refuses '{}' at publish",
            PHANTOM_HOST_SENTINEL, PHANTOM_HOST_SENTINEL
        );
        std::process::exit(2);
    }
    label
}

/// Bench 082 — the degenerate host label. One home, keyed on by BOTH
/// the runner-side refusal below and the reflex-site publisher's load
/// gate; the sentinel test pins the two layers together.
pub(crate) const PHANTOM_HOST_SENTINEL: &str = "unknown";

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
            None => PHANTOM_HOST_SENTINEL.to_string(),
        },
    }
}

#[cfg(test)]
mod host_label_tests {
    //! Issue 018 T5 — the host-label override law. Pure over its inputs so
    //! the precedence (override > uname > "unknown") and the blank-guard
    //! are pinned without touching process env. Bench 082 adds the
    //! runner refusal one layer up (`hostname_refusing_unknown`): the
    //! meta paths exit 2 on the `"unknown"` fallthrough instead of
    //! minting a phantom-host row — this module pins the string that
    //! refusal keys on so the two layers cannot drift apart silently.

    use super::host_label;

    #[test]
    fn override_wins_over_uname() {
        assert_eq!(
            host_label(Some("4090-windows"), Some("shikuwa")),
            "4090-windows"
        );
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

    /// Bench 082 — `"unknown"` is the exact sentinel the meta-path
    /// refusal (and the reflex-site publisher) keys on. A rename here
    /// without the matching edits would silently re-open the
    /// phantom-host class; this arm fails loud first.
    #[test]
    fn unknown_is_the_phantom_host_sentinel_both_layers_key_on() {
        const SENTINEL: &str = "unknown";
        assert_eq!(host_label(None, None), SENTINEL);
        assert_eq!(super::PHANTOM_HOST_SENTINEL, SENTINEL);
    }
}

#[cfg(test)]
mod issue88_pins {
    //! Issue 088 adjudication (2026-10-14): the mmlupro_jdi eval sample
    //! IS the documented hash-order law — executed under the HOUSE fold
    //! (multiplier 2³⁶+435, a transcribed FNV-1a prime; see
    //! `jdi_row_identity_hash`). These pins hold the end-to-end runtime
    //! chain (load_rows → stratified_split → build → cases_digest) against
    //! the recorded population identity, so any drift in the chain or a
    //! future prime "fix" fails loud instead of silently re-keying every
    //! recorded JDI number.

    /// The served gold sequence the 4090 lane observed (Bench 143) and this
    /// chain reproduces byte-identically on the canonical pool.
    const SERVED_FIRST_12_GOLDS: [usize; 12] = [7, 9, 5, 6, 2, 2, 1, 7, 0, 0, 0, 6];

    #[test]
    fn mmlupro_runtime_chain_serves_the_recorded_sample() {
        let dir = std::path::Path::new(".raw/datasets");
        if !dir.join("mmlupro_jdi").exists() {
            eprintln!("SKIP: no .raw/datasets pool beside the repo (fetch first)");
            return;
        }
        let eval_rows =
            super::load_rows(&dir.join("mmlupro_jdi"), "test").expect("load mmlupro test split");
        let n = eval_rows
            .get("rows")
            .and_then(|r| r.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        assert_eq!(n, 12_032, "the canonical mmlupro_jdi test population");
        let split = crate::harness::suites::stratified_split(&eval_rows, "mmlupro_jdi", 2000);
        assert_eq!(split.n_picks, 2000);
        let suite = super::build_mmlupro_jdi(&split.front, 0);
        assert_eq!(suite.cases.len(), 2000);
        let golds: Vec<usize> = suite.cases.iter().take(12).map(|c| c.gold[0].idx).collect();
        assert_eq!(golds, SERVED_FIRST_12_GOLDS, "the served gold sequence");
        // The manifest §15 population-identity pin — now known to be the
        // HOUSE fold, not canonical fnv1a64 (Issue 088).
        assert_eq!(
            super::cases_digest("mmlupro_jdi", &suite.cases),
            "fnv1a64-6a2d24628ce7b916"
        );
    }
}

#[cfg(test)]
mod drex_conf_readout_tests {
    //! Issue 076 T2 — the Drex lane's per-kind confidence cells + the
    //! split-half conformal floor companion, known-answer-tested: the
    //! kind split (noul never a cell), the occupancy floor, and the
    //! split halves (first calibrates, second is audited).

    use super::*;
    use crate::harness::suites::{GoldAnswer, SuiteCase, SuiteQuestion};

    /// One case of `kinds.len()` questions; gold idx 0 everywhere; the
    /// caller supplies the wire confidences per question (None for noul).
    fn case_of(kinds: &[QKind]) -> SuiteCase {
        SuiteCase {
            id: "c0".into(),
            state: serde_json::json!("s"),
            questions: kinds
                .iter()
                .enumerate()
                .map(|(i, k)| SuiteQuestion {
                    qid: format!("q{i}"),
                    kind: *k,
                    instructions: "i".into(),
                    criteria: Value::Null,
                })
                .collect(),
            gold: kinds
                .iter()
                .map(|_| GoldAnswer {
                    idx: 0,
                    soft: vec![],
                    gold_score: None,
                })
                .collect(),
        }
    }

    fn picks_all_zero(n: usize) -> Vec<usize> {
        vec![0; n]
    }

    #[test]
    fn cells_split_by_kind_and_noul_is_never_a_cell() {
        let kinds = [QKind::Choice, QKind::Noul, QKind::Score, QKind::Choice];
        let case = case_of(&kinds);
        let picks = vec![picks_all_zero(4)];
        let confs = vec![vec![Some(0.9), None, Some(0.4), Some(0.9)]];
        let r = assemble_drex_conf_readout(std::slice::from_ref(&case), &picks, &confs).unwrap();
        let choice = r.choice.expect("choice cell exists");
        assert_eq!(choice.n, 2);
        let score = r.score.expect("score cell exists");
        assert_eq!(score.n, 1);
        // The disclosure law: noul is a named null, never a cell.
        assert!(r.noul.is_none());
        // Both choice picks are gold idx 0 → correct → Brier over the
        // choice cell = (0.9−1)² twice, mean 0.01; the score cell reads
        // (0.4−1)² = 0.36.
        assert!((choice.brier - 0.01).abs() < 1e-12);
        assert!((score.brier - 0.36).abs() < 1e-12);
        // Floor: 3 pairs < the 40-pair occupancy floor → withheld, and the
        // support fields say zero rather than pretending a split happened.
        assert!(r.floor_ece.is_none());
        assert_eq!((r.floor_n_cal, r.floor_n_test), (0, 0));
    }

    #[test]
    fn readout_is_none_without_any_wire_confidence() {
        let kinds = [QKind::Noul, QKind::Noul];
        let case = case_of(&kinds);
        let picks = vec![picks_all_zero(2)];
        let confs = vec![vec![None, None]];
        assert!(assemble_drex_conf_readout(std::slice::from_ref(&case), &picks, &confs).is_none());
    }

    #[test]
    fn floor_below_the_occupancy_window_is_none_even_at_the_edge() {
        // 39 pairs: one under the floor → withheld.
        let kinds = [QKind::Choice; 39];
        let case = case_of(&kinds);
        let picks = vec![picks_all_zero(39)];
        let confs = vec![vec![Some(0.5); 39]];
        let r = assemble_drex_conf_readout(std::slice::from_ref(&case), &picks, &confs).unwrap();
        assert!(r.floor_ece.is_none());
    }

    #[test]
    fn floor_is_the_split_half_conformal_read_first_cal_second_test() {
        // Known answer: the cal half's confidences are ALL 0.0, so the
        // conformal CDF maps every test confidence to (1 + 20)/(20 + 1) =
        // 1.0; the test half is half-correct, so the floored ECE over the
        // conf=1.0 bin is |1.0 − 0.5| = 0.5 exactly. The engineered split
        // also pins WHICH half is which: a swapped split would read cal
        // confs 1.0 and floor the 0.0 half to 1/21 — never 0.5.
        const N: usize = 40;
        let kinds = [QKind::Choice; N];
        let case = case_of(&kinds);
        let mut confs_one_case = vec![Some(0.0); N / 2];
        confs_one_case.resize(N, Some(1.0));
        let confs = vec![confs_one_case];
        // Test-half correctness alternates → exactly half correct.
        let mut picks_one_case = vec![0usize; N];
        for (i, p) in picks_one_case.iter_mut().enumerate().skip(N / 2) {
            *p = usize::from(i % 2 == 0); // gold is idx 0; half the test picks miss
        }
        let picks = vec![picks_one_case];
        let r = assemble_drex_conf_readout(std::slice::from_ref(&case), &picks, &confs).unwrap();
        let floor = r.floor_ece.expect("40 pairs clear the occupancy floor");
        assert!((floor - 0.5).abs() < 1e-12, "floor {floor}");
        assert_eq!(r.floor_n_cal, N / 2);
        assert_eq!(r.floor_n_test, N - N / 2);
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
            gold: vec![
                crate::harness::suites::GoldAnswer {
                    idx: 0,
                    soft: vec![],
                    gold_score: None,
                };
                n
            ],
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
        assert_eq!(
            req.questions[0].options,
            vec!["negative", "neutral", "positive"]
        );
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

#[cfg(test)]
mod corpus_digest_tests {
    //! Issue 057 — the corpus-identity digest: the consumed-corpus law
    //! (capped drafter docs + fallback + optional nb sets), determinism,
    //! and sensitivity to every axis the corpus-cap echo cannot see (the
    //! typed 800→1200 lift class). Pure over its inputs — no dataset dir.

    use super::{TrainDoc, corpus_digest, per_label_corpus_docs};

    fn doc(label: &str, text: &str) -> TrainDoc {
        TrainDoc {
            label: label.to_string(),
            text: text.to_string(),
        }
    }

    fn pool() -> Vec<TrainDoc> {
        vec![
            doc("a", "alpha-1"),
            doc("a", "alpha-2"),
            doc("a", "alpha-3"),
            doc("b", "beta-1"),
        ]
    }

    #[test]
    fn digest_is_deterministic() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let (drafter, _) = per_label_corpus_docs(&pool(), &labels, 2);
        let d1 = corpus_digest("typed_decisions", &labels, &drafter, None);
        let d2 = corpus_digest("typed_decisions", &labels, &drafter, None);
        assert_eq!(d1, d2);
        assert!(d1.starts_with("fnv1a64-"));
    }

    /// The 057 specimen axis: pool growth under an unchanged cap moves the
    /// digest (the pre-lift under-filled pool vs the post-lift filled one).
    #[test]
    fn pool_growth_changes_the_digest_even_at_the_same_cap() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let (small, _) = per_label_corpus_docs(&pool(), &labels, 3);
        let mut grown = pool();
        grown.push(doc("b", "beta-2"));
        let (large, _) = per_label_corpus_docs(&grown, &labels, 3);
        assert_ne!(
            corpus_digest("s", &labels, &small, None),
            corpus_digest("s", &labels, &large, None)
        );
    }

    /// The cap is part of what the engine consumed: truncating differently
    /// is the same signal as changing the pool (the verdict's "cap
    /// truncated differently and pool content changed" are one axis).
    #[test]
    fn cap_change_changes_the_digest() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let (capped2, _) = per_label_corpus_docs(&pool(), &labels, 2);
        let (capped3, _) = per_label_corpus_docs(&pool(), &labels, 3);
        assert_ne!(
            corpus_digest("s", &labels, &capped2, None),
            corpus_digest("s", &labels, &capped3, None)
        );
    }

    /// The nb posture is part of the timed corpus: arming the count tables
    /// (which read the pool UNCAPPED) moves the digest even when the
    /// drafter docs are byte-identical.
    #[test]
    fn nb_arm_changes_the_digest() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let (drafter, _) = per_label_corpus_docs(&pool(), &labels, 2);
        let nb = vec![
            vec![
                "alpha-1".to_string(),
                "alpha-2".to_string(),
                "alpha-3".to_string(),
            ],
            vec!["beta-1".to_string()],
        ];
        assert_ne!(
            corpus_digest("s", &labels, &drafter, None),
            corpus_digest("s", &labels, &drafter, Some(&nb))
        );
    }

    /// Different corpus content (same shape) moves the digest; a different
    /// suite name does too (two suites never share an identity).
    #[test]
    fn content_and_suite_name_change_the_digest() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let (d1, _) = per_label_corpus_docs(&pool(), &labels, 3);
        let mut swapped = pool();
        swapped[0] = doc("a", "ALPHA-1");
        let (d2, _) = per_label_corpus_docs(&swapped, &labels, 3);
        assert_ne!(
            corpus_digest("s", &labels, &d1, None),
            corpus_digest("s", &labels, &d2, None)
        );
        assert_ne!(
            corpus_digest("typed_decisions", &labels, &d1, None),
            corpus_digest("ag_news", &labels, &d1, None)
        );
    }

    /// The self-doc fallback is part of the consumed corpus: a starved
    /// label's corpus is the label text itself, and the fallback labels
    /// come back named (the Issue-039 disclosure rides the same law).
    #[test]
    fn starved_label_falls_back_and_is_digested() {
        let labels = vec!["a".to_string(), "ghost".to_string()];
        let (drafter, fallbacks) = per_label_corpus_docs(&pool(), &labels, 2);
        assert_eq!(fallbacks, vec!["ghost".to_string()]);
        assert_eq!(drafter[1], vec!["ghost".to_string()]);
        // The fallback text is load-bearing: a label renamed ghost→phantom
        // (same empty pool) must move the digest.
        let labels2 = vec!["a".to_string(), "phantom".to_string()];
        let (drafter2, _) = per_label_corpus_docs(&pool(), &labels2, 2);
        assert_ne!(
            corpus_digest("s", &labels, &drafter, None),
            corpus_digest("s", &labels2, &drafter2, None)
        );
    }
}
