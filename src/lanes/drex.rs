//! The Drex DLM comparison lane (Issue 073, from `.research/008`): Nace.AI's
//! open-weights 8B diffusion-LM decision model (`nace-ai/drex-dlm` @
//! `6c63df2`, backbone `nvidia/Efficient-DLM-8B`; repo code MIT, model
//! weights **CC BY-NC 4.0** — MEASUREMENT ONLY: comparison lane yes, distill
//! teacher NO (NC contaminates — the bekko-MIT teacher posture does not
//! transfer; never wire `TeacherForward::Drex`), product serving NO).
//!
//! Their stack serves (`python serve.py`, port 8000 — or their llama.cpp
//! `edlm` fork, port 8097; the reply's own `model` field + the env disclose
//! which), our Rust harness measures — the MEASURE-vs-SERVE split, the
//! openthai lane's shape (the same TypeSafe `/v1/systemone` wire family).
//!
//! Wire (pinned from `code/kev/api.py` + `serve.py` + `inference.py` at the
//! pin):
//!
//! * `POST /v1/systemone` with `{state, model?, questions: {qid: {type:
//!   noul|choice|score, instructions, criteria}}}` — questions are an OBJECT
//!   keyed by qid (the openthai dialect, not agentjev's array); one decision
//!   per request (their `requests` wrapper refuses 400).
//! * noul → answer = p(true), reported as `{"type": "noul", "noul": p}`.
//! * choice → `{"type": "choice", "choice": key, "confidence": (max(p) −
//!   1/K)/(1 − 1/K), "probabilities": {key: p}}` — 1..=255 options.
//! * score → `{"type": "score", "score": E[level], "legend": [...],
//!   "probabilities": {"0": p, …}, "confidence": 1 − E|level−mode|/(L−1)}`
//!   — 1..=255 levels (their bound; the harness's own 2..=10 is stricter and
//!   stands upstream of the wire).
//! * Probabilities serialize at 4 decimals (`round_prob`) — the sum drifts
//!   within TypeSafe's tolerance; read as-is, never renormalized (the drift
//!   is the wire's).
//! * Response: `{model, answers: {qid: …}, usage: {input_tokens,
//!   output_tokens}, latency_ms}` — `latency_ms` is THEIR scorer-side wall
//!   time at TOP level (openthai nests server wall under usage; here the
//!   billing-style `output_tokens` — serialized-answer tokens, no generation
//!   — rides usage instead).
//! * Liveness: `GET /health` → `{"status": "ok", "model": …}` (their
//!   serve.py spells it `/health`, NOT openthai's `/healthz`); the model id
//!   also rides every decide response (never a hardcoded name — the gliner
//!   law).
//! * Over-context is REFUSED by their side (strict encode, 400 error, never
//!   a silent crop — their card law); a refusal fails the lane loudly.
//! * **Long-state caution (Issue 073 T6, carried from their own validation
//!   file):** their `validation/RESULTS.md` measured a 15,644-token state
//!   answering WRONG when the retrieval marker sat in the first third
//!   (correct in the final third) — a retrieval-correctness failure, not a
//!   capacity rejection, on both GGUF runners (their Python SIGSEGV'd
//!   outright). The lane does NOT add 16K-token states to its cases; a
//!   long-state probe is its own bench with the fragility disclosed, never
//!   a suite case.

use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
use crate::lanes::http_mini::{self, HttpReply};

/// The env var naming their service's base URL.
pub const SERVE_URL_ENV: &str = "DREX_SERVE_URL";
/// The default posture: loopback, their Python server's default port 8000
/// (their llama.cpp `edlm` fork serves 8097 — point the env there and the
/// reply's `model` field discloses the posture).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8000";

/// Their MAX_OPTIONS bound (`api.py`): 1..=255 criteria entries per
/// question. The harness's own score bound (2..=10) is stricter and is
/// checked first by the suites; this is the wire's own ceiling.
pub const MAX_OPTIONS: usize = 255;

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// One decide round's full yield: the answer triples, their scorer-side
/// `latency_ms`, and their billing-style token counts.
pub struct DecideOutcome {
    pub answers: Vec<AnswerTriple>,
    pub latency_ms: Option<f64>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// The lane: a thin loopback HTTP/1.1 client to their service.
/// std-only (the shared `crate::lanes::http_mini` micro-client; no new
/// deps — the G-ISO-4 import law: `crate::harness::suites` + `http_mini` +
/// std + serde_json).
#[derive(Debug, Clone)]
pub struct DrexLane {
    host: String,
    port: u16,
    timeout: Duration,
}

impl Default for DrexLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl DrexLane {
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

    /// The liveness probe: `GET /health` (their serve.py's spelling) — 200
    /// means up. The model id comes from the decide response (see
    /// [`Self::model_of`]); `/health`'s own `model` field is the warmup
    /// cross-check.
    pub fn health(&self) -> Result<(), String> {
        let (status, _headers, body) =
            self.get("/health").map_err(|e| format!("drex health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is their drex service serving \
                 at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL})?",
                String::from_utf8_lossy(&body)
            ));
        }
        Ok(())
    }

    /// The model id from a DECIDE response body (`model` field — their
    /// serve.py spreads `request.get("model") or model_name` first) — the
    /// provenance stamp, never a hardcoded name (the gliner law).
    pub fn model_of(body: &Value) -> String {
        body.get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("drex-dlm")
            .to_string()
    }

    /// One decision round: build the body for `case`, POST it, map the
    /// answers back. Latency is the CALLER's wall clock (the agentjev
    /// shape); their scorer-side `latency_ms` and the token counts ride the
    /// outcome for the runner's provenance lines.
    pub fn decide(&self, case: &SuiteCase) -> Result<(DecideOutcome, f64), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("drex round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "drex round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("drex response ({}): {e}", case.id))?;
        let outcome = DecideOutcome {
            answers: map_answers(case, &parsed)?,
            latency_ms: parsed.get("latency_ms").and_then(Value::as_f64),
            input_tokens: parsed
                .pointer("/usage/input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            output_tokens: parsed
                .pointer("/usage/output_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        };
        Ok((outcome, client_ms))
    }

    /// The raw response body for one case (the determinism check's
    /// byte-compare input — the agentjev/openthai lane law).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "drex round trip ({}) -> HTTP {status}: {}",
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
/// passed through unchanged (their `render()` flattens str|dict|list on
/// their side — no our-side prose law), the questions as an OBJECT keyed by
/// qid. Criteria pass through verbatim (dict for choice, array for score,
/// omitted when null — `Noul.criteria` is optional on their wire).
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
        // noul: criteria optional on their wire (`Noul.criteria: dict |
        // None` — the per-side prose keys `false`/`true` when present).
        QKind::Noul => {
            if !q.criteria.is_null() {
                qj["criteria"] = q.criteria.clone();
            }
        }
        // choice: the object we already carry (option key → description),
        // 1..=255 entries (their MAX_OPTIONS validator).
        QKind::Choice => {
            let n = q.criteria.as_object().map_or(0, Map::len);
            if n == 0 || n > MAX_OPTIONS {
                return Err(format!(
                    "case {}: choice needs 1..{MAX_OPTIONS} options, got {n}",
                    q.qid
                ));
            }
            qj["criteria"] = q.criteria.clone();
        }
        // score: the array we already carry, 1..=255 entries on their wire
        // (the harness's own 2..=10 bound is stricter and already checked
        // by the suites).
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            if n == 0 || n > MAX_OPTIONS {
                return Err(format!(
                    "case {}: score needs 1..{MAX_OPTIONS} levels, got {n}",
                    q.qid
                ));
            }
            qj["criteria"] = q.criteria.clone();
        }
    }
    Ok(qj)
}

// ───────────────────────────────────────────────────────── the response wire

/// Map their `{answers: {qid: …}}` body into one `(probabilities, pick,
/// conf)` triple per question, in OUR question order and OUR option order
/// (pinned from `api.py::to_answers`):
///
/// * noul: their `{"type": "noul", "noul": p_true}` (rounded 4 decimals) →
///   `[1-p, p]`; `conf` = the top side's probability. The same measured
///   shape the openthai lane reads (the shared TypeSafe lineage).
/// * choice: the `probabilities` dict read in OUR criteria-key order;
///   `pick` = their `choice` key's position (argmax fallback); `conf` =
///   their rescaled `confidence` — the FORMULA IS THEIRS, read as-is: this
///   is exactly the T4 axis (their card disclaims calibration; our ECE
///   settles it empirically).
/// * score: the `probabilities` dict keyed `"0".."k-1"`, read in level
///   order; `pick` = the argmax (their `score` field is E[level] — a
///   float, never a pick; the openthai lane's recorded mapping); `conf` =
///   their distance-from-mode `confidence`.
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
        let triple = match q.kind {
            QKind::Noul => {
                let p_true = a.get("noul").and_then(Value::as_f64).ok_or_else(|| {
                    format!(
                        "qid {}: noul answer carries no `noul` number (api.py \
                         to_answers spells p(true) `noul`)",
                        q.qid
                    )
                })?;
                let p_false = 1.0 - p_true;
                (
                    vec![p_false, p_true],
                    usize::from(p_true >= 0.5),
                    p_true.max(p_false),
                )
            }
            QKind::Choice => {
                let dist = a
                    .get("probabilities")
                    .and_then(Value::as_object)
                    .ok_or_else(|| format!("qid {}: choice answer has no probabilities", q.qid))?;
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
                let conf = a
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or_else(|| probs.iter().cloned().fold(0.0, f64::max));
                (probs, pick, conf)
            }
            QKind::Score => {
                let dist = a
                    .get("probabilities")
                    .and_then(Value::as_object)
                    .ok_or_else(|| format!("qid {}: score answer has no probabilities", q.qid))?;
                let n = q.criteria.as_array().map_or(0, Vec::len);
                let mut probs = Vec::with_capacity(n);
                for i in 0..n {
                    let p = dist
                        .get(&i.to_string())
                        .and_then(Value::as_f64)
                        .ok_or_else(|| {
                            format!("qid {}: score probabilities missing level {i}", q.qid)
                        })?;
                    probs.push(p);
                }
                // Their `score` is E[level] — a float; the pick is the
                // distribution's argmax (the openthai lane's recorded
                // mapping). Their `confidence` is the distance-from-mode
                // statistic — read as-is (the T4 axis), top-prob fallback.
                let pick = probs
                    .iter()
                    .enumerate()
                    .max_by(|x, y| x.1.total_cmp(y.1))
                    .map_or(0, |(i, _)| i);
                let conf = a
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or_else(|| probs.iter().cloned().fold(0.0, f64::max));
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
        // The openthai dialect (api.py SystemOneRequest: questions is an
        // OBJECT keyed by qid), state passes through unchanged, noul
        // criteria omitted when null.
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
    fn body_criteria_pass_through_verbatim_with_their_bounds() {
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
        // THEIR bound (1..=255): one level is legal on the wire (api.py
        // Score min_length=1) — unlike the harness suites' own 2..=10,
        // which is checked upstream, not here.
        let one = case(json!("s"), vec![("q3", QKind::Score, "r", json!(["only"]))]);
        assert!(build_body(&one).is_ok());
        // Zero options refuses (their min_length=1).
        let zero = case(json!("s"), vec![("q4", QKind::Score, "r", json!([]))]);
        assert!(build_body(&zero).is_err());
    }

    #[test]
    fn answers_map_in_our_option_order() {
        // Shapes pinned from api.py::to_answers @ 6c63df2.
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"a": null, "b": null})),
                ("q2", QKind::Score, "rate", json!(["lo", "mid", "hi"])),
            ],
        );
        let resp = json!({
            "model": "drex-dlm-stub",
            "answers": {
                "q0": {"type": "noul", "noul": 0.85},
                "q1": {
                    "type": "choice",
                    "choice": "b",
                    "confidence": 0.55,
                    "probabilities": {"a": 0.3, "b": 0.7}
                },
                "q2": {
                    "type": "score",
                    "score": 1.8,
                    "legend": ["lo", "mid", "hi"],
                    "probabilities": {"0": 0.1, "1": 0.2, "2": 0.7},
                    "confidence": 0.6
                }
            },
            "usage": {"input_tokens": 87, "output_tokens": 120},
            "latency_ms": 321.5
        });
        let out = map_answers(&c, &resp).unwrap();
        // noul: p(true) rides the `noul` field → [1-p, p], pick 1.
        assert!((out[0].0[0] - 0.15).abs() < 1e-12);
        assert!((out[0].0[1] - 0.85).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.85).abs() < 1e-12);
        // choice: our criteria-key order [a, b]; pick = their `choice`
        // key's position; conf = THEIR rescaled confidence, read as-is
        // (the T4 axis — their card disclaims calibration).
        assert_eq!(out[1].0, vec![0.3, 0.7]);
        assert_eq!(out[1].1, 1);
        assert!((out[1].2 - 0.55).abs() < 1e-12);
        // score: level order; pick = the argmax (their `score` E[level]
        // float is never the pick); conf = their distance-from-mode
        // statistic read as-is.
        assert_eq!(out[2].0, vec![0.1, 0.2, 0.7]);
        assert_eq!(out[2].1, 2);
        assert!((out[2].2 - 0.6).abs() < 1e-12);
        // Provenance: model + their latency + the token counts.
        assert_eq!(DrexLane::model_of(&resp), "drex-dlm-stub");
        assert_eq!(
            resp.pointer("/usage/input_tokens").and_then(Value::as_u64),
            Some(87)
        );
        assert_eq!(
            resp.pointer("/latency_ms").and_then(Value::as_f64),
            Some(321.5)
        );
    }

    #[test]
    fn answers_missing_qid_is_an_error() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let resp = json!({"answers": {}});
        assert!(map_answers(&c, &resp).is_err());
    }

    #[test]
    fn noul_answer_without_the_noul_field_is_a_loud_error() {
        // api.py ALWAYS spells p(true) as `noul` for noul answers — a
        // probabilities-dict shape is a different (broken) server, not a
        // tolerance case (the openthai lane's fallback existed because
        // THEIR types.py had two shapes; drex's to_answers has one).
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let bad = json!({"answers": {"q0": {"probabilities": {"true": 0.8}}}});
        assert!(map_answers(&c, &bad).is_err());
    }

    #[test]
    fn choice_pick_falls_back_to_argmax_without_their_choice_key() {
        let c = case(json!("s"), vec![("q1", QKind::Choice, "pick", json!({"a": null, "b": null}))]);
        let resp = json!({"answers": {"q1": {"probabilities": {"a": 0.3, "b": 0.7}}}});
        let out = map_answers(&c, &resp).unwrap();
        assert_eq!(out[0].1, 1, "argmax fallback when `choice` is absent");
    }

    #[test]
    fn url_parses_host_port_and_defaults() {
        let lane = DrexLane::from_url("http://127.0.0.1:8000");
        assert_eq!(lane.host, "127.0.0.1");
        assert_eq!(lane.port, 8000);
        let fork = DrexLane::from_url("http://127.0.0.1:8097");
        assert_eq!(fork.port, 8097);
        let bare = DrexLane::from_url("http://localhost");
        assert_eq!(bare.port, 8000);
    }

    /// T1's stub-listener round trip (the clm T2(b) pattern, the
    /// openthai stub's shape): proves the CLIENT path end to end
    /// (connect → POST /v1/systemone → parse → map) against their exact
    /// response shape, no live service needed.
    #[test]
    fn stub_listener_round_trip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let canned = json!({
            "model": "drex-dlm",
            "answers": {
                "q0": {"type": "noul", "noul": 0.25},
                "q1": {"type": "choice", "choice": "x", "confidence": 0.6,
                       "probabilities": {"x": 0.6, "y": 0.4}}
            },
            "usage": {"input_tokens": 42, "output_tokens": 77},
            "latency_ms": 12.5
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
                // send RST, which races the client's response read (the
                // macOS ECONNRESET flake the openthai stub documents).
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
        let lane = DrexLane::from_url(&format!("http://{addr}"));
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
        // Provenance rides the outcome.
        assert_eq!(outcome.input_tokens, 42);
        assert_eq!(outcome.output_tokens, 77);
        assert_eq!(outcome.latency_ms, Some(12.5));
    }
}
