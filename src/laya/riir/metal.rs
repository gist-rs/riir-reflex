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
//! - **candle's own flush shape (lazy sync)** — ops ENCODE into per-op
//!   command buffers and commit WITHOUT waiting; the wait happens only
//!   when the host genuinely reads a result ([`Backend::download_into`]:
//!   the scorer logits, the CLS row, the act logits — three syncs per
//!   forward). A literal commit+wait per op measured 0.59 ms of round-trip
//!   per dispatch × ~1100 dispatches/forward — 2.5 s/forward, pure sync
//!   overhead; the lazy shape is what candle-core's `Commands` actually
//!   does (flush at CPU readback) and lands in its latency class.
//!   Device memory is the single writer between syncs — forward bodies
//!   never read op outputs host-side except through `download_into`.
//! - **generation-keyed chain cache** — activations flow device-side, so
//!   scratch slices are cached by `(ptr, len, generation)` with the
//!   generation bumped at every sync: host-authored buffers (rope tables,
//!   mask, `act_in`) rebuilt at recycled heap addresses can never hit a
//!   stale entry from an earlier epoch, and the write-first audit of the
//!   two forward bodies guarantees a hit's device copy is current. True
//!   weights (agent-owned `Vec`s: every `matmul_w` weight, LN scales,
//!   biases, the embedding table) live in a permanent cache keyed by
//!   `(ptr, len)` — stable for the agent's lifetime, uploaded once.
//! - **one tiled GEMM kernel** covers all three matmul shapes via explicit
//!   row/column strides (naive 16×16 tiling — candle's MLX simdgroup kernel
//!   is out of v1 scope; the chart records the honest baseline).
//!
//! Binding contract: every kernel declares `[[buffer(N)]]` /
//! `[[threadgroup(N)]]` attributes explicitly — device buffers at 0.., then
//! the 4-byte `constant` scalars, then the GEMM's two threadgroup tiles —
//! and [`Metal::encode`] binds in exactly that order. Reduction order: LN /
//! softmax sum sequentially per row — GPU kernels cannot replicate the CPU
//! lane's `candle_vec_sum` NEON order; that reduction-order delta is
//! exactly the drift budget the G5 gate holds (≤ 1e-3, the same room
//! candle Metal passes within).

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use metal::{Buffer, CommandQueue, ComputePipelineState, Device, MTLResourceOptions, MTLSize};
use objc2::rc::autoreleasepool;

use super::super::{LayaError, Result};
use super::backend::Backend;

/// Shared storage with EXPLICIT tracked hazard tracking: our ops commit
/// PER-OP command buffers, and Metal only inserts the cross-command-buffer
/// memory barriers for TRACKED resources. On macOS the DEFAULT is
/// untracked (so merely dropping the flag changed nothing — measured), and
/// candle's `HazardTrackingModeUntracked` pairs with their ONE long-lived
/// command buffer, where intra-buffer encoder ordering provides the
/// visibility. With per-op buffers, untracked meant kernel N+1 read stale
/// memory written by kernel N — a timing-dependent race the G5 gate
/// caught. The tracked cost is part of the honest v1 baseline.
const RESOURCE_OPTIONS: MTLResourceOptions = MTLResourceOptions::StorageModeShared
    .union(MTLResourceOptions::HazardTrackingModeTracked);

/// GEMM tile edge (threadgroup edge too — one thread per output element).
const TS: u32 = 16;

/// The compiled-once kernel names (one `.msl` source, one library).
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
    "gemm_tiled",
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
];

/// The MSL source. Sizes fit u32 (every pinned extent < 2³¹); `erf_as` is
/// candle's kernel verbatim (their constants, their op order). Every
/// pointer/scalar argument carries its explicit buffer-space index.
const MSL: &str = r#"
#include <metal_stdlib>
using namespace metal;

constant uint TS = 16u;

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

// out[m×n] row-major = A[m×k] @ B[k×n]; element (i, j) of X at
// x[i·rs + j·cs]. One thread per padded (row, col); 16×16 threadgroup
// tiles of A and B.
kernel void gemm_tiled(
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
    threadgroup float* ta [[threadgroup(10)]],
    threadgroup float* tb [[threadgroup(11)]],
    uint2 gid [[thread_position_in_grid]],
    uint2 lid [[thread_position_in_threadgroup]])
{
    const uint row = gid.y;
    const uint col = gid.x;
    float acc = 0.0f;
    for (uint t = 0u; t < k; t += TS) {
        ta[lid.y * TS + lid.x] = (row < m && t + lid.x < k)
            ? a[row * a_rs + (t + lid.x) * a_cs] : 0.0f;
        tb[lid.y * TS + lid.x] = (t + lid.y < k && col < n)
            ? b[(t + lid.y) * b_rs + col * b_cs] : 0.0f;
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint l = 0u; l < TS; ++l) {
            acc += ta[lid.y * TS + l] * tb[l * TS + lid.x];
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }
    if (row < m && col < n) {
        out[row * n + col] = acc;
    }
}

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

// One thread per row: mean → centered² → 1/sqrt(var+eps) → (v−mean)·inv·w,
// the CPU op order; inv_d arrives as the CPU lane's f64-rounded constant.
kernel void ln_rows(device const float* x [[buffer(0)]],
                    device const float* w [[buffer(1)]],
                    device float* out [[buffer(2)]],
                    constant uint& rows [[buffer(3)]],
                    constant uint& d [[buffer(4)]],
                    constant float& inv_d [[buffer(5)]],
                    constant float& eps [[buffer(6)]],
                    uint gid [[thread_position_in_grid]]) {
    if (gid >= rows) { return; }
    device const float* row = x + gid * d;
    device float* orow = out + gid * d;
    float mean = 0.0f;
    for (uint i = 0u; i < d; ++i) { mean += row[i]; }
    mean *= inv_d;
    float var = 0.0f;
    for (uint i = 0u; i < d; ++i) {
        const float c = row[i] - mean;
        var += c * c;
    }
    var *= inv_d;
    const float inv = 1.0f / sqrt(var + eps);
    for (uint i = 0u; i < d; ++i) {
        orow[i] = (row[i] - mean) * inv * w[i];
    }
}

kernel void softmax_rows(device float* x [[buffer(0)]],
                         constant uint& rows [[buffer(1)]],
                         constant uint& n [[buffer(2)]],
                         uint gid [[thread_position_in_grid]]) {
    if (gid >= rows) { return; }
    device float* row = x + gid * n;
    float max = row[0];
    for (uint i = 1u; i < n; ++i) { max = fmax(max, row[i]); }
    float sum = 0.0f;
    for (uint i = 0u; i < n; ++i) {
        row[i] = precise::exp(row[i] - max);
        sum += row[i];
    }
    for (uint i = 0u; i < n; ++i) { row[i] /= sum; }
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
"#;

fn rt(detail: impl std::fmt::Display) -> LayaError {
    LayaError::Runtime(format!("riir metal backend: {detail}"))
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
    /// host-authored inputs. The generation (bumped at every sync) makes a
    /// recycled heap address miss instead of serving a stale epoch's
    /// bytes; within an epoch a hit's device copy is current because the
    /// forward bodies write every activation device-side before reading
    /// it (the write-first audit in the module doc).
    chain: Mutex<HashMap<(usize, usize, u64), Buffer>>,
    /// The sync generation (how many host-read barriers have run).
    epoch: AtomicU64,
    /// Debug kill-switch (`LAYA_METAL_PER_OP_SYNC`): `1` = sync + write
    /// every result back to its host slice after each op (the slow flow);
    /// `2` = sync only (no writeback — bisects barrier vs host-freshness).
    per_op_sync: u8,
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
        let lib = device
            .new_library_with_source(MSL, &metal::CompileOptions::new())
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
            epoch: AtomicU64::new(0),
            per_op_sync: std::env::var("LAYA_METAL_PER_OP_SYNC")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
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

    /// Host-read barrier: a fresh committed no-op dispatch on the SERIAL
    /// queue completes only after every prior encode. The epoch is NOT
    /// bumped here — syncs may happen many times within one forward (three
    /// in the head), and slots created early in the forward (the hidden
    /// state) stay live until its last download. Pass invalidation happens
    /// in [`Self::begin_pass`], once per forward.
    fn sync(&self) {
        let xb = self.upload(&[0.0f32]);
        self.run("scale", &[&xb], &[0], &[0.0], 0, true)
            .unwrap_or_else(|e| panic!("{e}"));
    }

    /// One forward is beginning: bump the pass epoch and drop the previous
    /// pass's slots. Host-authored buffers (rope tables, mask, `act_in`,
    /// the id list) are rebuilt per forward — often at recycled heap
    /// addresses with fresh contents — so last pass's keys must never hit;
    /// within a pass the serial queue keeps every slot device-current.
    fn begin_pass_impl(&self) {
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

    /// Encode one kernel dispatch and COMMIT (no wait — the lazy-flush
    /// shape). `buffers` bind at `[[buffer(0..)]]`, then `uargs`/`fargs` as
    /// 4-byte `constant` scalars; with `wait` the command buffer is also
    /// waited on (the sync path). Runs inside an autoreleasepool — the
    /// autoreleased command buffer drains per op instead of accumulating
    /// on a thread with no Cocoa runloop.
    fn run(
        &self,
        kernel: &'static str,
        buffers: &[&Buffer],
        uargs: &[u32],
        fargs: &[f32],
        len: u64,
        wait: bool,
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
            wait,
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
        wait: bool,
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
            wait,
        )
    }

    /// The 2D GEMM dispatch (padded 16×16 grid + two threadgroup tiles at
    /// buffer indices 10 and 11, per the MSL attributes). Operands bind at
    /// explicit byte offsets into their whole-buffer slots.
    fn run_gemm_at(
        &self,
        a: (&Buffer, u64),
        b: (&Buffer, u64),
        out: (&Buffer, u64),
        uargs: &[u32],
        m: u32,
        n: u32,
    ) -> Result<()> {
        let p = self
            .pipelines
            .get("gemm_tiled")
            .ok_or_else(|| rt("kernel gemm_tiled missing"))?;
        let tile_bytes = (std::mem::size_of::<f32>() * TS as usize * TS as usize) as u64;
        self.encode(
            p,
            &[a, b, out],
            uargs,
            &[],
            MTLSize {
                width: u64::from(n.next_multiple_of(TS)),
                height: u64::from(m.next_multiple_of(TS)),
                depth: 1,
            },
            MTLSize {
                width: u64::from(TS),
                height: u64::from(TS),
                depth: 1,
            },
            Some((10, tile_bytes)),
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn encode(
        &self,
        p: &ComputePipelineState,
        buffers: &[(&Buffer, u64)],
        uargs: &[u32],
        fargs: &[f32],
        grid: MTLSize,
        tpg: MTLSize,
        tiles: Option<(u64, u64)>,
        wait: bool,
    ) -> Result<()> {
        autoreleasepool(|_| {
            let cb = self.queue.new_command_buffer();
            let enc = cb.new_compute_command_encoder();
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
            if let Some((base, bytes)) = tiles {
                enc.set_threadgroup_memory_length(base, bytes);
                enc.set_threadgroup_memory_length(base + 1, bytes);
            }
            enc.dispatch_threads(grid, tpg);
            enc.end_encoding();
            cb.commit();
            if wait {
                cb.wait_until_completed();
            }
        });
        Ok(())
    }
}


/// Classification of every backend arg (the lazy-sync correctness
/// contract, argued in the module doc):
/// - `weight_buf` — agent-owned, stable for the agent's lifetime
///   (matmul_w weights, LN scales, biases, the embedding table);
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
        self.run_gemm_at(
            (&ab, (a_off * 4) as u64),
            (&bb, (b_off * 4) as u64),
            (&ob, (dst_off * 4) as u64),
            &[
                m as u32,
                n as u32,
                k as u32,
                k as u32, // a_rs
                1,        // a_cs
                n as u32, // b_rs
                1,        // b_cs
            ],
            m as u32,
            n as u32,
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
        self.run_gemm_at(
            (&ab, 0),
            (&wb, 0),
            (&ob, 0),
            &[
                m as u32,
                n as u32,
                k as u32,
                k as u32, // a_rs
                1,        // a_cs
                1,        // b_rs — B = Wᵀ, W row-major [n, k]
                k as u32, // b_cs
            ],
            m as u32,
            n as u32,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, dst);
    }

    #[allow(clippy::too_many_arguments)]
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
        self.run_gemm_at(
            (&qb, (q_off * 4) as u64),
            (&kb, (k_off * 4) as u64),
            (&ob, (dst_off * 4) as u64),
            &[
                m as u32,
                m as u32,
                hd as u32,
                hd as u32, // a_rs
                1,         // a_cs
                1,         // b_rs — B = Kᵀ, K row-major [m, hd]
                hd as u32, // b_cs
            ],
            m as u32,
            m as u32,
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
            false,
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
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn scale(&self, x: &mut [f32], s: f32) {
        let xb = self.chain_buf(x);
        self.run(
            "scale",
            &[&xb],
            &[x.len() as u32],
            &[s],
            x.len() as u64,
            false,
        )
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
        self.run(
            "ln_rows",
            &[&xb, &wb, &ob],
            &[rows as u32, d as u32],
            &[inv_d, eps],
            rows as u64,
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&ob, out);
    }

    fn softmax_rows(&self, x: &mut [f32], n: usize) {
        assert_eq!(x.len() % n, 0, "softmax row extent");
        let rows = x.len() / n;
        let xb = self.chain_buf(x);
        self.run(
            "softmax_rows",
            &[&xb],
            &[rows as u32, n as u32],
            &[],
            rows as u64,
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn relu(&self, x: &mut [f32]) {
        let xb = self.chain_buf(x);
        self.run(
            "relu",
            &[&xb],
            &[x.len() as u32],
            &[],
            x.len() as u64,
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        self.debug_writeback(&xb, x);
    }

    fn gelu_erf(&self, x: &mut [f32]) {
        let xb = self.chain_buf(x);
        self.run(
            "gelu_erf",
            &[&xb],
            &[x.len() as u32],
            &[],
            x.len() as u64,
            false,
        )
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
            false,
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
            false,
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
            false,
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
            false,
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
            false,
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
            false,
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
