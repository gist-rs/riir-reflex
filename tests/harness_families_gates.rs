//! Gates for the remaining harness family — Issue 061's
//! `semantic_defects` code-defect family (modelless, wide-law authored).
//! The six Issue-004 decision-point families were RETIRED 2026-10-02
//! (owner call — home-made synthetic evals the modelless engine reads at
//! chance on at the honest populations); their registry rows, wide evals,
//! gates, and the GOAT bench are gone with them, and the grounded-posture
//! gate died with its suite.
//!
//! What is asserted here:
//! - registry contract: semantic_defects registered synthetic AND modelless;
//! - count floors: eval ≥ 12, cal ≥ 16 (the fused-gate thresholds cannot
//!   fit below 16 cal observations — runner law), corpus ≥ 8, every class
//!   covered in all three slices;
//! - the self-inclusion leak law: corpus/cal/eval text sets pairwise
//!   disjoint (a cal case scoring against itself inflates every
//!   cal-slice quantile);
//! - gold/option agreement: gold indexes in range, Choice criteria match
//!   the label universe;
//! - engine smoke: wire-valid answers, normalized probabilities,
//!   per-request G2 < 1 ms, bit-identical repeats, anti-pathology accuracy
//!   floor (0.8× chance), and the DISCRIMINATION floor — distinct picks
//!   AND distinct probability vectors across the eval slice (the
//!   degenerate-lane lesson: a constant, input-independent pick sits at
//!   exactly chance on class-balanced fixtures, so accuracy alone cannot
//!   see it — only distinct counts can);
//! - Plan 009 REVISED-2 (issue 059): the WIDE-eval authoring law —
//!   population band + exact class balance, the label-token ban, the
//!   corpus/cal 3-gram wall (zero tolerance), unigram-overlap ceilings,
//!   and the BLAKE3 digest pin over the canonical wide-eval bytes.

#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, Question};
use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};
use riir_reflex::harness::families;
use riir_reflex::harness::fixture_hygiene;
use riir_reflex::harness::runner;
use riir_reflex::harness::suites::QKind;
use riir_reflex::pyjson::serialize_state;

use std::collections::HashSet;

const MODELLESS_FAMILIES: &[&str] = &["semantic_defects"];

/// Anti-pathology forced-accuracy floors per family — 0.8× chance. The
/// floors gate DOWNWARD only (they catch a systematically wrong engine —
/// anti-correlated picks — never certify accuracy); accuracy itself is
/// RECORDED, published, never gated. Historical note: before the engine
/// option-rank blend (Issue 004 T7, 2026-09-22) the lane sat at exactly
/// chance everywhere — including a constant pick the accuracy number could
/// not distinguish from honest chance, which is the discrimination test's
/// reason to exist. Post-blend the tables read ag_news 0.510, xnli 0.347,
/// banking77 0.446 (was 0.040 ≈ 1/77) — above chance materially, still
/// published-not-gated.
const ACC_FLOORS: &[(&str, f64)] = &[
    ("semantic_defects", 0.13), // chance ~0.17 (6-way)
];

#[test]
fn families_are_registered_with_the_right_lanes() {
    for name in MODELLESS_FAMILIES {
        let (synthetic, modelless) = runner::suite_lanes(name)
            .unwrap_or_else(|| panic!("{name} missing from the suite registry"));
        assert!(synthetic, "{name} must be an in-process synthetic suite");
        assert!(
            modelless,
            "{name} is modelless by default (Issue 004 / Issue 045)"
        );
    }
}

#[test]
fn modelless_families_meet_count_coverage_and_disjointness_floors() {
    for name in MODELLESS_FAMILIES {
        let def = families::family_def(name).unwrap_or_else(|| panic!("{name} has no FamilyDef"));
        assert!(
            def.eval.len() >= 12,
            "{name}: eval slice {} < 12",
            def.eval.len()
        );
        assert!(
            def.cal.len() >= 16,
            "{name}: cal slice {} < 16 — the fused-gate thresholds cannot fit \
             (runner law: confs.len() < 16 falls back to the birth constants)",
            def.cal.len()
        );
        assert!(
            def.corpus.len() >= 8,
            "{name}: corpus {} < 8 — routing has nothing to route by",
            def.corpus.len()
        );
        for (gi, label) in def.labels.iter().enumerate() {
            assert!(
                def.corpus.iter().any(|(cg, _)| cg == &gi),
                "{name}: class {label} has no corpus doc — the domain would arm \
                 on the self-doc fallback, not the authored corpus"
            );
            let cal_n = def.cal.iter().filter(|t| t.gold == gi).count();
            let eval_n = def.eval.iter().filter(|t| t.gold == gi).count();
            assert!(
                cal_n >= 2,
                "{name}: class {label} has {cal_n} cal cases (< 2)"
            );
            assert!(
                eval_n >= 2,
                "{name}: class {label} has {eval_n} eval cases (< 2)"
            );
        }
        // Gold indexes in range.
        for t in def.cal.iter().chain(def.eval.iter()) {
            assert!(
                t.gold < def.labels.len(),
                "{name}: gold {} out of range for {} options",
                t.gold,
                def.labels.len()
            );
        }
        // Pairwise-disjoint slices (the self-inclusion leak law).
        for (i, (cg, ct)) in def.corpus.iter().enumerate() {
            for b in def.cal.iter().chain(def.eval.iter()) {
                assert_ne!(
                    *ct, b.text,
                    "{name}: corpus text {i} (class {cg}) duplicated in cal/eval \
                     — the self-inclusion leak"
                );
            }
        }
        for (i, a) in def.cal.iter().enumerate() {
            for b in def.eval.iter() {
                assert_ne!(
                    a.text, b.text,
                    "{name}: cal text {i} duplicated in eval — the calibrator \
                     would fit on the test answers"
                );
            }
        }
    }
}

#[test]
fn synthdata_gold_and_options_agree() {
    for name in MODELLESS_FAMILIES {
        let d = families::synth_by_name(name).expect("family synth");
        assert_eq!(d.suite.name, *name);
        for c in &d.suite.cases {
            assert_eq!(c.questions.len(), 1, "{name}: one question per family case");
            let q = &c.questions[0];
            let g = &c.gold[0];
            match q.kind {
                QKind::Choice => {
                    let keys = q
                        .criteria
                        .as_object()
                        .unwrap_or_else(|| panic!("{name}: choice criteria must be an object"));
                    assert!(
                        g.idx < keys.len(),
                        "{name}: gold {} out of option range",
                        g.idx
                    );
                    assert!(
                        g.gold_score.is_none(),
                        "{name}: choice carries no gold_score"
                    );
                }
                QKind::Score => {
                    let levels = q
                        .criteria
                        .as_array()
                        .unwrap_or_else(|| panic!("{name}: score criteria must be an array"));
                    assert!(g.idx < levels.len(), "{name}: gold level out of range");
                    assert_eq!(
                        g.gold_score,
                        Some(g.idx as f64),
                        "{name}: score gold_score must equal the level"
                    );
                }
                QKind::Noul => {
                    assert!(q.criteria.is_null(), "{name}: noul criteria must be null");
                    assert!(
                        g.idx < 2,
                        "{name}: noul gold {} out of the yes/no pair",
                        g.idx
                    );
                    assert!(g.gold_score.is_none(), "{name}: noul carries no gold_score");
                }
            }
        }
        // Labels: every label has corpus docs; docs labels ⊆ labels.
        for l in &d.labels {
            assert!(
                d.docs.iter().any(|doc| &doc.label == l),
                "{name}: domain {l} armed with no corpus docs"
            );
        }
        for doc in &d.docs {
            assert!(
                d.labels.iter().any(|l| l == &doc.label),
                "{name}: doc label {} not in the domain set",
                doc.label
            );
        }
    }
}

/// Build the family engine exactly the way the runner does (domain order =
/// label order; corpus = that label's docs).
fn family_engine<const N: usize>(d: &families::SynthData) -> DecisionEngine<N, EMBED_DIM> {
    assert_eq!(
        d.labels.len(),
        N,
        "family {} arms {} domains",
        d.suite.name,
        N
    );
    let specs: Vec<ExpertSpec> = d
        .labels
        .iter()
        .map(|l| {
            let docs: Vec<String> = d
                .docs
                .iter()
                .filter(|doc| &doc.label == l)
                .map(|doc| doc.text.clone())
                .collect();
            ExpertSpec::new(l, &docs)
        })
        .collect();
    DecisionEngine::<N, EMBED_DIM>::build_specs(specs, EngineConfig::default())
        .expect("family engine well-formed")
}

/// Wire request for one case, the same shape `engine_request` builds
/// (state = pyjson-serialized bytes; choice options = criteria key order).
fn case_request(c: &riir_reflex::harness::suites::SuiteCase) -> DecisionRequest {
    let q = &c.questions[0];
    let question = match q.kind {
        QKind::Choice => {
            let keys: Vec<String> = q
                .criteria
                .as_object()
                .expect("choice criteria object")
                .keys()
                .cloned()
                .collect();
            Question::choice(q.qid.as_str(), q.instructions.as_str(), keys, None)
        }
        QKind::Score => {
            let levels: Vec<String> = q
                .criteria
                .as_array()
                .expect("score criteria array")
                .iter()
                .map(|v| v.as_str().expect("string level").to_string())
                .collect();
            Question::score(q.qid.as_str(), q.instructions.as_str(), levels)
        }
        QKind::Noul => Question::noul(q.qid.as_str(), q.instructions.as_str()),
    };
    DecisionRequest {
        state: serialize_state(&c.state),
        questions: vec![question],
    }
}

#[test]
fn engine_answers_families_deterministically_fast_and_above_chance() {
    // Full pipeline over every modelless family: determinism, wire
    // validity, L1 normalization, G2 latency, forced-pick accuracy.
    macro_rules! run_family {
        ($n:literal, $name:expr) => {{
            let d = families::synth_by_name($name).expect("family synth");
            let mut engine = family_engine::<$n>(&d);
            let mut sc: Scratch<EMBED_DIM> = Scratch::new();
            sc.prepare(1);
            let mut correct = 0usize;
            let mut total = 0usize;
            for c in &d.suite.cases {
                let req = case_request(c);
                let t0 = std::time::Instant::now();
                let resp = engine
                    .decide_with(&req, &mut sc)
                    .unwrap_or_else(|e| panic!("{}: decide failed: {e}", $name));
                let us = t0.elapsed().as_micros();
                assert!(
                    us < 1_000,
                    "G2 FAIL ({}): per-request {us} µs ≥ 1 ms",
                    $name
                );
                resp.validate_against(&req)
                    .unwrap_or_else(|e| panic!("{}: wire-invalid: {e}", $name));
                // Determinism: repeat run bit-identical.
                let resp2 = engine.decide_with(&req, &mut sc).expect("repeat decide");
                assert_eq!(
                    serde_json::to_string(&resp.answers).unwrap(),
                    serde_json::to_string(&resp2.answers).unwrap(),
                    "{}: repeat runs must be bit-identical (modelless lane)",
                    $name
                );
                drop(resp2);
                let a = &resp.answers[0];
                assert!(
                    a.probabilities.iter().all(|p| p.is_finite()),
                    "{}: non-finite probability",
                    $name
                );
                // Noul carries exactly ONE wire probability (p_yes — the
                // engine's internal [yes, no] pair stays internal).
                if d.suite.cases[0].questions[0].kind == QKind::Noul {
                    assert_eq!(
                        a.probabilities.len(),
                        1,
                        "{}: noul must carry exactly one wire probability",
                        $name
                    );
                    let p_yes = f64::from(a.probabilities[0]);
                    assert!(
                        (0.0..=1.0).contains(&p_yes),
                        "{}: noul p_yes {p_yes} outside [0, 1]",
                        $name
                    );
                } else {
                    let sum: f32 = a.probabilities.iter().sum();
                    assert!(
                        (sum - 1.0).abs() < 1e-2,
                        "{}: probabilities not L1-normalized (sum {sum})",
                        $name
                    );
                }
                let pick = match &a.outcome {
                    Some(katgpt_core::decision_wire::Outcome::Choice { index }) => *index as usize,
                    Some(katgpt_core::decision_wire::Outcome::Score { level }) => *level as usize,
                    Some(katgpt_core::decision_wire::Outcome::Noul { yes }) => usize::from(*yes),
                    _ => a
                        .probabilities
                        .iter()
                        .enumerate()
                        .max_by(|x, y| x.1.total_cmp(y.1))
                        .map_or(0, |(i, _)| i),
                };
                total += 1;
                if pick == c.gold[0].idx {
                    correct += 1;
                }
            }
            let acc = correct as f64 / total.max(1) as f64;
            if let Some((_, floor)) = ACC_FLOORS.iter().find(|(n, _)| **n == *$name) {
                assert!(
                    acc >= *floor,
                    "{}: forced accuracy {acc:.3} below the anti-pathology floor \
                     {floor} — picks are systematically wrong, not merely chance",
                    $name
                );
            }
            println!(
                "[recorded] {}: forced acc {acc:.3} (zero-shot losses are \
                 published, not gated — the lane's claims are \
                 calibration/abstain/latency/determinism)",
                $name
            );
            acc
        }};
    }
    let _defects = run_family!(6, "semantic_defects");
}

/// Discrimination floor (verdict round 3, the degenerate-lane lesson): a
/// family whose engine emits ONE pick (or one probability vector) for every
/// eval case is a constant function of the input — recorded accuracy sits
/// at exactly chance on class-balanced fixtures and the lane certifies
/// nothing. Every modelless family must produce ≥ 2 distinct picks and ≥ 2
/// distinct probability vectors across its eval slice.
#[test]
fn families_discriminate_their_inputs() {
    macro_rules! discriminate {
        ($n:literal, $name:expr) => {{
            let d = families::synth_by_name($name).expect("family synth");
            let mut engine = family_engine::<$n>(&d);
            let mut sc: Scratch<EMBED_DIM> = Scratch::new();
            sc.prepare(1);
            let mut picks: Vec<usize> = Vec::with_capacity(d.suite.cases.len());
            let mut vectors: Vec<String> = Vec::with_capacity(d.suite.cases.len());
            for c in &d.suite.cases {
                let req = case_request(c);
                let resp = engine
                    .decide_with(&req, &mut sc)
                    .unwrap_or_else(|e| panic!("{}: decide failed: {e}", $name));
                let a = &resp.answers[0];
                picks.push(match &a.outcome {
                    Some(katgpt_core::decision_wire::Outcome::Choice { index }) => *index as usize,
                    Some(katgpt_core::decision_wire::Outcome::Score { level }) => *level as usize,
                    Some(katgpt_core::decision_wire::Outcome::Noul { yes }) => usize::from(*yes),
                    _ => a
                        .probabilities
                        .iter()
                        .enumerate()
                        .max_by(|x, y| x.1.total_cmp(y.1))
                        .map_or(0, |(i, _)| i),
                });
                vectors.push(
                    a.probabilities
                        .iter()
                        .map(|p| format!("{p:.6}"))
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
            picks.sort_unstable();
            picks.dedup();
            let mut vs = vectors.clone();
            vs.sort();
            vs.dedup();
            println!(
                "[discrimination] {}: {} distinct picks / {} distinct vectors \
                 over {} cases",
                $name,
                picks.len(),
                vs.len(),
                d.suite.cases.len()
            );
            assert!(
                picks.len() >= 2,
                "{}: {} distinct pick across {} cases — the engine is a CONSTANT \
                 function of the input (degenerate lane)",
                $name,
                picks.len(),
                d.suite.cases.len()
            );
            assert!(
                vs.len() >= 2,
                "{}: {} distinct probability vector across {} cases — inputs do \
                 not reach the scores (degenerate lane)",
                $name,
                vs.len(),
                d.suite.cases.len()
            );
        }};
    }
    discriminate!(6, "semantic_defects");
}

// ── Plan 009 REVISED-2: the wide-eval authoring gates (issue 059) ───────
//
// The remaining family's eval population carries an authoring law the
// gates below enforce. (The six Issue-004 families retired 2026-10-02;
// the retired section's constants and pins went with them.)
//
// Baselines (measured on the pre-wide evals before the swap — the
// `.benchmarks/105` record carries the full table): old mean unigram
// overlap ran 0.68–0.78 with per-case maxes at 1.00 and 2–7 of 12–16 eval
// cases sharing a word 3-gram with corpus/cal. The wide authoring is
// STRICTER than every old number: zero shared 3-grams (the
// memorization-path wall), mean ≤ [`WIDE_MAX_MEAN_OVERLAP`], per-case ≤
// [`WIDE_MAX_CASE_OVERLAP`], and the exact label-token ban.

/// The widened families, in registry order.
const WIDE_FAMILIES: &[&str] = &["semantic_defects"];

/// Per-family banned label tokens (exact token match, lowercased).
const LABEL_BANS: &[(&str, &[&str])] = &[
    (
        "semantic_defects",
        &[
            "clean", "off", "by", "one", "inverted", "condition", "unwrapped",
            "none", "swapped", "lookup", "swallowed", "error",
        ],
    ),
];

/// Mean unigram overlap ceiling per family (eval-text token share that
/// appears in the family's corpus∪cal pool). The old evals measured
/// 0.68–0.78; the wide authoring targets materially fresher vocabulary.
const WIDE_MAX_MEAN_OVERLAP: f64 = 0.78;

/// Per-case unigram overlap ceiling — the old evals carried 1.00 cases
/// (every token already in the pool); the wide law requires every case to
/// carry fresh vocabulary.
const WIDE_MAX_CASE_OVERLAP: f64 = 0.92;

/// The acceptance band for a widened eval population (the plan's numbers).
const WIDE_MIN_CASES: usize = 88;
const WIDE_MAX_CASES: usize = 104;

fn wide_tokens(text: &str) -> Vec<String> {
    fixture_hygiene::tokens(text)
}

fn wide_trigrams(text: &str) -> HashSet<[String; 3]> {
    fixture_hygiene::trigrams(text)
}

/// BLAKE3 digest pin over the canonical wide-eval bytes — the shared
/// `fixture_hygiene::fixture_digest` law (name, then gold 0x1F text 0x1E
/// per case). Any fixture edit (text, gold, order, insertion, deletion)
/// moves the digest and reds this gate until consciously re-typed in the
/// same change.
fn wide_eval_digest(def: &families::FamilyDef) -> String {
    let eval: Vec<(usize, &str)> = def.eval.iter().map(|t| (t.gold, t.text)).collect();
    fixture_hygiene::fixture_digest(def.name, &eval)
}

/// The digest pins, one per widened family. Bootstrap: a mismatch panic
/// prints the computed digest — paste it here in the same change that
/// moved the fixtures (never delete a pin to make a red go away).
const WIDE_EVAL_DIGESTS: &[(&str, &str)] = &[
    (
        "semantic_defects",
        "52573a21b72da3444c97293728f85e6256226499ae6794ed75585227d69cac78",
    ),
];

#[test]
fn wide_evals_meet_the_authoring_law() {
    for name in WIDE_FAMILIES {
        let def = families::family_def(name).unwrap_or_else(|| panic!("{name} has no FamilyDef"));

        // Population band + exact class balance (max−min ≤ 1, every class
        // ≥ 16 — the small-n near-duplicate classes are gone).
        let n = def.eval.len();
        assert!(
            (WIDE_MIN_CASES..=WIDE_MAX_CASES).contains(&n),
            "{name}: eval population {n} outside the wide band \
             [{WIDE_MIN_CASES}, {WIDE_MAX_CASES}]"
        );
        let mut class_counts = vec![0usize; def.labels.len()];
        for t in def.eval {
            class_counts[t.gold] += 1;
        }
        let lo = *class_counts.iter().min().unwrap();
        let hi = *class_counts.iter().max().unwrap();
        assert!(
            hi - lo <= 1 && lo >= 16,
            "{name}: class balance {class_counts:?} — the wide law is exact \
             balance (max−min ≤ 1) with every class ≥ 16"
        );

        // Label-token bans: the family bans its exact label tokens on
        // eval AND cal.
        let bans = LABEL_BANS
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, b)| *b)
            .unwrap_or(&[]);
        assert!(!bans.is_empty(), "{name}: no label-ban row (gate bug)");
        for (i, t) in def.eval.iter().enumerate() {
            for w in wide_tokens(t.text) {
                assert!(
                    !bans.contains(&w.as_str()),
                    "{name}: eval case {i} carries the label token `{w}` — \
                     the text leaks its own answer"
                );
            }
        }
        for (i, t) in def.cal.iter().enumerate() {
            for w in wide_tokens(t.text) {
                assert!(
                    !bans.contains(&w.as_str()),
                    "{name}: cal case {i} carries the label token `{w}`"
                );
            }
        }

        // The memorization-path wall: no word 3-gram shared with corpus or
        // cal (the old evals carried 2–7 such cases per family; the wide
        // authoring is held to zero).
        let mut wall: HashSet<[String; 3]> = HashSet::new();
        for (_, ct) in def.corpus {
            wall.extend(wide_trigrams(ct));
        }
        for c in def.cal {
            wall.extend(wide_trigrams(c.text));
        }
        for (i, t) in def.eval.iter().enumerate() {
            let et = wide_trigrams(t.text);
            let hits: Vec<_> = et.intersection(&wall).take(3).collect();
            assert!(
                hits.is_empty(),
                "{name}: eval case {i} shares {hits:?} with corpus/cal — \
                 the memorization-path wall is zero-tolerance"
            );
        }

        // Unigram overlap ceilings (mean + per-case), measured against the
        // corpus∪cal token pool. Prints the actuals on every run — the
        // numbers the bench record cites.
        let mut pool: HashSet<String> = HashSet::new();
        for (_, ct) in def.corpus {
            pool.extend(wide_tokens(ct));
        }
        for c in def.cal {
            pool.extend(wide_tokens(c.text));
        }
        let mut sum = 0.0f64;
        let mut max = 0.0f64;
        for (i, t) in def.eval.iter().enumerate() {
            let toks = wide_tokens(t.text);
            let hits = toks.iter().filter(|w| pool.contains(*w)).count();
            let o = hits as f64 / toks.len() as f64;
            sum += o;
            max = max.max(o);
            assert!(
                o <= WIDE_MAX_CASE_OVERLAP,
                "{name}: eval case {i} unigram overlap {o:.3} > \
                 {WIDE_MAX_CASE_OVERLAP} — a wholesale vocabulary copy"
            );
        }
        let mean = sum / n as f64;
        assert!(
            mean <= WIDE_MAX_MEAN_OVERLAP,
            "{name}: mean unigram overlap {mean:.4} > {WIDE_MAX_MEAN_OVERLAP}"
        );

        // Eval-internal dedup (case-insensitive).
        let mut seen: HashSet<String> = HashSet::new();
        for (i, t) in def.eval.iter().enumerate() {
            assert!(
                seen.insert(t.text.to_lowercase()),
                "{name}: eval case {i} duplicates an earlier case"
            );
        }

        // The digest pin.
        let computed = wide_eval_digest(def);
        let pin = WIDE_EVAL_DIGESTS
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| *d)
            .expect("digest pin row");
        assert_eq!(
            pin, computed,
            "{name}: wide-eval digest moved (computed {computed}) — the \
             fixtures changed; re-pin in the same change, with the reason"
        );
        println!(
            "[recorded] {name}: eval {n} cases, balance {class_counts:?}, \
             mean_ovl {mean:.4}, max_ovl {max:.4}"
        );
    }
}
