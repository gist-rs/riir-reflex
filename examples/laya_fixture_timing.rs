//! Timing probe over the G5 fixture corpus (mirrors
//! `scripts/probe_orig_laya_latency.py`'s posture: warmup pass, then timed
//! passes, one `system_one` call per row). Not a gate — measurement only.
//!
//! Lane (first arg; default `riir`):
//! - `riir` — the riir-owned backend (`laya-riir`; CPU by default,
//!   `LAYA_DEVICE=metal` + `laya-riir-metal` for the MSL backend — the
//!   posture label comes from the agent, so a reading can never be mistaken
//!   for the other posture).
//! - `ane` — the whole-graph Apple Neural Engine lane (`laya-riir-ane`,
//!   Plan 002 P1 T1.3): CONSTRUCTOR-selected (`RiirAgent::load_ane` owns
//!   its posture — an env value can never demote an explicitly requested
//!   lane), runs the digest-pinned BC1S artifacts under
//!   `LAYA_ANE_ARTIFACTS_DIR` (else `assets/ane/`). Implies `laya-riir`,
//!   so one feature set runs both lanes.
//!
//! The `candle` lane this example once carried was removed with the candle
//! reference lane itself (`.issues/006`, owner directive 2026-09-22); its
//! historical readings are frozen in `.benchmarks/001_phase1_harness.md`
//! addendum 6.
//!
//! Run:
//!   cargo run --release --features laya-riir \
//!       --example laya_fixture_timing -- riir english 3
//!   LAYA_DEVICE=metal cargo run --release --features laya-riir-metal \
//!       --example laya_fixture_timing -- riir english 3
//!   cargo run --release --features laya-riir-ane \
//!       --example laya_fixture_timing -- ane english 3
#![cfg(feature = "laya-riir")]

use std::time::Instant;

/// The shared fixture plumbing: rows for one checkpoint + the
/// internal→raw question rebuild (the fixture stores t/ins/crit).
/// The ANE timing lane — the whole arm lives behind the lane's own cfg so
/// a laya-riir-only build compiles the `ane` arg to a LOUD remedy exit
/// instead of an error (the guard's layer-5 law: every lane's own feature
/// set must compile). Runs the timing itself and RETURNS when done.
#[cfg(all(target_os = "macos", feature = "laya-riir-ane"))]
fn run_ane_lane(ckpt: &str, reps: usize) {
    let ane_root = std::env::var_os("LAYA_ANE_ARTIFACTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("assets/ane"));
    let manifest = ane_root.join("manifest.json");
    if !manifest.exists() {
        eprintln!(
            "ane lane: no manifest at {} — the artifacts are local-only; \
             run scripts/ane_convert.py first (or set LAYA_ANE_ARTIFACTS_DIR)",
            manifest.display()
        );
        std::process::exit(2);
    }
    let agent = riir_reflex::laya::riir::RiirAgent::load_ane(
        &riir_reflex::laya::weights::weights_root(),
        checkpoint_arg(ckpt),
        &ane_root,
        &manifest,
    )
    .expect("ane agent load");
    // Drop the rows the lane would refuse (named, loud) BEFORE the
    // warmup pass — a panic mid-timing would poison the run.
    let bucket_max = agent.ane_bucket_max().expect("ane posture");
    let fixture_name = match ckpt {
        "typed" => "typed-decisions",
        other => other,
    };
    let rows = drop_out_of_bucket_rows(load_rows(ckpt), fixture_name, bucket_max);
    // The posture label comes from the AGENT and reads 'ane' — the
    // same law as the metal lane's label: a reading can never be
    // mistaken for another posture. LAYA_DEVICE is deliberately NOT
    // consulted here: the explicit lane argument owns the posture.
    time_agent(
        &format!("riir {}", agent.device()),
        ckpt,
        reps,
        &rows,
        |state, qs| {
            agent.system_one(state, qs).expect("forward");
        },
    );
}

/// The no-ANE build's `ane` arg: a loud refusal naming the rebuild, not a
/// compile error (the layer-5 law) and never a silent fall-through to the
/// default lane — the posture label must never lie.
#[cfg(not(all(target_os = "macos", feature = "laya-riir-ane")))]
fn run_ane_lane(_ckpt: &str, _reps: usize) {
    eprintln!(
        "ane lane: this build has no ANE lane (needs macOS + \
         --features laya-riir-ane) — rebuild with the feature"
    );
    std::process::exit(2);
}

fn load_rows(ckpt: &str) -> Vec<serde_json::Value> {
    // The fixture's checkpoint names are `Checkpoint::fixture_name`'s:
    // english / typed-decisions / multilingual.
    let fixture_name = match ckpt {
        "typed" => "typed-decisions",
        other => other,
    };
    let text = std::fs::read_to_string("tests/fixtures/laya_parity_v1.jsonl").unwrap();
    let all: Vec<serde_json::Value> = text
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .unwrap();
    all.into_iter()
        .filter(|r| r.get("id").and_then(serde_json::Value::as_str) != Some("_meta"))
        .filter(|r| {
            r["checkpoints"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .any(|s| s == fixture_name)
                })
                .unwrap_or(false)
        })
        .collect()
}

fn build_qs(row: &serde_json::Value) -> Vec<(String, serde_json::Value)> {
    row["questions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let mut def = serde_json::Map::new();
            def.insert("type".into(), q["t"].clone());
            def.insert("instructions".into(), q["ins"].clone());
            if !q["crit"].is_null() {
                def.insert("criteria".into(), q["crit"].clone());
            }
            (format!("q{i}"), serde_json::Value::Object(def))
        })
        .collect()
}

/// The ANE lane refuses sequences beyond its buckets (no-silent-fallback);
/// the timing lane drops those rows LOUDLY instead of crashing — named,
/// counted, never silently absorbed (the parity gate's skip rule, at the
/// timing lane's granularity). The expected capture carries the seq lens.
#[cfg(feature = "laya-riir-ane")]
fn drop_out_of_bucket_rows(
    rows: Vec<serde_json::Value>,
    fixture_name: &str,
    bucket_max: usize,
) -> Vec<serde_json::Value> {
    let path = "tests/fixtures/laya_parity_expected_v1.json";
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("expected file readable"))
            .expect("expected file is JSON");
    let ck = &raw["checkpoints"][fixture_name];
    let kept: Vec<serde_json::Value> = rows
        .into_iter()
        .filter(|row| {
            let id = row["id"].as_str().expect("row id");
            let n_q = row["questions"].as_array().map(Vec::len).unwrap_or(0);
            (0..n_q).all(|i| {
                ck[format!("{id}#q{i}")]["seq_len"]
                    .as_u64()
                    .map(|n| n as usize <= bucket_max)
                    .unwrap_or(true) // a row the capture lacks is not the ANE gate's business
            }) || {
                eprintln!("  [ane timing] skipped (n > L{bucket_max}): {id}");
                false
            }
        })
        .collect();
    kept
}

/// Warmup pass + timed passes over every row; prints the row p50/p90 and
/// the ms/question mean (the probe's published quantities). The rows are
/// passed in (the ANE lane pre-filters its out-of-bucket rows LOUDLY —
/// named, counted, never silently absorbed).
fn time_agent(
    label: &str,
    ckpt: &str,
    reps: usize,
    rows: &[serde_json::Value],
    mut run: impl FnMut(&serde_json::Value, &[(String, serde_json::Value)]),
) {
    for row in rows {
        run(&row["state"], &build_qs(row));
    }
    let mut times_ms: Vec<f64> = Vec::new();
    let mut timed_q = 0usize;
    for _ in 0..reps {
        for row in rows {
            let qs = build_qs(row);
            let t0 = Instant::now();
            run(&row["state"], &qs);
            times_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            timed_q += qs.len();
        }
    }
    times_ms.sort_by(|a, b| a.total_cmp(b));
    let n = times_ms.len();
    let p50 = times_ms[n / 2];
    let p90 = times_ms[(n as f64 * 0.9) as usize];
    let mean_row = times_ms.iter().sum::<f64>() / n as f64;
    let per_q = mean_row / (timed_q as f64 / n as f64);
    println!(
        "[{label}] {ckpt}: rows={n} reps={reps} row_p50={p50:.1}ms row_p90={p90:.1}ms ms/question={per_q:.1} (n_q/row={:.2})",
        timed_q as f64 / n as f64
    );
}

fn checkpoint_arg(name: &str) -> riir_reflex::laya::config::Checkpoint {
    match name {
        "english" => riir_reflex::laya::config::Checkpoint::English,
        "typed-decisions" | "typed" => riir_reflex::laya::config::Checkpoint::TypedDecisions,
        "multilingual" => riir_reflex::laya::config::Checkpoint::Multilingual,
        other => panic!("unknown checkpoint {other}"),
    }
}

fn main() {
    // Lane arg: `riir` (default / back-compat bare `[ckpt] [reps]`), `ane`;
    // `candle` is refused with the removal pointer (the lane died with
    // .issues/006).
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut ane_requested = false;
    match args.first().map(String::as_str) {
        Some("riir") => {
            args.remove(0);
        }
        Some("ane") => {
            args.remove(0);
            ane_requested = true;
        }
        Some("candle") => {
            eprintln!(
                "the candle lane was removed (.issues/006, owner directive) — \
                 the riir/ ane lanes are the backends; rerun with 'riir' or 'ane'"
            );
            std::process::exit(2);
        }
        _ => {} // back-compat: `[ckpt] [reps]` with no lane arg
    }
    let ckpt = args.first().cloned().unwrap_or_else(|| "english".into());
    let reps: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    if ane_requested {
        run_ane_lane(&ckpt, reps);
        return;
    }

    // The default (and only remaining) lane: the per-op riir backend —
    // LAYA_DEVICE picks cpu/metal; the ane lane dispatched above.
    {
        let agent = riir_reflex::laya::riir::RiirAgent::load(
            &riir_reflex::laya::weights::weights_root(),
            checkpoint_arg(&ckpt),
        )
        .expect("agent load");
        // The posture label comes from the AGENT (`.issues/005`):
        // LAYA_DEVICE=metal runs the MSL backend — a reading can never
        // be mistaken for the other posture.
        let rows = load_rows(&ckpt);
        time_agent(
            &format!("riir {}", agent.device()),
            &ckpt,
            reps,
            &rows,
            |state, qs| {
                agent.system_one(state, qs).expect("forward");
            },
        );
    }
}
