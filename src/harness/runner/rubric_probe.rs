//! Issue 090 C1 — the rubric-covariance + route-agreement probe (the
//! runner half).
//!
//! Modelless CONTROL only (the comparison servers are down at this window;
//! a comparison lane joins by answering the same case-shaped decide — the
//! null-probe's oracle law). The pure math lives in
//! [`crate::harness::rubric`]; this module owns the lane, the posture, the
//! wire rendering, the output record, and the render.
//!
//! Two legs:
//!
//! - **Identities (I1–I4)** at THREE surfaces, each SELF-consistent:
//!   `engine-raw` (the f32 `probabilities` widened, the engine pick),
//!   `engine-norm` (the exact f64 L1 the serve edge derives from the same
//!   answer, the engine pick), and `wire-round6` (a full
//!   translate→decide→map_answers pass through the serve edge's own path —
//!   its own request bytes, its own shipped pick). I1/I2/I3 red the
//!   CONTROL (a red refuses the run verdict — an engine or wire defect,
//!   never a quiet pass); I4 is measured against the ±1e-3 wire budget.
//! - **Route agreement (C1's second clause)** — the engine's CONTEXT route
//!   (`sc.domains[qi]`) vs the STATE-ALONE route and each OPTION's
//!   semantic route (`engine.semantic_route`, the same `pick_domain` argmax
//!   over the text embedded alone), plus the by-name resolution column.
//!   Characterization only: agreement fractions, no pre-registered bar
//!   (the issue asks for the agreement, not a gate).
//!
//! Measurement posture: the deployed engine build EXACTLY as the null
//! probe builds it (`fit_posture_inner` + `build_engine_with`, every
//! selection lever pinned off, the cal-slice `observe` calibration) — the
//! probe measures what ships.

use super::perm_probe::stride_indices;
use super::*;
use crate::harness::rubric::{
    RouteAgreement, SurfaceIdentities, WIRE_SUM_BUDGET, identity_residuals,
};

/// Default suites: the choice six (the null-probe set) + the two score
/// suites (sst5's 5-level rubric; s1mb_score) — C1 is the SCORE-primitive
/// spec, so a score-kind population is mandatory, and the choice suites
/// pin the same identities where k != N.
pub const DEFAULT_RUBRIC_SUITES: &[&str] = &[
    "typed_decisions",
    "ag_news",
    "emotion",
    "xnli_en",
    "banking77",
    "massive_intent_en",
    "sst5",
    "s1mb_score",
];

/// Default sampled cases per suite (the null-probe's 40; ONE decide per
/// CASE here, covering every choice/score question inside it).
pub const DEFAULT_RUBRIC_MAX_CASES: usize = 40;

/// The probe's knobs.
#[derive(Debug, Clone)]
pub struct RubricProbeOptions {
    pub max_cases: usize,
}

/// The record for one suite.
#[derive(Debug, Clone, Serialize)]
pub struct RubricSuiteReport {
    pub suite: String,
    /// Sampled case count.
    pub n_cases: usize,
    /// Answered choice/score question slots probed.
    pub n_slots: usize,
    /// Identity readout per surface (max residual over slots + the argmax
    /// disagreement count).
    pub identities: Vec<SurfaceIdentities>,
    /// Per-slot route-agreement records.
    pub routes: Vec<RouteAgreement>,
    pub skipped: Vec<String>,
    /// CONTROL verdict: red on any structural identity failure.
    pub verdict: RubricVerdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RubricVerdict {
    Pass,
    Red,
}

impl RubricVerdict {
    fn str_of(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Red => "RED",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RubricProbeMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub max_cases: usize,
    pub value_rule: String,
    pub gate_rules: String,
    pub box_start: crate::harness::box_state::BoxState,
    pub box_end: crate::harness::box_state::BoxState,
}

#[derive(Debug, Clone, Serialize)]
pub struct RubricProbeOutput {
    pub meta: RubricProbeMeta,
    pub red_suites: usize,
    pub suites: Vec<RubricSuiteReport>,
}

// ────────────────────────────────────────────── the per-suite cell ──

/// The modelless cell for one suite: the deployed engine build (the null
/// probe's posture, verbatim), answering every probed case at the engine
/// surface and re-rendering each case through the serve edge's own wire
/// path for the consumer surface.
#[allow(clippy::too_many_lines)]
fn rubric_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    ropts: &RubricProbeOptions,
) -> Result<RubricSuiteReport, String> {
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        gate_fit_selection: false,
        gate_distance_only: false,
        gate_fit_calibrated: true,
        gate_rho: None,
        kn_route: false,
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
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };
    let (mut engine, fallbacks) = build_engine_with::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        cfg,
        &[],
    )?;
    if !fallbacks.is_empty() {
        eprintln!(
            "  [issue-039 corpus guard] {}: {} option label(s) self-doc fallback",
            spec.name,
            fallbacks.len()
        );
    }
    // The deployed calibration (the null probe's pass, verbatim — without
    // it the score gate abstains everywhere and every slot would skip).
    {
        let mut raw_scratch = Scratch::<EMBED_DIM>::new();
        raw_scratch.prepare(1);
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
                let correct = cc.gold.get(qi).map(|g| pick == g.idx).unwrap_or(false);
                pairs.push((ans.confidence, correct));
            }
        }
        for (conf, correct) in pairs {
            engine.observe(conf, correct);
        }
    }

    // ── slot sampling: every case carrying a choice/score question ──
    let probed: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.questions
                .iter()
                .any(|q| matches!(q.kind, QKind::Choice | QKind::Score))
        })
        .map(|(i, _)| i)
        .collect();
    if probed.is_empty() {
        return Err(format!("{}: no choice/score questions", spec.name));
    }
    let sampled = stride_indices(probed.len(), ropts.max_cases);

    let domain_names: Vec<String> = engine
        .domain_names()
        .into_iter()
        .map(String::from)
        .collect();
    let mut sc = Scratch::<EMBED_DIM>::new();

    // ── the three-surface identity + route pass ──
    let mut surfaces: [SurfaceIdentities; 3] = [
        new_surface("engine-raw"),
        new_surface("engine-norm"),
        new_surface("wire-round6"),
    ];
    let mut routes: Vec<RouteAgreement> = Vec::new();
    let mut skips: Vec<String> = Vec::new();
    let mut n_slots = 0usize;
    let mut n_abstained = 0usize;
    let mut n_noul = 0usize;
    let mut wire_renders = 0usize;

    for &si in &sampled {
        let ci = probed[si];
        let case = &prepared.suite.cases[ci];
        let state_str = &prepared.state_strs[ci];

        // ── Surface 1/2: the harness-path decide (engine raw + norm) ──
        let req = engine_request(case, state_str)?;
        sc.prepare(case.questions.len());
        let resp = engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("engine decide ({}): {e}", case.id))?;
        if resp.answers.len() != case.questions.len() || sc.domains.len() != case.questions.len() {
            return Err(format!(
                "engine decide ({}): {}/{} answers, {}/{} routes",
                case.id,
                resp.answers.len(),
                case.questions.len(),
                sc.domains.len(),
                case.questions.len()
            ));
        }

        // The wire questions for this case's choice/score slots (the
        // serve edge's own shapes: choice criteria {label: description},
        // score criteria [levels]).
        let mut wire_questions = serde_json::Map::new();
        let mut probed_qi: Vec<usize> = Vec::new();
        for (qi, (q, ans)) in case.questions.iter().zip(resp.answers.iter()).enumerate() {
            match q.kind {
                QKind::Noul => {
                    n_noul += 1;
                    continue;
                }
                QKind::Choice | QKind::Score => {}
            }
            let outcome_pick = match (&ans.outcome, q.kind) {
                (Some(katgpt_core::decision_wire::Outcome::Choice { index }), QKind::Choice) => {
                    *index as usize
                }
                (Some(katgpt_core::decision_wire::Outcome::Score { level }), QKind::Score) => {
                    *level as usize
                }
                _ => usize::MAX,
            };
            if outcome_pick == usize::MAX || ans.probabilities.is_empty() {
                n_abstained += 1;
                continue;
            }
            n_slots += 1;
            probed_qi.push(qi);

            // Surface 1: engine raw (f32 widened, engine pick).
            let q_raw: Vec<f64> = ans.probabilities.iter().map(|p| f64::from(*p)).collect();
            fold_surface(&mut surfaces[0], &q_raw, outcome_pick);
            // Surface 2: the exact f64 L1 the serve edge derives from the
            // same answer (systemone::wire_distribution's math).
            let sum: f64 = q_raw.iter().sum();
            let q_norm: Vec<f64> = if sum > 0.0 && sum.is_finite() {
                q_raw.iter().map(|p| p / sum).collect()
            } else {
                vec![1.0 / q_raw.len() as f64; q_raw.len()]
            };
            fold_surface(&mut surfaces[1], &q_norm, outcome_pick);

            // The wire question, in the serve edge's own shape.
            let wq = match q.kind {
                QKind::Choice => serde_json::json!({
                    "type": "choice",
                    "instructions": q.instructions,
                    "criteria": q.criteria,
                }),
                QKind::Score => serde_json::json!({
                    "type": "score",
                    "instructions": q.instructions,
                    "criteria": q.criteria,
                }),
                QKind::Noul => unreachable!("filtered above"),
            };
            wire_questions.insert(q.qid.clone(), wq);

            // ── Route agreement (characterization) ──
            let context_route = sc.domains[qi];
            let options: Vec<String> = match q.kind {
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
                            .map(|v| v.as_str().unwrap_or_default().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                QKind::Noul => unreachable!("filtered above"),
            };
            let state_alone_route = engine.semantic_route(state_str);
            let option_routes: Vec<usize> =
                options.iter().map(|o| engine.semantic_route(o)).collect();
            let by_name: Vec<Option<usize>> = options
                .iter()
                .map(|o| domain_names.iter().position(|d| d == o))
                .collect();
            let option_context_agree = option_routes
                .iter()
                .filter(|&&r| r == context_route)
                .count();
            let name_semantic_agree = by_name
                .iter()
                .zip(option_routes.iter())
                .filter(|(n, r)| n.map(|nn| nn == **r).unwrap_or(false))
                .count();
            routes.push(RouteAgreement {
                context_route,
                state_route: state_alone_route,
                option_routes,
                by_name,
                option_context_agree,
                name_semantic_agree,
            });
        }

        // ── Surface 3: the WIRE pass (translate → decide → map_answers —
        // the serve edge's own path, its own request bytes, its own
        // shipped pick). Self-consistent, never cross-compared. ──
        if !wire_questions.is_empty() {
            let built = crate::systemone::translate(&case.state, &wire_questions)
                .map_err(|e| format!("wire translate ({}): {e}", case.id))?;
            let wresp = engine
                .decide(&built.request)
                .map_err(|e| format!("wire decide ({}): {e}", case.id))?;
            let wire = crate::systemone::map_answers(&built, &wresp);
            wire_renders += 1;
            for qi in &probed_qi {
                let q = &case.questions[*qi];
                let Some(serde_json::Value::Object(a)) = wire.get(&q.qid) else {
                    return Err(format!("wire answer missing ({}:{})", case.id, q.qid));
                };
                let Some(dist_map) = a
                    .get("probabilities")
                    .and_then(serde_json::Value::as_object)
                else {
                    return Err(format!(
                        "wire probabilities missing ({}:{})",
                        case.id, q.qid
                    ));
                };
                // The wire distribution in OPTION ORDER (choice: keyed by
                // the criteria keys; score: keyed "0".."k-1").
                let (wire_q, wire_pick): (Vec<f64>, usize) = match q.kind {
                    QKind::Choice => {
                        let keys: Vec<String> = q
                            .criteria
                            .as_object()
                            .map(|m| m.keys().cloned().collect())
                            .unwrap_or_default();
                        let probs: Vec<f64> = keys
                            .iter()
                            .map(|k| {
                                dist_map
                                    .get(k)
                                    .and_then(serde_json::Value::as_f64)
                                    .ok_or_else(|| format!("wire prob key {k} missing"))
                            })
                            .collect::<Result<_, _>>()?;
                        // The shipped pick is the rendered option TEXT —
                        // resolve it back to its index (the consumer's own
                        // join).
                        let pick_text = a
                            .get("choice")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| {
                                format!("wire choice missing ({}:{})", case.id, q.qid)
                            })?;
                        let pick = keys.iter().position(|k| k == pick_text).ok_or_else(|| {
                            format!(
                                "wire choice {pick_text:?} not an option ({}:{})",
                                case.id, q.qid
                            )
                        })?;
                        (probs, pick)
                    }
                    QKind::Score => {
                        let k = q.criteria.as_array().map_or(0, Vec::len);
                        let probs: Vec<f64> = (0..k)
                            .map(|i| {
                                dist_map
                                    .get(&i.to_string())
                                    .and_then(serde_json::Value::as_f64)
                                    .ok_or_else(|| format!("wire prob level {i} missing"))
                            })
                            .collect::<Result<_, _>>()?;
                        let pick = a
                            .get("score")
                            .and_then(serde_json::Value::as_u64)
                            .ok_or_else(|| format!("wire score missing ({}:{})", case.id, q.qid))?
                            as usize;
                        (probs, pick)
                    }
                    QKind::Noul => unreachable!("filtered above"),
                };
                fold_surface(&mut surfaces[2], &wire_q, wire_pick);
            }
        }
    }

    if n_slots == 0 {
        return Err(format!(
            "{}: no answered choice/score slots — the identities need distributions",
            spec.name
        ));
    }
    if n_abstained > 0 {
        skips.push(format!(
            "{n_abstained} abstained slot(s) skipped (no distribution)"
        ));
    }
    if n_noul > 0 {
        skips.push(format!("{n_noul} noul question(s) out of C1 scope"));
    }

    let verdict = if surfaces.iter().all(SurfaceIdentities::structural_ok)
        && surfaces.iter().all(|s| s.sum_residual <= WIRE_SUM_BUDGET)
    {
        RubricVerdict::Pass
    } else {
        RubricVerdict::Red
    };
    let max_scale = surfaces
        .iter()
        .map(|s| s.scale_residual)
        .fold(0.0f64, f64::max);
    let max_shift = surfaces
        .iter()
        .map(|s| s.shift_residual)
        .fold(0.0f64, f64::max);
    eprintln!(
        "    rubric[modelless]: {} slot(s) · wire renders {} · scale {max_scale:.1e} · shift \
         {max_shift:.1e} · argmax Δ {}/{}/{} · Σq {}/{}/{} · {}",
        n_slots,
        wire_renders,
        surfaces[0].argmax_disagreements,
        surfaces[1].argmax_disagreements,
        surfaces[2].argmax_disagreements,
        fmt_e(surfaces[0].sum_residual),
        fmt_e(surfaces[1].sum_residual),
        fmt_e(surfaces[2].sum_residual),
        verdict.str_of(),
    );
    Ok(RubricSuiteReport {
        suite: spec.name.to_string(),
        n_cases: sampled.len(),
        n_slots,
        identities: surfaces.into(),
        routes,
        skipped: skips,
        verdict,
    })
}

fn new_surface(surface: &'static str) -> SurfaceIdentities {
    SurfaceIdentities {
        surface,
        n: 0,
        scale_residual: 0.0,
        shift_residual: 0.0,
        sum_residual: 0.0,
        argmax_disagreements: 0,
    }
}

/// Fold one slot's identity record into a surface accumulator (max
/// residuals, summed disagreement count).
fn fold_surface(acc: &mut SurfaceIdentities, q: &[f64], pick: usize) {
    let r = identity_residuals(acc.surface, q, pick);
    acc.n += 1;
    acc.scale_residual = acc.scale_residual.max(r.scale_residual);
    acc.shift_residual = acc.shift_residual.max(r.shift_residual);
    acc.sum_residual = acc.sum_residual.max(r.sum_residual);
    acc.argmax_disagreements += r.argmax_disagreements;
}

fn fmt_e(v: f64) -> String {
    format!("{v:.1e}")
}

// ────────────────────────────────────────────── the N dispatch ──

macro_rules! with_engine_width {
    ($n:literal, $spec:expr, $prepared:expr, $ropts:expr) => {
        rubric_suite::<$n>($spec, $prepared, $ropts)
    };
}

/// Run the probe over the selected suites (default: the choice six + the
/// two score suites).
pub fn run_rubric_probe(
    opts: &RunOptions,
    ropts: &RubricProbeOptions,
) -> Result<RubricProbeOutput, String> {
    let box_start = crate::harness::box_state::capture();
    let mut suites = Vec::new();
    let mut prep_skipped: Vec<(String, String)> = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only || !DEFAULT_RUBRIC_SUITES.contains(&spec.name) {
            continue;
        }
        match prepare(spec, &opts.datasets_dir) {
            Ok(p) => {
                eprintln!("═══ suite {} (rubric) ═══", spec.name);
                let rep = match p.labels.len() {
                    2 => with_engine_width!(2, spec, &p, ropts),
                    3 => with_engine_width!(3, spec, &p, ropts),
                    4 => with_engine_width!(4, spec, &p, ropts),
                    5 => with_engine_width!(5, spec, &p, ropts),
                    6 => with_engine_width!(6, spec, &p, ropts),
                    7 => with_engine_width!(7, spec, &p, ropts),
                    8 => with_engine_width!(8, spec, &p, ropts),
                    10 => with_engine_width!(10, spec, &p, ropts),
                    15 => with_engine_width!(15, spec, &p, ropts),
                    59 => with_engine_width!(59, spec, &p, ropts),
                    77 => with_engine_width!(77, spec, &p, ropts),
                    842 => with_engine_width!(842, spec, &p, ropts),
                    n => Err(format!(
                        "no engine instantiation for {n} domains — extend the dispatch \
                         table in rubric_probe.rs"
                    )),
                };
                match rep {
                    Ok(r) => suites.push(r),
                    Err(e) => prep_skipped.push((spec.name.to_string(), e)),
                }
            }
            Err(e) => prep_skipped.push((spec.name.to_string(), format!("prepare: {e}"))),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "rubric probe: no suite produced a cell ({} request line(s)) — check --suites",
            opts.suites.len()
        ));
    }
    let red_suites = suites
        .iter()
        .filter(|s| s.verdict == RubricVerdict::Red)
        .count();
    let box_end = crate::harness::box_state::capture();
    let out = RubricProbeOutput {
        meta: RubricProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            max_cases: ropts.max_cases,
            value_rule: "families: v_i=(i+1) (ramp) and v_i=frac((i+1)·φ)·100 (golden-ratio \
                         scatter); I1: S(10v)=10S(v); I2: S(v+c·1)=S(v)+c·Σq with c=7; \
                         I3: shipped pick == argmax(surface distribution, ties→first); \
                         I4: |Σq−1| vs the ±1e-3 wire budget (measured, disclosed)."
                .to_string(),
            gate_rules: "CONTROL gates (a red refuses the run): I1/I2 residual ≤ 1e-9 at every \
                         surface; I3 ZERO argmax disagreements at every surface (engine-raw, \
                         engine-norm, wire-round6 — each SELF-consistent, never \
                         cross-compared); I4 ≤ 1e-3. Route agreement is characterization (no \
                         bar)."
                .to_string(),
            box_start,
            box_end,
        },
        red_suites,
        suites,
    };
    for (name, why) in prep_skipped {
        eprintln!("    rubric: skip [{name}]: {why}");
    }
    Ok(out)
}

/// The markdown render (the bench record's companion view).
#[allow(clippy::too_many_lines)]
pub fn render_rubric_markdown(out: &RubricProbeOutput) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# Rubric-covariance + route-agreement probe (Issue 090 C1)\n\n{} host {} · {} · \
         datasets `{}` · ≤{} case(s)/suite\n",
        out.meta.date_utc,
        out.meta.host,
        out.meta.git_sha.as_deref().unwrap_or("git: unknown"),
        out.meta.datasets_dir,
        out.meta.max_cases
    );
    let _ = writeln!(
        s,
        "| suite | slots | I1 scale (raw/norm/wire) | I2 shift | I3 argmax Δ | Σq | verdict |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|");
    for su in &out.suites {
        let f = |i: usize, f: fn(&SurfaceIdentities) -> String| {
            su.identities
                .get(i)
                .map(f)
                .unwrap_or_else(|| "n/a".to_string())
        };
        let scale = |x: &SurfaceIdentities| format!("{:.1e}", x.scale_residual);
        let shift = |x: &SurfaceIdentities| format!("{:.1e}", x.shift_residual);
        let arg = |x: &SurfaceIdentities| x.argmax_disagreements.to_string();
        let sum = |x: &SurfaceIdentities| format!("{:.1e}", x.sum_residual);
        let _ = writeln!(
            s,
            "| {} | {} | {}/{}/{} | {}/{}/{} | {}/{}/{} | {}/{}/{} | {} |",
            su.suite,
            su.n_slots,
            f(0, scale),
            f(1, scale),
            f(2, scale),
            f(0, shift),
            f(1, shift),
            f(2, shift),
            f(0, arg),
            f(1, arg),
            f(2, arg),
            f(0, sum),
            f(1, sum),
            f(2, sum),
            match su.verdict {
                RubricVerdict::Pass => "PASS",
                RubricVerdict::Red => "RED",
            },
        );
    }
    let _ = writeln!(
        s,
        "\nRoute agreement (characterization, context = state+prompt+criteria route): \
         context==state-alone {:.1}% · option-route==context {:.1}% · name==semantic {:.1}% \
         of named options.\n",
        100.0 * context_state_agreement(out),
        100.0 * option_context_agreement(out),
        100.0 * name_semantic_agreement(out),
    );
    for su in &out.suites {
        for sk in &su.skipped {
            let _ = writeln!(s, "- skip [{}/modelless]: {sk}", su.suite);
        }
    }
    s
}

fn context_state_agreement(out: &RubricProbeOutput) -> f64 {
    let mut agree = 0usize;
    let mut total = 0usize;
    for su in &out.suites {
        for r in &su.routes {
            total += 1;
            agree += usize::from(r.context_route == r.state_route);
        }
    }
    if total == 0 {
        f64::NAN
    } else {
        agree as f64 / total as f64
    }
}

fn option_context_agreement(out: &RubricProbeOutput) -> f64 {
    let mut agree = 0usize;
    let mut total = 0usize;
    for su in &out.suites {
        for r in &su.routes {
            total += r.option_routes.len();
            agree += r.option_context_agree;
        }
    }
    if total == 0 {
        f64::NAN
    } else {
        agree as f64 / total as f64
    }
}

fn name_semantic_agreement(out: &RubricProbeOutput) -> f64 {
    let mut agree = 0usize;
    let mut total = 0usize;
    for su in &out.suites {
        for r in &su.routes {
            total += r.by_name.iter().filter(|n| n.is_some()).count();
            agree += r.name_semantic_agree;
        }
    }
    if total == 0 {
        f64::NAN
    } else {
        agree as f64 / total as f64
    }
}
