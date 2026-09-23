//! Issue 004 T6: G2 (latency) + G4 (alloc) gates over the harness
//! decision-point family engines — the [`decision_set_goat`] law applied to
//! the six-point families (Research 579 / Issue 004).
//!
//! - **G2:** per decision-set p99 ≤ 1 ms, per modelless family, with the
//!   percentile's TAIL SUPPORT printed beside it (the repo-family
//!   percentile law). Release profile by construction (`cargo bench`).
//! - **G4:** the `solve_into` core is allocation-free after warmup under a
//!   counting global allocator with a CANARY first — a green zero from a
//!   dead counter is the green-zero trap. Wire materialization
//!   (`decide_with`) allocates BY DESIGN and is reported, never gated.
//! - **cache_reuse is absent by design** (LLM-lane only, Issue 004 T3) —
//!   no modelless engine exists for it to gate.

#![cfg(feature = "modelless")]

use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};
use riir_reflex::harness::families;
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── Counting allocator (the bench-811 convention, binary-unique) ─────────

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// One family engine bundled for the gates.
struct FamilyGate {
    name: &'static str,
    req: katgpt_core::decision_wire::DecisionRequest,
}

fn build<const N: usize>(name: &'static str) -> (DecisionEngine<N, EMBED_DIM>, FamilyGate) {
    let d = families::synth_by_name(name).expect("family synth");
    assert_eq!(d.labels.len(), N, "{name}: domain count mismatch");
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
    let engine = DecisionEngine::<N, EMBED_DIM>::build_specs(specs, EngineConfig::default())
        .expect("family engine well-formed");
    // The G2 set: one real family case, its question rebuilt ×8 with
    // distinct qids (the decision_set_goat set shape).
    let c = &d.suite.cases[0];
    let q = &c.questions[0];
    let mut questions = Vec::with_capacity(8);
    for i in 0..8 {
        let qid = format!("g{i}");
        let question = match q.kind {
            riir_reflex::harness::suites::QKind::Choice => {
                let keys: Vec<String> = q
                    .criteria
                    .as_object()
                    .expect("choice criteria object")
                    .keys()
                    .cloned()
                    .collect();
                katgpt_core::decision_wire::Question::choice(
                    qid.as_str(),
                    q.instructions.as_str(),
                    keys,
                    None,
                )
            }
            riir_reflex::harness::suites::QKind::Score => {
                let levels: Vec<String> = q
                    .criteria
                    .as_array()
                    .expect("score criteria array")
                    .iter()
                    .map(|v| v.as_str().expect("string level").to_string())
                    .collect();
                katgpt_core::decision_wire::Question::score(
                    qid.as_str(),
                    q.instructions.as_str(),
                    levels,
                )
            }
            riir_reflex::harness::suites::QKind::Noul => {
                unreachable!("no modelless family uses noul")
            }
        };
        questions.push(question);
    }
    (
        engine,
        FamilyGate {
            name,
            req: katgpt_core::decision_wire::DecisionRequest {
                state: c.state.as_str().expect("string state").to_string(),
                questions,
            },
        },
    )
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  harness_families_goat — G2 latency + G4 alloc (Issue 004 T6)");
    println!("  p99 per family decision set ≤ 1 ms; solve_into alloc-free");
    println!("═══════════════════════════════════════════════════════════════");

    // ── Canary: the counter is LIVE before any zero is trusted ───────────
    let before = ALLOCS.load(Ordering::Relaxed);
    let canary: Vec<u8> = vec![7u8; 1024];
    black_box(&canary);
    let canary_count = ALLOCS.load(Ordering::Relaxed) - before;
    assert!(
        canary_count > 0,
        "G4 canary: counting allocator must COUNT (got 0 — dead instrument)"
    );
    println!("[canary] allocator counted {canary_count} alloc(s) — instrument live");

    let (vis_engine, vis_gate) = build::<4>("harness_visibility");
    let (perm_engine, perm_gate) = build::<3>("harness_permissions");
    let (tool_engine, tool_gate) = build::<6>("harness_tool_fit");
    let (route_engine, route_gate) = build::<4>("harness_routing");
    let (sens_engine, sens_gate) = build::<5>("harness_sensitivity");

    macro_rules! gate_family {
        ($engine:expr, $gate:expr) => {{
            let mut engine = $engine;
            let gate: &FamilyGate = &$gate;
            let n_questions = gate.req.questions.len();
            let mut sc: Scratch<EMBED_DIM> = Scratch::new();
            sc.prepare(n_questions);
            // Warmup + determinism.
            let warm = engine.decide_with(&gate.req, &mut sc).expect("decide");
            warm.validate_against(&gate.req).expect("wire-valid");
            let j1 = serde_json::to_string(&engine.decide_with(&gate.req, &mut sc).expect("decide"))
                .unwrap();
            let j2 = serde_json::to_string(&engine.decide_with(&gate.req, &mut sc).expect("decide"))
                .unwrap();
            assert_eq!(j1, j2, "{}: repeat runs must be bit-identical", gate.name);
            // G2. REPS 1000 — tail support ≥ 10 at p99 (the percentile
            // law: an index landing at n-1 is the MAX wearing a p99 name;
            // at 300 the support was 4, at 1000 it is 10+).
            const REPS: usize = 1000;
            let mut durations_us: Vec<u64> = Vec::with_capacity(REPS);
            for _ in 0..REPS {
                let t0 = Instant::now();
                let resp = black_box(engine.decide_with(black_box(&gate.req), &mut sc).expect("decide"));
                black_box(&resp);
                durations_us.push(t0.elapsed().as_micros() as u64);
            }
            durations_us.sort_unstable();
            let idx = (REPS * 99).div_ceil(100) - 1;
            let p99 = durations_us[idx];
            let p50 = durations_us[REPS / 2];
            let tail_support = REPS - idx;
            println!(
                "[G2 {}] p50 = {p50} µs · p99 = {p99} µs (tail support {tail_support}/{REPS}) · ns/question ≈ {}",
                gate.name,
                p50 * 1000 / n_questions as u64
            );
            assert!(
                p99 <= 1_000,
                "G2 FAIL ({}): p99 per decision set {p99} µs exceeds the 1 ms bar",
                gate.name
            );
            // G4: solve_into alloc-free after warmup.
            let a0 = ALLOCS.load(Ordering::Relaxed);
            const G4_REPS: usize = 200;
            for _ in 0..G4_REPS {
                engine.solve_into(black_box(&gate.req), &mut sc).expect("solve");
                black_box(&sc);
            }
            let core_allocs = ALLOCS.load(Ordering::Relaxed) - a0;
            println!("[G4 {}] solve_into × {G4_REPS} after warmup: {core_allocs} allocations", gate.name);
            assert_eq!(
                core_allocs, 0,
                "G4 FAIL ({}): the zero-alloc core allocated {core_allocs} time(s) after warmup",
                gate.name
            );
        }};
    }

    gate_family!(vis_engine, vis_gate);
    gate_family!(perm_engine, perm_gate);
    gate_family!(tool_engine, tool_gate);
    gate_family!(route_engine, route_gate);
    gate_family!(sens_engine, sens_gate);

    println!("═══════════════════════════════════════════════════════════════");
    println!("  harness_families_goat PASS — G2 ≤ 1 ms + G4 alloc-free × 5 families");
    println!("  (harness_cache_reuse: LLM-lane only — no modelless engine to gate)");
    println!("═══════════════════════════════════════════════════════════════");
}
