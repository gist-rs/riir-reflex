//! Confidence readout dispatch — Plan 603 T1.7: INHERIT, don't re-derive.
//!
//! Bench 817's verdict binds (the `bench_817_argmax_dispatch_ab` +
//! `bench_817_structured_read_policy_arm` measurements): on a deterministic
//! forward, agreement-across-rereads is NOT a better ranking signal — the
//! analytic functionals the readout already carries (`argmax_label_prob`,
//! `label_entropy`) rank identically on single-modal subsets and diverge on
//! multi-modal ones. The engine ships BOTH, dispatched by option count:
//!
//! - **narrow** (≤ [`NARROW_MAX_OPTIONS`] options): inverted normalized
//!   label entropy `1 − H/ln K` — uses the whole distribution, which is the
//!   informative half when K is small;
//! - **wide** (> [`NARROW_MAX_OPTIONS`]): argmax-label-prob — the sharper
//!   tail signal once K grows past the point where entropy is dominated by
//!   the long tail.
//!
//! The dispatch TABLE (not the constant's value) is the contract here —
//! `regression_tests_pin_the_dispatch` pins the boundary and both arms. The
//! constant is repo-owned operating policy; changing it is a policy edit
//! with the test re-pin in the same commit, never a re-derivation of
//! Bench 817.

/// Narrow/wide dispatch bound: option counts at or below this use the
/// inverted-normalized-entropy functional; larger sets use argmax-label-prob.
pub const NARROW_MAX_OPTIONS: usize = 8;

/// Normalized Shannon entropy in nats, `H / ln K` ∈ [0, 1]. `K ≤ 1` reads
/// as fully peaked (0.0).
#[inline]
fn normalized_entropy(probs: &[f32]) -> f32 {
    let k = probs.len();
    if k <= 1 {
        return 0.0;
    }
    let ln_k = (k as f32).ln();
    let h: f32 = probs
        .iter()
        .filter(|p| **p > 0.0)
        .map(|p| -p * p.ln())
        .sum();
    (h / ln_k).clamp(0.0, 1.0)
}

/// The confidence readout for one answer distribution. Always in [0, 1].
pub fn confidence(probs: &[f32]) -> f32 {
    if probs.len() <= NARROW_MAX_OPTIONS {
        1.0 - normalized_entropy(probs)
    } else {
        probs.iter().copied().fold(0.0f32, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn regression_tests_pin_the_dispatch() {
        // A distribution that is IDENTICAL as a shape across the boundary.
        // K = 8 (narrow) → the entropy arm; K = 9 (wide) → the maxprob arm.
        let mut p8: Vec<f32> = vec![
            0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.0078125,
        ];
        let h8 = {
            let ln_k = (8.0f32).ln();
            let h: f32 = p8.iter().filter(|p| **p > 0.0).map(|p| -p * p.ln()).sum();
            (h / ln_k).clamp(0.0, 1.0)
        };
        let c8 = confidence(&p8);
        assert!(
            near(c8, 1.0 - h8),
            "K=8 must read the entropy arm (got {c8}, want {})",
            1.0 - h8
        );

        p8.push(0.0); // extend the SAME shape to K=9 — the wide arm now.
        let c9 = confidence(&p8);
        assert!(
            near(c9, 0.5),
            "K=9 must read the maxprob arm (got {c9}, want 0.5)"
        );
    }

    #[test]
    fn uniform_and_peaked_extremes() {
        let uniform2 = vec![0.5, 0.5];
        assert!(
            near(confidence(&uniform2), 0.0),
            "uniform K=2 is zero confidence"
        );
        let peaked2 = vec![1.0, 0.0];
        assert!(
            near(confidence(&peaked2), 1.0),
            "peaked K=2 is full confidence"
        );

        let uniform_wide = vec![1.0 / 64.0; 64];
        let c = confidence(&uniform_wide);
        assert!(
            near(c, 1.0 / 64.0),
            "wide uniform reads maxprob = 1/K (got {c})"
        );
    }

    #[test]
    fn bounded_unit_interval() {
        let mut p = vec![0.7, 0.2, 0.1];
        let c = confidence(&p);
        assert!((0.0..=1.0).contains(&c));
        p = vec![0.01; 16];
        let c = confidence(&p);
        assert!((0.0..=1.0).contains(&c));
    }
}
