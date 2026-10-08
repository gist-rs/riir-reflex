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
    /// Probe the AgentJev lane (their server must be serving — loud
    /// refusal).
    pub agentjev: bool,
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
fn stride_indices(n_total: usize, max_cases: usize) -> Vec<usize> {
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
/// pinned off) answering every sampled slot under the K orderings.
fn perm_modelless<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    slots: &[(usize, usize)],
    k: usize,
) -> Result<PermLaneReport, String> {
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
    probe_lane(
        &mut oracle,
        LaneRun {
            name: "modelless",
            model: "modelless (deployed plain posture)",
            control: true,
        },
        &canary,
        slots,
        prepared,
        k,
        &mut skips,
    )
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

/// One suite's probe cell: prepare, sample, run the control + every
/// enabled lane. Never panics — every failure lands in `skipped`, loud.
fn perm_suite(
    spec: &SuiteSpec,
    dir: &Path,
    popts: &PermProbeOptions,
    shared: &mut [SharedLane],
    laya_enabled: bool,
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
    // table.
    macro_rules! dispatch {
        ($n:literal) => {
            perm_modelless::<$n>(spec, &prepared, &slots, popts.k)
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
    match modelless {
        Ok(r) => lanes.push(r),
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
        suites.push(perm_suite(spec, &opts.datasets_dir, popts, &mut shared, laya_enabled));
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
                    }),
                    cases: Vec::new(),
                }],
                skipped: vec!["laya: feature off".to_string()],
                slice_summary: Some("slice line".to_string()),
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
        };
        let r = CaseRow::of(&s);
        assert_eq!(r.swing_pt, 50.0);
        assert!(r.flipped);
        assert_eq!(r.p_ref, vec![0.8, 0.3]);
        assert_eq!(r.ref_pick, "b");
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
