//! Issue 090 A1–A3 — the decoy-augmentation / null-stability probe (the
//! runner half).
//!
//! Lane-agnostic: every probed lane answers the SAME augmented cases
//! through its own production decide path (the
//! [`crate::harness::null_stability::NullOracle`] seam) — the probe
//! measures what ships, never a parallel rendering. The pure math lives in
//! [`crate::harness::null_stability`]; this module owns the lanes, the
//! decoy pool, the sampling, the output record, the Report-the-Floor leg,
//! and the render.
//!
//! Probed lanes:
//! - **modelless** — the CONTROL. Full legs: A1 (abstain/confidence drift
//!   curve + the conformal-naive floor), A2 (preference flips, gate ZERO
//!   beyond exact reference ties — independent per-option scoring ⇒ the
//!   ratio is set-invariant within the L1 normalizer's fp envelope), A3
//!   (the paired noul leg, bit-identity asserted — the engine is
//!   stateless across requests, anything else is a defect). A control red
//!   on A2 or A3 refuses the run verdict (exit 1), never a quiet pass.
//!   A1 itself is CHARACTERIZED (the issue's own ⚠: max-of-K drift is the
//!   measured quantity); its gate is the Report-the-Floor leg — the
//!   abstention AUROC/ECE of the shipped confidence must beat the
//!   conformal-naive floor over the raw max-prob (Plan 340 shape,
//!   `metrics::conformal_naive_floor`).
//! - **`--drex` / `--d1` / `--agentjev`** — comparison cells (their
//!   servers must be serving — loud refusal at construction, the
//!   comparison-lane law). They cannot abstain (their deciders return
//!   probs+pick+confidence only) and ship no noul readout: A1's abstain
//!   columns read n/a (never zero), A3's noul leg reads unanswered
//!   (disclosed, never a pass); confidence drift + A2 are measured.
//! - **laya** — NOT wired in this unit: the perm-probe's laya seam
//!   discards the answer's confidence field, and A1/A3 need it. The lane
//!   joins when its oracle exposes a confidence readout (disclosed here so
//!   the omission is a decision, not an oversight).
//!
//! Decoy law: the pool is the UNION of the run's selected suite label
//! vocabularies (the same namespace the engine scores), ranked by
//! trigram-Jaccard to the slot's gold, K evenly-spaced picks across the
//! ranked candidates (the linspace law) — deterministic, near→far graded.
//! A pool smaller than K caps the arm at the available count, disclosed
//! per suite as an aggregate line (never per slot).
//!
//! Measurement posture: the modelless engine is built EXACTLY as the
//! deployed lane builds it (`fit_posture_inner` + `build_engine_with`,
//! every selection lever pinned off) — the perm-probe's posture, verbatim.

use super::*;
use super::perm_probe::stride_indices;
use crate::harness::null_stability::{
    a1_stats, a2_stats, a3_stats, augmented_case, noul_case, null_case_record, AugAnswer,
    DecoyPool, NoulAnswer, NullCaseRecord, NullOracle, NullVerdict, NULL_PROBE_SEED,
};
use crate::harness::metrics::{auroc_of, conformal_naive_floor, ece_of, CalibrationPair};

/// The default suite set — the perm-probe's choice suites (diverse option
/// counts; the pool law wants the vocabulary union).
const DEFAULT_NULL_SUITES: &[&str] = &[
    "typed_decisions",
    "ag_news",
    "emotion",
    "xnli_en",
    "banking77",
    "massive_intent_en",
];

/// The floor leg's minimum calibration support — below this the cell is
/// a named skip, never a degenerate floor.
const MIN_FLOOR_CAL: usize = 20;

/// The probe's knobs (the CLI's `--null-k` / `--null-max-cases` plus the
/// comparison-lane opt-ins, which reuse the run flags' semantics).
#[derive(Debug, Clone)]
pub struct NullProbeOptions {
    /// The augmentation ladder (MUST start at 0 — the reference arm;
    /// enforced loud).
    pub k_set: Vec<usize>,
    pub max_cases: usize,
    pub drex: bool,
    pub d1: bool,
    pub agentjev: bool,
}

// ────────────────────────────────────────────────────────── the record ──

/// The Report-the-Floor cell (A1's gate): the shipped confidence
/// ("gate") vs the split-conformal calibration of the raw max-prob
/// ("floor") on the same slots. AUROC ranks correctness-separation; ECE
/// reads calibration quality. The gate must beat the floor on at least
/// one axis and lose on none.
#[derive(Debug, Clone, Serialize)]
pub struct FloorCell {
    pub n_cal: usize,
    pub n_test: usize,
    pub gate_auroc: f64,
    pub gate_ece: f64,
    pub floor_auroc: f64,
    pub floor_ece: f64,
    pub verdict: NullVerdict,
}

#[derive(Debug, Clone, Serialize)]
pub struct NullLaneReport {
    pub lane: String,
    pub model: String,
    pub control: bool,
    pub n_cases: usize,
    pub can_abstain: bool,
    /// A1 — the drift curve (characterized; the floor cell owns the gate).
    pub a1: Vec<crate::harness::null_stability::A1ArmStats>,
    /// A2 — preference flips per arm (the control gate: ZERO flips beyond
    /// exact reference ties).
    pub a2: Vec<crate::harness::null_stability::A2ArmStats>,
    /// A3 — noul bit-identity + confidence shape.
    pub a3: crate::harness::null_stability::A3Stats,
    /// The floor leg (modelless only — comparison lanes ship no gate).
    pub floor: Option<FloorCell>,
    pub verdict: NullVerdict,
    pub cases: Vec<NullCaseRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NullSuiteReport {
    pub suite: String,
    pub pool_size: usize,
    pub lanes: Vec<NullLaneReport>,
    /// Loud absences: preparation failures, pool caps, lane failures.
    pub skipped: Vec<String>,
    pub slice_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NullProbeMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub k_set: Vec<usize>,
    pub max_cases: usize,
    pub seed: u64,
    pub decoy_rule: String,
    pub gate_rules: String,
    pub laya_note: &'static str,
    pub box_start: crate::harness::box_state::BoxState,
    pub box_end: crate::harness::box_state::BoxState,
}

#[derive(Debug, Clone, Serialize)]
pub struct NullProbeOutput {
    pub meta: NullProbeMeta,
    pub red_cells: usize,
    pub suites: Vec<NullSuiteReport>,
}

// ─────────────────────────────────────────────────────────── the lanes ──

/// The modelless CONTROL oracle: the deployed engine's own decide path,
/// full readouts (`outcome: None` = abstain; `confidence` verbatim).
struct ModellessNullOracle<'e, const N: usize> {
    engine: &'e mut DecisionEngine<N, EMBED_DIM>,
    sc: Scratch<EMBED_DIM>,
}

impl<const N: usize> ModellessNullOracle<'_, N> {
    fn decide(&mut self, case: &SuiteCase) -> Result<katgpt_core::decision_wire::Answer, String> {
        let state_str = serialize_state(&case.state);
        let req = engine_request(case, &state_str)?;
        let resp = self
            .engine
            .decide_with(&req, &mut self.sc)
            .map_err(|e| format!("engine decide ({}): {e}", case.id))?;
        resp.answers
            .into_iter()
            .next()
            .ok_or_else(|| format!("engine decide ({}): no answers in response", case.id))
    }
}

impl<const N: usize> NullOracle for ModellessNullOracle<'_, N> {
    fn answer_null(&mut self, case: &SuiteCase) -> Result<AugAnswer, String> {
        let ans = self.decide(case)?;
        let mut probs = Vec::with_capacity(ans.probabilities.len());
        let pick = answer_probs_pick(QKind::Choice, &ans, &mut probs);
        Ok(AugAnswer {
            probs,
            pick,
            abstained: ans.outcome.is_none(),
            confidence: f64::from(ans.confidence),
            drafter_only: self.sc.slots.first().map(|s| s.drafter_only),
        })
    }

    fn answer_noul(&mut self, case: &SuiteCase) -> Result<Option<NoulAnswer>, String> {
        let ans = self.decide(case)?;
        let yes = match ans.outcome {
            Some(katgpt_core::decision_wire::Outcome::Noul { yes }) => Some(yes),
            _ => None,
        };
        let p_yes = ans
            .probabilities
            .first()
            .map(|p| f64::from(*p))
            .unwrap_or(f64::NAN);
        Ok(Some(NoulAnswer {
            p_yes,
            yes,
            confidence: f64::from(ans.confidence),
        }))
    }

    fn can_abstain(&self) -> bool {
        true
    }
}

/// A comparison lane over the shared `AnswerTriple` shape (probs, pick,
/// confidence) — no abstention (structurally cannot), no noul readout.
macro_rules! triple_lane {
    ($name:ident, $lane_ty:ty, $label:literal) => {
        struct $name {
            lane: $lane_ty,
        }

        impl NullOracle for $name {
            fn answer_null(&mut self, case: &SuiteCase) -> Result<AugAnswer, String> {
                let (outcome, _ms) = self.lane.decide(case)?;
                outcome
                    .answers
                    .first()
                    .map(|(probs, pick, conf)| AugAnswer {
                        probs: probs.clone(),
                        pick: *pick,
                        abstained: false,
                        confidence: *conf,
                        drafter_only: None,
                    })
                    .ok_or_else(|| format!(concat!($label, " ({}): no answers"), case.id))
            }
        }
    };
}

triple_lane!(DrexNullOracle, crate::lanes::drex::DrexLane, "drex");
triple_lane!(D1NullOracle, crate::lanes::d1::D1Lane, "d1");

struct AgentJevNullOracle {
    lane: crate::lanes::agentjev::AgentJevLane,
}

impl NullOracle for AgentJevNullOracle {
    fn answer_null(&mut self, case: &SuiteCase) -> Result<AugAnswer, String> {
        let (answers, _cms, _wms) = self.lane.decide(case)?;
        answers
            .first()
            .map(|(probs, pick, conf)| AugAnswer {
                probs: probs.clone(),
                pick: *pick,
                abstained: false,
                confidence: *conf,
                drafter_only: None,
            })
            .ok_or_else(|| format!("agentjev ({}): no answers", case.id))
    }
}

// ──────────────────────────────────────────────────────── the machinery ──

/// One lane ready to probe (boxed — the shared comparison lanes).
struct SharedNullLane {
    name: &'static str,
    model: String,
    oracle: Box<dyn NullOracle>,
}

/// Answer one slot across the whole ladder + the paired noul leg. The
/// pool may CAP an arm below its requested K (closed label spaces) — the
/// actual presented counts ride the record (`n_presented`), and the LANE
/// aggregates the cap disclosure (never a per-slot spam line).
fn answer_null_slot(
    oracle: &mut dyn NullOracle,
    case: &SuiteCase,
    qi: usize,
    labels: &[String],
    k_set: &[usize],
    pool: &DecoyPool,
    gold_label: Option<&str>,
) -> Result<NullCaseRecord, String> {
    let mut answers = Vec::with_capacity(k_set.len());
    let mut presented = Vec::with_capacity(k_set.len());
    let mut nouls = Vec::with_capacity(k_set.len());
    let noul = noul_case(case, qi);
    for &k in k_set {
        let decoys = pool.graded_decoys(labels, gold_label, k, NULL_PROBE_SEED);
        let aug = augmented_case(case, qi, &decoys)?;
        let n_pres = aug.questions[0]
            .criteria
            .as_object()
            .map(|m| m.len())
            .ok_or_else(|| format!("{}#{}: augmented criteria not an object", case.id, qi))?;
        answers.push(oracle.answer_null(&aug)?);
        presented.push(n_pres);
        // The paired noul leg: a REAL re-answer per arm (the request bytes
        // are identical by construction — the invariance assert catches
        // cross-request state, never a cached copy).
        nouls.push(oracle.answer_noul(&noul)?);
    }
    let ladder: Vec<(usize, usize)> = k_set.iter().copied().zip(presented).collect();
    null_case_record(
        &case.id,
        &case.questions[qi].qid,
        labels,
        &ladder,
        &answers,
        &nouls,
        gold_label,
    )
}

/// The Report-the-Floor leg: calibrate the conformal-naive floor on the
/// suite's cal slice, read both variables on the probe's reference-arm
/// slots. Gate = the shipped confidence; floor = the split-conformal
/// calibration of the RAW max-prob (monotone in raw ⇒ equal AUROC to raw;
/// the ECE is the floor's own calibration quality). Inputs are
/// (gate conf, raw max-prob, correct) triples — the SAME variable must be
/// calibrated and scored on each side.
fn floor_cell(cal: &[(f64, f64, bool)], test: &[(f64, f64, bool)]) -> FloorCell {
    let n_cal = cal.len();
    let n_test = test.len();
    let gate_pairs: Vec<(f64, bool)> = test.iter().map(|(g, _, c)| (*g, *c)).collect();
    // The floor calibrates the RAW variable (the naive forecaster).
    let raw_cal_pairs: Vec<CalibrationPair> = cal
        .iter()
        .map(|(_, r, c)| CalibrationPair {
            conf: *r,
            correct: *c,
        })
        .collect();
    let raw_test: Vec<f64> = test.iter().map(|(_, r, _)| *r).collect();
    let floor_vals = conformal_naive_floor(&raw_cal_pairs, &raw_test);
    let floor_pairs: Vec<(f64, bool)> = floor_vals
        .iter()
        .zip(test.iter().map(|(_, _, c)| *c))
        .map(|(v, c)| (*v, c))
        .collect();
    let gate_auroc = auroc_of(&gate_pairs);
    let gate_ece = ece_of(&gate_pairs);
    let floor_auroc = auroc_of(&floor_pairs);
    let floor_ece = ece_of(&floor_pairs);
    const EPS: f64 = 1e-12;
    let beats = (gate_auroc > floor_auroc + EPS || gate_ece < floor_ece - EPS)
        && gate_auroc >= floor_auroc - EPS
        && gate_ece <= floor_ece + EPS;
    FloorCell {
        n_cal,
        n_test,
        gate_auroc,
        gate_ece,
        floor_auroc,
        floor_ece,
        verdict: if beats { NullVerdict::Pass } else { NullVerdict::Red },
    }
}

/// The modelless cell for one suite: the deployed engine build answering
/// the ladder, the paired noul leg, and the floor leg over the cal slice.
fn null_modelless<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    slots: &[(usize, usize)],
    pool: &DecoyPool,
    nopts: &NullProbeOptions,
    kn_route: bool,
) -> Result<NullLaneReport, String> {
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        gate_fit_selection: false,
        gate_distance_only: false,
        gate_fit_calibrated: true,
        kn_route,
        drafter_fix: crate::engine::DrafterFix::Off,
        suite: &prepared.suite,
        train: &prepared.train,
        state_strs: &prepared.state_strs,
        cal_cases: &prepared.cal_cases,
        cal_state_strs: &prepared.cal_state_strs,
        labels: &prepared.labels,
        want_by_type: false,
        corpus_cap_per_label: spec.corpus_cap_per_label,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        oc_select: false,
        ridge_select: false,
        genome_select: false,
        genome_accept_margin: 0.0,
        cap_source_base: "registry",
        cal_select_caps: &[],
        pool_rows: &prepared.pool_rows,
        pair_head_ab: false,
        nli_feature_ab: false,
        nli_m1: false,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: None,
        density_gate: false,
        leak_flags: None,
    };
    let FittedPosture {
        effective_cap,
        default_cfg,
        score_threshold,
        distance_threshold,
        ..
    } = fit_posture_inner::<N>(&inp)?;
    #[cfg(feature = "option_cond")]
    if default_cfg.oc_scale > 0.0 {
        return Err(format!(
            "the fitted config arms option-conditioned tables (oc_scale {}) — \
             the probe pins the plain posture",
            default_cfg.oc_scale
        ));
    }
    #[cfg(feature = "nb_scope")]
    if default_cfg.nb_scale > 0.0 {
        return Err(format!(
            "the fitted config arms count tables (nb_scale {}) — the probe \
             pins the plain posture",
            default_cfg.nb_scale
        ));
    }
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };
    let (mut engine, fallbacks) =
        build_engine_with::<N>(spec.name, &prepared.train, &prepared.labels, effective_cap, cfg, &[])?;
    if !fallbacks.is_empty() {
        eprintln!(
            "  [issue-039 corpus guard] {}: {} option label(s) with NO train docs \
             → self-doc fallback: {}",
            spec.name,
            fallbacks.len(),
            fallbacks.join(", ")
        );
    }
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(1);

    // The deployed gate's calibration (run_modelless's `fitted` engine
    // shape, verbatim): observe the RAW engine's (conf, correct) over the
    // WHOLE cal slice (every question kind — deployed parity; the
    // calibrated_abstain column of the plain run is this engine), then
    // the ladder measures the DEPLOYED gate. Without this pass the
    // engine's confidence is the raw readout and the score gate
    // abstains everywhere (the raw_abstain rate 1.0 the plain run
    // discloses beside) — measuring a gate nothing ships. The floor's
    // calibration triples (choice-only — variable consistency for the
    // raw-max side) ride the same pass.
    let mut cal_triples: Vec<(f64, f64, bool)> = Vec::new();
    {
        let mut raw_scratch = Scratch::<EMBED_DIM>::new();
        raw_scratch.prepare(1);
        // Pass 1 — collect (conf, correct) over the WHOLE cal slice with
        // the RAW calibrator (no intervening observe: the plain path's
        // cal_pairs shape — every observe refits, so a mixed pass would
        // read progressively-contaminated confidences).
        let mut pairs: Vec<(f32, bool)> = Vec::new();
        for cc in &prepared.cal_cases {
            let state_str = serialize_state(&cc.state);
            let req = engine_request(cc, &state_str)?;
            let resp = engine
                .decide_with(&req, &mut raw_scratch)
                .map_err(|e| format!("engine cal decide ({}): {e}", cc.id))?;
            for (qi, ans) in resp.answers.iter().enumerate() {
                let mut tmp: Vec<f64> = Vec::with_capacity(ans.probabilities.len());
                let pick = answer_probs_pick(cc.questions[qi].kind, ans, &mut tmp);
                let correct = cc
                    .gold
                    .get(qi)
                    .map(|g| pick == g.idx)
                    .unwrap_or(false);
                if cc.questions[qi].kind == QKind::Choice {
                    let raw = tmp.iter().copied().fold(f64::MIN, f64::max);
                    cal_triples.push((f64::from(ans.confidence), raw, correct));
                }
                pairs.push((ans.confidence, correct));
            }
        }
        // Pass 2 — the deployed calibration: feed every pair (the plain
        // path's `for p in &cal_pairs { fitted.observe(...) }`, verbatim).
        for (conf, correct) in pairs {
            engine.observe(conf, correct);
        }
    }

    let mut oracle = ModellessNullOracle {
        engine: &mut engine,
        sc,
    };

    // ── the ladder over the sampled slots ──
    let mut skips = Vec::new();
    let mut recs: Vec<NullCaseRecord> = Vec::with_capacity(slots.len());
    for &(ci, qi) in slots {
        let case = &prepared.suite.cases[ci];
        let labels: Vec<String> = case.questions[qi]
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        let gold_label = case
            .gold
            .get(qi)
            .and_then(|g| labels.get(g.idx))
            .map(String::as_str);
        recs.push(answer_null_slot(
            &mut oracle,
            case,
            qi,
            &labels,
            &nopts.k_set,
            pool,
            gold_label,
        )?);
    }
    if recs.is_empty() {
        return Err(format!(
            "{}: no slots answered — the suite cannot contribute a cell",
            spec.name
        ));
    }
    // The pool-cap disclosure, one line per capped arm (never per slot):
    // min/max presented across the slots at that ladder rung.
    for (ai, &k) in nopts.k_set.iter().enumerate() {
        let capped: Vec<usize> = recs
            .iter()
            .filter(|r| r.arms[ai].n_presented < r.n_options + k)
            .map(|r| r.arms[ai].n_presented)
            .collect();
        if !capped.is_empty() {
            let mn = capped.iter().min().copied().unwrap_or(0);
            let mx = capped.iter().max().copied().unwrap_or(0);
            skips.push(format!(
                "arm K={k}: pool capped ({}/{} slots) — presented {mn}..{mx} vs requested",
                capped.len(),
                recs.len()
            ));
        }
    }

    // ── the floor leg ──
    // `cal_triples` (conf, raw max, correct) were collected on the RAW
    // engine inside the calibration pass above; the floor's conformal
    // calibration variable is the RAW side (monotone ⇒ AUROC equal to
    // raw's; the ECE is the floor's own calibration quality). Test pairs:
    // the reference arm of the ladder (the K=0 distribution, the
    // CALIBRATED engine's confidence — the deployed gate). A, B are the
    // reference top-2 by construction, so the raw max-prob is p_a;
    // abstained rows keep the would-be pick (the wire's own risk-coverage
    // law: an abstained answer may still carry the distribution +
    // confidence).
    let test: Vec<(f64, f64, bool)> = recs
        .iter()
        .map(|r| {
            let arm = &r.arms[0];
            let correct = r.gold.as_deref().map(|g| g == r.a_label).unwrap_or(false);
            (arm.confidence, arm.p_a, correct)
        })
        .collect();
    let floor = (cal_triples.len() >= MIN_FLOOR_CAL).then(|| floor_cell(&cal_triples, &test));
    if floor.is_none() {
        let n_cal = cal_triples.len();
        skips.push(format!(
            "floor: cal slice too small ({n_cal}/{MIN_FLOOR_CAL} pairs — the conformal \
             floor needs more)",
        ));
    }

    let a1 = a1_stats(&recs);
    let a2 = a2_stats(&recs);
    let a3 = a3_stats(&recs);
    let verdict = if a2.iter().any(|x| x.verdict == NullVerdict::Red)
        || a3.noul_bit_identical == Some(false)
        || floor.as_ref().map(|f| f.verdict) == Some(NullVerdict::Red)
    {
        NullVerdict::Red
    } else {
        NullVerdict::Pass
    };
    for s in skips {
        eprintln!("    null[modelless]: skip: {s}");
    }
    eprintln!(
        "    null[modelless]: {} slot(s) · A2 flips {} · A3 noul {} · conf {} · floor {}",
        recs.len(),
        a2.last()
            .map(|x| format!("{}/{}", x.flips, x.n_untied))
            .unwrap_or_else(|| "n/a".into()),
        match a3.noul_bit_identical {
            Some(true) => "BIT-IDENTICAL",
            Some(false) => "DRIFTED",
            None => "n/a",
        },
        a3.conf_shape.split(' ').next().unwrap_or("?"),
        floor
            .as_ref()
            .map(|f| f.verdict.str_of())
            .unwrap_or("n/a"),
    );
    Ok(NullLaneReport {
        lane: "modelless".to_string(),
        model: "modelless (deployed plain posture)".to_string(),
        control: true,
        n_cases: recs.len(),
        can_abstain: true,
        a1,
        a2,
        a3,
        floor,
        verdict,
        cases: recs,
    })
}

/// The per-suite driver: prepare (pass 1 collected all Prepared), slot
/// sampling (the perm-probe's stride law), modelless dispatch + shared
/// lanes.
#[allow(clippy::too_many_arguments)]
fn null_suite(
    spec: &SuiteSpec,
    prepared: &Prepared,
    pool: &DecoyPool,
    nopts: &NullProbeOptions,
    shared: &mut [SharedNullLane],
    kn_route: bool,
) -> NullSuiteReport {
    let mut lanes = Vec::new();
    let mut skipped = Vec::new();
    eprintln!("═══ suite {} (null) ═══", spec.name);
    let choice_cases: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c.questions.iter().any(|q| q.kind == QKind::Choice))
        .map(|(i, _)| i)
        .collect();
    if choice_cases.is_empty() {
        skipped.push("no choice questions — the probe is choice-only".to_string());
        return NullSuiteReport {
            suite: spec.name.to_string(),
            pool_size: pool.len(),
            lanes,
            skipped,
            slice_summary: prepared.slices.as_ref().map(|s| s.summary_line()),
        };
    }
    let sampled = stride_indices(choice_cases.len(), nopts.max_cases);
    let slots: Vec<(usize, usize)> = sampled
        .iter()
        .flat_map(|&si| {
            let ci = choice_cases[si];
            let case = &prepared.suite.cases[ci];
            (0..case.questions.len()).filter_map(move |qi| {
                (case.questions[qi].kind == QKind::Choice).then_some((ci, qi))
            })
        })
        .collect();
    eprintln!(
        "    null: {} choice slot(s) over {} sampled case(s) · ladder {:?}",
        slots.len(),
        sampled.len(),
        nopts.k_set
    );

    macro_rules! dispatch {
        ($n:literal) => {
            null_modelless::<$n>(spec, prepared, &slots, pool, nopts, kn_route)
        };
    }
    let modelless = match prepared.labels.len() {
        2 => dispatch!(2),
        3 => dispatch!(3),
        4 => dispatch!(4),
        5 => dispatch!(5),
        6 => dispatch!(6),
        7 => dispatch!(7),
        8 => dispatch!(8),
        10 => dispatch!(10),
        15 => dispatch!(15),
        59 => dispatch!(59),
        77 => dispatch!(77),
        842 => dispatch!(842),
        n => Err(format!(
            "no engine instantiation for {n} domains — extend the dispatch \
             table in null_probe.rs"
        )),
    };
    match modelless {
        Ok(rep) => lanes.push(rep),
        Err(e) => skipped.push(format!("modelless: {e}")),
    }

    for sl in shared.iter_mut() {
        let mut skips: Vec<String> = Vec::new();
        let mut recs = Vec::with_capacity(slots.len());
        let mut failed: Option<String> = None;
        for &(ci, qi) in &slots {
            let case = &prepared.suite.cases[ci];
            let labels: Vec<String> = case.questions[qi]
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            let gold_label = case
                .gold
                .get(qi)
                .and_then(|g| labels.get(g.idx))
                .map(String::as_str);
            match answer_null_slot(sl.oracle.as_mut(), case, qi, &labels, &nopts.k_set, pool, gold_label) {
                Ok(rec) => recs.push(rec),
                Err(e) => {
                    failed = Some(e);
                    break;
                }
            }
        }
        match failed {
            Some(e) => skipped.push(format!("{}: {e}", sl.name)),
            None => {
                // The same pool-cap disclosure as the control's (one line
                // per capped arm).
                for (ai, &k) in nopts.k_set.iter().enumerate() {
                    let capped: Vec<usize> = recs
                        .iter()
                        .filter(|r| r.arms[ai].n_presented < r.n_options + k)
                        .map(|r| r.arms[ai].n_presented)
                        .collect();
                    if !capped.is_empty() {
                        let mn = capped.iter().min().copied().unwrap_or(0);
                        let mx = capped.iter().max().copied().unwrap_or(0);
                        skips.push(format!(
                            "arm K={k}: pool capped ({}/{} slots) — presented {mn}..{mx} vs requested",
                            capped.len(),
                            recs.len()
                        ));
                    }
                }
                let a1 = a1_stats(&recs);
                let a2 = a2_stats(&recs);
                let a3 = a3_stats(&recs);
                let verdict = if a2.iter().any(|x| x.verdict == NullVerdict::Red) {
                    NullVerdict::Red
                } else {
                    NullVerdict::Pass
                };
                eprintln!(
                    "    null[{}]: {} slot(s) · A2 flips {} · conf {} · verdict {}",
                    sl.name,
                    recs.len(),
                    a2.last()
                        .map(|x| format!("{}/{}", x.flips, x.n_untied))
                        .unwrap_or_else(|| "n/a".into()),
                    a3.conf_shape.split(' ').next().unwrap_or("?"),
                    verdict.str_of(),
                );
                lanes.push(NullLaneReport {
                    lane: sl.name.to_string(),
                    model: sl.model.clone(),
                    control: false,
                    n_cases: recs.len(),
                    can_abstain: false,
                    a1,
                    a2,
                    a3,
                    floor: None,
                    verdict,
                    cases: recs,
                });
                for s in skips {
                    skipped.push(format!("{}: {s}", sl.name));
                }
            }
        }
    }

    NullSuiteReport {
        suite: spec.name.to_string(),
        pool_size: pool.len(),
        lanes,
        skipped,
        slice_summary: prepared.slices.as_ref().map(|s| s.summary_line()),
    }
}

/// Run the probe (the bin's `--null-probe` early-exit lane).
///
/// # Errors
/// Loud refusals: an empty ladder, a ladder not starting at 0, no suites
/// measured, unreachable comparison lanes.
pub fn run_null_probe(
    opts: &RunOptions,
    nopts: &NullProbeOptions,
) -> Result<NullProbeOutput, String> {
    if nopts.k_set.is_empty() {
        return Err("null probe: the augmentation ladder is empty".to_string());
    }
    if nopts.k_set[0] != 0 {
        return Err(format!(
            "null probe: the ladder must start at 0 (the reference arm) — got {:?}",
            nopts.k_set
        ));
    }
    let box_start = crate::harness::box_state::capture();
    let mut shared: Vec<SharedNullLane> = Vec::new();
    if nopts.drex {
        let lane = crate::lanes::drex::DrexLane::default();
        lane.health()
            .map_err(|e| format!("--null-probe --drex: {e}"))?;
        let canary = crate::harness::permutation::canary_case();
        let raw = lane
            .decide_raw(&canary)
            .map_err(|e| format!("--null-probe --drex warmup: {e}"))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("--null-probe --drex warmup parse: {e}"))?;
        let model = crate::lanes::drex::DrexLane::model_of(&parsed);
        shared.push(SharedNullLane {
            name: "drex",
            model,
            oracle: Box::new(DrexNullOracle { lane }),
        });
    }
    if nopts.d1 {
        let lane = crate::lanes::d1::D1Lane::default();
        lane.health()
            .map_err(|e| format!("--null-probe --d1: {e}"))?;
        let canary = crate::harness::permutation::canary_case();
        let raw = lane
            .decide_raw(&canary)
            .map_err(|e| format!("--null-probe --d1 warmup: {e}"))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("--null-probe --d1 warmup parse: {e}"))?;
        let model = crate::lanes::d1::D1Lane::model_of(&parsed);
        shared.push(SharedNullLane {
            name: "d1",
            model,
            oracle: Box::new(D1NullOracle { lane }),
        });
    }
    if nopts.agentjev {
        let lane = crate::lanes::agentjev::AgentJevLane::default();
        let (model, provenance) = lane
            .info()
            .map_err(|e| format!("--null-probe --agentjev: {e}"))?;
        let canary = crate::harness::permutation::canary_case();
        lane.decide(&canary)
            .map_err(|e| format!("--null-probe --agentjev warmup: {e}"))?;
        shared.push(SharedNullLane {
            name: "agentjev",
            model: format!("{model} ({provenance})"),
            oracle: Box::new(AgentJevNullOracle { lane }),
        });
    }

    // Pass 1: prepare every selected suite, build the decoy pool from the
    // label-vocabulary union (the same namespace the engine scores).
    let mut prepared_list: Vec<(&SuiteSpec, Prepared)> = Vec::new();
    let mut suite_labels: Vec<Vec<String>> = Vec::new();
    let mut prep_skipped: Vec<(String, String)> = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only || !DEFAULT_NULL_SUITES.contains(&spec.name) {
            continue;
        }
        match prepare(spec, &opts.datasets_dir) {
            Ok(p) => {
                suite_labels.push(p.labels.clone());
                prepared_list.push((spec, p));
            }
            Err(e) => prep_skipped.push((spec.name.to_string(), format!("prepare: {e}"))),
        }
    }
    if prepared_list.is_empty() {
        return Err(format!(
            "null probe: no suite prepared ({} request line(s)) — check --suites \
             against the registry",
            opts.suites.len()
        ));
    }
    let pool = DecoyPool::build(&suite_labels);
    if pool.is_empty() {
        return Err("null probe: the decoy pool is empty (no label vocabularies)".to_string());
    }

    // Pass 2: probe each suite.
    let mut suites = Vec::new();
    for (spec, prepared) in &prepared_list {
        suites.push(null_suite(spec, prepared, &pool, nopts, &mut shared, opts.kn_route));
    }
    for (name, why) in prep_skipped {
        suites.push(NullSuiteReport {
            suite: name,
            pool_size: pool.len(),
            lanes: Vec::new(),
            skipped: vec![why],
            slice_summary: None,
        });
    }
    let red_cells: usize = suites
        .iter()
        .map(|s| s.lanes.iter().filter(|l| l.verdict == NullVerdict::Red).count())
        .sum();
    Ok(NullProbeOutput {
        meta: NullProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            k_set: nopts.k_set.clone(),
            max_cases: nopts.max_cases,
            seed: NULL_PROBE_SEED,
            decoy_rule: "pool = the union of the run's suite label vocabularies \
                         (first-seen order); per slot: pool minus the slot's own \
                         labels, ranked by trigram-Jaccard to the GOLD label DESC, \
                         then K evenly-spaced picks across the ranked list (the \
                         linspace law: near→far graded, deterministic); decoys \
                         APPEND (originals keep positions); a pool smaller than K \
                         caps the arm (disclosed per suite)."
                .to_string(),
            gate_rules: "CONTROL gates (a red refuses the run): A2 — ZERO \
                         WITHIN-REGIME preference flips beyond exact reference ties \
                         (the engine's by-name route arms only while k ≤ N, so an arm \
                         past the domain count switches regimes BY DESIGN — those \
                         flips are the arming law, disclosed as cross-regime, never \
                         red); A3a — the paired noul readout BIT-identical across \
                         arms; A1-floor (Report-the-Floor, Plan 340 shape) — the \
                         DEPLOYED (cal-slice-calibrated) confidence's abstention \
                         AUROC/ECE must beat the conformal-naive floor over the raw \
                         max-prob on the cal/test split (≥1 axis better, none worse). \
                         A1's drift curve itself is CHARACTERIZED (the issue's own ⚠: \
                         max-of-K drift is the measured quantity), never gated on an \
                         unstudied ε. Comparison-lane reds are disclosed rows."
                .to_string(),
            laya_note: "laya not wired in this unit: the perm-probe seam discards the \
                        answer's confidence field and A1/A3 need it — the lane joins \
                        when its oracle exposes a confidence readout.",
            box_start,
            box_end: crate::harness::box_state::capture(),
        },
        red_cells,
        suites,
    })
}

/// The markdown rendering (the bench record's table block).
#[must_use]
pub fn render_null_probe_markdown(out: &NullProbeOutput) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# Null/K-stability probe — {} (host {}, datasets {}) — ladder {:?}, ≤{} slot(s)/suite",
        out.meta.date_utc,
        out.meta.host,
        out.meta.datasets_dir,
        out.meta.k_set,
        out.meta.max_cases
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "- decoys: {}", out.meta.decoy_rule);
    let _ = writeln!(s, "- gates: {}", out.meta.gate_rules);
    let _ = writeln!(s, "- laya: {}", out.meta.laya_note);
    let _ = writeln!(s);
    for suite in &out.suites {
        let _ = writeln!(
            s,
            "## {} (pool {} labels) — {} lane cell(s)",
            suite.suite,
            suite.pool_size,
            suite.lanes.len()
        );
        for lane in &suite.lanes {
            let _ = writeln!(
                s,
                "### {} — {} slot(s) · {} · {}",
                lane.lane,
                lane.n_cases,
                if lane.control { "CONTROL" } else { "comparison" },
                lane.verdict.str_of()
            );
            let _ = writeln!(
                s,
                "| K | abstain% | Δabstain pt | med\\|Δconf\\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |"
            );
            let _ = writeln!(s, "|---|---|---|---|---|---|---|---|");
            for (i, a1) in lane.a1.iter().enumerate() {
                let a2 = lane.a2.get(i);
                let _ = writeln!(
                    s,
                    "| {} | {:.1} | {:+.1} | {:.2} | {:.1} | {} | {} ({}/{}) | {} |",
                    a1.k,
                    a1.abstain_rate * 100.0,
                    a1.d_abstain_pt,
                    a1.median_abs_d_conf_pt,
                    a1.decoy_win_rate * 100.0,
                    a2.map(|x| x.n_untied).unwrap_or(0),
                    a2.map(|x| x.flips).unwrap_or(0),
                    a2.map(|x| x.within_regime_flips).unwrap_or(0),
                    a2.map(|x| x.cross_regime_flips).unwrap_or(0),
                    a2.map(|x| x.verdict.str_of()).unwrap_or("-"),
                );
            }
            let _ = writeln!(
                s,
                "- A3: noul bit-identity {} · confidence {} (med |Δconf| top arm {:.2} pt)",
                match lane.a3.noul_bit_identical {
                    Some(true) => "**BIT-IDENTICAL**",
                    Some(false) => "**DRIFTED**",
                    None => "n/a (no noul readout)",
                },
                lane.a3.conf_shape,
                lane.a3.median_abs_d_conf_top_pt
            );
            if let Some(f) = &lane.floor {
                let _ = writeln!(
                    s,
                    "- floor (Report-the-Floor): gate AUROC {:.4} / ECE {:.4} vs floor \
                     AUROC {:.4} / ECE {:.4} (n_cal {}, n_test {}) — **{}**",
                    f.gate_auroc,
                    f.gate_ece,
                    f.floor_auroc,
                    f.floor_ece,
                    f.n_cal,
                    f.n_test,
                    f.verdict.str_of()
                );
            }
        }
        if !suite.skipped.is_empty() {
            let _ = writeln!(s, "- skipped: {}", suite.skipped.join(" · "));
        }
        if let Some(ss) = &suite.slice_summary {
            let _ = writeln!(s, "- slice: {ss}");
        }
        let _ = writeln!(s);
    }
    let _ = writeln!(
        s,
        "red cells: {} — control reds refuse the run (exit 1)",
        out.red_cells
    );
    s
}
