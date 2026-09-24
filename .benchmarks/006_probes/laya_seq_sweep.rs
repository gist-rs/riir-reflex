//! Issue 020 length-scaling probe — the rust half of
//! `scripts/laya_seq_sweep.py`. Speaks `scripts/laya_python_lane.py`'s
//! line protocol (one JSON request per line: `{"state", "questions":
//! [{"qid","def"}]}`) so ONE driver times both lanes identically as a
//! round-trip. Each response carries the rust-only extras the driver
//! tabulates: `seq_len` (tokens the forward saw) and, under
//! `LAYA_SPLIT=1`, the encoder/head wall split.
//!
//! Run via the driver; standalone:
//!   LAYA_DEVICE=metal cargo run --release --features laya-riir-metal \
//!       --example laya_seq_sweep -- english
#![cfg(feature = "laya-riir")]

use std::io::{BufRead, Write};
use std::sync::atomic::Ordering;

use riir_reflex::laya::config::Checkpoint;
use riir_reflex::laya::riir::agent::{SPLIT_ENC_NS, SPLIT_HEAD_NS};
use riir_reflex::laya::riir::metal::{KFLASH, KTIME};

fn main() {
    let ck = match std::env::args().nth(1).as_deref().unwrap_or("english") {
        "english" => Checkpoint::English,
        "typed" => Checkpoint::TypedDecisions,
        "multilingual" => Checkpoint::Multilingual,
        other => panic!("unknown checkpoint {other}"),
    };
    let agent =
        riir_reflex::laya::riir::RiirAgent::load(&riir_reflex::laya::weights::weights_root(), ck)
            .expect("agent load");
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "{}", serde_json::json!({"ready": true, "device": agent.device()})).unwrap();
    out.flush().unwrap();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let req: serde_json::Value = serde_json::from_str(&line).expect("request JSON");
        let qs: Vec<(String, serde_json::Value)> = req["questions"]
            .as_array()
            .expect("questions")
            .iter()
            .map(|q| (q["qid"].as_str().unwrap().to_string(), q["def"].clone()))
            .collect();
        let resp = if req.get("probe_seq").and_then(serde_json::Value::as_bool) == Some(true) {
            // Untimed: the token length the forward sees for the FIRST question.
            let f = agent
                .forward_question(&req["state"], &qs[0].1)
                .expect("forward");
            serde_json::json!({"seq_len": f.seq_len})
        } else {
            SPLIT_ENC_NS.store(0, Ordering::Relaxed);
            SPLIT_HEAD_NS.store(0, Ordering::Relaxed);
            KTIME.lock().unwrap().clear();
            KFLASH.lock().unwrap().clear();
            agent.system_one(&req["state"], &qs).expect("forward");
            let kt: serde_json::Map<String, serde_json::Value> = KTIME
                .lock()
                .unwrap()
                .iter()
                .map(|(n, ns, c)| (n.to_string(), serde_json::json!([*ns as f64 / 1e6, c])))
                .collect();
            serde_json::json!({
                "enc_ms": SPLIT_ENC_NS.load(Ordering::Relaxed) as f64 / 1e6,
                "head_ms": SPLIT_HEAD_NS.load(Ordering::Relaxed) as f64 / 1e6,
                "ktime": kt,
                "kflash": KFLASH.lock().unwrap().iter().map(|n| *n as f64 / 1e6).collect::<Vec<_>>(),
            })
        };
        writeln!(out, "{resp}").unwrap();
        out.flush().unwrap();
    }
}
