//! Issue 066 — the fused-abstain DENSITY half's paired A/B (report-only
//! POC; `--density-gate`).
//!
//! The LSL App-E shape (arXiv:2610.02126): the shipped fused gate's score
//! axis measures CLASSIFIER confidence and its distance axis measures
//! exemplar max-cosine; the density half adds IN-SUPPORT-NESS — the
//! routed domain's two-density gate (`sigmoid(ℓ/τ)` over the per-domain
//! diagonal GMM vs the pooled reference, JL-projected per the fit-space
//! law) as a THIRD fused axis. This pass builds the density-armed engine
//! at the SAME fitted posture the deployed gate runs (score + distance
//! thresholds verbatim, the calibrator re-fit on the same cal pairs in
//! the same order), fits the density threshold at the cal-slice ρ=30
//! percentile (the T1.6 posture shared with the other two axes), reads
//! the SAME frozen test slice ONCE, and pairs per-question against the
//! shipped calibrated gate's read.
//!
//! # The pairing law (load-bearing)
//!
//! The gate NEVER touches scoring — picks and probabilities are provably
//! identical between arms (pinned by engine unit test), so the density
//! arm's abstain set is a strict SUPERSET of the shipped arm's and the
//! whole difference lives in the MARGINAL slice: questions the shipped
//! arm answered and the density arm abstained. `marginal_correct` low ⇒
//! the density half is removing ERRORS (the win surface: suites where
//! the cal-fitted threshold transfers badly, the Bench-061 abstain-
//! pathology class); `marginal_correct` high ⇒ it is throwing away
//! good answers. `pick_disagreements > 0` reds the A/B itself (the
//! pairing premise broke — never publish a number off it).
//!
//! Report-only by law: the published row stays the shipped posture's;
//! adoption is a separate flip gated on the paired verdict (the issue's
//! own gate task), never a default change riding a measurement pass.

use crate::embed::EMBED_DIM;
use crate::engine::threshold::threshold_recommendation;
use crate::engine::{DecisionEngine, EngineConfig, GateObservation, Posture, Scratch};
use crate::harness::metrics::{ece_of, CalibrationPair};
use crate::harness::runner::{eval_engine, engine_request, AbstainCauses};
use crate::harness::suites::{QKind, SuiteCase};
use serde::Serialize;

/// The birth-constant fallback (thin cal support) — engine const, mirrored
/// here only for the record's disclosure. Kept in sync by the unit test.
use crate::engine::DEFAULT_DENSITY_THRESHOLD;

/// The per-arm selective summary (the abstain/accuracy trade the A/B
/// exists to measure).
#[derive(Debug, Clone, Serialize)]
pub struct ArmStats {
    pub abstain_rate: f64,
    pub selective_accuracy: f64,
    pub selective_n: usize,
    /// Readout ECE over the arm's ANSWERED questions (the selective-
    /// calibration read — the all-questions ECE is arm-invariant by
    /// construction, picks and confs never move). `None` on an empty
    /// answered set.
    pub answered_ece: Option<f64>,
}

/// The Issue-066 A/B record (rides the modelless row, `density_ab`).
#[derive(Debug, Clone, Serialize)]
pub struct DensityAbRecord {
    pub suite: String,
    /// The ρ=30-cal-fitted density threshold actually applied.
    pub density_threshold: f32,
    pub density_tau: f32,
    /// `false` when the cal slice was too thin to fit
    /// (`THIN_SUPPORT_FLOOR` — jimothy's null rule); the birth constant
    /// ran instead, disclosed.
    pub density_threshold_fitted: bool,
    pub shipped: ArmStats,
    pub density: ArmStats,
    /// Questions the shipped arm ANSWERED and the density arm abstained
    /// (the whole difference — the superset law above).
    pub marginal_n: usize,
    /// How many of the marginal questions the shipped arm had CORRECT
    /// (each one is accuracy the density half threw away).
    pub marginal_correct: usize,
    /// Δ selective accuracy (density − shipped), and its one-sided-95%
    /// lower confidence bound (two-proportion, conservative on paired
    /// data — the cascade worthiness statistic, one home).
    pub delta_selacc: f64,
    pub delta_lcb95: f64,
    /// Pick disagreements on the doubly-answered set. MUST be 0 (the
    /// gate never touches scoring); nonzero reds the pairing premise.
    pub pick_disagreements: usize,
    /// The density arm's abstain-cause shares (the `density_gate` key is
    /// the new marginal arm's count — the why behind its abstain total).
    pub density_causes: AbstainCauses,
}

/// The shipped arm's frozen read, as plain slices (the runner's `Eval`
/// stays runner-side; only the three vectors the pairing needs cross).
pub struct ShippedArm<'a> {
    pub abstained: &'a [Vec<bool>],
    pub picks: &'a [Vec<usize>],
    pub confs: &'a [Vec<f64>],
}

/// One arm's selective walk over the frozen test cases — the runner's
/// `Eval::selective` shape, locally owned (the density arm owns its eval;
/// the shipped arm reads the caller's vectors).
fn arm_stats(
    abstained: &[Vec<bool>],
    picks: &[Vec<usize>],
    confs: &[Vec<f64>],
    cases: &[SuiteCase],
) -> ArmStats {
    let mut n = 0usize;
    let mut correct = 0usize;
    let mut pairs: Vec<(f64, bool)> = Vec::new();
    for (ci, case) in cases.iter().enumerate() {
        for (qi, _q) in case.questions.iter().enumerate() {
            if !abstained[ci][qi] {
                n += 1;
                let ok = picks[ci][qi] == case.gold[qi].idx;
                correct += usize::from(ok);
                pairs.push((confs[ci][qi], ok));
            }
        }
    }
    let total: usize = cases.iter().map(|c| c.questions.len()).sum();
    ArmStats {
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
        answered_ece: (!pairs.is_empty()).then(|| ece_of(&pairs)),
    }
}

/// Run the paired A/B. Loud `Err` on any shape drift — a misaligned
/// record would fabricate the delta exactly like a misaligned test read
/// would fabricate accuracy.
#[allow(clippy::too_many_arguments)]
pub fn density_ab_pass<const N: usize>(
    suite_name: &str,
    build: &dyn Fn(EngineConfig) -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String>,
    base_cfg: &EngineConfig,
    cal_pairs: &[CalibrationPair],
    cal_cases: &[SuiteCase],
    cal_state_strs: &[String],
    cases: &[SuiteCase],
    state_strs: &[String],
    shipped: ShippedArm<'_>,
) -> Result<DensityAbRecord, String> {
    // ── The density-armed posture: the deployed cfg verbatim + the knob.
    let mut dcfg = base_cfg.clone();
    dcfg.density_gate = true;

    // ── The density threshold fit: probe over the cal slice, ρ=30 (the
    // T1.6 posture shared with the other two axes; the gate q survives
    // only for the LAST question of a solve — the same observation law
    // the score/distance axes use).
    let (mut probe, _) = build(dcfg.clone())?;
    let mut sc = Scratch::<EMBED_DIM>::new();
    let max_q = cal_cases.iter().map(|c| c.questions.len()).max().unwrap_or(1);
    sc.prepare(max_q);
    let mut density_obs: Vec<GateObservation> = Vec::new();
    for (ci, case) in cal_cases.iter().enumerate() {
        let req = engine_request(case, &cal_state_strs[ci])?;
        probe
            .solve_into(&req, &mut sc)
            .map_err(|e| format!("density probe ({suite_name}, case {ci}): {e}"))?;
        if let (Some(slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) {
            let pick = match case.questions.last().map(|q| q.kind) {
                // Engine internal noul order is [yes, no]; gold uses the
                // wire [no, yes] convention (eval_engine's flip).
                Some(QKind::Noul) => (1 - slot.pick) as usize,
                _ => slot.pick as usize,
            };
            let correct = case.gold.last().is_some_and(|g| g.idx == pick);
            if let Some(conf) = probe.density_confidence(*dom, &sc.q) {
                density_obs.push(GateObservation {
                    score: conf,
                    correct,
                });
            }
        }
    }
    let rec = threshold_recommendation(&density_obs, Posture::Percentile { rho: 0.30 });
    let (density_threshold, fitted) = match rec {
        Some(r) => (r.threshold, true),
        // Thin support: the birth constant, disclosed (never a confident
        // number over a handful of rows — jimothy's null rule).
        None => (DEFAULT_DENSITY_THRESHOLD, false),
    };
    dcfg.density_threshold = density_threshold;
    let density_tau = dcfg.density_tau;

    // ── The density arm: same build path, same calibrator evidence, same
    // order — the ONLY delta vs the shipped engine is the density half.
    let (mut dengine, _) = build(dcfg)?;
    for p in cal_pairs {
        dengine.observe(p.conf as f32, p.correct);
    }
    let (deval, _) = eval_engine(&mut dengine, cases, state_strs, false)?;

    // ── Pairing (the superset law): per-question walk over the SAME
    // frozen read.
    if deval.abstained.len() != shipped.abstained.len() {
        return Err(format!(
            "density A/B ({suite_name}): shape drift — {} vs {} case vectors",
            deval.abstained.len(),
            shipped.abstained.len()
        ));
    }
    let mut marginal_n = 0usize;
    let mut marginal_correct = 0usize;
    let mut pick_disagreements = 0usize;
    for (ci, case) in cases.iter().enumerate() {
        for (qi, _q) in case.questions.iter().enumerate() {
            let s_abst = shipped.abstained[ci][qi];
            let d_abst = deval.abstained[ci][qi];
            // The superset law: the density arm only ADDS abstention. A
            // question the density arm answered but the shipped abstained
            // is a pairing break, not a finding.
            if !s_abst && d_abst {
                marginal_n += 1;
                marginal_correct += usize::from(shipped.picks[ci][qi] == case.gold[qi].idx);
            } else if s_abst && !d_abst {
                return Err(format!(
                    "density A/B ({suite_name}): superset law violated at case {ci} q{qi} \
                     — the density arm ANSWERED a question the shipped gate abstained"
                ));
            }
            if !s_abst && !d_abst && shipped.picks[ci][qi] != deval.picks[ci][qi] {
                pick_disagreements += 1;
            }
        }
    }

    let shipped_stats = arm_stats(shipped.abstained, shipped.picks, shipped.confs, cases);
    let density_stats = arm_stats(&deval.abstained, &deval.picks, &deval.confs, cases);
    let delta = density_stats.selective_accuracy - shipped_stats.selective_accuracy;
    let lcb = crate::harness::cascade::probe_delta_lcb95(
        density_stats.selective_accuracy,
        shipped_stats.selective_accuracy,
        density_stats.selective_n.max(shipped_stats.selective_n),
    );
    let record = DensityAbRecord {
        suite: suite_name.to_string(),
        density_threshold,
        density_tau,
        density_threshold_fitted: fitted,
        shipped: shipped_stats,
        density: density_stats,
        marginal_n,
        marginal_correct,
        delta_selacc: delta,
        delta_lcb95: lcb,
        pick_disagreements,
        density_causes: deval.abstain_causes(),
    };
    eprintln!(
        "  [density-ab] {suite_name}: threshold {density_threshold:.4} (fitted={fitted}) \
         abstain {:.3}→{:.3} selacc {:.4}→{:.4} Δ{delta:+.4} (lcb95 {:+.4}) \
         marginal {marginal_n} ({} correct) picks-disagree {pick_disagreements} causes {:?}",
        record.shipped.abstain_rate,
        record.density.abstain_rate,
        record.shipped.selective_accuracy,
        record.density.selective_accuracy,
        record.delta_lcb95,
        record.marginal_correct,
        record.density_causes,
    );
    if pick_disagreements > 0 {
        return Err(format!(
            "density A/B ({suite_name}): {pick_disagreements} pick disagreements — the \
             gate-never-touches-scoring premise broke; the numbers above are VOID"
        ));
    }
    Ok(record)
}
