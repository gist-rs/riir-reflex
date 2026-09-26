//! Per-label naive-Bayes log-odds scorer (issue 038 T1, opt-in `nb_scope`).
//!
//! WHAT: one `katgpt_core::contrastive_scope::ContrastiveScoreTable` per
//! domain, built one-vs-rest at engine-build time (the domain's docs are
//! the in-scope corpus, every other domain's docs the out-scope corpus)
//! over hashed unigram + bigram ids ([`crate::embed::hashed_tokens_into`],
//! the embedder's own lexicon at `NB_VOCAB` width instead of 256). A
//! state's in-scope score for domain `d` is `−D_d(x)` — the table's
//! smoothed log2-odds summed over the state's tokens (positive ⇒ reads
//! like `d`). Frozen counts, no gradient, no iteration: the modelless
//! count-table class (the corpus is still the model; this is a second
//! projection of it, beside the drafter and the centroid).
//!
//! WHY: the Bench 003 cap sweep measured the drafter + centroid path
//! getting WORSE with more corpus (ag_news .51 at 64/label → .46 at 512),
//! and the 256-bucket embed collides. A count table is the estimator that
//! improves monotonically with evidence and reads a vocabulary 512× wider.
//! The issue-038 POC measured plain NB on the fetched train rows at ag_news
//! .868 / emotion .570 / sst5 .387 (modelless today .510 / .282 / .217).
//!
//! BLEND: per option `nb_scale · σ(margin_d / n_tokens)` where
//! `margin_d = in_d − max_{e≠d} in_e` over ALL domains (bits per state
//! token). σ is monotone in `in_d` for every `d`, so the term ranks any
//! offered option subset exactly as the NB argmax would; the per-token
//! normalization keeps a long state from saturating σ (a saturated tie
//! would fall back to index order — the Issue 004 T7 constant-pick
//! class). Sigmoid, never softmax (the house law).
//!
//! SMOOTHING: [`NbAlpha::ObservedLaplace`] sets `α = observed / NB_VOCAB`
//! so the smoothing mass `α·V` equals the number of buckets the training
//! docs actually touch — Laplace over the observed vocabulary, the
//! textbook form. A fixed α over a hashed width charges smoothing mass for
//! 131k buckets nobody observed and flattens every table (measured in the
//! POC; disclosed in the bench because that probe read the test split —
//! the harness SELECTS between the rule and a fixed α on the cal slice,
//! never on test).

use crate::embed::hashed_tokens_into;
use katgpt_core::contrastive_scope::{ContrastiveScoreBuilder, ContrastiveScoreTable, scope_score};
use katgpt_core::exact_sigmoid;

/// Hashed vocabulary width (2^17). Wide enough that unigram + bigram
/// collisions are rare at suite scale; one `f32` row per domain
/// (512 KiB) — banking77's 77 domains ≈ 39 MiB, a build-time cost.
pub const NB_VOCAB: usize = 1 << 17;

/// Smoothing policy for the one-vs-rest tables.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NbAlpha {
    /// `α = observed_buckets / NB_VOCAB` — Laplace over the observed
    /// vocabulary (the default).
    ObservedLaplace,
    /// A fixed per-bucket pseudo-count.
    Fixed(f32),
}

/// The fitted per-domain tables. Immutable after the build; read-only on
/// the hot path.
pub struct NbScope {
    tables: Vec<ContrastiveScoreTable>,
    alpha: f32,
    observed: usize,
}

impl NbScope {
    /// Fit one-vs-rest tables from per-domain documents (`domains[d]` =
    /// domain `d`'s docs). Deterministic: fixed doc order, integer counts,
    /// no RNG — a re-fit is bit-identical.
    #[must_use]
    pub fn fit(domains: &[&[String]], alpha: NbAlpha) -> Self {
        let mut buf: Vec<u32> = Vec::new();
        let mut toks: Vec<Vec<Vec<u32>>> = Vec::with_capacity(domains.len());
        let mut seen = vec![false; NB_VOCAB];
        let mut observed = 0usize;
        for docs in domains {
            let mut rows = Vec::with_capacity(docs.len());
            for doc in *docs {
                hashed_tokens_into(doc.as_bytes(), NB_VOCAB, &mut buf);
                for &w in &buf {
                    let s = &mut seen[w as usize];
                    if !*s {
                        *s = true;
                        observed += 1;
                    }
                }
                rows.push(buf.clone());
            }
            toks.push(rows);
        }
        let alpha = match alpha {
            NbAlpha::ObservedLaplace => (observed.max(1) as f32) / NB_VOCAB as f32,
            NbAlpha::Fixed(a) => a,
        };
        let tables = (0..toks.len())
            .map(|d| {
                let mut b = ContrastiveScoreBuilder::new(NB_VOCAB, alpha);
                for (e, rows) in toks.iter().enumerate() {
                    for row in rows {
                        if e == d {
                            b.observe_in(row);
                        } else {
                            b.observe_out(row);
                        }
                    }
                }
                b.finish()
            })
            .collect();
        Self {
            tables,
            alpha,
            observed,
        }
    }

    /// Domain count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// True when no domain was fitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// The effective per-bucket α the tables were built with.
    #[must_use]
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// Buckets the training docs touched.
    #[must_use]
    pub fn observed(&self) -> usize {
        self.observed
    }

    /// In-scope log2-odds per domain for `tokens` into `out[..len()]`
    /// (positive ⇒ the tokens read like that domain). Zero-alloc.
    pub fn in_scores(&self, tokens: &[u32], out: &mut [f32]) {
        for (o, t) in out.iter_mut().zip(self.tables.iter()) {
            *o = -scope_score(t, tokens);
        }
    }

    /// The option blend term for domain `d` given every domain's in-scope
    /// score: `scale · σ(margin_d / n_tokens)`. An empty token list reads
    /// as no evidence: margin 0 ⇒ `scale · 0.5` for every option (a
    /// constant shift, never a ranking signal).
    #[must_use]
    pub fn blend_term(in_scores: &[f32], d: usize, n_tokens: usize, scale: f32) -> f32 {
        if n_tokens == 0 {
            return scale * 0.5;
        }
        let mut other = f32::NEG_INFINITY;
        for (e, &s) in in_scores.iter().enumerate() {
            if e != d && s > other {
                other = s;
            }
        }
        let margin = if other.is_finite() {
            in_scores[d] - other
        } else {
            in_scores[d]
        };
        scale * exact_sigmoid(margin / n_tokens as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    fn fit2() -> NbScope {
        let sport = docs(&[
            "the team won the match in overtime",
            "striker scores twice as the club wins the league",
            "coach praises the defense after the game",
        ]);
        let biz = docs(&[
            "shares fell as the company missed earnings",
            "the bank raised interest rates again",
            "investors sold stock after the profit warning",
        ]);
        NbScope::fit(&[&sport, &biz], NbAlpha::ObservedLaplace)
    }

    fn scores(nb: &NbScope, text: &str) -> ([f32; 2], usize) {
        let mut t = Vec::new();
        hashed_tokens_into(text.as_bytes(), NB_VOCAB, &mut t);
        let mut out = [0.0f32; 2];
        nb.in_scores(&t, &mut out);
        (out, t.len())
    }

    #[test]
    fn ranks_the_matching_domain_first() {
        let nb = fit2();
        let (s, _) = scores(&nb, "the club won the game");
        assert!(s[0] > s[1], "sport text must read sport: {s:?}");
        let (s, _) = scores(&nb, "the company stock and earnings");
        assert!(s[1] > s[0], "business text must read business: {s:?}");
    }

    #[test]
    fn fit_is_bit_identical() {
        let a = fit2();
        let b = fit2();
        for (x, y) in a.tables.iter().zip(b.tables.iter()) {
            assert_eq!(x.commitment(), y.commitment(), "re-fit must commit identically");
        }
        assert_eq!(a.alpha().to_bits(), b.alpha().to_bits());
    }

    #[test]
    fn observed_laplace_alpha_matches_touched_buckets() {
        let nb = fit2();
        assert!(nb.observed() > 0);
        let want = nb.observed() as f32 / NB_VOCAB as f32;
        assert_eq!(nb.alpha().to_bits(), want.to_bits());
    }

    #[test]
    fn blend_term_is_monotone_and_neutral_on_empty() {
        let s = [3.0f32, -1.0, 0.5];
        let t0 = NbScope::blend_term(&s, 0, 4, 1.0);
        let t1 = NbScope::blend_term(&s, 1, 4, 1.0);
        let t2 = NbScope::blend_term(&s, 2, 4, 1.0);
        assert!(t0 > t2 && t2 > t1, "term must follow in-scope order: {t0} {t2} {t1}");
        assert_eq!(NbScope::blend_term(&s, 1, 0, 2.0), 1.0, "no tokens ⇒ scale·0.5");
    }
}
