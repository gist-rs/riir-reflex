//! The Clef comparison lane (plan 011 Phase A, from
//! `.research/005_Cloudflare_Clef_Jev_Decision_Index.md`): Cloudflare's
//! decision models (`Cloudflare/clef`, `Cloudflare/clef-flash`, Apache-2.0,
//! not affiliated) served on Workers AI, measured over HTTP — the family
//! MEASURE-vs-SERVE split (their stack serves, our Rust measures).
//!
//! The wire is **Jev-shaped per the vendor** (`state` + `questions` keyed
//! by id with `type: noul|choice|score`, `criteria` as a list of score
//! levels or a map of choice options — the same shape the agentjev lane
//! pins) with TWO Workers-AI deltas, both unwrapped EXPLICITLY here:
//!
//! * the REST route wraps the reply in the `{result, success, errors}`
//!   envelope — `success: false` or a non-empty `errors` array is a LOUD
//!   lane error, and the raw envelope is never parsed as an answer set;
//! * there is no `GET /api/info` — the handshake is ONE minimal `noul`
//!   request ([`ClefLane::info`]); the reply proves the wire shape and
//!   carries the model identity when the envelope provides it.
//!
//! Until the wire fixture lands (plan 011 A2.5 — one real captured
//! request/response pair, token redacted, BLAKE3-pinned), the shape claim
//! stays "Jev-shaped per the vendor; to be verified": every read below is
//! STRICT (missing qid / missing `distribution` / foreign shape = a loud
//! error quoting the truncated reply), never a positional guess and never
//! a half-parsed row. A reply WITHOUT per-option probabilities refuses —
//! the accuracy-only fallback row is deliberately NOT implemented against
//! an unverified shape (plan 011 A4; implement it from the fixture, not
//! from the guess).
//!
//! Transport (plan 011 A0): the harness lanes are deliberately std-only
//! plaintext HTTP (`TcpStream`, zero TLS deps — the clm-lane law), so the
//! hosted HTTPS endpoint is reached through an OPERATOR-RUN loopback
//! TLS-terminating forwarder (a ~20-line process OUTSIDE this repo — the
//! no-sidecar repo law; it injects the bearer token and forwards the
//! Workers-AI route verbatim). The extra hop is part of serving latency
//! and is DISCLOSED in the run posture (the JDI's own hosted rows disclose
//! the same class: "network round-trip", not comparable to on-card
//! figures). Direct HTTPS via a feature-gated TLS dep is the owner-gated
//! boundary-change alternative — NOT taken by default.
//!
//! Env contract (plan 011 A1/A6 — never a committed value):
//!
//! * `CLEF_SERVE_URL` — default `http://127.0.0.1:8791` (the forwarder);
//! * `CLEF_ACCOUNT_ID` — Cloudflare account id, for the default run path;
//! * `CLEF_RUN_PATH` — full path override (e.g. a flat forwarder route);
//!   when set, `CLEF_ACCOUNT_ID` is not required;
//! * `CLEF_API_TOKEN` — bearer, attached when set (the forwarder may
//!   inject it instead);
//! * `CLEF_MODEL` — `clef` (default) | `clef-flash`; provenance stamp;
//! * `CLEF_TIMEOUT_MS` — default 120000;
//! * `CLEF_SMOKE_MAX_CASES` — spend ceiling, default 50 cases (pricing is
//!   undisclosed; a larger run refuses unless `CLEF_ALLOW_UNCAPPED=1`).
//!
//! Missing creds (neither `CLEF_ACCOUNT_ID` nor `CLEF_RUN_PATH`) refuse at
//! lane construction — the loud refusal is itself the no-creds gate
//! (plan 011 A5), never a half-run.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};

/// The env var naming the (forwarder) base URL.
pub const SERVE_URL_ENV: &str = "CLEF_SERVE_URL";
/// The default posture: the operator-run loopback TLS forwarder.
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8791";
/// The Cloudflare account id (default run path construction).
pub const ACCOUNT_ENV: &str = "CLEF_ACCOUNT_ID";
/// Full run-path override (skips the account requirement).
pub const RUN_PATH_ENV: &str = "CLEF_RUN_PATH";
/// The bearer token (attached when set; the forwarder may inject it).
pub const TOKEN_ENV: &str = "CLEF_API_TOKEN";
/// Model selector: `clef` | `clef-flash`.
pub const MODEL_ENV: &str = "CLEF_MODEL";
pub const DEFAULT_MODEL: &str = "clef";
/// Per-request timeout, milliseconds.
pub const TIMEOUT_ENV: &str = "CLEF_TIMEOUT_MS";
pub const DEFAULT_TIMEOUT_MS: u64 = 120_000;
/// The spend ceiling (cases per suite), plan 011 A6.
pub const MAX_CASES_ENV: &str = "CLEF_SMOKE_MAX_CASES";
pub const DEFAULT_MAX_CASES: usize = 50;
/// The explicit uncapped override (`=1`).
pub const UNCAPPED_ENV: &str = "CLEF_ALLOW_UNCAPPED";

/// One parsed HTTP/1.1 reply: `(status, lowercased headers, body bytes)`.
type HttpReply = (u16, HashMap<String, String>, Vec<u8>);

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// The lane: a thin plaintext-HTTP/1.1 client to the loopback forwarder
/// in front of the hosted Workers AI route. std-only (zero new deps).
#[derive(Debug, Clone)]
pub struct ClefLane {
    host: String,
    port: u16,
    token: Option<String>,
    run_path: String,
    model: String,
    timeout: Duration,
}

impl ClefLane {
    /// Build from the env contract. REFUSES (loud, naming the missing
    /// envs) when neither `CLEF_ACCOUNT_ID` nor `CLEF_RUN_PATH` is set —
    /// the no-creds gate.
    pub fn from_env() -> Result<Self, String> {
        let url = std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into());
        let run_path = match std::env::var(RUN_PATH_ENV).ok().filter(|p| !p.is_empty()) {
            Some(p) => p,
            None => {
                let account = std::env::var(ACCOUNT_ENV).ok().filter(|a| !a.is_empty());
                let model = std::env::var(MODEL_ENV)
                    .ok()
                    .filter(|m| !m.is_empty())
                    .unwrap_or_else(|| DEFAULT_MODEL.into());
                let Some(account) = account else {
                    return Err(format!(
                        "refusing: neither {ACCOUNT_ENV} nor {RUN_PATH_ENV} is set — the \
                         hosted Clef lane needs the owner's Workers AI credentials (plan \
                         011 A6). Set {ACCOUNT_ENV} (+ {TOKEN_ENV} if the forwarder does \
                         not inject the bearer) or point {RUN_PATH_ENV} at your forwarder's \
                         route; {SERVE_URL_ENV} defaults to the loopback forwarder \
                         ({DEFAULT_SERVE_URL}).",
                    ));
                };
                format!("/client/v4/accounts/{account}/ai/run/{model}")
            }
        };
        let (host, port) = Self::parse_url(&url)?;
        let timeout = Duration::from_millis(
            std::env::var(TIMEOUT_ENV)
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(DEFAULT_TIMEOUT_MS),
        );
        Ok(Self {
            host,
            port,
            token: std::env::var(TOKEN_ENV).ok().filter(|t| !t.is_empty()),
            run_path,
            model: std::env::var(MODEL_ENV)
                .ok()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| DEFAULT_MODEL.into()),
            timeout,
        })
    }

    /// Parse an `http://host[:port]` base URL (the forwarder binds
    /// loopback; the port rides the URL, default 8791).
    fn parse_url(url: &str) -> Result<(String, u16), String> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| format!("{SERVE_URL_ENV} must be an http:// loopback URL, got {url:?}"))?;
        match rest.rsplit_once(':') {
            Some((h, p)) => Ok((
                h.to_string(),
                p.parse()
                    .map_err(|_| format!("{SERVE_URL_ENV}: bad port in {url:?}"))?,
            )),
            None => Ok((rest.to_string(), 8791)),
        }
    }

    /// The handshake (plan 011 A1): ONE minimal `noul` request — the
    /// Workers AI route has no `GET /api/info`. The unwrapped reply proves
    /// the wire shape; the model identity comes from the envelope when it
    /// provides one, else the `CLEF_MODEL` stamp. Returns
    /// `(model provenance, serving posture)` — the posture carries the
    /// forwarder host:port because the extra hop is part of the row's
    /// latency claim and is disclosed beside it.
    pub fn info(&self) -> Result<(String, String), String> {
        let probe = SuiteCase {
            id: "clef-probe".into(),
            state: Value::String(
                "lane handshake: the minimal wire probe (discarded; not a measured case)".into(),
            ),
            questions: vec![SuiteQuestion {
                qid: "probe".into(),
                kind: QKind::Noul,
                instructions: "Is this the handshake probe?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let raw = self.decide_raw(&probe)?;
        let model = envelope_model(&raw).unwrap_or_else(|| format!("{}@workers-ai", self.model));
        let posture = format!(
            "loopback forwarder {}:{} -> Cloudflare Workers AI hosted {} \
             (the TLS-terminating hop is INCLUDED in the client round-trip)",
            self.host, self.port, self.model
        );
        Ok((model, posture))
    }

    /// One decision round: build the Jev-shaped body for `case`, POST it,
    /// unwrap the Workers-AI envelope, map the answers back in OUR option
    /// order. Returns `(answers, client_ms, None)` — the client round-trip
    /// is the cross-lane measure; Workers AI exposes no per-case server
    /// wall we can read without the unverified shape, so the third slot
    /// stays `None` (never a fabricated server-side figure).
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, Option<f64>), String> {
        let t0 = Instant::now();
        let raw = self.decide_raw(case)?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("clef response ({}): {e}", case.id))?;
        let result = unwrap_envelope(&parsed)?;
        let answers = map_answers(case, result)?;
        Ok((answers, client_ms, None))
    }

    /// The raw reply body for one case (the determinism check's
    /// byte-compare input — the agentjev lane's raw-line law).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("clef round trip ({}): {e}", case.id))?;
        if status != 200 {
            return Err(format!(
                "clef round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).to_string())
    }

    fn post(&self, body: &str) -> Result<HttpReply, String> {
        self.request(self.run_path.as_str(), body.as_bytes())
    }

    /// The hand-rolled HTTP/1.1 exchange (`Connection: close`), mirroring
    /// the agentjev lane's min-envelope posture + the bearer header when a
    /// token is configured.
    fn request(&self, path: &str, body: &[u8]) -> Result<HttpReply, String> {
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|e| {
                format!(
                    "connect {}:{}: {e} — is the loopback TLS forwarder serving at \
                     {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL})? plan 011 A0: the \
                     forwarder is operator-run, outside this repo",
                    self.host, self.port
                )
            })?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|e| format!("set timeout: {e}"))?;
        let mut head = format!(
            "POST {path} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\nContent-Type: \
             application/json\r\n",
            self.host, self.port
        );
        if let Some(t) = &self.token {
            head.push_str("Authorization: Bearer ");
            head.push_str(t);
            head.push_str("\r\n");
        }
        head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        stream
            .write_all(head.as_bytes())
            .and_then(|_| stream.write_all(body))
            .and_then(|_| stream.flush())
            .map_err(|e| format!("write: {e}"))?;
        let mut raw = Vec::new();
        stream
            .read_to_end(&mut raw)
            .map_err(|e| format!("read: {e}"))?;
        parse_http_response(&raw)
    }
}

/// Split a raw HTTP/1.1 response into `(status, lowercased-headers, body)`,
/// decoding `Transfer-Encoding: chunked` when the reply is chunked (a
/// forwarder is free to chunk; the agentjev parser's content-length-only
/// shape would silently mis-truncate there).
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
    let body_bytes = body.as_bytes().to_vec();
    let body_bytes = if headers
        .get("transfer-encoding")
        .is_some_and(|v| v.to_ascii_lowercase().contains("chunked"))
    {
        decode_chunked(&body_bytes)?
    } else if let Some(len) = headers.get("content-length").and_then(|v| v.parse::<usize>().ok()) {
        let mut b = body_bytes;
        b.truncate(len);
        b
    } else {
        body_bytes
    };
    Ok((status, headers, body_bytes))
}

/// RFC 7230 chunked decoding: `size CRLF data CRLF` repeats, terminated by
/// a `0 CRLF` chunk (trailer section consumed to the end).
fn decode_chunked(body: &[u8]) -> Result<Vec<u8>, String> {
    let text = String::from_utf8_lossy(body);
    let mut out = Vec::with_capacity(body.len());
    let mut rest = text.as_ref();
    loop {
        let Some((size_line, after)) = rest.split_once("\r\n") else {
            return Err("chunked body: missing chunk-size line".into());
        };
        let size = usize::from_str_radix(
            size_line.split(';').next().unwrap_or_default().trim(),
            16,
        )
        .map_err(|_| format!("chunked body: bad chunk size {size_line:?}"))?;
        if size == 0 {
            return Ok(out);
        }
        if after.len() < size {
            return Err("chunked body: truncated chunk data".into());
        }
        let (data, tail) = after.split_at(size);
        out.extend_from_slice(data.as_bytes());
        rest = tail
            .strip_prefix("\r\n")
            .ok_or_else(|| "chunked body: chunk data not CRLF-terminated".to_string())?;
    }
}

// ───────────────────────────────────────────────────────── the request wire

/// Build the Jev-shaped body for one harness case (the agentjev lane's
/// mapping, pinned HERE as this lane's own contract): the state passes
/// through unchanged; every question of the case rides one request in
/// harness order. noul carries NO options (the wire law — no per-side
/// criteria prose crosses); choice renders `criteria` as the options map
/// (key = description, description-or-key); score renders the levels list.
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
        QKind::Noul => Ok(base("noul")),
        QKind::Choice => {
            let obj = q
                .criteria
                .as_object()
                .ok_or_else(|| format!("case {}: choice criteria must be an object", q.qid))?;
            let mut options = Map::new();
            for (k, v) in obj {
                let desc = match v {
                    Value::Null => k.clone(),
                    Value::String(s) if s.is_empty() => k.clone(),
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                options.insert(k.clone(), Value::String(desc));
            }
            let mut qj = base("choice");
            qj["criteria"] = Value::Object(options);
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
            if levels.len() < 2 {
                return Err(format!(
                    "case {}: score needs at least 2 levels, got {}",
                    q.qid,
                    levels.len()
                ));
            }
            let mut qj = base("score");
            qj["criteria"] = json!(levels);
            Ok(qj)
        }
    }
}

// ──────────────────────────────────────────────────────── the response wire

/// Unwrap the Workers-AI `{result, success, errors}` envelope (plan 011
/// A2): `success: false` or a non-empty `errors` array is a loud error
/// carrying the errors; a missing `result` is a loud error quoting the
/// truncated envelope (the raw envelope is NEVER parsed as an answer set).
pub fn unwrap_envelope(parsed: &Value) -> Result<&Value, String> {
    let errors = parsed.get("errors").and_then(Value::as_array);
    if parsed.get("success").and_then(Value::as_bool) == Some(false)
        || errors.is_some_and(|e| !e.is_empty())
    {
        return Err(format!(
            "workers-ai envelope reports failure: errors={}",
            parsed
                .get("errors")
                .map(Value::to_string)
                .unwrap_or_else(|| "[]".into())
        ));
    }
    let result = parsed.get("result").ok_or_else(|| {
        format!(
            "envelope has no `result` — the reply is not the expected Workers-AI shape \
             (plan 011 A2.5 fixture pending): {}",
            truncate_for_error(parsed)
        )
    })?;
    Ok(result)
}

/// The model identity from an unwrapped-able raw reply, when the envelope
/// carries one (`result.model`). `None` = the caller's own stamp applies.
fn envelope_model(raw: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(raw).ok()?;
    let result = unwrap_envelope(&parsed).ok()?;
    result
        .get("model")
        .and_then(Value::as_str)
        .filter(|m| !m.is_empty())
        .map(str::to_string)
}

fn truncate_for_error(v: &Value) -> String {
    let s = v.to_string();
    if s.len() <= 240 {
        s
    } else {
        format!("{}…", &s[..240])
    }
}

/// Map the unwrapped `result` into one `(probabilities, pick, conf)` triple
/// per question, in OUR question order and OUR option order — the Jev
/// shapes the agentjev lane pins (`results[0].answers` keyed by id; choice
/// distribution keyed by OUR option keys; score distribution keyed by level
/// position `"0".."n"`). EVERY read is strict: a missing qid, a missing
/// `distribution`, or a foreign shape is a loud error quoting the
/// truncated result — never a positional guess, never a one-hot fabrication
/// (plan 011 A4: absent probabilities refuse; the accuracy-only fallback
/// waits for the A2.5 fixture).
pub fn map_answers(case: &SuiteCase, result: &Value) -> Result<Vec<AnswerTriple>, String> {
    let answers = result
        .pointer("/results/0/answers")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "result has no results[0].answers array — the Jev-shape claim is not what \
                 the endpoint speaks (plan 011 A2.5 fixture pending): {}",
                truncate_for_error(result)
            )
        })?;
    let by_id: HashMap<&str, &Value> = answers
        .iter()
        .filter_map(|a| a.get("id").and_then(Value::as_str).map(|id| (id, a)))
        .collect();
    let mut out = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let a = *by_id
            .get(q.qid.as_str())
            .ok_or_else(|| format!("qid {}: no answer in response", q.qid))?;
        let dist = a.get("distribution").and_then(Value::as_object).ok_or_else(|| {
            format!(
                "qid {}: answer has no per-option distribution — the accuracy-only \
                 fallback is deliberately unimplemented against an unverified shape \
                 (plan 011 A4; capture the A2.5 fixture first): {}",
                q.qid,
                truncate_for_error(a)
            )
        })?;
        let triple = match q.kind {
            QKind::Noul => {
                // The reply carries the boolean pair keyed `true`/`false`;
                // our noul order is [false, true].
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
                    .unwrap_or_else(|| probs.iter().cloned().fold(0.0, f64::max));
                (probs, pick, conf)
            }
            QKind::Score => {
                let n = q.criteria.as_array().map_or(0, Vec::len);
                let mut probs = Vec::with_capacity(n);
                for i in 0..n {
                    let p = dist.get(&i.to_string()).and_then(Value::as_f64).ok_or_else(
                        || format!("qid {}: score distribution missing level {i}", q.qid),
                    )?;
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

    // ── A2: the request wire ───────────────────────────────────────────

    #[test]
    fn body_passes_state_through_and_noul_carries_no_options() {
        let c = case(
            json!({"b": 2, "a": 1}),
            vec![("q0", QKind::Noul, "done?", Value::Null)],
        );
        let body = build_body(&c).unwrap();
        assert_eq!(body["state"], json!({"b": 2, "a": 1}));
        assert_eq!(body["questions"][0]["type"], "noul");
        assert_eq!(body["questions"][0]["question"], "done?");
        // The wire law: noul carries NO options/criteria.
        assert!(body["questions"][0].get("criteria").is_none());
        assert!(body["questions"][0].get("options").is_none());
    }

    #[test]
    fn body_choice_renders_criteria_as_options_map() {
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
        assert_eq!(
            body["questions"][0]["criteria"],
            json!({"a": "desc a", "b": "b", "c": "c"})
        );
    }

    #[test]
    fn body_score_renders_the_levels_list() {
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
        assert_eq!(
            body["questions"][0]["criteria"],
            json!(["low", "mid", "high"])
        );
    }

    #[test]
    fn score_needs_at_least_two_levels() {
        let c = case(json!("s"), vec![("q3", QKind::Score, "r", json!(["only"]))]);
        assert!(build_body(&c).is_err());
    }

    // ── A2: the envelope + answer mapping ──────────────────────────────

    fn reply(answers: Value) -> Value {
        json!({"result": {"results": [{"id": "0", "answers": answers}]},
               "success": true, "errors": []})
    }

    #[test]
    fn envelope_failure_is_a_loud_error() {
        let e = unwrap_envelope(&json!({
            "result": null, "success": false,
            "errors": [{"code": 7001, "message": "auth"}]
        }))
        .unwrap_err();
        assert!(e.contains("auth"), "error should carry the errors: {e}");
        assert!(unwrap_envelope(&json!({"success": true})).is_err());
    }

    #[test]
    fn answers_map_in_our_option_order() {
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"a": null, "b": null})),
                ("q2", QKind::Score, "r", json!(["lo", "hi"])),
            ],
        );
        let parsed = reply(json!([
            {"id": "q0", "type": "noul",
             "distribution": {"true": 0.8, "false": 0.2}},
            {"id": "q1", "type": "choice",
             "distribution": {"a": 0.3, "b": 0.7}, "value": "b",
             "top_probability": 0.7},
            {"id": "q2", "type": "score",
             "distribution": {"0": 0.1, "1": 0.9}, "level": 1}
        ]));
        let result = unwrap_envelope(&parsed).unwrap();
        let out = map_answers(&c, result).unwrap();
        // noul: [p_false, p_true], pick 1 (true), conf = the top side.
        assert!((out[0].0[0] - 0.2).abs() < 1e-12);
        assert!((out[0].0[1] - 0.8).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        // choice: criteria insertion order [a, b]; pick = the value key.
        assert_eq!(out[1].0, vec![0.3, 0.7]);
        assert_eq!(out[1].1, 1);
        assert!((out[1].2 - 0.7).abs() < 1e-12);
        // score: level order; pick = their level.
        assert_eq!(out[2].0, vec![0.1, 0.9]);
        assert_eq!(out[2].1, 1);
    }

    #[test]
    fn missing_qid_is_an_error_never_a_positional_guess() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let parsed = reply(json!([{"id": "OTHER", "type": "noul",
            "distribution": {"true": 1.0, "false": 0.0}}]));
        let result = unwrap_envelope(&parsed).unwrap();
        let e = map_answers(&c, result).unwrap_err();
        assert!(e.contains("q0"), "error should name the qid: {e}");
    }

    #[test]
    fn missing_distribution_refuses_never_fabricates() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let parsed = reply(json!([{"id": "q0", "type": "noul", "value": true}]));
        let result = unwrap_envelope(&parsed).unwrap();
        let e = map_answers(&c, result).unwrap_err();
        assert!(
            e.contains("A4"),
            "the refusal should cite the plan's A4 law: {e}"
        );
    }

    #[test]
    fn foreign_result_shape_is_a_loud_error_quoting_the_reply() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let result = json!({"completion": "true"});
        let e = map_answers(&c, &result).unwrap_err();
        assert!(e.contains("A2.5"), "error should cite A2.5: {e}");
        assert!(e.contains("completion"), "error should quote the reply");
    }

    // ── A1: env contract ───────────────────────────────────────────────

    #[test]
    fn url_parses_host_port_and_defaults() {
        assert_eq!(
            ClefLane::parse_url("http://127.0.0.1:8791").unwrap(),
            ("127.0.0.1".to_string(), 8791)
        );
        assert_eq!(
            ClefLane::parse_url("http://localhost").unwrap(),
            ("localhost".to_string(), 8791)
        );
        assert!(ClefLane::parse_url("https://nope").is_err());
    }

    #[test]
    fn chunked_body_decodes_and_plain_truncates() {
        let raw =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n3\r\n:1}\r\n0\r\n\r\n";
        let (status, headers, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"{\"a\":1}");
        assert!(headers.contains_key("transfer-encoding"));
        let raw2 =
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{} plus bytes a keep-alive peer \
              would still have in flight";
        let (_, _, body2) = parse_http_response(raw2).unwrap();
        assert_eq!(body2, b"{}");
    }
}
