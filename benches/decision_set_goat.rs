//! G2 (latency) + G4 (alloc) gates for the modelless decision lane —
//! Plan 603 T1.3's "land WITH the lane" gates.
//!
//! Runs under `cargo bench` (release profile by construction — a latency
//! gate in a debug build measures an unoptimised binary).
//!
//! - **G2:** p99 per decision SET (one request, all questions answered in
//!   the call) ≤ 1 ms in-process. The percentile index is written with its
//!   TAIL SUPPORT (n − idx) printed beside it — a p99 over 200 samples
//!   carries 2 observations and says so (the repo-family percentile law).
//! - **G4:** the [`riir_reflex::engine::DecisionEngine::solve_into`] core
//!   is allocation-free after warmup, measured under a counting global
//!   allocator with a CANARY first — a green zero from a dead counter is
//!   the green-zero trap, so the canary must COUNT before the zero is
//!   trusted. Wire materialization (`to_response`) is excluded BY DESIGN:
//!   the wire types own their payload; that is the boundary, not the hot
//!   path.

#![cfg(feature = "modelless")]

use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, Scratch};
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

const OPS_DOC: &str = "Deploy the server to staging and verify the rollout before promoting \
to production. The staging cluster mirrors production capacity and runs the \
same release candidate. Rollback is one command when a deploy regresses the \
error budget. Verify the health endpoints after every rollout step. \
Provision the database migration before the service restart and check the \
capacity headroom.";

const SUPPORT_DOC: &str = "The customer asked for a refund of the last invoice because the \
billing account was charged twice. Check the account balance and the payment \
history, then refund the duplicate charge to the original payment method. \
Escalate to the billing team when the invoice does not match the account \
records. Update the customer profile note and confirm the corrected \
statement by email.";

fn request(n_questions: usize) -> katgpt_core::decision_wire::DecisionRequest {
    use katgpt_core::decision_wire::Question;
    let mut questions = Vec::with_capacity(n_questions);
    for i in 0..n_questions {
        match i % 3 {
            0 => questions.push(Question::choice(
                format!("q{i}"),
                "Which environment should receive this build next?",
                vec![
                    "production".to_string(),
                    "staging".to_string(),
                    "local sandbox".to_string(),
                    "do not deploy".to_string(),
                ],
                None,
            )),
            1 => questions.push(Question::score(
                format!("q{i}"),
                "How risky is promoting this build now?",
                vec![
                    "low".to_string(),
                    "moderate".to_string(),
                    "high".to_string(),
                ],
            )),
            _ => questions.push(Question::noul(
                format!("q{i}"),
                "Should the rollout proceed now?",
            )),
        }
    }
    katgpt_core::decision_wire::DecisionRequest {
        state: "The release candidate passed staging smoke tests; the on-call engineer \
verified the health endpoints and the error budget is clean."
            .to_string(),
        questions,
    }
}

fn demo_specs() -> Vec<ExpertSpec> {
    vec![
        ExpertSpec::new("ops", &[OPS_DOC.to_string()]),
        ExpertSpec::new("support", &[SUPPORT_DOC.to_string()]),
    ]
}

/// Issue 038's armed request: 2-option choices (k == N, index-aligned —
/// the tables fire) interleaved with noul.
#[cfg(feature = "nb_scope")]
fn nb_request(n_questions: usize) -> katgpt_core::decision_wire::DecisionRequest {
    use katgpt_core::decision_wire::Question;
    let questions = (0..n_questions)
        .map(|i| match i % 2 {
            0 => Question::choice(
                format!("q{i}"),
                "Which team owns this?",
                vec!["ops".to_string(), "support".to_string()],
                None,
            ),
            _ => Question::noul(format!("q{i}"), "Is this an ops matter?"),
        })
        .collect();
    katgpt_core::decision_wire::DecisionRequest {
        state: "The release candidate passed staging smoke tests; the on-call engineer \
verified the health endpoints and the error budget is clean."
            .to_string(),
        questions,
    }
}

/// Warmup + determinism + G2 (p99 ≤ 1 ms) + G4 (solve_into alloc-free
/// after warmup) for one engine posture. Returns the p99 (µs).
fn run_gates(
    label: &str,
    mut engine: DecisionEngine<2, EMBED_DIM>,
    req: &katgpt_core::decision_wire::DecisionRequest,
    n_questions: usize,
) -> u64 {
    println!("── posture: {label} ──");
    // ── Warmup: grow every scratch once (drafter, wire Vecs, Vec<i32> …) ─
    let mut sc: Scratch<EMBED_DIM> = Scratch::new();
    sc.prepare(n_questions);
    let warm = engine.decide_with(req, &mut sc).unwrap();
    warm.validate_against(req)
        .expect("engine output must satisfy the wire contract");
    println!(
        "[warm] {} answers, routing: {}",
        warm.answers.len(),
        warm.routing.reason.as_deref().unwrap_or("")
    );

    // ── Determinism (modelless lane, caveat-3 scoped) ────────────────────
    let j1 = serde_json::to_string(&engine.decide_with(req, &mut sc).unwrap()).unwrap();
    let j2 = serde_json::to_string(&engine.decide_with(req, &mut sc).unwrap()).unwrap();
    assert_eq!(j1, j2, "repeat runs must be bit-identical (modelless lane)");
    println!("[determinism] repeat runs bit-identical");

    // ── G2: per-request latency, p99 with tail support printed ───────────
    const REPS: usize = 300;
    let mut durations_us: Vec<u64> = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t0 = Instant::now();
        let resp = black_box(engine.decide_with(black_box(req), &mut sc).unwrap());
        black_box(&resp);
        durations_us.push(t0.elapsed().as_micros() as u64);
    }
    durations_us.sort_unstable();
    // p99 index — nearest-rank: idx = ceil(0.99 * n) - 1.
    let idx = (REPS * 99).div_ceil(100) - 1;
    let p99 = durations_us[idx];
    let p50 = durations_us[REPS / 2];
    let tail_support = REPS - idx;
    println!(
        "[G2] per-request p50 = {p50} µs · p99 = {p99} µs (tail support {tail_support}/{REPS}) · ns/question ≈ {}",
        p50 * 1000 / n_questions as u64
    );
    assert!(
        p99 <= 1_000,
        "G2 FAIL ({label}): p99 per decision set {p99} µs exceeds the 1 ms bar"
    );

    // ── G4: solve_into alloc-free after warmup ───────────────────────────
    let a0 = ALLOCS.load(Ordering::Relaxed);
    const G4_REPS: usize = 200;
    for _ in 0..G4_REPS {
        engine.solve_into(black_box(req), &mut sc).unwrap();
        black_box(&sc);
    }
    let core_allocs = ALLOCS.load(Ordering::Relaxed) - a0;
    println!("[G4] solve_into × {G4_REPS} after warmup: {core_allocs} allocations");
    assert_eq!(
        core_allocs, 0,
        "G4 FAIL ({label}): the zero-alloc core allocated {core_allocs} time(s) after warmup"
    );

    // Wire materialization allocates BY DESIGN (the boundary, not the hot
    // path) — reported, never asserted.
    let a1 = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..G4_REPS {
        black_box(engine.decide_with(black_box(req), &mut sc).unwrap());
    }
    let full_allocs = ALLOCS.load(Ordering::Relaxed) - a1;
    println!(
        "[G4] full decide_with × {G4_REPS} (wire boundary included): {full_allocs} allocations (reported, not gated)"
    );
    p99
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  decision_set_goat — G2 latency + G4 alloc (modelless lane)");
    println!("  p99 per decision set ≤ 1 ms; solve_into alloc-free post-warmup");
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

    let engine: DecisionEngine<2, EMBED_DIM> = DecisionEngine::build_specs(
        demo_specs(),
        EngineConfig::default(),
    )
    .expect("demo corpus well-formed");
    let n_questions = 8;
    let p99 = run_gates("default", engine, &request(n_questions), n_questions);

    // ── Issue 038: the same gates with the count tables ARMED — a request
    // whose choice options index-align with the domains (k == N) so the
    // tables fire, plus a noul with a configured polarity. The default
    // request above never arms them (its options name no domain).
    #[cfg(feature = "nb_scope")]
    {
        let cfg = EngineConfig {
            nb_scale: 4.0,
            nb_noul_domain: Some(0),
            ..EngineConfig::default()
        };
        let engine: DecisionEngine<2, EMBED_DIM> =
            DecisionEngine::build_specs(demo_specs(), cfg).expect("demo corpus well-formed");
        run_gates("nb_scope armed", engine, &nb_request(n_questions), n_questions);
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!(
        "  G2 PASS (p99 {p99} µs ≤ 1000 µs) · G4 PASS (core alloc-free, canary {canary_count})"
    );
    println!("═══════════════════════════════════════════════════════════════");
}
