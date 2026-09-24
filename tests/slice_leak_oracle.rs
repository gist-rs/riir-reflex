//! Issue 024 G1 + G2 for the `slice_leak` index.
//!
//! G1: over the same `.raw/datasets/` checkout and the probe's train-vs-test
//! scope, the Rust index reproduces `scripts/slice_leak_probe.py`'s per-suite
//! counts EXACTLY. The probe is run live and its output parsed, so the
//! oracle is the script itself and never a transcription of it.
//!
//! G2: the per-row classify loop allocates nothing once its scratch is warm,
//! and one suite's build + scan stays under 1 s (asserted in `--release` only;
//! a debug build measures an unoptimised binary).
//!
//! Without `.raw/datasets` or `python3` the gate prints UNSEEN and returns,
//! and UNSEEN is not a pass. Set `SLICE_LEAK_REQUIRE_DATA=1` to make it fail.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use riir_reflex::harness::slice_leak::{
    LeakClass, LeakCounts, LeakParams, QueryScratch, ShingleIndex, leak_counts,
};
use serde_json::Value;

// ── Per-thread allocation counter (the test harness runs tests on threads).
struct Counting;
thread_local! {
    static ALLOCS: Cell<u64> = const { Cell::new(0) };
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.with(|a| a.set(a.get() + 1));
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.with(|a| a.set(a.get() + 1));
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;

fn allocs() -> u64 {
    ALLOCS.with(Cell::get)
}

fn datasets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(".raw/datasets")
}

/// Loud skip, or a failure when the caller demanded the data.
fn unseen(why: &str) {
    let msg =
        format!("slice_leak_oracle: UNSEEN ({why}) — nothing was measured; this is NOT a pass");
    match std::env::var("SLICE_LEAK_REQUIRE_DATA").as_deref() {
        Ok("1") => panic!("{msg}"),
        _ => eprintln!("⚠ {msg}"),
    }
}

// ── The probe's row readers, transcribed.

/// Python `json.dumps(s)` for a str (default `ensure_ascii=True`).
fn py_dumps_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            ' '..='~' => out.push(c),
            _ => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{u:04x}"));
                }
            }
        }
    }
    out.push('"');
}

/// Python `json.dumps(v, sort_keys=True)`. Exact for strings, ints, bools,
/// null and containers; floats use Rust's shortest repr, which differs from
/// Python's for exponents (`1e-5` vs `1e-05`). No suite here dumps a float.
fn py_dumps(v: &Value, out: &mut String) {
    match v {
        Value::String(s) => py_dumps_str(s, out),
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::Array(xs) => {
            out.push('[');
            for (i, x) in xs.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                py_dumps(x, out);
            }
            out.push(']');
        }
        Value::Object(m) => {
            let sorted: BTreeMap<&String, &Value> = m.iter().collect();
            out.push('{');
            for (i, (k, x)) in sorted.into_iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                py_dumps_str(k, out);
                out.push_str(": ");
                py_dumps(x, out);
            }
            out.push('}');
        }
    }
}

fn row_text(row: &Value) -> String {
    if let Some(t) = row.get("text").and_then(Value::as_str) {
        return t.to_string();
    }
    if let Some(state) = row.get("state") {
        let mut out = String::new();
        py_dumps(state, &mut out);
        return out;
    }
    if let Some(p) = row.get("premise").and_then(Value::as_str) {
        let h = row.get("hypothesis").and_then(Value::as_str).unwrap_or("");
        return format!("{p} || {h}");
    }
    // serde_json's `preserve_order` keeps file order, as Python dicts do.
    row.as_object()
        .and_then(|m| m.values().find_map(Value::as_str))
        .unwrap_or("")
        .to_string()
}

fn row_label(row: &Value) -> Value {
    ["label_text", "label", "workflow"]
        .iter()
        .find_map(|k| row.get(*k).cloned())
        .unwrap_or(Value::Null)
}

fn load(suite: &Path, split: &str) -> Vec<(String, Value)> {
    let prefix = format!("{split}-");
    let mut files: Vec<PathBuf> = std::fs::read_dir(suite)
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    files.retain(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".json"))
    });
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f).expect("read dataset shard");
        let v: Value = serde_json::from_str(&text).expect("parse dataset shard");
        for r in v
            .get("rows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let row = &r["row"];
            out.push((row_text(row), row_label(row)));
        }
    }
    out
}

/// One parsed probe line.
fn parse_probe_line(line: &str) -> Option<(String, LeakCounts)> {
    let mut it = line.split_whitespace();
    let suite = it.next()?.to_string();
    let mut c = LeakCounts::default();
    for tok in it {
        let Some((k, v)) = tok.split_once('=') else {
            continue;
        };
        let num = |s: &str| s.trim().parse::<usize>().ok();
        match k {
            "train" => c.reference = num(v)?,
            "test" => c.query = num(v)?,
            "exact" => c.exact = num(v)?,
            "exact_label_conflict" => c.exact_label_conflict = num(v)?,
            "near_same_label" => c.near_same_label = num(v)?,
            // `near>=0.8=A/B` splits at the first '=' into `near>` / `0.8=A/B`.
            "near>" => {
                let (_, frac) = v.split_once('=')?;
                let (a, b) = frac.split_once('/')?;
                c.near = num(a)?;
                c.near_scanned = num(b)?;
            }
            _ => {}
        }
    }
    Some((suite, c))
}

/// The probe prints `train=` padded (`train=  4000`), so re-join each `key=`
/// with the number after it before tokenising.
fn squeeze(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut after_eq = false;
    for ch in line.chars() {
        if after_eq && ch == ' ' {
            continue;
        }
        after_eq = ch == '=';
        out.push(ch);
    }
    out
}

fn run_probe() -> Option<BTreeMap<String, LeakCounts>> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/slice_leak_probe.py");
    let out = Command::new("python3")
        .arg(&script)
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .ok()?;
    assert!(
        out.status.success(),
        "probe failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("probe stdout is UTF-8");
    let mut map = BTreeMap::new();
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        assert!(
            !line.contains("UNSEEN"),
            "probe could not read a suite: {line}"
        );
        let (suite, c) = parse_probe_line(&squeeze(line))
            .unwrap_or_else(|| panic!("unparsed probe line: {line}"));
        map.insert(suite, c);
    }
    Some(map)
}

#[test]
fn g1_rust_index_reproduces_the_probe_exactly() {
    let root = datasets();
    if !root.is_dir() {
        return unseen(".raw/datasets is absent");
    }
    let Some(expected) = run_probe() else {
        return unseen("python3 is not runnable");
    };
    assert!(
        expected.len() >= 8,
        "probe reported {} suites; the checkout has at least 8 — a walk regression",
        expected.len()
    );
    for (suite, want) in &expected {
        let dir = root.join(suite);
        let (train, test) = (load(&dir, "train"), load(&dir, "test"));
        let t0 = Instant::now();
        let got = leak_counts(&train, &test, LeakParams::default())
            .unwrap_or_else(|| panic!("{suite}: empty side, but the probe measured it"));
        let dt = t0.elapsed();
        eprintln!(
            "{suite:20} exact={:4} conflict={} near={}/{} same={}  [{:.1} ms]",
            got.exact,
            got.exact_label_conflict,
            got.near,
            got.near_scanned,
            got.near_same_label,
            dt.as_secs_f64() * 1e3
        );
        assert_eq!(got, *want, "{suite}: Rust index disagrees with the probe");
        #[cfg(not(debug_assertions))]
        assert!(
            dt.as_secs_f64() <= 1.0,
            "{suite}: G2 budget 1 s, took {dt:?}"
        );
    }
}

#[test]
fn g2_classify_allocates_nothing_once_warm() {
    let root = datasets();
    let dir = root.join("banking77");
    if !dir.is_dir() {
        return unseen(".raw/datasets/banking77 is absent");
    }
    let (train, test) = (load(&dir, "train"), load(&dir, "test"));
    let texts: Vec<&str> = train.iter().map(|(t, _)| t.as_str()).collect();
    let index = ShingleIndex::build(&texts, LeakParams::default());
    let mut scratch = QueryScratch::for_index(&index);
    // Warm pass: the scratch grows to the longest row once.
    let mut warm = Vec::with_capacity(test.len());
    for (t, _) in &test {
        warm.push(index.classify(t, &mut scratch));
    }
    let before = allocs();
    let mut n_near = 0usize;
    for ((t, _), w) in test.iter().zip(&warm) {
        let c = index.classify(t, &mut scratch);
        n_near += usize::from(matches!(c, LeakClass::Near { .. }));
        assert_eq!(c, *w, "classify is not a pure function of its input");
    }
    let spent = allocs() - before;
    assert!(
        n_near > 0,
        "the measured pass saw no NEAR row — it exercised nothing"
    );
    assert_eq!(
        spent,
        0,
        "{spent} allocations over {} warm classify calls",
        test.len()
    );
}

#[test]
fn probe_line_parser_reads_the_padded_format() {
    let line = "banking77            train=  4000 test= 3076 exact=   4 (0.13%) \
                exact_label_conflict=0 near>=0.8=61/1996 (3.06%) near_same_label=60";
    let (suite, c) = parse_probe_line(&squeeze(line)).unwrap();
    assert_eq!(suite, "banking77");
    assert_eq!(
        c,
        LeakCounts {
            reference: 4000,
            query: 3076,
            exact: 4,
            exact_label_conflict: 0,
            near: 61,
            near_scanned: 1996,
            near_same_label: 60,
        }
    );
}

#[test]
fn py_dumps_matches_python_for_the_escape_classes() {
    let mut out = String::new();
    py_dumps(
        &Value::String("a\"b\\c\n\u{2014}\u{1F600}\x01~".into()),
        &mut out,
    );
    assert_eq!(out, r#""a\"b\\c\n\u2014\ud83d\ude00\u0001~""#);
    out.clear();
    py_dumps(
        &serde_json::json!({"b": [1, true, null], "a": "x"}),
        &mut out,
    );
    assert_eq!(out, r#"{"a": "x", "b": [1, true, null]}"#);
}
