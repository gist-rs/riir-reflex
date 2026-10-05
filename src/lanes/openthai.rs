//! The OpenThai comparison lane (Plan 003 Phase 2, T2.1–T2.5): their
//! `openthai_systemone` service (`iapp-technology/OpenThai-SystemOne`,
//! Apache-2.0, not affiliated) served on loopback, measured over HTTP —
//! the agentjev lane family's MEASURE-vs-SERVE split. Their stack serves,
//! our Rust harness measures; comparison lane, never a product lane.
//!
//! Transport (Issue 068 — the deferred T4.1 extraction, LANDED): the
//! HTTP/1.1 exchange is the shared std-only micro-client
//! ([`crate::lanes::http_mini`]); this file keeps only the wire shape
//! (bodies, answer mapping). Import law (G-ISO-4, agentjev-shaped, as
//! amended by Issue 068): imports only `crate::harness::suites` + the
//! std-only `crate::lanes::http_mini` + std + `serde_json`. **The
//! recorded trigger**: the moment any openthai module or test needs
//! `katgpt_core::decision_wire`, the lane takes
//! `openthai-lane = ["katgpt-core/decision_wire"]` (the clm shape) in the
//! same commit — G-ISO-4 yields to that rather than forcing a wrong
//! answer.
//!
//! Wire (research 003): `POST /v1/systemone` with
//! `{state, questions: {qid: {type, instructions, criteria}}}` — their
//! questions are an OBJECT keyed by qid (agentjev's is an array; the
//! recorded dialect divergence). Their spellings match ours exactly
//! (`choice` / `score` / `noul`). Response:
//! `{answers: {qid: {probabilities, confidence, choice?, abstain?}},
//! usage: {input_tokens}, model}` — probabilities renormalized over the
//! k options, abstain reported separately (their in-head slot-255
//! extension).
//!
//! Provenance posture (T2.2, the recorded divergence from agentjev): NO
//! `/api/info` endpoint — the lane's liveness probe is `GET /healthz`,
//! and the model id comes from the DECIDE response's own `model` field
//! (never a hardcoded name — the gliner law); `usage.input_tokens`
//! accumulates beside the latency columns.

use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
use crate::lanes::http_mini::{self, HttpReply};

/// The env var naming their service's base URL.
pub const SERVE_URL_ENV: &str = "OPENTHAI_SERVE_URL";
/// The default posture: loopback, their server's default port 8000.
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8000";

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// One decide round's full yield: the answer triples, how many questions
/// their abstain slot took (provenance observation — never folded into
/// the probabilities), and their `usage.input_tokens`.
pub struct DecideOutcome {
    pub answers: Vec<AnswerTriple>,
    pub abstains: usize,
    pub input_tokens: u64,
}

/// The lane: a thin loopback HTTP/1.1 client to their service.
/// std-only (the agentjev lane's hand-rolled posture; no new deps).
#[derive(Debug, Clone)]
pub struct OpenThaiLane {
    host: String,
    port: u16,
    timeout: Duration,
}

impl Default for OpenThaiLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl OpenThaiLane {
    /// Parse an `http://host:port` base URL (loopback-only posture — their
    /// server binds 127.0.0.1 beside the harness on the same box).
    pub fn from_url(url: &str) -> Self {
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(8000)),
            None => (rest.to_string(), 8000),
        };
        Self {
            host,
            port,
            timeout: Duration::from_secs(300),
        }
    }

    /// The liveness probe (T2.2): `GET /healthz` — 200 means up. There is
    /// deliberately no `/api/info` handshake; the model id is read from
    /// the decide response (see [`Self::model_of`]).
    pub fn health(&self) -> Result<(), String> {
        let (status, _headers, body) =
            self.get("/healthz").map_err(|e| format!("openthai healthz: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /healthz -> HTTP {status}: {} — is their openthai service \
                 serving at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL})?",
                String::from_utf8_lossy(&body)
            ));
        }
        Ok(())
    }

    /// The model id from a DECIDE response body (`model` field) — the
    /// provenance stamp (never a hardcoded name, the gliner law).
    pub fn model_of(body: &Value) -> String {
        body.get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("openthai-systemone")
            .to_string()
    }

    /// One decision round: build the body for `case`, POST it, map the
    /// answers back. Latency is the CALLER's wall clock (the agentjev
    /// shape); the client round-trip ms ride the outcome for the runner's
    /// eprintln.
    pub fn decide(&self, case: &SuiteCase) -> Result<(DecideOutcome, f64), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("openthai round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "openthai round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("openthai response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let abstains = count_abstains(case, &parsed)?;
        let input_tokens = parsed
            .pointer("/usage/input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        Ok((
            DecideOutcome {
                answers,
                abstains,
                input_tokens,
            },
            client_ms,
        ))
    }

    /// The raw response body for one case (the determinism check's
    /// byte-compare input — the agentjev lane's law).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "openthai round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).to_string())
    }

    fn post(&self, body: &str) -> Result<HttpReply, String> {
        http_mini::request(
            &self.host,
            self.port,
            "POST",
            "/v1/systemone",
            Some(body.as_bytes()),
            self.timeout,
            &[],
        )
        .map_err(|e| e.to_string())
    }

    fn get(&self, path: &str) -> Result<HttpReply, String> {
        http_mini::request(&self.host, self.port, "GET", path, None, self.timeout, &[])
            .map_err(|e| e.to_string())
    }
}

// ───────────────────────────────────────────────────────── the request wire

/// Build their `POST /v1/systemone` body for one harness case: the state
/// passed through unchanged (their wire takes `str | dict | list`), and
/// the questions as an OBJECT keyed by qid — the dialect divergence from
/// agentjev's array. Their spellings match ours (`choice` / `score` /
/// `noul`); criteria pass through verbatim (dict for choice, array for
/// score, omitted when null).
pub fn build_body(case: &SuiteCase) -> Result<Value, String> {
    let mut questions = Map::new();
    for q in &case.questions {
        questions.insert(q.qid.clone(), build_question(q)?);
    }
    Ok(json!({
        "state": case.state,
        "questions": questions,
    }))
}

fn build_question(q: &SuiteQuestion) -> Result<Value, String> {
    let mut qj = json!({
        "type": q.kind.as_str(),
        "instructions": q.instructions,
    });
    match q.kind {
        // noul = p(yes) on their wire; criteria is not part of the
        // question (omitted when null — their optional field).
        QKind::Noul => {
            if !q.criteria.is_null() {
                qj["criteria"] = q.criteria.clone();
            }
        }
        // Their choice criteria: the object we already carry (option key →
        // description); pass through verbatim (the near-native vocabulary
        // match — no desc/key rewrite law needed here, unlike agentjev).
        QKind::Choice => {
            if q.criteria.as_object().is_none() {
                return Err(format!(
                    "case {}: choice criteria must be an object",
                    q.qid
                ));
            }
            qj["criteria"] = q.criteria.clone();
        }
        // Their score levels: the array we already carry, 2..=10 levels
        // (their documented bound — same as the harness's).
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            if !(2..=10).contains(&n) {
                return Err(format!(
                    "case {}: score needs 2..10 levels, got {n}",
                    q.qid
                ));
            }
            qj["criteria"] = q.criteria.clone();
        }
    }
    Ok(qj)
}

// ───────────────────────────────────────────────────────── the response wire

/// Map their `{answers: {qid: {...}}}` body into one
/// `(probabilities, pick, conf)` triple per question, in OUR question
/// order and OUR option order:
///
/// * noul: their p(yes) → `[1-p, p]`; `conf` = the top side's probability.
///   The MEASURED wire (types.py @ 5d04bcca, verified live 2026-09-28):
///   NoulAnswer carries the p(yes) number in a **`noul` field** —
///   `{type: "noul", noul: 0.026}` — NO `probabilities` dict; the
///   research note's `{…, probabilities, …}` shape is the REQUEST-side
///   view, not the response. A bare-number/one-element-array
///   `probabilities` fallback is accepted for tolerance (the original
///   reading) but the `noul` field is the primary read.
/// * choice: the `probabilities` dict read in OUR criteria-key order;
///   `pick` = their `choice` key's position (argmax fallback); `conf` =
///   their `confidence`.
/// * score: the `probabilities` dict keyed `"0".."k-1"`, read in level
///   order; `pick` = the argmax (their `score` field is E[level] — a
///   float, never a pick; the plan's recorded mapping); `conf` = their
///   `confidence`.
///
/// Their `abstain: true` flags are counted separately
/// ([`count_abstains`]) — provenance observation, never folded into the
/// probabilities (which their side already renormalizes over the k
/// options).
pub fn map_answers(case: &SuiteCase, body: &Value) -> Result<Vec<AnswerTriple>, String> {
    let answers = body
        .get("answers")
        .and_then(Value::as_object)
        .ok_or_else(|| "response has no answers object".to_string())?;
    let mut out = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let a = answers
            .get(&q.qid)
            .ok_or_else(|| format!("qid {}: no answer in response", q.qid))?;
        let conf = a
            .get("confidence")
            .and_then(Value::as_f64);
        let triple = match q.kind {
            QKind::Noul => {
                // Primary: the measured `noul` field (types.py
                // NoulAnswer). Fallback: a bare-number probabilities
                // value (the original lane reading, kept for wire
                // tolerance).
                let p_yes = a
                    .get("noul")
                    .and_then(Value::as_f64)
                    .or_else(|| {
                        let probs_v = a.get("probabilities")?;
                        match probs_v {
                            Value::Number(_) => probs_v.as_f64(),
                            Value::Array(arr) if arr.len() == 1 => arr[0].as_f64(),
                            _ => None,
                        }
                    })
                    .ok_or_else(|| {
                        format!(
                            "qid {}: noul answer carries neither a `noul` number \
                             nor a bare-number `probabilities`",
                            q.qid
                        )
                    })?;
                let p_no = 1.0 - p_yes;
                (
                    vec![p_no, p_yes],
                    usize::from(p_yes >= 0.5),
                    conf.unwrap_or_else(|| p_yes.max(p_no)),
                )
            }
            QKind::Choice => {
                // Their ChoiceAnswer carries the probabilities dict +
                // confidence (types.py @ 5d04bcca).
                let probs_v = a.get("probabilities").ok_or_else(|| {
                    format!("qid {}: choice answer has no probabilities", q.qid)
                })?;
                let conf = conf
                    .ok_or_else(|| format!("qid {}: choice answer has no confidence", q.qid))?;
                let dist = probs_v.as_object().ok_or_else(|| {
                    format!("qid {}: choice probabilities must be an object", q.qid)
                })?;
                let keys: Vec<String> = q
                    .criteria
                    .as_object()
                    .map(|m| m.keys().cloned().collect())
                    .unwrap_or_default();
                let mut probs = Vec::with_capacity(keys.len());
                for k in &keys {
                    let p = dist.get(k).and_then(Value::as_f64).ok_or_else(|| {
                        format!("qid {}: choice probabilities missing key {k}", q.qid)
                    })?;
                    probs.push(p);
                }
                let pick = a
                    .get("choice")
                    .and_then(Value::as_str)
                    .and_then(|v| keys.iter().position(|k| k == v))
                    .unwrap_or_else(|| {
                        probs
                            .iter()
                            .enumerate()
                            .max_by(|x, y| x.1.total_cmp(y.1))
                            .map_or(0, |(i, _)| i)
                    });
                (probs, pick, conf)
            }
            QKind::Score => {
                let probs_v = a.get("probabilities").ok_or_else(|| {
                    format!("qid {}: score answer has no probabilities", q.qid)
                })?;
                let conf = conf
                    .ok_or_else(|| format!("qid {}: score answer has no confidence", q.qid))?;
                let dist = probs_v.as_object().ok_or_else(|| {
                    format!("qid {}: score probabilities must be an object", q.qid)
                })?;
                let n = q.criteria.as_array().map_or(0, Vec::len);
                let mut probs = Vec::with_capacity(n);
                for i in 0..n {
                    let p = dist.get(&i.to_string()).and_then(Value::as_f64).ok_or_else(|| {
                        format!("qid {}: score probabilities missing level {i}", q.qid)
                    })?;
                    probs.push(p);
                }
                // Their `score` is E[level] — a float; the pick is the
                // distribution's argmax (the plan's recorded mapping).
                let pick = probs
                    .iter()
                    .enumerate()
                    .max_by(|x, y| x.1.total_cmp(y.1))
                    .map_or(0, |(i, _)| i);
                (probs, pick, conf)
            }
        };
        out.push(triple);
    }
    Ok(out)
}

/// Count their `abstain: true` answers (the slot-255 extension) — a
/// provenance observation the runner reports beside the metrics, never a
/// silence and never a probability mutation.
pub fn count_abstains(case: &SuiteCase, body: &Value) -> Result<usize, String> {
    let answers = body
        .get("answers")
        .and_then(Value::as_object)
        .ok_or_else(|| "response has no answers object".to_string())?;
    let mut n = 0usize;
    for q in &case.questions {
        if answers
            .get(&q.qid)
            .and_then(|a| a.get("abstain"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            n += 1;
        }
    }
    Ok(n)
}

// ───────────────────────────────────────────────────────────────── tests

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Read, Write};

    fn case(state: Value, questions: Vec<(&str, QKind, &str, Value)>) -> SuiteCase {
        SuiteCase {
            id: "c0".into(),
            state,
            questions: questions
                .into_iter()
                .map(|(qid, kind, instructions, criteria)| SuiteQuestion {
                    qid: qid.into(),
                    kind,
                    instructions: instructions.into(),
                    criteria,
                })
                .collect(),
            gold: vec![],
        }
    }

    #[test]
    fn body_questions_are_an_object_keyed_by_qid() {
        // THE dialect divergence from agentjev (their wire: questions is
        // an OBJECT keyed by qid, not an array), and the state passes
        // through unchanged.
        let c = case(
            json!({"b": 2, "a": 1}),
            vec![("q0", QKind::Noul, "done?", Value::Null)],
        );
        let body = build_body(&c).unwrap();
        assert_eq!(body["state"], json!({"b": 2, "a": 1}));
        assert!(body["questions"].as_object().is_some());
        assert_eq!(body["questions"]["q0"]["type"], "noul");
        assert_eq!(body["questions"]["q0"]["instructions"], "done?");
        assert!(body["questions"]["q0"].get("criteria").is_none());
    }

    #[test]
    fn body_criteria_pass_through_verbatim() {
        let c = case(
            json!("s"),
            vec![
                ("q1", QKind::Choice, "pick", json!({"a": "desc a", "b": null})),
                ("q2", QKind::Score, "rate", json!(["lo", "mid", "hi"])),
            ],
        );
        let body = build_body(&c).unwrap();
        assert_eq!(body["questions"]["q1"]["type"], "choice");
        assert_eq!(
            body["questions"]["q1"]["criteria"],
            json!({"a": "desc a", "b": null})
        );
        assert_eq!(body["questions"]["q2"]["type"], "score");
        assert_eq!(body["questions"]["q2"]["criteria"], json!(["lo", "mid", "hi"]));
    }

    #[test]
    fn body_score_rejects_out_of_bound_levels() {
        let c = case(json!("s"), vec![("q3", QKind::Score, "r", json!(["only"]))]);
        assert!(build_body(&c).is_err());
        let c11 = case(
            json!("s"),
            vec![(
                "q4",
                QKind::Score,
                "r",
                json!(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"]),
            )],
        );
        assert!(build_body(&c11).is_err());
    }

    #[test]
    fn answers_map_in_our_option_order() {
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"a": null, "b": null})),
                ("q2", QKind::Score, "rate", json!(["lo", "hi"])),
            ],
        );
        let resp = json!({
            "model": "openthai-systemone-test",
            "answers": {
                "q0": {"type": "noul", "noul": 0.8},
                "q1": {
                    "probabilities": {"a": 0.3, "b": 0.7},
                    "confidence": 0.7,
                    "choice": "b"
                },
                "q2": {
                    "probabilities": {"0": 0.1, "1": 0.9},
                    "confidence": 0.9,
                    "score": 0.86
                }
            },
            "usage": {"input_tokens": 5432}
        });
        let out = map_answers(&c, &resp).unwrap();
        // noul: the MEASURED wire — p(yes) rides the `noul` field
        // (types.py NoulAnswer @ 5d04bcca, live-verified 2026-09-28), no
        // probabilities dict, no confidence → conf = the top side.
        // p(yes) 0.8 → [1-p, p], pick 1, conf 0.8.
        assert!((out[0].0[0] - 0.2).abs() < 1e-12);
        assert!((out[0].0[1] - 0.8).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.8).abs() < 1e-12);
        // choice: our criteria-key order [a, b]; pick = their choice key's
        // position; conf = their confidence.
        assert_eq!(out[1].0, vec![0.3, 0.7]);
        assert_eq!(out[1].1, 1);
        assert!((out[1].2 - 0.7).abs() < 1e-12);
        // score: level order [lo, hi]; pick = the distribution's argmax
        // (their E[level] float is recorded, never the pick).
        assert_eq!(out[2].0, vec![0.1, 0.9]);
        assert_eq!(out[2].1, 1);
        assert!((out[2].2 - 0.9).abs() < 1e-12);
        // Provenance rides the response: model + input tokens.
        assert_eq!(OpenThaiLane::model_of(&resp), "openthai-systemone-test");
        assert_eq!(count_abstains(&c, &resp).unwrap(), 0);
        assert_eq!(
            resp.pointer("/usage/input_tokens").and_then(Value::as_u64),
            Some(5432)
        );
    }

    #[test]
    fn answers_missing_qid_is_an_error() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let resp = json!({"answers": {}});
        assert!(map_answers(&c, &resp).is_err());
    }

    #[test]
    fn abstain_flags_count_without_mutating_probabilities() {
        let c = case(json!("s"), vec![
            ("q0", QKind::Choice, "pick", json!({"a": null, "b": null})),
        ]);
        let resp = json!({
            "answers": {
                "q0": {"probabilities": {"a": 0.55, "b": 0.45},
                        "confidence": 0.55, "choice": "a", "abstain": true}
            }
        });
        let out = map_answers(&c, &resp).unwrap();
        assert_eq!(out[0].0, vec![0.55, 0.45]);
        assert_eq!(out[0].1, 0);
        assert_eq!(count_abstains(&c, &resp).unwrap(), 1);
    }

    #[test]
    fn noul_answer_maps_the_measured_noul_field_and_the_fallback() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        // Primary: the MEASURED wire — the `noul` field (types.py
        // NoulAnswer @ 5d04bcca, live-verified 2026-09-28): no probabilities
        // dict, no confidence → conf falls back to the top side.
        let primary = json!({"answers": {"q0": {"type": "noul", "noul": 0.8}}});
        let out = map_answers(&c, &primary).unwrap();
        assert!((out[0].0[1] - 0.8).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.8).abs() < 1e-12);
        // Fallback (the original lane reading, kept for wire tolerance):
        // a bare-number `probabilities` value, and a one-element array.
        let bare = json!({"answers": {"q0": {"probabilities": 0.8, "confidence": 0.8}}});
        let out = map_answers(&c, &bare).unwrap();
        assert!((out[0].0[1] - 0.8).abs() < 1e-12);
        let one = json!({"answers": {"q0": {"probabilities": [0.8], "confidence": 0.8}}});
        let out = map_answers(&c, &one).unwrap();
        assert!((out[0].0[1] - 0.8).abs() < 1e-12);
        // A dict probabilities with NO noul field stays a loud error.
        let bad = json!({"answers": {"q0": {"probabilities": {"yes": 0.8}, "confidence": 0.8}}});
        assert!(map_answers(&c, &bad).is_err(), "a dict is not p(yes) — loud error");
    }

    #[test]
    fn url_parses_host_port_and_defaults() {
        let lane = OpenThaiLane::from_url("http://127.0.0.1:8000");
        assert_eq!(lane.host, "127.0.0.1");
        assert_eq!(lane.port, 8000);
        let bare = OpenThaiLane::from_url("http://localhost");
        assert_eq!(bare.port, 8000);
    }

    /// T2.4's stub-listener round trip (the clm T2(b) pattern): one
    /// accept, one canned response — proves the CLIENT path end to end
    /// (connect → POST /v1/systemone → parse → map) against their exact
    /// response shape, no live service needed.
    #[test]
    fn stub_listener_round_trip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let canned = json!({
            "model": "openthai-stub",
            "answers": {
                "q0": {"probabilities": 0.25, "confidence": 0.75},
                "q1": {"probabilities": {"x": 0.6, "y": 0.4},
                        "confidence": 0.6, "choice": "x"}
            },
            "usage": {"input_tokens": 42}
        })
        .to_string();
        let server = std::thread::spawn(move || {
            // Serve until a well-formed request arrives (the loaded-box
            // lesson: a single accept once read a partial connect under a
            // parallel test run — loop instead of racing the scheduler).
            loop {
                let (stream, _) = listener.accept().expect("accept");
                let mut writer = match stream.try_clone() {
                    Ok(w) => w,
                    Err(_) => continue,
                };
                let mut reader = BufReader::new(stream);
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                // Read the headers AND THE BODY: a close with unread
                // request bytes in the receive buffer makes the kernel
                // send RST, which races the client's response read
                // (macOS ECONNRESET — the flake this stub's first cut
                // shipped). The clm stub's content-length read is the
                // pattern.
                let mut content_length = 0usize;
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end();
                            if trimmed.is_empty() {
                                break;
                            }
                            if let Some((k, v)) = trimmed.split_once(':')
                                && k.eq_ignore_ascii_case("content-length")
                            {
                                content_length = v.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }
                let mut body = vec![0u8; content_length];
                if reader.read_exact(&mut body).is_err() {
                    continue;
                }
                let want = "POST /v1/systemone HTTP/1.1";
                if !request_line.starts_with(want) {
                    eprintln!("stub saw: {request_line:?}");
                    continue;
                }
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    canned.len(),
                    canned
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.flush();
                return;
            }
        });
        let lane = OpenThaiLane::from_url(&format!("http://{addr}"));
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "d?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"x": null, "y": null})),
            ],
        );
        let (outcome, _client_ms) = lane.decide(&c).expect("round trip");
        server.join().expect("server thread");
        // noul: p(yes) 0.25 → pick 0 (no), conf 0.75 (the no side).
        assert!((outcome.answers[0].0[1] - 0.25).abs() < 1e-12);
        assert_eq!(outcome.answers[0].1, 0);
        assert!((outcome.answers[0].2 - 0.75).abs() < 1e-12);
        // choice: their choice key "x" → position 0.
        assert_eq!(outcome.answers[1].1, 0);
        assert_eq!(outcome.input_tokens, 42);
    }

    /// The import law's tripwire (G-ISO-4, the recorded trigger): if this
    /// test ever stops compiling because `decision_wire` is needed, land
    /// `openthai-lane = ["katgpt-core/decision_wire"]` in the same commit.
    #[test]
    fn lane_builds_bodies_from_suite_cases_only() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let body = build_body(&c).unwrap();
        assert!(body.get("questions").is_some());
    }
}
