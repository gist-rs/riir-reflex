//! The PAW Posture B lane (`.issues/033`) — their LOCAL runtime as a
//! subprocess oracle (`scripts/paw_local_lane.py`, the gliner-lane
//! precedent): `paw.function(program_id)` runs the compiled bundle through
//! their llama.cpp interpreter (auto CUDA/CPU), greedy by construction.
//! Their stack serves, our Rust measures; comparison lane, never a product
//! lane.
//!
//! Everything the hosted lane ([`super::paw`]) established is REUSED, not
//! duplicated: the committed specs, the drift guard, the cache + its key,
//! the input rendering, the mapping law, the tally, and the row type.
//! Only the transport swaps — one Python subprocess per suite speaking the
//! line protocol:
//!
//! ```text
//! handshake: {"ready": true, "lane": "paw-local", "runtime": ..., "device": ...}
//! request:   {"program_id": "...", "input": "..."}   (one JSON line)
//! response:  {"output": "..."}  |  {"error": "..."}   (one JSON line)
//! ```
//!
//! **The lane NEVER compiles.** The program id comes from the hosted
//! lane's cache (`.raw/paw/programs.json`, keyed
//! `(suite, compiler, BLAKE3(spec))`) — the SAME compiled artifact the
//! hosted cells measured, so a local-vs-hosted delta isolates the runtime
//! posture (server inference vs local llama.cpp) instead of re-testing
//! the compile. A missing cache entry is a loud error pointing at
//! `--paw`, never a silent skip.
//!
//! Determinism: `paw.function` exposes no sampling knobs and the
//! feasibility probe measured repeat-stable outputs — the claim is
//! re-verified per run by the observed-repeat check (first 10 served
//! questions), recorded in the row, never assumed.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use serde_json::json;

use super::paw::{self, CompiledProgram, PawConfig, PawLaneResult, ProgramCache};
use crate::harness::suites::Suite;

/// The venv interpreter that has `programasweights` installed (the
/// `.raw/paw-env` lane venv on a bench box; `python3`/`python` default).
pub const PYTHON_ENV: &str = "PAW_LOCAL_PYTHON";
/// The oracle script (repo-relative when the harness runs from the root).
pub const SCRIPT_ENV: &str = "PAW_LOCAL_SCRIPT";
pub const DEFAULT_SCRIPT: &str = "scripts/paw_local_lane.py";
/// Default interpreter when [`PYTHON_ENV`] is unset (the gliner lane's
/// default spelling; a Windows bench box sets the venv path explicitly).
pub const DEFAULT_PYTHON: &str = "python3";

/// One oracle process. Stderr inherits: their loader's download/load
/// chatter stays visible in the run log.
pub struct PawLocalSession {
    child: Option<Child>,
    stdin: Box<dyn Write>,
    reader: BufReader<Box<dyn Read>>,
}

impl PawLocalSession {
    /// Spawn `python <script>` and read the handshake.
    ///
    /// # Errors
    /// A missing script, a failed spawn, a dead stream, or a handshake
    /// that is not `{"ready": true, "lane": "paw-local"}` — all loud.
    pub fn spawn(python: &str, script: &Path) -> Result<Self, String> {
        if !script.exists() {
            return Err(format!(
                "paw-local lane script not found at {} — run from the repo root or \
                 set {SCRIPT_ENV} (the lane is opt-in measurement tooling; the venv \
                 needs programasweights, e.g. `uv venv .raw/paw-env && uv pip install \
                 --python .raw/paw-env/Scripts/python.exe programasweights`)",
                script.display()
            ));
        }
        let mut child = Command::new(python)
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // stderr inherits: their loader's warnings stay visible.
            .spawn()
            .map_err(|e| format!("spawn {python} {} ({PYTHON_ENV}): {e}", script.display()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "paw-local oracle stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "paw-local oracle stdout unavailable".to_string())?;
        let mut session = Self {
            child: Some(child),
            stdin: Box::new(stdin),
            reader: BufReader::new(Box::new(stdout)),
        };
        let mut line = String::new();
        session.read_line(&mut line)?;
        let (runtime, device) = parse_handshake(&line)?;
        eprintln!("    [paw-local] oracle up: {runtime} on {device}");
        Ok(session)
    }

    /// Test seam: a session over injected channels (the protocol logic is
    /// identical; only the process is fake).
    #[cfg(test)]
    fn from_channels(input: Box<dyn Read>, output: Box<dyn Write>) -> Result<Self, String> {
        Ok(Self {
            child: None,
            stdin: output,
            reader: BufReader::new(input),
        })
    }

    fn read_line(&mut self, buf: &mut String) -> Result<(), String> {
        let n = self
            .reader
            .read_line(buf)
            .map_err(|e| format!("paw-local stdout: {e}"))?;
        if n == 0 {
            return Err(
                "paw-local oracle stream ended early — the subprocess died; its \
                 stderr above names the cause"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// One inference round-trip: `{"program_id", "input"}` → the free-text
    /// output. A `{"error": ...}` response or a malformed line fails loud.
    ///
    /// # Errors
    /// The protocol's own error channel, a malformed response, or a dead
    /// stream.
    pub fn infer(&mut self, program_id: &str, input: &str) -> Result<String, String> {
        let line = json!({"program_id": program_id, "input": input}).to_string();
        writeln!(self.stdin, "{line}").map_err(|e| format!("paw-local stdin: {e}"))?;
        self.stdin.flush().map_err(|e| format!("paw-local stdin flush: {e}"))?;
        let mut resp = String::new();
        self.read_line(&mut resp)?;
        parse_response(resp.trim())
    }
}

impl Drop for PawLocalSession {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            // The protocol has no shutdown line; closing stdin ends the
            // oracle's read loop. ⚠ KILL-THEN-WAIT, measured: a uv venv
            // `python.exe` is a TRAMPOLINE that spawns the real interpreter
            // as a child while keeping its own inherited copy of this pipe's
            // write handle — a bare `wait()` after closing stdin deadlocks
            // (trampoline waits for the interpreter; the interpreter waits
            // for stdin EOF; EOF needs every write handle closed). Killing
            // the trampoline closes its copies, the interpreter EOFs and
            // exits on its own, and `wait()` reaps what we spawned.
            let _ = child.stdin.take();
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// The handshake: `ready` must be true and the lane must name itself —
/// a wrong script on `PAW_LOCAL_SCRIPT` otherwise reads as protocol noise
/// three lines later. Returns `(runtime, device)` for the run log.
///
/// # Errors
/// Malformed JSON, `ready != true`, or a foreign `lane`.
fn parse_handshake(line: &str) -> Result<(String, String), String> {
    let v: serde_json::Value = serde_json::from_str(line.trim())
        .map_err(|e| format!("paw-local handshake: {e} (got: {line})"))?;
    if v.get("ready").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(format!("paw-local handshake: expected {{\"ready\":true}}, got {line}"));
    }
    let lane = v.get("lane").and_then(serde_json::Value::as_str).unwrap_or("");
    if lane != "paw-local" {
        return Err(format!(
            "paw-local handshake: lane {lane:?} is not \"paw-local\" — \
             {SCRIPT_ENV} points at the wrong script?"
        ));
    }
    let g = |k: &str| {
        v.get(k)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("?")
            .to_string()
    };
    Ok((g("runtime"), g("device")))
}

/// One response line → the free-text output; their error channel becomes
/// ours.
///
/// # Errors
/// A `{"error": ...}` response, a missing `output`, or malformed JSON.
fn parse_response(line: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(line).map_err(|e| format!("paw-local response: {e} (got: {line})"))?;
    if let Some(err) = v.get("error").and_then(serde_json::Value::as_str) {
        return Err(format!("paw-local oracle: {err}"));
    }
    v.get("output")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("paw-local response: no output field (got: {line})"))
}

/// Run the lane over one suite. `Ok(None)` = the suite has no committed
/// spec (an honest absence, printed by the caller). Reads the program id
/// from the hosted lane's cache — compilation is the HOSTED lane's job,
/// and this lane refuses to reach their compile API at all.
///
/// # Errors
/// A spec that fails the option-naming guard, a malformed cache, a
/// missing cached program (run `--paw` once), or any protocol failure —
/// loud, never a guessed row.
pub fn run_suite(
    session: &mut PawLocalSession,
    cfg: &PawConfig,
    suite: &Suite,
    max_questions: usize,
) -> Result<Option<PawLaneResult>, String> {
    let cases = paw::trimmed(&suite.cases, max_questions);
    // The hosted lane's per-shape resolution, shared verbatim: the per-qid
    // spec wins over suite-wide; no spec at all is an honest absence;
    // partial coverage is refused.
    let Some(resolved) = paw::resolve_shapes(&cfg.specs_dir, suite.name, cases)? else {
        return Ok(None);
    };
    let shape_specs = resolved.specs;
    let qid_shape = resolved.qid_shape;
    let cache = ProgramCache::load(&cfg.cache_file)?;
    // One CACHED program per shape — the local lane never compiles.
    // Compiler selection per shape: `PAW_COMPILER` pins the exact key;
    // unset, a shape with exactly ONE cached program uses it (printed
    // loud — the row's `model` still stamps the program's own
    // compiler_snapshot); ambiguity refuses, naming the choices. A silent
    // default would measure a different tier than the caller meant. The
    // per-shape prefix keeps qid keys from reading as sibling compilers:
    // a per-qid key is `suite|qid|compiler|hash` (4 segments), a
    // suite-wide key is `suite|compiler|hash` (3).
    let mut programs: Vec<CompiledProgram> = Vec::with_capacity(shape_specs.len());
    for (_path, spec, per_q, label) in &shape_specs {
        let key: String = if *per_q {
            match cfg.compiler.as_deref() {
                Some(c) => ProgramCache::key_for_qid(suite.name, label, Some(c), spec),
                None => {
                    let prefix = format!("{}|{}|", suite.name, label);
                    auto_select(&cache, &prefix, 2, suite.name, label, &cfg.cache_file)?
                }
            }
        } else {
            match cfg.compiler.as_deref() {
                Some(c) => ProgramCache::key(suite.name, Some(c), spec),
                None => {
                    let prefix = format!("{}|", suite.name);
                    auto_select(&cache, &prefix, 1, suite.name, label, &cfg.cache_file)?
                }
            }
        };
        let program = cache.entries.get(&key).ok_or_else(|| {
            format!(
                "no cached program for {} (shape {label}, compiler {}) in {} — the local \
                 lane never compiles; run the hosted lane once (--paw) to create the \
                 artifact, then re-run --paw-local",
                suite.name,
                cfg.compiler.as_deref().unwrap_or("server-default"),
                cfg.cache_file.display()
            )
        })?;
        eprintln!(
            "    [paw-local] program {} (shape {label}; {}; hosted compile {:.1}s) — same \
             artifact the hosted posture measured{}",
            program.program_id,
            program.compiler_snapshot.as_deref().unwrap_or("server-default"),
            program.compile_wall_s,
            if cfg.compiler.is_none() {
                " [PAW_COMPILER unset — the shape's only cached program]"
            } else {
                ""
            },
        );
        programs.push(program.clone());
    }

    let t_start = Instant::now();

    // WARMUP (the hosted lane's cold-start law, one layer down): the first
    // `paw.function(program_id)` per program may download the bundle + base
    // — the preload script (scripts/paw_preload.py) usually absorbs that,
    // and this warmup absorbs whatever is left, outside the measured loop.
    for program in &programs {
        session
            .infer(&program.program_id, "warmup: discarded, not a measured case")
            .map_err(|e| format!("paw-local warmup: {e}"))?;
    }

    let mut outputs: Vec<Vec<String>> = Vec::with_capacity(cases.len());
    let mut durs_us: Vec<u64> = Vec::new();
    let mut determinism_ok: Option<bool> = None;
    let mut served = 0usize;
    for (ci, case) in cases.iter().enumerate() {
        let input = paw::render_input(&case.state);
        let mut outs = Vec::with_capacity(case.questions.len());
        // Each question is answered by ITS shape's program (the hosted
        // lane's law, verbatim): the multi-shape suites get one compiled
        // task per question kind, not one text parsed two ways.
        for q in &case.questions {
            let program_id = &programs[qid_shape[&q.qid]].program_id;
            let t0 = Instant::now();
            let out = session
                .infer(program_id, &input)
                .map_err(|e| format!("paw-local infer (case {ci}): {e}"))?;
            durs_us.push(t0.elapsed().as_micros() as u64);
            if served < 10 {
                // The observed-repeat check (the hosted lane's law): the
                // rerun is UNtimed — only first calls feed the latency
                // columns, so the columns stay comparable across postures.
                let again = session
                    .infer(program_id, &input)
                    .map_err(|e| format!("paw-local determinism rerun (case {ci}): {e}"))?;
                let ok = determinism_ok.get_or_insert(true);
                *ok &= again == out;
            }
            served += 1;
            outs.push(out);
        }
        outputs.push(outs);
    }

    let t = paw::tally(cases, &outputs);
    let (p50, p99, support) = paw::percentiles_ms(&durs_us);
    let model = programs
        .iter()
        .find_map(|p| p.compiler_snapshot.clone())
        .or_else(|| cfg.compiler.clone())
        .unwrap_or_else(|| "server-default".to_string());
    // Multi-shape rows disclose every program: the ids / spec paths /
    // digests join in shape order, the compile wall sums (the total
    // hosted cost). The cache-hit flag is always true — a missing entry
    // is the loud error above, never a compile.
    Ok(Some(PawLaneResult {
        lane: "paw-local",
        model,
        posture: "local-subprocess".to_string(),
        program_id: programs
            .iter()
            .map(|p| p.program_id.clone())
            .collect::<Vec<_>>()
            .join(","),
        spec_file: shape_specs
            .iter()
            .map(|(p, _, _, _)| p.display().to_string())
            .collect::<Vec<_>>()
            .join(","),
        spec_blake3: shape_specs
            .iter()
            .map(|(_, s, _, _)| blake3::hash(s.as_bytes()).to_hex())
            .collect::<Vec<_>>()
            .join(","),
        compile_cache_hit: true,
        compile_wall_s: programs.iter().map(|p| p.compile_wall_s).sum(),
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
        // Their server latency_ms is a hosted-response field; the local
        // runtime has no such surface.
        server_latency_p50_ms: None,
        determinism_ok,
        seconds: t_start.elapsed().as_secs_f64(),
    }))
}

/// The unset-`PAW_COMPILER` auto-select, scoped to ONE shape: the keys
/// under `prefix` whose segment at `compiler_seg` is the compiler name
/// (a per-qid key is `suite|qid|compiler|hash` — segment 2; a suite-wide
/// key is `suite|compiler|hash` — segment 1). Exactly one match uses it
/// (the chosen compiler is returned for the loud line); none is the loud
/// never-compiles remedy; more is an ambiguity refusal naming the choices.
fn auto_select(
    cache: &ProgramCache,
    prefix: &str,
    compiler_seg: usize,
    suite_name: &str,
    shape_label: &str,
    cache_file: &Path,
) -> Result<String, String> {
    let matches: Vec<(String, String)> = cache
        .entries
        .keys()
        .filter(|k| k.starts_with(prefix))
        .filter(|k| k.split('|').count() == compiler_seg + 2)
        .map(|k| {
            let compiler = k
                .split('|')
                .nth(compiler_seg)
                .unwrap_or("server-default")
                .to_string();
            (k.clone(), compiler)
        })
        .collect();
    match matches.as_slice() {
        [(k, _)] => Ok(k.clone()),
        [] => Err(format!(
            "no cached program for {suite_name} (shape {shape_label}) in {} — the local \
             lane never compiles; run the hosted lane once (--paw) to create the \
             artifact, then re-run --paw-local",
            cache_file.display()
        )),
        many => {
            let comps: Vec<&str> = many.iter().map(|(_, c)| c.as_str()).collect();
            Err(format!(
                "{suite_name} (shape {shape_label}) has {} cached programs ({}) — set \
                 PAW_COMPILER to pick one",
                many.len(),
                comps.join(", ")
            ))
        }
    }
}

/// Resolve the interpreter + script from the env (their defaults).
#[must_use]
pub fn resolve_invocation() -> (String, PathBuf) {
    let nonempty = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
    (
        nonempty(PYTHON_ENV).unwrap_or_else(|| DEFAULT_PYTHON.to_string()),
        PathBuf::from(nonempty(SCRIPT_ENV).unwrap_or_else(|| DEFAULT_SCRIPT.to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::super::paw::CompiledProgram;
    use super::*;
    use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion, build_sst5};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    fn envelope(rows: &[(&str, i64)]) -> serde_json::Value {
        json!({
            "features": [],
            "rows": rows.iter().enumerate()
                .map(|(i, (t, l))| json!({"row_idx": i, "row": {"text": t, "label": l}}))
                .collect::<Vec<_>>(),
        })
    }

    /// Records what the session writes; responses are pre-loaded into the
    /// read buffer in protocol order. Line-buffered: `write_fmt` may split
    /// one `writeln!` across several `write` calls (the interpolation and
    /// the newline are separate), so bytes accumulate until a `\n`.
    struct ChanWriter {
        requests: Arc<Mutex<Vec<String>>>,
        partial: String,
    }
    impl Write for ChanWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.partial.push_str(&String::from_utf8_lossy(buf));
            while let Some(pos) = self.partial.find('\n') {
                let line: String = self.partial.drain(..=pos).collect();
                let line = line.trim_end_matches('\n').to_string();
                self.requests.lock().unwrap().push(line);
            }
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct ChanReader {
        buffer: Arc<Mutex<Vec<u8>>>,
    }
    impl Read for ChanReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut b = self.buffer.lock().unwrap();
            let n = buf.len().min(b.len());
            buf[..n].copy_from_slice(&b[..n]);
            b.drain(..n);
            Ok(n)
        }
    }

    fn session_with(script: &[String]) -> (PawLocalSession, Arc<Mutex<Vec<String>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let buffer = Arc::new(Mutex::new(
            script.iter().map(|l| format!("{l}\n")).collect::<String>().into_bytes(),
        ));
        let session = PawLocalSession::from_channels(
            Box::new(ChanReader { buffer }),
            Box::new(ChanWriter {
                requests: Arc::clone(&requests),
                partial: String::new(),
            }),
        )
        .unwrap();
        (session, requests)
    }

    fn test_cfg(cache_file: PathBuf) -> PawConfig {
        PawConfig {
            cache_file,
            ..PawConfig::from_env()
        }
    }

    fn scratch_dir() -> PathBuf {
        // The shared-temp-path law: pid-scoped, never a fixed name.
        std::env::temp_dir().join(format!("paw_local_test_{}", std::process::id()))
    }

    #[test]
    fn handshake_parses_and_rejects_foreign_lanes() {
        let (runtime, device) = parse_handshake(
            r#"{"ready": true, "lane": "paw-local", "runtime": "programasweights 0.4.10", "device": "auto"}"#,
        )
        .unwrap();
        assert_eq!(runtime, "programasweights 0.4.10");
        assert_eq!(device, "auto");
        assert!(parse_handshake(r#"{"ready": false}"#).is_err());
        assert!(
            parse_handshake(r#"{"ready": true, "lane": "gliner"}"#)
                .unwrap_err()
                .contains("not \"paw-local\"")
        );
        assert!(parse_handshake("not json").is_err());
    }

    #[test]
    fn response_parses_output_and_error_channel() {
        assert_eq!(parse_response(r#"{"output": "\"neutral\""}"#).unwrap(), "\"neutral\"");
        assert_eq!(parse_response(r#"{"output": "world"}"#).unwrap(), "world");
        assert!(
            parse_response(r#"{"error": "llama broke"}"#)
                .unwrap_err()
                .contains("llama broke")
        );
        assert!(parse_response(r#"{"nope": 1}"#).unwrap_err().contains("no output"));
        assert!(parse_response("garbage").is_err());
    }

    #[test]
    fn infer_round_trips_one_json_line_per_request() {
        let (mut session, requests) = session_with(&[json!({"output": "world"}).to_string()]);
        let out = session.infer("pid-1", "the input").unwrap();
        assert_eq!(out, "world");
        let reqs = requests.lock().unwrap();
        let v: serde_json::Value = serde_json::from_str(&reqs[0]).unwrap();
        assert_eq!(v["program_id"], "pid-1");
        assert_eq!(v["input"], "the input");
    }

    #[test]
    fn run_suite_local_pins_the_law_end_to_end() {
        // One suite, 3 cases, 1 question each; the scripted oracle answers
        // the gold for case 0, a quoted gold for case 1 (quote_stripped),
        // and prose for case 2 (refusal). The determinism rerun doubles
        // every served question (3 < 10).
        let s = build_sst5(&envelope(&[("a", 0), ("b", 4), ("c", 2)]), 0);
        let cfg = test_cfg(scratch_dir().join("programs_e2e.json"));
        let (_, spec) = paw::load_spec(&cfg.specs_dir, s.name).unwrap().unwrap();
        let opts = paw::option_set(&s.cases[0].questions[0]);

        let program = CompiledProgram {
            program_id: "test-program-id".to_string(),
            compiler_snapshot: Some("paw-ft-bs48-20260530".to_string()),
            compile_wall_s: 11.5,
            posture: "hosted-anonymous".to_string(),
        };
        let mut cache = ProgramCache { entries: BTreeMap::new() };
        cache
            .entries
            .insert(ProgramCache::key(s.name, None, &spec), program);
        cache.save(&cfg.cache_file).unwrap();

        let answers = [
            opts[0].0.clone(),                    // case 0: exact key
            format!("\"{}\"", opts[4].0.clone()), // case 1: quoted
            "I think neutral".to_string(),        // case 2: refusal
        ];
        let mut script = vec![json!({ "output": "warm" }).to_string()]; // warmup
        for a in &answers {
            script.push(json!({ "output": a }).to_string());
            script.push(json!({ "output": a }).to_string()); // rerun
        }
        let (mut session, requests) = session_with(&script);

        let r = run_suite(&mut session, &cfg, &s, 0).unwrap().expect("row");
        assert_eq!(r.lane, "paw-local");
        assert_eq!(r.posture, "local-subprocess");
        assert_eq!(r.program_id, "test-program-id");
        assert_eq!(r.model, "paw-ft-bs48-20260530");
        assert!(r.compile_cache_hit);
        assert_eq!((r.n_questions, r.n_answered, r.refusals), (3, 2, 1));
        assert_eq!(r.quote_stripped, 1);
        assert!((r.accuracy - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(r.determinism_ok, Some(true));
        assert!(r.server_latency_p50_ms.is_none());
        // Warmup + 3 measured + 3 rerun = 7 requests, every one carrying
        // the program id from the cache (never the suite's name or spec).
        let reqs = requests.lock().unwrap();
        assert_eq!(reqs.len(), 7);
        for line in reqs.iter() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(v["program_id"], "test-program-id");
        }
    }

    #[test]
    fn missing_cached_program_is_loud_and_names_the_remedy() {
        let s = build_sst5(&envelope(&[("a", 0)]), 0);
        let cfg = test_cfg(scratch_dir().join("absent.json"));
        let (mut session, _requests) = session_with(&[]);
        let err = run_suite(&mut session, &cfg, &s, 0).unwrap_err();
        assert!(err.contains("never compiles"), "{err}");
        assert!(err.contains("--paw"), "{err}");
    }

    #[test]
    fn ambiguous_compilers_refuse_and_name_the_choices() {
        let s = build_sst5(&envelope(&[("a", 0)]), 0);
        let cfg = test_cfg(scratch_dir().join("programs_ambiguous.json"));
        let (_, spec) = paw::load_spec(&cfg.specs_dir, s.name).unwrap().unwrap();
        let mut cache = ProgramCache { entries: BTreeMap::new() };
        for compiler in ["tier-a", "tier-b"] {
            cache.entries.insert(
                ProgramCache::key(s.name, Some(compiler), &spec),
                CompiledProgram {
                    program_id: format!("pid-{compiler}"),
                    compiler_snapshot: Some(compiler.to_string()),
                    compile_wall_s: 1.0,
                    posture: "hosted-anonymous".to_string(),
                },
            );
        }
        cache.save(&cfg.cache_file).unwrap();
        let (mut session, _requests) = session_with(&[]);
        let err = run_suite(&mut session, &cfg, &s, 0).unwrap_err();
        assert!(err.contains("PAW_COMPILER"), "{err}");
        assert!(err.contains("tier-a") && err.contains("tier-b"), "{err}");
    }

    #[test]
    fn pinned_compiler_selects_its_exact_key() {
        let s = build_sst5(&envelope(&[("a", 0)]), 0);
        let cfg = PawConfig {
            cache_file: scratch_dir().join("programs_pinned.json"),
            compiler: Some("tier-b".to_string()),
            ..PawConfig::from_env()
        };
        let (_, spec) = paw::load_spec(&cfg.specs_dir, s.name).unwrap().unwrap();
        let mut cache = ProgramCache { entries: BTreeMap::new() };
        for compiler in ["tier-a", "tier-b"] {
            cache.entries.insert(
                ProgramCache::key(s.name, Some(compiler), &spec),
                CompiledProgram {
                    program_id: format!("pid-{compiler}"),
                    compiler_snapshot: Some(compiler.to_string()),
                    compile_wall_s: 1.0,
                    posture: "hosted-anonymous".to_string(),
                },
            );
        }
        cache.save(&cfg.cache_file).unwrap();
        let script = vec![
            json!({ "output": "warm" }).to_string(),
            json!({ "output": "very negative" }).to_string(),
            json!({ "output": "very negative" }).to_string(),
        ];
        let (mut session, requests) = session_with(&script);
        let r = run_suite(&mut session, &cfg, &s, 0).unwrap().expect("row");
        assert_eq!(r.program_id, "pid-tier-b");
        assert_eq!(r.model, "tier-b");
        assert_eq!(r.accuracy, 1.0);
        // No ambiguity note when the compiler is pinned.
        let reqs = requests.lock().unwrap();
        let v: serde_json::Value = serde_json::from_str(&reqs[1]).unwrap();
        assert_eq!(v["program_id"], "pid-tier-b");
    }

    #[test]
    fn missing_spec_is_an_honest_absence_not_an_error() {
        let s = build_sst5(&envelope(&[("a", 0)]), 0);
        let dir = scratch_dir().join("no_specs_here");
        let cfg = PawConfig {
            specs_dir: dir,
            ..PawConfig::from_env()
        };
        let (mut session, _requests) = session_with(&[]);
        assert!(run_suite(&mut session, &cfg, &s, 0).unwrap().is_none());
    }

    #[test]
    fn multi_shape_suite_dispatches_each_question_to_its_own_program() {
        // The hosted lane's two_shape law, one layer down: per-qid specs,
        // two cached programs, every question answered by ITS shape's
        // program. The module answers map to gold (parsed); the is_pub
        // answer is prose (refused, never guessed) — both shapes' paths
        // exercised in one pass.
        let pid = std::process::id();
        let specs = scratch_dir().join(format!("multi_shape_specs_{pid}"));
        std::fs::create_dir_all(&specs).unwrap();
        let module_spec = "Pick a label:\nalpha\nbeta\n";
        let is_pub_spec = "Pick a word:\nfalse\ntrue\n";
        std::fs::write(specs.join("two_shape.module.txt"), module_spec).unwrap();
        std::fs::write(specs.join("two_shape.is_pub.txt"), is_pub_spec).unwrap();

        let suite = Suite {
            name: "two_shape",
            cases: vec![SuiteCase {
                id: "c0".into(),
                state: json!("fn sample() {}"),
                questions: vec![
                    SuiteQuestion {
                        qid: "module".into(),
                        kind: QKind::Choice,
                        instructions: String::new(),
                        criteria: json!({"alpha": null, "beta": null}),
                    },
                    SuiteQuestion {
                        qid: "is_pub".into(),
                        kind: QKind::Noul,
                        instructions: String::new(),
                        criteria: serde_json::Value::Null,
                    },
                ],
                gold: vec![
                    crate::harness::suites::GoldAnswer { idx: 0, soft: vec![0.0; 2], gold_score: None },
                    crate::harness::suites::GoldAnswer { idx: 1, soft: vec![0.0; 2], gold_score: None },
                ],
            }],
            option_counts_note: "",
        };
        let cache_file = scratch_dir().join(format!("programs_multi_{pid}.json"));
        let cfg = PawConfig {
            cache_file,
            specs_dir: specs.clone(),
            ..PawConfig::from_env()
        };
        let program = |id: &str, wall: f64| CompiledProgram {
            program_id: id.to_string(),
            compiler_snapshot: Some("tier-a".to_string()),
            compile_wall_s: wall,
            posture: "hosted-anonymous".to_string(),
        };
        let mut cache = ProgramCache { entries: BTreeMap::new() };
        cache.entries.insert(
            ProgramCache::key_for_qid(suite.name, "module", Some("tier-a"), module_spec),
            program("pid-module", 10.0),
        );
        cache.entries.insert(
            ProgramCache::key_for_qid(suite.name, "is_pub", Some("tier-a"), is_pub_spec),
            program("pid-is_pub", 20.0),
        );
        cache.save(&cfg.cache_file).unwrap();

        // 2 warmups (shape order: module, is_pub) + per question: measured
        // + untimed rerun (1 case < 10, so the rerun doubles it).
        let mut script = vec![
            json!({ "output": "warm-module" }).to_string(),
            json!({ "output": "warm-is_pub" }).to_string(),
        ];
        script.push(json!({ "output": "alpha" }).to_string()); // module: gold
        script.push(json!({ "output": "alpha" }).to_string()); // rerun
        script.push(json!({ "output": "I think so" }).to_string()); // is_pub: refusal
        script.push(json!({ "output": "I think so" }).to_string()); // rerun
        let (mut session, requests) = session_with(&script);

        let r = run_suite(&mut session, &cfg, &suite, 0).unwrap().expect("row");
        assert_eq!(r.program_id, "pid-module,pid-is_pub");
        assert_eq!(r.model, "tier-a");
        assert_eq!(
            r.spec_file,
            format!(
                "{},{}",
                specs.join("two_shape.module.txt").display(),
                specs.join("two_shape.is_pub.txt").display()
            )
        );
        assert_eq!(r.spec_blake3.split(',').count(), 2);
        assert!(r.compile_cache_hit);
        assert!((r.compile_wall_s - 30.0).abs() < 1e-9); // summed
        assert_eq!((r.n_questions, r.n_answered, r.refusals), (2, 1, 1));
        assert_eq!(r.accuracy, 0.5);
        assert_eq!(r.determinism_ok, Some(true));

        // Every request names ITS shape's program — the dispatch law.
        let reqs = requests.lock().unwrap();
        assert_eq!(reqs.len(), 6); // 2 warmups + 2 measured + 2 reruns
        let ids: Vec<String> = reqs
            .iter()
            .map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["program_id"].as_str().unwrap().to_string()
            })
            .collect();
        assert_eq!(
            ids,
            vec![
                "pid-module".to_string(),
                "pid-is_pub".to_string(), // warmups in shape order
                "pid-module".to_string(),
                "pid-module".to_string(), // q1 measured + its rerun
                "pid-is_pub".to_string(),
                "pid-is_pub".to_string(), // q2 measured + its rerun
            ]
        );
        let _ = std::fs::remove_dir_all(&specs);
    }

    #[test]
    fn multi_shape_missing_one_shape_refuses_partial_coverage() {
        // A suite-wide spec covers only SOME qids while per-qid specs own
        // the rest: the unspecced shape is partial coverage, refused —
        // never a partial row (the shared resolver's law).
        let pid = std::process::id();
        let specs = scratch_dir().join(format!("partial_specs_{pid}"));
        std::fs::create_dir_all(&specs).unwrap();
        std::fs::write(specs.join("two_shape.module.txt"), "Pick:\nalpha\nbeta\n").unwrap();
        let suite = Suite {
            name: "two_shape",
            cases: vec![SuiteCase {
                id: "c0".into(),
                state: json!("fn sample() {}"),
                questions: vec![
                    SuiteQuestion {
                        qid: "module".into(),
                        kind: QKind::Choice,
                        instructions: String::new(),
                        criteria: json!({"alpha": null, "beta": null}),
                    },
                    SuiteQuestion {
                        qid: "is_pub".into(),
                        kind: QKind::Noul,
                        instructions: String::new(),
                        criteria: serde_json::Value::Null,
                    },
                ],
                gold: vec![
                    crate::harness::suites::GoldAnswer { idx: 0, soft: vec![0.0; 2], gold_score: None },
                    crate::harness::suites::GoldAnswer { idx: 1, soft: vec![0.0; 2], gold_score: None },
                ],
            }],
            option_counts_note: "",
        };
        let cfg = PawConfig {
            cache_file: scratch_dir().join(format!("programs_partial_{pid}.json")),
            specs_dir: specs,
            ..PawConfig::from_env()
        };
        let (mut session, _requests) = session_with(&[]);
        let err = run_suite(&mut session, &cfg, &suite, 0).unwrap_err();
        assert!(err.contains("partial coverage refused"), "{err}");
        assert!(err.contains("is_pub"), "{err}");
    }
}
