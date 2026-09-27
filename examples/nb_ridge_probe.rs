//! Throwaway differential probe (issue 038 T7a): does the Rust [`NbRidge`]
//! fit reproduce the Python probe's emotion accuracy (~0.89 pure-ridge)?
//! Fits on the seat's train pool per label (the nb_doc_sets rule, uncapped)
//! and argmaxes the raw ridge scores on the test questions.
//!
//! Run: cargo run --release --features nb_ridge,option_cond --example nb_ridge_probe

use riir_reflex::embed::hashed_tokens_into;
use riir_reflex::harness::runner::seat::prepare_seat;
use riir_reflex::nb_ridge::fit;
use riir_reflex::nb_scope::NB_VOCAB;
use std::collections::HashSet;
use std::path::Path;

fn main() {
    let dir = Path::new(".raw/datasets_t20k");
    let seat = prepare_seat("emotion", dir).expect("seat");
    // Corpus pool = train (already the cal-front complement); per-label
    // doc sets, uncapped.
    let mut sets: Vec<Vec<String>> = vec![Vec::new(); seat.labels.len()];
    for d in &seat.train {
        if let Some(i) = seat.labels.iter().position(|l| l == &d.label) {
            sets[i].push(d.text.clone());
        }
    }
    let refs: Vec<&[String]> = sets.iter().map(|s| s.as_slice()).collect();
    let t0 = std::time::Instant::now();
    let ridge = fit(&refs, 10.0);
    println!("fit: {:.1}s, k={}, temp={:.4}", t0.elapsed().as_secs_f32(), ridge.feats_len(), ridge.temp());
    if std::env::var_os("RIDGE_DUMP").is_some() {
        // Same quantities the numpy replica prints, for a direct diff.
        let mut tok = Vec::new();
        hashed_tokens_into(seat.train[0].text.as_bytes(), NB_VOCAB, &mut tok);
        tok.sort_unstable();
        tok.dedup();
        let hits: Vec<usize> = tok.iter().filter_map(|w| ridge.feats_binary_search(*w)).collect();
        println!(
            "dump: doc0 bytes={} tok={} hits={} feats[0..5]={:?} ratios_c0[0..5]={:?} w_c0[0..5]={:?} w_c0_bias={}",
            seat.train[0].text.len(),
            tok.len(),
            hits.len(),
            &ridge.feats_dbg()[..5],
            &ridge.ratios_dbg(0)[..5],
            &ridge.weights_dbg(0)[..5],
            ridge.weights_dbg(0)[2048],
        );
    }

    let mut n = 0usize;
    let mut correct = 0usize;
    for (ci, case) in seat.suite.cases.iter().enumerate() {
        let mut tok = Vec::new();
        hashed_tokens_into(seat.state_strs[ci].as_bytes(), NB_VOCAB, &mut tok);
        // Dedup (binary presence).
        tok.sort_unstable();
        tok.dedup();
        for q in &case.questions {
            let keys: Vec<String> = match &q.criteria {
                serde_json::Value::Object(m) => m.keys().cloned().collect(),
                serde_json::Value::Array(a) => a.iter().map(|v| v.to_string()).collect(),
                _ => vec![],
            };
            // Option → domain: by name first, else the k == N index rule.
            let doms: Vec<Option<usize>> = if keys.iter().all(|k| seat.labels.contains(k)) {
                keys.iter().map(|k| seat.labels.iter().position(|l| l == k)).collect()
            } else if keys.len() == seat.labels.len() {
                (0..keys.len()).map(Some).collect()
            } else {
                vec![]
            };
            if doms.is_empty() {
                continue;
            }
            n += 1;
            let mut best = 0usize;
            let mut best_s = f32::NEG_INFINITY;
            for (oi, d) in doms.iter().enumerate() {
                let s = ridge.in_score(d.unwrap(), &tok);
                if s > best_s {
                    best_s = s;
                    best = oi;
                }
            }
            if best == case.gold[seat_question_index(case, &q.qid)].idx {
                correct += 1;
            }
        }
    }
    println!("pure-ridge argmax acc: {correct}/{n} = {:.4}", correct as f64 / n.max(1) as f64);
    let _ = HashSet::<u32>::new();
}

fn seat_question_index(
    case: &riir_reflex::harness::suites::SuiteCase,
    qid: &str,
) -> usize {
    case.questions.iter().position(|q| q.qid == qid).unwrap_or(0)
}
