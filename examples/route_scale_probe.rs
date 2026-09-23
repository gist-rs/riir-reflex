//! The `route_scale` sweep probe (issue 013 lever 2 — MEASUREMENT ONLY, not
//! a gate): forced accuracy of the raw modelless engine over the five
//! synthetic decision-point families across the blend-scale grid, vs the
//! shipped default 8.0. The blend only moves `route_active` questions
//! (`k == N` — options == domains), which is every family here.
//!
//! Scope is deliberately the synthetic families ONLY: they are
//! self-contained (`families::synth_by_name`, the same builder the gates
//! test consumes), so the probe duplicates no runner pipeline. A
//! full-suite sweep (dataset suites via the harness) is the follow-up only
//! if this grid shows signal — the promotion bar is a measured win at
//! parity everywhere else (the GOAT shape), never this probe alone.
//!
//! Run:
//!   cargo run --release --features modelless --example route_scale_probe

#![cfg(feature = "modelless")]

use katgpt_core::decision_wire::{DecisionRequest, Question};
use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};
use riir_reflex::harness::families;
use riir_reflex::pyjson::serialize_state;
use riir_reflex::harness::suites::{QKind, SuiteCase};

const SCALES: &[f32] = &[2.0, 4.0, 8.0, 16.0, 32.0];

/// The five modelless families (cache_reuse has no modelless lane by
/// construction) with their domain counts — the gates-test dispatch.
const FAMILIES: &[(&str, usize)] = &[
    ("harness_visibility", 4),
    ("harness_permissions", 3),
    ("harness_tool_fit", 6),
    ("harness_routing", 4),
    ("harness_sensitivity", 5),
];

/// The engine over one family's authored corpus at a given blend scale —
/// the gates-test builder with the one knob turned.
macro_rules! family_engine {
    ($n:literal, $d:expr, $scale:expr) => {{
        let d: &families::SynthData = $d;
        assert_eq!(d.labels.len(), $n, "family {} arms {} domains", d.suite.name, $n);
        let mut cfg = EngineConfig::default();
        cfg.route_scale = $scale;
        let specs: Vec<ExpertSpec> = d
            .labels
            .iter()
            .map(|l| {
                let docs: Vec<String> = d
                    .docs
                    .iter()
                    .filter(|doc| &doc.label == l)
                    .map(|doc| doc.text.clone())
                    .collect();
                ExpertSpec::new(l, &docs)
            })
            .collect();
        DecisionEngine::<$n, EMBED_DIM>::build_specs(specs, cfg).expect("family engine well-formed")
    }};
}

/// One case's wire request (the gates-test shape: choice options in the
/// criteria key order, score levels from the array, noul bare).
fn case_request(c: &SuiteCase) -> DecisionRequest {
    let q = &c.questions[0];
    let question = match q.kind {
        QKind::Choice => {
            let keys: Vec<String> = q
                .criteria
                .as_object()
                .expect("choice criteria object")
                .keys()
                .cloned()
                .collect();
            Question::choice(q.qid.as_str(), q.instructions.as_str(), keys, None)
        }
        QKind::Score => {
            let levels: Vec<String> = q
                .criteria
                .as_array()
                .expect("score criteria array")
                .iter()
                .map(|v| v.as_str().expect("string level").to_string())
                .collect();
            Question::score(q.qid.as_str(), q.instructions.as_str(), levels)
        }
        QKind::Noul => Question::noul(q.qid.as_str(), q.instructions.as_str()),
    };
    DecisionRequest {
        state: serialize_state(&c.state),
        questions: vec![question],
    }
}

/// Forced accuracy of the raw engine over the family's eval slice.
macro_rules! forced_acc {
    ($n:literal, $name:expr, $scale:expr) => {{
        let d = families::synth_by_name($name).expect("family synth");
        let mut engine = family_engine!($n, &d, $scale);
        let mut sc: Scratch<EMBED_DIM> = Scratch::new();
        sc.prepare(1);
        let mut correct = 0usize;
        for c in &d.suite.cases {
            let req = case_request(c);
            let resp = engine
                .decide_with(&req, &mut sc)
                .unwrap_or_else(|e| panic!("{}: decide failed: {e}", $name));
            // Forced pick: argmax of the probability vector (abstention
            // ignored — accuracy here is the same forced quantity the
            // harness tables publish).
            let pick = resp.answers[0]
                .probabilities
                .iter()
                .enumerate()
                .max_by(|x, y| x.1.total_cmp(y.1))
                .map_or(0, |(i, _)| i);
            if pick == c.gold[0].idx {
                correct += 1;
            }
        }
        correct as f64 / d.suite.cases.len().max(1) as f64
    }};
}

fn main() {
    let mut header = format!("| {:>5} ", "scale");
    for (name, _) in FAMILIES {
        header.push_str(&format!("| {} ", name.trim_start_matches("harness_")));
    }
    header.push('|');
    println!("{header}");
    println!(
        "|:{:-<5}{}|",
        "-",
        "-".repeat(FAMILIES.len() * 16)
    );

    for &scale in SCALES {
        let mut row = format!("| {:>5.1} ", scale);
        for (name, n) in FAMILIES {
            let acc = match n {
                3 => forced_acc!(3, name, scale),
                4 => forced_acc!(4, name, scale),
                5 => forced_acc!(5, name, scale),
                6 => forced_acc!(6, name, scale),
                _ => unreachable!("FAMILIES carries 3-6 domain counts"),
            };
            row.push_str(&format!("| {:>width$.3} ", acc, width = 14));
        }
        row.push('|');
        println!("{row}");
    }
    println!();
    println!("default is 8.0 (the Issue 004 T7 landing value). Measurement only —");
    println!("a promotion needs the full-suite GOAT shape (issue 013 lever 2).");
}
