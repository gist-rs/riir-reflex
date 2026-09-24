//! PROBE-ONLY (detached worktree, never committed) — T7 m-band pick, part 2:
//! the hot-path shape families run_sgemm serves that part 1 did not cover.
//!
//! (a) `matmul_w` at the small-m population (the head's a0 projection is
//!     1×1028×256; the suites run m ∈ {45..188}) + n=256;
//! (b) `matmul_kt_heads` (scores = q·kᵀ, batch = heads = 16, k = hd = 64,
//!     n = m — transposed-B staging, k ≤ BK) and `matmul_heads` (probs@v,
//!     batch = 16, n = 64) — the head MHA per question and the flash-off
//!     fallback path.
//!
//! Same posture as part 1: begin_pass per shape, pipelined 24-rep blocks,
//! 5 samples, median; divergence check vs the CPU arm (≤ 1e-3). One process
//! = one `LAYA_GEMM_FORCE` posture; the A/B interleaves process runs.

#![cfg(all(target_os = "macos", feature = "laya-riir-metal"))]

use riir_infer_laya::laya::riir::backend::{Backend, Cpu};
use riir_infer_laya::laya::riir::metal::Metal;
use std::time::Instant;

const HEADS: usize = 16;
const HD: usize = 64;
const GPU_REPS: usize = 24;

fn median(samples: &mut [u128]) -> f64 {
    samples.sort();
    samples[samples.len() / 2] as f64 / 1000.0 // ns → µs
}

fn main() {
    let metal = Metal::new().expect("metal backend");
    let cpu = Cpu;

    let small_ms: Vec<usize> = std::env::var("SGEMM_M_SMALL")
        .unwrap_or_else(|_| "1,4,45,92,106,188".into())
        .split(',')
        .map(|m| m.parse().expect("m"))
        .collect();
    let head_ms: Vec<usize> = std::env::var("SGEMM_M_HEAD")
        .unwrap_or_else(|_| "45,92,106,188,231,283,317,370,400,512".into())
        .split(',')
        .map(|m| m.parse().expect("m"))
        .collect();

    // (a) matmul_w small-m: (m, k, n, tag)
    let mut w_shapes = Vec::new();
    for &m in &small_ms {
        w_shapes.push((m, 1024usize, 1024usize, "o"));
        w_shapes.push((m, 1024usize, 3072usize, "qkv"));
        w_shapes.push((m, 1024usize, 5248usize, "wi"));
        w_shapes.push((m, 2624usize, 1024usize, "wo"));
        w_shapes.push((m, 1028usize, 256usize, "a0"));
    }
    // Weights live for the whole program — the (ptr, len) weight-cache
    // contract.
    let weights: Vec<Vec<f32>> = w_shapes
        .iter()
        .map(|&(_, k, n, _)| (0..n * k).map(|i| (i % 5) as f32 * 0.2 - 0.4).collect())
        .collect();

    for (idx, &(m, k, n, tag)) in w_shapes.iter().enumerate() {
        let a: Vec<f32> = (0..m * k).map(|i| (i % 7) as f32 * 0.125 - 0.375).collect();
        let w = &weights[idx];
        let mut dc = vec![0f32; m * n];
        let mut dm = vec![0f32; m * n];
        let mut dm2 = vec![0f32; m * n];
        cpu.matmul_w(&a, m, k, w, n, &mut dc);
        metal.begin_pass();
        metal.matmul_w(&a, m, k, w, n, &mut dm);
        metal.download_into(&dm, &mut dm2);
        let rel = rel_diff(&dc, &dm2);
        if rel > 1e-3 {
            panic!("matmul_w {m}x{k}x{n} [{tag}]: DIVERGED rel {rel:e}");
        }
        for _ in 0..3 {
            metal.matmul_w(&a, m, k, w, n, &mut dm);
        }
        metal.download_into(&dm, &mut dm2);
        let mut ns = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            for _ in 0..GPU_REPS {
                metal.matmul_w(&a, m, k, w, n, &mut dm);
            }
            metal.download_into(&dm, &mut dm2);
            ns.push(t0.elapsed().as_nanos() / GPU_REPS as u128);
        }
        println!(
            "matmul_w {m:>3}x{k:>4}x{n:>4}: gpu_p50={:>7.1}us (rel {rel:e})  # {tag}",
            median(&mut ns),
        );
    }

    // (b) head MHA pair at batch = HEADS. Weights are activations here
    // (chain class); scale values keep softmax sane.
    for &m in &head_ms {
        let q: Vec<f32> = (0..HEADS * m * HD).map(|i| (i % 11) as f32 * 0.09 - 0.45).collect();
        let k: Vec<f32> = (0..HEADS * m * HD).map(|i| (i % 13) as f32 * 0.08 - 0.5).collect();
        let v: Vec<f32> = (0..HEADS * m * HD).map(|i| (i % 7) as f32 * 0.125 - 0.375).collect();
        let mut sc_c = vec![0f32; HEADS * m * m];
        let mut sc_m = vec![0f32; HEADS * m * m];
        let mut sc_m2 = vec![0f32; HEADS * m * m];
        let mut ctx_c = vec![0f32; HEADS * m * HD];
        let mut ctx_m = vec![0f32; HEADS * m * HD];
        let mut ctx_m2 = vec![0f32; HEADS * m * HD];

        cpu.matmul_kt_heads(&q, &k, HEADS, m, HD, &mut sc_c);
        cpu.matmul_heads(&sc_c, &v, HEADS, m, m, HD, &mut ctx_c);
        metal.begin_pass();
        metal.matmul_kt_heads(&q, &k, HEADS, m, HD, &mut sc_m);
        metal.matmul_heads(&sc_m, &v, HEADS, m, m, HD, &mut ctx_m);
        metal.download_into(&sc_m, &mut sc_m2);
        metal.download_into(&ctx_m, &mut ctx_m2);
        let rel_kt = rel_diff(&sc_c, &sc_m2);
        let rel_hd = rel_diff(&ctx_c, &ctx_m2);
        if rel_kt > 1e-3 || rel_hd > 1e-3 {
            panic!("head MHA m={m}: DIVERGED kt {rel_kt:e} heads {rel_hd:e}");
        }
        for _ in 0..3 {
            metal.matmul_kt_heads(&q, &k, HEADS, m, HD, &mut sc_m);
        }
        metal.download_into(&sc_m, &mut sc_m2);
        let mut ns_kt = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            for _ in 0..GPU_REPS {
                metal.matmul_kt_heads(&q, &k, HEADS, m, HD, &mut sc_m);
            }
            metal.download_into(&sc_m, &mut sc_m2);
            ns_kt.push(t0.elapsed().as_nanos() / GPU_REPS as u128);
        }
        for _ in 0..3 {
            metal.matmul_heads(&sc_m, &v, HEADS, m, m, HD, &mut ctx_m);
        }
        metal.download_into(&ctx_m, &mut ctx_m2);
        let mut ns_hd = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            for _ in 0..GPU_REPS {
                metal.matmul_heads(&sc_m, &v, HEADS, m, m, HD, &mut ctx_m);
            }
            metal.download_into(&ctx_m, &mut ctx_m2);
            ns_hd.push(t0.elapsed().as_nanos() / GPU_REPS as u128);
        }
        println!(
            "kt_heads m={m:>3}: gpu_p50={:>7.1}us (rel {rel_kt:e})   heads m={m:>3}: gpu_p50={:>7.1}us (rel {rel_hd:e})  # headMHA",
            median(&mut ns_kt),
            median(&mut ns_hd),
        );
    }
}

fn rel_diff(reference: &[f32], observed: &[f32]) -> f64 {
    let scale = reference
        .iter()
        .fold(0.0f32, |acc, v| acc.max(v.abs()))
        .abs()
        .max(1e-30) as f64;
    reference
        .iter()
        .zip(observed.iter())
        .map(|(c, d)| (*c - *d).abs() as f64)
        .fold(0.0f64, f64::max)
        / scale
}

#[cfg(not(all(target_os = "macos", feature = "laya-riir-metal")))]
fn main() {
    eprintln!("build with --features laya-riir-metal on macOS");
    std::process::exit(2);
}
