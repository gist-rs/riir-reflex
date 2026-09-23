//! Timing probe over the G5 fixture corpus (mirrors
//! `scripts/probe_orig_laya_latency.py`'s posture: warmup pass, then timed
//! passes, one `system_one` call per row). Not a gate — measurement only.
//!
//! Lane (first arg; default `riir`):
//! - `riir` — the riir-owned backend (`laya-riir`; CPU by default,
//!   `LAYA_DEVICE=metal` + `laya-riir-metal` for the MSL backend — the
//!   posture label comes from the agent, so a reading can never be mistaken
//!   for the other posture).
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

#[cfg(feature = "laya-riir")]
use std::time::Instant;

/// The shared fixture plumbing: rows for one checkpoint + the
/// internal→raw question rebuild (the fixture stores t/ins/crit).
#[cfg(feature = "laya-riir")]
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

#[cfg(feature = "laya-riir")]
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

/// Warmup pass + timed passes over every row; prints the row p50/p90 and
/// the ms/question mean (the probe's published quantities).
#[cfg(feature = "laya-riir")]
fn time_agent(
    label: &str,
    ckpt: &str,
    reps: usize,
    mut run: impl FnMut(&serde_json::Value, &[(String, serde_json::Value)]),
) {
    let rows = load_rows(ckpt);
    for row in &rows {
        run(&row["state"], &build_qs(row));
    }
    let mut times_ms: Vec<f64> = Vec::new();
    let mut timed_q = 0usize;
    for _ in 0..reps {
        for row in &rows {
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

#[cfg(feature = "laya-riir")]
fn checkpoint_arg(name: &str) -> riir_reflex::laya::config::Checkpoint {
    match name {
        "english" => riir_reflex::laya::config::Checkpoint::English,
        "typed-decisions" | "typed" => riir_reflex::laya::config::Checkpoint::TypedDecisions,
        "multilingual" => riir_reflex::laya::config::Checkpoint::Multilingual,
        other => panic!("unknown checkpoint {other}"),
    }
}

#[cfg(feature = "laya-riir")]
fn main() {
    // Back-compat arg order: `[ckpt] [reps]` (and any explicit `riir`) all
    // select the one remaining lane.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("riir") => {
            args.remove(0);
        }
        Some("candle") => {
            eprintln!(
                "the candle lane was removed (.issues/006, owner directive) — \
                 the riir lane is the one backend; rerun without the lane arg"
            );
            std::process::exit(2);
        }
        _ => {}
    }
    let ckpt = args.first().cloned().unwrap_or_else(|| "english".into());
    let reps: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);

    let ck = checkpoint_arg(&ckpt);
    let agent =
        riir_reflex::laya::riir::RiirAgent::load(&riir_reflex::laya::weights::weights_root(), ck)
            .expect("agent load");
    // The posture label comes from the AGENT (`.issues/005`):
    // LAYA_DEVICE=metal runs the MSL backend — a reading can never
    // be mistaken for the other posture.
    time_agent(
        &format!("riir {}", agent.device()),
        &ckpt,
        reps,
        |state, qs| {
            agent.system_one(state, qs).expect("forward");
        },
    );
}

#[cfg(not(feature = "laya-riir"))]
fn main() {
    eprintln!("no laya lane compiled — build with --features laya-riir");
    std::process::exit(2);
}
