#![cfg(feature = "pair_free_probe")]
//! Pair-free transport probe — frozen laya hidden states vs the modelless
//! hashed-bag embedder (Issue 105, riir-train Issue 663 T2; arXiv:2610.39302
//! via katgpt-rs Research 646).
//!
//! Per suite, N train texts are embedded into BOTH spaces and measured
//! through:
//!
//! - **The pair-free chain** — [`katgpt_core::leakage_probe::probe`]
//!   (subspace whitening → Sinkhorn↔Procrustes multistart): per-side suite
//!   class labels, chance-normalized transfer lift, alignment mean cosine,
//!   neighborhood hit, and the honest verdict tier (including
//!   `InsufficientAlignment` — "not measurable here", never folded into a
//!   fail).
//! - **The paired arm (positive control)** — top-P PCA per side (f64
//!   [`katgpt_core::linalg::symmetric_eig`]), orthogonal Procrustes fitted
//!   on the first `--anchors` rows (the pairing the probe chain is NOT
//!   given), then post-Procrustes mean cosine on the held-out rest +
//!   **FOSCTTM** (mean rank fraction of the true counterpart; 0.5 = chance)
//!   under three transports: original spaces (R = I), PCA spaces
//!   (R = I), PCA + fitted R.
//!
//! Reading (pre-registered in the issue): laya is next-token-trained ⇒
//! GL(k)-identified + nuisance-carrying (Research 646 Laws 2–3); the PRH
//! paper's pair-free result on NTP encoders is FOSCTTM .287 ≈ chance. So
//! the expected record is pair-free `InsufficientAlignment`/`Low` with the
//! paired arm strictly better — the numbers decide, not the prior.
//!
//! Run (m3 Metal):
//! ```sh
//! LAYA_DEVICE=metal cargo run --release --features pair_free_probe \
//!   --example pair_free_probe -- --suite banking77 --n 1000
//! ```
#![allow(clippy::needless_range_loop)]

use katgpt_core::leakage_probe::{probe, LeakProbeConfig};
use katgpt_core::linalg::{symmetric_eig, SymmetricEigScratch};
use riir_reflex::embed::{Embedder, EMBED_DIM};
use riir_reflex::laya::config::Checkpoint;
use riir_reflex::laya::riir::RiirAgent;
use riir_reflex::laya::weights::weights_root;
use serde_json::json;

/// Default PCA width for the paired arm (both sides projected to this).
const PCA_K: usize = 64;
/// Default anchor count for the Procrustes fit.
const ANCHORS: usize = 64;

struct Row {
    text: String,
    label: String,
}

/// Load the suite's train shards (the datasets_t20k JSON format) up to
/// `n` rows. xnli has no `text` field — refused loud (not a probe suite).
fn load_rows(suite: &str, n: usize) -> Vec<Row> {
    let dir = std::path::Path::new("../riir-reflex/.raw/datasets_t20k").join(suite);
    let mut rows = Vec::new();
    let mut shards: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("suite dir {suite}: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.starts_with("train-") && f.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect();
    shards.sort();
    for shard in shards {
        let doc: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&shard).expect("read shard"))
                .expect("parse shard");
        for r in doc["rows"].as_array().expect("rows array") {
            let row = &r["row"];
            let text = row["text"].as_str().expect("text field").to_string();
            let label = match &row["label"] {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(v) => format!("c{}", v),
                other => panic!("unexpected label shape: {other}"),
            };
            rows.push(Row { text, label });
            if rows.len() >= n {
                return rows;
            }
        }
    }
    rows
}

/// Label strings → dense u32 ids (deterministic first-seen order).
fn label_ids(rows: &[Row]) -> (Vec<u32>, Vec<String>) {
    let mut names: Vec<String> = Vec::new();
    let mut ids = Vec::with_capacity(rows.len());
    for r in rows {
        let id = match names.iter().position(|n| n == &r.label) {
            Some(i) => i as u32,
            None => {
                names.push(r.label.clone());
                (names.len() - 1) as u32
            }
        };
        ids.push(id);
    }
    (ids, names)
}

/// Mean-pool the encoder's final hidden states into one d-vector.
fn mean_pool(hidden: &[f32], seq_len: usize, d: usize) -> Vec<f32> {
    let mut out = vec![0.0f64; d];
    for row in 0..seq_len {
        for (o, v) in out.iter_mut().zip(&hidden[row * d..row * d + d]) {
            *o += f64::from(*v);
        }
    }
    let inv = 1.0 / seq_len as f64;
    out.iter().map(|v| (*v * inv) as f32).collect()
}

/// Covariance top-P eigenvectors of a row-major `n×d` population (f64
/// internally). Returns the projection matrix, column-major columns packed
/// row-major as `proj[dim * p + col]`, plus the eigenvalues (for the
/// λ^-1/2 whitening).
fn pca_top(
    data: &[f32],
    n: usize,
    dim: usize,
    p: usize,
    scratch: &mut SymmetricEigScratch,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    // mean
    let mut mean = vec![0.0f64; dim];
    for row in 0..n {
        for (m, v) in mean.iter_mut().zip(&data[row * dim..row * dim + dim]) {
            *m += f64::from(*v);
        }
    }
    for m in mean.iter_mut() {
        *m /= n as f64;
    }
    // centered covariance (1/(n−1))
    let mut cov = vec![0.0f64; dim * dim];
    let mut cen = vec![0.0f64; dim];
    for row in 0..n {
        for (col, (c, v)) in cen.iter_mut().zip(&data[row * dim..row * dim + dim]).enumerate() {
            *c = f64::from(*v) - mean[col];
        }
        for i in 0..dim {
            let ci = cen[i];
            for (cv, cj) in cov[i * dim..i * dim + dim].iter_mut().zip(cen.iter()) {
                *cv += ci * cj;
            }
        }
    }
    let nf = (n - 1) as f64;
    for v in cov.iter_mut() {
        *v /= nf;
    }
    let mut vals = vec![0.0f64; dim];
    let mut vecs = vec![0.0f64; dim * dim];
    symmetric_eig(&mut vals, &mut vecs, &cov, scratch, dim, 60);
    // top-p by eigenvalue (descending stable sort of index pairs)
    let mut idx: Vec<usize> = (0..dim).collect();
    idx.sort_by(|a, b| vals[*b].total_cmp(&vals[*a]));
    let mut proj = vec![0.0f64; dim * p];
    let mut pv = vec![0.0f64; p];
    for (j, &src) in idx.iter().take(p).enumerate() {
        for i in 0..dim {
            proj[i * p + j] = vecs[i * dim + src];
        }
        pv[j] = vals[src];
    }
    (proj, pv, mean)
}

/// Project row-major data through `proj` (dim×p), center by `mean`, then
/// whiten per-axis by 1/sqrt(λ) (clamped at 1e-8).
fn project_whiten(
    data: &[f32],
    n: usize,
    dim: usize,
    proj: &[f64],
    pv: &[f64],
    mean: &[f64],
    p: usize,
) -> Vec<f32> {
    let mut out = vec![0.0f32; n * p];
    for row in 0..n {
        let src = &data[row * dim..row * dim + dim];
        for j in 0..p {
            let mut acc = 0.0;
            for (i, v) in src.iter().enumerate() {
                acc += (f64::from(*v) - mean[i]) * proj[i * p + j];
            }
            let lam = pv[j].max(1e-8);
            out[row * p + j] = (acc / lam.sqrt()) as f32;
        }
    }
    out
}

/// Orthogonal Procrustes: R = argmin ‖X − Y·R‖_F over row-major `n×p`
/// f64 buffers (the polar factor via eig of MᵀM, M = XᵀY).
fn procrustes_r(x: &[f64], y: &[f64], n: usize, p: usize, scratch: &mut SymmetricEigScratch) -> Vec<f64> {
    let mut m = vec![0.0f64; p * p];
    for row in 0..n {
        let xr = &x[row * p..row * p + p];
        let yr = &y[row * p..row * p + p];
        for i in 0..p {
            let xi = xr[i];
            for (mv, yj) in m[i * p..i * p + p].iter_mut().zip(yr) {
                *mv += xi * *yj;
            }
        }
    }
    let mut mtm = vec![0.0f64; p * p];
    for i in 0..p {
        for j in i..p {
            let acc: f64 = (0..p).map(|t| m[t * p + i] * m[t * p + j]).sum();
            mtm[i * p + j] = acc;
            mtm[j * p + i] = acc;
        }
    }
    let mut vals = vec![0.0f64; p];
    let mut v = vec![0.0f64; p * p];
    symmetric_eig(&mut vals, &mut v, &mtm, scratch, p, 60);
    let mut mv = vec![0.0f64; p * p];
    for i in 0..p {
        for t in 0..p {
            mv[i * p + t] = (0..p).map(|s| m[i * p + s] * v[s * p + t]).sum();
        }
    }
    let mut r = vec![0.0f64; p * p];
    for i in 0..p {
        for j in 0..p {
            r[i * p + j] = (0..p)
                .map(|t| {
                    let sig = vals[t].sqrt().max(1e-30);
                    mv[i * p + t] * v[j * p + t] / sig
                })
                .sum();
        }
    }
    r
}

/// Apply R (p×p, row-major) to every row of y (n×p f32) → f64 rows.
fn apply_r(y: &[f32], r: &[f64], n: usize, p: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; n * p];
    for row in 0..n {
        let yr = &y[row * p..row * p + p];
        for i in 0..p {
            out[row * p + i] = (0..p).map(|j| f64::from(yr[j]) * r[j * p + i]).sum();
        }
    }
    out
}

/// FOSCTTM over the first `m` rows of x/y (paired): mean over i of the
/// fraction of j≠i whose y_j sits closer (cosine) to x_i than y_i does.
fn foscttm(x: &[f64], y: &[f64], m: usize, p: usize) -> f64 {
    let mut total = 0.0f64;
    let mut norms_y = vec![0.0f64; m];
    for j in 0..m {
        norms_y[j] = y[j * p..j * p + p].iter().map(|v| v * v).sum::<f64>().sqrt();
    }
    for i in 0..m {
        let xr = &x[i * p..i * p + p];
        let nx = xr.iter().map(|v| v * v).sum::<f64>().sqrt();
        let mut worse = 0usize;
        let mut sim_true = None;
        let mut sims = Vec::with_capacity(m);
        for j in 0..m {
            let dot: f64 = xr.iter().zip(&y[j * p..j * p + p]).map(|(a, b)| a * b).sum();
            let denom = nx * norms_y[j];
            let sim = if denom > 1e-12 { dot / denom } else { 0.0 };
            if j == i {
                sim_true = Some(sim);
            }
            sims.push(sim);
        }
        let st = sim_true.expect("paired row");
        for (j, s) in sims.iter().enumerate() {
            if j != i && *s > st {
                worse += 1;
            }
        }
        total += worse as f64 / (m - 1) as f64;
    }
    total / m as f64
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut suite = String::from("banking77");
    let mut n = 1000usize;
    let mut anchors = ANCHORS;
    let mut pca_k = PCA_K;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--suite" if i + 1 < args.len() => {
                i += 1;
                suite = args[i].clone();
            }
            "--n" if i + 1 < args.len() => {
                i += 1;
                n = args[i].parse().expect("--n usize");
            }
            "--anchors" if i + 1 < args.len() => {
                i += 1;
                anchors = args[i].parse().expect("--anchors usize");
            }
            "--pca" if i + 1 < args.len() => {
                i += 1;
                pca_k = args[i].parse().expect("--pca usize");
            }
            other => panic!("unknown arg {other}"),
        }
        i += 1;
    }

    let rows = load_rows(&suite, n);
    let m = rows.len();
    assert!(m >= 32, "need >= 32 rows, got {m}");
    let (labels, label_names) = label_ids(&rows);
    eprintln!(
        "suite {suite}: {m} rows, {} classes, device env LAYA_DEVICE={}",
        label_names.len(),
        std::env::var("LAYA_DEVICE").unwrap_or_else(|_| "cpu".into())
    );

    // ── space B: the modelless hashed-bag embedder (256-d) ────────────────
    let embedder = Embedder;
    let mut mdl = vec![0.0f32; m * EMBED_DIM];
    for (r, row) in rows.iter().enumerate() {
        embedder.embed_into(row.text.as_bytes(), &mut mdl[r * EMBED_DIM..(r + 1) * EMBED_DIM]);
    }

    // ── space A: frozen laya final hidden states, mean-pooled ─────────────
    let agent = RiirAgent::load(&weights_root(), Checkpoint::English).expect("agent load");
    let qdef = json!({
        "type": "noul",
        "instructions": "Classify the sentence.",
        "criteria": null,
    });
    let mut laya: Vec<f32> = Vec::new();
    let mut d_laya = 0usize;
    for (r, row) in rows.iter().enumerate() {
        let enc = agent
            .encode_question(&json!(row.text), &qdef)
            .unwrap_or_else(|e| panic!("laya forward row {r}: {e}"));
        if d_laya == 0 {
            d_laya = enc.d;
            eprintln!("laya hidden width d = {d_laya}");
        }
        assert_eq!(enc.d, d_laya, "hidden width drifted mid-suite");
        let pooled = mean_pool(&enc.hidden, enc.seq_len, enc.d);
        laya.extend_from_slice(&pooled);
        if (r + 1) % 200 == 0 {
            eprintln!("  laya {}/{}", r + 1, m);
        }
    }

    // ── the pair-free chain (leakage_probe transport) ─────────────────────
    let cfg = LeakProbeConfig::default();
    let report = probe(
        &laya, &labels, d_laya, &mdl, &labels, EMBED_DIM, &cfg,
    )
    .unwrap_or_else(|e| panic!("leakage probe: {e}"));

    // ── the paired arm ────────────────────────────────────────────────────
    let eval = m - anchors;
    let mut scratch = SymmetricEigScratch::new();
    let (proj_l, pv_l, mean_l) = pca_top(&laya, m, d_laya, pca_k, &mut scratch);
    let (proj_m, pv_m, mean_m) = pca_top(&mdl, m, EMBED_DIM, pca_k, &mut scratch);
    let laya_p = project_whiten(&laya, m, d_laya, &proj_l, &pv_l, &mean_l, pca_k);
    let mdl_p = project_whiten(&mdl, m, EMBED_DIM, &proj_m, &pv_m, &mean_m, pca_k);
    // anchors: first `anchors` rows (f64 copies)
    let xa: Vec<f64> = laya_p[..anchors * pca_k]
        .iter()
        .map(|v| f64::from(*v))
        .collect();
    let ya: Vec<f64> = mdl_p[..anchors * pca_k]
        .iter()
        .map(|v| f64::from(*v))
        .collect();
    let r = procrustes_r(&xa, &ya, anchors, pca_k, &mut scratch);
    // held-out rest
    let xl = &laya_p[anchors * pca_k..];
    let yl = &mdl_p[anchors * pca_k..];
    let yr = apply_r(yl, &r, eval, pca_k);
    let xl64: Vec<f64> = xl.iter().map(|v| f64::from(*v)).collect();
    let yl64: Vec<f64> = yl.iter().map(|v| f64::from(*v)).collect();
    // residual mean cos: paired, held-out, under R
    let mut resid = 0.0f64;
    for row in 0..eval {
        let (xr, yrr) = (&xl64[row * pca_k..row * pca_k + pca_k], &yr[row * pca_k..row * pca_k + pca_k]);
        let (mut dot, mut nx, mut ny) = (0.0f64, 0.0f64, 0.0f64);
        for t in 0..pca_k {
            dot += xr[t] * yrr[t];
            nx += xr[t] * xr[t];
            ny += yrr[t] * yrr[t];
        }
        let denom = nx.sqrt() * ny.sqrt();
        resid += if denom > 1e-12 { dot / denom } else { 0.0 };
    }
    resid /= eval as f64;
    // FOSCTTM pair (original-space FOSCTTM is undefined across dims
    // (d_laya != 256); the PCA plane is the comparable one)
    let fos_pca_raw = foscttm(&xl64, &yl64, eval, pca_k);
    let fos_pca_paired = foscttm(&xl64, &yr, eval, pca_k);

    // ── verdict block ─────────────────────────────────────────────────────
    println!(
        "{}",
        json!({
            "suite": suite,
            "rows": m,
            "classes": label_names.len(),
            "laya_d": d_laya,
            "modelless_d": EMBED_DIM,
            "pca_k": pca_k,
            "anchors": anchors,
            "pair_free": {
                "verdict": report.verdict.to_string(),
                "attribute_transfer_top1": report.attribute_transfer_top1,
                "chance_baseline": report.chance_baseline,
                "lift": report.lift,
                "alignment_mean_cos": report.alignment_mean_cos,
                "neighborhood_hit_rate": report.neighborhood_hit_rate,
                "neighborhood_hit_chance": report.neighborhood_hit_chance,
            },
            "paired": {
                "post_procrustes_mean_cos_eval": resid,
                "foscttm_pca_raw": fos_pca_raw,
                "foscttm_pca_paired": fos_pca_paired,
                "note": "original-space FOSCTTM undefined across dims (d_laya != 256); the PCA plane is the comparable one",
            },
            "device": agent.device(),
            "checkpoint": agent.checkpoint(),
        })
    );
}
