//! The laya-lane HTTP edge (opt-in `RIIR_REFLEX_LAYA=1`): the edge answers
//! `X-Reflex-Lane: laya` requests FAIL-CLOSED in every non-ready state —
//! off → 503 naming the env, loading → 503 retry, failed → 500 with the
//! reason, unknown lane → 400 — never a silent modelless fallback (a
//! silently-served wrong lane would poison the arena's per-lane claims).
//! `/healthz` reports the lane's state. The Ready path's ANSWER MAPPING is
//! unit-tested weights-free under the feature (the forward itself is
//! G5-parity-pinned elsewhere; no test here needs weights).
#![cfg(feature = "modelless")]

use riir_reflex::serve::{LayaLane, demo_engine, serve_listener_with};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

fn spawn_lanes(laya: LayaLane) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr").to_string();
    let eng = Arc::new(Mutex::new(demo_engine()));
    let laya = Arc::new(Mutex::new(laya));
    std::thread::spawn(move || {
        let _ = serve_listener_with(listener, eng, laya, vec![]);
    });
    addr
}

fn roundtrip(addr: &str, raw: &str) -> String {
    let mut s = TcpStream::connect(addr).expect("connect");
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
            l.strip_prefix("Content-Length:")
                .map(|v| v.trim().parse::<usize>().ok())
        })
        .flatten()
    {
        let mut body = vec![0u8; cl];
        let _ = reader.read_exact(&mut body);
        buf.push_str(&String::from_utf8_lossy(&body));
    }
    buf
}

const BODY: &str = r#"{"state":"Deploy the server to staging","questions":[{"id":"q0","kind":"noul","prompt":"Roll back or promote?","options":[]}]}"#;

fn post_body(addr: &str, body: &str, lane: Option<&str>) -> String {
    let lane_hdr = lane
        .map(|l| format!("X-Reflex-Lane: {l}\r\n"))
        .unwrap_or_default();
    roundtrip(
        addr,
        &format!(
            "POST /decide HTTP/1.1\r\nHost: x\r\n{lane_hdr}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    )
}

fn post_decide(addr: &str, lane: Option<&str>) -> String {
    post_body(addr, BODY, lane)
}

#[test]
fn laya_off_is_fail_closed_503_naming_the_env() {
    let addr = spawn_lanes(LayaLane::Off);
    let resp = post_decide(&addr, Some("laya"));
    assert!(resp.starts_with("HTTP/1.1 503"), "got: {resp}");
    assert!(resp.contains("RIIR_REFLEX_LAYA=1"), "got: {resp}");
    assert!(
        !resp.contains("routing"),
        "no modelless response leaked: {resp}"
    );
}

#[test]
fn laya_loading_is_503_retry() {
    let addr = spawn_lanes(LayaLane::Loading);
    let resp = post_decide(&addr, Some("laya"));
    assert!(resp.starts_with("HTTP/1.1 503"), "got: {resp}");
    assert!(resp.contains("still loading"), "got: {resp}");
}

#[test]
fn laya_failed_is_500_with_reason() {
    let addr = spawn_lanes(LayaLane::Failed("weights sha mismatch".to_string()));
    let resp = post_decide(&addr, Some("laya"));
    assert!(resp.starts_with("HTTP/1.1 500"), "got: {resp}");
    assert!(resp.contains("weights sha mismatch"), "got: {resp}");
}

#[test]
fn unknown_lane_is_400() {
    let addr = spawn_lanes(LayaLane::Off);
    let resp = post_decide(&addr, Some("candle"));
    assert!(resp.starts_with("HTTP/1.1 400"), "got: {resp}");
    assert!(resp.contains("unknown lane"), "got: {resp}");
}

#[test]
fn healthz_reports_laya_state() {
    let addr = spawn_lanes(LayaLane::Loading);
    let resp = roundtrip(&addr, "GET /healthz HTTP/1.1\r\nHost: x\r\n\r\n");
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(resp.contains("\"status\":\"ok\""), "got: {resp}");
    assert!(resp.contains("\"modelless\":\"ready\""), "got: {resp}");
    assert!(
        resp.contains("\"raw\":\"ready\""),
        "the raw lane is advertised: {resp}"
    );
    assert!(resp.contains("\"laya\":\"loading\""), "got: {resp}");
}

#[test]
fn modelless_default_unchanged_without_header() {
    let addr = spawn_lanes(LayaLane::Off);
    let resp = post_decide(&addr, None);
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(resp.contains("\"lane\":\"modelless\""), "got: {resp}");
}

#[test]
fn modelless_lane_spelling_serves_modelless() {
    let addr = spawn_lanes(LayaLane::Off);
    let resp = post_decide(&addr, Some("modelless"));
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(resp.contains("\"lane\":\"modelless\""), "got: {resp}");
}

// ── the raw lane (issue 014) ────────────────────────────────────────────

/// The first fixture sentence (the same reader `game_heads_serve.rs` uses):
/// the one request shape where the default and raw lanes visibly differ —
/// the head answers it by default, the raw engine abstains off its corpus.
fn first_fixture_sentence() -> String {
    for line in include_str!("../assets/game_heads/tetris_oracle_laya_en_v2.jsonl").lines() {
        let v: serde_json::Value = serde_json::from_str(line).expect("fixture line parses");
        if v["state_id"] == "_meta" {
            continue;
        }
        return v["options"].as_array().expect("options")[0]["sentence"]
            .as_str()
            .expect("sentence")
            .to_string();
    }
    unreachable!("fixture has states");
}

fn spot_body(state: &str, prompt: &str) -> String {
    format!(
        r#"{{"state":{},"questions":[{{"id":"q0","kind":"noul","prompt":{},"options":[]}}]}}"#,
        serde_json::to_string(state).unwrap(),
        serde_json::to_string(prompt).unwrap()
    )
}

/// The paired smoke, BOTH directions on the request where the lanes
/// actually diverge: the fixture spot question is answered head-first by
/// default and answered by the raw cosine engine (an abstain off the demo
/// corpus) under `X-Reflex-Lane: raw` — never a silent fallback between
/// the two, and the raw response never claims the head's work (the
/// per-lane-claims law).
#[test]
fn raw_lane_skips_the_head_and_the_default_serves_it() {
    let addr = spawn_lanes(LayaLane::Off);
    let body = spot_body(&first_fixture_sentence(), "Does the stack look clean?");

    let resp = post_body(&addr, &body, None);
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(
        resp.contains("game-head/"),
        "default must serve the head: {resp}"
    );

    let resp = post_body(&addr, &body, Some("raw"));
    assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
    assert!(
        !resp.contains("game-head/"),
        "raw must not claim the head: {resp}"
    );
    assert!(resp.contains("\"lane\":\"modelless\""), "got: {resp}");
    assert!(
        resp.contains("\"outcome\":null"),
        "the raw engine abstains off its demo corpus: {resp}"
    );
}

/// A flappy and a lanes question are foreign prompts the head refuses
/// (`.issues/011`'s engine-side serving TODO). Under `raw` both are the
/// modelless engine's own abstain — the honest baseline the arena's third
/// board renders. The WITHOUT-header side is deliberately NOT pinned here:
/// today it falls through to the same abstain, and after `.issues/011`
/// lands it serves the head — either way `raw` stays the escape hatch.
#[test]
fn raw_lane_serves_foreign_questions_as_the_modelless_baseline() {
    let addr = spawn_lanes(LayaLane::Off);
    let sentence = first_fixture_sentence();
    for prompt in ["Should I flap?", "Which lane should I take?"] {
        let resp = post_body(&addr, &spot_body(&sentence, prompt), Some("raw"));
        assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp}");
        assert!(
            !resp.contains("game-head/"),
            "{prompt} raw must not claim the head: {resp}"
        );
        assert!(
            resp.contains("\"outcome\":null"),
            "{prompt} expected the honest abstain: {resp}"
        );
    }
}

/// The Ready path's wire mapping, weights-free: synthetic laya answers →
/// decision_wire (choice index by first-argmax, noul p(yes) passthrough,
/// laya routing + temperature disclosure). Same gating as the loader.
#[cfg(feature = "laya-riir")]
mod mapping {
    use katgpt_core::decision_wire::{DecisionRequest, Lane, Outcome, Question, QuestionKind};
    use riir_reflex::laya::types::Answer as LayaAnswer;
    use riir_reflex::serve::laya_serve::map_answers;

    fn req(kinds: &[QuestionKind]) -> DecisionRequest {
        DecisionRequest {
            state: "s".into(),
            questions: kinds
                .iter()
                .enumerate()
                .map(|(i, k)| Question {
                    id: format!("q{i}"),
                    kind: *k,
                    prompt: "p?".into(),
                    options: match k {
                        QuestionKind::Noul => vec![],
                        _ => vec!["a".into(), "b".into(), "c".into()],
                    },
                    criteria: None,
                })
                .collect(),
        }
    }

    fn laya_answer(
        qid: &str,
        t: &'static str,
        probs: Vec<(&str, f64)>,
        noul: Option<f64>,
    ) -> LayaAnswer {
        LayaAnswer {
            qid: qid.into(),
            t,
            choice: None,
            score: None,
            noul,
            probabilities: probs.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            confidence: 0.75,
            act_probability: 1.0,
            temperature_used: 1.7,
        }
    }

    #[test]
    fn choice_maps_first_argmax_and_option_order() {
        let r = req(&[QuestionKind::Choice]);
        let answers = vec![laya_answer(
            "q0",
            "choice",
            vec![("a", 0.2), ("b", 0.5), ("c", 0.3)],
            None,
        )];
        let resp = map_answers(&r, &answers).expect("map");
        assert_eq!(resp.routing.lane, Lane::Laya);
        assert_eq!(resp.calibration.temperature, 1.7);
        let a = &resp.answers[0];
        assert_eq!(a.outcome, Some(Outcome::Choice { index: 1 }));
        assert_eq!(a.probabilities, vec![0.2, 0.5, 0.3]);
    }

    #[test]
    fn choice_ties_keep_the_earliest_index() {
        let r = req(&[QuestionKind::Choice]);
        let answers = vec![laya_answer(
            "q0",
            "choice",
            vec![("a", 0.4), ("b", 0.4), ("c", 0.2)],
            None,
        )];
        let resp = map_answers(&r, &answers).expect("map");
        assert_eq!(resp.answers[0].outcome, Some(Outcome::Choice { index: 0 }));
    }

    #[test]
    fn noul_maps_p_yes_and_the_yes_flag() {
        let r = req(&[QuestionKind::Noul]);
        let answers = vec![laya_answer("q0", "noul", vec![], Some(0.83))];
        let resp = map_answers(&r, &answers).expect("map");
        assert_eq!(resp.answers[0].outcome, Some(Outcome::Noul { yes: true }));
        assert_eq!(resp.answers[0].probabilities, vec![0.83]);
    }

    #[test]
    fn arity_mismatch_is_an_error_not_a_guess() {
        let r = req(&[QuestionKind::Noul]);
        let answers: Vec<LayaAnswer> = vec![];
        assert!(map_answers(&r, &answers).is_err());
    }

    #[test]
    fn score_maps_the_argmax_level() {
        let r = req(&[QuestionKind::Score]);
        let answers = vec![laya_answer(
            "q0",
            "score",
            vec![("0", 0.1), ("1", 0.3), ("2", 0.6)],
            None,
        )];
        let resp = map_answers(&r, &answers).expect("map");
        assert_eq!(resp.answers[0].outcome, Some(Outcome::Score { level: 2 }));
    }
}
