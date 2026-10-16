//! Issue 077 T1–T3 — the option-PERMUTATION spread probe (the runner half).
//!
//! Lane-agnostic: every probed lane answers the SAME permuted
//! single-question cases through its OWN production decide path (the
//! [`crate::harness::permutation::ChoiceOracle`] seam) — the probe measures
//! what ships, never a parallel rendering. The pure math lives in
//! [`crate::harness::permutation`]; this module owns the lanes, the
//! sampling, the output record, and the render.
//!
//! Probed lanes (issue 077 T2):
//! - **modelless** — the CONTROL. The engine is permutation-invariant by
//!   construction (content-hashed option scoring, no position priors); the
//!   probe ASSERTS it per slot: every ordering's label-space probabilities
//!   must be BIT-identical to the identity ordering's. A control red is a
//!   finding (an engine defect) or a harness bug — either way the bin
//!   refuses the run verdict (exit 1), never a quiet pass.
//! - **`--drex`** — the Drex DLM comparison lane over their loopback
//!   TypeSafe wire (`DREX_SERVE_URL`); an unreachable server is a LOUD
//!   refusal at lane construction (the comparison-lane law), never a
//!   silent skip.
//! - **`--agentjev`** — their `jev_service` (`AGENTJEV_SERVE_URL`); the
//!   recorded wobble class (Bench 127's det re-read) — disclosed in the
//!   record, never adjusted.
//! - **laya** (feature `laya-riir`, honors `--skip-laya`) — the pooled-state
//!   cross-marker attention path, the one learnable position-sensitivity
//!   surface we run (the d1 research's item-4 subject). Each suite's
//!   PRIMARY checkpoint only (typed for typed_decisions, english else) —
//!   disclosed; a bucket-limited case skips the SLOT named (state-side,
//!   ordering-independent), never silently.
//!
//! Measurement posture: the modelless engine is built EXACTLY as the
//! deployed lane builds it — `fit_posture_inner` (the shipped cal-slice
//! threshold fit) + `build_engine_with` at the registry cap — with every
//! selection lever pinned OFF (oc/nb tables refuse loud if the fitted
//! config somehow arms them: the probe pins the plain posture). Latency
//! columns are deliberately absent: this is a distribution-stability
//! probe; the latency + determinism axes live in the det benches (126/127).
//!
//! Gate (issue 077 T1): per lane × suite, the MEDIAN top-pick swing across
//! orderings must stay ≤ 2 pt ([`crate::harness::permutation::
//! GATE_MEDIAN_SWING_PT`]); max + flips disclosed beside. The canary case
//! rides every lane in every run as the liveness cell — an order-biased
//! lane flips it (the probe proves it can fire; the mock-lane tests in the
//! pure module pin both directions).

use super::*;
use crate::harness::permutation::{
    canary_case, case_spread, label_space_probs, perm_orderings, permuted_case, slot_seed,
    spread_stats, CaseSpread, ChoiceOracle, OrderingAnswer, PROBE_SEED, SpreadVerdict,
    GATE_MEDIAN_SWING_PT,
};

/// Orderings per slot, INCLUDING the identity (default 5 = identity + 4
/// shuffles).
pub const DEFAULT_PERM_K: usize = 5;
/// Cases sampled per suite by deterministic stride over the suite's
/// choice-carrying cases in dataset order (default 40).
pub const DEFAULT_PERM_MAX_CASES: usize = 40;
/// The default suite set: the board's choice suites (diverse option
/// counts: 3 / 4 / 6 / ~20 / 77 / variable). Score- and noul-only suites
/// are structurally out of scope; `--suites` overrides.
const DEFAULT_PROBE_SUITES: &[&str] = &[
    "typed_decisions",
    "ag_news",
    "emotion",
    "xnli_en",
    "banking77",
    "massive_intent_en",
];

/// The probe's knobs (the CLI's `--perm-k` / `--perm-max-cases` plus the
/// comparison-lane opt-ins, which reuse the run flags' semantics).
#[derive(Debug, Clone)]
pub struct PermProbeOptions {
    pub k: usize,
    pub max_cases: usize,
    /// Probe the Drex lane (their server must be serving — loud refusal).
    pub drex: bool,
    /// Probe the LiquidAI d1 lane (their server must be serving — loud
    /// refusal). Issue 078 T4: d1's own recipe names option shuffling, so
    /// their model is the probe's most interesting subject.
    pub d1: bool,
    /// Probe the AgentJev lane (their server must be serving — loud
    /// refusal).
    pub agentjev: bool,
    /// Probe the Jeva lane (Issue 090 B1: our reference server must be
    /// serving — loud refusal). Their paper's own 0.00% order-flip bar
    /// (mean TV 3.17e-8) is the B2 0-flip floor's target shape.
    pub jeva: bool,
    /// Issue 086 T1 — also run the presentation-subspace certificate
    /// (`--basis-split`): rank-gated presentation basis + per-suite
    /// held-out leakage over the SAME sampled slots' perm pairs, routed
    /// through the hashed-feature embed. Report-only; never touches a
    /// decide path. Requires the `basis_cert` feature (the run refuses
    /// loud without it, never a silent skip).
    pub basis_split: bool,
}

// ────────────────────────────────────────────────────────── the record ──

/// One answered slot, serialized (the JSON record's per-case row).
#[derive(Debug, Clone, Serialize)]
pub struct CaseRow {
    pub case_id: String,
    pub qid: String,
    pub n_options: usize,
    /// The identity ordering's top-pick label.
    pub ref_pick: String,
    /// Top-pick label under each ordering ([0] == `ref_pick`).
    pub pick_labels: Vec<String>,
    /// Probability on `ref_pick` under each ordering.
    pub p_ref: Vec<f64>,
    pub swing_pt: f64,
    pub flipped: bool,
    /// The identity ordering's top-2 probabilities tie EXACTLY (margin
    /// 0.0) — the control's tie-break-exempt class, and T1's untied
    /// filter (issue 089).
    pub tie: bool,
    /// The gold option key (issue 089 T1 — the error-prediction join;
    /// `None` when the slot has no gold — never over the suite paths).
    pub gold: Option<String>,
    /// The identity ordering's top-2 margin `p(1) − p(2)` (issue 089 T1 —
    /// the margin-null baseline; `None` under two options).
    pub margin_top2: Option<f64>,
}

impl CaseRow {
    fn of(s: &CaseSpread) -> Self {
        Self {
            case_id: s.case_id.clone(),
            qid: s.qid.clone(),
            n_options: s.n_options,
            ref_pick: s.ref_pick.clone(),
            pick_labels: s.pick_labels.clone(),
            p_ref: s.p_ref.clone(),
            swing_pt: s.swing * 100.0,
            flipped: s.flipped,
            tie: s.tie,
            gold: s.gold.clone(),
            margin_top2: s.margin_top2,
        }
    }
}

/// One lane × suite cell: the aggregate spread, the gate verdict, the
/// control's byte-identity assert, and the per-case rows.
#[derive(Debug, Clone, Serialize)]
pub struct PermLaneReport {
    pub lane: String,
    pub model: String,
    /// True for the modelless control — a red here refuses the run
    /// verdict (an engine invariant failed or the harness is buggy).
    pub control: bool,
    pub n_cases: usize,
    pub n_orderings: usize,
    pub median_swing_pt: f64,
    pub max_swing_pt: f64,
    pub flips: usize,
    pub flip_rate: f64,
    /// The control's byte-identity verdict across orderings: every
    /// ordering's label-space probabilities are BIT-identical to the
    /// identity ordering's. `None` = not the control (the comparison is
    /// not the claim).
    pub control_invariant: Option<bool>,
    pub verdict: SpreadVerdict,
    /// The liveness cell (an order-biased lane flips it BY DESIGN).
    pub canary: Option<CaseRow>,
    pub cases: Vec<CaseRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermSuiteReport {
    pub suite: String,
    pub lanes: Vec<PermLaneReport>,
    /// Loud absences: preparation failures, structurally-out-of-scope
    /// suites (score/noul-only), bucket-limited slots, lane failures.
    pub skipped: Vec<String>,
    /// The slice-integrity summary line (the split provenance; None on
    /// the in-process suites).
    pub slice_summary: Option<String>,
    /// Issue 086 T1 — the certificate report (`--basis-split`). Gated
    /// with the lane: the type lives in the feature-gated module, so the
    /// field (and the record key) exists only where the lane compiles.
    #[cfg(feature = "basis_cert")]
    pub basis_cert: Option<crate::harness::basis_cert::BasisCertReport>,
    /// Issue 086 T3 — the Thm-3 perturbation certificate (`--basis-split`,
    /// requires the modelless engine + a certified split).
    #[cfg(feature = "basis_cert")]
    pub basis_perturb: Option<BasisPerturbReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermProbeMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub k: usize,
    pub max_cases: usize,
    pub seed: u64,
    pub gate_rule: String,
    pub sampling_rule: String,
    pub invariance_rule: String,
    /// The modelless control's route binding posture (issue 079): the
    /// incumbent legacy `k == N` index alignment, or the content-bound
    /// `--no-kn-route` posture — quote it with the numbers.
    pub route_binding: String,
    pub latency_note: String,
    pub laya_device: Option<String>,
    pub box_start: crate::harness::box_state::BoxState,
    pub box_end: crate::harness::box_state::BoxState,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermProbeOutput {
    pub meta: PermProbeMeta,
    pub red_cells: usize,
    pub suites: Vec<PermSuiteReport>,
}

// ─────────────────────────────────────────────────────────── the lanes ──

/// A comparison lane ready to probe: name + provenance + the oracle.
struct SharedLane {
    name: &'static str,
    model: String,
    oracle: Box<dyn ChoiceOracle>,
}

/// The modelless CONTROL oracle: the deployed engine's own decide path
/// (`engine_request` → `decide_with` → `answer_probs_pick`).
struct ModellessOracle<'e, const N: usize> {
    engine: &'e mut DecisionEngine<N, EMBED_DIM>,
    sc: Scratch<EMBED_DIM>,
}

impl<const N: usize> ChoiceOracle for ModellessOracle<'_, N> {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        let state_str = serialize_state(&case.state);
        let req = engine_request(case, &state_str)?;
        let resp = self
            .engine
            .decide_with(&req, &mut self.sc)
            .map_err(|e| format!("engine decide ({}): {e}", case.id))?;
        let ans = resp.answers.first().ok_or_else(|| {
            format!("engine decide ({}): no answers in response", case.id)
        })?;
        let mut probs = Vec::with_capacity(ans.probabilities.len());
        let pick = answer_probs_pick(QKind::Choice, ans, &mut probs);
        Ok(OrderingAnswer { probs, pick })
    }
}

struct DrexOracle {
    lane: crate::lanes::drex::DrexLane,
}

impl ChoiceOracle for DrexOracle {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        let (outcome, _client_ms) = self.lane.decide(case)?;
        outcome
            .answers
            .first()
            .map(|(probs, pick, _)| OrderingAnswer {
                probs: probs.clone(),
                pick: *pick,
            })
            .ok_or_else(|| format!("drex ({}): no answers in response", case.id))
    }
}

struct D1Oracle {
    lane: crate::lanes::d1::D1Lane,
}

impl ChoiceOracle for D1Oracle {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        let (outcome, _client_ms) = self.lane.decide(case)?;
        outcome
            .answers
            .first()
            .map(|(probs, pick, _)| OrderingAnswer {
                probs: probs.clone(),
                pick: *pick,
            })
            .ok_or_else(|| format!("d1 ({}): no answers in response", case.id))
    }
}

struct AgentJevOracle {
    lane: crate::lanes::agentjev::AgentJevLane,
}

impl ChoiceOracle for AgentJevOracle {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        let (answers, _client_ms, _wall_ms) = self.lane.decide(case)?;
        answers
            .first()
            .map(|(probs, pick, _)| OrderingAnswer {
                probs: probs.clone(),
                pick: *pick,
            })
            .ok_or_else(|| format!("agentjev ({}): no answers in response", case.id))
    }
}

struct JevaOracle {
    lane: crate::lanes::jeva::JevaLane,
}

impl ChoiceOracle for JevaOracle {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        let (outcome, _client_ms) = self.lane.decide(case)?;
        outcome
            .answers
            .first()
            .map(|(probs, pick, _)| OrderingAnswer {
                probs: probs.clone(),
                pick: *pick,
            })
            .ok_or_else(|| format!("jeva ({}): no answers in response", case.id))
    }
}

/// The laya oracle — the pooled-state cross-marker attention path, the
/// lane the d1 research's item 4 names as the interesting subject.
#[cfg(feature = "laya-riir")]
struct LayaOracle {
    agent: crate::laya::riir::RiirAgent,
}

#[cfg(feature = "laya-riir")]
impl ChoiceOracle for LayaOracle {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        match self.agent.system_one(&case.state, &case_questions(case)) {
            Ok(answers) => {
                let ans = answers.first().ok_or_else(|| {
                    format!("laya ({}): no answers in response", case.id)
                })?;
                // laya_prob_pick reads the option keys from the PERMUTED
                // question — the answer maps into presented order exactly
                // like every other lane (the ONE rendering law).
                let (probs, pick) = laya_prob_pick(&case.questions[0], ans);
                Ok(OrderingAnswer { probs, pick })
            }
            // Bucket limits are STATE-side (ordering-independent): the
            // caller skips the whole slot named.
            Err(crate::laya::LayaError::Bucket { seq, max, .. }) => Err(format!(
                "bucket: laya seq {seq} > max bucket {max}"
            )),
            Err(e) => Err(format!("laya: {e}")),
        }
    }
}

// ──────────────────────────────────────────────────────── the machinery ──

/// Deterministic stride sampling over `n_total` items: all of them when
/// they fit, else the first of every `ceil(n/max)` stride.
pub(super) fn stride_indices(n_total: usize, max_cases: usize) -> Vec<usize> {
    if n_total == 0 {
        return Vec::new();
    }
    if n_total <= max_cases {
        return (0..n_total).collect();
    }
    let stride = n_total.div_ceil(max_cases);
    (0..n_total).step_by(stride).take(max_cases).collect()
}

/// One lane's error convention: a "bucket:"-prefixed error is a STATE-side
/// skip (ordering-independent) — the slot is named and dropped; anything
/// else fails the lane loud. The `__SLOT_SKIPPED__` sentinel is never
/// returned — bucket skips surface as `Ok(None)` so the caller names them
/// without colliding with lane error strings.
/// Returns `(spread, control_invariant)`: the control's byte-identity
/// verdict across THIS slot's orderings (always `true` for non-controls —
/// the comparison is not their claim).
fn answer_slot(
    oracle: &mut dyn ChoiceOracle,
    case: &SuiteCase,
    qi: usize,
    k: usize,
    control: bool,
    skips: &mut Vec<String>,
) -> Result<Option<(CaseSpread, bool)>, String> {
    let q = &case.questions[qi];
    let labels: Vec<String> = q
        .criteria
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .ok_or_else(|| format!("case {}: not a choice question", case.id))?;
    let n = labels.len();
    let gold_label = case
        .gold
        .get(qi)
        .and_then(|g| labels.get(g.idx))
        .map(String::as_str);
    let orderings = perm_orderings(n, k, slot_seed(&case.id, &q.qid));
    let mut answers: Vec<OrderingAnswer> = Vec::with_capacity(orderings.len());
    // The control's byte-identity assert: the identity ordering's
    // label-space bits, compared against every other ordering's.
    let mut ref_bits: Option<Vec<u64>> = None;
    let mut invariant = true;
    for (rep, order) in orderings.iter().enumerate() {
        let pc = permuted_case(case, qi, order, rep)?;
        match oracle.answer(&pc) {
            Ok(a) => {
                if control {
                    let bits: Vec<u64> = label_space_probs(&a.probs, order, n)
                        .iter()
                        .map(|p| p.to_bits())
                        .collect();
                    match &ref_bits {
                        None => ref_bits = Some(bits),
                        Some(r) => {
                            if *r != bits {
                                invariant = false;
                            }
                        }
                    }
                }
                answers.push(a);
            }
            // Bucket limits are STATE-side (ordering-independent): every
            // ordering of this slot fails identically — name the skip and
            // drop the slot, never fabricate a spread.
            Err(e) if e.starts_with("bucket:") => {
                skips.push(format!(
                    "{}#{}: {e} (bucket-limited — state-side, ordering-independent)",
                    case.id, q.qid
                ));
                return Ok(None);
            }
            Err(e) => return Err(format!("{}#{}: {e}", case.id, q.qid)),
        }
    }
    let spread = case_spread(&case.id, &q.qid, &labels, &orderings, &answers, gold_label)?;
    Ok(Some((spread, invariant)))
}

/// A lane's identity for one probe pass: display name, provenance model
/// string, and whether it is the CONTROL (the byte-identity assert rides
/// that flag).
struct LaneRun<'a> {
    name: &'a str,
    model: &'a str,
    control: bool,
}

fn probe_lane(
    oracle: &mut dyn ChoiceOracle,
    run: LaneRun<'_>,
    canary: &SuiteCase,
    slots: &[(usize, usize)],
    prepared: &Prepared,
    k: usize,
    skips: &mut Vec<String>,
) -> Result<PermLaneReport, String> {
    // The canary FIRST — the run's liveness cell (it can never bucket:
    // a five-option plain-string case is under every backend's limits).
    let (canary_spread, canary_invariant) =
        match answer_slot(oracle, canary, 0, k, run.control, skips)? {
            Some(pair) => pair,
            None => return Err("canary: bucket-limited (unexpected — the canary's ".to_string()
                + "state is under every backend's limits)"),
        };
    let mut invariant = canary_invariant;
    let mut spreads: Vec<CaseSpread> = Vec::with_capacity(slots.len());
    for &(ci, qi) in slots {
        let case = &prepared.suite.cases[ci];
        match answer_slot(oracle, case, qi, k, run.control, skips) {
            Ok(Some((spread, slot_invariant))) => {
                invariant &= slot_invariant;
                if spread.flipped {
                    let moved: Vec<&String> = spread
                        .pick_labels
                        .iter()
                        .filter(|l| **l != spread.ref_pick)
                        .collect();
                    eprintln!(
                        "      perm[{}] FLIP {}#{}: {} → {} (swing {:.1} pt)",
                        run.name,
                        spread.case_id,
                        spread.qid,
                        spread.ref_pick,
                        moved
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>()
                            .join("|"),
                        spread.swing * 100.0
                    );
                }
                spreads.push(spread);
            }
            Ok(None) => {} // bucket skip — named in skips by answer_slot
            Err(e) => return Err(e),
        }
    }
    let stats = spread_stats(&spreads);
    let any_untied_flip = spreads.iter().any(|s| s.flipped && !s.tie);
    let verdict = if run.control {
        // The control's contract (issue 077, Bench 128's revision): (a)
        // CONTENT BINDING — the top pick may move ONLY on an exact top-2
        // tie (the deterministic first-position tie-break); (b) NUMERIC
        // STABILITY — the median swing across the UNTIED slots stays
        // within the L1 normalizer's measured fp envelope
        // (CONTROL_MEDIAN_CEILING_PT); a tied slot's p_ref movement is
        // the tie STRUCTURE itself (the mock-oracle test pins exactly
        // this), never a sensitivity datum. STRICT byte-identity stays a
        // disclosed column, never the verdict: the L1 sum reorders with
        // the presentation and moves ULPs by design.
        let untied: Vec<CaseSpread> = spreads.iter().filter(|s| !s.tie).cloned().collect();
        let ctl = spread_stats(&untied);
        if ctl.median_swing_pt > crate::harness::permutation::CONTROL_MEDIAN_CEILING_PT
            || any_untied_flip
        {
            SpreadVerdict::Red
        } else {
            SpreadVerdict::Pass
        }
    } else {
        stats.verdict
    };
    eprintln!(
        "    perm[{}]: median {:.2} pt · max {:.2} pt · flips {}/{} · {}{}",
        run.name,
        stats.median_swing_pt,
        stats.max_swing_pt,
        stats.flips,
        stats.n,
        verdict_str(verdict),
        if run.control {
            format!(
                " · control byte-identity: {} · untied flips: {}",
                if invariant { "HOLDS" } else { "BROKEN" },
                spreads.iter().filter(|s| s.flipped && !s.tie).count()
            )
        } else {
            String::new()
        },
    );
    Ok(PermLaneReport {
        lane: run.name.to_string(),
        model: run.model.to_string(),
        control: run.control,
        n_cases: stats.n,
        n_orderings: k,
        median_swing_pt: stats.median_swing_pt,
        max_swing_pt: stats.max_swing_pt,
        flips: stats.flips,
        flip_rate: stats.flip_rate,
        control_invariant: run.control.then_some(invariant),
        verdict,
        canary: Some(CaseRow::of(&canary_spread)),
        cases: spreads.iter().map(CaseRow::of).collect(),
    })
}

/// The modelless CONTROL cell for one suite: the deployed engine build
/// (`fit_posture_inner` + `build_engine_with`, every selection lever
/// pinned off) answering every sampled slot under the K orderings. The
/// route-binding posture rides the invocation (`--no-kn-route`, issue
/// 079) — the knob is an ENGINE posture, not a lever, so the plain-
/// posture refusal below does not apply to it.
/// Issue 086 T1+T2+T3 — the certificate lane's fit artifacts: the T1/T2
/// report plus T3's perturbation basis (the certified variant directions)
/// and the delta-scale anchor (median raw fit-pair delta norm).
#[cfg(feature = "basis_cert")]
struct BasisCertLane {
    report: crate::harness::basis_cert::BasisCertReport,
    directions: Option<crate::harness::basis_cert::BasisDirections>,
    median_delta_norm: Option<f64>,
}

/// The modelless lane's full outcome: the control report plus the issue-086
/// certificate lanes (which run INSIDE the const-generic fn — the engine
/// type differs per N and cannot cross the dispatch boundary).
struct ModellessOutcome {
    report: PermLaneReport,
    #[cfg(feature = "basis_cert")]
    basis_cert: Option<crate::harness::basis_cert::BasisCertReport>,
    #[cfg(feature = "basis_cert")]
    basis_perturb: Option<BasisPerturbReport>,
    #[cfg(feature = "basis_cert")]
    basis_skips: Vec<String>,
}

fn perm_modelless<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    slots: &[(usize, usize)],
    popts: &PermProbeOptions,
    kn_route: bool,
) -> Result<ModellessOutcome, String> {
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        // The probe is a report-only early-exit lane — no gate levers.
        // The calibrated scale-coherence fit is not a lever anymore (the
        // Issue-056 promotion): it is the deployed default the probe
        // reads alongside.
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
        set_refine: None,
        leak_flags: None,
    };
    let FittedPosture {
        effective_cap,
        default_cfg,
        score_threshold,
        distance_threshold,
        ..
    } = fit_posture_inner::<N>(&inp)?;
    // The probe pins the PLAIN deployed posture: the selection levers are
    // all off, so the fitted config must not arm the oc/nb tables. A
    // nonzero scale here means a future posture change reached the
    // fitted config — refuse loud rather than silently measure a
    // different engine than the plain build would.
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
    let mut oracle = ModellessOracle {
        engine: &mut engine,
        sc,
    };
    let canary = canary_case();
    let mut skips = Vec::new();
    let report = probe_lane(
        &mut oracle,
        LaneRun {
            name: "modelless",
            model: "modelless (deployed plain posture)",
            control: true,
        },
        &canary,
        slots,
        prepared,
        popts.k,
        &mut skips,
    )?;
    drop(oracle); // release &mut engine for the perturbation lane
    // Issue 086 T1+T3 — the certificate lanes run HERE (the engine's const
    // N is in scope; the outcomes are non-generic). T1 fits the presentation
    // basis from perm-pair renderings; T3 perturbs the DEPLOYED engine's
    // embeddings along the certified directions through the additive hook.
    #[cfg(feature = "basis_cert")]
    let mut basis_skips: Vec<String> = Vec::new();
    #[cfg(feature = "basis_cert")]
    let (basis_cert, basis_perturb) = if popts.basis_split {
        match basis_cert_suite(prepared, popts.k, &mut basis_skips) {
            Some(lane) => {
                let perturb = match (lane.directions.as_ref(), lane.median_delta_norm) {
                    (Some(dirs), Some(med)) => {
                        basis_perturb_suite(&mut engine, prepared, dirs, med, &mut basis_skips)
                    }
                    _ => None,
                };
                (Some(lane.report), perturb)
            }
            None => (None, None),
        }
    } else {
        (None, None)
    };
    Ok(ModellessOutcome {
        report,
        #[cfg(feature = "basis_cert")]
        basis_cert,
        #[cfg(feature = "basis_cert")]
        basis_perturb,
        #[cfg(feature = "basis_cert")]
        basis_skips,
    })
}

/// The laya cell for one suite — its PRIMARY checkpoint only (typed for
/// typed_decisions, english otherwise; multilingual for the thai probes),
/// disclosed. Bucket-limited slots are named skips.
#[cfg(feature = "laya-riir")]
fn perm_laya(
    spec: &SuiteSpec,
    prepared: &Prepared,
    slots: &[(usize, usize)],
    k: usize,
) -> Result<PermLaneReport, String> {
    use crate::laya::config::Checkpoint;
    let ck_name = laya_checkpoints_for(spec.name)[0];
    let ck = match ck_name {
        "english" => Checkpoint::English,
        "multilingual" => Checkpoint::Multilingual,
        "typed" => Checkpoint::TypedDecisions,
        other => return Err(format!("unknown checkpoint {other}")),
    };
    let agent = load_laya_agent(ck_name, ck)?;
    let device = crate::laya::riir::agent::DeviceKind::from_env()
        .map(|d| format!("{d:?}"))
        .unwrap_or_else(|_| "default".to_string());
    eprintln!("    perm[laya/{ck_name}]: device {device}");
    let mut oracle = LayaOracle { agent };
    let canary = canary_case();
    let mut skips = Vec::new();
    probe_lane(
        &mut oracle,
        LaneRun {
            name: &format!("laya/{ck_name}"),
            model: &format!("laya/{ck_name}"),
            control: false,
        },
        &canary,
        slots,
        prepared,
        k,
        &mut skips,
    )
}

// ────────────────────────────── the basis-split certificate (issue 086 T1) ──

/// The T2 label-rename: every option label suffixed "_2" — a parallel,
/// deterministic, token-changing rename (a DIFFERENT presentation-change
/// family from ordering).
#[cfg(feature = "basis_cert")]
fn renamed_labels(labels: &[String]) -> Vec<String> {
    labels.iter().map(|l| format!("{l}_2")).collect()
}

/// The certificate over one suite's sampled slots: its OWN slot budget
/// (the pair-budget law needs ≥ d fit pairs — the probe's default 40-slot
/// sample is far below; see [`CERT_SLOT_CAP`]). The perm-pair GENERATORS
/// are the probe's own (`perm_orderings` + `permuted_case`, same slot
/// seeds); the certificate just samples more slots. Fit half = even-index
/// slots, holdout = odd (deterministic stride law); the whitening
/// population is the slots' identity renderings PLUS the suite's train
/// docs (stride-capped, the pure core's cap). Casing pairs ride the
/// holdout slots' identity renderings (expected Δ ≡ 0 — the tokenizer is
/// case-insensitive).
#[cfg(feature = "basis_cert")]
fn basis_cert_suite(
    prepared: &Prepared,
    k: usize,
    skips: &mut Vec<String>,
) -> Option<BasisCertLane> {
    use crate::harness::basis_cert::{
        certify_with_directions, render_presentation, CertPair, PairFamily, PresentationQuestion,
        CERT_DIM, WHITEN_POP_CAP,
    };
    // The certificate's own choice-slot sample (all choice questions of
    // each sampled case, the probe's slot shape).
    let choice_cases: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c.questions.iter().any(|q| q.kind == QKind::Choice))
        .map(|(i, _)| i)
        .collect();
    if choice_cases.is_empty() {
        skips.push("basis-cert: no choice questions in suite".to_string());
        return None;
    }
    let sampled = stride_indices(choice_cases.len(), CERT_SLOT_CAP);
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
    let embed = crate::embed::Embedder;
    let embed_of = |q: &crate::harness::suites::SuiteQuestion,
                        state: &str|
     -> Option<[f32; CERT_DIM]> {
        let obj = q.criteria.as_object()?;
        let labels: Vec<String> = obj.keys().cloned().collect();
        let descs: Vec<String> = labels
            .iter()
            .map(|l| {
                obj.get(l)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        let text = render_presentation(&PresentationQuestion {
            state,
            instructions: q.instructions.as_str(),
            labels: &labels,
            descs: &descs,
        });
        let mut v = [0.0f32; CERT_DIM];
        embed.embed_into(text.as_bytes(), &mut v);
        Some(v)
    };

    // Per-slot: identity embed + K−1 ordering Δs. Even slot → fit half,
    // odd → holdout; the cross-family pairs (casing + rename) from the
    // holdout half.
    let mut fit: Vec<CertPair> = Vec::new();
    let mut cross: Vec<CertPair> = Vec::new();
    let mut hold: Vec<CertPair> = Vec::new();
    let mut ident_pop: Vec<[f32; CERT_DIM]> = Vec::new();
    for (slot_i, &(ci, qi)) in slots.iter().enumerate() {
        let case = &prepared.suite.cases[ci];
        let q = &case.questions[qi];
        let Some(state_str) = prepared.state_strs.get(ci).map(String::as_str) else {
            skips.push(format!(
                "basis-cert {}#{}: no state string — slot dropped",
                case.id, q.qid
            ));
            continue;
        };
        let Some(identity) = embed_of(q, state_str) else {
            skips.push(format!(
                "basis-cert {}#{}: not a choice question — slot dropped",
                case.id, q.qid
            ));
            continue;
        };
        ident_pop.push(identity);
        let n = q.criteria.as_object().map(|m| m.len()).unwrap_or(0);
        let orderings = perm_orderings(n, k, slot_seed(&case.id, &q.qid));
        for (rep, order) in orderings.iter().enumerate().skip(1) {
            let pc = match permuted_case(case, qi, order, rep) {
                Ok(pc) => pc,
                Err(e) => {
                    skips.push(format!("basis-cert {}#{}: {e}", case.id, q.qid));
                    continue;
                }
            };
            let Some(variant) = embed_of(&pc.questions[0], state_str) else {
                continue;
            };
            let delta: Vec<f64> = variant
                .iter()
                .zip(identity.iter())
                .map(|(a, b)| f64::from(*a) - f64::from(*b))
                .collect();
            let pair = CertPair {
                family: PairFamily::Ordering,
                delta,
            };
            if slot_i % 2 == 0 {
                fit.push(pair);
            } else {
                hold.push(pair);
            }
        }
        // The cross-family checks on the holdout half: (a) CASING — the
        // identity options case-flipped (the tokenizer must embed them
        // identically; the pure core skips the null pairs and reports
        // honestly); (b) RENAME — every option label suffixed "_2" (a
        // DIFFERENT presentation change; low residual = the families are
        // distinct).
        if slot_i % 2 == 1 {
            let Some(obj) = q.criteria.as_object() else { continue };
            let labels: Vec<String> = obj.keys().cloned().collect();
            let descs: Vec<String> = labels
                .iter()
                .map(|l| {
                    obj.get(l)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                })
                .collect();
            // (a) casing: first char of each label case-swapped.
            let flipped_labels: Vec<String> = labels
                .iter()
                .map(|l| {
                    let mut c: Vec<char> = l.chars().collect();
                    match c.first_mut() {
                        Some(f) if f.is_lowercase() => {
                            *f = f.to_ascii_uppercase();
                        }
                        Some(f) if f.is_uppercase() => {
                            *f = f.to_ascii_lowercase();
                        }
                        _ => {}
                    }
                    c.iter().collect::<String>()
                })
                .collect();
            for (labels_v, family) in [
                (&flipped_labels, PairFamily::Casing),
                (&renamed_labels(&labels), PairFamily::Rename),
            ] {
                let text = render_presentation(&PresentationQuestion {
                    state: state_str,
                    instructions: q.instructions.as_str(),
                    labels: labels_v,
                    descs: &descs,
                });
                let mut fv = [0.0f32; CERT_DIM];
                embed.embed_into(text.as_bytes(), &mut fv);
                let delta: Vec<f64> = fv
                    .iter()
                    .zip(identity.iter())
                    .map(|(a, b)| f64::from(*a) - f64::from(*b))
                    .collect();
                cross.push(CertPair { family, delta });
            }
        }
    }

    // Whitening population: slot identity renderings + train docs
    // (deterministic stride cap).
    for &ti in stride_indices(prepared.train.len(), WHITEN_POP_CAP).iter() {
        let mut v = [0.0f32; CERT_DIM];
        embed.embed_into(prepared.train[ti].text.as_bytes(), &mut v);
        ident_pop.push(v);
    }
    if fit.is_empty() && hold.is_empty() {
        return None;
    }
    // T3's delta-scale anchor: the median raw fit-pair delta norm (the
    // amplitude a REAL ordering flip moves the embedding by).
    let mut norms: Vec<f64> = fit
        .iter()
        .map(|p| p.delta.iter().map(|v| v * v).sum::<f64>().sqrt())
        .collect();
    norms.sort_by(|a, b| a.total_cmp(b));
    let median_delta_norm = (!norms.is_empty()).then(|| {
        norms[norms.len() / 2]
    });
    let (mut rep, directions) = certify_with_directions(&fit, &hold, &cross, &ident_pop);
    rep.lane = "hashed-feature embed (presentation rendering)".to_string();
    eprintln!(
        "    basis-cert: verdict {} · k {:?} · gap {:?} · leakage {:?} · fit {} + holdout {} pairs · pop {}",
        rep.verdict,
        rep.k,
        rep.gap_ratio.map(|g| (g * 1000.0).round() / 1000.0),
        rep.pair_leakage.map(|g| (g * 1000.0).round() / 1000.0),
        rep.n_fit_pairs,
        rep.n_holdout_pairs,
        rep.n_whiten_samples,
    );
    Some(BasisCertLane {
        report: rep,
        directions,
        median_delta_norm,
    })
}

/// The certificate's own slot cap: 320 slots × (K−1) deltas = 1280 fit/hold
/// pairs at the default K — ≈5× the d=256 structural floor and ≈1.3× the
/// Research-618 ≈1k identification budget.
#[cfg(feature = "basis_cert")]
pub const CERT_SLOT_CAP: usize = 320;

// ── Issue 086 T3 — the Thm-3 perturbation certificate on the scorer ──

/// Cases sampled for the perturbation certificate (stride; the fit used
/// 320 — the perturbation arm re-SOLVES per direction × rung × sign, so
/// its budget is solve-count-shaped, not pair-count-shaped).
#[cfg(feature = "basis_cert")]
pub const PERTURB_CASE_CAP: usize = 32;

/// Directions probed per case: the TOP-eigenvalue basis rows (the split
/// sorts descending — the first rows carry the dominant variant energy).
#[cfg(feature = "basis_cert")]
pub const PERTURB_DIR_CAP: usize = 32;

/// The scored-output tolerance at the gate rung (probabilities and
/// confidence are O(1) sigmoid outputs; 1e-3 is the disclosed bar —
/// drift larger than this at fp-envelope amplitude means the scorer's
/// continuous readout moves under representation noise).
#[cfg(feature = "basis_cert")]
pub const PERTURB_PROB_TOL: f64 = 1e-3;

/// One amplitude rung's aggregate over every (case, direction, sign)
/// comparison.
#[cfg(feature = "basis_cert")]
#[derive(Debug, Clone, Serialize)]
pub struct PerturbRung {
    pub label: String,
    /// The absolute epsilon (‖y‖₂ = 1 — the L2-normalized embed's own
    /// scale — for the fp rungs; the median fit-delta norm × fraction for
    /// the delta rungs).
    pub eps: f64,
    pub comparisons: usize,
    /// Discrete-output mismatches: pick index OR abstain bit changed.
    pub flips: usize,
    pub max_prob_delta: f64,
    pub max_conf_delta: f64,
}

/// Issue 086 T3 — the Thm-3 perturbation certificate over one suite: the
/// deployed engine's discrete outputs under additive `ε·v_k` perturbation
/// of BOTH embeddings it consumes (the per-question context embedding and
/// the per-request state embedding), along the certified presentation
/// directions, at a ladder from the fp envelope upward. The GATE rung is
/// `fp_1x` (one norm-ULP — the amplitude any summation reorder could
/// produce); the higher rungs measure MARGIN, they do not gate.
#[cfg(feature = "basis_cert")]
#[derive(Debug, Clone, Serialize)]
pub struct BasisPerturbReport {
    pub n_cases: usize,
    pub n_directions: usize,
    /// The certified variant dimension (may exceed `n_directions` — the
    /// cap is disclosed, not silent).
    pub k_total: usize,
    /// Median raw fit-pair delta norm (the delta-rung anchor).
    pub median_delta_norm: f64,
    /// Cases whose options resolve by-name to domains (the route terms
    /// consume the state embedding; drafter-only cases are embedding-inert
    /// on the score side by construction — disclosed, never pooled away).
    pub n_route_active_cases: usize,
    pub rungs: Vec<PerturbRung>,
    /// `pass` | `fail_flips_at_fp` | `fail_score_drift_at_fp` |
    /// `vacuous_no_response`.
    pub verdict: String,
}

/// The Thm-3 certificate lane: perturbs along the fitted presentation
/// directions through the ENGINE's own solve path (the `solve_sample_into`
/// additive hook), comparing discrete outcomes + scored readouts against
/// the unperturbed solve. Report-only; no decide-path change.
#[cfg(feature = "basis_cert")]
fn basis_perturb_suite<const N: usize>(
    engine: &mut DecisionEngine<N, EMBED_DIM>,
    prepared: &Prepared,
    dirs: &crate::harness::basis_cert::BasisDirections,
    median_delta_norm: f64,
    skips: &mut Vec<String>,
) -> Option<BasisPerturbReport> {
    use katgpt_core::decision_wire::QuestionKind;

    let choice_cases: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c.questions.iter().any(|q| q.kind == QKind::Choice))
        .map(|(i, _)| i)
        .collect();
    if choice_cases.is_empty() {
        skips.push("basis-perturb: no choice questions in suite".to_string());
        return None;
    }
    let sampled = stride_indices(choice_cases.len(), PERTURB_CASE_CAP);
    let n_dirs = dirs.k.min(PERTURB_DIR_CAP);
    if n_dirs == 0 {
        skips.push("basis-perturb: zero certified directions".to_string());
        return None;
    }
    let mut dirs_f32: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(n_dirs);
    for row in 0..n_dirs {
        let mut a = [0.0f32; EMBED_DIM];
        for (j, v) in dirs.dirs[row].iter().enumerate() {
            a[j] = *v as f32;
        }
        dirs_f32.push(a);
    }
    // The amplitude ladder: fp rungs are multiples of f32::EPSILON (the
    // embed is L2-normalized — ‖y‖ = 1 — so EPSILON IS one norm-ULP);
    // delta rungs are fractions of the median raw fit-pair delta norm
    // (the amplitude a REAL ordering flip moves the embedding by).
    let mut rung_defs: Vec<(String, f64)> = Vec::with_capacity(7);
    for (label, c) in [
        ("fp_1x", 1.0f64),
        ("fp_16x", 16.0),
        ("fp_256x", 256.0),
    ] {
        rung_defs.push((label.to_string(), c * f64::from(f32::EPSILON)));
    }
    for (label, f) in [
        ("delta_1e-3", 1e-3f64),
        ("delta_1e-2", 1e-2),
        ("delta_1e-1", 1e-1),
        ("delta_1x", 1.0),
    ] {
        rung_defs.push((label.to_string(), f * median_delta_norm));
    }
    let mut rungs: Vec<PerturbRung> = rung_defs
        .iter()
        .map(|(l, e)| PerturbRung {
            label: l.clone(),
            eps: *e,
            comparisons: 0,
            flips: 0,
            max_prob_delta: 0.0,
            max_conf_delta: 0.0,
        })
        .collect();
    let mut sc = Scratch::<EMBED_DIM>::new();
    let mut n_route_active = 0usize;
    let domain_names: Vec<String> = engine.domain_names().iter().map(|s| s.to_string()).collect();
    for &si in sampled.iter() {
        let ci = choice_cases[si];
        let case = &prepared.suite.cases[ci];
        let Some(state_str) = prepared.state_strs.get(ci).map(String::as_str) else {
            skips.push(format!(
                "basis-perturb {}: no state string — case dropped",
                case.id
            ));
            continue;
        };
        let req = match engine_request(case, state_str) {
            Ok(r) => r,
            Err(e) => {
                skips.push(format!("basis-perturb {}: {e}", case.id));
                continue;
            }
        };
        sc.prepare(req.questions.len());
        if engine.solve_into(&req, &mut sc).is_err() {
            skips.push(format!("basis-perturb {}: base solve failed", case.id));
            continue;
        }
        let base = engine.to_response(&req, &sc);
        // Route-active classification (mirrors the engine's by-name arm:
        // a choice question whose options ARE domain names consumes the
        // state embedding through the route terms).
        let route_active = req.questions.iter().any(|q| {
            q.kind != QuestionKind::Noul
                && q.options.len() <= N
                && q.options
                    .iter()
                    .all(|o| domain_names.iter().any(|n| n == o))
        });
        if route_active {
            n_route_active += 1;
        }
        for (ri, (_, eps)) in rung_defs.iter().enumerate() {
            let eps32 = *eps as f32;
            for dir in dirs_f32.iter() {
                for sign in [1.0f32, -1.0f32] {
                    if engine
                        .solve_sample_into(&req, &mut sc, None, Some((dir, sign * eps32)))
                        .is_err()
                    {
                        skips.push(format!("basis-perturb {}: perturbed solve failed", case.id));
                        continue;
                    }
                    let pert = engine.to_response(&req, &sc);
                    let rung = &mut rungs[ri];
                    rung.comparisons += 1;
                    for (b, p) in base.answers.iter().zip(pert.answers.iter()) {
                        if b.outcome != p.outcome {
                            rung.flips += 1;
                        }
                        for (bp, pp) in b.probabilities.iter().zip(p.probabilities.iter()) {
                            let d = (f64::from(*bp) - f64::from(*pp)).abs();
                            if d > rung.max_prob_delta {
                                rung.max_prob_delta = d;
                            }
                        }
                        let dc = (f64::from(b.confidence) - f64::from(p.confidence)).abs();
                        if dc > rung.max_conf_delta {
                            rung.max_conf_delta = dc;
                        }
                    }
                }
            }
        }
    }
    // Verdict: the gate rung is fp_1x (the fp envelope).
    let gate = &rungs[0];
    let total_comparisons: usize = rungs.iter().map(|r| r.comparisons).sum();
    let all_inert = rungs
        .iter()
        .all(|r| r.flips == 0 && r.max_prob_delta == 0.0 && r.max_conf_delta == 0.0);
    let verdict = if total_comparisons == 0 {
        "vacuous_no_comparisons"
    } else if all_inert {
        // Nothing moved even at a full median-delta amplitude along every
        // direction: the embedding path is inert for this suite's posture
        // (drafter-only scoring + no gate crossing) — an honest loud
        // verdict, never a silent pass (the absent-work defense).
        "vacuous_no_response"
    } else if gate.flips > 0 {
        "fail_flips_at_fp"
    } else if gate.max_prob_delta > PERTURB_PROB_TOL || gate.max_conf_delta > PERTURB_PROB_TOL {
        "fail_score_drift_at_fp"
    } else {
        "pass"
    };
    eprintln!(
        "    basis-perturb: {} case(s) × {} dir(s) × 7 rungs · route-active {} · verdict {}",
        sampled.len(),
        n_dirs,
        n_route_active,
        verdict
    );
    Some(BasisPerturbReport {
        n_cases: sampled.len(),
        n_directions: n_dirs,
        k_total: dirs.k,
        median_delta_norm,
        n_route_active_cases: n_route_active,
        rungs,
        verdict: verdict.to_string(),
    })
}

/// One suite's probe cell: prepare, sample, run the control + every
/// enabled lane. Never panics — every failure lands in `skipped`, loud.
fn perm_suite(
    spec: &SuiteSpec,
    dir: &Path,
    popts: &PermProbeOptions,
    shared: &mut [SharedLane],
    laya_enabled: bool,
    kn_route: bool,
) -> PermSuiteReport {
    let mut lanes: Vec<PermLaneReport> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    eprintln!("═══ suite {} (perm) ═══", spec.name);
    let prepared = match prepare(spec, dir) {
        Ok(p) => p,
        Err(e) => {
            skipped.push(format!("prepare: {e}"));
            return PermSuiteReport {
                suite: spec.name.to_string(),
                lanes,
                skipped,
                slice_summary: None,
                #[cfg(feature = "basis_cert")]
                basis_cert: None,
                #[cfg(feature = "basis_cert")]
                basis_perturb: None,
            };
        }
    };
    let slice_summary = prepared
        .slices
        .as_ref()
        .map(|s| s.summary_line());
    // Choice slots: the sampled case indices × their choice questions.
    let choice_cases: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c.questions.iter().any(|q| q.kind == QKind::Choice))
        .map(|(i, _)| i)
        .collect();
    if choice_cases.is_empty() {
        skipped.push(
            "no choice questions — the probe is choice-only (score levels are \
             ordinal and noul's [false, true] order is semantic, not \
             presentational)"
                .to_string(),
        );
        return PermSuiteReport {
            suite: spec.name.to_string(),
            lanes,
            skipped,
            slice_summary,
            #[cfg(feature = "basis_cert")]
            basis_cert: None,
            #[cfg(feature = "basis_cert")]
            basis_perturb: None,
        };
    }
    let sampled = stride_indices(choice_cases.len(), popts.max_cases);
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
        "    perm: {} choice slot(s) over {} sampled case(s) · K={} orderings",
        slots.len(),
        sampled.len(),
        popts.k
    );

    // The control (modelless) — the const-generic dispatch, the run()'s
    // table. The issue-086 certificate lanes ride inside (const-generic N).
    macro_rules! dispatch {
        ($n:literal) => {
            perm_modelless::<$n>(spec, &prepared, &slots, popts, kn_route)
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
             table in perm_probe.rs"
        )),
    };
    #[cfg(feature = "basis_cert")]
    let mut basis_cert: Option<crate::harness::basis_cert::BasisCertReport> = None;
    #[cfg(feature = "basis_cert")]
    let mut basis_perturb: Option<BasisPerturbReport> = None;
    match modelless {
        Ok(outcome) => {
            lanes.push(outcome.report);
            #[cfg(feature = "basis_cert")]
            {
                basis_cert = outcome.basis_cert;
                basis_perturb = outcome.basis_perturb;
                skipped.extend(outcome.basis_skips);
            }
        }
        Err(e) => skipped.push(format!("modelless: {e}")),
    }

    // The shared comparison lanes.
    for sl in shared.iter_mut() {
        let mut lane_skips = Vec::new();
        let canary = canary_case();
        match probe_lane(
            sl.oracle.as_mut(),
            LaneRun {
                name: sl.name,
                model: &sl.model,
                control: false,
            },
            &canary,
            &slots,
            &prepared,
            popts.k,
            &mut lane_skips,
        ) {
            Ok(r) => lanes.push(r),
            Err(e) => skipped.push(format!("{}: {e}", sl.name)),
        }
        skipped.extend(lane_skips);
    }

    // The laya lane (feature-gated, primary checkpoint).
    #[cfg(feature = "laya-riir")]
    if laya_enabled {
        match perm_laya(spec, &prepared, &slots, popts.k) {
            Ok(r) => lanes.push(r),
            Err(e) => skipped.push(format!("laya: {e}")),
        }
    }
    #[cfg(not(feature = "laya-riir"))]
    let _ = laya_enabled;

    PermSuiteReport {
        suite: spec.name.to_string(),
        lanes,
        skipped,
        slice_summary,
        #[cfg(feature = "basis_cert")]
        basis_cert,
        #[cfg(feature = "basis_cert")]
        basis_perturb,
    }
}

/// Run the probe over the requested suites. Comparison lanes refuse at
/// construction when their server is unreachable (the comparison-lane
/// law); the run itself never fails on a lane red — the record carries
/// the verdicts, the BIN decides what a control red means.
pub fn run_perm_probe(
    opts: &RunOptions,
    popts: &PermProbeOptions,
) -> Result<PermProbeOutput, String> {
    let box_start = crate::harness::box_state::capture();
    let mut shared: Vec<SharedLane> = Vec::new();
    if popts.drex {
        let lane = crate::lanes::drex::DrexLane::default();
        lane.health()
            .map_err(|e| format!("--perm-probe --drex: {e}"))?;
        // Provenance from a canary warm-up round (their `model` field —
        // never a hardcoded id, the gliner law). The round doubles as
        // the lane's warm-up.
        let canary = canary_case();
        let raw = lane
            .decide_raw(&canary)
            .map_err(|e| format!("--perm-probe --drex warmup: {e}"))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("--perm-probe --drex warmup parse: {e}"))?;
        let model = crate::lanes::drex::DrexLane::model_of(&parsed);
        shared.push(SharedLane {
            name: "drex",
            model,
            oracle: Box::new(DrexOracle { lane }),
        });
    }
    if popts.d1 {
        let lane = crate::lanes::d1::D1Lane::default();
        lane.health()
            .map_err(|e| format!("--perm-probe --d1: {e}"))?;
        // Provenance from a canary warm-up round (their `model` field —
        // never a hardcoded id, the gliner law). The round doubles as
        // the lane's warm-up.
        let canary = canary_case();
        let raw = lane
            .decide_raw(&canary)
            .map_err(|e| format!("--perm-probe --d1 warmup: {e}"))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("--perm-probe --d1 warmup parse: {e}"))?;
        let model = crate::lanes::d1::D1Lane::model_of(&parsed);
        shared.push(SharedLane {
            name: "d1",
            model,
            oracle: Box::new(D1Oracle { lane }),
        });
    }
    if popts.agentjev {
        let lane = crate::lanes::agentjev::AgentJevLane::default();
        let (model, provenance) = lane
            .info()
            .map_err(|e| format!("--perm-probe --agentjev: {e}"))?;
        let canary = canary_case();
        lane.decide(&canary)
            .map_err(|e| format!("--perm-probe --agentjev warmup: {e}"))?;
        shared.push(SharedLane {
            name: "agentjev",
            model: format!("{model} ({provenance})"),
            oracle: Box::new(AgentJevOracle { lane }),
        });
    }
    if popts.jeva {
        let lane = crate::lanes::jeva::JevaLane::default();
        lane.health()
            .map_err(|e| format!("--perm-probe --jeva: {e}"))?;
        // Provenance from /health (model + resolved dtype + allow_null
        // posture); the canary round doubles as the lane's warm-up.
        let (provenance, _device) = lane
            .info()
            .map_err(|e| format!("--perm-probe --jeva: {e}"))?;
        let canary = canary_case();
        let raw = lane
            .decide_raw(&canary)
            .map_err(|e| format!("--perm-probe --jeva warmup: {e}"))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("--perm-probe --jeva warmup parse: {e}"))?;
        let model = crate::lanes::jeva::JevaLane::model_of(&parsed);
        shared.push(SharedLane {
            name: "jeva",
            model: format!("{model} ({provenance})"),
            oracle: Box::new(JevaOracle { lane }),
        });
    }
    let laya_enabled = !opts.skip_laya;
    let laya_device: Option<String> = if cfg!(feature = "laya-riir") && laya_enabled {
        #[cfg(feature = "laya-riir")]
        {
            Some(
                crate::laya::riir::agent::DeviceKind::from_env()
                    .map(|d| format!("{d:?}"))
                    .unwrap_or_else(|_| "default".to_string()),
            )
        }
        #[cfg(not(feature = "laya-riir"))]
        {
            None
        }
    } else {
        None
    };
    let mut suites = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only || !DEFAULT_PROBE_SUITES.contains(&spec.name) {
            continue;
        }
        suites.push(perm_suite(
            spec,
            &opts.datasets_dir,
            popts,
            &mut shared,
            laya_enabled,
            opts.kn_route,
        ));
    }
    if suites.is_empty() {
        return Err(format!(
            "perm probe: no suite measured ({} request line(s)) — check --suites \
             against the registry",
            opts.suites.len()
        ));
    }
    let red_cells: usize = suites
        .iter()
        .map(|s| s.lanes.iter().filter(|l| l.verdict == SpreadVerdict::Red).count())
        .sum();
    Ok(PermProbeOutput {
        meta: PermProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            k: popts.k,
            max_cases: popts.max_cases,
            seed: PROBE_SEED,
            gate_rule: format!(
                "per lane x suite: MEDIAN top-pick swing across orderings <= \
                 {GATE_MEDIAN_SWING_PT} pt; max + flip rate disclosed beside. The \
                 modelless control holds a stricter contract: flips allowed ONLY on \
                 an exact top-2 tie (the deterministic first-position tie-break) and \
                 median swing within the L1 normalizer's measured fp envelope \
                 ({} pt) — strict byte-identity stays a disclosed column (the L1 \
                 sum reorders with the presentation and moves ULPs by design). A \
                 control red refuses the run verdict.",
                crate::harness::permutation::CONTROL_MEDIAN_CEILING_PT
            ),
            sampling_rule: "the suite's choice-carrying cases in dataset order, \
                            first of every ceil(n/max_cases) stride (deterministic); \
                            ALL choice questions of each sampled case. The canary \
                            case rides every lane as the liveness cell (never in \
                            the suite medians)."
                .to_string(),
            invariance_rule: "label-space mapping: option LABELS are the invariant \
                              under permutation (indexes move); every comparison \
                              happens on labels mapped back to the original \
                              criteria order."
                .to_string(),
            route_binding: if opts.kn_route {
                "by-name + index-anchored (score, k == N, index-named \
                 domains; issue 079)"
                    .to_string()
            } else {
                "strict by-name (--no-kn-route: the index-anchored arm \
                 disabled)"
                    .to_string()
            },
            latency_note: "latency columns deliberately absent — this is a \
                           distribution-stability probe; the latency/determinism \
                           axes live in the det benches (126/127)."
                .to_string(),
            laya_device,
            box_start,
            box_end: crate::harness::box_state::capture(),
        },
        red_cells,
        suites,
    })
}

/// The verdict's markdown/JSON spelling (Debug prints "Pass"/"Red"; the
/// record and the table share the UPPERCASE serde spelling).
fn verdict_str(v: SpreadVerdict) -> &'static str {
    match v {
        SpreadVerdict::Pass => "PASS",
        SpreadVerdict::Red => "RED",
    }
}

/// The markdown rendering (the bench record's table block).
#[must_use]
pub fn render_perm_probe_markdown(out: &PermProbeOutput) -> String {
    let m = &out.meta;
    let mut s = String::new();
    s.push_str("# Perm probe — option-permutation spread (issue 077)\n\n");
    s.push_str(&format!(
        "- date {} · sha {} · host {}\n- datasets {}\n- K {} orderings × ≤ {} \
         case(s)/suite · seed {}\n- laya device: {}\n",
        m.date_utc,
        m.git_sha.as_deref().unwrap_or("unknown"),
        m.host,
        m.datasets_dir,
        m.k,
        m.max_cases,
        m.seed,
        m.laya_device.as_deref().unwrap_or("not probed"),
    ));
    for (name, rule) in [
        ("gate", &m.gate_rule),
        ("sampling", &m.sampling_rule),
        ("invariance", &m.invariance_rule),
        ("modelless route", &m.route_binding),
        ("latency", &m.latency_note),
    ] {
        s.push_str(&format!("- {name}: {rule}\n"));
    }
    s.push_str("\n| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for suite in &out.suites {
        if suite.lanes.is_empty() {
            s.push_str(&format!(
                "| {} | — | — | — | — | — | — | — | — | SKIPPED |\n",
                suite.suite
            ));
            continue;
        }
        for l in &suite.lanes {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {:.2} | {:.2} | {} | {:.3} | {} | {:?} |\n",
                suite.suite,
                l.lane,
                l.model,
                l.n_cases,
                l.median_swing_pt,
                l.max_swing_pt,
                l.flips,
                l.flip_rate,
                l.control_invariant
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "—".to_string()),
                verdict_str(l.verdict),
            ));
        }
    }
    s.push_str("\n## Canary cell (liveness — an order-biased lane flips it BY DESIGN)\n\n");
    s.push_str("| suite | lane | ref | picks | swing pt | flipped |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    for suite in &out.suites {
        for l in &suite.lanes {
            if let Some(c) = &l.canary {
                s.push_str(&format!(
                    "| {} | {} | {} | {} | {:.1} | {} |\n",
                    suite.suite,
                    l.lane,
                    c.ref_pick,
                    c.pick_labels
                        .iter()
                        .map(|p| p.as_str())
                        .collect::<Vec<&str>>()
                        .join("|"),
                    c.swing_pt,
                    c.flipped,
                ));
            }
        }
    }
    // Issue 086 T1 — the presentation-subspace certificate section (the
    // `--basis-split` modifier; absent = not requested).
    #[cfg(feature = "basis_cert")]
    if out.suites.iter().any(|s| s.basis_cert.is_some()) {
        s.push_str(
            "\n## Basis-split certificate (issue 086 T1 — presentation subspace, hashed-feature embed)\n\n",
        );
        s.push_str(
            "Fit/holdout = even/odd slots; whitening = identity renderings + train docs (stride-capped); \
             τ_gap 4.0. `unidentified_*` is a RESULT (order-insensitive embed), not a failure; \
             `pair_leakage` is a generalization residual of the fitted presentation null space, never purity.\n\n",
        );
        s.push_str("| suite | verdict | k | gap | holdout leakage | agreement residual | casing residual | rename residual | fit+hold pairs | whitening pop |\n");
        s.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
        for suite in &out.suites {
            if let Some(c) = &suite.basis_cert {
                s.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {}+{} | {} |\n",
                    suite.suite,
                    c.verdict,
                    c.k.map(|k| k.to_string()).unwrap_or_else(|| "—".into()),
                    c.gap_ratio
                        .map(|g| format!("{g:.2}"))
                        .unwrap_or_else(|| "—".into()),
                    c.pair_leakage
                        .map(|g| format!("{g:.4}"))
                        .unwrap_or_else(|| "—".into()),
                    c.agreement_residual
                        .map(|g| format!("{g:.4}"))
                        .unwrap_or_else(|| "—".into()),
                    c.casing_null_residual
                        .map(|g| format!("{g:.4}"))
                        .unwrap_or_else(|| "none (all null)".into()),
                    c.rename_residual
                        .map(|g| format!("{g:.4}"))
                        .unwrap_or_else(|| "—".into()),
                    c.n_fit_pairs,
                    c.n_holdout_pairs,
                    c.n_whiten_samples,
                ));
            }
        }
    }
    // Issue 086 T3 — the Thm-3 perturbation certificate section (the
    // `--basis-split` modifier; absent = not requested / not certified).
    #[cfg(feature = "basis_cert")]
    if out.suites.iter().any(|s| s.basis_perturb.is_some()) {
        s.push_str(
            "\n## Basis-perturb certificate (issue 086 T3 — Thm-3, the scorer under ε·v_k along the presentation basis)\n\n",
        );
        s.push_str(
            "GATE rung = `fp_1x` (one norm-ULP — ‖y‖ = 1 on the L2-normalized embed: the amplitude any \
             summation reorder could produce). Discrete outputs (pick + abstain) must be bit-identical there \
             and scored outputs within 1e-3; the higher rungs measure margin, they do not gate. \
             `vacuous_no_response` = even a full median-delta perturbation moved nothing (an inert posture, \
             honestly disclosed — never a pass).\n\n",
        );
        s.push_str("| suite | verdict | cases | dirs (of k) | route-active | median ‖Δ‖ | gate flips | gate max Δp | first-flip rung |\n");
        s.push_str("|---|---|---|---|---|---|---|---|---|\n");
        for suite in &out.suites {
            if let Some(p) = &suite.basis_perturb {
                let gate = p.rungs.first();
                let first_flip = p
                    .rungs
                    .iter()
                    .find(|r| r.flips > 0)
                    .map(|r| r.label.clone())
                    .unwrap_or_else(|| "—".into());
                s.push_str(&format!(
                    "| {} | {} | {} | {} (of {}) | {}/{} | {:.4} | {} | {} | {} |\n",
                    suite.suite,
                    p.verdict,
                    p.n_cases,
                    p.n_directions,
                    p.k_total,
                    p.n_route_active_cases,
                    p.n_cases,
                    p.median_delta_norm,
                    gate.map(|g| g.flips).unwrap_or(0),
                    gate.map(|g| format!("{:.2e}", g.max_prob_delta))
                        .unwrap_or_else(|| "—".into()),
                    first_flip,
                ));
            }
        }
    }
    let skipped_any = out.suites.iter().any(|s| !s.skipped.is_empty());
    if skipped_any {
        s.push_str("\n## Skipped / absent (loud)\n\n");
        for suite in &out.suites {
            for e in &suite.skipped {
                s.push_str(&format!("- {}: {e}\n", suite.suite));
            }
        }
    }
    s.push_str(&format!(
        "\n**RED cells: {}**{}\n",
        out.red_cells,
        if out.red_cells == 0 {
            " — every probed lane holds the median gate (or is the control holding \
             byte-identity)"
                .to_string()
        } else {
            " — disclosed above; a control red refuses the run verdict".to_string()
        },
    ));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice_case(id: &str, options: &[&str]) -> SuiteCase {
        use crate::harness::suites::{GoldAnswer, QKind, SuiteQuestion};
        let mut m = serde_json::Map::new();
        for o in options {
            m.insert((*o).to_string(), serde_json::Value::Null);
        }
        SuiteCase {
            id: id.to_string(),
            state: serde_json::Value::String("state".to_string()),
            questions: vec![SuiteQuestion {
                qid: "q".to_string(),
                kind: QKind::Choice,
                instructions: "pick".to_string(),
                criteria: serde_json::Value::Object(m),
            }],
            gold: vec![GoldAnswer {
                idx: 0,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    /// A content-faithful oracle: p 0.9 on the FIRST option's label
    /// wherever it sits — invariant by construction, so the control
    /// machinery must read invariant=true and a zero spread.
    struct ContentOracle {
        pick_label: String,
    }
    impl ChoiceOracle for ContentOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            let keys: Vec<String> = case.questions[0]
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            let pick = keys
                .iter()
                .position(|k| *k == self.pick_label)
                .expect("label present");
            let mut probs = vec![0.025; keys.len()];
            probs[pick] = 0.9;
            Ok(OrderingAnswer { probs, pick })
        }
    }

    /// A position reader: always the first presented option — the
    /// order-biased class. The control machinery must read
    /// invariant=false and the spread must flip.
    struct PositionOracle;
    impl ChoiceOracle for PositionOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            let n = case.questions[0]
                .criteria
                .as_object()
                .map(serde_json::Map::len)
                .unwrap_or(0);
            let mut probs = vec![0.0; n];
            probs[0] = 1.0;
            Ok(OrderingAnswer { probs, pick: 0 })
        }
    }

    #[test]
    fn stride_indices_covers_exact_under_and_over() {
        assert_eq!(stride_indices(0, 40), Vec::<usize>::new());
        assert_eq!(stride_indices(5, 40), vec![0, 1, 2, 3, 4]);
        // stride = ceil(100/40) = 3 → 0,3,…,99 = ceil(100/3) = 34 samples
        // (an upper bound of 40, a deterministic stride over the range).
        let idx = stride_indices(100, 40);
        assert_eq!(idx.len(), 34);
        assert_eq!(idx[0], 0);
        assert_eq!(*idx.last().expect("nonempty"), 99);
        for w in idx.windows(2) {
            assert_eq!(w[1] - w[0], 3);
        }
        // Never out of range, never duplicates.
        assert!(idx.iter().all(|&i| i < 100));
        let mut sorted = idx.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), idx.len());
    }

    #[test]
    fn answer_slot_reads_invariance_for_the_control() {
        let canary = canary_case();
        let mut skips = Vec::new();
        let (spread, inv) = answer_slot(
            &mut ContentOracle {
                pick_label: "gamma".to_string(),
            },
            &canary,
            0,
            5,
            true,
            &mut skips,
        )
        .expect("content oracle answers")
        .expect("no bucket skip for the canary");
        assert!(inv, "a content-faithful lane is byte-invariant");
        assert!(!spread.flipped);
        assert!(skips.is_empty());

        let mut skips2 = Vec::new();
        let (spread2, inv2) = answer_slot(
            &mut PositionOracle,
            &canary,
            0,
            5,
            true,
            &mut skips2,
        )
        .expect("position oracle answers")
        .expect("no bucket skip for the canary");
        assert!(!inv2, "a position reader must FAIL the control's assert");
        assert!(spread2.flipped);
    }

    #[test]
    fn probe_lane_reds_the_control_on_untied_flips() {
        let canary = canary_case();
        let case = choice_case("c1", &["a", "b", "c"]);
        let prepared = Prepared {
            suite: crate::harness::suites::Suite {
                name: "t",
                cases: vec![case],
                option_counts_note: "n",
            },
            train: Vec::new(),
            state_strs: Vec::new(),
            cal_cases: Vec::new(),
            cal_state_strs: Vec::new(),
            labels: Vec::new(),
            pool_rows: serde_json::Value::Null,
            slices: None,
        };
        let slots = vec![(0usize, 0usize)];
        let mut skips = Vec::new();
        let r = probe_lane(
            &mut PositionOracle,
            LaneRun {
                name: "mock-pos",
                model: "mock",
                control: true,
            },
            &canary,
            &slots,
            &prepared,
            5,
            &mut skips,
        )
        .expect("lane probes");
        assert_eq!(r.control_invariant, Some(false));
        assert_eq!(r.verdict, SpreadVerdict::Red, "untied position flips RED");
        assert_eq!(r.n_cases, 1);
        assert!(r.canary.as_ref().expect("canary row").flipped);
    }

    /// Ties + byte-jitter on a NON-TOP option: the top-2 tie exactly and
    /// the pick moves by the first-position tie-break, while a low option's
    /// value carries the ordering rep (byte-identity BROKEN). The control
    /// must PASS — tie-break flips are exempt, the jitter is inside the
    /// disclosed column, never the verdict.
    struct TiedJitterOracle;
    impl ChoiceOracle for TiedJitterOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            // The rep rides the case id suffix (permN).
            let rep: usize = case
                .id
                .rsplit("perm")
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let n = case.questions[0]
                .criteria
                .as_object()
                .map(serde_json::Map::len)
                .unwrap_or(0);
            let mut probs = vec![0.0; n];
            probs[0] = 1.0;
            if n > 2 {
                probs[2] = 0.5 + rep as f64 * 1e-9; // byte jitter, never top-2
            }
            probs[1] = 1.0; // exact top-2 tie with position 0
            Ok(OrderingAnswer { probs, pick: 0 })
        }
    }

    #[test]
    fn the_control_exempts_tie_break_flips_and_discloses_bytes() {
        let canary = canary_case();
        let case = choice_case("c1", &["a", "b", "c"]);
        let prepared = Prepared {
            suite: crate::harness::suites::Suite {
                name: "t",
                cases: vec![case],
                option_counts_note: "n",
            },
            train: Vec::new(),
            state_strs: Vec::new(),
            cal_cases: Vec::new(),
            cal_state_strs: Vec::new(),
            labels: Vec::new(),
            pool_rows: serde_json::Value::Null,
            slices: None,
        };
        let slots = vec![(0usize, 0usize)];
        let mut skips = Vec::new();
        let r = probe_lane(
            &mut TiedJitterOracle,
            LaneRun {
                name: "mock-tied",
                model: "mock",
                control: true,
            },
            &canary,
            &slots,
            &prepared,
            5,
            &mut skips,
        )
        .expect("lane probes");
        assert_eq!(r.control_invariant, Some(false), "bytes jitter by rep");
        assert_eq!(r.verdict, SpreadVerdict::Pass, "flips are exact ties");
        assert!(r.canary.as_ref().expect("canary row").flipped);
    }

    #[test]
    fn probe_lane_passes_a_content_faithful_control() {
        let canary = canary_case();
        // "gamma" sits in BOTH the canary's option set and this case's —
        // probe_lane answers the canary first with the same oracle.
        let case = choice_case("c1", &["alpha", "beta", "gamma"]);
        let prepared = Prepared {
            suite: crate::harness::suites::Suite {
                name: "t",
                cases: vec![case],
                option_counts_note: "n",
            },
            train: Vec::new(),
            state_strs: Vec::new(),
            cal_cases: Vec::new(),
            cal_state_strs: Vec::new(),
            labels: Vec::new(),
            pool_rows: serde_json::Value::Null,
            slices: None,
        };
        let slots = vec![(0usize, 0usize)];
        let mut skips = Vec::new();
        let r = probe_lane(
            &mut ContentOracle {
                pick_label: "gamma".to_string(),
            },
            LaneRun {
                name: "mock-content",
                model: "mock",
                control: true,
            },
            &canary,
            &slots,
            &prepared,
            5,
            &mut skips,
        )
        .expect("lane probes");
        assert_eq!(r.control_invariant, Some(true));
        assert_eq!(r.verdict, SpreadVerdict::Pass);
    }

    #[test]
    fn the_render_carries_the_verdicts_and_the_loud_absences() {
        let out = PermProbeOutput {
            meta: PermProbeMeta {
                date_utc: "2026-10-08T00:00:00Z".to_string(),
                git_sha: Some("abc123".to_string()),
                host: "test-host".to_string(),
                datasets_dir: ".raw/datasets".to_string(),
                k: 5,
                max_cases: 40,
                seed: 1,
                gate_rule: "gate rule".to_string(),
                sampling_rule: "sampling rule".to_string(),
                invariance_rule: "invariance rule".to_string(),
                route_binding: "legacy k==N index alignment (incumbent)".to_string(),
                latency_note: "latency note".to_string(),
                laya_device: None,
                box_start: crate::harness::box_state::capture(),
                box_end: crate::harness::box_state::capture(),
            },
            red_cells: 1,
            suites: vec![PermSuiteReport {
                suite: "s1".to_string(),
                lanes: vec![PermLaneReport {
                    lane: "modelless".to_string(),
                    model: "modelless".to_string(),
                    control: true,
                    n_cases: 3,
                    n_orderings: 5,
                    median_swing_pt: 0.0,
                    max_swing_pt: 0.0,
                    flips: 0,
                    flip_rate: 0.0,
                    control_invariant: Some(true),
                    verdict: SpreadVerdict::Pass,
                    canary: Some(CaseRow {
                        case_id: "canary".to_string(),
                        qid: "canary".to_string(),
                        n_options: 5,
                        ref_pick: "gamma".to_string(),
                        pick_labels: vec!["gamma".to_string()],
                        p_ref: vec![1.0],
                        swing_pt: 0.0,
                        flipped: false,
                        tie: false,
                        gold: Some("gamma".to_string()),
                        margin_top2: Some(1.0),
                    }),
                    cases: Vec::new(),
                }],
                skipped: vec!["laya: feature off".to_string()],
                slice_summary: Some("slice line".to_string()),
                #[cfg(feature = "basis_cert")]
                basis_cert: None,
                #[cfg(feature = "basis_cert")]
                basis_perturb: None,
            }],
        };
        let md = render_perm_probe_markdown(&out);
        assert!(md.contains("| s1 | modelless |"));
        assert!(md.contains("PASS"));
        assert!(md.contains("RED cells: 1"));
        assert!(md.contains("laya: feature off"));
        assert!(md.contains("control byte-identity") || md.contains("gate rule"));
    }

    #[test]
    fn slot_seed_distributes_over_slots() {
        // Different slots must (with overwhelming likelihood) see different
        // shuffles; identical slots must be identical.
        assert_eq!(slot_seed("a", "q"), slot_seed("a", "q"));
        let distinct = (0..8)
            .map(|i| slot_seed(&format!("case-{i}"), "q"))
            .collect::<Vec<u64>>();
        let mut sorted = distinct.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 8, "seeds must spread across slots");
    }

    #[test]
    fn case_row_of_maps_the_spread() {
        let s = CaseSpread {
            case_id: "c".to_string(),
            qid: "q".to_string(),
            n_options: 4,
            ref_pick: "b".to_string(),
            pick_labels: vec!["b".to_string(), "a".to_string()],
            p_ref: vec![0.8, 0.3],
            p_gold: None,
            swing: 0.5,
            flipped: true,
            tie: false,
            gold: Some("a".to_string()),
            margin_top2: Some(0.5),
        };
        let r = CaseRow::of(&s);
        assert_eq!(r.swing_pt, 50.0);
        assert!(r.flipped);
        assert!(!r.tie);
        assert_eq!(r.p_ref, vec![0.8, 0.3]);
        assert_eq!(r.ref_pick, "b");
        // Issue 089 T1 — the join fields carry through.
        assert_eq!(r.gold.as_deref(), Some("a"));
        assert_eq!(r.margin_top2, Some(0.5));
    }

    // The JSON record must serialize — the bench pipeline reads it.
    #[test]
    fn output_serializes() {
        let out = PermProbeOutput {
            meta: PermProbeMeta {
                date_utc: "d".to_string(),
                git_sha: None,
                host: "h".to_string(),
                datasets_dir: "dd".to_string(),
                k: 5,
                max_cases: 40,
                seed: 0,
                gate_rule: "g".to_string(),
                sampling_rule: "s".to_string(),
                invariance_rule: "i".to_string(),
                route_binding: "legacy k==N index alignment (incumbent)".to_string(),
                latency_note: "l".to_string(),
                laya_device: None,
                box_start: crate::harness::box_state::capture(),
                box_end: crate::harness::box_state::capture(),
            },
            red_cells: 0,
            suites: Vec::new(),
        };
        let v = serde_json::to_string(&out).expect("serialize");
        assert!(v.contains("\"red_cells\":0"));
    }

    // Guard the lane-construction refuse-loud law at the type level: the
    // shared-lane name/model/oracle triple is what the suite loop walks.
    // json macro not used here; keep serde_json paths explicit.
    #[test]
    fn shared_lane_oracle_dispatches_through_the_trait() {
        let mut sl = SharedLane {
            name: "mock",
            model: "m".to_string(),
            oracle: Box::new(ContentOracle {
                pick_label: "alpha".to_string(),
            }),
        };
        let canary = canary_case();
        let a = sl.oracle.answer(&canary).expect("answers");
        assert_eq!(a.pick, 0, "alpha is the FIRST option of the canary");
        assert!((a.probs[0] - 0.9).abs() < 1e-12);
    }

    // The default suite set stays a registry subset — a typo'd name would
    // silently probe nothing (the empty-run refusal is the runtime backstop,
    // this is the compile-time signal).
    #[test]
    fn default_probe_suites_are_registered() {
        for name in DEFAULT_PROBE_SUITES {
            assert!(
                SUITES.iter().any(|s| s.name == *name),
                "default probe suite {name} is not in the registry"
            );
        }
    }
}
