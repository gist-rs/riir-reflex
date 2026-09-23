//! The localhost HTTP edge — the hexagonal seam, and ONLY the seam.
//!
//! One std-only binary: `TcpListener` + a hand-rolled HTTP/1.1 min-envelope
//! (request line, headers, Content-Length body). No daemon framework, no
//! runtime, no TLS — the engine serves LOCAL processes only (the default
//! bind is loopback; a non-loopback bind requires an explicit env override
//! and is the operator's own risk posture, not a feature). The engine's hot
//! path never enters this module: cold-path JSON in, JSON out.

use crate::embed::EMBED_DIM;
use crate::engine::DecisionEngine;
use katgpt_core::decision_wire::DecisionRequest;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Default bind: loopback, port 7331.
pub const DEFAULT_BIND: &str = "127.0.0.1:7331";
/// Body ceiling (a decision request is text; 4 MiB is generous).
const MAX_BODY: usize = 4 * 1024 * 1024;
/// Cold-path read timeout (a stuck client must not hold a thread forever).
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// The browser-facing CORS allow-list: `RIIR_REFLEX_ALLOWED_ORIGIN` (comma-
/// separated origins, e.g. `https://reflex.gist.rs`). Default EMPTY — no
/// `Access-Control-Allow-Origin` header is ever emitted, so no web page can
/// reach the engine cross-origin (the drive-by posture stays closed; the
/// arena playground prints the exact launch command that opens it).
pub fn allowed_origins() -> Vec<String> {
    std::env::var("RIIR_REFLEX_ALLOWED_ORIGIN")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// The ACAO echo value when the request's Origin is allow-listed — `None`
/// means emit NO CORS header at all (the default posture).
fn cors_echo(origin: Option<&str>, allow: &[String]) -> Option<String> {
    let o = origin?;
    allow.iter().find(|a| a.as_str() == o).cloned()
}

/// The bind address: `RIIR_REFLEX_BIND` ("host:port") overrides the
/// loopback default.
pub fn bind_addr() -> String {
    std::env::var("RIIR_REFLEX_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string())
}

/// The day-one engine posture: two demo domains over the fixture corpus
/// (gate semantics: `abstain_confidence = sigmoid(scale·(max_sim − mid))`).
/// The harness (Plan 603 T1.5) is the real corpus driver; this exists so
/// the binary is HONEST out of the box (it abstains off-corpus rather
/// than pretending competence on an empty model).
pub fn demo_engine() -> DecisionEngine<2, EMBED_DIM> {
    const OPS: &str = "Deploy the server to staging and verify the rollout before promoting \
to production. The staging cluster mirrors production capacity and runs the \
same release candidate. Rollback is one command when a deploy regresses the \
error budget. Verify the health endpoints after every rollout step.";
    const SUPPORT: &str = "The customer asked for a refund of the last invoice because the \
billing account was charged twice. Check the account balance and the payment \
history, then refund the duplicate charge to the original payment method. \
Escalate to the billing team when the invoice does not match the account \
records.";
    crate::engine::DecisionEngine::build_specs(
        vec![
            crate::engine::ExpertSpec::new("ops", &[OPS.to_string()]),
            crate::engine::ExpertSpec::new("support", &[SUPPORT.to_string()]),
        ],
        crate::engine::EngineConfig::default(),
    )
    .expect("demo corpus is well-formed")
}

/// Bind + serve (blocking). Errors are `io::Error`s from the bind/accept
/// path; per-connection errors are logged and never kill the loop.
pub fn run() -> std::io::Result<()> {
    let addr = bind_addr();
    let listener = TcpListener::bind(&addr)?;
    eprintln!(
        "[riir-reflex] listening on http://{addr} (modelless lane, {})",
        crate::VERSION
    );
    eprintln!("[riir-reflex] POST /decide  — DecisionRequest JSON → DecisionResponse JSON");
    eprintln!("[riir-reflex] POST /feedback — {{p, outcome}} → calibrator observe/refit");
    eprintln!("[riir-reflex] GET  /healthz — liveness");
    let allow = allowed_origins();
    if allow.is_empty() {
        eprintln!(
            "[riir-reflex] CORS: closed (no RIIR_REFLEX_ALLOWED_ORIGIN) — browser pages cannot reach this engine"
        );
    } else {
        eprintln!("[riir-reflex] CORS: allowed origins — {}", allow.join(", "));
    }
    let engine = Arc::new(Mutex::new(demo_engine()));
    serve_listener(listener, engine)
}

/// Serve on an ALREADY-BOUND listener with an EXPLICIT CORS allow-list (the
/// test seam — no process-global env mutation across test threads).
pub fn serve_listener_with<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    allow: Vec<String>,
) -> std::io::Result<()> {
    let allow: Arc<[String]> = allow.into();
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let eng = Arc::clone(&engine);
                let allow = Arc::clone(&allow);
                std::thread::spawn(move || {
                    if let Err(e) = handle_conn(s, eng, &allow) {
                        eprintln!("[riir-reflex] conn error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("[riir-reflex] accept error: {e}"),
        }
    }
    Ok(())
}

/// Serve on an ALREADY-BOUND listener (the production seam — the allow-list
/// comes from `allowed_origins()`).
pub fn serve_listener<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
) -> std::io::Result<()> {
    serve_listener_with(listener, engine, allowed_origins())
}

struct Req {
    method: String,
    path: String,
    content_length: usize,
    origin: Option<String>,
}

fn read_request(reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<Req>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None); // clean EOF between requests
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_ascii_uppercase();
    let path = parts.next().unwrap_or("/").to_string();
    let mut content_length = 0usize;
    let mut origin = None;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 {
            break;
        }
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((name, value)) = h.split_once(':') {
            let (name, value) = (name.trim(), value.trim());
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().unwrap_or(0);
            } else if name.eq_ignore_ascii_case("origin") {
                origin = Some(value.to_string());
            }
        }
    }
    Ok(Some(Req {
        method,
        path,
        content_length,
        origin,
    }))
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    body: &str,
    content_type: &str,
    cors: Option<&str>,
) {
    let bytes = body.as_bytes();
    let cors_hdr = cors
        .map(|o| format!("Access-Control-Allow-Origin: {o}\r\nVary: Origin\r\n"))
        .unwrap_or_default();
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n{cors_hdr}Connection: close\r\n\r\n",
        bytes.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(bytes);
    let _ = stream.flush();
}

fn json_response(stream: &mut TcpStream, status: &str, body: &str, cors: Option<&str>) {
    respond(stream, status, body, "application/json", cors);
}

fn handle_conn<const N: usize, const D: usize>(
    stream: TcpStream,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    allow: &[String],
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let Some(req) = read_request(&mut reader)? else {
        return Ok(());
    };
    let cors = cors_echo(req.origin.as_deref(), allow);
    match (req.method.as_str(), req.path.as_str()) {
        // Browser preflight: answered ONLY for allow-listed origins, and the
        // refusal carries no ACAO so the browser blocks it (a listed origin
        // is the arena playground; anything else is drive-by).
        ("OPTIONS", _) => match cors {
            Some(o) => {
                let head = format!(
                    "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: {o}\r\nVary: Origin\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nAccess-Control-Max-Age: 86400\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                );
                let _ = writer.write_all(head.as_bytes());
                let _ = writer.flush();
            }
            None => {
                let head = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                let _ = writer.write_all(head.as_bytes());
                let _ = writer.flush();
            }
        },
        ("GET", "/healthz") => {
            respond(&mut writer, "200 OK", "ok", "text/plain", cors.as_deref());
        }
        ("POST", "/decide") => {
            if req.content_length > MAX_BODY {
                json_response(
                    &mut writer,
                    "413 Payload Too Large",
                    "{\"error\":\"body too large\"}",
                    cors.as_deref(),
                );
                return Ok(());
            }
            let mut body = vec![0u8; req.content_length];
            reader.read_exact(&mut body)?;
            let parsed: Result<DecisionRequest, _> = serde_json::from_slice(&body);
            match parsed {
                Ok(req) => match engine.lock().unwrap().decide(&req) {
                    Ok(resp) => json_response(
                        &mut writer,
                        "200 OK",
                        &serde_json::to_string(&resp).unwrap_or_default(),
                        cors.as_deref(),
                    ),
                    Err(e) => json_response(
                        &mut writer,
                        "422 Unprocessable Entity",
                        &format!(
                            "{{\"error\":{}}}",
                            serde_json::to_string(&e.to_string()).unwrap_or_default()
                        ),
                        cors.as_deref(),
                    ),
                },
                Err(e) => json_response(
                    &mut writer,
                    "400 Bad Request",
                    &format!(
                        "{{\"error\":{}}}",
                        serde_json::to_string(&e.to_string()).unwrap_or_default()
                    ),
                    cors.as_deref(),
                ),
            }
        }
        ("POST", "/feedback") => {
            if req.content_length > MAX_BODY {
                json_response(
                    &mut writer,
                    "413 Payload Too Large",
                    "{\"error\":\"body too large\"}",
                    cors.as_deref(),
                );
                return Ok(());
            }
            let mut body = vec![0u8; req.content_length];
            reader.read_exact(&mut body)?;
            #[derive(serde::Deserialize)]
            struct Feedback {
                p: f32,
                outcome: bool,
            }
            match serde_json::from_slice::<Feedback>(&body) {
                Ok(f) => {
                    let moved = engine.lock().unwrap().observe(f.p, f.outcome);
                    json_response(
                        &mut writer,
                        "200 OK",
                        &format!("{{\"refit\":{moved}}}"),
                        cors.as_deref(),
                    );
                }
                Err(e) => json_response(
                    &mut writer,
                    "400 Bad Request",
                    &format!(
                        "{{\"error\":{}}}",
                        serde_json::to_string(&e.to_string()).unwrap_or_default()
                    ),
                    cors.as_deref(),
                ),
            }
        }
        _ => json_response(
            &mut writer,
            "404 Not Found",
            "{\"error\":\"not found\"}",
            cors.as_deref(),
        ),
    }
    Ok(())
}
