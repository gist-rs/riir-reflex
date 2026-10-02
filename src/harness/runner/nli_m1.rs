//! Issue 047 M1 — the pick/confidence-separation reopen lane over the
//! xnli_en VALIDATION split (`--nli-m1`; pre-registered by
//! `.plans/006_nli_m1_reopen.md`, which fixes the posture, grids, seeds,
//! promotion legs and void criteria BEFORE any validation number exists).
//!
//! Posture (the issue's own words): keep the blend's pick; take confidence
//! from a 3-feature logistic `σ(b + w1·logit(c_eng) + w2·margin +
//! w3·agree)`. The binding rules are the verdict review's R1–R6:
//!
//! - **R1** — the confirmation surface is the ~2490-item validation split,
//!   read ONCE. The 300-item test split is never touched: this lane
//!   refuses loudly on any suite other than `xnli_en_val` (whose
//!   registry entry reads the `validation` split from disk).
//! - **R2** — one primary posture (M1); M2 (log-odds combination) and M3
//!   (gated switch) are computed in the SAME single read and REPORTED
//!   WITHOUT promotion. M4 (fit-data scale) is composed into the one arm
//!   (the grown pool) rather than run separately.
//! - **R3** — the `NliLda` head is fit K-fold over the pool; every pool
//!   item's head quantities are OUT-OF-FOLD. The deployed head is the
//!   full-pool fit. The gate (logistic + piecewise-Platt) is fit on the
//!   OOF pairs only.
//! - **R4** — paired-bootstrap intervals (B = 2000, seed 20260928) on the
//!   accuracy delta and the ECE delta; the SAME replicate indices feed
//!   both arms of each comparison.
//! - **R5** — per-item pick logging ([`M1ItemLog`]) for every validation
//!   item, so McNemar / win-loss / bootstrap are recomputable after the
//!   run.
//! - **R6** — AUROC (confidence vs correctness) and the Brier resolution
//!   term reported BESIDE every ECE, including the engine's own surface
//!   (the near-constant-confidence ECE-gaming disclosure the review
//!   demanded).
//!
//! Pool architecture (all fit-side; no validation item participates in
//! any fit): cal front (corpus-free by construction) + train-rest rows
//! EXCLUDING (a) the corpus-cap docs — the first `effective_cap` valid
//! docs per label in rest order, the engine build's exact
//! `.filter(label).take(cap)` law — and (b) any item whose premise string
//! equals a corpus doc's premise (the MultiNLI premise-sharing guard).
//! Disclosed residual bias: pool rows remain inside the NB count tables;
//! at ~6.6k docs per label each row's own-token mass is ~1/6600 of its
//! table — measured-negligible, recorded in the plan rather than
//! engineered around.
use std::collections::{HashMap, HashSet};

use katgpt_core::sigmoid_calibration::CalibratedGateSet;
use serde::Serialize;
use serde_json::Value;

use super::nli_lane::{blend_pick_conf, pair_features, pair_state, well_shaped, NliLda, OracleRow, FEAT_DIM};
use super::{Eval, Suite, SuiteCase};
use crate::harness::metrics::{
    auroc_of, brier_resolution_of, conformal_naive_floor, CalibrationPair, ece_of, nll_of,
};
use crate::harness::suites::{build_xnli_en, train_row_label, SplitMix64};

/// The only suite this lane reads (R1: the confirmation surface is the
/// validation split; the suite's registry entry wires `eval_split`).
const SUITE_NAME: &str = "xnli_en_val";

/// The blend λ ladder — 069's grid verbatim.
pub const LAMBDAS: [f64; 8] = [0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0];

/// Cross-fit folds (R3; the plan's K=10).
const K_FOLDS: usize = 10;

/// The M1 logistic's L2 ridge on the three weights (never the intercept).
const RIDGE: f64 = 1e-3;

/// Paired-bootstrap replicates + seed (R4; the plan's fixed values).
const BOOTSTRAP_B: usize = 2000;
const BOOTSTRAP_SEED: u64 = 20260928;

/// The house arming bar (069's promotion criterion, leg 3).
const ACC_BAR: f64 = 0.05;

/// Confidence clamp before `logit` (the plan's CONF_EPS).
const CONF_EPS: f64 = 1e-4;

/// M3's gated-switch grids.
const M3_TAUS: [f64; 5] = [0.5, 0.6, 0.7, 0.8, 0.9];
const M3_MARGINS: [f64; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0];

// ── inputs ──────────────────────────────────────────────────────────────

/// Everything the pass reads, precomputed by the runner (the pass itself
/// never touches an engine — pure data in, record out).
pub struct M1Inputs<'a> {
    pub spec_name: &'a str,
    pub suite: &'a Suite,
    /// Deployed RAW engine over the validation cases (the blend's pe +
    /// the A0 picks).
    pub raw_eval: &'a Eval,
    /// Deployed CALIBRATED engine over the validation cases (x1).
    pub cal_eval_test: &'a Eval,
    /// Per-validation-item conformal floor pairs `(floor_conf, eng_ok)` —
    /// the runner's own `floor_pairs`, the binding G1 floor leg's bytes.
    pub floor_pairs: &'a [(f64, bool)],
    pub floor_ece: f64,
    /// The fit pool: [`build_pool_cases`]'s output plus its two
    /// deployed-engine evals (raw + fitted).
    pub pool_cases: &'a [SuiteCase],
    pub pool_raw: &'a Eval,
    pub pool_fitted: &'a Eval,
    pub n_premise_excluded: usize,
    pub cal_capacity: usize,
    pub cal_min_obs: usize,
}

// ── record types (serde — results.json) ─────────────────────────────────

/// One metric surface: `acc` is the PICKS' accuracy; every confidence
/// column is the named surface read over those SAME picks (R6: sharpness
/// beside ECE, everywhere, engine included).
#[derive(Debug, Clone, Serialize)]
pub struct M1Surface {
    pub acc: f64,
    pub ece: f64,
    pub nll: f64,
    pub brier: f64,
    pub resolution: f64,
    pub uncertainty: f64,
    pub auroc: f64,
}

/// A paired-bootstrap interval (R4): the replicate statistic's mean plus
/// its 2.5/97.5 percentiles.
#[derive(Debug, Clone, Serialize)]
pub struct BootstrapCi {
    pub mean: f64,
    pub lo95: f64,
    pub hi95: f64,
}

/// McNemar b/c counts (engine right+blend wrong / engine wrong+blend
/// right) + the two-sided exact binomial p.
#[derive(Debug, Clone, Serialize)]
pub struct McnemarRow {
    pub b: usize,
    pub c: usize,
    pub p_two_sided: f64,
}

/// A one-parameter secondary posture (M2): reported, never promoted.
#[derive(Debug, Clone, Serialize)]
pub struct SecondaryScalar {
    pub selected: f64,
    pub acc: f64,
    pub n_overrides: usize,
}

/// The two-parameter M3 gated switch: reported, never promoted.
#[derive(Debug, Clone, Serialize)]
pub struct SecondaryGate {
    pub tau: f64,
    pub margin: f64,
    pub acc: f64,
    pub n_switched: usize,
}

/// The pool-side fit diagnostics + the pre-registered sanity gates
/// (S1–S5). `void == true` marks the run's adjudication VOID (the
/// plan's recorded cost of an untested instrument).
#[derive(Debug, Clone, Serialize)]
pub struct PoolDiagnostics {
    pub n_pool: usize,
    pub n_premise_excluded: usize,
    pub oof_pairs: usize,
    pub beta: [f64; 4],
    pub oof_gate_nll: f64,
    pub oof_base_rate_nll: f64,
    pub oof_a0_acc: f64,
    pub oof_blend_acc: f64,
    pub ladder: Vec<(f64, f64)>,
    pub full_head_pool_acc: f64,
    pub oof_head_pool_acc: f64,
    pub sanity: [bool; 5],
    pub void: bool,
    pub void_reason: Option<&'static str>,
}

/// One validation item's full record (R5).
#[derive(Debug, Clone, Serialize)]
pub struct M1ItemLog {
    pub idx: usize,
    pub gold: usize,
    pub eng_pick: usize,
    pub eng_conf_cal: f64,
    pub head_pick: usize,
    pub margin: f64,
    pub agree: u8,
    pub blend_pick: usize,
    pub blend_conf_raw: f64,
    pub m1_conf: f64,
    pub platt2_conf: f64,
    pub m2_pick: usize,
    pub m3_pick: usize,
    pub eng_ok: bool,
    pub head_ok: bool,
    pub blend_ok: bool,
    pub m2_ok: bool,
    pub m3_ok: bool,
}

/// The four pre-registered promotion legs (plan §promotion criterion).
#[derive(Debug, Clone, Serialize)]
pub struct PromotionLegs {
    pub floor_leg: bool,
    pub own_surface_leg: bool,
    pub acc_leg: bool,
    pub bootstrap_leg: bool,
}

/// The whole record.
#[derive(Debug, Clone, Serialize)]
pub struct M1Result {
    pub n_items: usize,
    pub lambda_star: f64,
    pub pool: PoolDiagnostics,
    /// A0: the engine's forced accuracy; ECE/AUROC/etc. over its shipped
    /// CALIBRATED readout surface.
    pub a0: M1Surface,
    pub head_alone_acc: f64,
    pub oracle: OracleRow,
    /// The blend at λ*: its own max-normalized raw surface (the 068/069
    /// surface) — the accuracy leg's carrier AND its own uncalibrated
    /// comparison surface.
    pub blend: M1Surface,
    /// PRIMARY (M1): blend picks + the 3-feature logistic confidence.
    pub m1: M1Surface,
    /// SECONDARY: same picks + the agree/disagree piecewise Platt
    /// (`CalibratedGateSet<2>`, the issue's substrate note).
    pub platt2: M1Surface,
    /// The binding G1 floor leg (engine-derived conformal floor ECE).
    pub floor_engine: f64,
    /// Disclosed secondary floor: conformal fit on the pool's OOF blend
    /// pairs, read at the validation blend confidences (the review's
    /// hole-4 "the blend's own floor").
    pub floor_blend: f64,
    pub bootstrap_acc: BootstrapCi,
    pub bootstrap_ece: BootstrapCi,
    pub mcnemar: McnemarRow,
    pub m2: SecondaryScalar,
    pub m3: SecondaryGate,
    pub legs: PromotionLegs,
    pub promotable_candidate: bool,
    pub items: Vec<M1ItemLog>,
}

// ── pool construction ───────────────────────────────────────────────────

/// Assemble the fit pool: cal front (premise-guarded) + train-rest rows
/// minus the corpus-cap docs and every premise-shared row. Returns the
/// pool cases (cal-front order first, then rest order — deterministic)
/// and the number of premise-shared rows excluded.
///
/// The corpus-cap law mirrors the engine build exactly: for each label,
/// the FIRST `cap_per_label` valid train docs in rest order (the
/// `.filter(label).take(cap)` walk `specs_from_pool` performs).
pub fn build_pool_cases(
    cal_cases: &[SuiteCase],
    pool_rows: &Value,
    cap_per_label: usize,
) -> Result<(Vec<SuiteCase>, usize), String> {
    let rows: Vec<&Value> = pool_rows
        .get("rows")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|r| r.get("row")).collect())
        .ok_or_else(|| "nli-m1: pool envelope lacks a rows array".to_string())?;

    // Walk rest in order, tag the first cap valid docs per label as
    // corpus, keep the rest; collect the corpus premises as we go.
    let mut per_label: HashMap<String, usize> = HashMap::new();
    let mut corpus_premises: HashSet<String> = HashSet::new();
    let mut rest_keep: Vec<&Value> = Vec::new();
    for row in rows {
        let Some(label) = train_row_label(SUITE_NAME, row) else {
            continue; // never a corpus doc, never a pool item
        };
        let count = per_label.entry(label).or_insert(0);
        if *count < cap_per_label {
            *count += 1;
            if let Some(p) = row.get("premise").and_then(Value::as_str) {
                corpus_premises.insert(p.to_string());
            }
            continue;
        }
        rest_keep.push(row);
    }

    // The premise-sharing guard, applied to BOTH sources (the cal front
    // carries the same SuiteCase shape).
    let in_corpus = |p: Option<&str>| p.is_some_and(|p| corpus_premises.contains(p));
    let case_shared = |c: &SuiteCase| in_corpus(c.state.get("premise").and_then(Value::as_str));
    let n_premise_excluded = cal_cases.iter().filter(|c| case_shared(c)).count()
        + rest_keep
            .iter()
            .filter(|r| in_corpus(r.get("premise").and_then(Value::as_str)))
            .count();

    let mut arr: Vec<Value> = Vec::with_capacity(rest_keep.len());
    for row in rest_keep {
        if !in_corpus(row.get("premise").and_then(Value::as_str)) {
            arr.push(serde_json::json!({ "row": row }));
        }
    }
    let rest_suite = build_xnli_en(&serde_json::json!({ "rows": arr }), 0);

    let mut cases: Vec<SuiteCase> = cal_cases.iter().filter(|c| !case_shared(c)).cloned().collect();
    cases.extend(rest_suite.cases);
    Ok((cases, n_premise_excluded))
}

// ── the pass ────────────────────────────────────────────────────────────

/// Run the M1 lane. `Ok(None)` = loud skip (wrong suite / unshaped
/// cases — R1's structural guard); `Ok(Some(record))` = the read.
#[allow(clippy::too_many_lines)]
pub fn nli_m1_pass(inp: &M1Inputs) -> Result<Option<M1Result>, String> {
    if inp.spec_name != SUITE_NAME {
        eprintln!(
            "  [nli-m1] suite {}: NOT the validation confirmation surface — SKIPPED \
             (issue 047 R1: the lane reads {SUITE_NAME} only; the 300-item test split \
             is never read for this question)",
            inp.spec_name
        );
        return Ok(None);
    }
    let suite = inp.suite;
    if suite.cases.is_empty() || !suite.cases.iter().all(well_shaped) {
        let shaped = suite.cases.iter().filter(|c| well_shaped(c)).count();
        eprintln!(
            "  [nli-m1] {}: {}/{} eval cases are premise/hypothesis-shaped — SKIPPED",
            inp.spec_name,
            shaped,
            suite.cases.len()
        );
        return Ok(None);
    }

    // ── pool features (the head pool is the FULL pool; the gate pairs are
    // the premise-guarded subset `build_pool_cases` already selected) ──
    let n_pool = inp.pool_cases.len();
    let mut pool_x: Vec<[f64; FEAT_DIM]> = Vec::with_capacity(n_pool);
    let mut pool_y: Vec<usize> = Vec::with_capacity(n_pool);
    for case in inp.pool_cases {
        let (p, h) = pair_state(case).expect("well_shaped checked upstream");
        pool_x.push(pair_features(&p, &h));
        pool_y.push(case.gold[0].idx);
    }

    // ── R3: the K-fold cross-fit — OOF head quantities for every item ──
    let folds = fold_assign(&pool_y, K_FOLDS);
    let mut oof_delta = vec![[0.0_f64; 3]; n_pool];
    let mut oof_posterior = vec![[0.0_f64; 3]; n_pool];
    let mut oof_pick = vec![0_usize; n_pool];
    let mut oof_margin = vec![0.0_f64; n_pool];
    let mut fold_ok = true;
    for k in 0..K_FOLDS {
        let mut counts = [0_usize; 3];
        let mut fit_x = Vec::with_capacity(n_pool);
        let mut fit_y = Vec::with_capacity(n_pool);
        for (i, &y) in pool_y.iter().enumerate() {
            if folds[i] == k {
                counts[y] += 1;
            } else {
                fit_x.push(pool_x[i]);
                fit_y.push(y);
            }
        }
        if counts.contains(&0) || counts.iter().any(|&c| c < 3) {
            fold_ok = false;
        }
        let Some(head_k) = NliLda::fit(&fit_x, &fit_y) else {
            fold_ok = false;
            continue;
        };
        for i in 0..n_pool {
            if folds[i] != k {
                continue;
            }
            let d = head_k.delta(&pool_x[i]);
            oof_delta[i] = d;
            oof_posterior[i] = head_k.posterior(&d);
            oof_pick[i] = head_k.pick(&d);
            oof_margin[i] = head_k.margin(&d);
        }
    }

    // ── pool engine reads ──
    let oof_eng_pick: Vec<usize> = (0..n_pool)
        .map(|i| argmax(&inp.pool_raw.probs[i][0]))
        .collect();
    let oof_c1: Vec<f64> = (0..n_pool).map(|i| inp.pool_fitted.confs[i][0]).collect();

    // ── selection ladders (all OOF, all pool-side) ──
    let mut ladder: Vec<(f64, f64)> = Vec::with_capacity(LAMBDAS.len());
    let mut blend_oof_by_lambda: Vec<Vec<usize>> = Vec::with_capacity(LAMBDAS.len());
    for &lam in &LAMBDAS {
        let mut correct = 0usize;
        let mut picks = Vec::with_capacity(n_pool);
        for i in 0..n_pool {
            let (pick, _) = blend_pick_conf(&inp.pool_raw.probs[i][0], &oof_posterior[i], lam);
            picks.push(pick);
            correct += usize::from(pick == pool_y[i]);
        }
        blend_oof_by_lambda.push(picks);
        ladder.push((lam, correct as f64 / n_pool as f64));
    }
    let oof_a0_acc = oof_eng_pick
        .iter()
        .zip(pool_y.iter())
        .filter(|(p, y)| p == y)
        .count() as f64
        / n_pool as f64;
    // Selection: max OOF accuracy, ties → smaller λ (the ascending ladder
    // keeps the first max).
    let li_star = {
        let mut best = 0usize;
        for (i, (_, acc)) in ladder.iter().enumerate() {
            if *acc > ladder[best].1 {
                best = i;
            }
        }
        best
    };
    let lambda_star = LAMBDAS[li_star];
    let oof_blend_acc = ladder[li_star].1;

    // M2 (secondary): score_k = ln(pe_k) + λ2·(δh_k − mean δh), OOF; ties
    // → smaller λ2.
    let mut m2_best = (LAMBDAS[1], 0usize, 0usize);
    for &lam2 in &LAMBDAS[1..] {
        let mut correct = 0usize;
        let mut overrides = 0usize;
        for i in 0..n_pool {
            let pick = m2_pick(&inp.pool_raw.probs[i][0], &oof_delta[i], lam2);
            correct += usize::from(pick == pool_y[i]);
            overrides += usize::from(pick != oof_eng_pick[i]);
        }
        if correct > m2_best.1 {
            m2_best = (lam2, correct, overrides);
        }
    }
    let m2_lambda = m2_best.0;

    // M3 (secondary): gated switch, OOF; ties → larger τ then larger m
    // (the conservative side — fewer switches).
    let mut m3_best: Option<(f64, f64, usize, usize)> = None;
    for &tau in &M3_TAUS {
        for &m in &M3_MARGINS {
            let mut correct = 0usize;
            let mut switched = 0usize;
            for i in 0..n_pool {
                let pick = if oof_c1[i] < tau && oof_margin[i] > m {
                    switched += 1;
                    oof_pick[i]
                } else {
                    oof_eng_pick[i]
                };
                correct += usize::from(pick == pool_y[i]);
            }
            if m3_best.is_none_or(|(_, _, c, _)| correct > c) {
                m3_best = Some((tau, m, correct, switched));
            }
        }
    }
    let m3_sel = m3_best.expect("non-empty grid");

    // ── the gate fits (OOF pairs, premise-guarded — the pool the caller
    // built IS the guard) ──
    let mut pair_x: Vec<[f64; 4]> = Vec::with_capacity(n_pool);
    let mut pair_y: Vec<bool> = Vec::with_capacity(n_pool);
    for i in 0..n_pool {
        let pick = blend_oof_by_lambda[li_star][i];
        let agree = pick == oof_eng_pick[i];
        pair_x.push(gate_features(oof_c1[i], oof_margin[i], agree));
        pair_y.push(pick == pool_y[i]);
    }
    let n_oof_pairs = pair_x.len();
    let oof_base_rate = pair_y.iter().filter(|y| **y).count() as f64
        / n_oof_pairs.max(1) as f64;
    let oof_base_rate_nll = binary_nll_at_rate(oof_base_rate);
    let Some(beta) = logistic_fit(&pair_x, &pair_y) else {
        return Err(
            "nli-m1: the logistic fit failed (singular Hessian) — instrument defect, \
             no read adjudicated"
                .to_string(),
        );
    };
    let oof_gate_pairs: Vec<(f64, bool)> = pair_x
        .iter()
        .zip(pair_y.iter())
        .map(|(x, &y)| (sigmoid(dot4(beta, x)), y))
        .collect();
    let oof_gate_nll = nll_of(&oof_gate_pairs);

    // The piecewise-Platt secondary (agree / disagree directions), the
    // lane's own calibrator window config, fit on the same OOF pairs.
    let mut platt2 = CalibratedGateSet::<2>::new(inp.cal_capacity, inp.cal_min_obs);
    for (i, &y) in pair_y.iter().enumerate() {
        let agree = pair_x[i][3] > 0.5;
        platt2.observe(usize::from(agree), oof_c1[i] as f32, y);
    }
    platt2.refit_all();

    // ── the deployed head (full-pool fit) + its pool accuracy (S5) ──
    let Some(full_head) = NliLda::fit(&pool_x, &pool_y) else {
        return Err("nli-m1: the full-pool head fit lacks a class — instrument defect".to_string());
    };
    let full_head_pool_acc = pool_x
        .iter()
        .zip(pool_y.iter())
        .filter(|&(x, y)| {
            let d = full_head.delta(x);
            full_head.pick(&d) == *y
        })
        .count() as f64
        / n_pool as f64;
    let oof_head_pool_acc = oof_pick
        .iter()
        .zip(pool_y.iter())
        .filter(|(p, y)| p == y)
        .count() as f64
        / n_pool as f64;

    // ── the pre-registered sanity gates (plan §pool-side sanity) ──
    let beta_norm = (beta[1] * beta[1] + beta[2] * beta[2] + beta[3] * beta[3]).sqrt();
    let s1 = n_oof_pairs >= 1000 && beta_norm > 1e-6;
    let s2 = oof_gate_nll < oof_base_rate_nll - 0.02;
    let s3 = oof_blend_acc >= oof_a0_acc;
    let s4 = fold_ok;
    let s5 = full_head_pool_acc >= oof_head_pool_acc - 0.02;
    let sanity = [s1, s2, s3, s4, s5];
    let void_reason = [
        "S1: the logistic never moved or too few OOF pairs",
        "S2: the gate learned nothing (NLL at the base rate)",
        "S3: the blend selection regressed OOF accuracy",
        "S4: a fold lacks class support",
        "S5: the full-fit head undercuts its OOF estimate",
    ]
    .into_iter()
    .zip(sanity)
    .find(|(_, ok)| !ok)
    .map(|(why, _)| why);
    let is_void = void_reason.is_some();

    // ══ THE ONE VALIDATION READ ══
    let n = suite.cases.len();
    let mut items: Vec<M1ItemLog> = Vec::with_capacity(n);
    let mut a0_correct = 0usize;
    let mut head_correct = 0usize;
    let (mut both_right, mut both_wrong, mut hwins, mut hloss) = (0usize, 0usize, 0usize, 0usize);
    for (ci, case) in suite.cases.iter().enumerate() {
        let (p, h) = pair_state(case).expect("well_shaped checked");
        let d = full_head.delta(&pair_features(&p, &h));
        let head_pick = full_head.pick(&d);
        let margin = full_head.margin(&d);
        let ph = full_head.posterior(&d);
        let pe = &inp.raw_eval.probs[ci][0];
        let eng_pick = argmax(pe);
        let gold = case.gold[0].idx;
        let c1 = inp.cal_eval_test.confs[ci][0];
        let agree = eng_pick == head_pick;
        let (blend_pick, blend_conf) = blend_pick_conf(pe, &ph, lambda_star);
        let m1_conf = sigmoid(dot4(beta, &gate_features(c1, margin, agree)));
        let platt2_conf = f64::from(platt2.apply(usize::from(agree), c1 as f32));
        let m2_pick = m2_pick(pe, &d, m2_lambda);
        let m3_pick = if c1 < m3_sel.0 && margin > m3_sel.1 {
            head_pick
        } else {
            eng_pick
        };
        let (eng_ok, head_ok) = (eng_pick == gold, head_pick == gold);
        match (eng_ok, head_ok) {
            (true, true) => both_right += 1,
            (false, false) => both_wrong += 1,
            (false, true) => hwins += 1,
            (true, false) => hloss += 1,
        }
        a0_correct += usize::from(eng_ok);
        head_correct += usize::from(head_ok);
        items.push(M1ItemLog {
            idx: ci,
            gold,
            eng_pick,
            eng_conf_cal: c1,
            head_pick,
            margin,
            agree: u8::from(agree),
            blend_pick,
            blend_conf_raw: blend_conf,
            m1_conf,
            platt2_conf,
            m2_pick,
            m3_pick,
            eng_ok,
            head_ok,
            blend_ok: blend_pick == gold,
            m2_ok: m2_pick == gold,
            m3_ok: m3_pick == gold,
        });
    }
    let f = |c: usize| c as f64 / n.max(1) as f64;
    let a0_acc = f(a0_correct);
    let blend_acc = f(items.iter().filter(|it| it.blend_ok).count());

    // Surfaces (R6: every surface carries its sharpness columns).
    let surface = |acc: f64, pairs: &[(f64, bool)]| {
        let (brier, resolution, uncertainty) = brier_resolution_of(pairs);
        M1Surface {
            acc,
            ece: ece_of(pairs),
            nll: nll_of(pairs),
            brier,
            resolution,
            uncertainty,
            auroc: auroc_of(pairs),
        }
    };
    let blend_pairs: Vec<(f64, bool)> =
        items.iter().map(|it| (it.blend_conf_raw, it.blend_ok)).collect();
    let m1_pairs: Vec<(f64, bool)> = items.iter().map(|it| (it.m1_conf, it.blend_ok)).collect();
    let platt2_pairs: Vec<(f64, bool)> =
        items.iter().map(|it| (it.platt2_conf, it.blend_ok)).collect();
    let engine_pairs: Vec<(f64, bool)> =
        items.iter().map(|it| (it.eng_conf_cal, it.eng_ok)).collect();

    // Floors: the binding engine-derived one (the runner's own bytes) +
    // the blend-surface-derived one (pool OOF blend pairs → validation).
    let floor_oof_pairs: Vec<CalibrationPair> = (0..n_pool)
        .map(|i| {
            let (_, conf) = blend_pick_conf(&inp.pool_raw.probs[i][0], &oof_posterior[i], lambda_star);
            CalibrationPair {
                conf,
                correct: blend_oof_by_lambda[li_star][i] == pool_y[i],
            }
        })
        .collect();
    let val_blend_confs: Vec<f64> = items.iter().map(|it| it.blend_conf_raw).collect();
    let floored_blend = conformal_naive_floor(&floor_oof_pairs, &val_blend_confs);
    let floor_blend = ece_of(
        &floored_blend
            .into_iter()
            .zip(items.iter().map(|it| it.blend_ok))
            .collect::<Vec<_>>(),
    );

    // Bootstrap (R4): paired replicates; ONE index set per replicate feeds
    // both the accuracy delta and the ECE delta.
    let mut rng = SplitMix64::new(BOOTSTRAP_SEED);
    let mut acc_stats = Vec::with_capacity(BOOTSTRAP_B);
    let mut ece_stats = Vec::with_capacity(BOOTSTRAP_B);
    for _ in 0..BOOTSTRAP_B {
        let mut blend_sum = 0usize;
        let mut eng_sum = 0usize;
        let mut m1_b: Vec<(f64, bool)> = Vec::with_capacity(n);
        let mut floor_b: Vec<(f64, bool)> = Vec::with_capacity(n);
        for _ in 0..n {
            let i = rng.below(n);
            blend_sum += usize::from(items[i].blend_ok);
            eng_sum += usize::from(items[i].eng_ok);
            m1_b.push((items[i].m1_conf, items[i].blend_ok));
            floor_b.push(inp.floor_pairs[i]);
        }
        acc_stats.push(blend_sum as f64 / n as f64 - eng_sum as f64 / n as f64);
        ece_stats.push(ece_of(&m1_b) - ece_of(&floor_b));
    }
    let bootstrap_acc = ci_of(&acc_stats);
    let bootstrap_ece = ci_of(&ece_stats);

    // McNemar (engine vs blend), exact two-sided.
    let b_count = items.iter().filter(|it| it.eng_ok && !it.blend_ok).count();
    let c_count = items.iter().filter(|it| !it.eng_ok && it.blend_ok).count();
    let mcnemar = McnemarRow {
        b: b_count,
        c: c_count,
        p_two_sided: mcnemar_exact(b_count, c_count),
    };

    // Secondaries (validation read, REPORTED WITHOUT promotion).
    let m2_correct = items.iter().filter(|it| it.m2_ok).count();
    let m2_overrides = items.iter().filter(|it| it.m2_pick != it.eng_pick).count();
    let m3_correct = items.iter().filter(|it| it.m3_ok).count();
    let m3_switched = items.iter().filter(|it| it.m3_pick != it.eng_pick).count();

    // The pre-registered promotion legs (plan §promotion criterion).
    let m1_surf = surface(blend_acc, &m1_pairs);
    let legs = PromotionLegs {
        floor_leg: m1_surf.ece <= inp.floor_ece,
        own_surface_leg: m1_surf.ece < ece_of(&blend_pairs),
        acc_leg: blend_acc >= a0_acc + ACC_BAR,
        bootstrap_leg: bootstrap_acc.lo95 > 0.0,
    };
    let promotable_candidate =
        !is_void && legs.floor_leg && legs.own_surface_leg && legs.acc_leg && legs.bootstrap_leg;

    eprintln!(
        "  [nli-m1] {SUITE_NAME}: n={n} λ*={lambda_star} pool={n_pool} \
         A0={a0_acc:.4} blend={blend_acc:.4} head={ha:.4} | \
         m1_ece={me:.4} blend_ece={be:.4} floor={fe:.4} | \
         boot_acc=[{lo:.4},{hi:.4}] mcnemar b={b} c={c} | \
         legs: floor={floor_leg} own={own_leg} acc≥{ACC_BAR}={acc_leg} boot={boot_leg} | {verdict}",
        ha = f(head_correct),
        me = m1_surf.ece,
        be = ece_of(&blend_pairs),
        fe = inp.floor_ece,
        lo = bootstrap_acc.lo95,
        hi = bootstrap_acc.hi95,
        b = b_count,
        c = c_count,
        floor_leg = legs.floor_leg,
        own_leg = legs.own_surface_leg,
        acc_leg = legs.acc_leg,
        boot_leg = legs.bootstrap_leg,
        verdict = if is_void {
            format!("VOID ({})", void_reason.unwrap_or_default())
        } else if promotable_candidate {
            "PROMOTABLE-CANDIDATE".to_string()
        } else {
            "not promotable".to_string()
        },
    );

    Ok(Some(M1Result {
        n_items: n,
        lambda_star,
        pool: PoolDiagnostics {
            n_pool,
            n_premise_excluded: inp.n_premise_excluded,
            oof_pairs: n_oof_pairs,
            beta,
            oof_gate_nll,
            oof_base_rate_nll,
            oof_a0_acc,
            oof_blend_acc,
            ladder,
            full_head_pool_acc,
            oof_head_pool_acc,
            sanity,
            void: is_void,
            void_reason,
        },
        a0: surface(a0_acc, &engine_pairs),
        head_alone_acc: f(head_correct),
        oracle: OracleRow {
            both_right,
            both_wrong,
            head_unique_wins: hwins,
            head_unique_losses: hloss,
        },
        blend: surface(blend_acc, &blend_pairs),
        m1: m1_surf,
        platt2: surface(blend_acc, &platt2_pairs),
        floor_engine: inp.floor_ece,
        floor_blend,
        bootstrap_acc,
        bootstrap_ece,
        mcnemar,
        m2: SecondaryScalar {
            selected: m2_lambda,
            acc: f(m2_correct),
            n_overrides: m2_overrides,
        },
        m3: SecondaryGate {
            tau: m3_sel.0,
            margin: m3_sel.1,
            acc: f(m3_correct),
            n_switched: m3_switched,
        },
        legs,
        promotable_candidate,
        items,
    }))
}

// ── statistical helpers ─────────────────────────────────────────────────

/// First-max-wins argmax (the engine's own tie convention).
fn argmax(probs: &[f64]) -> usize {
    let mut best = 0usize;
    for (i, &p) in probs.iter().enumerate() {
        if p > probs[best] {
            best = i;
        }
    }
    best
}

/// The M2 log-odds pick: `score_k = ln(pe_k) + λ·(δh_k − mean δh)` — the
/// mean-centering is the 3-class generalization of the issue's
/// `σ(δ_e + λ·δ_h)` (the LDA deltas are already log-likelihood ratios).
fn m2_pick(pe: &[f64], d: &[f64; 3], lambda: f64) -> usize {
    let mean = (d[0] + d[1] + d[2]) / 3.0;
    let mut best = 0usize;
    let mut best_s = f64::NEG_INFINITY;
    for (k, &pk) in pe.iter().enumerate() {
        let s = pk.clamp(1e-12, 1.0).ln() + lambda * (d[k] - mean);
        if s > best_s {
            best_s = s;
            best = k;
        }
    }
    best
}

/// The gate features `[1, logit(c), margin, agree]` — the intercept first.
fn gate_features(c: f64, margin: f64, agree: bool) -> [f64; 4] {
    let c = c.clamp(CONF_EPS, 1.0 - CONF_EPS);
    [1.0, (c / (1.0 - c)).ln(), margin, f64::from(agree)]
}

#[inline]
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[inline]
fn dot4(a: [f64; 4], x: &[f64; 4]) -> f64 {
    a[0] * x[0] + a[1] * x[1] + a[2] * x[2] + a[3] * x[3]
}

/// The NLL of a constant predictor at rate `r` (the base-rate reference).
fn binary_nll_at_rate(r: f64) -> f64 {
    let r = r.clamp(1e-12, 1.0 - 1e-12);
    -(r * r.ln() + (1.0 - r) * (1.0 - r).ln())
}

/// Fit the 4-parameter logistic by IRLS/Newton on the NLL objective —
/// f64, deterministic, ridge `RIDGE` on the three weights (never the
/// intercept), max 100 steps, converged when the step's ∞-norm < 1e-10.
/// `None` on a singular normal matrix (the caller treats that as an
/// instrument defect, never as a fit).
fn logistic_fit(xs: &[[f64; 4]], ys: &[bool]) -> Option<[f64; 4]> {
    let n = xs.len();
    if n == 0 || ys.len() != n {
        return None;
    }
    let mut beta = [0.0_f64; 4];
    let base = ys.iter().filter(|y| **y).count() as f64 / n as f64;
    beta[0] = ((base.clamp(1e-6, 1.0 - 1e-6)) / (1.0 - base.clamp(1e-6, 1.0 - 1e-6))).ln();
    for _ in 0..100 {
        let mut g = [0.0_f64; 4];
        let mut h = [[0.0_f64; 4]; 4];
        for (x, &y) in xs.iter().zip(ys.iter()) {
            let p = sigmoid(dot4(beta, x)).clamp(1e-9, 1.0 - 1e-9);
            let w = p * (1.0 - p);
            let r = f64::from(y) - p;
            for (j, &xj) in x.iter().enumerate() {
                g[j] += xj * r;
                for (k, &xk) in x.iter().enumerate() {
                    h[j][k] += xj * xk * w;
                }
            }
        }
        for j in 1..4 {
            h[j][j] += RIDGE;
            g[j] -= RIDGE * beta[j];
        }
        let step = solve4(&h, &g)?;
        for (b, &s) in beta.iter_mut().zip(step.iter()) {
            *b += s;
        }
        if step.iter().fold(0.0_f64, |m, v| m.max(v.abs())) < 1e-10 {
            break;
        }
    }
    Some(beta)
}

/// Gaussian elimination with partial pivoting on a 4×4 system.
fn solve4(a: &[[f64; 4]; 4], b: &[f64; 4]) -> Option<[f64; 4]> {
    let mut m = *a;
    let mut rhs = *b;
    for col in 0..4 {
        let pivot = (col..4).fold(col, |best, r| if m[r][col].abs() > m[best][col].abs() { r } else { best });
        if m[pivot][col].abs() < 1e-14 {
            return None;
        }
        m.swap(pivot, col);
        rhs.swap(pivot, col);
        for r in (col + 1)..4 {
            let factor = m[r][col] / m[col][col];
            if factor == 0.0 {
                continue;
            }
            let pivot_row = m[col];
            for (k, mrc) in m[r][col..4].iter_mut().enumerate() {
                *mrc -= factor * pivot_row[col + k];
            }
            rhs[r] -= factor * rhs[col];
        }
    }
    let mut x = [0.0_f64; 4];
    for r in (0..4).rev() {
        let mut s = rhs[r];
        for k in (r + 1)..4 {
            s -= m[r][k] * x[k];
        }
        x[r] = s / m[r][r];
    }
    Some(x)
}

/// Label round-robin fold assignment: buckets by label in
/// first-appearance order, one item per bucket per round, fold =
/// pick_index % K. Balanced classes per fold, deterministic, no RNG.
fn fold_assign(labels: &[usize], k: usize) -> Vec<usize> {
    let mut label_order: Vec<usize> = Vec::new();
    let mut queues: HashMap<usize, std::collections::VecDeque<usize>> = HashMap::new();
    for (i, &l) in labels.iter().enumerate() {
        if !queues.contains_key(&l) {
            label_order.push(l);
        }
        queues.entry(l).or_default().push_back(i);
    }
    let mut folds = vec![usize::MAX; labels.len()];
    let mut pick = 0usize;
    loop {
        let mut advanced = false;
        for &l in &label_order {
            if let Some(i) = queues.get_mut(&l).and_then(std::collections::VecDeque::pop_front) {
                folds[i] = pick % k;
                pick += 1;
                advanced = true;
            }
        }
        if !advanced {
            break;
        }
    }
    folds
}

/// The 95% bootstrap interval: replicate mean + 2.5/97.5 nearest-rank
/// percentiles (n = B = 2000, so both tails carry 50 replicates of
/// support — no degenerate max-index percentile here).
fn ci_of(stats: &[f64]) -> BootstrapCi {
    let n = stats.len();
    if n == 0 {
        return BootstrapCi { mean: f64::NAN, lo95: f64::NAN, hi95: f64::NAN };
    }
    let mut sorted = stats.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo = sorted[((n as f64 * 0.025) as usize).min(n - 1)];
    let hi = sorted[((n as f64 * 0.975) as usize).min(n - 1)];
    BootstrapCi { mean: stats.iter().sum::<f64>() / n as f64, lo95: lo, hi95: hi }
}

/// McNemar's exact two-sided binomial p on discordant counts `(b, c)`:
/// `min(1, 2·Σ_{k≤min(b,c)} C(n,k)/2^n)` with `n = b + c`, computed in
/// log space. `n == 0` → 1.0 (no discordance, no evidence).
fn mcnemar_exact(b: usize, c: usize) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let min = b.min(c);
    let mut terms: Vec<f64> = Vec::with_capacity(min + 1);
    let mut log_choose = 0.0_f64; // log C(n,0) = 0
    for k in 0..=min {
        if k > 0 {
            log_choose += ((n - k + 1) as f64).ln() - (k as f64).ln();
        }
        terms.push(log_choose - (n as f64) * 2.0_f64.ln());
    }
    let max_t = terms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let sum = max_t + terms.iter().map(|t| (t - max_t).exp()).sum::<f64>().ln();
    (2.0 * sum.exp()).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{GoldAnswer, SuiteQuestion, QKind};

    fn nli_case(id: &str, premise: &str, hypothesis: &str, gold: usize) -> SuiteCase {
        SuiteCase {
            id: id.into(),
            state: serde_json::json!({"premise": premise, "hypothesis": hypothesis}),
            questions: vec![SuiteQuestion {
                qid: "relation".into(),
                kind: QKind::Choice,
                instructions: "rel?".into(),
                criteria: serde_json::json!
                    ({"entailment": "e", "neutral": "n", "contradiction": "c"}),
            }],
            gold: vec![GoldAnswer { idx: gold, soft: vec![0.0; 3], gold_score: None }],
        }
    }

    /// A hand-rolled Eval: `probs[i]` is the single question's 3-vector;
    /// picks are its argmax; confs carry the readout surface under test.
    fn eval_from(probs: Vec<Vec<f64>>, confs: Vec<f64>) -> Eval {
        let picks: Vec<Vec<usize>> = probs.iter().map(|q| vec![argmax(q)]).collect();
        Eval {
            probs: probs.into_iter().map(|q| vec![q]).collect(),
            picks,
            confs: confs.into_iter().map(|c| vec![c]).collect(),
            abstained: Vec::new(),
            causes: Vec::new(),
        }
    }

    /// One deterministic engine draw per case: the latent correctness is
    /// drawn ONCE and both surfaces derive from it — the RAW probs/pick
    /// and the CALIBRATED confidence (jittered independently). This is
    /// the real harness's relationship (raw_eval and cal_eval_test are
    /// the SAME engine on the same cases), and it is what makes `c_eng`
    /// informative about raw-pick correctness.
    struct EngineDraw {
        raw: Eval,
        fitted_confs: Vec<f64>,
    }

    fn engine_draw(cases: &[SuiteCase], rate: f64, rng: &mut SplitMix64) -> EngineDraw {
        let threshold = (rate * 100.0) as usize;
        let mut probs = Vec::with_capacity(cases.len());
        let mut raw_confs = Vec::with_capacity(cases.len());
        let mut fitted_confs = Vec::with_capacity(cases.len());
        for case in cases {
            let gold = case.gold[0].idx;
            let right = rng.below(100) < threshold;
            let pick = if right { gold } else { (gold + 1) % 3 };
            let mut p = [0.05_f64; 3];
            p[pick] = 0.7;
            probs.push(p.to_vec());
            raw_confs.push(if right {
                0.7 + 0.2 * rng.below(100) as f64 / 100.0
            } else {
                0.4 + 0.1 * rng.below(100) as f64 / 100.0
            });
            fitted_confs.push(if right {
                0.7 + 0.2 * rng.below(100) as f64 / 100.0
            } else {
                0.4 + 0.1 * rng.below(100) as f64 / 100.0
            });
        }
        EngineDraw { raw: eval_from(probs, raw_confs), fitted_confs }
    }

    fn synthetic_suite(name: &'static str, n: usize) -> Suite {
        let words = ["alpha", "beta", "gamma"];
        Suite {
            name,
            cases: (0..n)
                .map(|i| {
                    let w = words[i % 3];
                    nli_case(
                        "s",
                        &format!("{w} common premise {i}"),
                        &format!("{w} hypothesis {i}"),
                        i % 3,
                    )
                })
                .collect(),
            option_counts_note: "test",
        }
    }

    // ── logistic ──

    #[test]
    fn logistic_fits_a_separable_rule_and_predicts_it() {
        // y == agree, with c/margin pure noise: the fit must recover a
        // large w3 and predict high on agree / low on disagree.
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for i in 0..400 {
            let agree = i % 2 == 0;
            let c = 0.3 + 0.4 * ((i * 7) % 10) as f64 / 10.0;
            let margin = ((i * 13) % 10) as f64 / 5.0;
            xs.push(gate_features(c, margin, agree));
            ys.push(agree);
        }
        let beta = logistic_fit(&xs, &ys).expect("fit");
        let hi = sigmoid(dot4(beta, &gate_features(0.5, 1.0, true)));
        let lo = sigmoid(dot4(beta, &gate_features(0.5, 1.0, false)));
        assert!(hi > 0.9, "agree predicts correct: {hi}");
        assert!(lo < 0.1, "disagree predicts wrong: {lo}");
    }

    #[test]
    fn logistic_fit_on_pure_noise_stays_bounded_and_finite() {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for i in 0..500 {
            let agree = i % 3 == 0;
            let c = 0.2 + 0.6 * ((i * 11) % 17) as f64 / 17.0;
            xs.push(gate_features(c, c, agree));
            ys.push((i * 5 + i / 3) % 2 == 0);
        }
        let beta = logistic_fit(&xs, &ys).expect("fit");
        assert!(beta.iter().all(|b| b.is_finite()));
        assert!(beta.iter().all(|b| b.abs() < 50.0));
    }

    // ── fold assignment ──

    #[test]
    fn fold_assignment_is_balanced_deterministic_and_complete() {
        let labels: Vec<usize> = (0..300).map(|i| i % 3).collect();
        let folds = fold_assign(&labels, 10);
        assert_eq!(folds.len(), 300);
        assert!(folds.iter().all(|f| *f < 10));
        let mut per_fold = vec![[0_usize; 3]; 10];
        for (i, &f) in folds.iter().enumerate() {
            per_fold[f][labels[i]] += 1;
        }
        for fold in &per_fold {
            for count in fold {
                assert_eq!(*count, 10, "every fold holds exactly 10 per class");
            }
        }
        assert_eq!(fold_assign(&labels, 10), folds, "deterministic");
    }

    // ── bootstrap + McNemar ──

    #[test]
    fn bootstrap_ci_separates_a_real_gap_from_a_tie_and_is_deterministic() {
        // A replicate statistic with a real positive gap → CI above 0.
        let stats: Vec<f64> =
            (0..2000).map(|i| 0.02 + 0.01 * ((i % 7) as f64 / 7.0)).collect();
        let ci = ci_of(&stats);
        assert!(ci.lo95 > 0.0);
        assert!(ci.hi95 > ci.lo95);
        let ci2 = ci_of(&stats);
        assert_eq!((ci.mean, ci.lo95, ci.hi95), (ci2.mean, ci2.lo95, ci2.hi95));
        // A centered statistic straddles 0.
        let centered: Vec<f64> = (0..2000).map(|i| (i % 2) as f64 - 0.5).collect();
        let c2 = ci_of(&centered);
        assert!(c2.lo95 < 0.0 && c2.hi95 > 0.0);
    }

    #[test]
    fn mcnemar_known_answers() {
        // No discordance → p = 1.
        assert!((mcnemar_exact(0, 0) - 1.0).abs() < 1e-12);
        // Perfectly balanced discordance → p = 1.
        assert!((mcnemar_exact(10, 10) - 1.0).abs() < 1e-9);
        // 10 vs 1 → significant but not overwhelming (exact ≈ 0.0215).
        let p = mcnemar_exact(10, 1);
        assert!(p < 0.05, "10/1 discordance is significant: {p}");
        assert!(p > 0.01, "...but not overwhelming: {p}");
        // Symmetric in its arguments; never exceeds 1.
        assert!((mcnemar_exact(3, 17) - mcnemar_exact(17, 3)).abs() < 1e-12);
        assert!(mcnemar_exact(1, 1) <= 1.0);
    }

    // ── pool construction ──

    #[test]
    fn pool_builder_excludes_corpus_docs_and_shared_premises() {
        // 3 labels; cap 1 per label. Each label's first row becomes a
        // corpus doc; a row sharing that premise is excluded; a distinct
        // row is kept. The cal-front case with a corpus-shared premise is
        // dropped too.
        let row = |p: &str, h: &str, label: usize| {
            serde_json::json!({ "row": {"premise": p, "hypothesis": h, "label": label} })
        };
        let pool = serde_json::json!({ "rows": [
            row("PA", "h1", 0), // corpus doc (label 0)
            row("PA", "h2", 0), // premise-shared → excluded
            row("PB", "h3", 0), // kept
            row("PC", "h4", 1), // corpus doc (label 1)
            row("PD", "h5", 1), // kept
            row("PE", "h6", 2), // corpus doc (label 2)
            row("PF", "h7", 2), // kept
        ]});
        let cal = vec![
            nli_case("c0", "PA", "cal-shared", 0), // premise-shared → dropped
            nli_case("c1", "PZ", "cal-kept", 1),
        ];
        let (cases, excluded) = build_pool_cases(&cal, &pool, 1).expect("pool");
        // Kept: cal c1 first, then the rest side (PB, PD, PF). Excluded:
        // the premise-shared rest row (PA/h2) + the cal-front shared case.
        assert_eq!(cases.len(), 4);
        assert_eq!(excluded, 2);
        assert_eq!(cases[0].id, "c1");
        assert_eq!(cases[1].id, "xnli_en:0");
    }

    #[test]
    fn pool_builder_refuses_an_envelope_without_rows() {
        let bad = serde_json::json!({ "rows": 3 });
        assert!(build_pool_cases(&[], &bad, 4).is_err());
    }

    // ── end to end (synthetic) ──

    /// Run the full pass over synthetic slices: `n_pool` fit items,
    /// `n_val` validation items, the engine right at `rate` with an
    /// informative readout confidence (0.8 right / 0.45 wrong), a
    /// feature-useless head (the states share their discriminative word
    /// across classes, so the LDA is chance-level and λ* stays at 0).
    /// Everything is drawn from one seed → byte-identical on re-run.
    fn run_synthetic(n_pool: usize, n_val: usize, rate: f64, seed: u64) -> M1Result {
        let pool_suite = synthetic_suite(SUITE_NAME, n_pool);
        let val_suite = synthetic_suite(SUITE_NAME, n_val);
        let cal_cases: Vec<SuiteCase> = (0..60)
            .map(|i| {
                let w = ["alpha", "beta", "gamma"][i % 3];
                nli_case("c", &format!("{w} common premise {i}"), &format!("{w} hyp {i}"), i % 3)
            })
            .collect();
        let mut rng = SplitMix64::new(seed);
        let val_draw = engine_draw(&val_suite.cases, rate, &mut rng);
        let raw_eval = val_draw.raw;
        let cal_eval_test = eval_from(
            (0..n_val).map(|i| raw_eval.probs[i][0].clone()).collect(),
            val_draw.fitted_confs,
        );
        let pool_draw = engine_draw(&pool_suite.cases, rate, &mut rng);
        let pool_raw = pool_draw.raw;
        let pool_fitted = eval_from(
            (0..n_pool).map(|i| pool_raw.probs[i][0].clone()).collect(),
            pool_draw.fitted_confs,
        );
        let cal_draw = engine_draw(&cal_cases, rate, &mut rng);
        let cal_pairs: Vec<CalibrationPair> = cal_cases
            .iter()
            .enumerate()
            .map(|(i, c)| CalibrationPair {
                conf: cal_draw.raw.confs[i][0],
                correct: cal_draw.raw.picks[i][0] == c.gold[0].idx,
            })
            .collect();
        let raw_confs: Vec<f64> = (0..n_val).map(|i| raw_eval.confs[i][0]).collect();
        let floored = conformal_naive_floor(&cal_pairs, &raw_confs);
        let floor_pairs: Vec<(f64, bool)> = floored
            .into_iter()
            .zip((0..n_val).map(|i| raw_eval.picks[i][0] == val_suite.cases[i].gold[0].idx))
            .collect();
        let floor_ece = ece_of(&floor_pairs);
        let inp = M1Inputs {
            spec_name: SUITE_NAME,
            suite: &val_suite,
            raw_eval: &raw_eval,
            cal_eval_test: &cal_eval_test,
            floor_pairs: &floor_pairs,
            floor_ece,
            pool_cases: &pool_suite.cases,
            pool_raw: &pool_raw,
            pool_fitted: &pool_fitted,
            n_premise_excluded: 0,
            cal_capacity: 256,
            cal_min_obs: 32,
        };
        nli_m1_pass(&inp).expect("pass runs").expect("right suite")
    }

    #[test]
    fn e2e_synthetic_large_pool_is_not_void_and_is_deterministic() {
        let r1 = run_synthetic(1200, 200, 0.6, 7);
        assert_eq!(r1.n_items, 200);
        assert_eq!(r1.items.len(), 200);
        assert!(LAMBDAS.contains(&r1.lambda_star));
        assert!(!r1.pool.void, "expected green sanity gates: {:?}", r1.pool.void_reason);
        assert_eq!(r1.pool.oof_pairs, 1200);
        // The informative confidence makes the gate learn (S2's premise).
        assert!(r1.pool.oof_gate_nll < r1.pool.oof_base_rate_nll - 0.02);
        // λ* stays at 0 (the head is feature-useless here), so the blend
        // ties A0 and the accuracy leg cannot pass — the honest posture.
        assert_eq!(r1.lambda_star, 0.0);
        assert_eq!(r1.blend.acc, r1.a0.acc);
        assert!(!r1.legs.acc_leg);
        assert!(!r1.promotable_candidate);
        // The per-item log is complete and self-consistent.
        let b = r1.items.iter().filter(|it| it.eng_ok && !it.blend_ok).count();
        let c = r1.items.iter().filter(|it| !it.eng_ok && it.blend_ok).count();
        assert_eq!((r1.mcnemar.b, r1.mcnemar.c), (b, c));
        // Determinism: the same seed re-runs byte-identically.
        let r2 = run_synthetic(1200, 200, 0.6, 7);
        let (s1, s2) = (serde_json::to_string(&r1).unwrap(), serde_json::to_string(&r2).unwrap());
        assert_eq!(s1, s2);
    }

    #[test]
    fn e2e_synthetic_small_pool_marks_the_run_void_at_s1() {
        // 40 pool items → < 1000 OOF pairs → S1 fails → VOID, loudly.
        let r = run_synthetic(40, 50, 0.6, 11);
        assert!(r.pool.void);
        assert_eq!(r.pool.void_reason, Some("S1: the logistic never moved or too few OOF pairs"));
        assert!(!r.promotable_candidate);
    }

    #[test]
    fn the_lane_refuses_any_suite_but_the_validation_surface() {
        let pool_suite = synthetic_suite(SUITE_NAME, 40);
        let wrong = synthetic_suite("xnli_en", 20);
        let mut rng = SplitMix64::new(3);
        let raw_eval = engine_draw(&wrong.cases, 0.6, &mut rng).raw;
        let cal_eval_test = engine_draw(&wrong.cases, 0.6, &mut rng).raw;
        let pool_raw = engine_draw(&pool_suite.cases, 0.6, &mut rng).raw;
        let pool_fitted = engine_draw(&pool_suite.cases, 0.6, &mut rng).raw;
        let inp = M1Inputs {
            spec_name: "xnli_en",
            suite: &wrong,
            raw_eval: &raw_eval,
            cal_eval_test: &cal_eval_test,
            floor_pairs: &[],
            floor_ece: 0.1,
            pool_cases: &pool_suite.cases,
            pool_raw: &pool_raw,
            pool_fitted: &pool_fitted,
            n_premise_excluded: 0,
            cal_capacity: 256,
            cal_min_obs: 32,
        };
        let got = nli_m1_pass(&inp).expect("no error");
        assert!(got.is_none(), "R1: a non-validation suite must loud-skip");
    }
}
