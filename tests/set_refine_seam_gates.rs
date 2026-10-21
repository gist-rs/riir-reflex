//! Plan 627 T2 / reflex Issue 090 C2 — the `set_refine` ENGINE SEAM's
//! birth-gate suite (whole-file gated; the `[[test]]` row pins
//! `required-features = ["modelless", "set_refine"]` so a feature-less
//! selection skips loudly, never a green zero).
//!
//! The substrate suite (S1–S5) lives katgpt-core-side
//! (`katgpt-rs/tests/set_refine_suite.rs`). THIS suite pins the SEAM:
//! that the engine wires the substrate exactly as the law says — keys =
//! the options' resolved domain directions, base = the state-alone
//! embedding, route terms recomputed per question, noul + drafter-only
//! questions byte-identical, knob 0 byte-identical.
//!
//! The strongest gate here is [`seam_equals_substrate_composition`]: it
//! recomputes the armed engine's ENTIRE probability vector by hand —
//! embed → dirs → substrate `refine_state_into` → route terms →
//! drafter scores → L1 — using only public API, and requires the engine
//! to agree to float rounding. Any seam detail wrong (wrong base vector,
//! wrong keys, wrong scale placement, wrong question scope) reds it.

#![cfg(feature = "set_refine")]

use katgpt_core::decision_wire::{DecisionRequest, Question, QuestionKind};
use katgpt_core::exact_sigmoid;
use katgpt_core::set_refine::{SetRefineConfig, SetRefineScratch, refine_state_into};
use katgpt_core::variable_rank_domain_expert::pick_domain;
use riir_reflex::embed::{EMBED_DIM, Embedder};
use riir_reflex::engine::{DecisionEngine, DomainExpert, EngineConfig};

/// Two-option by-name fixture: the option strings ARE the domain names,
/// so by-name resolution arms route terms (k == N == 2) — the seam's
/// legal scope. The docs are shaped so the centroids separate.
fn fixture_docs() -> [Vec<String>; 2] {
    [
        vec![
            "refund the customer invoice balance in full".to_string(),
            "process the refund request for the overpaid invoice".to_string(),
            "issue a credit note for the refunded invoice balance".to_string(),
        ],
        vec![
            "escalate the outage to the on-call engineer".to_string(),
            "open a priority incident for the service outage".to_string(),
            "page the on-call engineer about the production outage".to_string(),
        ],
    ]
}

const NAMES: [&str; 2] = ["refund_invoice", "escalate_outage"];

fn build_engine(gamma: f32, tau: f32) -> DecisionEngine<2, EMBED_DIM> {
    let docs = fixture_docs();
    let specs: Vec<DomainExpert<EMBED_DIM>> = NAMES
        .iter()
        .zip(docs.iter())
        .map(|(name, dd)| DomainExpert::new(*name, dd, &Embedder))
        .collect();
    let cfg = EngineConfig {
        #[cfg(feature = "set_refine")]
        set_refine_gamma: gamma,
        #[cfg(feature = "set_refine")]
        set_refine_tau: tau,
        ..EngineConfig::default()
    };
    DecisionEngine::<2, EMBED_DIM>::build(specs, cfg).expect("2-domain fixture engine builds")
}

fn choice_request(state: &str, prompt: &str, options: [&str; 2]) -> DecisionRequest {
    DecisionRequest {
        state: state.to_string(),
        questions: vec![Question {
            id: "q0".to_string(),
            kind: QuestionKind::Choice,
            prompt: prompt.to_string(),
            options: options.iter().map(|s| s.to_string()).collect(),
            criteria: None,
        }],
    }
}

/// G3 posture: the DEFAULT is off (gamma 0) — the feature never changes
/// a shipped posture by existing.
#[test]
fn default_is_off() {
    let cfg = EngineConfig::default();
    assert_eq!(cfg.set_refine_gamma, 0.0);
    assert_eq!(cfg.set_refine_tau, 0.5);
}

/// G3 twin: gamma == 0 is byte-identical to the default cfg (the S5
/// engine-side identity — the knob-off engine and the default engine
/// answer identically, bit for bit).
#[test]
fn gamma_zero_is_byte_identical_to_default() {
    let req = choice_request(
        "the customer wants their invoice balance back",
        "what does the customer want?",
        NAMES,
    );
    let a = build_engine(0.0, 0.5).decide(&req).unwrap();
    let b = build_engine(0.0, 99.0).decide(&req).unwrap(); // tau moot at gamma 0
    let c = DecisionEngine::<2, EMBED_DIM>::build(
        NAMES
            .iter()
            .zip(fixture_docs().iter())
            .map(|(name, dd)| DomainExpert::new(*name, dd, &Embedder))
            .collect(),
        EngineConfig::default(),
    )
    .unwrap()
    .decide(&req)
    .unwrap();
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
    assert_eq!(format!("{a:?}"), format!("{c:?}"));
}

/// Fail-closed: an armed knob with a non-positive tau refuses at BUILD
/// (the σ(dot/τ) gate is undefined at τ ≤ 0 — never a silent NaN).
#[test]
fn armed_bad_tau_refuses_at_build() {
    let docs = fixture_docs();
    for bad in [0.0f32, -0.5] {
        let specs: Vec<DomainExpert<EMBED_DIM>> = NAMES
            .iter()
            .zip(docs.iter())
            .map(|(name, dd)| DomainExpert::new(*name, dd, &Embedder))
            .collect();
        let cfg = EngineConfig {
            set_refine_gamma: 1.0,
            set_refine_tau: bad,
            ..EngineConfig::default()
        };
        assert!(
            DecisionEngine::<2, EMBED_DIM>::build(specs, cfg).is_err(),
            "tau {bad} must refuse"
        );
    }
}

/// THE strong gate: the armed engine's probability vector equals the
/// hand-composed pipeline — substrate refine over the state-alone
/// embedding with the options' resolved directions as keys, route terms
/// recomputed, drafter scores + sigmoid + L1 — to float rounding. Uses
/// only public API; every seam detail is load-bearing here.
#[test]
fn seam_equals_substrate_composition() {
    let state = "the customer wants their invoice balance back";
    let prompt = "what does the customer want?";
    let tau = 0.5f32;
    let gamma = 1.0f32;

    let mut engine = build_engine(gamma, tau);
    let cfg = EngineConfig::default();
    let req = choice_request(state, prompt, NAMES);
    let resp = engine.decide(&req).unwrap();
    let probs: Vec<f32> = resp.answers[0].probabilities.clone();

    // ── hand composition (public API only) ──
    let embedder = Embedder;
    let docs = fixture_docs();
    // dirs: per-domain unit centroid of the embedded docs (the
    // DomainExpert construction contract).
    let mut dirs = [[0.0f32; EMBED_DIM]; 2];
    for (di, dd) in docs.iter().enumerate() {
        let mut acc = [0.0f32; EMBED_DIM];
        for d in dd {
            let mut v = [0.0f32; EMBED_DIM];
            embedder.embed_into(d.as_bytes(), &mut v);
            for (a, x) in acc.iter_mut().zip(v.iter()) {
                *a += x;
            }
        }
        for a in acc.iter_mut() {
            *a /= dd.len() as f32;
        }
        let n = acc.iter().map(|x| x * x).sum::<f32>().sqrt();
        for a in acc.iter_mut() {
            *a /= n;
        }
        dirs[di] = acc;
    }
    // state-alone embedding (the seam's base vector).
    let mut state_q = [0.0f32; EMBED_DIM];
    embedder.embed_into(state.as_bytes(), &mut state_q);
    // keys: options resolved BY NAME (option string == domain name) →
    // their directions, in option order.
    let mut keys: Vec<f32> = Vec::with_capacity(2 * EMBED_DIM);
    for opt in NAMES {
        let di = NAMES.iter().position(|n| *n == opt).unwrap();
        keys.extend_from_slice(&dirs[di]);
    }
    // substrate refine — the SAME call the seam makes.
    let mut scratch = SetRefineScratch::new(EMBED_DIM, 2);
    let mut refined = [0.0f32; EMBED_DIM];
    refine_state_into(
        &state_q,
        &keys,
        &mut refined,
        &SetRefineConfig::new(tau, gamma, EMBED_DIM),
        &mut scratch,
    )
    .unwrap();
    // route terms recomputed from the refined state.
    let route: Vec<f32> = dirs
        .iter()
        .map(|dir| {
            let mut dot = 0.0f32;
            for (qv, dv) in refined.iter().zip(dir.iter()) {
                dot += qv * dv;
            }
            exact_sigmoid(dot * cfg.route_scale)
        })
        .collect();
    // routed domain: the CONTEXT embedding's argmax (pick_domain).
    let ctx = format!("{state}\n{prompt}");
    let mut q = [0.0f32; EMBED_DIM];
    embedder.embed_into(ctx.as_bytes(), &mut q);
    let di = pick_domain::<2, EMBED_DIM>(&q, &dirs);
    // the routed domain's drafter, built from the SAME corpus the
    // DomainExpert holds (docs joined by newline — its construction law).
    let corpus = docs[di].join("\n");
    let mut drafter = katgpt_core::compression_drafter::Lz4FlexDrafter::new(corpus.into_bytes());
    let mut scores: Vec<f32> = Vec::with_capacity(2);
    for (i, opt) in NAMES.iter().enumerate() {
        let s_raw = drafter.score_into(ctx.as_bytes(), opt.as_bytes());
        scores.push(exact_sigmoid(s_raw as f32 / cfg.score_temperature) + route[i]);
    }
    let sum: f32 = scores.iter().sum();
    let expected: Vec<f32> = scores.iter().map(|s| s / sum).collect();

    assert_eq!(probs.len(), expected.len());
    for (p, e) in probs.iter().zip(expected.iter()) {
        assert!(
            (p - e).abs() <= 1e-6,
            "engine probs {probs:?} vs substrate-composed {expected:?}"
        );
    }
}

/// The seam only moves ROUTE-ACTIVE questions: a noul question in the
/// same request (route never active — issue 030 legality) must be
/// byte-identical between the armed and unarmed engines.
#[test]
fn noul_is_byte_identical_armed_vs_unarmed() {
    let docs = fixture_docs();
    let specs = || {
        NAMES
            .iter()
            .zip(docs.iter())
            .map(|(name, dd)| DomainExpert::new(*name, dd, &Embedder))
            .collect::<Vec<_>>()
    };
    let req = DecisionRequest {
        state: "the customer wants their invoice balance back".to_string(),
        questions: vec![
            Question {
                id: "n0".to_string(),
                kind: QuestionKind::Noul,
                prompt: "is this about a refund?".to_string(),
                options: vec![],
                criteria: None,
            },
            Question {
                id: "q0".to_string(),
                kind: QuestionKind::Choice,
                prompt: "what does the customer want?".to_string(),
                options: NAMES.iter().map(|s| s.to_string()).collect(),
                criteria: None,
            },
        ],
    };
    let mut armed = DecisionEngine::<2, EMBED_DIM>::build(
        specs(),
        EngineConfig {
            set_refine_gamma: 1.0,
            set_refine_tau: 0.5,
            ..EngineConfig::default()
        },
    )
    .unwrap();
    let mut unarmed =
        DecisionEngine::<2, EMBED_DIM>::build(specs(), EngineConfig::default()).unwrap();
    let a = armed.decide(&req).unwrap();
    let u = unarmed.decide(&req).unwrap();
    // The noul answer (index 0) is byte-identical.
    assert_eq!(
        format!("{:?}", a.answers[0].probabilities),
        format!("{:?}", u.answers[0].probabilities),
        "noul must never take the refinement (issue-030 legality)"
    );
    assert_eq!(a.answers[0].outcome, u.answers[0].outcome);
    // The choice answer (index 1) is the seam's target: its probability
    // vector must MOVE on this asymmetric fixture (distinct option keys
    // → distinct refined route terms) — the capability the whole plan
    // exists to price, pinned present on the seam's own fixture.
    let ap: &[f32] = &a.answers[1].probabilities;
    let up: &[f32] = &u.answers[1].probabilities;
    assert_eq!(ap.len(), 2);
    let moved = ap
        .iter()
        .zip(up.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max);
    assert!(
        moved > 1e-9,
        "armed choice probs must move vs unarmed: {ap:?} vs {up:?}"
    );
}

/// Engine-level S2: by-name resolution is order-symmetric — permuting
/// the option list permutes the probabilities with it (the refinement
/// sum is order-symmetric; by-name keys track their options).
#[test]
fn option_permutation_tracks_options() {
    let mut engine = build_engine(1.0, 0.5);
    let fwd = engine
        .decide(&choice_request(
            "the customer wants their invoice balance back",
            "what does the customer want?",
            NAMES,
        ))
        .unwrap();
    let rev = engine
        .decide(&choice_request(
            "the customer wants their invoice balance back",
            "what does the customer want?",
            [NAMES[1], NAMES[0]],
        ))
        .unwrap();
    let pf: &[f32] = &fwd.answers[0].probabilities;
    let pr: &[f32] = &rev.answers[0].probabilities;
    assert_eq!(pf.len(), 2);
    assert!(
        (pf[0] - pr[1]).abs() <= 1e-5 && (pf[1] - pr[0]).abs() <= 1e-5,
        "permuted options must permute probs: {pf:?} vs {pr:?}"
    );
}

/// Determinism: the armed engine is a deterministic fold — two identical
/// requests answer byte-identically (the kernel's own stability law,
/// pinned at the seam).
#[test]
fn armed_is_deterministic() {
    let mut engine = build_engine(1.0, 0.5);
    let req = choice_request(
        "the customer wants their invoice balance back",
        "what does the customer want?",
        NAMES,
    );
    let a = engine.decide(&req).unwrap();
    let b = engine.decide(&req).unwrap();
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}
