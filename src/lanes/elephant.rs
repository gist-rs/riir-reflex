//! The elephant SELF-HOSTED lane (riir-train Issue 640 T2(a)/T3, feeding
//! rethink Issue 032 T10.2's local-posture row): Base Labs' `elephant-1`
//! (9B, Qwen3.5-9B full FT — JDI 0.4.1 board #4 public_skill 72.24)
//! served by OUR reference stdlib server (`scripts/elephant_server.py`)
//! over THEIR OWN runtime (OneJev `qev` @81ce62f + the repo's
//! `serve.patch` — the checkpoint's block-bidirectional + slot_lm serve
//! shape), on the M3 (`mps`) or the 4090 (`cuda:0`).
//!
//! LICENSE (riir-train 640 T0, verified 2026-10-24): weights MIT, base
//! models Apache-2.0 (Qwen3.5), runtime Apache-2.0 (OneJev) — PERMISSIVE,
//! the first board-top model with no license asterisk (the startlux
//! NC and jeva UNDECLARED contrasts). No RESTRICTED posture, adoption
//! not fail-closed. The T0 caveat: no LICENSE FILE ships in the repos —
//! the assertion is the HF tag + README front-matter, re-read at any
//! adoption act.
//!
//! CAPTURE (the contrast with the hosted [`crate::lanes::mammoth`] lane):
//! the mammoth lane deliberately ships no RIDT arm (MEASUREMENT-ONLY —
//! API outputs never enter a capture artifact). THIS lane is the
//! self-hosted twin and it IS the capture path: self-hosted weights,
//! local compute, no third party — the teacher lineage stays purely
//! self-hosted per 640 T2(a).
//!
//! Wire: the pplx/d3/mammoth family's SystemOne dialect — the wire
//! functions are REUSED from the pplx lane
//! ([`crate::lanes::pplx::build_body_for_model`]/`map_answers`/
//! `input_tokens_of`), one wire, many oracles. qev's answers are keyed
//! by OPTION LABEL (choice) / level-index string (score), confidence
//! always emitted — the pplx parser's exact expectations (verified live
//! against the checkpoint 2026-10-24, 640 T2(a) probe; probabilities
//! rounded 6 decimals by their `_round`). Their engine CHUNKS choice
//! questions over 256 slots natively (the NONE_OPTION reweighting), so
//! banking77's 77-way rows serve as one question.
//!
//! Env: `ELEPHANT_SERVE_URL` (default `http://127.0.0.1:8096`; fresh in
//! the lane family). The `/health` handshake (our server binds AFTER the
//! model loads — no lazy-load window) captures the resolved engine info
//! (dtype / attention / fork_mode) as provenance.

use std::time::{Duration, Instant};

use serde_json::Value;

use crate::harness::suites::SuiteCase;
use crate::lanes::http_mini;
use crate::lanes::pplx::{AnswerTriple, build_body_for_model, input_tokens_of, map_answers};

/// The env var naming our reference server's base URL.
pub const SERVE_URL_ENV: &str = "ELEPHANT_SERVE_URL";
/// The default posture: loopback, port 8096 (fresh in the lane family).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8096";
/// The model label the RIDT header carries (the server echoes a path tag
/// over it; this is the provenance floor).
pub const MODEL_LABEL: &str = "baselabs/elephant-1";
/// The license line (the bekko attribution law — external teachers name
/// their license in the RIDT header).
pub const LICENSE_LINE: &str = "MIT (baselabs/elephant-1; base Qwen3.5 \
Apache-2.0; runtime OneJev Apache-2.0) — T0-verified 2026-10-24 per \
riir-train 640; no LICENSE file ships (tag+front-matter assertion)";
/// Per-request timeout: a 77-way chunked serve on mps runs ~2-9 s; the
/// default is generous because the capture loop is throughput-bound, not
/// latency-bound.
pub const DEFAULT_TIMEOUT_MS: u64 = 600_000;
/// The spend ceiling (cases per suite), the clef A6 law — the SAME shape as
/// the hosted lanes even though the default serve is local: a typo'd
/// `ELEPHANT_SERVE_URL` pointing at someone else's endpoint must hit a
/// ceiling, not an unmetered run (the opendecider precedent).
pub const MAX_CASES_ENV: &str = "ELEPHANT_SMOKE_MAX_CASES";
pub const DEFAULT_MAX_CASES: usize = 50;
/// The explicit uncapped override (`=1`) — the local zero-marginal-cost
/// posture (our hardware; the panel and the capture passes ride it).
pub const UNCAPPED_ENV: &str = "ELEPHANT_ALLOW_UNCAPPED";

/// The lane: a thin loopback HTTP/1.1 client to our reference server.
/// std-only (the shared `crate::lanes::http_mini` micro-client; no new
/// deps).
#[derive(Debug, Clone)]
pub struct ElephantLane {
    host: String,
    port: u16,
    timeout: Duration,
}

impl Default for ElephantLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl ElephantLane {
    /// Build from the env contract. NO creds gate — the default posture is
    /// OUR reference server on OUR hardware (the hosted-mammoth contrast:
    /// there the loud refusal IS the gate; here there is no third party to
    /// bill and nothing to acknowledge). Not loopback-restricted: the 4090
    /// throughput arm (`cuda:0`) is the same lane pointed at the box over
    /// the LAN.
    pub fn from_env() -> Result<Self, String> {
        Ok(Self::default())
    }

    /// Parse an `http://host:port` base URL (loopback-only posture — our
    /// reference server binds 127.0.0.1 beside the harness).
    pub fn from_url(url: &str) -> Self {
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(8096)),
            None => (rest.to_string(), 8096),
        };
        Self {
            host,
            port,
            timeout: Duration::from_millis(
                std::env::var("ELEPHANT_TIMEOUT_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS),
            ),
        }
    }

    /// The liveness probe: `GET /health` — 200 means up (the model loads
    /// before the listener binds — no lazy-load window). A transport
    /// failure carries the same server-startup hint as a non-200 (the
    /// loud-refusal law: the error names the remedy, never just errno).
    pub fn health(&self) -> Result<(), String> {
        let (status, _headers, body) = self.get("/health").map_err(|e| {
            format!(
                "elephant health: {e} — is our elephant reference server serving at \
                 {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL}; start it: \
                 scripts/elephant_server.py under the qev venv — riir-train 640 \
                 T2(a) records the exact venv + weight pin)?"
            )
        })?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is our elephant reference \
                 server serving at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL}; \
                 start it: scripts/elephant_server.py under the qev venv — \
                 riir-train 640 T2(a) records the exact venv + weight pin)?",
                String::from_utf8_lossy(&body)
            ));
        }
        Ok(())
    }

    /// `(provenance, device)` off `/health` — the construct-time capture
    /// the distill seam uses (the pplx law): the resolved engine posture
    /// (dtype / attention / fork_mode) rides the provenance string, never
    /// a hardcoded name (the gliner law).
    pub fn info(&self) -> Result<(String, String), String> {
        let (status, _headers, body) = self
            .get("/health")
            .map_err(|e| format!("elephant health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {}",
                String::from_utf8_lossy(&body)
            ));
        }
        let parsed: Value =
            serde_json::from_slice(&body).map_err(|e| format!("elephant health parse: {e}"))?;
        if !parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            return Err("elephant /health carries ok=false".to_string());
        }
        let dtype = parsed
            .get("dtype")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let attention = parsed
            .get("attention")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let fork = parsed
            .get("fork_mode")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        // The attention posture is LOAD-BEARING (the 635 lesson: a
        // causal-only serve reads as the wrong model) — surface it in the
        // provenance so a causal-drift row is visible in the record.
        let provenance = format!("{MODEL_LABEL} dtype={dtype} attention={attention} fork={fork}");
        let device = parsed
            .get("device")
            .and_then(Value::as_str)
            .unwrap_or("mps")
            .to_string();
        Ok((provenance, device))
    }

    /// One decision round: build the body (the pplx wire grammar), POST
    /// it, map the answers back (the pplx parser, verbatim — qev's
    /// answer shape is its dialect's origin). Returns `(answer triples,
    /// client_ms, input_tokens)`.
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, u64), String> {
        let body = build_body_for_model(case, MODEL_LABEL)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("elephant round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "elephant round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("elephant response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let input_tokens = input_tokens_of(&parsed).unwrap_or(0);
        Ok((answers, client_ms, input_tokens))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// in-memory compare input (their engine is a deterministic one-pass
    /// forward at debias=1; the probe's repeat was bit-identical).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body_for_model(case, MODEL_LABEL)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "elephant round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    /// The determinism-compare projection of one raw body: the decision
    /// content (`model` + `answers` + `usage`) with the clock REMOVED. Our
    /// reference server stamps top-level `latency_ms` (wall-clock per call,
    /// BY DESIGN — the mammoth family's qev serve-meta clock, one level
    /// up): comparing the raw envelope would read it as answer instability
    /// on every suite (the measured 2026-10-24 sst5 lesson). The answers
    /// are the content; a qev deterministic one-pass forward repeats
    /// bit-identically (the probe's repeat). In memory only.
    pub fn determinism_view(raw: &str) -> Result<String, String> {
        let parsed: Value =
            serde_json::from_str(raw).map_err(|e| format!("determinism view parse: {e}"))?;
        let mut obj = match parsed {
            Value::Object(m) => m,
            _ => return Err("determinism view: body is not a JSON object".into()),
        };
        obj.shift_remove("latency_ms");
        Ok(Value::Object(obj).to_string())
    }

    fn post(&self, body: &str) -> Result<http_mini::HttpReply, String> {
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

    fn get(&self, path: &str) -> Result<http_mini::HttpReply, String> {
        http_mini::request(&self.host, self.port, "GET", path, None, self.timeout, &[])
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::QKind;
    use serde_json::json;

    fn sample_case() -> SuiteCase {
        SuiteCase {
            id: "c1".into(),
            state: Value::String("state".into()),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "q1".into(),
                kind: QKind::Choice,
                instructions: "pick".into(),
                criteria: json!({"A": "first", "B": "second", "C": "third"}),
            }],
            gold: vec![],
        }
    }

    // The stub-listener round trip (the jeva lane's proven shape): a
    // real TCP listener serving canned /health + /decide responses —
    // the lane's HTTP client + the pplx mapping exercised end to end
    // without the model. The serve LOOP reads each request IN FULL (an
    // unread close RSTs the client's response read) and RETURNS after
    // the second POST (never a racing accept leak).
    #[test]
    fn stub_listener_round_trip() {
        use std::io::{BufRead, BufReader, Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let health = br#"{"ok":true,"model":"baselabs/elephant-1","dtype":"bfloat16","device":"mps","attention":"noncausal","readout":"slot_lm","fork_mode":"batched","license":"MIT"}"#;
        let decide = br#"{"model":"elephant-1@elephant-1","answers":{"q1":{"type":"choice","choice":"B","probabilities":{"A":0.25,"B":0.5,"C":0.25},"confidence":0.25}},"usage":{"input_tokens":42,"output_tokens":1},"latency_ms":1.0}"#;
        let serve = std::thread::spawn(move || {
            // Three connections in order: info() GET, decide() POST,
            // decide_raw() POST.
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

        let lane = ElephantLane::from_url(&format!("http://127.0.0.1:{port}"));
        let (provenance, device) = lane.info().unwrap();
        assert!(provenance.contains("attention=noncausal"), "{provenance}");
        assert_eq!(device, "mps");

        let c = sample_case();
        let (answers, _ms, tokens) = lane.decide(&c).unwrap();
        assert_eq!(answers.len(), 1);
        let (probs, pick, conf) = &answers[0];
        assert_eq!(probs.len(), 3);
        assert_eq!(*pick, 1, "B carries the max prob");
        assert!((conf - 0.25).abs() < 1e-12);
        assert_eq!(tokens, 42);

        let raw = lane.decide_raw(&c).unwrap();
        assert!(raw.contains("\"probabilities\""), "{raw}");

        serve.join().unwrap();
    }

    #[test]
    fn determinism_view_drops_the_clock_and_keeps_the_content() {
        let raw = r#"{"model":"elephant-1@elephant-1","answers":{"q1":{"type":"choice","choice":"B","probabilities":{"A":0.25,"B":0.5,"C":0.25},"confidence":0.25}},"usage":{"input_tokens":42,"output_tokens":1},"latency_ms":1730.25}"#;
        let v1 = ElephantLane::determinism_view(raw).unwrap();
        // The same body with a DIFFERENT wall-clock stamps the same view.
        let raw2 = raw.replace("1730.25", "2411.75");
        let v2 = ElephantLane::determinism_view(&raw2).unwrap();
        assert_eq!(v1, v2, "the clock must not read as answer instability");
        assert!(!v1.contains("latency_ms"), "{v1}");
        assert!(v1.contains("probabilities"), "{v1}");
        assert!(v1.contains("input_tokens"), "{v1}");
        // A non-object body is a loud error, never a false pass.
        assert!(ElephantLane::determinism_view("[1,2]").is_err());
    }

    #[test]
    fn from_env_never_refuses_the_local_posture() {
        // The self-hosted contrast with the hosted mammoth gate: no creds
        // to acknowledge, no third party to bill.
        assert!(ElephantLane::from_env().is_ok());
    }

    #[test]
    fn health_failure_names_the_server_startup() {
        // Nothing listens here; the error must name the server script +
        // the env (the loud-refusal law).
        let lane = ElephantLane::from_url("http://127.0.0.1:1");
        let err = lane.health().expect_err("must fail");
        assert!(err.contains("elephant_server.py"), "{err}");
        assert!(err.contains(SERVE_URL_ENV), "{err}");
    }
}
