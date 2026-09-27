//! Issue 044 T4 — the FROZEN `code_fixtures` population.
//!
//! The suite's population used to be harvested from THIS repo's own
//! `src/` at every build (`extract_fns` over a fixed module list). That is
//! commit-relative by construction: files move (`laya/agent.rs` and
//! `laya/router.rs` left for riir-infer — two of the eight option labels
//! lost their corpus docs and fell to the self-doc fallback), fns are
//! added and deleted, and the published accuracy moves with the tree —
//! measured: the same binary read 0.2500 at the published run and
//! 0.2917 a few days later, with four of eight labels unanswerable.
//!
//! The fix is the pin: the population is harvested ONCE, committed as
//! [`FROZEN_JSON`] (a BLAKE3-pinned sibling file), and the builders read
//! the frozen bytes. The suite no longer touches the tree — the published
//! number stays comparable forever, and regenerating is a deliberate act
//! (`cargo run --features modelless --example gen_code_frozen`) that
//! changes the fixture and says so in the new digest row.
//!
//! The harvest logic ([`extract_fns`], [`harvest_live`]) is the ORIGINAL
//! extractor moved here verbatim, so a regeneration reproduces the same
//! segmentation for the same bytes; [`slice_module`] carries the slice law
//! (eval = fns[0..2], cal = fns[2..10], docs = fns[10..16] — the eval/cal
//! slices STRICTLY precede the corpus docs, the measured self-inclusion
//! leak rule).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The committed fixture (digest-pinned; see [`frozen_modules`]).
pub const FROZEN_JSON: &str = include_str!("code_fixtures_frozen.json");

/// One frozen function span.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrozenFn {
    pub src: String,
    pub is_pub: bool,
}

/// One frozen module: its provenance path, the option label, and the three
/// disjoint slices (eval / cal / docs).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrozenModule {
    /// The repo-relative source path the span was harvested from (display
    /// provenance — the suite never reads the tree).
    pub path: String,
    /// The option label (the domain name the engine sees).
    pub label: String,
    pub eval: Vec<FrozenFn>,
    pub cal: Vec<FrozenFn>,
    pub docs: Vec<FrozenFn>,
}

/// The parsed, digest-verified fixture (parsed once per process).
pub fn frozen_modules() -> &'static [FrozenModule] {
    static FROZEN: OnceLock<Vec<FrozenModule>> = OnceLock::new();
    FROZEN.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(FROZEN_JSON)
            .expect("code_fixtures_frozen.json: valid JSON");
        let modules = v.get("modules").cloned().unwrap_or(serde_json::Value::Null);
        let expected = v
            .get("digest")
            .and_then(serde_json::Value::as_str)
            .expect("code_fixtures_frozen.json: digest field");
        let canonical = serde_json::to_vec(&modules)
            .expect("code_fixtures_frozen.json: modules re-serialize");
        let got = blake3::hash(&canonical).to_hex();
        assert_eq!(
            got.as_str(),
            expected,
            "code_fixtures_frozen.json: BLAKE3 digest mismatch — the fixture was \
             edited by hand; regenerate with `cargo run --features modelless --example \
             gen_code_frozen` (never edit the JSON directly)"
        );
        serde_json::from_value(modules).expect("code_fixtures_frozen.json: module shapes")
    })
}

/// Provenance strings (for the suite's own disclosure row).
pub fn frozen_provenance() -> &'static (String, String) {
    static PROV: OnceLock<(String, String)> = OnceLock::new();
    PROV.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(FROZEN_JSON)
            .expect("code_fixtures_frozen.json: valid JSON");
        (
            v.get("generated_from")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            v.get("generated_utc")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        )
    })
}

// ── the live harvest (the ORIGINAL extractor, moved verbatim) ──────────

/// One live-extracted function span.
#[derive(Debug, Clone)]
pub struct LiveFn {
    pub src: String,
    pub is_pub: bool,
}

/// Extract top-level `fn` decls from one source file: returns (decl line,
/// body text up to the closing brace at column 0 or a 80-line cap). The
/// historical extractor's exact semantics — an impl-heavy file yields one
/// unit per fn-led 80-line window, and that is the unit the frozen suite
/// speaks in.
#[must_use]
pub fn extract_fns(path: &Path) -> Vec<LiveFn> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut fns = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let is_pub = trimmed.starts_with("pub fn ");
        let is_fn = trimmed.starts_with("fn ");
        if is_pub || is_fn {
            let start = i;
            let mut body = String::new();
            while i < lines.len() && i - start < 80 {
                body.push_str(lines[i]);
                body.push('\n');
                if i > start && lines[i].starts_with('}') {
                    break;
                }
                i += 1;
            }
            fns.push(LiveFn { src: body, is_pub });
        }
        i += 1;
    }
    fns
}

/// The harvest module set: (repo-relative path, option label), in option
/// order. Curated for the freeze (Issue 044 T4): every module yields a
/// FULL slice set (≥16 extract units — 2 eval + 8 cal + 6 docs), so no
/// option label falls to the self-doc fallback the rot produced.
pub const HARVEST_MODULES: &[(&str, &str)] = &[
    ("src/harness/runner.rs", "harness::runner"),
    ("src/harness/suites.rs", "harness::suites"),
    ("src/game_heads.rs", "game_heads"),
    ("src/serve.rs", "serve"),
    ("src/engine.rs", "engine"),
    ("src/lanes/paw.rs", "lanes::paw"),
    ("src/lanes/clm.rs", "lanes::clm"),
    ("src/harness/slice_leak.rs", "harness::slice_leak"),
];

/// The slice law: eval = fns[0..2], cal = fns[2..10], docs = fns[10..16]
/// — eval/cal STRICTLY precede the corpus docs (the measured
/// self-inclusion leak rule).
#[must_use]
pub fn slice_module(fns: &[LiveFn]) -> (Vec<FrozenFn>, Vec<FrozenFn>, Vec<FrozenFn>) {
    let conv = |s: &LiveFn| FrozenFn {
        src: s.src.clone(),
        is_pub: s.is_pub,
    };
    let eval = fns.iter().take(2).map(conv).collect();
    let cal = fns.iter().skip(2).take(8).map(conv).collect();
    let docs = fns.iter().skip(10).take(6).map(conv).collect();
    (eval, cal, docs)
}

/// Harvest the live tree into per-module fn lists (the generator's input).
/// Deliberately NOT compared against the fixture anywhere: the fixture is
/// frozen and the tree moves — divergence is the design, and the only path
/// that changes the fixture is a regeneration.
#[must_use]
pub fn harvest_live() -> Vec<(String, String, Vec<LiveFn>)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    HARVEST_MODULES
        .iter()
        .map(|(rel, label)| {
            let fns = extract_fns(&root.join(rel));
            (rel.to_string(), label.to_string(), fns)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_fixture_parses_and_carries_the_full_slice_shape() {
        let modules = frozen_modules();
        assert_eq!(modules.len(), HARVEST_MODULES.len());
        for m in modules {
            assert_eq!(m.eval.len(), 2, "{}: 2 eval fns", m.label);
            assert_eq!(m.cal.len(), 8, "{}: 8 cal fns", m.label);
            assert_eq!(m.docs.len(), 6, "{}: 6 docs fns", m.label);
            assert!(!m.path.is_empty());
        }
    }

    #[test]
    fn provenance_is_present() {
        let (from, at) = frozen_provenance();
        assert!(!from.is_empty());
        assert!(!at.is_empty());
    }

    #[test]
    fn extract_fns_is_deterministic_and_column0_terminated() {
        // A synthetic file exercises the window rule: the fn-led window
        // stops at the first column-0 '}' (or 80 lines).
        let dir = std::env::temp_dir().join(format!(
            "code_frozen_extract_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("synthetic.rs");
        std::fs::write(
            &path,
            "fn a() -> usize {\n    let x = 1;\n    impl X {\n        fn inner() {}\n    }\n}\n\nfn b() {}\n",
        )
        .expect("write");
        let fns = extract_fns(&path);
        assert_eq!(fns.len(), 2, "the column-0 close brace ends window a; b starts fresh");
        assert!(!fns[0].is_pub);
        assert!(fns[1].src.contains("fn b"));
        let again = extract_fns(&path);
        assert_eq!(fns.len(), again.len());
        std::fs::remove_dir_all(&dir).ok();
    }
}
