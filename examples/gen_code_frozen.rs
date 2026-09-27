//! The frozen code_fixtures population GENERATOR (Issue 044 T4).
//!
//! Harvests the live tree via [`riir_reflex::harness::code_frozen`]'s
//! extractor (the ORIGINAL `extract_fns` semantics, moved verbatim),
//! applies the slice law, canonicalizes, and writes the BLAKE3-pinned
//! fixture `src/harness/code_fixtures_frozen.json`. Run from the repo
//! root:
//!
//! ```sh
//! cargo run --features modelless --example gen_code_frozen
//! ```
//!
//! The fixture is COMMITTED data: a regeneration is a deliberate act that
//! changes the published suite's meaning (new digest row in the same
//! commit, the old number marked historical). Never edit the JSON by hand
//! — the digest check refuses it at parse time.

use riir_reflex::harness::code_frozen::{harvest_live, slice_module, FrozenModule};

fn main() {
    let harvested = harvest_live();
    let mut modules: Vec<FrozenModule> = Vec::with_capacity(harvested.len());
    println!("harvest (per module: total / eval / cal / docs):");
    for (path, label, fns) in &harvested {
        if fns.len() < 16 {
            // A thin module would re-create the corpus-fallback rot the
            // freeze exists to prevent — refuse rather than bake it.
            eprintln!(
                "REFUSING: {path} ({label}) yields only {} extract units — the freeze \
                 requires ≥16 (2 eval + 8 cal + 6 docs). Curate a healthier module in \
                 HARVEST_MODULES and re-run.",
                fns.len()
            );
            std::process::exit(1);
        }
        let (eval, cal, docs) = slice_module(fns);
        println!(
            "  {label:24} <- {path:28} total {:>3}  eval 2  cal 8  docs 6",
            fns.len()
        );
        modules.push(FrozenModule {
            path: path.clone(),
            label: label.clone(),
            eval,
            cal,
            docs,
        });
    }

    let canonical = serde_json::to_vec(&modules).expect("canonicalize");
    let digest = blake3::hash(&canonical).to_hex().to_string();

    let (from, utc) = tree_provenance();
    let file = serde_json::json!({
        "digest": digest,
        "generated_from": from,
        "generated_utc": utc,
        "modules": modules,
    });
    let rendered = serde_json::to_string_pretty(&file).expect("render") + "\n";
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/src/harness/code_fixtures_frozen.json");
    std::fs::write(out, rendered).expect("write fixture");
    println!("digest {digest}");
    println!("wrote {out}");
}

/// Tree provenance: HEAD sha + branch, UTC timestamp (best effort — the
/// fields are disclosure, not the pin; the digest is).
fn tree_provenance() -> (String, String) {
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let utc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let day = utc / 86_400;
    let rem = utc % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Civil-from-days (Howard Hinnant's algorithm) — deterministic, no
    // chrono dep.
    let z = day as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mth = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mth <= 2 { y + 1 } else { y };
    (
        format!("{sha} ({branch})"),
        format!("{y:04}-{mth:02}-{d:02}T{h:02}:{m:02}:{s:02}Z"),
    )
}
