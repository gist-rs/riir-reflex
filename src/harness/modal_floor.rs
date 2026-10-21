//! Issue 094 T1 — the input-keyed modal-answer floor (pure, ungated).
//!
//! The arXiv:2610.17495 measurement discipline's anti-mimicry control,
//! adapted to the harness: a lookup baseline that predicts, for every
//! question, the MODAL gold answer among the population's OTHER questions
//! sharing its input key (leave-one-out — the case's own gold never votes
//! on itself). Any lane whose accuracy sits at/below this floor carries no
//! input-independent signal on that suite — the paper's law: what cannot
//! beat the census is mimicry, not sensing.
//!
//! Determinism law (katgpt-rs Issue 940): the pick is quoted in the tables,
//! so the census pins its construction — integer counts only (no float
//! fold), suite-order iteration, and ONE fixed tie rule (smallest gold
//! index wins). Two runs over the same suite are byte-identical.
//!
//! Report-only (issue 094 T5): no exit-code coupling, no serving-path edit.

use serde::Serialize;
use std::collections::BTreeMap;

use super::suites::{QKind, SuiteCase};

/// Bucket width in normalized content tokens (issue 094: "normalized-input
/// bucket"). Four tokens: fine enough that plain-text suites (classification
/// families) key on real content; structured states (typed_decisions /
/// xnli JSON objects) spend the first token on the constant field name, so
/// their effective width is three content tokens.
const BUCKET_TOKENS: usize = 4;

/// The census record (results.json, additive). The disclosure fields exist
/// so a DEGENERATE floor is visible rather than load-bearing: when the key
/// pools collapse (few keys, high global-fallback count, `global_modal_share`
/// ≈ the floor itself), the floor has degraded to the plain majority class
/// and the tables say so.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModalFloorRow {
    /// Floor accuracy: the fraction of questions whose leave-one-out
    /// modal-lookup pick equals gold.
    pub acc: f64,
    pub n_questions: usize,
    /// Distinct input keys (bucket × qid × option-count).
    pub n_keys: usize,
    /// Questions whose key pool (excluding self) was empty — the global
    /// modal served them. High counts = over-fine keying, disclosed.
    pub n_global_fallback: usize,
    /// The global modal answer's population share — the degenerate-floor
    /// tell: a floor AT this share is just the majority class.
    pub global_modal_share: f64,
    /// The fixed construction, quoted beside the number (provenance law).
    pub key_rule: &'static str,
}

const KEY_RULE: &str = "leave-one-out modal census; key = first-4 normalized \
                        input tokens × qid × option-count; ties → first index";

/// The meta-level definition line (RunMeta.modal_floor_definition) — the
/// WHY beside the conformal floor's definition in every results.json.
pub const MODAL_FLOOR_DEFINITION: &str = "input-keyed modal-answer floor = leave-one-out \
    modal gold over questions sharing the input key (first-4 normalized input \
    tokens × qid × option-count; ties → first index; arXiv:2610.17495's \
    anti-mimicry census) — a lane at/below it carries no input-independent \
    signal (issue 094)";

/// The normalized input bucket: first `BUCKET_TOKENS` alphanumeric tokens
/// of the serialized state, lowercased, space-joined. Splitting on
/// non-alphanumeric makes JSON/object states key on content after the
/// constant field names.
fn input_bucket(state_str: &str) -> String {
    let mut out = String::new();
    for w in state_str
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .take(BUCKET_TOKENS)
    {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&w.to_lowercase());
    }
    out
}

/// The gold-space option count for one question (size only — noul's
/// engine-space order differs, its count does not).
fn option_count(kind: QKind, criteria: &serde_json::Value) -> usize {
    match kind {
        QKind::Noul => 2,
        QKind::Choice => criteria.as_object().map_or(0, serde_json::Map::len),
        QKind::Score => criteria.as_array().map_or(0, Vec::len),
    }
}

/// One question's flat identity: the lookup key and its gold.
struct QuestionKey {
    key: String,
    gold: usize,
}

fn question_keys(cases: &[SuiteCase]) -> Vec<QuestionKey> {
    let mut out = Vec::new();
    for case in cases {
        let bucket = input_bucket(&crate::pyjson::serialize_state(&case.state));
        for (q, g) in case.questions.iter().zip(case.gold.iter()) {
            let n = option_count(q.kind, &q.criteria);
            out.push(QuestionKey {
                key: format!("{bucket}\u{1}{}\u{1}{n}", q.qid),
                gold: g.idx,
            });
        }
    }
    out
}

/// Modal pick over a count map (ties → smallest gold index). `exclude` is
/// the caller's own gold (leave-one-out): its single vote is withheld.
fn modal_pick(counts: &BTreeMap<usize, u32>, exclude: usize) -> Option<usize> {
    let mut best: Option<(u32, usize)> = None;
    for (&idx, &n) in counts {
        let n = if idx == exclude { n - 1 } else { n };
        if n == 0 {
            continue;
        }
        match best {
            Some((bn, _)) if bn >= n => {}
            _ => best = Some((n, idx)),
        }
    }
    best.map(|(_, idx)| idx)
}

/// The census over a suite's question population. `None` only when the
/// suite carries no questions (an honest absence, never a fabricated row).
#[must_use]
pub fn census(cases: &[SuiteCase]) -> Option<ModalFloorRow> {
    let keys = question_keys(cases);
    let n = keys.len();
    if n == 0 {
        return None;
    }

    // Pass 1: totals per key and per suite (integer counts, suite order).
    let mut per_key: BTreeMap<&str, BTreeMap<usize, u32>> = BTreeMap::new();
    let mut total: BTreeMap<usize, u32> = BTreeMap::new();
    for k in &keys {
        *per_key
            .entry(k.key.as_str())
            .or_default()
            .entry(k.gold)
            .or_insert(0) += 1;
        *total.entry(k.gold).or_insert(0) += 1;
    }

    // Pass 2: the leave-one-out lookup, question by question. The fallback
    // count is the questions whose key pool had NO vote left after the
    // self-exclusion (a singleton key) — the global modal served them.
    let mut correct = 0usize;
    let mut n_global_fallback = 0usize;
    for k in &keys {
        let pool_pick = modal_pick(&per_key[k.key.as_str()], k.gold);
        let pick = pool_pick.or_else(|| modal_pick(&total, k.gold));
        if pick == Some(k.gold) {
            correct += 1;
        }
        if pool_pick.is_none() {
            n_global_fallback += 1;
        }
    }

    let global_modal_share = f64::from(total.values().copied().max().unwrap_or(0)) / n as f64;

    Some(ModalFloorRow {
        acc: correct as f64 / n as f64,
        n_questions: n,
        n_keys: per_key.len(),
        n_global_fallback,
        global_modal_share,
        key_rule: KEY_RULE,
    })
}

/// The tables block for one suite: the floor line + one ⛔ line per lane at
/// or below the floor (issue 094 T1: "a lane at/below the floor carries no
/// internal signal and the table says so"). `entries` = (lane, model,
/// forced accuracy) for every lane row the suite's table prints.
#[must_use]
pub fn render_block(entries: &[(&str, &str, f64)], floor: &ModalFloorRow) -> String {
    let mut s = format!(
        "**modal floor (issue 094):** acc `{:.4}` over {} questions — {} \
         ({} keys, {} global fallbacks, global-modal share `{:.4}`)\n\n",
        floor.acc,
        floor.n_questions,
        floor.key_rule,
        floor.n_keys,
        floor.n_global_fallback,
        floor.global_modal_share,
    );
    let mut flagged = 0usize;
    for &(lane, model, acc) in entries {
        if acc <= floor.acc {
            flagged += 1;
            s.push_str(&format!(
                "- ⛔ `{lane} · {model}` acc `{acc:.4}` ≤ floor — no input-independent \
                 signal on this suite\n"
            ));
        }
    }
    if flagged > 0 {
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{GoldAnswer, SuiteQuestion};
    use serde_json::json;

    fn choice_q(qid: &str, options: &[&str]) -> SuiteQuestion {
        SuiteQuestion {
            qid: qid.to_string(),
            kind: QKind::Choice,
            instructions: "pick".to_string(),
            criteria: {
                let mut m = serde_json::Map::new();
                for o in options {
                    m.insert(
                        (*o).to_string(),
                        serde_json::Value::String((*o).to_string()),
                    );
                }
                serde_json::Value::Object(m)
            },
        }
    }

    fn case(id: &str, state: &str, gold: usize) -> SuiteCase {
        SuiteCase {
            id: id.to_string(),
            state: json!(state),
            questions: vec![choice_q("sentiment", &["pos", "neg", "neu"])],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    #[test]
    fn bucket_pool_beats_global_and_singleton_falls_back() {
        // Three cases share the 4-token bucket "card charged twice today"
        // (the 5th token varies); one singleton bucket falls back to the
        // global modal (self-vote withheld).
        let cases = vec![
            case("a", "card charged twice today morning", 0),
            case("b", "card charged twice today evening", 0),
            case("c", "card charged twice today badly", 1),
            case("d", "completely different text here", 1),
        ];
        let f = census(&cases).expect("row");
        assert_eq!(f.n_questions, 4);
        assert_eq!(f.n_keys, 2);
        // a: pool {0:1, 1:1} → tie → first index 0 == gold ✓. b: same
        // pool → 0 == gold ✓. c: pool {0:2} after self-exclusion → 0 ≠ 1 ✗.
        // d: singleton → global modal (others: 0,0,1 → 0) ≠ 1 ✗.
        assert!((f.acc - 0.5).abs() < 1e-12);
        assert_eq!(f.n_global_fallback, 1);
        assert!((f.global_modal_share - 0.5).abs() < 1e-12);
    }

    #[test]
    fn tie_breaks_to_the_first_index() {
        // Pool with a 1–1 tie after self-exclusion: smallest gold index
        // wins — and here that pick is wrong for every gold-holder, so the
        // census reads 0. The companion test below flips the golds to show
        // the same rule reading correct.
        let cases = vec![
            case("a", "same opening words here now", 1),
            case("b", "same opening words here too", 2),
            case("c", "same opening words here also", 2),
            case("d", "other words entirely x", 0),
        ];
        let f = census(&cases).expect("row");
        // a: pool {2:2} after self-exclusion → 2 ≠ 1 ✗. b: pool {1:1, 2:1}
        // → tie → 1 ≠ 2 ✗. c: same → 1 ≠ 2 ✗. d: singleton → global modal
        // {1:1, 2:2} → 2 ≠ 0 ✗.
        assert!((f.acc - 0.0).abs() < 1e-12);
    }

    #[test]
    fn tie_first_index_can_be_correct() {
        // Same tie shape, first-index holders: the fixed rule reads 2/4.
        let cases = vec![
            case("a", "same opening words here now", 1),
            case("b", "same opening words here too", 2),
            case("c", "same opening words here also", 1),
            case("d", "other words entirely x", 0),
        ];
        let f = census(&cases).expect("row");
        // a: pool {1:1, 2:1} → tie → 1 == 1 ✓. b: pool {1:2} → 1 ≠ 2 ✗.
        // c: pool {1:1, 2:1} → tie → 1 ✓. d: singleton → {1:2} ≠ 0 ✗.
        assert!((f.acc - 0.5).abs() < 1e-12);
    }

    #[test]
    fn census_is_deterministic_across_runs() {
        let cases = vec![
            case("a", "card charged twice today morning", 0),
            case("b", "card charged twice today evening", 0),
            case("c", "card charged twice today badly", 1),
            case("d", "completely different text here", 1),
        ];
        assert_eq!(census(&cases), census(&cases));
    }

    #[test]
    fn empty_suite_is_none() {
        assert!(census(&[]).is_none());
        assert!(
            census(&[SuiteCase {
                id: "e".into(),
                state: json!("x"),
                questions: vec![],
                gold: vec![],
            }])
            .is_none()
        );
    }

    #[test]
    fn structured_state_keys_past_the_field_name() {
        // A JSON-object state (typed_decisions shape): the constant field
        // name eats one bucket token; content keys the rest.
        let obj = |txt: &str| json!({"premise": txt, "hypothesis": "x"});
        let mk = |id: &str, txt: &str, gold: usize| SuiteCase {
            id: id.to_string(),
            state: obj(txt),
            questions: vec![choice_q("entail", &["yes", "no", "maybe"])],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![],
                gold_score: None,
            }],
        };
        let cases = vec![
            mk("a", "a man inspects the roof", 0),
            mk("b", "a man inspects the fence", 0),
            mk("c", "the dog sleeps soundly", 1),
        ];
        let f = census(&cases).expect("row");
        // a and b share the bucket ("premise a man inspects" — 4 tokens);
        // c keys alone.
        assert_eq!(f.n_keys, 2);
        assert!((f.acc - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn noul_option_count_is_two_without_criteria() {
        let q = SuiteQuestion {
            qid: "injection".into(),
            kind: QKind::Noul,
            instructions: "injection?".into(),
            criteria: serde_json::Value::Null,
        };
        assert_eq!(option_count(QKind::Noul, &q.criteria), 2);
        assert_eq!(option_count(QKind::Score, &json!([1, 2, 3, 4, 5])), 5);
    }

    #[test]
    fn render_block_flags_only_at_or_below_the_floor() {
        let floor = ModalFloorRow {
            acc: 0.5,
            n_questions: 4,
            n_keys: 2,
            n_global_fallback: 1,
            global_modal_share: 0.5,
            key_rule: KEY_RULE,
        };
        let entries = [
            ("modelless", "modelless", 0.51),
            ("laya", "english", 0.5),
            ("laya", "typed", 0.25),
        ];
        let s = render_block(&entries, &floor);
        assert!(!s.contains("modelless · modelless"));
        assert!(s.contains("laya · english"));
        assert!(s.contains("laya · typed"));
        assert!(s.contains("no input-independent signal"));
    }
}
