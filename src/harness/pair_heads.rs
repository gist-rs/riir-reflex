//! Fitted pairwise disambiguation heads (Issue 013 lever 3 — "more fitted
//! heads", the Bench 881 game-head precedent applied to the dataset suites).
//!
//! The lever-3 probe (Bench 005) measured the suites' error structure: on
//! several suites most errors flow INTO one label (xnli_en: 95% of all
//! errors predict `neutral`; emotion: 73% predict `joy`) — the broad class's
//! unit centroid sits near the data mean and attracts everything, the
//! classic centroid-collapse pathology of nearest-centroid scoring.
//!
//! A pair head is the minimal fitted correction for one confused pair
//! {a, b}: a diagonal-LDA discriminant fitted from the pair's CORPUS docs —
//! the same docs the engine's domains already consume (no new data, no
//! training: closed-form moments, bit-deterministic, the modelless fit
//! law). At eval, when the engine's top-2 matches an armed pair, the head
//! re-decides between exactly those two options from the state embedding;
//! every other question keeps the engine's pick, so the arm cannot regress
//! an untouched question — the blast radius is the armed subset, disclosed
//! per pair in the A/B record.
//!
//! Protocol (the lever-1 precedent, load-bearing): pairs are ARMED from
//! CAL-slice confusion only — a pair picked on the test split would be a
//! test-set-selected hyperparameter; heads are FITTED from corpus docs
//! only; the test split is read once, in the A/B table.

use serde::Serialize;

/// At most this many pairs armed per suite (by cal-slice confusion mass).
pub const MAX_ARMED_PAIRS: usize = 4;

/// A pair needs at least this many cal-slice mispredictions to arm: below
/// this the selection is noise, and the confusion evidence itself is too
/// thin to claim the pair is confusable.
pub const MIN_CAL_SUPPORT: usize = 8;

/// Diagonal-variance shrinkage: `σ²[d] += VAR_SHRINK · mean(σ²)`. The raw
/// per-dim variance from ~40 docs per class is noisy, and a dim that is
/// (near-)constant in BOTH classes carries no between-class signal —
/// dividing by ~0 would explode the direction into that dim.
const VAR_SHRINK: f64 = 0.01;

/// Absolute variance floor — the shrink above is RELATIVE to the mean
/// variance, which is itself 0 when every dim is constant in both classes
/// (identical corpora); without it `w = 0/0 = NaN` (caught by the module
/// test, not reasoned about).
const MIN_VAR_FLOOR: f64 = 1e-12;

/// One fitted pair head: diagonal-LDA over the pair's two classes.
///
/// Discriminant (shared diagonal `σ²`, equal priors): pick `a` iff
/// `(μ_a − μ_b)·x / σ² > (|μ_a|² − |μ_b|²) / (2σ²)` — precomputed as
/// `w·x > t`. An exact tie picks `b` (strict `>` keeps the incumbent, the
/// same first-max-wins convention `hard_metrics`' argmax uses).
#[derive(Debug, Clone, PartialEq)]
pub struct PairHead<const D: usize> {
    /// Canonical lower option index of the pair.
    pub a: usize,
    /// Canonical higher option index of the pair.
    pub b: usize,
    w: [f64; D],
    t: f64,
}

impl<const D: usize> PairHead<D> {
    /// Fit from the two classes' document embeddings — the raw hashed-bag
    /// vectors `DomainExpert` averages into its routing direction, here
    /// kept UNNORMALIZED (LDA models the actual class means). Deterministic
    /// closed form; `None` when either class has no docs.
    pub fn fit(a: usize, b: usize, docs_a: &[[f32; D]], docs_b: &[[f32; D]]) -> Option<Self> {
        if docs_a.is_empty() || docs_b.is_empty() {
            return None;
        }
        let mut ma = [0.0f64; D];
        let mut mb = [0.0f64; D];
        for v in docs_a {
            for (m, x) in ma.iter_mut().zip(v.iter()) {
                *m += f64::from(*x);
            }
        }
        for v in docs_b {
            for (m, x) in mb.iter_mut().zip(v.iter()) {
                *m += f64::from(*x);
            }
        }
        let (na, nb) = (docs_a.len() as f64, docs_b.len() as f64);
        for m in ma.iter_mut() {
            *m /= na;
        }
        for m in mb.iter_mut() {
            *m /= nb;
        }
        // Pooled per-dim variance over both classes (population form).
        let mut var = [0.0f64; D];
        for v in docs_a {
            for i in 0..D {
                let d = f64::from(v[i]) - ma[i];
                var[i] += d * d;
            }
        }
        for v in docs_b {
            for i in 0..D {
                let d = f64::from(v[i]) - mb[i];
                var[i] += d * d;
            }
        }
        for x in var.iter_mut() {
            *x /= na + nb;
        }
        let mean_var = var.iter().sum::<f64>() / D as f64;
        let mut w = [0.0f64; D];
        let mut t = 0.0f64;
        for i in 0..D {
            let s = var[i] + VAR_SHRINK * mean_var + MIN_VAR_FLOOR;
            w[i] = (ma[i] - mb[i]) / s;
            t += (ma[i] * ma[i] - mb[i] * mb[i]) / (2.0 * s);
        }
        Some(Self { a, b, w, t })
    }

    /// The head's pick between `a` and `b` for one state embedding.
    pub fn pick(&self, x: &[f32; D]) -> usize {
        let mut dot = 0.0f64;
        for (w, v) in self.w.iter().zip(x.iter()) {
            dot += w * f64::from(*v);
        }
        if dot > self.t { self.a } else { self.b }
    }
}

/// An armed pair: canonical indices plus its selection evidence.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArmedPair {
    pub a: usize,
    pub b: usize,
    /// Cal-slice mispredictions that armed this pair.
    pub cal_support: usize,
}

/// Pair selection from CAL-slice `(gold, pick)` mispredictions: unordered
/// canonical pairs, counted, sorted by count desc then (a, b) — the first
/// `k` pairs with support ≥ `min_support`. Pure and deterministic.
#[must_use]
pub fn select_pairs(mispairs: &[(usize, usize)], k: usize, min_support: usize) -> Vec<ArmedPair> {
    let mut counts: Vec<((usize, usize), usize)> = Vec::new();
    for &(g, p) in mispairs {
        if g == p {
            continue;
        }
        let key = (g.min(p), g.max(p));
        match counts.iter_mut().find(|(kk, _)| *kk == key) {
            Some((_, c)) => *c += 1,
            None => counts.push((key, 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    counts
        .into_iter()
        .filter(|(_, c)| *c >= min_support)
        .take(k)
        .map(|((a, b), cal_support)| ArmedPair { a, b, cal_support })
        .collect()
}

/// One armed pair's A/B subset detail.
#[derive(Debug, Clone, Serialize)]
pub struct PairSubsetRow {
    pub a: usize,
    pub b: usize,
    /// Display keys resolved from the suite's option universe (falls back
    /// to the engine domain label).
    pub a_label: String,
    pub b_label: String,
    /// Subset size: questions whose engine top-2 matched this pair.
    pub n: usize,
    pub baseline_correct: usize,
    pub head_correct: usize,
    /// Subset questions whose GOLD is one of the pair — the only ones a
    /// head can move (gold-outside rows are wrong under both arms by
    /// construction, and their share IS the top-k-exclusion evidence).
    pub n_gold_in_pair: usize,
    pub baseline_correct_in_pair: usize,
    pub head_correct_in_pair: usize,
}

/// The harness A/B record for one suite (present in the result row only
/// when the `--pair-head-ab` arm ran).
#[derive(Debug, Clone, Serialize)]
pub struct PairHeadAb {
    pub armed: Vec<ArmedPair>,
    /// Forced accuracy over the counted (Choice + Score) questions — the
    /// engine baseline.
    pub baseline_acc: f64,
    /// Forced accuracy with the heads overriding armed-pair subsets.
    pub head_acc: f64,
    /// The counted population the accs read over (an all-Noul suite counts
    /// 0 — its accs are 0.0 by the empty-denominator law, never a claim).
    pub n_counted: usize,
    /// Questions whose engine top-2 matched an armed pair (the union
    /// subset; per-pair overlap possible in principle).
    pub n_overrides: usize,
    pub subset_n: usize,
    pub baseline_correct_on_subset: usize,
    pub head_correct_on_subset: usize,
    /// The gold-in-pair slice of the subset (the movable mass).
    pub subset_gold_in_pair: usize,
    pub baseline_correct_in_pair: usize,
    pub head_correct_in_pair: usize,
    pub per_pair: Vec<PairSubsetRow>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counting, canonicalization, count-desc tie-(a,b) ordering, support
    /// floor, and the k cap.
    #[test]
    fn select_pairs_orders_and_floors() {
        let mispairs = vec![
            (3, 1),
            (1, 3),
            (3, 1),
            (0, 2),
            (2, 0),
            (0, 2),
            (0, 2),
            (4, 0), // support 1 — below the floor
        ];
        let armed = select_pairs(&mispairs, 4, 2);
        assert_eq!(armed.len(), 2);
        assert_eq!(armed[0].a, 0);
        assert_eq!(armed[0].b, 2);
        assert_eq!(armed[0].cal_support, 4);
        assert_eq!(armed[1].a, 1);
        assert_eq!(armed[1].b, 3);
        assert_eq!(armed[1].cal_support, 3);
        // The k cap holds even with more eligible pairs.
        let many: Vec<(usize, usize)> = (0..10).flat_map(|i| vec![(i, i + 100); 5]).collect();
        assert_eq!(select_pairs(&many, 3, 2).len(), 3);
        assert!(select_pairs(&mispairs, 4, 9).is_empty());
    }

    /// A direction that carries all the between-class signal: dim 0
    /// separates, dim 1 is pure noise with big variance. The diagonal LDA
    /// must down-weight dim 1 and classify every point correctly, while a
    /// plain centroid-difference sign would be dragged by the noise dim.
    #[test]
    fn fit_separates_where_signal_lives() {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in 0..24 {
            // class a: dim0 high, dim1 alternating +/-2 (variance 4, no signal)
            a.push([
                3.0f32 + (i % 2) as f32 * 0.2,
                if i % 2 == 0 { 2.0 } else { -2.0 },
            ]);
            // class b: dim0 low, same dim1 spread
            b.push([1.0f32, if i % 2 == 0 { 2.0 } else { -2.0 }]);
        }
        let head = PairHead::<2>::fit(0, 1, &a, &b).expect("both classes populated");
        for v in &a {
            assert_eq!(head.pick(v), 0);
        }
        for v in &b {
            assert_eq!(head.pick(v), 1);
        }
        // Canonical index order is preserved whatever the call order.
        let swapped = PairHead::<2>::fit(1, 0, &b, &a).expect("fit");
        assert_eq!(swapped.a, 1);
        assert_eq!(swapped.b, 0);
        for v in &a {
            assert_eq!(swapped.pick(v), 0);
        }
    }

    /// Empty input refuses; equal classes produce a deterministic constant
    /// head (w = 0, t = 0 → tie → picks b) — never a NaN.
    #[test]
    fn fit_refuses_empty_and_stays_finite_on_identical_classes() {
        let one = [[1.0f32, 0.0]];
        assert!(PairHead::<2>::fit(0, 1, &one, &[]).is_none());
        assert!(PairHead::<2>::fit(0, 1, &[], &one).is_none());
        let head = PairHead::<2>::fit(0, 1, &one, &one).expect("fit");
        assert_eq!(head, PairHead::<2>::fit(0, 1, &one, &one).expect("fit"));
        assert_eq!(head.pick(&[1.0, 0.0]), 1);
        assert!(head.w.iter().all(|w| w.is_finite()));
    }
}
