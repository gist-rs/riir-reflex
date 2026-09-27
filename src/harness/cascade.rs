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
//! Issue 042 lever 3 — per-suite escalation-worthiness (opt-in): the
//! escalation decision is DIRECTION-blind by construction (Bench 061:
//! banking77 lost 14.0 pt on escalation while xnli gained 11.7 — the gate
//! cannot tell). The worthiness probe reads the direction from the CAL
//! slice, where selection is protocol-legal: the escalator answers the
//! cal questions the calibrated gate abstained on, and the cascade stays
//! ARMED only where the escalator reads ≥ the forced modelless picks on
//! that probe set (delta ≥ `min_delta`, default 0.0). A negative probe
//! disarms the suite's escalation — every abstain then stands as the
//! modelless forced pick, disclosed as `disarmed` in the row. No cal
//! records / thin support (< [`PROBE_MIN_QUESTIONS`]) stays armed with a
//! named `unprobed_reason` — the probe never invents a disarm it cannot
//! measure (the pre-lever T4′ behavior holds where it cannot read).
//!
//! Noul/score questions need no special case: both lanes' picks are already
//! label-space (`[no, yes]` for noul, level index for score — the harness's
//! own conversion law), and in this harness every question has gold, so an
//! abstain is never itself a correct answer here.

use serde::Serialize;

use super::suites::SuiteCase;

/// Minimum probe support to ARM/DISARM on the measured delta. Below it the
/// verdict is `unprobed` (armed + named reason) — the same thin-support
/// law the engine's own threshold fit carries (THIN_SUPPORT_FLOOR == 16).
pub const PROBE_MIN_QUESTIONS: usize = 16;

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
    /// Issue 042 lever 3: the same calibrated gate over the suite's CAL
    /// slice — the worthiness probe's modelless half. `None` when the
    /// lever is off or the suite has no cal slice (the probe then reports
    /// unprobed and the pre-lever behavior holds).
    pub cal_gate: Option<CalGateRecords>,
}

/// The worthiness probe's modelless half: the deployed calibrated gate
/// (picks + abstain flags) over the CAL case list, index-aligned with it.
#[derive(Debug, Clone)]
pub struct CalGateRecords {
    pub picks: Vec<Vec<usize>>,
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
    /// Issue 042 lever 3: the worthiness verdict when the lever is on.
    /// `None` = lever off — byte-identical with the landed T4′ lane.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worthiness: Option<WorthinessVerdict>,
}

/// The cal-slice worthiness verdict (issue 042 lever 3), serialized into
/// the cascade row when `--cascade-worthiness` is on.
#[derive(Debug, Clone, Serialize)]
pub struct WorthinessVerdict {
    /// True = the suite's escalation is disabled (every abstain stands as
    /// the modelless forced pick; `n_escalated` stays 0).
    pub disarmed: bool,
    /// Probe-set size: cal questions the gate abstained on AND the
    /// escalator served.
    pub probe_n: usize,
    pub probe_laya_acc: f64,
    pub probe_modelless_acc: f64,
    /// `probe_laya_acc − probe_modelless_acc`.
    pub delta: f64,
    /// The configured arm bar (arm iff `delta ≥ min_delta`).
    pub min_delta: f64,
    /// Why the probe could not read (no cal records / no served cal
    /// cases / thin support). Present = the pre-lever behavior (armed)
    /// held, named — never a silent fallthrough.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unprobed_reason: Option<String>,
}

/// Everything the worthiness probe reads, per (suite, checkpoint). All
/// halves are cal-slice data (protocol-legal selection) — never test.
pub struct WorthinessInput<'a> {
    /// The deployed calibrated gate over the cal slice (modelless half).
    pub gate: Option<&'a CalGateRecords>,
    /// The checkpoint's answers over the served cal subset.
    pub escalator: Option<&'a LayaQuestions>,
    /// The suite's cal case list (gold + question counts). Index space
    /// for both halves.
    pub cal_cases: &'a [SuiteCase],
    /// Arm bar: the probe disarms when `delta < min_delta`.
    pub min_delta: f64,
}

impl WorthinessInput<'_> {
    /// Compute the verdict. Loud `Err` on any shape drift — a misaligned
    /// cal record would fabricate the delta exactly like a misaligned
    /// test record would fabricate accuracy.
    fn probe(&self) -> Result<WorthinessVerdict, String> {
        let unprobed = |reason: String, n, la, ma| WorthinessVerdict {
            disarmed: false,
            probe_n: n,
            probe_laya_acc: la,
            probe_modelless_acc: ma,
            delta: if n > 0 { la - ma } else { 0.0 },
            min_delta: self.min_delta,
            unprobed_reason: Some(reason),
        };
        let Some(gate) = self.gate else {
            return Ok(unprobed(
                "modelless cal-gate records absent (no cal slice or probe half off)".to_string(),
                0,
                0.0,
                0.0,
            ));
        };
        let Some(esc) = self.escalator else {
            return Ok(unprobed(
                "escalator cal records absent (cal serving failed or was not requested)"
                    .to_string(),
                0,
                0.0,
                0.0,
            ));
        };
        if gate.picks.len() != self.cal_cases.len() || gate.abstained.len() != self.cal_cases.len()
        {
            return Err(format!(
                "worthiness gate shape drift: {} cal cases, picks {} / abstained {}",
                self.cal_cases.len(),
                gate.picks.len(),
                gate.abstained.len()
            ));
        }
        for (ci, case) in self.cal_cases.iter().enumerate() {
            if gate.picks[ci].len() != case.questions.len()
                || gate.abstained[ci].len() != case.questions.len()
            {
                return Err(format!(
                    "worthiness gate shape drift at cal case {ci} ({}): picks {} / abstained {} \
                     vs {} questions",
                    case.id,
                    gate.picks[ci].len(),
                    gate.abstained[ci].len(),
                    case.questions.len()
                ));
            }
        }
        if esc.picks.len() != esc.served_orig.len() {
            return Err(format!(
                "worthiness escalator shape drift: {} served cal indices vs {} pick vectors",
                esc.served_orig.len(),
                esc.picks.len()
            ));
        }
        for (j, &orig) in esc.served_orig.iter().enumerate() {
            let Some(case) = self.cal_cases.get(orig) else {
                return Err(format!(
                    "worthiness escalator served_orig[{j}] = {orig} is out of range ({} cal cases)",
                    self.cal_cases.len()
                ));
            };
            if esc.picks[j].len() != case.questions.len() {
                return Err(format!(
                    "worthiness escalator shape drift at served cal case {j} ({}): {} picks \
                     vs {} questions",
                    case.id,
                    esc.picks[j].len(),
                    case.questions.len()
                ));
            }
        }

        let (mut laya_ok, mut ml_ok, mut probe_n) = (0usize, 0usize, 0usize);
        for (j, &ci) in esc.served_orig.iter().enumerate() {
            let case = &self.cal_cases[ci];
            for (qi, gold) in case.gold.iter().enumerate() {
                if !gate.abstained[ci][qi] {
                    continue;
                }
                probe_n += 1;
                if esc.picks[j][qi] == gold.idx {
                    laya_ok += 1;
                }
                if gate.picks[ci][qi] == gold.idx {
                    ml_ok += 1;
                }
            }
        }
        if probe_n == 0 {
            return Ok(unprobed(
                "empty probe set: the gate abstained on no served cal question".to_string(),
                0,
                0.0,
                0.0,
            ));
        }
        let laya_acc = laya_ok as f64 / probe_n as f64;
        let ml_acc = ml_ok as f64 / probe_n as f64;
        let delta = laya_acc - ml_acc;
        if probe_n < PROBE_MIN_QUESTIONS {
            return Ok(unprobed(
                format!(
                    "thin support: {probe_n} probe questions < {PROBE_MIN_QUESTIONS} — \
                     armed, pre-lever behavior holds"
                ),
                probe_n,
                laya_acc,
                ml_acc,
            ));
        }
        Ok(WorthinessVerdict {
            disarmed: delta < self.min_delta,
            probe_n,
            probe_laya_acc: laya_acc,
            probe_modelless_acc: ml_acc,
            delta,
            min_delta: self.min_delta,
            unprobed_reason: None,
        })
    }
}

/// Compose the cascade over one suite. Loud `Err` on any shape drift —
/// misaligned per-question records would otherwise fabricate accuracy.
/// `worthiness` (issue 042 lever 3, opt-in) gates the escalation per
/// suite: a disarming verdict turns every abstain into the modelless
/// forced pick, with the verdict carried on the report.
pub fn compose(
    cases: &[SuiteCase],
    modelless: &ModellessQuestions,
    laya: &LayaQuestions,
    worthiness: Option<&WorthinessInput>,
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

    let worthiness_verdict = match worthiness {
        Some(w) => Some(w.probe()?),
        None => None,
    };
    // Issue 042 lever 3: a disarmed suite keeps the modelless forced pick
    // on every abstain — the escalation path below is skipped entirely.
    let escalation_armed = worthiness_verdict
        .as_ref()
        .is_none_or(|v| !v.disarmed);

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
                Some(&j) if escalation_armed => {
                    n_escalated += 1;
                    if laya.picks[j][qi] == gold.idx {
                        n_correct += 1;
                        escalated_laya_correct += 1;
                    }
                    if modelless.picks[ci][qi] == gold.idx {
                        escalated_modelless_correct += 1;
                    }
                }
                // No escalator coverage (or disarmed by worthiness): the
                // modelless forced pick stands.
                _ => {
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
        worthiness: worthiness_verdict,
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
        ModellessQuestions {
            picks,
            abstained,
            cal_gate: None,
        }
    }

    fn mq_with_cal(
        picks: Vec<Vec<usize>>,
        abstained: Vec<Vec<bool>>,
        cal_gate: Option<CalGateRecords>,
    ) -> ModellessQuestions {
        ModellessQuestions {
            picks,
            abstained,
            cal_gate,
        }
    }

    fn worthiness_input<'a>(
        gate: Option<&'a CalGateRecords>,
        escalator: Option<&'a LayaQuestions>,
        cal_cases: &'a [SuiteCase],
        min_delta: f64,
    ) -> WorthinessInput<'a> {
        WorthinessInput {
            gate,
            escalator,
            cal_cases,
            min_delta,
        }
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
        let rep = compose(&cases, &m, &l, None).expect("compose");
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
        let rep = compose(&cases, &m, &l, None).expect("compose");
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
        let rep = compose(&cases, &m, &l, None).expect("compose");
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
        let rep = compose(&cases, &m, &l, None).expect("compose");
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
        assert!(compose(&cases, &m, &l, None).is_err());
        // laya served_orig out of range.
        let m = mq(vec![vec![0, 0]], vec![vec![false, false]]);
        let l = lq(vec![7], vec![vec![0, 0]]);
        assert!(compose(&cases, &m, &l, None).is_err());
        // laya pick vector shorter than the case's questions.
        let l = lq(vec![0], vec![vec![0]]);
        assert!(compose(&cases, &m, &l, None).is_err());
        // empty case set.
        assert!(compose(&[], &mq(vec![], vec![]), &lq(vec![], vec![]), None).is_err());
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
        let rep = compose(&cases, &m, &l, None).expect("compose");
        assert_eq!(rep.escalated_laya_correct, 1);
        assert_eq!(rep.escalated_modelless_correct, 1);
        // now the escalator flips BOTH: cascade +2 where modelless lost one
        // and won one → accuracy strictly better.
        let l2 = lq(vec![0, 1], vec![vec![0, 0], vec![0, 0]]);
        let rep2 = compose(&cases, &m, &l2, None).expect("compose");
        assert!(rep2.accuracy > rep.accuracy);
        assert_eq!(rep2.escalated_laya_correct, 2);
        assert_eq!(rep2.escalated_modelless_correct, 1);
    }

    // ── Issue 042 lever 3: the worthiness probe ──

    /// Ten cal cases × 2 questions (gold [0, 1] per case); the gate
    /// abstains ALL cal questions — a 20-question probe set, above the
    /// thin-support floor so the arm/disarm tests measure the delta, not
    /// the floor. The test cases abstain c0q0 and c1q0 — the escalation
    /// the verdict gates.
    fn worthiness_fixture() -> (Vec<SuiteCase>, Vec<SuiteCase>, ModellessQuestions) {
        let cal_cases: Vec<SuiteCase> = (0..10).map(|k| case(&format!("cal{k}"), 2)).collect();
        let test_cases = vec![case("c0", 2), case("c1", 2)];
        // Modelless forced cal picks: even cases [0,0] (1/2), odd [0,1]
        // (2/2) → 15/20 = 0.75 over the full-abstain probe set.
        let cal_gate_picks: Vec<Vec<usize>> = (0..10)
            .map(|k| if k % 2 == 0 { vec![0, 0] } else { vec![0, 1] })
            .collect();
        let m = mq_with_cal(
            vec![vec![0, 0], vec![0, 1]],
            vec![vec![true, false], vec![true, false]],
            Some(CalGateRecords {
                picks: cal_gate_picks,
                abstained: vec![vec![true, true]; 10],
            }),
        );
        (cal_cases, test_cases, m)
    }

    #[test]
    fn worthiness_disarms_when_escalator_reads_negative_on_cal() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        // Escalator: 1/2 per cal case (10/20 = 0.5) vs modelless 0.75.
        // Delta −0.25 < 0 → disarm.
        let cal_laya = lq(
            (0..10).collect(),
            (0..10).map(|_| vec![1, 1]).collect(),
        );
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let w = worthiness_input(m.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.0);
        let rep = compose(&test_cases, &m, &test_laya, Some(&w)).expect("compose");
        let v = rep.worthiness.as_ref().expect("verdict present");
        assert!(v.disarmed);
        assert_eq!(v.probe_n, 20);
        assert!((v.probe_laya_acc - 0.5).abs() < 1e-12);
        assert!((v.probe_modelless_acc - 0.75).abs() < 1e-12);
        assert!((v.delta - (-0.25)).abs() < 1e-12);
        // Disarmed: no escalation at all — both abstains (c0q0, c1q0)
        // stand as the forced picks.
        assert_eq!(rep.n_escalated, 0);
        assert_eq!(rep.n_un_escalated, 2);
        assert_eq!(rep.n_answered + rep.n_escalated + rep.n_un_escalated, rep.n_questions);
        // accuracy == the modelless forced accuracy over the same set
        // (c0q0 ✓, c0q1 pick 0 vs gold 1 ✗, c1q0 forced 0 vs gold 0 ✓,
        // c1q1 pick 1 vs gold 1 ✓ → 3/4).
        assert!((rep.accuracy - 0.75).abs() < 1e-12);
    }

    #[test]
    fn worthiness_arms_when_escalator_reads_positive_on_cal() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        // Escalator right everywhere on cal (20/20 = 1.0 vs modelless
        // 0.75): delta +0.25 ≥ 0 → armed; the test-side escalation happens.
        let cal_laya = lq(
            (0..10).collect(),
            (0..10).map(|_| vec![0, 1]).collect(),
        );
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let w = worthiness_input(m.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.0);
        let rep = compose(&test_cases, &m, &test_laya, Some(&w)).expect("compose");
        let v = rep.worthiness.as_ref().expect("verdict present");
        assert!(!v.disarmed);
        assert!(v.unprobed_reason.is_none());
        assert_eq!(rep.n_escalated, 2);
        // armed rows escalate exactly as the pre-lever compose would.
        assert_eq!(rep.escalated_laya_correct, 1);
        assert_eq!(rep.escalated_modelless_correct, 2);
    }

    #[test]
    fn worthiness_delta_at_margin_is_armed() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        // delta exactly +0.25 against min_delta +0.25: >= arms.
        let cal_laya = lq(
            (0..10).collect(),
            (0..10).map(|_| vec![0, 1]).collect(),
        );
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let w = worthiness_input(m.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.25);
        let rep = compose(&test_cases, &m, &test_laya, Some(&w)).expect("compose");
        let v = rep.worthiness.as_ref().expect("verdict present");
        assert!(!v.disarmed);
        assert!(v.unprobed_reason.is_none());
        // one tick below the margin disarms.
        let w2 = worthiness_input(m.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.250001);
        let rep2 = compose(&test_cases, &m, &test_laya, Some(&w2)).expect("compose");
        assert!(rep2.worthiness.as_ref().expect("verdict").disarmed);
        assert_eq!(rep2.n_escalated, 0);
    }

    #[test]
    fn worthiness_unprobed_halves_stay_armed_with_reason() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        // gate half absent.
        let w = worthiness_input(None, Some(&test_laya), &cal_cases, 0.0);
        let rep = compose(&test_cases, &m, &test_laya, Some(&w)).expect("compose");
        let v = rep.worthiness.as_ref().expect("verdict");
        assert!(!v.disarmed);
        assert!(v.unprobed_reason.as_deref().unwrap().contains("modelless cal-gate"));
        assert_eq!(rep.n_escalated, 2); // pre-lever behavior: escalation ran
        // escalator half absent.
        let w2 = worthiness_input(m.cal_gate.as_ref(), None, &cal_cases, 0.0);
        let rep2 = compose(&test_cases, &m, &test_laya, Some(&w2)).expect("compose");
        let v2 = rep2.worthiness.as_ref().expect("verdict");
        assert!(!v2.disarmed);
        assert!(v2.unprobed_reason.as_deref().unwrap().contains("escalator cal records"));
        assert_eq!(rep2.n_escalated, 2);
    }

    #[test]
    fn worthiness_thin_support_arms_with_reason() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        // One served cal case → probe_n 2 < 16 → unprobed (armed + reason),
        // even though the measured delta reads parity (laya 1/2 vs
        // modelless 1/2 on cal0) — support, not sign, decides here.
        let cal_laya = lq(vec![0], vec![vec![1, 1]]); // 0/2 vs modelless 2/2 on cal0
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let w = worthiness_input(m.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.0);
        let rep = compose(&test_cases, &m, &test_laya, Some(&w)).expect("compose");
        let v = rep.worthiness.as_ref().expect("verdict");
        assert!(!v.disarmed);
        assert_eq!(v.probe_n, 2);
        assert!(v.unprobed_reason.as_deref().unwrap().contains("thin support"));
        assert_eq!(rep.n_escalated, 2);
    }

    #[test]
    fn worthiness_probe_reads_only_served_and_abstained_cal_questions() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        // cal1..cal9 are never served; cal0 q1 is NOT abstained in a
        // modified gate — the probe set must shrink accordingly.
        let cal_laya = lq(vec![0], vec![vec![0, 1]]);
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let mut gate = m.cal_gate.as_ref().expect("gate").clone();
        gate.abstained[0] = vec![true, false];
        let m2 = mq_with_cal(m.picks.clone(), m.abstained.clone(), Some(gate));
        let w = worthiness_input(m2.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.0);
        let rep = compose(&test_cases, &m2, &test_laya, Some(&w)).expect("compose");
        let v = rep.worthiness.as_ref().expect("verdict");
        // probe = cal0 q0 (abstained + served) only → n 1 < 16 → unprobed,
        // but the COUNT proves the read walked exactly the served ∩
        // abstained set.
        assert_eq!(v.probe_n, 1);
        assert!(v.unprobed_reason.is_some());
    }

    #[test]
    fn worthiness_shape_drift_refuses_loud() {
        let (cal_cases, test_cases, m) = worthiness_fixture();
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let cal_laya = lq(vec![0, 1], vec![vec![0, 1], vec![0, 1]]);
        // gate pick vector shorter than the cal case's questions.
        let bad_gate = CalGateRecords {
            picks: vec![vec![0], vec![0, 1]],
            abstained: vec![vec![true, true], vec![true, true]],
        };
        let m_bad = mq_with_cal(m.picks.clone(), m.abstained.clone(), Some(bad_gate));
        let w = worthiness_input(m_bad.cal_gate.as_ref(), Some(&cal_laya), &cal_cases, 0.0);
        assert!(compose(&test_cases, &m_bad, &test_laya, Some(&w)).is_err());
        // escalator served_orig out of cal range (10 cases → 0..10 valid;
        // 10 is the first out-of-range index).
        let bad_cal_laya = lq(vec![10], vec![vec![0, 1]]);
        let w2 = worthiness_input(m.cal_gate.as_ref(), Some(&bad_cal_laya), &cal_cases, 0.0);
        assert!(compose(&test_cases, &m, &test_laya, Some(&w2)).is_err());
    }

    #[test]
    fn worthiness_off_is_byte_identical_semantics() {
        // Lever off (None): escalation runs everywhere — the landed T4′
        // shape, and no worthiness block on the row.
        let (_cal_cases, test_cases, m) = worthiness_fixture();
        let test_laya = lq(vec![0, 1], vec![vec![0, 0], vec![1, 1]]);
        let rep = compose(&test_cases, &m, &test_laya, None).expect("compose");
        assert!(rep.worthiness.is_none());
        assert_eq!(rep.n_escalated, 2);
    }
}
