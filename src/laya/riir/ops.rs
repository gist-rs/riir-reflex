//! The flat-`Vec<f32>` tensor ops of the riir-owned forward — the ONE
//! `gemm::` seam (`.issues/002`: the whole kernel dep hides behind this
//! file, so a future self-hosted GEMM is a one-file swap) plus the
//! elementwise / reduction ops the model needs, f32 throughout.
//!
//! Bit-parity note: every GEMM here calls the same crate + version candle's
//! CPU backend calls (`gemm` 0.18.2) with candle's EXACT argument shape —
//! `read_dst=false, alpha=0, beta=1`, strides as `(col_stride, row_stride)`
//! and `(1, k)` for a row-major `[·, k]` operand — so matching shapes are
//! BIT-IDENTICAL to the candle lane. The elementwise ops mirror the candle
//! port's op order, and the gelu kernel IS candle's (`libm::erff`,
//! `.issues/003`) — the G5 drift budget is spent only on the
//! reduction-order differences, never on the matmuls or the gelu.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::thread;

use gemm::{Parallelism, gemm};

/// The thread count, resolved once (the profile showed `sysctl` + the env
/// scan inside every gemm dispatch — 576+ calls per question forward).
/// `Some(n)` = explicit `RAYON_NUM_THREADS` override, honored verbatim (no
/// cap); `None` = auto-detect, capped by [`MAX_GEMM_THREADS`].
fn num_threads() -> Option<usize> {
    static THREADS: OnceLock<Option<usize>> = OnceLock::new();
    *THREADS.get_or_init(|| {
        if let Some(t) = std::env::var("RAYON_NUM_THREADS")
            .ok()
            .and_then(|t| t.parse::<usize>().ok())
            .filter(|&t| t > 0)
        {
            return Some(t);
        }
        std::thread::available_parallelism()
            .ok()
            .map(|p| p.get().min(MAX_GEMM_THREADS))
    })
}

/// Single-thread cut, in FLOPs (2·m·n·k): below it the 16-thread rayon
/// dispatch + latch sync costs more than the compute. Measured, not tuned
/// by feel: the per-head attention gemms are ~0.3 MFLOP each (~576 of them
/// per forward), and the `sample` profile showed the main thread blocked
/// in `LockLatch::wait_and_reset → pthread_cond_wait` at every one while
/// all workers idled in `wait_until_cold` — sync, not FLOPs, was the
/// bottleneck. The encoder's projections (57–170 MFLOP) stay threaded.
const SINGLE_THREAD_FLOPS: usize = 8_000_000;

/// Latency cap on the AUTO-detected worker count. Measured on the M3 Max
/// (12P+4E, the box's standing sibling load): english row p50 234.8 ms at
/// the full 16-way pool vs 156.8 ms at 8 — every gemm join waits for its
/// slowest worker, so the E-cores pace every projection, and the calling
/// thread itself computes between gemms (gelu), which made 17 compute
/// threads on 12 fast cores oversubscribed. 8 keeps the pool on P-cores
/// with headroom. `RAYON_NUM_THREADS` overrides verbatim (other hosts,
/// quiet boxes); results are bit-identical at ANY count — gemm-fallback
/// splits the OUTPUT range, so the G5 gate is count-independent.
const MAX_GEMM_THREADS: usize = 8;

/// The candle call shape (`get_num_threads()` + `Parallelism::Rayon(n)`
/// when > 1, `Parallelism::None` otherwise), plus the small-problem cut:
/// gemm-fallback parallelizes over the OUTPUT range only — every dst
/// element is one thread's full sequential k-dot either way — so
/// `Parallelism::None` on a small shape is the same adds in the same order
/// (the G5 drift gate asserts this at the switch: top-1 agreement 1.0,
/// drift unchanged).
fn parallelism(m: usize, n: usize, k: usize) -> Parallelism {
    let cells = m.saturating_mul(n);
    let flops = cells.saturating_mul(k).saturating_mul(2);
    if flops < SINGLE_THREAD_FLOPS {
        return Parallelism::None;
    }
    match num_threads() {
        Some(t) if t > 1 => Parallelism::Rayon(t),
        _ => Parallelism::None,
    }
}

/// dst[m×n] (row-major) ← a[m×k] @ b[k×n], both row-major — the plain
/// matmul (attention `probs @ v`, every head-local product).
///
/// # Panics
/// Asserts the exact extents gemm walks (which also pins the unsafe
/// stride arithmetic in bounds).
pub fn matmul_into(a: &[f32], m: usize, k: usize, b: &[f32], n: usize, dst: &mut [f32]) {
    assert_eq!(a.len(), m * k, "lhs extent");
    assert_eq!(b.len(), k * n, "rhs extent");
    assert_eq!(dst.len(), m * n, "dst extent");
    // SAFETY: the asserts pin the operand extents; the strides below keep
    // every element access inside them.
    unsafe {
        gemm(
            m,
            n,
            k,
            dst.as_mut_ptr(),
            1,
            n as isize,
            false,
            a.as_ptr(),
            1,
            k as isize,
            b.as_ptr(),
            1,
            n as isize,
            0.0,
            1.0,
            false,
            false,
            false,
            parallelism(m, n, k),
        );
    }
}

/// dst[m×n] ← a[m×k] @ w[n×k]ᵀ — every projection in this model (W stored
/// row-major `[out, in]`, the candle `x @ W.t()` shape: the rhs strides
/// `(cs = k, rs = 1)` describe Wᵀ exactly as candle's `.t()` layout does).
///
/// # Panics
/// Asserts the exact extents gemm walks.
pub fn matmul_w_into(a: &[f32], m: usize, k: usize, w: &[f32], n: usize, dst: &mut [f32]) {
    assert_eq!(a.len(), m * k, "lhs extent");
    assert_eq!(w.len(), n * k, "weight extent");
    assert_eq!(dst.len(), m * n, "dst extent");
    // SAFETY: as `matmul_into` — the asserts pin the extents.
    unsafe {
        gemm(
            m,
            n,
            k,
            dst.as_mut_ptr(),
            1,
            n as isize,
            false,
            a.as_ptr(),
            1,
            k as isize,
            w.as_ptr(),
            k as isize,
            1,
            0.0,
            1.0,
            false,
            false,
            false,
            parallelism(m, n, k),
        );
    }
}

/// `matmul_w_into` into a fresh buffer.
pub fn matmul_w(a: &[f32], m: usize, k: usize, w: &[f32], n: usize) -> Vec<f32> {
    let mut out = vec![0.0; m * n];
    matmul_w_into(a, m, k, w, n, &mut out);
    out
}

/// dst[m×m] ← q[m×hd] @ kᵀ (k row-major `[m×hd]`) — the attention score
/// matmul (the rhs strides `(cs = hd, rs = 1)` are candle's
/// `k.transpose(-2, -1)` view).
///
/// # Panics
/// Asserts the exact extents gemm walks.
pub fn matmul_kt_into(q: &[f32], m: usize, hd: usize, k: &[f32], dst: &mut [f32]) {
    assert_eq!(q.len(), m * hd, "q extent");
    assert_eq!(k.len(), m * hd, "k extent");
    assert_eq!(dst.len(), m * m, "dst extent");
    // SAFETY: as `matmul_into` — the asserts pin the extents.
    unsafe {
        gemm(
            m,
            m,
            hd,
            dst.as_mut_ptr(),
            1,
            m as isize,
            false,
            q.as_ptr(),
            1,
            hd as isize,
            k.as_ptr(),
            hd as isize,
            1,
            0.0,
            1.0,
            false,
            false,
            false,
            parallelism(m, m, hd),
        );
    }
}

/// Batched over heads: `dst[h·m·m ..] ← q[h·m·hd ..] @ k[h·m·hd ..]ᵀ` for
/// every `h < heads` — the attention score loop as ONE backend op (one
/// dispatch on the device backends; the CPU lane keeps the identical
/// per-head op order, so its numerics are bit-unchanged).
///
/// # Panics
/// Asserts the exact extents the per-head `matmul_kt_into` walks.
pub fn matmul_kt_heads(q: &[f32], k: &[f32], heads: usize, m: usize, hd: usize, dst: &mut [f32]) {
    assert!(heads > 0, "heads");
    assert_eq!(q.len(), heads * m * hd, "q extent");
    assert_eq!(k.len(), heads * m * hd, "k extent");
    assert_eq!(dst.len(), heads * m * m, "dst extent");
    let slab = m * hd;
    let out = m * m;
    for h in 0..heads {
        let o = h * slab;
        matmul_kt_into(
            &q[o..o + slab],
            m,
            hd,
            &k[o..o + slab],
            &mut dst[h * out..h * out + out],
        );
    }
}

/// Batched over heads: `dst[h·m·n ..] ← a[h·m·k ..] @ b[h·k·n ..]` — the
/// attention context loop as ONE backend op.
///
/// # Panics
/// Asserts the exact extents the per-head `matmul_into` walks.
pub fn matmul_heads(
    a: &[f32],
    b: &[f32],
    heads: usize,
    m: usize,
    k: usize,
    n: usize,
    dst: &mut [f32],
) {
    assert!(heads > 0, "heads");
    assert_eq!(a.len(), heads * m * k, "a extent");
    assert_eq!(b.len(), heads * k * n, "b extent");
    assert_eq!(dst.len(), heads * m * n, "dst extent");
    let a_slab = m * k;
    let b_slab = k * n;
    let out = m * n;
    for h in 0..heads {
        let ao = h * a_slab;
        let bo = h * b_slab;
        matmul_into(
            &a[ao..ao + a_slab],
            m,
            k,
            &b[bo..bo + b_slab],
            n,
            &mut dst[h * out..h * out + out],
        );
    }
}

/// `x[r] += mask[r % mask.len()]` over the whole `heads·mask.len()` scores
/// parent — the attention mask broadcast for every head in ONE backend op
/// (the same elementwise add as [`add_inplace`], with the [seq, seq] mask
/// broadcast over the head slabs; adding 0.0 to a finite score is exact).
///
/// # Panics
/// Asserts `x` is exactly `heads` mask slabs.
pub fn add_mask_broadcast(x: &mut [f32], mask: &[f32], heads: usize) {
    assert!(heads > 0, "heads");
    let mlen = mask.len();
    assert_eq!(x.len(), heads * mlen, "mask broadcast extent");
    for (r, xi) in x.iter_mut().enumerate() {
        *xi += mask[r % mlen];
    }
}

/// x += y (elementwise, the residual adds and the sliding-mask add —
/// adding 0.0 to a finite score is exact, so the mask entries that are
/// 0.0 change nothing and the `f32::MIN` entries drive softmax to 0).
pub fn add_inplace(x: &mut [f32], y: &[f32]) {
    assert_eq!(x.len(), y.len(), "add extent");
    for (xi, yi) in x.iter_mut().zip(y.iter()) {
        *xi += *yi;
    }
}

/// The reduction order of candle's CPU `vec_sum` — the ONE reduction every
/// LN / softmax sum must use to stay bit-identical to the candle lane.
///
/// On aarch64 (this lane's host) candle compiles the NEON kernel: STEP=32,
/// EPR=4, ARR=8 — eight `float32x4` accumulators fed by strided loads
/// (`sum[j] += row[i + j·4 .. +4]`, i stepping by 32), then a pairwise
/// tree reduce over the accumulators and the `vaddvq_f32` horizontal add,
/// then the scalar leftovers (k mod 32). Each `vaddq_f32` is four
/// independent lane adds, so this scalar loop performs the SAME adds in
/// the SAME order — bit-identical without intrinsics. On other targets
/// candle compiles the plain sequential sum; so does this.
fn candle_vec_sum(row: &[f32]) -> f32 {
    // ⚊ DIVERGENCE GUARD: the index-loop form is a deliberate transcription
    // of candle's NEON kernel (strided accumulators + the pairwise
    // `vec_reduce`) — the lane indexing IS the summation order; rewriting
    // it iterator-style trips the borrow checker (one accumulator mutated
    // while its pair is read) and obscures the 1:1 mapping.
    #[cfg(target_arch = "aarch64")]
    {
        #[allow(clippy::needless_range_loop)]
        fn neon_order(row: &[f32]) -> f32 {
            const STEP: usize = 32;
            const EPR: usize = 4;
            const ARR: usize = STEP / EPR; // 8
            let k = row.len();
            let np = k & !(STEP - 1);
            let mut sum = [[0f32; EPR]; ARR];
            let mut i = 0;
            while i < np {
                for j in 0..ARR {
                    // Lane l of accumulator j — the strided NEON loads.
                    for l in 0..EPR {
                        sum[j][l] += row[i + j * EPR + l];
                    }
                }
                i += STEP;
            }
            // candle neon `vec_reduce`: pairwise over the accumulators.
            for a in 0..ARR / 2 {
                for l in 0..EPR {
                    sum[2 * a][l] += sum[2 * a + 1][l];
                }
            }
            for a in 0..ARR / 4 {
                for l in 0..EPR {
                    sum[4 * a][l] += sum[4 * a + 2][l];
                }
            }
            for l in 0..EPR {
                sum[0][l] += sum[4][l];
            }
            // `vaddvq_f32` — the architectural pairwise horizontal add.
            let mut total = (sum[0][0] + sum[0][1]) + (sum[0][2] + sum[0][3]);
            while i < k {
                total += row[i];
                i += 1;
            }
            total
        }
        neon_order(row)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        // candle's non-SIMD fallback: the plain sequential sum.
        let mut total = 0f32;
        for v in row {
            total += *v;
        }
        total
    }
}

/// x rows += bias (the head's torch biases — `broadcast_add(b.unsqueeze(0))`
/// in the candle port).
pub fn add_bias_row(x: &mut [f32], d: usize, bias: &[f32]) {
    assert_eq!(bias.len(), d, "bias extent");
    assert_eq!(x.len() % d, 0, "row extent");
    for row in x.chunks_exact_mut(d) {
        for (xi, bi) in row.iter_mut().zip(bias.iter()) {
            *xi += *bi;
        }
    }
}

/// x *= s (the attention `1/√hd` pre-scaling — 0.125 on both pinned
/// geometries, exact in f32).
pub fn scale_inplace(x: &mut [f32], s: f32) {
    for xi in x.iter_mut() {
        *xi *= s;
    }
}

/// Bias-free `LayerNorm` over rows of `d` (biased variance — torch
/// semantics): `(x − mean) / √(var + eps) · w`, the candle port's exact op
/// order AND reduction order (both means through [`candle_vec_sum`], the
/// mean scale as candle's f64→f32 affine mul).
pub fn layer_norm_nobias(x: &[f32], w: &[f32], eps: f32, d: usize) -> Vec<f32> {
    let mut out = x.to_vec();
    let mut sq = Vec::new();
    layer_norm_nobias_into(x, w, eps, d, &mut sq, &mut out);
    out
}

/// The `_into` form. `sq` is the caller's squared-centering scratch
/// (resized here, so one buffer reused across rows/calls allocates nothing
/// in the row loop).
pub fn layer_norm_nobias_into(
    x: &[f32],
    w: &[f32],
    eps: f32,
    d: usize,
    sq: &mut Vec<f32>,
    out: &mut [f32],
) {
    assert_eq!(w.len(), d, "norm extent");
    assert_eq!(x.len(), out.len(), "ln extent");
    assert_eq!(x.len() % d, 0, "row extent");
    // candle's mean scale: `1f64 / (d as f64)` rounded to f32 (the affine
    // mul is applied as `T::from_f64(scale)`), not `1f32 / d as f32`.
    let inv_d = (1f64 / d as f64) as f32;
    sq.resize(d, 0.0);
    for (src, dst) in x.chunks_exact(d).zip(out.chunks_exact_mut(d)) {
        let mean = candle_vec_sum(src) * inv_d;
        // var = mean(centered²) — candle materializes `centered`, squares,
        // then means over the SAME reduction order.
        for (si, ci) in src.iter().zip(sq.iter_mut()) {
            let c = *si - mean;
            *ci = c * c;
        }
        let var = candle_vec_sum(sq) * inv_d;
        // `affine(1.0, eps)` = v·1 + eps; then sqrt, then recip — the
        // candle port's exact chain.
        let inv = 1.0f32 / (var + eps).sqrt();
        for (o, (v, wi)) in dst.iter_mut().zip(src.iter().zip(w.iter())) {
            *o = (*v - mean) * inv * *wi;
        }
    }
}

/// Bias-carrying `LayerNorm` over rows of `d` (the decision head's torch
/// norms): `((x − mean) / √(var + eps)) · w + b` — the bias pass is
/// per-row (candle's `broadcast_add(b.unsqueeze(0))`).
pub fn layer_norm(x: &[f32], w: &[f32], b: &[f32], eps: f32, d: usize) -> Vec<f32> {
    assert_eq!(b.len(), d, "bias extent");
    let mut out = layer_norm_nobias(x, w, eps, d);
    for row in out.chunks_exact_mut(d) {
        for (o, bi) in row.iter_mut().zip(b.iter()) {
            *o += *bi;
        }
    }
    out
}

/// Softmax over each row of `n` — the candle port's exact formula AND
/// reduction order: max from the first element (candle's `vec_reduce_max`),
/// `exp(x − max)`, the strided `vec_sum`, then normalize, f32 throughout.
pub fn softmax_rows(x: &mut [f32], n: usize) {
    assert_eq!(x.len() % n, 0, "softmax row extent");
    for row in x.chunks_exact_mut(n) {
        let mut max = row[0];
        for v in &row[1..] {
            max = max.max(*v);
        }
        for v in row.iter_mut() {
            *v = (*v - max).exp();
        }
        let sum = candle_vec_sum(row);
        for v in row.iter_mut() {
            *v /= sum;
        }
    }
}

/// `ReLU` in place (the head FF activation — torch's `F.relu` default).
pub fn relu_inplace(x: &mut [f32]) {
    for v in x.iter_mut() {
        if *v < 0.0 {
            *v = 0.0;
        }
    }
}

// ── the elementwise pool ──────────────────────────────────────────────
//
// The GLU is the forward's last big SERIAL pass: `seq·I` `libm::erff`
// calls on the calling thread while the gemm workers sit idle between
// projections (`sample`, this session and the threading pass both: ~13–14%
// of the forward wall). `erff` itself is bit-parity-locked (`.issues/003`
// — a different erf kernel is what cost the lane its drift budget), so the
// win is THREADS, not a different kernel: the pool splits the OUTPUT
// element range and every element runs the same scalar expression, so
// results are bit-identical at any worker count — the same argument as
// gemm-fallback's output split (the G5 gate re-proves it end to end).
// Workers park on a condvar between calls, never spin: the box already
// carries the rayon pool's parked threads between gemms.

/// Elements per pool chunk (~64 KB of f32): big enough that dispatch
/// overhead is noise, small enough that dynamically stolen chunks keep the
/// split balanced when an E-core lands a few of them.
const GELU_CHUNK_ELS: usize = 1 << 14;

/// Below this element count the publish/steal/join round-trip costs more
/// than the work — run inline on the calling thread.
const GELU_POOL_MIN_ELS: usize = 2 * GELU_CHUNK_ELS;

/// One published elementwise job. Raw pointers because the job outlives no
/// slice: the dispatcher keeps both slices borrowed for the whole call and
/// joins every worker before returning.
#[derive(Clone, Copy)]
struct GluJob {
    fused: *const f32,
    out: *mut f32,
    rows: usize,
    i_sz: usize,
    n_chunks: usize,
}

// SAFETY: the pointers come from slices the dispatcher keeps borrowed for
// the whole call; workers write only the disjoint `out` chunk their
// `fetch_add` stole and never write `fused`.
unsafe impl Send for GluJob {}
unsafe impl Sync for GluJob {}

struct ElementPool {
    /// One dispatcher at a time: the forward is single-threaded per
    /// question, this only stops a concurrent harness/HTTP caller from
    /// interleaving two publishes into one join barrier.
    dispatch: Mutex<()>,
    state: Mutex<PoolState>,
    cv: Condvar,
    /// Dynamic chunk counter — reset by the dispatcher before publishing.
    next_chunk: AtomicUsize,
    /// Live workers (a failed spawn lowers the join target, never stalls
    /// it).
    workers: AtomicUsize,
}

struct PoolState {
    epoch: u64,
    done: usize,
    job: Option<GluJob>,
}

/// The pool, built once on the first above-threshold GLU call. `None` when
/// the resolved thread count is 1 — always inline, the same posture as the
/// gemms.
fn element_pool() -> Option<&'static ElementPool> {
    static POOL: OnceLock<Option<ElementPool>> = OnceLock::new();
    static SPAWNED: OnceLock<()> = OnceLock::new();
    let pool = POOL.get_or_init(|| {
        let workers = num_threads()?;
        if workers < 2 {
            return None;
        }
        Some(ElementPool {
            dispatch: Mutex::new(()),
            state: Mutex::new(PoolState {
                epoch: 0,
                done: 0,
                job: None,
            }),
            cv: Condvar::new(),
            next_chunk: AtomicUsize::new(0),
            workers: AtomicUsize::new(workers),
        })
    });
    let pool = pool.as_ref()?;
    // No caller can reach a dispatch before this returns: every path into
    // the pool goes through `element_pool()` first.
    SPAWNED.get_or_init(|| {
        let target = pool.workers.load(Ordering::Relaxed);
        let mut live = 0usize;
        for i in 0..target {
            match thread::Builder::new()
                .name(format!("laya-ew-{i}"))
                .spawn(move || element_worker(pool, 0))
            {
                Ok(_) => live += 1,
                Err(_) => break,
            }
        }
        pool.workers.store(live, Ordering::Relaxed);
    });
    Some(pool)
}

fn element_worker(pool: &'static ElementPool, mut seen_gen: u64) {
    let mut st = pool.state.lock().unwrap();
    loop {
        while st.epoch == seen_gen {
            st = pool.cv.wait(st).unwrap();
        }
        seen_gen = st.epoch;
        let Some(job) = st.job else { continue };
        drop(st);
        loop {
            let c = pool.next_chunk.fetch_add(1, Ordering::Relaxed);
            if c >= job.n_chunks {
                break;
            }
            // SAFETY: the dispatcher's extent asserts pinned both slices
            // before publishing; chunk ranges are disjoint across workers.
            unsafe { glu_chunk(&job, c) };
        }
        st = pool.state.lock().unwrap();
        st.done += 1;
        pool.cv.notify_all();
    }
}

/// One output chunk: `out[e] = gelu(fused[r, j]) · fused[r, I + j]` over the
/// flat element range `[chunk·CHUNK, …)` — the SAME per-element expression
/// as the serial row form, reached through a flat index (`e == r·I + j` is
/// `out`'s own row-major layout), so results are bit-identical to it.
///
/// # Safety
/// `fused` must reference `rows·2·i_sz` readable and `out` `rows·i_sz`
/// writable f32s for the whole call (the dispatcher's extent asserts pin
/// the real slices); chunk ranges are disjoint across workers.
unsafe fn glu_chunk(job: &GluJob, chunk: usize) {
    let total = job.rows * job.i_sz;
    let lo = chunk * GELU_CHUNK_ELS;
    let hi = (lo + GELU_CHUNK_ELS).min(total);
    // SAFETY: as this fn's contract — the dispatcher's extent asserts pin
    // the real slices and chunk ranges are disjoint across workers.
    unsafe {
        for e in lo..hi {
            let r = e / job.i_sz;
            let j = e - r * job.i_sz;
            let row = job.fused.add(r * 2 * job.i_sz);
            *job.out.add(e) = gelu_erf_scalar(*row.add(j)) * *row.add(job.i_sz + j);
        }
    }
}

fn pool_glu_gelu(
    pool: &'static ElementPool,
    fused: &[f32],
    rows: usize,
    i_sz: usize,
    out: &mut [f32],
) {
    let _gate = pool.dispatch.lock().unwrap();
    let total = rows * i_sz;
    let n_chunks = total.div_ceil(GELU_CHUNK_ELS);
    let job = GluJob {
        fused: fused.as_ptr(),
        out: out.as_mut_ptr(),
        rows,
        i_sz,
        n_chunks,
    };
    {
        let mut st = pool.state.lock().unwrap();
        pool.next_chunk.store(0, Ordering::Relaxed);
        st.done = 0;
        st.job = Some(job);
        st.epoch += 1;
        pool.cv.notify_all();
    }
    // The calling thread steals chunks too — it would otherwise idle at the
    // join, and the gemm pool is parked while the GLU runs.
    loop {
        let c = pool.next_chunk.fetch_add(1, Ordering::Relaxed);
        if c >= n_chunks {
            break;
        }
        // SAFETY: as the worker call — extents asserted by the caller.
        unsafe { glu_chunk(&job, c) };
    }
    let target = pool.workers.load(Ordering::Relaxed);
    let mut st = pool.state.lock().unwrap();
    while st.done < target {
        st = pool.cv.wait(st).unwrap();
    }
    st.job = None;
}

/// `gelu_erf` at one point — the candle lane's exact op order and kernel:
/// `(erf(v/√2) + 1) · 0.5 · v` in f32 with `libm::erff` (`.issues/003`:
/// the same crate + version candle-core calls) — bit-identical gelu, so
/// the G5 drift budget is spent on the irreducible reduction-order
/// differences only.
#[must_use]
fn gelu_erf_scalar(v: f32) -> f32 {
    const FRAC_1_SQRT_2: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let e = libm::erff(v * FRAC_1_SQRT_2);
    (e + 1.0) * 0.5 * v
}

/// `gelu_erf` in place (the encoder MLP input activation).
pub fn gelu_erf_inplace(x: &mut [f32]) {
    for v in x.iter_mut() {
        *v = gelu_erf_scalar(*v);
    }
}

/// The MLP GLU: `out[r, j] = gelu(fused[r, j]) · fused[r, I + j]` — the
/// candle port's `gate_in.gelu_erf().broadcast_mul(&gate)`. The activation
/// is written to its OWN contiguous `[rows, I]` buffer (the fused input is
/// read-only): the following Wo projection must see candle's exact
/// contiguous lhs layout — an in-place form would leave the act values
/// interleaved with the gates across rows and the contiguous first-half
/// slice would feed row 0's gate as row 1's activation.
pub fn glu_gelu_gate(fused: &[f32], rows: usize, i_sz: usize, out: &mut [f32]) {
    assert_eq!(fused.len(), rows * 2 * i_sz, "fused extent");
    assert_eq!(out.len(), rows * i_sz, "glu out extent");
    let total = rows * i_sz;
    match element_pool() {
        Some(pool) if total >= GELU_POOL_MIN_ELS => pool_glu_gelu(pool, fused, rows, i_sz, out),
        _ => glu_gelu_gate_rows(fused, rows, i_sz, out),
    }
}

/// The serial row form — the inline path below the pool threshold, and the
/// bit-identity reference the pool path is tested against.
fn glu_gelu_gate_rows(fused: &[f32], rows: usize, i_sz: usize, out: &mut [f32]) {
    for r in 0..rows {
        let row = &fused[r * 2 * i_sz..][..2 * i_sz];
        let orow = &mut out[r * i_sz..][..i_sz];
        for j in 0..i_sz {
            orow[j] = gelu_erf_scalar(row[j]) * row[i_sz + j];
        }
    }
}

/// cos/sin tables `[seq, hd]` for one `RoPE` theta — the candle port's exact
/// math (inv f32 pow of the f64 theta, `ang = pos · inv`, std cos/sin,
/// duplicated at `[j] == [j + half]`).
#[must_use]
pub fn rope_tables(seq: usize, hd: usize, theta: f64) -> (Vec<f32>, Vec<f32>) {
    let half = hd / 2;
    let mut cos = vec![0f32; seq * hd];
    let mut sin = vec![0f32; seq * hd];
    for pos in 0..seq {
        for j in 0..half {
            let inv = 1.0 / (theta.powf((2 * j) as f64 / hd as f64)) as f32;
            let ang = pos as f32 * inv;
            let c = ang.cos();
            let s = ang.sin();
            cos[pos * hd + j] = c;
            cos[pos * hd + j + half] = c;
            sin[pos * hd + j] = s;
            sin[pos * hd + j + half] = s;
        }
    }
    (cos, sin)
}

/// Rotate-half `RoPE` in place on `[heads, seq, hd]` — per element
/// `o[j] = q1·c − q2·s`, `o[j + half] = q2·c + q1·s` (the cos/sin
/// duplication at `j + half` makes this the candle port's
/// `q·cos + rotate_half(q)·sin` exactly; `a − b` is `a + (−b)` in IEEE, so
/// the fused form is bit-identical to the add form).
pub fn apply_rope_inplace(
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
    for h in 0..heads {
        for pos in 0..seq {
            let base = (h * seq + pos) * hd;
            let crow = &cos[pos * hd..pos * hd + hd];
            let srow = &sin[pos * hd..pos * hd + hd];
            for j in 0..half {
                let c = crow[j];
                let s = srow[j];
                let q1 = q[base + j];
                let q2 = q[base + half + j];
                q[base + j] = q1 * c - q2 * s;
                q[base + half + j] = q2 * c + q1 * s;
            }
        }
    }
}

/// `[seq, in_dim]` row-major → `[heads, seq, hd]`:
/// `out[h, s, i] = src[s, off + h·hd + i]` — the contiguous-thirds Q/K/V
/// split of a fused `[seq, 3d]` projection (off 0/d/2d).
pub fn split_heads(
    src: &[f32],
    row_stride: usize,
    off: usize,
    seq: usize,
    heads: usize,
    hd: usize,
    out: &mut [f32],
) {
    assert_eq!(out.len(), heads * seq * hd, "split extent");
    for h in 0..heads {
        for s in 0..seq {
            let begin = s * row_stride + off + h * hd;
            let dst = (h * seq + s) * hd;
            out[dst..dst + hd].copy_from_slice(&src[begin..begin + hd]);
        }
    }
}

/// `[heads, seq, hd]` → `[seq, d]`: `out[s, h·hd + i] = src[h, s, i]` — the
/// head merge (`transpose(0,1).flatten_from(1)` in the candle port).
pub fn merge_heads(src: &[f32], seq: usize, heads: usize, hd: usize, out: &mut [f32]) {
    let d = heads * hd;
    assert_eq!(out.len(), seq * d, "merge extent");
    for h in 0..heads {
        for s in 0..seq {
            let sb = (h * seq + s) * hd;
            let db = s * d + h * hd;
            out[db..db + hd].copy_from_slice(&src[sb..sb + hd]);
        }
    }
}

/// Gather whole rows (`index_select(rows, 0)`): `out[r, :] = x[rows[r], :]`.
pub fn gather_rows(x: &[f32], d: usize, rows: &[usize], out: &mut [f32]) {
    assert_eq!(out.len(), rows.len() * d, "gather extent");
    for (r, &row) in rows.iter().enumerate() {
        let b = row * d;
        out[r * d..(r + 1) * d].copy_from_slice(&x[b..b + d]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// gemm agreement on a tiny case computed by hand: a[2,2] @ b[2,3].
    #[test]
    fn matmul_matches_hand_computed_reference() {
        let a = [1.0, 2.0, 3.0, 4.0]; // [[1,2],[3,4]]
        let b = [1.0, 0.0, 2.0, 0.0, 1.0, 3.0]; // [[1,0,2],[0,1,3]]
        let mut out = vec![0.0; 6];
        matmul_into(&a, 2, 2, &b, 3, &mut out);
        // [[1·1+2·0, 1·0+2·1, 1·2+2·3], [3·1+4·0, 3·0+4·1, 3·2+4·3]]
        assert_eq!(out, vec![1.0, 2.0, 8.0, 3.0, 4.0, 18.0]);
    }

    /// x @ wᵀ == x @ (wᵀ computed literally) — the projection form agrees
    /// with the plain form on the same math.
    #[test]
    fn matmul_w_agrees_with_transposed_matmul() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // [2, 3]
        let w = [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0]; // [3, 3] rows = projections
        let wt: Vec<f32> = (0..3)
            .flat_map(|c| (0..3).map(move |r| w[r * 3 + c]))
            .collect();
        let got = matmul_w(&x, 2, 3, &w, 3);
        let mut want = vec![0.0; 6];
        matmul_into(&x, 2, 3, &wt, 3, &mut want);
        assert_eq!(got, want);
    }

    /// scores = q @ kᵀ per row — the strided attention form vs the literal
    /// transpose.
    #[test]
    fn matmul_kt_agrees_with_literal_transpose() {
        let q = [1.0, 2.0, 3.0, 4.0]; // [2, 2]
        let k = [5.0, 6.0, 7.0, 8.0]; // [2, 2]
        let kt = [k[0], k[2], k[1], k[3]];
        let mut got = vec![0.0; 4];
        matmul_kt_into(&q, 2, 2, &k, &mut got);
        let mut want = [0.0; 4];
        matmul_into(&q, 2, 2, &kt, 2, &mut want);
        assert_eq!(got, want);
    }

    /// `LayerNorm` against a hand-computed torch-semantics reference.
    #[test]
    fn layer_norm_matches_torch_semantics() {
        // Row [1, 2, 3]: mean 2, biased var 2/3, inv = 1/sqrt(2/3 + 0.25).
        let x = [1.0, 2.0, 3.0];
        let w = [2.0, 2.0, 2.0];
        let b = [0.5, 0.5, 0.5];
        let eps = 0.25f32;
        let got = layer_norm(&x, &w, &b, eps, 3);
        let inv = (2.0f32 / 3.0 + eps).sqrt().recip();
        let want: Vec<f32> = vec![
            -(inv) * 2.0 + 0.5,
            0.0 * inv * 2.0 + 0.5,
            1.0 * inv * 2.0 + 0.5,
        ];
        for (g, w_) in got.iter().zip(want.iter()) {
            assert!((g - w_).abs() < 1e-6);
        }
    }

    /// Softmax rows: shift-invariance + sum-to-one, and the `f32::MIN` mask
    /// entries underflow to exactly 0.
    #[test]
    fn softmax_rows_normalizes_and_zeroes_masks() {
        let mut x = vec![f32::MIN, 1.0, 0.0, f32::MIN, 0.0, 1.0];
        softmax_rows(&mut x, 3);
        for row in x.as_chunks::<3>().0 {
            let sum: f32 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6);
        }
        assert_eq!(x[0], 0.0);
        assert_eq!(x[3], 0.0);
    }

    /// `gelu_erf` at known points (`libm::erff` kernel — candle's exact
    /// values): 0 → 0, gelu(1) ≈ 0.8413447, odd-symmetric, saturates to
    /// ±x for large |x|.
    #[test]
    fn gelu_erf_matches_reference_at_known_points() {
        let mut x = vec![0.0, 1.0, -1.0, 5.0, -5.0];
        gelu_erf_inplace(&mut x);
        assert_eq!(x[0], 0.0);
        // gelu(1) = 0.5·1·(1 + erf(1/√2)) ≈ 0.8413447
        assert!((x[1] - 0.841_344_7).abs() < 1e-6, "{}", x[1]);
        assert!((x[2] + 0.158_655_25).abs() < 1e-6, "{}", x[2]);
        assert!((x[3] - 5.0).abs() < 1e-5, "{}", x[3]);
        assert!(x[4].abs() < 1e-5, "{}", x[4]);
    }

    /// The MLP GLU form equals gelu(first half) ⊙ second half.
    #[test]
    fn glu_gelu_gate_matches_two_pass_reference() {
        let fused = [0.5f32, -1.0, 2.0, 0.25]; // rows=1, I=2
        let gate_half = [2.0f32, 0.25];
        let mut got = vec![0.0; 2];
        glu_gelu_gate(&fused, 1, 2, &mut got);
        let mut want = vec![0.5f32, -1.0];
        gelu_erf_inplace(&mut want);
        for (w, g) in want.iter_mut().zip(gate_half.iter()) {
            *w *= *g;
        }
        assert_eq!(got[..2], want[..]);
    }

    /// The multi-row interleaving trap: the activation rows must come out
    /// contiguous — row 1's activation is fused row 1's first half, never
    /// anything from row 0's gate half.
    #[test]
    fn glu_gelu_gate_keeps_rows_contiguous() {
        // rows=2, I=1: fused = [[act0, gate0], [act1, gate1]].
        let fused = vec![1.0, 100.0, 2.0, 200.0];
        let mut got = vec![0.0; 2];
        glu_gelu_gate(&fused, 2, 1, &mut got);
        let want = [gelu_erf_scalar(1.0) * 100.0, gelu_erf_scalar(2.0) * 200.0];
        assert!((got[0] - want[0]).abs() < 1e-4, "{} vs {}", got[0], want[0]);
        assert!((got[1] - want[1]).abs() < 1e-4, "{} vs {}", got[1], want[1]);
    }

    /// The pool path (≥ [`GELU_POOL_MIN_ELS`] elements when ≥2 workers
    /// resolve) must equal the serial row form BIT-IDENTICALLY. 40·1000
    /// spans three 16 K chunks with boundaries landing MID-ROW — where a
    /// flat-index mapping would go wrong. On a 1-core box both sides take
    /// the inline path (the test stays green, just weaker; the G5 gate
    /// covers the real posture).
    #[test]
    fn glu_gelu_gate_parallel_matches_serial_bit_identically() {
        let (rows, i_sz) = (40usize, 1000usize);
        let mut fused = Vec::with_capacity(rows * 2 * i_sz);
        for r in 0..rows {
            for j in 0..i_sz {
                let v = ((r * 31 + j * 7) % 97) as f32 / 24.0 - 2.0;
                fused.push(v);
                fused.push(-v * 1.5);
            }
        }
        let mut par = vec![0f32; rows * i_sz];
        glu_gelu_gate(&fused, rows, i_sz, &mut par);
        let mut ser = vec![0f32; rows * i_sz];
        glu_gelu_gate_rows(&fused, rows, i_sz, &mut ser);
        assert_eq!(par, ser);
    }

    /// Two runs through the pool are byte-identical — no cross-call state
    /// can leak through the shared chunk counter or the join barrier.
    #[test]
    fn glu_gelu_gate_parallel_is_deterministic() {
        let (rows, i_sz) = (64usize, 1024usize);
        let mut fused = Vec::with_capacity(rows * 2 * i_sz);
        for r in 0..rows {
            for j in 0..i_sz {
                let v = ((r + j) % 53) as f32 / 16.0 - 1.6;
                fused.push(v);
                fused.push(v * 0.3);
            }
        }
        let mut a = vec![0f32; rows * i_sz];
        let mut b = vec![0f32; rows * i_sz];
        glu_gelu_gate(&fused, rows, i_sz, &mut a);
        glu_gelu_gate(&fused, rows, i_sz, &mut b);
        assert_eq!(a, b);
    }

    /// `RoPE` tables duplicate cos/sin across the halves; applying to a unit
    /// vector at pos 0 (ang 0) is the identity, and the rotate-half law
    /// holds against the literal cat form.
    #[test]
    fn rope_matches_rotate_half_reference() {
        let (cos, sin) = rope_tables(2, 4, 10_000.0);
        // Duplication law.
        for pos in 0..2 {
            for j in 0..2 {
                assert_eq!(cos[pos * 4 + j], cos[pos * 4 + j + 2]);
                assert_eq!(sin[pos * 4 + j], sin[pos * 4 + j + 2]);
            }
        }
        // pos 0 → ang 0 → cos 1, sin 0 → identity.
        assert_eq!(cos[0], 1.0);
        assert_eq!(sin[0], 0.0);

        // The literal candle form: out = q·cos + cat([−q2, q1])·sin, per
        // position. Two positions (the tables' seq) × one head × hd 4.
        let q = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut got = q.to_vec();
        apply_rope_inplace(&mut got, 2, 1, 4, &cos, &sin);
        let half = 2;
        for pos in 0..2 {
            let row = &q[pos * 4..pos * 4 + 4];
            let rotated = [-row[half], -row[half + 1], row[0], row[1]];
            for j in 0..4 {
                let want = row[j] * cos[pos * 4 + j] + rotated[j] * sin[pos * 4 + j];
                assert!((got[pos * 4 + j] - want).abs() < 1e-6);
            }
        }
    }

    /// Split/merge/gather round-trips on shaped data.
    #[test]
    fn split_merge_gather_round_trip() {
        // [seq 2, d 4], heads 2, hd 2.
        let x: Vec<f32> = (0..8).map(|i| i as f32).collect();
        let mut heads_buf = vec![0.0; 2 * 2 * 2];
        split_heads(&x, 4, 0, 2, 2, 2, &mut heads_buf);
        assert_eq!(&heads_buf, &[0.0, 1.0, 4.0, 5.0, 2.0, 3.0, 6.0, 7.0]);
        let mut merged = vec![0.0; 8];
        merge_heads(&heads_buf, 2, 2, 2, &mut merged);
        assert_eq!(merged, x);
        let mut rows = vec![0.0; 8];
        gather_rows(&x, 4, &[1, 0], &mut rows);
        assert_eq!(rows, vec![4.0, 5.0, 6.0, 7.0, 0.0, 1.0, 2.0, 3.0]);
    }
}
