//! The G5-ANE gate for the whole-graph Apple Neural Engine lane (reflex
//! Plan 002 P1 T1.2) — SAME fixture corpus + expected capture as the
//! per-op lanes' `tests/laya_riir_parity.rs`, but the gates are
//! DECISION-LEVEL by design (issue 017): the ANE artifact is FP16 end to
//! end, so it fails the 1e-3 p-drift bar by nature and is gated instead
//! on top-1 agreement + the near-tie band, with p-drift published as
//! OBSERVATION, never a gate.
//!
//! Gates (hard, per checkpoint, over the BUCKET-COVERED rows):
//! 1. structural parity — markers / seq_len / bucket / temperature exact
//!    (the shared render/tokenize stack is the same code; this pins the
//!    ANE forward's input construction against the goldens);
//! 2. top-1 agreement ≥ 99.9% (the meta's `top1_agreement_min`);
//! 3. every top-1 flip must be a near-tie: the GOLDEN's top-2 margin
//!    < 2 × 0.02 (issue 017's band — a flip is explainable only when the
//!    two top probabilities were closer than the fp16 error budget on
//!    both sides). A flip outside the band is a HARD MISMATCH.
//!
//! Published as observation: max prob err (the 0.02 class — Plan 002's
//! ml-0.0200 question), prob/logit/act/conf drift vs the goldens, the
//! near-tie flip list, and the out-of-bucket skips (sequences longer
//! than every bucket are named, counted, and floored — the ANE lane
//! refuses them at forward time, so the gate would silently shrink if
//! the corpus grew long-row-heavy without a floor).
//!
//! Artifacts are LOCAL ONLY (~3 GB, gitignored; the release scope is
//! download-on-demand, later): the test loud-SKIPS when they are absent,
//! naming the remedy — a green zero is never printed (the corpus_db
//! golden-round-trip precedent for local-only substrate).
//!
//! Run: `cargo test --release --features laya-riir-ane --test laya_ane_parity`
#![cfg(all(feature = "laya-riir-ane", target_os = "macos"))]
// `target_os = "macos"` — the SAME scope the substrate gates
// `RiirAgent::load_ane`/`ane_bucket_max` with (Core ML artifacts); a
// non-macOS `--all-features` build compiles this file to nothing (the
// green-zero law: the `required-features` row names the lane, this cfg
// names the platform — both are load-bearing).

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use riir_reflex::laya::config::Checkpoint;
use riir_reflex::laya::riir::RiirAgent;
use riir_reflex::laya::Result;

/// issue 017's near-tie tolerance — a decision flip is explainable only
/// when the golden's top-2 margin is under `2 × NEAR_TIE` (the fp16 error
/// budget on BOTH probabilities).
const NEAR_TIE: f64 = 0.02;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The artifact root + manifest path: `LAYA_ANE_ARTIFACTS_DIR` (the
/// release-scope env), else the committed `assets/ane/` tree. `None` =
/// absent — the caller loud-skips with the remedy.
fn ane_roots() -> Option<(PathBuf, PathBuf)> {
    if let Some(p) = std::env::var_os("LAYA_ANE_ARTIFACTS_DIR") {
        let root = PathBuf::from(p);
        let manifest = root.join("manifest.json");
        return manifest.exists().then_some((root, manifest));
    }
    let root = manifest_dir().join("assets/ane");
    let manifest = root.join("manifest.json");
    manifest.exists().then_some((root, manifest))
}

fn load_fixtures() -> Vec<Value> {
    let path = manifest_dir().join("tests/fixtures/laya_parity_v1.jsonl");
    let text = std::fs::read_to_string(&path).expect("fixture corpus readable");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str::<Value>)
        .map(|v| v.expect("fixture line is JSON"))
        .filter(|v| v.get("id").and_then(Value::as_str) != Some("_meta"))
        .collect()
}

fn load_expected() -> BTreeMap<(String, String), Value> {
    let path = manifest_dir().join("tests/fixtures/laya_parity_expected_v1.json");
    let raw: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("expected file readable"))
            .expect("expected file is JSON");
    let mut out = BTreeMap::new();
    for (ckpt, rows) in raw["checkpoints"].as_object().expect("checkpoints map") {
        for (key, entry) in rows.as_object().expect("rows map") {
            out.insert((ckpt.clone(), key.clone()), entry.clone());
        }
    }
    out
}

fn drift(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f64, f64::max)
}

fn argmax(p: &[f64]) -> usize {
    let mut best = 0usize;
    for (i, v) in p.iter().enumerate() {
        if *v > p[best] {
            best = i;
        }
    }
    best
}

/// The golden's top-2 probability margin (the near-tie test's denominator).
fn top2_margin(p: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = p.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    if sorted.len() > 1 {
        sorted[0] - sorted[1]
    } else {
        f64::MAX
    }
}

#[test]
fn g5_ane_decision_parity_per_checkpoint() -> Result<()> {
    let Some((ane_root, manifest_path)) = ane_roots() else {
        eprintln!(
            "[laya_ane_parity] SKIPPED (loud) — no ANE artifacts. The ~3 GB BC1S \
             .mlpackage tree is local-only, never committed; run the conversion first:\
             \n  uv run python scripts/ane_convert.py convert --model multilingual english typed \
             (see assets/ane/conversion_log.md)\
             \nor point LAYA_ANE_ARTIFACTS_DIR at an existing artifact root."
        );
        return Ok(());
    };

    let fixtures = load_fixtures();
    let expected = load_expected();
    let top1_min = load_expected_top1_min();

    let weights_root = riir_reflex::laya::weights::weights_root();

    let mut summary = String::new();
    let mut any_gate_failed = false;

    for ckpt in Checkpoint::ALL {
        let name = ckpt.fixture_name();
        // The ANE lane CONSTRUCTOR-selects its posture — an env value can
        // never demote an explicitly requested lane, and the agent label
        // (asserted below) is what makes a green run name its device.
        let agent = RiirAgent::load_ane(&weights_root, ckpt, &ane_root, &manifest_path)?;
        assert_eq!(
            agent.device(),
            "ane",
            "{name}: the ANE agent must label itself 'ane' — a green line that \
             cannot name its device verifies nothing"
        );
        println!("{name}: G5-ANE posture = {} (constructor-selected)", agent.device());

        let mut forwards = 0usize;
        let mut agree = 0usize;
        let mut hard_mismatches: Vec<String> = Vec::new();
        let mut near_tie_flips: Vec<String> = Vec::new();
        let mut skipped_out_of_bucket: Vec<String> = Vec::new();
        let mut max_prob_err = 0f64;
        let mut max_prob_drift = 0f64;
        let mut max_logit_drift = 0f64;
        let mut max_act_drift = 0f64;
        let mut max_conf_drift = 0f64;

        for row in &fixtures {
            let row_id = row["id"].as_str().expect("row id");
            let wanted: Vec<&str> = row["checkpoints"]
                .as_array()
                .expect("checkpoints array")
                .iter()
                .map(|c| c.as_str().expect("checkpoint name"))
                .collect();
            if !wanted.contains(&name) {
                continue;
            }
            let state = &row["state"];
            for (qi, qdef) in row["questions"]
                .as_array()
                .expect("questions array")
                .iter()
                .enumerate()
            {
                let key = format!("{row_id}#q{qi}");
                let exp = expected
                    .get(&(name.to_string(), key.clone()))
                    .unwrap_or_else(|| {
                        panic!("expected capture missing {name}/{key} — the capture is incomplete")
                    });
                let exp_len = exp["seq_len"].as_u64().expect("seq_len") as usize;
                let q_long = serde_json::json!({
                    "type": qdef["t"],
                    "instructions": qdef["ins"],
                    "criteria": qdef.get("crit").cloned().unwrap_or(Value::Null),
                });

                // Out-of-bucket rows: named + counted, never measured (the
                // lane refuses them; only buckets {64, 128} exist — L256
                // rides a later plan).
                if exceeds_ane_buckets(&agent, exp_len) {
                    skipped_out_of_bucket.push(key);
                    continue;
                }

                let fwd = agent.forward_question(state, &q_long)?;

                // Structural parity — exact (same pins as the per-op G5).
                let exp_markers: Vec<usize> = exp["markers"]
                    .as_array()
                    .expect("markers")
                    .iter()
                    .map(|m| m.as_u64().expect("marker int") as usize)
                    .collect();
                assert_eq!(
                    fwd.markers, exp_markers,
                    "{name}/{key}: marker positions diverge"
                );
                assert_eq!(
                    fwd.seq_len, exp_len,
                    "{name}/{key}: sequence length diverges"
                );
                assert_eq!(
                    fwd.bucket,
                    exp["bucket"].as_str().expect("bucket"),
                    "{name}/{key}: bucket diverges"
                );
                let exp_t = exp["temperature_used"].as_f64().expect("temperature");
                assert!(
                    (fwd.temperature_used - exp_t).abs() < 1e-6,
                    "{name}/{key}: temperature {} vs captured {exp_t}",
                    fwd.temperature_used
                );

                // Numeric comparison — OBSERVATION (never a gate; fp16).
                let exp_probs: Vec<f64> = exp["probs"]
                    .as_array()
                    .expect("probs")
                    .iter()
                    .map(|p| p.as_f64().expect("prob"))
                    .collect();
                let probs64: Vec<f64> = fwd.probs.iter().map(|p| *p as f64).collect();
                assert_eq!(
                    probs64.len(),
                    exp_probs.len(),
                    "{name}/{key}: option count diverges"
                );
                let pd = drift(&probs64, &exp_probs);
                max_prob_drift = max_prob_drift.max(pd);
                max_logit_drift = max_logit_drift.max(drift(
                    &fwd.logits.iter().map(|p| *p as f64).collect::<Vec<_>>(),
                    &exp["logits"]
                        .as_array()
                        .expect("logits")
                        .iter()
                        .map(|p| p.as_f64().expect("logit"))
                        .collect::<Vec<_>>(),
                ));
                let act64: Vec<f64> = fwd.act_probabilities.iter().map(|p| *p as f64).collect();
                max_act_drift = max_act_drift.max(drift(
                    &act64,
                    &exp["act_probabilities"]
                        .as_array()
                        .expect("act")
                        .iter()
                        .map(|p| p.as_f64().expect("act"))
                        .collect::<Vec<_>>(),
                ));
                let exp_conf = exp["confidence"].as_f64().expect("confidence");
                max_conf_drift = max_conf_drift.max((fwd.confidence - exp_conf).abs());

                // DECISION gates — the only gates on the numbers.
                let my_arg = argmax(&probs64);
                let exp_arg = argmax(&exp_probs);
                if my_arg == exp_arg {
                    agree += 1;
                } else if top2_margin(&exp_probs) < 2.0 * NEAR_TIE {
                    near_tie_flips.push(format!(
                        "{key}: margin {:.4}, arg {exp_arg}->{my_arg} (prob err {pd:.4})",
                        top2_margin(&exp_probs)
                    ));
                } else {
                    hard_mismatches.push(format!(
                        "{key}: margin {:.4} ≥ {} (the near-tie band), arg {exp_arg}->{my_arg}, \
                         prob err {pd:.4}",
                        top2_margin(&exp_probs),
                        2.0 * NEAR_TIE
                    ));
                }
                max_prob_err = max_prob_err.max(pd);
                forwards += 1;
            }
        }

        // Floors + gates.
        assert!(
            forwards > 0,
            "{name}: ZERO in-bucket forwards — the artifact/ manifest pairing or \
             the corpus moved; the gate refuses to pass over nothing"
        );
        let agreement = agree as f64 / forwards as f64;
        let top1_ok = agreement >= top1_min;
        let flips_ok = hard_mismatches.is_empty();
        let bucket_floor = {
            let total = fixtures
                .iter()
                .filter(|r| {
                    r["checkpoints"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .any(|s| s == name)
                        })
                        .unwrap_or(false)
                })
                .map(|r| r["questions"].as_array().map(Vec::len).unwrap_or(0))
                .sum::<usize>();
            // Blindness floor: the gate must keep measuring at least 3/4 of
            // the checkpoint's corpus — a bucket limit that silently swallows
            // the coverage is the green-zero shape, one axis over.
            forwards * 4 >= total * 3
        };
        println!(
            "{name}: {forwards} forwards · skipped (n > buckets) {} · \
             top-1 {agree}/{forwards} = {agreement:.6} (gate ≥ {top1_min}) · \
             hard mismatches {} · near-tie flips {} · \
             max prob err {max_prob_err:.4} (0.02 class, OBSERVED) · \
             prob drift {max_prob_drift:.3e} (OBSERVED) · logit {max_logit_drift:.3e} · \
             act {max_act_drift:.3e} · conf {max_conf_drift:.3e}",
            skipped_out_of_bucket.len(),
            hard_mismatches.len(),
            near_tie_flips.len(),
        );
        for f in &near_tie_flips {
            println!("  near-tie flip: {f}");
        }
        for f in &hard_mismatches {
            println!("  HARD MISMATCH: {f}");
        }
        if !bucket_floor {
            println!(
                "  ⛔ bucket floor: {forwards} served < 3/4 of the corpus — the lane is \
                 measuring too little to gate"
            );
        }
        let ok = top1_ok && flips_ok && bucket_floor;
        summary.push_str(&format!(
            "{name}: agreement {agreement:.6}, flips {}/{} hard, served {forwards} — {}\n",
            near_tie_flips.len(),
            hard_mismatches.len(),
            if ok { "PASS" } else { "FAIL" }
        ));
        if !ok {
            any_gate_failed = true;
        }
    }

    println!("G5-ANE parity summary:\n{summary}");
    assert!(
        !any_gate_failed,
        "G5-ANE FAILED — the ANE lane is PROVISIONAL in every table its numbers appear in"
    );
    Ok(())
}

/// Would a forward of `n` tokens refuse on bucket grounds? Read off the
/// agent's own manifest (the single bucket source) — no probe forward.
fn exceeds_ane_buckets(agent: &RiirAgent, n: usize) -> bool {
    match agent.ane_bucket_max() {
        Some(max) => n > max,
        None => true, // not the ANE posture — nothing is servable
    }
}

/// The top-1 gate from the expected file's meta (single source of truth —
/// the test never restates it). The ANE lane consumes ONLY the top-1 gate
/// from the meta; the drift gate stays the per-op lanes'.
fn load_expected_top1_min() -> f64 {
    let path = manifest_dir().join("tests/fixtures/laya_parity_expected_v1.json");
    let raw: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("readable"))
        .expect("expected file is JSON");
    raw.pointer("/_meta/gates/top1_agreement_min")
        .and_then(Value::as_f64)
        .expect("gate top1 recorded")
}
