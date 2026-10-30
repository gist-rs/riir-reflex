//! riir-train Issue 657 R8 — the label-PERMUTATION accuracy probe (the pure core).
//!
//! A seeded BIJECTIVE RENAMING of a suite's option-label strings: σ is a
//! Sattolo derangement over the suite's option-key alphabet (every label
//! moves), option ORDER is preserved, and the criteria KEYS are replaced in
//! place. The gold rides for free: `GoldAnswer.idx` is positional, so the
//! renamed gold STRING is exactly `σ(labels[gold.idx])` with the index
//! unchanged — "options and gold follow the same bijection" by construction.
//!
//! NOT Bench 119 (formatting-flip): Bench 119's axis is formatting churn of
//! the STATE text (case sensitivity, whitespace); this probe's axis is the
//! label IDENTITY itself — which string names which class. The two measure
//! different presentation subspaces and share no machinery.
//!
//! Everything here is PURE: the derangement, the σ application, the
//! single-question case builder, and the corpus relabel half. The lanes live
//! in [`crate::harness::runner::label_perm`] behind the
//! [`crate::harness::permutation::ChoiceOracle`] seam — each real lane
//! answers through its own production decide path, so the probe measures
//! what ships, never a parallel rendering.
//!
//! Measurement laws:
//! - **Sattolo, not Fisher–Yates.** A plain shuffle can leave labels fixed
//!   (an unchanged option measures nothing for itself); Sattolo's `j < i`
//!   variant is a cyclic derangement BY CONSTRUCTION — every label moves,
//!   every slot's key actually changes.
//! - **Order preserved, content replaced.** This is the complement of the
//!   `--perm-probe` family: there the LABELS are the invariant and the
//!   positions move; here the POSITIONS are the invariant and the labels
//!   move. A case's option list never reorders.
//! - **Deterministic.** σ(seed) is SplitMix64-seeded; the per-suite seed
//!   base is `LABEL_PERM_SEED ^ FNV-1a(suite_name)` and the probe runs
//!   base, base+1, base+2 — the same maps every run, byte-reproducible.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::suites::{QKind, SplitMix64, SuiteCase, SuiteQuestion, TrainDoc};

/// The probe's seed constant (any fixed value; reproducibility is what
/// matters, not the value — the MASSIVE seed law). XORed with FNV-1a of the
/// suite name so different suites see different derangements.
pub const LABEL_PERM_SEED: u64 = 0x0657_0001;

/// Classic FNV-1a over a byte string (the same family `slot_seed` uses for
/// its per-slot mixing, at the standard offset basis).
#[must_use]
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The per-suite seed base: the probe seed mixed with the suite name —
/// orderings differ across suites, stay stable across runs.
#[must_use]
pub fn suite_seed_base(suite: &str) -> u64 {
    LABEL_PERM_SEED ^ fnv1a(suite.as_bytes())
}

/// Sattolo's in-place shuffle: for i from n−1 down to 1, swap `xs[i]` with
/// `xs[j]` where `j = rng.below(i)` (STRICTLY below i — the one-line
/// difference from the Fisher–Yates [`super::suites::shuffle`]). Produces a
/// cyclic derangement BY CONSTRUCTION: for len ≥ 2 every element ends up in
/// a different position.
pub fn sattolo<T>(xs: &mut [T], rng: &mut SplitMix64) {
    for i in (1..xs.len()).rev() {
        let j = rng.below(i);
        xs.swap(i, j);
    }
}

/// σ: the seeded bijection over `alphabet` — `alphabet[i] ↦ moved[i]` where
/// `moved` is a Sattolo derangement of a copy of the alphabet. Every label
/// moves (len ≥ 2); the map is injective by construction (a permutation of
/// the same set), so renamed option keys can never collide.
#[must_use]
pub fn label_bijection(alphabet: &[String], seed: u64) -> BTreeMap<String, String> {
    let mut moved: Vec<String> = alphabet.to_vec();
    let mut rng = SplitMix64::new(seed);
    sattolo(&mut moved, &mut rng);
    alphabet.iter().cloned().zip(moved).collect()
}

/// Rename one question's option labels through `sigma`. Choice questions
/// rebuild their criteria map with keys replaced IN PLACE (insertion order —
/// the presentation order — is preserved; serde_json `preserve_order`, the
/// same rendering fact [`super::permutation::permute_choice`] relies on) and
/// values (the option descriptions) untouched. Score criteria is an ordered
/// level array (index = level) and noul's `[false, true]` order is semantic —
/// neither is a label space, both pass through unchanged (the perm-probe's
/// choice-only law, mirrored).
///
/// A key outside σ (impossible for suite keys — the alphabet IS the suite's
/// option-key union) is kept verbatim rather than dropped; the injectivity
/// of σ means the rebuilt keys can never collide.
#[must_use]
pub fn rename_question(q: &SuiteQuestion, sigma: &BTreeMap<String, String>) -> SuiteQuestion {
    let criteria = match (&q.kind, q.criteria.as_object()) {
        (QKind::Choice, Some(obj)) => {
            let mut m = Map::new();
            for (k, v) in obj {
                let nk = sigma.get(k).cloned().unwrap_or_else(|| k.clone());
                m.insert(nk, v.clone());
            }
            Value::Object(m)
        }
        _ => q.criteria.clone(),
    };
    SuiteQuestion {
        qid: q.qid.clone(),
        kind: q.kind,
        instructions: q.instructions.clone(),
        criteria,
    }
}

/// Rename every choice question of `case` through `sigma`. The case id,
/// state, and gold are untouched: the gold is positional, so it follows the
/// bijection automatically (the label at `gold.idx` is renamed in place).
#[must_use]
pub fn rename_case(case: &SuiteCase, sigma: &BTreeMap<String, String>) -> SuiteCase {
    SuiteCase {
        id: case.id.clone(),
        state: case.state.clone(),
        questions: case
            .questions
            .iter()
            .map(|q| rename_question(q, sigma))
            .collect(),
        gold: case.gold.clone(),
    }
}

/// The single-question case a lane answers for one slot — the same shape
/// [`super::permutation::permuted_case`] materializes (the oracle contract
/// reads the FIRST answer). The id gains a `#<qid>` suffix; the runner uses
/// the SAME id on both arms so the paired join keys line up.
///
/// # Errors
/// When `qi` is out of range for the questions or the parallel gold list.
pub fn single_question_case(case: &SuiteCase, qi: usize) -> Result<SuiteCase, String> {
    let q = case
        .questions
        .get(qi)
        .ok_or_else(|| format!("case {}: question index {qi} out of range", case.id))?;
    let gold = case
        .gold
        .get(qi)
        .ok_or_else(|| format!("case {}: gold index {qi} out of range", case.id))?;
    Ok(SuiteCase {
        id: format!("{}#{}", case.id, q.qid),
        state: case.state.clone(),
        questions: vec![q.clone()],
        gold: vec![gold.clone()],
    })
}

/// Map the engine domain labels through σ (element-wise; a label outside σ
/// is kept verbatim — over the probe's suites the alphabet IS the label
/// universe, so this arm never fires in practice).
#[must_use]
pub fn relabel_labels(labels: &[String], sigma: &BTreeMap<String, String>) -> Vec<String> {
    labels
        .iter()
        .map(|l| sigma.get(l).cloned().unwrap_or_else(|| l.clone()))
        .collect()
}

/// Relabel the corpus docs through σ. Returns the renamed docs and the count
/// of docs whose label was OUTSIDE σ (kept verbatim — disclosed by the
/// caller, never silently mixed; the full-consistent arm expects 0).
pub fn relabel_train_docs(
    docs: &[TrainDoc],
    sigma: &BTreeMap<String, String>,
) -> (Vec<TrainDoc>, usize) {
    let mut untouched = 0usize;
    let out = docs
        .iter()
        .map(|d| match sigma.get(&d.label) {
            Some(nl) => TrainDoc {
                label: nl.clone(),
                text: d.text.clone(),
            },
            None => {
                untouched += 1;
                TrainDoc {
                    label: d.label.clone(),
                    text: d.text.clone(),
                }
            }
        })
        .collect();
    (out, untouched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{GoldAnswer, QKind, SuiteQuestion};

    fn alphabet(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("lbl{i}")).collect()
    }

    fn choice_question(keys: &[String], with_descs: bool) -> SuiteQuestion {
        let mut m = Map::new();
        for (i, k) in keys.iter().enumerate() {
            let v = if with_descs {
                Value::String(format!("desc{i}"))
            } else {
                Value::Null
            };
            m.insert(k.clone(), v);
        }
        SuiteQuestion {
            qid: "q".to_string(),
            kind: QKind::Choice,
            instructions: "pick".to_string(),
            criteria: Value::Object(m),
        }
    }

    fn choice_case(id: &str, keys: &[String], gold_idx: usize) -> SuiteCase {
        SuiteCase {
            id: id.to_string(),
            state: serde_json::Value::String("state".to_string()),
            questions: vec![choice_question(keys, false)],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    #[test]
    fn sattolo_deranges_every_element() {
        for n in 2..=9 {
            for seed in 0..6u64 {
                let mut xs: Vec<usize> = (0..n).collect();
                let mut rng = SplitMix64::new(seed);
                sattolo(&mut xs, &mut rng);
                // Every element moved (the Sattolo guarantee for len >= 2)…
                for (pos, &v) in xs.iter().enumerate() {
                    assert_ne!(v, pos, "n={n} seed={seed}: element {v} stayed fixed");
                }
                // …and the result is still a permutation (bijective).
                let mut sorted = xs.clone();
                sorted.sort_unstable();
                let identity: Vec<usize> = (0..n).collect();
                assert_eq!(sorted, identity);
            }
        }
    }

    #[test]
    fn label_bijection_is_deterministic_and_seed_sensitive() {
        let a = alphabet(6);
        let s = suite_seed_base("ag_news");
        let m1 = label_bijection(&a, s);
        let m2 = label_bijection(&a, s);
        assert_eq!(m1, m2, "same seed must reproduce the same map");
        let m3 = label_bijection(&a, s + 1);
        assert_ne!(m1, m3, "seed+1 must move the map (deterministic inputs)");
        // Domain and codomain are the same set (a bijection onto itself)…
        let mut vals: Vec<&String> = m1.values().collect();
        vals.sort();
        let mut keys: Vec<&String> = m1.keys().collect();
        keys.sort();
        assert_eq!(vals, keys);
        // …and no label is fixed (the derangement law).
        for (k, v) in &m1 {
            assert_ne!(k, v, "label {k} must move");
        }
    }

    #[test]
    fn identity_sigma_preserves_every_position() {
        let keys = alphabet(4);
        let case = choice_case("c0", &keys, 2);
        let id_map: BTreeMap<String, String> =
            keys.iter().cloned().zip(keys.iter().cloned()).collect();
        let renamed = rename_case(&case, &id_map);
        assert_eq!(renamed, case);
        // The biting half: position-wise keys are IDENTICAL under the
        // identity map (serde_json Map equality is order-insensitive, so the
        // Value assert above alone would not prove order preservation).
        let orig_keys: Vec<&String> = case.questions[0]
            .criteria
            .as_object()
            .expect("object")
            .keys()
            .collect();
        let ren_keys: Vec<&String> = renamed.questions[0]
            .criteria
            .as_object()
            .expect("renamed object")
            .keys()
            .collect();
        assert_eq!(orig_keys, ren_keys);
    }

    #[test]
    fn gold_follows_the_bijection_by_position() {
        let keys = alphabet(5);
        let case = choice_case("c1", &keys, 3);
        let sigma = label_bijection(&keys, suite_seed_base("emotion"));
        let renamed = rename_case(&case, &sigma);
        // The gold INDEX is untouched (position is the invariant)…
        assert_eq!(renamed.gold[0].idx, 3);
        // …so the renamed gold STRING is exactly σ(original gold string).
        let original_gold = &keys[3];
        assert_eq!(
            renamed.questions[0]
                .criteria
                .as_object()
                .map(|m| m.keys().nth(3).cloned().unwrap())
                .expect("renamed criteria"),
            sigma[original_gold]
        );
    }

    #[test]
    fn order_and_descriptions_are_preserved() {
        let keys = alphabet(4);
        let mut case = choice_case("c2", &keys, 0);
        case.questions[0] = choice_question(&keys, true);
        let sigma = label_bijection(&keys, suite_seed_base("xnli_en"));
        let renamed = rename_case(&case, &sigma);
        let orig_obj = case.questions[0].criteria.as_object().expect("object");
        let ren_obj = renamed.questions[0]
            .criteria
            .as_object()
            .expect("renamed object");
        let orig_keys: Vec<&String> = orig_obj.keys().collect();
        let ren_keys: Vec<&String> = ren_obj.keys().collect();
        assert_eq!(orig_keys.len(), ren_keys.len(), "order-count preserved");
        for (pos, k) in orig_keys.iter().enumerate() {
            assert_eq!(
                sigma[*k], *ren_keys[pos],
                "position {pos}: renamed key must be σ(original key)"
            );
            assert_eq!(
                orig_obj[*k], ren_obj[ren_keys[pos]],
                "position {pos}: the description value must ride along untouched"
            );
        }
    }

    #[test]
    fn score_and_noul_questions_pass_through() {
        let score = SuiteQuestion {
            qid: "s".to_string(),
            kind: QKind::Score,
            instructions: "rate".to_string(),
            criteria: serde_json::json!(["low", "mid", "high"]),
        };
        let mut case = SuiteCase {
            id: "c3".to_string(),
            state: serde_json::Value::String("state".to_string()),
            questions: vec![score],
            gold: vec![GoldAnswer {
                idx: 1,
                soft: vec![],
                gold_score: Some(2.0),
            }],
        };
        let sigma = label_bijection(&alphabet(3), 42);
        let renamed = rename_case(&case, &sigma);
        assert_eq!(renamed.questions[0], case.questions[0]);
        // A choice sibling in the same case IS renamed: σ has no fixed
        // points, so EVERY position's key changes. (serde_json Map equality
        // is order-insensitive and σ maps the alphabet onto itself, so a
        // Value-level assert_ne! would be vacuous here — the assertion must
        // be position-wise.)
        let keys = alphabet(3);
        case.questions.push(choice_question(&keys, false));
        let renamed2 = rename_case(&case, &sigma);
        assert_eq!(renamed2.questions[0], case.questions[0], "score untouched");
        let orig_keys: Vec<&String> = case.questions[1]
            .criteria
            .as_object()
            .expect("object")
            .keys()
            .collect();
        let ren_keys: Vec<&String> = renamed2.questions[1]
            .criteria
            .as_object()
            .expect("renamed object")
            .keys()
            .collect();
        for (pos, k) in orig_keys.iter().enumerate() {
            assert_ne!(*k, ren_keys[pos], "position {pos}: every label must move");
            assert_eq!(
                &sigma[*k], ren_keys[pos],
                "position {pos}: renamed = σ(original)"
            );
        }
    }

    #[test]
    fn single_question_case_carries_state_and_gold() {
        let keys = alphabet(3);
        let case = choice_case("c4", &keys, 1);
        let sub = single_question_case(&case, 0).expect("in range");
        assert_eq!(sub.id, "c4#q");
        assert_eq!(sub.questions.len(), 1);
        assert_eq!(sub.gold[0].idx, 1);
        assert!(single_question_case(&case, 5).is_err());
    }

    #[test]
    fn relabel_train_docs_maps_and_counts_foreign_labels() {
        let keys = alphabet(3);
        let sigma = label_bijection(&keys, 7);
        let docs = vec![
            TrainDoc {
                label: keys[0].clone(),
                text: "d0".to_string(),
            },
            TrainDoc {
                label: keys[2].clone(),
                text: "d2".to_string(),
            },
            TrainDoc {
                label: "foreign".to_string(),
                text: "df".to_string(),
            },
        ];
        let (renamed, untouched) = relabel_train_docs(&docs, &sigma);
        assert_eq!(
            untouched, 1,
            "the foreign-label doc is counted, never dropped"
        );
        assert_eq!(renamed.len(), 3);
        assert_eq!(renamed[0].label, sigma[&keys[0]]);
        assert_eq!(renamed[1].label, sigma[&keys[2]]);
        assert_eq!(renamed[2].label, "foreign", "foreign doc kept verbatim");
        assert_eq!(renamed[0].text, "d0", "doc TEXT is never touched by σ");
    }

    #[test]
    fn suite_seed_base_is_stable_and_suite_separated() {
        let a = suite_seed_base("banking77");
        assert_eq!(a, suite_seed_base("banking77"));
        assert_ne!(a, suite_seed_base("ag_news"));
        assert_eq!(a, LABEL_PERM_SEED ^ fnv1a(b"banking77"));
    }
}
