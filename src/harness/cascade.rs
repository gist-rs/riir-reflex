//! Issue 038 T4′ — the "Reflex · cascade" lane: the modelless lane answers
//! every question; questions where the CALIBRATED fused gate abstained
//! (score + [`katgpt_core::distance_abstain::CorpusDistanceGate`], fitted at
//! the cal-slice 30th percentile — the shipped T1.6 arena posture) escalate
//! to the laya lane's answer for the same question. No training, no new
//! model — a composition of two lanes that already answer byte-identical
//! questions over the same case set.
//!
//! Pure composition (ungated): the runner hands over the two lanes'
//! per-question records ([`ModellessQuestions`] from the modelless eval,
//! [`LayaQuestions`] from the riir laya backend) and this module returns the
//! [`CascadeReport`] — accuracy over ALL questions plus the escalation rate,
//! which IS the lane's latency claim (a deployed cascade pays the escalator
//! only on the escalated fraction).
//!
//! Escalation semantics, pinned:
//! - not abstained → the modelless pick stands (label space; the same forced
//!   picks that define the published modelless row);
//! - abstained AND the escalator served that case → the escalator's pick;
//! - abstained AND the escalator did NOT serve that case (ANE bucket skip,
//!   `--laya-max-questions` trim, weights absent) → the abstain cannot be
//!   escalated: the modelless forced pick stands and the question counts in
//!   `n_un_escalated` (disclosed, never folded into the escalation rate — a
//!   missing escalator is a coverage limit, not a latency fact).
//!
//! Noul/score questions need no special case: both lanes' picks are already
//! label-space (`[no, yes]` for noul, level index for score — the harness's
//! own conversion law), and in this harness every question has gold, so an
//! abstain is never itself a correct answer here.

use serde::Serialize;

use super::suites::SuiteCase;

/// The modelless lane's per-question record over the FULL case set (one
/// entry per case, per question — index-aligned with `suite.cases`).
#[derive(Debug, Clone)]
pub struct ModellessQuestions {
    /// Forced label-space pick (argmax of the probability vector; the same
    /// picks that define the published modelless `hard.accuracy` row).
    pub picks: Vec<Vec<usize>>,
    /// The CALIBRATED fused-gate abstain flag per question — the shipped
    /// posture (fitted thresholds), not the raw birth constants.
    pub abstained: Vec<Vec<bool>>,
}

/// The escalator lane's per-question record over the SERVED case subset.
#[derive(Debug, Clone)]
pub struct LayaQuestions {
    /// `suite.cases` index of each served case, in serving order (the ANE
    /// bucket skips and the question-cap trim are already removed — the
    /// runner's own `served_orig` walk).
    pub served_orig: Vec<usize>,
    /// Label-space picks per served case, per question.
    pub picks: Vec<Vec<usize>>,
}

/// The composed lane's row (results.json + the TABLES.md cascade block).
#[derive(Debug, Clone, Serialize)]
pub struct CascadeReport {
    /// Questions in the suite (the composition denominator).
    pub n_questions: usize,
    /// Questions the modelless lane answered (pass-through).
    pub n_answered: usize,
    /// Abstains that were escalated (the escalator served the case).
    pub n_escalated: usize,
    /// Abstains with NO escalator answer — the modelless forced pick stood.
    /// A coverage disclosure, excluded from the escalation rate.
    pub n_un_escalated: usize,
    /// `n_escalated / n_questions` — the lane's latency claim (a deployed
    /// cascade pays the escalator's per-question cost on this fraction).
    pub escalation_rate: f64,
    /// Cascade accuracy over ALL questions (escalated + un-escalated +
    /// pass-through), forced — abstains never count as correct here.
    pub accuracy: f64,
    pub n_correct: usize,
    /// The escalator's correctness ON the escalated set — the causal half
    /// of the delta vs the modelless row (the tables print it beside what
    /// the forced modelless picks would have scored on the same set).
    pub escalated_laya_correct: usize,
    /// What the modelless forced picks would have scored on the same
    /// escalated set (diagnostic — never part of `accuracy`).
    pub escalated_modelless_correct: usize,
    /// Questions the escalator served (any posture) — the coverage face.
    pub escalator_served_questions: usize,
}

/// Compose the cascade over one suite. Loud `Err` on any shape drift —
/// misaligned per-question records would otherwise fabricate accuracy.
pub fn compose(
    cases: &[SuiteCase],
    modelless: &ModellessQuestions,
    laya: &LayaQuestions,
) -> Result<CascadeReport, String> {
    if modelless.picks.len() != cases.len() || modelless.abstained.len() != cases.len() {
        return Err(format!(
            "modelless record shape drift: {} cases, picks {} / abstained {} (expected {})",
            cases.len(),
            modelless.picks.len(),
            modelless.abstained.len(),
            cases.len(),
        ));
    }
    if laya.picks.len() != laya.served_orig.len() {
        return Err(format!(
            "escalator record shape drift: {} served case indices vs {} pick vectors",
            laya.served_orig.len(),
            laya.picks.len(),
        ));
    }
    for (ci, case) in cases.iter().enumerate() {
        if modelless.picks[ci].len() != case.questions.len()
            || modelless.abstained[ci].len() != case.questions.len()
        {
            return Err(format!(
                "modelless record shape drift at case {ci} ({}): picks {} / abstained {} \
                 vs {} questions",
                case.id,
                modelless.picks[ci].len(),
                modelless.abstained[ci].len(),
                case.questions.len(),
            ));
        }
    }
    for (j, &orig) in laya.served_orig.iter().enumerate() {
        let Some(case) = cases.get(orig) else {
            return Err(format!(
                "escalator served_orig[{j}] = {orig} is out of range ({} cases)",
                cases.len(),
            ));
        };
        if laya.picks[j].len() != case.questions.len() {
            return Err(format!(
                "escalator record shape drift at served case {j} ({}): {} picks vs {} questions",
                case.id,
                laya.picks[j].len(),
                case.questions.len(),
            ));
        }
    }

    // case index → served position (the escalation lookup).
    let mut served_pos: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::with_capacity(laya.served_orig.len());
    for (j, &orig) in laya.served_orig.iter().enumerate() {
        served_pos.insert(orig, j);
    }

    let mut n_questions = 0usize;
    let mut n_answered = 0usize;
    let mut n_escalated = 0usize;
    let mut n_un_escalated = 0usize;
    let mut n_correct = 0usize;
    let mut escalated_laya_correct = 0usize;
    let mut escalated_modelless_correct = 0usize;
    let mut escalator_served_questions = 0usize;

    for (ci, case) in cases.iter().enumerate() {
        let laya_j = served_pos.get(&ci);
        if laya_j.is_some() {
            escalator_served_questions += case.questions.len();
        }
        for (qi, gold) in case.gold.iter().enumerate() {
            n_questions += 1;
            if !modelless.abstained[ci][qi] {
                n_answered += 1;
                if modelless.picks[ci][qi] == gold.idx {
                    n_correct += 1;
                }
                continue;
            }
            match laya_j {
                Some(&j) => {
                    n_escalated += 1;
                    if laya.picks[j][qi] == gold.idx {
                        n_correct += 1;
                        escalated_laya_correct += 1;
                    }
                    if modelless.picks[ci][qi] == gold.idx {
                        escalated_modelless_correct += 1;
                    }
                }
                None => {
                    n_un_escalated += 1;
                    if modelless.picks[ci][qi] == gold.idx {
                        n_correct += 1;
                    }
                }
            }
        }
    }

    if n_questions == 0 {
        return Err("empty case set — nothing to compose".to_string());
    }

    Ok(CascadeReport {
        n_questions,
        n_answered,
        n_escalated,
        n_un_escalated,
        escalation_rate: n_escalated as f64 / n_questions as f64,
        accuracy: n_correct as f64 / n_questions as f64,
        n_correct,
        escalated_laya_correct,
        escalated_modelless_correct,
        escalator_served_questions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{GoldAnswer, QKind, SuiteQuestion};

    fn question(qid: &str) -> SuiteQuestion {
        SuiteQuestion {
            qid: qid.to_string(),
            kind: QKind::Choice,
            instructions: "test".to_string(),
            criteria: serde_json::json!({"a": "opt a", "b": "opt b"}),
        }
    }

    fn case(id: &str, n_questions: usize) -> SuiteCase {
        SuiteCase {
            id: id.to_string(),
            state: serde_json::json!({"text": id}),
            questions: (0..n_questions).map(|qi| question(&format!("{id}-q{qi}"))).collect(),
            gold: (0..n_questions)
                .map(|qi| GoldAnswer {
                    idx: qi % 2, // deterministic gold: 0, 1, 0, 1, …
                    soft: vec![],
                    gold_score: None,
                })
                .collect(),
        }
    }

    fn mq(picks: Vec<Vec<usize>>, abstained: Vec<Vec<bool>>) -> ModellessQuestions {
        ModellessQuestions { picks, abstained }
    }

    fn lq(served_orig: Vec<usize>, picks: Vec<Vec<usize>>) -> LayaQuestions {
        LayaQuestions { served_orig, picks }
    }

    #[test]
    fn compose_basic_math_and_decomposition() {
        // 2 cases × 2 questions. Gold per case: [0, 1].
        let cases = vec![case("c0", 2), case("c1", 2)];
        // Modelless: case 0 answers both (correct, wrong); case 1 abstains q0,
        // answers q1 (wrong).
        let m = mq(
            vec![vec![0, 0], vec![0, 1]], // picks (case 1 q1: pick 1 vs gold 1 → correct? gold[1]=1 → correct)
            vec![vec![false, false], vec![true, false]],
        );
        // Laya serves BOTH cases; on the escalated question (c1 q0, gold 0)
        // laya picks 1 → wrong.
        let l = lq(vec![0, 1], vec![vec![0, 1], vec![1, 1]]);
        let rep = compose(&cases, &m, &l).expect("compose");
        assert_eq!(rep.n_questions, 4);
        assert_eq!(rep.n_answered, 3);
        assert_eq!(rep.n_escalated, 1);
        assert_eq!(rep.n_un_escalated, 0);
        assert!((rep.escalation_rate - 0.25).abs() < 1e-12);
        // correct: c0q0 (0==0 ✓), c0q1 (0 vs 1 ✗), c1q1 (1==1 ✓), escalated
        // c1q0 laya 1 vs 0 ✗ → 2/4.
        assert_eq!(rep.n_correct, 2);
        assert!((rep.accuracy - 0.5).abs() < 1e-12);
        assert_eq!(rep.escalated_laya_correct, 0);
        // the forced modelless pick on c1q0 was 0 == gold → would have scored.
        assert_eq!(rep.escalated_modelless_correct, 1);
        assert_eq!(rep.escalator_served_questions, 4);
        // decomposition: answered + escalated + un_escalated == total.
        assert_eq!(rep.n_answered + rep.n_escalated + rep.n_un_escalated, rep.n_questions);
    }

    #[test]
    fn compose_bucket_skip_makes_un_escalated() {
        let cases = vec![case("c0", 2), case("c1", 2), case("c2", 2)];
        // every case abstains everything.
        let m = mq(
            vec![vec![0, 0], vec![0, 0], vec![0, 0]],
            vec![vec![true, true], vec![true, true], vec![true, true]],
        );
        // laya served ONLY case 0 (bucket-skipped case 1, trimmed case 2).
        let l = lq(vec![0], vec![vec![0, 1]]);
        let rep = compose(&cases, &m, &l).expect("compose");
        assert_eq!(rep.n_questions, 6);
        assert_eq!(rep.n_escalated, 2);
        assert_eq!(rep.n_un_escalated, 4);
        assert!((rep.escalation_rate - 2.0 / 6.0).abs() < 1e-12);
        assert_eq!(rep.escalator_served_questions, 2);
        // correct: c0q0 laya 0 ✓, c0q1 laya 1 ✓, the rest forced modelless
        // pick 0 vs gold 0/1 → q0s of c1/c2 ✓ (2), q1s ✗ → total 4/6.
        assert_eq!(rep.n_correct, 4);
    }

    #[test]
    fn compose_no_abstain_is_pure_modelless() {
        let cases = vec![case("c0", 2), case("c1", 2)];
        let m = mq(vec![vec![0, 1], vec![1, 0]], vec![vec![false, false], vec![false, false]]);
        let l = lq(vec![0, 1], vec![vec![1, 1], vec![1, 1]]);
        let rep = compose(&cases, &m, &l).expect("compose");
        assert_eq!(rep.n_escalated, 0);
        assert!((rep.escalation_rate).abs() < 1e-12);
        // cascade accuracy == the forced modelless accuracy (laya never read).
        // gold per case [0, 1]: c0 ✓✓, c1 ✗✗ → 2/4.
        assert!((rep.accuracy - 0.5).abs() < 1e-12);
    }

    #[test]
    fn compose_all_abstain_all_escalated_reads_laya_accuracy() {
        let cases = vec![case("c0", 2)];
        let m = mq(vec![vec![0, 0]], vec![vec![true, true]]);
        let l = lq(vec![0], vec![vec![1, 0]]);
        let rep = compose(&cases, &m, &l).expect("compose");
        assert_eq!(rep.n_escalated, 2);
        assert!((rep.escalation_rate - 1.0).abs() < 1e-12);
        // gold [0, 1], laya [1, 0] → 0 correct: accuracy IS laya's here.
        assert!((rep.accuracy).abs() < 1e-12);
    }

    #[test]
    fn compose_shape_drift_refuses_loud() {
        let cases = vec![case("c0", 2)];
        // missing abstained vector for the case.
        let m = mq(vec![vec![0, 0]], vec![]);
        let l = lq(vec![0], vec![vec![0, 0]]);
        assert!(compose(&cases, &m, &l).is_err());
        // laya served_orig out of range.
        let m = mq(vec![vec![0, 0]], vec![vec![false, false]]);
        let l = lq(vec![7], vec![vec![0, 0]]);
        assert!(compose(&cases, &m, &l).is_err());
        // laya pick vector shorter than the case's questions.
        let l = lq(vec![0], vec![vec![0]]);
        assert!(compose(&cases, &m, &l).is_err());
        // empty case set.
        assert!(compose(&[], &mq(vec![], vec![]), &lq(vec![], vec![])).is_err());
    }

    #[test]
    fn compose_escalation_beats_and_loses_modelless_both_honest() {
        // The gate is DATA, not a direction: the same composition must read
        // the delta the escalator actually causes, either sign.
        let cases = vec![case("c0", 2), case("c1", 2)];
        // modelless abstains c0q0 (pick 0 == gold) and c1q0 (pick 1 ≠ gold 0).
        let m = mq(
            vec![vec![0, 1], vec![1, 1]],
            vec![vec![true, false], vec![true, false]],
        );
        // laya: right where modelless was right (c0q0 → 0), wrong where it
        // was wrong (c1q0 → 1): cascade == modelless forced exactly.
        let l = lq(vec![0, 1], vec![vec![0, 0], vec![1, 0]]);
        let rep = compose(&cases, &m, &l).expect("compose");
        assert_eq!(rep.escalated_laya_correct, 1);
        assert_eq!(rep.escalated_modelless_correct, 1);
        // now the escalator flips BOTH: cascade +2 where modelless lost one
        // and won one → accuracy strictly better.
        let l2 = lq(vec![0, 1], vec![vec![0, 0], vec![0, 0]]);
        let rep2 = compose(&cases, &m, &l2).expect("compose");
        assert!(rep2.accuracy > rep.accuracy);
        assert_eq!(rep2.escalated_laya_correct, 2);
        assert_eq!(rep2.escalated_modelless_correct, 1);
    }
}
