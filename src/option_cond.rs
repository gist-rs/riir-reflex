//! Option-conditioned count tables (issue 038 T7b).
//!
//! The domain-level tables ([`crate::nb_scope`]) arm only when the option
//! set resolves to domains (`by_name` or `k == N`). `typed_decisions` never
//! qualifies — variable per-row option sets over `choice`/`score`/`noul`
//! questions — so the strongest state-side signal there is the gold
//! OPTION's own statistics: one count table per (question id, option key),
//! fitted from the train rows whose gold picked that option, read against
//! the question's OWN option set.
//!
//! Premise (probed before any Rust, `scripts/issue038_t7_probe.py`):
//! per-(qid, option) multinomial counts read **0.4475** on the typed test
//! questions against the published modelless **0.32** (+13 pt; laya best
//! 0.745 — the rest is the cascade lane's job). Variant forms (no prior,
//! contrastive margins, IDF, char n-grams) read the same or worse, so the
//! shipped one-vs-rest machinery is the right shape: DRY with `nb_scope`,
//! and the blend term is the same bounded sigmoid margin.
//!
//! Modelless: frozen integer counts, deterministic fit (insertion-ordered
//! groups, key-sorted index), no RNG. Hot path is alloc-free: one FNV key
//! hash + binary search + a table score per option.

use crate::embed::hashed_tokens_into;
use crate::nb_scope::NbAlpha;
use katgpt_core::contrastive_scope::{scope_score, ContrastiveScoreBuilder, ContrastiveScoreTable};

/// Vocabulary width of the underlying token hash (the `nb_scope` pool —
/// one lexicon across both table families keeps the token stream
/// comparable and the code shared).
pub const OC_VOCAB: usize = crate::nb_scope::NB_VOCAB;

/// One fit input: a train observation "question `qid` was answered
/// `option` on this document". `doc` is the same corpus text the domain
/// tables read (the raw stored state string, for typed).
#[derive(Clone, Debug)]
pub struct OcEvent {
    pub qid: String,
    pub option: String,
    pub doc: String,
}

/// Score-time key: FNV-1a over `qid ‖ 0x00 ‖ option` (the 0 separator makes
/// the concatenation injective on the byte level). Mirrors nothing on the
/// wire — this is an internal index key only.
fn oc_key(qid: &str, option: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    let step = |b: u8, h: &mut u64| {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(0x100_0000_01B3);
    };
    for &b in qid.as_bytes() {
        step(b, &mut h);
    }
    step(0x00, &mut h);
    for &b in option.as_bytes() {
        step(b, &mut h);
    }
    h
}

/// The fitted option-conditioned tables. Immutable after the build;
/// read-only on the hot path.
#[derive(Debug, Default)]
pub struct OptionCond {
    /// `(key, table)` sorted by key — binary-search lookup, deterministic
    /// order by construction.
    index: Vec<(u64, ContrastiveScoreTable)>,
    /// Every token any event doc carried (fit-time union) — the evidence
    /// bitmap behind [`Self::seen_count`], the exact [`crate::nb_scope::NbScope`]
    /// resolution. Frozen at fit time.
    seen: Vec<u64>,
}

impl OptionCond {
    /// Fit one-vs-rest tables from events. The "rest" of a (qid, option)
    /// table is the SAME qid's other-option events — the signal is "why
    /// this option and not the alternatives offered by this question",
    /// which is exactly the decision the engine is asked to make.
    ///
    /// Deterministic: groups form in event order, options within a qid in
    /// first-appearance order, the index sorts by key. A re-fit is
    /// bit-identical.
    #[must_use]
    pub fn fit(events: &[OcEvent], alpha: NbAlpha, view: crate::nb_scope::NbView) -> Self {
        // Pass 1: tokenize each event once (one hash per doc), and track
        // the seen-bucket count for the observed-Laplace α (the exact
        // `nb_scope` resolution).
        let mut token_rows: Vec<Vec<u32>> = Vec::with_capacity(events.len());
        let mut buf: Vec<u32> = Vec::new();
        let mut seen = vec![0u64; OC_VOCAB.div_ceil(64)];
        let mut observed = 0usize;
        for ev in events {
            view_tokens_oc(view, ev.doc.as_bytes(), &mut buf);
            for &w in &buf {
                let slot = &mut seen[w as usize / 64];
                let bit = 1u64 << (w as usize % 64);
                if *slot & bit == 0 {
                    *slot |= bit;
                    observed += 1;
                }
            }
            token_rows.push(std::mem::take(&mut buf));
        }
        let alpha = match alpha {
            NbAlpha::ObservedLaplace => (observed.max(1) as f32) / OC_VOCAB as f32,
            NbAlpha::Fixed(a) => a,
        };

        // Pass 2: group rows by qid (insertion-ordered) then by option
        // (first-appearance within the qid).
        let mut qids: Vec<u32> = Vec::new(); // first-appearance qid ids
        let mut qid_ids: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        // Per qid: (option String, row indices) in first-appearance order.
        let mut groups: Vec<Vec<(String, Vec<usize>)>> = Vec::new();
        for (i, ev) in events.iter().enumerate() {
            let q = match qid_ids.get(ev.qid.as_str()) {
                Some(&q) => q,
                None => {
                    let q = qids.len() as u32;
                    qid_ids.insert(&ev.qid, q);
                    qids.push(q);
                    groups.push(Vec::new());
                    q
                }
            } as usize;
            let opts = &mut groups[q];
            match opts.iter_mut().find(|(o, _)| *o == ev.option) {
                Some((_, rows)) => rows.push(i),
                None => opts.push((ev.option.clone(), vec![i])),
            }
        }

        // Pass 3: one contrastive table per (qid, option).
        let mut index: Vec<(u64, ContrastiveScoreTable)> = Vec::new();
        for opts in groups.iter() {
            let qid = events[opts[0].1[0]].qid.as_str();
            for (d, (opt, rows)) in opts.iter().enumerate() {
                let mut b = ContrastiveScoreBuilder::new(OC_VOCAB, alpha);
                for &i in rows {
                    b.observe_in(&token_rows[i]);
                }
                for (e, (_, other_rows)) in opts.iter().enumerate() {
                    if e == d {
                        continue;
                    }
                    for &i in other_rows {
                        b.observe_out(&token_rows[i]);
                    }
                }
                index.push((oc_key(qid, opt), b.finish()));
            }
        }
        index.sort_unstable_by_key(|(k, _)| *k);
        Self { index, seen }
    }

    /// Table count (distinct fitted keys).
    #[must_use]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// How many of `tokens`' events any fitted event doc saw — the
    /// evidence count `n` of the riir-instinct issue-005 hybrid gate
    /// (`g = σ((n − n_min)/τ_n)`), the exact [`crate::nb_scope::NbScope::seen_count`
    /// resolution over this family's own event stream. Counts events,
    /// duplicates included. Zero-alloc.
    #[must_use]
    pub fn seen_count(&self, tokens: &[u32]) -> usize {
        tokens
            .iter()
            .filter(|&&w| self.seen[w as usize / 64] & (1u64 << (w as usize % 64)) != 0)
            .count()
    }

    /// True when nothing was fitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// In-scope log2-odds for `(qid, option)` over `tokens` (positive ⇒ the
    /// tokens read like that option), `None` when the key has no table.
    #[must_use]
    pub fn in_score(&self, qid: &str, option: &str, tokens: &[u32]) -> Option<f32> {
        let key = oc_key(qid, option);
        let pos = self
            .index
            .binary_search_by(|(k, _)| k.cmp(&key))
            .ok()?;
        Some(-scope_score(&self.index[pos].1, tokens))
    }

    /// Per-option in-scores for one question into `out` (length = the
    /// option count; missing keys contribute `None` → the caller's term is
    /// absent for that option, never a fabricated zero).
    pub fn in_scores(&self, qid: &str, options: &[impl AsRef<str>], tokens: &[u32], out: &mut [Option<f32>]) {
        for (slot, opt) in out.iter_mut().zip(options.iter()) {
            *slot = self.in_score(qid, opt.as_ref(), tokens);
        }
    }
}

/// Bag-or-pair view over the event docs, mirroring [`crate::nb_scope`]
/// (a JSON state yields its string values as fields under the pair view;
/// the typed premise was probed on the bag view and the pair view of a
/// JSON state is the same token stream when the object is flat).
fn view_tokens_oc(view: crate::nb_scope::NbView, text: &[u8], out: &mut Vec<u32>) {
    match view {
        crate::nb_scope::NbView::Bag => hashed_tokens_into(text, OC_VOCAB, out),
        crate::nb_scope::NbView::Pair => crate::nb_scope::pair_tokens_into(text, out),
    }
}

/// The option blend term for option `d` of one question, given every
/// option's in-scope score: `scale · σ(margin_d / n_tokens)` — the exact
/// [`NbScope::blend_term`] shape (an absent score is `None`: the caller
/// adds nothing for that option rather than a fabricated neutral).
#[must_use]
pub fn oc_blend_term(in_score: f32, best_other: Option<f32>, n_tokens: usize, scale: f32) -> f32 {
    if n_tokens == 0 {
        return scale * 0.5;
    }
    let margin = match best_other {
        Some(o) => in_score - o,
        None => in_score,
    };
    scale * katgpt_core::exact_sigmoid(margin / n_tokens as f32)
}

/// Best in-score among the OTHER options (the margin denominator of
/// [`oc_blend_term`]) — `None` for a single-option question.
#[must_use]
pub fn best_other(in_scores: &[Option<f32>], d: usize) -> Option<f32> {
    in_scores
        .iter()
        .enumerate()
        .filter(|(i, s)| *i != d && s.is_some())
        .map(|(_, s)| s.expect("filtered is_some"))
        .fold(None, |acc: Option<f32>, x| match acc {
            None => Some(x),
            Some(m) => Some(m.max(x)),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events() -> Vec<OcEvent> {
        vec![
            OcEvent {
                qid: "urgency".into(),
                option: "high".into(),
                doc: "breach detected destructive security policy".into(),
            },
            OcEvent {
                qid: "urgency".into(),
                option: "high".into(),
                doc: "security breach destructive".into(),
            },
            OcEvent {
                qid: "urgency".into(),
                option: "low".into(),
                doc: "routine queue cosmetic cosmetic".into(),
            },
            OcEvent {
                qid: "risk".into(),
                option: "high".into(),
                doc: "unrelated question risk text".into(),
            },
        ]
    }

    #[test]
    fn ranks_the_matching_option_first() {
        let oc = OptionCond::fit(&events(), NbAlpha::Fixed(1.0), crate::nb_scope::NbView::Bag);
        assert_eq!(oc.len(), 3); // (urgency,high) (urgency,low) (risk,high)
        let mut tok = Vec::new();
        hashed_tokens_into(b"security breach destructive", OC_VOCAB, &mut tok);
        let hi = oc.in_score("urgency", "high", &tok).expect("key present");
        let lo = oc.in_score("urgency", "low", &tok).expect("key present");
        assert!(hi > lo, "matching option must out-score the alternative");
    }

    #[test]
    fn fit_is_bit_identical_and_key_order_free() {
        let mut a = events();
        a.reverse();
        let x = OptionCond::fit(&events(), NbAlpha::Fixed(1.0), crate::nb_scope::NbView::Bag);
        let y = OptionCond::fit(&a, NbAlpha::Fixed(1.0), crate::nb_scope::NbView::Bag);
        // Reverse event order: groups still form (same set), tables are
        // built from the same multiset of rows — the SCORES must agree.
        let mut tok = Vec::new();
        hashed_tokens_into(b"security breach destructive", OC_VOCAB, &mut tok);
        assert_eq!(
            x.in_score("urgency", "high", &tok),
            y.in_score("urgency", "high", &tok)
        );
        // Re-fit of the same input is byte-identical on the index order.
        let z = OptionCond::fit(&events(), NbAlpha::Fixed(1.0), crate::nb_scope::NbView::Bag);
        let ks = |o: &OptionCond| o.index.iter().map(|(k, _)| *k).collect::<Vec<_>>();
        assert_eq!(ks(&x), ks(&z));
    }

    #[test]
    fn missing_key_is_none_never_zero() {
        let oc = OptionCond::fit(&events(), NbAlpha::Fixed(1.0), crate::nb_scope::NbView::Bag);
        assert!(oc.in_score("urgency", "absent", &[]).is_none());
        assert!(oc.in_score("no-such-qid", "high", &[]).is_none());
    }

    #[test]
    fn blend_term_is_bounded_and_margin_monotone() {
        let n = 5usize;
        let t_hi = oc_blend_term(3.0, Some(1.0), n, 1.0);
        let t_lo = oc_blend_term(1.0, Some(3.0), n, 1.0);
        assert!(t_hi > t_lo);
        assert!(t_hi < 1.0 && t_lo >= 0.0);
        // No other option: the raw score drives the term.
        let solo = oc_blend_term(3.0, None, n, 1.0);
        assert!(solo > t_lo);
        // Empty tokens: neutral half.
        assert_eq!(oc_blend_term(3.0, Some(1.0), 0, 1.0), 0.5);
    }

    #[test]
    fn best_other_skips_missing_and_self() {
        let rows: Vec<Option<f32>> = vec![Some(1.0), None, Some(3.0)];
        assert_eq!(best_other(&rows, 0), Some(3.0));
        assert_eq!(best_other(&rows, 1), Some(3.0));
        assert_eq!(best_other(&rows, 2), Some(1.0));
        let single: Vec<Option<f32>> = vec![Some(2.0)];
        assert_eq!(best_other(&single, 0), None);
    }

    #[test]
    fn in_scores_maps_every_option() {
        let oc = OptionCond::fit(&events(), NbAlpha::Fixed(1.0), crate::nb_scope::NbView::Bag);
        let opts = ["high", "low", "absent"];
        let mut tok = Vec::new();
        hashed_tokens_into(b"routine queue", OC_VOCAB, &mut tok);
        let mut out = [None; 3];
        oc.in_scores("urgency", &opts, &tok, &mut out);
        assert!(out[0].is_some() && out[1].is_some() && out[2].is_none());
    }
}
