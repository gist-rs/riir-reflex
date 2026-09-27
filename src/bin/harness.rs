//! `harness` — the Plan 603 T1.5 runner CLI.
//!
//! Usage:
//! ```text
//! cargo run --release --bin harness -- [--suites a,b] [--laya-max-questions N]
//!                                      [--skip-laya] [--laya-python] [--out DIR]
//!                                      [--corpus-cap N] [--cal-select-cap [LIST]]
//!                                      [--head-scale F] [--head-select]
//!                                      [--nb-select] [--oc-select]
//!                                      [--ridge-select] [--genome-select]
//!                                      [--genome-accept-margin F]
//!                                      [--pair-head-ab] [--nli-feature-ab]
//!                                      [--runs-kv] [--kv-dir DIR] [--save-corpus a,b]
//!                                      [--clm] [--gliner] [--agentjev] [--paw]
//!                                      [--paw-local] [--cascade]
//!                                      [--cascade-worthiness-lcb F]
//!                                      [--gate-fit-selection] [--gate-distance-only]
//! ```
//! `--laya-python` adds the ORIGINAL torch reference as a JSONL subprocess
//! oracle lane (measurement-only; needs python3 + torch/transformers and the
//! weights in the shared cache — absent pieces are loud absences, never
//! silent skips).
//! `--clm` adds the CLM comparison lane (Issue 019 T3 / `.issues/027`):
//! the external Contrastive-LM reference over `/v1/systemone` (their stack
//! serving at `CLM_SERVE_URL`, default `http://127.0.0.1:8700`). Needs the
//! `clm-lane` feature; an unreachable server is a loud absence, never a
//! silent skip.
//! `--gliner` adds the GLiNER comparison lane (Issue 029):
//! fastino/GLiNER2.5-Decide as a JSONL subprocess oracle over THEIR gliner2
//! package (`scripts/gliner_lane.py` — the venv needs gliner2 + torch +
//! transformers + peft + accelerate). Env: `GLINER_PYTHON` (the venv
//! python), `GLINER_PY_DEVICE` (default cuda), `GLINER_MODEL`. A missing
//! script/venv is a loud absence, never a silent skip.
//! `--agentjev` adds the AgentJev comparison lane (Issue 025 amendment 4 /
//! `.issues/027`): their `jev_service` (malevrigns/agent-jev @ a965ca8f,
//! Apache-2.0, not affiliated) answered over HTTP at `AGENTJEV_SERVE_URL`
//! (default `http://127.0.0.1:8149`) — their stack serves, our Rust
//! measures. The lane's owned data point: AgentJev's GOLD-LABEL accuracy
//! on our split (their published 79.25% is teacher-argmax agreement — a
//! different protocol). An unreachable server is a loud absence, never a
//! silent skip.
//! `--paw` adds the PAW comparison lane (Issue 033): ProgramAsWeights
//! (MIT SDK, not affiliated) — one program compiled per specced suite from
//! the committed `scripts/paw_specs/<suite>.txt` (cached by
//! `(suite, compiler, BLAKE3(spec))` in `.raw/paw/programs.json`, so a
//! re-run never recompiles), then one hosted `/api/v1/infer` per question
//! over a `curl` subprocess. `PAW_API_KEY` is OPTIONAL — unset = their
//! anonymous tier (20 compiles/h; programs are PUBLIC on their hub), printed
//! loud. Env: `PAW_API_URL`, `PAW_COMPILER`, `PAW_COMPILE_ASYNC=1`,
//! `PAW_SPECS_DIR`, `PAW_PROGRAM_CACHE`, `PAW_CURL`. Free-text answers are
//! mapped by the exact-match law; unparseable = a counted refusal, never a
//! guess. Suites without a spec are a loud absence.
//! `--paw-local` adds the PAW Posture B lane (Issue 033): the SAME
//! compiled programs answered through their LOCAL llama.cpp runtime
//! (`paw.function(program_id)` over the `programasweights` package as a
//! Python subprocess oracle — the gliner-lane shape). The lane NEVER
//! compiles: the program id comes from the hosted lane's cache, so the
//! local-vs-hosted delta isolates the runtime posture on identical
//! artifacts. Greedy by construction; determinism re-verified per run
//! (observed-repeat, first 10). Env: `PAW_LOCAL_PYTHON` (the venv
//! interpreter, e.g. `.raw/paw-env/Scripts/python.exe`), `PAW_LOCAL_SCRIPT`
//! (default `scripts/paw_local_lane.py`), `PAW_GPU_LAYERS`/`PAW_LOCAL_N_CTX`
//! (their loader's knobs). One-time setup: the venv + a `--paw` run for the
//! program cache (`scripts/paw_preload.py` warms the base download).
//! `--runs-kv` appends ONE Warm-tier row per run via the released `ndb`
//! binary (table `harness_runs`, value = the exact results.json bytes) and
//! `--save-corpus` stores each named suite's dataset as ONE digest-pinned
//! row — both need the `corpus_db` feature + the binary (NDB_BIN or PATH);
//! an explicit flag without either refuses LOUD, never silently skips
//! (Issue 007 P1).
//! `--corpus-cap N` pins every dataset suite's per-label corpus cap
//! (measurement-only, Issue 013 lever 1). `--cal-select-cap [LIST]` is the
//! protocol-clean alternative: cal accuracy is measured at each candidate
//! cap (LIST, or the default ladder 8,16,32,64,128,256,512 — the registry
//! default always joins), the argmax is picked on the CAL SLICE ONLY, and
//! the test split is read once at the selected cap. The two are mutually
//! exclusive (run() refuses the combination).
//! `--cascade` (needs `laya-riir`, mutually exclusive with `--skip-laya`)
//! adds the cascade lane (issue 038 T4′): the modelless answers stand;
//! the calibrated fused gate's abstains escalate to each served riir-laya
//! checkpoint. Accuracy AND the escalation rate per suite (the rate is
//! the latency claim); LLM-only families get an honest absence.
//! `--cascade-worthiness` (needs `--cascade`, issue 042 lever 3) gates
//! the escalation per suite by a cal-slice probe: the checkpoint answers
//! the cal questions the calibrated gate abstained on, and the suite's
//! escalation stays armed only where it reads ≥ the forced modelless
//! picks (`--cascade-worthiness-margin <F64>`, default 0.16 — the
//! Bench-063/066 measured bar, promoted per the issue's own trigger; 0.0
//! restores arm-at-parity). A negative
//! probe disarms the suite's escalation (disclosed in the row); missing
//! cal records / thin support stay armed with the named reason.
//!
//! `--cascade-worthiness-lcb <F64>` (needs `--cascade-worthiness`,
//! issue 046 lever 4) adds a support-aware arm leg: the suite also arms
//! where the probe delta's one-sided-95% lower bound ≥ F — the
//! Bench-070 verified answer to the fused family's ag_news/massive
//! magnitude tie (support separates what the margin provably cannot;
//! conservative on paired data; the Bench-063 preregistered floor is
//! 0.05).
//!
//! `--gate-fit-selection` (issue 042 lever 1) fits the fused-gate
//! thresholds on the STRATIFIED selection slice instead of the train-tail
//! cal slice — the shared held-out instrument, with the probe corpus
//! excluding the fit docs so the observed geometry matches the test side.
//! `--gate-distance-only` (issue 042 lever 2) disables the gate's score
//! axis (threshold 0.0): abstain/escalation runs on the corpus-distance
//! axis alone, at its fitted rho=30% threshold — the transfer-stable
//! axis (Bench 061). Both are gate postures, composable with the cascade
//! lane; default off = the shipped T1.6 fused fit.
//! `--e0` (needs `nb_scope`) runs the riir-instinct Issue-005 E0
//! evidence-density measurement INSTEAD of the lanes: per dataset suite, on
//! the stratified selection slice, the distribution of seen-token counts
//! over the deployed count tables + the rumor fraction — report only, no
//! gold, no test-row eval. Writes `e0.json` + `E0.md` into `--out`.
//! Writes `results.json` + `TABLES.md` into `--out`
//! (default `.benchmarks/001_phase1_tables/`). Datasets come from
//! `.raw/datasets/` (scripts/fetch_datasets.sh). Exit 0 iff every requested
//! suite ran BOTH lanes clean — absences are printed loud and listed in the
//! tables, never silently dropped (the frontier-report law).

use riir_reflex::harness::runner::{self, DEFAULT_DATASETS_DIR, RunOptions};

/// Harness stack size. The runner keeps several `DecisionEngine<N, 256>`
/// values live per suite (probe, raw, cal, fitted, plus issue 038's
/// transductive build), each carrying `N` experts by value. On the
/// 77-domain banking77 that overflowed the 1 MiB Windows main-thread stack
/// (measured on the 4090, 2026-09-26) while the 8 MiB macOS default fit.
/// One explicit size for every platform instead of per-OS luck.
const HARNESS_STACK_BYTES: usize = 64 << 20;

fn main() {
    let worker = std::thread::Builder::new()
        .name("harness".to_string())
        .stack_size(HARNESS_STACK_BYTES)
        .spawn(harness_main)
        .expect("spawn the harness thread");
    if worker.join().is_err() {
        // The panic message already printed on the worker thread.
        std::process::exit(101);
    }
}

fn harness_main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = RunOptions {
        datasets_dir: std::env::current_dir()
            .map(|d| d.join(DEFAULT_DATASETS_DIR))
            .unwrap_or_else(|_| DEFAULT_DATASETS_DIR.into()),
        suites: Vec::new(),
        laya_max_questions: 0,
        skip_laya: false,
        cascade: false,
        cascade_worthiness: false,
        // Issue 042 T3 (a), promoted per the issue's own trigger (a second
        // independent lane run reproduced 10/10 at 0.16 — byte-identical
        // probe deltas, recorded in the 042 HISTORY row): the measured
        // arm bar sits in the (0.150, 0.288] gap that separated every
        // probe family measured (Bench 063 fused, Bench 066 distance +
        // combined). An explicit --cascade-worthiness-margin overrides.
        cascade_worthiness_margin: 0.16,
        // Issue 046 lever 4: the support-aware arm leg — off by default
        // (byte-identical with the lever-3 lane).
        cascade_worthiness_lcb: None,
        gate_fit_selection: false,
        gate_distance_only: false,
        laya_python: false,
        clm: false,
        gliner: false,
        agentjev: false,
        paw: false,
        paw_local: false,
        corpus_cap_override: 0,
        cal_select_caps: Vec::new(),
        pair_head_ab: false,
        nli_feature_ab: false,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        oc_select: false,
        ridge_select: false,
        genome_select: false,
        genome_accept_margin: 0.05,
    };
    let mut out_dir = std::path::PathBuf::from(".benchmarks/001_phase1_tables");
    let mut runs_kv = false;
    let mut kv_dir: Option<std::path::PathBuf> = None;
    let mut save_corpus: Vec<String> = Vec::new();
    let mut e0 = false;
    let mut distill = false;
    // Consumed only by the laya-riir-gated --distill block below; the
    // default-features build would carry them as dead stores (clippy -D).
    #[cfg(feature = "laya-riir")]
    let mut distill_out = std::path::PathBuf::from(".raw/distill_teacher");
    #[cfg(feature = "laya-riir")]
    let mut distill_limit = 0usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--suites" => {
                i += 1;
                opts.suites = args
                    .get(i)
                    .unwrap_or_else(|| die("--suites needs a comma list"))
                    .split(',')
                    .map(str::to_string)
                    .collect();
            }
            "--laya-max-questions" => {
                i += 1;
                opts.laya_max_questions = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--laya-max-questions needs a number"));
            }
            "--skip-laya" => opts.skip_laya = true,
            "--cascade" => opts.cascade = true,
            "--cascade-worthiness" => opts.cascade_worthiness = true,
            "--gate-fit-selection" => opts.gate_fit_selection = true,
            "--gate-distance-only" => opts.gate_distance_only = true,
            "--cascade-worthiness-margin" => {
                i += 1;
                opts.cascade_worthiness_margin = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--cascade-worthiness-margin needs a number"));
            }
            "--cascade-worthiness-lcb" => {
                i += 1;
                opts.cascade_worthiness_lcb = Some(
                    args.get(i)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or_else(|| die("--cascade-worthiness-lcb needs a number")),
                );
            }
            "--pair-head-ab" => opts.pair_head_ab = true,
            "--nli-feature-ab" => opts.nli_feature_ab = true,
            "--head-scale" => {
                i += 1;
                opts.head_scale = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--head-scale needs a number (0 = off)"));
            }
            "--head-select" => opts.head_select = true,
            "--nb-select" => opts.nb_select = true,
            "--oc-select" => opts.oc_select = true,
            "--ridge-select" => opts.ridge_select = true,
            "--genome-select" => opts.genome_select = true,
            "--genome-accept-margin" => {
                i += 1;
                opts.genome_accept_margin = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--genome-accept-margin needs a fraction (e.g. 0.03)"));
            }
            "--e0" => e0 = true,
            "--distill" => distill = true,
            #[cfg(feature = "laya-riir")]
            "--distill-out" => {
                i += 1;
                distill_out = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--distill-out needs a path"));
            }
            #[cfg(not(feature = "laya-riir"))]
            "--distill-out" => {
                i += 1; // value skipped; --distill itself refuses below
            }
            #[cfg(feature = "laya-riir")]
            "--limit" => {
                i += 1;
                distill_limit = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--limit needs a number (0 = the whole split)"));
            }
            #[cfg(not(feature = "laya-riir"))]
            "--limit" => {
                i += 1; // value skipped; --distill itself refuses below
            }
            "--laya-python" => opts.laya_python = true,
            "--clm" => opts.clm = true,
            "--gliner" => opts.gliner = true,
            "--agentjev" => opts.agentjev = true,
            "--paw" => opts.paw = true,
            "--paw-local" => opts.paw_local = true,
            "--corpus-cap" => {
                i += 1;
                opts.corpus_cap_override = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--corpus-cap needs a number (0 = registry default)"));
            }
            "--cal-select-cap" => {
                // Optional comma list; the bare flag = the default ladder.
                // A following non-numeric arg (e.g. --skip-laya) stays put.
                let list = args.get(i + 1).filter(|v| {
                    !v.is_empty() && v.split(',').all(|p| p.trim().parse::<usize>().is_ok())
                });
                let parsed = match list {
                    Some(v) => {
                        i += 1;
                        runner::parse_cal_select_caps(Some(v))
                    }
                    None => runner::parse_cal_select_caps(None),
                };
                opts.cal_select_caps = parsed.unwrap_or_else(|e| die(&e));
            }
            "--out" => {
                i += 1;
                out_dir = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--out needs a path"));
            }
            "--datasets-dir" => {
                // Issue 038 T2: read a non-canonical dataset pull (e.g. a
                // TRAIN_CAP=20000 fetch into its own dir). Disclosed in the
                // run meta's datasets line like the default.
                i += 1;
                opts.datasets_dir = args
                    .get(i)
                    .map(Into::into)
                    .unwrap_or_else(|| die("--datasets-dir needs a path"));
            }
            "--runs-kv" => runs_kv = true,
            "--kv-dir" => {
                i += 1;
                kv_dir = Some(
                    args.get(i)
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| die("--kv-dir needs a path")),
                );
            }
            "--save-corpus" => {
                i += 1;
                save_corpus = args
                    .get(i)
                    .unwrap_or_else(|| die("--save-corpus needs a comma list"))
                    .split(',')
                    .map(str::to_string)
                    .collect();
            }
            other => die(&format!("unknown flag {other}")),
        }
        i += 1;
    }

    // The kv flags are EXPLICIT ops — a compile-time-missing feature is a
    // loud refusal naming the rebuild, never an unknown-flag error.
    if (runs_kv || !save_corpus.is_empty()) && !cfg!(feature = "corpus_db") {
        die(
            "--runs-kv / --save-corpus need the `corpus_db` feature — rebuild: \
             cargo build --release --features corpus_db --bin harness",
        );
    }
    #[cfg(not(feature = "corpus_db"))]
    let _ = kv_dir;

    // Issue 005 (riir-instinct) E0: an EXCLUSIVE early-exit measurement
    // mode — evidence density on the stratified selection slice, report
    // only, before any lane machinery runs. Needs `nb_scope` (it reads the
    // count tables' fit-time seen set); a missing feature is a loud refusal
    // naming the rebuild (the build-stamp law).
    #[cfg(feature = "nb_scope")]
    if e0 {
        println!(
            "harness --e0: datasets {} · suites {:?}",
            opts.datasets_dir.display(),
            opts.suites
        );
        let out = match runner::run_e0(&opts) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            die(&format!("create {}: {e}", out_dir.display()));
        }
        let json_path = out_dir.join("e0.json");
        let md_path = out_dir.join("E0.md");
        let json = serde_json::to_string_pretty(&out).expect("e0 serialize");
        let md = runner::render_e0_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --e0: absence: {e}");
        }
        println!(
            "harness --e0: PASSED — {} suite(s) measured, {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    #[cfg(not(feature = "nb_scope"))]
    if e0 {
        die(
            "--e0 needs the `nb_scope` feature — rebuild: cargo build --release \
             --features nb_scope --bin harness",
        );
    }
    if e0 && distill {
        die("--e0 and --distill are both exclusive early-exit modes — pass one");
    }

    // riir-train Issue 576 T3: the Arm-B TEACHER pass — laya probabilities
    // over the train rows of the six Arm-A suites, dumped as frozen data
    // (magic `RIDT` + a `.blake3` sidecar per suite) for the distillation
    // student in ../riir-train. Early-exit like --e0: no eval lane, no test
    // row. Needs `laya-riir` (the lane); a missing feature is a loud refusal
    // naming the rebuild (the build-stamp law).
    #[cfg(feature = "laya-riir")]
    if distill {
        println!(
            "harness --distill: datasets {} · suites {:?} · limit {} · out {}",
            opts.datasets_dir.display(),
            opts.suites,
            distill_limit,
            distill_out.display()
        );
        let out = match runner::run_distill(&opts, &distill_out, distill_limit) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&distill_out) {
            die(&format!("create {}: {e}", distill_out.display()));
        }
        let json_path = distill_out.join("distill.json");
        let md_path = distill_out.join("DISTILL.md");
        let json = serde_json::to_string_pretty(&out).expect("distill serialize");
        let md = runner::render_distill_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --distill: absence: {e}");
        }
        println!(
            "harness --distill: PASSED — {} suite(s) dumped, {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    #[cfg(not(feature = "laya-riir"))]
    if distill {
        die(
            "--distill needs the `laya-riir` feature — rebuild: cargo build --release \
             --features laya-riir-metal --bin harness (macOS; elsewhere: laya-riir)",
        );
    }

    // The clm flag is an EXPLICIT lane request — a compile-time-missing
    // feature is a loud refusal naming the rebuild (the build-stamp law),
    // never a silent column of absences.
    if opts.clm && !cfg!(feature = "clm-lane") {
        die(
            "--clm needs the `clm-lane` feature — rebuild: cargo build --release \
             --features clm-lane --bin harness (and their stack serving at \
             CLM_SERVE_URL, default http://127.0.0.1:8700 — .issues/027)",
        );
    }

    println!(
        "harness: datasets {} · suites {:?}",
        opts.datasets_dir.display(),
        opts.suites
    );
    let (output, errors) = match runner::run(&opts) {
        Ok(r) => r,
        Err(e) => die(&e),
    };

    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        die(&format!("create {}: {e}", out_dir.display()));
    }
    let json_path = out_dir.join("results.json");
    let md_path = out_dir.join("TABLES.md");
    let json = serde_json::to_string_pretty(&output).expect("results serialize");
    let md = runner::render_markdown(&output, &errors);
    if let Err(e) = std::fs::write(&json_path, json) {
        die(&format!("write {}: {e}", json_path.display()));
    }
    if let Err(e) = std::fs::write(&md_path, &md) {
        die(&format!("write {}: {e}", md_path.display()));
    }
    println!("harness: wrote {}", json_path.display());
    println!("harness: wrote {}", md_path.display());

    // Issue 007 P1: ONE run-history row per run — the exact results.json
    // bytes under a sortable date/sha/time key. Explicit-flag failures die
    // AFTER the files are on disk (the run happened; the persistence failed
    // — both facts are loud).
    #[cfg(feature = "corpus_db")]
    if runs_kv && let Err(e) = write_run_row(&output, &json_path, kv_dir.as_deref()) {
        die(&e);
    }
    #[cfg(feature = "corpus_db")]
    for name in &save_corpus {
        if let Err(e) = save_corpus_row(&opts.datasets_dir, name, kv_dir.as_deref()) {
            die(&e);
        }
    }

    if !errors.is_empty() {
        eprintln!("harness: {} absence(s)/error(s):", errors.len());
        for e in &errors {
            eprintln!("  - {e}");
        }
    }
    // Clean run = every suite present with BOTH lanes. Absences make the
    // run LOUD, never a silent green.
    if errors.is_empty() {
        println!(
            "harness: PASSED — {} suite(s), no absences",
            output.suites.len()
        );
    } else {
        println!(
            "harness: PASSED WITH ABSENCES — {} suite(s), {} absence(s) listed above and in TABLES.md",
            output.suites.len(),
            errors.len()
        );
    }
}

#[cfg(feature = "corpus_db")]
fn write_run_row(
    output: &riir_reflex::harness::runner::RunOutput,
    json_path: &std::path::Path,
    kv_dir: Option<&std::path::Path>,
) -> Result<(), String> {
    use riir_reflex::harness::corpus_db::{NdbRunStore, RUNS_TABLE};
    let kv_path = kv_dir
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(default_kv_dir);
    let store = NdbRunStore::resolve(&kv_path).map_err(|e| format!("--runs-kv: {e}"))?;
    let version = store.probe().map_err(|e| format!("--runs-kv: {e}"))?;
    let bytes = std::fs::read(json_path)
        .map_err(|e| format!("--runs-kv: read {}: {e}", json_path.display()))?;
    let key = format!(
        "{}/{}/{}",
        output.meta.date_utc,
        &output.meta.git_sha[..output.meta.git_sha.len().min(8)],
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let ack = store
        .put_row(RUNS_TABLE, &key, &bytes)
        .map_err(|e| format!("--runs-kv: {e}"))?;
    println!(
        "harness: runs.kv row {key} ({} bytes, ndb {}, store {})",
        ack.bytes,
        version.version,
        store.store_dir().display()
    );
    Ok(())
}

#[cfg(feature = "corpus_db")]
fn save_corpus_row(
    datasets_dir: &std::path::Path,
    name: &str,
    kv_dir: Option<&std::path::Path>,
) -> Result<(), String> {
    use riir_reflex::harness::corpus_db::NdbRunStore;
    let kv_path = kv_dir
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(default_kv_dir);
    let store = NdbRunStore::resolve(&kv_path).map_err(|e| format!("--save-corpus {name}: {e}"))?;
    let envelope = runner::load_suite_envelope(datasets_dir, name)
        .map_err(|e| format!("--save-corpus {name}: {e}"))?;
    let blob = serde_json::to_vec(&envelope)
        .map_err(|e| format!("--save-corpus {name}: serialize: {e}"))?;
    let (ack, digest) = store
        .put_corpus(name, &blob)
        .map_err(|e| format!("--save-corpus {name}: {e}"))?;
    // Read-back verification: the stored row must digest back to the pin.
    let read_back = store
        .get_corpus(name, &digest)
        .map_err(|e| format!("--save-corpus {name}: verify: {e}"))?;
    if read_back != blob {
        return Err(format!(
            "--save-corpus {name}: read-back digest mismatch — the stored row is \
             not the corpus that was written; refusing to report success"
        ));
    }
    println!(
        "harness: corpus row {}/{} ({} bytes, blake3 {}…, store {})",
        name,
        ack.store_key,
        ack.bytes,
        &digest[..16],
        store.store_dir().display()
    );
    Ok(())
}

#[cfg(feature = "corpus_db")]
fn default_kv_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(".harness/ndb-data")
}

fn die(msg: &str) -> ! {
    eprintln!("harness: {msg}");
    std::process::exit(2);
}
