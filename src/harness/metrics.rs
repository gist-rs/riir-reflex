//! Metric definitions ported verbatim from `.docs/laya_bench_protocols.md` §5
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

    let pairs: Vec<(f64, bool)> = confs
        .iter()
        .copied()
        .zip(correct.iter().copied())
        .collect();
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
    let tv: f64 = 0.5 * pp.iter().zip(&gp).map(|(&p, &g)| (p - g).abs()).sum::<f64>();
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
        let mut sorted: Vec<(f64, f64)> =
            test_confs.iter().copied().zip(out.iter().copied()).collect();
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
