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

use crate::embed::{fnv1a_word, hashed_tokens_into, token};
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

/// How a text becomes count-table events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbView {
    /// Unigram + bigram hashes over the whole text (the default).
    Bag,
    /// Sentence-PAIR view (issue 038 T3): the LAST field is read against
    /// every earlier one — see [`pair_tokens_into`]. Texts with fewer than
    /// two fields fall back to [`NbView::Bag`].
    Pair,
}

/// Events for `text` under `view` into `out` (cleared first).
pub fn view_tokens_into(view: NbView, text: &[u8], out: &mut Vec<u32>) {
    match view {
        NbView::Bag => hashed_tokens_into(text, NB_VOCAB, out),
        NbView::Pair => pair_tokens_into(text, out),
    }
}

/// Max fields the pair view tracks (stack array; extras fold into the
/// context side).
const MAX_FIELDS: usize = 8;
const SALT_HYP: u64 = 0x5a17_0000_0000_0001;
const SALT_HYP_BI: u64 = 0x5a17_0000_0000_0002;
const SALT_NEW: u64 = 0x5a17_0000_0000_0003;
const SALT_META: u64 = 0x5a17_0000_0000_0004;
/// Context-word membership bitset width (bits) — a stack `[u64; 64]`.
const CTX_BITS: usize = 4096;

/// Field spans of `text`, in order. A text opening with `{` is read as a
/// flat JSON object and yields its STRING VALUES (the serialized-state
/// shape: `{"premise": "…", "hypothesis": "…"}`); anything else yields
/// its non-empty lines (the train-doc shape: `premise\nhypothesis`). Escapes
/// stay in place (the tokenizer trims them at word edges). Zero-alloc.
fn field_spans(text: &[u8], spans: &mut [(usize, usize); MAX_FIELDS]) -> usize {
    let mut n = 0usize;
    let mut push = |a: usize, b: usize, n: &mut usize| {
        if b > a {
            if *n < MAX_FIELDS {
                spans[*n] = (a, b);
                *n += 1;
            } else {
                // Fold extras into the last context slot's end.
                spans[MAX_FIELDS - 2].1 = b;
            }
        }
    };
    if text.first() == Some(&b'{') {
        // A string value opens after `": "` (Python-JSON) or `":"`.
        let mut i = 0usize;
        while i + 1 < text.len() {
            if text[i] == b'"' && text[i + 1] == b':' {
                let mut j = i + 2;
                while j < text.len() && text[j] == b' ' {
                    j += 1;
                }
                if j < text.len() && text[j] == b'"' {
                    let start = j + 1;
                    let mut k = start;
                    while k < text.len() {
                        match text[k] {
                            b'\\' => k += 2,
                            b'"' => break,
                            _ => k += 1,
                        }
                    }
                    let end = k.min(text.len());
                    push(start, end, &mut n);
                    i = end + 1;
                    continue;
                }
            }
            i += 1;
        }
    } else {
        let mut a = 0usize;
        for (i, &b) in text.iter().enumerate() {
            if b == b'\n' {
                push(a, i, &mut n);
                a = i + 1;
            }
        }
        push(a, text.len(), &mut n);
    }
    n
}

/// A small fixed English negation lexicon (hand-authored rule, the
/// hybrid-rules half of the engine).
fn is_negation(w: &[u8]) -> bool {
    const NEG: [&[u8]; 14] = [
        b"not", b"no", b"never", b"nobody", b"nothing", b"none", b"neither", b"nor", b"cannot",
        b"without", b"nowhere", b"hardly", b"isnt", b"dont",
    ];
    let lower_eq = |a: &[u8], b: &[u8]| {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_ascii_lowercase() == *y)
    };
    NEG.iter().any(|n| lower_eq(w, n))
        || (w.len() > 3 && {
            let t = &w[w.len() - 3..];
            t[0] == b'n' && (t[1] == b'\'' || t[1] == b'"') && (t[2] | 0x20) == b't'
        })
}

/// The sentence-pair events (issue 038 T3), measured on a train-only
/// hold-out before it was built (xnli bag .386 → pair .505): the last
/// field's words + bigrams (salted apart from the bag lexicon), each of
/// its words ABSENT from the earlier fields (the classic lexical
/// entailment cue), a coverage bucket, a length bucket, and the
/// context/last negation pattern. Falls back to the bag view below two
/// fields. Zero-alloc beyond `out`'s capacity.
pub fn pair_tokens_into(text: &[u8], out: &mut Vec<u32>) {
    let mut spans = [(0usize, 0usize); MAX_FIELDS];
    let n = field_spans(text, &mut spans);
    if n < 2 {
        hashed_tokens_into(text, NB_VOCAB, out);
        return;
    }
    out.clear();
    let id = |h: u64| (h % NB_VOCAB as u64) as u32;
    let words = |span: (usize, usize)| {
        text[span.0..span.1]
            .split(|b: &u8| b.is_ascii_whitespace())
            .map(token)
            .filter(|t| !t.is_empty())
    };
    let mut ctx = [0u64; CTX_BITS / 64];
    let mut ctx_neg = false;
    for &span in &spans[..n - 1] {
        for w in words(span) {
            let b = (fnv1a_word(w, SALT_NEW) as usize) % CTX_BITS;
            ctx[b / 64] |= 1 << (b % 64);
            ctx_neg |= is_negation(w);
        }
    }
    let (mut n_words, mut covered, mut hyp_neg) = (0usize, 0usize, false);
    let mut prev: Option<u64> = None;
    for w in words(spans[n - 1]) {
        n_words += 1;
        hyp_neg |= is_negation(w);
        let h = fnv1a_word(w, SALT_HYP);
        out.push(id(h));
        if let Some(p) = prev {
            let mut pair = [0u8; 16];
            pair[..8].copy_from_slice(&p.to_le_bytes());
            pair[8..].copy_from_slice(&h.to_le_bytes());
            out.push(id(fnv1a_word(&pair, SALT_HYP_BI)));
        }
        prev = Some(h);
        let nh = fnv1a_word(w, SALT_NEW);
        let b = (nh as usize) % CTX_BITS;
        if ctx[b / 64] & (1 << (b % 64)) != 0 {
            covered += 1;
        } else {
            out.push(id(nh));
        }
    }
    let cov_bucket = (covered * 5).checked_div(n_words).unwrap_or(0);
    let meta = [
        [b'c', b'o', b'v', b'0' + cov_bucket as u8],
        [b'l', b'e', b'n', b'0' + (n_words / 4).min(5) as u8],
        [
            b'n',
            b'e',
            b'g',
            b'0' + u8::from(ctx_neg) * 2 + u8::from(hyp_neg),
        ],
    ];
    for m in &meta {
        out.push(id(fnv1a_word(m, SALT_META)));
    }
}

/// The fitted per-domain tables. Immutable after the build; read-only on
/// the hot path.
pub struct NbScope {
    tables: Vec<ContrastiveScoreTable>,
    alpha: f32,
    observed: usize,
    /// Bitset of buckets any training doc touched (issue 005 E0): `n` of
    /// the hybrid evidence gate `g = σ((n − n_min)/τ_n)` counts a state's
    /// events against this set. 16 KiB — retained, not recomputed.
    seen: Box<[u64]>,
}

impl NbScope {
    /// Fit one-vs-rest tables from per-domain documents (`domains[d]` =
    /// domain `d`'s docs). Deterministic: fixed doc order, integer counts,
    /// no RNG — a re-fit is bit-identical.
    #[must_use]
    pub fn fit(domains: &[&[String]], alpha: NbAlpha, view: NbView) -> Self {
        let mut buf: Vec<u32> = Vec::new();
        let mut toks: Vec<Vec<Vec<u32>>> = Vec::with_capacity(domains.len());
        let mut seen = vec![0u64; NB_VOCAB.div_ceil(64)];
        let mut observed = 0usize;
        for docs in domains {
            let mut rows = Vec::with_capacity(docs.len());
            for doc in *docs {
                view_tokens_into(view, doc.as_bytes(), &mut buf);
                for &w in &buf {
                    let slot = &mut seen[w as usize / 64];
                    let bit = 1u64 << (w as usize % 64);
                    if *slot & bit == 0 {
                        *slot |= bit;
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
            seen: seen.into(),
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

    /// How many of `tokens`' events the training docs saw — the evidence
    /// count `n` of the riir-instinct issue-005 hybrid gate
    /// (`g = σ((n − n_min)/τ_n)`). Counts events, duplicates included: the
    /// same event stream the blend's σ normalizer divides by, so `n` and
    /// `n_tokens` always describe one stream. Zero-alloc.
    #[must_use]
    pub fn seen_count(&self, tokens: &[u32]) -> usize {
        tokens
            .iter()
            .filter(|&&w| self.seen[w as usize / 64] & (1u64 << (w as usize % 64)) != 0)
            .count()
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
        NbScope::fit(&[&sport, &biz], NbAlpha::ObservedLaplace, NbView::Bag)
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
            assert_eq!(
                x.commitment(),
                y.commitment(),
                "re-fit must commit identically"
            );
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
    fn pair_view_reads_json_state_and_line_doc_identically() {
        let mut a = Vec::new();
        let mut b = Vec::new();
        pair_tokens_into(
            br#"{"premise": "The cat sat on the mat.", "hypothesis": "The cat did not sit."}"#,
            &mut a,
        );
        pair_tokens_into(b"The cat sat on the mat.\nThe cat did not sit.", &mut b);
        assert_eq!(
            a, b,
            "state JSON and train-doc lines must yield the same events"
        );
        let mut bag = Vec::new();
        hashed_tokens_into(b"The cat sat on the mat.", NB_VOCAB, &mut bag);
        let mut one = Vec::new();
        pair_tokens_into(b"The cat sat on the mat.", &mut one);
        assert_eq!(bag, one, "one field falls back to the bag view");
    }

    #[test]
    fn pair_view_marks_new_words_and_negation() {
        let mut covered = Vec::new();
        let mut novel = Vec::new();
        pair_tokens_into(b"a dog runs in the park\na dog runs", &mut covered);
        pair_tokens_into(b"a dog runs in the park\na cat sleeps", &mut novel);
        assert!(novel.len() > covered.len(), "absent words add NEW events");
        assert!(is_negation(b"not") && is_negation(b"didn't") && !is_negation(b"note"));
    }

    #[test]
    fn blend_term_is_monotone_and_neutral_on_empty() {
        let s = [3.0f32, -1.0, 0.5];
        let t0 = NbScope::blend_term(&s, 0, 4, 1.0);
        let t1 = NbScope::blend_term(&s, 1, 4, 1.0);
        let t2 = NbScope::blend_term(&s, 2, 4, 1.0);
        assert!(
            t0 > t2 && t2 > t1,
            "term must follow in-scope order: {t0} {t2} {t1}"
        );
        assert_eq!(
            NbScope::blend_term(&s, 1, 0, 2.0),
            1.0,
            "no tokens \u{21d2} scale\u{b7}0.5"
        );
    }

    #[test]
    fn seen_count_reads_the_fit_time_event_stream() {
        let nb = fit2();
        let mut t = Vec::new();
        // Doc 1 verbatim: unigrams AND bigrams all observed at fit.
        hashed_tokens_into(b"the team won the match", NB_VOCAB, &mut t);
        assert_eq!(
            nb.seen_count(&t),
            t.len(),
            "a verbatim training sentence is fully seen"
        );
        // One novel word adds exactly two unseen events: the unigram and
        // its bigram with the previous word. Duplicates count ("the" ×2).
        hashed_tokens_into(b"the team won the match zzzq", NB_VOCAB, &mut t);
        assert_eq!(
            nb.seen_count(&t),
            t.len() - 2,
            "exactly the novel unigram + bigram are unseen"
        );
        assert_eq!(nb.seen_count(&[]), 0);
    }
}
