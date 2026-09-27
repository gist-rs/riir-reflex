//! Thai posture pins (Plan 003 Phase 1 — T1.1 + T1.2; tests only, zero
//! runtime code): the shipped modelless posture's Thai behavior is PINNED
//! as contract, so a silent engine change cannot rebrand an accidental
//! answer as safe degradation.
//!
//! - T1.1 — modelless posture pin: the G5 `ml-thai` fixture text (read from
//!   `tests/fixtures/laya_parity_v1.jsonl` VERBATIM — G-ISO-3 forbids
//!   editing it) through the embedder and the full demo engine. The
//!   MEASURED mechanism (this pin's landing recorded it — the research
//!   note's "clause-unit hash bag" guess was wrong): `embed.rs`'s `token()`
//!   trims NON-alphanumeric-ASCII edges, so a pure-Thai clause is emptied
//!   BEFORE hashing and the state embeds to the ZERO vector — the designed
//!   no-signal shape (never NaN; the `distance_abstain::unit` law) — and
//!   the distance gate abstains on it by construction (gate confidence
//!   sigmoid(8·(0−0.35)) ≈ 0.057 < 0.5).  The pin asserts the zero-vector
//!   embedding, deterministic byte-identical answers (repeat + fresh
//!   engine) and the abstain; a confident answer would be a finding —
//!   file it in `.issues/`, never tune the gate to make this pin pass.
//! - T1.2 — serve-edge safe-degradation pin: `/decide` over a REAL
//!   loopback socket (the serve_cors pattern) on the same Thai state:
//!   200, a well-formed `DecisionResponse` that validates against the
//!   request, finite confidences, no 5xx — abstain-forward expected.
//!
//! Non-contamination contract (Plan 003): G-ISO-1 (this file tests the
//! serve path's behavior but changes no runtime code), G-ISO-2/G-ISO-3
//! (no suite or fixture edits), G-ISO-4 (no new packages — `serde_json`
//! is already a crate dep).

#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, DecisionResponse, Question};
use riir_reflex::embed::{EMBED_DIM, Embedder};
use riir_reflex::engine::Scratch;
use riir_reflex::serve::{LayaLane, demo_engine, serve_listener_with};

// ── The fixture row (read, never edited — G-ISO-3) ───────────────────────

#[derive(serde::Deserialize)]
struct FixtureLine {
    id: String,
    state: String,
    #[serde(default)]
    questions: Vec<FixtureQ>,
}

#[derive(serde::Deserialize)]
struct FixtureQ {
    t: String,
    ins: String,
}

/// The G5 `ml-thai` row's exact (state, question prompt) — the multilingual
/// checkpoint's Thai ticket. Read from the frozen fixture so the pin can
/// never drift from what the G5 gate measures.
fn ml_thai() -> (String, String) {
    for line in include_str!("fixtures/laya_parity_v1.jsonl").lines() {
        let Ok(f) = serde_json::from_str::<FixtureLine>(line) else {
            continue;
        };
        if f.id == "ml-thai" {
            let q = f.questions.first().expect("ml-thai carries one question");
            assert_eq!(q.t, "noul", "the ml-thai fixture's question kind");
            return (f.state, q.ins.clone());
        }
    }
    panic!(
        "ml-thai fixture row missing from tests/fixtures/laya_parity_v1.jsonl \
         — G-ISO-3 forbids editing that file; restore the row"
    );
}

fn thai_request() -> DecisionRequest {
    let (state, prompt) = ml_thai();
    DecisionRequest {
        state,
        questions: vec![Question::noul("ml-thai-q1", prompt)],
    }
}

// ── T1.1 — the modelless posture pin ─────────────────────────────────────

#[test]
fn modelless_thai_posture_is_deterministic_and_abstains() {
    let req = thai_request();

    // The embed half: the Thai state is non-ASCII-only, so `token()` (the
    // ASCII-alphanumeric edge trim) empties every whitespace-split clause
    // BEFORE hashing — the measured mechanism is the ZERO vector, the
    // designed no-signal shape (finite, passes through un-normalized; the
    // `distance_abstain::unit` law). The research note's "clause-unit hash
    // bag" guess is corrected by this pin.
    let mut v = [0.0f32; EMBED_DIM];
    let (state, _) = ml_thai();
    Embedder.embed_into(state.as_bytes(), &mut v);
    assert!(
        v.iter().all(|x| x.is_finite()),
        "Thai embedding must be finite (the zero-vector law)"
    );
    assert!(
        v.iter().all(|x| *x == 0.0),
        "EXPECTED the zero vector for pure-Thai input (token() empties the \
         clause before hashing); a NON-zero vector means the tokenizer \
         changed — re-adjudicate this pin and the research note"
    );

    // The full-engine half, on the shipped demo posture (the binary's
    // out-of-box engine — the posture this pin owns).
    let mut eng = demo_engine();
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(req.questions.len());
    eng.solve_into(&req, &mut sc)
        .expect("the Thai request must solve");
    let slot = &sc.slots[0];
    // The honest verdict, printed on every run: THIS is the pinned posture.
    println!(
        "modelless Thai posture: abstained={} confidence={:.6} domain={:?}",
        slot.abstained,
        slot.confidence,
        sc.domains.first().map(|d| eng.domain_names()[*d])
    );
    assert!(slot.confidence.is_finite(), "no NaN confidence");
    assert!(
        slot.abstained,
        "modelless answered a Thai state with confidence {:.6} — Plan-003 \
         T1.1 FINDING: file it in .issues/ (the research note's false-\
         confidence risk), never tune the gate to make this pin pass",
        slot.confidence
    );

    // Determinism: repeat decide + a fresh engine — byte-identical wire.
    let r1 = eng.decide(&req).expect("decide");
    let r2 = eng.decide(&req).expect("decide again");
    let fresh = demo_engine().decide(&req).expect("fresh engine decide");
    let (b1, b2, b3) = (
        serde_json::to_string(&r1).expect("serialize"),
        serde_json::to_string(&r2).expect("serialize"),
        serde_json::to_string(&fresh).expect("serialize"),
    );
    assert_eq!(b1, b2, "repeat decides must be bit-identical");
    assert_eq!(b1, b3, "a fresh engine must answer byte-identically");
    r1.validate_against(&req)
        .expect("the answer must satisfy the wire contract");
}

// ── T1.2 — the serve-edge safe-degradation pin ───────────────────────────

/// The serve_cors spawn pattern, minimal: demo engine, laya lane off, no
/// allowed origins (CORS is not this pin's subject).
fn spawn_server() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr").to_string();
    let eng = std::sync::Arc::new(std::sync::Mutex::new(demo_engine()));
    let laya = std::sync::Arc::new(std::sync::Mutex::new(LayaLane::Off));
    std::thread::spawn(move || {
        let heads = std::sync::Arc::new(riir_reflex::game_heads::GameHeads::build(
            include_str!("../assets/game_heads/tetris_oracle_laya_en_v3.jsonl"),
            include_str!("../assets/game_heads/lanes_oracle_laya_en_v1.jsonl"),
            include_str!("../assets/game_heads/flappy_oracle_laya_en_v3.jsonl"),
        ));
        let _ = serve_listener_with(listener, eng, laya, Vec::new(), heads);
    });
    addr
}

/// One POST over a real loopback socket; returns the full raw response.
fn post(addr: &str, path: &str, body: &str) -> String {
    use std::io::{BufRead, BufReader, Read, Write};
    let raw = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let mut s = std::net::TcpStream::connect(addr).expect("connect");
    s.write_all(raw.as_bytes()).expect("write");
    let mut buf = String::new();
    let mut reader = BufReader::new(s);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => buf.push_str(&line),
        }
        if buf.contains("\r\n\r\n") {
            break;
        }
    }
    if let Some(cl) = buf
        .lines()
        .find_map(|l| {
            l.trim_end()
                .strip_prefix("Content-Length:")
                .map(|v| v.trim().parse::<usize>().ok())
        })
        .flatten()
    {
        let mut b = vec![0u8; cl];
        let _ = reader.read_exact(&mut b);
        buf.push_str(&String::from_utf8_lossy(&b));
    }
    buf
}

#[test]
fn serve_edge_thai_degrades_safely() {
    let req = thai_request();
    let body = serde_json::to_string(&req).expect("serialize request");
    let addr = spawn_server();

    let resp = post(&addr, "/decide", &body);
    let status = resp
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    assert!(
        status.contains(" 200 "),
        "a Thai state must never 5xx on the serve edge (got: {status})"
    );
    let json_body = resp
        .split("\r\n\r\n")
        .nth(1)
        .filter(|b| !b.trim().is_empty())
        .unwrap_or_else(|| panic!("no body in response: {status}"));
    let parsed: DecisionResponse =
        serde_json::from_str(json_body).expect("a well-formed DecisionResponse");
    parsed
        .validate_against(&req)
        .expect("the wire contract must hold over the serve edge");
    for a in &parsed.answers {
        assert!(
            a.confidence.is_finite() && (0.0..=1.0).contains(&a.confidence),
            "serve-edge confidence must be finite and in [0,1] (no NaN may \
             serialize) — got {}",
            a.confidence
        );
    }
    assert!(
        parsed.answers.iter().all(|a| a.outcome.is_none()),
        "the serve edge answered a Thai state with confidence — Plan-003 \
         T1.2 FINDING: file it in .issues/, never tune the engine silently"
    );
}
