//! The cua-s1-forms arena arm (`.issues/035`) — the CoreML/ANE serving
//! posture of the System-One family, measured on ITS OWN home task.
//!
//! Measurement only — never a gate, never in the default run (an example
//! target, spawned by hand). Their stack serves the CoreML model
//! (`scripts/cua_s1_lane.py`, coremltools subprocess — the gliner /
//! laya-python lane shape); our Rust measures every lane.
//!
//! # The comparability claim (pinned — G1 of `.issues/035`)
//!
//! **Fixture = THEIR published synthetic test split**, not our suites:
//! HF dataset `cua-ai/cua-s1-forms` @ `8273f34778b99ac2e12d9f6e7d57dad99ae20845`,
//! `test.jsonl`, 24,370 rows, SHA-256
//! `d63a7e0db195d4d20154a40b2f8dd09ce3bb65487a158c638da5c609d4475e7c`
//! (their card's pin; verified at download) — BLAKE3 [`TEST_BLAKE3`] is the
//! house-hash pin this example refuses to run without. Their model is a
//! form-filling specialist; our 15 NLU suites are out-of-distribution for
//! it by construction (T1), so the honest cell is its home task.
//!
//! **Mapping (one row → one choice question):** the row's `context` is the
//! wire `state` (a string, verbatim); its `options` (2–32, always ending
//! `check`/`click`/`skip`, every earlier one a `fill <entity>: <value>`) are
//! the choice options IN ORDER; `label` is the gold option index. Nothing is
//! dropped, reordered, or truncated on our side (their encoder truncates by
//! bytes at 224/96 — the fixture's max context is 217 bytes and max option
//! 92, so no row is truncated either). Every lane sees the same rows.
//!
//! **What each lane knows (the asymmetry the table must carry):**
//! - `coreml` — Cua's 706K-param model, TRAINED on the same generator's
//!   train split (form-signature-disjoint from test per their card). A
//!   trained in-distribution specialist. Probabilities = their softmax head,
//!   taken verbatim; their own report shows strict numerical parity FAILS
//!   (11 rows > 0.005), so only argmax accuracy + latency are published —
//!   never calibration.
//! - `modelless` — OUR engine, no gradients: the house corpus-is-the-model
//!   protocol (train rows → capped per-domain corpora, FIRST-N in train
//!   order). Domains = the gold ACTION (`fill`/`check`/`click`/`skip`,
//!   `meta.action`); each corpus doc is the query's own byte layout with the
//!   gold option appended (`{context}\n{PROMPT}{gold option}`), so the LZ4
//!   drafter's compressed-length delta rewards an option that continues a
//!   train context the way its gold did. Route terms are STRUCTURALLY
//!   inactive (k ∈ [7, 32] ≠ N = 4, and a `fill …` option never names a
//!   domain) — routing picks the corpus, the drafter ranks the options.
//! - `laya` — the laya `typed` checkpoint (the choice specialist), ZERO-SHOT:
//!   state = the context string, one choice question, criteria = the options
//!   in order (null descriptions), instructions = [`PROMPT`].
//!
//! Floors printed beside every table: uniform chance (mean 1/k) and the
//! constant-`skip` pick (skip is the majority gold action, ~52%).
//!
//! # Subsampling (disclosed)
//!
//! `--n-<lane> N` (0 = the full split) takes the EVEN-STRIDE subsample
//! `row ⌊i·24370/N⌋, i ∈ [0, N)` — deterministic, first row included. The
//! paired table re-reads every lane on the smallest lane's row set.
//!
//! # Latency (the laya-python measurement law)
//!
//! Per-row wall time around the lane call: the coreml lane's `rt` is the
//! full subprocess round-trip (request write → response read), `model` is
//! THEIR timer around `model.predict` alone (their card's scope). Warmup
//! rows are unmeasured (3 for coreml/modelless — their protocol; 1 for
//! laya — the GPU pre-ramp). Box state is stamped at start + end; run
//! `scripts/bench_preflight.sh` first and quote its PROVENANCE line.
//!
//! # Run (macOS — the coreml lane skips LOUD elsewhere)
//!
//! ```sh
//! # one-time, gitignored .raw/ (never vendored):
//! hf download FluidInference/cua-s1-forms-coreml --revision 8e18ee41083f251b2fc3641ebf2671eab19a1650 \
//!     --local-dir .raw/cua-s1-forms/coreml
//! hf download cua-ai/cua-s1-forms --repo-type dataset \
//!     --revision 8273f34778b99ac2e12d9f6e7d57dad99ae20845 --local-dir .raw/cua-s1-forms/dataset
//! uv venv --python 3.12 .raw/cua-s1-forms/venv
//! VIRTUAL_ENV=.raw/cua-s1-forms/venv uv pip install coremltools==9.0 numpy==1.26.4
//!
//! scripts/bench_preflight.sh
//! cargo run --release --example cua_s1_forms_arena                       # coreml + modelless, full split
//! LAYA_DEVICE=metal cargo run --release --features laya-riir-metal \
//!     --example cua_s1_forms_arena -- --lanes coreml,modelless,laya --n-laya 240
//! ```
//!
//! Env: `CUA_S1_DIR` (default `.raw/cua-s1-forms`), `CUA_S1_PYTHON`
//! (default `$CUA_S1_DIR/venv/bin/python`), `CUA_S1_LANE_SCRIPT` (default
//! `scripts/cua_s1_lane.py`), `CUA_S1_COMPUTE_UNITS` (forwarded; default
//! `CPU_AND_NE`). A missing fixture exits 2 (nothing measured); a missing
//! model / venv / laya checkpoint SKIPS that lane LOUD — never a row.
#![cfg(feature = "modelless")]

use std::path::{Path, PathBuf};
use std::time::Instant;

use katgpt_core::decision_wire::{DecisionRequest, Outcome, Question};
use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};
use riir_reflex::harness::box_state::{self, BoxStateSpan};
use serde_json::{Value, json};

/// BLAKE3 of the pinned `test.jsonl` (dataset rev `8273f347…`).
const TEST_BLAKE3: &str = "78223e7a5ac8557ae0deea3acf484b4e5a40c6ea4c439fe3fb55ca08167ae14b";
/// BLAKE3 of the pinned `train.jsonl` (same revision) — the modelless corpus source.
const TRAIN_BLAKE3: &str = "19206c9df46e0692ef0c82d9dfedbe7ce7eff14658efdeb02465f7ce10c5191f";
const TEST_ROWS: usize = 24_370;
/// The one instruction our lanes read (modelless prompt, laya instructions).
const PROMPT: &str = "Choose the one action for the ELEMENT: fill it with the matching \
                      document entry, or check, click, or skip it.";
/// The modelless domains (the gold `meta.action` vocabulary).
const ACTIONS: [&str; 4] = ["fill", "check", "click", "skip"];
/// Per-domain corpus cap (FIRST-N in train order) — pre-declared, never
/// selected on test (the house default ag_news cap).
const DEFAULT_CAP: usize = 64;

struct Row {
    context: String,
    options: Vec<String>,
    label: usize,
    action: String,
}

/// One lane's per-row outcome over its row set.
struct LaneRun {
    name: String,
    detail: String,
    rows: Vec<usize>,
    picks: Vec<usize>,
    lat_us: Vec<f64>,
    /// A second timer when the lane has one (coreml: THEIR predict-only).
    lat2_us: Vec<f64>,
}

fn fail(msg: &str) -> ! {
    eprintln!("⛔ {msg}");
    std::process::exit(2);
}

fn load_rows(path: &Path, pin: &str) -> Result<Vec<Row>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let got = blake3::hash(&bytes).to_hex().to_string();
    if got != pin {
        return Err(format!(
            "{}: BLAKE3 {got} ≠ pinned {pin} — not the pinned revision",
            path.display()
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let v: Value = serde_json::from_str(line).map_err(|e| format!("line {i}: {e}"))?;
        let options: Vec<String> = v["options"]
            .as_array()
            .ok_or_else(|| format!("line {i}: options"))?
            .iter()
            .map(|o| o.as_str().map(str::to_string))
            .collect::<Option<_>>()
            .ok_or_else(|| format!("line {i}: non-string option"))?;
        let label = v["label"]
            .as_u64()
            .ok_or_else(|| format!("line {i}: label"))? as usize;
        if label >= options.len() {
            return Err(format!("line {i}: label {label} ≥ {} options", options.len()));
        }
        rows.push(Row {
            context: v["context"].as_str().ok_or("context")?.to_string(),
            options,
            label,
            action: v["meta"]["action"].as_str().unwrap_or("?").to_string(),
        });
    }
    Ok(rows)
}

/// The even-stride subsample law (module doc): `⌊i·total/n⌋`.
fn stride(total: usize, n: usize) -> Vec<usize> {
    match n {
        0 => (0..total).collect(),
        n if n >= total => (0..total).collect(),
        n => (0..n).map(|i| i * total / n).collect(),
    }
}

/// Nearest-rank percentile (`idx = ⌈n·p⌉ − 1`) + its tail support.
fn pct(sorted: &[f64], p: f64) -> (f64, usize) {
    let n = sorted.len();
    match n {
        0 => (f64::NAN, 0),
        _ => {
            let idx = ((n as f64 * p).ceil() as usize).clamp(1, n) - 1;
            (sorted[idx], n - idx)
        }
    }
}

// ── the modelless lane ──────────────────────────────────────────────────

fn run_modelless(train: &[Row], test: &[Row], idx: &[usize], cap: usize) -> Result<LaneRun, String> {
    let mut specs = Vec::with_capacity(ACTIONS.len());
    for a in ACTIONS {
        let docs: Vec<String> = train
            .iter()
            .filter(|r| r.action == a)
            .take(cap)
            .map(|r| format!("{}\n{PROMPT}{}", r.context, r.options[r.label]))
            .collect();
        if docs.is_empty() {
            return Err(format!("no train rows for action {a}"));
        }
        specs.push(ExpertSpec::new(a, &docs));
    }
    let mut engine = DecisionEngine::<4, EMBED_DIM>::build_specs(specs, EngineConfig::default())
        .map_err(|e| format!("engine build: {e}"))?;
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    sc.prepare(1);
    let request = |r: &Row| DecisionRequest {
        state: r.context.clone(),
        questions: vec![Question::choice("q0", PROMPT, r.options.clone(), None)],
    };
    for &i in idx.iter().take(3) {
        engine
            .decide_with(&request(&test[i]), &mut sc)
            .map_err(|e| format!("warmup: {e}"))?;
    }
    let mut run = LaneRun {
        name: "reflex · modelless".into(),
        detail: format!("in-process, cap {cap}/action, route terms inactive"),
        rows: idx.to_vec(),
        picks: Vec::with_capacity(idx.len()),
        lat_us: Vec::with_capacity(idx.len()),
        lat2_us: Vec::new(),
    };
    for &i in idx {
        let req = request(&test[i]);
        let t0 = Instant::now();
        let resp = engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("row {i}: {e}"))?;
        run.lat_us.push(t0.elapsed().as_secs_f64() * 1e6);
        let a = &resp.answers[0];
        // Forced pick: an abstained answer still carries its distribution.
        let pick = match a.outcome {
            Some(Outcome::Choice { index }) => index as usize,
            _ => argmax_f32(&a.probabilities),
        };
        run.picks.push(pick);
    }
    Ok(run)
}

fn argmax_f32(p: &[f32]) -> usize {
    let mut best = 0usize;
    for (i, v) in p.iter().enumerate() {
        if *v > p[best] {
            best = i;
        }
    }
    best
}

// ── the coreml lane (macOS only) ────────────────────────────────────────

#[cfg(target_os = "macos")]
fn run_coreml(dir: &Path, test: &[Row], idx: &[usize]) -> Result<LaneRun, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    let model_dir = dir.join("coreml");
    let package = std::env::var("CUA_S1_PACKAGE")
        .unwrap_or_else(|_| "cua_s1_forms_fp16_options32.mlpackage".into());
    if !model_dir.join(&package).exists() || !model_dir.join("preprocessing.py").is_file() {
        return Err(format!(
            "model absent at {} (need {package} + preprocessing.py) — hf download \
             FluidInference/cua-s1-forms-coreml (module doc)",
            model_dir.display()
        ));
    }
    let python = std::env::var("CUA_S1_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dir.join("venv/bin/python"));
    if !python.exists() {
        return Err(format!(
            "venv python absent at {} — uv venv + coremltools==9.0 numpy==1.26.4 (module doc)",
            python.display()
        ));
    }
    let script = std::env::var("CUA_S1_LANE_SCRIPT").unwrap_or_else(|_| "scripts/cua_s1_lane.py".into());
    if !Path::new(&script).is_file() {
        return Err(format!("lane script absent at {script} — run from the repo root"));
    }
    let mut child = Command::new(&python)
        .arg(&script)
        .arg(&model_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", python.display()))?;
    let mut stdin = child.stdin.take().ok_or("oracle stdin")?;
    let mut reader = BufReader::new(child.stdout.take().ok_or("oracle stdout")?);
    let mut line = String::new();
    let read = |reader: &mut BufReader<_>, line: &mut String| -> Result<Value, String> {
        line.clear();
        let n = reader.read_line(line).map_err(|e| format!("oracle stdout: {e}"))?;
        if n == 0 {
            return Err("oracle stream ended — the subprocess died; its stderr names the cause".into());
        }
        serde_json::from_str(line.trim()).map_err(|e| format!("oracle json: {e} ({})", line.trim()))
    };
    let ready = read(&mut reader, &mut line)?;
    if ready.get("ready") != Some(&Value::Bool(true)) {
        return Err(format!("handshake: expected ready, got {ready}"));
    }
    let detail = format!(
        "{} {} · {} · coremltools {} · load {} ms",
        ready["model"].as_str().unwrap_or("?"),
        ready["package"].as_str().unwrap_or("?"),
        ready["compute_units"].as_str().unwrap_or("?"),
        ready["coremltools"].as_str().unwrap_or("?"),
        ready["load_ms"]
    );
    eprintln!("  [coreml] oracle up: {detail}");
    let mut run = LaneRun {
        name: "cua-s1-forms · coreml".into(),
        detail,
        rows: idx.to_vec(),
        picks: Vec::with_capacity(idx.len()),
        lat_us: Vec::with_capacity(idx.len()),
        lat2_us: Vec::with_capacity(idx.len()),
    };
    let warm: Vec<usize> = idx.iter().take(3).copied().collect();
    for (k, &i) in warm.iter().chain(idx.iter()).enumerate() {
        let r = &test[i];
        let req = json!({"context": r.context, "options": r.options}).to_string();
        let t0 = Instant::now();
        writeln!(stdin, "{req}").map_err(|e| format!("oracle stdin: {e}"))?;
        stdin.flush().map_err(|e| format!("oracle flush: {e}"))?;
        let resp = read(&mut reader, &mut line)?;
        let rt = t0.elapsed().as_secs_f64() * 1e6;
        if let Some(err) = resp.get("error") {
            return Err(format!("row {i}: oracle refused: {err}"));
        }
        if k < warm.len() {
            continue;
        }
        let pick = resp["pick"].as_u64().ok_or_else(|| format!("row {i}: no pick"))? as usize;
        let p_len = resp["p"].as_array().map_or(0, Vec::len);
        if p_len != r.options.len() || pick >= r.options.len() {
            return Err(format!("row {i}: response shape {p_len}/{pick} vs {} options", r.options.len()));
        }
        run.picks.push(pick);
        run.lat_us.push(rt);
        run.lat2_us.push(resp["predict_us"].as_f64().unwrap_or(f64::NAN));
    }
    drop(stdin);
    let _ = child.wait();
    Ok(run)
}

#[cfg(not(target_os = "macos"))]
fn run_coreml(_dir: &Path, _test: &[Row], _idx: &[usize]) -> Result<LaneRun, String> {
    Err("this lane is macOS-only (CoreML/ANE) — not measured on this platform".into())
}

// ── the laya lane (feature-gated) ───────────────────────────────────────

#[cfg(feature = "laya-riir")]
fn run_laya(test: &[Row], idx: &[usize], ckpt: &str) -> Result<LaneRun, String> {
    use riir_reflex::laya::config::Checkpoint;
    let checkpoint = match ckpt {
        "typed" => Checkpoint::TypedDecisions,
        "english" => Checkpoint::English,
        "multilingual" => Checkpoint::Multilingual,
        other => return Err(format!("unknown laya checkpoint {other}")),
    };
    let root = riir_reflex::laya::weights::weights_root();
    let agent = riir_reflex::laya::riir::RiirAgent::load(&root, checkpoint)
        .map_err(|e| format!("laya {ckpt} load (weights root {}): {e}", root.display()))?;
    let questions = |r: &Row| {
        let mut crit = serde_json::Map::new();
        for o in &r.options {
            crit.insert(o.clone(), Value::Null);
        }
        vec![(
            "q0".to_string(),
            json!({"type": "choice", "instructions": PROMPT, "criteria": Value::Object(crit)}),
        )]
    };
    let mut run = LaneRun {
        name: format!("reflex · laya-riir·{ckpt}"),
        detail: format!("zero-shot, device {}", agent.device()),
        rows: idx.to_vec(),
        picks: Vec::with_capacity(idx.len()),
        lat_us: Vec::with_capacity(idx.len()),
        lat2_us: Vec::new(),
    };
    if let Some(&i) = idx.first() {
        let r = &test[i];
        agent
            .system_one(&Value::String(r.context.clone()), &questions(r))
            .map_err(|e| format!("laya warmup: {e}"))?;
    }
    for &i in idx {
        let r = &test[i];
        let (state, qs) = (Value::String(r.context.clone()), questions(r));
        let t0 = Instant::now();
        let ans = agent
            .system_one(&state, &qs)
            .map_err(|e| format!("laya row {i}: {e}"))?;
        run.lat_us.push(t0.elapsed().as_secs_f64() * 1e6);
        let key = ans
            .first()
            .and_then(|a| a.choice.clone())
            .ok_or_else(|| format!("laya row {i}: no choice"))?;
        let pick = r
            .options
            .iter()
            .position(|o| *o == key)
            .ok_or_else(|| format!("laya row {i}: choice {key:?} is not an option"))?;
        run.picks.push(pick);
    }
    Ok(run)
}

#[cfg(not(feature = "laya-riir"))]
fn run_laya(_test: &[Row], _idx: &[usize], _ckpt: &str) -> Result<LaneRun, String> {
    Err("this build has no laya lane — rebuild with --features laya-riir (or laya-riir-metal)".into())
}

// ── reporting ───────────────────────────────────────────────────────────

fn acc_on(run: &LaneRun, test: &[Row], subset: Option<&[usize]>) -> (usize, usize) {
    let mut n = 0usize;
    let mut ok = 0usize;
    for (&i, &p) in run.rows.iter().zip(run.picks.iter()) {
        if subset.is_some_and(|s| s.binary_search(&i).is_err()) {
            continue;
        }
        n += 1;
        ok += usize::from(p == test[i].label);
    }
    (ok, n)
}

fn per_action(run: &LaneRun, test: &[Row]) -> String {
    ACTIONS
        .iter()
        .map(|a| {
            let (mut n, mut ok) = (0usize, 0usize);
            for (&i, &p) in run.rows.iter().zip(run.picks.iter()) {
                if test[i].action == *a {
                    n += 1;
                    ok += usize::from(p == test[i].label);
                }
            }
            format!("{a} {ok}/{n}")
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// What the lane PICKED, bucketed by the picked option's action — the
/// degenerate-lane readout (Issue 004 T7): a lane whose picks collapse onto
/// one bucket regardless of gold is a constant pick, whatever its accuracy.
fn pick_hist(run: &LaneRun, test: &[Row]) -> String {
    let mut h = [0usize; 4];
    for (&i, &p) in run.rows.iter().zip(run.picks.iter()) {
        let o = test[i].options[p].as_str();
        let b = ACTIONS.iter().position(|a| o == *a).unwrap_or(0); // `fill …` → 0
        h[b] += 1;
    }
    ACTIONS
        .iter()
        .zip(h)
        .map(|(a, n)| format!("{a} {n}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn floors(test: &[Row], idx: &[usize]) -> (f64, f64) {
    let n = idx.len().max(1) as f64;
    let chance = idx.iter().map(|&i| 1.0 / test[i].options.len() as f64).sum::<f64>() / n;
    let skip = idx
        .iter()
        .filter(|&&i| test[i].options[test[i].label] == "skip")
        .count() as f64
        / n;
    (chance, skip)
}

fn lat_cells(v: &[f64]) -> (String, Value) {
    let mut s = v.to_vec();
    s.retain(|x| x.is_finite());
    s.sort_by(f64::total_cmp);
    let (p50, _) = pct(&s, 0.50);
    let (p95, t95) = pct(&s, 0.95);
    let (p99, t99) = pct(&s, 0.99);
    let cell = format!("{:.3} / {:.3} / {:.3} ms (tail {t95}/{t99})", p50 / 1e3, p95 / 1e3, p99 / 1e3);
    let j = json!({"p50_ms": p50 / 1e3, "p95_ms": p95 / 1e3, "p99_ms": p99 / 1e3,
                   "tail_support_p95": t95, "tail_support_p99": t99, "n": s.len()});
    (cell, j)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let num = |name: &str, default: usize| {
        flag(name).map_or(default, |v| v.parse().unwrap_or_else(|_| fail(&format!("{name}: {v}"))))
    };
    let lanes = flag("--lanes").unwrap_or_else(|| "coreml,modelless".into());
    let cap = num("--cap", DEFAULT_CAP);
    let laya_ckpt = flag("--laya-ckpt").unwrap_or_else(|| "typed".into());
    let dir = std::env::var("CUA_S1_DIR").map_or_else(|_| PathBuf::from(".raw/cua-s1-forms"), PathBuf::from);

    let test_path = dir.join("dataset/test.jsonl");
    if !test_path.is_file() {
        fail(&format!(
            "UNSEEN — fixture absent at {} (hf download cua-ai/cua-s1-forms @ 8273f347…, module doc); nothing measured",
            test_path.display()
        ));
    }
    let test = load_rows(&test_path, TEST_BLAKE3).unwrap_or_else(|e| fail(&e));
    if test.len() != TEST_ROWS {
        fail(&format!("test.jsonl has {} rows, pinned {TEST_ROWS}", test.len()));
    }
    let span_start = box_state::capture();

    let mut runs: Vec<LaneRun> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    for lane in lanes.split(',').map(str::trim).filter(|l| !l.is_empty()) {
        let t0 = Instant::now();
        let res = match lane {
            "modelless" => {
                let train_path = dir.join("dataset/train.jsonl");
                load_rows(&train_path, TRAIN_BLAKE3)
                    .and_then(|train| run_modelless(&train, &test, &stride(TEST_ROWS, num("--n-modelless", 0)), cap))
            }
            "coreml" => run_coreml(&dir, &test, &stride(TEST_ROWS, num("--n-coreml", 0))),
            "laya" => run_laya(&test, &stride(TEST_ROWS, num("--n-laya", 240)), &laya_ckpt),
            other => fail(&format!("unknown lane {other} (coreml | modelless | laya)")),
        };
        match res {
            Ok(run) => {
                eprintln!("  [{lane}] {} rows in {:.1}s", run.rows.len(), t0.elapsed().as_secs_f64());
                runs.push(run);
            }
            Err(e) => {
                eprintln!("SKIP — lane {lane}: {e}");
                skipped.push(format!("{lane} ({e})"));
            }
        }
    }
    let span = BoxStateSpan {
        start: span_start,
        end: box_state::capture(),
    };
    if runs.is_empty() {
        fail(&format!("every requested lane SKIPPED — nothing measured: {}", skipped.join("; ")));
    }

    println!("\n## cua-s1-forms arena — their test split @ 8273f347 ({TEST_ROWS} rows)\n");
    println!("box: {}\n", box_state::render_line(&span));
    println!(
        "| lane | serving | N | top-1 | per gold action | picked | latency p50 / p95 / p99 | 2nd timer p50 / p95 / p99 |"
    );
    println!("|---|---|---:|---:|---|---|---|---|");
    let mut report = Vec::new();
    for run in &runs {
        let (ok, n) = acc_on(run, &test, None);
        let (cell, lat) = lat_cells(&run.lat_us);
        let (cell2, lat2) = match run.lat2_us.is_empty() {
            true => ("—".to_string(), Value::Null),
            false => lat_cells(&run.lat2_us),
        };
        let (chance, skip) = floors(&test, &run.rows);
        println!(
            "| {} | {} | {n} | {ok}/{n} = {:.4}% | {} | {} | {cell} | {cell2} |",
            run.name,
            run.detail,
            100.0 * ok as f64 / n as f64,
            per_action(run, &test),
            pick_hist(run, &test)
        );
        let errors: Vec<Value> = run
            .rows
            .iter()
            .zip(run.picks.iter())
            .filter(|&(&i, &p)| p != test[i].label)
            .take(40)
            .map(|(&i, &p)| json!({"row": i, "gold": test[i].options[test[i].label], "pick": test[i].options[p]}))
            .collect();
        report.push(json!({"lane": run.name, "detail": run.detail, "n": n, "correct": ok,
                           "per_action": per_action(run, &test),
                           "picked": pick_hist(run, &test), "latency": lat, "latency_2nd": lat2,
                           "floor_chance": chance, "floor_skip": skip, "first_errors": errors}));
    }
    let (chance, skip) = floors(&test, &stride(TEST_ROWS, 0));
    println!(
        "\nfloors (full split): uniform chance {:.4}% · constant-skip {:.4}%",
        100.0 * chance,
        100.0 * skip
    );
    // Paired table: every lane on the SMALLEST lane's row set (the stride
    // law nests only when N divides evenly, so intersect explicitly).
    if runs.len() > 1 {
        let mut common: Vec<usize> = runs[0].rows.clone();
        for r in &runs[1..] {
            common.retain(|i| r.rows.binary_search(i).is_ok());
        }
        let (pc, ps) = floors(&test, &common);
        println!(
            "\npaired on the common {} rows (chance {:.2}%, constant-skip {:.2}%):",
            common.len(),
            100.0 * pc,
            100.0 * ps
        );
        for run in &runs {
            let (ok, n) = acc_on(run, &test, Some(&common));
            println!("  {:<28} {ok}/{n} = {:.2}%", run.name, 100.0 * ok as f64 / n.max(1) as f64);
        }
    }
    if !skipped.is_empty() {
        println!("\nSKIPPED (not measured, no row): {}", skipped.join("; "));
    }
    if let Some(out) = flag("--out") {
        let j = json!({"fixture": {"repo": "cua-ai/cua-s1-forms", "revision": "8273f34778b99ac2e12d9f6e7d57dad99ae20845",
                                   "file": "test.jsonl", "rows": TEST_ROWS, "blake3": TEST_BLAKE3,
                                   "sha256": "d63a7e0db195d4d20154a40b2f8dd09ce3bb65487a158c638da5c609d4475e7c"},
                       "box": span, "cap": cap, "prompt": PROMPT, "lanes": report, "skipped": skipped,
                       "floors_full": {"chance": chance, "skip": skip}});
        std::fs::write(&out, serde_json::to_string_pretty(&j).unwrap_or_default())
            .unwrap_or_else(|e| fail(&format!("--out {out}: {e}")));
        eprintln!("  wrote {out}");
    }
}
