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
use serde::Deserialize;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The serve-edge request envelope: the shared `DecisionRequest` wire plus
/// the OPTIONAL `sidecar` field (riir-instinct Issue 009 T5's widened-wire
/// design — the raw afterstate a hybrid lane may carry beside the unchanged
/// 5-class text). THIS lane never consumes it: the tetris hybrid lane owns
/// the sidecar's schema and its consumption; reflex parses-and-ignores so a
/// sender can attach the sidecar to any `/decide` request while the
/// modelless answers, fixtures, and pins stay byte-identical. The field is
/// deliberately `Value`-permissive (no shape validation here): the consumer
/// validates its own contract, and a shape this lane does not know must not
/// break an answer it does not affect.
#[derive(Debug, Deserialize)]
struct ServeRequest {
    #[serde(flatten)]
    request: DecisionRequest,
    /// Parsed-and-ignored ON PURPOSE — this lane never consumes it (the
    /// consuming lane owns both); serde's read during deserialize is the
    /// only "use" the field has here.
    #[allow(dead_code)]
    #[serde(default)]
    sidecar: Option<serde_json::Value>,
}

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

/// The route a `/decide` request takes through the laya lane — the wire's
/// `X-Reflex-Lane` spelling resolved. `laya` is the auto-router: the ANE
/// device when loaded and the request fits its buckets, the default device
/// otherwise — every choice and every hop recorded in `routing.reason`.
/// `laya-ane` is EXPLICIT-device: ANE only, out-of-bucket is a loud 422
/// (never a pad up, never a silent hop — the laya-apple doctrine).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayaRoute {
    Auto,
    AneExplicit,
}

/// The typed serve-edge error — the edge maps each variant to its own
/// status: `Bucket` → 422 (a coverage LIMIT, the substrate's own variant
/// surfaced — never silently absorbed), `Unavailable` → 503, `Other` →
/// 502 (the request failed on the device that owned it).
#[derive(Debug)]
pub enum LayaServeError {
    Bucket {
        checkpoint: &'static str,
        seq: usize,
        max: usize,
    },
    Unavailable(String),
    Other(String),
}

/// A served laya-lane decision — the loaded agent behind a closure, so the
/// edge's state machine is testable without weights (the closure exists only
/// under the `laya-riir` feature; the ROUTER it runs is
/// [`laya_serve::route_job`]).
pub type LayaDecideFn = Arc<
    dyn Fn(&DecisionRequest, LayaRoute) -> Result<DecisionResponse, LayaServeError> + Send + Sync,
>;

/// The laya lane's serving state. `Off` is the default posture: the lane is
/// opt-in (`RIIR_REFLEX_LAYA=1`, or `RIIR_REFLEX_LAYA_ANE=1` for the lane
/// WITH ANE routing — starts the weights download + load in a background
/// thread at boot), and the edge answers `X-Reflex-Lane: laya`/`laya-ane`
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

/// `RIIR_REFLEX_LAYA_ANE` truthiness — implies the laya lane (one env
/// turns the whole lane on with ANE routing; fetch-on-first-use + the
/// load-time compute-plan gate run inside the loader). The ANE lane is an
/// OPT-IN routing tier: without the env the lane serves the default
/// device byte-identically to before.
fn ane_lane_requested() -> bool {
    std::env::var("RIIR_REFLEX_LAYA_ANE")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Boot the ANE routing half: fetch-on-first-use (`ensure_artifacts` —
/// present entries untouched, downloads digest-gated) then the explicit
/// `load_ane` (constructor-selected; its load-time compute-plan gate
/// re-verifies the whole-graph placement). ANY failure is a LOUD demotion
/// to the default device with the reason recorded on every response —
/// never a boot failure, never a silent swap (the laya-apple doctrine:
/// "a corrupt/failed ANE artifact is dropped with a warning and its
/// requests go to the other device WITH the reason recorded").
#[cfg(all(target_os = "macos", feature = "laya-riir", feature = "laya-riir-ane"))]
fn ane_setup() -> (Option<crate::laya::riir::RiirAgent>, Option<String>) {
    use crate::laya::config::Checkpoint;
    use crate::laya::riir::ane::ensure_artifacts;
    use crate::laya::riir::RiirAgent;
    use crate::laya::weights::weights_root;
    if !ane_lane_requested() {
        return (None, None);
    }
    let ane_root = std::env::var_os("LAYA_ANE_ARTIFACTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("assets/ane"));
    let boot = || -> Result<crate::laya::riir::RiirAgent, String> {
        let manifest = ane_root.join("manifest.json");
        if !manifest.exists() {
            return Err(format!(
                "no ane manifest at {} — run scripts/ane_convert.py (offline, \
                 one-time) or set LAYA_ANE_ARTIFACTS_DIR",
                manifest.display()
            ));
        }
        let base = std::env::var("RIIR_REFLEX_ANE_BASE_URL").ok();
        ensure_artifacts(&ane_root, base.as_deref()).map_err(|e| e.to_string())?;
        RiirAgent::load_ane(&weights_root(), Checkpoint::English, &ane_root, &manifest)
            .map_err(|e| e.to_string())
    };
    match boot() {
        Ok(agent) => {
            let max = agent
                .ane_bucket_max()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "?".into());
            eprintln!("[riir-reflex] laya lane: ane routing armed (buckets ≤ L{max})");
            (Some(agent), None)
        }
        Err(e) => {
            let note = format!("ane unavailable: {e} — serving the default device");
            eprintln!("[riir-reflex] laya lane: {note}");
            (None, Some(note))
        }
    }
}

/// The no-ANE build's posture: the env is honored with a LOUD demotion
/// note (recorded on every auto-route response) — the lane still serves.
#[cfg(all(feature = "laya-riir", not(all(
    target_os = "macos",
    feature = "laya-riir-ane"
))))]
fn ane_setup() -> (Option<crate::laya::riir::RiirAgent>, Option<String>) {
    if !ane_lane_requested() {
        return (None, None);
    }
    let note = "RIIR_REFLEX_LAYA_ANE=1 but this build has no ANE lane \
                (needs macOS + --features laya-riir-ane) — serving the default device"
        .to_string();
    eprintln!("[riir-reflex] laya lane: {note}");
    (None, Some(note))
}

/// Start the background laya loader when `RIIR_REFLEX_LAYA` (or the ANE
/// spelling, which implies the lane) is set. Loud in every posture:
/// loading/ready/failed here, feature-missing logged when the binary was
/// built without `laya-riir`.
fn spawn_laya_loader_if_requested(laya: &Arc<Mutex<LayaLane>>) {
    if !(laya_requested() || ane_lane_requested()) {
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
            // `RiirAgent` is !Send (the Metal/objc backend), so the agents are
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
            let (ane, ane_note) = ane_setup();
            let (tx, rx) = std::sync::mpsc::channel::<laya_serve::LayaJob>();
            let f: LayaDecideFn = Arc::new(move |req, route| {
                let (rtx, rrx) = std::sync::mpsc::sync_channel(1);
                tx.send(laya_serve::LayaJob {
                    req: req.clone(),
                    route,
                    resp_tx: rtx,
                })
                .map_err(|_| LayaServeError::Other("laya lane worker stopped".into()))?;
                rrx.recv()
                    .map_err(|_| LayaServeError::Other("laya lane worker dropped the reply".into()))?
            });
            *laya.lock().unwrap() = LayaLane::Ready(f);
            match (&ane, ane_note.as_deref()) {
                (Some(_), _) => eprintln!(
                    "[riir-reflex] laya lane: ready (english, ane + default {} routed, {} s)",
                    agent.device(),
                    t0.elapsed().as_secs()
                ),
                (None, Some(note)) => eprintln!(
                    "[riir-reflex] laya lane: ready (english, device {}, {} s) — {note}",
                    agent.device(),
                    t0.elapsed().as_secs()
                ),
                (None, None) => eprintln!(
                    "[riir-reflex] laya lane: ready (english, device {}, {} s)",
                    agent.device(),
                    t0.elapsed().as_secs()
                ),
            }
            while let Ok(job) = rx.recv() {
                let out = laya_serve::route_job(
                    &agent,
                    ane.as_ref(),
                    ane_note.as_deref(),
                    job.req,
                    job.route,
                );
                let _ = job.resp_tx.send(out);
            }
            eprintln!("[riir-reflex] laya lane: worker stopped (all clients gone)");
        });
    }
    #[cfg(not(feature = "laya-riir"))]
    {
        let _ = laya;
        eprintln!(
            "[riir-reflex] RIIR_REFLEX_LAYA=1 (or RIIR_REFLEX_LAYA_ANE=1) but this build has \
             no laya lane — rebuild with --features laya-riir"
        );
    }
}

/// The Ready closure's body lives in [`laya_serve`] (feature-gated).
#[cfg(feature = "laya-riir")]
pub mod laya_serve {
    use super::{LayaRoute, LayaServeError};
    use crate::laya::riir::RiirAgent;
    use crate::laya::types::Answer as LayaAnswer;
    use crate::laya::LayaError;
    use katgpt_core::decision_wire::{
        Answer, Calibration, DecisionRequest, DecisionResponse, Lane, Question, QuestionKind,
        Routing,
    };
    use serde_json::Value;

    /// One queued decision: the request, the resolved route, and the
    /// one-shot reply channel (the worker owns the !Send agents; clients
    /// stay Send + Sync).
    pub struct LayaJob {
        pub req: DecisionRequest,
        pub route: LayaRoute,
        pub resp_tx:
            std::sync::mpsc::SyncSender<Result<DecisionResponse, LayaServeError>>,
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

    /// Serve one laya-lane decision (the router's forward): wire → laya
    /// qdefs, one `system_one` call, laya answers → wire. The error stays
    /// TYPED ([`LayaError`]) so the coverage-limit variant survives to the
    /// router — a stringified Bucket could not be told apart from a real
    /// failure, and the two demand opposite handling.
    pub fn decide(
        agent: &RiirAgent,
        req: &DecisionRequest,
    ) -> Result<DecisionResponse, LayaError> {
        req.validate()
            .map_err(|e| LayaError::Question(format!("invalid request: {e}")))?;
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
        let answers = agent.system_one(&state, &questions)?;
        map_answers(req, &answers).map_err(LayaError::Question)
    }

    fn map_edge_err(e: LayaError) -> LayaServeError {
        match e {
            LayaError::Bucket {
                checkpoint,
                seq,
                max,
            } => LayaServeError::Bucket {
                checkpoint,
                seq,
                max,
            },
            other => LayaServeError::Other(other.to_string()),
        }
    }

    /// The no-silent-fallback router (the laya-apple doctrine, serve-edge
    /// form): the route is decided BEFORE the request runs, every device
    /// choice and every hop lands in `routing.reason`, and a non-Bucket
    /// failure on a device fails the request ON that device — never a
    /// mid-request swap. `Auto`: the ANE device when loaded and the request
    /// fits its buckets; the Bucket coverage LIMIT is the designed loud hop
    /// to the default device. `AneExplicit`: ANE only — out-of-bucket is a
    /// loud [`LayaServeError::Bucket`] (422), never a pad up.
    pub fn route_job(
        local: &RiirAgent,
        ane: Option<&RiirAgent>,
        ane_note: Option<&str>,
        req: DecisionRequest,
        route: LayaRoute,
    ) -> Result<DecisionResponse, LayaServeError> {
        match route {
            LayaRoute::AneExplicit => {
                let Some(ane) = ane else {
                    return Err(LayaServeError::Unavailable(
                        ane_note.map_or_else(
                            || "the ANE device is not loaded in this lane".to_string(),
                            str::to_string,
                        ),
                    ));
                };
                decide(ane, &req)
                    .map(|mut resp| {
                        resp.routing.reason = Some("requested lane=laya-ane (device ane)".into());
                        resp
                    })
                    .map_err(map_edge_err)
            }
            LayaRoute::Auto => {
                let mut hop: Option<String> = None;
                if let Some(ane) = ane {
                    match decide(ane, &req) {
                        Ok(mut resp) => {
                            resp.routing.reason = Some("requested lane=laya (device ane)".into());
                            return Ok(resp);
                        }
                        Err(e @ LayaError::Bucket { .. }) => {
                            hop = Some(format!(
                                "seq over every ane bucket ({e}) — served on {}",
                                local.device()
                            ));
                        }
                        Err(e) => return Err(LayaServeError::Other(e.to_string())),
                    }
                }
                let mut resp =
                    decide(local, &req).map_err(|e| LayaServeError::Other(e.to_string()))?;
                let mut reason = format!("requested lane=laya (device {})", local.device());
                if let Some(note) = ane_note {
                    reason.push_str(" — ");
                    reason.push_str(note);
                }
                if let Some(hop) = hop {
                    reason.push_str(" — ");
                    reason.push_str(&hop);
                }
                resp.routing.reason = Some(reason);
                Ok(resp)
            }
        }
    }
}

/// What corpus the serve posture booted on — `/healthz` discloses it
/// (issue 063 T2): `"demo"` for the day-one posture, or the domain list
/// for a `RIIR_REFLEX_CORPUS` boot.
#[derive(Clone, Debug, PartialEq)]
pub enum CorpusInfo {
    Demo,
    Domains(Vec<String>),
}

impl CorpusInfo {
    /// The `/healthz` JSON fragment: `"demo"` or `{"domains":[...]}`.
    pub fn as_json(&self) -> String {
        match self {
            Self::Demo => "\"demo\"".to_string(),
            Self::Domains(d) => format!(
                "{{\"domains\":{}}}",
                serde_json::to_string(d).unwrap_or_else(|_| "[]".to_string())
            ),
        }
    }
}

/// The day-one corpus, verbatim — shared by [`demo_engine`] and the boot
/// dispatch's demo arm (issue 063: one spec source, two consumers).
fn demo_specs() -> Vec<crate::engine::ExpertSpec> {
    const OPS: &str = "Deploy the server to staging and verify the rollout before promoting \
to production. The staging cluster mirrors production capacity and runs the \
same release candidate. Rollback is one command when a deploy regresses the \
error budget. Verify the health endpoints after every rollout step.";
    const SUPPORT: &str = "The customer asked for a refund of the last invoice because the \
billing account was charged twice. Check the account balance and the payment \
history, then refund the duplicate charge to the original payment method. \
Escalate to the billing team when the invoice does not match the account \
records.";
    vec![
        crate::engine::ExpertSpec::new("ops", &[OPS.to_string()]),
        crate::engine::ExpertSpec::new("support", &[SUPPORT.to_string()]),
    ]
}

/// The day-one engine posture: two demo domains over the fixture corpus
/// (gate semantics: `abstain_confidence = sigmoid(scale·(max_sim − mid))`).
/// The harness (Plan 603 T1.5) is the real corpus driver; this exists so
/// the binary is HONEST out of the box (it abstains off-corpus rather
/// than pretending competence on an empty model).
pub fn demo_engine() -> DecisionEngine<2, EMBED_DIM> {
    crate::engine::DecisionEngine::build_specs(demo_specs(), crate::engine::EngineConfig::default())
        .expect("demo corpus is well-formed")
}

/// The resolved serve corpus: the healthz disclosure plus the engine's
/// builder input.
struct ServeCorpus {
    info: CorpusInfo,
    specs: Vec<crate::engine::ExpertSpec>,
}

/// Resolve the boot corpus (issue 063 T1): `RIIR_REFLEX_CORPUS=<dir>` loads
/// the user's corpus; absent/empty = the demo engine, unchanged. A named
/// corpus that refuses to load EXITS — never a silent demo fallback (the
/// per-lane-claims law).
fn resolve_serve_corpus() -> ServeCorpus {
    let dir = std::env::var("RIIR_REFLEX_CORPUS")
        .ok()
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    match dir {
        None => ServeCorpus {
            info: CorpusInfo::Demo,
            specs: demo_specs(),
        },
        Some(dir) => match crate::corpus::load_dir(std::path::Path::new(&dir)) {
            Ok(loaded) => {
                let named = loaded
                    .domains
                    .iter()
                    .zip(&loaded.docs_per_domain)
                    .map(|(d, n)| format!("{d}({n})"))
                    .collect::<Vec<_>>()
                    .join(", ");
                eprintln!(
                    "[riir-reflex] corpus: {} domain(s) from {} — {named}",
                    loaded.domains.len(),
                    dir
                );
                ServeCorpus {
                    info: CorpusInfo::Domains(loaded.domains),
                    specs: loaded.specs,
                }
            }
            Err(e) => {
                eprintln!("[riir-reflex] RIIR_REFLEX_CORPUS={dir} refused: {e}");
                eprintln!(
                    "[riir-reflex] fix the corpus directory or unset RIIR_REFLEX_CORPUS — \
                     a named corpus is never silently swapped for the demo engine"
                );
                std::process::exit(2);
            }
        },
    }
}

/// Bind + serve (blocking). Errors are `io::Error`s from the bind/accept
/// path; per-connection errors are logged and never kill the loop.
pub fn run() -> std::io::Result<()> {
    // The corpus posture resolves FIRST: a malformed RIIR_REFLEX_CORPUS
    // refuses before anything binds (never a half-booted serve).
    let corpus = resolve_serve_corpus();
    let addr = bind_addr();
    let listener = TcpListener::bind(&addr)?;
    eprintln!(
        "[riir-reflex] listening on http://{addr} (modelless lane, {})",
        crate::VERSION
    );
    eprintln!("[riir-reflex] POST /decide  — DecisionRequest JSON → DecisionResponse JSON");
    eprintln!("[riir-reflex] POST /v1/systemone — the TypeSafe dialect (typesafe_sdk clients; Jev-Mem: TYPESAFE_BASE_URL + a dummy TYPESAFE_API_KEY)");
    eprintln!("[riir-reflex] POST /feedback — {{p, outcome}} → calibrator observe/refit");
    eprintln!("[riir-reflex] GET  /healthz — liveness");
    if corpus.info == CorpusInfo::Demo {
        eprintln!(
            "[riir-reflex] corpus: demo (2 domains: ops, support) — set \
             RIIR_REFLEX_CORPUS=<dir> to serve your own corpus"
        );
    }
    let allow = allowed_origins();
    if allow.is_empty() {
        eprintln!(
            "[riir-reflex] CORS: closed (no RIIR_REFLEX_ALLOWED_ORIGIN) — browser pages cannot reach this engine"
        );
    } else {
        eprintln!("[riir-reflex] CORS: allowed origins — {}", allow.join(", "));
    }
    let laya = Arc::new(Mutex::new(LayaLane::Off));
    spawn_laya_loader_if_requested(&laya);
    eprintln!(
        "[riir-reflex] laya lane: {} (set RIIR_REFLEX_LAYA=1 to enable the comparison lane)",
        laya.lock().unwrap().as_str()
    );
    // The game heads (Plan 607's decoded arms): loaded from the minted
    // PUBLIC-RELEASE head vessels (instinct Proposal 001 T4 — A1: bytes
    // are runtime, vessels never committed, never compiled in; the boot
    // fit is GONE). Absent dir = a LOUD serving posture (the boards
    // abstain), never a silent empty; a present-but-broken vessel is a
    // hard boot failure — a vessel that asked to be a head and failed is
    // never papered over.
    let heads = Arc::new(resolve_serve_heads());
    if heads.has_tetris() {
        eprintln!(
            "[riir-reflex] game head: tetris loaded from vessel ({} corpus options, λ {}, digest {})",
            heads.n_options(),
            heads.lambda(),
            heads.digest_hex()
        );
    }
    if heads.has_lanes() {
        let (lanes_lambda, lanes_digest, lanes_n) = heads.lanes_fit();
        eprintln!(
            "[riir-reflex] game head: lanes loaded from vessel ({} corpus options, λ {}, digest {})",
            lanes_n, lanes_lambda, lanes_digest
        );
    }
    if heads.has_flappy() {
        let (flappy_lambda, flappy_digest, flappy_n) = heads.flappy_fit();
        eprintln!(
            "[riir-reflex] game head: flappy v3 loaded from vessel ({} corpus options, λ {}, digest {})",
            flappy_n, flappy_lambda, flappy_digest
        );
    }
    serve_boot(listener, corpus, laya, heads)
}

/// Is a `RIIR_REFLEX_*` knob env var set to a nonblank value?
fn env_set(name: &str) -> bool {
    std::env::var(name)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

/// Parse a positive-scale env knob's value (`RIIR_REFLEX_NB_SCALE`,
/// `RIIR_REFLEX_RIDGE_SCALE`): `Ok(None)` = unset/blank (the standing
/// off posture, byte-identical), `Ok(Some(v))` = a positive finite scale,
/// `Err(why)` = the boot refuses (never a quiet default).
fn parse_positive_scale(name: &str, raw: Option<&str>) -> Result<Option<f32>, String> {
    let Some(raw) = raw.map(str::trim).filter(|r| !r.is_empty()) else {
        return Ok(None);
    };
    let v: f32 = raw
        .parse()
        .map_err(|_| format!("{name}={raw:?} is not a number"))?;
    if !v.is_finite() || v <= 0.0 {
        return Err(format!("{name}={raw:?} must be a positive finite scale"));
    }
    Ok(Some(v))
}

/// The N-dispatched boot (issue 063 option 1): build the corpus engine at
/// the request's domain count — one monomorphised arm per `N ∈ 1..=8`, the
/// dispatch happens ONCE per process, and the serve loop below stays
/// compile-time-shaped (the hot path is untouched). The demo posture
/// dispatches through the same arms — it is just `N = 2`.
fn serve_boot(
    listener: TcpListener,
    corpus: ServeCorpus,
    laya: Arc<Mutex<LayaLane>>,
    heads: Arc<GameHeads>,
) -> std::io::Result<()> {
    let ServeCorpus { info, specs } = corpus;
    let cfg = if info == CorpusInfo::Demo {
        if env_set("RIIR_REFLEX_NB_SCALE")
            || env_set("RIIR_REFLEX_RIDGE_SCALE")
            || env_set("RIIR_REFLEX_NB_PAIR_SCALE")
        {
            eprintln!(
                "[riir-reflex] RIIR_REFLEX_NB_SCALE/RIIR_REFLEX_RIDGE_SCALE/RIIR_REFLEX_NB_PAIR_SCALE \
                 ignored on the demo posture (the fixture engine is fixed; set RIIR_REFLEX_CORPUS \
                 to arm them)"
            );
        }
        crate::engine::EngineConfig::default()
    } else {
        // The first-corpus posture (measured, issue 063): a first corpus is
        // too thin for the confidence readout to separate — the k≥3 readout
        // sits near-uniform (0.33/0.34/0.33, confidence ≈ 0.0002) and
        // abstains at EVERY default score threshold (the harness's own
        // finding: the 0.35/0.5 birth thresholds do not transfer to real
        // corpora). The corpus IS labeled by construction, so the
        // corpus-distance gate decides: in-corpus answers with the engine's
        // best pick, off-corpus still abstains (in-corpus distance
        // confidence ≈ 0.93+, off-corpus ≈ 0.06 — the 0.5 default midpoint
        // sits in the desert between them).
        let mut cfg = crate::engine::EngineConfig {
            score_threshold: 0.0,
            ..crate::engine::EngineConfig::default()
        };
        // Issue 081 T2c: arm the per-domain count tables (`nb_scope`,
        // issue 038) and/or the NBSVM ridge readout (`nb_ridge`) on a
        // corpus boot — the fitted-token head surfaces the memory-control
        // lane needs (a discriminative state token such as `depth` is
        // invisible to the compression drafter but is a plain count-table
        // feature). Scales are SELECTED offline (the eval arm); a bad value
        // refuses the boot, never a quiet default.
        let mut nb_scale_val: Option<f32> = None;
        let mut ridge_scale_val: Option<f32> = None;
        let mut nb_pair_val: Option<f32> = None;
        for (name, slot) in [
            ("RIIR_REFLEX_NB_SCALE", &mut nb_scale_val),
            ("RIIR_REFLEX_RIDGE_SCALE", &mut ridge_scale_val),
            ("RIIR_REFLEX_NB_PAIR_SCALE", &mut nb_pair_val),
        ] {
            match parse_positive_scale(name, std::env::var(name).ok().as_deref()) {
                Ok(None) => {}
                Ok(Some(v)) => *slot = Some(v),
                Err(why) => {
                    eprintln!("[riir-reflex] {why} — refusing");
                    std::process::exit(2);
                }
            }
        }
        #[cfg(feature = "nb_scope")]
        if let Some(scale) = nb_scale_val {
            cfg.nb_scale = scale;
            eprintln!(
                "[riir-reflex] corpus nb count tables armed: nb_scale={scale} \
                 (RIIR_REFLEX_NB_SCALE; by-name option routing resolves per-question)"
            );
        }
        #[cfg(feature = "nb_scope")]
        if let Some(scale) = nb_pair_val {
            cfg.nb_pair_scale = scale;
            eprintln!(
                "[riir-reflex] corpus nb PAIRWISE head armed: nb_pair_scale={scale} \
                 (RIIR_REFLEX_NB_PAIR_SCALE; option-set-scoped margins in bits — \
                 the O(1)-token functional, issue 081 T2c)"
            );
        }
        #[cfg(feature = "nb_ridge")]
        if let Some(scale) = ridge_scale_val {
            cfg.ridge_scale = scale;
            eprintln!(
                "[riir-reflex] corpus nb ridge readout armed: ridge_scale={scale} \
                 (RIIR_REFLEX_RIDGE_SCALE; discriminative margins over the count-table stream)"
            );
        }
        // The knobs never silently no-op (the flag-does-nothing bug class):
        // a build without the fitted surface refuses the boot naming the
        // rebuild. (nb_ridge implies nb_scope, so the nb check covers the
        // compile-gate for both only when checked in the right order —
        // check each against its own feature.)
        #[cfg(not(feature = "nb_scope"))]
        if nb_scale_val.is_some() || nb_pair_val.is_some() {
            eprintln!(
                "[riir-reflex] RIIR_REFLEX_NB_SCALE/RIIR_REFLEX_NB_PAIR_SCALE set but this build \
                 compiles without the nb_scope feature — rebuild with \
                 --features nb_scope or unset the knob"
            );
            std::process::exit(2);
        }
        #[cfg(not(feature = "nb_ridge"))]
        if ridge_scale_val.is_some() {
            eprintln!(
                "[riir-reflex] RIIR_REFLEX_RIDGE_SCALE is set but this build \
                 compiles without the nb_ridge feature — rebuild with \
                 --features nb_ridge or unset the knob"
            );
            std::process::exit(2);
        }
        let _ = (nb_scale_val.is_some(), ridge_scale_val.is_some(), nb_pair_val.is_some());
        cfg
    };
    macro_rules! boot {
        ($n:literal) => {{
            match crate::engine::DecisionEngine::<$n, EMBED_DIM>::build_specs(specs, cfg) {
                Ok(eng) => {
                    return serve_listener_heads_corpus(
                        listener,
                        Arc::new(Mutex::new(eng)),
                        laya,
                        allowed_origins(),
                        heads,
                        &info,
                    );
                }
                Err(e) => {
                    eprintln!("[riir-reflex] corpus engine refused: {e}");
                    std::process::exit(1);
                }
            }
        }};
    }
    if info != CorpusInfo::Demo {
        eprintln!(
            "[riir-reflex] corpus gates: distance-only (score axis open — a first corpus is \
             too thin to calibrate the confidence readout; off-corpus still abstains)"
        );
    }
    match specs.len() {
        1 => boot!(1),
        2 => boot!(2),
        3 => boot!(3),
        4 => boot!(4),
        5 => boot!(5),
        6 => boot!(6),
        7 => boot!(7),
        8 => boot!(8),
        n => {
            eprintln!(
                "[riir-reflex] {n} domains exceed the shipped 1..={} dispatch — \
                 split the corpus or use the in-repo harness lane",
                crate::corpus::MAX_DOMAINS
            );
            std::process::exit(2);
        }
    }
}

/// Resolve the serve-time game heads. DEFAULT posture (no
/// `RIIR_REFLEX_HEADS_DIR`): loud-absent — the arena game boards abstain
/// and the boot line names the env + the mint command. With a dir: the
/// vessels are opened (strict ed25519 + blake3) and the heads installed
/// WHOLE — nothing is re-fitted.
#[cfg(feature = "vessel_public_read")]
fn resolve_serve_heads() -> crate::game_heads::GameHeads {
    use crate::game_heads::{absent_heads_message, GameHeads, HEAD_VESSEL_FILES};
    let dir = match std::env::var("RIIR_REFLEX_HEADS_DIR") {
        Ok(d) if !d.trim().is_empty() => std::path::PathBuf::from(d),
        _ => {
            eprintln!("[riir-reflex] {}", absent_heads_message("RIIR_REFLEX_HEADS_DIR"));
            return GameHeads::absent();
        }
    };
    // Trust anchors: the compiled pin table first (EMPTY until the first
    // release artifact ships), then the operator env — the same
    // pins-first-wildcard-adds shape the reflexer bin uses. A dir with
    // vessels but NO trust anchor is a config gap: refuse loud (exit 2)
    // naming the env, never fail every vessel with a generic unknown-key.
    let mut pins = reflexer_vessel::default_pins();
    let wildcard = std::env::var("RIIR_REFLEX_HEADS_PUBKEY").ok();
    if let Some(hex) = wildcard.as_deref() {
        let bytes = hex_decode32(hex.trim()).unwrap_or_else(|e| {
            eprintln!("[riir-reflex] RIIR_REFLEX_HEADS_PUBKEY: {e}");
            std::process::exit(2);
        });
        let vk = reflexer_vessel::ed25519_dalek::VerifyingKey::from_bytes(&bytes)
            .unwrap_or_else(|e| {
                eprintln!("[riir-reflex] RIIR_REFLEX_HEADS_PUBKEY: bad verifying key: {e}");
                std::process::exit(2);
            });
        pins = pins.with_wildcard(vk);
    }
    let dir_has_vessels = HEAD_VESSEL_FILES.iter().any(|f| dir.join(f).exists());
    if dir_has_vessels && reflexer_vessel::DEFAULT_PIN_KEYS.is_empty() && wildcard.is_none() {
        eprintln!(
            "[riir-reflex] game heads: {} carries vessels but no trust anchor is configured — \
             set RIIR_REFLEX_HEADS_PUBKEY to the minting key's verifying key hex (printed by \
             `reflexer sign` / `reflex mint-heads`)",
            dir.display()
        );
        std::process::exit(2);
    }
    match GameHeads::from_vessel_dir(&dir, &pins) {
        Ok(heads) => {
            if !heads.has_any() {
                eprintln!(
                    "[riir-reflex] game heads: {} carries none of {:?} — {}",
                    dir.display(),
                    HEAD_VESSEL_FILES,
                    absent_heads_message("RIIR_REFLEX_HEADS_DIR")
                );
            }
            heads
        }
        Err(e) => {
            eprintln!("[riir-reflex] game heads refused from {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
}

/// Feature-off posture: the vessel reader is not compiled — serve boots
/// WITHOUT head vessels, with the same loud line (never a silent empty).
#[cfg(not(feature = "vessel_public_read"))]
fn resolve_serve_heads() -> crate::game_heads::GameHeads {
    eprintln!(
        "[riir-reflex] {}",
        crate::game_heads::absent_heads_message("RIIR_REFLEX_HEADS_DIR")
    );
    eprintln!(
        "[riir-reflex] game heads: unavailable — this build was compiled without the \
         vessel_public_read feature"
    );
    crate::game_heads::GameHeads::absent()
}

/// Parse a 64-hex verifying key (the `RIIR_REFLEX_HEADS_PUBKEY` env form).
#[cfg(feature = "vessel_public_read")]
fn hex_decode32(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("expected 64 hex chars (32 bytes), got {} chars", s.len()));
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16).expect("hex");
        let lo = (chunk[1] as char).to_digit(16).expect("hex");
        out[i] = ((hi << 4) | lo) as u8;
    }
    Ok(out)
}

/// Serve on an ALREADY-BOUND listener with an EXPLICIT CORS allow-list and
/// an EXPLICIT game-head lane (the test seam — no process-global env
/// mutation across test threads, and no boot-fit: the caller owns the
/// heads, fitted or loaded).
pub fn serve_listener_with<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    allow: Vec<String>,
    heads: Arc<GameHeads>,
) -> std::io::Result<()> {
    serve_listener_heads(listener, engine, laya, allow, heads)
}

/// Serve on an ALREADY-BOUND listener with an EXPLICIT game-head lane (the
/// `run()` seam — the boot-fit owns the head; the edge reads it). The
/// corpus posture is the demo one (issue 063: `/healthz` discloses
/// `"demo"`); a corpus boot goes through [`serve_listener_heads_corpus`].
pub fn serve_listener_heads<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    allow: Vec<String>,
    heads: Arc<GameHeads>,
) -> std::io::Result<()> {
    serve_listener_heads_corpus(listener, engine, laya, allow, heads, &CorpusInfo::Demo)
}

/// The corpus-aware funnel (`run()`'s path — issue 063): the same accept
/// loop, with the boot's corpus posture disclosed on `/healthz`. The legacy
/// fns delegate with [`CorpusInfo::Demo`], so the demo posture's healthz
/// stays honest for what it serves.
pub fn serve_listener_heads_corpus<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    allow: Vec<String>,
    heads: Arc<GameHeads>,
    corpus: &CorpusInfo,
) -> std::io::Result<()> {
    let allow: Arc<[String]> = allow.into();
    let corpus: Arc<str> = Arc::from(corpus.as_json().as_str());
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let eng = Arc::clone(&engine);
                let laya = Arc::clone(&laya);
                let allow = Arc::clone(&allow);
                let heads = Arc::clone(&heads);
                let corpus = Arc::clone(&corpus);
                std::thread::spawn(move || {
                    if let Err(e) = handle_conn(s, eng, laya, &allow, &heads, &corpus) {
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
/// comes from `allowed_origins()`, the laya lane stays off, and the caller
/// owns the heads).
pub fn serve_listener<const N: usize, const D: usize>(
    listener: TcpListener,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    heads: Arc<GameHeads>,
) -> std::io::Result<()> {
    serve_listener_heads(
        listener,
        engine,
        Arc::new(Mutex::new(LayaLane::Off)),
        allowed_origins(),
        heads,
    )
}

struct Req {
    method: String,
    path: String,
    content_length: usize,
    origin: Option<String>,
    /// `X-Reflex-Lane` — the lane hint for `/decide` (`laya` | `modelless` | `raw`).
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

/// The one JSON error shape every refusal shares (`{"error":"…"}`, the
/// message JSON-escaped — a quote in the reason must never break the body).
fn json_error(stream: &mut TcpStream, status: &str, text: &str, cors: Option<&str>) {
    json_response(
        stream,
        status,
        &format!(
            "{{\"error\":{}}}",
            serde_json::to_string(text).unwrap_or_default()
        ),
        cors,
    );
}

fn handle_conn<const N: usize, const D: usize>(
    stream: TcpStream,
    engine: Arc<Mutex<DecisionEngine<N, D>>>,
    laya: Arc<Mutex<LayaLane>>,
    allow: &[String],
    heads: &GameHeads,
    corpus: &str,
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
                let head =
                    "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                let _ = writer.write_all(head.as_bytes());
                let _ = writer.flush();
            }
        },
        ("GET", "/healthz") => {
            let laya_state = laya.lock().unwrap().as_str();
            // The game-head map reflects the ACTUAL lanes: true when the
            // head is installed (loaded from its vessel — or fitted, the
            // caller's choice), false when absent (the loud-absent
            // serving posture — the arena page labels precisely instead
            // of guessing).
            json_response(
                &mut writer,
                "200 OK",
                &format!(
                    "{{\"status\":\"ok\",\"lanes\":{{\"modelless\":\"ready\",\"raw\":\"ready\",\"laya\":\"{laya_state}\"}},\"heads\":{{\"tetris\":{},\"lanes\":{},\"flappy\":{}}},\"corpus\":{corpus}}}",
                    heads.has_tetris(),
                    heads.has_lanes(),
                    heads.has_flappy()
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
            let parsed: Result<ServeRequest, _> = serde_json::from_slice(&body);
            if let Some(other) = &req.lane
                && !matches!(other.as_str(), "modelless" | "laya" | "laya-ane" | "raw")
            {
                json_error(
                    &mut writer,
                    "400 Bad Request",
                    &format!(
                        "unknown lane {other:?} (supported: modelless, raw, laya, laya-ane)"
                    ),
                    cors.as_deref(),
                );
                return Ok(());
            }
            match parsed {
                Ok(parsed) => {
                    // Parsed-and-ignored: the widened-wire sidecar never
                    // reaches a lane below (riir-instinct Issue 009 T5).
                    let ServeRequest {
                        request: parsed_req,
                        sidecar: _,
                    } = parsed;
                    let laya_route = match req.lane.as_deref() {
                        Some("laya") => Some(LayaRoute::Auto),
                        Some("laya-ane") => Some(LayaRoute::AneExplicit),
                        _ => None,
                    };
                    if let Some(route) = laya_route {
                        laya_edge(&mut writer, &laya, &parsed_req, route, cors.as_deref());
                        return Ok(());
                    }
                    // The raw lane (`X-Reflex-Lane: raw`): the client asks
                    // for the modelless engine WITHOUT the game-head try —
                    // the honest baseline the arena's third board renders.
                    // An abstain IS the answer here, never a head fallback.
                    if req.lane.as_deref() == Some("raw") {
                        engine_decide(&mut writer, &engine, &parsed_req, cors.as_deref());
                        return Ok(());
                    }
                    // Default (and the `modelless` spelling): the fitted
                    // game head first (Plan 607's decoded Tetris head): a
                    // well-formed spot question is answered from the
                    // boot-fitted head. Everything else — grammar-invalid
                    // state, foreign question — falls through to the
                    // cosine engine, which abstains off-corpus as before.
                    if let Some(resp) = heads.respond(&parsed_req) {
                        json_response(
                            &mut writer,
                            "200 OK",
                            &serde_json::to_string(&resp).unwrap_or_default(),
                            cors.as_deref(),
                        );
                        return Ok(());
                    }
                    engine_decide(&mut writer, &engine, &parsed_req, cors.as_deref());
                }
                Err(e) => json_error(
                    &mut writer,
                    "400 Bad Request",
                    &e.to_string(),
                    cors.as_deref(),
                ),
            }
        }
        ("POST", "/v1/systemone") => {
            // The TypeSafe dialect (issue 081): THEIR SDK calls US — Jev-Mem
            // (or any typesafe_sdk client) points TYPESAFE_BASE_URL here.
            // Straight to the modelless engine: no game-head try (that is
            // the arena's posture), no lane header (the dialect has no
            // lanes — one modelless answer per question, always).
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
            let mut eng = engine.lock().unwrap();
            match crate::systemone::respond(&mut eng, &body) {
                Ok(out) => json_response(&mut writer, "200 OK", &out, cors.as_deref()),
                Err(e) => json_error(&mut writer, "400 Bad Request", &e, cors.as_deref()),
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

/// The raw modelless engine's own answer — the `X-Reflex-Lane: raw` path
/// and the head's fall-through share it. The engine's response discloses
/// itself (lane `modelless`, the engine's own routing reason), never the
/// head's: the per-lane-claims law holds by construction.
fn engine_decide<const N: usize, const D: usize>(
    writer: &mut TcpStream,
    engine: &Arc<Mutex<DecisionEngine<N, D>>>,
    req: &DecisionRequest,
    cors: Option<&str>,
) {
    match engine.lock().unwrap().decide(req) {
        Ok(resp) => json_response(
            writer,
            "200 OK",
            &serde_json::to_string(&resp).unwrap_or_default(),
            cors,
        ),
        Err(e) => json_error(writer, "422 Unprocessable Entity", &e.to_string(), cors),
    }
}

/// The `X-Reflex-Lane: laya`/`laya-ane` edge: fail-closed in every
/// non-ready state (a silent modelless fallback would serve the WRONG lane
/// under the arena's per-lane claims), and the router's decision when
/// ready. Each typed error carries its OWN status: Bucket → 422 (the
/// explicit-device coverage limit — never padded), Unavailable → 503,
/// Other → 502 (the request failed on the device that owned it).
fn laya_edge(
    writer: &mut TcpStream,
    laya: &Arc<Mutex<LayaLane>>,
    req: &DecisionRequest,
    route: LayaRoute,
    cors: Option<&str>,
) {
    let state = laya.lock().unwrap().clone();
    match state {
        LayaLane::Ready(f) => match f(req, route) {
            Ok(resp) => json_response(
                writer,
                "200 OK",
                &serde_json::to_string(&resp).unwrap_or_default(),
                cors,
            ),
            Err(LayaServeError::Bucket { seq, max, .. }) => json_response(
                writer,
                "422 Unprocessable Entity",
                &format!(
                    "{{\"error\":{}}}",
                    serde_json::to_string(&format!(
                        "sequence length {seq} exceeds every ane bucket (max {max}) — \
                         an explicit-device (laya-ane) request is never padded up; \
                         retry with X-Reflex-Lane: laya (auto) or shorten the state"
                    ))
                    .unwrap_or_default()
                ),
                cors,
            ),
            Err(LayaServeError::Unavailable(e)) => json_response(
                writer,
                "503 Service Unavailable",
                &format!(
                    "{{\"error\":{}}}",
                    serde_json::to_string(&format!("laya ane: {e}"))
                        .unwrap_or_default()
                ),
                cors,
            ),
            Err(LayaServeError::Other(e)) => json_response(
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
        LayaLane::Off => {
            let remedy = match route {
                LayaRoute::Auto => "RIIR_REFLEX_LAYA=1",
                // The explicit-device spelling names its own env.
                LayaRoute::AneExplicit => "RIIR_REFLEX_LAYA_ANE=1",
            };
            json_response(
                writer,
                "503 Service Unavailable",
                &format!(
                    "{{\"error\":\"laya lane is not enabled — restart the engine with {remedy}\"}}"
                ),
                cors,
            )
        }
        LayaLane::Failed(e) => json_response(
            writer,
            "500 Internal Server Error",
            &format!(
                "{{\"error\":{}}}",
                serde_json::to_string(&format!("laya lane failed to load: {e}"))
                    .unwrap_or_default()
            ),
            cors,
        ),
    }
}

#[cfg(test)]
mod nb_scale_tests {
    use super::parse_positive_scale;

    #[test]
    fn scale_unset_or_blank_is_none() {
        assert_eq!(parse_positive_scale("K", None), Ok(None));
        assert_eq!(parse_positive_scale("K", Some("")), Ok(None));
        assert_eq!(parse_positive_scale("K", Some("   ")), Ok(None));
    }

    #[test]
    fn scale_positive_finite_parses() {
        assert_eq!(parse_positive_scale("K", Some("16")), Ok(Some(16.0)));
        assert_eq!(parse_positive_scale("K", Some(" 0.5 ")), Ok(Some(0.5)));
        assert_eq!(parse_positive_scale("K", Some("1e3")), Ok(Some(1000.0)));
    }

    #[test]
    fn scale_bad_values_refuse_never_default() {
        // Not a number, zero, negative, non-finite: every bad spelling
        // refuses (Err) — the boot exits, it never arms a guessed scale.
        assert!(parse_positive_scale("K", Some("abc")).is_err());
        assert!(parse_positive_scale("K", Some("0")).is_err());
        assert!(parse_positive_scale("K", Some("-4")).is_err());
        assert!(parse_positive_scale("K", Some("inf")).is_err());
        assert!(parse_positive_scale("K", Some("nan")).is_err());
        // The refusal NAMES the knob (two knobs, two names).
        let why = parse_positive_scale("RIIR_REFLEX_RIDGE_SCALE", Some("x")).unwrap_err();
        assert!(why.contains("RIIR_REFLEX_RIDGE_SCALE"));
    }
}
