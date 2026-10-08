//! The LiquidAI d1 comparison lane (Issue 078, from `.research/009`):
//! LiquidAI's open-weights d1-3B decision model (`LiquidAI/d1-3B` @
//! `051bcc4`, LFM2.5-VL-3B backbone; weights **`license: other` /
//! `license_name: lfm1.0`** — the LFM Open License, NOT Apache/MIT:
//! MEASUREMENT ONLY posture; read `LICENSE` before ANY local weight
//! redistribution or product use — the Drex CC BY-NC lesson; the blog's
//! "without restrictions" is marketing, the license tag is `other`).
//!
//! Their stack is a Python class, not a server: the repo ships
//! `modeling_d1.D1Model` (`system_one` / `system_one_batch`,
//! `trust_remote_code=True`, transformers ≥5.14) with NO HTTP layer — the
//! reference posture is OUR stdlib-listener wrapper over their in-repo
//! class on plaintext loopback (the Drex-lane shape; `.raw/d1_server.py`).
//! Hosted `d1:free` is HTTPS → a loopback TLS-terminating forwarder or an
//! owner-gated `ureq` (the Research-005 Clef transport finding verbatim;
//! both owner-gated, neither the default). Our Rust harness measures.
//!
//! Wire (pinned from `api.py` + `prompt.py` + `runner.py` at the pin):
//!
//! * `POST /decisions/v1/systemone` with `{model?, state, questions: {qid:
//!   {type: noul|choice|score, instructions, criteria}}}` — questions are
//!   an OBJECT keyed by qid (the openthai dialect); one decision per
//!   request in this lane (their `system_one_batch` packing is their
//!   serving shape, not our measurement posture).
//! * noul → answer = P(yes), reported as `{"type": "noul", "noul": p}` —
//!   their `readout` scores the yes/no token forms, `probs[0]` is yes.
//! * choice → `{"type": "choice", "choice": key, "confidence": max(p),
//!   "probabilities": {key: p}}` — the confidence IS the max probability
//!   (raw softmax; no rescale).
//! * score → `{"type": "score", "score": E[level], "confidence": max(p),
//!   "probabilities": {"0": p, …}, "legend": {...}}`.
//! * Response: `{model, answers, usage: {input_tokens, output_tokens},
//!   latency_ms}` — `usage.output_tokens` is **0 always** (their zero
//!   output-token law: the answer is read off the logits at the answer
//!   slot, nothing is decoded, a schema violation is impossible); the
//!   zero rides the run's token tally as the law made visible. The model
//!   id rides every decide response (never a hardcoded name — the gliner
//!   law).
//! * Liveness: `GET /health`.
//! * **Calibration posture (the T3 axis):** the open weights ship NO
//!   temperature artifact — `config.json` carries no temperatures key and
//!   their `D1Model.engine` passes `calibration=None`, so the in-repo
//!   serving posture is RAW SOFTMAX. The card's "calibrated" claim (the
//!   per-type temperatures are the hosted tier's shipped artifact on the
//!   omni card) is exactly what our ECE/Brier cells test — the G1 axis,
//!   never assumed.
//! * **Dtype posture (the T2 law):** their own card publishes the flip
//!   table — fp16 = 0 top-answer flips vs fp32 (243 text rows), bf16
//!   flips 0.8% text / 1.7% audio. The reference server serves **fp16**
//!   and the SERVED dtype is recorded beside every cell (`/health` +
//!   boot logs), never assumed from the checkpoint's bf16 storage dtype.

use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
use crate::lanes::http_mini::{self, HttpReply};

/// The env var naming their service's base URL.
pub const SERVE_URL_ENV: &str = "D1_SERVE_URL";
/// The default posture: loopback, port 8078 (fresh in the lane family —
/// 8000 is drex/openthai's, 8149 agentjev's; the env points anywhere).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8078";

/// The wire's option ceiling (their `aliases` assigns single-token codes
/// well past 26 options; the harness suites' own bounds are stricter and
/// stand upstream of the wire — the same shape as the drex lane's 255).
pub const MAX_OPTIONS: usize = 255;

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// One decide round's full yield: the answer triples, the server wall
/// `latency_ms`, the token counts (`output_tokens` is their ALWAYS-ZERO
/// law made visible), and THEIR wire `confidence` per question (`None`
/// for noul — the wire carries no noul confidence; see
/// [`map_confidences`]). The runner's confidence-vs-correctness readout
/// (the shared `assemble_drex_conf_readout` path — one math shape for
/// every comparison lane, never a parallel ECE) consumes the last
/// channel. For d1 the wire `confidence` IS the max probability (their
/// `answer()`), so the readout's cells and the hard metrics' max-prob
/// ECE measure the same axis — the readout adds the per-kind split, the
/// noul absence disclosure, and the conformal floor companion.
pub struct DecideOutcome {
    pub answers: Vec<AnswerTriple>,
    pub confidences: Vec<Option<f64>>,
    pub latency_ms: Option<f64>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// The lane: a thin loopback HTTP/1.1 client to their service.
/// std-only (the shared `crate::lanes::http_mini` micro-client; no new
/// deps — the G-ISO-4 import law: `crate::harness::suites` + `http_mini`
/// + std + serde_json).
#[derive(Debug, Clone)]
pub struct D1Lane {
    host: String,
    port: u16,
    timeout: Duration,
}

impl Default for D1Lane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl D1Lane {
    /// Parse an `http://host:port` base URL (loopback-only posture — their
    /// server binds 127.0.0.1 beside the harness on the same box).
    pub fn from_url(url: &str) -> Self {
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(8078)),
            None => (rest.to_string(), 8078),
        };
        Self {
            host,
            port,
            timeout: Duration::from_secs(300),
        }
    }

    /// The liveness probe: `GET /health` — 200 means up. The model id
    /// comes from the decide response (see [`Self::model_of`]).
    pub fn health(&self) -> Result<(), String> {
        let (status, _headers, body) =
            self.get("/health").map_err(|e| format!("d1 health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is their d1 service serving \
                 at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL})?",
                String::from_utf8_lossy(&body)
            ));
        }
        Ok(())
    }

    /// The model id from a DECIDE response body (`model` field — our
    /// reference server echoes the served checkpoint label; the hosted
    /// forwarder posture would name their deployment) — the provenance
    /// stamp, never a hardcoded name (the gliner law).
    pub fn model_of(body: &Value) -> String {
        body.get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("d1-3b")
            .to_string()
    }

    /// One decision round: build the body for `case`, POST it, map the
    /// answers back. Latency is the CALLER's wall clock (the agentjev
    /// shape); their server-side `latency_ms` and the token counts ride
    /// the outcome for the runner's provenance lines.
    pub fn decide(&self, case: &SuiteCase) -> Result<(DecideOutcome, f64), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("d1 round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "d1 round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("d1 response ({}): {e}", case.id))?;
        let outcome = DecideOutcome {
            answers: map_answers(case, &parsed)?,
            confidences: map_confidences(case, &parsed)?,
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

    /// The response body for one case with the VOLATILE TIMING TAIL
    /// removed — the determinism check's compare input (the drex lane's
    /// law: a raw byte-compare measures their clock, not the decision;
    /// everything else stays byte-exact).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "d1 round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let mut parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("d1 response ({}): {e}", case.id))?;
        if let Some(obj) = parsed.as_object_mut() {
            obj.remove("latency_ms");
        }
        Ok(parsed.to_string())
    }

    fn post(&self, body: &str) -> Result<HttpReply, String> {
        http_mini::request(
            &self.host,
            self.port,
            "POST",
            "/decisions/v1/systemone",
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

/// Build their `POST /decisions/v1/systemone` body for one harness case:
/// the state passed through unchanged (their `state_block` renders
/// str|dict under the json_only default on their side — no our-side prose
/// law), the questions as an OBJECT keyed by qid. Criteria pass through
/// verbatim (dict for choice, array for score, omitted when null —
/// `Noul.criteria` is optional on their wire).
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
        // noul: criteria optional on their wire (`Noul.criteria: Mapping |
        // None` — the per-side prose keys `false`/`true` when present).
        QKind::Noul => {
            if !q.criteria.is_null() {
                qj["criteria"] = q.criteria.clone();
            }
        }
        // choice: the object we already carry (option key → description),
        // 1..=255 entries (the lane's wire ceiling).
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
        // score: the array we already carry — their primitive is DEFINED
        // for 2..=10 ordered levels (prompt.py readout_ids raises outside
        // it: single-token digits); the harness's own 2..=10 bound is the
        // same bound and is already checked by the suites.
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            if !(2..=10).contains(&n) {
                return Err(format!(
                    "case {}: score needs 2..=10 levels on their wire, got {n}",
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
/// (pinned from `api.py::answer`):
///
/// * noul: their `{"type": "noul", "noul": p_yes}` → `[1-p, p]`; `conf` =
///   the top side's probability. The wire carries NO noul confidence
///   field — the derived top-side probability must never masquerade as
///   one (see [`map_confidences`]).
/// * choice: the `probabilities` dict read in OUR criteria-key order;
///   `pick` = their `choice` key's position (argmax fallback); `conf` =
///   their `confidence` — the max probability, read as-is (their answer()
///   emits it; the ECE cells test its calibration, never assume it).
/// * score: the `probabilities` dict keyed `"0".."k-1"`, read in level
///   order; `pick` = the argmax (their `score` field is E[level] — a
///   float, never a pick); `conf` = their `confidence` (max probability).
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
                         answer spells P(yes) `noul`)",
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
                // distribution's argmax. Their `confidence` is the max
                // probability — read as-is (the T3 axis), top-prob
                // fallback.
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

/// THEIR wire `confidence` per question, in OUR question order: `Some`
/// for choice/score (the max probability — the T3 audit axis), `None`
/// for noul (the wire carries no noul confidence — never fabricated). A
/// choice/score answer WITHOUT the field is a loud error, not a silent
/// fallback: their `api.py::answer` always emits it, so its absence means
/// the wire drifted and every calibration cell built on it would silently
/// read a substitute.
pub fn map_confidences(case: &SuiteCase, body: &Value) -> Result<Vec<Option<f64>>, String> {
    let answers = body
        .get("answers")
        .and_then(Value::as_object)
        .ok_or_else(|| "response has no answers object".to_string())?;
    let mut out = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let a = answers
            .get(&q.qid)
            .ok_or_else(|| format!("qid {}: no answer in response", q.qid))?;
        match q.kind {
            QKind::Noul => out.push(None),
            QKind::Choice | QKind::Score => {
                let conf = a
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| {
                        format!(
                            "qid {}: {} answer carries no `confidence` number (their \
                             api.py answer always emits it — the wire drifted)",
                            q.qid,
                            q.kind.as_str()
                        )
                    })?;
                out.push(Some(conf));
            }
        }
    }
    Ok(out)
}

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
        // Their dialect (api.py system_one: questions is an OBJECT keyed
        // by name), state passes through unchanged, noul criteria omitted
        // when null.
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
        // THEIR bound: score is DEFINED for 2..=10 (prompt.py readout_ids
        // raises outside it) — one level refuses at body build, unlike
        // the drex wire's 1..=255.
        let one = case(json!("s"), vec![("q3", QKind::Score, "r", json!(["only"]))]);
        assert!(build_body(&one).is_err());
        let zero = case(json!("s"), vec![("q4", QKind::Score, "r", json!([]))]);
        assert!(build_body(&zero).is_err());
        // Eleven levels also refuse (the vendor's own 2..=10 primitive).
        let eleven = case(
            json!("s"),
            vec![(
                "q5",
                QKind::Score,
                "r",
                json!(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]),
            )],
        );
        assert!(build_body(&eleven).is_err());
    }

    #[test]
    fn answers_map_in_our_option_order() {
        // Shapes pinned from api.py::answer @ 051bcc4.
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"a": null, "b": null})),
                ("q2", QKind::Score, "rate", json!(["lo", "mid", "hi"])),
            ],
        );
        let resp = json!({
            "model": "d1-3b-stub",
            "answers": {
                "q0": {"type": "noul", "noul": 0.85},
                "q1": {
                    "type": "choice",
                    "choice": "b",
                    "confidence": 0.7,
                    "probabilities": {"a": 0.3, "b": 0.7}
                },
                "q2": {
                    "type": "score",
                    "score": 1.6,
                    "confidence": 0.7,
                    "probabilities": {"0": 0.1, "1": 0.2, "2": 0.7},
                    "legend": {"0": "lo", "1": "mid", "2": "hi"}
                }
            },
            "usage": {"input_tokens": 87, "output_tokens": 0},
            "latency_ms": 8.2
        });
        let out = map_answers(&c, &resp).unwrap();
        // noul: p(yes) rides the `noul` field → [1-p, p], pick 1.
        assert!((out[0].0[0] - 0.15).abs() < 1e-12);
        assert!((out[0].0[1] - 0.85).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.85).abs() < 1e-12);
        // choice: our criteria-key order [a, b]; pick = their `choice`
        // key's position; conf = THEIR max-probability confidence, read
        // as-is (the T3 axis — the cells test the calibration claim).
        assert_eq!(out[1].0, vec![0.3, 0.7]);
        assert_eq!(out[1].1, 1);
        assert!((out[1].2 - 0.7).abs() < 1e-12);
        // score: level order; pick = the argmax (their `score` E[level]
        // float is never the pick); conf = their max-prob confidence.
        assert_eq!(out[2].0, vec![0.1, 0.2, 0.7]);
        assert_eq!(out[2].1, 2);
        assert!((out[2].2 - 0.7).abs() < 1e-12);
        // Provenance: model + their latency + the token counts — and the
        // ZERO output-token law (their answer slot reads, never decodes).
        assert_eq!(D1Lane::model_of(&resp), "d1-3b-stub");
        assert_eq!(
            resp.pointer("/usage/input_tokens").and_then(Value::as_u64),
            Some(87)
        );
        assert_eq!(
            resp.pointer("/usage/output_tokens").and_then(Value::as_u64),
            Some(0)
        );
        assert_eq!(
            resp.pointer("/latency_ms").and_then(Value::as_f64),
            Some(8.2)
        );
    }

    #[test]
    fn answers_missing_qid_is_an_error() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let resp = json!({"answers": {}});
        assert!(map_answers(&c, &resp).is_err());
    }

    #[test]
    fn wire_confidences_read_per_kind_and_noul_stays_none() {
        // The same answers body as the mapping test — THEIR `confidence`
        // rides `Some` for choice/score, `None` for noul (the wire
        // carries no noul confidence; the derived top-side probability
        // must never masquerade as one).
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"a": null, "b": null})),
                ("q2", QKind::Score, "rate", json!(["lo", "mid", "hi"])),
            ],
        );
        let resp = json!({
            "answers": {
                "q0": {"type": "noul", "noul": 0.85},
                "q1": {
                    "type": "choice", "choice": "b", "confidence": 0.7,
                    "probabilities": {"a": 0.3, "b": 0.7}
                },
                "q2": {
                    "type": "score", "score": 1.6, "confidence": 0.7,
                    "probabilities": {"0": 0.1, "1": 0.2, "2": 0.7}
                }
            }
        });
        let confs = map_confidences(&c, &resp).unwrap();
        assert_eq!(confs, vec![None, Some(0.7), Some(0.7)]);
    }

    #[test]
    fn missing_choice_confidence_is_a_loud_error_not_a_fallback() {
        // api.py::answer ALWAYS emits `confidence` for choice/score — a
        // body without it is a drifted wire, and a silent top-prob
        // substitute would make every calibration cell read a fabricated
        // axis.
        let c = case(
            json!("s"),
            vec![("q1", QKind::Choice, "pick", json!({"a": null, "b": null}))],
        );
        let resp = json!({
            "answers": {
                "q1": {"type": "choice", "choice": "b", "probabilities": {"a": 0.3, "b": 0.7}}
            }
        });
        let err = map_confidences(&c, &resp).unwrap_err();
        assert!(err.contains("confidence"), "loud error names the axis: {err}");
    }

    #[test]
    fn noul_answer_without_the_noul_field_is_a_loud_error() {
        // api.py::answer ALWAYS spells P(yes) as `noul` for noul answers
        // — a probabilities-dict shape is a different (broken) server,
        // not a tolerance case.
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
        let lane = D1Lane::from_url("http://127.0.0.1:8078");
        assert_eq!(lane.host, "127.0.0.1");
        assert_eq!(lane.port, 8078);
        let moved = D1Lane::from_url("http://127.0.0.1:9000");
        assert_eq!(moved.port, 9000);
        let bare = D1Lane::from_url("http://localhost");
        assert_eq!(bare.port, 8078);
    }

    /// T1's stub-listener round trip (the drex T1 pattern): proves the
    /// CLIENT path end to end (connect → POST /decisions/v1/systemone →
    /// parse → map) against their exact response shape, no live service
    /// needed.
    #[test]
    fn stub_listener_round_trip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let canned = json!({
            "model": "LiquidAI/d1-3B",
            "answers": {
                "q0": {"type": "noul", "noul": 0.25},
                "q1": {"type": "choice", "choice": "x", "confidence": 0.6,
                       "probabilities": {"x": 0.6, "y": 0.4}}
            },
            "usage": {"input_tokens": 42, "output_tokens": 0},
            "latency_ms": 8.0
        })
        .to_string();
        let server = std::thread::spawn(move || {
            // Serve until a well-formed request arrives (the loaded-box
            // lesson: loop instead of racing the scheduler).
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
                // send RST, which races the client's response read.
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
                let want = "POST /decisions/v1/systemone HTTP/1.1";
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
        let lane = D1Lane::from_url(&format!("http://{addr}"));
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
        // Provenance rides the outcome — and the zero-token law holds.
        assert_eq!(outcome.input_tokens, 42);
        assert_eq!(outcome.output_tokens, 0);
        assert_eq!(outcome.latency_ms, Some(8.0));
    }

    /// The determinism compare input strips the volatile timing tail
    /// (the drex lane's law): two responses differing ONLY in
    /// `latency_ms` normalize byte-identical, while the answers surface
    /// survives verbatim — the check measures the decision, never the
    /// server's clock.
    #[test]
    fn decide_raw_strips_only_the_latency_tail() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let base = json!({
            "model": "LiquidAI/d1-3B",
            "answers": {"q0": {"type": "noul", "noul": 0.25}},
            "usage": {"input_tokens": 9, "output_tokens": 0},
            "latency_ms": 111.0
        });
        let raw_with = |latency: f64| {
            let mut body = base.clone();
            body["latency_ms"] = json!(latency);
            let canned = body.to_string();
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
            let addr = listener.local_addr().expect("addr");
            let server = std::thread::spawn(move || loop {
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
                // Read the headers AND THE BODY (the RST-race lesson).
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
                let mut req_body = vec![0u8; content_length];
                if reader.read_exact(&mut req_body).is_err() {
                    continue;
                }
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    canned.len(),
                    canned
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.flush();
                return;
            });
            let raw = D1Lane::from_url(&format!("http://{addr}"))
                .decide_raw(&c)
                .expect("round trip");
            server.join().expect("server thread");
            raw
        };
        let raw1 = raw_with(111.0);
        let raw2 = raw_with(408.0);
        assert_eq!(raw1, raw2, "timing-only differences normalize away");
        assert!(!raw1.contains("latency_ms"));
        // The answers surface survives verbatim.
        assert!(raw1.contains("0.25"), "{raw1}");
    }
}
