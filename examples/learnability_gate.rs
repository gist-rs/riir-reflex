//! Issue 104 — the learnability-gate class lane (T1 report + T2 gated
//! artifacts).
//!
//! Scores every row of a sealed SYN v2 synth artifact with closed-form
//! trivial-solver margins (prior-frequency, lexical-overlap,
//! longest-option), reports the margin distributions, and derives the
//! gated variants (drop the too-easy pole at the given quantile rungs).
//! The corpus-ab lane (the V5 read) stays untouched — this example only
//! produces the report + the sealed gated artifacts it consumes.
//!
//! Usage:
//!   cargo run --release --example learnability_gate -- <artifact.jsonl> \
//!       --out <dir> [--rungs 0.5,0.75,0.9]

use std::path::PathBuf;

use riir_reflex::harness::runner::{render_learnability_markdown, run_learnability_gate};

fn die(msg: &str) -> ! {
    eprintln!("learnability_gate: {msg}");
    std::process::exit(2);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(artifact) = args.next() else {
        die("usage: learnability_gate <artifact.jsonl> --out <dir> [--rungs 0.5,0.75,0.9]");
    };
    if artifact.starts_with('-') {
        die("first argument must be the artifact path");
    }
    let mut out = PathBuf::from(".raw/corpus_synth_learn104");
    let mut rungs: Vec<f64> = vec![0.5, 0.75, 0.9];
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--out" => {
                out = PathBuf::from(
                    args.next()
                        .unwrap_or_else(|| die("--out needs a directory")),
                );
            }
            "--rungs" => {
                let csv = args.next().unwrap_or_else(|| die("--rungs needs a csv"));
                rungs = csv
                    .split(',')
                    .map(|t| {
                        t.trim()
                            .parse::<f64>()
                            .unwrap_or_else(|_| die(&format!("--rungs: {t:?} is not a number")))
                    })
                    .collect();
            }
            other => die(&format!("unknown flag {other:?}")),
        }
    }

    println!(
        "learnability_gate: artifact {} · out {} · rungs {:?}",
        artifact,
        out.display(),
        rungs
    );
    let output = match run_learnability_gate(std::path::Path::new(&artifact), &out, &rungs) {
        Ok(o) => o,
        Err(e) => die(&e),
    };

    std::fs::create_dir_all(&out)
        .unwrap_or_else(|e| die(&format!("create {}: {e}", out.display())));
    let json_path = out.join("learnability_report.json");
    let json = serde_json::to_string_pretty(&output).unwrap_or_else(|e| die(&e.to_string()));
    std::fs::write(&json_path, json).unwrap_or_else(|e| die(&format!("write {json_path:?}: {e}")));
    let md_path = out.join("LEARNABILITY.md");
    let md = render_learnability_markdown(&output);
    std::fs::write(&md_path, &md).unwrap_or_else(|e| die(&format!("write {md_path:?}: {e}")));
    print!("{md}");
    println!(
        "learnability_gate: DONE — {} row(s), {} rung(s); wrote {} + {}",
        output.rows,
        output.rungs.len(),
        json_path.display(),
        md_path.display()
    );
}
