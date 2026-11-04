//! Issue 106 — the ρ-per-suite embedding read + the ρ-gated cross-lane
//! Procrustes probe (katgpt-rs Issue 978 T5 consumer lane; REPORT-ONLY —
//! nothing arms here, no exit-code coupling to any serving lane).
//!
//! # What one run prints (per suite, per k arm)
//!
//! Suites: banking77 first-1200 rows, emotion + ag_news all rows (400 each).
//! Two spaces measured at the SAME k so the Procrustes is defined:
//! (a) the shipped modelless hashed embedder (`crate::embed::Embedder`,
//! FNV-1a word+bigram sign-hash) at k buckets, (b) the pinned laya english
//! encoder (`laya-riir`) mean-pooled final hidden states, stride-folded
//! 1024→k (a FIXED deterministic projection, disclosed as part of space B's
//! definition). The laya forwards run ONCE (1024 pooled), folded per arm.
//! Two k arms: 256 (the 4:1 fold — the fit half is DEGENERATE at n=400,
//! p+q = 512 > 200: disclosed, held-out scoring remains valid) and 64 (the
//! 16:1 fold — fully non-degenerate, k ≤ n/10 at every suite).
//!
//! 1. **ρ per space grouped by gold label** — katgpt-core
//!    `nuisance_fraction`; singleton labels excluded BY NAME (the
//!    primitive refuses a single-view group loud — never silently
//!    filtered).
//! 2. **The pre-registered gate** — refuse the cross-lane transport iff
//!    `alignment_floor(ρ_a, ρ_b, k) ≥ RMS²(R = I)` (population whitened
//!    points): the Thm-3 lower bound on the OPTIMAL population rotation
//!    already exceeding the do-nothing residual means no rotation can
//!    gain. Regime-free — no calibrated ρ threshold (every suite sits in
//!    the G≪k compression regime where absolute ρ compresses; the rule
//!    never compares ρ across shapes).
//! 3. **The out-of-band adjudication — HELD-OUT, per the round-2 verdict**
//!    (the in-sample optimum is NOT an estimate of the population optimum
//!    the floor bounds: at k/n = 0.64 an in-sample solve reads deep below
//!    identity on PURE NOISE — the round-2 simulation reproduces the
//!    first run's 131/276 from independent Gaussians at 134.8/306.9; the
//!    "floor violation" in the first record was this artifact):
//!
//!    - split the paired points into even/odd halves (deterministic);
//!    - fit R = polar(AᵀB) on the FIT half (in-sample post recorded —
//!      disclosed as the artifact-prone number);
//!    - score post-RMS² on the HELD-OUT half (the adjudicating number);
//!    - a seeded PERMUTATION NULL (B rows shuffled, same split) prints
//!      the artifact magnitude for this exact (n, k) — the null's
//!      in-sample post reproduces the overfit floor, its held-out post
//!      reads ≈ 2k;
//!    - bound check: HELD iff held-out post ≥ floor·(1 − 1e-3);
//!    - a refusal is CORRECT iff the bound held AND the held-out gain
//!      (identity − held-out post) ≤ 5% of identity — the doomed
//!      confirmation. A wrong refusal fails the lane honest.
//!
//!    The PASS condition: ≥ 1 suite-arm refused AND every refusal
//!    correct. No refusal ⇒ the negative is recorded under katgpt-rs
//!    Issue 978 and the primitive stays opt-in.
//! 4. **The cost ledger** — wall-clock(ρ reads + whitening, post-encode)
//!    vs wall-clock(the Procrustes solve) per arm; the laya forwards are
//!    the paired-data cost, common to both paths, disclosed separately.
//!
//! # Posture + hygiene
//!
//! ```text
//! LAYA_DEVICE=metal cargo run --release \
//!   --features modelless,laya-riir,laya-riir-metal,rho_probe \
//!   --example rho_suite_probe
//! ```
//!
//! Encodings ride the pinned english checkpoint loader unchanged
//! (`RiirAgent::load`, the harness's own path); the lane reads NO
//! accuracy — geometry only. Deterministic end to end: BTreeMap grouping,
//! sorted shard order, blake3-seeded permutation, no HashMap iteration.

#![cfg(all(feature = "laya-riir", feature = "rho_probe"))]

use std::collections::BTreeMap;
use std::time::Instant;

use serde_json::Value;

use riir_reflex::embed::Embedder;
use riir_reflex::harness::runner::case_questions;
use riir_reflex::harness::suites::{Suite, build_ag_news, build_banking77_mteb, build_emotion};

use katgpt_core::basis_split::build_zca_whitener;
use katgpt_core::linalg::symmetric_eig::SymmetricEigScratch;
use katgpt_core::nuisance_fraction::{RhoScratch, alignment_floor, rho_fraction};

/// Laya's hidden width (the pooled vector this lane folds per arm).
const LAYA_D: usize = 1024;
/// The k arms: 256 (4:1 fold; fit-half degenerate at n=400 — disclosed)
/// and 64 (16:1 fold; non-degenerate everywhere, k ≤ n/10).
const K_ARMS: [usize; 2] = [256, 64];
/// banking77 rows fed to the builder (the mirror's head is label-clustered;
/// 1200 keeps n ≫ k while bounding laya forwards).
const BANKING_ROWS: usize = 1200;
/// emotion / ag_news feed ALL rows on disk (400 each).
const FULL_ROWS: usize = 400;

/// The pinned correct-refusal bars (pre-registered — module docs).
const MAX_GAIN_SHARE: f64 = 0.05;
/// The permutation seed (fixed — the null must be reproducible).
const PERM_SEED: u64 = 0x106_E978;
/// The held-out estimate is VALID only when the fit is non-degenerate
/// (n_fit ≥ k — below it the rotation is not an estimate of any population
/// object) AND the null sits at ≈2k (null_held ≥ 0.95·2k — the split-half
/// protocol itself is sound). INVALID arms make no bound statement: their
/// held-out number is null-equivalent by construction, and comparing a
/// null-equivalent number against a population bound is meaningless (the
/// first record's "46–74% floor violation" was exactly this, plus the
/// in-sample artifact).
const NULL_AT_2K_SHARE: f64 = 0.95;
/// The held-out post is a finite-sample ESTIMATE of the population optimum
/// — it may sit slightly below the population bound without violating the
/// theorem. The estimator band: 2% of the floor (pre-registered).
const ESTIMATOR_BAND: f64 = 0.02;
/// The disclosed shrinkage floor for the polar factor's MᵀM (the
/// formatting_flip precedent): at k > n_fit (e.g. the null arm at
/// emotion/ag_news k=256, fit half 200) MᵀM is exactly rank-deficient and
/// the shipped builder refuses `CovarianceNotPositiveDefinite`; a
/// relative-diagonal floor keeps T = (MᵀM + δI)^{-1/2} defined at a
/// disclosed, negligible-for-nondegenerate cost.
const SHRINKAGE_REL: f64 = 1e-8;

struct SuiteRows {
    rows_file: Value,
    max_rows: usize,
}

fn load_suite_rows(name: &str, max_rows: usize) -> Option<SuiteRows> {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(format!(".raw/datasets/{name}"))
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|f| f.starts_with("test-") && f.ends_with(".json"))
        })
        .collect();
    paths.sort();
    if paths.is_empty() {
        return None;
    }
    let mut rows: Vec<Value> = Vec::new();
    for p in paths {
        let v: Value = serde_json::from_str(
            &std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display())),
        )
        .unwrap_or_else(|e| panic!("parse {}: {e}", p.display()));
        if let Some(rs) = v.get("rows").and_then(Value::as_array) {
            rows.extend(rs.iter().cloned());
        }
    }
    Some(SuiteRows {
        rows_file: serde_json::json!({ "rows": rows }),
        max_rows,
    })
}

/// The question text out of a built case's state (the three builder
/// shapes: banking77 `message`, ag_news `article`, emotion `text`).
fn case_text(state: &Value) -> Option<&str> {
    ["message", "article", "text"]
        .iter()
        .find_map(|key| state.get(key).and_then(Value::as_str))
}

/// The fixed stride-fold 1024→k (a disclosed deterministic projection of
/// space B's definition; 1024 % k must be 0 — both arms satisfy it).
fn fold(pooled: &[f32], k: usize) -> Vec<f32> {
    debug_assert_eq!(pooled.len(), LAYA_D);
    debug_assert_eq!(LAYA_D % k, 0, "fold requires an exact split");
    let block = LAYA_D / k;
    let mut out = vec![0.0f32; k];
    for (i, v) in pooled.iter().enumerate() {
        out[i % k] += *v;
    }
    let _ = block;
    out
}

/// Population mean + Σ (1/N) + ZCA whitener (formatting_flip's recipe).
/// Returns (mean, whitened centered points).
fn whiten(points: &[Vec<f32>], scratch: &mut SymmetricEigScratch) -> (Vec<f64>, Vec<f32>) {
    let k = points[0].len();
    let n = points.len();
    let mut mean = vec![0.0f64; k];
    for p in points {
        for (m, x) in mean.iter_mut().zip(p) {
            *m += f64::from(*x);
        }
    }
    for m in &mut mean {
        *m /= n as f64;
    }
    let mut sigma = vec![0.0f64; k * k];
    for p in points {
        let cen: Vec<f64> = p.iter().map(|x| f64::from(*x)).collect();
        for i in 0..k {
            let ci = cen[i] - mean[i];
            for (j, cj) in cen.iter().enumerate() {
                sigma[i * k + j] += ci * (cj - mean[j]);
            }
        }
    }
    for v in &mut sigma {
        *v /= n as f64;
    }
    let mut whitener = vec![0.0f64; k * k];
    let mut vals = vec![0.0f64; k];
    let mut vecs = vec![0.0f64; k * k];
    build_zca_whitener(&sigma, k, &mut whitener, &mut vals, &mut vecs, scratch)
        .expect("zca whitener");
    let mut out = Vec::with_capacity(n * k);
    for p in points {
        let mut w = vec![0.0f64; k];
        for (i, x) in p.iter().enumerate() {
            let ci = f64::from(*x) - mean[i];
            for (o, row) in w.iter_mut().zip(whitener[i * k..(i + 1) * k].iter()) {
                *o += ci * row;
            }
        }
        out.extend(w.iter().map(|v| *v as f32));
    }
    (mean, out)
}

fn rms2(a: &[f32], b: &[f32], k: usize) -> f64 {
    let n = a.len() / k;
    let mut acc = 0.0f64;
    for i in 0..n {
        let (ar, br) = (&a[i * k..(i + 1) * k], &b[i * k..(i + 1) * k]);
        for (x, y) in ar.iter().zip(br) {
            let d = f64::from(*x) - f64::from(*y);
            acc += d * d;
        }
    }
    acc / n as f64
}

/// Score ‖a·R − b‖² mean per row at the given index subset.
fn rms2_at(a: &[f32], b: &[f32], r: &[f64], k: usize, idx: &[usize]) -> f64 {
    let mut acc = 0.0f64;
    for &i in idx {
        let ar = &a[i * k..(i + 1) * k];
        let br = &b[i * k..(i + 1) * k];
        for (y, by) in br.iter().enumerate() {
            let mut o = 0.0f64;
            for (x, ax) in ar.iter().enumerate() {
                o += f64::from(*ax) * r[x * k + y];
            }
            let d = o - f64::from(*by);
            acc += d * d;
        }
    }
    acc / idx.len() as f64
}

/// The orthogonal Procrustes fit on the index subset `fit_idx`:
/// R = M·(MᵀM)^{-1/2}, M = Σ â_iᵀ·b̂_i over the subset (the shipped
/// `build_zca_whitener` supplies the inverse square root — a hand-rolled
/// V·diag·Vᵀ mis-pairs the solver's eigenvector storage layout, measured
/// the hard way: post-RMS² 110× above identity, an impossibility the
/// optimal ≤ identity invariant caught).
fn procrustes_fit(
    a: &[f32],
    b: &[f32],
    k: usize,
    fit_idx: &[usize],
    scratch: &mut SymmetricEigScratch,
) -> Vec<f64> {
    let mut m = vec![0.0f64; k * k];
    for &i in fit_idx {
        let ar = &a[i * k..(i + 1) * k];
        let br = &b[i * k..(i + 1) * k];
        for (x, ax) in ar.iter().enumerate() {
            let axv = f64::from(*ax);
            for (y, by) in br.iter().enumerate() {
                m[x * k + y] += axv * f64::from(*by);
            }
        }
    }
    let mut s = vec![0.0f64; k * k];
    for x in 0..k {
        for y in x..k {
            let mut acc = 0.0f64;
            for t in 0..k {
                acc += m[t * k + x] * m[t * k + y];
            }
            s[x * k + y] = acc;
            s[y * k + x] = acc;
        }
    }
    let tr: f64 = (0..k).map(|i| s[i * k + i]).sum();
    let s_floor = SHRINKAGE_REL * tr / k as f64;
    for i in 0..k {
        s[i * k + i] += s_floor;
    }
    let mut t = vec![0.0f64; k * k];
    {
        let mut vals = vec![0.0f64; k];
        let mut vecs = vec![0.0f64; k * k];
        build_zca_whitener(&s, k, &mut t, &mut vals, &mut vecs, scratch)
            .expect("inverse square root of MᵀM");
    }
    let mut r = vec![0.0f64; k * k];
    for i in 0..k {
        for j in 0..k {
            let mut acc = 0.0f64;
            for t2 in 0..k {
                acc += m[i * k + t2] * t[t2 * k + j];
            }
            r[i * k + j] = acc;
        }
    }
    r
}

/// blake3-seeded Fisher-Yates: the deterministic permutation the null uses.
fn seeded_permutation(n: usize) -> Vec<usize> {
    let mut h = blake3::Hasher::new();
    h.update(&PERM_SEED.to_le_bytes());
    let digest = h.finalize();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(digest.as_bytes());
    // xorshift64* from the blake3 digest head — deterministic, no deps.
    let mut state = u64::from_le_bytes(seed[..8].try_into().expect("8 bytes"));
    let mut perm: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state >> 11) as usize % (i + 1);
        perm.swap(i, j);
    }
    perm
}

fn rho_read(
    name: &str,
    points: &[Vec<f32>],
    labels: &[usize],
    k: usize,
    scratch: &mut RhoScratch,
) -> Option<f64> {
    let mut groups: BTreeMap<usize, Vec<Vec<f32>>> = BTreeMap::new();
    for (p, l) in points.iter().zip(labels) {
        groups.entry(*l).or_default().push(p.clone());
    }
    let kept: Vec<usize> = groups
        .keys()
        .copied()
        .filter(|l| groups[l].len() >= 2)
        .collect();
    let singles: Vec<usize> = groups
        .keys()
        .copied()
        .filter(|l| groups[l].len() < 2)
        .collect();
    if singles.is_empty() {
        println!("  {name:>10}: all {} labels have ≥2 members", groups.len());
    } else {
        println!(
            "  {name:>10}: {} labels ≥2 members; singleton labels EXCLUDED BY NAME: {:?}",
            kept.len(),
            singles
        );
    }
    if kept.len() < 2 {
        println!("  {name:>10}: REFUSED loud (fewer than 2 multi-member labels)");
        return None;
    }
    let flat: Vec<Vec<f32>> = kept
        .iter()
        .map(|l| groups[l].iter().flatten().copied().collect())
        .collect();
    let refs: Vec<&[f32]> = flat.iter().map(|v| &v[..]).collect();
    let n: usize = refs.iter().map(|g| g.len() / k).sum();
    if n < k {
        println!("  {name:>10}: REFUSED loud (rank-deficient: n={n} < k={k})");
        return None;
    }
    match rho_fraction(&refs, scratch) {
        Ok(r) => {
            println!(
                "  {name:>10}: ρ={:.4}  (G={}, k={k}, n={n})",
                r.rho,
                kept.len()
            );
            Some(r.rho)
        }
        Err(e) => {
            println!("  {name:>10}: REFUSED loud: {e}");
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Verdict {
    RefusedCorrect,
    RefusedWrong,
    Permitted,
    Skipped,
}

/// One (suite, k) arm. `pooled` is the already-encoded 1024 mean-pool per
/// question; folding per arm is local.
#[allow(clippy::too_many_lines)]
fn run_arm(
    name: &str,
    pooled: &[Vec<f32>],
    texts: &[&str],
    labels: &[usize],
    k: usize,
    embedder: &Embedder,
    eig: &mut SymmetricEigScratch,
) -> Verdict {
    println!("\n  ── k={k} arm ──");
    let n = pooled.len();
    // Modelless space at k buckets.
    let mut mdl_pts: Vec<Vec<f32>> = Vec::with_capacity(n);
    for text in texts {
        let mut buf = vec![0.0f32; k];
        embedder.embed_into(text.as_bytes(), &mut buf);
        mdl_pts.push(buf);
    }
    let laya_pts: Vec<Vec<f32>> = pooled.iter().map(|p| fold(p, k)).collect();

    let mut rho_scratch = RhoScratch::new(k);
    let t_rho = Instant::now();
    let rho_mdl = rho_read("modelless", &mdl_pts, labels, k, &mut rho_scratch);
    let rho_laya = rho_read("laya", &laya_pts, labels, k, &mut rho_scratch);
    let rho_secs = t_rho.elapsed().as_secs_f64();
    let (Some(ra), Some(rb)) = (rho_mdl, rho_laya) else {
        println!("  {name} k={k}: gate skipped — a ρ read refused loud");
        return Verdict::Skipped;
    };

    // Population whitening (the ρ convention), then the split.
    let (_, a_w) = whiten(&mdl_pts, eig);
    let (_, b_w) = whiten(&laya_pts, eig);
    let ident = rms2(&a_w, &b_w, k);
    let all: Vec<usize> = (0..n).collect();
    let fit_idx: Vec<usize> = all.iter().copied().step_by(2).collect();
    let held_idx: Vec<usize> = all.iter().copied().skip(1).step_by(2).collect();

    let floor = alignment_floor(ra, rb, k);
    let refuse = floor >= ident;
    println!(
        "  gate: floor=2k(1−min ρ)={floor:.4}  RMS²(R=I)={ident:.4}  ⇒ {}",
        match refuse {
            true => "REFUSE (floor ≥ identity — no population rotation can gain)",
            false => "permit (floor < identity — solve runs anyway)",
        }
    );

    // ── held-out adjudication (the round-2 protocol) ──
    let t_solve = Instant::now();
    let r_fit = procrustes_fit(&a_w, &b_w, k, &fit_idx, eig);
    let post_insample = rms2_at(&a_w, &b_w, &r_fit, k, &fit_idx);
    let post_held = rms2_at(&a_w, &b_w, &r_fit, k, &held_idx);
    // The permutation null: destroy the pairing, same split — calibrates
    // the in-sample artifact for this exact (n, k).
    let perm = seeded_permutation(n);
    let mut b_shuf = vec![0.0f32; b_w.len()];
    for (i, &j) in perm.iter().enumerate() {
        b_shuf[i * k..(i + 1) * k].copy_from_slice(&b_w[j * k..(j + 1) * k]);
    }
    let r_null = procrustes_fit(&a_w, &b_shuf, k, &fit_idx, eig);
    let null_insample = rms2_at(&a_w, &b_shuf, &r_null, k, &fit_idx);
    let null_held = rms2_at(&a_w, &b_shuf, &r_null, k, &held_idx);
    let solve_secs = t_solve.elapsed().as_secs_f64();

    let bound_est_valid = fit_idx.len() >= k && null_held >= NULL_AT_2K_SHARE * 2.0 * k as f64;
    let bound_held = post_held >= floor * (1.0 - ESTIMATOR_BAND);
    let gain = ident - post_held;
    let gain_share = gain / ident.max(1e-12);
    let verdict = match refuse {
        true if bound_est_valid && bound_held && gain_share <= MAX_GAIN_SHARE => {
            Verdict::RefusedCorrect
        }
        true => Verdict::RefusedWrong,
        false => Verdict::Permitted,
    };
    println!(
        "  adjudication (held-out): fit-half in-sample post={post_insample:.4} (artifact-prone); \
         HELD-OUT post={post_held:.4}; null: in-sample={null_insample:.4} held-out={null_held:.4}"
    );
    println!(
        "  bound statement: {} (n_fit={} vs k={k}, null/2k={:.3}); {}",
        match bound_est_valid {
            true => "estimate VALID",
            false => "estimate NOT VALID — degenerate fit or null off 2k; NO bound statement",
        },
        fit_idx.len(),
        null_held / (2.0 * k as f64),
        match bound_est_valid {
            true => match bound_held {
                true => format!(
                    "floor={floor:.4} ≤ held-out post (within the {}% estimator band) ⇒ HELD",
                    ESTIMATOR_BAND * 100.0
                ),
                false => format!(
                    "floor={floor:.4} > held-out post beyond the estimator band ⇒ VIOLATED — investigate"
                ),
            },
            false => "no comparison drawn".to_owned(),
        }
    );
    println!(
        "  held-out gain: {gain:.4} ({:.1}% of identity) ⇒ {}",
        gain_share * 100.0,
        match verdict {
            Verdict::RefusedCorrect => "REFUSAL CORRECT (doomed transport confirmed)",
            Verdict::RefusedWrong => "REFUSAL WRONG (fails the lane)",
            Verdict::Permitted if gain_share <= MAX_GAIN_SHARE => {
                "permitted but held-out gain ≤5% — a refusal WOULD have been correct"
            }
            Verdict::Permitted =>
                "permitted and the rotation gained held-out — the floor was right to permit",
            Verdict::Skipped => unreachable!(),
        }
    );
    println!(
        "  ledger: ρ+whiten {rho_secs:.3}s | solves (fit+null) {solve_secs:.3}s | \
         laya forwards: common cost (see suite header)"
    );
    verdict
}

fn run_suite(suite: &Suite, agent: &mut laya_face::Agent, embedder: &Embedder) -> Vec<Verdict> {
    let name = suite.name;
    let mut texts: Vec<&str> = Vec::new();
    let mut labels: Vec<usize> = Vec::new();
    let mut skipped_state = 0usize;
    for case in &suite.cases {
        match case_text(&case.state) {
            Some(t) => {
                texts.push(t);
                labels.push(case.gold[0].idx);
            }
            None => skipped_state += 1,
        }
    }
    if skipped_state > 0 {
        println!(
            "\n== {name}: {} cases have no message/article/text state — skipped, counted",
            skipped_state
        );
    }
    let n = texts.len();
    println!("\n== {name}: {n} questions ==");
    let t0 = Instant::now();
    let mut pooled: Vec<Vec<f32>> = Vec::with_capacity(n);
    let mut forward_refusals = 0usize;
    for (case, _text) in suite.cases.iter().zip(&texts) {
        let qdef = &case_questions(case)[0].1;
        match agent.encode(&case.state, qdef) {
            Ok(enc) => pooled.push(enc),
            Err(e) => {
                forward_refusals += 1;
                if forward_refusals == 1 {
                    println!("  {name}: laya forward refusals begin: {e}");
                }
            }
        }
    }
    let laya_secs = t0.elapsed().as_secs_f64();
    if forward_refusals > 0 {
        println!(
            "  {name}: {forward_refusals} of {n} laya forwards refused — the suite reads on \
             the landed subset"
        );
    }
    let landed = pooled.len();
    println!("  laya forwards: {landed} in {laya_secs:.3}s (common cost, once per suite)");
    if landed <= *K_ARMS.iter().max().expect("non-empty") {
        println!(
            "  {name}: too few landed points ({landed}) for the largest k arm — suite skipped"
        );
        return vec![Verdict::Skipped];
    }
    let texts = &texts[..landed];
    let labels = &labels[..landed];
    let mut eig = SymmetricEigScratch::new();
    let mut verdicts = Vec::new();
    for k in K_ARMS {
        verdicts.push(run_arm(
            name,
            &pooled[..landed],
            texts,
            labels,
            k,
            embedder,
            &mut eig,
        ));
    }
    verdicts
}

fn main() {
    let embedder = Embedder;
    let mut agent = laya_face::Agent::load();

    let suites: Vec<Suite> = [
        load_suite_rows("banking77", BANKING_ROWS)
            .map(|s| build_banking77_mteb(&s.rows_file, s.max_rows)),
        load_suite_rows("emotion", FULL_ROWS).map(|s| build_emotion(&s.rows_file, s.max_rows)),
        load_suite_rows("ag_news", FULL_ROWS).map(|s| build_ag_news(&s.rows_file, s.max_rows)),
    ]
    .into_iter()
    .flatten()
    .collect();
    if suites.len() < 2 {
        panic!(
            "fewer than 2 suites loaded from .raw/datasets — run scripts/fetch_datasets.sh \
             first (a missing corpus is a loud refusal, never a green zero)"
        );
    }
    println!(
        "suites: {} ({} rows total) | k arms {K_ARMS:?} | pinned bars: estimator_band={}%, null/2k floor={}, \
         max_gain_share={MAX_GAIN_SHARE}, perm_seed={PERM_SEED:#x}",
        suites.iter().map(|s| s.name).collect::<Vec<_>>().join(", "),
        suites.iter().map(|s| s.cases.len()).sum::<usize>(),
        ESTIMATOR_BAND * 100.0,
        NULL_AT_2K_SHARE,
    );

    let mut refused = 0usize;
    let mut refused_wrong = 0usize;
    let mut permitted = 0usize;
    let mut skipped = 0usize;
    for s in &suites {
        for v in run_suite(s, &mut agent, &embedder) {
            match v {
                Verdict::RefusedCorrect => refused += 1,
                Verdict::RefusedWrong => refused_wrong += 1,
                Verdict::Permitted => permitted += 1,
                Verdict::Skipped => skipped += 1,
            }
        }
    }
    println!(
        "\nlane tally: {refused} correct refusals / {refused_wrong} wrong refusals / \
         {permitted} permitted / {skipped} skipped arms — {}",
        match (refused, refused_wrong) {
            (r, 0) if r >= 1 => "PASS (≥1 refused, every refusal correct per the pinned bars)",
            (_, 0) =>
                "FAIL (no refusal — no saving shown; the negative is recorded under \
                      katgpt-rs 978)",
            (_, _) => "FAIL (a wrong refusal is worse than no gate)",
        }
    );
    println!("REPORT-ONLY: nothing armed here; results land under issue 106 + katgpt-rs 978.");
}

/// The laya face, isolated so the encoder surface stays one call away.
mod laya_face {
    use serde_json::Value;

    use crate::LAYA_D;

    pub struct Agent(riir_reflex::laya::riir::RiirAgent);

    impl Agent {
        pub fn load() -> Self {
            use riir_reflex::laya::config::Checkpoint;
            use riir_reflex::laya::weights::weights_root;
            let inner =
                riir_reflex::laya::riir::RiirAgent::load(&weights_root(), Checkpoint::English)
                    .expect("load the pinned english checkpoint");
            Self(inner)
        }

        /// Mean-pooled final hidden states at the native 1024 width (the
        /// caller folds per k arm — the forward runs once per question).
        pub fn encode(&mut self, state: &Value, qdef: &Value) -> Result<Vec<f32>, String> {
            let enc = self
                .0
                .encode_question(state, qdef)
                .map_err(|e| e.to_string())?;
            let seq = enc.hidden.len() / LAYA_D;
            let mut acc = vec![0.0f32; LAYA_D];
            for chunk in enc.hidden.as_chunks::<LAYA_D>().0 {
                for (o, v) in acc.iter_mut().zip(chunk) {
                    *o += *v;
                }
            }
            for v in &mut acc {
                *v /= seq as f32;
            }
            Ok(acc)
        }
    }
}
