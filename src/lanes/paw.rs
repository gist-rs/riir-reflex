//! The PAW comparison lane (`.issues/033`; ProgramAsWeights, MIT SDK,
//! not affiliated) — the compile-a-classifier product category measured on
//! the same harness cases as every other lane. Their stack serves, our
//! Rust measures; comparison lane, never a product lane.
//!
//! The contract (read at the pinned SDK tree `programasweights-python` @
//! `74919f69`, `client.py` / `_remote.py` / `docs/api-reference/rest-api.md`
//! — NOT the issue's guesses; three of them were wrong, see below):
//!
//! * **Compile once per suite:** `POST /api/v1/compile`
//!   `{spec, public[, compiler]}` → `{status: "ready", program_id,
//!   compiler_snapshot, timings, ...}` (HTTP 202 on success — the server
//!   answers synchronously with `status` already `ready`). The explicit
//!   finetune compilers (`paw-ft-*`) go through
//!   `POST /api/v1/compile/async` + `GET /api/v1/compile/{job_id}` polling
//!   (`queued` / `compiling` / `ready` / `failed` / `cancelled`) — opt in
//!   with `PAW_COMPILE_ASYNC=1` beside `PAW_COMPILER`.
//! * **Infer per question:** `POST /api/v1/infer`
//!   `{program_id, input, temperature, max_tokens}` → `{output,
//!   latency_ms, ...}`. `temperature` is pinned to 0.0 (their own default).
//! * **Auth is OPTIONAL** (their AGENTS.md: "Everything works without
//!   it") — the issue's "no key = SKIP" premise is wrong. Anonymous
//!   compile is rate-limited (20/h, 1 concurrent) and **cannot create a
//!   private program** (HTTP 401 `auth_required`, measured 2026-09-26), so
//!   the anonymous posture compiles `public: true`; with `PAW_API_KEY` set
//!   the lane sends `X-API-Key` and compiles `public: false`. The posture
//!   is stamped into the row and printed LOUDLY — never silent either way.
//! * **Base URL env is `PAW_API_URL`** (the SDK's own name), default
//!   `https://programasweights.com` — a stub server rides the same knob.
//!
//! Transport: hosted PAW is HTTPS-only and this repo carries no TLS client
//! (BOUNDARY: no new deps for a comparison lane), so requests go through a
//! `curl` SUBPROCESS — the in-repo subprocess precedent (`ndb`, the Python
//! oracles). The body rides stdin, the API key rides a 0600 header FILE
//! (`-H @file`), never argv (a key in argv is visible to `ps`).
//!
//! **The mapping law** (declared BEFORE the first measured cell; the gliner
//! lane's never-guess law): the stripped output is matched EXACTLY
//! (case-sensitive) against the question's option KEYS, then against its
//! option DESCRIPTIONS when those are unique. One layer of matching
//! surrounding quotes (`"…"` / `'…'` / `` `…` ``) is stripped first — a
//! measured output shape (their compiled pseudo-programs quote the label:
//! `"\"neutral\""`), counted per suite as `quote_stripped` so the
//! normalization is visible. Anything else — a case mismatch, a trailing
//! period, prose, two labels — is a recorded REFUSAL, counted per suite,
//! never guessed. Accuracy counts a refusal as wrong (n = every served
//! question); `answered_accuracy` beside it reads the answered subset.
//! noul → `false`/`true` exact strings; score → the level strings verbatim.
//! PAW returns free text, not a distribution — NO confidence / ECE columns
//! (the disclosed divergence, the clm-lane noul law style).

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::harness::suites::{QKind, Suite, SuiteCase, SuiteQuestion};

/// Base URL env (the SDK's own name, `config.get_api_url`).
pub const API_URL_ENV: &str = "PAW_API_URL";
/// Their hosted API.
pub const DEFAULT_API_URL: &str = "https://programasweights.com";
/// Optional API key env (the SDK's own name). Absent = anonymous posture.
pub const API_KEY_ENV: &str = "PAW_API_KEY";
/// Optional compiler name (absent = the server default, today
/// `paw-4b-qwen3-0.6b`).
pub const COMPILER_ENV: &str = "PAW_COMPILER";
/// `1` = compile through the async endpoint + status polling (requires
/// `PAW_COMPILER` naming a finetune compiler — their endpoint's rule).
pub const COMPILE_ASYNC_ENV: &str = "PAW_COMPILE_ASYNC";
/// Where the committed per-suite spec texts live.
pub const SPECS_DIR_ENV: &str = "PAW_SPECS_DIR";
pub const DEFAULT_SPECS_DIR: &str = "scripts/paw_specs";
/// The program-id cache (keyed `(suite, compiler, BLAKE3(spec))`).
pub const CACHE_FILE_ENV: &str = "PAW_PROGRAM_CACHE";
pub const DEFAULT_CACHE_FILE: &str = ".raw/paw/programs.json";
/// The curl binary (override for a non-PATH install).
pub const CURL_ENV: &str = "PAW_CURL";

/// Generation bound per answer: the longest option string in the four
/// specced suites is ~10 tokens (+ quotes); an output that needs more than
/// this cannot be an exact match anyway, so the cap only bounds a runaway
/// generation's latency (disclosed posture, never an accuracy lever).
pub const MAX_TOKENS: u32 = 48;

/// Retries for 429 / 502 / 503 / 504 (Retry-After honored, else
/// exponential, capped 30 s). Any other non-2xx is a LOUD lane failure.
const MAX_ATTEMPTS: u32 = 6;

/// The lane's configuration — built from the env in production, directly in
/// tests (env mutation is racy across test threads).
#[derive(Debug, Clone)]
pub struct PawConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub compiler: Option<String>,
    pub compile_async: bool,
    pub specs_dir: PathBuf,
    pub cache_file: PathBuf,
    pub curl: String,
    /// Async-compile poll interval + total deadline.
    pub poll: Duration,
    pub compile_deadline: Duration,
}

impl PawConfig {
    #[must_use]
    pub fn from_env() -> Self {
        let nonempty = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        Self {
            api_url: nonempty(API_URL_ENV).unwrap_or_else(|| DEFAULT_API_URL.to_string()),
            api_key: nonempty(API_KEY_ENV),
            compiler: nonempty(COMPILER_ENV),
            compile_async: nonempty(COMPILE_ASYNC_ENV).is_some_and(|v| v == "1"),
            specs_dir: PathBuf::from(
                nonempty(SPECS_DIR_ENV).unwrap_or_else(|| DEFAULT_SPECS_DIR.to_string()),
            ),
            cache_file: PathBuf::from(
                nonempty(CACHE_FILE_ENV).unwrap_or_else(|| DEFAULT_CACHE_FILE.to_string()),
            ),
            curl: nonempty(CURL_ENV).unwrap_or_else(|| "curl".to_string()),
            poll: Duration::from_secs(10),
            compile_deadline: Duration::from_secs(2400),
        }
    }

    /// The posture word stamped into every row.
    #[must_use]
    pub fn posture(&self) -> &'static str {
        match self.api_key {
            Some(_) => "hosted-authenticated",
            None => "hosted-anonymous",
        }
    }
}

// ── transport: curl subprocess ─────────────────────────────────────────

/// One parsed HTTP reply: status, lowercased headers, body.
#[derive(Debug)]
struct HttpReply {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

/// Split curl `-i` output into the FINAL response (skipping `100 Continue`
/// interim blocks). Pure — pinned by a unit test.
fn parse_curl_include(raw: &[u8]) -> Result<HttpReply, String> {
    let mut rest = raw;
    loop {
        let (head, body) = split_head(rest)
            .ok_or_else(|| "curl output carried no HTTP header block".to_string())?;
        let head = String::from_utf8_lossy(head);
        let mut lines = head.lines();
        let status_line = lines.next().unwrap_or("");
        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("unparseable HTTP status line {status_line:?}"))?;
        if status == 100 {
            rest = body;
            continue;
        }
        let headers = lines
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
            .collect();
        return Ok(HttpReply {
            status,
            headers,
            body: body.to_vec(),
        });
    }
}

fn split_head(raw: &[u8]) -> Option<(&[u8], &[u8])> {
    if let Some(i) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
        return Some((&raw[..i], &raw[i + 4..]));
    }
    raw.windows(2)
        .position(|w| w == b"\n\n")
        .map(|i| (&raw[..i], &raw[i + 2..]))
}

/// The API-key header file: 0600, pid-suffixed (the shared-temp-path law),
/// removed on drop.
#[derive(Debug)]
struct KeyHeaderFile(PathBuf);

impl KeyHeaderFile {
    fn create(key: &str) -> Result<Self, String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path = std::env::temp_dir().join(format!(
            "reflex_paw_key_{}_{nanos}.hdr",
            std::process::id()
        ));
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts
            .open(&path)
            .map_err(|e| format!("paw key header file {}: {e}", path.display()))?;
        writeln!(f, "X-API-Key: {key}").map_err(|e| format!("paw key header write: {e}"))?;
        Ok(Self(path))
    }
}

impl Drop for KeyHeaderFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The hosted-API client (compile + infer) over `curl`.
#[derive(Debug)]
pub struct PawClient {
    cfg: PawConfig,
    key_file: Option<KeyHeaderFile>,
}

/// A program ready to infer against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledProgram {
    pub program_id: String,
    pub compiler_snapshot: Option<String>,
    /// Client wall time of the compile that produced this id (from the
    /// cache on a hit — the product-comparison disclosure).
    pub compile_wall_s: f64,
    /// The posture that compiled it (`hosted-anonymous` programs are public).
    pub posture: String,
}

impl PawClient {
    /// # Errors
    /// When the key header file cannot be written.
    pub fn new(cfg: PawConfig) -> Result<Self, String> {
        let key_file = match &cfg.api_key {
            Some(k) => Some(KeyHeaderFile::create(k)?),
            None => None,
        };
        Ok(Self { cfg, key_file })
    }

    #[must_use]
    pub fn config(&self) -> &PawConfig {
        &self.cfg
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.cfg.api_url.trim_end_matches('/'))
    }

    /// One request, no retry.
    fn request_once(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        timeout: Duration,
    ) -> Result<HttpReply, String> {
        let mut cmd = Command::new(&self.cfg.curl);
        cmd.args(["-sS", "-i", "-X", method, "--max-time"])
            .arg(timeout.as_secs().max(1).to_string())
            .args(["-H", "Content-Type: application/json", "-H", "Expect:"]);
        if let Some(kf) = &self.key_file {
            cmd.arg("-H").arg(format!("@{}", kf.0.display()));
        }
        if body.is_some() {
            cmd.args(["--data-binary", "@-"]);
        }
        cmd.arg(self.url(path))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("spawn {} (the PAW lane's transport): {e}", self.cfg.curl))?;
        {
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| "curl stdin unavailable".to_string())?;
            if let Some(b) = body {
                stdin
                    .write_all(b.to_string().as_bytes())
                    .map_err(|e| format!("curl stdin: {e}"))?;
            }
        }
        let mut out = Vec::new();
        child
            .stdout
            .take()
            .ok_or_else(|| "curl stdout unavailable".to_string())?
            .read_to_end(&mut out)
            .map_err(|e| format!("curl stdout: {e}"))?;
        let mut err = String::new();
        if let Some(mut e) = child.stderr.take() {
            let _ = e.read_to_string(&mut err);
        }
        let status = child.wait().map_err(|e| format!("curl wait: {e}"))?;
        if !status.success() {
            return Err(format!(
                "curl {method} {path} exited {status}: {}",
                err.trim()
            ));
        }
        parse_curl_include(&out)
    }

    /// One request with the bounded retry law (429/502/503/504 only).
    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        timeout: Duration,
    ) -> Result<HttpReply, String> {
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let reply = self.request_once(method, path, body, timeout)?;
            let retryable = matches!(reply.status, 429 | 502 | 503 | 504);
            if !retryable || attempt >= MAX_ATTEMPTS {
                return Ok(reply);
            }
            let wait = reply
                .headers
                .get("retry-after")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1u64 << attempt.min(5))
                .min(30);
            eprintln!(
                "    [paw] {method} {path} -> HTTP {} — retry {attempt}/{MAX_ATTEMPTS} in {wait}s",
                reply.status
            );
            std::thread::sleep(Duration::from_secs(wait));
        }
    }

    fn json_ok(reply: &HttpReply, what: &str) -> Result<Value, String> {
        let body = String::from_utf8_lossy(&reply.body);
        if !(200..300).contains(&reply.status) {
            return Err(format!("{what} -> HTTP {}: {}", reply.status, body.trim()));
        }
        serde_json::from_slice(&reply.body).map_err(|e| format!("{what}: bad JSON ({e}): {body}"))
    }

    fn compile_body(&self, spec: &str) -> Value {
        let mut body = json!({
            "spec": spec,
            // Anonymous programs cannot be private (HTTP 401 auth_required).
            "public": self.cfg.api_key.is_none(),
        });
        if let Some(c) = &self.cfg.compiler {
            body["compiler"] = json!(c);
        }
        body
    }

    fn program_from(&self, v: &Value, wall: f64, what: &str) -> Result<CompiledProgram, String> {
        let status = v.get("status").and_then(Value::as_str).unwrap_or("");
        let id = v.get("program_id").and_then(Value::as_str).unwrap_or("");
        if status != "ready" || id.is_empty() {
            return Err(format!(
                "{what}: not ready (status {status:?}, program_id {id:?}, error {})",
                v.get("error").unwrap_or(&Value::Null)
            ));
        }
        Ok(CompiledProgram {
            program_id: id.to_string(),
            compiler_snapshot: v
                .get("compiler_snapshot")
                .and_then(Value::as_str)
                .map(str::to_string),
            compile_wall_s: wall,
            posture: self.cfg.posture().to_string(),
        })
    }

    /// Compile `spec` (sync endpoint, or async + polling when configured).
    ///
    /// # Errors
    /// Any transport failure, non-2xx, or a non-`ready` terminal status.
    pub fn compile(&self, spec: &str) -> Result<CompiledProgram, String> {
        let t0 = Instant::now();
        let body = self.compile_body(spec);
        if !self.cfg.compile_async {
            let reply = self.request(
                "POST",
                "/api/v1/compile",
                Some(&body),
                self.cfg.compile_deadline,
            )?;
            let v = Self::json_ok(&reply, "POST /api/v1/compile")?;
            return self.program_from(&v, t0.elapsed().as_secs_f64(), "compile");
        }
        if self.cfg.compiler.is_none() {
            return Err(format!(
                "{COMPILE_ASYNC_ENV}=1 needs {COMPILER_ENV} naming a finetune compiler \
                 (their async endpoint accepts explicit finetune compilers only)"
            ));
        }
        let reply = self.request(
            "POST",
            "/api/v1/compile/async",
            Some(&body),
            Duration::from_secs(30),
        )?;
        let mut v = Self::json_ok(&reply, "POST /api/v1/compile/async")?;
        let job = v
            .get("job_id")
            .and_then(Value::as_str)
            .filter(|j| !j.is_empty())
            .ok_or_else(|| format!("compile/async: no job_id in {v}"))?
            .to_string();
        loop {
            match v.get("status").and_then(Value::as_str).unwrap_or("") {
                "ready" => {
                    return self.program_from(&v, t0.elapsed().as_secs_f64(), "compile/async");
                }
                "failed" | "cancelled" => {
                    return Err(format!("compile job {job}: terminal {v}"));
                }
                _ => {}
            }
            if t0.elapsed() > self.cfg.compile_deadline {
                return Err(format!(
                    "compile job {job}: not ready after {:?} (last {v})",
                    self.cfg.compile_deadline
                ));
            }
            std::thread::sleep(self.cfg.poll);
            let reply = self.request(
                "GET",
                &format!("/api/v1/compile/{job}"),
                None,
                Duration::from_secs(10),
            )?;
            v = Self::json_ok(&reply, "GET /api/v1/compile/{job}")?;
        }
    }

    /// One hosted inference: `(output, server latency_ms if reported)`.
    ///
    /// # Errors
    /// Transport failure, non-2xx, or a reply without a string `output`.
    pub fn infer(&self, program_id: &str, input: &str) -> Result<(String, Option<f64>), String> {
        let body = json!({
            "program_id": program_id,
            "input": input,
            "temperature": 0.0,
            "max_tokens": MAX_TOKENS,
        });
        let reply = self.request("POST", "/api/v1/infer", Some(&body), Duration::from_secs(120))?;
        let v = Self::json_ok(&reply, "POST /api/v1/infer")?;
        let out = v
            .get("output")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("infer: no string output in {v}"))?
            .to_string();
        Ok((out, v.get("latency_ms").and_then(Value::as_f64)))
    }
}

// ── program-id cache ───────────────────────────────────────────────────

/// `(suite, compiler, BLAKE3(spec))` → program. A re-run never recompiles.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProgramCache {
    pub entries: BTreeMap<String, CompiledProgram>,
}

impl ProgramCache {
    #[must_use]
    pub fn key(suite: &str, compiler: Option<&str>, spec: &str) -> String {
        format!(
            "{suite}|{}|{}",
            compiler.unwrap_or("server-default"),
            blake3::hash(spec.as_bytes()).to_hex()
        )
    }

    /// Missing file = empty cache; a MALFORMED file is a loud error (an
    /// unreadable cache must never read as "nothing compiled yet" — that
    /// would silently recompile and burn the anonymous 20/h budget).
    ///
    /// # Errors
    /// Unreadable or unparseable file.
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("paw program cache {} is malformed: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("paw program cache {}: {e}", path.display())),
        }
    }

    /// Atomic write (temp sibling + rename).
    ///
    /// # Errors
    /// Filesystem failure.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
        }
        let tmp = path.with_extension(format!("tmp{}", std::process::id()));
        let text = serde_json::to_string_pretty(self).map_err(|e| format!("cache json: {e}"))?;
        std::fs::write(&tmp, text).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))
    }
}

// ── specs + the mapping law (pure) ─────────────────────────────────────

/// The suite's committed spec text, `None` when the suite has no spec (an
/// absence, never an error — only the specced suites have a PAW cell).
///
/// # Errors
/// A spec file that exists but cannot be read.
pub fn load_spec(dir: &Path, suite: &str) -> Result<Option<(PathBuf, String)>, String> {
    let path = dir.join(format!("{suite}.txt"));
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(Some((path, s))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("paw spec {}: {e}", path.display())),
    }
}

/// A question's option set: `(key, description)` in OUR option order.
#[must_use]
pub fn option_set(q: &SuiteQuestion) -> Vec<(String, Option<String>)> {
    match q.kind {
        QKind::Choice => match &q.criteria {
            Value::Object(m) => m
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().map(str::to_string)))
                .collect(),
            Value::Array(a) => a
                .iter()
                .map(|v| (v.as_str().map_or_else(|| v.to_string(), str::to_string), None))
                .collect(),
            _ => Vec::new(),
        },
        QKind::Score => match &q.criteria {
            Value::Array(a) => a
                .iter()
                .map(|v| (v.as_str().map_or_else(|| v.to_string(), str::to_string), None))
                .collect(),
            _ => Vec::new(),
        },
        QKind::Noul => vec![("false".to_string(), None), ("true".to_string(), None)],
    }
}

/// The spec must NAME every option verbatim, one per line — the guard
/// that keeps a committed spec from drifting off the suite's label set
/// (a drifted spec would measure a different task and read as PAW's fault).
///
/// # Errors
/// Lists every option key missing from the spec's lines.
pub fn check_spec_names_options(spec: &str, q: &SuiteQuestion) -> Result<(), String> {
    let lines: std::collections::HashSet<&str> = spec.lines().map(str::trim).collect();
    let missing: Vec<String> = option_set(q)
        .into_iter()
        .map(|(k, _)| k)
        .filter(|k| !lines.contains(k.as_str()))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "spec does not name {} option(s) verbatim on their own line: {missing:?}",
            missing.len()
        ))
    }
}

/// The parse outcome for one free-text answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parsed {
    /// Option index in OUR order; `quoted` = one surrounding quote pair was
    /// stripped to get there.
    Pick { idx: usize, quoted: bool },
    Refusal,
}

/// The mapping law (module docs): trim, strip ONE matched surrounding quote
/// pair, trim; exact key match, else unique exact description match; else
/// a refusal. Case-sensitive, never fuzzy, never guessed.
#[must_use]
pub fn parse_answer(output: &str, opts: &[(String, Option<String>)]) -> Parsed {
    let t = output.trim();
    let (t, quoted) = strip_one_quote_pair(t);
    let t = t.trim();
    if let Some(idx) = opts.iter().position(|(k, _)| k == t) {
        return Parsed::Pick { idx, quoted };
    }
    let mut hits = opts
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| d.as_deref() == Some(t));
    match (hits.next(), hits.next()) {
        (Some((idx, _)), None) => Parsed::Pick { idx, quoted },
        _ => Parsed::Refusal,
    }
}

fn strip_one_quote_pair(t: &str) -> (&str, bool) {
    for q in ['"', '\'', '`'] {
        if t.len() >= 2 && t.starts_with(q) && t.ends_with(q) {
            return (&t[1..t.len() - 1], true);
        }
    }
    (t, false)
}

/// The single text input a PAW function takes: a single-string-field state
/// object → that string (the natural product call — every specced suite is
/// shaped this way); a bare string → itself; anything else → compact JSON.
#[must_use]
pub fn render_input(state: &Value) -> String {
    match state {
        Value::String(s) => s.clone(),
        Value::Object(m) if m.len() == 1 => match m.values().next() {
            Some(Value::String(s)) => s.clone(),
            _ => state.to_string(),
        },
        _ => state.to_string(),
    }
}

// ── the lane row ───────────────────────────────────────────────────────

/// Maximum refusal samples kept per suite (diagnostics; truncated).
const REFUSAL_SAMPLES: usize = 8;

/// The PAW lane's per-suite row. No probability surface → no ECE / Brier /
/// NLL / AURC / confidence (disclosed divergence).
#[derive(Debug, Clone, Serialize)]
pub struct PawLaneResult {
    pub lane: &'static str,
    /// The compiler snapshot that built the program (or the configured
    /// compiler / `server-default` when the server did not report one).
    pub model: String,
    pub posture: String,
    pub program_id: String,
    pub spec_file: String,
    pub spec_blake3: String,
    pub compile_cache_hit: bool,
    pub compile_wall_s: f64,
    pub n_cases: usize,
    pub n_questions: usize,
    pub n_answered: usize,
    pub refusals: usize,
    /// Answers reached only after stripping one surrounding quote pair.
    pub quote_stripped: usize,
    /// Correct / every served question (a refusal counts wrong).
    pub accuracy: f64,
    /// Correct / answered (None when nothing parsed).
    pub answered_accuracy: Option<f64>,
    pub refusal_rate: f64,
    /// Score questions, answered subset only (a refusal has no level).
    pub score_mae_answered: Option<f64>,
    pub within_1_answered: Option<f64>,
    pub refusal_samples: Vec<String>,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub latency_tail_support: usize,
    /// Their reported `latency_ms` (server side), p50 — beside the client
    /// round-trip, which includes the network + the curl spawn.
    pub server_latency_p50_ms: Option<f64>,
    /// Observed-repeat check over the first 10 questions (hosted inference
    /// is not promised deterministic — recorded, never assumed).
    pub determinism_ok: Option<bool>,
    pub seconds: f64,
}

/// The TABLES.md line under a suite's table: refusals are the finding the
/// accuracy cell alone hides, so they print beside it, with the program
/// provenance (id, compile wall time, cache hit, posture).
#[must_use]
pub fn render_detail_line(r: &PawLaneResult) -> String {
    let opt4 = |v: Option<f64>| v.map_or("—".to_string(), |x| format!("{x:.4}"));
    let mut s = format!(
        "\n**PAW lane:** refusals **{}/{}** ({:.1}%) · answered-acc {} · quote-stripped {} · \
         program `{}` ({}, {}) · compile {:.1} s{} · server p50 {}",
        r.refusals,
        r.n_questions,
        r.refusal_rate * 100.0,
        opt4(r.answered_accuracy),
        r.quote_stripped,
        r.program_id,
        r.model,
        r.posture,
        r.compile_wall_s,
        if r.compile_cache_hit { " (cached)" } else { "" },
        r.server_latency_p50_ms
            .map_or("—".to_string(), |v| format!("{v:.1} ms")),
    );
    if let Some(mae) = r.score_mae_answered {
        s.push_str(&format!(
            " · score MAE (answered) {mae:.4} · within-1 {}",
            opt4(r.within_1_answered)
        ));
    }
    s.push('\n');
    if !r.refusal_samples.is_empty() {
        s.push_str(&format!("refusal samples: {:?}\n", r.refusal_samples));
    }
    s
}

/// Per-question tallies → the accuracy half of the row. Pure — pinned by
/// unit tests. `outputs[ci][qi]` is the raw text PAW returned.
#[derive(Debug, Clone, PartialEq)]
pub struct Tally {
    pub n: usize,
    pub answered: usize,
    pub correct: usize,
    pub refusals: usize,
    pub quote_stripped: usize,
    pub score_mae: Option<f64>,
    pub within_1: Option<f64>,
    pub refusal_samples: Vec<String>,
}

#[must_use]
pub fn tally(cases: &[SuiteCase], outputs: &[Vec<String>]) -> Tally {
    let mut t = Tally {
        n: 0,
        answered: 0,
        correct: 0,
        refusals: 0,
        quote_stripped: 0,
        score_mae: None,
        within_1: None,
        refusal_samples: Vec::new(),
    };
    let (mut msum, mut wsum, mut mn) = (0.0f64, 0.0f64, 0usize);
    for (case, outs) in cases.iter().zip(outputs) {
        for ((q, gold), out) in case.questions.iter().zip(&case.gold).zip(outs) {
            t.n += 1;
            match parse_answer(out, &option_set(q)) {
                Parsed::Pick { idx, quoted } => {
                    t.answered += 1;
                    t.quote_stripped += usize::from(quoted);
                    t.correct += usize::from(idx == gold.idx);
                    if let Some(gs) = gold.gold_score {
                        let err = (idx as f64 - gs).abs();
                        msum += err;
                        wsum += f64::from(u8::from(err <= 1.0));
                        mn += 1;
                    }
                }
                Parsed::Refusal => {
                    t.refusals += 1;
                    if t.refusal_samples.len() < REFUSAL_SAMPLES {
                        t.refusal_samples.push(out.chars().take(80).collect());
                    }
                }
            }
        }
    }
    if mn > 0 {
        t.score_mae = Some(msum / mn as f64);
        t.within_1 = Some(wsum / mn as f64);
    }
    t
}

/// The trim law (the gliner/agentjev lanes' law): the SAME question cap as
/// the laya lanes, so a capped PAW row never reads against a full-N one.
#[must_use]
pub fn trimmed(cases: &[SuiteCase], max_questions: usize) -> &[SuiteCase] {
    if max_questions == 0 {
        return cases;
    }
    let mut n = 0usize;
    for (i, c) in cases.iter().enumerate() {
        n += c.questions.len();
        if n >= max_questions {
            return &cases[..=i];
        }
    }
    cases
}

/// Nearest-rank p50 / p99 / tail support over µs samples (the repo-family
/// percentile law — the same formula as the runner's).
fn percentiles_ms(durs_us: &[u64]) -> (f64, f64, usize) {
    let mut d = durs_us.to_vec();
    d.sort_unstable();
    let n = d.len();
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    let idx99 = (n * 99).div_ceil(100) - 1;
    (d[n / 2] as f64 / 1e3, d[idx99] as f64 / 1e3, n - idx99)
}

/// Run the lane over one suite. `Ok(None)` = the suite has no committed
/// spec (an honest absence, printed by the caller). Compiles at most once
/// per `(suite, compiler, spec)` across runs (the cache).
///
/// # Errors
/// A spec that fails the option-naming guard, a malformed cache, or any
/// transport / API failure — loud, never a guessed row.
pub fn run_suite(
    client: &PawClient,
    suite: &Suite,
    max_questions: usize,
) -> Result<Option<PawLaneResult>, String> {
    let cfg = client.config();
    let Some((spec_path, spec)) = load_spec(&cfg.specs_dir, suite.name)? else {
        return Ok(None);
    };
    let cases = trimmed(&suite.cases, max_questions);
    // Every served question must be named by the spec (the drift guard).
    for case in cases {
        for q in &case.questions {
            check_spec_names_options(&spec, q)
                .map_err(|e| format!("{}: {e}", spec_path.display()))?;
        }
    }
    let t_start = Instant::now();
    let key = ProgramCache::key(suite.name, cfg.compiler.as_deref(), &spec);
    let mut cache = ProgramCache::load(&cfg.cache_file)?;
    let (program, hit) = match cache.entries.get(&key) {
        Some(p) => (p.clone(), true),
        None => {
            eprintln!(
                "    [paw] compiling {} ({} posture{}) …",
                spec_path.display(),
                cfg.posture(),
                if cfg.api_key.is_none() {
                    " — PAW_API_KEY unset: anonymous tier, program is PUBLIC on their hub"
                } else {
                    ""
                }
            );
            let p = client.compile(&spec)?;
            cache.entries.insert(key.clone(), p.clone());
            cache.save(&cfg.cache_file)?;
            (p, false)
        }
    };
    eprintln!(
        "    [paw] program {} ({}; compile {:.1}s{})",
        program.program_id,
        program.compiler_snapshot.as_deref().unwrap_or("server-default"),
        program.compile_wall_s,
        if hit { ", CACHED — not recompiled" } else { "" }
    );

    // WARMUP (the clm lane's cold-start law): one fixed throwaway input
    // absorbs their adapter-load cold path (measured ~3 s on first call).
    client
        .infer(&program.program_id, "warmup: discarded, not a measured case")
        .map_err(|e| format!("paw warmup: {e}"))?;

    let mut outputs: Vec<Vec<String>> = Vec::with_capacity(cases.len());
    let mut durs_us: Vec<u64> = Vec::new();
    let mut server_ms: Vec<u64> = Vec::new();
    let mut determinism_ok: Option<bool> = None;
    let mut served = 0usize;
    for (ci, case) in cases.iter().enumerate() {
        let input = render_input(&case.state);
        let mut outs = Vec::with_capacity(case.questions.len());
        // One PAW program answers one question shape per suite: every
        // question of the case gets the same single-input call.
        for _q in &case.questions {
            let t0 = Instant::now();
            let (out, srv) = client
                .infer(&program.program_id, &input)
                .map_err(|e| format!("paw infer (case {ci}): {e}"))?;
            durs_us.push(t0.elapsed().as_micros() as u64);
            if let Some(s) = srv {
                server_ms.push((s * 1e3) as u64);
            }
            if served < 10 {
                let (again, _) = client
                    .infer(&program.program_id, &input)
                    .map_err(|e| format!("paw determinism rerun (case {ci}): {e}"))?;
                let ok = determinism_ok.get_or_insert(true);
                *ok &= again == out;
            }
            served += 1;
            outs.push(out);
        }
        outputs.push(outs);
    }

    let t = tally(cases, &outputs);
    let (p50, p99, support) = percentiles_ms(&durs_us);
    let server_p50 = (!server_ms.is_empty()).then(|| percentiles_ms(&server_ms).0);
    let model = program
        .compiler_snapshot
        .clone()
        .or_else(|| cfg.compiler.clone())
        .unwrap_or_else(|| "server-default".to_string());
    Ok(Some(PawLaneResult {
        lane: "paw",
        model,
        posture: program.posture.clone(),
        program_id: program.program_id.clone(),
        spec_file: spec_path.display().to_string(),
        spec_blake3: blake3::hash(spec.as_bytes()).to_hex().to_string(),
        compile_cache_hit: hit,
        compile_wall_s: program.compile_wall_s,
        n_cases: cases.len(),
        n_questions: t.n,
        n_answered: t.answered,
        refusals: t.refusals,
        quote_stripped: t.quote_stripped,
        accuracy: if t.n == 0 { 0.0 } else { t.correct as f64 / t.n as f64 },
        answered_accuracy: (t.answered > 0).then(|| t.correct as f64 / t.answered as f64),
        refusal_rate: if t.n == 0 { 0.0 } else { t.refusals as f64 / t.n as f64 },
        score_mae_answered: t.score_mae,
        within_1_answered: t.within_1,
        refusal_samples: t.refusal_samples,
        latency_p50_ms: p50,
        latency_p99_ms: p99,
        latency_tail_support: support,
        server_latency_p50_ms: server_p50,
        determinism_ok,
        seconds: t_start.elapsed().as_secs_f64(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{build_ag_news, build_emotion, build_sst5};
    use std::sync::{Arc, Mutex};

    fn envelope(rows: &[(&str, i64)]) -> Value {
        json!({
            "features": [],
            "rows": rows.iter().enumerate()
                .map(|(i, (t, l))| json!({"row_idx": i, "row": {"text": t, "label": l}}))
                .collect::<Vec<_>>(),
        })
    }

    #[test]
    fn parse_law_exact_quoted_description_and_refusals() {
        let opts: Vec<(String, Option<String>)> = vec![
            ("world".into(), Some("world news".into())),
            ("sports".into(), Some("sports".into())),
            ("sci_tech".into(), Some("science and technology".into())),
        ];
        assert_eq!(parse_answer("sports", &opts), Parsed::Pick { idx: 1, quoted: false });
        assert_eq!(parse_answer("  sci_tech\n", &opts), Parsed::Pick { idx: 2, quoted: false });
        assert_eq!(parse_answer("\"world\"", &opts), Parsed::Pick { idx: 0, quoted: true });
        assert_eq!(parse_answer("'world'", &opts), Parsed::Pick { idx: 0, quoted: true });
        // description match (unique), key wins where both coincide
        assert_eq!(
            parse_answer("science and technology", &opts),
            Parsed::Pick { idx: 2, quoted: false }
        );
        // never guessed: case, punctuation, prose, two labels, nested quotes
        for bad in ["World", "world.", "It is world", "world, sports", "\"\"world\"\"", "", "\""] {
            assert_eq!(parse_answer(bad, &opts), Parsed::Refusal, "{bad:?}");
        }
        // an ambiguous description is a refusal, never a pick
        let dup: Vec<(String, Option<String>)> =
            vec![("a".into(), Some("x".into())), ("b".into(), Some("x".into()))];
        assert_eq!(parse_answer("x", &dup), Parsed::Refusal);
    }

    #[test]
    fn noul_and_score_option_sets() {
        let noul = SuiteQuestion {
            qid: "q".into(),
            kind: QKind::Noul,
            instructions: String::new(),
            criteria: Value::Null,
        };
        assert_eq!(parse_answer("true", &option_set(&noul)), Parsed::Pick { idx: 1, quoted: false });
        assert_eq!(parse_answer("yes", &option_set(&noul)), Parsed::Refusal);
        let sst5 = build_sst5(&envelope(&[("fine", 2)]), 0);
        let opts = option_set(&sst5.cases[0].questions[0]);
        assert_eq!(opts.len(), 5);
        assert_eq!(parse_answer("\"very positive\"", &opts), Parsed::Pick { idx: 4, quoted: true });
    }

    #[test]
    fn tally_counts_refusals_as_wrong_and_scores_answered_only() {
        let s = build_sst5(&envelope(&[("a", 0), ("b", 4), ("c", 2)]), 0);
        let outs = vec![
            vec!["very negative".to_string()],
            vec!["\"positive\"".to_string()],
            vec!["I think neutral".to_string()],
        ];
        let t = tally(&s.cases, &outs);
        assert_eq!((t.n, t.answered, t.correct, t.refusals, t.quote_stripped), (3, 2, 1, 1, 1));
        assert_eq!(t.score_mae, Some(0.5));
        assert_eq!(t.within_1, Some(1.0));
        assert_eq!(t.refusal_samples, vec!["I think neutral".to_string()]);
    }

    #[test]
    fn committed_specs_name_every_option_verbatim() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SPECS_DIR);
        let suites = [
            build_ag_news(&envelope(&[("x", 0)]), 0),
            build_emotion(&envelope(&[("x", 0)]), 0),
            build_sst5(&envelope(&[("x", 0)]), 0),
        ];
        for s in &suites {
            let (_, spec) = load_spec(&dir, s.name).unwrap().expect("spec committed");
            check_spec_names_options(&spec, &s.cases[0].questions[0])
                .unwrap_or_else(|e| panic!("{}: {e}", s.name));
        }
        // banking77's key set derives from the data (77 sorted label_text,
        // underscores → spaces): the spec carries exactly 77 label lines.
        let (_, spec) = load_spec(&dir, "banking77").unwrap().expect("spec committed");
        let labels: Vec<&str> = spec
            .split("Labels:\n")
            .nth(1)
            .expect("Labels: block")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        assert_eq!(labels.len(), 77);
        assert!(labels.contains(&"Refund not showing up"));
        assert!(labels.contains(&"reverted card payment?"));
        // a drifted spec is caught by the guard
        let q = &suites[0].cases[0].questions[0];
        assert!(check_spec_names_options("world\nsports\nbusiness\n", q).is_err());
        assert_eq!(load_spec(&dir, "no_such_suite").unwrap(), None);
    }

    #[test]
    fn input_rendering_and_trim_law() {
        assert_eq!(render_input(&json!({"text": "hi"})), "hi");
        assert_eq!(render_input(&json!("raw")), "raw");
        assert_eq!(render_input(&json!({"a": "1", "b": "2"})), r#"{"a":"1","b":"2"}"#);
        let s = build_ag_news(&envelope(&[("a", 0), ("b", 1), ("c", 2)]), 0);
        assert_eq!(trimmed(&s.cases, 0).len(), 3);
        assert_eq!(trimmed(&s.cases, 2).len(), 2);
        assert_eq!(trimmed(&s.cases, 99).len(), 3);
    }

    #[test]
    fn curl_include_parser_skips_interim_and_lowercases_headers() {
        let raw = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 429 Too Many\r\nRetry-After: 7\r\n\r\n{\"x\":1}";
        let r = parse_curl_include(raw).unwrap();
        assert_eq!(r.status, 429);
        assert_eq!(r.headers.get("retry-after").map(String::as_str), Some("7"));
        assert_eq!(r.body, b"{\"x\":1}");
        assert!(parse_curl_include(b"garbage").is_err());
    }

    #[test]
    fn malformed_cache_is_loud_missing_cache_is_empty() {
        let dir = std::env::temp_dir().join(format!("reflex_paw_cache_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.json");
        std::fs::write(&bad, "{not json").unwrap();
        assert!(ProgramCache::load(&bad).unwrap_err().contains("malformed"));
        assert!(ProgramCache::load(&dir.join("absent.json")).unwrap().entries.is_empty());
        // the key is (suite, compiler, BLAKE3(spec)) — any axis moves it
        let k = ProgramCache::key("sst5", None, "spec");
        assert_ne!(k, ProgramCache::key("sst5", Some("paw-ft-bs48"), "spec"));
        assert_ne!(k, ProgramCache::key("sst5", None, "spec2"));
        assert_ne!(k, ProgramCache::key("emotion", None, "spec"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// One recorded request at the stub server.
    #[derive(Debug, Clone)]
    struct Seen {
        line: String,
        headers: BTreeMap<String, String>,
        body: Value,
    }

    /// A stub PAW server: compile → ready program; infer → a scripted
    /// output keyed by input text. Serves until the process exits.
    fn stub_server(outputs: BTreeMap<String, String>) -> (u16, Arc<Mutex<Vec<Seen>>>) {
        use std::io::BufRead;
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("ephemeral bind");
        let port = listener.local_addr().unwrap().port();
        let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let mut reader = std::io::BufReader::new(stream.try_clone().expect("clone"));
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                let mut headers = BTreeMap::new();
                loop {
                    let mut h = String::new();
                    reader.read_line(&mut h).expect("header");
                    let h = h.trim_end();
                    if h.is_empty() {
                        break;
                    }
                    if let Some((k, v)) = h.split_once(':') {
                        headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
                    }
                }
                let len: usize = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
                let mut body = vec![0u8; len];
                reader.read_exact(&mut body).expect("body");
                let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                let (status, resp) = if line.starts_with("POST /api/v1/compile ") {
                    (
                        202,
                        json!({"job_id": "j1", "status": "ready",
                               "program_id": "abcdef0123456789abcd",
                               "compiler_snapshot": "paw-4b-qwen3-0.6b-20260407"}),
                    )
                } else if line.starts_with("POST /api/v1/infer ") {
                    let input = body["input"].as_str().unwrap_or("");
                    let out = outputs.get(input).cloned().unwrap_or_else(|| "world".into());
                    (200, json!({"output": out, "latency_ms": 12.5}))
                } else {
                    (404, json!({"detail": "not found"}))
                };
                log.lock().unwrap().push(Seen { line: line.trim_end().to_string(), headers, body });
                let resp = resp.to_string();
                let mut stream = stream;
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n\
                         Content-Length: {}\r\nConnection: close\r\n\r\n{resp}",
                        resp.len()
                    )
                    .as_bytes(),
                );
            }
        });
        (port, seen)
    }

    fn curl_available() -> bool {
        Command::new("curl").arg("--version").output().is_ok_and(|o| o.status.success())
    }

    fn test_cfg(port: u16, tag: &str, key: Option<&str>) -> PawConfig {
        PawConfig {
            api_url: format!("http://127.0.0.1:{port}"),
            api_key: key.map(str::to_string),
            compiler: None,
            compile_async: false,
            specs_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SPECS_DIR),
            cache_file: std::env::temp_dir()
                .join(format!("reflex_paw_{tag}_{}", std::process::id()))
                .join("programs.json"),
            curl: "curl".to_string(),
            poll: Duration::from_millis(10),
            compile_deadline: Duration::from_secs(30),
        }
    }

    /// The Issue 033 wire pin (the clm `stub_http_round_trip_pins_the_request_wire`
    /// precedent): compile + infer request shapes, anonymous posture (no key,
    /// public program), refusal accounting, and the cache hit on re-run
    /// (zero recompiles).
    #[test]
    fn stub_http_round_trip_pins_the_paw_wire_and_the_cache() {
        if !curl_available() {
            eprintln!("SKIP (loud): curl not on PATH — the PAW lane's transport is absent");
            return;
        }
        let outputs: BTreeMap<String, String> = [
            ("rates fall", "business"),
            ("cup final", "\"sports\""),
            ("new chip", "Science!"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
        let (port, seen) = stub_server(outputs);
        let suite = build_ag_news(&envelope(&[("rates fall", 2), ("cup final", 1), ("new chip", 3)]), 0);
        let cfg = test_cfg(port, "anon", None);
        let client = PawClient::new(cfg.clone()).unwrap();
        assert_eq!(cfg.posture(), "hosted-anonymous");

        let r = run_suite(&client, &suite, 0).unwrap().expect("specced suite runs");
        assert_eq!(r.program_id, "abcdef0123456789abcd");
        assert_eq!(r.model, "paw-4b-qwen3-0.6b-20260407");
        assert_eq!(r.posture, "hosted-anonymous");
        assert!(!r.compile_cache_hit);
        assert_eq!((r.n_questions, r.n_answered, r.refusals, r.quote_stripped), (3, 2, 1, 1));
        assert!((r.accuracy - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(r.answered_accuracy, Some(1.0));
        assert_eq!(r.refusal_samples, vec!["Science!".to_string()]);
        assert_eq!(r.determinism_ok, Some(true));
        assert_eq!(r.server_latency_p50_ms, Some(12.5));

        let log = seen.lock().unwrap().clone();
        let compiles: Vec<&Seen> = log.iter().filter(|s| s.line.starts_with("POST /api/v1/compile ")).collect();
        assert_eq!(compiles.len(), 1);
        let spec = std::fs::read_to_string(cfg.specs_dir.join("ag_news.txt")).unwrap();
        assert_eq!(compiles[0].body["spec"], json!(spec));
        assert_eq!(compiles[0].body["public"], json!(true), "anonymous programs are public");
        assert!(compiles[0].body.get("compiler").is_none());
        assert!(log.iter().all(|s| !s.headers.contains_key("x-api-key")));
        let infers: Vec<&Seen> = log.iter().filter(|s| s.line.starts_with("POST /api/v1/infer ")).collect();
        // warmup + 3 measured + 3 determinism re-asks
        assert_eq!(infers.len(), 7);
        assert_eq!(infers[1].body["program_id"], "abcdef0123456789abcd");
        assert_eq!(infers[1].body["input"], "rates fall");
        assert_eq!(infers[1].body["temperature"], json!(0.0));
        assert_eq!(infers[1].body["max_tokens"], json!(MAX_TOKENS));

        // Re-run: cache hit, NO recompile.
        let before = seen.lock().unwrap().len();
        let r2 = run_suite(&client, &suite, 0).unwrap().unwrap();
        assert!(r2.compile_cache_hit);
        let after = seen.lock().unwrap().clone();
        assert!(after[before..].iter().all(|s| !s.line.starts_with("POST /api/v1/compile")));

        // A suite without a spec is an absence, never a request.
        let mut nospec = suite.clone();
        nospec.name = "no_such_suite";
        let n = seen.lock().unwrap().len();
        assert!(run_suite(&client, &nospec, 0).unwrap().is_none());
        assert_eq!(seen.lock().unwrap().len(), n);
        let _ = std::fs::remove_dir_all(cfg.cache_file.parent().unwrap());
    }

    /// With a key: `X-API-Key` rides every request (from the 0600 header
    /// file, never argv) and the program compiles private.
    #[test]
    fn stub_http_authenticated_posture_sends_the_key_and_compiles_private() {
        if !curl_available() {
            eprintln!("SKIP (loud): curl not on PATH — the PAW lane's transport is absent");
            return;
        }
        let (port, seen) = stub_server(BTreeMap::new());
        let suite = build_emotion(&envelope(&[("i feel great", 1)]), 0);
        let cfg = test_cfg(port, "auth", Some("paw_sk_test"));
        let client = PawClient::new(cfg.clone()).unwrap();
        let key_path = client.key_file.as_ref().unwrap().0.clone();
        let r = run_suite(&client, &suite, 0).unwrap().unwrap();
        assert_eq!(r.posture, "hosted-authenticated");
        // stub answers "world" — not an emotion label → a refusal
        assert_eq!((r.n_questions, r.refusals), (1, 1));
        let log = seen.lock().unwrap().clone();
        assert!(!log.is_empty());
        assert!(log.iter().all(|s| s.headers.get("x-api-key").map(String::as_str) == Some("paw_sk_test")));
        let compile = log.iter().find(|s| s.line.starts_with("POST /api/v1/compile ")).unwrap();
        assert_eq!(compile.body["public"], json!(false));
        drop(client);
        assert!(!key_path.exists(), "the key header file is removed on drop");
        let _ = std::fs::remove_dir_all(cfg.cache_file.parent().unwrap());
    }
}
