//! Issue 094 T1 — the input-keyed modal-answer floor (pure, ungated).
//!
//! The arXiv:2610.17495 measurement discipline's anti-mimicry control,
//! adapted to the harness: a lookup baseline that predicts, for every
//! eval question, the MODAL gold among a reference population's questions
//! sharing its input key — then maps that winner back through the case's
//! own option order. Any lane whose accuracy sits at/below this floor
//! carries no input-independent signal on that suite — the paper's law:
//! what cannot beat the census is mimicry, not sensing.
//!
//! Two construction laws, both learned from the first board read (bench
//! 156, 2026-10-22) and load-bearing:
//!
//! 1. **Identity pooling, never position pooling.** Suites shuffle option
//!    order per case (massive_intent_en, typed_decisions rows): a gold
//!    INDEX means a different label in every case. The census pools the
//!    criteria KEY at each gold position (the option's canonical identity)
//!    and predicts among the queried case's own options — so per-case
//!    shuffles and sampled distractor subsets are handled by construction.
//! 2. **Reference-side census, not LOO.** A leave-one-out census over the
//!    eval population is ANTI-correlated in small pools (a balanced
//!    bucket's neighbors-modal deterministically excludes the queried
//!    label — measured: exactly-0.0000 floors on xnli_en). The census side
//!    is the CAL slice (test∩cal disjointness is slice-guard-asserted);
//!    the LOO form survives only for suites with no cal slice, disclosed
//!    as `reference` on the row.
//!
//! Determinism law (katgpt-rs Issue 940): the pick is quoted in the tables,
//! so the census pins its construction — integer counts only (no float
//! fold), suite-order iteration, and ONE fixed tie rule (pool count, then
//! global count, then the case's own option order). Two runs over the same
//! suite are byte-identical.
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

const REFERENCE_CAL: &str = "cal-split census (test∩cal disjoint)";
const REFERENCE_LOO: &str = "leave-one-out eval census (no cal slice — small-pool \
                             exclusion bias disclosed)";

/// The census record (results.json, additive). The disclosure fields exist
/// so a DEGENERATE floor is visible rather than load-bearing: when the key
/// pools collapse (`n_fallback` high) or the reference modal dominates
/// (`global_modal_share` ≈ the floor itself), the floor has degraded toward
/// the plain majority class and the tables say so.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModalFloorRow {
    /// Floor accuracy: the fraction of eval questions whose modal-lookup
    /// pick equals gold.
    pub acc: f64,
    pub n_questions: usize,
    /// Distinct input keys (bucket × qid × option-count).
    pub n_keys: usize,
    /// Questions whose key pool carried no mass on any of the case's own
    /// options — the global reference modal served them.
    pub n_fallback: usize,
    /// The reference population's modal answer share — the degenerate-floor
    /// tell: a floor AT this share is just the majority class.
    pub global_modal_share: f64,
    /// Which population the census was computed over (the two laws above).
    pub reference: &'static str,
    /// The fixed construction, quoted beside the number (provenance law).
    pub key_rule: &'static str,
}

const KEY_RULE: &str = "modal census pooled by option identity; key = first-4 \
                        normalized input tokens × qid × option-count; ties → global \
                        prior → the case's own option order";

/// The meta-level definition line (RunMeta.modal_floor_definition) — the
/// WHY beside the conformal floor's definition in every results.json.
pub const MODAL_FLOOR_DEFINITION: &str = "input-keyed modal-answer floor = the cal-split \
    (reference) modal gold over questions sharing the input key, pooled by option \
    identity and mapped through each case's own option order (first-4 normalized input \
    tokens × qid × option-count; ties → global prior → case order; arXiv:2610.17495's \
    anti-mimicry census; LOO over eval only when no cal slice exists) — a lane at/below \
    it carries no input-independent signal (issue 094)";

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

/// The case's ordered option identities: the criteria KEY at each position
/// for choice (the canonical label — stable across per-case shuffles), the
/// position itself for score/noul (level/index order is fixed by contract).
fn option_identities(q: &super::suites::SuiteQuestion) -> Vec<String> {
    match q.kind {
        QKind::Noul | QKind::Score => {
            let n = match q.kind {
                QKind::Noul => 2,
                QKind::Score => q.criteria.as_array().map_or(0, Vec::len),
                QKind::Choice => unreachable!("matched above"),
            };
            (0..n).map(|i| i.to_string()).collect()
        }
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
    }
}

/// One eval question's lookup shape.
#[derive(Clone)]
struct QuestionEntry {
    key: String,
    /// The canonical identity of the gold option.
    own: Option<String>,
    /// The case's ordered option identities (the pick maps back through
    /// this).
    options: Vec<String>,
    gold_idx: usize,
}

fn question_entries(cases: &[SuiteCase]) -> Vec<QuestionEntry> {
    let mut out = Vec::new();
    for case in cases {
        let bucket = input_bucket(&crate::pyjson::serialize_state(&case.state));
        for (q, g) in case.questions.iter().zip(case.gold.iter()) {
            let options = option_identities(q);
            let n = option_identities_len(q, &options);
            let own = options.get(g.idx).cloned();
            out.push(QuestionEntry {
                key: format!("{bucket}\u{1}{}\u{1}{n}", q.qid),
                own,
                options,
                gold_idx: g.idx,
            });
        }
    }
    out
}

/// The option-count half of the lookup key (size only — noul's engine-space
/// order differs, its count does not).
fn option_identities_len(q: &super::suites::SuiteQuestion, options: &[String]) -> usize {
    match q.kind {
        QKind::Noul => 2,
        QKind::Score => q.criteria.as_array().map_or(0, Vec::len),
        QKind::Choice => options.len(),
    }
}

/// The global identity → count census (also each key pool's value type).
type IdentityCounts<'a> = BTreeMap<&'a str, u32>;
/// The key → identity pools (bucket × qid × option-count → label census).
type KeyPools<'a> = BTreeMap<&'a str, IdentityCounts<'a>>;

/// (key → identity → count) over one population, plus the global census.
fn accumulate(entries: &[QuestionEntry]) -> (KeyPools<'_>, IdentityCounts<'_>) {
    let mut per_key: KeyPools<'_> = BTreeMap::new();
    let mut global: IdentityCounts<'_> = BTreeMap::new();
    for e in entries {
        if let Some(own) = &e.own {
            *per_key
                .entry(e.key.as_str())
                .or_default()
                .entry(own.as_str())
                .or_insert(0) += 1;
            *global.entry(own.as_str()).or_insert(0) += 1;
        }
    }
    (per_key, global)
}

/// The pick for one eval question: rank the case's OWN options by pool
/// count (desc), then global count (desc), then case order; when the pool
/// carries no mass on any of them, rank by global count alone. `withhold`
/// is LOO mode (census == eval population): the caller's own gold identity
/// vote is withdrawn from BOTH counts. Reference mode passes false — the
/// populations are disjoint, so the eval question's vote was never in
/// those counts and subtracting one would penalize its gold identity.
/// Returns (pick_idx, used_fallback).
fn pick(
    e: &QuestionEntry,
    pool: &IdentityCounts<'_>,
    global: &IdentityCounts<'_>,
    withhold: bool,
) -> (usize, bool) {
    let adj = |counts: &IdentityCounts<'_>, id: &str| -> u32 {
        let c = counts.get(id).copied().unwrap_or(0);
        if withhold && Some(id) == e.own.as_deref() {
            c.saturating_sub(1)
        } else {
            c
        }
    };
    // Phase A: any own option carrying pool mass?
    let pool_has_mass = e.options.iter().any(|id| adj(pool, id) > 0);
    let mut best: Option<(u32, u32, usize)> = None;
    for (pos, id) in e.options.iter().enumerate() {
        let pc = if pool_has_mass { adj(pool, id) } else { 0 };
        let gc = adj(global, id);
        let rank = (pc, gc);
        let better = match best {
            None => true,
            Some((bpc, bgc, _)) => rank > (bpc, bgc),
        };
        if better {
            best = Some((pc, gc, pos));
        }
    }
    let pos = best.map_or(0, |(_, _, p)| p);
    (pos, !pool_has_mass)
}

/// The census over a suite's eval questions, with the census side taken
/// from `reference` when it carries questions (the cal slice — the
/// disjoint-population law), else leave-one-out over the eval population
/// itself (the disclosed degraded form). `None` only when the suite
/// carries no questions (an honest absence, never a fabricated row).
#[must_use]
pub fn census(eval: &[SuiteCase], reference: Option<&[SuiteCase]>) -> Option<ModalFloorRow> {
    let entries = question_entries(eval);
    let n = entries.len();
    if n == 0 {
        return None;
    }
    let loo = reference.is_none_or(|r| r.is_empty());
    let census_entries: Vec<QuestionEntry> = if loo {
        entries.clone()
    } else {
        question_entries(reference.unwrap_or(&[]))
    };
    let (per_key, global) = accumulate(&census_entries);

    let mut correct = 0usize;
    let mut n_fallback = 0usize;
    for e in &entries {
        let empty_pool: BTreeMap<&str, u32> = BTreeMap::new();
        let pool = per_key.get(e.key.as_str()).unwrap_or(&empty_pool);
        let (pick_idx, fell_back) = pick(e, pool, &global, loo);
        if pick_idx == e.gold_idx {
            correct += 1;
        }
        if fell_back {
            n_fallback += 1;
        }
    }

    let census_n = census_entries.len();
    let global_modal_share =
        f64::from(global.values().copied().max().unwrap_or(0)) / census_n.max(1) as f64;

    Some(ModalFloorRow {
        acc: correct as f64 / n as f64,
        n_questions: n,
        n_keys: per_key.len(),
        n_fallback,
        global_modal_share,
        reference: if loo { REFERENCE_LOO } else { REFERENCE_CAL },
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
         ({} keys, {} fallbacks, reference-modal share `{:.4}`)\n\n",
        floor.acc,
        floor.n_questions,
        floor.key_rule,
        floor.n_keys,
        floor.n_fallback,
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

    /// A choice case with an EXPLICIT option order (the per-case-shuffle
    /// shape: gold idx indexes THIS order).
    fn ordered_case(
        id: &str,
        state: &str,
        options: &[&str],
        gold_idx: usize,
        qid: &str,
    ) -> SuiteCase {
        SuiteCase {
            id: id.to_string(),
            state: json!(state),
            questions: vec![choice_q(qid, options)],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    fn case(id: &str, state: &str, gold: usize) -> SuiteCase {
        ordered_case(id, state, &["pos", "neg", "neu"], gold, "sentiment")
    }

    #[test]
    fn reference_pool_predicts_and_unseen_key_falls_back() {
        // Reference: bucket "card charged twice today" carries pos×2, the
        // fallback bucket nothing. Eval: two same-bucket questions + one
        // unseen-key question (global modal serves it).
        let reference = vec![
            case("r0", "card charged twice today morning", 0),
            case("r1", "card charged twice today evening", 0),
            case("r2", "totally other words here", 1),
        ];
        let eval = vec![
            case("a", "card charged twice today noon", 0),
            case("b", "card charged twice today later", 1),
            case("c", "never seen this key before", 0),
        ];
        let f = census(&eval, Some(&reference)).expect("row");
        assert_eq!(f.reference, REFERENCE_CAL);
        assert_eq!(f.n_questions, 3);
        // a: pool pos=2 > neg=0 → pos ✓. b: pool pos=2 → pos ≠ neg ✗. c:
        // unseen key → global modal pos (2 vs 1) ✓.
        assert!((f.acc - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(f.n_fallback, 1);
        assert!((f.global_modal_share - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn identity_pooling_maps_through_per_case_orders() {
        // The massive/typed shape: reference + eval cases shuffle their
        // option orders. The census pools the option IDENTITY ("neu" won
        // the reference bucket — from TWO different positions) and maps it
        // through the eval case's own order — a position-pooled census
        // would read idx 1 as a different label and miss.
        let reference = vec![
            ordered_case(
                "r0",
                "same opening words here one",
                &["pos", "neg", "neu"],
                2,
                "sentiment",
            ),
            ordered_case(
                "r1",
                "same opening words here two",
                &["neg", "neu", "pos"],
                1,
                "sentiment",
            ),
        ];
        // Eval case presents the shuffled order [neg, neu, pos]: the modal
        // identity "neu" sits at idx 1 — the pick must be 1, not the
        // position the identity held in either reference case.
        let eval = vec![ordered_case(
            "a",
            "same opening words here three",
            &["neg", "neu", "pos"],
            1,
            "sentiment",
        )];
        let f = census(&eval, Some(&reference)).expect("row");
        assert!((f.acc - 1.0).abs() < 1e-12, "identity-pooled pick: {f:?}");
        assert_eq!(f.n_fallback, 0, "the key pool engaged");
    }

    #[test]
    fn ties_break_toward_the_global_prior_then_case_order() {
        // Pool tie (pos 1 / neu 1 in the shared bucket): the GLOBAL prior
        // (pos, 3) breaks it; a global tie would fall to case order.
        let reference = vec![
            ordered_case(
                "r0",
                "same opening words here one",
                &["pos", "neg", "neu"],
                0,
                "sentiment",
            ),
            ordered_case(
                "r1",
                "same opening words here two",
                &["pos", "neg", "neu"],
                2,
                "sentiment",
            ),
            // Global-only mass: a second key carrying pos twice.
            ordered_case(
                "r2",
                "other words entirely same one",
                &["pos", "neg", "neu"],
                0,
                "sentiment",
            ),
            ordered_case(
                "r3",
                "other words entirely same two",
                &["pos", "neg", "neu"],
                0,
                "sentiment",
            ),
        ];
        // Eval: key "same opening words here" pool {pos:1, neu:1} tie →
        // global pos 3 > neu 1 → the pick is pos (idx 0), the gold.
        let eval = vec![ordered_case(
            "a",
            "same opening words here three",
            &["pos", "neg", "neu"],
            0,
            "sentiment",
        )];
        let f = census(&eval, Some(&reference)).expect("row");
        assert!((f.acc - 1.0).abs() < 1e-12, "global-prior tie: {f:?}");
        assert_eq!(f.n_keys, 2);
    }

    #[test]
    fn options_absent_from_the_reference_rank_by_case_order() {
        // A sampled-distractor case (the massive shape): the eval case's
        // options are mostly absent from the reference census — zero-count
        // ties resolve to the case's own option order, deterministically.
        let reference = vec![ordered_case(
            "r0",
            "same opening words here one",
            &["alpha", "beta", "gamma"],
            0,
            "intent",
        )];
        let eval = vec![ordered_case(
            "a",
            "same opening words here two",
            &["delta", "alpha", "epsilon"],
            2,
            "intent",
        )];
        let f = census(&eval, Some(&reference)).expect("row");
        // Pool: alpha=1 (the only option with mass) → pick alpha (idx 1) ≠
        // gold (epsilon) — the honest read, not a crash.
        assert!((f.acc - 0.0).abs() < 1e-12);
        assert_eq!(f.n_fallback, 0, "pool had mass on alpha");
    }

    #[test]
    fn loo_mode_withholds_the_own_vote_and_is_disclosed() {
        // No cal slice → the LOO form: the eval population IS the census,
        // with each question's own vote withheld from both counts.
        let eval = vec![
            case("a", "same opening words here one", 0),
            case("b", "same opening words here two", 0),
            case("c", "same opening words here three", 1),
        ];
        let f = census(&eval, None).expect("row");
        assert_eq!(f.reference, REFERENCE_LOO);
        // a: pool {pos:2, neg:1} − self(pos) → 1 v 1 → global pos 2−1=1 v
        // neg 1 → tie → case order → pos ✓. b: same shape → pos ✓. c:
        // pool {pos:2, neg:0} → pos ≠ neg ✗.
        assert!((f.acc - 2.0 / 3.0).abs() < 1e-12, "loo picks: {f:?}");
        assert_eq!(f.n_fallback, 0);
    }

    #[test]
    fn census_is_deterministic_across_runs() {
        let reference = vec![
            case("r0", "card charged twice today morning", 0),
            case("r1", "card charged twice today evening", 0),
        ];
        let eval = vec![
            case("a", "card charged twice today noon", 0),
            case("b", "card charged twice today later", 1),
            case("c", "never seen this key before", 0),
        ];
        assert_eq!(
            census(&eval, Some(&reference)),
            census(&eval, Some(&reference))
        );
    }

    #[test]
    fn empty_suite_is_none() {
        assert!(census(&[], None).is_none());
        assert!(census(&[], Some(&[case("r", "x", 0)])).is_none());
        assert!(
            census(
                &[SuiteCase {
                    id: "e".into(),
                    state: json!("x"),
                    questions: vec![],
                    gold: vec![],
                }],
                None
            )
            .is_none()
        );
    }

    #[test]
    fn noul_and_score_identity_is_the_fixed_index() {
        let q = SuiteQuestion {
            qid: "injection".into(),
            kind: QKind::Noul,
            instructions: "injection?".into(),
            criteria: serde_json::Value::Null,
        };
        assert_eq!(
            option_identities(&q),
            vec!["0".to_string(), "1".to_string()]
        );
        let sq = SuiteQuestion {
            qid: "rate".into(),
            kind: QKind::Score,
            instructions: "rate".into(),
            criteria: json!(["very bad", "bad", "ok", "good", "very good"]),
        };
        assert_eq!(option_identities(&sq).len(), 5);
        assert_eq!(option_identities(&sq)[3], "3");
    }

    #[test]
    fn render_block_flags_only_at_or_below_the_floor() {
        let floor = ModalFloorRow {
            acc: 0.5,
            n_questions: 4,
            n_keys: 2,
            n_fallback: 1,
            global_modal_share: 0.5,
            reference: REFERENCE_CAL,
            key_rule: KEY_RULE,
        };
        let entries = [
            ("modelless", "modelless", 0.51),
            ("laya", "english", 0.5),
            ("laya", "typed", 0.25),
        ];
        let s = render_block(&entries, &floor);
        assert!(!s.contains("`modelless · modelless`"));
        assert!(s.contains("laya · english"));
        assert!(s.contains("laya · typed"));
        assert!(s.contains("no input-independent signal"));
    }
}
