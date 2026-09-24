//! PROBE-ONLY (detached worktree, never committed) — the T7 m-band pick
//! decision table: times the forward's REAL `matmul_w` geometries across the
//! m ∈ [231, 512] band with `LAYA_GEMM_FORCE=narrow|wide|xwide` selecting the
//! sgemm instance, so the dispatch-predicate rung is decided at the kernel.
//!
//! Posture mirrors `examples/sgemm_shape_timing` in riir-reflex (begin_pass
//! per shape, pipelined 24-rep blocks, 5 samples, median, relative divergence
//! check ≤ 1e-3 against the CPU triple loop). One process = one forced
//! posture (the env is read once via OnceLock in the probe diff) — the A/B
//! interleaves process invocations with a rotating order.

#![cfg(all(target_os = "macos", feature = "laya-riir-metal"))]

use riir_infer_laya::laya::riir::backend::{Backend, Cpu};
use riir_infer_laya::laya::riir::metal::Metal;
use std::time::Instant;

/// The valley band plus controls on both edges: 231 = narrow-zone control
/// (m < 256, current pick = narrow), 257/283 = the BM-64 padding peak,
/// 317 = banking77's seq, 370/400 = mid-band, 512 = wide-zone control
/// (no BM-64 padding). Shapes are the d=1024 checkpoint's four encoder
/// projections.
fn shapes() -> Vec<(usize, usize, usize, &'static str)> {
    let ms: Vec<usize> = std::env::var("SGEMM_M")
        .unwrap_or_else(|_| "231,257,283,317,370,400,512".into())
        .split(',')
        .map(|m| m.parse().expect("m"))
        .collect();
    let mut out = Vec::new();
    for m in ms {
        out.push((m, 1024, 1024, "o"));
        out.push((m, 2624, 1024, "wo"));
        out.push((m, 1024, 3072, "qkv"));
        out.push((m, 1024, 5248, "wi"));
    }
    out
}

const GPU_REPS: usize = 24;

fn median(samples: &mut [u128]) -> f64 {
    samples.sort();
    samples[samples.len() / 2] as f64 / 1000.0 // ns → µs
}

fn main() {
    let metal = Metal::new().expect("metal backend");
    let cpu = Cpu;
    let shapes = shapes();
    eprintln!(
        "posture={} shapes={} reps={GPU_REPS}",
        std::env::var("LAYA_GEMM_FORCE").as_deref().unwrap_or("dispatch-default"),
        shapes.len(),
    );

    // All weight buffers live for the whole program — the (ptr, len) weight
    // cache contract (a dropped weight whose address a same-len weight
    // reuses would hit STALE device content).
    let weights: Vec<Vec<f32>> = shapes
        .iter()
        .map(|&(_, k, n, _)| (0..n * k).map(|i| (i % 5) as f32 * 0.2 - 0.4).collect())
        .collect();

    for (idx, &(m, k, n, label)) in shapes.iter().enumerate() {
        let a: Vec<f32> = (0..m * k).map(|i| (i % 7) as f32 * 0.125 - 0.375).collect();
        let w = &weights[idx];
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
        let rel = max_diff / max_abs_c.max(1e-30);
        if rel > 1e-3 {
            panic!("shape {m}x{k}x{n} [{label}]: DIVERGED rel {rel:e}");
        }

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

        println!(
            "matmul_w {m:>3}x{k:>4}x{n:>4}: gpu_p50={:>8.1}us (rel {rel:e})  # {label}",
            median(&mut gpu_ns),
        );
        std::hint::black_box((&a as *const _ as usize, w.as_ptr() as usize));
    }
}

#[cfg(not(all(target_os = "macos", feature = "laya-riir-metal")))]
fn main() {
    eprintln!("build with --features laya-riir-metal on macOS");
    std::process::exit(2);
}
