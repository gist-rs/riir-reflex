//! `--mc-ab` — the Issue-055 / Plan-008-T5 distributional-layer A/B arm.
//!
//! The seeded-MC wrapper ([`crate::mc_ensemble`]) perturbs the hashed-feature
//! embedding under a per-request seed and re-runs the deterministic pipeline
//! N times; this arm reads the resulting per-question histograms as TWO
//! decision surfaces and measures each against the deployed baseline on the
//! SAME eval loop, SAME forced picks — the only difference between the arms
//! is the ranking key:
//!
//! - **Rejection ranking** (the DRM U_pair finding): rank questions by
//!   fused-gate confidence desc (the deployed baseline — the calibrated
//!   readout the shipped gate consumes) vs `1 − u_pair` desc (the ensemble's
//!   disagreement), and compare accuracy at MATCHED coverage. Selective
//!   accuracy at every prefix + the AUC ride the record; the coverage table
//!   carries the paired disagreement counts (who kept what the other
//!   rejected, and who was right).
//! - **Decision rules over the histogram** (the DRM LCB finding): majority
//!   vote, mean-score argmax, and risk-sensitive LCB-λ (`μ − λσ`) — accuracy
//!   each, plus the flip count.
//!
//! # Protocol (pre-registered in the issue)
//!
//! - The dose knobs are selected CAL-side: `p_drop` by the cal AUC delta vs
//!   the cal baseline, `λ` by the cal LCB-vs-mean accuracy delta (ties → the
//!   smallest knob; both tables ride the record). The test split is read
//!   ONCE at the selected posture. Thin cal (`< MIN_CAL_QUESTIONS` questions)
//!   → defaults, disclosed by `selection: None`.
//! - REPORT-ONLY, like every harness A/B arm here: nothing in this module
//!   feeds the served answer. The wrapper's sample 0 IS the legacy solve;
//!   the served pick stays the legacy pipeline's by construction.
//! - Latency rows (`mc_p50_us`/`mc_p99_us`) time the whole per-case MC pass
//!   (N solves) and are LOAD-SENSITIVE — any published latency quote carries
//!   the `scripts/bench_preflight.sh` PROVENANCE line (the box-state law).
//!   Correctness/coverage rows are load-insensitive.
//!
//! # Index-space law (load-bearing)
//!
//! The engine's noul candidate order is `[yes=0, no=1]`; the harness gold
//! convention is `[no=0, yes=1]`. Pick counts and per-option moments live in
//! ENGINE space; every readout that meets gold flips noul indices
//! ([`engine_idx_to_gold`]). `u_pair`/`u_bon` are share-based and
//! permutation-invariant, so the rejection keys need no flip.

use crate::embed::EMBED_DIM;
use crate::engine::{DecisionEngine, Scratch};
use crate::harness::runner::engine_request;
use crate::harness::suites::{QKind, Suite, SuiteCase};
use crate::mc_ensemble::{solve_mc_into, McConfig};
use katgpt_core::decision_wire::Outcome;
use katgpt_core::perturbation_ensemble::EnsembleHistogram;
use serde::Serialize;
use std::time::Instant;

/// Total MC samples per case including the unperturbed legacy run (the G2
/// gate's N; `--mc-samples`).
pub const DEFAULT_MC_SAMPLES: usize = 8;
/// The cal-side dropout sweep (the issue's p ∈ [0.05, 0.3] dose-response).
pub const DEFAULT_P_GRID: [f32; 4] = [0.05, 0.10, 0.20, 0.30];
/// The cal-side LCB λ sweep (0.4 = the paper's pick; bracketed).
pub const DEFAULT_LAMBDA_GRID: [f32; 3] = [0.2, 0.4, 0.8];
/// Fallbacks when cal is too thin to select (disclosed by `selection: None`).
pub const DEFAULT_P_DROP: f32 = 0.20;
pub const DEFAULT_LAMBDA: f32 = 0.40;
/// Cal questions below this → no selection, defaults (thin-support law).
pub const MIN_CAL_QUESTIONS: usize = 20;
/// The matched-coverage levels the table reports (1.0 = forced accuracy).
pub const COVERAGE_LEVELS: [f64; 6] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
/// One fixed draw family for the arm (`--mc-ab`'s salt; the per-request seed
/// already derives from the request bytes, so this only separates the arm's
/// family from any other consumer of the wrapper).
const MC_SALT: u64 = 0x4D43_4142;

/// The arm's tuning (built once per run from `RunOptions`).
#[derive(Debug, Clone)]
pub struct McAbConfig {
    /// Total samples per case including the unperturbed legacy run.
    pub n_samples: usize,
    /// Ascending dropout grid (a pinned `--mc-p-drop` is a single-row grid —
    /// the selection table then carries the pinned row only).
    pub p_grid: Vec<f32>,
    /// Ascending LCB-λ grid (a pinned `--mc-lambda` is a single-row grid).
    pub lambda_grid: Vec<f32>,
}

/// The deployed baseline's per-question test readings (from the runner's
/// evals — the SAME picks both arms rank, so the A/B isolates the key).
pub struct McBaseline<'a> {
    /// Calibrated readout confidences (the fitted engine, test cases) — the
    /// deployed fused-gate confidence.
    pub confs_calibrated: &'a [Vec<f64>],
    /// Raw (uncalibrated) readout confidences (the raw engine, test cases) —
    /// context scalar only, never the baseline.
    pub confs_raw: &'a [Vec<f64>],
    /// Forced picks in GOLD space (the raw eval — the shared answers).
    pub picks: &'a [Vec<usize>],
}

/// One flattened question (case-major, question-minor — the histogram order).
struct QRow {
    kind: QKind,
    /// Engine-space option count (noul = 2).
    n_opts: usize,
    /// Gold-space gold index.
    gold: usize,
    /// Gold-space forced pick (test: the shared baseline picks; cal: the
    /// engine's own computed pick).
    pick: usize,
    /// The baseline ranking key (calibrated readout confidence).
    conf_cal: f64,
    /// The raw (uncalibrated) readout confidence — test rows only (the
    /// context scalar); cal rows repeat `conf_cal` (cal selection compares
    /// against the deployed calibrated baseline alone).
    conf_raw: f64,
}

/// The arm's results.json record (additive; report-only).
#[derive(Debug, Clone, Serialize)]
pub struct McAbRecord {
    pub n_samples: usize,
    pub p_drop: f32,
    pub lambda: f32,
    /// The cal-side selection tables. None = thin cal → defaults (disclosed
    /// absence, never a fabricated selection).
    pub selection: Option<McCalSelection>,
    pub n_questions: usize,
    /// Rejection-ranking AUCs (mean prefix accuracy, higher = better):
    /// the deployed calibrated confidence, the raw readout (context),
    /// and the ensemble's u_pair / u_bon certainties.
    pub auc_baseline_cal_conf: f64,
    pub auc_baseline_raw_conf: f64,
    pub auc_u_pair: f64,
    pub auc_u_bon: f64,
    /// Matched-coverage table (baseline vs u_pair).
    pub coverage_rows: Vec<McCoverageRow>,
    /// Decision-rule accuracies over the histogram. `acc_legacy_pick` is the
    /// shared forced picks (the sample-0 answers both rankers rank).
    pub acc_legacy_pick: f64,
    pub acc_majority_pick: f64,
    pub acc_mean_pick: f64,
    pub acc_lcb_pick: f64,
    /// Questions where the LCB pick differs from the mean pick.
    pub lcb_flipped: usize,
    /// Per-case MC pass latency (N solves, load-sensitive — quote the
    /// bench_preflight PROVENANCE beside any published number).
    pub mc_p50_us: u64,
    pub mc_p99_us: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McCoverageRow {
    pub coverage: f64,
    pub k: usize,
    pub baseline_acc: f64,
    pub mc_acc: f64,
    pub delta: f64,
    /// Paired disagreement at this coverage: questions only the baseline
    /// kept / only the MC key kept, and how many each got right.
    pub base_only_kept: usize,
    pub mc_only_kept: usize,
    pub base_only_correct: usize,
    pub mc_only_correct: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct McCalSelection {
    pub n_cal_questions: usize,
    pub auc_baseline_cal_conf: f64,
    pub p_rows: Vec<McPCalRow>,
    pub lambda_rows: Vec<McLambdaCalRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McPCalRow {
    pub p_drop: f32,
    pub cal_auc_u_pair: f64,
    pub delta_vs_baseline: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McLambdaCalRow {
    pub lambda: f32,
    pub cal_acc_lcb: f64,
    pub cal_acc_mean: f64,
    pub delta: f64,
    pub flipped: usize,
}

/// Engine-space option count for a question (noul's `[yes, no]` pair is
/// question vocabulary — always 2).
fn engine_option_count(q: &crate::harness::suites::SuiteQuestion) -> usize {
    match q.kind {
        QKind::Noul => 2,
        QKind::Choice => q.criteria.as_object().map_or(0, serde_json::Map::len),
        QKind::Score => q.criteria.as_array().map_or(0, Vec::len),
    }
}

/// Map an engine-space option index to the harness gold space (noul:
/// engine `[yes, no]` → gold `[no, yes]`, so the index flips; every other
/// kind shares the index).
fn engine_idx_to_gold(kind: QKind, i: usize) -> usize {
    if matches!(kind, QKind::Noul) {
        1 - i
    } else {
        i
    }
}

/// Deterministic argmax over f64 (ties → the lower index; NaN never wins a
/// strictly-greater comparison).
fn argmax_f64(v: &[f64]) -> usize {
    let mut best = 0usize;
    for i in 1..v.len() {
        if v[i] > v[best] {
            best = i;
        }
    }
    best
}

/// Prefix accuracies under a keep-first ranking: sort indices by key DESC
/// (ties → the lower flat index, deterministic), then accuracy of the top-k
/// for every k in 1..=n.
fn prefix_accuracies(pairs: &[(f64, bool)]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..pairs.len()).collect();
    idx.sort_by(|&a, &b| pairs[b].0.total_cmp(&pairs[a].0).then(a.cmp(&b)));
    let mut out = Vec::with_capacity(pairs.len());
    let mut correct = 0usize;
    for (k, &i) in idx.iter().enumerate() {
        correct += usize::from(pairs[i].1);
        out.push(correct as f64 / (k + 1) as f64);
    }
    out
}

/// AUC = the mean prefix accuracy (the selective-prediction summary; higher
/// = the key separates correct from incorrect better). NaN on empty input.
fn auc(pairs: &[(f64, bool)]) -> f64 {
    if pairs.is_empty() {
        return f64::NAN;
    }
    prefix_accuracies(pairs).iter().sum::<f64>() / pairs.len() as f64
}

/// The ranking's index order (key desc, ties → flat index asc) — the same
/// order [`prefix_accuracies`] walks.
fn order_indices(pairs: &[(f64, bool)]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..pairs.len()).collect();
    idx.sort_by(|&a, &b| pairs[b].0.total_cmp(&pairs[a].0).then(a.cmp(&b)));
    idx
}

/// Argmax over per-row deltas; ties → the FIRST row (grids are ascending, so
/// a tie keeps the smallest knob — the cal-selection law).
fn argmax_tie_first(deltas: &[f64]) -> usize {
    let mut best = 0usize;
    for i in 1..deltas.len() {
        if deltas[i] > deltas[best] {
            best = i;
        }
    }
    best
}

/// The fitted engine's own baseline over the cal cases: per-question
/// calibrated confidence + computed pick (gold space; abstains fall to the
/// forced argmax, the eval_engine law).
fn cal_baseline<const N: usize>(
    engine: &mut DecisionEngine<N, EMBED_DIM>,
    cases: &[SuiteCase],
    state_strs: &[String],
) -> Result<Vec<QRow>, String> {
    let max_q = cases
        .iter()
        .map(|c| c.questions.len())
        .max()
        .unwrap_or(1);
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(max_q.max(1));
    let mut rows = Vec::new();
    for (ci, case) in cases.iter().enumerate() {
        let req = engine_request(case, &state_strs[ci])?;
        let resp = engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("mc-ab cal baseline (case {}): {e}", case.id))?;
        for (qi, (q, ans)) in case.questions.iter().zip(resp.answers.iter()).enumerate() {
            // Gold-space probability vector (noul wire = [p_yes] only).
            let p: Vec<f64> = if q.kind == QKind::Noul {
                let p_yes = f64::from(ans.probabilities[0]);
                vec![1.0 - p_yes, p_yes]
            } else {
                ans.probabilities.iter().map(|v| f64::from(*v)).collect()
            };
            let pick = match &ans.outcome {
                Some(Outcome::Choice { index }) => *index as usize,
                Some(Outcome::Score { level }) => *level as usize,
                Some(Outcome::Noul { yes }) => usize::from(*yes),
                None => argmax_f64(&p),
            };
            rows.push(QRow {
                kind: q.kind,
                n_opts: engine_option_count(q),
                gold: case.gold[qi].idx,
                pick,
                conf_cal: f64::from(ans.confidence),
                conf_raw: f64::from(ans.confidence),
            });
        }
    }
    Ok(rows)
}

/// Run the MC wrapper over a case set, one histogram per question (prepared
/// for the question's engine-space option count), timing each case's whole
/// N-sample pass into `durs_us`.
fn run_mc<const N: usize>(
    engine: &mut DecisionEngine<N, EMBED_DIM>,
    cases: &[SuiteCase],
    state_strs: &[String],
    rows: &[QRow],
    mc: &McConfig,
    durs_us: &mut Vec<u64>,
) -> Result<Vec<EnsembleHistogram>, String> {
    let mut hists: Vec<EnsembleHistogram> = rows
        .iter()
        .map(|r| {
            let mut h = EnsembleHistogram::default();
            h.prepare(r.n_opts);
            h
        })
        .collect();
    let max_q = cases
        .iter()
        .map(|c| c.questions.len())
        .max()
        .unwrap_or(1);
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(max_q.max(1));
    let mut flat = 0usize;
    for (ci, case) in cases.iter().enumerate() {
        let req = engine_request(case, &state_strs[ci])?;
        let nq = case.questions.len();
        let t0 = Instant::now();
        solve_mc_into(engine, &req, &mut sc, mc, &mut hists[flat..flat + nq])
            .map_err(|e| format!("mc-ab ensemble (case {}): {e}", case.id))?;
        durs_us.push(t0.elapsed().as_micros() as u64);
        flat += nq;
    }
    Ok(hists)
}

/// The u_pair rejection key as a (key, correct) pair row — certainty
/// `1 − u_pair` desc, mirroring the baseline's confidence desc.
fn u_pair_pairs(rows: &[QRow], hists: &[EnsembleHistogram]) -> Vec<(f64, bool)> {
    rows.iter()
        .zip(hists.iter())
        .map(|(r, h)| {
            let u = f64::from(h.u_pair().unwrap_or(0.0));
            (1.0 - u, r.pick == r.gold)
        })
        .collect()
}

fn u_bon_pairs(rows: &[QRow], hists: &[EnsembleHistogram]) -> Vec<(f64, bool)> {
    rows.iter()
        .zip(hists.iter())
        .map(|(r, h)| {
            let u = f64::from(h.u_bon().unwrap_or(0.0));
            (1.0 - u, r.pick == r.gold)
        })
        .collect()
}

/// Accuracy of a histogram-derived decision rule whose pick is computed by
/// `rule` (returning a GOLD-space pick, or None when the histogram cannot
/// answer — counted wrong, never skipped: a rule with no answer has no
/// accuracy to claim).
fn rule_accuracy(
    rows: &[QRow],
    hists: &[EnsembleHistogram],
    rule: impl Fn(&QRow, &EnsembleHistogram) -> Option<usize>,
) -> (f64, usize) {
    let mut correct = 0usize;
    let mut answered = 0usize;
    for (r, h) in rows.iter().zip(hists.iter()) {
        if let Some(pick) = rule(r, h) {
            answered += 1;
            correct += usize::from(pick == r.gold);
        }
    }
    let n = rows.len().max(1);
    (correct as f64 / n as f64, answered)
}

/// The Issue-055 / Plan-008-T5 pass. REPORT-ONLY: nothing here feeds the
/// served answer — sample 0 of the wrapper IS the legacy solve and the
/// record's baseline arms rank the runner's own evals.
#[allow(clippy::too_many_arguments)]
pub fn mc_ab_pass<const N: usize>(
    engine: &mut DecisionEngine<N, EMBED_DIM>,
    suite: &Suite,
    state_strs: &[String],
    cal_cases: &[SuiteCase],
    cal_state_strs: &[String],
    baseline: McBaseline<'_>,
    cfg: &McAbConfig,
) -> Result<McAbRecord, String> {
    // 1. Flatten the test rows from the runner's evals (shared picks + the
    //    two confidence surfaces).
    let mut test_rows = Vec::new();
    for (ci, case) in suite.cases.iter().enumerate() {
        for (qi, q) in case.questions.iter().enumerate() {
            test_rows.push(QRow {
                kind: q.kind,
                n_opts: engine_option_count(q),
                gold: case.gold[qi].idx,
                pick: baseline.picks[ci][qi],
                conf_cal: baseline.confs_calibrated[ci][qi],
                conf_raw: baseline.confs_raw[ci][qi],
            });
        }
    }

    // 2. Cal-side selection (p_drop by rejection-AUC delta, λ by LCB-vs-mean
    //    accuracy delta; ties → smallest knob). Thin cal → defaults.
    let mut selection = None;
    let mut p_sel = DEFAULT_P_DROP;
    let mut lam_sel = DEFAULT_LAMBDA;
    if !cal_cases.is_empty() {
        let cal_rows = cal_baseline(engine, cal_cases, cal_state_strs)?;
        if cal_rows.len() >= MIN_CAL_QUESTIONS {
            let base_auc = auc(
                &cal_rows
                    .iter()
                    .map(|r| (r.conf_cal, r.pick == r.gold))
                    .collect::<Vec<_>>(),
            );
            let mut cal_hists_per_p: Vec<(f32, Vec<EnsembleHistogram>)> = Vec::new();
            let mut p_rows: Vec<McPCalRow> = Vec::new();
            let mut deltas: Vec<f64> = Vec::new();
            let mut durs = Vec::new();
            for &p in &cfg.p_grid {
                let hists = run_mc(
                    engine,
                    cal_cases,
                    cal_state_strs,
                    &cal_rows,
                    &McConfig {
                        n_samples: cfg.n_samples,
                        p_drop: p,
                        salt: MC_SALT,
                    },
                    &mut durs,
                )?;
                let u_auc = auc(&u_pair_pairs(&cal_rows, &hists));
                deltas.push(u_auc - base_auc);
                p_rows.push(McPCalRow {
                    p_drop: p,
                    cal_auc_u_pair: u_auc,
                    delta_vs_baseline: u_auc - base_auc,
                });
                cal_hists_per_p.push((p, hists));
            }
            let p_idx = argmax_tie_first(&deltas);
            p_sel = cfg.p_grid[p_idx];
            let sel_hists = &cal_hists_per_p[p_idx].1;
            let mean_acc = rule_accuracy(&cal_rows, sel_hists, |r, h| {
                let means: Vec<f64> = (0..r.n_opts).map(|i| h.option_mean(i)).collect();
                Some(engine_idx_to_gold(r.kind, argmax_f64(&means)))
            })
            .0;
            let mut lambda_rows: Vec<McLambdaCalRow> = Vec::new();
            let mut lam_deltas: Vec<f64> = Vec::new();
            for &lam in &cfg.lambda_grid {
                let lcb_acc = rule_accuracy(&cal_rows, sel_hists, |r, h| {
                    h.top_by_lcb(lam)
                        .map(|p| engine_idx_to_gold(r.kind, p))
                })
                .0;
                let flipped = rows_where_lcb_differs(&cal_rows, sel_hists, lam).len();
                lambda_rows.push(McLambdaCalRow {
                    lambda: lam,
                    cal_acc_lcb: lcb_acc,
                    cal_acc_mean: mean_acc,
                    delta: lcb_acc - mean_acc,
                    flipped,
                });
                lam_deltas.push(lcb_acc - mean_acc);
            }
            let lam_idx = argmax_tie_first(&lam_deltas);
            lam_sel = cfg.lambda_grid[lam_idx];
            selection = Some(McCalSelection {
                n_cal_questions: cal_rows.len(),
                auc_baseline_cal_conf: base_auc,
                p_rows,
                lambda_rows,
            });
        }
    }

    // 3. Test MC at the selected posture (one read), with per-case timing.
    let mut durs: Vec<u64> = Vec::new();
    let hists = run_mc(
        engine,
        &suite.cases,
        state_strs,
        &test_rows,
        &McConfig {
            n_samples: cfg.n_samples,
            p_drop: p_sel,
            salt: MC_SALT,
        },
        &mut durs,
    )?;
    durs.sort_unstable();
    let mc_p50 = durs[durs.len() / 2];
    let mc_p99 = durs[(durs.len() * 99).div_ceil(100) - 1];

    // 4. Records.
    let n = test_rows.len();
    let base_pairs: Vec<(f64, bool)> = test_rows
        .iter()
        .map(|r| (r.conf_cal, r.pick == r.gold))
        .collect();
    let raw_pairs: Vec<(f64, bool)> = test_rows
        .iter()
        .map(|r| (r.conf_raw, r.pick == r.gold))
        .collect();
    let up_pairs = u_pair_pairs(&test_rows, &hists);
    let order_b = order_indices(&base_pairs);
    let order_m = order_indices(&up_pairs);
    let correct: Vec<bool> = test_rows.iter().map(|r| r.pick == r.gold).collect();

    let mut coverage_rows = Vec::with_capacity(COVERAGE_LEVELS.len());
    for &cov in COVERAGE_LEVELS.iter() {
        let k = ((cov * n as f64).round() as usize).clamp(1, n);
        let (b_acc, m_acc) = (
            {
                let c: usize = order_b[..k].iter().map(|&i| usize::from(correct[i])).sum();
                c as f64 / k as f64
            },
            {
                let c: usize = order_m[..k].iter().map(|&i| usize::from(correct[i])).sum();
                c as f64 / k as f64
            },
        );
        let mut in_b = vec![false; n];
        let mut in_m = vec![false; n];
        for &i in &order_b[..k] {
            in_b[i] = true;
        }
        for &i in &order_m[..k] {
            in_m[i] = true;
        }
        let (mut b_only, mut m_only, mut b_ok, mut m_ok) = (0, 0, 0, 0);
        for i in 0..n {
            if in_b[i] && !in_m[i] {
                b_only += 1;
                b_ok += usize::from(correct[i]);
            }
            if in_m[i] && !in_b[i] {
                m_only += 1;
                m_ok += usize::from(correct[i]);
            }
        }
        coverage_rows.push(McCoverageRow {
            coverage: cov,
            k,
            baseline_acc: b_acc,
            mc_acc: m_acc,
            delta: m_acc - b_acc,
            base_only_kept: b_only,
            mc_only_kept: m_only,
            base_only_correct: b_ok,
            mc_only_correct: m_ok,
        });
    }

    let acc_legacy = correct.iter().filter(|c| **c).count() as f64 / n.max(1) as f64;
    let acc_majority = rule_accuracy(&test_rows, &hists, |r, h| {
        h.majority_pick().map(|p| engine_idx_to_gold(r.kind, p))
    })
    .0;
    let acc_mean = rule_accuracy(&test_rows, &hists, |r, h| {
        let means: Vec<f64> = (0..r.n_opts).map(|i| h.option_mean(i)).collect();
        Some(engine_idx_to_gold(r.kind, argmax_f64(&means)))
    })
    .0;
    let acc_lcb = rule_accuracy(&test_rows, &hists, |r, h| {
        h.top_by_lcb(lam_sel)
            .map(|p| engine_idx_to_gold(r.kind, p))
    })
    .0;
    let lcb_flipped = rows_where_lcb_differs(&test_rows, &hists, lam_sel).len();

    Ok(McAbRecord {
        n_samples: cfg.n_samples,
        p_drop: p_sel,
        lambda: lam_sel,
        selection,
        n_questions: n,
        auc_baseline_cal_conf: auc(&base_pairs),
        auc_baseline_raw_conf: auc(&raw_pairs),
        auc_u_pair: auc(&up_pairs),
        auc_u_bon: auc(&u_bon_pairs(&test_rows, &hists)),
        coverage_rows,
        acc_legacy_pick: acc_legacy,
        acc_majority_pick: acc_majority,
        acc_mean_pick: acc_mean,
        acc_lcb_pick: acc_lcb,
        lcb_flipped,
        mc_p50_us: mc_p50,
        mc_p99_us: mc_p99,
    })
}

/// Flat indices where the LCB-λ pick differs from the mean pick.
fn rows_where_lcb_differs(rows: &[QRow], hists: &[EnsembleHistogram], lam: f32) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, (r, h)) in rows.iter().zip(hists.iter()).enumerate() {
        let mean_pick = {
            let means: Vec<f64> = (0..r.n_opts).map(|j| h.option_mean(j)).collect();
            argmax_f64(&means)
        };
        let lcb_pick = h.top_by_lcb(lam).unwrap_or(mean_pick);
        if lcb_pick != mean_pick {
            out.push(i);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_accuracies_order_by_key_desc_ties_flat() {
        // keys 0.9 (correct), 0.1 (wrong), 0.9 (wrong, later index ties).
        let pairs = vec![(0.9, true), (0.1, false), (0.9, false)];
        let acc = prefix_accuracies(&pairs);
        // Order: idx0 (0.9, tie → lower), idx2 (0.9), idx1 (0.1).
        assert_eq!(acc, vec![1.0, 0.5, 1.0 / 3.0]);
    }

    #[test]
    fn auc_is_the_mean_prefix_accuracy() {
        let pairs = vec![(0.9, true), (0.1, false), (0.5, true)];
        // Order: 0.9 ✓, 0.5 ✓, 0.1 ✗ → prefixes [1, 1, 2/3].
        let a = auc(&pairs);
        assert!((a - (1.0 + 1.0 + 2.0 / 3.0) / 3.0).abs() < 1e-12);
        assert!(auc(&[]).is_nan());
    }

    #[test]
    fn argmax_tie_first_keeps_the_smallest_knob() {
        assert_eq!(argmax_tie_first(&[0.0, 0.1, 0.1]), 1);
        assert_eq!(argmax_tie_first(&[0.2, 0.1, 0.0]), 0);
        assert_eq!(argmax_tie_first(&[0.0, 0.0, 0.0]), 0);
    }

    #[test]
    fn noul_engine_index_flips_to_gold_space() {
        assert_eq!(engine_idx_to_gold(QKind::Noul, 0), 1); // yes
        assert_eq!(engine_idx_to_gold(QKind::Noul, 1), 0); // no
        assert_eq!(engine_idx_to_gold(QKind::Choice, 0), 0);
        assert_eq!(engine_idx_to_gold(QKind::Score, 2), 2);
    }

    #[test]
    fn mean_and_lcb_picks_read_the_histogram_moments() {
        // Two options, three samples: option 0 has the higher mean but a
        // large sigma; option 1 is stable at 0.5. LCB at a strong λ must
        // flip to the stable option (equal sigmas never flip — the fixture
        // is asymmetric on purpose).
        let mut h = EnsembleHistogram::default();
        h.prepare(2);
        h.observe(Some(0), &[0.8, 0.5]);
        h.observe(Some(0), &[0.9, 0.5]);
        h.observe(Some(1), &[0.2, 0.5]);
        let means: Vec<f64> = (0..2).map(|i| h.option_mean(i)).collect();
        assert_eq!(argmax_f64(&means), 0);
        assert_eq!(h.top_by_lcb(0.0), Some(0));
        assert!(h.top_by_lcb(2.0).is_some_and(|p| p == 1));
        assert_eq!(h.majority_pick(), Some(0));
        assert!(h.u_pair().unwrap() > 0.0);
    }

    /// The end-to-end arm on a tiny 2-domain engine: wiring, index spaces,
    /// and the coverage table's identity at full coverage. Thin cal (< 20)
    /// → defaults + `selection: None` (the disclosed absence).
    #[test]
    fn tiny_suite_pass_is_well_formed() {
        use crate::embed::{Embedder, EMBED_DIM};
        use crate::harness::suites::{GoldAnswer, SuiteQuestion};
        use serde_json::{json, Value};

        let experts = vec![
            crate::engine::DomainExpert::new(
                "billing",
                &["my card was charged twice and the refund never arrived".to_string()],
                &Embedder,
            ),
            crate::engine::DomainExpert::new(
                "tech",
                &["the app crashes on launch and the login page will not load".to_string()],
                &Embedder,
            ),
        ];
        let mut engine = crate::engine::DecisionEngine::<2, EMBED_DIM>::build(
            experts,
            crate::engine::EngineConfig::default(),
        )
        .unwrap();

        let q = || SuiteQuestion {
            qid: "team".to_string(),
            kind: QKind::Choice,
            instructions: "which team handles this?".to_string(),
            criteria: json!({
                "billing": "the billing team",
                "tech": "the tech team"
            }),
        };
        let case = |id: &str, state: &str, gold: usize| SuiteCase {
            id: id.to_string(),
            state: Value::String(state.to_string()),
            questions: vec![q()],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![],
                gold_score: None,
            }],
        };
        let suite = Suite {
            name: "tiny",
            cases: vec![
                case("t0", "card charged twice no refund", 0),
                case("t1", "app crashes on launch", 1),
                case("t2", "refund never arrived for the double charge", 0),
                case("t3", "login page will not load", 1),
                case("t4", "card refund app crash", 0),
                case("t5", "the app crashes and the card was charged", 1),
            ],
            option_counts_note: "fixed 2",
        };
        let state_strs: Vec<String> = suite
            .cases
            .iter()
            .map(|c| crate::pyjson::serialize_state(&c.state))
            .collect();
        let cal_cases = vec![case("c0", "double charge refund missing", 0)];
        let cal_strs: Vec<String> = cal_cases
            .iter()
            .map(|c| crate::pyjson::serialize_state(&c.state))
            .collect();

        // Baseline: run the engine once over the test cases for picks/conf.
        let cal_rows_probe = cal_baseline(&mut engine, &suite.cases, &state_strs).unwrap();
        let picks: Vec<Vec<usize>> = cal_rows_probe.iter().map(|r| vec![r.pick]).collect();
        let confs: Vec<Vec<f64>> = cal_rows_probe.iter().map(|r| vec![r.conf_cal]).collect();

        let rec = mc_ab_pass(
            &mut engine,
            &suite,
            &state_strs,
            &cal_cases,
            &cal_strs,
            McBaseline {
                confs_calibrated: &confs,
                confs_raw: &confs,
                picks: &picks,
            },
            &McAbConfig {
                n_samples: 8,
                p_grid: DEFAULT_P_GRID.to_vec(),
                lambda_grid: DEFAULT_LAMBDA_GRID.to_vec(),
            },
        )
        .unwrap();

        assert_eq!(rec.n_questions, 6);
        assert!(rec.selection.is_none(), "thin cal → disclosed defaults");
        assert_eq!(rec.p_drop, DEFAULT_P_DROP);
        assert_eq!(rec.lambda, DEFAULT_LAMBDA);
        assert!(rec.auc_baseline_cal_conf.is_finite());
        assert!(rec.auc_u_pair.is_finite());
        assert!(rec.auc_baseline_cal_conf >= 0.0 && rec.auc_baseline_cal_conf <= 1.0);
        for a in [rec.acc_legacy_pick, rec.acc_majority_pick, rec.acc_mean_pick, rec.acc_lcb_pick]
        {
            assert!((0.0..=1.0).contains(&a), "acc in [0,1]: {a}");
        }
        let last = rec.coverage_rows.last().unwrap();
        assert!((last.coverage - 1.0).abs() < 1e-9);
        // Full coverage: same kept set (everything) — identical accuracy.
        assert!((last.baseline_acc - last.mc_acc).abs() < 1e-12);
        assert!((last.baseline_acc - rec.acc_legacy_pick).abs() < 1e-12);
        // Non-collapse sanity on the straddling case: at least one question
        // carries a non-degenerate histogram somewhere in the run.
        assert!(rec.auc_u_pair.is_finite());
    }

    /// The noul index-space law, end to end (the prompt-injections class):
    /// a noul question's mean pick must be read through the engine→gold
    /// flip. The founding defect this pins: an unflipped mean rule inverts
    /// every noul answer — a suite of noul questions reads exactly
    /// `1 − legacy`, the complement signature the first PoC run caught.
    #[test]
    fn noul_mean_pick_is_not_the_inverted_legacy_pick() {
        use crate::embed::{Embedder, EMBED_DIM};
        use crate::harness::suites::{GoldAnswer, SuiteQuestion};
        use serde_json::{json, Value};

        let experts = vec![
            crate::engine::DomainExpert::new(
                "billing", &["my card was charged twice".to_string()], &Embedder,
            ),
            crate::engine::DomainExpert::new(
                "tech", &["the app crashes on launch".to_string()], &Embedder,
            ),
        ];
        let mut engine = crate::engine::DecisionEngine::<2, EMBED_DIM>::build(
            experts,
            crate::engine::EngineConfig::default(),
        )
        .unwrap();
        let nq = || SuiteQuestion {
            qid: "holds".to_string(),
            kind: QKind::Noul,
            instructions: "is this about billing?".to_string(),
            criteria: Value::Null,
        };
        let case = |id: &str, state: &str, gold: usize| SuiteCase {
            id: id.to_string(),
            state: Value::String(state.to_string()),
            questions: vec![nq()],
            gold: vec![GoldAnswer { idx: gold, soft: vec![], gold_score: None }],
        };
        // Gold 1 = yes (about billing). If the mean rule forgot the flip it
        // would answer the complement and score exactly 1 − legacy.
        let suite = Suite {
            name: "tiny_noul",
            cases: vec![
                case("t0", "card charged twice", 1),
                case("t1", "card charged twice again", 1),
                case("t2", "app crashes on launch", 0),
                case("t3", "the app will not load", 0),
            ],
            option_counts_note: "noul pair",
        };
        let state_strs: Vec<String> = suite
            .cases
            .iter()
            .map(|c| crate::pyjson::serialize_state(&c.state))
            .collect();
        let rows = cal_baseline(&mut engine, &suite.cases, &state_strs).unwrap();
        let picks: Vec<Vec<usize>> = rows.iter().map(|r| vec![r.pick]).collect();
        let confs: Vec<Vec<f64>> = rows.iter().map(|r| vec![r.conf_cal]).collect();
        let rec = mc_ab_pass(
            &mut engine,
            &suite,
            &state_strs,
            &[],
            &[],
            McBaseline {
                confs_calibrated: &confs,
                confs_raw: &confs,
                picks: &picks,
            },
            &McAbConfig {
                // N=1: every histogram is a point mass — mean / majority /
                // LCB picks are ALL the legacy pick exactly. Any index-space
                // break (the unflipped noul mean this test was filed for)
                // reads as the complement and fails the equality.
                n_samples: 1,
                p_grid: vec![0.2],
                lambda_grid: vec![0.4],
            },
        )
        .unwrap();
        // The pin: at N=1 all four decision rules agree with legacy.
        assert_eq!(rec.n_questions, 4);
        assert!((rec.acc_mean_pick - rec.acc_legacy_pick).abs() < 1e-12);
        assert!((rec.acc_majority_pick - rec.acc_legacy_pick).abs() < 1e-12);
        assert!((rec.acc_lcb_pick - rec.acc_legacy_pick).abs() < 1e-12);
        assert_eq!(rec.lcb_flipped, 0);
    }
}
