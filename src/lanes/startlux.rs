//! The StartLux-Decision v1.1 comparison lane (reflex Issue 095, feeding
//! riir-train 635 T3): `startlux-models/StartLux-Decision-v1.1-27B-Instant`
//! — board #1 (public_skill 72.88, coverage 1.0, ECE 0.0261 on the pinned
//! snapshot) — measured on OUR frozen suites before any "use it" claim
//! (the B5 law: board rank is never our-suite rank).
//!
//! **License — the lane's first divergence from its teacher-eligible
//! siblings:** weights CC BY-NC 4.0 (DECLARED-NC → RESTRICTED posture,
//! adjudicated in riir-train 635 T0: capture-for-MEASUREMENT allowed,
//! teacher-CAPTURE allowed, any ADOPTION fail-closed). Like jeva/drex, a
//! measurement-posture lane: comparison reads + RIDT captures with the
//! license token riding the header — never a product lane, never a shipped
//! column. The NC-lineage law (635, binding): every artifact deriving from
//! StartLux captures carries `NC-lineage: startlux-decision-v1.1 (CC BY-NC
//! 4.0)`. Engineering-grade reading, not legal advice.
//!
//! **Serving:** THEIR Apache-2.0 engine `startlux-decide` v1.1.0 over a
//! llama.cpp GGUF build — the MEASURE-vs-SERVE split every comparison lane
//! holds: their stack serves, our Rust measures. The Instant checkpoint
//! reads its prompt BIDIRECTIONALLY; the serve posture is non-causal
//! (`STARTLUX_LLAMACPP_BIDIR=server` over a llama-server whose context is
//! non-causal) — a causal/off reading is a DIFFERENT model than the one
//! trained (the −26 pt class, 635 T2's reason for the exact-tag build), so
//! the lane's handshake checks the posture, not just liveness. Lossy-surface
//! law: the quant posture (our T2/T2b arm: Q4_K_M, `-ngl 99`) is PINNED per
//! capture via [`DEVICE_ENV`] and quoted beside every number.
//!
//! The wire is the TypeSafe SystemOne dialect — the pplx/d3 family's
//! `/v1/systemone` — pinned from THEIR engine source at v1.1.0
//! (`startlux_decide/{server,controls,pipeline,prompt,readout}.py`), with
//! three divergences from the pplx lane's reading (all verified in source):
//!
//! * request extras are IGNORED, never forbidden: the server reads only
//!   `state`, `questions`, `images` + the known controls — the pplx
//!   [`crate::lanes::pplx::build_body`] (which sends the `model` alias)
//!   passes verbatim. The wire functions are REUSED, not forked (one wire,
//!   many oracles — the d3 law);
//! * `GET /health` answers `status: "ok"` (never "ready"/"loading"); the
//!   full shape (no API keys on a loopback serve) adds `kind`,
//!   `prompt_attention`, `engine` (with `bidir_mode`), `defaults`,
//!   `context_length` — the lane refuses loud unless
//!   `kind == "instant"` ∧ `prompt_attention == "bidirectional"` ∧
//!   `engine.bidir_mode == "server"` (the posture gate above);
//! * choice answers carry `none_probability` (`none_check` default-on at
//!   serve) — extra vs the pplx parser, which reads only what it knows and
//!   ignores unknown fields. Noul answers carry NO `confidence` on their
//!   wire (`{"type": "noul", "noul": p_true}` — the pplx parser's noul arm
//!   never reads one); score answers are the pplx shapes exactly.
//!
//! `usage` carries `input_tokens`/`output_tokens` (thought tokens);
//! `timing_ms` rides the body ONLY with `return_details` — which this lane
//! never sets — so the no-clock law holds: `decide_raw` returns the body
//! VERBATIM and the determinism column measures compute determinism,
//! never their clock.
//!
//! Bounds (their `prompt.py`): choice 2..=26 options per pass (`MAX_OPTIONS
//! = 26`, one letter per option; their engine runs knockout rounds beyond —
//! a 77-way banking77 row is their engine's multi-pass job, not ours);
//! score 2..=10 levels; noul criteria optional. The pplx `build_body`'s
//! 1..=255 choice check is looser than their single-pass bound and correct
//! on their wire.
//!
//! **Port:** the engine's `--port` default is 8090 — also OUR default (the
//! documented 635 boot line serves the wrapper there; llama-server rides
//! :8081 behind it).
//!
//! Env contract (never a committed credential):
//!
//! * `STARTLUX_SERVE_URL` — default `http://127.0.0.1:8090`;
//! * `STARTLUX_TIMEOUT_MS` — per-request timeout, default 300000 (generous:
//!   knockout multi-pass rows and a contended GPU box are slow);
//! * `STARTLUX_DEVICE` — the quant-posture disclosure stamp, default
//!   `cuda:0` (set it to the REAL posture, e.g. `cuda:0+q4_k_m-ngl99` — the
//!   lossy-surface law: which quant produced the scores gates the captures);
//! * `STARTLUX_API_KEY` — optional bearer (their `--api-keys-file` lane;
//!   attached when set — a loopback serve usually runs without).

use std::time::{Duration, Instant};

use serde_json::Value;

use crate::harness::suites::SuiteCase;
use crate::lanes::http_mini;
use crate::lanes::pplx::{AnswerTriple, build_body, input_tokens_of, map_answers};

/// The env var naming the serve base URL.
pub const SERVE_URL_ENV: &str = "STARTLUX_SERVE_URL";
/// The default posture: loopback, port 8090 — the engine's own `--port`
/// default and the documented 635 boot line (llama-server rides :8081
/// behind the wrapper).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8090";
/// The default port (see [`DEFAULT_SERVE_URL`]).
pub const DEFAULT_PORT: u16 = 8090;
/// The env var for the per-request timeout, milliseconds.
pub const TIMEOUT_ENV: &str = "STARTLUX_TIMEOUT_MS";
/// Default timeout: 300 s — knockout multi-pass rows (77-option banking77)
/// and a contended GPU box are slow per request.
pub const DEFAULT_TIMEOUT_MS: u64 = 300_000;
/// The env var carrying the serve-posture provenance stamp (the quant
/// disclosure — e.g. `cuda:0+q4_k_m-ngl99`).
pub const DEVICE_ENV: &str = "STARTLUX_DEVICE";
/// The env var for the optional bearer token (their `--api-keys-file`).
pub const TOKEN_ENV: &str = "STARTLUX_API_KEY";
/// The license token every artifact from this lane must carry (the 635
/// NC-lineage law, binding) — also the RIDT header's license string.
pub const LICENSE_LINE: &str = "CC BY-NC 4.0 (startlux-models/StartLux-Decision \
     v1.1 weights; engine Apache-2.0) — DECLARED-NC → RESTRICTED capture \
     posture per riir-train 635 T0; NC-lineage: startlux-decision-v1.1; \
     adoption fail-closed";

/// The lane: a thin plaintext-HTTP/1.1 client to their `startlux-decide`
/// server. std-only (the shared `crate::lanes::http_mini` micro-client; no
/// new deps). The wire functions are REUSED from the pplx lane — one
/// SystemOne wire, many oracles (the dry law: `build_body`/`map_answers`
/// are the pplx module's; this lane owns the endpoint + the POSTURE gate).
#[derive(Debug, Clone)]
pub struct StartluxLane {
    host: String,
    port: u16,
    timeout: Duration,
    token: Option<String>,
}

impl Default for StartluxLane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl StartluxLane {
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

    /// The handshake + POSTURE gate: `GET /health` must answer 200 with
    /// `status: "ok"` AND the bidirectional Instant serve posture —
    /// `kind: "instant"`, `prompt_attention: "bidirectional"`,
    /// `engine.bidir_mode: "server"`. A causal or per-request-bidir serve
    /// misreads the Instant checkpoint (a different model than the one
    /// trained — the −26 pt class), and an API-keys-configured server
    /// omits the posture fields entirely (the public health shape) — both
    /// refuse LOUD naming the boot line, never a half-run. Returns
    /// `(provenance, device)`: the served model name from `/health` (their
    /// `--name`, default the model folder's name — never a hardcoded id,
    /// the gliner law) and the [`DEVICE_ENV`] disclosure stamp.
    pub fn info(&self) -> Result<(String, String), String> {
        let (status, _headers, body) = self
            .get("/health")
            .map_err(|e| format!("startlux health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is their startlux-decide \
                 server up (boot line: llama-server on :8081 + STARTLUX_LLAMACPP_BIDIR=server \
                 startlux-decide serve --backend llamacpp --port 8090)?",
                String::from_utf8_lossy(&body)
            ));
        }
        let parsed: Value =
            serde_json::from_slice(&body).map_err(|e| format!("startlux health: {e}"))?;
        let state = parsed.get("status").and_then(Value::as_str).unwrap_or("");
        if state != "ok" {
            return Err(format!(
                "startlux health: status {state:?} — their server answers \
                 status \"ok\" when healthy (startlux_decide/server.py); got \
                 a body their engine does not spell"
            ));
        }
        // The posture gate (see [`Self::info`]). A keys-configured server
        // answers the PUBLIC shape without these fields — fail-closed.
        let kind = parsed.get("kind").and_then(Value::as_str).unwrap_or("");
        if kind != "instant" {
            return Err(format!(
                "startlux health: kind {kind:?} — this lane pins the Instant \
                 one-pass bidirectional posture (kind \"instant\"); an \
                 AutoThink folder is a different serve"
            ));
        }
        let attention = parsed
            .get("prompt_attention")
            .and_then(Value::as_str)
            .unwrap_or("");
        if attention != "bidirectional" {
            return Err(format!(
                "startlux health: prompt_attention {attention:?} — the Instant \
                 checkpoint reads its prompt bidirectionally; a causal/off \
                 serve is a different model than the one trained (the −26 pt \
                 class). Fix the boot: STARTLUX_LLAMACPP_BIDIR=server over a \
                 non-causal llama-server context"
            ));
        }
        let bidir = parsed
            .pointer("/engine/bidir_mode")
            .and_then(Value::as_str)
            .unwrap_or("");
        // Their describe() appends " (auto)" when the mode was auto-chosen;
        // accept the suffix, refuse everything else.
        let bidir_ok = bidir == "server" || bidir == "server (auto)";
        if !bidir_ok {
            return Err(format!(
                "startlux health: engine.bidir_mode {bidir:?} — the lane pins \
                 the server-side non-causal context (bidir_mode \"server\"); \
                 \"request\"/\"off\" postures are disclosed measurement arms, \
                 never this lane's default. Fix the boot: \
                 STARTLUX_LLAMACPP_BIDIR=server"
            ));
        }
        let model = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("startlux-decision")
            .to_string();
        let device = std::env::var(DEVICE_ENV).unwrap_or_else(|_| "cuda:0".to_string());
        Ok((model, device))
    }

    /// One decision round: build the body for `case` (the pplx wire shape —
    /// their server ignores the extra `model` field), POST it, map the
    /// answers back (the pplx parser, verbatim — their answer shapes are the
    /// pplx family's plus the ignored `none_probability`). Returns
    /// `(answer triples, client_ms, input_tokens)` — the client round-trip
    /// is the cross-lane latency measure.
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, u64), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("startlux round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "startlux round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("startlux response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let input_tokens = input_tokens_of(&parsed).unwrap_or(0);
        Ok((answers, client_ms, input_tokens))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// compare input. Nothing is stripped (the no-clock law: `timing_ms`
    /// rides their body only with `return_details`, which this lane never
    /// sets), so the det column measures compute determinism, never their
    /// clock.
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "startlux round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).into_owned())
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

    fn get(&self, path: &str) -> Result<http_mini::HttpReply, String> {
        http_mini::request(&self.host, self.port, "GET", path, None, self.timeout, &[])
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
impl StartluxLane {
    fn host_for_test(&self) -> &str {
        &self.host
    }
    fn port_for_test(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::QKind;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Read, Write};

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

    /// A one-shot stub server: serves `canned` for EVERY request, looping
    /// on accept (the loaded-box lesson), reading the headers AND the body
    /// before replying, and capturing each `(request line, request body)`
    /// on a channel (the d3 stub, verbatim — one shape, many lanes).
    fn serve(
        canned: String,
    ) -> (
        std::net::SocketAddr,
        std::sync::mpsc::Receiver<(String, Vec<u8>)>,
    ) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
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
            }
        });
        (addr, rx)
    }

    fn healthy(model: &str) -> String {
        json!({
            "status": "ok",
            "model": model,
            "kind": "instant",
            "images": false,
            "prompt_attention": "bidirectional",
            "engine": {"name": "llamacpp", "bidir_mode": "server",
                       "prompt_attention": "bidirectional (server)"},
            "defaults": {"think_mode": "never", "gate": 0.6, "think_budget": 4096, "mix": 0.75},
            "max_think_budget": 4096,
            "context_length": 8192,
        })
        .to_string()
    }

    #[test]
    fn url_parses_host_port_and_defaults() {
        let lane = StartluxLane::from_url("http://127.0.0.1:9111");
        let bare = StartluxLane::from_url("127.0.0.1");
        // Both constructors agree on the defaults (host/port are private —
        // the round-trip tests prove the endpoint; the env contract is
        // pinned by the constants).
        assert_eq!(lane.host_for_test(), "127.0.0.1");
        assert_eq!(lane.port_for_test(), 9111);
        assert_eq!(bare.port_for_test(), DEFAULT_PORT);
    }

    #[test]
    fn health_parses_provenance_and_enforces_the_bidirectional_posture() {
        // The healthy full shape: provenance = their --name (the folder
        // name by default).
        let (addr, _rx) = serve(healthy("startlux-decision-v1.1-27b-instant"));
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let (provenance, _device) = lane.info().unwrap();
        assert_eq!(provenance, "startlux-decision-v1.1-27b-instant");

        // describe()'s " (auto)" suffix is the same server posture.
        let auto = json!({
            "status": "ok", "model": "m", "kind": "instant",
            "prompt_attention": "bidirectional",
            "engine": {"bidir_mode": "server (auto)"},
        });
        let (addr, _rx) = serve(auto.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        assert_eq!(lane.info().unwrap().0, "m");
    }

    #[test]
    fn health_refuses_every_wrong_posture_loud() {
        // Causal/off bidir: the −26 pt class — refused.
        let causal = json!({
            "status": "ok", "model": "m", "kind": "instant",
            "prompt_attention": "bidirectional",
            "engine": {"bidir_mode": "request"},
        });
        let (addr, _rx) = serve(causal.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let err = lane.info().unwrap_err();
        assert!(err.contains("bidir_mode"), "{err}");

        // Non-bidirectional prompt attention: refused.
        let off = json!({
            "status": "ok", "model": "m", "kind": "instant",
            "prompt_attention": "causal (not the trained reading)",
            "engine": {"bidir_mode": "server"},
        });
        let (addr, _rx) = serve(off.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let err = lane.info().unwrap_err();
        assert!(err.contains("prompt_attention"), "{err}");

        // An AutoThink folder: refused (the lane pins Instant).
        let autothink = json!({
            "status": "ok", "model": "m", "kind": "autothink",
            "prompt_attention": "bidirectional",
            "engine": {"bidir_mode": "server"},
        });
        let (addr, _rx) = serve(autothink.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let err = lane.info().unwrap_err();
        assert!(err.contains("kind"), "{err}");

        // The API-keys public shape omits the posture fields entirely —
        // fail-closed, never a silent pass.
        let public = json!({"status": "ok", "model": "m", "kind": "instant"});
        let (addr, _rx) = serve(public.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let err = lane.info().unwrap_err();
        assert!(err.contains("prompt_attention"), "{err}");

        // A foreign status spelling: refused (their engine answers "ok").
        let weird = json!({"status": "ready", "model": "m"});
        let (addr, _rx) = serve(weird.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let err = lane.info().unwrap_err();
        assert!(err.contains("status"), "{err}");
    }

    #[test]
    fn stub_listener_round_trip() {
        // One decision round through the REAL micro-client against the
        // stub: the request reaches /v1/systemone as the pplx body shape
        // (their server ignores the extra `model` field), and the answers
        // map through the shared pplx parser — choice with
        // none_probability (ignored), noul without confidence (their wire
        // spelling), usage.input_tokens riding the provenance tail.
        let canned = json!({
            "model": "startlux-decision-v1.1-27b-instant",
            "answers": {
                "q_choice": {
                    "type": "choice",
                    "choice": "billing",
                    "confidence": 0.9948,
                    "probabilities": {"team": 0.003, "billing": 0.9948, "sales": 0.0022},
                    "none_probability": 0.0011,
                },
                "q_noul": {"type": "noul", "noul": 0.7449},
            },
            "usage": {"input_tokens": 191, "output_tokens": 0},
        });
        let (addr, rx) = serve(canned.to_string());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
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
        // Choice: probs in OUR criteria-key order, pick = their `choice`.
        let (probs, pick, conf) = &answers[0];
        assert_eq!(probs.len(), 3);
        assert_eq!(*pick, 1);
        assert!((conf - 0.9948).abs() < 1e-9);
        // Noul: the pair [p_no, p_yes] over the gold idx space; their
        // answer carries no confidence and the parser never asks.
        let (nprobs, npick, _nconf) = &answers[1];
        assert_eq!(nprobs.len(), 2);
        assert!((nprobs[1] - 0.7449).abs() < 1e-9);
        assert_eq!(*npick, 1);

        let (request_line, body) = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(
            request_line.starts_with("POST /v1/systemone"),
            "{request_line}"
        );
        let sent: Value = serde_json::from_slice(&body).unwrap();
        // The pplx body shape: {model, state, questions} — the extra
        // `model` field rides along (their server ignores it).
        assert_eq!(sent.as_object().map(|o| o.len()), Some(3));
        assert_eq!(sent["questions"]["q_choice"]["type"], "choice");
        assert_eq!(
            sent["questions"]["q_choice"]["criteria"]["billing"],
            "The billing team"
        );
        // Noul with null criteria: omitted from the body (their optional
        // criteria default applies server-side).
        assert!(sent["questions"]["q_noul"].get("criteria").is_none());
    }

    #[test]
    fn decide_raw_returns_the_body_verbatim() {
        // The no-clock law: what their server sent is what the det column
        // compares — including the none_probability field the pplx parser
        // ignores.
        let canned = json!({
            "model": "m",
            "answers": {"q0": {"type": "noul", "noul": 0.6}},
            "usage": {"input_tokens": 5, "output_tokens": 0},
        });
        let raw = canned.to_string();
        let (addr, _rx) = serve(raw.clone());
        let lane = StartluxLane::from_url(&format!("http://{addr}"));
        let c = case(json!("s"), vec![("q0", QKind::Noul, "q?", Value::Null)]);
        assert_eq!(lane.decide_raw(&c).unwrap(), raw);
    }
}
