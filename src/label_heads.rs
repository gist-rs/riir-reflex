//! Fitted per-label linear heads over the hashed-bag features (issue 030
//! lever 4 — the feature-class arc).
//!
//! WHAT: one weight row (+ bias) per domain, fitted one-vs-all at
//! engine-build time from the SAME post-cal corpus pool the drafter corpora
//! use (the cal slice is excluded upstream, so no case ever scores against
//! itself). Deterministic by construction: fixed doc order grouped by
//! domain, fixed epoch count, linearly decaying step, no shuffling, no RNG,
//! pure sequential f32 — a re-fit is bit-identical, and two engines built
//! from the same specs decide identically forever.
//!
//! WHY: the route term ranks options by the state's cosine to a domain's
//! CENTROID — a generative, per-domain signal that never looks at the other
//! domains. The fitted head is the discriminative sibling: each row is
//! pushed AWAY from every other domain's docs, which is the signal a
//! centroid cannot express (near-synonym intents — banking77's
//! `card_arrival` vs `card_delivery` — differ in their negatives, not their
//! positives). One-vs-rest SIGMOID, never softmax (the house law); the
//! blend term is `0.5 + scale·(σ(logit) − 0.5)` — the fitted model's
//! deviation from neutral, weighted by the engine's `head_scale` knob, so
//! it joins the existing σ-shaped drafter + route terms at a comparable
//! magnitude.
//!
//! Sibling precedents: the served game heads (katgpt-rs Benches 881/882 —
//! fitted heads from a digest-pinned fixture) and riir-clippy's rule_embed
//! (Bench 099). The Embedder is UNTOUCHED (issue 030's blast-radius
//! warning: a feature change there invalidates every frozen fixture,
//! game-head anchor and corpus gate — that is a separate plan with its own
//! G5-style parity).

use katgpt_core::exact_sigmoid;

/// Training epochs over the (already per-label-capped) corpus pool. Fixed,
/// not tuned on any test split — the fastText-style default for a pool this
/// size; a change here is a re-measurement, never a knob.
pub const HEAD_EPOCHS: usize = 12;
/// Initial step size (decays linearly to 0 over the whole run).
pub const HEAD_LR0: f32 = 0.5;
/// L2 decay folded into every update (`w ← w·(1 − lr·λ) − lr·g·x`).
pub const HEAD_L2: f32 = 1e-4;

/// One weight row per domain over the hashed-bag latent — the discriminative
/// sibling of the routing centroid. Fitted once at engine build; read-only
/// on the hot path.
pub struct LabelHeads<const N: usize, const D: usize> {
    /// Row-major `N × D` weights.
    w: Vec<f32>,
    /// Per-domain bias.
    b: [f32; N],
}

impl<const N: usize, const D: usize> LabelHeads<N, D> {
    /// Fit one-vs-all logistic heads from per-domain embedded rows
    /// (`rows[d]` = domain `d`'s document embeddings, unit-norm from the
    /// embedder). Sequential SGD over the docs in their given order — the
    /// determinism contract above; parallelizing this loop would trade a
    /// bit-identical fit for speed the hot path never needs (the fit runs
    /// once per engine build).
    #[must_use]
    pub fn fit(rows: &[Vec<[f32; D]>]) -> Self {
        assert_eq!(rows.len(), N, "one row set per domain");
        let n_docs: usize = rows.iter().map(Vec::len).sum();
        let steps = (HEAD_EPOCHS * n_docs).max(1) as f32;
        let mut w = vec![0.0f32; N * D];
        let mut b = [0.0f32; N];
        let mut t = 0.0f32;
        for _ in 0..HEAD_EPOCHS {
            for (label, docs) in rows.iter().enumerate() {
                for x in docs {
                    let lr = HEAD_LR0 * (1.0 - t / steps);
                    t += 1.0;
                    let decay = 1.0 - lr * HEAD_L2;
                    for (d, row) in w.as_chunks_mut::<D>().0.iter_mut().enumerate() {
                        let bias = &mut b[d];
                        // Forward: one row dot against the shared doc.
                        let mut z = *bias;
                        for (wv, &xv) in row.iter().zip(x.iter()) {
                            z += wv * xv;
                        }
                        // One-vs-rest sigmoid gradient (never softmax).
                        let g = exact_sigmoid(z) - if d == label { 1.0 } else { 0.0 };
                        let step = lr * g;
                        for (i, wv) in row.iter_mut().enumerate() {
                            *wv = *wv * decay - step * x[i];
                        }
                        *bias -= step;
                    }
                }
            }
        }
        Self { w, b }
    }

    /// The raw fitted logit for domain `d` at query `q` (`w_d·q + b_d`).
    #[must_use]
    pub fn raw_logit(&self, d: usize, q: &[f32; D]) -> f32 {
        let row = &self.w[d * D..(d + 1) * D];
        let mut z = self.b[d];
        for (wv, &xv) in row.iter().zip(q.iter()) {
            z += wv * xv;
        }
        z
    }

    /// The engine blend term: the fitted model's deviation from neutral,
    /// weighted by the caller's scale — `0.5 + scale·(σ(logit) − 0.5)`.
    /// Scale 0 would be the constant 0.5; the engine skips the term
    /// entirely at 0 instead (byte-identical to the pre-head posture).
    #[must_use]
    pub fn blend_term(&self, d: usize, q: &[f32; D], scale: f32) -> f32 {
        0.5 + scale * (exact_sigmoid(self.raw_logit(d, q)) - 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{Embedder, EMBED_DIM};

    fn corpus() -> Vec<Vec<[f32; EMBED_DIM]>> {
        let domains: [&[&str]; 3] = [
            &[
                "refund the customer invoice balance",
                "billing account charged twice refund",
                "invoice payment refund request",
            ],
            &[
                "deploy the server to staging rollout",
                "staging cluster release candidate deploy",
                "verify the deployment health rollout",
            ],
            &[
                "the quest monster drops loot when defeated",
                "monster spawn loot table defeat",
                "loot drops from the defeated monster",
            ],
        ];
        domains
            .iter()
            .map(|docs| {
                docs.iter()
                    .map(|d| {
                        let mut v = [0.0f32; EMBED_DIM];
                        Embedder.embed_into(d.as_bytes(), &mut v);
                        v
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn fit_is_bit_identical_across_calls() {
        let rows = corpus();
        let a = LabelHeads::<3, EMBED_DIM>::fit(&rows);
        let b = LabelHeads::<3, EMBED_DIM>::fit(&rows);
        assert_eq!(
            a.w.len(),
            b.w.len(),
            "same shape is the precondition of the bit compare"
        );
        for (x, y) in a.w.iter().zip(b.w.iter()) {
            assert_eq!(x.to_bits(), y.to_bits(), "weights must refit bit-identically");
        }
        for (x, y) in a.b.iter().zip(b.b.iter()) {
            assert_eq!(x.to_bits(), y.to_bits(), "biases must refit bit-identically");
        }
    }

    #[test]
    fn separable_corpus_fits_to_its_own_labels() {
        let rows = corpus();
        let heads = LabelHeads::<3, EMBED_DIM>::fit(&rows);
        for (label, docs) in rows.iter().enumerate() {
            for x in docs {
                let best = (0..3)
                    .map(|d| heads.raw_logit(d, x))
                    .enumerate()
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(d, _)| d)
                    .expect("3 domains");
                assert_eq!(best, label, "fitted head must rank its own domain first");
            }
        }
    }

    #[test]
    fn blend_term_is_neutral_bounded_and_monotone_in_scale() {
        let rows = corpus();
        let heads = LabelHeads::<3, EMBED_DIM>::fit(&rows);
        let q = rows[1][0];
        for scale in [0.5f32, 1.0, 2.0] {
            let t = heads.blend_term(1, &q, scale);
            assert!(
                (t - 0.5).abs() <= scale / 2.0 + 1e-6,
                "term must stay within scale/2 of neutral (scale {scale}, term {t})"
            );
            // The winning domain's term moves UP with scale; a loser's moves DOWN.
            let loser = heads.blend_term(0, &q, scale);
            assert!(t > loser, "the fitted winner must out-rank a loser");
        }
        let zero = heads.blend_term(1, &q, 0.0);
        assert!((zero - 0.5).abs() < 1e-7, "scale 0 is exactly neutral");
    }
}
