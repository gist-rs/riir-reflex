//! Issue 044 T3 — the modelless NLI pair-feature head (`--nli-feature-ab`,
//! report-only). XNLI is pair reasoning, which a corpus-bound lane is
//! structurally weak at (the issue's own words); before declaring the gap
//! out of reach, this instrument measures the cheapest modelless pair
//! mechanism: closed-form diagonal LDA over lexical pair features
//! (overlap / negation / length / numbers / negation-prefix / antonym
//! cross) extracted from the state's `premise`/`hypothesis` strings.
//!
//! Modelless law: no training, no gradient descent — the fit is closed-form
//! moments over the CAL slice only (the [`crate::harness::pair_heads`]
//! diagonal-LDA math generalized to the 3 NLI classes). The test split is
//! read ONCE, under three postures, all reported in one record:
//!
//! 1. `head_alone` — the feature head's own argmax (its standalone power).
//! 2. `blend_additive` — pick argmax(`p_engine` + λ·`p_head`), λ
//!    cal-selected from a ladder (ties → smaller λ, staying closer to the
//!    engine).
//! 3. `override_gate` — the head overrides the engine's pick only when its
//!    own class margin exceeds τ, τ cal-selected (ties → larger τ, the
//!    conservative side).
//!
//! An UNARMED posture (λ = 0 / τ above every margin) reproduces the
//! engine's picks exactly — the blast radius is the firing subset,
//! disclosed per posture. The engine probabilities are the lane's own
//! forced `raw_eval.probs` (the same vector `hard` argmaxes).
//!
//! **The G1-constrained posture** (Bench 069 protocol — the pre-registered
//! reopen path Bench 068 recorded for the failed λ=0.5 promotion):
//! selection is CAL-SIDE ONLY. The cal slice is split into interleaved
//! halves (even indices fit, odd indices held out — interleaving keeps a
//! label-grouped cal slice balanced on both sides); ONE engine-derived
//! conformal floor is read on the held-out half (the test-side G1
//! construction, miniaturized); and for each λ on a ladder extended down
//! to 0 the blend's RECALIBRATED readout (the lane's own
//! `SigmoidGateCalibrator` family, fit on the fit half) must beat BOTH of
//! the lane's G1 bars on the held-out half — its own uncalibrated surface
//! AND the floor — while not losing accuracy against the unarmed λ=0 rung.
//! λ* = max cal accuracy among feasible rungs (ties → the smaller λ); no
//! feasible rung → λ*=0 + `constraint_unsatisfiable` (the refusal is a
//! finding, never a silent fallback to a worse posture). The ONE test read
//! then reports both G1 triples at λ* (raw blend + recalibrated blend)
//! against the lane's TEST-side floor. What is NOT claimed: the
//! interleaved-half bar is an estimate; the verdict is the test-side
//! triple, and promotion remains gated on it.
//!
//!
//! The head posterior is a SIGMOID readout, never softmax (the house
//! sigmoid law): `p_k = σ(δ_k − mean_j δ_j)`, normalized to sum 1.
//!
//! Scope: the arm runs only where EVERY eval and cal case is well-shaped
//! (state object with string `premise` + `hypothesis`, exactly one Choice
//! question with exactly 3 options). Anything else is a LOUD skip
//! (`Ok(None)` + a stderr line) — never a silent absence, never a partial
//! population wearing the suite's name.

use serde::Serialize;

use katgpt_core::sigmoid_calibration::SigmoidGateCalibrator;

use super::{Eval, QKind, Suite, SuiteCase};
use crate::harness::metrics::{CalibrationPair, conformal_naive_floor, ece_of};

/// The G1-constrained λ ladder — the unconstrained ladder extended DOWN so
/// the calibration constraint has somewhere to stand (ties → the smaller
/// λ); rung 0 is the unarmed engine, the guaranteed no-op fallback.
const G1_LAMBDAS: [f64; 8] = [0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0];

/// Below this cal size the interleaved halves are degenerate — the
/// constrained posture loud-skips.
const MIN_G1_CAL: usize = 16;

/// Feature dimensionality (see [`FEAT_NAMES`]).
pub const FEAT_DIM: usize = 10;

/// The feature names, in vector order (the record's `fit` block names the
/// per-class means with these).
pub const FEAT_NAMES: [&str; FEAT_DIM] = [
    "jaccard",
    "contain_h_in_p",
    "len_ratio",
    "hyp_neg",
    "prem_neg",
    "neg_diff",
    "num_shared",
    "num_hyp_only",
    "negprefix_cross",
    "antonym_cross",
];

/// Negation vocabulary (lowercase exact tokens, counted and capped at 3).
const NEGATIONS: [&str; 11] = [
    "not", "no", "never", "nothing", "none", "nobody", "nowhere", "neither", "nor", "cannot",
    "without",
];

/// Closed antonym table (unordered pairs; a cross match is (a∈P ∧ b∈H) or
/// (b∈P ∧ a∈H)). Deliberately small and single-token — a modelless lookup,
/// not a lexicon resource.
const ANTONYMS: [(&str, &str); 24] = [
    ("win", "lose"),
    ("open", "close"),
    ("open", "shut"),
    ("arrive", "depart"),
    ("arrive", "leave"),
    ("enter", "exit"),
    ("buy", "sell"),
    ("give", "take"),
    ("love", "hate"),
    ("increase", "decrease"),
    ("rise", "fall"),
    ("gain", "lose"),
    ("accept", "reject"),
    ("allow", "forbid"),
    ("appear", "disappear"),
    ("stay", "leave"),
    ("start", "stop"),
    ("begin", "finish"),
    ("remember", "forget"),
    ("hold", "release"),
    ("push", "pull"),
    ("fill", "empty"),
    ("expand", "shrink"),
    ("succeed", "fail"),
];

/// Lowercase word tokens; `n't` is rewritten to ` not` first so contracted
/// negations tokenize into the negation vocabulary.
fn tokenize(text: &str, out: &mut Vec<String>) {
    out.clear();
    let lowered = text.to_lowercase().replace("n't", " not");
    let mut cur = String::new();
    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
}

fn is_num_token(t: &str) -> bool {
    !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit())
}

/// The closed-form pair features for one (premise, hypothesis) case.
/// Deterministic; token buffers are the only allocations.
#[must_use]
pub fn pair_features(premise: &str, hypothesis: &str) -> [f64; FEAT_DIM] {
    let mut pt = Vec::with_capacity(64);
    let mut ht = Vec::with_capacity(64);
    tokenize(premise, &mut pt);
    tokenize(hypothesis, &mut ht);

    let pset: std::collections::HashSet<&str> = pt.iter().map(String::as_str).collect();
    let hset: std::collections::HashSet<&str> = ht.iter().map(String::as_str).collect();
    let inter = pset.intersection(&hset).count();
    let union = pset.union(&hset).count();

    let cap3 = |n: usize| n.min(3) as f64;
    let hyp_neg = ht
        .iter()
        .filter(|t| NEGATIONS.contains(&t.as_str()))
        .count();
    let prem_neg = pt
        .iter()
        .filter(|t| NEGATIONS.contains(&t.as_str()))
        .count();

    let nums_p: std::collections::HashSet<String> =
        pt.iter().filter(|t| is_num_token(t)).cloned().collect();
    let nums_h: std::collections::HashSet<String> =
        ht.iter().filter(|t| is_num_token(t)).cloned().collect();

    // Negation-prefix cross: one side's token is the other side's token
    // under an un-/dis-/non- prefix (len ≥ 3 base). Capped at 3.
    let negprefix = |a: &str, b: &str| -> bool {
        for pre in ["un", "dis", "non"] {
            if a.len() >= 3 && b.strip_prefix(pre) == Some(a) {
                return true;
            }
            if b.len() >= 3 && a.strip_prefix(pre) == Some(b) {
                return true;
            }
        }
        false
    };
    let mut cross = 0usize;
    'outer: for a in &pset {
        for b in &hset {
            if negprefix(a, b) {
                cross += 1;
                if cross >= 3 {
                    break 'outer;
                }
            }
        }
    }

    let mut antonym = 0usize;
    'outer2: for (w1, w2) in ANTONYMS {
        let hit =
            (pset.contains(w1) && hset.contains(w2)) || (pset.contains(w2) && hset.contains(w1));
        if hit {
            antonym += 1;
            if antonym >= 3 {
                break 'outer2;
            }
        }
    }

    [
        inter as f64 / union.max(1) as f64,
        inter as f64 / hset.len().max(1) as f64,
        ht.len() as f64 / pt.len().max(1) as f64,
        cap3(hyp_neg),
        cap3(prem_neg),
        cap3(hyp_neg.abs_diff(prem_neg)),
        nums_p.intersection(&nums_h).count() as f64,
        nums_h.difference(&nums_p).count() as f64,
        cross as f64,
        antonym as f64,
    ]
}

/// The premise/hypothesis strings of a well-shaped case, else `None`.
fn pair_state(case: &SuiteCase) -> Option<(String, String)> {
    let obj = case.state.as_object()?;
    let premise = obj.get("premise")?.as_str()?.to_string();
    let hypothesis = obj.get("hypothesis")?.as_str()?.to_string();
    Some((premise, hypothesis))
}

/// A case is well-shaped when it carries a premise/hypothesis state and
/// exactly one Choice question with exactly 3 options (the NLI shape).
fn well_shaped(case: &SuiteCase) -> bool {
    if case.questions.len() != 1 || case.gold.len() != 1 {
        return false;
    }
    let q = &case.questions[0];
    if q.kind != QKind::Choice || case.gold[0].idx >= 3 {
        return false;
    }
    q.criteria.as_object().is_some_and(|m| m.len() == 3) && pair_state(case).is_some()
}

/// K-class diagonal LDA over the pair features (K = 3 NLI classes).
/// Closed-form moments from the CAL slice: per-class means, a shared
/// diagonal variance with relative shrinkage (the
/// [`crate::harness::pair_heads`] law — a near-constant dim carries no
/// between-class signal and would explode the direction), and an absolute
/// floor for the all-constant case.
#[derive(Debug, Clone)]
pub struct NliLda {
    /// Per-class means (K × D).
    mu: Vec<[f64; FEAT_DIM]>,
    /// Inverse pooled diagonal variance (D).
    inv_var: [f64; FEAT_DIM],
    /// Per-class intercept −|μ_k|²/(2σ²) folded (K).
    t: Vec<f64>,
}

const VAR_SHRINK: f64 = 0.01;
const MIN_VAR_FLOOR: f64 = 1e-12;

impl NliLda {
    /// Fit from cal (features, class). `None` when any class is absent —
    /// the cal slice is label-stratified, so this is a structural state the
    /// caller surfaces as a loud skip.
    pub fn fit(xs: &[[f64; FEAT_DIM]], ys: &[usize]) -> Option<Self> {
        const K: usize = 3;
        let mut counts = [0usize; K];
        for &y in ys {
            counts.get(y).copied()?;
            counts[y] += 1;
        }
        if counts.contains(&0) {
            return None;
        }
        let mut mu = vec![[0.0; FEAT_DIM]; K];
        for (x, &y) in xs.iter().zip(ys.iter()) {
            for (m, v) in mu[y].iter_mut().zip(x.iter()) {
                *m += v;
            }
        }
        for (kk, m) in mu.iter_mut().enumerate() {
            for v in m.iter_mut() {
                *v /= counts[kk] as f64;
            }
        }
        // Shared diagonal variance around the class means.
        let mut var = [0.0; FEAT_DIM];
        for (x, &y) in xs.iter().zip(ys.iter()) {
            for (d, v) in x.iter().enumerate() {
                let diff = v - mu[y][d];
                var[d] += diff * diff;
            }
        }
        for v in var.iter_mut() {
            *v /= xs.len() as f64;
        }
        let mean_var = var.iter().sum::<f64>() / FEAT_DIM as f64;
        for v in var.iter_mut() {
            *v += VAR_SHRINK * mean_var;
            *v = v.max(MIN_VAR_FLOOR);
        }
        let mut inv_var = [0.0; FEAT_DIM];
        let mut t = vec![0.0; K];
        for (kk, m) in mu.iter().enumerate() {
            let mut norm = 0.0;
            for (d, v) in m.iter().enumerate() {
                inv_var[d] = 1.0 / var[d];
                norm += v * v * inv_var[d];
            }
            t[kk] = -0.5 * norm;
        }
        Some(Self { mu, inv_var, t })
    }

    /// Per-class discriminants δ_k(x) = μ_k·(x/σ²) − |μ_k|²/(2σ²).
    #[must_use]
    pub fn delta(&self, x: &[f64; FEAT_DIM]) -> [f64; 3] {
        let mut out = [0.0; 3];
        for (kk, m) in self.mu.iter().enumerate() {
            let mut dot = 0.0;
            for (d, v) in m.iter().enumerate() {
                dot += v * x[d] * self.inv_var[d];
            }
            out[kk] = dot + self.t[kk];
        }
        out
    }

    /// First-max-wins argmax (the engine's own tie convention).
    #[must_use]
    pub fn pick(&self, d: &[f64; 3]) -> usize {
        let mut best = 0usize;
        for (i, &v) in d.iter().enumerate() {
            if v > d[best] {
                best = i;
            }
        }
        best
    }

    /// Margin of the top class over the runner-up.
    #[must_use]
    pub fn margin(&self, d: &[f64; 3]) -> f64 {
        let (mut first, mut second) = (0usize, 0usize);
        for (i, &v) in d.iter().enumerate() {
            if v > d[first] {
                second = first;
                first = i;
            } else if i != first && v > d[second] {
                second = i;
            }
        }
        d[first] - d[second]
    }

    /// Sigmoid-ratio posterior: `p_k = σ(δ_k − mean_δ)`, normalized to sum
    /// 1 (the house sigmoid law — never softmax).
    #[must_use]
    pub fn posterior(&self, d: &[f64; 3]) -> [f64; 3] {
        let mean = d.iter().sum::<f64>() / 3.0;
        let mut p = [0.0; 3];
        for (pk, dk) in p.iter_mut().zip(d.iter()) {
            *pk = 1.0 / (1.0 + (mean - *dk).exp());
        }
        let s = p.iter().sum::<f64>().max(1e-12);
        for pk in p.iter_mut() {
            *pk /= s;
        }
        p
    }
}

/// One blend posture's test-side reading.
#[derive(Debug, Clone, Serialize)]
pub struct BlendRow {
    /// The cal-selected parameter (λ for additive, τ for the override gate).
    pub selected: f64,
    pub acc: f64,
    /// Questions where the posture's pick differs from the engine's.
    pub n_overrides: usize,
    /// Fraction of questions where the posture agrees with the engine.
    pub engine_agree: f64,
    /// The cal-side ladder read: (param, cal accuracy), ascending param.
    pub ladder: Vec<LadderPoint>,
}

/// One ladder rung.
#[derive(Debug, Clone, Serialize)]
pub struct LadderPoint {
    pub param: f64,
    pub cal_acc: f64,
}

/// The fit's own disclosure (means + inverse variance — enough to re-derive
/// the head; the instrument's state IS the record).
#[derive(Debug, Clone, Serialize)]
pub struct NliFitInfo {
    pub n_cal: usize,
    pub feature_names: [&'static str; FEAT_DIM],
    /// The 3 class names, in class order (= the criteria key order).
    pub class_labels: Vec<String>,
    /// Per-class feature means, in class order.
    pub class_means: Vec<[f64; FEAT_DIM]>,
    pub inv_var: [f64; FEAT_DIM],
    /// The head's own accuracy on the cal slice.
    pub cal_head_acc: f64,
}

/// The oracle diagnostic: the head's unique wins/losses against the engine
/// on the test split — where an ideal mixture could gain and what it would
/// pay.
#[derive(Debug, Clone, Serialize)]
pub struct OracleRow {
    pub both_right: usize,
    pub both_wrong: usize,
    pub head_unique_wins: usize,
    pub head_unique_losses: usize,
}

/// The full record (results.json; the lane's own row is unchanged).
#[derive(Debug, Clone, Serialize)]
pub struct NliFeatureAb {
    pub n: usize,
    /// The engine's forced accuracy over the counted questions (the lane's
    /// own `hard.accuracy` — repeated here so the record is self-contained).
    pub baseline_acc: f64,
    pub head_alone_acc: f64,
    /// [gold][head pick] counts, class order = criteria order.
    pub head_alone_confusion: Vec<[usize; 3]>,
    pub fit: NliFitInfo,
    pub blend_additive: BlendRow,
    pub override_gate: BlendRow,
    pub oracle: OracleRow,
    /// G1 evidence for the blend posture: ECE of the blended readout
    /// `q ∝ p_engine + λ·p_head` (max-prob confidence, the same 15-bin
    /// `ece_of` the lane's own readout ECE uses), beside the engine's
    /// calibrated readout ECE and the suite's conformal-naive floor — the
    /// three numbers a promotion decision reads. `None` when the floor is
    /// unavailable (no floor on this row → no G1 claim either way).
    pub blend_readout_g1: Option<BlendG1>,
    /// The G1-constrained blend posture (Bench 069 protocol): λ selected
    /// CAL-SIDE under the pre-registered calibration constraint, one test
    /// read carrying both G1 triples. `None` when the lane's G1 inputs are
    /// absent or the cal slice is below [`MIN_G1_CAL`].
    pub blend_g1_constrained: Option<G1ConstrainedRow>,
}

/// The blend's G1 triple.
#[derive(Debug, Clone, Serialize)]
pub struct BlendG1 {
    pub blend_ece: f64,
    pub engine_calibrated_ece: f64,
    pub conformal_floor: f64,
    /// blend ECE ≤ floor — the UQ law's own bar.
    pub pass: bool,
}

/// G1-side inputs for the constrained blend posture (Bench 069 protocol):
/// the lane's TEST-side G1 numbers (the record's verdict triples read
/// against them) + the lane's calibrator window config (the recalibration
/// leg reuses the SAME calibrator family, never a private twin).
#[derive(Debug, Clone, Copy)]
pub struct G1Inputs {
    /// The engine's calibrated readout ECE on the TEST split (the lane's
    /// own `readout_ece_cal`).
    pub engine_calibrated_test_ece: f64,
    /// The suite's conformal-naive floor on the TEST split.
    pub conformal_floor_test: f64,
    /// Calibrator evidence-window capacity (the lane's `EngineConfig`).
    pub cal_capacity: usize,
    /// Calibrator occupancy floor (the lane's `EngineConfig`).
    pub cal_min_obs: usize,
}

/// One G1-constrained ladder rung. `cal_acc` is over the FULL cal slice
/// (the existing ladder's convention); both ECEs are on the HELD-OUT cal
/// half (the interleaved split — the fit half never scores itself).
#[derive(Debug, Clone, Serialize)]
pub struct G1LadderPoint {
    pub param: f64,
    pub cal_acc: f64,
    /// The blend's own (uncalibrated) readout ECE on the held-out half.
    pub raw_b_ece: f64,
    /// The recalibrated blend readout ECE on the held-out half.
    pub recal_b_ece: f64,
    /// The pre-registered feasibility predicate (module doc).
    pub feasible: bool,
}

/// The selection bar's own disclosure (the two engine-side quantities the
/// constraint was hung on, at the held-out half).
#[derive(Debug, Clone, Serialize)]
pub struct G1ConstraintBar {
    /// The engine-derived conformal floor read on the held-out half
    /// (fit on the fit half) — the binding selection bar.
    pub conformal_floor_b: f64,
    /// The engine's own RAW readout ECE on the held-out half —
    /// disclosure (the law's engine-side "own outputs" leg).
    pub engine_raw_b_ece: f64,
    pub n_fit_half: usize,
    pub n_eval_half: usize,
}

/// The G1-constrained blend posture's full record (Bench 069 protocol).
#[derive(Debug, Clone, Serialize)]
pub struct G1ConstrainedRow {
    pub selected: f64,
    /// True when NO rung was feasible — λ fell back to 0 (the engine
    /// alone) and the posture records the refusal instead of a blend.
    pub constraint_unsatisfiable: bool,
    pub bar: G1ConstraintBar,
    pub ladder: Vec<G1LadderPoint>,
    pub acc: f64,
    pub n_overrides: usize,
    pub engine_agree: f64,
    /// The raw blend readout's G1 triple at λ* on the TEST split.
    pub g1_raw: BlendG1,
    /// The recalibrated blend readout's G1 triple at λ* (the same
    /// calibrator family fit on the FULL cal slice's blend pairs) on the
    /// TEST split.
    pub g1_recalibrated: BlendG1,
}

/// The A/B pass. `Ok(None)` = loud skip (the suite is not premise/
/// hypothesis-shaped); the caller prints nothing further — the pass has
/// already announced itself on stderr.
pub fn nli_feature_ab_pass(
    raw_eval: &Eval,
    suite: &Suite,
    cal_eval: &Eval,
    cal_cases: &[SuiteCase],
    // The lane's TEST-side G1 numbers + calibrator config — `None` = no
    // floor (no G1 claim, and no constrained posture, either way).
    g1_inputs: Option<G1Inputs>,
) -> Result<Option<NliFeatureAb>, String> {
    let shaped_msg = |shaped: usize, total: usize, slice: &str| {
        format!(
            "  [nli-feature-ab] {suite_name} {slice}: {shaped}/{total} cases are \
             premise/hypothesis-shaped — SKIPPED (the arm requires the full population, \
             never a partial one)",
            suite_name = suite.name,
        )
    };
    if suite.cases.is_empty() || !suite.cases.iter().all(well_shaped) {
        let shaped = suite.cases.iter().filter(|c| well_shaped(c)).count();
        eprintln!("{}", shaped_msg(shaped, suite.cases.len(), "eval"));
        return Ok(None);
    }
    if cal_cases.is_empty() || !cal_cases.iter().all(well_shaped) {
        let shaped = cal_cases.iter().filter(|c| well_shaped(c)).count();
        eprintln!("{}", shaped_msg(shaped, cal_cases.len(), "cal"));
        return Ok(None);
    }

    // The class names, in class order (= the criteria key order).
    let class_labels: Vec<String> = suite.cases[0].questions[0]
        .criteria
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    if class_labels.len() != 3 {
        eprintln!(
            "  [nli-feature-ab] suite {}: criteria carry {} keys, expected 3 — SKIPPED",
            suite.name,
            class_labels.len()
        );
        return Ok(None);
    }

    // ── Fit on CAL only ──
    let mut cal_x: Vec<[f64; FEAT_DIM]> = Vec::with_capacity(cal_cases.len());
    let mut cal_y: Vec<usize> = Vec::with_capacity(cal_cases.len());
    for case in cal_cases {
        let (p, h) = pair_state(case).expect("well_shaped checked");
        cal_x.push(pair_features(&p, &h));
        cal_y.push(case.gold[0].idx);
    }
    let Some(lda) = NliLda::fit(&cal_x, &cal_y) else {
        eprintln!(
            "  [nli-feature-ab] suite {}: cal slice lacks a class — SKIPPED",
            suite.name
        );
        return Ok(None);
    };
    let cal_head_acc = cal_x
        .iter()
        .zip(cal_y.iter())
        .filter(|(x, y)| lda.pick(&lda.delta(x)) == **y)
        .count() as f64
        / cal_x.len() as f64;

    // Cal-side blend ladders (the engine term = the same forced probs).
    const LAMBDAS: [f64; 5] = [0.5, 1.0, 2.0, 4.0, 8.0];
    const TAUS: [f64; 6] = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0];
    let mut cal_correct_add = vec![0usize; LAMBDAS.len()];
    let mut cal_correct_ovr = vec![0usize; TAUS.len()];
    for (ci, case) in cal_cases.iter().enumerate() {
        let (p, h) = pair_state(case).expect("well_shaped checked");
        let d = lda.delta(&pair_features(&p, &h));
        let ph = lda.posterior(&d);
        let pe = &cal_eval.probs[ci][0];
        let gold = case.gold[0].idx;
        for (li, &lam) in LAMBDAS.iter().enumerate() {
            let mut best = 0usize;
            let mut best_s = f64::NEG_INFINITY;
            for (k, &pk) in ph.iter().enumerate() {
                let s = pe[k] + lam * pk;
                if s > best_s {
                    best_s = s;
                    best = k;
                }
            }
            cal_correct_add[li] += usize::from(best == gold);
        }
        let margin = lda.margin(&d);
        for (ti, &tau) in TAUS.iter().enumerate() {
            let pick = if margin > tau {
                lda.pick(&d)
            } else {
                argmax(pe)
            };
            cal_correct_ovr[ti] += usize::from(pick == gold);
        }
    }
    let add_ladder: Vec<LadderPoint> = LAMBDAS
        .iter()
        .zip(cal_correct_add)
        .map(|(&param, c)| LadderPoint {
            param,
            cal_acc: c as f64 / cal_cases.len() as f64,
        })
        .collect();
    let ovr_ladder: Vec<LadderPoint> = TAUS
        .iter()
        .zip(cal_correct_ovr)
        .map(|(&param, c)| LadderPoint {
            param,
            cal_acc: c as f64 / cal_cases.len() as f64,
        })
        .collect();
    // Selection: additive ties → SMALLER λ (closer to the engine — the
    // ascending ladder keeps the FIRST max); override ties → LARGER τ (the
    // conservative side — the ascending ladder keeps the LAST max).
    let sel_add = {
        let mut best = 0usize;
        for (i, r) in add_ladder.iter().enumerate() {
            if r.cal_acc > add_ladder[best].cal_acc {
                best = i;
            }
        }
        add_ladder[best].param
    };
    let sel_tau = {
        let mut best = 0usize;
        for (i, r) in ovr_ladder.iter().enumerate() {
            if r.cal_acc >= ovr_ladder[best].cal_acc {
                best = i;
            }
        }
        ovr_ladder[best].param
    };

    // ── The G1-constrained blend posture (Bench 069 protocol): λ selected
    // CAL-SIDE under the pre-registered calibration constraint. Every
    // quantity the selection reads lives on the cal slice; the test split
    // is untouched until the single read below.
    let (eng_test_ece, floor_test) = match g1_inputs {
        Some(i) => (i.engine_calibrated_test_ece, i.conformal_floor_test),
        None => (f64::NAN, f64::NAN),
    };
    let had_g1_inputs = g1_inputs.is_some();
    let g1_plan = g1_inputs.and_then(|inp| build_g1_plan(inp, &lda, cal_cases, cal_eval));

    // ── The ONE test read ──
    let n = suite.cases.len();
    let mut baseline_correct = 0usize;
    let mut head_correct = 0usize;
    let mut head_confusion = vec![[0usize; 3]; 3];
    let (mut add_correct, mut add_overrides, mut add_agree) = (0usize, 0usize, 0usize);
    let (mut ovr_correct, mut ovr_overrides, mut ovr_agree) = (0usize, 0usize, 0usize);
    let (mut both_right, mut both_wrong, mut hwins, mut hloss) = (0usize, 0usize, 0usize, 0usize);
    let mut blend_pairs: Vec<(f64, bool)> = Vec::with_capacity(n);
    let (mut g1_correct, mut g1_overrides, mut g1_agree) = (0usize, 0usize, 0usize);
    let mut g1_raw_pairs: Vec<(f64, bool)> = Vec::with_capacity(n);
    let mut g1_recal_pairs: Vec<(f64, bool)> = Vec::with_capacity(n);
    for (ci, case) in suite.cases.iter().enumerate() {
        let (p, h) = pair_state(case).expect("well_shaped checked");
        let d = lda.delta(&pair_features(&p, &h));
        let head_pick = lda.pick(&d);
        let ph = lda.posterior(&d);
        let pe = &raw_eval.probs[ci][0];
        let eng_pick = argmax(pe);
        let gold = case.gold[0].idx;

        let eng_ok = eng_pick == gold;
        let head_ok = head_pick == gold;
        baseline_correct += usize::from(eng_ok);
        head_correct += usize::from(head_ok);
        head_confusion[gold][head_pick] += 1;
        match (eng_ok, head_ok) {
            (true, true) => both_right += 1,
            (false, false) => both_wrong += 1,
            (false, true) => hwins += 1,
            (true, false) => hloss += 1,
        }

        // additive at sel_add
        let (best, blend_conf) = blend_pick_conf(pe, &ph, sel_add);
        add_correct += usize::from(best == gold);
        add_overrides += usize::from(best != eng_pick);
        add_agree += usize::from(best == eng_pick);

        // The blended readout q ∝ pe + λ·ph — the promoted posture's own
        // confidence surface, for the G1 triple.
        if g1_inputs.is_some() {
            blend_pairs.push((blend_conf, best == gold));
        }

        // G1-constrained posture at λ* — same surface, the constrained
        // blend weight, both confidence legs (raw + the test-side
        // recalibrator fit on the FULL cal slice's blend pairs).
        if let Some(plan) = &g1_plan {
            let (pick, conf) = blend_pick_conf(pe, &ph, plan.lambda);
            let ok = pick == gold;
            g1_correct += usize::from(ok);
            g1_overrides += usize::from(pick != eng_pick);
            g1_agree += usize::from(pick == eng_pick);
            g1_raw_pairs.push((conf, ok));
            g1_recal_pairs.push((plan.recal.apply(conf as f32) as f64, ok));
        }

        // override gate at sel_tau
        let pick = if lda.margin(&d) > sel_tau {
            head_pick
        } else {
            eng_pick
        };
        ovr_correct += usize::from(pick == gold);
        ovr_overrides += usize::from(pick != eng_pick);
        ovr_agree += usize::from(pick == eng_pick);
    }
    let f = |c: usize| c as f64 / n.max(1) as f64;

    let blend_ece = ece_of(&blend_pairs);
    let blend_g1_constrained = g1_plan.map(|plan| {
        let recal_ece = ece_of(&g1_recal_pairs);
        G1ConstrainedRow {
            selected: plan.lambda,
            constraint_unsatisfiable: plan.unsatisfiable,
            bar: plan.bar,
            ladder: plan.ladder,
            acc: f(g1_correct),
            n_overrides: g1_overrides,
            engine_agree: f(g1_agree),
            g1_raw: BlendG1 {
                blend_ece,
                engine_calibrated_ece: eng_test_ece,
                conformal_floor: floor_test,
                pass: blend_ece <= floor_test,
            },
            g1_recalibrated: BlendG1 {
                blend_ece: recal_ece,
                engine_calibrated_ece: eng_test_ece,
                conformal_floor: floor_test,
                pass: recal_ece <= floor_test,
            },
        }
    });
    if let Some(row) = &blend_g1_constrained {
        eprintln!(
            "  [nli-feature-ab] {suite_name}: G1-constrained blend λ={lam}{unsat} \
             acc={acc:.4} (engine {base:.4}) raw_ece={re:.4} floor={fl:.4} pass={rp} — \
             recal_ece={ce:.4} pass={cp}",
            suite_name = suite.name,
            lam = row.selected,
            unsat = if row.constraint_unsatisfiable {
                " (UNSATISFIABLE)"
            } else {
                ""
            },
            acc = row.acc,
            base = f(baseline_correct),
            re = row.g1_raw.blend_ece,
            fl = row.g1_raw.conformal_floor,
            rp = row.g1_raw.pass,
            ce = row.g1_recalibrated.blend_ece,
            cp = row.g1_recalibrated.pass,
        );
    }

    Ok(Some(NliFeatureAb {
        n,
        baseline_acc: f(baseline_correct),
        head_alone_acc: f(head_correct),
        head_alone_confusion: head_confusion,
        fit: NliFitInfo {
            n_cal: cal_x.len(),
            feature_names: FEAT_NAMES,
            class_labels,
            class_means: lda.mu.clone(),
            inv_var: lda.inv_var,
            cal_head_acc,
        },
        blend_additive: BlendRow {
            selected: sel_add,
            acc: f(add_correct),
            n_overrides: add_overrides,
            engine_agree: f(add_agree),
            ladder: add_ladder,
        },
        override_gate: BlendRow {
            selected: sel_tau,
            acc: f(ovr_correct),
            n_overrides: ovr_overrides,
            engine_agree: f(ovr_agree),
            ladder: ovr_ladder,
        },
        oracle: OracleRow {
            both_right,
            both_wrong,
            head_unique_wins: hwins,
            head_unique_losses: hloss,
        },
        blend_readout_g1: had_g1_inputs.then_some(BlendG1 {
            blend_ece,
            engine_calibrated_ece: eng_test_ece,
            conformal_floor: floor_test,
            pass: blend_ece <= floor_test,
        }),
        blend_g1_constrained,
    }))
}

fn argmax(probs: &[f64]) -> usize {
    let mut best = 0usize;
    for (i, &p) in probs.iter().enumerate() {
        if p > probs[best] {
            best = i;
        }
    }
    best
}

/// The additive blend's first-max pick + max-normalized confidence
/// (`q ∝ pe + λ·ph`; the same surface the blend G1 triples read). The one
/// blend arithmetic — the cal ladder, the test read and the constrained
/// posture all consume it, so a convention change lands once.
fn blend_pick_conf(pe: &[f64], ph: &[f64; 3], lambda: f64) -> (usize, f64) {
    let mut best = 0usize;
    let mut best_s = f64::NEG_INFINITY;
    for (k, &pk) in ph.iter().enumerate() {
        let s = pe[k] + lambda * pk;
        if s > best_s {
            best_s = s;
            best = k;
        }
    }
    let mut q = [0.0f64; 3];
    let mut s = 0.0;
    for k in 0..3 {
        q[k] = pe[k] + lambda * ph[k];
        s += q[k];
    }
    let s = s.max(1e-12);
    let conf = q.iter().fold(0.0f64, |m, &v| m.max(v / s));
    (best, conf)
}

/// The pre-registered selection: max cal accuracy among FEASIBLE rungs,
/// ties → the smaller λ (the ascending ladder keeps the FIRST max); no
/// feasible rung → rung 0 + unsatisfiable (the engine alone, loudly).
fn select_g1_rung(ladder: &[G1LadderPoint]) -> (usize, bool) {
    let mut best: Option<usize> = None;
    for (i, p) in ladder.iter().enumerate() {
        if !p.feasible {
            continue;
        }
        match best {
            None => best = Some(i),
            Some(b) => {
                if p.cal_acc > ladder[b].cal_acc {
                    best = Some(i);
                }
            }
        }
    }
    match best {
        Some(i) => (i, false),
        None => (0, true),
    }
}

/// The G1-constrained posture's CAL-side selection (Bench 069 protocol —
/// see the module doc). Returns `None` = loud skip (cal slice below the
/// floor or misaligned with its eval); every other state returns a plan.
fn build_g1_plan(
    inp: G1Inputs,
    lda: &NliLda,
    cal_cases: &[SuiteCase],
    cal_eval: &Eval,
) -> Option<G1Plan> {
    let n_cal = cal_cases.len();
    if n_cal < MIN_G1_CAL || cal_eval.confs.len() != n_cal || cal_eval.probs.len() != n_cal {
        eprintln!(
            "  [nli-feature-ab] cal slice n={n_cal} below the G1 posture floor \
             ({MIN_G1_CAL}) or misaligned with its eval — constrained posture SKIPPED"
        );
        return None;
    }

    // Per-λ blend pairs over the FULL cal slice (one question per
    // well-shaped case — the pass's shape guard).
    let mut rung_pairs: Vec<Vec<(f64, bool)>> = vec![Vec::with_capacity(n_cal); G1_LAMBDAS.len()];
    let mut rung_correct = vec![0usize; G1_LAMBDAS.len()];
    for (ci, case) in cal_cases.iter().enumerate() {
        let (p, h) = pair_state(case).expect("well_shaped checked");
        let d = lda.delta(&pair_features(&p, &h));
        let ph = lda.posterior(&d);
        let pe = &cal_eval.probs[ci][0];
        let gold = case.gold[0].idx;
        for (li, &lam) in G1_LAMBDAS.iter().enumerate() {
            let (pick, conf) = blend_pick_conf(pe, &ph, lam);
            let ok = pick == gold;
            rung_correct[li] += usize::from(ok);
            rung_pairs[li].push((conf, ok));
        }
    }

    // Interleaved halves: the fit half (even indices) fits, the held-out
    // half (odd) reads — interleaving keeps a label-grouped cal slice
    // balanced on both sides. The floor is ENGINE-derived (ONE floor per
    // suite — the test-side G1's own construction, miniaturized).
    let fit_idx: Vec<usize> = (0..n_cal).filter(|i| i % 2 == 0).collect();
    let eval_idx: Vec<usize> = (0..n_cal).filter(|i| i % 2 == 1).collect();
    let eng_conf_at = |i: usize| cal_eval.confs[i][0];
    let eng_ok_at = |i: usize| cal_eval.picks[i][0] == cal_cases[i].gold[0].idx;
    let fit_pairs: Vec<CalibrationPair> = fit_idx
        .iter()
        .map(|&i| CalibrationPair {
            conf: eng_conf_at(i),
            correct: eng_ok_at(i),
        })
        .collect();
    let eval_confs: Vec<f64> = eval_idx.iter().map(|&i| eng_conf_at(i)).collect();
    let eval_oks: Vec<bool> = eval_idx.iter().map(|&i| eng_ok_at(i)).collect();
    let floored = conformal_naive_floor(&fit_pairs, &eval_confs);
    let floor_b_ece = ece_of(
        &floored
            .into_iter()
            .zip(eval_oks.iter().copied())
            .collect::<Vec<(f64, bool)>>(),
    );
    let engine_raw_b_ece = ece_of(
        &eval_confs
            .iter()
            .copied()
            .zip(eval_oks.iter().copied())
            .collect::<Vec<(f64, bool)>>(),
    );

    let engine_acc = rung_correct[0] as f64 / n_cal as f64;
    let ladder: Vec<G1LadderPoint> = G1_LAMBDAS
        .iter()
        .enumerate()
        .map(|(li, &lam)| {
            // The held-out half through the SAME f32 lens both legs read —
            // an identity fit must compare bit-equal to the raw surface,
            // never to a widening rounding artifact.
            let b_pairs: Vec<(f64, bool)> = eval_idx
                .iter()
                .map(|&i| {
                    let (c, ok) = rung_pairs[li][i];
                    ((c as f32) as f64, ok)
                })
                .collect();
            let raw_b = ece_of(&b_pairs);
            // The recalibration leg: the SAME calibrator family fit on the
            // fit half's blend pairs, read on the held-out half — a fit
            // half never scores its own fit. `observe` only RECORDS; the
            // Platt solve is `refit()` (the lane's own convention).
            let mut cal = SigmoidGateCalibrator::new(inp.cal_capacity, inp.cal_min_obs);
            for &i in &fit_idx {
                let (c, ok) = rung_pairs[li][i];
                cal.observe(c as f32, ok);
            }
            cal.refit();
            let recal_b = ece_of(
                &b_pairs
                    .iter()
                    .map(|(c, ok)| (cal.apply(*c as f32) as f64, *ok))
                    .collect::<Vec<(f64, bool)>>(),
            );
            let cal_acc = rung_correct[li] as f64 / n_cal as f64;
            // Pre-registered feasibility: the recalibrated blend readout
            // must beat BOTH bars of the lane's own G1 law on the held-out
            // half (its own uncalibrated surface AND the engine-derived
            // floor), and must not lose accuracy against the unarmed rung
            // (the no-regression leg at selection time). A BEHAVIORALLY
            // identity recalibration (thin window below `min_obs`, or a
            // constant-confidence window whose Platt solve is collinear
            // and keeps the identity parameters) cannot beat its own
            // surface: the raw leg degrades to equality there, the floor
            // leg stays strict. Behavioral, not the `moved` flag — a
            // degenerate fit reports moved while returning identity.
            let behaviorally_identity = b_pairs
                .iter()
                .all(|(c, _)| cal.apply(*c as f32) == *c as f32);
            let beats_raw = if behaviorally_identity {
                recal_b <= raw_b
            } else {
                recal_b < raw_b
            };
            let feasible = beats_raw && recal_b < floor_b_ece && cal_acc >= engine_acc;
            G1LadderPoint {
                param: lam,
                cal_acc,
                raw_b_ece: raw_b,
                recal_b_ece: recal_b,
                feasible,
            }
        })
        .collect();
    let (sel, unsat) = select_g1_rung(&ladder);

    // The test-side recalibrator: fit on the SELECTED rung's FULL cal
    // pairs (the lane's own convention — fit on the full cal slice, read
    // on the test split).
    let mut recal = SigmoidGateCalibrator::new(inp.cal_capacity, inp.cal_min_obs);
    for (c, ok) in &rung_pairs[sel] {
        recal.observe(*c as f32, *ok);
    }
    recal.refit();
    Some(G1Plan {
        lambda: G1_LAMBDAS[sel],
        unsatisfiable: unsat,
        bar: G1ConstraintBar {
            conformal_floor_b: floor_b_ece,
            engine_raw_b_ece,
            n_fit_half: fit_idx.len(),
            n_eval_half: eval_idx.len(),
        },
        ladder,
        recal,
    })
}

/// The selected plan (internal — the record carries the serialized half).
struct G1Plan {
    lambda: f64,
    unsatisfiable: bool,
    bar: G1ConstraintBar,
    ladder: Vec<G1LadderPoint>,
    recal: SigmoidGateCalibrator,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{GoldAnswer, SuiteQuestion};

    #[test]
    fn negation_prefix_and_tokens_featurize_deterministically() {
        let a = pair_features("The cat sat on the mat.", "The cat did not sit on the mat.");
        let b = pair_features("The cat sat on the mat.", "The cat did not sit on the mat.");
        assert_eq!(a, b);
        // "did not" → hyp_neg ≥ 1 after the n't rewrite path (explicit not).
        assert!(a[3] >= 1.0);
        assert_eq!(a[5], 1.0); // neg_diff = 1 − 0
    }

    #[test]
    fn contracted_negation_tokenizes_into_the_negation_vocabulary() {
        let f = pair_features("He is happy.", "He isn't happy.");
        assert!(f[3] >= 1.0, "isn't must count as a hypothesis negation");
    }

    #[test]
    fn lda_prefers_the_better_separated_class() {
        // Class 0: high overlap; class 1: low overlap; class 2: mixed.
        let mut xs = Vec::with_capacity(20);
        let mut ys = Vec::with_capacity(20);
        for _ in 0..20 {
            xs.push([0.9, 0.9, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            ys.push(0);
            xs.push([0.1, 0.1, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            ys.push(1);
            xs.push([0.5, 0.5, 1.0, 2.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0]);
            ys.push(2);
        }
        let lda = NliLda::fit(&xs, &ys).expect("all classes present");
        assert_eq!(
            lda.pick(&lda.delta(&[0.95, 0.95, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
            0
        );
        assert_eq!(
            lda.pick(&lda.delta(&[0.05, 0.05, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
            1
        );
        assert_eq!(
            lda.pick(&lda.delta(&[0.5, 0.5, 1.0, 2.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0])),
            2
        );
    }

    #[test]
    fn fit_refuses_a_missing_class() {
        let xs = vec![[0.1; FEAT_DIM]; 4];
        let ys = vec![0, 0, 1, 1];
        assert!(NliLda::fit(&xs, &ys).is_none());
    }

    #[test]
    fn well_shaped_rejects_wrong_shapes() {
        let mk = |state: serde_json::Value| SuiteCase {
            id: "t".into(),
            state,
            questions: vec![SuiteQuestion {
                qid: "relation".into(),
                kind: QKind::Choice,
                instructions: "rel?".into(),
                criteria: serde_json::json!({"entailment": "e", "neutral": "n", "contradiction": "c"}),
            }],
            gold: vec![GoldAnswer {
                idx: 0,
                soft: vec![0.0; 3],
                gold_score: None,
            }],
        };
        let good = mk(serde_json::json!({"premise": "a", "hypothesis": "b"}));
        assert!(well_shaped(&good));
        let bad_state = mk(serde_json::json!({"article": "a"}));
        assert!(!well_shaped(&bad_state));
        // Two questions — not the NLI shape.
        let mut two = good.clone();
        two.questions.push(two.questions[0].clone());
        two.gold.push(two.gold[0].clone());
        assert!(!well_shaped(&two));
    }

    fn nli_case(id: &str, premise: &str, hypothesis: &str, gold: usize) -> SuiteCase {
        SuiteCase {
            id: id.into(),
            state: serde_json::json!({"premise": premise, "hypothesis": hypothesis}),
            questions: vec![SuiteQuestion {
                qid: "relation".into(),
                kind: QKind::Choice,
                instructions: "rel?".into(),
                criteria: serde_json::json!(
                    {"entailment": "e", "neutral": "n", "contradiction": "c"}
                ),
            }],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![0.0; 3],
                gold_score: None,
            }],
        }
    }

    #[test]
    fn select_g1_rung_prefers_feasible_then_max_acc_then_smallest_lambda() {
        let pt = |param: f64, cal_acc: f64, feasible: bool| G1LadderPoint {
            param,
            cal_acc,
            raw_b_ece: 0.0,
            recal_b_ece: 0.0,
            feasible,
        };
        // All feasible: max accuracy wins; a TIE keeps the SMALLER λ (first).
        let all = vec![
            pt(0.0, 0.50, true),
            pt(0.5, 0.61, true),
            pt(1.0, 0.61, true),
            pt(2.0, 0.55, true),
        ];
        assert_eq!(select_g1_rung(&all), (1, false));
        // The best-accuracy rung infeasible → the best FEASIBLE one.
        let blocked = vec![
            pt(0.0, 0.50, true),
            pt(0.5, 0.70, false),
            pt(1.0, 0.60, true),
        ];
        assert_eq!(select_g1_rung(&blocked), (2, false));
        // Nothing feasible → rung 0 + the loud unsatisfiable verdict.
        let none = vec![pt(0.0, 0.50, false), pt(0.5, 0.60, false)];
        assert_eq!(select_g1_rung(&none), (0, true));
    }

    #[test]
    fn g1_constrained_posture_runs_end_to_end_on_synthetic_slices() {
        // Class-separated pairs: 0 = hypothesis contained in the premise
        // (high overlap), 1 = disjoint vocabulary, 2 = antonym cross.
        let pair = |k: usize| match k {
            0 => (
                "the quick brown fox jumps over the lazy dog",
                "a fox jumps",
                0usize,
            ),
            1 => (
                "the committee approved the budget yesterday",
                "gravity bends light",
                1,
            ),
            _ => (
                "the team will win the final match",
                "the team will lose the final match",
                2,
            ),
        };
        let mut cal_cases = Vec::new();
        let mut cases = Vec::new();
        for i in 0..24usize {
            for k in 0..3usize {
                let (p, h, g) = pair(k);
                cal_cases.push(nli_case(&format!("c{i}{k}"), p, h, g));
                cases.push(nli_case(&format!("t{i}{k}"), p, h, g));
            }
        }
        // The engine: an over-confident class-0 bias — always picks 0 at
        // conf 0.9, so its raw readout is badly miscalibrated (acc 1/3).
        let eval = |n: usize| Eval {
            probs: (0..n).map(|_| vec![vec![0.9, 0.05, 0.05]]).collect(),
            picks: (0..n).map(|_| vec![0usize]).collect(),
            confs: (0..n).map(|_| vec![0.9]).collect(),
            abstained: (0..n).map(|_| vec![false]).collect(),
        };
        let suite = Suite {
            name: "synthetic",
            cases,
            option_counts_note: "test",
        };
        let res = nli_feature_ab_pass(
            &eval(suite.cases.len()),
            &suite,
            &eval(cal_cases.len()),
            &cal_cases,
            Some(G1Inputs {
                engine_calibrated_test_ece: 0.05,
                conformal_floor_test: 0.30,
                cal_capacity: 512,
                cal_min_obs: 8,
            }),
        )
        .unwrap()
        .expect("well-shaped synthetic suite must run");
        let g1 = res.blend_g1_constrained.expect("g1 posture present");
        assert_eq!(g1.ladder.len(), 8);
        assert_eq!(g1.ladder[0].param, 0.0);
        assert!(g1.selected >= 0.0 && g1.selected <= 8.0);
        assert!(g1.bar.conformal_floor_b.is_finite());
        assert!(g1.bar.engine_raw_b_ece.is_finite());
        for p in &g1.ladder {
            assert!(p.raw_b_ece.is_finite() && p.recal_b_ece.is_finite());
        }
        // The head separates the synthetic classes: some armed rung must
        // read above the engine's 1/3 accuracy on cal.
        assert!(g1.ladder[7].cal_acc > g1.ladder[0].cal_acc);
        // The no-regression leg: the selected posture never reads below
        // the engine on the test read (equality = the unarmed fallback).
        assert!(g1.acc >= res.baseline_acc);
        // The miniature G1 machinery: recalibrating the engine's own
        // miscalibrated surface must improve it on the held-out half.
        assert!(g1.ladder[0].feasible);
        assert!(g1.g1_raw.blend_ece.is_finite());
        assert!(g1.g1_recalibrated.blend_ece.is_finite());
    }
}
