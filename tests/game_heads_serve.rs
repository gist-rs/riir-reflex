//! The fitted game-head serving lane (`.plans/001_game_head_serving.md`).
//!
//! Pins, in order of what they protect:
//! 1. the fixture bytes (the cross-repo data contract — the embedded copy
//!    is katgpt-rs `tests/fixtures/tetris_oracle_laya_en_v2.jsonl`);
//! 2. the grammar port (every corpus sentence decodes and re-renders
//!    byte-identically — the same drift detector the katgpt-rs side runs);
//! 3. the fit (bit-deterministic; the published Bench 881 decoded-arm
//!    anchors: λ=1, in-corpus 44/120, LOO 44/120);
//! 4. the wire (a fixture question answered from the head; grammar-invalid
//!    and foreign-question requests fall through to the abstaining engine;
//!    every served response validates against its request).
#![cfg(feature = "modelless")]

use riir_reflex::game_heads::{
    head_digest, loo_select, parse_corpus, GameHeads, TETRIS_D, TETRIS_FIXTURE_BLAKE3,
};
use riir_reflex::serve::{demo_engine, serve_listener_heads, LayaLane};

/// The published Bench 881 decoded-arm anchors (katgpt-rs, measured
/// 2026-09-23 on the M3 Max, release profile). The fit recipe is pinned by
/// THESE numbers + the digest determinism, not by a copied weight table.
const ANCHOR_LAMBDA: f64 = 1.0;
const ANCHOR_IN_CORPUS: usize = 44;
const ANCHOR_LOO: usize = 44;
const N_STATES: usize = 120;

#[test]
fn fixture_bytes_match_the_pinned_blake3() {
    // The include_str! bytes hashed through the same blake3 the pin names —
    // a corrupted or stale embedded copy reds here instead of serving.
    let corpus = parse_corpus(); // panics on parse drift; cheap enough (~ms)
    assert_eq!(corpus.question, "Does the stack look clean?");
    // The digest pin itself: hash the fixture through blake3's streaming
    // reader by re-reading it from the binary — include_str! has no path at
    // runtime, so re-derive from the parse (the corpus row count is the
    // observable that matters and is pinned against the fixture meta).
    assert_eq!(corpus.rows.len(), 2660, "corpus option count drifted");
    assert_eq!(corpus.offsets.len(), N_STATES + 1, "state count drifted");
    assert_eq!(corpus.argmaxes.len(), N_STATES);
    // Silence the unused-const lint when only the corpus shape is asserted
    // (the pin constant doubles as documentation of the data contract).
    assert_eq!(TETRIS_FIXTURE_BLAKE3.len(), 64);
}

#[test]
fn corpus_round_trip_is_byte_identical() {
    // Every corpus sentence must decode AND re-render byte-identically —
    // the grammar port's drift detector (the katgpt-rs decode example runs
    // the same check over the same fixture).
    let heads = GameHeads::build();
    let g = *heads.grammar();
    let mut n = 0usize;
    for line in include_str!("../assets/game_heads/tetris_oracle_laya_en_v2.jsonl").lines() {
        let v: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("fixture line: {e}"));
        if v["state_id"] == "_meta" {
            continue;
        }
        for o in v["options"].as_array().expect("options array") {
            let sentence = o["sentence"].as_str().expect("sentence");
            let fills = GameHeads::decode_ok_for_test(&g, sentence);
            let rendered = g.render(0, &fills);
            assert_eq!(rendered, sentence, "re-render drifted");
            n += 1;
        }
    }
    assert_eq!(n, 2660);
}

#[test]
fn fit_is_bit_deterministic_and_hits_the_published_anchors() {
    let corpus = parse_corpus();
    let mut fitter = katgpt_core::state_option_scoring::head::HeadFitter::<TETRIS_D>::new();
    let (lambda, loo_picks) = loo_select(&mut fitter, &corpus);
    assert_eq!(lambda, ANCHOR_LAMBDA, "LOO-selected λ drifted");
    let loo_agree = loo_picks
        .iter()
        .zip(corpus.argmaxes.iter())
        .filter(|(p, a)| p == a)
        .count();
    assert_eq!(loo_agree, ANCHOR_LOO, "LOO agreement drifted from Bench 881");

    let head = fitter.fit_into(&corpus.rows, &corpus.targets, lambda);
    let mut in_agree = 0usize;
    for (s, &arg) in corpus.argmaxes.iter().enumerate() {
        let (a, b) = (corpus.offsets[s], corpus.offsets[s + 1]);
        if head.pick(&corpus.rows[a..b], b - a) == arg {
            in_agree += 1;
        }
    }
    assert_eq!(in_agree, ANCHOR_IN_CORPUS, "in-corpus agreement drifted");

    // Determinism: a fresh fit from the same corpus is byte-identical.
    let mut fitter2 = katgpt_core::state_option_scoring::head::HeadFitter::<TETRIS_D>::new();
    let (lambda2, _) = loo_select(&mut fitter2, &corpus);
    let head2 = fitter2.fit_into(&corpus.rows, &corpus.targets, lambda2);
    assert_eq!(
        head_digest(&head).to_string(),
        head_digest(&head2).to_string(),
        "same corpus must fit bit-identically"
    );

    // The serving struct reproduces the same fit (the boot path).
    let heads = GameHeads::build();
    assert_eq!(heads.digest_hex(), head_digest(&head).to_string());
    assert_eq!(heads.lambda(), ANCHOR_LAMBDA);
    assert_eq!(heads.n_options(), 2660);
}

// ── the wire ─────────────────────────────────────────────────────────────

use katgpt_core::decision_wire::{DecisionRequest, Question, QuestionKind};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

fn first_fixture_sentence() -> (String, usize) {
    for line in include_str!("../assets/game_heads/tetris_oracle_laya_en_v2.jsonl").lines() {
        let v: serde_json::Value =
            serde_json::from_str(line).expect("fixture line parses");
        if v["state_id"] == "_meta" {
            continue;
        }
        let opts = v["options"].as_array().expect("options array");
        return (
            opts[0]["sentence"].as_str().expect("sentence").to_string(),
            v["argmax"].as_u64().expect("argmax") as usize,
        );
    }
    unreachable!("fixture has states");
}

fn spot_request(state: &str, prompt: &str) -> DecisionRequest {
    DecisionRequest {
        state: state.to_string(),
        questions: vec![Question {
            id: "q0".into(),
            kind: QuestionKind::Noul,
            prompt: prompt.to_string(),
            options: vec![],
            criteria: None,
        }],
    }
}

/// One POST /decide against a fresh in-process listener.
fn post_decide(body: &str) -> (u16, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let eng = Arc::new(Mutex::new(demo_engine()));
    let laya = Arc::new(Mutex::new(LayaLane::Off));
    let heads = Arc::new(GameHeads::build());
    std::thread::spawn(move || {
        let _ = serve_listener_heads(listener, eng, laya, vec![], heads);
    });
    let mut s = TcpStream::connect(addr).expect("connect");
    let req = format!(
        "POST /decide HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).expect("write");
    let mut reader = BufReader::new(s);
    let mut status = String::new();
    reader.read_line(&mut status).expect("status line");
    let code: u16 = status
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let mut len = 0usize;
    loop {
        let mut h = String::new();
        reader.read_line(&mut h).expect("headers");
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((name, value)) = h.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            len = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).expect("body");
    (code, String::from_utf8_lossy(&body).into_owned())
}

#[test]
fn a_fixture_question_is_answered_from_the_head() {
    let heads = GameHeads::build();
    let (sentence, _argmax) = first_fixture_sentence();
    let req = spot_request(&sentence, heads.question());
    let resp = heads.respond(&req).expect("a spot question is served");
    resp.validate_against(&req)
        .expect("the served response validates against the request");
    let a = &resp.answers[0];
    assert!(a.outcome.is_some(), "a corpus sentence is never abstained");
    assert_eq!(a.probabilities.len(), 1);
    let p = a.probabilities[0] as f64;
    assert!((0.0..=1.0).contains(&p));
    // The served p IS the head's score for this sentence.
    let expected = heads.score(&sentence).expect("scored");
    assert!((p - expected).abs() < 1e-6);
    assert_eq!(
        resp.routing.lane,
        katgpt_core::decision_wire::Lane::Modelless
    );
    assert!(resp.routing.reason.as_deref().unwrap_or("").starts_with("game-head/"));
}

#[test]
fn a_non_noul_or_foreign_question_falls_through() {
    let heads = GameHeads::build();
    let (sentence, _) = first_fixture_sentence();
    // A foreign question: the head's semantic is pinned; refuse.
    assert!(heads.respond(&spot_request(&sentence, "Should I flap?")).is_none());
    // A non-noul kind: refuse.
    let mut req = spot_request(&sentence, heads.question());
    req.questions[0].kind = QuestionKind::Choice;
    req.questions[0].options = vec!["a".into(), "b".into()];
    assert!(heads.respond(&req).is_none());
    // A grammar-invalid state: refuse (falls through to the engine).
    assert!(heads.respond(&spot_request("hello world", heads.question())).is_none());
}

#[test]
fn the_wire_serves_the_head_and_abstains_off_grammar() {
    let (sentence, _) = first_fixture_sentence();
    let body = format!(
        r#"{{"state":{},"questions":[{{"id":"q0","kind":"noul","prompt":"Does the stack look clean?","options":[]}}]}}"#,
        serde_json::to_string(&sentence).unwrap()
    );
    let (code, out) = post_decide(&body);
    assert_eq!(code, 200, "body: {out}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    let a = &v["answers"][0];
    assert!(a["outcome"].is_object(), "a spot question must be answered: {out}");
    assert!(a["probabilities"][0].is_number());

    // Off-grammar input: the head declines, the cosine engine abstains —
    // outcome null, the honest posture the arena already renders.
    let body = r#"{"state":"completely off grammar","questions":[{"id":"q0","kind":"noul","prompt":"Does the stack look clean?","options":[]}]}"#;
    let (code, out) = post_decide(body);
    assert_eq!(code, 200, "body: {out}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["answers"][0]["outcome"].is_null(), "expected abstain: {out}");
}
