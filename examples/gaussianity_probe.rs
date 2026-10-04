//! Issue 066 PRE-CHECK — is a diagonal Gaussian the right model class for
//! the seat corpus's hashed-bag embedding space, BEFORE any LSL-gate A/B
//! number is read?
//!
//! The LSL App-E theory (arXiv:2610.02126) wants a density-side gate input
//! `ℓ(x) = log Φ_suite(x) − log Φ_ref(x)` over a diagonal GMM pair. A
//! diagonal GMM is a per-component independence assumption — hashed-bag
//! features are sparse, signed, and heavily discretized, so the honest
//! first move (the issue's own PRE-CHECK task) is to run katgpt-core's
//! `sketched_gaussianity` on the REAL populations the gate would model:
//!
//! - **whole-corpus** — expected to REJECT when labels are mixed (a mixture
//!   of clusters is multimodal by construction; this row is the ceiling,
//!   not the verdict);
//! - **per-label pools** — the per-COMPONENT question a diagonal GMM with
//!   per-label components actually needs; THIS is the row that decides the
//!   model class;
//! - **JL projections** (Rademacher ±1/√k, splitmix64-seeded — zero new
//!   deps) at k ∈ {64, 32, 16} — the issue's "try the JL-projected variant"
//!   arm: projection smoothing is the classic rescue when raw sparse
//!   features reject (CLT over the projected sum).
//!
//! Determinism end to end (pure embedding + fixed-seed projection + the
//! probe's own fixed direction table): two runs are byte-identical.
//!
//! Report-first: prints a per-suite × per-posture table + a verdict block;
//! exit 0 on a real sweep, exit 1 on unreadable data (never a green zero).
//! USAGE:
//! ```text
//! cargo run --release --features gaussianity_probe --example gaussianity_probe -- \
//!     --datasets-dir .raw/datasets_t20k [--suites a,b] [--jl 64,32,16] [--max-per-label 400]
//! ```

// The belt behind the [[example]] required-features row (the
// laya_fixture_timing law): a featureless direct compile is an empty crate,
// never an E0601.
#![cfg(feature = "gaussianity_probe")]

use std::collections::BTreeMap;

use katgpt_core::data_probe::gaussianity::{sketched_gaussianity, GaussianityScratch};
use riir_reflex::embed::{Embedder, EMBED_DIM};
use riir_reflex::harness::runner::load_suite_envelope;
use riir_reflex::harness::suites::train_docs;

/// splitmix64 — the deterministic JL sign source (fixed seed, no deps).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Project `rows` (n × EMBED_DIM) to n × k with a fixed-seed Rademacher
/// matrix scaled 1/√k. Deterministic: same (rows, k, seed) → identical bits.
fn jl_project(rows: &[[f32; EMBED_DIM]], k: usize, seed: u64) -> Vec<Vec<f32>> {
    let scale = 1.0f32 / (k as f32).sqrt();
    let mut rng = seed;
    // k × EMBED_DIM sign table (row-major), consumed in fixed order.
    let mut signs = vec![0u8; k * EMBED_DIM];
    for s in signs.iter_mut() {
        *s = (splitmix64(&mut rng) & 1) as u8;
    }
    rows.iter()
        .map(|row| {
            (0..k)
                .map(|j| {
                    let mut acc = 0.0f32;
                    let base = j * EMBED_DIM;
                    for (i, v) in row.iter().enumerate() {
                        let s = if signs[base + i] == 1 { 1.0f32 } else { -1.0f32 };
                        acc += v * s;
                    }
                    acc * scale
                })
                .collect()
        })
        .collect()
}

/// One probe reading over a flat n × d population (n ≥ 8 to keep the KS
/// p-value meaningful; smaller pools are reported as `n/a`, never scored).
fn probe(rows: &[Vec<f32>]) -> Option<(f32, f32)> {
    if rows.len() < 8 {
        return None;
    }
    let d = rows.first()?.len();
    let n = rows.len();
    let mut flat = Vec::with_capacity(n * d);
    for r in rows {
        flat.extend_from_slice(r);
    }
    let mut scratch = GaussianityScratch::new(n, d, 7);
    let rep = sketched_gaussianity(&flat, &mut scratch);
    Some((rep.score, rep.min_p_value))
}

fn main() {
    let mut datasets_dir = ".raw/datasets_t20k".to_string();
    let mut suites = "massive_intent_en,banking77,ag_news,emotion,sst5,xnli_en".to_string();
    let mut jl_dims = "64,32,16".to_string();
    let mut max_per_label = 400usize;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--datasets-dir" => datasets_dir = args.next().unwrap_or_default(),
            "--suites" => suites = args.next().unwrap_or_default(),
            "--jl" => jl_dims = args.next().unwrap_or_default(),
            "--max-per-label" => {
                max_per_label = args.next().and_then(|v| v.parse().ok()).unwrap_or(400)
            }
            other => {
                eprintln!("unknown arg {other}");
                std::process::exit(1);
            }
        }
    }
    let jl: Vec<usize> = jl_dims.split(',').filter_map(|d| d.parse().ok()).collect();
    if jl.is_empty() {
        eprintln!("--jl produced no dims");
        std::process::exit(1);
    }

    println!("# gaussianity PRE-CHECK (issue 066) — model class before any A/B");
    println!("# datasets_dir={datasets_dir} jl={jl:?} max_per_label={max_per_label} probe_seed=7");
    println!("# score > 0.5 accepts Gaussianity (katgpt-core's own fixture threshold)");

    let embedder = Embedder;
    let mut any_suite_ran = false;
    let mut failures = 0usize;

    for suite in suites.split(',').filter(|s| !s.is_empty()) {
        let env = match load_suite_envelope(std::path::Path::new(&datasets_dir), suite) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("REFUSE — suite {suite}: {e}");
                failures += 1;
                continue;
            }
        };
        let train = env.get("train_rows").cloned().unwrap_or(serde_json::Value::Null);
        let docs = train_docs(&train, suite);
        if docs.is_empty() {
            eprintln!("REFUSE — suite {suite}: zero train docs");
            failures += 1;
            continue;
        }
        any_suite_ran = true;

        // Embed the corpus once (deterministic), group per label.
        let mut by_label: BTreeMap<String, Vec<[f32; EMBED_DIM]>> = BTreeMap::new();
        let mut zero_vecs = 0usize;
        for doc in &docs {
            let mut v = [0f32; EMBED_DIM];
            embedder.embed_into(doc.text.as_bytes(), &mut v);
            if v.iter().all(|x| *x == 0.0) {
                zero_vecs += 1;
            }
            by_label.entry(doc.label.clone()).or_default().push(v);
        }
        // Cap per-label pools (deterministic: first-N in corpus order — the
        // probe measures shape, not the exact pool identity).
        for pool in by_label.values_mut() {
            pool.truncate(max_per_label);
        }
        let all: Vec<[f32; EMBED_DIM]> = by_label.values().flatten().copied().collect();
        let labels = by_label.len();

        println!("\n## {suite}: {n} docs · {labels} labels · {zero_vecs} zero-vec embeddings (disclosed)", n = docs.len());

        // Whole-corpus raw.
        let whole: Vec<Vec<f32>> = all.iter().map(|r| r.to_vec()).collect();
        match probe(&whole) {
            Some((score, p)) => {
                let verdict = if score > 0.5 { "ACCEPT" } else { "REJECT" };
                println!("  whole-corpus  raw-d{d:<4} score={score:.4} p_min={p:.2e}  {verdict}", d = EMBED_DIM)
            }
            None => println!("  whole-corpus  raw-d{d:<4} n/a (n < 8)", d = EMBED_DIM),
        }
        // Whole-corpus JL arms.
        for k in &jl {
            let proj = jl_project(&all, *k, jl_seed(*k));
            match probe(&proj) {
                Some((score, p)) => println!(
                    "  whole-corpus  jl-d{k:<4} score={score:.4} p_min={p:.2e}  {}",
                    if score > 0.5 { "ACCEPT" } else { "REJECT" }
                ),
                None => println!("  whole-corpus  jl-d{k:<4} n/a"),
            }
        }
        // Per-label (the GMM component question) — summary stats over labels.
        for posture in std::iter::once(None).chain(jl.iter().map(Some)) {
            let mut scores = Vec::new();
            let mut skipped = 0usize;
            for pool in by_label.values() {
                let rows: Vec<Vec<f32>> = match posture {
                    None => pool.iter().map(|r| r.to_vec()).collect(),
                    Some(k) => jl_project(pool, *k, jl_seed(*k)),
                };
                match probe(&rows) {
                    Some((score, _)) => scores.push(score),
                    None => skipped += 1,
                }
            }
            if scores.is_empty() {
                println!("  per-label     {pl:<8} n/a (no pool >= 8)", pl = posture_label(posture));
                continue;
            }
            scores.sort_by(|a, b| a.total_cmp(b));
            let median = scores[scores.len() / 2];
            let accepting = scores.iter().filter(|s| **s > 0.5).count();
            let p10 = scores[scores.len() / 10];
            let p90 = scores[(scores.len() * 9) / 10];
            let skipped_note = if skipped > 0 {
                format!(" (+{skipped} pools < 8 skipped)")
            } else {
                String::new()
            };
            println!(
                "  per-label     {pl:<8} median={median:.4} accept {accepting}/{n} labels  [p10={p10:.4} p90={p90:.4}]{skipped_note}",
                pl = posture_label(posture),
                n = scores.len()
            );
        }
    }

    if !any_suite_ran || failures > 0 {
        eprintln!("REFUSE — no suite produced a reading (failures: {failures})");
        std::process::exit(1);
    }
    println!("\n# verdict: read the per-label rows — a diagonal GMM with per-label components");
    println!("# is adjudicated THERE (whole-corpus REJECT is the label-mixture ceiling, not");
    println!("# the model-class answer). If per-label raw rejects and per-label JL accepts,");
    println!("# the gate must ride the projected space (the issue's JL arm).");
}

fn posture_label(k: Option<&usize>) -> String {
    match k {
        None => "raw-d256".to_string(),
        Some(k) => format!("jl-d{k}"),
    }
}

/// Per-(k) fixed seed so each JL width is its own reproducible posture.
const fn jl_seed(k: usize) -> u64 {
    0x0660_0000_0000_0000u64 ^ (k as u64)
}
