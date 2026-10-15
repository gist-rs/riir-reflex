//! Issue 090 C1 — the rubric-covariance + template probes.
//!
//! The Jeva paper's rubric discussion converts our score-readout arithmetic
//! into pin-able identities, and our routing geometry into pin-able route
//! agreements. Three law families land here:
//!
//! 1. **The readout shape** — the harness's score readout is exactly
//!    `Σ i·p_i` over the rubric level distribution `p` (values `v_i` are the
//!    0-based level INDEX; the engine never reads level text as a value).
//!    [`riir_reflex::harness::metrics::score_metrics`] is the seam.
//! 2. **The covariance identities** (to fp tolerance) — `v→10v` scales the
//!    readout ×10; `v→v+c` shifts it by exactly `c·Σq` (== `c` at the
//!    normalization law `Σp = 1`); the picked level is the first-index
//!    argmax of `p` and is **value-blind** (it cannot move under any affine
//!    value transform — unlike a nearest-to-expected law, which is
//!    scale-sensitive and can land off the rubric entirely).
//! 3. **The route agreement** — the corpus route is a STATE property:
//!    the question context (`state + prompt + criteria`) is what routes, so
//!    option ORDER (never serialized into the context) cannot move it, the
//!    score/choice template TYPE is scoring-invisible when both spell the
//!    same option set with no criteria (byte-identical context), and on
//!    on-topic states the context route agrees with the state-alone
//!    centroid ranking — "state route == option routes".
//!
//! The suite-level complement of family 3 (template mass on real corpora)
//! is measured, not gated — it rides the existing probe lanes
//! (`--perm-probe` / `--null-probe`); this file pins the fixture-level laws
//! whose violation would be a structural regression.
#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, Question};
use riir_reflex::embed::{EMBED_DIM, Embedder};
use riir_reflex::engine::{DecisionEngine, DomainExpert, EngineConfig, Scratch};
use riir_reflex::harness::metrics::score_metrics;

/// The rubric level spellings — ordinal corpora, the sst5 shape: the
/// domains ARE the level-index strings, so every score option resolves to
/// its domain by name (the `option_name_route` arm).
const LEVELS: [&str; 5] = ["0", "1", "2", "3", "4"];

/// Two documents per ordinal domain (the corpus IS the model); the
/// sentiment vocabularies are disjoint across levels by construction.
const LEVEL_DOCS: [[&str; 2]; 5] = [
    [
        "terrible awful horrible disaster worst experience broken useless hated every minute",
        "the worst broken mess a horrible useless disaster hated it",
    ],
    [
        "bad poor disappointing weak boring dull lacks quality disliked it",
        "a dull weak mess poor boring and disappointing disliked",
    ],
    [
        "average okay mediocre ordinary mixed fair passable neither here nor there",
        "an ordinary fair passable film mediocre mixed and average okay",
    ],
    [
        "good solid nice pleasant decent enjoyed works well liked it",
        "a solid nice pleasant surprise good decent enjoyed and liked",
    ],
    [
        "excellent amazing wonderful perfect outstanding loved best fantastic",
        "an amazing wonderful film excellent perfect outstanding loved the best fantastic",
    ],
];

/// One curated on-topic state per level: the level's own first document,
/// VERBATIM — the state sits inside its domain's corpus, so the route term
/// (`sigmoid(cos(state, centroid)·8)`) and the drafter delta both peak at
/// that level with a decisive margin.
fn curated_state(level: usize) -> &'static str {
    LEVEL_DOCS[level][0]
}

/// A deliberate BLEND of levels 3 and 1 vocabulary — the non-degenerate
/// live distribution the identities are also asserted on (its PICK is not
/// asserted: either ordinal neighbor is a legitimate blend verdict).
const MIXED_STATE: &str = "good solid nice pleasant decent enjoyed it works well but bad poor disappointing weak boring dull lacks quality";

fn ordinal_experts() -> Vec<DomainExpert<EMBED_DIM>> {
    LEVEL_DOCS
        .iter()
        .enumerate()
        .map(|(level, docs)| {
            let owned: Vec<String> = docs.iter().map(|d| (*d).to_string()).collect();
            DomainExpert::new(LEVELS[level], &owned, &Embedder)
        })
        .collect()
}

fn score_request(state: &str, prompt: &str, options: &[&str]) -> DecisionRequest {
    DecisionRequest {
        state: state.to_string(),
        questions: vec![Question::score(
            "q0",
            prompt,
            options.iter().map(|o| (*o).to_string()).collect(),
        )],
    }
}

/// First-index argmax — the engine's pick tie rule (a strict `>` scan keeps
/// the earliest index on exact ties).
fn first_argmax(p: &[f64]) -> usize {
    let mut best = 0;
    for (i, &v) in p.iter().enumerate() {
        if v > p[best] {
            best = i;
        }
    }
    best
}

/// The covariance identities of the score readout, to fp tolerance:
/// shape (`Σ i·p`), the `v→10v` ×10 law, the `v→v+c` `c·Σp` shift law
/// (== `c` at `Σp = 1`), and first-index-argmax pickability.
fn assert_rubric_identities(p32: &[f32], label: &str) {
    let p: Vec<f64> = p32.iter().map(|&x| x as f64).collect();
    assert!(!p.is_empty(), "{label}: empty distribution");
    for (i, &v) in p.iter().enumerate() {
        assert!(
            v.is_finite() && (0.0..=1.0).contains(&v),
            "{label}: p[{i}] = {v} outside [0, 1]"
        );
    }
    let sum: f64 = p.iter().sum();
    assert!(
        (sum - 1.0).abs() <= 1e-5,
        "{label}: Σp = {sum}, not a distribution"
    );

    // The shipped readout IS Σ i·p_i (the 0-based index vector).
    let e = score_metrics(&p, 0.0).expected;
    let e_ref: f64 = p.iter().enumerate().map(|(i, &pi)| i as f64 * pi).sum();
    assert!(
        (e - e_ref).abs() <= 1e-9,
        "{label}: readout {e} != Σ i·p {e_ref} — the readout shape moved"
    );

    // v→10v scales ×10 exactly (the readout is LINEAR in v — no clipping,
    // rounding, or argmax-collapse in the seam).
    let e10: f64 = p
        .iter()
        .enumerate()
        .map(|(i, &pi)| (10.0 * i as f64) * pi)
        .sum();
    assert!(
        (e10 - 10.0 * e).abs() <= 1e-9,
        "{label}: v→10v law broke: {e10} != 10 × {e}"
    );

    // v→v+c shifts by exactly c·Σp; at the normalization law (Σp = 1) the
    // shift is exactly c.
    let c = 2.5f64;
    let eshift: f64 = p
        .iter()
        .enumerate()
        .map(|(i, &pi)| (i as f64 + c) * pi)
        .sum();
    assert!(
        (eshift - (e + c * sum)).abs() <= 1e-9,
        "{label}: v→v+c law broke: {} != {} + {c}·Σp",
        eshift,
        e
    );
    assert!(
        (eshift - (e + c)).abs() <= 1e-4,
        "{label}: shift at Σp=1 broke: {} != {} + {c}",
        eshift,
        e
    );

    // The distribution is argmax-pickable (first-index tie).
    let _ = first_argmax(&p);
}

#[test]
fn readout_identities_hold_on_the_synthetic_battery() {
    let battery: Vec<(&str, Vec<f64>)> = vec![
        ("uniform", vec![0.2; 5]),
        ("one-hot-low", vec![1.0, 0.0, 0.0, 0.0, 0.0]),
        ("one-hot-high", vec![0.0, 0.0, 0.0, 0.0, 1.0]),
        ("peaked", vec![0.05, 0.10, 0.15, 0.30, 0.40]),
        ("exact-tie", vec![0.5, 0.5, 0.0, 0.0, 0.0]),
        ("near-tie", vec![0.34, 0.33, 0.33, 0.0, 0.0]),
        ("skew-forward", vec![0.4, 0.35, 0.25, 0.0, 0.0]),
    ];
    for (label, p) in &battery {
        let p32: Vec<f32> = p.iter().map(|&v| v as f32).collect();
        assert_rubric_identities(&p32, label);
    }

    // First-index tie rule: exact ties resolve to the EARLIEST index.
    assert_eq!(first_argmax(&[0.5, 0.5, 0.0, 0.0, 0.0]), 0);
    assert_eq!(first_argmax(&[0.1, 0.5, 0.5, 0.0, 0.0]), 1);

    // The value-blind law's teeth: the shipped pick is the argmax over q —
    // it CANNOT move under an affine value transform. The nearest-to-
    // expected law is the counter-law: value-SENSITIVE, and under v→10v it
    // lands OFF the rubric entirely (8.5 has no nearest level in 0..5).
    // p = [0.4, 0.35, 0.25, 0, 0]: argmax = 0, E = 0.85 → nearest = 1 —
    // the two laws already disagree; the engine ships the argmax (pinned
    // on live distributions below).
    let p = [0.4, 0.35, 0.25, 0.0, 0.0];
    let e: f64 = p.iter().enumerate().map(|(i, &pi)| i as f64 * pi).sum();
    assert!((e - 0.85).abs() <= 1e-12);
    assert_ne!(first_argmax(&p), e.round() as usize);
    let e10: f64 = p
        .iter()
        .enumerate()
        .map(|(i, &pi)| (10.0 * i as f64) * pi)
        .sum();
    assert!((e10 - 8.5).abs() <= 1e-12);
    assert!(
        e10.round() as usize >= p.len(),
        "nearest law left the rubric"
    );
    assert_eq!(first_argmax(&p), 0, "argmax never moved under v→10v");
}

#[test]
fn live_score_distributions_satisfy_the_covariance_identities_and_argmax_pick() {
    let experts = ordinal_experts();
    // The fused gate (score + distance thresholds) is the serving posture's
    // abstention knob, not a covariance subject: on a 6-document fixture the
    // 1-byte level candidates keep the drafter term at sigmoid(0) = 0.5 for
    // every option, so p is near-uniform across the squashed route terms and
    // the cold-start entropy readout sits far below any calibrated threshold.
    // The identities and the pick law are gate-independent; the gates are
    // disarmed so the distributions are observable at all.
    let cfg = EngineConfig {
        score_threshold: 0.0,
        distance_threshold: 0.0,
        ..EngineConfig::default()
    };
    let mut eng: DecisionEngine<5, EMBED_DIM> = DecisionEngine::build(experts, cfg).unwrap();
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();

    for level in 0..5 {
        let state = curated_state(level);
        sc.prepare(1);
        eng.solve_into(
            &score_request(state, "rate the sentiment", &LEVELS),
            &mut sc,
        )
        .unwrap();
        let slot = &sc.slots[0];
        assert!(
            !slot.abstained,
            "level {level} state must answer, abstained via {:?}",
            slot.cause
        );
        assert_eq!(slot.prob_len, 5, "level {level}: full arity");
        let p: Vec<f32> = sc.probs[slot.prob_lo..slot.prob_lo + slot.prob_len].to_vec();
        assert_rubric_identities(&p, &format!("level {level}"));

        // The by-name route arm must be ACTIVE (options "0".."4" all
        // resolve to the ordinal domains) — the option-route path exists.
        assert!(
            !slot.drafter_only,
            "level {level}: route terms must arm on the ordinal fixture"
        );

        // The curated state IS its level's corpus text: the pick is that
        // level (the fixture's behavioral sanity).
        assert_eq!(
            slot.pick as usize,
            level,
            "level {level} state picked {:?}",
            &sc.probs[slot.prob_lo..slot.prob_lo + slot.prob_len]
        );
    }

    // The blended state: identities hold on a NON-degenerate live
    // distribution; the pick is not asserted (either ordinal neighbor of
    // the blend is legitimate).
    sc.prepare(1);
    eng.solve_into(
        &score_request(MIXED_STATE, "rate the sentiment", &LEVELS),
        &mut sc,
    )
    .unwrap();
    let slot = &sc.slots[0];
    assert!(
        !slot.abstained,
        "mixed state must answer, abstained via {:?}",
        slot.cause
    );
    let p: Vec<f32> = sc.probs[slot.prob_lo..slot.prob_lo + slot.prob_len].to_vec();
    assert_rubric_identities(&p, "mixed");
    assert_eq!(
        slot.pick as usize,
        first_argmax(&p.iter().map(|&v| v as f64).collect::<Vec<_>>())
    );
}

#[test]
fn corpus_route_is_a_state_property_across_template_types_and_option_order() {
    let experts = ordinal_experts();

    // The state-alone centroid ranking, recomputed from the SAME fixture
    // docs + the SAME embedder (the mirror-the-math pattern): the "state
    // route" the question route must agree with.
    let centroids: Vec<[f32; EMBED_DIM]> = LEVEL_DOCS
        .iter()
        .map(|docs| {
            let mut acc = [0.0f32; EMBED_DIM];
            for d in docs.iter() {
                let mut v = [0.0f32; EMBED_DIM];
                Embedder.embed_into(d.as_bytes(), &mut v);
                for (a, x) in acc.iter_mut().zip(v.iter()) {
                    *a += x;
                }
            }
            let k = docs.len() as f32;
            for x in acc.iter_mut() {
                *x /= k;
            }
            let n = acc.iter().map(|x| x * x).sum::<f32>().sqrt();
            if n > 0.0 {
                for x in acc.iter_mut() {
                    *x /= n;
                }
            }
            acc
        })
        .collect();
    let state_route = |state: &str| {
        let mut q = [0.0f32; EMBED_DIM];
        Embedder.embed_into(state.as_bytes(), &mut q);
        let mut dots = [0.0f32; 5];
        for (d, c) in centroids.iter().enumerate() {
            dots[d] = q.iter().zip(c.iter()).map(|(a, b)| a * b).sum();
        }
        let mut best = 0usize;
        for (d, &s) in dots.iter().enumerate() {
            if s > dots[best] {
                best = d;
            }
        }
        best
    };

    let reversed: [&str; 5] = ["4", "3", "2", "1", "0"];
    // The same disarmed-gate posture as the identities test — the route is
    // upstream of the fused gate, and a 6-document fixture cannot clear the
    // serving thresholds (the abstention knob is not a route subject).
    let cfg = EngineConfig {
        score_threshold: 0.0,
        distance_threshold: 0.0,
        ..EngineConfig::default()
    };
    let mut eng: DecisionEngine<5, EMBED_DIM> = DecisionEngine::build(experts, cfg).unwrap();
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    let mut choice_sc: Scratch<EMBED_DIM> = Scratch::new();

    for level in 0..5 {
        let state = curated_state(level);

        // (a) The full score spelling.
        sc.prepare(1);
        eng.solve_into(
            &score_request(state, "rate the sentiment", &LEVELS),
            &mut sc,
        )
        .unwrap();
        let route_full = sc.domains[0];
        let probs_full = sc.probs.clone();

        // (b) The MINIMAL-template spelling — a one-word prompt. The wire
        // refuses an empty prompt (fail-closed), so this is the closest
        // legal approach to the state-alone context: the engine's own
        // state-route proxy. A heavier template moving the route on an
        // on-topic state is the failure this leg exists to catch.
        sc.prepare(1);
        eng.solve_into(&score_request(state, "rate", &LEVELS), &mut sc)
            .unwrap();
        let route_bare = sc.domains[0];

        // (c) The CHOICE template spelling of the SAME option set — no
        // criteria on either side, so the context bytes are IDENTICAL and
        // the whole solve (route AND probabilities) must be bit-identical:
        // the template type is scoring-invisible.
        choice_sc.prepare(1);
        eng.solve_into(
            &DecisionRequest {
                state: state.to_string(),
                questions: vec![Question::choice(
                    "q0",
                    "rate the sentiment",
                    LEVELS.iter().map(|o| (*o).to_string()).collect(),
                    None,
                )],
            },
            &mut choice_sc,
        )
        .unwrap();
        let route_choice = choice_sc.domains[0];

        // (d) The REVERSED option order — options never enter the context,
        // so the route must be bit-identical under permutation.
        sc.prepare(1);
        eng.solve_into(
            &score_request(state, "rate the sentiment", &reversed),
            &mut sc,
        )
        .unwrap();
        let route_perm = sc.domains[0];

        // (e) The state-alone route — TWO independent computations that
        // must agree with each other AND with the engine's route: the
        // accessor `semantic_route` (the engine's own routing table over
        // the text embedded alone) and the MIRROR (centroids recomputed
        // in-test from the same fixture docs + embedder — an oracle that
        // does not read the engine's tables, so a routing-table regression
        // cannot hide behind the accessor that reads it).
        let route_accessor = eng.semantic_route(state);
        let route_state = state_route(state);

        assert_eq!(
            route_full, level,
            "level {level}: context route must be the state's own domain"
        );
        assert_eq!(
            route_bare, route_full,
            "level {level}: the bare-prompt (state-only) route must agree — \
             template presence moved the route"
        );
        assert_eq!(
            route_choice, route_full,
            "level {level}: the choice/score template type moved the route"
        );
        assert_eq!(
            choice_sc.probs, probs_full,
            "level {level}: the choice/score template type moved the scoring"
        );
        assert_eq!(
            route_perm, route_full,
            "level {level}: option ORDER moved the route (order leaked into ctx)"
        );
        assert_eq!(
            route_state, route_full,
            "level {level}: state route != option routes — the context route \
             disagrees with the state-alone centroid mirror"
        );
        assert_eq!(
            route_accessor, route_state,
            "level {level}: semantic_route disagrees with the independent \
             centroid mirror — the accessor and the mirror must agree"
        );
    }
}
