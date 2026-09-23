//! `harness` — the Plan 603 T1.5 runner CLI.
//!
//! Usage:
//! ```text
//! cargo run --release --bin harness -- [--suites a,b] [--laya-max-questions N]
//!                                      [--skip-laya] [--out DIR]
//! ```
//! Writes `results.json` + `TABLES.md` into `--out`
//! (default `.benchmarks/001_phase1_tables/`). Datasets come from
//! `.raw/datasets/` (scripts/fetch_datasets.sh). Exit 0 iff every requested
//! suite ran BOTH lanes clean — absences are printed loud and listed in the
//! tables, never silently dropped (the frontier-report law).

use riir_reflex::harness::runner::{self, RunOptions, DEFAULT_DATASETS_DIR};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = RunOptions {
        datasets_dir: std::env::current_dir()
            .map(|d| d.join(DEFAULT_DATASETS_DIR))
            .unwrap_or_else(|_| DEFAULT_DATASETS_DIR.into()),
        suites: Vec::new(),
        laya_max_questions: 0,
        skip_laya: false,
    };
    let mut out_dir = std::path::PathBuf::from(".benchmarks/001_phase1_tables");
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
            "--out" => {
                i += 1;
                out_dir = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--out needs a path"));
            }
            other => die(&format!("unknown flag {other}")),
        }
        i += 1;
    }

    println!("harness: datasets {} · suites {:?}", opts.datasets_dir.display(), opts.suites);
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

    if !errors.is_empty() {
        eprintln!("harness: {} absence(s)/error(s):", errors.len());
        for e in &errors {
            eprintln!("  - {e}");
        }
    }
    // Clean run = every suite present with BOTH lanes. Absences make the
    // run LOUD, never a silent green.
    if errors.is_empty() {
        println!("harness: PASSED — {} suite(s), no absences", output.suites.len());
    } else {
        println!(
            "harness: PASSED WITH ABSENCES — {} suite(s), {} absence(s) listed above and in TABLES.md",
            output.suites.len(),
            errors.len()
        );
    }
}

fn die(msg: &str) -> ! {
    eprintln!("harness: {msg}");
    std::process::exit(2);
}
