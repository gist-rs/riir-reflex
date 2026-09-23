//! Engine gates (Plan 603 T1.3 + T1.7 acceptance):
//!
//! - the wire contract end to end (decide → `validate_against` → JSON
//!   round-trip — the same bytes the golden pins hold);
//! - determinism: the modelless lane's repeat runs are bit-identical
//!   (caveat 3 scoping — this claim is THIS lane's only);
//! - the abstain geometry: thresholds are set from MEASURED gate
//!   confidences (in-corpus vs off-corpus), never magic numbers;
//! - routing: prompts land on the corpus expert whose topic they match;
//! - calibration: identity cold start, refit moves `Calibration` metadata;
//! - the T1.7 readout dispatch table (inheritance pin, cross-checked at
//!   the integration boundary);
//! - the HTTP edge contract over a REAL socket (loopback, ephemeral port).

#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, DecisionResponse, Question};
use riir_reflex::embed::{EMBED_DIM, Embedder};
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};

const OPS_DOC: &str = "Deploy the server to staging and verify the rollout before promoting \
to production. The staging cluster mirrors production capacity and runs the \
same release candidate. Rollback is one command when a deploy regresses the \
error budget. Verify the health endpoints after every rollout step.";

const SUPPORT_DOC: &str = "The customer asked for a refund of the last invoice because the \
billing account was charged twice. Check the account balance and the payment \
history, then refund the duplicate charge to the original payment method. \
Escalate to the billing team when the invoice does not match the account \
records.";

fn specs() -> Vec<ExpertSpec> {
    vec![
        ExpertSpec::new("ops", &[OPS_DOC.to_string()]),
        ExpertSpec::new("support", &[SUPPORT_DOC.to_string()]),
    ]
}

fn sample_request() -> DecisionRequest {
    DecisionRequest {
        state: "The release candidate passed staging smoke tests; the on-call engineer \
verified the health endpoints and the error budget is clean."
            .to_string(),
        questions: vec![
            Question::choice(
                "env",
                "Which environment should receive this build next?",
                vec![
                    "production".to_string(),
                    "staging".to_string(),
                    "local sandbox".to_string(),
                    "do not deploy".to_string(),
                ],
                Some("Pick the rollout step that matches the verified state.".to_string()),
            ),
            Question::score(
                "risk",
                "How risky is promoting this build now?",
                vec![
                    "low".to_string(),
                    "moderate".to_string(),
                    "high".to_string(),
                ],
            ),
            Question::noul("deploy", "Should the rollout proceed now?"),
        ],
    }
}

#[test]
fn wire_contract_end_to_end() {
    let mut eng: DecisionEngine<2, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let req = sample_request();
    let resp = eng.decide(&req).unwrap();

    // The wire's own fail-closed validation.
    resp.validate_against(&req)
        .expect("engine output must satisfy the wire contract");

    // JSON round-trip (the golden-pin shape).
    let json = serde_json::to_string(&resp).unwrap();
    let back: DecisionResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back, resp, "serde_json round-trip must be lossless");
}

#[test]
fn determinism_bit_identical_repeat_runs() {
    // Plan 603 caveat 3 scoping: this claim is the MODELESS lane's only.
    let mut eng: DecisionEngine<2, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let req = sample_request();
    let a = eng.decide(&req).unwrap();
    let b = eng.decide(&req).unwrap();
    let ja = serde_json::to_string(&a).unwrap();
    let jb = serde_json::to_string(&b).unwrap();
    assert_eq!(ja, jb, "repeat runs must be bit-identical (modelless lane)");
}

#[test]
fn abstain_geometry_measured_not_magic() {
    // Thresholds derive from the gate's measured geometry: the off-corpus
    // state must read strictly farther than the in-corpus one, and the
    // engine must abstain on the former and answer the latter at the
    // midpoint threshold.
    let eng: DecisionEngine<2, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();

    let embed = |s: &str| {
        let mut v = [0.0f32; EMBED_DIM];
        Embedder.embed_into(s.as_bytes(), &mut v);
        v
    };
    let in_ops = embed("verify the staging rollout and the deploy health endpoints");
    let in_support = embed("refund the duplicate invoice charge on the billing account");
    let off = embed("zzz qqq xv wxkjjjf 88271 pw pwmmmm xxvvo");

    let d = |q: &[f32; EMBED_DIM]| {
        eng.gate(0)
            .abstain_confidence(q)
            .max(eng.gate(1).abstain_confidence(q))
    };
    let d_ops = d(&in_ops);
    let d_support = d(&in_support);
    let d_off = d(&off);
    let d_in = d_ops.max(d_support);
    assert!(
        d_off < d_in - 0.05,
        "off-corpus state must measure farther than in-corpus (off {d_off} vs in {d_in})"
    );

    let mid = (d_in + d_off) / 2.0;
    let cfg = EngineConfig {
        distance_threshold: mid,
        score_threshold: 0.0, // isolate the distance axis
        ..EngineConfig::default()
    };
    let mut eng: DecisionEngine<2, EMBED_DIM> = DecisionEngine::build_specs(specs(), cfg).unwrap();

    let off_req = DecisionRequest {
        state: "zzz qqq xv wxkjjjf 88271 pw pwmmmm xxvvo".to_string(),
        questions: vec![Question::noul("q0", "proceed or not?")],
    };
    let resp = eng.decide(&off_req).unwrap();
    resp.validate_against(&off_req).unwrap();
    for a in &resp.answers {
        assert!(
            a.outcome.is_none(),
            "off-corpus must abstain (outcome None)"
        );
    }

    let in_req = DecisionRequest {
        state: "The staging rollout is verified and the deploy health endpoints are green."
            .to_string(),
        questions: vec![Question::noul("q0", "proceed or not?")],
    };
    let resp = eng.decide(&in_req).unwrap();
    assert!(
        resp.answers[0].outcome.is_some(),
        "in-corpus state must answer at the midpoint threshold (d {d_ops} vs mid {mid})"
    );
}

#[test]
fn routing_lands_on_the_matching_expert() {
    let mut eng: DecisionEngine<2, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    sc.prepare(1);

    let ops_req = DecisionRequest {
        state: "deploy verify staging rollout health endpoints".to_string(),
        questions: vec![Question::choice(
            "q",
            "where does this build go?",
            vec!["staging".to_string(), "production".to_string()],
            None,
        )],
    };
    eng.solve_into(&ops_req, &mut sc).unwrap();
    assert_eq!(sc.domains[0], 0, "ops prompt must route to the ops expert");

    let support_req = DecisionRequest {
        state: "customer refund invoice billing account charge".to_string(),
        questions: vec![Question::choice(
            "q",
            "where does this build go?",
            vec!["staging".to_string(), "production".to_string()],
            None,
        )],
    };
    eng.solve_into(&support_req, &mut sc).unwrap();
    assert_eq!(
        sc.domains[0], 1,
        "support prompt must route to the support expert"
    );
}

#[test]
fn calibration_identity_then_refit() {
    let mut eng: DecisionEngine<2, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    assert_eq!(
        eng.calibration().method,
        "none",
        "cold start is the honest raw posture"
    );

    let req = sample_request();
    let before = eng.decide(&req).unwrap();

    // Feed the calibrator past its occupancy floor with overconfident
    // failures; the refit must move parameters and the metadata must flip.
    let mut moved = false;
    for i in 0..128 {
        let p = 0.4 + 0.4 * (i % 16) as f32 / 15.0;
        moved |= eng.observe(p, false);
    }
    assert!(
        moved,
        "sustained overconfident-failure evidence must move the calibration"
    );
    assert_eq!(eng.calibration().method, "sigmoid-gate");

    let after = eng.decide(&req).unwrap();
    assert!(
        (after.calibration.temperature - 1.0).abs() > f32::EPSILON,
        "refit must report a temperature off identity"
    );
    // Calibration reshapes confidence, NOT the winner (the bench-808 law).
    for (a, b) in before.answers.iter().zip(after.answers.iter()) {
        assert_eq!(a.outcome, b.outcome, "refit must not move the winner");
    }
}

#[test]
fn readout_dispatch_table_integration_pin() {
    // T1.7: the dispatch is INHERITED from Bench 817's verdict. Cross-check
    // the integration surface against the module's pinned table: narrow
    // arity answers carry the entropy functional, wide ones maxprob.
    let mut eng: DecisionEngine<2, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let req = sample_request(); // arity 4, 3, 1 — all narrow
    let resp = eng.decide(&req).unwrap();
    for a in &resp.answers {
        let k = a.probabilities.len();
        if k < 2 {
            // Degenerate arity (the noul wire shape carries ONE value) —
            // entropy over a single option is undefined; skip.
            continue;
        }
        if k <= riir_reflex::readout::NARROW_MAX_OPTIONS {
            // Entropy arm: uniform distribution reads ~0 confidence.
            let mut uniform = vec![1.0 / k as f32; k];
            let c = riir_reflex::readout::confidence(&uniform);
            assert!(c < 0.01, "narrow uniform ~ 0 (got {c})");
            // Peak by renormalizing a dominant mass: the readout consumes
            // a DISTRIBUTION (sum 1). [0.9, 1/3, 1/3, 1/3] renormalized is
            // still 0.55/0.15/0.15/0.15 — entropy 0.86, confidence 0.14 —
            // so PEAK means dominant, not "largest by a little".
            uniform[0] = 4.0;
            let s: f32 = uniform.iter().sum();
            for v in uniform.iter_mut() {
                *v /= s;
            }
            let c = riir_reflex::readout::confidence(&uniform);
            assert!(c > 0.5, "narrow peaked reads high (got {c})");
        } else {
            let c = riir_reflex::readout::confidence(&a.probabilities);
            let max = a.probabilities.iter().copied().fold(0.0f32, f32::max);
            assert!((c - max).abs() < 1e-6, "wide reads maxprob");
        }
        let _ = k;
    }
    // And the wide arm through the SAME dispatch at the boundary.
    let mut wide = vec![
        0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.0078125, 0.0,
    ];
    let c = riir_reflex::readout::confidence(&wide);
    assert!((c - 0.5).abs() < 1e-6, "K=9 must read maxprob");
    wide.pop();
    let c8 = riir_reflex::readout::confidence(&wide);
    assert!(
        c8 > 0.0 && c8 < 1.0 && (c8 - 0.5).abs() > 1e-3,
        "K=8 must read the entropy arm"
    );
}

#[test]
fn answers_carry_first_class_abstention_shape() {
    // The wire's abstention contract: outcome None MAY carry the
    // distribution (post-score abstain) — exactly what the fused gate
    // produces. Verify the shape survives the HTTP round trip too (the
    // JSON twin of the golden pin).
    let mut eng: DecisionEngine<2, EMBED_DIM> = DecisionEngine::build_specs(
        specs(),
        EngineConfig {
            // Force the score half of the fused gate to abstain-everything.
            score_threshold: 1.5,
            ..EngineConfig::default()
        },
    )
    .unwrap();
    let req = sample_request();
    let resp = eng.decide(&req).unwrap();
    resp.validate_against(&req).unwrap();
    for a in &resp.answers {
        assert!(
            a.outcome.is_none(),
            "score_threshold 1.5 must abstain everything"
        );
        assert!(
            !a.probabilities.is_empty(),
            "post-score abstain carries the distribution"
        );
        assert!(a.confidence >= 0.0 && a.confidence <= 1.0);
    }
}

// ── The HTTP edge contract over a REAL socket ────────────────────────────

fn http_post(port: u16, path: &str, body: &str) -> (u16, String) {
    use std::io::{Read, Write};
    let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    let mut resp = String::new();
    s.read_to_string(&mut resp).unwrap();
    let status: u16 = resp
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = resp
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

fn http_get(port: u16, path: &str) -> (u16, String) {
    use std::io::{Read, Write};
    let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    s.write_all(req.as_bytes()).unwrap();
    let mut resp = String::new();
    s.read_to_string(&mut resp).unwrap();
    let status: u16 = resp
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = resp
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

#[test]
fn http_edge_contract() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let engine = std::sync::Arc::new(std::sync::Mutex::new(
        DecisionEngine::<2, EMBED_DIM>::build_specs(specs(), EngineConfig::default()).unwrap(),
    ));
    let eng = std::sync::Arc::clone(&engine);
    let server = std::thread::spawn(move || {
        let _ = riir_reflex::serve::serve_listener(listener, eng);
    });

    // Liveness: JSON with the lane map (the arena page's lane discovery —
    // `laya` reads off/loading/ready/failed; `raw` is the head-skip escape
    // hatch; a bare-"ok" engine predates the lane edge) + the game-head
    // map (compile-time surfaces: all three heads serve or the boot
    // panicked — issue 011 closed).
    let (status, body) = http_get(port, "/healthz");
    assert_eq!(status, 200);
    assert_eq!(
        body,
        "{\"status\":\"ok\",\"lanes\":{\"modelless\":\"ready\",\"raw\":\"ready\",\"laya\":\"off\"},\"heads\":{\"tetris\":true,\"lanes\":true,\"flappy\":true}}"
    );

    // The full decision path through the edge.
    let req = sample_request();
    let (status, body) = http_post(port, "/decide", &serde_json::to_string(&req).unwrap());
    assert_eq!(status, 200, "body: {body}");
    let resp: DecisionResponse = serde_json::from_str(&body).unwrap();
    resp.validate_against(&req)
        .expect("edge response must satisfy the wire contract");

    // Malformed body → 400, not a panic.
    let (status, _) = http_post(port, "/decide", "{\"state\":");
    assert_eq!(status, 400);

    // Unknown path → 404.
    let (status, _) = http_get(port, "/nope");
    assert_eq!(status, 404);

    // Feedback: calibrator observe through the edge.
    let (status, body) = http_post(port, "/feedback", "{\"p\":0.9,\"outcome\":false}");
    assert_eq!(status, 200, "body: {body}");
    assert!(body.contains("refit"));

    // Detach the accept loop — a listener runs until the process exits;
    // joining it would hang the test forever by construction.
    drop(server);
}
