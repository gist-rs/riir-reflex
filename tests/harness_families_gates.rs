//! Issue 004 (Research 579) T6 gates for the six harness decision-point
//! families — `harness_visibility` · `harness_permissions` ·
//! `harness_tool_fit` · `harness_routing` · `harness_sensitivity`
//! (modelless DEFAULT) and `harness_cache_reuse` (LLM-lane ONLY).
//!
//! What is asserted here:
//! - registry contract: all six registered synthetic; cache_reuse marked
//!   `modelless_lane = false` (the loud-skip contract);
//! - count floors: eval ≥ 12, cal ≥ 16 (the fused-gate thresholds cannot
//!   fit below 16 cal observations — runner law), corpus ≥ 8, every class
//!   covered in all three slices;
//! - the self-inclusion leak law: corpus/cal/eval text sets pairwise
//!   disjoint per family (a cal case scoring against itself inflates every
//!   cal-slice quantile);
//! - gold/option agreement: gold indexes in range, score families carry
//!   `gold_score == level`;
//! - engine smoke over every modelless family: wire-valid answers,
//!   L1-normalized probabilities, per-request G2 < 1 ms, bit-identical
//!   repeats, anti-pathology accuracy floors (0.8× chance), and the
//!   DISCRIMINATION floor — distinct picks AND distinct probability vectors
//!   across the eval slice (the degenerate-lane lesson: a constant,
//!   input-independent pick sits at exactly chance on class-balanced
//!   fixtures, so accuracy alone cannot see it — only distinct counts can);
//! - cache_reuse: fixtures exist, noul-shaped, and its SynthData ships NO
//!   modelless corpus (the honest absence, by construction).

#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, Question};
use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};
use riir_reflex::harness::families;
use riir_reflex::harness::runner;
use riir_reflex::harness::suites::QKind;
use riir_reflex::pyjson::serialize_state;

const MODELLESS_FAMILIES: &[&str] = &[
    "harness_visibility",
    "harness_permissions",
    "harness_tool_fit",
    "harness_routing",
    "harness_sensitivity",
];

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
    ("harness_visibility", 0.20),  // chance 0.25
    ("harness_permissions", 0.26), // chance ~0.33
    ("harness_tool_fit", 0.13),    // chance ~0.17
    ("harness_routing", 0.20),     // chance 0.25
];

#[test]
fn families_are_registered_with_the_right_lanes() {
    for name in MODELLESS_FAMILIES {
        let (synthetic, modelless) = runner::suite_lanes(name)
            .unwrap_or_else(|| panic!("{name} missing from the suite registry"));
        assert!(synthetic, "{name} must be an in-process synthetic suite");
        assert!(modelless, "{name} is modelless by default (Issue 004)");
    }
    let (synthetic, modelless) = runner::suite_lanes("harness_cache_reuse")
        .expect("harness_cache_reuse registered");
    assert!(synthetic);
    assert!(
        !modelless,
        "harness_cache_reuse is LLM-lane only (Issue 004 T3) — the registry \
         must mark it modelless_lane = false or the runner would fabricate a \
         modelless row for a family the modelless lane cannot answer"
    );
}

#[test]
fn modelless_families_meet_count_coverage_and_disjointness_floors() {
    for name in MODELLESS_FAMILIES {
        let def = families::family_def(name)
            .unwrap_or_else(|| panic!("{name} has no FamilyDef"));
        assert!(def.eval.len() >= 12, "{name}: eval slice {} < 12", def.eval.len());
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
            assert!(cal_n >= 2, "{name}: class {label} has {cal_n} cal cases (< 2)");
            assert!(eval_n >= 2, "{name}: class {label} has {eval_n} eval cases (< 2)");
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
                    assert!(g.idx < keys.len(), "{name}: gold {} out of option range", g.idx);
                    assert!(g.gold_score.is_none(), "{name}: choice carries no gold_score");
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
                QKind::Noul => panic!("{name}: modelless families do not use noul"),
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
fn family_engine<const N: usize>(
    d: &families::SynthData,
) -> DecisionEngine<N, EMBED_DIM> {
    assert_eq!(d.labels.len(), N, "family {} arms {} domains", d.suite.name, N);
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
                let sum: f32 = a.probabilities.iter().sum();
                assert!(
                    (sum - 1.0).abs() < 1e-2,
                    "{}: probabilities not L1-normalized (sum {sum})",
                    $name
                );
                let pick = match &a.outcome {
                    Some(katgpt_core::decision_wire::Outcome::Choice { index }) => *index as usize,
                    Some(katgpt_core::decision_wire::Outcome::Score { level }) => *level as usize,
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
    let _vis = run_family!(4, "harness_visibility");
    let _perm = run_family!(3, "harness_permissions");
    let _tool = run_family!(6, "harness_tool_fit");
    let _route = run_family!(4, "harness_routing");
    let _sens = run_family!(5, "harness_sensitivity");
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
    discriminate!(4, "harness_visibility");
    discriminate!(3, "harness_permissions");
    discriminate!(6, "harness_tool_fit");
    discriminate!(4, "harness_routing");
    discriminate!(5, "harness_sensitivity");
}

#[test]
fn sensitivity_scores_within_one_level() {
    // The score-family anti-pathology floor: the expected level under the
    // calibrated probs lands within 1.0 of gold for ≥ 0.35 of cases — the
    // threshold the verdict round pinned (uniform picks measure 0.40 by
    // adjacent-mass arithmetic, so anything at or above 0.35 with real
    // discrimination is a live, non-anti-correlated scorer; the floor only
    // catches a systematically wrong engine, never gates accuracy claims).
    let d = families::synth_by_name("harness_sensitivity").expect("family synth");
    let mut engine = family_engine::<5>(&d);
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    sc.prepare(1);
    let mut within = 0usize;
    for c in &d.suite.cases {
        let req = case_request(c);
        let resp = engine.decide_with(&req, &mut sc).expect("decide");
        let a = &resp.answers[0];
        let expected: f64 = a
            .probabilities
            .iter()
            .enumerate()
            .map(|(i, p)| i as f64 * f64::from(*p))
            .sum();
        let gold = c.gold[0].gold_score.expect("score gold");
        if (expected - gold).abs() <= 1.0 {
            within += 1;
        }
    }
    let rate = within as f64 / d.suite.cases.len() as f64;
    assert!(
        rate >= 0.35,
        "sensitivity: within-1 rate {rate:.3} < 0.35 — picks are \
         systematically wrong (the recorded baseline is 0.40 at uniform \
         picks; zero-shot accuracy is published, not gated)"
    );
}

#[test]
fn cache_reuse_is_llm_only_by_construction() {
    let fixtures = families::cache_reuse_eval();
    assert!(
        fixtures.len() >= 12,
        "cache_reuse: {} fixtures < 12 — the family needs a measurable set",
        fixtures.len()
    );
    for t in fixtures {
        assert!(t.gold < 2, "cache_reuse gold must be noul (0/1)");
    }
    let d = families::synth_cache_reuse();
    assert!(
        d.docs.is_empty() && d.cal_cases.is_empty(),
        "cache_reuse must ship NO modelless corpus/cal — a modelless answer \
         here is the fake task Issue 004 T3 refuses"
    );
    assert!(d.suite.cases.len() == fixtures.len());
    // Every case is noul-shaped with the reuse/rebuild question.
    for c in &d.suite.cases {
        assert_eq!(c.questions[0].kind, QKind::Noul);
        assert!(c.questions[0].criteria.is_null());
    }
}
