//! The vLLM-SR d3 comparison lane (riir-train Issue 629 T2, the B3 arm):
//! vLLM Semantic Router's `d3` (Decision 3.0) 27B — **board #1 blended /
//! #2 public-only (65.45) on the Jev Decision Index 0.3.2** — measured on
//! OUR frozen suites before any "use it" claim. The B5 law cuts both ways:
//! board rank is never our-suite rank. This lane produces the read that
//! decides (the 082 law: no teacher capture before the lane's read exists).
//!
//! **License — teacher-eligible:** Apache-2.0, ungated, open weights
//! (verified 2026-10-14 at tag `v3.1.0` — raw LICENSE fetch + the HF api
//! tag; NOTICE: "Built on Qwen/Qwen3.8-27B (Apache-2.0)", base pinned at
//! revision `1d4bf0f2ff60` — the SAME base pin pplx carries; riir-train
//! 629 T0 records the full provenance + the contamination read).
//!
//! **The model is a multimodal classifier, not a generator** (26.09B incl.
//! a 0.46B vision encoder; text + images + videos): Qwen3.8-27B backbone
//! finetune + a single-token readout head over 256 option codes
//! (`d3-code-readout-v1`, pinned in `decision_config.json`),
//! `attention_mode: noncausal_full_attention` and `pooling: last`, exactly
//! the pplx checkpoint's class. No tokens are generated; a forward pass
//! yields the masked option-score distribution directly. Consequence:
//! serving MUST go through THEIR `d3_server.py` (the package's own
//! FastAPI server — "adapted from perplexity-ai/pplx-decider-v1.1-27b",
//! the same wire family), the MEASURE-vs-SERVE split every comparison
//! lane holds: their stack serves, our Rust measures.
//!
//! The wire is the TypeSafe SystemOne dialect — IDENTICAL to the pplx
//! lane's ([`crate::lanes::pplx`]); shapes pinned from THEIR source at tag
//! `v3.1.0` (`d3_server.py` + `d3_runtime.py` + `d3_format.py`), not from
//! a captured fixture:
//!
//! * the request accepts `{model, state, questions, images, videos}`;
//!   this lane sends exactly `{model, state, questions}` — the `model`
//!   field value is NOT validated by their server (REQUEST_FIELDS only
//!   bounds the KEY set), so the pplx lane's stable alias `jev-latest`
//!   passes verbatim (their `d3_server.py` ignores the value; the field
//!   is kept for wire-family consistency);
//! * choice criteria: 1..=255 entries (`d3_format.py` `MAX_OPTIONS`,
//!   the same bound as pplx's server);
//! * score criteria: 2..=10 levels (`d3_runtime.py` `SCORE_LEVELS`) —
//!   same as pplx;
//! * answers: `probabilities` keyed by OUR option keys (choice) or level
//!   index `"0".."k-1"` (score), `noul` = p(true), `confidence`, the
//!   "Decision 2.0 response shapes" (`d3_runtime.py` `product_answer`)
//!   — the pplx lane's [`crate::lanes::pplx::map_answers`] parses this
//!   family verbatim (REUSED, not forked: one wire, many oracles);
//! * a request over the input limit refuses the WHOLE request with 422
//!   naming the maximum context length (nothing truncated); malformed
//!   requests also 422 — both are loud lane errors, never half-parsed.
//!
//! **The no-clock law** (inherited from the pplx lane): their response
//! body carries NO timing anywhere — `usage` is token counts
//! (`output_tokens` always 0) — so `decide_raw` returns the body
//! VERBATIM, nothing stripped; the det column measures compute
//! determinism, never their clock.
//!
//! **Posture** (disclosed, never pooled silently): a LOCAL serve on our
//! GPU box — their `d3_server.py --model <dir> --port 8794` — over
//! plaintext loopback HTTP. The 52 GB bf16 checkpoint exceeds a 24 GB
//! card, so the serve posture is an offload/quant-patched loader
//! (the pplx rig precedent); the quant posture is PINNED per capture
//! (riir-train 629 T2's law: which quant produced the teacher scores +
//! a per-family spot check against the board's published per-benchmark
//! numbers BEFORE trusting the captures — the lossy-surface rule);
//! `D3_DEVICE` carries the disclosure stamp into the provenance line,
//! and the run posture quotes it. Latency = client round-trip INCLUDING
//! the network hop to the serve box.
//!
//! **Port:** their server's default is 8000 — the drex lane's default.
//! OUR default is **8794** so the d3/clef/pplx/drex lanes can all serve
//! beside each other on one box (8791/8793/8794/8000); serve their side
//! with `--port 8794` (or point `D3_SERVE_URL`).
//!
//! **Provenance:** their `/health` carries NO `checkpoint` field (unlike
//! pplx's autojev server) — the model id comes from the SERVED package's
//! name, so serve version-true with `--name d3-v3.1.0` (the package's
//! bare name is just "d3"; the tag belongs in the provenance line).
//!
//! Env contract (never a committed credential):
//!
//! * `D3_SERVE_URL` — default `http://127.0.0.1:8794`;
//! * `D3_TIMEOUT_MS` — per-request timeout, default 300000 (generous:
//!   the offload/quant posture is slow per forward);
//! * `D3_DEVICE` — provenance stamp, default `cuda:0` (set it to the
//!   REAL posture, e.g. `cuda:0+nf4`, when serving patched);
//! * `D3_API_KEY` — optional bearer (their server's `DECISION_API_KEY`
//!   check; attached when set — a loopback serve usually runs without).

use std::time::{Duration, Instant};

use serde_json::Value;

use crate::harness::suites::SuiteCase;
use crate::lanes::http_mini;
use crate::lanes::pplx::{build_body, input_tokens_of, map_answers, AnswerTriple};

/// The env var naming the serve base URL.
pub const SERVE_URL_ENV: &str = "D3_SERVE_URL";
/// The default posture: loopback, port 8794 (their `--port` default 8000
/// is the drex lane's — this lane's default keeps all four decision-lane
/// servers servable beside each other; serve their side with
/// `--port 8794`).
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8794";
/// The default port (see [`DEFAULT_SERVE_URL`]).
pub const DEFAULT_PORT: u16 = 8794;
/// The env var for the per-request timeout, milliseconds.
pub const TIMEOUT_ENV: &str = "D3_TIMEOUT_MS";
/// Default timeout: 300 s — the offload/quant-patched posture is slow per
/// forward, and a contended GPU box slower still.
pub const DEFAULT_TIMEOUT_MS: u64 = 300_000;
/// The env var carrying the serve-posture provenance stamp.
pub const DEVICE_ENV: &str = "D3_DEVICE";
/// The env var for the optional bearer token (their `DECISION_API_KEY`).
pub const TOKEN_ENV: &str = "D3_API_KEY";

/// The lane: a thin plaintext-HTTP/1.1 client to their d3 server. std-only
/// (the shared `crate::lanes::http_mini` micro-client; no new deps). The
/// wire functions are REUSED from the pplx lane — one SystemOne wire,
/// many oracles (the DRY law: `build_body`/`map_answers` are the pplx
/// module's, this lane only owns the endpoint + the health shape).
#[derive(Debug, Clone)]
pub struct D3Lane {
    host: String,
    port: u16,
    timeout: Duration,
    token: Option<String>,
}

impl Default for D3Lane {
    fn default() -> Self {
        Self::from_url(&std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into()))
    }
}

impl D3Lane {
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
    /// answers `"loading"` and the refusal says so (their respond would
    /// 503 anyway — fail before the first case, not at case 0). Returns
    /// `(provenance, device)`: the served model id from `/health` (serve
    /// version-true with `--name d3-v3.1.0` — the bare package name is
    /// just "d3"; never a hardcoded model id, the gliner law), and the
    /// `D3_DEVICE` disclosure stamp.
    pub fn info(&self) -> Result<(String, String), String> {
        let (status, _headers, body) =
            self.get("/health").map_err(|e| format!("d3 health: {e}"))?;
        if status != 200 {
            return Err(format!(
                "GET /health -> HTTP {status}: {} — is their d3_server.py \
                 serving at {SERVE_URL_ENV} (default {DEFAULT_SERVE_URL}, serve \
                 their side with --port 8794)?",
                String::from_utf8_lossy(&body)
            ));
        }
        let parsed: Value =
            serde_json::from_slice(&body).map_err(|e| format!("d3 health: {e}"))?;
        let state = parsed.get("status").and_then(Value::as_str).unwrap_or("");
        if state != "ready" {
            return Err(format!(
                "d3 health: status {state:?} — the model is still loading \
                 (their lazy first-load window); retry once it completes"
            ));
        }
        let model = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or("d3")
            .to_string();
        let device = std::env::var(DEVICE_ENV).unwrap_or_else(|_| "cuda:0".to_string());
        Ok((model, device))
    }

    /// One decision round: build the body for `case`, POST it, map the
    /// answers back (the pplx wire functions, verbatim). Returns
    /// `(answer triples, client_ms, input_tokens)` — the client
    /// round-trip is the cross-lane latency measure.
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, u64), String> {
        let body = build_body(case)?.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("d3 round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "d3 round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("d3 response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let input_tokens = input_tokens_of(&parsed).unwrap_or(0);
        Ok((answers, client_ms, input_tokens))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// compare input. Nothing is stripped (the no-clock law, inherited
    /// from the pplx lane: `usage` is token counts; no server wall rides
    /// the body).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let body = build_body(case)?.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "d3 round trip ({}) -> HTTP {status}: {}",
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
impl D3Lane {
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

    /// A one-shot stub server: serves `canned` for EVERY request, looping
    /// on accept (the loaded-box lesson), reading the headers AND the body
    /// before replying (the macOS ECONNRESET flake the openthai stub
    /// documents), and capturing each `(request line, request body)` on a
    /// channel (the pplx stub, verbatim — one wire, many oracles).
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
    fn url_parses_host_port_and_defaults() {
        let lane = D3Lane::from_url("http://127.0.0.1:9110");
        let bare = D3Lane::from_url("127.0.0.1");
        // Both constructors agree on the defaults (host/port are private —
        // the round-trip tests below prove the endpoint; the env contract
        // is pinned by the constants).
        assert_eq!(lane.host_for_test(), "127.0.0.1");
        assert_eq!(lane.port_for_test(), 9110);
        assert_eq!(bare.port_for_test(), DEFAULT_PORT);
    }

    #[test]
    fn health_parses_provenance_and_refuses_while_loading() {
        let ready = json!({
            "status": "ready",
            "model": "d3-v3.1.0",
            "max_input_tokens": null,
            "modalities": ["text", "image", "video"],
            "authentication": false,
        });
        let (addr, _rx) = serve(ready.to_string());
        let lane = D3Lane::from_url(&format!("http://{addr}"));
        let (provenance, _device) = lane.info().unwrap();
        // The model id IS the provenance (their /health carries no
        // checkpoint field — serve version-true with --name).
        assert_eq!(provenance, "d3-v3.1.0");

        let loading = json!({"status": "loading", "model": null});
        let (addr, _rx) = serve(loading.to_string());
        let lane = D3Lane::from_url(&format!("http://{addr}"));
        let err = lane.info().unwrap_err();
        assert!(err.contains("loading"), "{err}");
    }

    #[test]
    fn stub_listener_round_trip() {
        // One decision round through the REAL micro-client against the
        // stub: the request reaches /v1/systemone as exactly
        // {model, state, questions} (their REQUEST_FIELDS bound the KEY
        // set; the model value is not validator-checked), and the answer
        // maps through the pplx wire parser (the shared map_answers).
        let canned = json!({
            "model": "d3-v3.1.0",
            "answers": {
                "q0": {
                    "type": "noul",
                    "noul": 0.75,
                    "probabilities": {"0": 0.25, "1": 0.75},
                    "confidence": 0.75,
                }
            },
            "usage": {"input_tokens": 12, "output_tokens": 0},
        });
        let (addr, rx) = serve(canned.to_string());
        let lane = D3Lane::from_url(&format!("http://{addr}"));
        let c = case(
            json!("The order arrived damaged."),
            vec![("q0", QKind::Noul, "damaged?", Value::Null)],
        );
        let (answers, _ms, tokens) = lane.decide(&c).unwrap();
        assert_eq!(tokens, 12);
        let (probs, pick, _conf) = &answers[0];
        // Noul: the pair [p_no, p_yes] over the gold idx space.
        assert_eq!(probs.len(), 2);
        assert!((probs[1] - 0.75).abs() < 1e-9);
        assert_eq!(*pick, 1);

        let (request_line, body) = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(request_line.starts_with("POST /v1/systemone"), "{request_line}");
        let sent: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(sent.as_object().map(|o| o.len()), Some(3));
        assert_eq!(sent["questions"]["q0"]["type"], "noul");
    }
}
