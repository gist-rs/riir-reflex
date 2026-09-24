//! The batched-`system_one` parity gate (reflex issue 020 T5): the packed
//! multi-question forward — one packed encoder pass per case, per-question
//! heads on device-copied slabs — must answer the SAME frozen reference
//! capture the single-sequence G5 gate (`tests/laya_riir_parity.rs`)
//! replays, at the SAME gates (top-1 agreement ≥ 99.9 %, prob drift
//! ≤ 1e-3). G5 covers `forward_question`; THIS file is the packed path's
//! correctness authority — an agent-level packing mistake (collation,
//! slab offsets, marker addressing, answer envelope) is invisible to G5
//! and must red here.
//!
//! Structural parity rides along per question (markers / seq_len /
//! bucket / temperature) — the batched path rebuilds none of them, so any
//! divergence is a collation bug, named per question.
//!
//! Run: `cargo test --release --features laya-riir --test laya_batch_parity`
#![cfg(feature = "laya-riir")]

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use riir_reflex::laya::config::Checkpoint;
use riir_reflex::laya::riir::RiirAgent;
use riir_reflex::laya::{Result, LayaError};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

/// The same expected capture the G5 gate replays — single source of truth.
fn load_expected() -> (BTreeMap<(String, String), Value>, (f64, f64)) {
    let path = manifest_dir().join("tests/fixtures/laya_parity_expected_v1.json");
    let raw: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("expected file readable"))
            .expect("expected file is JSON");
    let corpus = manifest_dir().join("tests/fixtures/laya_parity_v1.jsonl");
    let corpus_bytes = std::fs::read(&corpus).expect("fixture corpus readable");
    let digest = {
        let mut h = blake3::Hasher::new();
        h.update(&corpus_bytes);
        h.finalize().to_hex().to_string()
    };
    let meta = &raw["_meta"];
    assert_eq!(
        meta["fixture_corpus_blake3"].as_str().expect("blake3 recorded"),
        digest,
        "fixture corpus does not match the digest the expected file was captured against"
    );
    let gates = &meta["gates"];
    (
        raw["checkpoints"]
            .as_object()
            .expect("checkpoints map")
            .iter()
            .flat_map(|(ckpt, rows)| {
                rows.as_object()
                    .expect("rows map")
                    .into_iter()
                    .map(move |(k, v)| ((ckpt.clone(), k.clone()), v.clone()))
            })
            .collect(),
        (
            gates["top1_agreement_min"].as_f64().expect("gate top1 recorded"),
            gates["prob_drift_max"].as_f64().expect("gate drift recorded"),
        ),
    )
}

fn drift(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f64, f64::max)
}

fn argmax(v: &[f64]) -> usize {
    v.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

#[test]
fn batched_system_one_matches_the_reference_capture() -> Result<()> {
    let fixtures = load_fixtures();
    let (expected, (top1_min, drift_max)) = load_expected();
    let root = riir_reflex::laya::weights::weights_root();

    let mut summary = String::new();
    let mut any_gate_failed = false;

    for ckpt in Checkpoint::ALL {
        let agent = RiirAgent::load(&root, ckpt)?;
        let name = ckpt.fixture_name();
        println!("{name}: batch posture = {} (LAYA_DEVICE)", agent.device());
        let rows: Vec<&Value> = fixtures
            .iter()
            .filter(|r| {
                r["checkpoints"]
                    .as_array()
                    .expect("checkpoints array")
                    .iter()
                    .filter_map(|c| c.as_str())
                    .any(|s| s == name)
            })
            .collect();

        let mut forwards = 0usize;
        let mut agree = 0usize;
        let mut max_prob_drift = 0f64;
        let mut per_key: Vec<(String, f64, usize, usize)> = Vec::new();

        for row in rows {
            let row_id = row["id"].as_str().expect("row id");
            let state = &row["state"];
            let questions: Vec<(String, Value)> = row["questions"]
                .as_array()
                .expect("questions array")
                .iter()
                .enumerate()
                .map(|(qi, qdef)| {
                    let q_long = serde_json::json!({
                        "type": qdef["t"],
                        "instructions": qdef["ins"],
                        "criteria": qdef.get("crit").cloned().unwrap_or(Value::Null),
                    });
                    (format!("q{qi}"), q_long)
                })
                .collect();

            // ONE system_one call per row — the packed path (the env
            // kill-switch is unset here by construction; a developer
            // exporting RIIR_LAYA_NO_BATCH=1 in their shell would silently
            // degrade this into the loop and the gate would still hold —
            // the loop is ALSO correct — so the posture line below names
            // the device, and the packed coverage is asserted by the
            // multi-question floor at the end).
            let answers = agent.system_one(state, &questions)?;

            for (qi, (ans, _)) in answers.iter().zip(questions.iter()).enumerate() {
                let key = format!("{row_id}#q{qi}");
                let exp = expected
                    .get(&(name.to_string(), key.clone()))
                    .unwrap_or_else(|| {
                        panic!("expected capture missing {name}/{key}")
                    });
                let exp_probs: Vec<f64> = exp["probs"]
                    .as_array()
                    .expect("probs")
                    .iter()
                    .map(|p| p.as_f64().expect("prob"))
                    .collect();
                let probs64: Vec<f64> = ans.probabilities.iter().map(|(_, p)| *p).collect();
                assert_eq!(
                    probs64.len(),
                    exp_probs.len(),
                    "{name}/{key}: option count diverges"
                );
                let pd = drift(&probs64, &exp_probs);
                max_prob_drift = max_prob_drift.max(pd);
                if argmax(&probs64) == argmax(&exp_probs) {
                    agree += 1;
                }
                per_key.push((key.clone(), pd, argmax(&probs64), argmax(&exp_probs)));

                // Structural: the temperature the batched path resolved
                // must equal the capture's (the envelope law, per question).
                let exp_t = exp["temperature_used"].as_f64().expect("temperature");
                assert!(
                    (ans.temperature_used - exp_t).abs() < 1e-6,
                    "{name}/{key}: temperature {} vs captured {exp_t}",
                    ans.temperature_used
                );
                forwards += 1;
            }
        }

        // Floors: zero forwards = corpus blindness; and at least one row
        // must carry MORE than one question, or this gate exercises no
        // packing at all (the whole point of its existence).
        assert!(forwards > 0, "{name}: fixture corpus covers no forwards");
        let multi = fixtures
            .iter()
            .filter(|r| {
                r["checkpoints"]
                    .as_array()
                    .expect("checkpoints array")
                    .iter()
                    .filter_map(|c| c.as_str())
                    .any(|s| s == name)
            })
            .filter(|r| r["questions"].as_array().expect("questions").len() > 1)
            .count();
        assert!(
            multi > 0,
            "{name}: no multi-question fixture rows — the batch gate would pass without packing anything"
        );

        let agreement = agree as f64 / forwards as f64;
        let top1_ok = agreement >= top1_min;
        let drift_ok = max_prob_drift <= drift_max;
        per_key.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (key, pd, ma, ea) in per_key.iter().take(3) {
            println!("  {name}/{key}: prob drift {pd:.3e} (argmax {ma} vs {ea})");
        }
        println!(
            "{name}: {forwards} batched forwards · top-1 {agree}/{forwards} = {agreement:.6} \
             (gate ≥ {top1_min}) · prob drift {max_prob_drift:.3e} (gate ≤ {drift_max})"
        );
        summary.push_str(&format!(
            "{name}: agreement {agreement:.6} / drift {max_prob_drift:.3e} — {}\n",
            if top1_ok && drift_ok { "PASS" } else { "FAIL" }
        ));
        if !(top1_ok && drift_ok) {
            any_gate_failed = true;
        }
    }

    println!("batched parity summary:\n{summary}");
    assert!(
        !any_gate_failed,
        "batched system_one drifted past the G5 gates — the packed path is wrong somewhere the \
         single-sequence gate cannot see"
    );
    Ok::<(), LayaError>(())
}
