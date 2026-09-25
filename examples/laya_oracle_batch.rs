//! Batch laya oracle over a JSONL decision manifest — the fixture generator
//! for external arena fixtures (katgpt-rs Plan 607 T0b is the first
//! consumer; the katgpt-rs arena book cites NUMBERS only, this repo stays
//! game-free — the game vocabulary lives in the INPUT data, never here).
//!
//! Manifest record shape (one JSON object per line):
//! ```json
//! {"state_id": "...", "question": "Does the stack look clean?",
//!  "options": [{"sentence": "..."}, ...]}
//! ```
//! Each option's `sentence` is forwarded as the laya `state` with a `noul`
//! question carrying the record's `question` as instructions; p(true) is
//! captured per option and the argmax (lowest index on ties, the manifest's
//! pinned option order) is the decision.
//!
//! A record may also carry an optional `state_sentence` (katgpt-rs Plan 609
//! T1.6's two-line envelope): when present, every option's forward becomes
//! `<state_sentence>\n<option sentence>`, so the decision can condition on
//! the state context (the tetris v4 preview arm). Absent → the historical
//! option-only forward, byte-identical behavior for the v2/v3 manifests.
//!
//! Output: one JSONL line per record —
//! `{"state_id": "...", "question": "...", "checkpoint": "english",
//!   "p_clean": [0.02, 0.94, ...], "argmax": 1}`
//! — followed by a BLAKE3 digest on stderr (the provenance anchor the
//! consumer's fixture records).
//!
//! Determinism: the forward is a fixed-weight deterministic pass; same
//! manifest + checkpoint in, same bytes out.
//!
//! Run:
//!   cargo run --release --features laya-riir --example laya_oracle_batch -- \
//!       --dump <manifest.jsonl> --out <oracle.jsonl> [--checkpoint english]

#[cfg(feature = "laya-riir")]
use riir_reflex::laya::config::Checkpoint;
#[cfg(feature = "laya-riir")]
use riir_reflex::laya::riir::RiirAgent;
#[cfg(feature = "laya-riir")]
use riir_reflex::laya::weights::weights_root;

#[cfg(feature = "laya-riir")]
fn main() {
    let mut dump = String::new();
    let mut out = String::from("laya_oracle_out.jsonl");
    let mut ckpt_name = String::from("english");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dump" if i + 1 < args.len() => {
                i += 1;
                dump = args[i].clone();
            }
            "--out" if i + 1 < args.len() => {
                i += 1;
                out = args[i].clone();
            }
            "--checkpoint" if i + 1 < args.len() => {
                i += 1;
                ckpt_name = args[i].clone();
            }
            other => {
                eprintln!(
                    "unknown arg {other:?} — usage: --dump <p> --out <p> [--checkpoint english|typed-decisions|multilingual]"
                );
                std::process::exit(1);
            }
        }
        i += 1;
    }
    if dump.is_empty() {
        eprintln!("--dump <manifest.jsonl> is required");
        std::process::exit(1);
    }
    let ckpt = match ckpt_name.as_str() {
        "english" => Checkpoint::English,
        "typed-decisions" | "typed" => Checkpoint::TypedDecisions,
        "multilingual" => Checkpoint::Multilingual,
        other => {
            eprintln!("unknown checkpoint {other:?}");
            std::process::exit(1);
        }
    };

    let manifest: Vec<serde_json::Value> = std::fs::read_to_string(&dump)
        .expect("read manifest")
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .expect("parse manifest JSONL");

    let root = weights_root();
    let agent = RiirAgent::load(&root, ckpt).expect("load checkpoint (weights resolve under ~/.cache/riir-reflex/laya, LAYA_WEIGHTS_DIR/LAYA_HOME override)");
    eprintln!(
        "loaded {ckpt_name} — posture: {} (LAYA_DEVICE)",
        agent.device()
    );

    let mut buf = String::new();
    let mut n_options = 0usize;
    let t0 = std::time::Instant::now();
    for rec in &manifest {
        let state_id = rec["state_id"].as_str().expect("state_id");
        let question = rec["question"].as_str().expect("question");
        let state_line = rec
            .get("state_sentence")
            .and_then(|v| v.as_str());
        let options = rec["options"].as_array().expect("options array");
        let qdef = serde_json::json!({
            "type": "noul",
            "instructions": question,
            "criteria": null,
        });
        let mut p_clean: Vec<f32> = Vec::with_capacity(options.len());
        for opt in options {
            let sentence = opt["sentence"].as_str().expect("option sentence");
            // The two-line envelope: the state line joins the option line
            // at the forward (Plan 609 T1.6). Without a state line the
            // forward is exactly the historical option-only text.
            let payload = match state_line {
                Some(s) => format!("{s}\n{sentence}"),
                None => sentence.to_string(),
            };
            let fwd = agent
                .forward_question(&serde_json::Value::String(payload), &qdef)
                .expect("forward");
            // noul option keys are ["false", "true"] — p(true) is the
            // yes-probability for the record's question.
            let p_true = fwd
                .probs
                .get(1)
                .copied()
                .expect("noul forward carries two probabilities");
            p_clean.push(p_true);
        }
        n_options += p_clean.len();
        // Argmax with the pinned tie-break: lowest index wins.
        let mut best = 0usize;
        let mut best_p = f32::NEG_INFINITY;
        for (idx, &p) in p_clean.iter().enumerate() {
            if p > best_p {
                best_p = p;
                best = idx;
            }
        }
        let line = serde_json::json!({
            "state_id": state_id,
            "question": question,
            "checkpoint": ckpt_name,
            "p_clean": p_clean,
            "argmax": best,
        });
        buf.push_str(&serde_json::to_string(&line).expect("serialize"));
        buf.push('\n');
    }

    std::fs::write(&out, &buf).expect("write output");
    let digest = blake3::hash(buf.as_bytes());
    eprintln!(
        "oracle: {} records, {n_options} forwards over {} in {:.1}s -> {out}",
        manifest.len(),
        ckpt_name,
        t0.elapsed().as_secs_f64()
    );
    eprintln!("blake3: {digest}");
}

#[cfg(not(feature = "laya-riir"))]
fn main() {
    eprintln!("laya_oracle_batch needs --features laya-riir (the laya lane is opt-in)");
    std::process::exit(2);
}
