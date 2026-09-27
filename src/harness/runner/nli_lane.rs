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
//! The head posterior is a SIGMOID readout, never softmax (the house
//! sigmoid law): `p_k = σ(δ_k − mean_j δ_j)`, normalized to sum 1.
//!
//! Scope: the arm runs only where EVERY eval and cal case is well-shaped
//! (state object with string `premise` + `hypothesis`, exactly one Choice
//! question with exactly 3 options). Anything else is a LOUD skip
//! (`Ok(None)` + a stderr line) — never a silent absence, never a partial
//! population wearing the suite's name.

use serde::Serialize;

use super::{Eval, QKind, Suite, SuiteCase};

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
    let hyp_neg = ht.iter().filter(|t| NEGATIONS.contains(&t.as_str())).count();
    let prem_neg = pt.iter().filter(|t| NEGATIONS.contains(&t.as_str())).count();

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
        let hit = (pset.contains(w1) && hset.contains(w2))
            || (pset.contains(w2) && hset.contains(w1));
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
    q.criteria
        .as_object()
        .is_some_and(|m| m.len() == 3)
        && pair_state(case).is_some()
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

/// The A/B pass. `Ok(None)` = loud skip (the suite is not premise/
/// hypothesis-shaped); the caller prints nothing further — the pass has
/// already announced itself on stderr.
pub fn nli_feature_ab_pass(
    raw_eval: &Eval,
    suite: &Suite,
    cal_eval: &Eval,
    cal_cases: &[SuiteCase],
    // (the engine's calibrated readout ECE, the suite's conformal-naive
    // floor) — both already computed by the lane; `None` = no floor (no
    // G1 claim either way).
    g1_inputs: Option<(f64, f64)>,
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
        eprintln!("  [nli-feature-ab] suite {}: cal slice lacks a class — SKIPPED", suite.name);
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
            let pick = if margin > tau { lda.pick(&d) } else { argmax(pe) };
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

    // ── The ONE test read ──
    let n = suite.cases.len();
    let mut baseline_correct = 0usize;
    let mut head_correct = 0usize;
    let mut head_confusion = vec![[0usize; 3]; 3];
    let (mut add_correct, mut add_overrides, mut add_agree) = (0usize, 0usize, 0usize);
    let (mut ovr_correct, mut ovr_overrides, mut ovr_agree) = (0usize, 0usize, 0usize);
    let (mut both_right, mut both_wrong, mut hwins, mut hloss) = (0usize, 0usize, 0usize, 0usize);
    let mut blend_pairs: Vec<(f64, bool)> = Vec::with_capacity(n);
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
        let mut best = 0usize;
        let mut best_s = f64::NEG_INFINITY;
        for (k, &pk) in ph.iter().enumerate() {
            let s = pe[k] + sel_add * pk;
            if s > best_s {
                best_s = s;
                best = k;
            }
        }
        add_correct += usize::from(best == gold);
        add_overrides += usize::from(best != eng_pick);
        add_agree += usize::from(best == eng_pick);

        // The blended readout q ∝ pe + λ·ph — the promoted posture's own
        // confidence surface, for the G1 triple.
        if g1_inputs.is_some() {
            let mut q = [0.0f64; 3];
            let mut s = 0.0;
            for k in 0..3 {
                q[k] = pe[k] + sel_add * ph[k];
                s += q[k];
            }
            let s = s.max(1e-12);
            let conf = q.iter().fold(0.0f64, |m, &v| m.max(v / s));
            blend_pairs.push((conf, best == gold));
        }

        // override gate at sel_tau
        let pick = if lda.margin(&d) > sel_tau { head_pick } else { eng_pick };
        ovr_correct += usize::from(pick == gold);
        ovr_overrides += usize::from(pick != eng_pick);
        ovr_agree += usize::from(pick == eng_pick);
    }
    let f = |c: usize| c as f64 / n.max(1) as f64;

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
        blend_readout_g1: g1_inputs.map(|(cal_ece, floor)| {
            let blend_ece = crate::harness::metrics::ece_of(&blend_pairs);
            BlendG1 {
                blend_ece,
                engine_calibrated_ece: cal_ece,
                conformal_floor: floor,
                pass: blend_ece <= floor,
            }
        }),
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
        assert_eq!(lda.pick(&lda.delta(&[0.95, 0.95, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])), 0);
        assert_eq!(lda.pick(&lda.delta(&[0.05, 0.05, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])), 1);
        assert_eq!(lda.pick(&lda.delta(&[0.5, 0.5, 1.0, 2.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0])), 2);
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
            gold: vec![GoldAnswer { idx: 0, soft: vec![0.0; 3], gold_score: None }],
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
}
