//! Metric definitions ported verbatim from `.docs/02_protocols/laya_bench_protocols.md` §5
//! (Plan 603 T1.5). Pure f64 math over already-calibrated probabilities — no
//! engine, no laya, no I/O, no sibling imports. Every public function is a
//! known-answer unit test target in `tests/harness_units.rs`.
//!
//! Divergences from the Python reference (all deliberate, all tiny):
//! - sorts are Rust `sort_by` (stable) where numpy's default argsort is not
//!   stable — tie order is first-index-wins, which is deterministic across
//!   both lanes of THIS harness;
//! - cross-language FP summation order can differ in the last ulp from
//!   numpy; the protocol compares lanes that both run THESE functions, so
//!   internal consistency is what carries the comparison.

use serde::Serialize;

/// Hard metrics over `(gold_idx, probs)` rows — the §5.1 reference port.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HardMetrics {
    pub n: usize,
    pub accuracy: f64,
    pub macro_f1: f64,
    pub ece: f64,
    pub brier: f64,
    pub nll: f64,
    pub aurc: f64,
    pub mean_confidence: f64,
    pub acc_at_50_coverage: f64,
    pub acc_at_80_coverage: f64,
}

/// Soft-distribution metrics (§5.2): calibrated probs vs the normalized gold
/// soft target (typed-decisions only).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SoftMetrics {
    /// probability the model puts on gold under the soft truth
    pub soft_acc: f64,
    pub brier_soft: f64,
    /// total variation, 0.5 * L1
    pub tv: f64,
    /// KL(gp || pp) with the reference's 1e-12 / 1e4 clips
    pub kl: f64,
}

/// Score metrics (§5.3) for score-type questions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoreMetrics {
    /// expected level under the calibrated probs: sum(i * p_i)
    pub expected: f64,
    pub mae: f64,
    /// |expected - gold| <= 1.0, as 0.0/1.0 (the reference's float(bool))
    pub within_1: f64,
}

/// One (confidence, correct) calibration observation — the split-conformal
/// calibration set element.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CalibrationPair {
    pub conf: f64,
    pub correct: bool,
}

/// §5.1 `hard_metrics`. `rows` are `(gold_idx, probs)` pairs with None-probs
/// already dropped upstream; non-empty is asserted (the reference would
/// produce a degenerate mean otherwise).
#[must_use]
pub fn hard_metrics(rows: &[(usize, Vec<f64>)]) -> HardMetrics {
    assert!(
        !rows.is_empty(),
        "hard_metrics: non-empty rows required (drop None-probs upstream)"
    );
    let n = rows.len() as f64;

    let mut golds = Vec::with_capacity(rows.len());
    let mut preds = Vec::with_capacity(rows.len());
    let mut confs = Vec::with_capacity(rows.len());
    let mut correct = Vec::with_capacity(rows.len());
    for (gold, probs) in rows {
        assert!(
            !probs.is_empty(),
            "hard_metrics: probability vector must be non-empty"
        );
        assert!(
            *gold < probs.len(),
            "hard_metrics: gold index {gold} out of range for {} probs",
            probs.len()
        );
        let (pred, conf) = argmax_first(probs);
        golds.push(*gold);
        preds.push(pred);
        confs.push(conf);
        correct.push(*gold == pred);
    }

    // argsort(-conf) descending, STABLE (first index wins ties)
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| confs[b].total_cmp(&confs[a]));

    let accuracy = correct.iter().filter(|&&c| c).count() as f64 / n;

    // classes = sorted(set(gold) | set(pred)); F1_c = 2tp / max(1, 2tp+fp+fn)
    let mut classes: Vec<usize> = golds.iter().chain(preds.iter()).copied().collect();
    classes.sort_unstable();
    classes.dedup();
    let f1_sum: f64 = classes
        .iter()
        .map(|&c| {
            let mut tp = 0.0_f64;
            let mut fp = 0.0_f64;
            let mut fnegative = 0.0_f64;
            for (&g, &p) in golds.iter().zip(preds.iter()) {
                match (g == c, p == c) {
                    (true, true) => tp += 1.0,
                    (false, true) => fp += 1.0,
                    (true, false) => fnegative += 1.0,
                    (false, false) => {}
                }
            }
            2.0 * tp / 1.0_f64.max(2.0 * tp + fp + fnegative)
        })
        .sum();
    let macro_f1 = f1_sum / classes.len() as f64;

    let pairs: Vec<(f64, bool)> = confs.iter().copied().zip(correct.iter().copied()).collect();
    let ece = ece_of(&pairs);

    // brier: sum over ALL classes of THAT question's probability vector
    let mut brier = 0.0;
    for (gold, probs) in rows {
        for (c, &p) in probs.iter().enumerate() {
            let d = p - f64::from(c == *gold);
            brier += d * d;
        }
    }
    let brier = brier / n;

    let nll: f64 = rows
        .iter()
        .map(|(gold, probs)| -(probs[*gold].max(1e-12)).ln())
        .sum::<f64>()
        / n;

    // aurc: mean over prefix lengths r=1..n of cum_error(r)/r
    let mut cum_error = 0.0_f64;
    let mut aurc = 0.0_f64;
    for (r, &i) in order.iter().enumerate() {
        if !correct[i] {
            cum_error += 1.0;
        }
        aurc += cum_error / (r as f64 + 1.0);
    }
    let aurc = aurc / n;

    let acc_at = |frac: f64| -> f64 {
        // Python int(len*frac) truncates toward zero; len*frac >= 0 so == floor
        let k = ((rows.len() as f64) * frac).floor() as usize;
        let k = k.max(1);
        let hits = order.iter().take(k).filter(|&&i| correct[i]).count();
        hits as f64 / k as f64
    };

    HardMetrics {
        n: rows.len(),
        accuracy,
        macro_f1,
        ece,
        brier,
        nll,
        aurc,
        mean_confidence: confs.iter().sum::<f64>() / n,
        acc_at_50_coverage: acc_at(0.5),
        acc_at_80_coverage: acc_at(0.8),
    }
}

/// §5.2 soft metrics. Normalizes the gold soft target to sum 1 (None when the
/// sum is <= 0 — including an empty target); truncates or zero-pads `p_cal`
/// to the target length, renormalizes with the 1e-12 guard.
#[must_use]
pub fn soft_metrics(p_cal: &[f64], gold_soft: &[f64]) -> Option<SoftMetrics> {
    let gp_sum: f64 = gold_soft.iter().sum();
    if gp_sum <= 0.0 {
        return None;
    }
    let gp: Vec<f64> = gold_soft.iter().map(|&v| v / gp_sum).collect();

    let mut pp: Vec<f64> = p_cal.iter().copied().take(gp.len()).collect();
    pp.resize(gp.len(), 0.0);
    let pp_norm = pp.iter().sum::<f64>().max(1e-12);
    for v in &mut pp {
        *v /= pp_norm;
    }

    let soft_acc: f64 = pp.iter().zip(&gp).map(|(&p, &g)| p * g).sum();
    let brier_soft: f64 = pp
        .iter()
        .zip(&gp)
        .map(|(&p, &g)| {
            let d = p - g;
            d * d
        })
        .sum();
    let tv: f64 = 0.5
        * pp.iter()
            .zip(&gp)
            .map(|(&p, &g)| (p - g).abs())
            .sum::<f64>();
    // KL(gp || pp): gp / clip(pp, 1e-12, None), then clip to [1e-12, 1e4]
    let kl: f64 = gp
        .iter()
        .zip(&pp)
        .map(|(&g, &p)| g * (g / p.max(1e-12)).clamp(1e-12, 1e4).ln())
        .sum();

    Some(SoftMetrics {
        soft_acc,
        brier_soft,
        tv,
        kl,
    })
}

/// §5.3 score metrics over the calibrated probability vector.
#[must_use]
pub fn score_metrics(p_cal: &[f64], gold_score: f64) -> ScoreMetrics {
    let expected: f64 = p_cal.iter().enumerate().map(|(i, &p)| i as f64 * p).sum();
    let mae = (expected - gold_score).abs();
    ScoreMetrics {
        expected,
        mae,
        within_1: if mae <= 1.0 { 1.0 } else { 0.0 },
    }
}

/// The split-conformal confidence recalibration floor: for each test score
/// `s`, `c'(s) = (1 + #{i in cal : s_i <= s}) / (n_cal + 1)` — the standard
/// conformal p-value CDF, exchangeability-valid recalibration. This is the
/// "conformal-naive floor" the G1 calibration gate must beat.
///
/// Monotone non-decreasing in `s` by construction (debug-asserted over the
/// s-sorted output); the (n_cal + 1) denominator keeps every value in (0, 1].
#[must_use]
pub fn conformal_naive_floor(cal: &[CalibrationPair], test_confs: &[f64]) -> Vec<f64> {
    let n_cal = cal.len();
    let scores: Vec<f64> = cal.iter().map(|p| p.conf).collect();
    let out: Vec<f64> = test_confs
        .iter()
        .map(|&s| {
            let count = scores.iter().filter(|&&si| si <= s).count();
            let v = (1 + count) as f64 / (n_cal + 1) as f64;
            // (0, 1] by construction (count in [0, n]); the clamp is
            // belt-and-braces per the G1 floor contract.
            v.clamp(1.0 / (n_cal + 1) as f64, 1.0)
        })
        .collect();

    #[cfg(debug_assertions)]
    {
        let mut sorted: Vec<(f64, f64)> = test_confs
            .iter()
            .copied()
            .zip(out.iter().copied())
            .collect();
        sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
        for w in sorted.windows(2) {
            debug_assert!(
                w[0].1 <= w[1].1,
                "conformal floor must be monotone non-decreasing in s"
            );
        }
    }
    out
}

/// The same 15-bin ECE as `hard_metrics`, over a plain (conf, correct) list.
/// Empty input → NaN (the reference convention); an input whose confidences
/// all fall in no bin (e.g. all exactly 0.0) → 0.0.
#[must_use]
pub fn ece_of(pairs: &[(f64, bool)]) -> f64 {
    if pairs.is_empty() {
        return f64::NAN;
    }
    let edges = ece_edges();
    let n = pairs.len() as f64;
    let mut cnt = [0_usize; 15];
    let mut conf_sum = [0.0_f64; 15];
    let mut corr_cnt = [0_usize; 15];
    for &(conf, correct) in pairs {
        if let Some(b) = bin_of(conf, &edges) {
            cnt[b] += 1;
            conf_sum[b] += conf;
            if correct {
                corr_cnt[b] += 1;
            }
        }
    }
    let mut ece = 0.0;
    for b in 0..15 {
        if cnt[b] > 0 {
            let weight = cnt[b] as f64 / n;
            let mean_conf = conf_sum[b] / cnt[b] as f64;
            let mean_acc = corr_cnt[b] as f64 / cnt[b] as f64;
            ece += weight * (mean_conf - mean_acc).abs();
        }
    }
    ece
}

/// The reference ECE bin edges: `np.linspace(0, 1, 16)` — 15 equal bins.
/// Interior edges are `i / 15`; a confidence sitting exactly on an interior
/// edge in exact arithmetic may differ from numpy's `i * (1/15)` by 1 ulp
/// (the knife-edge FP boundary the reference itself has). The exact
/// endpoints match: 0.0 and 1.0 are computed exactly, and binning is
/// left-open/right-closed, so conf 0.0 falls in NO bin and conf 1.0 falls in
/// the last.
#[must_use]
pub fn ece_edges() -> [f64; 16] {
    let mut edges = [0.0_f64; 16];
    for (i, e) in edges.iter_mut().enumerate() {
        *e = i as f64 / 15.0;
    }
    edges
}

/// Index of the `(edges[i], edges[i+1]]` bin containing `conf`, or None.
/// Left-open, right-closed per the reference: a conf exactly ON an edge
/// belongs to the bin it CLOSES (the edge is that bin's upper bound), and
/// `conf == edges[0]` (0.0) falls in no bin.
#[must_use]
pub fn bin_of(conf: f64, edges: &[f64]) -> Option<usize> {
    if edges.len() < 2 {
        return None;
    }
    (0..edges.len() - 1).find(|&i| conf > edges[i] && conf <= edges[i + 1])
}

/// The G1 calibration-gate verdict, spelled out for the tables and the
/// published JSON. `NoClaim` is the honest third state: the calibrator never
/// fitted (`refit` never moved it — the cal window is below the occupancy
/// floor), so the "calibrated vs raw" comparison compares the raw readout
/// against itself and there is no calibration claim to pass or fail. Never
/// read as a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum G1Verdict {
    /// Fitted AND beats both the uncalibrated readout and the conformal floor.
    Pass,
    /// Fitted but fails to beat both — including the identity fit (the
    /// calibrator looked at the evidence and concluded "no correction", which
    /// cannot strictly beat raw; the floor contract judges it).
    Fail,
    /// Nothing was fitted (cal window below the occupancy floor) — no
    /// calibration claim made.
    NoClaim,
}

/// The G1 verdict derivation — pure so the known-answer tests pin it. The
/// bool half is `g1_pass`'s wire-compatible projection (`None` for NoClaim —
/// a lane that fitted nothing must not serialize as a failed gate).
#[must_use]
pub fn g1_verdict_of(
    fitted: bool,
    n_cal_pairs: usize,
    readout_ece_cal: f64,
    readout_ece_raw: f64,
    floor_ece: f64,
) -> (Option<bool>, G1Verdict) {
    if !fitted || n_cal_pairs == 0 {
        return (None, G1Verdict::NoClaim);
    }
    let pass = readout_ece_cal < readout_ece_raw && readout_ece_cal < floor_ece;
    let verdict = if pass {
        G1Verdict::Pass
    } else {
        G1Verdict::Fail
    };
    (Some(pass), verdict)
}

// ── confusion readout (Issue 013 lever-3 probe) ────────────────────────────

/// One confused (gold → prediction) label pair, counted over the forced
/// categorical questions of one suite lane.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfusionRow {
    pub gold: String,
    pub pred: String,
    pub count: usize,
    /// count / total mispredictions over the counted questions.
    pub share_of_errors: f64,
}

/// Top-K confused pairs from `(gold_key, pred_key)` misprediction rows — the
/// pair-concentration probe behind Issue 013 lever 3 (are a suite's errors
/// pair-structured, i.e. is a fitted pair head worth fitting?). Pure reduce:
/// BTreeMap counting, ties broken by (gold, pred) — deterministic by
/// construction, never a HashMap-order artifact.
#[must_use]
pub fn confusion_top(mispairs: &[(String, String)], top: usize) -> Vec<ConfusionRow> {
    let errors = mispairs.len();
    if errors == 0 {
        return Vec::new();
    }
    let mut counts: std::collections::BTreeMap<(String, String), usize> =
        std::collections::BTreeMap::new();
    for (g, p) in mispairs {
        *counts.entry((g.clone(), p.clone())).or_default() += 1;
    }
    let mut rows: Vec<ConfusionRow> = counts
        .into_iter()
        .map(|((gold, pred), count)| ConfusionRow {
            gold,
            pred,
            count,
            share_of_errors: count as f64 / errors as f64,
        })
        .collect();
    rows.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.gold.cmp(&b.gold))
            .then_with(|| a.pred.cmp(&b.pred))
    });
    rows.truncate(top);
    rows
}

/// argmax with first-max-wins on ties (np.argmax semantics).
fn argmax_first(probs: &[f64]) -> (usize, f64) {
    let mut best = 0;
    for i in 1..probs.len() {
        if probs[i] > probs[best] {
            best = i;
        }
    }
    (best, probs[best])
}

/// Issue 024 T3: hard accuracy over the eval rows whose CASE carries no
/// leak flag — the same forced-row walk [`hard_metrics`] reads, restricted
/// to the unflagged cases. `rows` = the flattened forced rows in case
/// order (the order `forced_rows` produced); `flagged_case` = one flag per
/// case, `true` = the case is leak-flagged and is DROPPED (the same
/// semantics `scan_eval`'s flags carry — one meaning everywhere, so no
/// call site can silently invert it); `questions_per_case` = one question
/// count per case (their sum MUST equal `rows.len()` — the alignment
/// assertion, never a silent mis-walk). `None` when zero rows remain:
/// nothing measured, never a fabricated rate.
#[must_use]
pub fn subset_accuracy(
    rows: &[(usize, Vec<f64>)],
    flagged_case: &[bool],
    questions_per_case: &[usize],
) -> Option<f64> {
    assert_eq!(
        flagged_case.len(),
        questions_per_case.len(),
        "subset_accuracy: {} leak flags vs {} case question counts — the \
         flags and the case set disagree",
        flagged_case.len(),
        questions_per_case.len()
    );
    assert_eq!(
        questions_per_case.iter().sum::<usize>(),
        rows.len(),
        "subset_accuracy: the case question counts sum to {} but {} forced \
         rows were handed over — the walk would silently mis-align",
        questions_per_case.iter().sum::<usize>(),
        rows.len()
    );
    let (mut kept, mut correct) = (0usize, 0usize);
    let mut row = 0usize;
    for (ci, &q) in questions_per_case.iter().enumerate() {
        let keep = !flagged_case[ci];
        for _ in 0..q {
            let (gold, probs) = &rows[row];
            if keep {
                kept += 1;
                correct += usize::from(*gold == argmax_first(probs).0);
            }
            row += 1;
        }
    }
    (kept > 0).then(|| correct as f64 / kept as f64)
}

#[cfg(test)]
mod subset_accuracy_tests {
    use super::subset_accuracy;

    fn rows() -> Vec<(usize, Vec<f64>)> {
        // 3 cases of 1/2/1 questions; picks correct/WRONG/WRONG/correct.
        vec![
            (0, vec![0.9, 0.1]), // case 0: correct (pick 0)
            (0, vec![0.2, 0.8]), // case 1 q0: WRONG (gold 0, pick 1)
            (1, vec![0.8, 0.2]), // case 1 q1: WRONG (gold 1, pick 0)
            (1, vec![0.3, 0.7]), // case 2: correct (pick 1)
        ]
    }

    #[test]
    fn drops_only_the_flagged_cases() {
        // Flag case 1 (both its rows are wrong) → 2/2 correct among kept.
        let got = subset_accuracy(&rows(), &[false, true, false], &[1, 2, 1]);
        assert_eq!(got, Some(1.0));
        // Flag nothing → 2/4.
        let got = subset_accuracy(&rows(), &[false, false, false], &[1, 2, 1]);
        assert_eq!(got, Some(0.5));
        // Flag a CORRECT case (0) → the kept set drops to 1/3 — the de-leaked
        // read is allowed to be LOWER (the flag is a disclosure, not a boost).
        let got = subset_accuracy(&rows(), &[true, false, false], &[1, 2, 1]);
        assert_eq!(got, Some(1.0 / 3.0));
    }

    #[test]
    fn everything_flagged_is_none_never_a_zero() {
        let got = subset_accuracy(&rows(), &[true, true, true], &[1, 2, 1]);
        assert_eq!(got, None);
    }

    #[test]
    #[should_panic(expected = "mis-align")]
    fn misaligned_counts_refuse_loudly() {
        let _ = subset_accuracy(&rows(), &[false, true], &[1, 2]);
    }

    #[test]
    #[should_panic(expected = "disagree")]
    fn flag_and_case_count_mismatch_refuses() {
        let _ = subset_accuracy(&rows(), &[false, false, false], &[1, 2]);
    }
}
