//! The pplx-decider comparison lane (Issue 082): Perplexity's
//! `pplx-decider-v1.1-27b` — **#1 on the Jev Decision Index's 2026-10-07
//! edition** (balanced_raw 71.66 / balanced_skill 62.75; Jev #13, clef #18,
//! clef-flash #30) — measured on OUR frozen suites before any "use it"
//! claim. The B5 law cuts both ways: clef is #18-by-skill on JDI yet holds
//! our banking77/massive/sst5 bars; JDI rank is not our-suite rank. This
//! lane produces the read that decides (reflex `.issues/082`).
//!
//! **License — the lane's first divergence from its license-restricted
//! siblings:** Apache-2.0, UNGATED, open weights (11 safetensors shards;
//! verified at filing against the HF api `perplexity-ai/pplx-decider-v1.1-27b`,
//! repo revision `3b45dead91dfa6d95aad6b95764a606fab2bf7a6`, published
//! 2026-10-05). Unlike drex (CC BY-NC: measurement only, never a teacher)
//! and d1 (lfm1.0: same law), pplx IS teacher-eligible — it is riir-train
//! Issue 623's second teacher candidate, and THIS LANE'S READ decides the
//! pick (no teacher capture before the read exists; the 082 law).
//!
//! **The model is a classifier, not a generator** (pipeline_tag
//! `text-classification`): Qwen3.8-27B backbone finetune + a single-token
//! readout head over 255 option codes (A..Z, AA.., pinned in
//! `decision_config.json`), `attention_mode: noncausal_full_attention`
//! (bidirectional SDPA — a forward hook their `DecisionModel` installs;
//! a plain causal GENERATION serve would misread the checkpoint) and
//! `pooling: last`, `temperature: 1.0087`. No tokens are generated; a
//! forward pass yields the masked softmax over option codes directly.
//! Consequence: serving MUST go through THEIR autojev server
//! (`source/src/autojev/server.py` in the model repo — FastAPI, batch-of-8
//! forward per request, `torch==2.14.0`/`transformers==5.17.0`/
//! `flash-linear-attention==0.5.2`), the MEASURE-vs-SERVE split every
//! comparison lane holds: their stack serves, our Rust measures.
//!
//! The wire is the TypeSafe SystemOne dialect (the drex lane's lineage —
//! `POST /v1/systemone`, questions as an OBJECT keyed by qid, answers as
//! an object keyed by qid), shapes pinned from THEIR source at the repo
//! revision above — `server.py` `EvaluationRequest`/pydantic validators,
//! `types.py` `ChoiceAnswer`/`NoulAnswer`/`ScoreAnswer`, `model.py`
//! `options()`/`answer()` — not from a captured fixture:
//!
//! * the request REQUIRES a `model` field their validator checks against
//!   its alias set; this lane sends the stable alias `jev-latest`
//!   (accepted for every checkpoint version — [`MODEL_FIELD_VALUE`]);
//! * their pydantic models are `extra="forbid"`: the request carries
//!   exactly `{model, state, questions}` and each question exactly
//!   `{type, instructions, criteria?}`;
//! * choice criteria: 1..=255 entries (their `max_length=255`);
//! * score criteria: **2..=10 levels** (their `min_length=2,
//!   max_length=10`) — STRICTER than the drex wire's 1..=255; the harness
//!   suites' own 2..=10 bound already satisfies it;
//! * noul criteria: optional; omitted when our criteria is null (their
//!   default per-side prose "No / false"/"Yes / true" applies);
//! * answers: `probabilities` keyed by OUR option keys (choice) or level
//!   index `"0".."k-1"` (score), `choice` = their picked key, `noul` =
//!   p(true), `confidence` = their rescaled statistic, `usage` = token
//!   counts with `output_tokens` always 0 (no generation).
//!
//! **Strictness laws** (fields their code ALWAYS emits are loud errors when
//! missing — never a positional guess, never a half-parsed row): a missing
//! qid, a missing `probabilities` object, a noul answer without the `noul`
//! number, or a choice/score answer without `confidence` all refuse the
//! lane loudly. Recomputing their `confidence` formula from the
//! distribution would pin a copy of their code — absence means the wire
//! drifted, and the error says so. The ONE fallback kept is the choice
//! `pick` argmax when their `choice` key is absent (mathematically their
//! own `max(range(len(values)), key=…)` — the drex lane's defensive shape).
//!
//! **The no-clock law** (this lane's determinism divergence from the
//! agentjev/drex strip): their response body carries NO timing anywhere —
//! `usage` is token counts, `server-timing` is a response HEADER — so
//! [`PplxLane::decide_raw`] returns the body VERBATIM, nothing stripped.
//! The det column therefore measures compute determinism (bf16 device
//! wobble included), never their clock.
//!
//! **Posture** (disclosed, never pooled silently): a LOCAL serve on our GPU
//! box — their server bound with `AUTOJEV_HOST` + `PORT=8793` — reached
//! over plaintext HTTP (LAN or loopback; a TLS hop would be an
//! operator-run forwarder per the clef A0 law). The 52 GB bf16 checkpoint
//! exceeds a 24 GB card, so the serve posture may be an offload-patched
//! loader (accelerate `device_map="auto"` — numerically exact bf16, slower
//! per forward); `PPLX_DEVICE` carries the disclosure stamp into the
//! provenance line, and the run posture quotes it. Latency = client
//! round-trip INCLUDING the network hop to the serve box — the JDI's own
//! hosted rows disclose the same class ("network round-trip"); not
//! comparable to on-card figures. The HOSTED posture (Perplexity's own
//! endpoint, if one ships) is owner-gated and never pooled with local
//! reads — the clef law.
//!
//! **Port:** their server's `PORT` default is 8000 — the drex lane's
//! default. OUR default is **8793** so both lanes can serve beside each
//! other on one box; serve with `PORT=8793` (or point `PPLX_SERVE_URL`).
//!
//! Env contract (never a committed credential):
//!
//! * `PPLX_SERVE_URL` — default `http://127.0.0.1:8793`;
//! * `PPLX_TIMEOUT_MS` — per-request timeout, default 300000 (generous:
//!   the offload posture is slow per forward);
//! * `PPLX_DEVICE` — provenance stamp, default `cuda:0` (set it to the
//!   REAL posture, e.g. `cuda:0+cpu-offload`, when serving patched);
//! * `PPLX_API_KEY` — optional bearer (their server's `AUTOJEV_API_KEY`
//!   check; attached when set — a loopback serve usually runs without).

use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
use crate::lanes::http_mini::{self, HttpReply};

/// The env var naming the serve base URL.
pub const SERVE_URL_ENV: &str = "PPLX_SERVE_URL";
/// The default posture: loopback, port 8793 (their `PORT` default 8000 is
/// the drex lane's — this lane's default keeps both servable beside each
/// other; serve their side with `PORT=8793`).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8793";
/// The default port (see [`DEFAULT_SERVE_URL`]).
pub const DEFAULT_PORT: u16 = 8793;
/// The env var for the per-request timeout, milliseconds.
pub const TIMEOUT_ENV: &str = "PPLX_TIMEOUT_MS";
/// Default timeout: 300 s — the offload-patched posture is slow per
/// forward, and a contended GPU box slower still.
pub const DEFAULT_TIMEOUT_MS: u64 = 300_000;
/// The env var carrying the serve-posture provenance stamp.
pub const DEVICE_ENV: &str = "PPLX_DEVICE";
/// The env var for the optional bearer token (their `AUTOJEV_API_KEY`).
pub const TOKEN_ENV: &str = "PPLX_API_KEY";
/// The request `model` field: a stable alias their validator accepts for
/// every checkpoint version (`server.py` ALIASES — never a versioned name
/// that a server bump would strand).
pub const MODEL_FIELD_VALUE: &str = "jev-latest";
/// Their choice bound (`server.py` `Choice.criteria` pydantic):
/// 1..=255 entries.
pub const MAX_OPTIONS: usize = 255;
/// Their score bound (`server.py` `Score.criteria` pydantic): 2..=10
/// levels — STRICTER than the drex wire's 1..=255 (the shared-TypeSafe
/// lineage divergence; the harness suites' own 2..=10 already satisfies
/// it, the check encodes THEIR wire).
pub const SCORE_MIN_LEVELS: usize = 2;
pub const SCORE_MAX_LEVELS: usize = 10;

/// The per-question answer triple every comparison lane feeds the metrics
/// tail: `(probabilities in OUR option order, picked index, confidence)`.
pub type AnswerTriple = (Vec<f64>, usize, f64);

/// The lane: a thin plaintext-HTTP/1.1 client to their autojev server.
/// std-only (the shared `crate::lanes::http_mini` micro-client; no new
/// deps — the G-ISO-4 import law: `crate::harness::suites` + `http_mini`
/// + std + serde_json).
#[derive(Debug, Clone)]
pub struct PplxLane {
    host: String,
    port: u16,
    timeout: Duration,
    token: Option<String>,
}

impl Default for PplxLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl PplxLane {
    /// Parse an `http://host[:port]` base URL (plaintext HTTP only — the
    /// micro-client speaks no TLS; a hosted-HTTPS posture would go through
    /// an operator-run forwarder per the clef A0 law, owner-gated).
    pub fn from_url(url: &str) -> Self {
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(DEFAULT_PORT)),
            None => (rest.to_string(), DEFAULT_PORT),
        };
        Self {
            host,
            port,
            timeout: Duration::from_millis(
                std::env::var(TIMEOUT_ENV)
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS),
            ),
            token: std::env::var(TOKEN_ENV).ok().filter(|t| !t.is_empty()),
        }
    }

    /// The handshake: `GET /health` (their server's spelling) — 200 with
    /// `status: "ready"` means up AND loaded; their lazy first-load window
    /// answers `"loading"` and the refusal says so (their predict would
    /// 503 anyway — fail before the first case, not at case 0). Returns
    /// `(provenance, device)`: the model id from `/health` composed with
    /// the checkpoint directory's basename (the serving identity — never a
    /// hardcoded model id, the gliner law), and the `PPLX_DEVICE`
    /// disclosure stamp.
    pub fn info(&self) -> Result<(String, String), String> {
        let (status, _headers, body) =
            self.get("/health").map_err(|e| format!("pplx health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is their autojev server \
                 serving at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL}, \
                 serve their side with PORT=8793)?",
                String::from_utf8_lossy(&body)
            ));
        }
        let parsed: Value =
            serde_json::from_slice(&body).map_err(|e| format!("pplx health: {e}"))?;
        let state = parsed.get("status").and_then(Value::as_str).unwrap_or("");
        if state != "ready" {
            return Err(format!(
                "pplx health: status {state:?} — the model is still loading \
                 (their lazy first-load window); retry once it completes"
            ));
        }
        let model = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("autojev")
            .to_string();
        let checkpoint = parsed
            .get("checkpoint")
            .and_then(Value::as_str)
            .unwrap_or("");
        // Both separators: their serve box is Windows (backslash paths),
        // the dev box macOS — accept either spelling of the same dir.
        let basename = checkpoint
            .rsplit(['/', '\\'])
            .next()
            .filter(|s| !s.is_empty());
        let provenance = match basename {
            Some(b) => format!("{model}@{b}"),
            None => model,
        };
        let device = std::env::var(DEVICE_ENV).unwrap_or_else(|_| "cuda:0".to_string());
        Ok((provenance, device))
    }

    /// One decision round: build the body for `case`, POST it, map the
    /// answers back. Returns `(answer triples, client_ms, input_tokens)` —
    /// the client round-trip is the cross-lane latency measure (their body
    /// carries no server wall at all; the token count rides along for the
    /// runner's provenance line).
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, u64), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("pplx round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "pplx round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("pplx response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let input_tokens = input_tokens_of(&parsed).unwrap_or(0);
        Ok((answers, client_ms, input_tokens))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// compare input. Nothing is stripped: their wire carries no clock
    /// anywhere in the body (`usage` is token counts; `server-timing` is a
    /// header), so unlike the agentjev/drex timing-tail strips, the det
    /// column here measures compute determinism (bf16 device wobble
    /// included) — never their clock.
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "pplx round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    fn post(&self, body: &str) -> Result<HttpReply, String> {
        let bearer = self.token.as_deref().map(|t| format!("Bearer {t}"));
        let headers: &[(&str, &str)] = match &bearer {
            Some(b) => &[("authorization", b.as_str())],
            None => &[],
        };
        http_mini::request(
            &self.host,
            self.port,
            "POST",
            "/v1/systemone",
            Some(body.as_bytes()),
            self.timeout,
            headers,
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
/// passed through unchanged (their `describe()` renders str|dict|list on
/// their side — no our-side prose law), the questions as an OBJECT keyed
/// by qid, and the REQUIRED `model` alias. Question order and option order
/// are the harness criteria's insertion order — the label order the
/// answers map back onto.
pub fn build_body(case: &SuiteCase) -> Result<Value, String> {
    let mut questions = Map::new();
    for q in &case.questions {
        questions.insert(q.qid.clone(), build_question(q)?);
    }
    Ok(json!({
        "model": MODEL_FIELD_VALUE,
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
        // noul: criteria optional on their wire (`Noul.criteria:
        // dict{"true","false"} | None` — the per-side prose keys when
        // present; their defaults "No / false"/"Yes / true" otherwise).
        // Foreign keys would 422 on THEIR validator — the loud error is
        // theirs, never a silent crop here.
        QKind::Noul => {
            if !q.criteria.is_null() {
                qj["criteria"] = q.criteria.clone();
            }
        }
        // choice: the object we already carry (option key → description or
        // null — their `Content | None` accepts both), 1..=255 entries.
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
        // score: the array we already carry, THEIR bound 2..=10 (the
        // divergence from the drex wire's 1..=255 — see SCORE_MAX_LEVELS).
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            if !(SCORE_MIN_LEVELS..=SCORE_MAX_LEVELS).contains(&n) {
                return Err(format!(
                    "case {}: score needs {SCORE_MIN_LEVELS}..={SCORE_MAX_LEVELS} \
                     levels (their pydantic bound), got {n}",
                    q.qid
                ));
            }
            qj["criteria"] = q.criteria.clone();
        }
    }
    Ok(qj)
}

// ───────────────────────────────────────────────────────── the response wire

/// Map their `{model, answers: {qid: …}, usage}` body into one
/// `(probabilities, pick, conf)` triple per question, in OUR question
/// order and OUR option order (shapes pinned from `types.py` `Answer` +
/// `model.py` `answer()` @ repo revision `3b45dea`):
///
/// * noul: their `{"type": "noul", "noul": p_true}` → `[1-p, p]`; `pick`
///   = the top side; `conf` = the top side's probability.
/// * choice: the `probabilities` dict read in OUR criteria-key order;
///   `pick` = their `choice` key's position (argmax fallback — their own
///   `max(range(len(values)), key=…)`); `conf` = their rescaled
///   `confidence` — the FORMULA IS THEIRS, read as-is.
/// * score: the `probabilities` dict keyed `"0".."k-1"`, read in level
///   order; `pick` = the argmax (their `score` field is E[level] — a
///   float, never a pick; the drex/openthai recorded mapping); `conf` =
///   their distance-from-mode `confidence`, read as-is.
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
                        "qid {}: noul answer carries no `noul` number (their \
                         model.py answer() spells p(true) as `noul`)",
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
                    .ok_or_else(|| {
                        format!("qid {}: choice answer has no probabilities", q.qid)
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
                // STRICT: their answer() ALWAYS emits `confidence` for
                // choice/score. Recomputing their formula would pin a copy
                // of their code; absence = wire drift = loud.
                let conf = a.get("confidence").and_then(Value::as_f64).ok_or_else(|| {
                    format!(
                        "qid {}: choice answer carries no `confidence` number \
                         (their model.py answer() always emits it — the wire \
                         drifted; a recomputed substitute would pin a copy of \
                         their formula)",
                        q.qid
                    )
                })?;
                (probs, pick, conf)
            }
            QKind::Score => {
                let dist = a
                    .get("probabilities")
                    .and_then(Value::as_object)
                    .ok_or_else(|| {
                        format!("qid {}: score answer has no probabilities", q.qid)
                    })?;
                let n = q.criteria.as_array().map_or(0, Vec::len);
                let mut probs = Vec::with_capacity(n);
                for i in 0..n {
                    let p = dist.get(&i.to_string()).and_then(Value::as_f64).ok_or_else(
                        || format!("qid {}: score probabilities missing level {i}", q.qid),
                    )?;
                    probs.push(p);
                }
                // Their `score` is E[level] — a float, never the pick; the
                // pick is the distribution's argmax (the drex/openthai
                // recorded mapping).
                let pick = probs
                    .iter()
                    .enumerate()
                    .max_by(|x, y| x.1.total_cmp(y.1))
                    .map_or(0, |(i, _)| i);
                let conf = a.get("confidence").and_then(Value::as_f64).ok_or_else(|| {
                    format!(
                        "qid {}: score answer carries no `confidence` number \
                         (their model.py answer() always emits it — the wire \
                         drifted; a recomputed substitute would pin a copy of \
                         their formula)",
                        q.qid
                    )
                })?;
                (probs, pick, conf)
            }
        };
        out.push(triple);
    }
    Ok(out)
}

/// Their `usage.input_tokens` (the runner's provenance line); their
/// `output_tokens` is always 0 on this wire (a forward pass, no
/// generation) — nothing else rides their usage.
pub fn input_tokens_of(body: &Value) -> Option<u64> {
    body.pointer("/usage/input_tokens").and_then(Value::as_u64)
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

    /// A one-shot stub server: serves `canned` for EVERY request, looping
    /// on accept (the loaded-box lesson: a single accept once read a
    /// partial connect under a parallel test run), reading the headers
    /// AND the body before replying (a close with unread request bytes
    /// makes the kernel send RST, which races the client's response read —
    /// the macOS ECONNRESET flake the openthai stub documents), and
    /// capturing each `(request line, request body)` on a channel so the
    /// tests can assert what actually reached the wire.
    fn serve(canned: String) -> (std::net::SocketAddr, std::sync::mpsc::Receiver<(String, Vec<u8>)>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || loop {
            let (stream, _) = match listener.accept() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut writer = match stream.try_clone() {
                Ok(w) => w,
                Err(_) => continue,
            };
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue;
            }
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
            let _ = tx.send((request_line.clone(), body));
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                canned.len(),
                canned
            );
            let _ = writer.write_all(resp.as_bytes());
            let _ = writer.flush();
        });
        (addr, rx)
    }

    #[test]
    fn body_carries_model_field_and_questions_object_keyed_by_qid() {
        // Their EvaluationRequest REQUIRES `model` (validator-checked) and
        // forbids unknown fields — the body is exactly {model, state,
        // questions}, questions an OBJECT keyed by qid (the TypeSafe
        // dialect), noul criteria omitted when null.
        let c = case(
            json!({"b": 2, "a": 1}),
            vec![("q0", QKind::Noul, "done?", Value::Null)],
        );
        let body = build_body(&c).unwrap();
        assert_eq!(body["model"], "jev-latest");
        assert_eq!(body["state"], json!({"b": 2, "a": 1}));
        assert!(body["questions"].as_object().is_some());
        assert_eq!(body["questions"]["q0"]["type"], "noul");
        assert_eq!(body["questions"]["q0"]["instructions"], "done?");
        assert!(body["questions"]["q0"].get("criteria").is_none());
        // extra="forbid": exactly the three top-level fields their
        // validator accepts.
        assert_eq!(body.as_object().map(Map::len), Some(3));
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
        // THEIR score bound is 2..=10 (server.py Score pydantic) — UNLIKE
        // the drex wire's 1..=255: one level refuses HERE, and eleven too.
        let one = case(json!("s"), vec![("q3", QKind::Score, "r", json!(["only"]))]);
        assert!(build_body(&one).is_err());
        let eleven_levels: Vec<String> = (0..11).map(|i| format!("l{i}")).collect();
        let eleven = case(
            json!("s"),
            vec![("q4", QKind::Score, "r", json!(eleven_levels))],
        );
        assert!(build_body(&eleven).is_err());
        // Zero choice options refuses (their min_length=1).
        let zero = case(json!("s"), vec![("q5", QKind::Choice, "p", json!({}))]);
        assert!(build_body(&zero).is_err());
        // 256 choice options refuses (their max_length=255).
        let over: std::collections::BTreeMap<String, Value> =
            (0..=MAX_OPTIONS).map(|i| (format!("o{i}"), Value::Null)).collect();
        let over = case(
            json!("s"),
            vec![("q6", QKind::Choice, "p", serde_json::to_value(over).unwrap())],
        );
        assert!(build_body(&over).is_err());
    }

    #[test]
    fn answers_map_in_our_option_order() {
        // Shapes pinned from types.py Answer + model.py answer() @ 3b45dea.
        // Their score `legend` is an OBJECT keyed by level index (drex's
        // fixture carried an array — different server, same lineage).
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "done?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"a": null, "b": null})),
                ("q2", QKind::Score, "rate", json!(["lo", "mid", "hi"])),
            ],
        );
        let resp = json!({
            "model": "autojev-qwen3.8-27b",
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
                    "legend": {"0": "lo", "1": "mid", "2": "hi"},
                    "probabilities": {"0": 0.1, "1": 0.2, "2": 0.7},
                    "confidence": 0.6
                }
            },
            "usage": {"input_tokens": 87, "output_tokens": 0}
        });
        let out = map_answers(&c, &resp).unwrap();
        // noul: p(true) rides the `noul` field → [1-p, p], pick 1.
        assert!((out[0].0[0] - 0.15).abs() < 1e-12);
        assert!((out[0].0[1] - 0.85).abs() < 1e-12);
        assert_eq!(out[0].1, 1);
        assert!((out[0].2 - 0.85).abs() < 1e-12);
        // choice: our criteria-key order [a, b]; pick = their `choice`
        // key's position; conf = THEIR rescaled confidence, read as-is.
        assert_eq!(out[1].0, vec![0.3, 0.7]);
        assert_eq!(out[1].1, 1);
        assert!((out[1].2 - 0.55).abs() < 1e-12);
        // score: level order; pick = the argmax (their `score` E[level]
        // float is never the pick); conf = their distance-from-mode
        // statistic read as-is.
        assert_eq!(out[2].0, vec![0.1, 0.2, 0.7]);
        assert_eq!(out[2].1, 2);
        assert!((out[2].2 - 0.6).abs() < 1e-12);
        // Provenance: the token count rides their usage.
        assert_eq!(input_tokens_of(&resp), Some(87));
    }

    #[test]
    fn answers_missing_qid_is_an_error() {
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let resp = json!({"answers": {}});
        assert!(map_answers(&c, &resp).is_err());
    }

    #[test]
    fn missing_choice_confidence_is_a_loud_error_not_a_fallback() {
        // Their model.py answer() ALWAYS emits `confidence` for
        // choice/score — a body without it is a drifted wire, and a
        // recomputed substitute would pin a copy of their formula (the
        // strictness law lives IN map_answers on this lane — there is no
        // separate confidences mapper to carry it).
        let c = case(
            json!("s"),
            vec![("q1", QKind::Choice, "pick", json!({"a": null, "b": null}))],
        );
        let resp = json!({
            "answers": {
                "q1": {"type": "choice", "choice": "b",
                       "probabilities": {"a": 0.3, "b": 0.7}}
            }
        });
        let err = map_answers(&c, &resp).unwrap_err();
        assert!(err.contains("confidence"), "loud error names the axis: {err}");
        // The score arm carries the same law.
        let s = case(json!("s"), vec![("q2", QKind::Score, "r", json!(["lo", "hi"]))]);
        let sresp = json!({
            "answers": {
                "q2": {"type": "score", "score": 0.5,
                       "probabilities": {"0": 0.5, "1": 0.5}}
            }
        });
        assert!(map_answers(&s, &sresp).unwrap_err().contains("confidence"));
    }

    #[test]
    fn noul_answer_without_the_noul_field_is_a_loud_error() {
        // Their answer() ALWAYS spells p(true) as `noul` for noul answers
        // — a probabilities-dict shape is a different (broken) server, not
        // a tolerance case.
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let bad = json!({"answers": {"q0": {"probabilities": {"true": 0.8}}}});
        assert!(map_answers(&c, &bad).is_err());
    }

    #[test]
    fn choice_pick_falls_back_to_argmax_without_their_choice_key() {
        // The ONE defensive fallback (their own argmax — the drex shape);
        // `confidence` stays strict.
        let c = case(
            json!("s"),
            vec![("q1", QKind::Choice, "pick", json!({"a": null, "b": null}))],
        );
        let resp = json!({
            "answers": {
                "q1": {"type": "choice", "confidence": 0.7,
                       "probabilities": {"a": 0.3, "b": 0.7}}
            }
        });
        let out = map_answers(&c, &resp).unwrap();
        assert_eq!(out[0].1, 1, "argmax fallback when `choice` is absent");
    }

    #[test]
    fn url_parses_host_port_and_defaults() {
        let lane = PplxLane::from_url("http://127.0.0.1:8793");
        assert_eq!(lane.host, "127.0.0.1");
        assert_eq!(lane.port, 8793);
        let bare = PplxLane::from_url("http://localhost");
        assert_eq!(bare.port, DEFAULT_PORT);
        assert_eq!(bare.port, 8793);
    }

    /// T1's stub-listener round trip (the clm T2(b) pattern, drex's
    /// shape): proves the CLIENT path end to end (connect → POST
    /// /v1/systemone → parse → map) against their exact response shape,
    /// no live service needed — AND captures the request that reached the
    /// wire, pinning the REQUIRED `model` alias (their validator would
    /// 422 any misspelling).
    #[test]
    fn stub_listener_round_trip() {
        let canned = json!({
            "model": "autojev-qwen3.8-27b",
            "answers": {
                "q0": {"type": "noul", "noul": 0.25},
                "q1": {"type": "choice", "choice": "x", "confidence": 0.6,
                       "probabilities": {"x": 0.6, "y": 0.4}}
            },
            "usage": {"input_tokens": 42, "output_tokens": 0}
        })
        .to_string();
        let (addr, rx) = serve(canned);
        let lane = PplxLane::from_url(&format!("http://{addr}"));
        let c = case(
            json!("s"),
            vec![
                ("q0", QKind::Noul, "d?", Value::Null),
                ("q1", QKind::Choice, "pick", json!({"x": null, "y": null})),
            ],
        );
        let (answers, _client_ms, tokens) = lane.decide(&c).expect("round trip");
        let (line, body) = rx.recv().expect("captured request");
        assert!(line.starts_with("POST /v1/systemone HTTP/1.1"), "saw {line:?}");
        let seen: Value = serde_json::from_slice(&body).expect("request body is json");
        assert_eq!(seen["model"], "jev-latest");
        assert_eq!(seen["questions"]["q0"]["type"], "noul");
        assert_eq!(seen["questions"]["q1"]["type"], "choice");
        // noul: p(yes) 0.25 → pick 0 (no), conf 0.75 (the no side).
        assert!((answers[0].0[1] - 0.25).abs() < 1e-12);
        assert_eq!(answers[0].1, 0);
        assert!((answers[0].2 - 0.75).abs() < 1e-12);
        // choice: their choice key "x" → position 0.
        assert_eq!(answers[1].1, 0);
        // Provenance rides the outcome.
        assert_eq!(tokens, 42);
    }

    /// The no-clock law: their body carries no timing anywhere, so
    /// `decide_raw` is VERBATIM — identical serves compare byte-identical
    /// with nothing stripped, and the det column measures compute, never
    /// their clock (the agentjev/drex strips have nothing to remove here).
    #[test]
    fn decide_raw_is_verbatim_no_clock_to_strip() {
        let canned = json!({
            "model": "autojev-qwen3.8-27b",
            "answers": {"q0": {"type": "noul", "noul": 0.25}},
            "usage": {"input_tokens": 9, "output_tokens": 0}
        })
        .to_string();
        let (addr, _rx) = serve(canned.clone());
        let lane = PplxLane::from_url(&format!("http://{addr}"));
        let c = case(json!("s"), vec![("q0", QKind::Noul, "d?", Value::Null)]);
        let raw1 = lane.decide_raw(&c).expect("round trip 1");
        let raw2 = lane.decide_raw(&c).expect("round trip 2");
        assert_eq!(raw1, raw2);
        assert_eq!(raw1, canned, "verbatim — nothing is stripped on this wire");
    }

    /// The `/health` handshake: `status: "ready"` composes the provenance
    /// (model@checkpoint-basename — both path separators, the serve box is
    /// Windows), and `"loading"` REFUSES loudly (their lazy first-load
    /// window — fail at the handshake, not at case 0).
    #[test]
    fn health_parses_provenance_and_refuses_while_loading() {
        let ready = json!({
            "status": "ready",
            "model": "autojev-qwen3.8-27b",
            "checkpoint": "E:\\pplx\\pplx-decider-v1.1-27b",
            "authentication": false,
            "modalities": ["text", "image"]
        })
        .to_string();
        let (addr, _rx) = serve(ready);
        let lane = PplxLane::from_url(&format!("http://{addr}"));
        let (provenance, device) = lane.info().expect("info");
        assert_eq!(provenance, "autojev-qwen3.8-27b@pplx-decider-v1.1-27b");
        assert!(!device.is_empty(), "the PPLX_DEVICE stamp always rides");

        let loading = json!({
            "status": "loading",
            "model": "autojev-qwen3.8-27b",
            "checkpoint": "E:\\pplx\\pplx-decider-v1.1-27b",
            "authentication": false,
            "modalities": ["text", "image"]
        })
        .to_string();
        let (addr2, _rx2) = serve(loading);
        let lane2 = PplxLane::from_url(&format!("http://{addr2}"));
        let err = lane2.info().unwrap_err();
        assert!(err.contains("loading"), "the refusal names the window: {err}");
    }
}
