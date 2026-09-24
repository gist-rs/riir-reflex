//! `harness` — the Plan 603 T1.5 runner CLI.
//!
//! Usage:
//! ```text
//! cargo run --release --bin harness -- [--suites a,b] [--laya-max-questions N]
//!                                      [--skip-laya] [--laya-python] [--out DIR]
//!                                      [--corpus-cap N] [--cal-select-cap [LIST]]
//!                                      [--runs-kv] [--kv-dir DIR] [--save-corpus a,b]
//!                                      [--clm]
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
//! Writes `results.json` + `TABLES.md` into `--out`
//! (default `.benchmarks/001_phase1_tables/`). Datasets come from
//! `.raw/datasets/` (scripts/fetch_datasets.sh). Exit 0 iff every requested
//! suite ran BOTH lanes clean — absences are printed loud and listed in the
//! tables, never silently dropped (the frontier-report law).

use riir_reflex::harness::runner::{self, DEFAULT_DATASETS_DIR, RunOptions};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = RunOptions {
        datasets_dir: std::env::current_dir()
            .map(|d| d.join(DEFAULT_DATASETS_DIR))
            .unwrap_or_else(|_| DEFAULT_DATASETS_DIR.into()),
        suites: Vec::new(),
        laya_max_questions: 0,
        skip_laya: false,
        laya_python: false,
        clm: false,
        corpus_cap_override: 0,
        cal_select_caps: Vec::new(),
        pair_head_ab: false,
    };
    let mut out_dir = std::path::PathBuf::from(".benchmarks/001_phase1_tables");
    let mut runs_kv = false;
    let mut kv_dir: Option<std::path::PathBuf> = None;
    let mut save_corpus: Vec<String> = Vec::new();
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
            "--pair-head-ab" => opts.pair_head_ab = true,
            "--laya-python" => opts.laya_python = true,
            "--clm" => opts.clm = true,
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
