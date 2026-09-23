//! Isolated sgemm shape-timing probe (measurement only, never a gate) —
//! the kernel-rung A/B instrument: times the forward's REAL projection ops
//! (`matmul_w` — every encoder projection goes through it) at the forward's
//! real geometries, so a staging/instance change is measured at the kernel,
//! not through the whole forward's noise.
//!
//! Posture mirrors the forward (agent.rs `forward_internal`): `begin_pass()`
//! per shape (advances the chain epoch — without it, dropped Vecs at
//! recycled heap addresses alias stale chain entries and `download_into`
//! resolves to the WRONG device buffer — measured: shape 3 diverged by
//! exactly max|CPU − stale-b| on both A/B sides identically), then a
//! pipelined timed BLOCK: R ops encoded back-to-back (chain hits, no
//! uploads), ONE sync at the end, wall/R — the forward never waits per op,
//! so per-op commit+wait cycles would measure submission overhead, not the
//! kernel.
//!
//! Divergence check is RELATIVE (max|c−d| / max|c| ≤ 1e-3): both the CPU
//! triple loop and the MSL kernels accumulate each output element
//! sequentially in k, so results are usually BIT-identical (reported as
//! info), but the check must not depend on that.
//!
//! Posture comes from the build + `LAYA_DEVICE` exactly like
//! `laya_fixture_timing`; the agent labels the reading. Position-balanced
//! A/B discipline: never compare across positions, only within swapped
//! pairs (the cold-GPU sequencing trap, `.issues/015` obs).
//!
//! Run (both sides of an A/B must run the SAME binary pair):
//!   LAYA_DEVICE=metal cargo run --release --features laya-riir-metal \
//!       --example sgemm_shape_timing

#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
use riir_reflex::laya::riir::backend::{Backend, Cpu};
#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
use riir_reflex::laya::riir::metal::Metal;
#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
use std::time::Instant;

#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
/// The forward's real `matmul_w` geometries (encoder.rs, d=1024 checkpoint,
/// i_sz=2624): QKV (n=3072) and fused gate/up (n=2·2624=5248) are XWIDE
/// (n ≥ 2048); O proj (k=1024) and down proj (k=2624) are the WIDE pair —
/// the WBK rung's population (~30% of forward GEMM FLOPs). The narrow pair
/// is ag_news's seq≈106 geometry (m < 256 → the narrow instance, BK=64,
/// untouched by this rung — the control).
const SHAPES: &[(usize, usize, usize, &str)] = &[
    (317, 1024, 1024, "O proj wide k=1024 (RUNG TARGET)"),
    (317, 2624, 1024, "down proj wide k=2624 (RUNG TARGET)"),
    (317, 1024, 3072, "QKV xwide control"),
    (317, 1024, 5248, "gate/up fused xwide control"),
    (106, 1024, 1024, "ag_news O narrow control"),
    (106, 2624, 1024, "ag_news down narrow control"),
];

#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
const GPU_REPS: usize = 24;

#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
fn median(samples: &mut [u128]) -> f64 {
    samples.sort();
    samples[samples.len() / 2] as f64 / 1000.0 // ns → µs
}

#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
fn main() {
    let metal = Metal::new().expect("metal backend");
    let cpu = Cpu;

    // ALL weight buffers are allocated up front and stay live for the whole
    // program: the weight cache is permanent and keyed by (ptr, len), so a
    // dropped weight whose address a later same-len weight reuses would hit
    // STALE device content. Live-and-distinct allocations cannot alias.
    let weights: Vec<Vec<f32>> = SHAPES
        .iter()
        .map(|&(_, k, n, _)| (0..n * k).map(|i| (i % 5) as f32 * 0.2 - 0.4).collect())
        .collect();

    for (shape_idx, &(m, k, n, label)) in SHAPES.iter().enumerate() {
        // Fresh activations per shape; begin_pass mirrors the forward's
        // pass boundary (chain map cleared, epoch bumped) so every shape
        // uploads into fresh slots — no cross-shape slot aliasing.
        let a: Vec<f32> = (0..m * k).map(|i| (i % 7) as f32 * 0.125 - 0.375).collect();
        let w = &weights[shape_idx];
        let mut dc = vec![0f32; m * n];
        let mut dm = vec![0f32; m * n];
        let mut dm2 = vec![0f32; m * n];

        cpu.matmul_w(&a, m, k, w, n, &mut dc);
        metal.begin_pass();
        metal.matmul_w(&a, m, k, w, n, &mut dm);
        metal.download_into(&dm, &mut dm2);
        let max_abs_c = dc.iter().fold(0.0f32, |acc, v| acc.max(v.abs()));
        let max_diff = dc
            .iter()
            .zip(dm2.iter())
            .map(|(c, d)| (c - d).abs())
            .fold(0.0f32, f32::max);
        let bit_identical = max_diff == 0.0;
        let rel = max_diff / max_abs_c.max(1e-30);
        if rel > 1e-3 {
            panic!("shape {m}x{k}x{n}: DIVERGED rel {rel:e} (max {max_diff:e})");
        }

        // Warmup, then the pipelined timed block: R encodes back-to-back
        // (chain hits — no uploads), ONE sync at the end. wall/R is the
        // amortized per-op time in the forward's own (one-CB-per-pass)
        // posture.
        for _ in 0..3 {
            metal.matmul_w(&a, m, k, w, n, &mut dm);
        }
        metal.download_into(&dm, &mut dm2);
        let mut gpu_ns = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            for _ in 0..GPU_REPS {
                metal.matmul_w(&a, m, k, w, n, &mut dm);
            }
            metal.download_into(&dm, &mut dm2);
            gpu_ns.push(t0.elapsed().as_nanos() / GPU_REPS as u128);
        }

        // CPU arm: single-shot (the big shapes cost >1 s per rep on the CPU
        // triple loop — the decision axis is the GPU A/B, the CPU number is
        // only orientation).
        let t1 = Instant::now();
        cpu.matmul_w(&a, m, k, w, n, &mut dc);
        let cpu_us = t1.elapsed().as_nanos() as f64 / 1000.0;

        println!(
            "matmul_w {m}x{k}x{n:>4}: gpu_p50={:>8.1}us cpu={:>8.1}us (rel {rel:e}{})  # {label}",
            median(&mut gpu_ns),
            cpu_us,
            if bit_identical { ", bit-identical" } else { "" },
        );
        // Touch the operands so the compiler cannot hoist them across the
        // timed block.
        std::hint::black_box((&a as *const _ as usize, w.as_ptr() as usize));
    }
}

#[cfg(not(all(target_os = "macos", feature = "laya-riir-metal")))]
fn main() {
    eprintln!("no metal lane compiled — build with --features laya-riir-metal on macOS");
    std::process::exit(2);
}
