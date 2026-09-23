//! The threshold-recommendation surface (Issue 009 — the jimothy steal:
//! the per-task abstention cutoff as fit-time engine metadata).
//!
//! jimothy (`github.com/AndrewPrifer/jimothy` — the adjacent TRAINED
//! per-task classifier) ships the CONTRACT this module steals, not the
//! method:
//!
//! 1. **Recommend at fit time** — the cutoff is computed once from a
//!    labeled cal slice ([`threshold_recommendation`]), never hardcoded
//!    and never re-derived per call.
//! 2. **Return everything at run time** — the engine never silently drops
//!    to abstain; applying a cutoff stays the consumer's choice (the
//!    harness posture: fitted thresholds ride the result row, the engine
//!    keeps answering).
//! 3. **Null on thin support** — [`None`] (jimothy's `null`) when the cal
//!    slice is smaller than [`THIN_SUPPORT_FLOOR`], and every
//!    recommendation DISCLOSES its support ([`ThresholdSupport`]) — the
//!    percentile/tail-support law: never a confident number over 3 rows.
//! 4. **Validate in the deployment environment** — a threshold fitted under
//!    one corpus/runtime posture is a claim with a timestamp on it; the
//!    metadata carries the [`Posture`] so consumers cite it alongside the
//!    number.
//!
//! Modelless by construction: the fit is a deterministic scan over observed
//! sigmoid-gate scores (no RNG, no trained calibrator — jimothy's method is
//! the adjacent baseline named for comparison, never adopted).
//!
//! ## Percentile parity (the T2 migration bar)
//!
//! [`Posture::Percentile`] reproduces the harness runner's `quantile` law
//! VERBATIM — ascending `total_cmp` sort, `idx = floor(n·ρ)` clamped to
//! `n-1`, take `s[idx]` — so migrating the harness to this surface
//! (Issue 009 T2) is byte-identical on every current suite. The law's known
//! small-n coarseness (`idx` can land on `n-1`, the MAX, for high ρ at
//! small n) is not "fixed" here: fixing it would move published numbers,
//! which is a re-fit, not a migration. The [`ThresholdSupport`] disclosure
//! is the honest answer.
//!
//! NaN scores: `total_cmp` is a total order (NaN sorts above `+inf`), and a
//! NaN score never passes a threshold (`NaN >= t` is false) — NaN rows
//! deterministically abstain and are counted in
//! [`ThresholdSupport::n_abstain`].

use serde::{Deserialize, Serialize};

/// Minimum labeled observations before a recommendation exists (jimothy's
/// null rule). 16 is the floor the T1.5 harness measured in as its
/// `confs.len() < 16` fallback to the birth constants — frozen here so the
/// surface and the harness agree on what "thin" means.
pub const THIN_SUPPORT_FLOOR: usize = 16;

/// One labeled cal-slice observation: the gate score the cutoff applies to,
/// and whether the engine's answer on that case was correct. Percentile
/// postures select on `score` only; `correct` rides along as the accuracy
/// disclosure ([`ThresholdRecommendation::target_accuracy`]); the
/// target-accuracy posture selects on both.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GateObservation {
    /// The observed gate score (calibrated confidence for the score axis,
    /// corpus-distance confidence for the distance axis).
    pub score: f32,
    /// Whether the engine's top answer on the cal case was correct.
    pub correct: bool,
}

/// The fit posture: which deterministic selection the scan performs. Rides
/// the metadata (contract part 4 — cite the posture with the number).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "posture", rename_all = "camelCase")]
pub enum Posture {
    /// The arena posture (T1.6, ρ = 30%): threshold at the ρ-quantile of
    /// the observed scores — ρ of in-distribution questions abstain, the
    /// rest pass. Reproduces the harness `quantile` law exactly (the T2
    /// migration bar).
    Percentile {
        /// Target abstain rate, `0.0 <= rho <= 1.0`.
        rho: f64,
    },
    /// jimothy's shape: the LOWEST cutoff whose above-cutoff accuracy on
    /// the cal slice meets `target` (max answered volume at target
    /// quality). When no cutoff meets it, the best-achievable cutoff is
    /// returned with [`RecommendationStatus::TargetUnmet`] — disclosed,
    /// never silent.
    TargetAccuracy {
        /// Target accuracy, `0.0 < target <= 1.0`.
        target: f64,
    },
}

/// Whether the posture's target was reachable (percentile postures have no
/// accuracy target and are always [`RecommendationStatus::Fitted`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecommendationStatus {
    /// The posture's selection held (percentile fit, or a target that was
    /// met).
    Fitted,
    /// No cutoff achieved the target — the recommendation is the
    /// best-achievable cutoff and `target_accuracy` reports what it
    /// achieves.
    TargetUnmet,
}

/// Support disclosure (the tail-support law): what the fit actually saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdSupport {
    /// Observations the fit saw (`n >= THIN_SUPPORT_FLOOR` or the fit is
    /// `None`).
    pub n: usize,
    /// Observations at or above the recommended threshold (would pass).
    pub n_pass: usize,
    /// Observations below it (would abstain).
    pub n_abstain: usize,
}

/// A fit-time cutoff recommendation (jimothy's
/// `metadata.thresholdRecommendation` shape; [`None`] is his `null`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdRecommendation {
    /// The recommended cutoff: `score >= threshold` passes, below abstains.
    pub threshold: f32,
    /// [`RecommendationStatus::Fitted`] /
    /// [`RecommendationStatus::TargetUnmet`].
    pub status: RecommendationStatus,
    /// The realized accuracy at-or-above the cutoff on the cal slice — for
    /// a met target posture this is `>= target`; percentile postures
    /// report the accuracy their cutoff happens to keep (a disclosure,
    /// never a selection criterion).
    pub target_accuracy: f32,
    /// What the fit saw.
    pub support: ThresholdSupport,
}

/// The fused-gate pair (both axes) under one posture — the shape the
/// harness fits (Issue 009 T2's entry point) and the metadata surface
/// cites: `thresholdRecommendation: { posture, score, distance }`, `null`
/// per axis on thin support.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FusedGateRecommendation {
    /// The posture that produced both numbers (cite it with them — contract
    /// part 4).
    pub posture: Posture,
    /// Score-axis recommendation (calibrated-confidence cutoff).
    pub score: Option<ThresholdRecommendation>,
    /// Distance-axis recommendation (corpus-distance-confidence cutoff).
    pub distance: Option<ThresholdRecommendation>,
}

/// Recommend a cutoff for one gate axis from a labeled cal slice.
///
/// Deterministic: no RNG, no time, no ambient state — the same observations
/// and posture always produce a bit-identical recommendation (pinned by
/// gate). Fit-time only; never a per-call cost.
pub fn threshold_recommendation(
    observations: &[GateObservation],
    posture: Posture,
) -> Option<ThresholdRecommendation> {
    // jimothy's null rule — thin support is disclosed by ABSENCE, never a
    // confident number over a handful of rows.
    if observations.len() < THIN_SUPPORT_FLOOR {
        return None;
    }
    match posture {
        Posture::Percentile { rho } => {
            assert!(
                (0.0..=1.0).contains(&rho),
                "percentile posture requires rho in [0, 1]"
            );
            let threshold = percentile_law(observations, rho);
            Some(finish(observations, threshold, RecommendationStatus::Fitted))
        }
        Posture::TargetAccuracy { target } => {
            assert!(
                target > 0.0 && target <= 1.0,
                "target-accuracy posture requires target in (0, 1]"
            );
            let (threshold, status) = target_scan(observations, target);
            Some(finish(observations, threshold, status))
        }
    }
}

/// Fit both fused-gate axes under one posture (the harness T2 entry point).
///
/// The axes are fitted independently (the T1.5 law: score threshold from
/// calibrated confidences, distance threshold from corpus-distance
/// confidences, both at the same ρ); each axis carries its own thin-support
/// null.
pub fn recommend_fused_gate(
    score_obs: &[GateObservation],
    distance_obs: &[GateObservation],
    posture: Posture,
) -> FusedGateRecommendation {
    FusedGateRecommendation {
        posture,
        score: threshold_recommendation(score_obs, posture),
        distance: threshold_recommendation(distance_obs, posture),
    }
}

/// The harness runner's `quantile`, frozen verbatim (the T2 migration bar —
/// see the module doc). Callers guarantee `n >= 1` and `0 <= rho <= 1`.
fn percentile_law(observations: &[GateObservation], rho: f64) -> f32 {
    let mut s: Vec<f32> = observations.iter().map(|o| o.score).collect();
    s.sort_by(|a, b| a.total_cmp(b));
    let idx = ((s.len() as f64) * rho) as usize;
    s[idx.min(s.len() - 1)]
}

/// Deterministic target-accuracy scan: sweep DISTINCT score boundaries in
/// descending order; the pass set (`score >= t`) grows as the threshold
/// drops. Returns the lowest threshold whose above-cutoff accuracy still
/// meets `target`; when none does, the best-achievable threshold (accuracy
/// argmax, ties broken toward the larger pass set). O(B·n) over B distinct
/// boundaries — fit-time only, cal slices are tens of rows.
fn target_scan(observations: &[GateObservation], target: f64) -> (f32, RecommendationStatus) {
    let mut boundaries: Vec<f32> = observations.iter().map(|o| o.score).collect();
    boundaries.sort_by(|a, b| b.total_cmp(a));
    boundaries.dedup();
    let mut chosen: Option<f32> = None;
    let mut best: Option<(f32, f32)> = None; // (threshold, accuracy)
    for &t in &boundaries {
        let (n_pass, n_correct) = observations
            .iter()
            .fold((0usize, 0usize), |(p, c), o| {
                if o.score >= t {
                    (p + 1, c + o.correct as usize)
                } else {
                    (p, c)
                }
            });
        // n_pass >= 1 at every boundary: the boundary value itself passes.
        let acc = n_correct as f32 / n_pass as f32;
        if acc >= target as f32 {
            chosen = Some(t); // keep overwriting → the LOWEST meeting threshold
        }
        match best {
            Some((_, best_acc)) if acc < best_acc => {}
            // equal accuracy → prefer the LOWER threshold (larger pass set)
            _ => best = Some((t, acc)),
        }
    }
    chosen
        .map(|t| (t, RecommendationStatus::Fitted))
        .unwrap_or_else(|| {
            let (t, _) = best.expect("at least one distinct score exists");
            (t, RecommendationStatus::TargetUnmet)
        })
}

/// Recompute support and realized accuracy AT the threshold (tie-inclusive:
/// every observation with `score >= threshold` passes, including duplicates
/// of the boundary value) — the disclosed numbers always match the actual
/// threshold semantics, never the scan's prefix arithmetic.
fn finish(
    observations: &[GateObservation],
    threshold: f32,
    status: RecommendationStatus,
) -> ThresholdRecommendation {
    let mut n_pass = 0usize;
    let mut n_correct = 0usize;
    for o in observations {
        if o.score >= threshold {
            n_pass += 1;
            n_correct += o.correct as usize;
        }
    }
    let target_accuracy = if n_pass > 0 {
        n_correct as f32 / n_pass as f32
    } else {
        0.0
    };
    ThresholdRecommendation {
        threshold,
        status,
        target_accuracy,
        support: ThresholdSupport {
            n: observations.len(),
            n_pass,
            n_abstain: observations.len() - n_pass,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic labeled observations (no RNG dep, no seed drift).
    fn obs_from_scores(scores: &[f32], correct_above: f32) -> Vec<GateObservation> {
        scores
            .iter()
            .enumerate()
            .map(|(i, &s)| GateObservation {
                score: s,
                correct: (s / 100.0) >= correct_above || (i % 7 == 0),
            })
            .collect()
    }

    /// Distinct descending scores 100, 99, ... (n rows).
    fn distinct_scores(n: usize) -> Vec<f32> {
        (0..n).map(|i| 100.0 - i as f32).collect()
    }

    #[test]
    fn thin_support_is_null_below_floor() {
        let floor_pin = 16; // the T1.5 harness fallback (`confs.len() < 16`)
        assert_eq!(THIN_SUPPORT_FLOOR, floor_pin);
        let thin = obs_from_scores(&distinct_scores(15), 0.5);
        let enough = obs_from_scores(&distinct_scores(16), 0.5);
        assert!(threshold_recommendation(&thin, Posture::Percentile { rho: 0.30 }).is_none());
        assert!(threshold_recommendation(&thin, Posture::TargetAccuracy { target: 0.9 }).is_none());
        assert!(threshold_recommendation(&enough, Posture::Percentile { rho: 0.30 }).is_some());
        assert!(threshold_recommendation(&enough, Posture::TargetAccuracy { target: 0.1 }).is_some());
    }

    #[test]
    fn determinism_bit_identical() {
        let obs = obs_from_scores(&distinct_scores(50), 0.6);
        for posture in [
            Posture::Percentile { rho: 0.30 },
            Posture::TargetAccuracy { target: 0.7 },
        ] {
            let a = threshold_recommendation(&obs, posture).unwrap();
            let b = threshold_recommendation(&obs, posture).unwrap();
            assert_eq!(a, b);
            assert_eq!(a.threshold.to_bits(), b.threshold.to_bits());
            assert_eq!(a.target_accuracy.to_bits(), b.target_accuracy.to_bits());
        }
    }

    #[test]
    fn percentile_parity_with_the_harness_quantile_law() {
        // The harness runner's `quantile`, replicated verbatim — the T2
        // migration bar pinned from the T1 side. If either side drifts,
        // this reds.
        let harness_quantile = |sample: &[f32], q: f64| -> f32 {
            let mut s = sample.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            let idx = ((s.len() as f64) * q) as usize;
            s[idx.min(s.len() - 1)]
        };
        // Deterministic mixed populations (duplicates + spread), not just
        // ladders — ties and clamping are where quantile laws diverge.
        let cases: Vec<Vec<f32>> = vec![
            distinct_scores(16),
            distinct_scores(31),
            distinct_scores(100),
            (0..64)
                .map(|i| ((i * 37) % 13) as f32 + if i % 3 == 0 { 0.5 } else { 0.0 })
                .collect(),
        ];
        for scores in cases {
            let obs = obs_from_scores(&scores, 0.5);
            for rho in [0.30, 0.5, 0.99] {
                let got =
                    threshold_recommendation(&obs, Posture::Percentile { rho }).unwrap().threshold;
                let want = harness_quantile(&scores, rho);
                assert_eq!(got.to_bits(), want.to_bits(), "n={}", scores.len());
            }
        }
    }

    #[test]
    fn percentile_support_discloses_the_posture() {
        // Distinct scores → n_abstain is exactly the quantile index
        // floor(n·ρ) — the posture's realized abstain rate, disclosed.
        for n in [16usize, 31, 50] {
            let obs = obs_from_scores(&distinct_scores(n), 0.5);
            let r = threshold_recommendation(&obs, Posture::Percentile { rho: 0.30 }).unwrap();
            assert_eq!(r.support.n, n);
            assert_eq!(r.support.n_pass + r.support.n_abstain, n);
            assert_eq!(r.support.n_abstain, (n as f64 * 0.30) as usize);
            assert!(r.support.n_pass >= 1);
            assert_eq!(r.status, RecommendationStatus::Fitted);
        }
    }

    #[test]
    fn target_met_picks_lowest_meeting_cutoff() {
        // 10 correct (high scores) → 10 wrong → 10 correct → 6 wrong tail.
        // At target 0.8 the meeting boundaries are k=10 (acc 1.0, t=81),
        // k=11 (0.909, t=70), k=12 (0.833, t=69); k=13 (0.769) and
        // everything deeper fall short — the recommendation is the LOWEST
        // meeting cutoff (k=12, threshold 69.0, the 12th distinct score
        // descending: the score series jumps 81 → 70 between blocks).
        let mut scores: Vec<f32> = Vec::new();
        for i in 0..10 {
            scores.push(90.0 - i as f32); // correct block
        }
        for i in 0..10 {
            scores.push(70.0 - i as f32); // wrong block
        }
        for i in 0..10 {
            scores.push(50.0 - i as f32); // correct block
        }
        for i in 0..6 {
            scores.push(30.0 - i as f32); // wrong tail (n = 36 ≥ floor)
        }
        let obs: Vec<GateObservation> = scores
            .iter()
            .map(|&s| GateObservation {
                score: s,
                correct: s > 80.0 || (40.0..60.0).contains(&s),
            })
            .collect();
        let r = threshold_recommendation(&obs, Posture::TargetAccuracy { target: 0.8 }).unwrap();
        assert_eq!(r.status, RecommendationStatus::Fitted);
        assert_eq!(r.support.n_pass, 12);
        assert!((r.target_accuracy - 10.0 / 12.0).abs() < 1e-6);
        assert_eq!(r.threshold, 69.0);
    }

    #[test]
    fn target_unmet_returns_best_achievable() {
        // All wrong: no cutoff reaches 0.5 — the best-achievable accuracy
        // is 0.0 at every boundary, tie broken toward the largest pass set
        // (the lowest threshold).
        let scores = distinct_scores(20);
        let obs: Vec<GateObservation> = scores
            .iter()
            .map(|&s| GateObservation {
                score: s,
                correct: false,
            })
            .collect();
        let r = threshold_recommendation(&obs, Posture::TargetAccuracy { target: 0.5 }).unwrap();
        assert_eq!(r.status, RecommendationStatus::TargetUnmet);
        assert_eq!(r.target_accuracy, 0.0);
        assert_eq!(r.support.n_pass, obs.len());
        assert_eq!(r.support.n_abstain, 0);
    }

    #[test]
    fn ties_at_threshold_pass_together() {
        // All-equal scores: ρ=0.30 lands on the shared value, everything
        // passes (>= threshold), nothing abstains, accuracy is the honest
        // population rate.
        let scores = [0.6f32; 24];
        let obs: Vec<GateObservation> = scores
            .iter()
            .enumerate()
            .map(|(i, &s)| GateObservation {
                score: s,
                correct: i % 2 == 0,
            })
            .collect();
        let r = threshold_recommendation(&obs, Posture::Percentile { rho: 0.30 }).unwrap();
        assert_eq!(r.threshold, 0.6);
        assert_eq!(r.support.n_pass, 24);
        assert_eq!(r.support.n_abstain, 0);
        assert_eq!(r.target_accuracy, 0.5);
    }

    #[test]
    fn nan_rows_abstain_deterministically() {
        let mut scores = distinct_scores(18);
        scores[3] = f32::NAN;
        scores[17] = f32::NAN;
        let obs = obs_from_scores(&scores, 0.5);
        let r = threshold_recommendation(&obs, Posture::Percentile { rho: 0.30 }).unwrap();
        assert_eq!(r.support.n_pass + r.support.n_abstain, r.support.n);
        // NaN sorts above +inf under total_cmp but never passes a cutoff —
        // both NaN rows sit in the abstain side deterministically.
        assert_eq!(r.support.n_abstain, 2 + (18.0f64 * 0.30) as usize);
    }

    #[test]
    fn serde_metadata_shape_matches_jimothy() {
        // Numbers only, camelCase — the `metadata.thresholdRecommendation`
        // shape downstream tables + the arena site cite verbatim.
        let obs = obs_from_scores(&distinct_scores(20), 0.5);
        let fused = recommend_fused_gate(&obs, &obs, Posture::Percentile { rho: 0.30 });
        let json = serde_json::to_string(&fused).unwrap();
        assert!(json.contains("\"posture\":\"percentile\""));
        assert!(json.contains("\"rho\":0.3"));
        assert!(json.contains("\"targetAccuracy\""));
        assert!(json.contains("\"nPass\""));
        assert!(json.contains("\"nAbstain\""));
        // The null rule renders as real JSON null per axis.
        let thin = obs_from_scores(&distinct_scores(3), 0.5);
        let fused_thin = recommend_fused_gate(&thin, &thin, Posture::Percentile { rho: 0.30 });
        let json_thin = serde_json::to_string(&fused_thin).unwrap();
        assert!(json_thin.contains("\"score\":null"));
        assert!(json_thin.contains("\"distance\":null"));
    }
}
