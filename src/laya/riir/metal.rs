//! The riir forward's Metal backend (feature `laya-riir-metal`, macOS) —
//! the same op semantics as [`super::ops`], replayed as MSL compute
//! kernels (`.issues/005`, owner directive: the fair three-way compare is
//! all-Metal).
//!
//! Transcription provenance:
//! - **gelu erf is candle's own METAL kernel** (`candle-metal-kernels`
//!   `unary.metal`: the A&S 7.1.26 f32 erf + `x·(1+erf(x·√½))/2`) — copied
//!   verbatim, not re-derived: candle Metal is the chart column this lane
//!   is compared against, and it passes the G5 ≤ 1e-3 gate with exactly
//!   this kernel. The CPU lane keeps `libm::erff` (`.issues/003`) — each
//!   lane matches the candle posture it mirrors.
//! - **candle's flush shape, per PASS** — ops ENCODE into one pass-scoped
//!   command buffer and commit WITHOUT waiting; the wait happens only when
//!   the host genuinely reads a result ([`Backend::download_into`]: the
//!   scorer logits, the CLS row, the act logits — three syncs per forward).
//!   The v1 shape committed a fresh command buffer PER OP (~600–1400
//!   commits/forward — measured the lane's dominant overhead); the pass
//!   buffer plus a 1024-encode pipeline-flush cap keeps a long pass under
//!   Metal's per-buffer encoder ceiling without ever waiting mid-pass.
//!   A literal commit+wait per op measured 0.59 ms of round-trip per
//!   dispatch × ~1100 dispatches/forward — 2.5 s/forward, pure sync
//!   overhead. Device memory is the single writer between syncs — forward
//!   bodies never read op outputs host-side except through
//!   `download_into`.
//! - **generation-keyed chain cache** — activations flow device-side, so
//!   scratch slices are cached by `(ptr, len, generation)` with the
//!   generation bumped at every pass: host-authored buffers (rope tables,
//!   mask, `act_in`) rebuilt at recycled heap addresses can never hit a
//!   stale entry from an earlier epoch, and the write-first audit of the
//!   two forward bodies guarantees a hit's device copy is current. True
//!   weights (agent-owned `Vec`s: every `matmul_w` weight, LN scales,
//!   biases, the embedding table) live in a permanent cache keyed by
//!   `(ptr, len)` — stable for the agent's lifetime, uploaded once.
//! - **one batched simdgroup GEMM** covers all matmul shapes — including
//!   the all-heads attention batch — via explicit row/column strides plus
//!   a batch count with per-batch strides, in THREE tile geometries
//!   picked per call (32×64×64 narrow / 64×64×32 wide / 64×128×32 xwide;
//!   the geometry constants in this file document the pick),
//!   `simdgroup_multiply_accumulate` over 8×8 frags, threadgroup staging
//!   at padded odd strides (bank-conflict + overlap guards), and a
//!   guarded per-simdgroup store path for ragged edge tiles. The v1
//!   kernel was a naive 16×16 one-thread-per-element tile (the recorded
//!   honest baseline, ~3.5% of peak); this is the recorded optimization
//!   ladder climbed.
//! - **attention is ONE fused dispatch per layer** — `flash_attn` consumes
//!   the packed qkv directly (split, rope, q-scale, scores, sliding window,
//!   softmax, value mix, head merge in-kernel) and materializes NO seq²
//!   scores parent; sliding-window layers walk only their windowed key
//!   slice, which is where the long-sequence win lives (window 64 vs seq
//!   317 ≈ 2.4× less attention FLOPs). The two-pass form normalizes
//!   without rescaling the accumulator. `LAYA_METAL_FLASH=0` falls back to
//!   the reference op sequence ([`Backend::attention_forward_default`]);
//!   the CPU lane keeps the identical per-head op order, so its numerics
//!   are bit-unchanged.
//! - **softmax / LN are row-parallel** — one threadgroup (one simdgroup,
//!   32 lanes) per row with `simd_max`/`simd_sum` reductions; the v1
//!   kernels ran one thread per row (32× the GPU idle). Reduction order
//!   differs from the CPU lane's `candle_vec_sum` NEON order — exactly
//!   the drift budget the G5 gate holds (≤ 1e-3, the same room candle
//!   Metal passes within).
//!
//! Binding contract: every kernel declares `[[buffer(N)]]` /
//! `[[threadgroup(N)]]` attributes explicitly — device buffers at 0..,
//! then the 4-byte `constant` scalars, then the GEMM's three staging
//! buffers — and [`Metal::encode`] binds in exactly that order.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use metal::{
    Buffer, CommandBuffer, CommandQueue, ComputePipelineState, Device, MTLResourceOptions, MTLSize,
};
use objc2::rc::autoreleasepool;

use super::super::{LayaError, Result};
use super::backend::{AttnScratch, Backend};

/// Shared storage with EXPLICIT tracked hazard tracking: the pipeline
/// flush commits a full command buffer mid-pass, and Metal only inserts
/// the cross-command-buffer memory barriers for TRACKED resources. On
/// macOS the DEFAULT is untracked (so merely dropping the flag changed
/// nothing — measured), and candle's `HazardTrackingModeUntracked` pairs
/// with their ONE long-lived command buffer, where intra-buffer encoder
/// ordering provides the visibility. With multiple committed buffers per
/// pass, untracked meant kernel N+1 read stale memory written by kernel N
/// — a timing-dependent race the G5 gate caught. The tracked cost is part
/// of the honest baseline.
const RESOURCE_OPTIONS: MTLResourceOptions =
    MTLResourceOptions::StorageModeShared.union(MTLResourceOptions::HazardTrackingModeTracked);

/// sgemm tile geometry — MUST mirror the MSL constants in the instances
/// below; the `metal_ops_smoke` ragged-shape arms exercise every edge path
/// this mirroring could get wrong. Shared staging law: a row stride must
/// EXCEED the tile's row width (33 over a 64-wide tile overlaps itself:
/// column 63 of row kk collides with column 30 of row kk+1) and stays odd
/// for banks — hence strides 65 (over 64-wide rows) and 33 (over 32-wide
/// rows) and 129 (over 128-wide rows).
///
/// Three instances picked per call (see `run_sgemm`):
/// - narrow `sgemm` (32×64×64) for m < 256 — BK 64 halves the k-loop's
///   barrier count at the short-sequence suites (its staging still fits:
///   A [32][65] + B [64][65] = 6240 floats).
/// - wide `sgemm_wide` (64×64×32) for m ≥ 256, n < 2048 — the attention
///   scores/context shapes at long sequence (n = seq, n = 64). BK 48 (the
///   largest k-chunk fitting 32 KB at 64×64) was tried and measured FLAT
///   on the forward's wide population (O k=1024, down k=2624 — the
///   sgemm_shape_timing probe, 2026-09-24): the ~34% fewer staging
///   barriers were offset by +50% uncoalesced Wᵀ staging per iteration —
///   barriers are not the wide instance's binding constraint. Don't re-try
///   bigger wide BK without a new mechanism.
/// - xwide `sgemm_xwide` (64×128×32) for m ≥ 256, n ≥ 2048 — the QKV and
///   gate/up projections. BK stays 32: at BN 128 a BK-48 B tile is
///   [48][129] = 6192 floats and the pair outgrows 32 KB. Staging arithmetic
///   intensity BM·BN/(BM+BN) =
///   42.7 MAC/staged-element (wide 32, narrow 21 — the axis the
///   narrow→wide promotion was measured on); every n it serves (3072,
///   5248, 4096) is an exact multiple of 128, so the bigger BN pads
///   nothing, and the pick floor is measured: at n = 1024 the 64×128
///   tiles yield only 40 threadgroups at seq ~317 — one per GPU core, no
///   over-subscription — and the suite regressed (see [`XWIDE_N_MIN`]).
///
/// The ragged edge route reuses the staging front after the k-loop
/// (barrier-ordered); every instance's staging fits Metal's 32 KB
/// threadgroup limit (asserted below).
const WIDE_M_MIN: usize = 256;
/// The xwide pick: n ≥ this AND m ≥ [`WIDE_M_MIN`] routes to the 64×128
/// instance. Measured floor, not a guess: at n = 1024 (O proj + down
/// proj) the 64×128 tiles yield only ⌈317/64⌉×⌈1024/128⌉ = 40
/// threadgroups at seq ~317 — exactly one per GPU core, no
/// over-subscription to hide staging latency — and banking77 regressed;
/// at n ≥ 2048 (QKV 3072, gate/up 5248) the tile count stays ≥ 120 and
/// the bigger BN wins (~3% suite p50, position-balanced A/B). 2048 keeps
/// the two largest projections (≈70% of the forward's GEMM FLOPs) on
/// xwide and everything else on the 64×64 instance.
const XWIDE_N_MIN: usize = 2048;
/// Narrow instance staging (A [32][65] + B [64][65] floats) and dispatch.
const NARROW_STAGING_BYTES: u64 = ((32 * 65 + 64 * 65) * std::mem::size_of::<f32>()) as u64;
const NARROW_THREADS: u64 = 512;
/// Wide instance staging (A [64][33] + B [32][65] floats) and dispatch.
const WIDE_STAGING_BYTES: u64 = ((64 * 33 + 32 * 65) * std::mem::size_of::<f32>()) as u64;
const WIDE_THREADS: u64 = 1024;
/// xwide instance staging (A [64][33] + B [32][129] floats) and dispatch.
const XWIDE_STAGING_BYTES: u64 = ((64 * 33 + 32 * 129) * std::mem::size_of::<f32>()) as u64;
const XWIDE_THREADS: u64 = 1024;
/// Encoders per command buffer before a pipelined (no-wait) flush — keeps
/// a pass under Metal's per-buffer encoder ceiling without stalling; the
/// buffers join the committed list and are waited at the next sync.
const MAX_ENCODERS_PER_CB: u32 = 1024;

/// flash_attn staging (Q tile [32][65] + Kᵀ tile [64][33] + V tile
/// [32][65] + scores/probs [32][33] + 64 row-stats floats) and dispatch.
/// BQ = 32 query rows per threadgroup; the accumulator [32][64] lives as
/// one 8×8 frag per simdgroup (1024 threads = 32 simdgroups).
const FLASH_STAGING_BYTES: u64 =
    ((32 * 65 + 64 * 33 + 32 * 65 + 32 * 33 + 64) * std::mem::size_of::<f32>()) as u64;
const FLASH_THREADS: u64 = 1024;
/// Query-block rows of the fused attention kernel; hd is pinned to 64
/// (both shipped checkpoints — the kernel's rope pairing and tile mapping
/// assert it host-side).
const FLASH_HD: usize = 64;
/// Query rows per fused-attention threadgroup (mirrors the MSL `FBQ`).
const FLASH_BQ: u64 = 32;

/// The edge staging must fit inside the carved buffer's front half (the
/// k-loop's A/B tiles are dead by then; the barrier orders the reuse) and
/// the whole allocation must fit Metal's 32 KB threadgroup memory limit —
/// for BOTH instances.
/// (The asserts are const-foldable by construction — the guard exists to
/// fail the build the day a geometry edit outgrows its staging, so the
/// clippy always-true lint is allowed, not silent.)
#[allow(clippy::assertions_on_constants)]
const _: () = {
    assert!(16 * 128 <= 32 * 65 + 64 * 65);
    assert!(32 * 128 <= 64 * 33 + 32 * 65);
    assert!(32 * 128 <= 64 * 33 + 32 * 129);
    assert!(NARROW_STAGING_BYTES <= 32768);
    assert!(WIDE_STAGING_BYTES <= 32768);
    assert!(XWIDE_STAGING_BYTES <= 32768);
    assert!(FLASH_STAGING_BYTES <= 32768);
};

/// Debug trace flag (`LAYA_METAL_TRACE=1`): log every chain-cache miss.
fn trace_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("LAYA_METAL_TRACE").as_deref() == Ok("1"))
}

/// Debug-trace instance id source (separates the per-checkpoint Metal
/// instances in the miss log).
fn next_trace_instance() -> usize {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

const KERNELS: &[&str] = &[
    "sgemm",
    "sgemm_wide",
    "sgemm_xwide",
    "flash_attn",
    "add",
    "copy",
    "add_bias_row",
    "scale",
    "relu",
    "gelu_erf",
    "glu_gelu_gate",
    "ln_rows",
    "softmax_rows",
    "rope",
    "split_heads",
    "merge_heads",
    "gather_rows",
    "add_mask_bcast",
];

/// The MSL source. Sizes fit u32 (every pinned extent < 2³¹); `erf_as` is
/// candle's kernel verbatim (their constants, their op order). Every
/// pointer/scalar argument carries its explicit buffer-space index.
const MSL_HEAD: &str = r#"
#include <metal_stdlib>
#include <metal_simdgroup_matrix>
using namespace metal;

// ── sgemm geometry, THREE instantiations picked per-call by `m`/`n`:
// narrow `sgemm` (BM=32, 512 threads) for short sequences — its 32-row
// tiles pad little at m ≈ 100–200 and the doubled threadgroup count hides
// latency; wide `sgemm_wide` (BM=64, 1024 threads) for m ≥ 256 — the
// doubled staging arithmetic intensity (64 MAC/element vs 21) is worth
// ~14% at the seq-317 suites. Shared laws: BK is per-instance (narrow 64
// — A [32][65] + B [64][65]; wide 32 — BK 48 fits 32 KB at 64×64 but
// measured FLAT, the barrier saving is offset by the bigger uncoalesced
// staging gather; xwide 32 — a BK-48 B tile at BN 128 is [48][129] and
// the pair outgrows the limit); a staging stride must EXCEED the tile's
// row width or the tile overlaps itself (33 over a
// 64-wide tile corrupts every row from row 1's column 30 on); the ragged
// edge route reuses the staging front after the k-loop (barrier-ordered).

// candle-metal-kernels unary.metal — A&S 7.1.26 f32 erf, their constants.
inline float erf_as(float x) {
    const float a1 =  0.254829592f;
    const float a2 = -0.284496736f;
    const float a3 =  1.421413741f;
    const float a4 = -1.453152027f;
    const float a5 =  1.061405429f;
    const float p  =  0.3275911f;
    float sign = 1.0f;
    if (x < 0.0f) { sign = -1.0f; x = -x; }
    float t = 1.0f / (1.0f + p * x);
    float y = 1.0f - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * exp(-x * x);
    return sign * y;
}

// candle unary.metal gelu_erf: x * (1 + erf(x * sqrt(1/2))) / 2.
inline float gelu_as(float x) {
    return x * (1.0f + erf_as(x * 0.70710678f)) / 2.0f;
}

// ── wide instance: 64×64 output per threadgroup, 32 simdgroups (1024
// threads), two row-twin accumulators per simdgroup (rows sgr·8 and
// (sgr+4)·8 of its column block).
"#;

/// The wide instance (the `m ≥ WIDE_M_MIN` geometry).
const MSL_SGEMM_WIDE: &str = r#"
constant uint WBM = 64u;
constant uint WBN = 64u;
constant uint WBK = 32u;
constant uint WTAS = 33u;
constant uint WTBS = 65u;

kernel void sgemm_wide(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant uint& m [[buffer(3)]],
    constant uint& n [[buffer(4)]],
    constant uint& k [[buffer(5)]],
    constant uint& a_rs [[buffer(6)]],
    constant uint& a_cs [[buffer(7)]],
    constant uint& b_rs [[buffer(8)]],
    constant uint& b_cs [[buffer(9)]],
    constant uint& a_bs [[buffer(10)]],
    constant uint& b_bs [[buffer(11)]],
    constant uint& c_bs [[buffer(12)]],
    threadgroup float* raw [[threadgroup(13)]],
    uint3 gtp [[threadgroup_position_in_grid]],
    uint lid [[thread_index_in_threadgroup]])
{
    // One carved staging allocation (24 960 B): A tile [64][33] = 2112
    // floats, B tile [32][65] = 2080, and the ragged-edge store path
    // reuses the front 4096 floats AFTER the k-loop (the barrier below
    // orders it past the last A/B loads). WBK 48 (the largest k-chunk
    // fitting 32 KB here) measured FLAT vs 32 — the k-loop's barriers are
    // not this instance's binding constraint.
    threadgroup float* ta = raw;
    threadgroup float* tb = raw + 64u * WTAS;
    threadgroup float* edge = raw;
    const uint m0 = gtp.y * WBM;
    const uint n0 = gtp.x * WBN;
    device const float* A = a + gtp.z * a_bs;
    device const float* B = b + gtp.z * b_bs;
    device float* C = out + gtp.z * c_bs;

    const uint sg = lid >> 5u;   // simdgroup id 0..31
    const uint lane = lid & 31u;
    const uint sgr = sg >> 3u;   // 8×8 block row 0..3 (the +4 twin below)
    const uint sgc = sg & 7u;    // 8×8 block col 0..7

    // Two accumulators: rows sgr·8 and (sgr+4)·8 for this simdgroup's
    // column block — 32 simdgroups × 2 blocks cover the 64×64 tile.
    simdgroup_float8x8 acc0 = simdgroup_float8x8(0.0f);
    simdgroup_float8x8 acc1 = simdgroup_float8x8(0.0f);

    for (uint t = 0u; t < k; t += WBK) {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // Stage A [WBM][WBK] and B [WBK][WBN]: 2 of each 1024 elements per
        // thread. Element (kk, col) of B always lands at tb[kk·WTBS + col];
        // the two load mappings below both keep consecutive lanes on
        // consecutive global addresses for their layout.
        for (uint q = 0u; q < 2u; ++q) {
            // A tile [WBM][WBK] = [64][32]: two elements per thread.
            const uint idx = lid + q * 1024u;
            const uint r = idx >> 5u;
            const uint c = idx & 31u;
            const uint gr = m0 + r;
            const uint ac = t + c;
            ta[r * WTAS + c] = (gr < m && ac < k) ? A[gr * a_rs + ac * a_cs] : 0.0f;
        }
        for (uint q = 0u; q < 2u; ++q) {
            // B tile [WBK][WBN] = [32][64] at stride WTBS: k ∈ 0..31 from the
            // high bits, n ∈ 0..63 from the low (consecutive lanes →
            // consecutive n, coalesced along B's rows).
            const uint idx = lid + q * 1024u;
            const uint kk = idx >> 6u;
            const uint col = idx & 63u;
            const uint bc = t + kk;
            if (b_cs == 1u) {
                // Row-major B [k][n]: contiguous along n → n-fastest lanes.
                tb[kk * WTBS + col] =
                    (bc < k && n0 + col < n) ? B[bc * b_rs + n0 + col] : 0.0f;
            } else {
                tb[kk * WTBS + col] =
                    (bc < k && n0 + col < n) ? B[bc * b_rs + (n0 + col) * b_cs] : 0.0f;
            }
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint kk = 0u; kk < WBK; kk += 8u) {
            simdgroup_float8x8 fa0, fa1, fb;
            simdgroup_load(fa0, ta + sgr * 8u * WTAS + kk, WTAS);
            simdgroup_load(fa1, ta + (sgr + 4u) * 8u * WTAS + kk, WTAS);
            simdgroup_load(fb, tb + kk * WTBS + sgc * 8u, WTBS);
            simdgroup_multiply_accumulate(acc0, fa0, fb, acc0);
            simdgroup_multiply_accumulate(acc1, fa1, fb, acc1);
        }
    }

    if ((m0 + WBM <= m) && (n0 + WBN <= n)) {
        simdgroup_store(acc0, C + (m0 + sgr * 8u) * n + (n0 + sgc * 8u), n);
        simdgroup_store(acc1, C + (m0 + sgr * 8u + 4u * 8u) * n + (n0 + sgc * 8u), n);
    } else {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        simdgroup_store(acc0, edge + sg * 128u, 8u);
        simdgroup_store(acc1, edge + sg * 128u + 64u, 8u);
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint q = 0u; q < 2u; ++q) {
            const uint e = lane + q * 32u;
            const uint er = e >> 3u;
            const uint ec = e & 7u;
            const uint gr0 = m0 + sgr * 8u + er;
            const uint gr1 = gr0 + 4u * 8u;
            const uint gc = n0 + sgc * 8u + ec;
            if (gr0 < m && gc < n) { C[gr0 * n + gc] = edge[sg * 128u + e]; }
            if (gr1 < m && gc < n) { C[gr1 * n + gc] = edge[sg * 128u + 64u + e]; }
        }
    }
}
"#;

/// The xwide instance (the `m ≥ WIDE_M_MIN && n ≥ XWIDE_N_MIN` geometry):
/// 64×128 output per threadgroup, 32 simdgroups (1024 threads), four
/// accumulators per simdgroup — the row twins (sgr·8, (sgr+4)·8) × the
/// column twins (sgc·8, sgc·8+64) — covering the 8×16 grid of 8×8 blocks.
const MSL_SGEMM_XWIDE: &str = r#"
constant uint XBM = 64u;
constant uint XBN = 128u;
constant uint XBK = 32u;
constant uint XTAS = 33u;
constant uint XTBS = 129u;

kernel void sgemm_xwide(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant uint& m [[buffer(3)]],
    constant uint& n [[buffer(4)]],
    constant uint& k [[buffer(5)]],
    constant uint& a_rs [[buffer(6)]],
    constant uint& a_cs [[buffer(7)]],
    constant uint& b_rs [[buffer(8)]],
    constant uint& b_cs [[buffer(9)]],
    constant uint& a_bs [[buffer(10)]],
    constant uint& b_bs [[buffer(11)]],
    constant uint& c_bs [[buffer(12)]],
    threadgroup float* raw [[threadgroup(13)]],
    uint3 gtp [[threadgroup_position_in_grid]],
    uint lid [[thread_index_in_threadgroup]])
{
    // Carved staging (24 960 B): A tile [64][33] = 2112 floats, B tile
    // [32][129] = 4128. The ragged-edge route reuses the front 4096
    // floats, in TWO phases (four accumulators need 8192 floats at once,
    // which the allocation does not hold — phase 1 drains the row-twin
    // pair, the barrier orders the reads past the reuse, phase 2 the
    // +32 row pair).
    threadgroup float* ta = raw;
    threadgroup float* tb = raw + 64u * XTAS;
    threadgroup float* edge = raw;
    const uint m0 = gtp.y * XBM;
    const uint n0 = gtp.x * XBN;
    device const float* A = a + gtp.z * a_bs;
    device const float* B = b + gtp.z * b_bs;
    device float* C = out + gtp.z * c_bs;

    const uint sg = lid >> 5u;   // simdgroup id 0..31
    const uint lane = lid & 31u;
    const uint sgr = sg >> 3u;   // row block 0..3 (the +4 twin below)
    const uint sgc = sg & 7u;    // column block 0..7 (the +8 twin below)

    // Four accumulators: rows sgr·8 / sgr·8+32 × columns sgc·8 /
    // sgc·8+64 — 32 simdgroups × 4 blocks cover the 64×128 tile.
    simdgroup_float8x8 acc00 = simdgroup_float8x8(0.0f);
    simdgroup_float8x8 acc01 = simdgroup_float8x8(0.0f);
    simdgroup_float8x8 acc10 = simdgroup_float8x8(0.0f);
    simdgroup_float8x8 acc11 = simdgroup_float8x8(0.0f);

    for (uint t = 0u; t < k; t += XBK) {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // Stage A [64][32]: two elements per thread.
        for (uint q = 0u; q < 2u; ++q) {
            const uint idx = lid + q * 1024u;
            const uint r = idx >> 5u;
            const uint c = idx & 31u;
            const uint gr = m0 + r;
            const uint ac = t + c;
            ta[r * XTAS + c] = (gr < m && ac < k) ? A[gr * a_rs + ac * a_cs] : 0.0f;
        }
        // Stage B [32][128]: four elements per thread; k ∈ 0..31 from the
        // high bits, n ∈ 0..127 from the low (consecutive lanes →
        // consecutive n, coalesced along B's rows; the staged tile is the
        // SAME [K][N] layout for both B orientations).
        for (uint q = 0u; q < 4u; ++q) {
            const uint idx = lid + q * 1024u;
            const uint kk = idx >> 7u;
            const uint col = idx & 127u;
            const uint bc = t + kk;
            if (b_cs == 1u) {
                tb[kk * XTBS + col] =
                    (bc < k && n0 + col < n) ? B[bc * b_rs + n0 + col] : 0.0f;
            } else {
                tb[kk * XTBS + col] =
                    (bc < k && n0 + col < n) ? B[bc * b_rs + (n0 + col) * b_cs] : 0.0f;
            }
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint kk = 0u; kk < XBK; kk += 8u) {
            simdgroup_float8x8 fa0, fa1, fb0, fb1;
            simdgroup_load(fa0, ta + sgr * 8u * XTAS + kk, XTAS);
            simdgroup_load(fa1, ta + (sgr + 4u) * 8u * XTAS + kk, XTAS);
            simdgroup_load(fb0, tb + kk * XTBS + sgc * 8u, XTBS);
            simdgroup_load(fb1, tb + kk * XTBS + sgc * 8u + 64u, XTBS);
            simdgroup_multiply_accumulate(acc00, fa0, fb0, acc00);
            simdgroup_multiply_accumulate(acc01, fa0, fb1, acc01);
            simdgroup_multiply_accumulate(acc10, fa1, fb0, acc10);
            simdgroup_multiply_accumulate(acc11, fa1, fb1, acc11);
        }
    }

    if ((m0 + XBM <= m) && (n0 + XBN <= n)) {
        simdgroup_store(acc00, C + (m0 + sgr * 8u) * n + (n0 + sgc * 8u), n);
        simdgroup_store(acc01, C + (m0 + sgr * 8u) * n + (n0 + sgc * 8u + 64u), n);
        simdgroup_store(acc10, C + (m0 + sgr * 8u + 32u) * n + (n0 + sgc * 8u), n);
        simdgroup_store(acc11, C + (m0 + sgr * 8u + 32u) * n + (n0 + sgc * 8u + 64u), n);
    } else {
        // Ragged tile: drain the four accumulators through the shared
        // front in two phases (a barrier after each store burst and after
        // each scalar drain orders the reuse).
        threadgroup_barrier(mem_flags::mem_threadgroup);
        simdgroup_store(acc00, edge + sg * 128u, 8u);
        simdgroup_store(acc01, edge + sg * 128u + 64u, 8u);
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint q = 0u; q < 2u; ++q) {
            const uint e = lane + q * 32u;
            const uint er = e >> 3u;
            const uint ec = e & 7u;
            const uint gr0 = m0 + sgr * 8u + er;
            const uint gc0 = n0 + sgc * 8u + ec;
            const uint gc1 = gc0 + 64u;
            if (gr0 < m && gc0 < n) { C[gr0 * n + gc0] = edge[sg * 128u + e]; }
            if (gr0 < m && gc1 < n) { C[gr0 * n + gc1] = edge[sg * 128u + 64u + e]; }
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        simdgroup_store(acc10, edge + sg * 128u, 8u);
        simdgroup_store(acc11, edge + sg * 128u + 64u, 8u);
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint q = 0u; q < 2u; ++q) {
            const uint e = lane + q * 32u;
            const uint er = e >> 3u;
            const uint ec = e & 7u;
            const uint gr0 = m0 + sgr * 8u + 32u + er;
            const uint gc0 = n0 + sgc * 8u + ec;
            const uint gc1 = gc0 + 64u;
            if (gr0 < m && gc0 < n) { C[gr0 * n + gc0] = edge[sg * 128u + e]; }
            if (gr0 < m && gc1 < n) { C[gr0 * n + gc1] = edge[sg * 128u + 64u + e]; }
        }
    }
}
"#;

/// The narrow instance: 32×64 output per threadgroup, 16 simdgroups (512
/// threads), two column-twin accumulators per simdgroup (columns sgc·8 and
/// (sgc+4)·8 of its row block). Same staging laws as the wide instance.
const MSL_SGEMM_NARROW: &str = r#"
constant uint BM = 32u;
constant uint BN = 64u;
constant uint BK = 64u;
constant uint TAS = 65u;
constant uint TBS = 65u;

// out[b][m×n] = A[b][m×k] @ B[b][k×n]; element (i, j) of X at
// x[i·xrs + j·xcs]; batch b's operands start b·bxs elements in (the
// batch-1 call sites pass zero batch strides). Row-major B (b_cs == 1)
// stages coalesced along n; transposed B — the Wᵀ / Kᵀ shapes (b_rs == 1)
// — along k; the staged tile is the SAME [K][N] layout either way.
kernel void sgemm(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant uint& m [[buffer(3)]],
    constant uint& n [[buffer(4)]],
    constant uint& k [[buffer(5)]],
    constant uint& a_rs [[buffer(6)]],
    constant uint& a_cs [[buffer(7)]],
    constant uint& b_rs [[buffer(8)]],
    constant uint& b_cs [[buffer(9)]],
    constant uint& a_bs [[buffer(10)]],
    constant uint& b_bs [[buffer(11)]],
    constant uint& c_bs [[buffer(12)]],
    threadgroup float* raw [[threadgroup(13)]],
    uint3 gtp [[threadgroup_position_in_grid]],
    uint lid [[thread_index_in_threadgroup]])
{
    // Carved staging (24 960 B): A tile [32][65] = 2080 floats, B tile
    // [64][65] = 4160; BK 64 halves the k-loop's two barriers per unit of
    // work against the BK 32 geometry. The ragged-edge route reuses the
    // front 2048 floats after the k-loop.
    threadgroup float* ta = raw;
    threadgroup float* tb = raw + 32u * TAS;
    threadgroup float* edge = raw;
    const uint m0 = gtp.y * BM;
    const uint n0 = gtp.x * BN;
    device const float* A = a + gtp.z * a_bs;
    device const float* B = b + gtp.z * b_bs;
    device float* C = out + gtp.z * c_bs;

    const uint sg = lid >> 5u;   // simdgroup id 0..15
    const uint lane = lid & 31u;
    const uint sgr = sg >> 2u;   // 8×8 block row 0..3
    const uint sgc = sg & 3u;    // 8×8 block col 0..3 (the +4 twin below)

    simdgroup_float8x8 acc0 = simdgroup_float8x8(0.0f);
    simdgroup_float8x8 acc1 = simdgroup_float8x8(0.0f);

    for (uint t = 0u; t < k; t += BK) {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint q = 0u; q < 4u; ++q) {
            // A tile [BM][BK] = [32][64]: four elements per thread.
            const uint idx = lid + q * 512u;
            const uint r = idx >> 6u;
            const uint c = idx & 63u;
            const uint gr = m0 + r;
            const uint ac = t + c;
            ta[r * TAS + c] = (gr < m && ac < k) ? A[gr * a_rs + ac * a_cs] : 0.0f;
        }
        for (uint q = 0u; q < 8u; ++q) {
            // B tile [BK][BN] = [64][64]: k ∈ 0..63 from the high bits,
            // n ∈ 0..63 from the low (consecutive lanes → consecutive n,
            // coalesced along B's rows); the staged tile is the SAME [K][N]
            // layout for both B orientations.
            const uint idx = lid + q * 512u;
            const uint kk = idx >> 6u;
            const uint col = idx & 63u;
            const uint bc = t + kk;
            if (b_cs == 1u) {
                tb[kk * TBS + col] =
                    (bc < k && n0 + col < n) ? B[bc * b_rs + n0 + col] : 0.0f;
            } else {
                tb[kk * TBS + col] =
                    (bc < k && n0 + col < n) ? B[bc * b_rs + (n0 + col) * b_cs] : 0.0f;
            }
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint kk = 0u; kk < BK; kk += 8u) {
            simdgroup_float8x8 fa, fb0, fb1;
            simdgroup_load(fa, ta + sgr * 8u * TAS + kk, TAS);
            simdgroup_load(fb0, tb + kk * TBS + sgc * 8u, TBS);
            simdgroup_load(fb1, tb + kk * TBS + (sgc + 4u) * 8u, TBS);
            simdgroup_multiply_accumulate(acc0, fa, fb0, acc0);
            simdgroup_multiply_accumulate(acc1, fa, fb1, acc1);
        }
    }

    if ((m0 + BM <= m) && (n0 + BN <= n)) {
        simdgroup_store(acc0, C + (m0 + sgr * 8u) * n + (n0 + sgc * 8u), n);
        simdgroup_store(acc1, C + (m0 + sgr * 8u) * n + (n0 + sgc * 8u + 4u * 8u), n);
    } else {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        simdgroup_store(acc0, edge + sg * 128u, 8u);
        simdgroup_store(acc1, edge + sg * 128u + 64u, 8u);
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint q = 0u; q < 2u; ++q) {
            const uint e = lane + q * 32u;
            const uint er = e >> 3u;
            const uint ec = e & 7u;
            const uint gr = m0 + sgr * 8u + er;
            const uint gc0 = n0 + sgc * 8u + ec;
            const uint gc1 = gc0 + 4u * 8u;
            if (gr < m && gc0 < n) { C[gr * n + gc0] = edge[sg * 128u + e]; }
            if (gr < m && gc1 < n) { C[gr * n + gc1] = edge[sg * 128u + 64u + e]; }
        }
    }
}
"#;

/// The fused attention kernel (the Metal lane's flash form): ONE dispatch
/// per layer over the packed qkv — split, rope, q-scale, scores, sliding
/// window, softmax and value mix, and the head merge all in-kernel; the
/// seq² scores parent the reference sequence materializes (and its mask
/// add + multi-pass softmax + context re-read) never exist. The accumulator
/// is normalized with the TWO-PASS form: pass 1 walks the key tiles for the
/// row max only, pass 2 recomputes each tile's scores against the FINAL max
/// and accumulates exp(s−m)·V — no running-max rescale of the accumulator,
/// at the cost of a second score MMA and a second K read (the rope partner
/// reads hit the same cache lines). Sliding-window layers predicate on
/// `window` — each query block reads only its [q₀−w, q_end+w] key slice —
/// which is also where the FLOP win lives at seq ≫ window (the english
/// geometry: window 64, ~2/3 sliding layers); `window == seq` (the host
/// clamps) means full attention. The additive mask tensor the reference
/// sequence consumes describes the same allowed set and is never read
/// here.
const MSL_FLASH: &str = r#"
// Self-contained geometry (no sgemm instance constants referenced).
constant uint FBQ = 32u;   // query rows per threadgroup
constant uint FHD = 64u;   // head dim (asserted host-side)
constant uint FTAS = 65u;  // row stride over a 64-wide tile
constant uint FTKS = 33u;  // row stride over the Kᵀ tile's 32-wide kt rows

kernel void flash_attn(
    device const float* qkv [[buffer(0)]],
    device const float* cos [[buffer(1)]],
    device const float* sin [[buffer(2)]],
    device float* out [[buffer(3)]],
    constant uint& seq [[buffer(4)]],
    constant uint& heads [[buffer(5)]],
    constant uint& hd [[buffer(6)]],
    constant uint& window [[buffer(7)]],
    constant float& scale [[buffer(8)]],
    threadgroup float* raw [[threadgroup(9)]],
    uint2 gtp [[threadgroup_position_in_grid]],
    uint lid [[thread_index_in_threadgroup]])
{
    // Carved staging: ta Q tile [32][65] (rope+scale applied), tk Kᵀ tile
    // [64][33] ([kt][key], rope applied — both halves of each rope pair
    // live in the tile), tv V tile [32][65] ([key][hd] natural), ts
    // scores/probs [32][33], st row stats (m in st[0..32], l in
    // st[32..64]). ta is reused for the accumulator after the key loop;
    // the barriers order every reuse.
    threadgroup float* ta = raw;
    threadgroup float* tk = raw + 32u * FTAS;
    threadgroup float* tv = tk + 64u * FTKS;
    threadgroup float* ts = tv + 32u * FTAS;
    threadgroup float* st = ts + 32u * FTKS;

    const uint q0 = gtp.x * FBQ;
    const uint h = gtp.y;
    const uint d = heads * FHD;
    device const float* Q = qkv + h * FHD;
    device const float* K = qkv + d + h * FHD;
    device const float* V = qkv + 2u * d + h * FHD;

    const uint sg = lid >> 5u;   // simdgroup id 0..31
    const uint sgr = sg >> 3u;   // accumulator row group 0..3
    const uint sgc = sg & 7u;    // accumulator col group 0..7

    // Key range [lo, hi): every (q, k) pair with |q − k| ≤ window for
    // every live row q of this block, clamped to the sequence. window
    // arrives clamped to seq (full attention ⇒ lo 0, hi seq).
    const uint last_row = min(q0 + FBQ - 1u, seq - 1u);
    const uint lo = (q0 > window) ? (q0 - window) : 0u;
    const uint hi = min(last_row + window + 1u, seq);

    // Stage the Q block once — one rope pair per thread, (r, j) and
    // (r, j + 32): rotate-half RoPE, then the 1/√hd scale (the reference
    // sequence's rope-then-scale order). Rows past seq stage zeros (the
    // ragged block tail; their outputs are never stored).
    {
        const uint r = lid >> 5u;
        const uint j = lid & 31u;
        const uint row = q0 + r;
        float qa = 0.0f, qb = 0.0f;
        float c = 0.0f, s = 0.0f;
        if (row < seq) {
            qa = Q[row * (3u * d) + j];
            qb = Q[row * (3u * d) + 32u + j];
            c = cos[row * FHD + j];
            s = sin[row * FHD + j];
        }
        ta[r * FTAS + j] = (qa * c - qb * s) * scale;
        ta[r * FTAS + 32u + j] = (qb * c + qa * s) * scale;
    }

    // Pass 1 — running row max only.
    if (lid < 32u) { st[lid] = -3.402823466e+38f; }
    for (uint t = lo; t < hi; t += FBQ) {
        const uint wk = min(FBQ, hi - t);   // live keys in this tile
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // Stage Kᵀ: one rope pair per thread — (kt j, j+32) × key. Reads
        // both pair elements (the partner is outside the staged kt half
        // but the same cache row); rotates, then writes both kt rows.
        {
            const uint j = lid >> 5u;    // rope pair index 0..31
            const uint key = lid & 31u;  // tile key 0..31
            const uint krow = t + key;
            float ka = 0.0f, kb = 0.0f;
            float c = 0.0f, s = 0.0f;
            if (krow < hi) {
                ka = K[krow * (3u * d) + j];
                kb = K[krow * (3u * d) + 32u + j];
                c = cos[krow * FHD + j];
                s = sin[krow * FHD + j];
            }
            tk[j * FTKS + key] = ka * c - kb * s;
            tk[(j + 32u) * FTKS + key] = kb * c + ka * s;
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // scores frag = Q tile × Kᵀ over the full hd contraction. The
        // scores tile is [32 rows][32 keys] — only the four key-col groups
        // (sgc < 4) hold live frags; sgc ≥ 4 must neither compute nor
        // store (their store would run past the 32-key row into the next
        // row of the [32][33] buffer).
        if (sgc < 4u) {
            simdgroup_float8x8 sf = simdgroup_float8x8(0.0f);
            for (uint kk = 0u; kk < FHD; kk += 8u) {
                simdgroup_float8x8 fa, fb;
                simdgroup_load(fa, ta + sgr * 8u * FTAS + kk, FTAS);
                simdgroup_load(fb, tk + kk * FTKS + sgc * 8u, FTKS);
                simdgroup_multiply_accumulate(sf, fa, fb, sf);
            }
            simdgroup_store(sf, ts + sgr * 8u * FTKS + sgc * 8u, FTKS);
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // Per-row window predicate: the block key range is only a bounds
        // optimization — within a tile a key is live for a row only when
        // |q − k| ≤ window (the reference's mask row; out-of-window keys
        // are exp(f32::MIN − m) = 0 there, so skipping them is exact).
        if (lid < 32u) {
            const uint q = q0 + lid;
            float m = st[lid];
            for (uint c = 0u; c < wk; ++c) {
                const uint k = t + c;
                const uint dk = (k > q) ? (k - q) : (q - k);
                if (dk <= window) { m = max(m, ts[lid * FTKS + c]); }
            }
            st[lid] = m;
        }
    }
    if (lid < 32u) { st[32u + lid] = 0.0f; }

    // Pass 2 — recompute each tile against the final max, accumulate
    // exp(s − m)·V and the per-row l in registers (the same lane owns row
    // `lane` every tile). ONE accumulator frag per simdgroup: 32 sg as 4
    // row groups × 8 col groups cover the [32][64] output exactly.
    float l_reg = 0.0f;
    simdgroup_float8x8 acc = simdgroup_float8x8(0.0f);
    for (uint t = lo; t < hi; t += FBQ) {
        const uint wk = min(FBQ, hi - t);
        threadgroup_barrier(mem_flags::mem_threadgroup);
        {
            const uint j = lid >> 5u;
            const uint key = lid & 31u;
            const uint krow = t + key;
            float ka = 0.0f, kb = 0.0f;
            float c = 0.0f, s = 0.0f;
            if (krow < hi) {
                ka = K[krow * (3u * d) + j];
                kb = K[krow * (3u * d) + 32u + j];
                c = cos[krow * FHD + j];
                s = sin[krow * FHD + j];
            }
            tk[j * FTKS + key] = ka * c - kb * s;
            tk[(j + 32u) * FTKS + key] = kb * c + ka * s;
        }
        for (uint q = 0u; q < 2u; ++q) {
            // V tile [32 key][64 hd]: two elements per thread, natural
            // layout (no rope).
            const uint idx = lid + q * 1024u;
            const uint key = idx >> 6u;
            const uint col = idx & 63u;
            const uint krow = t + key;
            tv[key * FTAS + col] =
                (krow < hi) ? V[krow * (3u * d) + col] : 0.0f;
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        if (sgc < 4u) {
            simdgroup_float8x8 sf = simdgroup_float8x8(0.0f);
            for (uint kk = 0u; kk < FHD; kk += 8u) {
                simdgroup_float8x8 fa, fb;
                simdgroup_load(fa, ta + sgr * 8u * FTAS + kk, FTAS);
                simdgroup_load(fb, tk + kk * FTKS + sgc * 8u, FTKS);
                simdgroup_multiply_accumulate(sf, fa, fb, sf);
            }
            simdgroup_store(sf, ts + sgr * 8u * FTKS + sgc * 8u, FTKS);
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        if (lid < 32u) {
            const float m = st[lid];
            const uint q = q0 + lid;
            float acc = 0.0f;
            for (uint c = 0u; c < wk; ++c) {
                const uint k = t + c;
                const uint dk = (k > q) ? (k - q) : (q - k);
                float p = 0.0f;
                if (dk <= window) { p = precise::exp(ts[lid * FTKS + c] - m); }
                ts[lid * FTKS + c] = p;
                acc += p;
            }
            l_reg += acc;
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // acc += P × V over this tile's 32 keys (padding columns hold
        // exp(0 − m) against V rows staged zero — contributes exactly 0;
        // l skips them, so the normalization stays exact).
        for (uint kk = 0u; kk < FBQ; kk += 8u) {
            simdgroup_float8x8 fa, fb;
            simdgroup_load(fa, ts + sgr * 8u * FTKS + kk, FTKS);
            simdgroup_load(fb, tv + kk * FTAS + sgc * 8u, FTAS);
            simdgroup_multiply_accumulate(acc, fa, fb, acc);
        }
    }
    // Publish the per-row l (each lane < 32 owns row `lid`'s register).
    if (lid < 32u) { st[32u + lid] = l_reg; }

    // Drain: the accumulator frag → the ta front (Q is dead), then the
    // per-row 1/l normalize + the merged-heads store [seq, d].
    threadgroup_barrier(mem_flags::mem_threadgroup);
    simdgroup_store(acc, ta + (sgr * 8u) * 64u + sgc * 8u, 64u);
    threadgroup_barrier(mem_flags::mem_threadgroup);
    {
        const uint r = lid >> 5u;
        const uint j = lid & 31u;
        const uint row = q0 + r;
        if (row < seq) {
            const float inv = 1.0f / st[32u + r];
            const uint ob = row * d + h * FHD;
            out[ob + j] = ta[r * 64u + j] * inv;
            out[ob + 32u + j] = ta[r * 64u + 32u + j] * inv;
        }
    }
}
"#;

/// The kernels after the two sgemm instances.
const MSL_TAIL: &str = r#"

kernel void add(device float* x [[buffer(0)]],
                device const float* y [[buffer(1)]],
                constant uint& len [[buffer(2)]],
                uint gid [[thread_position_in_grid]]) {
    if (gid < len) { x[gid] += y[gid]; }
}

kernel void copy(device float* dst [[buffer(0)]],
                 device const float* src [[buffer(1)]],
                 constant uint& len [[buffer(2)]],
                 uint gid [[thread_position_in_grid]]) {
    if (gid < len) { dst[gid] = src[gid]; }
}

kernel void add_bias_row(device float* x [[buffer(0)]],
                         device const float* bias [[buffer(1)]],
                         constant uint& len [[buffer(2)]],
                         constant uint& d [[buffer(3)]],
                         uint gid [[thread_position_in_grid]]) {
    if (gid < len) { x[gid] += bias[gid % d]; }
}

kernel void scale(device float* x [[buffer(0)]],
                  constant uint& len [[buffer(1)]],
                  constant float& s [[buffer(2)]],
                  uint gid [[thread_position_in_grid]]) {
    if (gid < len) { x[gid] *= s; }
}

kernel void relu(device float* x [[buffer(0)]],
                 constant uint& len [[buffer(1)]],
                 uint gid [[thread_position_in_grid]]) {
    if (gid < len && x[gid] < 0.0f) { x[gid] = 0.0f; }
}

kernel void gelu_erf(device float* x [[buffer(0)]],
                     constant uint& len [[buffer(1)]],
                     uint gid [[thread_position_in_grid]]) {
    if (gid < len) { x[gid] = gelu_as(x[gid]); }
}

// out[r, j] = gelu_erf(fused[r, j]) * fused[r, I + j] — the CPU lane's
// exact multiply order ((e+1)·0.5·v, then ·g).
kernel void glu_gelu_gate(device const float* fused [[buffer(0)]],
                          device float* out [[buffer(1)]],
                          constant uint& rows [[buffer(2)]],
                          constant uint& i_sz [[buffer(3)]],
                          uint gid [[thread_position_in_grid]]) {
    if (gid >= rows * i_sz) { return; }
    const uint r = gid / i_sz;
    const uint j = gid % i_sz;
    const uint base = r * 2u * i_sz;
    const float act = gelu_as(fused[base + j]);
    out[gid] = act * fused[base + i_sz + j];
}

// One threadgroup (one simdgroup) per row: strided lanes + simd
// reductions. Mean → centered² → 1/sqrt(var+eps) → (v−mean)·inv·w, the
// CPU op order; inv_d arrives as the CPU lane's f64-rounded constant.
kernel void ln_rows(device const float* x [[buffer(0)]],
                    device const float* w [[buffer(1)]],
                    device float* out [[buffer(2)]],
                    constant uint& rows [[buffer(3)]],
                    constant uint& d [[buffer(4)]],
                    constant float& inv_d [[buffer(5)]],
                    constant float& eps [[buffer(6)]],
                    uint tpg [[threadgroup_position_in_grid]],
                    uint lane [[thread_index_in_threadgroup]]) {
    if (tpg >= rows) { return; }
    device const float* row = x + tpg * d;
    device float* orow = out + tpg * d;
    float acc = 0.0f;
    for (uint i = lane; i < d; i += 32u) { acc += row[i]; }
    const float mean = simd_sum(acc) * inv_d;
    float vacc = 0.0f;
    for (uint i = lane; i < d; i += 32u) {
        const float c = row[i] - mean;
        vacc += c * c;
    }
    const float var = simd_sum(vacc) * inv_d;
    const float inv = 1.0f / sqrt(var + eps);
    for (uint i = lane; i < d; i += 32u) { orow[i] = (row[i] - mean) * inv * w[i]; }
}

// One threadgroup (one simdgroup) per row: strided lanes + simd
// reductions. The v1 kernel ran one thread per row — 32× the GPU idle.
kernel void softmax_rows(device float* x [[buffer(0)]],
                         constant uint& rows [[buffer(1)]],
                         constant uint& n [[buffer(2)]],
                         uint tpg [[threadgroup_position_in_grid]],
                         uint lane [[thread_index_in_threadgroup]]) {
    if (tpg >= rows) { return; }
    device float* row = x + tpg * n;
    float mx = -3.402823466e+38f;
    for (uint i = lane; i < n; i += 32u) { mx = fmax(mx, row[i]); }
    mx = simd_max(mx);
    float sum = 0.0f;
    for (uint i = lane; i < n; i += 32u) {
        const float e = precise::exp(row[i] - mx);
        row[i] = e;
        sum += e;
    }
    sum = simd_sum(sum);
    const float inv = 1.0f / sum;
    for (uint i = lane; i < n; i += 32u) { row[i] *= inv; }
}

// q[(h·seq+pos)·hd + j] paired with its +half twin; one thread per
// (h·seq+pos, j).
kernel void rope(device float* q [[buffer(0)]],
                 device const float* cos_t [[buffer(1)]],
                 device const float* sin_t [[buffer(2)]],
                 constant uint& seq [[buffer(3)]],
                 constant uint& heads [[buffer(4)]],
                 constant uint& hd [[buffer(5)]],
                 uint gid [[thread_position_in_grid]]) {
    const uint hf = hd / 2u;
    if (gid >= heads * seq * hf) { return; }
    const uint hps = gid / hf;
    const uint j = gid % hf;
    const uint base = hps * hd;
    const uint prow = (hps % seq) * hd;
    const float c = cos_t[prow + j];
    const float s = sin_t[prow + j];
    const float q1 = q[base + j];
    const float q2 = q[base + hf + j];
    q[base + j] = q1 * c - q2 * s;
    q[base + hf + j] = q2 * c + q1 * s;
}

kernel void split_heads(device const float* src [[buffer(0)]],
                        device float* out [[buffer(1)]],
                        constant uint& row_stride [[buffer(2)]],
                        constant uint& off [[buffer(3)]],
                        constant uint& seq [[buffer(4)]],
                        constant uint& heads [[buffer(5)]],
                        constant uint& hd [[buffer(6)]],
                        uint gid [[thread_position_in_grid]]) {
    if (gid >= heads * seq * hd) { return; }
    const uint h = gid / (seq * hd);
    const uint r = gid % (seq * hd);
    const uint s = r / hd;
    const uint i = r % hd;
    out[gid] = src[s * row_stride + off + h * hd + i];
}

kernel void merge_heads(device const float* src [[buffer(0)]],
                        device float* out [[buffer(1)]],
                        constant uint& seq [[buffer(2)]],
                        constant uint& heads [[buffer(3)]],
                        constant uint& hd [[buffer(4)]],
                        uint gid [[thread_position_in_grid]]) {
    const uint d = heads * hd;
    if (gid >= seq * d) { return; }
    const uint s = gid / d;
    const uint rem = gid % d;
    const uint h = rem / hd;
    const uint i = rem % hd;
    out[gid] = src[(h * seq + s) * hd + i];
}

kernel void gather_rows(device const float* x [[buffer(0)]],
                        device const uint* rows [[buffer(1)]],
                        device float* out [[buffer(2)]],
                        constant uint& d [[buffer(3)]],
                        uint gid [[thread_position_in_grid]]) {
    const uint r = gid / d;
    const uint i = gid % d;
    out[gid] = x[rows[r] * d + i];
}

// scores[r] += mask[r % mlen] — the sliding-window mask broadcast over
// every head's slab of the scores parent in ONE dispatch (was one add
// dispatch per head).
kernel void add_mask_bcast(device float* x [[buffer(0)]],
                           device const float* mask [[buffer(1)]],
                           constant uint& len [[buffer(2)]],
                           constant uint& mlen [[buffer(3)]],
                           uint gid [[thread_position_in_grid]]) {
    if (gid < len) { x[gid] += mask[gid % mlen]; }
}
"#;

fn rt(detail: impl std::fmt::Display) -> LayaError {
    LayaError::Runtime(format!("riir metal backend: {detail}"))
}

/// A pass-scoped command buffer: created on the first encode after a sync,
/// appended-to by every op, committed at the sync (or the encoder cap).
struct PendingPass {
    cb: CommandBuffer,
    encodes: u32,
}

/// The open pass buffer plus the committed-but-unwaited pipeline flushes.
/// `sync()` waits both — a flush must never let a `download_into` read
/// ahead of in-flight writes.
#[derive(Default)]
struct PendingState {
    open: Option<PendingPass>,
    committed: Vec<CommandBuffer>,
}

/// The Metal backend: one device + queue, the compiled-once kernel library,
/// the permanent weight cache, and the generation-keyed activation cache.
pub struct Metal {
    device: Device,
    queue: CommandQueue,
    pipelines: HashMap<&'static str, ComputePipelineState>,
    /// `(ptr, len)` → device buffer for the agent-OWNED weight slices
    /// (stable addresses and contents for the agent's lifetime: every
    /// `matmul_w` weight, LN scales, biases, the embedding table).
    weights: Mutex<HashMap<(usize, usize), Buffer>>,
    /// `(ptr, len, gen)` → device buffer for activations and per-forward
    /// host-authored inputs. The generation (bumped at every pass) makes a
    /// recycled heap address miss instead of serving a stale epoch's
    /// bytes; within an epoch a hit's device copy is current because the
    /// forward bodies write every activation device-side before reading
    /// it (the write-first audit in the module doc).
    chain: Mutex<HashMap<(usize, usize, u64), Buffer>>,
    /// The pass-scoped command buffer + the committed drain list.
    pending: Mutex<PendingState>,
    /// The sync generation (how many host-read barriers have run).
    epoch: AtomicU64,
    /// Debug kill-switch (`LAYA_METAL_PER_OP_SYNC`): `1` = sync + write
    /// every result back to its host slice after each op (the slow flow);
    /// `2` = sync only (no writeback — bisects barrier vs host-freshness).
    per_op_sync: u8,
    /// Kill-switch (`LAYA_METAL_FLASH=0`): route `attention_forward` back
    /// through the reference op sequence instead of the fused kernel — the
    /// A/B and bisect posture, never a silent default.
    flash_disabled: bool,
    /// Debug-trace instance id.
    trace_id: usize,
}

impl Metal {
    /// Build the backend — fails loud when no Metal device exists (the
    /// candle lane's `device_from_env` precedent; never a silent CPU
    /// fallback).
    pub fn new() -> Result<Self> {
        let Some(device) = Device::system_default() else {
            return Err(rt("LAYA_DEVICE=metal: no Metal device on this host"));
        };
        let queue = device.new_command_queue();
        let msl = format!(
            "{MSL_HEAD}{MSL_SGEMM_NARROW}{MSL_SGEMM_WIDE}{MSL_SGEMM_XWIDE}{MSL_FLASH}{MSL_TAIL}"
        );
        let lib = device
            .new_library_with_source(&msl, &metal::CompileOptions::new())
            .map_err(|e| rt(format!("MSL compile failed: {e}")))?;
        let mut pipelines = HashMap::with_capacity(KERNELS.len());
        for name in KERNELS {
            let f = lib
                .get_function(name, None)
                .map_err(|e| rt(format!("kernel {name}: {e}")))?;
            let p = device
                .new_compute_pipeline_state_with_function(&f)
                .map_err(|e| rt(format!("pipeline {name}: {e}")))?;
            pipelines.insert(*name, p);
        }
        Ok(Self {
            device,
            queue,
            pipelines,
            weights: Mutex::new(HashMap::new()),
            chain: Mutex::new(HashMap::new()),
            pending: Mutex::new(PendingState::default()),
            epoch: AtomicU64::new(0),
            per_op_sync: std::env::var("LAYA_METAL_PER_OP_SYNC")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            flash_disabled: std::env::var("LAYA_METAL_FLASH").as_deref() == Ok("0"),
            trace_id: next_trace_instance(),
        })
    }

    fn upload(&self, data: &[f32]) -> Buffer {
        self.device.new_buffer_with_data(
            data.as_ptr().cast::<c_void>(),
            std::mem::size_of_val(data) as u64,
            RESOURCE_OPTIONS,
        )
    }

    fn upload_u32(&self, data: &[u32]) -> Buffer {
        self.device.new_buffer_with_data(
            data.as_ptr().cast::<c_void>(),
            (data.len() * 4) as u64,
            RESOURCE_OPTIONS,
        )
    }

    /// Device-resident copy of an agent-owned weight slice — permanent
    /// cache, first-miss copy, never invalidated.
    fn weight_buf(&self, data: &[f32]) -> Buffer {
        let key = (data.as_ptr() as usize, data.len());
        let mut map = self.weights.lock().expect("weight cache poison");
        if let Some(b) = map.get(&key) {
            return b.clone();
        }
        let b = self.upload(data);
        map.insert(key, b.clone());
        b
    }

    /// Activation / per-forward input: hit within the current epoch → the
    /// device copy is current (no copy); miss → create + copy.
    fn chain_buf(&self, data: &[f32]) -> Buffer {
        let epoch = self.epoch.load(Ordering::Relaxed);
        let key = (data.as_ptr() as usize, data.len(), epoch);
        let mut map = self.chain.lock().expect("chain cache poison");
        if let Some(b) = map.get(&key) {
            return b.clone();
        }
        let b = self.upload(data);
        map.insert(key, b.clone());
        if trace_enabled() {
            eprintln!(
                "[trace] chain MISS inst {} ptr {:p} len {} epoch {epoch}",
                self.trace_id,
                data.as_ptr(),
                data.len()
            );
        }
        b
    }

    /// A device destination slot for this epoch's `(ptr, len)`. `dst` is
    /// the WHOLE parent slice the op writes into (the forward bodies pass
    /// whole parents + explicit offsets), so per-head loops share one slot
    /// and serial GPU ordering keeps it current. Dsts are write-first, so
    /// a hit reuses the existing buffer.
    fn chain_slot_for(&self, dst: &[f32]) -> Buffer {
        let epoch = self.epoch.load(Ordering::Relaxed);
        let key = (dst.as_ptr() as usize, dst.len(), epoch);
        let mut map = self.chain.lock().expect("chain cache poison");
        if let Some(b) = map.get(&key) {
            return b.clone();
        }
        let b = self.scratch(dst.len());
        map.insert(key, b.clone());
        b
    }

    fn scratch(&self, len_f32: usize) -> Buffer {
        self.device.new_buffer(
            (std::mem::size_of::<f32>() * len_f32) as u64,
            RESOURCE_OPTIONS,
        )
    }

    /// Host-read barrier: commit the open pass buffer, then wait every
    /// committed-but-unwaited buffer (the pipeline flushes included).
    fn sync(&self) {
        let mut st = self.pending.lock().expect("pending cb poison");
        if let Some(pending) = st.open.take() {
            pending.cb.commit();
            st.committed.push(pending.cb);
        }
        for cb in st.committed.drain(..) {
            cb.wait_until_completed();
        }
    }

    /// One forward is beginning: drain any outstanding GPU work (a cleared
    /// chain entry releases its Buffer — that must never happen while
    /// commands still reference it), bump the pass epoch, and drop the
    /// previous pass's slots. Host-authored buffers (rope tables, mask,
    /// `act_in`, the id list) are rebuilt per forward — often at recycled
    /// heap addresses with fresh contents — so last pass's keys must never
    /// hit; within a pass the serial queue keeps every slot device-current.
    fn begin_pass_impl(&self) {
        self.sync();
        self.epoch.fetch_add(1, Ordering::Relaxed);
        self.chain.lock().expect("chain cache poison").clear();
    }

    fn copy_out(buf: &Buffer, out: &mut [f32]) {
        // SAFETY: the buffer was created with ≥ out.len() f32 elements in
        // storage mode shared, and the queue has been synced, so its
        // contents are host-readable.
        unsafe {
            std::ptr::copy_nonoverlapping(
                buf.contents().cast::<f32>(),
                out.as_mut_ptr(),
                out.len(),
            );
        }
    }

    /// Per-op debug sync: barrier + (mode 1) write the slot's whole
    /// content back to its host slice (host bytes stay current — the
    /// pre-lazy semantics). Mode 2 barriers only.
    fn debug_writeback(&self, slot: &Buffer, dst: &mut [f32]) {
        if self.per_op_sync >= 1 {
            self.sync();
            if self.per_op_sync == 1 {
                Self::copy_out(slot, dst);
            }
        }
    }

    /// Encode one kernel dispatch into the PASS command buffer (no commit
    /// — the lazy shape; commit + wait happens in [`Self::sync`], or a
    /// pipelined no-wait flush at the encoder cap). `buffers` bind at
    /// `[[buffer(0..)]]`, then `uargs`/`fargs` as 4-byte `constant`
    /// scalars, then any staging buffers. `threadgroups` selects
    /// `dispatch_thread_groups` (the 2D/3D kernels) over
    /// `dispatch_threads` (the 1D elementwise kernels). Runs inside an
    /// autoreleasepool — the autoreleased encoders drain per op instead
    /// of accumulating on a thread with no Cocoa runloop.
    #[allow(clippy::too_many_arguments)]
    fn encode(
        &self,
        p: &ComputePipelineState,
        buffers: &[(&Buffer, u64)],
        uargs: &[u32],
        fargs: &[f32],
        grid: MTLSize,
        tpg: MTLSize,
        tiles: Option<&[(u64, u64)]>,
        threadgroups: bool,
    ) -> Result<()> {
        autoreleasepool(|_| {
            let mut st = self.pending.lock().expect("pending cb poison");
            if st.open.is_none() {
                st.open = Some(PendingPass {
                    cb: self.queue.new_command_buffer().to_owned(),
                    encodes: 0,
                });
            }
            let pending = st.open.as_mut().expect("just inserted");
            let enc = pending.cb.new_compute_command_encoder();
            enc.set_compute_pipeline_state(p);
            for (i, (b, off)) in buffers.iter().enumerate() {
                enc.set_buffer(i as u64, Some(b), *off);
            }
            let mut idx = buffers.len() as u64;
            for v in uargs {
                enc.set_bytes(idx, 4, std::ptr::from_ref(v).cast::<c_void>());
                idx += 1;
            }
            for v in fargs {
                enc.set_bytes(idx, 4, std::ptr::from_ref(v).cast::<c_void>());
                idx += 1;
            }
            if let Some(tiles) = tiles {
                for (base, bytes) in tiles {
                    enc.set_threadgroup_memory_length(*base, *bytes);
                }
            }
            if threadgroups {
                enc.dispatch_thread_groups(grid, tpg);
            } else {
                enc.dispatch_threads(grid, tpg);
            }
            enc.end_encoding();
            pending.encodes += 1;
            if pending.encodes >= MAX_ENCODERS_PER_CB {
                let done = st.open.take().expect("checked above");
                done.cb.commit();
                st.committed.push(done.cb);
            }
        });
        Ok(())
    }

    /// The 1D elementwise dispatch.
    fn run(
        &self,
        kernel: &'static str,
        buffers: &[&Buffer],
        uargs: &[u32],
        fargs: &[f32],
        len: u64,
    ) -> Result<()> {
        let p = self
            .pipelines
            .get(kernel)
            .ok_or_else(|| rt(format!("kernel {kernel} missing")))?;
        let width = p.thread_execution_width();
        let groups = len.div_ceil(width).max(1);
        let bufs: Vec<(&Buffer, u64)> = buffers.iter().map(|b| (*b, 0)).collect();
        self.encode(
            p,
            &bufs,
            uargs,
            fargs,
            MTLSize {
                width: groups * width,
                height: 1,
                depth: 1,
            },
            MTLSize {
                width,
                height: 1,
                depth: 1,
            },
            None,
            false,
        )
    }

    /// The 1D dispatch at explicit operand byte offsets (whole-parent
    /// slots + offsets, the `add` mask-slab shape).
    #[allow(clippy::too_many_arguments)]
    fn run_at(
        &self,
        kernel: &'static str,
        a: (&Buffer, u64),
        b: (&Buffer, u64),
        uargs: &[u32],
        fargs: &[f32],
        len: u64,
    ) -> Result<()> {
        let p = self
            .pipelines
            .get(kernel)
            .ok_or_else(|| rt(format!("kernel {kernel} missing")))?;
        let width = p.thread_execution_width();
        let groups = len.div_ceil(width).max(1);
        self.encode(
            p,
            &[a, b],
            uargs,
            fargs,
            MTLSize {
                width: groups * width,
                height: 1,
                depth: 1,
            },
            MTLSize {
                width,
                height: 1,
                depth: 1,
            },
            None,
            false,
        )
    }

    /// The row-parallel reduction kernels (softmax / LN): ONE threadgroup —
    /// one simdgroup — per row, 32 threads, no staging memory.
    fn run_rows(
        &self,
        kernel: &'static str,
        buffers: &[&Buffer],
        uargs: &[u32],
        fargs: &[f32],
        rows: u64,
    ) -> Result<()> {
        let p = self
            .pipelines
            .get(kernel)
            .ok_or_else(|| rt(format!("kernel {kernel} missing")))?;
        let bufs: Vec<(&Buffer, u64)> = buffers.iter().map(|b| (*b, 0)).collect();
        self.encode(
            p,
            &bufs,
            uargs,
            fargs,
            MTLSize {
                width: rows.max(1),
                height: 1,
                depth: 1,
            },
            MTLSize {
                width: 32,
                height: 1,
                depth: 1,
            },
            None,
            true,
        )
    }

    /// The batched simdgroup GEMM dispatch: grid = (⌈n/BN⌉, ⌈m/BM⌉, batch),
    /// the instance picked per `m` and `n` (see [`WIDE_M_MIN`] /
    /// [`XWIDE_N_MIN`]). `uargs` =
    /// [m, n, k, `a_rs`, `a_cs`, `b_rs`, `b_cs`, `a_bs`, `b_bs`, `c_bs`] — element
    /// strides, then per-batch element strides (batch-1 callers pass
    /// zeros).
    #[allow(clippy::too_many_arguments)]
    fn run_sgemm(
        &self,
        a: (&Buffer, u64),
        b: (&Buffer, u64),
        out: (&Buffer, u64),
        uargs: &[u32; 10],
        m: u32,
        n: u32,
        batch: u32,
    ) -> Result<()> {
        let (name, bm, bn, staging, threads) = if m as usize >= WIDE_M_MIN {
            if n as usize >= XWIDE_N_MIN {
                (
                    "sgemm_xwide",
                    64u64,
                    128u64,
                    XWIDE_STAGING_BYTES,
                    XWIDE_THREADS,
                )
            } else {
                ("sgemm_wide", 64, 64, WIDE_STAGING_BYTES, WIDE_THREADS)
            }
        } else {
            ("sgemm", 32, 64, NARROW_STAGING_BYTES, NARROW_THREADS)
        };
        let p = self
            .pipelines
            .get(name)
            .ok_or_else(|| rt(format!("kernel {name} missing")))?;
        self.encode(
            p,
            &[a, b, out],
            uargs,
            &[],
            MTLSize {
                width: u64::from(n.div_ceil(bn as u32)),
                height: u64::from(m).div_ceil(bm),
                depth: u64::from(batch),
            },
            MTLSize {
                width: threads,
                height: 1,
                depth: 1,
            },
            Some(&[(13, staging)]),
            true,
        )
    }
}

/// Classification of every backend arg (the lazy-sync correctness
/// contract, argued in the module doc):
/// - `weight_buf` — agent-owned, stable for the agent's lifetime
///   (`matmul_w` weights, LN scales, biases, the embedding table);
/// - `chain_buf` — activations + per-forward host-authored inputs, always
///   the WHOLE parent buffer (per-head access goes through explicit offset
///   args, so one slot per parent and serial GPU ordering makes every hit
///   device-current);
/// - `chain_slot_for` — destinations (one slot per parent extent per epoch,
///   results stay device-resident until `download_into`);
/// - everything else (`rows` in gather) — fresh upload per call.
#[allow(clippy::too_many_arguments)]
impl Backend for Metal {
    fn name(&self) -> &'static str {
        "metal"
    }

    fn matmul(
        &self,
        a: &[f32],
        a_off: usize,
        m: usize,
        k: usize,
        b: &[f32],
        b_off: usize,
        n: usize,
        dst: &mut [f32],
        dst_off: usize,
    ) {
        assert!(a.len() >= m * k + a_off, "lhs extent");
        assert!(b.len() >= k * n + b_off, "rhs extent");
        assert!(dst.len() >= m * n + dst_off, "dst extent");
        let ab = self.chain_buf(a);
        // b is an ACTIVATION here (the probs@v rhs) — the chain class.
        let bb = self.chain_buf(b);
        let ob = self.chain_slot_for(dst);
        self.run_sgemm(
            (&ab, (a_off * 4) as u64),
            (&bb, (b_off * 4) as u64),
            (&ob, (dst_off * 4) as u64),
            &[
                m as u32, n as u32, k as u32, k as u32, // a_rs
                1,        // a_cs
                n as u32, // b_rs
                1,        // b_cs
                0,        // batch strides (batch = 1)
                0, 0,
            ],
            m as u32,
            n as u32,
            1,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, dst);
    }

    fn matmul_w(&self, a: &[f32], m: usize, k: usize, w: &[f32], n: usize, dst: &mut [f32]) {
        assert_eq!(a.len(), m * k, "lhs extent");
        assert_eq!(w.len(), n * k, "weight extent");
        assert_eq!(dst.len(), m * n, "dst extent");
        let ab = self.chain_buf(a);
        let wb = self.weight_buf(w);
        let ob = self.chain_slot_for(dst);
        self.run_sgemm(
            (&ab, 0),
            (&wb, 0),
            (&ob, 0),
            &[
                m as u32, n as u32, k as u32, k as u32, // a_rs
                1,        // a_cs
                1,        // b_rs — B = Wᵀ, W row-major [n, k]
                k as u32, // b_cs
                0,        // batch strides (batch = 1)
                0, 0,
            ],
            m as u32,
            n as u32,
            1,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, dst);
    }

    fn matmul_kt(
        &self,
        q: &[f32],
        q_off: usize,
        m: usize,
        hd: usize,
        k: &[f32],
        k_off: usize,
        dst: &mut [f32],
        dst_off: usize,
    ) {
        assert!(q.len() >= m * hd + q_off, "q extent");
        assert!(k.len() >= m * hd + k_off, "k extent");
        assert!(dst.len() >= m * m + dst_off, "dst extent");
        let qb = self.chain_buf(q);
        // k is an ACTIVATION here (the per-head K matrix) — the chain class.
        let kb = self.chain_buf(k);
        let ob = self.chain_slot_for(dst);
        self.run_sgemm(
            (&qb, (q_off * 4) as u64),
            (&kb, (k_off * 4) as u64),
            (&ob, (dst_off * 4) as u64),
            &[
                m as u32, m as u32, hd as u32, hd as u32, // a_rs
                1,         // a_cs
                1,         // b_rs — B = Kᵀ, K row-major [m, hd]
                hd as u32, // b_cs
                0,         // batch strides (batch = 1)
                0, 0,
            ],
            m as u32,
            m as u32,
            1,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, dst);
    }

    /// The fused attention block: ONE dispatch per layer over the packed
    /// qkv (see [`MSL_FLASH`]) — the split/rope/scale/scores/mask/softmax/
    /// value-mix/merge sequence and its seq² scores parent collapse into a
    /// single kernel that walks only each query block's windowed key slice.
    /// `LAYA_METAL_FLASH=0` falls back to the reference sequence (the
    /// kill-switch; also the bisect posture for a kernel-shaped divergence).
    #[allow(clippy::too_many_arguments)]
    fn attention_forward(
        &self,
        qkv: &[f32],
        rope_cos: &[f32],
        rope_sin: &[f32],
        scale: f32,
        seq: usize,
        heads: usize,
        hd: usize,
        window: usize,
        mask: Option<&[f32]>,
        scratch: &mut AttnScratch,
        out: &mut [f32],
    ) {
        if self.flash_disabled || hd != FLASH_HD {
            // The reference sequence consumes the mask tensor; the fused
            // kernel predicates on the window (the same allowed set). hd
            // outside the pinned geometry takes the reference path too.
            return self.attention_forward_default(
                qkv, rope_cos, rope_sin, scale, seq, heads, hd, window, mask, scratch, out,
            );
        }
        let _ = mask; // the window describes the same allowed set
        let qb = self.chain_buf(qkv);
        let cb = self.chain_buf(rope_cos);
        let sb = self.chain_buf(rope_sin);
        let ob = self.chain_slot_for(out);
        // window clamped to seq: full attention ⇒ lo 0 / hi seq in-kernel
        // (and no u32 overflow in the key-range arithmetic).
        let w = window.min(seq) as u32;
        let p = self
            .pipelines
            .get("flash_attn")
            .ok_or_else(|| rt("kernel flash_attn missing"))
            .unwrap_or_else(|e| panic!("{e}"));
        self.encode(
            p,
            &[(&qb, 0), (&cb, 0), (&sb, 0), (&ob, 0)],
            &[seq as u32, heads as u32, hd as u32, w],
            &[scale],
            MTLSize {
                width: u64::from(seq as u32).div_ceil(FLASH_BQ),
                height: u64::from(heads as u32),
                depth: 1,
            },
            MTLSize {
                width: FLASH_THREADS,
                height: 1,
                depth: 1,
            },
            Some(&[(9, FLASH_STAGING_BYTES)]),
            true,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    /// scores[h] ← q[h]·k[h]ᵀ for every head in ONE dispatch.
    fn matmul_kt_heads(
        &self,
        q: &[f32],
        k: &[f32],
        heads: usize,
        m: usize,
        hd: usize,
        dst: &mut [f32],
    ) {
        assert_eq!(q.len(), heads * m * hd, "q extent");
        assert_eq!(k.len(), heads * m * hd, "k extent");
        assert_eq!(dst.len(), heads * m * m, "dst extent");
        let qb = self.chain_buf(q);
        let kb = self.chain_buf(k);
        let ob = self.chain_slot_for(dst);
        self.run_sgemm(
            (&qb, 0),
            (&kb, 0),
            (&ob, 0),
            &[
                m as u32,
                m as u32,
                hd as u32,
                hd as u32,       // a_rs
                1,               // a_cs
                1,               // b_rs — B = Kᵀ, K row-major [m, hd]
                hd as u32,       // b_cs
                (m * hd) as u32, // a_bs
                (m * hd) as u32, // b_bs
                (m * m) as u32,  // c_bs
            ],
            m as u32,
            m as u32,
            heads as u32,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, dst);
    }

    /// ctx[h] ← probs[h] @ v[h] for every head in ONE dispatch.
    fn matmul_heads(
        &self,
        a: &[f32],
        b: &[f32],
        heads: usize,
        m: usize,
        k: usize,
        n: usize,
        dst: &mut [f32],
    ) {
        assert_eq!(a.len(), heads * m * k, "a extent");
        assert_eq!(b.len(), heads * k * n, "b extent");
        assert_eq!(dst.len(), heads * m * n, "dst extent");
        let ab = self.chain_buf(a);
        let bb = self.chain_buf(b);
        let ob = self.chain_slot_for(dst);
        self.run_sgemm(
            (&ab, 0),
            (&bb, 0),
            (&ob, 0),
            &[
                m as u32,
                n as u32,
                k as u32,
                k as u32,       // a_rs
                1,              // a_cs
                n as u32,       // b_rs
                1,              // b_cs
                (m * k) as u32, // a_bs
                (k * n) as u32, // b_bs
                (m * n) as u32, // c_bs
            ],
            m as u32,
            n as u32,
            heads as u32,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, dst);
    }

    fn add(&self, x: &mut [f32], x_off: usize, y: &[f32], y_off: usize, len: usize) {
        assert!(x.len() >= len + x_off, "add x extent");
        assert!(y.len() >= len + y_off, "add y extent");
        let xb = self.chain_buf(x);
        // y: chain either way — attn_out (device-current) or the per-forward
        // mask (fresh this epoch, hit on later layers).
        let yb = self.chain_buf(y);
        self.run_at(
            "add",
            (&xb, (x_off * 4) as u64),
            (&yb, (y_off * 4) as u64),
            &[len as u32],
            &[],
            len as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    /// scores[r] += mask[r % mlen] over the whole heads·seq² parent — the
    /// sliding-window mask for every head in ONE dispatch.
    fn add_mask_broadcast(&self, x: &mut [f32], mask: &[f32], heads: usize) {
        let mlen = mask.len();
        assert!(heads > 0, "heads");
        assert_eq!(x.len(), heads * mlen, "mask broadcast extent");
        let xb = self.chain_buf(x);
        let mb = self.chain_buf(mask);
        let len = x.len();
        self.run_at(
            "add_mask_bcast",
            (&xb, 0),
            (&mb, 0),
            &[len as u32, mlen as u32],
            &[],
            len as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn add_bias_row(&self, x: &mut [f32], d: usize, bias: &[f32]) {
        assert_eq!(bias.len(), d, "bias extent");
        assert_eq!(x.len() % d, 0, "row extent");
        let xb = self.chain_buf(x);
        let bb = self.weight_buf(bias);
        self.run(
            "add_bias_row",
            &[&xb, &bb],
            &[x.len() as u32, d as u32],
            &[],
            x.len() as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn scale(&self, x: &mut [f32], s: f32) {
        let xb = self.chain_buf(x);
        self.run("scale", &[&xb], &[x.len() as u32], &[s], x.len() as u64)
            .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn layer_norm_nobias_into(
        &self,
        x: &[f32],
        w: &[f32],
        eps: f32,
        d: usize,
        _sq: &mut Vec<f32>,
        out: &mut [f32],
    ) {
        assert_eq!(w.len(), d, "norm extent");
        assert_eq!(x.len(), out.len(), "ln extent");
        assert_eq!(x.len() % d, 0, "row extent");
        // The CPU lane's exact mean scale: f64 1/d rounded to f32.
        let inv_d = (1f64 / d as f64) as f32;
        let rows = x.len() / d;
        let xb = self.chain_buf(x);
        let wb = self.weight_buf(w);
        let ob = self.chain_slot_for(out);
        self.run_rows(
            "ln_rows",
            &[&xb, &wb, &ob],
            &[rows as u32, d as u32],
            &[inv_d, eps],
            rows as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    fn softmax_rows(&self, x: &mut [f32], n: usize) {
        assert_eq!(x.len() % n, 0, "softmax row extent");
        let rows = x.len() / n;
        let xb = self.chain_buf(x);
        self.run_rows(
            "softmax_rows",
            &[&xb],
            &[rows as u32, n as u32],
            &[],
            rows as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn relu(&self, x: &mut [f32]) {
        let xb = self.chain_buf(x);
        self.run("relu", &[&xb], &[x.len() as u32], &[], x.len() as u64)
            .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn gelu_erf(&self, x: &mut [f32]) {
        let xb = self.chain_buf(x);
        self.run("gelu_erf", &[&xb], &[x.len() as u32], &[], x.len() as u64)
            .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn glu_gelu_gate(&self, fused: &[f32], rows: usize, i_sz: usize, out: &mut [f32]) {
        assert_eq!(fused.len(), rows * 2 * i_sz, "fused extent");
        assert_eq!(out.len(), rows * i_sz, "glu out extent");
        let fb = self.chain_buf(fused);
        let ob = self.chain_slot_for(out);
        self.run(
            "glu_gelu_gate",
            &[&fb, &ob],
            &[rows as u32, i_sz as u32],
            &[],
            (rows * i_sz) as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    fn apply_rope(
        &self,
        q: &mut [f32],
        seq: usize,
        heads: usize,
        hd: usize,
        cos: &[f32],
        sin: &[f32],
    ) {
        let half = hd / 2;
        assert_eq!(q.len(), heads * seq * hd, "q extent");
        assert_eq!(cos.len(), seq * hd, "cos extent");
        assert_eq!(sin.len(), seq * hd, "sin extent");
        let qb = self.chain_buf(q);
        // cos/sin: rebuilt fresh per forward → fresh this epoch; hit on the
        // layer's second call and on every later layer.
        let cb = self.chain_buf(cos);
        let sb = self.chain_buf(sin);
        self.run(
            "rope",
            &[&qb, &cb, &sb],
            &[seq as u32, heads as u32, hd as u32],
            &[],
            (heads * seq * half) as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&qb, q);
    }

    fn split_heads(
        &self,
        src: &[f32],
        row_stride: usize,
        off: usize,
        seq: usize,
        heads: usize,
        hd: usize,
        out: &mut [f32],
    ) {
        assert_eq!(out.len(), heads * seq * hd, "split extent");
        let sb = self.chain_buf(src);
        let ob = self.chain_slot_for(out);
        self.run(
            "split_heads",
            &[&sb, &ob],
            &[
                row_stride as u32,
                off as u32,
                seq as u32,
                heads as u32,
                hd as u32,
            ],
            &[],
            (heads * seq * hd) as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    fn merge_heads(&self, src: &[f32], seq: usize, heads: usize, hd: usize, out: &mut [f32]) {
        let d = heads * hd;
        assert_eq!(out.len(), seq * d, "merge extent");
        let sb = self.chain_buf(src);
        let ob = self.chain_slot_for(out);
        self.run(
            "merge_heads",
            &[&sb, &ob],
            &[seq as u32, heads as u32, hd as u32],
            &[],
            (seq * d) as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    fn gather_rows(&self, x: &[f32], d: usize, rows: &[usize], out: &mut [f32]) {
        assert_eq!(out.len(), rows.len() * d, "gather extent");
        // x is an ACTIVATION (the head's hidden state) — the chain class
        // (the embedding gather is host-side in the encoder, so no
        // activation-sized weight ever lands in the permanent map).
        let xb = self.chain_buf(x);
        let u32s: Vec<u32> = rows.iter().map(|&r| r as u32).collect();
        let rb = self.upload_u32(&u32s);
        let ob = self.chain_slot_for(out);
        self.run(
            "gather_rows",
            &[&xb, &rb, &ob],
            &[d as u32],
            &[],
            out.len() as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    fn copy_into(&self, src: &[f32], dst: &mut [f32]) {
        assert_eq!(src.len(), dst.len(), "copy extent");
        let sb = self.chain_buf(src);
        let db = self.chain_slot_for(dst);
        self.run(
            "copy",
            &[&db, &sb],
            &[dst.len() as u32],
            &[],
            dst.len() as u64,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }

    fn begin_pass(&self) {
        self.begin_pass_impl();
    }

    fn download_into(&self, src: &[f32], out: &mut [f32]) {
        assert!(src.len() <= out.len(), "download extent");
        self.sync();
        let ptr = src.as_ptr() as usize;
        let map = self.chain.lock().expect("chain cache poison");
        // The src may be a PREFIX of the written buffer (the CLS row is the
        // leading d of the whole hidden slot), so match by base pointer and
        // sufficient extent, taking the newest epoch.
        let slot = map
            .iter()
            .filter(|(k, _)| k.0 == ptr && k.1 >= src.len())
            .max_by_key(|(k, _)| k.2)
            .map(|(_, b)| b.clone());
        let Some(b) = slot else {
            panic!(
                "download_into: no device buffer for this slice — host reads \
                 require a backend-produced buffer (the lazy-sync contract)"
            );
        };
        drop(map);
        Self::copy_out(&b, out);
    }
}
