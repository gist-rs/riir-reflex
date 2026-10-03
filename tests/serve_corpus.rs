//! Issue 063: `RIIR_REFLEX_CORPUS` — the shipped binary serves a user
//! corpus. The loader's contract lives in `src/corpus.rs`'s unit tests;
//! this file pins the SERVE-side integration: the sample corpus builds a
//! 3-domain engine through the same builder the boot dispatch uses, an
//! in-corpus question gets a non-null answer, and `/healthz` discloses the
//! corpus posture (`"demo"` for the legacy funnel, the domain list for a
//! corpus boot).
#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, Question};
use riir_reflex::corpus;
use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::DecisionEngine;
use riir_reflex::serve::{CorpusInfo, LayaLane, serve_listener_heads, serve_listener_heads_corpus};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn sample_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join("first_corpus")
}

/// The engine exactly as `serve_boot` builds it (same builder, the
/// distance-only corpus posture) for a 3-domain corpus.
fn sample_engine() -> DecisionEngine<3, EMBED_DIM> {
    let loaded = corpus::load_dir(&sample_dir()).expect("sample corpus loads");
    assert_eq!(loaded.domains, vec!["billing", "deploy", "onboarding"]);
    let cfg = riir_reflex::engine::EngineConfig {
        score_threshold: 0.0, // the serve_boot corpus posture
        ..riir_reflex::engine::EngineConfig::default()
    };
    DecisionEngine::build_specs(loaded.specs, cfg).expect("sample corpus builds")
}

fn route_request(state: &str) -> DecisionRequest {
    DecisionRequest {
        state: state.to_string(),
        questions: vec![Question::choice(
            "route",
            "Route this ticket to the team that owns it.",
            vec![
                "billing".to_string(),
                "deploy".to_string(),
                "onboarding".to_string(),
            ],
            None,
        )],
    }
}

#[test]
fn sample_corpus_answers_an_in_corpus_question() {
    let mut eng = sample_engine();
    // In-corpus deploy state (the rollback doc's own vocabulary).
    let req = route_request(
        "our deploy regressed the error budget after the rollout, run the rollback \
         and verify the health endpoints",
    );
    let resp = eng.decide(&req).expect("decide");
    let answer = &resp.answers[0];
    assert!(
        answer.outcome.is_some(),
        "an in-corpus question must answer non-null, got {answer:?}"
    );
    assert!(
        answer.confidence > 0.0,
        "an answered question carries a positive confidence, got {answer:?}"
    );
    match answer.outcome {
        Some(katgpt_core::decision_wire::Outcome::Choice { index }) => {
            assert_eq!(index, 1, "the deploy domain owns the rollback question");
        }
        other => panic!("expected a choice outcome, got {other:?}"),
    }
    let reason = resp.routing.reason.as_deref().unwrap_or_default();
    assert!(
        reason.contains("deploy"),
        "the routing reason names the winning domain, got: {reason}"
    );
}

#[test]
fn sample_corpus_answers_deterministically() {
    let mut a = sample_engine();
    let mut b = sample_engine();
    let req = route_request(
        "the customer was charged twice on the same invoice and wants the duplicate \
         charge refunded to the original payment method",
    );
    let ra = a.decide(&req).expect("decide a");
    let rb = b.decide(&req).expect("decide b");
    assert_eq!(
        serde_json::to_string(&ra).unwrap(),
        serde_json::to_string(&rb).unwrap(),
        "same corpus, same question → byte-identical answers"
    );
    assert!(ra.answers[0].outcome.is_some(), "in-corpus billing answers");
}

#[test]
fn off_corpus_questions_still_abstain() {
    let mut eng = sample_engine();
    let req = route_request(
        "my sourdough starter stopped rising after i moved it to a colder kitchen",
    );
    let resp = eng.decide(&req).expect("decide");
    assert!(
        resp.answers[0].outcome.is_none(),
        "an off-corpus question abstains, got {:?}",
        resp.answers[0]
    );
}

// ── The /healthz corpus disclosure ───────────────────────────────────────

fn http_get(addr: &str, target: &str) -> String {
    let mut s = TcpStream::connect(addr).expect("connect");
    s.write_all(format!("GET {target} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").as_bytes())
        .unwrap();
    let mut body = String::new();
    for line in BufReader::new(s).lines() {
        body.push_str(&line.expect("body line"));
    }
    body
}

#[test]
fn healthz_discloses_the_corpus_domains_on_a_corpus_boot() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let eng = Arc::new(Mutex::new(sample_engine()));
    let laya = Arc::new(Mutex::new(LayaLane::Off));
    let heads = Arc::new(riir_reflex::game_heads::GameHeads::absent());
    std::thread::spawn(move || {
        let _ = serve_listener_heads_corpus(
            listener,
            eng,
            laya,
            vec![],
            heads,
            &CorpusInfo::Domains(vec!["billing".into(), "deploy".into(), "onboarding".into()]),
        );
    });
    let resp = http_get(&addr, "/healthz");
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(
        resp.contains("\"corpus\":{\"domains\":[\"billing\",\"deploy\",\"onboarding\"]}"),
        "healthz discloses the corpus domains, got: {resp}"
    );
}

#[test]
fn healthz_discloses_demo_on_the_legacy_funnel() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let eng = Arc::new(Mutex::new(riir_reflex::serve::demo_engine()));
    let laya = Arc::new(Mutex::new(LayaLane::Off));
    let heads = Arc::new(riir_reflex::game_heads::GameHeads::absent());
    std::thread::spawn(move || {
        let _ = serve_listener_heads(listener, eng, laya, vec![], heads);
    });
    let resp = http_get(&addr, "/healthz");
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(
        resp.contains("\"corpus\":\"demo\""),
        "healthz discloses the demo posture, got: {resp}"
    );
}
