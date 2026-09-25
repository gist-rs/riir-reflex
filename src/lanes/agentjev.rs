//! The AgentJev comparison lane (Issue 025 amendment 4 / `.issues/027`;
//! Apache-2.0, not affiliated).
//!
//! Their Apache-2.0 Python service (`jev_service.server`, pinned tree
//! `malevrigns/agent-jev` @ `a965ca8f`) serves on loopback; our Rust
//! harness measures over HTTP — the CLM-lane split posture (their stack
//! serves, our Rust measures; comparison lane, never a product lane, and
//! never in the shipped binary's product path).
//!
//! The contract (read at the pin, `jev_service/contract.py`):
//!
//! * `POST /api/evaluate` with `{"state": <str|obj>, "questions": [...]}`,
//!   one case per request — a case's questions share ONE state and are
//!   scored together (their several-decisions-at-once pattern), which is
//!   also our per-case latency unit.
//! * `state`: a string reaches their encoder as-is (prefixed `[STATE] `);
//!   an object is serialized by THEIR `semantic()` as stable JSON
//!   (sort_keys, compact separators) — we pass our wire state through
//!   unchanged and let their side render it (no server shim, no our-side
//!   prose law to pin).
//! * `question` text = our question's `instructions` (their contract reads
//!   `question` or `instructions` — we send `question`).
//! * noul → `type: "boolean"`; the wire cannot carry their per-side
//!   criteria prose, so their DEFAULT `TRUE`/`FALSE` candidates apply —
//!   the documented adapter divergence (the clm lane's noul law).
//! * choice → `type: "choice"`, `options {key: description}` — the
//!   description when one is given, else the key (their `semantic()`
//!   refuses null; the clm candidates law).
//! * score → `type: "score"`, `levels` = the rubric level texts (2..10,
//!   lowest first — the wire's own convention).
//! * Answers return a `distribution` object: boolean keyed `true`/`false`,
//!   choice keyed by OUR option keys, score keyed by the level POSITION
//!   (`"0".."n"`). Probabilities are read back in OUR option order — the
//!   same label order every lane scores against.
//! * Over-length input (state + question + candidate > 2048 tokens) is
//!   REFUSED by their service (their card: an error, never a silent crop)
//!   — a refusal fails the lane loudly, it is never a guessed answer.
//! * Candidate descriptions must be distinct within a question (their
//!   validation); duplicate descriptions in a harness case surface as
//!   their 400, not as silent misalignment.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};

/// The env var naming their service's base URL.
pub const SERVE_URL_ENV: &str = "AGENTJEV_SERVE_URL";
/// The default posture: loopback, their server's default port 8149.
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8149";

/// One parsed HTTP/1.1 reply: `(status, lowercased headers, body bytes)`.
type HttpReply = (u16, HashMap<String, String>, Vec<u8>);

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// The lane: a thin loopback HTTP/1.1 client to their `jev_service`.
/// std-only (the clm lane's hand-rolled posture; no new deps).
#[derive(Debug, Clone)]
pub struct AgentJevLane {
    host: String,
    port: u16,
    timeout: Duration,
}

impl Default for AgentJevLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl AgentJevLane {
    /// Parse an `http://host:port` base URL (loopback-only posture — their
    /// server binds 127.0.0.1 beside the harness on the same box).
    pub fn from_url(url: &str) -> Self {
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(8149)),
            None => (rest.to_string(), 8149),
        };
        Self {
            host,
            port,
            timeout: Duration::from_secs(300),
        }
    }

    /// The handshake: `GET /api/info` — their advertised model id (+ the
    /// checkpoint sha they report, as provenance), stamped into the lane's
    /// row (never a hardcoded model id — the gliner lane's law).
    pub fn info(&self) -> Result<(String, String), String> {
        let (status, _headers, body) =
            self.get("/api/info").map_err(|e| format!("agentjev info: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /api/info -> HTTP {status}: {} — is their jev_service serving at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL})?",
                String::from_utf8_lossy(&body)
            ));
        }
        let parsed: Value =
            serde_json::from_slice(&body).map_err(|e| format!("agentjev info: {e}"))?;
        // Their info carries `model` ("AgentJev-0.6B") and a checkpoint
        // sha, but NOT a device field — the sha prefix (8 hex) rides along
        // as the provenance spelling.
        let model = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("agentjev-v1")
            .to_string();
        let sha = parsed
            .get("checkpoint_sha256")
            .and_then(Value::as_str)
            .unwrap_or("");
        let device = std::env::var("AGENTJEV_DEVICE")
            .unwrap_or_else(|_| "cuda:0".to_string());
        let provenance = if sha.len() >= 8 {
            format!("{model}@{}", &sha[..8])
        } else {
            model
        };
        Ok((provenance, device))
    }

    /// One decision round: build the body for `case`, POST it, map the
    /// answers back. Returns `(answer-per-question, client_ms, wall_ms)`
    /// — the 025 amendment-4 latency law: record the client round-trip
    /// (the cross-lane measure) and their server-side `usage.wall_ms`
    /// beside it.
    pub fn decide(
        &self,
        case: &SuiteCase,
    ) -> Result<(Vec<AnswerTriple>, f64, Option<f64>), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("agentjev round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "agentjev round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("agentjev response ({}): {e}", case.id))?;
        let wall_ms = parsed
            .pointer("/usage/wall_ms")
            .and_then(Value::as_f64);
        let answers = map_answers(case, &parsed)?;
        Ok((answers, client_ms, wall_ms))
    }

    /// The raw response body for one case (the determinism check's
    /// byte-compare input — same law as the gliner lane's raw line).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "agentjev round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).to_string())
    }

    fn post(&self, body: &str) -> Result<HttpReply, String> {
        self.request("POST", "/api/evaluate", Some(body.as_bytes()))
    }

    fn get(&self, path: &str) -> Result<HttpReply, String> {
        self.request("GET", path, None)
    }

    /// The hand-rolled HTTP/1.1 exchange (`Connection: close`), mirroring
    /// the clm lane's min-envelope posture.
    fn request(&self, method: &str, path: &str, body: Option<&[u8]>) -> Result<HttpReply, String> {
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|e| format!("connect {}:{}: {e}", self.host, self.port))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|e| format!("set timeout: {e}"))?;
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n",
            self.host, self.port
        );
        let head = match body {
            Some(b) => format!(
                "{head}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                b.len()
            ),
            None => format!("{head}\r\n"),
        };
        stream
            .write_all(head.as_bytes())
            .and_then(|_| match body {
                Some(b) => stream.write_all(b),
                None => Ok(()),
            })
            .and_then(|_| stream.flush())
            .map_err(|e| format!("write: {e}"))?;
        let mut raw = Vec::new();
        stream
            .read_to_end(&mut raw)
            .map_err(|e| format!("read: {e}"))?;
        parse_http_response(&raw)
    }
}

/// Split a raw HTTP/1.1 response into `(status, lowercased-headers, body)`.
fn parse_http_response(raw: &[u8]) -> Result<HttpReply, String> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| "response has no header/body separator".to_string())?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("malformed status line: {status_line}"))?;
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let mut body_bytes = body.as_bytes().to_vec();
    if let Some(len) = headers.get("content-length").and_then(|v| v.parse::<usize>().ok()) {
        body_bytes.truncate(len);
    }
    Ok((status, headers, body_bytes))
}

// ───────────────────────────────────────────────────────── the request wire

/// Build their `POST /api/evaluate` body for one harness case: the state
/// passed through unchanged (their `semantic()` renders objects as stable
/// JSON — no our-side prose law), every question of the case in ONE
/// request (their several-decisions-at-once pattern). Question order and
/// option order are the harness criteria's insertion order — the label
/// order the answers map back onto.
pub fn build_body(case: &SuiteCase) -> Result<Value, String> {
    let mut questions = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        questions.push(build_question(q)?);
    }
    Ok(json!({
        "state": case.state,
        "questions": questions,
    }))
}

fn build_question(q: &SuiteQuestion) -> Result<Value, String> {
    let base = |kind: &str| {
        json!({
            "id": q.qid,
            "type": kind,
            "question": q.instructions,
        })
    };
    match q.kind {
        // Their default TRUE/FALSE candidates (the wire cannot carry their
        // per-side criteria prose — the documented adapter divergence).
        QKind::Noul => Ok(base("boolean")),
        QKind::Choice => {
            let obj = q.criteria.as_object().ok_or_else(|| {
                format!("case {}: choice criteria must be an object", q.qid)
            })?;
            let mut options = Map::new();
            for (k, v) in obj {
                // The description when one is given, else the key (their
                // `semantic()` refuses null — the clm candidates law).
                let desc = match v {
                    Value::Null => k.clone(),
                    Value::String(s) if s.is_empty() => k.clone(),
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                options.insert(k.clone(), Value::String(desc));
            }
            let mut qj = base("choice");
            qj["options"] = Value::Object(options);
            Ok(qj)
        }
        QKind::Score => {
            let levels: Vec<String> = q
                .criteria
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect()
                })
                .ok_or_else(|| format!("case {}: score criteria must be an array", q.qid))?;
            if !(2..=10).contains(&levels.len()) {
                return Err(format!(
                    "case {}: score needs 2..10 levels, got {}",
                    q.qid,
                    levels.len()
                ));
            }
            let mut qj = base("score");
            qj["levels"] = json!(levels);
            Ok(qj)
        }
    }
}

// ───────────────────────────────────────────────────────── the response wire

/// Map their `{"results": [{"answers": [...]}]}` body into one
/// `(probabilities, pick, conf)` triple per question, in OUR question
/// order and OUR option order:
///
/// * noul (boolean): `[p_false, p_true]` — the `[1-n, n]` convention every
///   lane's noul row reports; `conf` = the top side's probability.
/// * choice: the distribution keyed by OUR option keys, read in criteria
///   insertion order; `pick` = their `value` key's position (argmax
///   fallback); `conf` = their `top_probability`.
/// * score: the distribution keyed by level position (`"0".."n"`), read
///   in level order; `pick` = their `level` (argmax index); `conf` = the
///   top level's probability.
pub fn map_answers(case: &SuiteCase, body: &Value) -> Result<Vec<AnswerTriple>, String> {
    let answers = body
        .pointer("/results/0/answers")
        .and_then(Value::as_array)
        .ok_or_else(|| "response has no results[0].answers array".to_string())?;
    let by_id: HashMap<&str, &Value> = answers
        .iter()
        .filter_map(|a| {
            a.get("id")
                .and_then(Value::as_str)
                .map(|id| (id, a))
        })
        .collect();
    let mut out = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let a = *by_id
            .get(q.qid.as_str())
            .ok_or_else(|| format!("qid {}: no answer in response", q.qid))?;
        let dist = a
            .get("distribution")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("qid {}: answer has no distribution", q.qid))?;
        let triple = match q.kind {
            QKind::Noul => {
                let p_true = dist
                    .get("true")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| format!("qid {}: boolean distribution missing true", q.qid))?;
                let p_false = 1.0 - p_true;
                (
                    vec![p_false, p_true],
                    usize::from(p_true >= 0.5),
                    p_true.max(p_false),
                )
            }
            QKind::Choice => {
                let keys: Vec<String> = q
                    .criteria
                    .as_object()
                    .map(|m| m.keys().cloned().collect())
                    .unwrap_or_default();
                let mut probs = Vec::with_capacity(keys.len());
                for k in &keys {
                    let p = dist.get(k).and_then(Value::as_f64).ok_or_else(|| {
                        format!("qid {}: choice distribution missing key {k}", q.qid)
                    })?;
                    probs.push(p);
                }
                let pick = a
                    .get("value")
                    .and_then(Value::as_str)
                    .and_then(|v| keys.iter().position(|k| k == v))
                    .unwrap_or_else(|| {
                        probs
                            .iter()
                            .enumerate()
                            .max_by(|x, y| x.1.total_cmp(y.1))
                            .map_or(0, |(i, _)| i)
                    });
                let conf = a
                    .get("top_probability")
                    .and_then(Value::as_f64)
                    .unwrap_or_else(|| {
                        probs.iter().cloned().fold(0.0, f64::max)
                    });
                (probs, pick, conf)
            }
            QKind::Score => {
                let n = q.criteria.as_array().map_or(0, Vec::len);
                let mut probs = Vec::with_capacity(n);
                for i in 0..n {
                    let p = dist.get(&i.to_string()).and_then(Value::as_f64).ok_or_else(|| {
                        format!("qid {}: score distribution missing level {i}", q.qid)
                    })?;
                    probs.push(p);
                }
                let pick = a
                    .get("level")
                    .and_then(Value::as_u64)
                    .map_or(0, |l| l as usize)
                    .min(n.saturating_sub(1));
                let conf = probs.iter().cloned().fold(0.0, f64::max);
                (probs, pick, conf)
            }
        };
        out.push(triple);
    }
    Ok(out)
}

// ───────────────────────────────────────────────────────────────── tests

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn case(state: Value, questions: Vec<(&str, QKind, &str, Value)>) -> SuiteCase {
        SuiteCase {
            id: "c0".into(),
            state,
            questions: questions
                .into_iter()
                .map(|(qid, kind, instructions, criteria)| crate::harness::suites::SuiteQuestion {
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
    fn body_passes_state_through_and_defaults_noul() {
        let c = case(
            json!({"b": 2, "a": 1}),
            vec![("q0", QKind::Noul, "done?", Value::Null)],
        );
        let body = build_body(&c).unwrap();
        // State passes through UNCHANGED (their semantic() renders objects
        // — no our-side law), one request per case.
        assert_eq!(body["state"], json!({"b": 2, "a": 1}));
        assert_eq!(body["questions"][0]["type"], "boolean");
        assert_eq!(body["questions"][0]["question"], "done?");
        // No criteria key — their default TRUE/FALSE candidates apply.
        assert!(body["questions"][0].get("criteria").is_none());
    }

    #[test]
    fn body_choice_options_are_desc_or_key() {
        let c = case(
            json!("s"),
            vec![(
                "q1",
                QKind::Choice,
                "pick",
                json!({"a": "desc a", "b": null, "c": ""}),
            )],
        );
        let body = build_body(&c).unwrap();
        assert_eq!(body["questions"][0]["type"], "choice");
        assert_eq!(body["questions"][0]["options"], json!({"a": "desc a", "b": "b", "c": "c"}));
    }

    #[test]
    fn body_score_levels_pass_through() {
        let c = case(
            json!("s"),
            vec![(
                "q2",
                QKind::Score,
                "how bad",
                json!(["low", "mid", "high"]),
            )],
        );
        let body = build_body(&c).unwrap();
        assert_eq!(body["questions"][0]["type"], "score");
        assert_eq!(body["questions"][0]["levels"], json!(["low", "mid", "high"]));
    }

    #[test]
    fn score_needs_two_to_ten_levels() {
        let c = case(
            json!("s"),
            vec![("q3", QKind::Score, "r", json!(["only"]))],
        );
        assert!(build_body(&c).is_err());
    }

    #[test]
    fn answers_map_in_our_option_order() {
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                (
                    "q1",
                    QKind::Choice,
                    "pick",
                    json!({"a": null, "b": null}),
                ),
                ("q2", QKind::Score, "r", json!(["lo", "hi"])),
            ],
        );
        let resp = json!({
            "results": [{"id": "0", "answers": [
                {"id": "q0", "type": "boolean",
                 "distribution": {"true": 0.8, "false": 0.2},
                 "probability": 0.8, "value": true},
                {"id": "q1", "type": "choice",
                 "distribution": {"a": 0.3, "b": 0.7},
                 "value": "b", "top_probability": 0.7, "margin": 0.4},
                {"id": "q2", "type": "score",
                 "distribution": {"0": 0.1, "1": 0.9},
                 "score": 0.9, "level": 1, "legend": ["lo", "hi"]}
            ]}],
            "usage": {"wall_ms": 12.5}
        });
        let out = map_answers(&c, &resp).unwrap();
        // noul: [p_false, p_true], pick 1 (true), conf = top side. The
        // false side is 1.0 - p_true (float-exact in the vector, compared
        // approximately here).
        assert!((out[0].0[0] - 0.2).abs() < 1e-12);
        assert!((out[0].0[1] - 0.8).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.8).abs() < 1e-12);
        // choice: criteria insertion order [a, b], pick = b's position.
        assert_eq!(out[1].0, vec![0.3, 0.7]);
        assert_eq!(out[1].1, 1);
        assert!((out[1].2 - 0.7).abs() < 1e-12);
        // score: level order [lo, hi], pick = their level.
        assert_eq!(out[2].0, vec![0.1, 0.9]);
        assert_eq!(out[2].1, 1);
        assert!((out[2].2 - 0.9).abs() < 1e-12);
    }

    #[test]
    fn answers_missing_qid_is_an_error() {
        let c = case(
            json!("s"),
            vec![("q0", QKind::Noul, "d?", Value::Null)],
        );
        let resp = json!({"results": [{"answers": []}]});
        assert!(map_answers(&c, &resp).is_err());
    }

    #[test]
    fn url_parses_host_port_and_defaults() {
        let lane = AgentJevLane::from_url("http://127.0.0.1:8149");
        assert_eq!(lane.host, "127.0.0.1");
        assert_eq!(lane.port, 8149);
        let bare = AgentJevLane::from_url("http://localhost");
        assert_eq!(bare.port, 8149);
    }

    #[test]
    fn http_response_splits_head_and_body() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
        let (status, headers, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(headers.get("content-type").map(String::as_str), Some("application/json"));
        assert_eq!(body, b"{}");
    }
}
