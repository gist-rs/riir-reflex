//! Issue 109 — the gate-fit threshold-stability audit (REPORT-ONLY runner
//! half).
//!
//! The harness fits fused-gate thresholds per suite at calibration ρ=30
//! (`--gate-fit-calibrated` is the promoted default since Issue 056), and
//! rethink pins per-suite `ESC_GATE_RHOS`. Neither surface audits whether
//! the fitted operating point is STABLE (would resampling the calibration
//! slice have produced a very different threshold?) or ON A PLATEAU (is
//! held-out performance flat around t\*, or does everything hinge on a
//! knife's edge the next re-fit could fall off?). This arm runs that audit
//! over the deterministic calibration pass itself: the pass is re-run per
//! suite (cheap, no model calls — the same build + solve the gate fit
//! already performs), the per-case [`GateObservation`]s are collected, and
//! the audit bootstraps them in memory. No per-case persistence exists
//! today, so there is no stored-record posture — an arm refusing loud is
//! preferred over bootstrapping constants if that ever changes.
//!
//! The pre-registered instrument (issue 109, recorded here so the numbers
//! carry their own construction):
//!
//! - **Bootstrap the gate fit per suite** (`B = 200` resamples with
//!   replacement, seeded [`SplitMix64`] — seed + iterations pinned in the
//!   output meta): each resample is fitted under the SAME posture, giving
//!   the t\* distribution (mean / SD / p05 / p95 under the repo's
//!   `idx = ⌊n·p⌋` clamp law with tail support).
//! - **Held-out curve at fixed offsets**: the eval front (the stratified
//!   test sample `prepare` already builds — held out from the cal slice by
//!   construction) is solved through the SAME engine, and accuracy(t) /
//!   pass_rate(t) are read at t\* ± {0.02, 0.05, 0.10} on the calibrated
//!   scale. A suite whose nearest-neighbor (±0.02) metric drops ≥2× is
//!   flagged single-point-sensitive.
//! - **Vacuity guard**: a resample yielding fewer than
//!   `max(10, 5% of n)` passing cases at its own t\* is VACUOUS — counted
//!   and disclosed; any vacuous resample marks the suite UNSTABLE (never a
//!   silent pass).
//! - **Stability verdict**: STABLE iff t\* SD ≤ 0.05 (the issue's bar),
//!   zero vacuous resamples, and no knife-edge flag — otherwise UNSTABLE
//!   with the named reasons. A knife-edge finding does NOT demote anything
//!   by itself: it names the suite and the risk; the re-fit decision is a
//!   separate gate with its own pre-registered bar.
//! - **Population floor**: the number of calibrated suites audited is
//!   printed every run — a green zero over an empty population is not a
//!   result (the repo's green-zero law).
//!
//! Determinism: same datasets + seed → byte-identical CELLS (the meta's
//! provenance lines — date/host/sha — are disclosure, not cells). The gate
//! test replays synthetic observations through the audit core twice and
//! asserts byte-identity, the vacuity guard's fire, and the knife-edge
//! flag's fire.
//!
//! Verdicts are COMPUTED, never gated: the arm is report-only (no
//! exit-code coupling, no auto-retune — the Issue-056 default posture is
//! not reopened by this audit).

use serde::Serialize;

use super::*;
use crate::embed::EMBED_DIM;
use crate::engine::{
    EngineConfig, GateObservation, Posture, Scratch, THIN_SUPPORT_FLOOR, threshold_recommendation,
};
use crate::harness::suites::{QKind, SplitMix64};
use std::path::Path;

/// Bootstrap resamples per suite-axis (pinned in the meta).
pub const GATE_STABILITY_ITERS: usize = 200;

/// The bootstrap seed (any fixed value; reproducibility is what matters —
/// the MASSIVE_OPTION_SEED convention).
pub const GATE_STABILITY_SEED: u64 = 0x1105_2026_0009;

/// Fixed offsets around t\* the held-out curve reads (absolute, on the
/// threshold's own [0, 1] scale; pinned in the meta). The FIRST offset is
/// the knife-edge's "nearest fixed neighbor".
pub const GATE_STABILITY_OFFSETS: [f32; 3] = [0.02, 0.05, 0.10];

/// The issue's stability bar on the t\* standard deviation.
pub const TSTAR_SD_BAR: f64 = 0.05;

/// The knife-edge bar: a ≥2× metric drop at the nearest fixed neighbor
/// flags single-point sensitivity.
pub const KNIFE_EDGE_DROP: f64 = 2.0;

/// The vacuity floor: a resample with fewer passing cases than
/// `max(10, 5% of n)` at its own t\* is vacuous.
const VACUITY_ABS_FLOOR: usize = 10;
const VACUITY_REL_FLOOR: f64 = 0.05;

/// The audit's default suite set: the serving board (the suites whose gate
/// fits the arena/serve posture actually consumes). `--suites` overrides.
pub const GATE_STABILITY_SUITES: &[&str] = &[
    "massive_intent_en",
    "banking77",
    "typed_decisions",
    "prompt_injections",
    "sst5",
    "ag_news",
    "xnli_en",
    "emotion",
];

/// Default held-out cases per suite (stride-sampled over the stratified
/// eval front — the test surface stays a sample; this is a report, not a
/// full eval run).
pub const DEFAULT_MAX_EVAL_PER_SUITE: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GateStabilityOptions {
    /// Bootstrap resamples per suite-axis.
    pub iters: usize,
    /// The pinned bootstrap seed.
    pub seed: u64,
    /// The percentile posture's ρ (0.30 = the shipped arena posture).
    pub rho: f64,
    /// Held-out cases per suite (stride-sampled over the eval front).
    pub max_eval_per_suite: usize,
    /// Issue 056 posture: fit + read the score axis on the CALIBRATED
    /// scale (the promoted default). Exposed so the raw-scale posture
    /// stays measurable; the report meta discloses which ran.
    pub gate_fit_calibrated: bool,
}

impl Default for GateStabilityOptions {
    fn default() -> Self {
        Self {
            iters: GATE_STABILITY_ITERS,
            seed: GATE_STABILITY_SEED,
            rho: 0.30,
            max_eval_per_suite: DEFAULT_MAX_EVAL_PER_SUITE,
            gate_fit_calibrated: true,
        }
    }
}

// ───────────────────────────────────────────── report shapes ──

/// One bootstrap t\* distribution summary. Quantiles ride the repo's
/// `idx = ⌊n·p⌋` clamp law WITH tail support (the p99-at-small-n trap).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TStarDistribution {
    pub n_resamples: usize,
    /// Resamples whose fit returned None (cannot happen at
    /// n ≥ [`THIN_SUPPORT_FLOOR`] — the resample has exactly n rows; the
    /// field keeps the shape total and the count is disclosed, never
    /// pooled into the statistics).
    pub n_none: usize,
    pub mean: f64,
    pub sd: f64,
    pub p05: f32,
    pub p05_rank: usize,
    pub p05_support: usize,
    pub p95: f32,
    pub p95_rank: usize,
    pub p95_support: usize,
    pub min: f32,
    pub max: f32,
}

/// The held-out metric at one threshold point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CurvePoint {
    /// The threshold read (t\* itself, or t\* + offset).
    pub t: f32,
    /// Above-threshold accuracy on the held-out front — None when nothing
    /// passes (a support cliff, disclosed via `pass_rate`; an Option, not
    /// a NaN, so cells stay equatable and JSON-null is explicit).
    pub accuracy: Option<f64>,
    /// Share of held-out cases at or above t (the operating volume).
    pub pass_rate: f64,
    pub n_pass: usize,
    pub n: usize,
}

/// One axis's audit cell (score or distance) for one suite.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateStabilityAxis {
    /// "score" | "distance".
    pub axis: &'static str,
    /// The full-slice fit's t\* (None = thin support — the axis carries no
    /// fit and the cell reports WHY instead of a number).
    pub t_star: Option<f32>,
    pub cal_n: usize,
    pub bootstrap: Option<TStarDistribution>,
    /// Resamples failing the vacuity floor at their own t\*.
    pub vacuous_resamples: usize,
    /// The held-out curve: t\* first, then the ±offset ladder (nearest out
    /// to farthest below, mirrored above).
    pub curve: Vec<CurvePoint>,
    /// Nearest-neighbor (±first offset) ≥2× metric drop found.
    pub knife_edge: bool,
    /// Which metric tripped the knife-edge flag ("accuracy" | "pass_rate").
    pub knife_edge_metric: Option<&'static str>,
    pub stable: bool,
    /// Named reasons when unstable (empty when stable — STABLE is the
    /// three bars passing BY ASSERTION, and the population floor line
    /// prints the audited count every run).
    pub reasons: Vec<String>,
}

/// One suite's audit cell.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateStabilitySuite {
    pub suite: String,
    pub labels: usize,
    pub corpus_rows: usize,
    pub eval_n: usize,
    pub score: GateStabilityAxis,
    pub distance: GateStabilityAxis,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateStabilityMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub suites: Vec<String>,
    pub iters: usize,
    pub seed: u64,
    pub rho: f64,
    pub offsets: Vec<f32>,
    pub tstar_sd_bar: f64,
    pub knife_edge_drop: f64,
    pub vacuity_floor_abs: usize,
    pub vacuity_floor_rel: f64,
    pub gate_fit_calibrated: bool,
    pub posture_note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateStabilityOutput {
    pub meta: GateStabilityMeta,
    pub suites: Vec<GateStabilitySuite>,
}

// ───────────────────────────────────────────── core ──

/// The deterministic bootstrap: `iters` resamples with replacement over the
/// observations (seeded [`SplitMix64`] — one fresh stream per axis; the
/// axis salt keeps the two axes' draws decorrelated), each resample fitted
/// under the audit posture. Returns the t\* draws plus the vacuity count
/// (a resample with fewer than `max(10, 5% of n)` passing cases at its own
/// t\*).
fn bootstrap_t_star(
    obs: &[GateObservation],
    rho: f64,
    iters: usize,
    seed: u64,
    salt: u64,
) -> (Vec<f32>, usize) {
    let n = obs.len();
    let vac_floor = VACUITY_ABS_FLOOR.max((n as f64 * VACUITY_REL_FLOOR) as usize);
    let mut rng = SplitMix64::new(seed ^ salt);
    let mut draws: Vec<f32> = Vec::with_capacity(iters);
    let mut vacuous = 0usize;
    for _ in 0..iters {
        // The resample: n draws with replacement, one u64 per draw, the
        // bounded index via `r % n` (the bias is bounded by n and this is
        // a report instrument — the method is pinned here and in the meta
        // note so any future refinement re-pins consciously).
        let mut sample: Vec<GateObservation> = Vec::with_capacity(n);
        for _ in 0..n {
            let r = rng.next_u64();
            sample.push(obs[(r % n as u64) as usize]);
        }
        match threshold_recommendation(&sample, Posture::Percentile { rho }) {
            Some(rec) => {
                if rec.support.n_pass < vac_floor {
                    vacuous += 1;
                }
                draws.push(rec.threshold);
            }
            // Thin support cannot fire at n ≥ THIN_SUPPORT_FLOOR (the
            // resample has exactly n rows); the draw lands NaN and the
            // distribution discloses the count.
            None => draws.push(f32::NAN),
        }
    }
    (draws, vacuous)
}

/// The t\* distribution summary over the draws.
fn t_star_distribution(draws: &[f32]) -> TStarDistribution {
    let finite: Vec<f32> = draws.iter().copied().filter(|d| d.is_finite()).collect();
    let n_none = draws.len() - finite.len();
    let mean = if finite.is_empty() {
        f64::NAN
    } else {
        finite.iter().map(|&d| d as f64).sum::<f64>() / finite.len() as f64
    };
    let sd = if finite.len() < 2 {
        0.0
    } else {
        finite
            .iter()
            .map(|&d| {
                let x = d as f64 - mean;
                x * x
            })
            .sum::<f64>()
            / (finite.len() - 1) as f64
    }
    .sqrt();
    let mut sorted = finite.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let q = |p: f64| -> (f32, usize, usize) {
        if sorted.is_empty() {
            return (f32::NAN, 0, 0);
        }
        let n = sorted.len();
        let rank = ((n as f64 * p) as usize).min(n - 1);
        (sorted[rank], rank, n - rank)
    };
    let (p05, p05_rank, p05_support) = q(0.05);
    let (p95, p95_rank, p95_support) = q(0.95);
    TStarDistribution {
        n_resamples: draws.len(),
        n_none,
        mean,
        sd,
        p05,
        p05_rank,
        p05_support,
        p95,
        p95_rank,
        p95_support,
        min: sorted.first().copied().unwrap_or(f32::NAN),
        max: sorted.last().copied().unwrap_or(f32::NAN),
    }
}

/// The held-out metric at one threshold point.
fn curve_point(obs: &[GateObservation], t: f32) -> CurvePoint {
    let n = obs.len();
    let mut n_pass = 0usize;
    let mut n_correct = 0usize;
    for o in obs {
        if o.score >= t {
            n_pass += 1;
            n_correct += o.correct as usize;
        }
    }
    CurvePoint {
        t,
        accuracy: if n_pass > 0 {
            Some(n_correct as f64 / n_pass as f64)
        } else {
            None
        },
        pass_rate: if n > 0 { n_pass as f64 / n as f64 } else { 0.0 },
        n_pass,
        n,
    }
}

/// The knife-edge read: does the nearest fixed neighbor halve either
/// metric against t\*? Accuracy compares only when both ends carry
/// passing support (a NaN at one end is a support cliff, disclosed via
/// pass_rate — never a flag by itself).
fn knife_edge_of(star: &CurvePoint, near: &CurvePoint) -> (bool, Option<&'static str>) {
    let mut hit = None;
    if near.pass_rate > 0.0 && star.pass_rate / near.pass_rate >= KNIFE_EDGE_DROP {
        hit = Some("pass_rate");
    }
    if hit.is_none()
        && let (Some(sa), Some(na)) = (star.accuracy, near.accuracy)
        && na > 0.0
        && sa / na >= KNIFE_EDGE_DROP
    {
        hit = Some("accuracy");
    }
    (hit.is_some(), hit)
}

fn axis_cell(
    axis: &'static str,
    cal_obs: &[GateObservation],
    eval_obs: &[GateObservation],
    opts: &GateStabilityOptions,
) -> GateStabilityAxis {
    // The axis salt salts the bootstrap stream (score 0x53.., distance
    // 0x44.. — "SCOR" / "DIST" hex marks; any fixed pair works).
    let salt: u64 = if axis == "score" {
        0x5343_4f52_0000_0000
    } else {
        0x4449_5354_0000_0000
    };
    let t_star = threshold_recommendation(cal_obs, Posture::Percentile { rho: opts.rho })
        .map(|r| r.threshold);
    let cal_n = cal_obs.len();
    let mut reasons: Vec<String> = Vec::new();
    let (bootstrap, vacuous, curve, knife_edge, knife_metric) = if cal_n < THIN_SUPPORT_FLOOR {
        reasons.push(format!(
            "thin support: cal n {cal_n} < {THIN_SUPPORT_FLOOR} — no fit, no numbers"
        ));
        (None, 0usize, Vec::new(), false, None)
    } else {
        let (draws, vacuous) = bootstrap_t_star(cal_obs, opts.rho, opts.iters, opts.seed, salt);
        let dist = t_star_distribution(&draws);
        // Held-out curve: t\* first, then the ±offset ladder (nearest out
        // to farthest below, mirrored above).
        let mut curve = vec![curve_point(eval_obs, t_star.unwrap_or(0.0))];
        for off in GATE_STABILITY_OFFSETS {
            curve.push(curve_point(eval_obs, t_star.unwrap_or(0.0) - off));
        }
        for off in GATE_STABILITY_OFFSETS.iter().rev() {
            curve.push(curve_point(eval_obs, t_star.unwrap_or(0.0) + off));
        }
        let t = t_star.unwrap_or(0.0);
        let near_lo = curve_point(eval_obs, (t - GATE_STABILITY_OFFSETS[0]).max(0.0));
        let near_hi = curve_point(eval_obs, (t + GATE_STABILITY_OFFSETS[0]).min(1.0));
        let (mut edge, mut metric) = knife_edge_of(&curve[0], &near_lo);
        if !edge {
            let (e, m) = knife_edge_of(&curve[0], &near_hi);
            edge = e;
            metric = m;
        }
        (Some(dist), vacuous, curve, edge, metric)
    };
    if let Some(dist) = &bootstrap {
        if dist.sd > TSTAR_SD_BAR {
            reasons.push(format!(
                "t* sd {:.4} > {TSTAR_SD_BAR} (mean {:.4}, p05 {:.4}, p95 {:.4})",
                dist.sd, dist.mean, dist.p05, dist.p95
            ));
        }
        if vacuous > 0 {
            reasons.push(format!("{vacuous} vacuous resample(s) at their own t*"));
        }
    }
    if knife_edge {
        reasons.push(format!(
            "knife edge: nearest neighbor (±{:.2}) halves {}",
            GATE_STABILITY_OFFSETS[0],
            knife_metric.unwrap_or("a metric")
        ));
    }
    GateStabilityAxis {
        axis,
        t_star,
        cal_n,
        bootstrap,
        vacuous_resamples: vacuous,
        curve,
        knife_edge,
        knife_edge_metric: if knife_edge { knife_metric } else { None },
        stable: reasons.is_empty(),
        reasons,
    }
}

// ───────────────────────────────────────────── per-suite driver ──

struct SuiteAudit {
    labels: usize,
    corpus_rows: usize,
    eval_n: usize,
    score: Vec<GateObservation>,
    distance: Vec<GateObservation>,
    eval_score: Vec<GateObservation>,
    eval_distance: Vec<GateObservation>,
}

/// The deterministic calibration pass, re-run verbatim at the default
/// posture (levers off): build the probe engine over the corpus pool,
/// solve the cal slice, collect the fit pairs + both axes' last-slot
/// observations, apply the Issue-056 calibrator when the posture arms,
/// then solve the held-out front through the SAME engine with the SAME
/// calibrator (the deployed serve shape — the gate reads calibrated
/// confidences at eval time).
fn audit_suite_observations(
    spec: &SuiteSpec,
    dir: &Path,
    opts: &GateStabilityOptions,
) -> Result<SuiteAudit, String> {
    let prepared = super::prepare(spec, dir)?;
    macro_rules! dispatch {
        ($n:literal) => {
            audit_observations::<$n>(spec, &prepared, opts)
        };
    }
    match prepared.labels.len() {
        2 => dispatch!(2),
        3 => dispatch!(3),
        4 => dispatch!(4),
        5 => dispatch!(5),
        6 => dispatch!(6),
        7 => dispatch!(7),
        8 => dispatch!(8),
        10 => dispatch!(10),
        15 => dispatch!(15),
        56 => dispatch!(56),
        59 => dispatch!(59),
        77 => dispatch!(77),
        842 => dispatch!(842),
        n => Err(format!(
            "{}: no engine instantiation for {n} domains — extend \
             gate_stability's dispatch",
            spec.name
        )),
    }
}

#[allow(clippy::too_many_lines)]
fn audit_observations<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &GateStabilityOptions,
) -> Result<SuiteAudit, String> {
    let effective_cap = spec.corpus_cap_per_label;
    // The incumbent shipped posture (RunOptions::lanes_off): kn_route on,
    // no lever overrides.
    let cfg = EngineConfig {
        legacy_kn_route: true,
        ..EngineConfig::default()
    };
    let (mut probe, _) = super::build_engine::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        cfg,
    )?;
    let cal_defaults = EngineConfig::default();

    // ── the cal-slice fit surface (the deployed fit loop, verbatim) ──
    let mut score_obs: Vec<GateObservation> = Vec::new();
    let mut distance_obs: Vec<GateObservation> = Vec::new();
    let mut fit_pairs: Vec<(f32, bool)> = Vec::new();
    {
        let mut sc = Scratch::<EMBED_DIM>::new();
        let max_q = prepared
            .cal_cases
            .iter()
            .map(|c| c.questions.len())
            .max()
            .unwrap_or(1);
        sc.prepare(max_q);
        for (ci, case) in prepared.cal_cases.iter().enumerate() {
            let req = super::engine_request(case, &prepared.cal_state_strs[ci])?;
            probe.solve_into(&req, &mut sc).map_err(|e| {
                format!(
                    "gate-stability cal solve ({}, case {}): {e}",
                    spec.name, case.id
                )
            })?;
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
            // sample the last slot per case (the fit loop's own law).
            if let (Some(slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) {
                let pick = match case.questions.last().map(|q| q.kind) {
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
    }

    // ── the Issue-056 calibrator (the promoted posture) ──
    let calibrated_scale = if opts.gate_fit_calibrated {
        let mut cal = katgpt_core::sigmoid_calibration::SigmoidGateCalibrator::new(
            cal_defaults.cal_capacity,
            cal_defaults.cal_min_obs,
        );
        for (c, ok) in &fit_pairs {
            cal.observe(*c, *ok);
        }
        cal.refit();
        for obs in &mut score_obs {
            obs.score = cal.apply(obs.score);
        }
        Some(cal)
    } else {
        None
    };

    // ── the held-out front through the SAME engine + SAME calibrator ──
    let mut eval_score: Vec<GateObservation> = Vec::new();
    let mut eval_distance: Vec<GateObservation> = Vec::new();
    {
        let picked: Vec<usize> =
            super::perm_probe::stride_indices(prepared.suite.cases.len(), opts.max_eval_per_suite);
        let mut sc = Scratch::<EMBED_DIM>::new();
        let max_q = picked
            .iter()
            .filter_map(|&i| prepared.suite.cases.get(i))
            .map(|c| c.questions.len())
            .max()
            .unwrap_or(1);
        sc.prepare(max_q);
        for &i in &picked {
            let case = &prepared.suite.cases[i];
            let req = super::engine_request(case, &prepared.state_strs[i])?;
            probe.solve_into(&req, &mut sc).map_err(|e| {
                format!(
                    "gate-stability eval solve ({}, case {}): {e}",
                    spec.name, case.id
                )
            })?;
            let (Some(slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) else {
                continue;
            };
            let pick = match case.questions.last().map(|q| q.kind) {
                Some(QKind::Noul) => (1 - slot.pick) as usize,
                _ => slot.pick as usize,
            };
            let correct = case.gold.last().is_some_and(|g| g.idx == pick);
            let raw_conf = slot.confidence;
            let conf = calibrated_scale
                .as_ref()
                .map_or(raw_conf, |cal| cal.apply(raw_conf));
            eval_score.push(GateObservation {
                score: conf,
                correct,
            });
            eval_distance.push(GateObservation {
                score: probe.gate(*dom).abstain_confidence(&sc.q),
                correct,
            });
        }
    }

    Ok(SuiteAudit {
        labels: prepared.labels.len(),
        corpus_rows: prepared.train.len(),
        eval_n: eval_score.len(),
        score: score_obs,
        distance: distance_obs,
        eval_score,
        eval_distance,
    })
}

fn spec_of(name: &str) -> Result<&'static SuiteSpec, String> {
    SUITES
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("gate-stability: suite {name} is not in the registry"))
}

// ───────────────────────────────────────────── entry ──

/// Run the audit over the named suites (empty = the serving board). The
/// per-case collection runs at the DEFAULT harness posture (no selection
/// levers); report-only — no gate, no exit-code coupling.
pub fn run_gate_stability(
    opts: &RunOptions,
    gopts: GateStabilityOptions,
) -> Result<GateStabilityOutput, String> {
    if gopts.iters == 0 {
        return Err("gate-stability: iters must be ≥ 1".to_string());
    }
    if !(0.0..=1.0).contains(&gopts.rho) {
        return Err("gate-stability: rho must be in [0, 1]".to_string());
    }
    let names: Vec<String> = if opts.suites.is_empty() {
        GATE_STABILITY_SUITES
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    } else {
        opts.suites.clone()
    };
    let mut suites: Vec<GateStabilitySuite> = Vec::new();
    for name in &names {
        let spec = spec_of(name)?;
        if spec.synthetic.is_some() {
            eprintln!(
                "  [gate-stability] {name}: synthetic — skipped (the audit rides dataset suites)"
            );
            continue;
        }
        let audit = audit_suite_observations(spec, &opts.datasets_dir, &gopts)?;
        let score = axis_cell("score", &audit.score, &audit.eval_score, &gopts);
        let distance = axis_cell("distance", &audit.distance, &audit.eval_distance, &gopts);
        eprintln!(
            "  [gate-stability] {}: score t* {:.4} sd {:.4} stable {} · distance t* {:.4} sd {:.4} stable {}",
            name,
            score.t_star.unwrap_or(f32::NAN),
            score.bootstrap.as_ref().map_or(f64::NAN, |b| b.sd),
            score.stable,
            distance.t_star.unwrap_or(f32::NAN),
            distance.bootstrap.as_ref().map_or(f64::NAN, |b| b.sd),
            distance.stable,
        );
        suites.push(GateStabilitySuite {
            suite: (*name).to_string(),
            labels: audit.labels,
            corpus_rows: audit.corpus_rows,
            eval_n: audit.eval_n,
            score,
            distance,
        });
    }
    Ok(GateStabilityOutput {
        meta: GateStabilityMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            suites: names,
            iters: gopts.iters,
            seed: gopts.seed,
            rho: gopts.rho,
            offsets: GATE_STABILITY_OFFSETS.to_vec(),
            tstar_sd_bar: TSTAR_SD_BAR,
            knife_edge_drop: KNIFE_EDGE_DROP,
            vacuity_floor_abs: VACUITY_ABS_FLOOR,
            vacuity_floor_rel: VACUITY_REL_FLOOR,
            gate_fit_calibrated: gopts.gate_fit_calibrated,
            posture_note: "default harness posture (levers off, kn_route on, \
             calibrated gate fit per Issue 056) — the calibration pass re-run \
             verbatim; bootstrap = SplitMix64 resamples with replacement, \
             bounded index r % n (method pinned, report instrument); held-out \
             curve on the stratified eval front through the SAME engine + \
             calibrator. Cells are byte-identical across runs; the meta's \
             provenance lines are disclosure, not cells."
                .to_string(),
        },
        suites,
    })
}

// ───────────────────────────────────────────── render ──

fn pct(x: Option<f64>) -> String {
    match x {
        Some(v) if v.is_finite() => format!("{v:.4}"),
        _ => "n/a".to_string(),
    }
}

fn render_axis(a: &GateStabilityAxis) -> Vec<String> {
    let mut out = Vec::new();
    let t = a
        .t_star
        .map_or_else(|| "none".to_string(), |t| format!("{t:.4}"));
    out.push(format!(
        "- t*: {t} · cal n {} · stable: {}",
        a.cal_n, a.stable
    ));
    if let Some(b) = &a.bootstrap {
        out.push(format!(
            "  bootstrap: mean {} · sd {} · p05 {:.4} (support {}) · p95 {:.4} (support {}) · min {:.4} · max {:.4} · none {}",
            pct(Some(b.mean)),
            pct(Some(b.sd)),
            b.p05,
            b.p05_support,
            b.p95,
            b.p95_support,
            b.min,
            b.max,
            b.n_none
        ));
        out.push(format!(
            "  vacuous resamples: {} · knife edge: {}{}",
            a.vacuous_resamples,
            a.knife_edge,
            a.knife_edge_metric
                .map_or_else(String::new, |m| format!(" ({m})"))
        ));
        if !a.curve.is_empty() {
            out.push("  held-out curve (t · acc · pass):".to_string());
            for p in &a.curve {
                out.push(format!(
                    "    {:.4} · {} · {} ({} / {})",
                    p.t,
                    pct(p.accuracy),
                    pct(Some(p.pass_rate)),
                    p.n_pass,
                    p.n
                ));
            }
        }
    }
    for r in &a.reasons {
        out.push(format!("  ⚠ {r}"));
    }
    out
}

/// The markdown report (the stdout surface + the .md artifact — one
/// render, byte-identical for a given output).
pub fn render_gate_stability_markdown(out: &GateStabilityOutput) -> String {
    let mut md = String::new();
    md.push_str("# Gate-fit threshold-stability audit (issue 109)\n\n");
    md.push_str(&format!(
        "- suites: {} · iters {} · seed {} · rho {:.2} · offsets {:?}\n",
        out.meta.suites.join(", "),
        out.meta.iters,
        out.meta.seed,
        out.meta.rho,
        out.meta.offsets
    ));
    md.push_str(&format!(
        "- bars: t* sd ≤ {} · vacuity floor max({}, {}·n) · knife edge ≥{}× drop at ±{:.2}\n",
        out.meta.tstar_sd_bar,
        out.meta.vacuity_floor_abs,
        out.meta.vacuity_floor_rel,
        out.meta.knife_edge_drop,
        out.meta.offsets[0]
    ));
    md.push_str(&format!(
        "- posture: calibrated fit {} · datasets {} · {}\n\n",
        out.meta.gate_fit_calibrated,
        out.meta.datasets_dir,
        out.meta.posture_note.replace('\n', " ")
    ));
    let audited = out.suites.len();
    let unstable_axes: usize = out
        .suites
        .iter()
        .map(|s| usize::from(!s.score.stable) + usize::from(!s.distance.stable))
        .sum();
    md.push_str(&format!(
        "population floor: {audited} calibrated suite(s) audited · {unstable_axes} unstable axis(es)\n\n"
    ));
    for s in &out.suites {
        md.push_str(&format!(
            "## {} (labels {} · corpus rows {} · eval n {})\n\n",
            s.suite, s.labels, s.corpus_rows, s.eval_n
        ));
        for line in render_axis(&s.score) {
            md.push_str(&line);
            md.push('\n');
        }
        for line in render_axis(&s.distance) {
            md.push_str(&line);
            md.push('\n');
        }
        md.push('\n');
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(rows: &[(f32, bool)]) -> Vec<GateObservation> {
        rows.iter()
            .map(|&(score, correct)| GateObservation { score, correct })
            .collect()
    }

    /// t\* = 0.5 at ρ = 0.30 over n = 100: 30 rows at 0.0, 70 rows at
    /// 0.5 — idx = ⌊100·0.30⌋ = 30 lands on the first 0.5.
    fn cal_cliff() -> Vec<GateObservation> {
        let mut rows = vec![(0.0f32, true); 30];
        rows.extend(vec![(0.5f32, true); 70]);
        obs(&rows)
    }

    /// A dense cal slice: 100 distinct ascending scores — the resampled
    /// 30th percentile wiggles in a narrow band, so the SD bar never
    /// fires on this fixture.
    fn cal_dense() -> Vec<GateObservation> {
        obs(&(0..100)
            .map(|i| (0.10 + 0.008 * i as f32, true))
            .collect::<Vec<_>>())
    }

    #[test]
    fn t_star_lands_at_the_percentile_law() {
        let rec = threshold_recommendation(&cal_cliff(), Posture::Percentile { rho: 0.30 })
            .expect("n = 100 fits");
        assert_eq!(rec.threshold, 0.5);
        assert_eq!(rec.support.n, 100);
        assert_eq!(rec.support.n_pass, 70);
        assert_eq!(rec.support.n_abstain, 30);
    }

    #[test]
    fn bootstrap_is_deterministic_across_runs() {
        let a = bootstrap_t_star(&cal_dense(), 0.30, 50, GATE_STABILITY_SEED, 0x01);
        let b = bootstrap_t_star(&cal_dense(), 0.30, 50, GATE_STABILITY_SEED, 0x01);
        assert_eq!(a, b);
        // The axis salt decorrelates the streams: different salt, so a
        // different draw sequence is ALLOWED — pin only that the same
        // salt reproduces (above) and that the draws are all finite.
        assert!(a.0.iter().all(|d| d.is_finite()));
        assert_eq!(a.1, 0, "dense slice at rho 0.30 is never vacuous");
    }

    #[test]
    fn axis_cell_is_byte_identical_across_calls() {
        let opts = GateStabilityOptions {
            iters: 40,
            ..GateStabilityOptions::default()
        };
        let eval = cal_dense();
        let a = axis_cell("score", &cal_dense(), &eval, &opts);
        let b = axis_cell("score", &cal_dense(), &eval, &opts);
        assert_eq!(
            a, b,
            "same inputs + seed → identical cell (PartialEq, no NaN)"
        );
        assert!(a.stable);
        assert!(a.reasons.is_empty());
        assert_eq!(a.vacuous_resamples, 0);
        assert!(!a.knife_edge);
    }

    #[test]
    fn vacuity_guard_fires_at_a_high_rho() {
        // ρ = 0.98 → t\* at the 98th percentile of a dense slice → the
        // pass set holds ~2% of n — under max(10, 5%·n) → vacuous.
        let (draws, vacuous) = bootstrap_t_star(&cal_dense(), 0.98, 30, GATE_STABILITY_SEED, 0x02);
        assert_eq!(draws.len(), 30);
        assert!(draws.iter().all(|d| d.is_finite()));
        assert!(vacuous > 0, "the 98th percentile fit leaves < 10 passing");
    }

    #[test]
    fn knife_edge_fires_on_an_accuracy_cliff() {
        let opts = GateStabilityOptions {
            iters: 40,
            ..GateStabilityOptions::default()
        };
        // t\* = 0.5; the held-out front: 30 rows at 0.0 (never pass),
        // 40 rows at 0.49 (pass only at t\*−0.02, nearly all wrong),
        // 30 rows at 0.9 (all correct). Star accuracy 1.0 vs the lower
        // neighbor's 2/70 — the ≥2× accuracy drop fires on the accuracy
        // metric (pass_rate ratio 0.3/0.7 never trips).
        let mut eval = vec![(0.0f32, false); 30];
        eval.extend(vec![(0.49f32, false); 38]);
        eval.extend(vec![(0.49f32, true); 2]);
        eval.extend(vec![(0.9f32, true); 30]);
        let cell = axis_cell("score", &cal_cliff(), &obs(&eval), &opts);
        assert!(cell.knife_edge, "the accuracy cliff must flag");
        assert_eq!(cell.knife_edge_metric, Some("accuracy"));
        assert!(!cell.stable);
        assert!(cell.reasons.iter().any(|r| r.contains("knife edge")));
    }

    #[test]
    fn knife_edge_fires_on_a_pass_rate_cliff() {
        let opts = GateStabilityOptions {
            iters: 40,
            ..GateStabilityOptions::default()
        };
        // Mass parked just ABOVE t\*: at t\*+0.02 the pass set collapses
        // 70 → 10 — pass_rate ratio 7 ≥ 2, checked before accuracy.
        let mut eval = vec![(0.0f32, false); 30];
        eval.extend(vec![(0.51f32, false); 60]);
        eval.extend(vec![(0.9f32, true); 10]);
        let cell = axis_cell("score", &cal_cliff(), &obs(&eval), &opts);
        assert!(cell.knife_edge);
        assert_eq!(cell.knife_edge_metric, Some("pass_rate"));
    }

    #[test]
    fn thin_support_reports_no_numbers() {
        let opts = GateStabilityOptions::default();
        let thin = obs(&[(0.5f32, true); THIN_SUPPORT_FLOOR - 1]);
        let cell = axis_cell("score", &thin, &thin, &opts);
        assert!(cell.t_star.is_none());
        assert!(cell.bootstrap.is_none());
        assert!(cell.curve.is_empty());
        assert!(!cell.stable);
        assert!(cell.reasons.iter().any(|r| r.contains("thin support")));
    }

    #[test]
    fn curve_ladder_is_star_first_then_mirrored_offsets() {
        let opts = GateStabilityOptions {
            iters: 40,
            ..GateStabilityOptions::default()
        };
        let eval = cal_dense();
        let cell = axis_cell("score", &cal_dense(), &eval, &opts);
        let t = cell.t_star.expect("dense cal fits");
        assert_eq!(cell.curve.len(), 1 + 2 * GATE_STABILITY_OFFSETS.len());
        assert_eq!(cell.curve[0].t, t);
        for (k, off) in GATE_STABILITY_OFFSETS.iter().enumerate() {
            assert_eq!(cell.curve[1 + k].t, t - off);
        }
        for (k, off) in GATE_STABILITY_OFFSETS.iter().rev().enumerate() {
            assert_eq!(cell.curve[1 + GATE_STABILITY_OFFSETS.len() + k].t, t + off);
        }
    }

    #[test]
    fn distribution_quantiles_carry_support() {
        let (draws, _) = bootstrap_t_star(&cal_dense(), 0.30, 200, GATE_STABILITY_SEED, 0x03);
        let dist = t_star_distribution(&draws);
        assert_eq!(dist.n_resamples, 200);
        assert_eq!(dist.n_none, 0);
        assert!(dist.sd >= 0.0);
        assert!(dist.p05 <= dist.p95);
        assert_eq!(dist.p05_rank, (200.0_f64 * 0.05) as usize);
        assert_eq!(dist.p05_support, 200 - dist.p05_rank);
        assert_eq!(dist.p95_rank, (200.0_f64 * 0.95) as usize);
        assert_eq!(dist.p95_support, 200 - dist.p95_rank);
        assert_eq!(
            dist.min,
            draws.iter().copied().fold(f32::INFINITY, f32::min)
        );
        assert_eq!(
            dist.max,
            draws.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        );
    }

    #[test]
    fn render_is_deterministic_for_a_fixed_output() {
        let opts = GateStabilityOptions {
            iters: 40,
            ..GateStabilityOptions::default()
        };
        let eval = cal_dense();
        let suite = GateStabilitySuite {
            suite: "fixture".to_string(),
            labels: 2,
            corpus_rows: 64,
            eval_n: eval.len(),
            score: axis_cell("score", &cal_dense(), &eval, &opts),
            distance: axis_cell("distance", &cal_dense(), &eval, &opts),
        };
        let meta = GateStabilityMeta {
            date_utc: "fixed".to_string(),
            git_sha: Some("fixed".to_string()),
            host: "fixed".to_string(),
            datasets_dir: "fixed".to_string(),
            suites: vec!["fixture".to_string()],
            iters: opts.iters,
            seed: opts.seed,
            rho: opts.rho,
            offsets: GATE_STABILITY_OFFSETS.to_vec(),
            tstar_sd_bar: TSTAR_SD_BAR,
            knife_edge_drop: KNIFE_EDGE_DROP,
            vacuity_floor_abs: 10,
            vacuity_floor_rel: VACUITY_REL_FLOOR,
            gate_fit_calibrated: true,
            posture_note: "fixed".to_string(),
        };
        let out = GateStabilityOutput {
            meta,
            suites: vec![suite],
        };
        assert_eq!(
            render_gate_stability_markdown(&out),
            render_gate_stability_markdown(&out)
        );
        assert!(
            render_gate_stability_markdown(&out)
                .contains("population floor: 1 calibrated suite(s) audited")
        );
    }
}
