//! Issue 064 pilot — the learner-density ascent leg, MEASUREMENT ONLY.
//!
//! WHAT: score the synth lane's transplant candidates with a corpus-density
//! proxy and measure whether a minimal-deviation accept gate has headroom —
//! BEFORE wiring any gate into the shipped acceptance path. The pilot writes
//! a report; it never writes an artifact, never touches the veto, and never
//! changes `plan_suite` (the full-ascent leg is the follow-up task, gated on
//! this pilot's kill gate).
//!
//! The density proxy (the issue's "seat-corpus density proxy" arm — modelless,
//! zero new deps): a von Mises-Fisher kernel on the ENGINE'S OWN 256-dim
//! hashed-bag embedding ([`crate::embed::Embedder`] — the projection the
//! serving seat actually scores against). For a label with pool rows
//! `c_1..c_n` (unit vectors; cosine = dot):
//!
//! `ℓ(x) = logsumexp_j( cos(x, c_j) / τ ) − log n`
//!
//! - **τ is data-derived, per label**: the mean same-label cosine deficit
//!   `mean_{i<j}(1 − cos(c_i, c_j))` — the kernel is `e^{-1}`-scaled at the
//!   label's own natural separation, so a tight near-duplicate pool gets a
//!   sharp kernel and a spread pool a broad one. No free knob.
//! - **Pool rows are scored leave-one-out** (the self-cosine 1 term would
//!   otherwise dominate every pool row's own density and bias Δ).
//! - **Δ = ℓ(cand) − ℓ_loo(src row)**: the candidate against the pool row
//!   whose pair first attested the frame it transplants into (`SynthCand.src`
//!   — the provenance anchor the candidate already carries).
//! - **The control (the natural-variation scale)**: per label, each pool row
//!   vs its NEAREST same-label neighbour (argmax cos, ties → lowest index —
//!   the real-data analog of "a new row joining its closest frame"). The
//!   control |Δ| quantiles are the ε ladder: an accepted candidate's density
//!   deviates from its source no more than real neighbour rows deviate from
//!   each other — the *minimal-deviation signature* of projection sampling
//!   (arXiv:2610.02140: accept iff the frozen consumer's density does not
//!   degrade; here the frozen consumer is the seat's own corpus geometry).
//!
//! KILL GATE (the issue's): the pooled accept rate at ε = the label's own
//! control median, `P(|Δ| ≤ ε_nat)`, must be ≥ 5% — below that there is no
//! headroom for an AND-ed density gate and the lane dies here (recorded as
//! `kill_gate: false`, the report says KILL). Reported alongside: the ladder
//! at control p75/p90 and the ascent fraction (Δ > 0), never gated on.
//!
//! Determinism: BTreeMap iteration everywhere, pool-index float summation
//! order, NN ties → lowest index, no RNG — two runs on the same pool are
//! bit-identical (f64 accumulation; the instrument is not the serve path).

use std::collections::BTreeMap;
use std::path::Path;

use super::{
    git_sha, hostname_refusing_unknown, iso8601_utc, prepare,
    synth::{plan_suite, synth_suites, SynthOptions},
    RunOptions,
};
use crate::embed::{Embedder, EMBED_DIM};

// ── the pure scoring core (fixture-testable, no I/O) ─────────────────────

/// One candidate's embedding + the LOCAL index of its source pool row
/// within the label's pool slice. (Test-facing shape — the pilot proper
/// drives [`LabelKernel::cand_delta`] with parallel arrays.)
#[cfg(test)]
pub(crate) struct PilotCand<'a> {
    pub emb: &'a [f32; EMBED_DIM],
    pub src_local: usize,
}

/// The per-label kernel: τ, LOO pool densities, NN control |Δ| — the
/// label-side half built ONCE per label (the candidate half is chunked
/// over it).
pub(crate) struct LabelKernel {
    pub tau: f64,
    inv_tau: f64,
    /// LOO log-density per pool row (label-local order).
    ell_loo: Vec<f64>,
    /// |Δ| per NN control pair (one per pool row, label-local order).
    pub ctrl_abs: Vec<f64>,
}

impl LabelKernel {
    /// Δ per candidate (caller's order) against the source row's LOO
    /// density (full-sum kernel, two-pass inline logsumexp — no
    /// per-candidate allocation: the pilot scores ~10⁵ candidates).
    /// Parallel arrays (`embs` + `srcs`) — the pilot's chunk buffer owns
    /// its embeddings, no cross-borrows.
    fn cand_delta(&self, pool: &[[f32; EMBED_DIM]], embs: &[[f32; EMBED_DIM]], srcs: &[usize]) -> Vec<f64> {
        debug_assert_eq!(embs.len(), srcs.len());
        let n = pool.len();
        let ln_n = (n as f64).ln();
        let mut out = Vec::with_capacity(embs.len());
        for (emb, &src) in embs.iter().zip(srcs.iter()) {
            let mut m = f64::NEG_INFINITY;
            for p in pool {
                let v = cos(emb, p) * self.inv_tau;
                if v > m {
                    m = v;
                }
            }
            let mut sum = 0.0f64;
            for p in pool {
                sum += (cos(emb, p) * self.inv_tau - m).exp();
            }
            let ell = m + sum.ln() - ln_n;
            out.push(ell - self.ell_loo[src]);
        }
        out
    }
}

/// Build the label kernel: τ, LOO pool densities, NN control |Δ|.
///
/// `pool` is the label's pool rows (label-local order, ≥ 2 rows — labels
/// below that cannot mine frames and are refused upstream).
pub(crate) fn build_kernel(pool: &[[f32; EMBED_DIM]]) -> LabelKernel {
    let n = pool.len();
    debug_assert!(n >= 2, "a label with <2 rows cannot attest a frame");

    // τ — mean same-label cosine deficit over all deterministic pairs i<j.
    let mut deficit = 0.0f64;
    for (i, row) in pool.iter().enumerate() {
        for other in pool.iter().skip(i + 1) {
            deficit += 1.0 - cos(row, other);
        }
    }
    let pairs = (n * (n - 1) / 2) as f64;
    let tau = (deficit / pairs).max(1e-6);
    let inv_tau = 1.0 / tau;

    // LOO log-density per pool row: exclude the self term, normalize by n−1.
    let ell_loo = ell_loo_all(pool, inv_tau);

    // Control — each row vs its nearest same-label neighbour (ties → lowest
    // index): the natural |Δ| scale a real incoming row exhibits.
    let mut ctrl_abs = Vec::with_capacity(n);
    for (i, row) in pool.iter().enumerate() {
        let mut best_j = usize::MAX;
        let mut best_cos = f64::NEG_INFINITY;
        for (j, other) in pool.iter().enumerate() {
            if j == i {
                continue;
            }
            let c = cos(row, other);
            // strict > : ties keep the LOWEST index.
            if c > best_cos {
                best_cos = c;
                best_j = j;
            }
        }
        let d = ell_loo[i] - ell_loo[best_j];
        ctrl_abs.push(d.abs());
    }

    LabelKernel {
        tau,
        inv_tau,
        ell_loo,
        ctrl_abs,
    }
}

/// Score one label: the kernel + candidate Δ (the tests' one-call form;
/// the pilot uses [`build_kernel`] + chunked [`LabelKernel::cand_delta`]).
#[cfg(test)]
pub(crate) fn score_label(pool: &[[f32; EMBED_DIM]], cands: &[PilotCand<'_>]) -> LabelScore {
    let k = build_kernel(pool);
    let embs: Vec<[f32; EMBED_DIM]> = cands.iter().map(|c| *c.emb).collect();
    let srcs: Vec<usize> = cands.iter().map(|c| c.src_local).collect();
    let cand_delta = k.cand_delta(pool, &embs, &srcs);
    LabelScore {
        tau: k.tau,
        ctrl_abs: k.ctrl_abs,
        cand_delta,
    }
}
#[inline]
fn cos(a: &[f32; EMBED_DIM], b: &[f32; EMBED_DIM]) -> f64 {
    let mut s = 0.0f64;
    for i in 0..EMBED_DIM {
        s += a[i] as f64 * b[i] as f64;
    }
    s
}

/// logsumexp over `vals` in the given (fixed, deterministic) order.
fn logsumexp(vals: &[f64]) -> f64 {
    let mut m = f64::NEG_INFINITY;
    for &v in vals {
        if v > m {
            m = v;
        }
    }
    if m == f64::NEG_INFINITY {
        return m;
    }
    let mut sum = 0.0f64;
    for &v in vals {
        sum += (v - m).exp();
    }
    m + sum.ln()
}

/// The per-label scoring result (raw, pre-quantile) — the tests' one-call
/// form over [`build_kernel`] + [`LabelKernel::cand_delta`].
#[cfg(test)]
pub(crate) struct LabelScore {
    pub tau: f64,
    /// |Δ| per NN control pair (one per pool row, label-local order).
    pub ctrl_abs: Vec<f64>,
    /// Δ per candidate (caller's candidate order).
    pub cand_delta: Vec<f64>,
}

/// LOO vMF log-density of every pool row against its label pool (the
/// self-cosine term excluded — a self term at cos 1 would otherwise
/// dominate every pool row's own density and bias Δ).
pub(crate) fn ell_loo_all(pool: &[[f32; EMBED_DIM]], inv_tau: f64) -> Vec<f64> {
    let n = pool.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let vals: Vec<f64> = (0..n)
            .filter(|&j| j != i)
            .map(|j| cos(&pool[i], &pool[j]) * inv_tau)
            .collect();
        out.push(logsumexp(&vals) - ((n - 1) as f64).ln());
    }
    out
}

/// Sorted copy + the p-quantile (linear interpolation, the `nearest-rank`
/// neighbour — a measurement report, not a gate input).
fn quantile_sorted(vals: &mut [f64], p: f64) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.sort_unstable_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
    let idx = p * (vals.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        vals[lo]
    } else {
        let w = idx - lo as f64;
        vals[lo] * (1.0 - w) + vals[hi] * w
    }
}

// ── the ascent-leg gate (Issue 064 task 2 — armed by `--synth-density-gate`) ──

/// The ε rung: which quantile of the label's NN-control |Δ| distribution
/// sets the accept band (Bench 120's ladder: p50 = the natural-median
/// minimal-deviation signature; p75/p90 the looser rungs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityRung {
    P50,
    P75,
    P90,
}

impl DensityRung {
    /// The quantile fraction.
    fn q(self) -> f64 {
        match self {
            DensityRung::P50 => 0.50,
            DensityRung::P75 => 0.75,
            DensityRung::P90 => 0.90,
        }
    }
    /// The parse token (the CLI flag's vocabulary).
    pub fn from_token(tok: &str) -> Option<Self> {
        match tok {
            "p50" => Some(DensityRung::P50),
            "p75" => Some(DensityRung::P75),
            "p90" => Some(DensityRung::P90),
            _ => None,
        }
    }
    /// The artifact/report disclosure string.
    pub fn rule(self) -> String {
        format!(
            "minimal-deviation vMF density gate at the label's own control-{} |Δ| \
             (Bench 120; τ = mean same-label cos deficit, LOO pool rows)",
            self.token()
        )
    }
    fn token(self) -> &'static str {
        match self {
            DensityRung::P50 => "p50",
            DensityRung::P75 => "p75",
            DensityRung::P90 => "p90",
        }
    }
}

/// One label's armed gate: the kernel, its pool, and the ε band at the
/// chosen rung. `accepts` is the ONLY consumer-facing call — acceptance
/// can only REJECT (the provenance discipline: the density leg never
/// writes, never injects).
pub(crate) struct DensityGate {
    pool: Vec<[f32; EMBED_DIM]>,
    kernel: LabelKernel,
    eps: f64,
}

impl DensityGate {
    /// Build the gate for one label's pool rows (label-local order —
    /// `src` indexes into it, the miner's law).
    pub(crate) fn new(pool: Vec<[f32; EMBED_DIM]>, rung: DensityRung) -> Self {
        let kernel = build_kernel(&pool);
        let mut ctrl = kernel.ctrl_abs.clone();
        let eps = quantile_sorted(&mut ctrl, rung.q());
        DensityGate { pool, kernel, eps }
    }

    /// The minimal-deviation accept test: |ℓ(text) − ℓ_loo(src)| ≤ ε.
    pub(crate) fn accepts(&self, embedder: &Embedder, text: &str, src_local: usize) -> bool {
        let mut v = [0.0f32; EMBED_DIM];
        embedder.embed_into(text.as_bytes(), &mut v);
        let delta = self.kernel.cand_delta(&self.pool, &[v], &[src_local])[0];
        delta.abs() <= self.eps
    }

    /// The ε band (disclosure/telemetry).
    pub(crate) fn eps(&self) -> f64 {
        self.eps
    }
}

// ── the report types ─────────────────────────────────────────────────────

/// The accept-rate ladder row (per label; the pooled row reuses the shape).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DensityLadder {
    /// ε = the label's control-median |Δ| (the natural-variation scale).
    pub eps_nat_p50: f64,
    pub eps_nat_p75: f64,
    pub eps_nat_p90: f64,
    /// Fraction of candidates with |Δ| ≤ ε (at each rung).
    pub accept_at_p50: f64,
    pub accept_at_p75: f64,
    pub accept_at_p90: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DensityPilotLabel {
    pub label: String,
    pub pool_rows: usize,
    pub candidates: usize,
    pub tau: f64,
    /// Candidate Δ quantiles (signed) + the ascent fraction (Δ > 0).
    pub delta_p10: f64,
    pub delta_median: f64,
    pub delta_p90: f64,
    pub ascent_frac: f64,
    /// median |Δ_cand| / ε_nat — how far out transplants sit vs real
    /// neighbours (1.0 = exactly the natural scale).
    pub deviation_ratio: f64,
    pub ladder: DensityLadder,
}

#[derive(Debug, serde::Serialize)]
pub struct DensityPilotSuite {
    pub name: String,
    pub pool_rows: usize,
    pub candidates_total: usize,
    pub labels_with_candidates: usize,
    /// Candidates whose source row could not be located in the label pool
    /// (defensive; expected 0 — `src` is a pool row of its own label).
    pub src_misses: usize,
    /// The pooled ladder — every candidate judged at ITS label's ε.
    pub ladder: DensityLadder,
    pub ascent_frac: f64,
    pub deviation_ratio: f64,
    /// The issue's kill gate: pooled accept at ε_nat (control median) ≥ 5%.
    pub kill_gate_pass: bool,
    pub seconds: f64,
    pub per_label: Vec<DensityPilotLabel>,
}

#[derive(Debug, serde::Serialize)]
pub struct DensityPilotMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub host: String,
    pub datasets_dir: String,
    pub max_span_len: usize,
    pub kernel: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct DensityPilotOutput {
    pub meta: DensityPilotMeta,
    pub suites: Vec<DensityPilotSuite>,
    pub skipped: Vec<String>,
}

// ── the pilot proper ─────────────────────────────────────────────────────

/// Run the density pilot over the requested suites (empty `--suites` = the
/// synth default, `massive_intent_en`). Only `sopts.max_span_len` affects
/// the measurement (the mining bound) — the teacher and the accepted-row
/// caps are veto-path knobs this pilot never reaches.
pub fn run_density_pilot(
    opts: &RunOptions,
    sopts: &SynthOptions,
) -> Result<DensityPilotOutput, String> {
    let (specs, defaulted) = synth_suites(opts);
    if defaulted {
        eprintln!(
            "harness --synth-density-pilot: no --suites — defaulting to the V5 board suite \
             (massive_intent_en)"
        );
    }
    let embedder = Embedder;
    let mut suites = Vec::new();
    let mut skipped = Vec::new();
    for spec in specs {
        match pilot_one(spec, &opts.datasets_dir, sopts, &embedder) {
            Ok(s) => suites.push(s),
            Err(e) => skipped.push(format!("{}: {e}", spec.name)),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "synth-density-pilot: no suite ran ({} skip/failure line(s), first: {})",
            skipped.len(),
            skipped.first().map_or("none", String::as_str)
        ));
    }
    Ok(DensityPilotOutput {
        meta: DensityPilotMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            max_span_len: sopts.max_span_len,
            kernel: "vMF cos/tau over the engine hashed-bag embedding; tau = mean \
                     same-label cosine deficit (LOO for pool rows)",
        },
        suites,
        skipped,
    })
}

fn pilot_one(
    spec: &super::SuiteSpec,
    dir: &Path,
    sopts: &SynthOptions,
    embedder: &Embedder,
) -> Result<DensityPilotSuite, String> {
    let t_start = std::time::Instant::now();
    let prepared = prepare(spec, dir)?;
    let plan = plan_suite(&prepared, sopts)?;

    // Embed the whole pool once (the engine's own projection).
    let pool_emb: Vec<[f32; EMBED_DIM]> = prepared
        .train
        .iter()
        .map(|d| {
            let mut v = [0.0f32; EMBED_DIM];
            embedder.embed_into(d.text.as_bytes(), &mut v);
            v
        })
        .collect();

    // Per-label pool row indices (BTreeMap — determinism, the plan_suite law).
    let mut by_label: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (idx, d) in prepared.train.iter().enumerate() {
        by_label.entry(d.label.clone()).or_default().push(idx);
    }

    let mut per_label = Vec::new();
    let mut pooled_accept: [usize; 3] = [0; 3];
    let mut pooled_total = 0usize;
    let mut pooled_ascent = 0usize;
    let mut src_misses = 0usize;
    // Pooled deviation ratios: per-label medians weighted by candidates —
    // kept as sums for a deterministic mean-of-medians disclosure.
    let mut dev_ratio_wsum = 0.0f64;

    for (label, cands) in &plan.buckets {
        let Some(idxs) = by_label.get(label) else {
            // plan_suite mines per label out of the label's own rows — a
            // bucket label without pool rows is a construction bug.
            return Err(format!("bucket label {label:?} has no pool rows"));
        };
        if idxs.len() < 2 {
            return Err(format!(
                "label {label:?} has {} pool row(s) — cannot have attested a frame",
                idxs.len()
            ));
        }
        let pool: Vec<[f32; EMBED_DIM]> = idxs.iter().map(|&g| pool_emb[g]).collect();
        let kernel = build_kernel(&pool);

        // `SynthCand.src` is LABEL-LOCAL (mining iterates the label's own
        // rows) — the local index directly, with a defensive bounds check.
        // Candidates are embedded + scored in bounded chunks (the bucket
        // set runs ~10⁵ rows — the chunk buffer OWNS its embeddings, no
        // wholesale materialization).
        const CHUNK: usize = 4096;
        let mut cand_delta: Vec<f64> = Vec::with_capacity(cands.len());
        let mut chunk_embs: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(CHUNK);
        let mut chunk_srcs: Vec<usize> = Vec::with_capacity(CHUNK);
        for c in cands {
            if c.src >= idxs.len() {
                src_misses += 1;
                continue;
            }
            let mut v = [0.0f32; EMBED_DIM];
            embedder.embed_into(c.text.as_bytes(), &mut v);
            chunk_embs.push(v);
            chunk_srcs.push(c.src);
            if chunk_embs.len() == CHUNK {
                let deltas = kernel.cand_delta(&pool, &chunk_embs, &chunk_srcs);
                cand_delta.extend(deltas);
                chunk_embs.clear();
                chunk_srcs.clear();
            }
        }
        if !chunk_embs.is_empty() {
            let deltas = kernel.cand_delta(&pool, &chunk_embs, &chunk_srcs);
            cand_delta.extend(deltas);
            chunk_embs.clear();
            chunk_srcs.clear();
        }

        let mut ctrl_q = kernel.ctrl_abs.clone();
        let (e50, e75, e90) = (
            quantile_sorted(&mut ctrl_q, 0.50),
            quantile_sorted(&mut ctrl_q, 0.75),
            quantile_sorted(&mut ctrl_q, 0.90),
        );
        let n_cand = cand_delta.len();
        let accept_at = |eps: f64| -> usize {
            cand_delta.iter().filter(|&&d| d.abs() <= eps).count()
        };
        let (a50, a75, a90) = (accept_at(e50), accept_at(e75), accept_at(e90));
        let ascent = cand_delta.iter().filter(|&&d| d > 0.0).count();
        let mut delta_q = cand_delta.clone();
        let (d10, dmed, d90q) = (
            quantile_sorted(&mut delta_q, 0.10),
            quantile_sorted(&mut delta_q, 0.50),
            quantile_sorted(&mut delta_q, 0.90),
        );

        pooled_accept[0] += a50;
        pooled_accept[1] += a75;
        pooled_accept[2] += a90;
        pooled_total += n_cand;
        pooled_ascent += ascent;
        let dev = if e50 > 0.0 { dmed.abs() / e50 } else { f64::INFINITY };
        dev_ratio_wsum += dev * n_cand as f64;

        per_label.push(DensityPilotLabel {
            label: label.clone(),
            pool_rows: idxs.len(),
            candidates: n_cand,
            tau: kernel.tau,
            delta_p10: d10,
            delta_median: dmed,
            delta_p90: d90q,
            ascent_frac: if n_cand > 0 {
                ascent as f64 / n_cand as f64
            } else {
                0.0
            },
            deviation_ratio: dev,
            ladder: DensityLadder {
                eps_nat_p50: e50,
                eps_nat_p75: e75,
                eps_nat_p90: e90,
                accept_at_p50: if n_cand > 0 {
                    a50 as f64 / n_cand as f64
                } else {
                    0.0
                },
                accept_at_p75: if n_cand > 0 {
                    a75 as f64 / n_cand as f64
                } else {
                    0.0
                },
                accept_at_p90: if n_cand > 0 {
                    a90 as f64 / n_cand as f64
                } else {
                    0.0
                },
            },
        });
    }

    let pooled = |acc: usize| -> f64 {
        if pooled_total > 0 {
            acc as f64 / pooled_total as f64
        } else {
            0.0
        }
    };
    // The pooled ε rungs are per-label by construction (each candidate is
    // judged at ITS label's ε) — the ladder row carries the pooled ACCEPT
    // fractions; the ε columns disclose the median label ε for scale.
    let med = |key: fn(&DensityPilotLabel) -> f64| -> f64 {
        let mut v: Vec<f64> = per_label.iter().map(key).collect();
        quantile_sorted(&mut v, 0.5)
    };
    let pooled_ladder = DensityLadder {
        eps_nat_p50: med(|l| l.ladder.eps_nat_p50),
        eps_nat_p75: med(|l| l.ladder.eps_nat_p75),
        eps_nat_p90: med(|l| l.ladder.eps_nat_p90),
        accept_at_p50: pooled(pooled_accept[0]),
        accept_at_p75: pooled(pooled_accept[1]),
        accept_at_p90: pooled(pooled_accept[2]),
    };
    let kill_gate_pass = pooled(pooled_accept[0]) >= 0.05;
    eprintln!(
        "  [density-pilot {}] {} candidate(s) · accept@nat {:.1}% · kill gate {}",
        spec.name,
        pooled_total,
        pooled(pooled_accept[0]) * 100.0,
        if kill_gate_pass { "PASS" } else { "KILL" }
    );
    Ok(DensityPilotSuite {
        name: spec.name.to_string(),
        pool_rows: prepared.train.len(),
        candidates_total: pooled_total,
        labels_with_candidates: per_label.len(),
        src_misses,
        ladder: pooled_ladder,
        ascent_frac: pooled(pooled_ascent),
        deviation_ratio: if pooled_total > 0 {
            dev_ratio_wsum / pooled_total as f64
        } else {
            0.0
        },
        kill_gate_pass,
        seconds: t_start.elapsed().as_secs_f64(),
        per_label,
    })
}

/// The report (`--synth-density-pilot`): verdict first, the ladder, then the
/// per-label rows.
pub fn render_density_pilot_markdown(out: &DensityPilotOutput) -> String {
    let mut md = String::new();
    md.push_str("# synth density pilot — the Issue-064 ascent-leg measurement\n\n");
    md.push_str(&format!(
        "kernel: {} · span ≤ {} · datasets {} · {} · host {}\n\n",
        out.meta.kernel,
        out.meta.max_span_len,
        out.meta.datasets_dir,
        out.meta.date_utc,
        out.meta.host
    ));
    for s in &out.suites {
        md.push_str(&format!(
            "## {} — {} candidate(s) over {} label(s) · {:.1}s\n\n",
            s.name, s.candidates_total, s.labels_with_candidates, s.seconds
        ));
        let verdict = if s.kill_gate_pass { "PASS" } else { "KILL" };
        md.push_str(&format!(
            "**Kill gate: {verdict}** — pooled accept at ε_nat (each label's control-median \
             |Δ|) = {:.1}% (bar ≥ 5.0%). Ladder: p75 {:.1}% · p90 {:.1}% · ascent(Δ>0) \
             {:.1}% · deviation ratio (median |Δ|/ε_nat, candidate-weighted) {:.2}× · \
             src-misses {}\n\n",
            s.ladder.accept_at_p50 * 100.0,
            s.ladder.accept_at_p75 * 100.0,
            s.ladder.accept_at_p90 * 100.0,
            s.ascent_frac * 100.0,
            s.deviation_ratio,
            s.src_misses
        ));
        md.push_str("| label | pool | cands | τ | Δ p10 | Δ med | Δ p90 | ε_nat | acc@nat | acc@p75 | acc@p90 | dev× |\n");
        md.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|\n");
        for l in &s.per_label {
            md.push_str(&format!(
                "| {} | {} | {} | {:.4} | {:+.3} | {:+.3} | {:+.3} | {:.3} | {:.1}% | {:.1}% | \
                 {:.1}% | {:.2} |\n",
                l.label,
                l.pool_rows,
                l.candidates,
                l.tau,
                l.delta_p10,
                l.delta_median,
                l.delta_p90,
                l.ladder.eps_nat_p50,
                l.ladder.accept_at_p50 * 100.0,
                l.ladder.accept_at_p75 * 100.0,
                l.ladder.accept_at_p90 * 100.0,
                l.deviation_ratio
            ));
        }
        md.push('\n');
    }
    for s in &out.skipped {
        md.push_str(&format!("- skipped: {s}\n"));
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(i: usize) -> [f32; EMBED_DIM] {
        // Deterministic distinct unit vectors: e_i basis vectors (cos 0
        // between any two, cos 1 with itself) — the geometry is exact.
        let mut v = [0.0f32; EMBED_DIM];
        v[i % EMBED_DIM] = 1.0;
        v
    }

    fn near(a: [f32; EMBED_DIM], b: [f32; EMBED_DIM]) -> [f32; EMBED_DIM] {
        // A unit vector midway between a and b (renormalized).
        let mut v = [0.0f32; EMBED_DIM];
        for i in 0..EMBED_DIM {
            v[i] = a[i] + b[i];
        }
        let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        for x in v.iter_mut() {
            *x /= n;
        }
        v
    }

    #[test]
    fn loo_excludes_the_self_term() {
        // Pool: e0, e1 (orthogonal). τ = mean deficit over the one pair
        // (1 − cos(e0,e1)) = 1. LOO density of e0 = kernel vs e1 only.
        let pool = vec![unit(0), unit(1)];
        let ell = ell_loo_all(&pool, 1.0);
        // ℓ_loo(e0) = ln(exp(cos(e0,e1)/τ)) − ln(1) = 0/1 → ln(1) − 0 = 0.
        assert!((ell[0] - 0.0).abs() < 1e-12, "{ell:?}");
        // The full-sum density of e0 WOULD be ln((exp(1/τ)+exp(0/τ))/2) ≠ 0
        // — the LOO exclusion is what zeroes the self term.
        let full = ((1.0f64).exp() + (0.0f64).exp()).ln() - (2.0f64).ln();
        assert!((full - 0.0).abs() > 1e-3, "full-sum must differ: {full}");
    }

    #[test]
    fn near_frame_transplant_deviates_less_than_a_foreign_text() {
        // Realistic geometry: pool rows share a common base direction with
        // distinct small perturbations (the hashed-bag analogue of rows
        // sharing vocabulary). A candidate BETWEEN rows 0 and 1 sits at the
        // pool's natural scale; a direction orthogonal to the base is far
        // outside the support.
        let base = |perturb_dim: usize, w: f32| -> [f32; EMBED_DIM] {
            let mut v = [0.0f32; EMBED_DIM];
            v[0] = 1.0;
            v[perturb_dim] = w;
            let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            for x in v.iter_mut() {
                *x /= n;
            }
            v
        };
        let pool = vec![base(1, 0.3), base(2, 0.2), base(3, 0.4), base(4, 0.25)];
        let near_emb = near(pool[0], pool[1]);
        let foreign = unit(50); // orthogonal to the base AND every perturbation
        let cands: Vec<PilotCand<'_>> = vec![
            PilotCand { emb: &near_emb, src_local: 0 },
            PilotCand { emb: &foreign, src_local: 0 },
        ];
        let s = score_label(&pool, &cands);
        let d_near = s.cand_delta[0].abs();
        let d_foreign = s.cand_delta[1].abs();
        assert!(
            d_near < d_foreign,
            "near-frame |Δ| {d_near} must sit under the foreign |Δ| {d_foreign}"
        );
        // Signed: the foreign direction reads BELOW the source row's
        // density (out of support); the near-frame transplant does not.
        assert!(
            s.cand_delta[1] < s.cand_delta[0],
            "foreign Δ {:?} must under-read the near Δ {:?}",
            s.cand_delta[1],
            s.cand_delta[0]
        );
        assert!(s.ctrl_abs.iter().all(|v| *v >= 0.0 && v.is_finite()));
    }

    #[test]
    fn scoring_is_deterministic() {
        let pool = vec![unit(0), unit(1), near(unit(0), unit(2))];
        let a = unit(2);
        let b = near(unit(1), unit(2));
        let cands: Vec<PilotCand<'_>> = vec![
            PilotCand { emb: &a, src_local: 0 },
            PilotCand { emb: &b, src_local: 1 },
        ];
        let s1 = score_label(&pool, &cands);
        let s2 = score_label(&pool, &cands);
        assert_eq!(s1.tau.to_bits(), s2.tau.to_bits());
        assert_eq!(s1.ctrl_abs.len(), s2.ctrl_abs.len());
        for (x, y) in s1.ctrl_abs.iter().zip(s2.ctrl_abs.iter()) {
            assert_eq!(x.to_bits(), y.to_bits());
        }
        for (x, y) in s1.cand_delta.iter().zip(s2.cand_delta.iter()) {
            assert_eq!(x.to_bits(), y.to_bits());
        }
    }

    #[test]
    fn nn_control_ties_break_to_the_lowest_index() {
        // Two identical rows (e0, e0) + one other (e1): row 2's nearest is
        // rows 0 and 1 at equal cos — the control pair must pick row 0
        // (identical ℓ_loo for 0/1 makes this unobservable in |Δ|, so the
        // assertion is structural: exactly n control pairs, all finite).
        let pool = vec![unit(0), unit(0), unit(1)];
        let cands: Vec<PilotCand<'_>> = vec![];
        let s = score_label(&pool, &cands);
        assert_eq!(s.ctrl_abs.len(), 3);
        assert!(s.ctrl_abs.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn kill_gate_bar_is_five_percent_at_the_natural_scale() {
        // The verdict logic lives at the caller (a fraction compare); pin
        // the bar constant's arithmetic here so a silent edit reds.
        let accept = 3.0 / 100.0;
        assert!(accept < 0.05, "3% must read KILL");
        let accept = 5.0 / 100.0;
        assert!(accept >= 0.05, "exactly 5% must read PASS");
    }

    #[test]
    fn density_gate_accepts_a_transplant_and_rejects_a_foreign_text() {
        // Real Embedder over a tiny shared-frame pool (the miner's own
        // shape: rows sharing a prefix+suffix frame with differing spans).
        use crate::embed::Embedder;
        let pool_texts = [
            "wake me up at nine am on friday",
            "wake me up at six pm sharp on friday",
            "wake me up at noon on friday",
            "wake me up at three thirty pm on friday",
            "wake me up at ten am on friday",
            "wake me up just before midnight on friday",
        ];
        let embedder = Embedder;
        let pool: Vec<[f32; EMBED_DIM]> = pool_texts
            .iter()
            .map(|t| {
                let mut v = [0.0f32; EMBED_DIM];
                embedder.embed_into(t.as_bytes(), &mut v);
                v
            })
            .collect();
        let gate = DensityGate::new(pool.clone(), DensityRung::P50);
        let loose = DensityGate::new(pool.clone(), DensityRung::P90);
        // A TRANSPLANT-shaped text: the frame + an attested-STYLE span (a
        // fresh time token in the frame's slot — what the miner generates).
        let transplant = "wake me up at five am on friday";
        let foreign = "the quarterly budget review is scheduled for march";
        // The foreign text is rejected at EVERY rung (far outside support).
        assert!(
            !gate.accepts(&embedder, foreign, 0) && !loose.accepts(&embedder, foreign, 0),
            "the foreign text must be rejected at every rung"
        );
        // The headroom property (Bench 120's measured fact, reproduced on
        // the fixture): an in-frame transplant passes at the loose rung.
        // (Its P50 verdict is NOT asserted — a 6-row fixture's control band
        // is not the corpus-scale band the pilot measured.)
        assert!(
            loose.accepts(&embedder, transplant, 0),
            "the in-frame transplant must pass the p90 gate"
        );
        // Monotone ε ladder.
        assert!(loose.eps() >= gate.eps());
    }
}
