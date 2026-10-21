//! Plan 627 T3 / reflex Issue 090 C2 — the `set_refine` paired A/B arm
//! (report-only POC; `--set-refine-ab`, needs the `set_refine` feature).
//!
//! The engine seam (reflex `engine.rs`, feature `set_refine`) refines the
//! state-alone embedding against the question's own option set before the
//! route/head terms derive from it — the set-dependence an independent
//! per-option scorer structurally lacks (Jeva Prop 1, arXiv:2610.18773).
//! THIS pass prices it honestly: it rebuilds the engine at the SAME
//! fitted posture with the knob armed (`set_refine_gamma` from the CLI,
//! everything else verbatim), re-fits the calibrator on the SAME cal
//! pairs in the same order, reads the SAME frozen test slice ONCE, and
//! pairs per-question against the shipped (unarmed) row's read.
//!
//! # The pairing law (the OPPOSITE of the density arm's)
//!
//! `set_refine` deliberately CHANGES SCORES — that is its entire job —
//! so picks MAY flip between the arms (there is no superset law to
//! enforce). Every flip is decomposed: `flips_to_correct` /
//! `flips_to_wrong` over the doubly-answered set, plus the accuracy
//! delta with a one-sided-95% paired lower bound. A flip is not a
//! defect here; it is the measurement.
//!
//! Report-only by law: the published row stays the shipped posture's;
//! adoption is a separate GOAT verdict (Plan 627 §4 — G1 real-suite gain,
//! G2 latency, G3 flag-off identity, G4 alloc), never a default change
//! riding a measurement pass. Noul and drafter-only questions are
//! byte-identical by the seam's own legality scope (issue 030) — their
//! paired rows necessarily agree; they still count toward the accuracy
//! base (they move nothing).
//!
//! # Index-space law
//!
//! Engine noul candidate order is `[yes=0, no=1]`; harness gold is
//! `[no=0, yes=1]`. `eval_engine` flips noul indices into gold space
//! (picks are comparable directly — the same flip both arms).

use crate::embed::EMBED_DIM;
use crate::engine::{DecisionEngine, EngineConfig};
use crate::harness::metrics::{CalibrationPair, ece_of};
use crate::harness::runner::eval_engine;
use crate::harness::suites::SuiteCase;
use serde::Serialize;

/// One arm's summary (the shipped/unarmed read vs the refined arm's).
#[derive(Debug, Clone, Serialize)]
pub struct SetRefineArmStats {
    /// Forced-pick accuracy over ALL questions (picks exist regardless of
    /// the fused gate — the like-for-like comparison surface).
    pub accuracy: f64,
    /// The fused gate's abstain rate at this arm's own calibrated
    /// confidence (the knob moves scores, so the gate can fire
    /// differently — disclosed, never equalized).
    pub abstain_rate: f64,
    /// Readout ECE over the arm's ANSWERED questions. `None` on an empty
    /// answered set.
    pub answered_ece: Option<f64>,
}

/// The Plan-627 A/B record (rides the modelless row, `set_refine_ab`).
#[derive(Debug, Clone, Serialize)]
pub struct SetRefineAbRecord {
    pub suite: String,
    /// The knob posture this record measured.
    pub gamma: f32,
    pub tau: f32,
    pub shipped: SetRefineArmStats,
    pub refined: SetRefineArmStats,
    /// Total paired questions.
    pub n: usize,
    /// Questions whose PICK moved (refined != shipped) on the doubly-
    /// answered set — decomposed by correctness outcome:
    pub flips: usize,
    /// shipped wrong → refined correct (the wins).
    pub flips_to_correct: usize,
    /// shipped correct → refined wrong (the losses).
    pub flips_to_wrong: usize,
    /// Both wrong but differently (no accuracy effect).
    pub flips_wrong_to_wrong: usize,
    /// Δ accuracy (refined − shipped), all questions, forced picks.
    pub delta_acc: f64,
    /// One-sided-95% paired lower bound on the accuracy delta (the
    /// cascade worthiness statistic — same home as the density arm).
    pub delta_lcb95: f64,
}

fn arm_stats(
    picks: &[Vec<usize>],
    abstained: &[Vec<bool>],
    confs: &[Vec<f64>],
    cases: &[SuiteCase],
) -> SetRefineArmStats {
    let mut n = 0usize;
    let mut correct = 0usize;
    let mut abstained_n = 0usize;
    let mut total = 0usize;
    let mut pairs: Vec<(f64, bool)> = Vec::new();
    for (ci, case) in cases.iter().enumerate() {
        for (qi, _q) in case.questions.iter().enumerate() {
            total += 1;
            if abstained[ci][qi] {
                abstained_n += 1;
            } else {
                pairs.push((confs[ci][qi], picks[ci][qi] == case.gold[qi].idx));
            }
            // Forced-pick accuracy counts EVERY question (the A/B's
            // like-for-like surface — the gate's abstention is a separate
            // disclosed axis, not an accuracy exclusion).
            n += 1;
            correct += usize::from(picks[ci][qi] == case.gold[qi].idx);
        }
    }
    SetRefineArmStats {
        accuracy: if n > 0 {
            correct as f64 / n as f64
        } else {
            0.0
        },
        abstain_rate: if total > 0 {
            abstained_n as f64 / total as f64
        } else {
            0.0
        },
        answered_ece: (!pairs.is_empty()).then(|| ece_of(&pairs)),
    }
}

/// Run the paired A/B. Loud `Err` on any shape drift — a misaligned
/// record would fabricate the delta exactly like a misaligned test read
/// would fabricate accuracy.
#[allow(clippy::too_many_arguments)]
pub fn set_refine_ab_pass<const N: usize>(
    suite_name: &str,
    build: &dyn Fn(EngineConfig) -> Result<(DecisionEngine<N, EMBED_DIM>, Vec<String>), String>,
    base_cfg: &EngineConfig,
    gamma: f32,
    tau: f32,
    cal_pairs: &[CalibrationPair],
    cases: &[SuiteCase],
    state_strs: &[String],
    shipped_picks: &[Vec<usize>],
    shipped_abstained: &[Vec<bool>],
    shipped_confs: &[Vec<f64>],
) -> Result<SetRefineAbRecord, String> {
    // ── The refined posture: the deployed cfg verbatim + the knob armed.
    let mut rcfg = base_cfg.clone();
    rcfg.set_refine_gamma = gamma;
    rcfg.set_refine_tau = tau;

    // ── The refined arm: same build path, same calibrator evidence in
    // the same order — the ONLY delta vs the shipped engine is the knob.
    let (mut rengine, _) = build(rcfg)?;
    for p in cal_pairs {
        rengine.observe(p.conf as f32, p.correct);
    }
    let (reval, _) = eval_engine(&mut rengine, cases, state_strs, false)?;

    // ── Shape guard before any pairing.
    if reval.picks.len() != shipped_picks.len() {
        return Err(format!(
            "set_refine A/B ({suite_name}): shape drift — {} vs {} case vectors",
            reval.picks.len(),
            shipped_picks.len()
        ));
    }

    // ── Pairing: per-question walk over the SAME frozen read. Picks MAY
    // flip — that is the measurement; every flip is decomposed.
    let mut n = 0usize;
    let mut flips = 0usize;
    let mut flips_to_correct = 0usize;
    let mut flips_to_wrong = 0usize;
    let mut flips_wrong_to_wrong = 0usize;
    for (ci, case) in cases.iter().enumerate() {
        for (qi, _q) in case.questions.iter().enumerate() {
            n += 1;
            let s_ok = shipped_picks[ci][qi] == case.gold[qi].idx;
            let r_ok = reval.picks[ci][qi] == case.gold[qi].idx;
            if shipped_picks[ci][qi] != reval.picks[ci][qi] {
                flips += 1;
                match (s_ok, r_ok) {
                    (false, true) => flips_to_correct += 1,
                    (true, false) => flips_to_wrong += 1,
                    _ => flips_wrong_to_wrong += 1,
                }
            }
        }
    }

    let shipped_stats = arm_stats(shipped_picks, shipped_abstained, shipped_confs, cases);
    let refined_stats = arm_stats(&reval.picks, &reval.abstained, &reval.confs, cases);
    let delta = refined_stats.accuracy - shipped_stats.accuracy;
    let lcb = crate::harness::cascade::probe_delta_lcb95(
        refined_stats.accuracy,
        shipped_stats.accuracy,
        n,
    );
    let record = SetRefineAbRecord {
        suite: suite_name.to_string(),
        gamma,
        tau,
        shipped: shipped_stats,
        refined: refined_stats,
        n,
        flips,
        flips_to_correct,
        flips_to_wrong,
        flips_wrong_to_wrong,
        delta_acc: delta,
        delta_lcb95: lcb,
    };
    eprintln!(
        "  [set-refine-ab] {suite_name}: γ={gamma} τ={tau} acc {:.4}→{:.4} Δ{delta:+.4} \
         (lcb95 {:+.4}) flips {flips} (+{flips_to_correct} correct / −{flips_to_wrong} wrong / \
         {flips_wrong_to_wrong} w↔w) abstain {:.3}→{:.3}",
        record.shipped.accuracy,
        record.refined.accuracy,
        record.delta_lcb95,
        record.shipped.abstain_rate,
        record.refined.abstain_rate,
    );
    Ok(record)
}
