//! Issue 077 — the option-PERMUTATION spread probe (the pure core).
//!
//! LiquidAI's d1 recipe names "shuffling answer options" among the levers
//! that mattered: order sensitivity is a live failure class for decision
//! models. This module is the lane-agnostic measurement: a choice question
//! is re-answered under K deterministic option orderings and the probe
//! reads how much the answer MOVES — per case (top-pick spread + flips) and
//! in aggregate (median + max swing, the ≤ 2 pt gate).
//!
//! Everything here is PURE: orderings, permutation application, label-space
//! mapping, spread math, the gate, and the canary. The lanes live in
//! [`super::perm_probe`] behind the [`ChoiceOracle`] seam — each real lane
//! answers through its own production decide path, so the probe measures
//! what ships, never a parallel rendering.
//!
//! Measurement laws:
//! - **Label space, never position space.** Option indexes move under
//!   permutation; the option LABELS are the invariant. Every comparison
//!   (picks, probability of a pick) happens on labels mapped back to the
//!   original criteria order.
//! - **Choice questions only.** Score criteria is an ordered level array
//!   (index = level) and noul's order is fixed `[false, true]` by law —
//!   permuting either changes the QUESTION, not the presentation.
//! - **Deterministic orderings.** Identity first, then K−1 SplitMix64
//!   Fisher–Yates shuffles from a fixed per-slot seed (recorded in the
//!   output) — the same orderings every run, byte-reproducible.
//! - **The gate is the MEDIAN.** median swing ≤ [`GATE_MEDIAN_SWING_PT`]
//!   (2 pt) per lane per suite; the max is disclosed beside it, never
//!   gatekeeping alone (one order-fragile case must surface, but a single
//!   case must not condemn a lane).

use serde::Serialize;
use serde_json::{Map, Value};

use super::suites::{shuffle, GoldAnswer, QKind, SplitMix64, SuiteCase, SuiteQuestion};

/// The order-sensitivity gate: a lane's MEDIAN top-pick swing across
/// orderings must stay ≤ 2 probability points (issue 077 T1).
pub const GATE_MEDIAN_SWING_PT: f64 = 2.0;

/// The CONTROL's numeric ceiling (issue 077): the modelless engine's L1
/// normalizer sums option scores in PRESENTATION order, and f32 addition
/// is not associative — permuting the options reorders the sum and moves
/// every probability by ULPs. The measured envelope on the by-name-bound
/// suites (banking77 / massive_intent_en, Bench 128): max swing below the
/// 0.005 pt display grain, zero flips. The control's verdict therefore
/// keys on 0.01 pt (2× that envelope — a measured noise ceiling, not a
/// behavior constant) plus tie-aware flip freedom; STRICT byte-identity
/// stays reported beside it (`PermLaneReport::control_invariant`) as the
/// honest "not bit-stable, stable within the normalizer's fp envelope"
/// disclosure.
pub const CONTROL_MEDIAN_CEILING_PT: f64 = 0.01;

/// The seed constant for the probe's ordering stream (any fixed value;
/// reproducibility is what matters, not the value — the MASSIVE seed law).
/// XORed with a per-slot mix so different slots see different shuffles.
pub const PROBE_SEED: u64 = 0x0777_0001;

/// One lane answer to one presented ordering: probabilities + argmax pick,
/// both in PRESENTED order (position space — map through the ordering
/// before comparing anything).
#[derive(Debug, Clone)]
pub struct OrderingAnswer {
    pub probs: Vec<f64>,
    pub pick: usize,
}

/// K deterministic orderings of `n` options: the identity FIRST (the
/// reference ordering every spread reads against), then K−1 shuffles.
/// `k` is clamped to ≥ 1; a single option yields K copies of the identity
/// (nothing to permute — the spread is structurally zero).
#[must_use]
pub fn perm_orderings(n: usize, k: usize, seed: u64) -> Vec<Vec<usize>> {
    let k = k.max(1);
    let mut out = Vec::with_capacity(k);
    out.push((0..n).collect());
    for rep in 1..k {
        let mut rng = SplitMix64::new(seed ^ (rep as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut ord: Vec<usize> = (0..n).collect();
        shuffle(&mut ord, &mut rng);
        out.push(ord);
    }
    out
}

/// The per-slot ordering seed: the probe seed mixed with the case id and
/// qid (FNV-1a — orderings differ across slots, stay stable across runs).
#[must_use]
pub fn slot_seed(case_id: &str, qid: &str) -> u64 {
    let mut h: u64 = PROBE_SEED;
    for b in case_id.as_bytes().iter().chain(qid.as_bytes()) {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Rebuild a CHOICE question's criteria object under `order`
/// (`order[pos] = the ORIGINAL option index presented at pos`). The
/// insertion order of the rebuilt object IS the new presentation order
/// (serde_json preserve_order) — the one rendering fact every lane reads.
///
/// # Errors
/// When the question is not a choice (or its criteria is not an object),
/// or `order` is not a permutation of `0..n`.
pub fn permute_choice(q: &SuiteQuestion, order: &[usize]) -> Result<SuiteQuestion, String> {
    let obj = q
        .criteria
        .as_object()
        .ok_or_else(|| format!("qid {}: permutation needs a choice question", q.qid))?;
    let n = obj.len();
    if !is_permutation(order, n) {
        return Err(format!(
            "qid {}: order is not a permutation of 0..{n}",
            q.qid
        ));
    }
    let keys: Vec<&String> = obj.keys().collect();
    let mut m = Map::new();
    for &pos in order {
        let key = keys[pos];
        m.insert(key.clone(), obj[key].clone());
    }
    Ok(SuiteQuestion {
        qid: q.qid.clone(),
        kind: QKind::Choice,
        instructions: q.instructions.clone(),
        criteria: Value::Object(m),
    })
}

fn is_permutation(order: &[usize], n: usize) -> bool {
    order.len() == n && {
        let mut seen = vec![false; n];
        order.iter().all(|&o| o < n && !seen[o] && {
            seen[o] = true;
            true
        })
    }
}

/// The single-question case a lane answers for one (slot, ordering):
/// the case's own state + the PERMUTED choice question. Gold rides along
/// (index-mapped) so the record keeps the suite's shape lawful — no lane
/// decide path reads it.
///
/// # Errors
/// When `qi` is out of range or the question does not permute.
pub fn permuted_case(
    case: &SuiteCase,
    qi: usize,
    order: &[usize],
    rep: usize,
) -> Result<SuiteCase, String> {
    let q = case
        .questions
        .get(qi)
        .ok_or_else(|| format!("case {}: question index {qi} out of range", case.id))?;
    let pq = permute_choice(q, order)?;
    Ok(SuiteCase {
        id: format!("{}#{}/perm{rep}", case.id, q.qid),
        state: case.state.clone(),
        questions: vec![pq],
        gold: vec![case.gold[qi].clone()],
    })
}

/// Map presented-order probabilities into LABEL space (original criteria
/// order): `out[order[pos]] = probs[pos]`.
#[must_use]
pub fn label_space_probs(probs: &[f64], order: &[usize], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n];
    for (pos, &p) in probs.iter().enumerate() {
        if let Some(&orig) = order.get(pos)
            && orig < n
        {
            out[orig] = p;
        }
    }
    out
}

/// The per-case spread record: how one lane's answer to one question moves
/// under the K orderings, in LABEL space.
#[derive(Debug, Clone, Serialize)]
pub struct CaseSpread {
    pub case_id: String,
    pub qid: String,
    pub n_options: usize,
    /// The identity ordering's top-pick LABEL — the reference every
    /// ordering is read against.
    pub ref_pick: String,
    /// Top-pick label under each ordering (index-aligned with the
    /// orderings; [0] is always `ref_pick`).
    pub pick_labels: Vec<String>,
    /// Probability on `ref_pick` under each ordering.
    pub p_ref: Vec<f64>,
    /// Probability on the gold label under each ordering (`None` when the
    /// gold label is not among the options — never over the suite paths).
    pub p_gold: Option<Vec<f64>>,
    /// `max(p_ref) − min(p_ref)` — the case's swing (probability points
    /// ×100 live in [`SpreadStats`]).
    pub swing: f64,
    /// Any ordering's top pick disagrees with `ref_pick`.
    pub flipped: bool,
    /// The identity ordering's top-2 probabilities tie EXACTLY (margin
    /// 0.0): any flip on a tied slot is the engine's deterministic
    /// first-position tie-break, not order sensitivity — the control's
    /// flip rule exempts it.
    pub tie: bool,
}

/// Compute the spread for one slot from its per-ordering answers.
///
/// # Errors
/// When the answer/ordering counts disagree or an answer's vector length
/// does not match the option count.
pub fn case_spread(
    case_id: &str,
    qid: &str,
    labels: &[String],
    orderings: &[Vec<usize>],
    answers: &[OrderingAnswer],
    gold_label: Option<&str>,
) -> Result<CaseSpread, String> {
    let n = labels.len();
    if orderings.len() != answers.len() || answers.is_empty() {
        return Err(format!(
            "case {case_id}/{qid}: {} orderings vs {} answers",
            orderings.len(),
            answers.len()
        ));
    }
    for (k, a) in answers.iter().enumerate() {
        if a.probs.len() != n || orderings[k].len() != n {
            return Err(format!(
                "case {case_id}/{qid}: ordering {k} width mismatch (probs {}, \
                 order {}, options {n})",
                a.probs.len(),
                orderings[k].len()
            ));
        }
        if a.pick >= n {
            return Err(format!(
                "case {case_id}/{qid}: pick {} out of range {n}",
                a.pick
            ));
        }
    }
    let label_at = |ord: &[usize], pos: usize| -> String {
        labels[ord[pos]].clone()
    };
    // Reference pick from the identity ordering ([0]).
    let ref_orig = orderings[0][answers[0].pick];
    let ref_pick = labels[ref_orig].clone();
    let pick_labels: Vec<String> = orderings
        .iter()
        .zip(answers.iter())
        .map(|(ord, a)| label_at(ord, a.pick))
        .collect();
    let p_ref: Vec<f64> = orderings
        .iter()
        .zip(answers.iter())
        .map(|(ord, a)| a.probs[ord.iter().position(|&o| o == ref_orig).unwrap_or(0)])
        .collect();
    let p_gold = gold_label.and_then(|g| labels.iter().position(|l| l == g)).map(
        |gold_orig| {
            orderings
                .iter()
                .zip(answers.iter())
                .map(|(ord, a)| a.probs[ord.iter().position(|&o| o == gold_orig).unwrap_or(0)])
                .collect::<Vec<f64>>()
        },
    );
    let max = p_ref.iter().copied().fold(f64::MIN, f64::max);
    let min = p_ref.iter().copied().fold(f64::MAX, f64::min);
    let swing = (max - min).clamp(0.0, 1.0);
    // The identity ordering's top-2 margin — exact f64 equality is the
    // tie test (the engine's f32 scores widen losslessly; a tie in f32 is
    // a tie here).
    let mut sorted0 = answers[0].probs.clone();
    sorted0.sort_by(|a, b| b.total_cmp(a));
    let tie = sorted0.len() >= 2 && sorted0[0] == sorted0[1];
    let flipped = pick_labels.iter().any(|l| *l != ref_pick);
    Ok(CaseSpread {
        case_id: case_id.to_string(),
        qid: qid.to_string(),
        n_options: n,
        ref_pick,
        pick_labels,
        p_ref,
        p_gold,
        swing: swing.clamp(0.0, 1.0),
        flipped,
        tie,
    })
}

/// The gate verdict for one lane × suite cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SpreadVerdict {
    Pass,
    Red,
}

/// Aggregate spread over one lane's cases: the issue's median + max
/// top-pick spread, the flip rate, and the ≤ 2 pt gate verdict.
#[derive(Debug, Clone, Serialize)]
pub struct SpreadStats {
    pub n: usize,
    pub median_swing_pt: f64,
    pub max_swing_pt: f64,
    pub flips: usize,
    pub flip_rate: f64,
    pub verdict: SpreadVerdict,
}

/// Nearest-rank percentile over a SORTED slice: `ceil(q·n)`, 1-based (the
/// e0 law; the tail support lives beside every published n).
fn percentile(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    debug_assert!(n > 0);
    let idx = ((q * n as f64).ceil() as usize).clamp(1, n) - 1;
    sorted[idx]
}

/// Aggregate the per-case spreads. Empty input is a structurally-zero PASS
/// at n 0 — callers keep empty cells loud at the suite level instead.
#[must_use]
pub fn spread_stats(spreads: &[CaseSpread]) -> SpreadStats {
    let n = spreads.len();
    if n == 0 {
        return SpreadStats {
            n: 0,
            median_swing_pt: 0.0,
            max_swing_pt: 0.0,
            flips: 0,
            flip_rate: 0.0,
            verdict: SpreadVerdict::Pass,
        };
    }
    let mut swings: Vec<f64> = spreads.iter().map(|s| s.swing * 100.0).collect();
    swings
        .sort_by(|a, b| a.total_cmp(b));
    let median = percentile(&swings, 0.5);
    let max = swings[n - 1];
    let flips = spreads.iter().filter(|s| s.flipped).count();
    SpreadStats {
        n,
        median_swing_pt: median,
        max_swing_pt: max,
        flips,
        flip_rate: flips as f64 / n as f64,
        verdict: if median > GATE_MEDIAN_SWING_PT {
            SpreadVerdict::Red
        } else {
            SpreadVerdict::Pass
        },
    }
}

/// The canary case: five distinct options, neutral instructions, a state
/// every lane renders safely (plain strings). Its job is to prove the
/// probe CAN fire: any lane that reads POSITION instead of content flips
/// on it (the [`crate::harness::permutation` tests] ship both directions),
/// and every published probe run carries a canary cell per lane as the
/// run's own liveness witness.
#[must_use]
pub fn canary_case() -> SuiteCase {
    let mut m = Map::new();
    for l in ["alpha", "beta", "gamma", "delta", "epsilon"] {
        m.insert(
            l.to_string(),
            Value::String(format!("the {l} option")),
        );
    }
    SuiteCase {
        id: "perm_canary".to_string(),
        state: Value::String("The quick brown fox jumps over the lazy dog.".to_string()),
        questions: vec![SuiteQuestion {
            qid: "canary".to_string(),
            kind: QKind::Choice,
            instructions: "Pick the most relevant option.".to_string(),
            criteria: Value::Object(m),
        }],
        gold: vec![GoldAnswer {
            idx: 2,
            soft: vec![],
            gold_score: None,
        }],
    }
}

/// The lane-agnostic answer seam: answer ONE choice question rendered as a
/// single-question case (options in presentation order). Every probed lane
/// implements this over its OWN production decide path — the probe never
/// re-renders a lane's wire.
pub trait ChoiceOracle {
    /// (probabilities, pick) in the case's PRESENTED option order.
    ///
    /// # Errors
    /// Whatever the lane's decide path returns (unreachable server, wire
    /// drift) — propagated loud, never mapped to a fake answer.
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q5() -> SuiteQuestion {
        let mut m = Map::new();
        for l in ["alpha", "beta", "gamma", "delta", "epsilon"] {
            m.insert(l.to_string(), Value::Null);
        }
        SuiteQuestion {
            qid: "q".to_string(),
            kind: QKind::Choice,
            instructions: "pick".to_string(),
            criteria: Value::Object(m),
        }
    }

    fn labels5() -> Vec<String> {
        ["alpha", "beta", "gamma", "delta", "epsilon"]
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    }

    #[test]
    fn orderings_are_identity_first_and_deterministic() {
        let a = perm_orderings(5, 4, 42);
        let b = perm_orderings(5, 4, 42);
        assert_eq!(a, b, "same seed must reproduce byte-identical orderings");
        assert_eq!(a.len(), 4);
        assert_eq!(a[0], vec![0, 1, 2, 3, 4], "identity is always [0]");
        for (rep, ord) in a.iter().enumerate() {
            let mut sorted = ord.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..5).collect::<Vec<usize>>(), "rep {rep} valid");
        }
        assert_ne!(a[1], a[0], "a shuffle must move something");
        // k clamps to >= 1; n = 1 degenerates to identity copies.
        assert_eq!(perm_orderings(5, 0, 7).len(), 1);
        assert!(perm_orderings(1, 5, 7).iter().all(|o| o == &[0usize]));
    }

    #[test]
    fn permute_choice_rebuilds_criteria_under_the_order() {
        let q = q5();
        let order = vec![3, 0, 4, 1, 2]; // delta alpha epsilon beta gamma
        let p = permute_choice(&q, &order).expect("permutes");
        let keys: Vec<&String> = p.criteria.as_object().expect("object").keys().collect();
        let want = ["delta", "alpha", "epsilon", "beta", "gamma"];
        for (k, w) in keys.iter().zip(want) {
            assert_eq!(k.as_str(), w);
        }
        // The original question is untouched.
        let orig: Vec<&String> = q.criteria.as_object().expect("object").keys().collect();
        assert_eq!(orig[0], "alpha");
        // Non-permutation refused.
        assert!(permute_choice(&q, &[0, 0, 2, 3, 4]).is_err());
        // Score questions refuse (permuting levels changes the question).
        let mut sq = q5();
        sq.kind = QKind::Score;
        assert!(permute_choice(&sq, &[1, 0]).is_err());
    }

    #[test]
    fn label_space_round_trips_the_identity() {
        let n = 5;
        let order = perm_orderings(n, 3, 9)[1].clone();
        let mut presented = vec![0.0; n];
        for (pos, &orig) in order.iter().enumerate() {
            presented[pos] = (orig as f64 + 1.0) / 10.0;
        }
        let labels = label_space_probs(&presented, &order, n);
        for (orig, p) in order.iter().zip(presented.iter()) {
            assert_eq!(labels[*orig], *p);
        }
    }

    #[test]
    fn case_spread_reads_swing_flips_and_gold() {
        let labels = labels5();
        let orderings = perm_orderings(5, 3, 11);
        // A lane whose answer follows CONTENT: always "gamma" (original
        // index 2) with p 0.8, wherever it sits.
        let answers: Vec<OrderingAnswer> = orderings
            .iter()
            .map(|ord| {
                let pos = ord.iter().position(|&o| o == 2).expect("gamma present");
                let mut probs = vec![0.05; 5];
                probs[pos] = 0.8;
                OrderingAnswer { probs, pick: pos }
            })
            .collect();
        let s = case_spread("c", "q", &labels, &orderings, &answers, Some("gamma"))
            .expect("spread");
        assert_eq!(s.ref_pick, "gamma");
        assert!(!s.flipped);
        assert!(s.swing < 1e-12, "content-faithful lane must not swing");
        assert_eq!(s.p_gold.as_ref().expect("gold vec")[0], 0.8);
        let st = spread_stats(std::slice::from_ref(&s));
        assert_eq!(st.verdict, SpreadVerdict::Pass);
        assert_eq!(st.flips, 0);
    }

    #[test]
    fn case_spread_rejects_shape_mismatch() {
        let labels = labels5();
        let orderings = perm_orderings(5, 2, 3);
        let answers = vec![OrderingAnswer {
            probs: vec![1.0],
            pick: 0,
        }];
        assert!(case_spread("c", "q", &labels, &orderings, &answers, None).is_err());
    }

    // ── the canary + mock oracles: the probe proves it can fire ─────────

    /// A POSITION reader: always the first presented option, p 1.0. The
    /// order-biased lane class the canary exists to catch.
    struct FirstOptionOracle;
    impl ChoiceOracle for FirstOptionOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            let n = case.questions[0]
                .criteria
                .as_object()
                .map(Map::len)
                .unwrap_or(0);
            let mut probs = vec![0.0; n];
            probs[0] = 1.0;
            Ok(OrderingAnswer { probs, pick: 0 })
        }
    }

    /// A CONTENT reader: always the gold label's option, p 1.0, wherever
    /// it sits. The invariant lane class the gate must never accuse.
    struct GoldContentOracle {
        gold_label: String,
    }
    impl ChoiceOracle for GoldContentOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            let keys: Vec<String> = case.questions[0]
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            let pick = keys
                .iter()
                .position(|k| *k == self.gold_label)
                .unwrap_or(0);
            let mut probs = vec![0.0; keys.len()];
            probs[pick] = 1.0;
            Ok(OrderingAnswer { probs, pick })
        }
    }

    /// Run a slot end to end through an oracle — the exact loop the
    /// runner half uses (shared so the mocks measure the REAL path).
    pub(crate) fn run_slot(
        oracle: &mut dyn ChoiceOracle,
        case: &SuiteCase,
        qi: usize,
        k: usize,
        gold_label: Option<&str>,
    ) -> Result<CaseSpread, String> {
        let q = &case.questions[qi];
        let labels: Vec<String> = q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        let orderings = perm_orderings(labels.len(), k, slot_seed(&case.id, &q.qid));
        let mut answers = Vec::with_capacity(orderings.len());
        for (rep, order) in orderings.iter().enumerate() {
            let pc = permuted_case(case, qi, order, rep)?;
            answers.push(oracle.answer(&pc)?);
        }
        case_spread(&case.id, &q.qid, &labels, &orderings, &answers, gold_label)
    }

    #[test]
    fn the_canary_reds_an_order_biased_lane() {
        let canary = canary_case();
        let s = run_slot(&mut FirstOptionOracle, &canary, 0, 5, Some("gamma"))
            .expect("canary spread");
        assert!(s.flipped, "a position reader must flip under permutation");
        assert!(
            s.swing > 0.9,
            "the first-option lane's p_ref collapses to 0 under shuffles: swing {}",
            s.swing
        );
        let st = spread_stats(std::slice::from_ref(&s));
        assert_eq!(st.verdict, SpreadVerdict::Red, "the gate MUST fire");
        assert!(st.median_swing_pt > GATE_MEDIAN_SWING_PT);
    }

    #[test]
    fn the_canary_greens_a_content_faithful_lane() {
        let canary = canary_case();
        let mut o = GoldContentOracle {
            gold_label: "gamma".to_string(),
        };
        let s = run_slot(&mut o, &canary, 0, 5, Some("gamma")).expect("canary spread");
        assert!(!s.flipped);
        assert!(s.swing < 1e-12);
        assert_eq!(
            spread_stats(std::slice::from_ref(&s)).verdict,
            SpreadVerdict::Pass
        );
    }

    #[test]
    fn spread_stats_median_max_and_the_gate_boundary() {
        let mk = |swing: f64, flipped: bool| CaseSpread {
            case_id: "c".into(),
            qid: "q".into(),
            n_options: 5,
            ref_pick: "alpha".into(),
            pick_labels: vec!["alpha".into()],
            p_ref: vec![1.0 - swing],
            p_gold: None,
            swing,
            flipped,
            tie: false,
        };
        // Exactly at the gate (2 pt) = PASS; a hair above = RED.
        let at_gate = [mk(0.02, false), mk(0.01, false), mk(0.03, true)];
        let st = spread_stats(&at_gate);
        assert_eq!(st.n, 3);
        assert_eq!(st.median_swing_pt, 2.0, "nearest-rank p50 of [1,2,3]");
        assert_eq!(st.max_swing_pt, 3.0);
        assert_eq!(st.flips, 1);
        assert!((st.flip_rate - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(st.verdict, SpreadVerdict::Pass, "median 2pt is NOT > 2pt");
        let over_gate = [mk(0.02, false), mk(0.04, true), mk(0.03, false)];
        assert_eq!(spread_stats(&over_gate).verdict, SpreadVerdict::Red);
        // Empty input is structurally quiet — callers keep it loud.
        let st0 = spread_stats(&[]);
        assert_eq!(st0.verdict, SpreadVerdict::Pass);
        assert_eq!(st0.n, 0);
    }

    #[test]
    fn permuted_case_carries_the_state_and_mapped_gold() {
        let mut canary = canary_case();
        canary.gold[0].idx = 2;
        let pc = permuted_case(&canary, 0, &[4, 3, 2, 1, 0], 1).expect("case");
        assert_eq!(pc.questions.len(), 1);
        assert_eq!(pc.state, canary.state);
        assert_eq!(pc.gold[0].idx, 2, "gold rides along index-mapped");
        let keys: Vec<&String> = pc.questions[0]
            .criteria
            .as_object()
            .expect("object")
            .keys()
            .collect();
        assert_eq!(keys[0], "epsilon");
        assert_eq!(pc.id, "perm_canary#canary/perm1");
        assert!(permuted_case(&canary, 7, &[0], 0).is_err());
    }

    #[test]
    fn case_spread_flags_an_exact_top2_tie() {
        let labels = labels5();
        // Explicit orderings: identity + a swap that moves alpha off the
        // first position (a seeded shuffle could land alpha-first by
        // chance — the tie-break flip must be deterministic in the test).
        let orderings = vec![vec![0, 1, 2, 3, 4], vec![1, 0, 2, 3, 4]];
        // All-equal probabilities: an exact tie in the identity ordering.
        let answers: Vec<OrderingAnswer> = orderings
            .iter()
            .map(|_ord| OrderingAnswer {
                probs: vec![0.2; 5],
                pick: 0,
            })
            .collect();
        let s = case_spread("c", "q", &labels, &orderings, &answers, None).expect("spread");
        assert!(s.tie, "uniform probabilities tie exactly");
        assert!(s.flipped, "position-0 tie-break moves with the ordering");
        assert_eq!(s.ref_pick, "alpha");
        assert_eq!(s.pick_labels[1], "beta");
        // A clear winner: no tie.
        let answers2: Vec<OrderingAnswer> = orderings
            .iter()
            .map(|ord| {
                let pos = ord.iter().position(|&o| o == 2).expect("gamma");
                let mut probs = vec![0.05; 5];
                probs[pos] = 0.8;
                OrderingAnswer { probs, pick: pos }
            })
            .collect();
        let s2 = case_spread("c", "q", &labels, &orderings, &answers2, None).expect("spread");
        assert!(!s2.tie);
        assert!(!s2.flipped);
    }

    #[test]
    fn slot_seed_is_stable_and_spreads() {
        assert_eq!(slot_seed("a", "q"), slot_seed("a", "q"));
        assert_ne!(slot_seed("a", "q"), slot_seed("b", "q"));
        assert_ne!(slot_seed("a", "q"), slot_seed("q", "a"));
    }
}
