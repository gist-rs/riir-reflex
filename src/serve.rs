//! The localhost HTTP edge — the hexagonal seam, and ONLY the seam.
//!
//! One std-only binary: `TcpListener` + a hand-rolled HTTP/1.1 min-envelope
//! (request line, headers, Content-Length body). No daemon framework, no
//! runtime, no TLS — the engine serves LOCAL processes only (the default
//! bind is loopback; a non-loopback bind requires an explicit env override
//! and is the operator's own risk posture, not a feature). The engine's hot
//! path never enters this module: cold-path JSON in, JSON out.

use crate::{embed::EMBED_DIM, engine::DecisionEngine, game_heads::GameHeads};
use katgpt_core::decision_wire::{DecisionRequest, DecisionResponse};
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

// ── the laya comparison lane over HTTP (opt-in) ──────────────────────────

/// A served laya-lane decision — the loaded agent behind a closure, so the
/// edge's state machine is testable without weights (the closure exists only
/// under the `laya-riir` feature).
pub type LayaDecideFn = Arc<dyn Fn(&DecisionRequest) -> Result<DecisionResponse, String> + Send + Sync>;

/// The laya lane's serving state. `Off` is the default posture: the lane is
/// opt-in (`RIIR_REFLEX_LAYA=1` starts the weights download + load in a
/// background thread at boot), and the edge answers `X-Reflex-Lane: laya`
/// requests FAIL-CLOSED in every non-ready state — never a silent modelless
/// fallback, because a silently-served wrong lane would poison the arena's
/// per-lane claims (every published table scopes to the lane that produced
/// it).
#[derive(Clone, Default)]
pub enum LayaLane {
    #[default]
    Off,
    /// Weights downloading / model loading (first boot: ~650 MB).
    Loading,
    Ready(LayaDecideFn),
    Failed(String),
}

impl LayaLane {
    /// The `/healthz` spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Loading => "loading",
            Self::Ready(_) => "ready",
            Self::Failed(_) => "failed",
        }
    }
}

/// `RIIR_REFLEX_LAYA` truthiness: `1` / `true` / `yes` (case-insensitive).
fn laya_requested() -> bool {
    std::env::var("RIIR_REFLEX_LAYA")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Start the background laya loader when `RIIR_REFLEX_LAYA` is set. Loud in
/// every posture: loading/ready/failed here, feature-missing logged when the
/// binary was built without `laya-riir`.
fn spawn_laya_loader_if_requested(laya: &Arc<Mutex<LayaLane>>) {
    if !laya_requested() {
        return;
    }
    #[cfg(feature = "laya-riir")]
    {
        *laya.lock().unwrap() = LayaLane::Loading;
        let laya = Arc::clone(laya);
        std::thread::spawn(move || {
            use crate::laya::config::Checkpoint;
            use crate::laya::riir::RiirAgent;
            use crate::laya::weights::weights_root;
            let root = weights_root();
            eprintln!(
                "[riir-reflex] laya lane: loading english checkpoint from {} (first boot downloads the open weights)",
                root.display()
            );
            let t0 = std::time::Instant::now();
            // `RiirAgent` is !Send (the Metal/objc backend), so the agent is
            // loaded AND served on this ONE thread; clients get a Send + Sync
            // channel handle. Forwards are serialized (one model, one forward
            // at a time) — also the right posture when a game fires ~34 spot
            // reads concurrently.
            let agent = match RiirAgent::load(&root, Checkpoint::English) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("[riir-reflex] laya lane: FAILED to load: {e}");
                    *laya.lock().unwrap() = LayaLane::Failed(e.to_string());
                    return;
                }
            };
            let (tx, rx) = std::sync::mpsc::channel::<laya_serve::LayaJob>();
            let f: LayaDecideFn = Arc::new(move |req| {
                let (rtx, rrx) = std::sync::mpsc::sync_channel(1);
                tx.send(laya_serve::LayaJob {
                    req: req.clone(),
                    resp_tx: rtx,
                })
                .map_err(|_| "laya lane worker stopped".to_string())?;
                rrx.recv()
                    .map_err(|_| "laya lane worker dropped the reply".to_string())?
            });
            *laya.lock().unwrap() = LayaLane::Ready(f);
            eprintln!(
                "[riir-reflex] laya lane: ready (english, device {}, {} s)",
                agent.device(),
                t0.elapsed().as_secs()
            );
            while let Ok(job) = rx.recv() {
                let out = laya_serve::decide(&agent, &job.req);
                let _ = job.resp_tx.send(out);
            }
            eprintln!("[riir-reflex] laya lane: worker stopped (all clients gone)");
        });
    }
    #[cfg(not(feature = "laya-riir"))]
    {
        let _ = laya;
        eprintln!(
            "[riir-reflex] RIIR_REFLEX_LAYA=1 but this build has no laya lane — rebuild with --features laya-riir"
        );
    }
}

/// The Ready closure's body lives in [`laya_serve`] (feature-gated).
#[cfg(feature = "laya-riir")]
pub mod laya_serve {
    use crate::laya::riir::RiirAgent;
    use crate::laya::types::Answer as LayaAnswer;
    use katgpt_core::decision_wire::{
        Answer, Calibration, DecisionRequest, DecisionResponse, Lane, Question, QuestionKind,
        Routing,
    };
    use serde_json::Value;

    /// One queued decision: the request plus the one-shot reply channel (the
    /// worker owns the !Send agent; clients stay Send + Sync).
    pub struct LayaJob {
        pub req: DecisionRequest,
        pub resp_tx: std::sync::mpsc::SyncSender<Result<DecisionResponse, String>>,
    }

    /// wire → laya qdef `criteria`: an explicit wire `criteria` JSON string
    /// wins (object/array form); otherwise the wire `options` list becomes
    /// the criteria array — `to_internal` turns a list into the bare-key
    /// ordered map, so `option_keys` and the answer probabilities arrive in
    /// wire-options order. `noul` carries no criteria unless explicitly
    /// given (laya's `{"true": …, "false": …}` form).
    fn criteria(q: &Question) -> Option<Value> {
        if let Some(c) = &q.criteria
            && let Ok(v) = serde_json::from_str::<Value>(c)
            && (v.is_object() || v.is_array())
        {
            return Some(v);
        }
        if q.kind != QuestionKind::Noul && !q.options.is_empty() {
            return Some(Value::Array(
                q.options.iter().map(|o| Value::String(o.clone())).collect(),
            ));
        }
        None
    }

    /// First-argmax (strictly-greater keeps the earliest index — np.argmax
    /// semantics; `Iterator::max_by` would keep the LAST tie).
    fn argmax_first(ps: &[f64]) -> usize {
        let mut best = 0usize;
        for (i, v) in ps.iter().enumerate() {
            if *v > ps[best] {
                best = i;
            }
        }
        best
    }

    /// Map laya answers (same order as `req.questions`) onto the wire.
    /// Laya cannot abstain — every answer carries an outcome.
    pub fn map_answers(
        req: &DecisionRequest,
        answers: &[LayaAnswer],
    ) -> Result<DecisionResponse, String> {
        if answers.len() != req.questions.len() {
            return Err(format!(
                "laya returned {} answers for {} questions",
                answers.len(),
                req.questions.len()
            ));
        }
        let mut out = Vec::with_capacity(answers.len());
        for (q, a) in req.questions.iter().zip(answers.iter()) {
            let answer = match q.kind {
                QuestionKind::Choice => {
                    let probs: Vec<f64> = a.probabilities.iter().map(|(_, v)| *v).collect();
                    let idx = argmax_first(&probs);
                    Answer::choice(
                        q.id.clone(),
                        idx as u32,
                        probs.iter().map(|v| *v as f32).collect(),
                        a.confidence as f32,
                    )
                }
                QuestionKind::Score => {
                    let probs: Vec<f64> = a.probabilities.iter().map(|(_, v)| *v).collect();
                    let level = argmax_first(&probs);
                    Answer::score(
                        q.id.clone(),
                        level as u32,
                        probs.iter().map(|v| *v as f32).collect(),
                        a.confidence as f32,
                    )
                }
                QuestionKind::Noul => {
                    let p = a.noul.unwrap_or(0.5) as f32;
                    Answer::noul(q.id.clone(), p >= 0.5, p, a.confidence as f32)
                }
            };
            out.push(answer);
        }
        // The english checkpoint ships temperature-calibrated; the applied
        // per-(kind, option-count) temperature rides every answer — report
        // the first one (all questions in a request share the shape only by
        // convention, so this is a disclosure, not a per-answer claim).
        let temperature = answers
            .first()
            .map(|a| a.temperature_used as f32)
            .unwrap_or(1.0);
        Ok(DecisionResponse {
            answers: out,
            routing: Routing {
                lane: Lane::Laya,
                reason: Some("requested lane=laya".to_string()),
            },
            calibration: Calibration {
                method: "laya-temperature".to_string(),
                temperature,
            },
        })
    }

    /// Serve one laya-lane decision (the Ready closure's body): wire → laya
    /// qdefs, one `system_one` call, laya answers → wire.
    pub fn decide(agent: &RiirAgent, req: &DecisionRequest) -> Result<DecisionResponse, String> {
        req.validate()
            .map_err(|e| format!("invalid request: {e}"))?;
        let state = Value::String(req.state.clone());
        let mut questions: Vec<(String, Value)> = Vec::with_capacity(req.questions.len());
        for q in &req.questions {
            let mut def = serde_json::Map::new();
            def.insert("type".into(), Value::String(q.kind.as_str().into()));
            def.insert("instructions".into(), Value::String(q.prompt.clone()));
            if let Some(c) = criteria(q) {
                def.insert("criteria".into(), c);
            }
            questions.push((q.id.clone(), Value::Object(def)));
        }
        let answers = agent
            .system_one(&state, &questions)
            .map_err(|e| e.to_string())?;
        map_answers(req, &answers)
    }
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
    let laya = Arc::new(Mutex::new(LayaLane::Off));
    spawn_laya_loader_if_requested(&laya);
    eprintln!(
        "[riir-reflex] laya lane: {} (set RIIR_REFLEX_LAYA=1 to enable the comparison lane)",
        laya.lock().unwrap().as_str()
    );
    // The fitted game head (Plan 607's decoded Tetris head — boot-fitted
    // from the digest-pinned oracle fixture). Failure here is a broken
    // build, never a runtime condition: the fixture is compile-time and
    // the tests pin the fit's determinism + agreement anchors.
    let heads = Arc::new(GameHeads::build());
    eprintln!(
        "[riir-reflex] game head: tetris fitted ({} corpus options, λ {}, digest {})",
        heads.n_options(),
        heads.lambda(),
        heads.digest_hex()
    );
    serve_listener_lanes(listener, engine, laya, heads)
}

/// Serve on an ALREADY-BOUND listener with an EXPLICIT CORS allow-list (the
/// test seam — no process-global env mutation across test threads).
pub fn serve_listener_with<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    allow: Vec<String>,
) -> std::io::Result<()> {
    serve_listener_heads(listener, engine, laya, allow, Arc::new(GameHeads::build()))
}

/// Serve on an ALREADY-BOUND listener with an EXPLICIT game-head lane (the
/// `run()` seam — the boot-fit owns the head; the edge reads it).
pub fn serve_listener_heads<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    allow: Vec<String>,
    heads: Arc<GameHeads>,
) -> std::io::Result<()> {
    let allow: Arc<[String]> = allow.into();
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let eng = Arc::clone(&engine);
                let laya = Arc::clone(&laya);
                let allow = Arc::clone(&allow);
                let heads = Arc::clone(&heads);
                std::thread::spawn(move || {
                    if let Err(e) = handle_conn(s, eng, laya, &allow, &heads) {
                        eprintln!("[riir-reflex] conn error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("[riir-reflex] accept error: {e}"),
        }
    }
    Ok(())
}

/// Serve on an ALREADY-BOUND listener with an EXPLICIT laya-lane state (the
/// `run()` seam — the background loader owns the state; the edge reads it).
pub fn serve_listener_lanes<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    heads: Arc<GameHeads>,
) -> std::io::Result<()> {
    serve_listener_heads(listener, engine, laya, allowed_origins(), heads)
}

/// Serve on an ALREADY-BOUND listener (the production seam — the allow-list
/// comes from `allowed_origins()`, the laya lane stays off).
pub fn serve_listener<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
) -> std::io::Result<()> {
    serve_listener_heads(
        listener,
        engine,
        Arc::new(Mutex::new(LayaLane::Off)),
        allowed_origins(),
        Arc::new(GameHeads::build()),
    )
}

struct Req {
    method: String,
    path: String,
    content_length: usize,
    origin: Option<String>,
    /// `X-Reflex-Lane` — the lane hint for `/decide` (`laya` | `modelless`).
    lane: Option<String>,
    /// `Access-Control-Request-Private-Network: true` — Chromium's PNA
    /// preflight for a public page reaching a local address.
    pna_requested: bool,
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
    let mut lane = None;
    let mut pna_requested = false;
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
            } else if name.eq_ignore_ascii_case("x-reflex-lane") {
                lane = Some(value.to_ascii_lowercase());
            } else if name.eq_ignore_ascii_case("access-control-request-private-network")
                && value.eq_ignore_ascii_case("true")
            {
                pna_requested = true;
            }
        }
    }
Ok(Some(Req {
    method,
    path,
    content_length,
    origin,
    lane,
    pna_requested,
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
    laya: Arc<Mutex<LayaLane>>,
    allow: &[String],
    heads: &GameHeads,
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
                // Private Network Access: a public (https) page fetching a
                // local address makes Chromium send the PNA preflight header;
                // the engine must consent or the browser blocks the request
                // (measured live: prod page → loopback engine denied without
                // it). Consented ONLY alongside an allow-listed origin — the
                // drive-by posture stays closed.
                let pna = if req.pna_requested {
                    "Access-Control-Allow-Private-Network: true\r\n"
                } else {
                    ""
                };
                let head = format!(
                    "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: {o}\r\nVary: Origin\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, X-Reflex-Lane\r\n{pna}Access-Control-Max-Age: 86400\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
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
            let laya_state = laya.lock().unwrap().as_str();
            json_response(
                &mut writer,
                "200 OK",
                &format!(
                    "{{\"status\":\"ok\",\"lanes\":{{\"modelless\":\"ready\",\"laya\":\"{laya_state}\"}}}}"
                ),
                cors.as_deref(),
            );
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
            if req.lane.as_deref() == Some("laya") {
                match parsed {
                    Ok(req) => laya_edge(&mut writer, &laya, &req, cors.as_deref()),
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
                return Ok(());
            }
            if let Some(other) = &req.lane
                && other != "modelless"
            {
                json_response(
                    &mut writer,
                    "400 Bad Request",
                    &format!(
                        "{{\"error\":{}}}",
                        serde_json::to_string(&format!(
                            "unknown lane {other:?} (supported: modelless, laya)"
                        ))
                        .unwrap_or_default()
                    ),
                    cors.as_deref(),
                );
                return Ok(());
            }
            match parsed {
                Ok(req) => {
                    // The fitted game head first (Plan 607's decoded
                    // Tetris head): a well-formed spot question is
                    // answered from the boot-fitted head. Everything
                    // else — grammar-invalid state, foreign question —
                    // falls through to the cosine engine, which abstains
                    // off-corpus as before.
                    if let Some(resp) = heads.respond(&req) {
                        json_response(
                            &mut writer,
                            "200 OK",
                            &serde_json::to_string(&resp).unwrap_or_default(),
                            cors.as_deref(),
                        );
                        return Ok(());
                    }
                    match engine.lock().unwrap().decide(&req) {
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
                    }
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

/// The `X-Reflex-Lane: laya` edge: fail-closed in every non-ready state (a
/// silent modelless fallback would serve the WRONG lane under the arena's
/// per-lane claims), and the loaded agent's decision when ready.
fn laya_edge(
    writer: &mut TcpStream,
    laya: &Arc<Mutex<LayaLane>>,
    req: &DecisionRequest,
    cors: Option<&str>,
) {
    let state = laya.lock().unwrap().clone();
    match state {
        LayaLane::Ready(f) => match f(req) {
            Ok(resp) => json_response(
                writer,
                "200 OK",
                &serde_json::to_string(&resp).unwrap_or_default(),
                cors,
            ),
            Err(e) => json_response(
                writer,
                "502 Bad Gateway",
                &format!(
                    "{{\"error\":{}}}",
                    serde_json::to_string(&format!("laya lane: {e}")).unwrap_or_default()
                ),
                cors,
            ),
        },
        LayaLane::Loading => json_response(
            writer,
            "503 Service Unavailable",
            "{\"error\":\"laya lane is still loading (weights download + load on first boot) — retry shortly\"}",
            cors,
        ),
        LayaLane::Off => json_response(
            writer,
            "503 Service Unavailable",
            "{\"error\":\"laya lane is not enabled — restart the engine with RIIR_REFLEX_LAYA=1\"}",
            cors,
        ),
        LayaLane::Failed(e) => json_response(
            writer,
            "500 Internal Server Error",
            &format!(
                "{{\"error\":{}}}",
                serde_json::to_string(&format!("laya lane failed to load: {e}")).unwrap_or_default()
            ),
            cors,
        ),
    }
}
