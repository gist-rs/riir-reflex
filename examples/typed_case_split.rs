//! The packed-case stage-split probe (measurement only — never a gate;
//! reflex issue 020, the typed-trio lever pricing). Decomposes a REAL
//! `typed_decisions` 5-question case wall at the metal lane into:
//!
//! - `enc`  — `Encoder::forward_packed` host-enqueue wall (no sync; the
//!   first head download drains it)
//! - `qsum` — per-question `copy_at` + `Head::forward` wall (each ends in
//!   that question's act-logits download, so the per-question wall
//!   carries its own dispatch stream + its pipeline drain)
//! - `wall` — the whole case, the `system_one_packed` shape
//! - `loop` — the same case through the loop path's shape (5 ×
//!   single-question encoder forward + head), the T5 arm
//!
//! The question the numbers answer: which share a v2 packed head could
//! move (the per-question head walls) and which it cannot (the packed
//! encoder) — before building it.
//!
//! Two modes, one posture per process:
//! - default — the wall split above;
//! - `LAYA_METAL_PROFILE=1` — the substrate's per-dispatch profiler on,
//!   drained per stage; per-stage kernel GPU shares + dispatch counts
//!   print instead of walls (profiling serializes every dispatch, so
//!   absolute walls in that mode are NOT the shipped wall).
//!
//! Run (repo root; AC + quiet box; quote `uptime` beside the numbers):
//!   cargo run --release --features laya-riir-metal --example `typed_case_split` -- english
//!   `LAYA_METAL_PROFILE=1` cargo run --release --features laya-riir-metal \
//!       --example typed_case_split -- english
//!
//! Env: `LAYA_SPLIT_CASES` (default 16, evenly spaced over the suite),
//! `LAYA_SPLIT_ROUNDS` (default 7 timed rounds after 2 warmups),
//! `LAYA_SPLIT_LOOP=0` skips the loop arm.
#![cfg(all(target_os = "macos", feature = "laya-riir-metal"))]

use riir_reflex::harness::suites::{build_typed_decisions, SuiteCase};
use riir_reflex::laya::config::{load_checkpoint_configs, Checkpoint};
use riir_reflex::laya::riir::backend::Backend;
use riir_reflex::laya::riir::encoder::Encoder;
use riir_reflex::laya::riir::head::{Head, HeadScratch};
use riir_reflex::laya::riir::metal::{self, Metal};
use riir_reflex::laya::riir::weights as ckpt_weights;
use riir_reflex::laya::tokenize::{build_sequence, to_internal, Tok};
use riir_reflex::laya::weights::{ensure_checkpoint, weights_root};

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

/// One collated case: per-question token ids + markers + qtype, the packed
/// slab inputs, and the per-question device slabs + scratches (allocated
/// up front and alive for the WHOLE probe — the aliasing law: within a
/// chain epoch every logical buffer owns its (ptr, len) key).
struct SplitCase {
    id: String,
    seqs: Vec<usize>,
    markers: Vec<Vec<usize>>,
    qtypes: Vec<usize>,
    total_ids: Vec<u32>,
    slabs: Vec<Vec<f32>>,
    scratches: Vec<HeadScratch>,
    d: usize,
}

/// The harness's own question form (`case_questions` in the runner is
/// private — mirrored here, verbatim shape).
fn case_questions(case: &SuiteCase) -> Vec<(String, serde_json::Value)> {
    let mut questions = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let mut def = serde_json::Map::new();
        def.insert("type".into(), serde_json::Value::String(q.kind.as_str().into()));
        def.insert("instructions".into(), serde_json::Value::String(q.instructions.clone()));
        if !q.criteria.is_null() {
            def.insert("criteria".into(), q.criteria.clone());
        }
        questions.push((q.qid.clone(), serde_json::Value::Object(def)));
    }
    questions.shrink_to_fit();
    questions
}

fn load_cases(n_want: usize) -> Vec<SuiteCase> {
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for f in ["test-000.json", "test-001.json", "test-002.json", "test-003.json"] {
        let path = format!(".raw/datasets/typed_decisions/{f}");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("dataset page unreadable: {path}: {e}"));
        let v: serde_json::Value = serde_json::from_str(&text).expect("dataset page is JSON");
        rows.extend(v["rows"].as_array().cloned().unwrap_or_default());
    }
    let file = serde_json::Value::Object({
        let mut m = serde_json::Map::new();
        m.insert("rows".into(), serde_json::Value::Array(rows));
        m
    });
    let mut suite = build_typed_decisions(&file, 0);
    let n = suite.cases.len();
    assert!(n > 0, "typed_decisions built zero cases — dataset path wrong?");
    let step = (n / n_want.max(1)).max(1);
    (0..n)
        .step_by(step)
        .take(n_want)
        .map(|i| suite.cases.swap_remove(i))
        .collect()
}

fn collate(cases: &[SuiteCase], tok: &Tok, max_len: usize, head_max_len: usize, d: usize) -> Vec<SplitCase> {
    let t_all = std::time::Instant::now();
    let out: Vec<SplitCase> = cases
        .iter()
        .map(|case| {
            let mut seqs = Vec::new();
            let mut markers = Vec::new();
            let mut qtypes = Vec::new();
            let mut total_ids = Vec::new();
            let mut slabs = Vec::new();
            let mut scratches = Vec::new();
            for (_qid, qdef) in case_questions(case) {
                let q = to_internal(&qdef).expect("question internalizes");
                let (ids, mk) =
                    build_sequence(tok, &case.state, &q, max_len, head_max_len).expect("sequence");
                seqs.push(ids.len());
                markers.push(mk);
                qtypes.push(q.qtype);
                total_ids.extend(ids.iter().copied());
                slabs.push(vec![0f32; ids.len() * d]);
                scratches.push(HeadScratch::new());
            }
            SplitCase {
                id: case.id.clone(),
                seqs,
                markers,
                qtypes,
                total_ids,
                slabs,
                scratches,
                d,
            }
        })
        .collect();
    println!(
        "collate: {:.1} ms total over {} cases ({:.2} ms/case, host only)",
        t_all.elapsed().as_secs_f64() * 1e3,
        out.len(),
        t_all.elapsed().as_secs_f64() * 1e3 / out.len() as f64
    );
    out
}

fn packed_round(
    c: &mut SplitCase,
    enc: &Encoder,
    head: &Head,
    b: &dyn Backend,
) -> (f64, f64, f64) {
    b.begin_pass();
    let t0 = std::time::Instant::now();
    let hidden = enc.forward_packed(b, &c.total_ids, &c.seqs).expect("enc");
    let t_enc = t0.elapsed().as_secs_f64() * 1e3;
    let mut off = 0usize;
    let mut qsum = 0f64;
    for i in 0..c.seqs.len() {
        let rows = c.seqs[i];
        let tq = std::time::Instant::now();
        b.copy_at(&hidden, off * c.d, &mut c.slabs[i], 0, rows * c.d);
        let _ = head.forward(b, &mut c.slabs[i], c.qtypes[i], &c.markers[i], &mut c.scratches[i]);
        qsum += tq.elapsed().as_secs_f64() * 1e3;
        off += rows;
    }
    (t_enc, qsum, t0.elapsed().as_secs_f64() * 1e3)
}

fn loop_round(c: &mut SplitCase, enc: &Encoder, head: &Head, b: &dyn Backend) -> f64 {
    let t0 = std::time::Instant::now();
    let mut row_off = 0usize;
    for i in 0..c.seqs.len() {
        let rows = c.seqs[i];
        let ids = &c.total_ids[row_off..row_off + rows];
        b.begin_pass();
        let hidden = enc.forward_packed(b, ids, &c.seqs[i..=i]).expect("loop enc");
        b.copy_at(&hidden, 0, &mut c.slabs[i], 0, rows * c.d);
        let _ = head.forward(b, &mut c.slabs[i], c.qtypes[i], &c.markers[i], &mut c.scratches[i]);
        row_off += rows;
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn run_walls(cases: &mut [SplitCase], enc: &Encoder, head: &Head, b: &dyn Backend, rounds: usize, loop_arm: bool) {
    println!("\n── wall split (ms; qsum = Σ(copy_at + head + its drain); min = load-robust floor) ──");
    let (mut e, mut q, mut w, mut l): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut wmin_all: Vec<f64> = Vec::new();
    for c in cases.iter_mut() {
        for _ in 0..2 {
            packed_round(c, enc, head, b);
        }
        let (mut ew, mut qw, mut cw, mut lw) = (
            Vec::with_capacity(rounds),
            Vec::with_capacity(rounds),
            Vec::with_capacity(rounds),
            Vec::with_capacity(rounds),
        );
        for _ in 0..rounds {
            let (e, q, w) = packed_round(c, enc, head, b);
            ew.push(e);
            qw.push(q);
            cw.push(w);
        }
        if loop_arm {
            for _ in 0..rounds {
                lw.push(loop_round(c, enc, head, b));
            }
        }
        let (em, qm, cm) = (median(&mut ew), median(&mut qw), median(&mut cw));
        let lm = if lw.is_empty() { f64::NAN } else { median(&mut lw) };
        let cmin = cw.iter().cloned().fold(f64::INFINITY, f64::min);
        wmin_all.push(cmin);
        println!(
            "[{:>26}] Σseq {:>4} · enc {:6.1} · qsum {:6.1} · wall {:6.1} (min {:6.1}) · loop {:>6}",
            c.id,
            c.total_ids.len(),
            em,
            qm,
            cm,
            cmin,
            if lw.is_empty() { "-".into() } else { format!("{lm:.1}") },
        );
        e.push(em);
        q.push(qm);
        w.push(cm);
        if !lm.is_nan() {
            l.push(lm);
        }
    }
    let (em, qm, wm) = (median(&mut e), median(&mut q), median(&mut w));
    let lm = if l.is_empty() { f64::NAN } else { median(&mut l) };
    let gmin = wmin_all.iter().cloned().fold(f64::INFINITY, f64::min);
    println!(
        "\nMEDIANS over {} cases: enc {em:.1} · qsum {qm:.1} · wall {wm:.1} · loop {lm:.1} · min-wall-per-case median {:.1} · global min {gmin:.1}",
        cases.len(),
        median(&mut wmin_all),
    );
    if !lm.is_nan() {
        println!(
            "shares of wall: enc {:.1}% · qsum {:.1}% · resid {:.1}%",
            em / wm * 100.0,
            qm / wm * 100.0,
            (wm - em - qm) / wm * 100.0
        );
    }
}

fn agg(map: &mut std::collections::BTreeMap<&'static str, (f64, u64)>, rows: Vec<metal::ProfileRow>) {
    for r in rows {
        let e = map.entry(r.kernel).or_insert((0.0, 0));
        e.0 += r.gpu_s * 1e3;
        e.1 += 1;
    }
}

fn run_profile(cases: &mut [SplitCase], enc: &Encoder, head: &Head, b: &dyn Backend) {
    println!("\n── per-stage kernel GPU shares (profile mode; walls NOT comparable) ──");
    let rounds = 3usize;
    for c in cases.iter_mut() {
        packed_round(c, enc, head, b);
    }
    let _ = metal::profile_take();

    let mut enc_rows = std::collections::BTreeMap::new();
    let mut q_rows = std::collections::BTreeMap::new();
    for c in cases.iter_mut() {
        for _ in 0..rounds {
            b.begin_pass();
            let hidden = enc.forward_packed(b, &c.total_ids, &c.seqs).expect("enc");
            agg(&mut enc_rows, metal::profile_take());
            let mut off = 0usize;
            for i in 0..c.seqs.len() {
                let rows = c.seqs[i];
                b.copy_at(&hidden, off * c.d, &mut c.slabs[i], 0, rows * c.d);
                let _ = head.forward(b, &mut c.slabs[i], c.qtypes[i], &c.markers[i], &mut c.scratches[i]);
                agg(&mut q_rows, metal::profile_take());
                off += rows;
            }
        }
    }
    let n = (cases.len() * rounds) as f64;
    let print = |title: &str, m: &std::collections::BTreeMap<&'static str, (f64, u64)>| {
        let total: f64 = m.values().map(|(s, _)| *s).sum();
        println!("{title}: {total:.1} ms GPU over {n:.0} case-runs");
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by(|a, b| b.1 .0.total_cmp(&a.1 .0));
        for (k, (s, cnt)) in v.iter().take(10) {
            println!("  {k:<18} {:8.1} ms {:5.1}%  n/case {:.1}", s, s / total * 100.0, *cnt as f64 / n);
        }
    };
    print("ENCODER", &enc_rows);
    print("HEAD+COPY (Σ over questions)", &q_rows);
    let eg: f64 = enc_rows.values().map(|(s, _)| *s).sum();
    let qg: f64 = q_rows.values().map(|(s, _)| *s).sum();
    println!(
        "GPU share: encoder {:.1}% · head+copy {:.1}%",
        eg / (eg + qg) * 100.0,
        qg / (eg + qg) * 100.0
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ckpt = match args.first().map_or("english", String::as_str) {
        "english" => Checkpoint::English,
        "typed" | "typed-decisions" => Checkpoint::TypedDecisions,
        "multilingual" => Checkpoint::Multilingual,
        other => panic!("unknown checkpoint {other}"),
    };
    let n_cases: usize = std::env::var("LAYA_SPLIT_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(16);
    let rounds: usize = std::env::var("LAYA_SPLIT_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(7);
    let loop_arm = std::env::var("LAYA_SPLIT_LOOP").as_deref() != Ok("0");
    let profile = std::env::var("LAYA_METAL_PROFILE").as_deref() == Ok("1");

    let dir = ensure_checkpoint(&weights_root(), ckpt).expect("checkpoint present");
    let name = ckpt.subfolder();
    let (agent_cfg, enc_cfg) = load_checkpoint_configs(&dir, name).expect("configs");
    let tok = Tok::from_dir(&dir, name).expect("tokenizer");
    let mut raw = ckpt_weights::load(&dir.join("model.safetensors"), name).expect("weights");
    let enc = Encoder::from_map(&mut raw, enc_cfg.clone(), name).expect("encoder");
    let backend = Metal::new().expect("metal backend");
    let head = Head::from_map(&mut raw, name, enc_cfg.hidden, enc_cfg.eps).expect("head");
    drop(raw);
    enc.warm(&backend);
    head.warm(&backend);
    let d = enc_cfg.hidden;

    let picked = load_cases(n_cases);
    let mut cases = collate(&picked, &tok, agent_cfg.max_len, agent_cfg.head_max_len, d);
    cases.sort_by_key(|c| c.total_ids.len());

    println!(
        "typed_case_split: ckpt={name} d={} layers={} cases={} rounds={} profile={}",
        d, enc_cfg.layers, cases.len(), rounds, profile
    );
    for c in &cases {
        println!("  {} seqs={:?} Σ={}", c.id, c.seqs, c.total_ids.len());
    }
    if profile {
        run_profile(&mut cases, &enc, &head, &backend);
    } else {
        run_walls(&mut cases, &enc, &head, &backend, rounds, loop_arm);
    }
}
