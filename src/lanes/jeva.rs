//! The Jeva comparison lane (Issue 090 B1, feeding riir-train 632 T2) —
//! the first decision-native PAPER model measured on our split:
//! `LIQIIIII/Jeva-4B` (arXiv:2610.18773, "Jeva: System One Models from
//! First Principles", Li & Wang, NUS; Qwen3.5-4B backbone + a 13.37M
//! decision head) served by OUR reference stdlib server
//! (`scripts/jeva_server.py`) over their in-repo `jeva.Jeva` class.
//!
//! LICENSE POSTURE (riir-train 632 T0, verified 2026-10-15): the HF
//! weights are `gated: "auto"` (accept-conditions auto-approval — NOT
//! contact-gated; correcting Issue 090's original gate note) with an
//! UNDECLARED license (no license field/tag/file anywhere on the repo or
//! card). Undeclared is neither open nor restricted-declared:
//! **measurement + teacher-capture allowed, weight ADOPTION fail-closed**
//! until the author declares. The distill seam's RIDT header carries the
//! license note (the bekko attribution law).
//!
//! Wire (`scripts/jeva_server.py`, pinned from their `types.py` +
//! `inference.py` at the pin below): `POST /decide` with
//! `{state, questions: {qid: {type, instructions, criteria?, values?}}}`,
//! answers keyed by qid carrying their `Answer` verbatim —
//!
//! * `probabilities` — **JOINT** ((1−ρ)·q; the conditional q rides
//!   `conditional_probabilities` — the paper's A3 readout). At the lane's
//!   default allow_null=False posture ρ=0 and the two coincide; the
//!   JEVA_ALLOW_NULL=1 server posture (the A1 probe arm) dilutes them.
//! * `null_probability` — the coverage gate ρ (0.0 at allow_null=False;
//!   their Noul/Score questions never carry a null).
//! * `choice` — their pick (labels[argmax q]); `None` ONLY under the
//!   allow_null posture when ρ ≥ max joint (an abstain — the lane maps it
//!   to the argmax over joint probs, DISCLOSED, never a fabricated pick).
//! * `noul` — q_true, the CONDITIONAL P(yes) (the A3 conditional shape —
//!   their Noul readout is the positive candidate's conditional by
//!   design, never the joint).
//! * `score` — E[level] = Σ q_i·v_i (a float, never a pick; the pick is
//!   the argmax over the level probabilities, the d1 law).
//! * `confidence` — max(ρ, max joint) (their readout; the ECE/Brier cells
//!   test it, never assume it).
//!
//! Comparison posture: gold-label accuracy + the shared confidence
//! readout (`assemble_drex_conf_readout` — one math shape for every
//! comparison lane) + `--perm-probe`/A1–A3 probes ride the runner. Their
//! published 78.30 typed is THEIR protocol on THEIR hardware — never
//! quoted cross-protocol (the lane law).
//!
//! Env: `JEVA_SERVE_URL` (default `http://127.0.0.1:8095`). Serve with
//! `JEVA_PYTHON=<venv python with `jeva` installed> … scripts/jeva_server.py`
//! (their repo: `uv pip install -e .raw/jeva` + torch/transformers).

use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
use crate::lanes::http_mini::{self, HttpReply};

/// The env var naming our reference server's base URL.
pub const SERVE_URL_ENV: &str = "JEVA_SERVE_URL";
/// The default posture: loopback, port 8095 (fresh in the lane family —
/// 8000 drex/openthai, 8078 d1, 8149 agentjev, 8793 pplx, 8794 d3; the
/// env points anywhere).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8095";

/// The wire's option ceiling (their `_criteria` enforces unique nonempty
/// labels with no count bound; the harness suites' own bounds are
/// stricter and stand upstream of the wire — the d1 lane's 255 shape).
pub const MAX_OPTIONS: usize = 255;

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// One decide round's full yield (the d1 shape): the answer triples, the
/// wire `confidence` per question (Jeva emits it for EVERY kind — noul
/// included, unlike d1's wire), the server wall `latency_ms`, the token
/// counts (`input_tokens` is state+ENCODED options — it moves with their
/// hybrid option cache between cold/warm repeats, which is why
/// [`JevaLane::decide_raw`] strips `usage`; `output_tokens` is their
/// ALWAYS-ZERO law — the answer-slot read — made visible), and their
/// per-question ABSTENTION (true iff `choice` is null — reachable ONLY
/// under the server's JEVA_ALLOW_NULL=1 posture; the A1 probe's subject).
pub struct DecideOutcome {
    pub answers: Vec<AnswerTriple>,
    pub confidences: Vec<Option<f64>>,
    pub abstained: Vec<bool>,
    pub latency_ms: Option<f64>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// The lane: a thin loopback HTTP/1.1 client to our reference server.
/// std-only (the shared `crate::lanes::http_mini` micro-client; no new
/// deps — the G-ISO-4 import law: `crate::harness::suites` + `http_mini`
/// + std + serde_json).
#[derive(Debug, Clone)]
pub struct JevaLane {
    host: String,
    port: u16,
    timeout: Duration,
}

impl Default for JevaLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl JevaLane {
    /// Parse an `http://host:port` base URL (loopback-only posture — our
    /// reference server binds 127.0.0.1 beside the harness).
    pub fn from_url(url: &str) -> Self {
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(8095)),
            None => (rest.to_string(), 8095),
        };
        Self {
            host,
            port,
            timeout: Duration::from_secs(300),
        }
    }

    /// The liveness probe: `GET /health` — 200 means up (the model is
    /// loaded before the listener binds — no lazy-load window).
    pub fn health(&self) -> Result<(), String> {
        let (status, _headers, body) = self
            .get("/health")
            .map_err(|e| format!("jeva health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is our jeva reference server \
                 serving at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL}; start it: \
                 scripts/jeva_server.py under the venv python)?",
                String::from_utf8_lossy(&body)
            ));
        }
        Ok(())
    }

    /// `(provenance, device)` off `/health` — the construct-time capture
    /// the distill seam uses (the pplx law): the model id + the RESOLVED
    /// served dtype + the allow_null posture ride the provenance string,
    /// never a hardcoded name (the gliner law).
    pub fn info(&self) -> Result<(String, String), String> {
        let (status, _headers, body) = self
            .get("/health")
            .map_err(|e| format!("jeva health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {}",
                String::from_utf8_lossy(&body)
            ));
        }
        let parsed: Value =
            serde_json::from_slice(&body).map_err(|e| format!("jeva health parse: {e}"))?;
        if !parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            return Err("jeva /health carries ok=false".to_string());
        }
        let model = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("jeva-4b")
            .to_string();
        let dtype = parsed
            .get("dtype")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let allow_null = parsed
            .get("allow_null")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let device = parsed
            .get("device")
            .and_then(Value::as_str)
            .unwrap_or("cuda")
            .to_string();
        let provenance = if allow_null {
            format!("{model} dtype={dtype} allow_null=on")
        } else {
            format!("{model} dtype={dtype}")
        };
        Ok((provenance, device))
    }

    /// The model id from a DECIDE response body (`model` field — our
    /// reference server echoes the served checkpoint label) — the
    /// provenance stamp, never a hardcoded name (the gliner law).
    pub fn model_of(body: &Value) -> String {
        body.get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("jeva-4b")
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
            .map_err(|e| format!("jeva round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "jeva round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("jeva response ({}): {e}", case.id))?;
        let outcome = DecideOutcome {
            answers: map_answers(case, &parsed)?,
            confidences: map_confidences(case, &parsed)?,
            abstained: map_abstains(case, &parsed)?,
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

    /// The response body for one case with the VOLATILE TAIL removed —
    /// the determinism check's compare input (the drex law: a raw
    /// byte-compare measures their clock, not the decision). Strips the
    /// top-level `latency_ms` + `usage` AND the per-answer `latency_ms` +
    /// `usage` (their `input_tokens` counts state + ENCODED options, and
    /// the encoded count moves with the hybrid option cache between
    /// cold/warm repeats — a cache-hit delta is not non-determinism).
    /// Everything else stays byte-exact.
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "jeva round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let mut parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("jeva response ({}): {e}", case.id))?;
        if let Some(obj) = parsed.as_object_mut() {
            obj.remove("latency_ms");
            obj.remove("usage");
        }
        if let Some(answers) = parsed.get_mut("answers").and_then(Value::as_object_mut) {
            for a in answers.values_mut() {
                if let Some(ao) = a.as_object_mut() {
                    ao.remove("latency_ms");
                    ao.remove("usage");
                }
            }
        }
        Ok(parsed.to_string())
    }

    fn post(&self, body: &str) -> Result<HttpReply, String> {
        http_mini::request(
            &self.host,
            self.port,
            "POST",
            "/decide",
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

/// Build the `POST /decide` body for one harness case: the state passed
/// through unchanged, the questions as an OBJECT keyed by qid (their
/// `system_one` Mapping shape). The allow_null posture is SERVER-side
/// (`JEVA_ALLOW_NULL` env — the comparison default is the forced-pick
/// posture every reflex lane speaks; the A1 probe arm arms it there).
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
        // noul: their `Noul` dataclass carries NO criteria (the paper's
        // readout is the conditional q_true) — sending one is a server
        // 400, so the field is omitted, never `null`.
        QKind::Noul => {}
        // choice: the object we already carry (option key → description);
        // their option text renders "key: description" (the d1 law).
        // Bare-label suites (emotion, banking77, semantic_defects) go as
        // their SEQUENCE arm (array of keys; their `_criteria` sets texts
        // = keys verbatim) — their mapping arm rejects empty descriptions
        // with a 400. The null-probe's augmented arms on those suites
        // arrive MIXED (bare originals + decoys whose value == their own
        // key): the sequence arm is the faithful rendering there too —
        // their texts = keys, so the decoy values never reach the model
        // and the presentation stays consistent across K arms (the A2
        // probe's comparability law). A GENUINELY mixed suite (a real
        // nonempty description beside a real empty one) keeps the mapping
        // arm and their loud 400 — the wire has no faithful mixed shape.
        // Their sequence-arm labels ARE the key strings, so the response
        // mapping is unchanged.
        QKind::Choice => {
            let obj = q.criteria.as_object();
            let n = obj.map_or(0, Map::len);
            if n == 0 || n > MAX_OPTIONS {
                return Err(format!(
                    "case {}: choice needs 1..{MAX_OPTIONS} options, got {n}",
                    q.qid
                ));
            }
            let bare_shape = obj.is_some_and(|m| {
                m.iter().all(|(k, v)| match v.as_str() {
                    // null / empty / whitespace = bare label
                    None => true,
                    Some(s) => s.trim().is_empty() || s == k,
                })
            });
            if bare_shape {
                let keys: Vec<&String> = obj.expect("checked above").keys().collect();
                qj["criteria"] = json!(keys);
            } else {
                qj["criteria"] = q.criteria.clone();
            }
        }
        // score: the array we already carry — their `_criteria` sequence
        // arm keys the levels BY THEIR OWN STRINGS (labels == texts), and
        // `values` defaults to range(len) level indices (their E[level]
        // and our argmax pick agree on it). >=2 levels (their law).
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            if !(2..=MAX_OPTIONS).contains(&n) {
                return Err(format!(
                    "case {}: score needs 2..{MAX_OPTIONS} levels, got {n}",
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
/// conf)` triple per question, in OUR question order and OUR option
/// order (pinned from their `inference.py::_predict`):
///
/// * noul: their `noul` = q_true (the CONDITIONAL P(yes) — the paper's
///   A3 readout) → `[1-p, p]`; `conf` = their `confidence` (max joint —
///   ρ is 0 on their noul questions by construction).
/// * choice: the `probabilities` dict (JOINT — at the default
///   allow_null=False posture ρ=0 and joint == conditional) read in OUR
///   criteria-key order; `pick` = their `choice` key's position (their
///   argmax over q; `None` only under the allow_null posture — the lane
///   falls back to the argmax over joint probs, DISCLOSED, never a
///   fabricated pick); `conf` = their `confidence` = max(ρ, max joint).
/// * score: the `probabilities` dict keyed by OUR level strings (their
///   sequence-arm labels), read in level order; `pick` = the argmax
///   (their `score` is E[level] — a float, never a pick); `conf` =
///   their `confidence`.
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
                        "qid {}: noul answer carries no `noul` number (their inference \
                         spells the conditional P(yes) `noul`)",
                        q.qid
                    )
                })?;
                let p_false = 1.0 - p_true;
                let conf = required_conf(a, q)?;
                (vec![p_false, p_true], usize::from(p_true >= 0.5), conf)
            }
            QKind::Choice => {
                let (probs, keys) = keyed_probs_by_object(a, q)?;
                let pick = match a.get("choice") {
                    // Their pick: the labels[argmax q] position (their
                    // choice is a LABEL STRING, never an index).
                    Some(Value::String(s)) => {
                        keys.iter().position(|k| k == s).ok_or_else(|| {
                            format!("qid {}: their choice {s:?} is not a presented key", q.qid)
                        })?
                    }
                    // allow_null posture abstain (choice: null) — the
                    // argmax over the JOINT probabilities, disclosed in
                    // the module doc, never fabricated.
                    _ => probs
                        .iter()
                        .enumerate()
                        .max_by(|x, y| x.1.total_cmp(y.1))
                        .map_or(0, |(i, _)| i),
                };
                let conf = required_conf(a, q)?;
                (probs, pick, conf)
            }
            QKind::Score => {
                let (probs, _keys) = keyed_probs_by_array(a, q)?;
                // Their `score` is E[level] — a float; the pick is the
                // distribution's argmax (the d1 law).
                let pick = probs
                    .iter()
                    .enumerate()
                    .max_by(|x, y| x.1.total_cmp(y.1))
                    .map_or(0, |(i, _)| i);
                let conf = required_conf(a, q)?;
                (probs, pick, conf)
            }
        };
        out.push(triple);
    }
    Ok(out)
}

/// The choice mapping: their probabilities keyed by OUR criteria keys
/// (their mapping-arm labels), read in OUR presentation order.
fn keyed_probs_by_object(a: &Value, q: &SuiteQuestion) -> Result<(Vec<f64>, Vec<String>), String> {
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
        let p = dist
            .get(k)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("qid {}: choice probabilities missing key {k}", q.qid))?;
        probs.push(p);
    }
    Ok((probs, keys))
}

/// The score mapping: their probabilities keyed by OUR level strings
/// (their sequence-arm labels ARE the level strings), read in OUR level
/// order.
fn keyed_probs_by_array(a: &Value, q: &SuiteQuestion) -> Result<(Vec<f64>, Vec<String>), String> {
    let dist = a
        .get("probabilities")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("qid {}: score answer has no probabilities", q.qid))?;
    let levels: Vec<String> = q
        .criteria
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|v| v.as_str().map_or_else(|| v.to_string(), ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let mut probs = Vec::with_capacity(levels.len());
    for lvl in &levels {
        let p = dist
            .get(lvl)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("qid {}: score probabilities missing level {lvl}", q.qid))?;
        probs.push(p);
    }
    Ok((probs, levels))
}

/// THEIR per-question abstention (the A1 probe's subject): `true` iff the
/// answer's `choice` is null — reachable ONLY under the server's
/// JEVA_ALLOW_NULL=1 posture (their coverage gate refusing the candidate
/// set: ρ ≥ max joint). At the default forced-pick posture this is always
/// false by construction. Noul/score answers carry `choice: null` BY
/// THEIR WIRE (the field is choice-kind-only) — only CHOICE questions
/// can abstain; the other kinds read false.
pub fn map_abstains(case: &SuiteCase, body: &Value) -> Result<Vec<bool>, String> {
    let answers = body
        .get("answers")
        .and_then(Value::as_object)
        .ok_or_else(|| "response has no answers object".to_string())?;
    let mut out = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let a = answers
            .get(&q.qid)
            .ok_or_else(|| format!("qid {}: no answer in response", q.qid))?;
        let abstained = q.kind == QKind::Choice && a.get("choice") == Some(&Value::Null);
        out.push(abstained);
    }
    Ok(out)
}

/// THEIR wire `confidence` — required for EVERY kind (their inference
/// always emits it, noul included; the one semantic difference from d1's
/// wire). Its absence is a loud error, not a fallback: the wire drifted.
fn required_conf(a: &Value, q: &SuiteQuestion) -> Result<f64, String> {
    a.get("confidence")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("qid {}: answer carries no `confidence` number (their inference always emits it — the wire drifted)", q.qid))
}

/// THEIR wire `confidence` per question, in OUR question order: `Some`
/// for every kind (the jeva wire emits confidence on noul too — unlike
/// d1's noul-absence law; see [`map_answers`]). A missing field is a
/// loud error, never a silent fallback.
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
        out.push(Some(required_conf(a, q)?));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn case(kind: QKind, criteria: Value) -> SuiteCase {
        SuiteCase {
            id: "c0".into(),
            state: Value::String("state text".into()),
            questions: vec![SuiteQuestion {
                qid: "q0".into(),
                kind,
                instructions: "pick".into(),
                criteria,
            }],
            gold: vec![],
        }
    }

    #[test]
    fn body_questions_are_an_object_keyed_by_qid() {
        let c = case(QKind::Noul, Value::Null);
        let body = build_body(&c).unwrap();
        assert_eq!(body["state"], json!("state text"));
        assert!(body["questions"].is_object());
        assert!(body["questions"].get("q0").is_some());
    }

    #[test]
    fn noul_body_carries_no_criteria_field() {
        // Their Noul dataclass has NO criteria — the field is omitted,
        // never `null` (their server 400s an unknown field class).
        let c = case(QKind::Noul, Value::Null);
        let body = build_body(&c).unwrap();
        assert!(body["questions"]["q0"].get("criteria").is_none());
        assert_eq!(body["questions"]["q0"]["type"], json!("noul"));
    }

    #[test]
    fn choice_body_passes_the_object_through_verbatim() {
        let c = case(
            QKind::Choice,
            json!({"billing": "invoices", "eng": "outages"}),
        );
        let body = build_body(&c).unwrap();
        assert_eq!(
            body["questions"]["q0"]["criteria"],
            json!({"billing": "invoices", "eng": "outages"})
        );
    }

    #[test]
    fn score_body_passes_the_array_through_verbatim() {
        let c = case(QKind::Score, json!(["lo", "mid", "hi"]));
        let body = build_body(&c).unwrap();
        assert_eq!(
            body["questions"]["q0"]["criteria"],
            json!(["lo", "mid", "hi"])
        );
    }

    #[test]
    fn score_with_one_level_refuses() {
        let c = case(QKind::Score, json!(["only"]));
        assert!(build_body(&c).is_err());
    }

    #[test]
    fn choice_body_bare_labels_go_as_their_sequence_arm() {
        // Their `_criteria` mapping arm REJECTS empty descriptions (the
        // measured 400 on emotion/banking77/semantic_defects); the
        // sequence arm sets texts = keys. Null and empty-string values
        // both read as bare labels.
        let null_c = case(QKind::Choice, json!({"joy": null, "sadness": null}));
        let body = build_body(&null_c).unwrap();
        assert_eq!(
            body["questions"]["q0"]["criteria"],
            json!(["joy", "sadness"])
        );

        let empty_c = case(QKind::Choice, json!({"joy": "", "sadness": "  "}));
        let body = build_body(&empty_c).unwrap();
        assert_eq!(
            body["questions"]["q0"]["criteria"],
            json!(["joy", "sadness"])
        );
    }

    #[test]
    fn choice_body_augmented_bare_labels_stay_the_sequence_arm() {
        // The null-probe's augmented arms on bare-label suites arrive
        // MIXED: originals carry null, decoys carry value == key (the
        // augmentation's own namespace). Their faithful rendering is the
        // sequence arm — texts = keys, presentation consistent across K
        // arms (the A2 probe compares across them).
        let c = case(
            QKind::Choice,
            json!({"joy": null, "sadness": null, "decoy_a": "decoy_a"}),
        );
        let body = build_body(&c).unwrap();
        assert_eq!(
            body["questions"]["q0"]["criteria"],
            json!(["joy", "sadness", "decoy_a"])
        );
    }

    #[test]
    fn choice_body_mixed_descriptions_stay_the_mapping_arm() {
        // Any nonempty description keeps the mapping arm (their option
        // text renders "key: description"); a mixed map with an empty
        // value is a per-case loud refusal (the harness law), never a
        // silent description drop.
        let c = case(QKind::Choice, json!({"billing": "invoices", "eng": ""}));
        let body = build_body(&c).unwrap();
        assert_eq!(
            body["questions"]["q0"]["criteria"],
            json!({"billing": "invoices", "eng": ""})
        );
    }

    #[test]
    fn sequence_arm_answers_read_in_our_key_order() {
        // The sequence arm's labels ARE the key strings — the response
        // mapping (keyed_probs_by_object over OUR object keys) is
        // unchanged by what went on the wire.
        let c = case(QKind::Choice, json!({"joy": null, "sadness": null}));
        let body = json!({
            "answers": {
                "q0": {
                    "probabilities": {"joy": 0.3, "sadness": 0.7},
                    "choice": "sadness",
                    "confidence": 0.7,
                }
            }
        });
        let triples = map_answers(&c, &body).unwrap();
        assert_eq!(triples[0].0, vec![0.3, 0.7]);
        assert_eq!(triples[0].1, 1);
    }

    #[test]
    fn noul_answer_reads_the_conditional_and_clamps_the_pair() {
        // p(yes)=0.7 → [0.3, 0.7], pick 1
        let c = case(QKind::Noul, Value::Null);
        let body = json!({"answers": {"q0": {"noul": 0.7, "confidence": 0.7}}});
        let out = map_answers(&c, &body).unwrap();
        assert!((out[0].0[0] - 0.3).abs() < 1e-9, "{:?}", out[0].0);
        assert!((out[0].0[1] - 0.7).abs() < 1e-9, "{:?}", out[0].0);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.7).abs() < 1e-12);
    }

    #[test]
    fn choice_answers_map_in_our_option_order() {
        let c = case(
            QKind::Choice,
            json!({"a": "first", "b": "second", "c": "third"}),
        );
        // Server returns keyed probabilities + their pick "c" (index 2).
        let body = json!({"answers": {"q0": {
            "probabilities": {"a": 0.2, "b": 0.5, "c": 0.3},
            "choice": "c", "confidence": 0.5, "null_probability": 0.0
        }}});
        let out = map_answers(&c, &body).unwrap();
        assert_eq!(out[0].0, vec![0.2, 0.5, 0.3]);
        assert_eq!(out[0].1, 2);
        assert!((out[0].2 - 0.5).abs() < 1e-12);
    }

    #[test]
    fn choice_null_under_allow_null_falls_back_to_joint_argmax_disclosed() {
        let c = case(QKind::Choice, json!({"a": "x", "b": "y"}));
        let body = json!({"answers": {"q0": {
            "probabilities": {"a": 0.2, "b": 0.4},
            "choice": null, "confidence": 0.6, "null_probability": 0.6
        }}});
        let out = map_answers(&c, &body).unwrap();
        // Joint probs are (1-ρ)·q → b=0.4 > a=0.2 → pick 1 (the argmax
        // fallback for an abstain, disclosed in the module doc).
        assert_eq!(out[0].1, 1);
    }

    #[test]
    fn choice_probability_missing_key_is_a_loud_error() {
        let c = case(QKind::Choice, json!({"a": "x", "b": "y"}));
        let body = json!({"answers": {"q0": {
            "probabilities": {"a": 0.5}, "choice": "a", "confidence": 0.5
        }}});
        assert!(map_answers(&c, &body).is_err());
    }

    #[test]
    fn choice_pick_not_a_presented_key_is_a_loud_error() {
        let c = case(QKind::Choice, json!({"a": "x"}));
        let body = json!({"answers": {"q0": {
            "probabilities": {"a": 1.0}, "choice": "zzz", "confidence": 1.0
        }}});
        assert!(map_answers(&c, &body).is_err());
    }

    #[test]
    fn score_answers_map_by_level_string_and_pick_argmax() {
        let c = case(QKind::Score, json!(["lo", "mid", "hi"]));
        let body = json!({"answers": {"q0": {
            "probabilities": {"lo": 0.1, "mid": 0.7, "hi": 0.2},
            "score": 1.1, "confidence": 0.7
        }}});
        let out = map_answers(&c, &body).unwrap();
        assert_eq!(out[0].0, vec![0.1, 0.7, 0.2]);
        assert_eq!(out[0].1, 1); // argmax, never their E[level]=1.1
        assert!((out[0].2 - 0.7).abs() < 1e-12);
    }

    #[test]
    fn missing_confidence_is_a_loud_error_on_every_kind() {
        for (kind, criteria, answer) in [
            (QKind::Noul, Value::Null, json!({"noul": 0.5})),
            (
                QKind::Choice,
                json!({"a": "x"}),
                json!({"probabilities": {"a": 1.0}, "choice": "a"}),
            ),
            (
                QKind::Score,
                json!(["lo", "hi"]),
                json!({"probabilities": {"lo": 0.5, "hi": 0.5}, "score": 0.5}),
            ),
        ] {
            let c = case(kind, criteria);
            let body = json!({"answers": {"q0": answer}});
            assert!(map_answers(&c, &body).is_err(), "{kind:?}");
            assert!(map_confidences(&c, &body).is_err(), "{kind:?}");
        }
    }

    #[test]
    fn confidences_are_some_for_every_kind_including_noul() {
        // The semantic difference from d1: jeva emits confidence on noul.
        let mut c = case(QKind::Noul, Value::Null);
        c.questions.push(SuiteQuestion {
            qid: "q1".into(),
            kind: QKind::Choice,
            instructions: "pick".into(),
            criteria: json!({"a": "x"}),
        });
        let body = json!({"answers": {
            "q0": {"noul": 0.8, "confidence": 0.8},
            "q1": {"probabilities": {"a": 1.0}, "choice": "a", "confidence": 1.0}
        }});
        let confs = map_confidences(&c, &body).unwrap();
        assert_eq!(confs.len(), 2);
        assert!(confs.iter().all(Option::is_some));
    }

    #[test]
    fn answers_missing_qid_is_an_error() {
        let c = case(QKind::Noul, Value::Null);
        let body = json!({"answers": {}});
        assert!(map_answers(&c, &body).is_err());
    }

    #[test]
    fn url_parses_host_port_and_defaults() {
        let lane = JevaLane::from_url("http://127.0.0.1:9001");
        assert_eq!(lane.port, 9001);
        let lane = JevaLane::from_url(DEFAULT_SERVE_URL);
        assert_eq!(lane.port, 8095);
    }

    #[test]
    fn model_of_reads_the_response_model_field() {
        assert_eq!(
            JevaLane::model_of(&json!({"model": "LIQIIIII/Jeva-4B"})),
            "LIQIIIII/Jeva-4B"
        );
        // The display fallback, never a hardcoded claim (the gliner law):
        assert_eq!(JevaLane::model_of(&json!({})), "jeva-4b");
    }

    #[test]
    fn choice_abstention_reads_choice_null_only_for_choice_kind() {
        let mut c = case(QKind::Choice, json!({"a": "x", "b": "y"}));
        c.questions.push(SuiteQuestion {
            qid: "q1".into(),
            kind: QKind::Noul,
            instructions: "ask".into(),
            criteria: Value::Null,
        });
        // q0: choice null under allow_null → abstained; q1: noul carries
        // choice: null BY WIRE (choice-kind-only field) → NOT abstained.
        let body = json!({"answers": {
            "q0": {"probabilities": {"a": 0.2, "b": 0.3}, "choice": null, "confidence": 0.5, "null_probability": 0.5},
            "q1": {"noul": 0.8, "choice": null, "confidence": 0.8}
        }});
        let abs = map_abstains(&c, &body).unwrap();
        assert!(abs[0]);
        assert!(!abs[1]);
        // The forced-pick posture: choice present → false.
        let body2 = json!({"answers": {
            "q0": {"probabilities": {"a": 0.6, "b": 0.3}, "choice": "a", "confidence": 0.6, "null_probability": 0.0},
            "q1": {"noul": 0.8, "choice": null, "confidence": 0.8}
        }});
        let abs2 = map_abstains(&c, &body2).unwrap();
        assert!(!abs2[0]);
        assert!(!abs2[1]);
    }

    // The stub-listener round trip: a real TCP listener serving canned
    // /health + /decide responses — the lane's HTTP client + mapping
    // exercised end to end without their model (the d1 stub shape: a
    // serve LOOP, never a racing sequential accept — the loaded-box
    // lesson; and the request is read IN FULL so an unread close never
    // RSTs the client's response read).
    #[test]
    fn stub_listener_round_trip() {
        use std::io::{BufRead, BufReader, Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let health = br#"{"ok":true,"model":"LIQIIIII/Jeva-4B","dtype":"torch.bfloat16","device":"cuda","cache":"hybrid","allow_null":false,"lora_merged":true,"license":"UNDECLARED"}"#;
        let decide = br#"{"model":"LIQIIIII/Jeva-4B","answers":{"q0":{"kind":"noul","probabilities":{"false":0.25,"true":0.75},"conditional_probabilities":{"false":0.25,"true":0.75},"null_probability":0.0,"choice":null,"noul":0.75,"score":null,"confidence":0.75,"candidate_probabilities":[0.25,0.75],"usage":{"input_tokens":9},"latency_ms":3.2}},"usage":{"input_tokens":9},"latency_ms":3.5}"#;
        let serve = std::thread::spawn(move || {
            // The test makes exactly THREE connections, in order: the
            // info() GET, the decide() POST, the decide_raw() POST. Serve
            // each; return after the second POST.
            let mut posts_served = 0;
            loop {
                let (stream, _) = listener.accept().unwrap();
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
                let payload: &[u8] = if request_line.starts_with("GET /health") {
                    health
                } else if request_line.starts_with("POST /decide") {
                    posts_served += 1;
                    decide
                } else {
                    eprintln!("jeva stub saw: {request_line:?}");
                    continue;
                };
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    payload.len()
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.write_all(payload);
                let _ = writer.flush();
                if request_line.starts_with("POST /decide") && posts_served >= 2 {
                    return;
                }
            }
        });

        let lane = JevaLane::from_url(&format!("http://127.0.0.1:{port}"));
        let (provenance, device) = lane.info().unwrap();
        assert_eq!(provenance, "LIQIIIII/Jeva-4B dtype=torch.bfloat16");
        assert_eq!(device, "cuda");

        let c = case(QKind::Noul, Value::Null);
        let (outcome, _ms) = lane.decide(&c).unwrap();
        assert_eq!(outcome.answers.len(), 1);
        assert!((outcome.answers[0].0[0] - 0.25).abs() < 1e-12);
        assert!((outcome.answers[0].0[1] - 0.75).abs() < 1e-12);
        assert_eq!(outcome.answers[0].1, 1);
        assert!((outcome.latency_ms.unwrap() - 3.5).abs() < 1e-9);
        assert_eq!(outcome.input_tokens, 9);
        assert!(!outcome.abstained[0]);

        let raw = lane.decide_raw(&c).unwrap();
        assert!(!raw.contains("latency_ms"));
        assert!(!raw.contains("usage"));
        assert!(raw.contains("\"noul\":0.75"));

        serve.join().unwrap();
    }
}
