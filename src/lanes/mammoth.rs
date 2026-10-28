//! The mammoth/elephant hosted comparison lane (rethink Issue 032 T10 /
//! Research 012): Base Labs' **mammoth-1** (27B — the JDI 0.4.1 board-#1
//! public_skill print 75.34) and **elephant-1** (9B — 72.24), the FIRST
//! permissively-licensed board-#1 models, served on Baseten Model APIs and
//! measured on OUR frozen suites (the B5 law: board rank is never
//! our-suite rank).
//!
//! **License — the startlux contrast:** weights MIT (both models), no NC
//! posture, no RESTRICTED lineage. Teacher capture from the SELF-HOSTED
//! weights is the riir-train 640 lane and is NOT this lane's business.
//!
//! **MEASUREMENT-ONLY (the T10.2 law, binding):** API outputs never enter
//! a capture artifact — this lane deliberately ships NO RIDT arm, no
//! artifact writer, no raw-body persistence beyond the determinism
//! rerun's in-memory compare. The teacher lineage stays purely
//! self-hosted. The T10.2 window discipline rides the RUNBOOK, not code:
//! re-fetch the Baseten ToS AND any Model-APIs usage policy immediately
//! before the run (the staleness law — the SubQ precedent), free week
//! ends 2026-10-27 23:59 PT, zero data retention on by default.
//!
//! **Transport (the clef A0 law, verbatim posture):** the harness lanes
//! are std-only plaintext HTTP (the shared `crate::lanes::http_mini`
//! micro-client), so the hosted HTTPS endpoint (`inference.baseten.co`)
//! is reached through an OPERATOR-RUN loopback TLS-terminating forwarder
//! (a ~20-line process OUTSIDE this repo — the no-sidecar repo law; it
//! may inject the bearer token). The extra hop is part of serving latency
//! and is DISCLOSED in the run posture — a hosted row is never pooled
//! with on-card latencies. Direct HTTPS via a feature-gated TLS dep is
//! the owner-gated boundary-change alternative — NOT taken.
//!
//! **Wire:** the pplx/d3/startlux family's SystemOne dialect — the wire
//! functions are REUSED from the pplx lane ([`crate::lanes::pplx::
//! build_body_for_model`]/`map_answers`/`input_tokens_of`), one wire,
//! many oracles. The `model` field carries the Baseten catalog id
//! ([`MODEL_ENV`], never hardcoded — the gliner law).
//!
//! **Handshake (the clef law):** no `GET /health` is assumed on the
//! hosted route — the handshake is ONE minimal `noul` probe
//! ([`MammothLane::info`]); a 200 + a well-formed answers object proves
//! the wire AND the key, and the reply's own `model` field is the
//! provenance when the envelope provides it. The shape claim stays "the
//! pplx-family wire per the vendor; verified at the probe": every read is
//! STRICT (the pplx parser's own loud errors), never a positional guess.
//!
//! Env contract (never a committed credential):
//!
//! * `MAMMOTH_SERVE_URL` — default `http://127.0.0.1:8793` (the forwarder);
//! * `MAMMOTH_API_TOKEN` — Baseten key, attached as a bearer when set;
//!   REQUIRED unless `MAMMOTH_FORWARDER_AUTH=1` (the explicit
//!   forwarder-injects acknowledgment) — the no-creds gate refuses at
//!   lane construction, never a half-run;
//! * `MAMMOTH_MODEL` — the catalog id; default the mammoth-1 id from the
//!   T10 record (`decision-models/baseten-mammoth-27b-latest`); the
//!   elephant parity arm sets its own id;
//! * `MAMMOTH_DEVICE` — the posture disclosure stamp, default
//!   `baseten-model-apis` (the API IS the full-precision serve; a row
//!   still quotes the stamp — the lossy-surface law's provenance half);
//! * `MAMMOTH_TIMEOUT_MS` — default 120000 (a shared free-week endpoint
//!   is slow per request);
//! * `MAMMOTH_SMOKE_MAX_CASES` — spend ceiling, default 50 cases (free
//!   week ≠ unmetered); a larger run refuses unless
//!   `MAMMOTH_ALLOW_UNCAPPED=1` (the clef A6 law).
//!
//! The lane is ALSO the self-hosted capture path (`--distill-teacher
//! mammoth`, riir-train 640 T3b — the elephant arm's mechanical twin over
//! the same qev runtime): point `MAMMOTH_SERVE_URL` at the operator rig's
//! local OneJev serve (`.raw/mammoth-local/serve_mammoth.py`, loopback
//! 8793), set `MAMMOTH_MODEL=mammoth-1` (qev's model_name = the dir
//! basename), `MAMMOTH_FORWARDER_AUTH=1` (the explicit no-credential ack),
//! and `MAMMOTH_DEVICE=local-mps-bf16` (the honest posture stamp). Weights
//! MIT (640 T0, capture-time re-verified); the block-bidirectional mask
//! arms from the checkpoint's own modeling_overrides — the window's
//! bootcheck reads the `attention` field off the serve log (qev logs it at
//! engine construction; a causal serve reads as the wrong model, the 635
//! −26 pts class).

use std::time::{Duration, Instant};

use serde_json::Value;

use crate::harness::suites::SuiteCase;
use crate::lanes::http_mini;
use crate::lanes::pplx::{AnswerTriple, build_body_for_model, input_tokens_of, map_answers};

/// The env var naming the (forwarder) base URL.
pub const SERVE_URL_ENV: &str = "MAMMOTH_SERVE_URL";
/// The default posture: the operator-run loopback TLS forwarder.
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8793";
/// The bearer token (the Baseten API key; attached when set).
pub const TOKEN_ENV: &str = "MAMMOTH_API_TOKEN";
/// The explicit forwarder-injects acknowledgment (`=1`) — the token-gate's
/// escape hatch when the forwarder holds the credential.
pub const FORWARDER_AUTH_ENV: &str = "MAMMOTH_FORWARDER_AUTH";
/// The catalog id (default the T10 record's mammoth-1 id).
pub const MODEL_ENV: &str = "MAMMOTH_MODEL";
pub const DEFAULT_MODEL: &str = "decision-models/baseten-mammoth-27b-latest";
/// The posture disclosure stamp.
pub const DEVICE_ENV: &str = "MAMMOTH_DEVICE";
pub const DEFAULT_DEVICE: &str = "baseten-model-apis";
/// Per-request timeout, milliseconds.
pub const TIMEOUT_ENV: &str = "MAMMOTH_TIMEOUT_MS";
pub const DEFAULT_TIMEOUT_MS: u64 = 120_000;
/// The spend ceiling (cases per suite), the clef A6 law.
pub const MAX_CASES_ENV: &str = "MAMMOTH_SMOKE_MAX_CASES";
pub const DEFAULT_MAX_CASES: usize = 50;
/// The explicit uncapped override (`=1`).
pub const UNCAPPED_ENV: &str = "MAMMOTH_ALLOW_UNCAPPED";

/// The lane: a thin plaintext-HTTP/1.1 client to the loopback forwarder in
/// front of Baseten's hosted SystemOne route. std-only (zero new deps).
#[derive(Debug, Clone)]
pub struct MammothLane {
    host: String,
    port: u16,
    token: Option<String>,
    model: String,
    timeout: Duration,
}

impl MammothLane {
    /// Build from the env contract. REFUSES (loud, naming the missing envs)
    /// when neither `MAMMOTH_API_TOKEN` nor the explicit
    /// `MAMMOTH_FORWARDER_AUTH=1` acknowledgment is present — the no-creds
    /// gate (the clef A5 law: the loud refusal IS the gate).
    pub fn from_env() -> Result<Self, String> {
        let url = std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into());
        let token = std::env::var(TOKEN_ENV).ok().filter(|t| !t.is_empty());
        let forwarder_auth = std::env::var(FORWARDER_AUTH_ENV).is_ok_and(|v| v == "1");
        if token.is_none() && !forwarder_auth {
            return Err(format!(
                "refusing: neither {TOKEN_ENV} nor {FORWARDER_AUTH_ENV}=1 is set — the \
                 no-creds gate (T10.1: the Baseten key is an OWNER act; the free-week \
                 window ends 2026-10-27 23:59 PT); the forwarder-injects posture is the \
                 explicit {FORWARDER_AUTH_ENV}=1 acknowledgment"
            ));
        }
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(&url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(80)),
            None => (rest.to_string(), 80),
        };
        Ok(Self {
            host,
            port,
            token,
            model: std::env::var(MODEL_ENV)
                .ok()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| DEFAULT_MODEL.into()),
            timeout: Duration::from_millis(
                std::env::var(TIMEOUT_ENV)
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS),
            ),
        })
    }

    /// The served model id (the env-selected catalog id — provenance, never
    /// a hardcoded name).
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The posture disclosure stamp ([`DEVICE_ENV`]).
    pub fn device(&self) -> String {
        std::env::var(DEVICE_ENV).unwrap_or_else(|_| DEFAULT_DEVICE.into())
    }

    /// Build directly (the test seam — the env contract is pinned by the
    /// constants + the creds-gate test; the wire tests never touch process
    /// env).
    #[cfg(test)]
    pub(crate) fn for_test(addr: std::net::SocketAddr, token: Option<&str>, model: &str) -> Self {
        Self {
            host: addr.ip().to_string(),
            port: addr.port(),
            token: token.map(str::to_string),
            model: model.to_string(),
            timeout: Duration::from_secs(10),
        }
    }

    /// The handshake (the clef law): ONE minimal `noul` probe. A 200 with a
    /// well-formed `answers` object proves the wire shape AND the key; the
    /// reply's own `model` field is the provenance when the envelope
    /// provides it. Returns `(model_echo, device)`.
    pub fn info(&self) -> Result<(String, String), String> {
        let probe = SuiteCase {
            id: "handshake".into(),
            state: Value::String(
                "handshake: the lane's minimal noul probe (the clef A2 law; \
                 discarded, never a measured case)"
                    .into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "probe".into(),
                kind: crate::harness::suites::QKind::Noul,
                instructions: "Is this the handshake probe?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let body = build_body_for_model(&probe, &self.model)?.to_string();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("mammoth handshake: {e}"))?;
        if status != 200 {
            return Err(format!(
                "mammoth handshake -> HTTP {status}: {} — is the forwarder up and \
                 pointing at inference.baseten.co/v1/systemone, and is {} a valid key? \
                 (401/403 = the key; 404 = the route; the T10.2 discipline: re-fetch \
                 the Baseten ToS + Model-APIs policy BEFORE the run)",
                String::from_utf8_lossy(&raw),
                TOKEN_ENV
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw).map_err(|e| {
            format!(
                "mammoth handshake parse: {e} — the reply is not the \
                 SystemOne shape; check the forwarder route"
            )
        })?;
        // STRICT: the answers object must exist and carry the probe's qid —
        // the wire proof, never a half-check (the clef A2 law).
        let answers = parsed
            .get("answers")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                "mammoth handshake: no answers object — not the SystemOne wire".to_string()
            })?;
        if !answers.contains_key("probe") {
            return Err(
                "mammoth handshake: answers carries no qid for the probe — not the \
                 SystemOne wire shape this lane pins"
                    .to_string(),
            );
        }
        let echo = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or(&self.model)
            .to_string();
        Ok((echo, self.device()))
    }

    /// One decision round: build the body (the pplx wire grammar, the
    /// catalog id as the model field), POST it, map the answers back (the
    /// pplx parser, verbatim). Returns `(answer triples, client_ms,
    /// input_tokens)` — the client round-trip (forwarder hop included) is
    /// the cross-lane latency measure and is DISCLOSED as hosted, never
    /// pooled with on-card figures.
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, u64), String> {
        let body = build_body_for_model(case, &self.model)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("mammoth round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "mammoth round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("mammoth response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let input_tokens = input_tokens_of(&parsed).unwrap_or(0);
        Ok((answers, client_ms, input_tokens))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// in-memory compare input. Nothing is persisted (the MEASUREMENT-ONLY
    /// law: API outputs never enter a capture artifact).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body_for_model(case, &self.model)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "mammoth round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    /// The determinism-compare projection of one raw body: the decision
    /// content (`model` + `answers` + `usage`) with the serve-meta block
    /// REMOVED. A qev-backed serve (the local MPS rig AND the hosted
    /// Baseten route — same stack) stamps wall-clock timing inside the
    /// envelope's `qev` key, which varies per call BY DESIGN; the pplx
    /// bodies carry no clock (their lane compares verbatim — the recorded
    /// divergence, the pplx test `decide_raw_is_verbatim_no_clock_to_strip`).
    /// Comparing the raw envelope would read that clock as answer
    /// instability on every suite (measured 2026-10-24, sst5: the answers
    /// byte-identical across repeated probes, `determinism_ok` false on
    /// the timing artifact alone). The projection stays in memory —
    /// nothing persisted (the MEASUREMENT-ONLY law).
    pub fn determinism_view(raw: &str) -> Result<String, String> {
        let parsed: Value =
            serde_json::from_str(raw).map_err(|e| format!("determinism view parse: {e}"))?;
        let mut obj = match parsed {
            Value::Object(m) => m,
            _ => return Err("determinism view: body is not a JSON object".into()),
        };
        obj.shift_remove("qev");
        Ok(Value::Object(obj).to_string())
    }

    fn post(&self, body: &str) -> Result<http_mini::HttpReply, String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::QKind;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::sync::Mutex;

    /// The env contract is process-global; every env-touching test takes
    /// the guard (serializes them — the shared-temp-path lesson, in-process
    /// form).
    static ENV_GUARD: Mutex<()> = Mutex::new(());

    /// Set env vars for one test, restoring the previous values on drop
    /// (the sibling-agents lesson: never leak env into other tests). The
    /// CALLER wraps the constructors in `unsafe { }` — edition 2024 marks
    /// the process-env mutation unsafe; ENV_GUARD is the safety invariant
    /// (no concurrent env reader while held).
    struct EnvPatch(Vec<(String, Option<String>)>);
    impl EnvPatch {
        fn set(k: &str, v: &str) -> Self {
            let old = std::env::var(k).ok();
            // SAFETY: ENV_GUARD held by the caller — no concurrent env reader.
            unsafe { std::env::set_var(k, v) };
            Self(vec![(k.into(), old)])
        }
        fn clear(k: &str) -> Self {
            let old = std::env::var(k).ok();
            // SAFETY: ENV_GUARD held by the caller — no concurrent env reader.
            unsafe { std::env::remove_var(k) };
            Self(vec![(k.into(), old)])
        }
    }
    impl Drop for EnvPatch {
        fn drop(&mut self) {
            for (k, old) in &self.0 {
                // SAFETY: ENV_GUARD held by the test that owns this patch.
                match old {
                    Some(v) => unsafe { std::env::set_var(k, v) },
                    None => unsafe { std::env::remove_var(k) },
                }
            }
        }
    }

    fn case(state: Value, questions: Vec<(&str, QKind, &str, Value)>) -> SuiteCase {
        SuiteCase {
            id: "c0".into(),
            state,
            questions: questions
                .into_iter()
                .map(
                    |(qid, kind, instructions, criteria)| crate::harness::suites::SuiteQuestion {
                        qid: qid.into(),
                        kind,
                        instructions: instructions.into(),
                        criteria,
                    },
                )
                .collect(),
            gold: vec![],
        }
    }

    /// A one-shot stub server (the d3/startlux stub, verbatim — one shape,
    /// many lanes): serves `canned` with `status` for EVERY request.
    fn serve_with(status: u16, canned: String) -> std::net::SocketAddr {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            loop {
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
                let resp = format!(
                    "HTTP/1.1 {status} ST\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    canned.len(),
                    canned
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.flush();
            }
        });
        addr
    }

    fn serve(canned: String) -> std::net::SocketAddr {
        serve_with(200, canned)
    }

    fn answers_body(model: &str) -> String {
        json!({
            "model": model,
            "answers": {
                "probe": {"type": "noul", "noul": 0.9},
                "q_choice": {
                    "type": "choice",
                    "choice": "billing",
                    "confidence": 0.9948,
                    "probabilities": {"team": 0.003, "billing": 0.9948, "sales": 0.0022},
                },
                "q_noul": {"type": "noul", "noul": 0.7449},
            },
            "usage": {"input_tokens": 191, "output_tokens": 0},
        })
        .to_string()
    }

    #[test]
    fn no_creds_refuse_is_loud_and_names_the_gate() {
        let _g = ENV_GUARD.lock().unwrap();
        let _t = EnvPatch::set(TOKEN_ENV, "");
        let _f = EnvPatch::clear(FORWARDER_AUTH_ENV);
        let err = MammothLane::from_env().unwrap_err();
        assert!(err.contains(TOKEN_ENV), "{err}");
        assert!(err.contains(FORWARDER_AUTH_ENV), "{err}");
    }

    #[test]
    fn forwarder_auth_acknowledgment_satisfies_the_gate() {
        let _g = ENV_GUARD.lock().unwrap();
        let _t = EnvPatch::set(TOKEN_ENV, "");
        let _f = EnvPatch::set(FORWARDER_AUTH_ENV, "1");
        assert!(MammothLane::from_env().is_ok());
    }

    #[test]
    fn handshake_probes_noul_and_reads_the_model_echo() {
        let addr = serve(answers_body("the-echoed-model"));
        let lane = MammothLane::for_test(addr, Some("k-test"), DEFAULT_MODEL);
        let (echo, _device) = lane.info().unwrap();
        assert_eq!(echo, "the-echoed-model");
    }

    #[test]
    fn handshake_refuses_every_wrong_shape_loud() {
        // A non-200 (401 key, 404 route): refused naming the discipline.
        let addr = serve_with(401, json!({"error": "unauthorized"}).to_string());
        let err = MammothLane::for_test(addr, Some("k"), DEFAULT_MODEL)
            .info()
            .unwrap_err();
        assert!(err.contains("HTTP 401"), "{err}");

        // 200 but NOT the SystemOne shape (no answers object): refused.
        let addr = serve(json!({"result": {}}).to_string());
        let err = MammothLane::for_test(addr, Some("k"), DEFAULT_MODEL)
            .info()
            .unwrap_err();
        assert!(err.contains("answers"), "{err}");

        // 200 with an answers object missing the probe qid: refused —
        // the wire proof, never a half-check (the clef A2 law).
        let addr = serve(
            json!({"model": "m", "answers": {"other": {"type": "noul", "noul": 0.5}}}).to_string(),
        );
        let err = MammothLane::for_test(addr, Some("k"), DEFAULT_MODEL)
            .info()
            .unwrap_err();
        assert!(err.contains("qid"), "{err}");
    }

    #[test]
    fn stub_round_trip_carries_the_catalog_id_and_maps_answers() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let canned = answers_body(DEFAULT_MODEL);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            loop {
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
                let mut saw_auth = false;
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
                            if let Some((k, v)) = trimmed.split_once(':') {
                                if k.eq_ignore_ascii_case("content-length") {
                                    content_length = v.trim().parse().unwrap_or(0);
                                }
                                if k.eq_ignore_ascii_case("authorization")
                                    && v.trim() == "Bearer k-live-test"
                                {
                                    saw_auth = true;
                                }
                            }
                        }
                    }
                }
                let mut body = vec![0u8; content_length];
                if reader.read_exact(&mut body).is_err() {
                    continue;
                }
                let _ = tx.send((request_line.clone(), saw_auth, body));
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    canned.len(),
                    canned
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.flush();
            }
        });
        let lane = MammothLane::for_test(addr, Some("k-live-test"), DEFAULT_MODEL);
        let c = case(
            json!("My card was charged twice."),
            vec![
                (
                    "q_choice",
                    QKind::Choice,
                    "Which team should handle this?",
                    json!({"team": "The product team", "billing": "The billing team", "sales": "The sales team"}),
                ),
                ("q_noul", QKind::Noul, "damaged?", Value::Null),
            ],
        );
        let (answers, _ms, tokens) = lane.decide(&c).unwrap();
        assert_eq!(tokens, 191);
        let (probs, pick, conf) = &answers[0];
        assert_eq!(probs.len(), 3);
        assert_eq!(*pick, 1);
        assert!((conf - 0.9948).abs() < 1e-9);
        let (nprobs, npick, _nc) = &answers[1];
        assert_eq!(nprobs.len(), 2);
        assert!((nprobs[1] - 0.7449).abs() < 1e-9);
        assert_eq!(*npick, 1);

        let (request_line, saw_auth, body) =
            rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(
            request_line.starts_with("POST /v1/systemone"),
            "{request_line}"
        );
        assert!(saw_auth, "the bearer must ride the request");
        let sent: Value = serde_json::from_slice(&body).unwrap();
        // The catalog id — never a hardcoded alias — rides the model field.
        assert_eq!(sent["model"], DEFAULT_MODEL);
        assert_eq!(sent["questions"]["q_choice"]["type"], "choice");
        assert!(sent["questions"]["q_noul"].get("criteria").is_none());
    }

    #[test]
    fn decide_raw_returns_the_body_verbatim() {
        let raw = answers_body("m");
        let addr = serve(raw.clone());
        let lane = MammothLane::for_test(addr, Some("k-test"), DEFAULT_MODEL);
        let c = case(json!("s"), vec![("probe", QKind::Noul, "q?", Value::Null)]);
        assert_eq!(lane.decide_raw(&c).unwrap(), raw);
    }

    #[test]
    fn determinism_view_drops_the_serve_meta_and_keeps_the_content() {
        let raw = r#"{"model":"mammoth-1","answers":{"q1":{"type":"choice",
            "choice":"a","probabilities":{"a":0.6,"b":0.4},"confidence":0.2}},
            "usage":{"input_tokens":42,"output_tokens":1},
            "qev":{"prompt_version":"qev-labels-v2","timing":{"forward_s":1.234,
            "total_s":1.300}}}"#;
        let v1 = MammothLane::determinism_view(raw).unwrap();
        let v2 = MammothLane::determinism_view(raw).unwrap();
        assert_eq!(v1, v2, "the projection is a pure function");
        assert!(!v1.contains("timing"), "the serve-meta block is gone");
        assert!(v1.contains("\"answers\""), "the answers stay");
        assert!(v1.contains("\"usage\""), "the usage stays");
        // A body WITHOUT the meta block round-trips through the same
        // projection unchanged (the hosted-posture shape if their prod
        // strips it).
        let bare = r#"{"model":"mammoth-1","answers":{}}"#;
        let vb = MammothLane::determinism_view(bare).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&vb).unwrap();
        assert_eq!(parsed["model"], "mammoth-1");
        // Non-object bodies refuse loud (never a silent pass).
        assert!(MammothLane::determinism_view("[1,2]").is_err());
        assert!(MammothLane::determinism_view("not json").is_err());
    }
}
