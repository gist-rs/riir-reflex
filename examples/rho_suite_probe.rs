//! Issue 106 — the ρ-per-suite embedding read + the ρ-gated cross-lane
//! Procrustes probe (katgpt-rs Issue 978 T5 consumer lane; REPORT-ONLY —
//! nothing arms here, no exit-code coupling to any serving lane).
//!
//! # What one run prints
//!
//! For each suite (banking77 first-1200 rows, emotion + ag_news all rows):
//!
//! 1. **ρ per space grouped by gold label** — the state-explained variance
//!    fraction (katgpt-core `nuisance_fraction`) of the suite's questions
//!    embedded in TWO spaces: (a) the shipped modelless hashed embedder
//!    (`crate::embed::Embedder`, FNV-1a word+bigram sign-hash) at k=256
//!    buckets, (b) the pinned laya english encoder (`laya-riir`) mean-pooled
//!    final hidden states, stride-folded 1024→256 (a FIXED deterministic
//!    projection — dims i, i+256, i+512, i+768 summed — disclosed as part
//!    of space B's definition; both spaces measure at the same k so the
//!    Procrustes is defined). Singleton labels (fewer than 2 questions) are
//!    excluded BY NAME — a single-view group refuses loud in the primitive,
//!    never silently.
//! 2. **The pre-registered gate** — katgpt-rs Issue 978 T4's bar needs a
//!    consumer showing the alignment floor refuses a doomed cross-lane
//!    transport BEFORE its solve is spent. The transport here is the
//!    orthogonal Procrustes R (k×k) mapping the modelless space onto the
//!    laya space over the paired questions. The pinned refusal rule is
//!    theorem-pure, no calibration knob: **refuse iff
//!    `alignment_floor(ρ_a, ρ_b, k) ≥ RMS²(R = I)`** — the Thm-3 lower
//!    bound on the OPTIMAL rotation already exceeds the residual of doing
//!    nothing, so no rotation can gain (optimal ≥ floor ≥ identity
//!    ⟹ optimal ≈ identity). The rule is regime-free: it needs no
//!    calibrated ρ threshold, which matters because every suite here sits
//!    in the G≪k compression regime (G = 77/6/4 labels vs k = 256) where
//!    absolute ρ values compress (monotone-informative within the shape,
//!    not calibrated — the primitive's own caveat; both spaces share the
//!    shape, and the rule never compares ρ across shapes).
//! 3. **The out-of-band adjudication** — the Procrustes runs REGARDLESS
//!    (the ground truth the refusal is adjudicated against): R = polar of
//!    the whitened cross-covariance (M·(MᵀM)^{-1/2}, the Issue-978 T3 arm's
//!    convention, via the public `symmetric_eig`), post-RMS² recorded.
//!    A refusal is CORRECT iff (a) the bound held:
//!    `post-RMS² ≥ floor·(1 − 1e-3)` (finite-sample solver tolerance), and
//!    (b) the rotation gained ≤ 5% of the identity residual — the doomed
//!    confirmation. A refusal where either bar fails is a WRONG refusal and
//!    fails the lane honest (a gate that refuses wrongly is worse than no
//!    gate). The PASS condition: ≥ 1 suite refused AND every refusal
//!    correct. No refusal (floor < identity everywhere) ⇒ the negative is
//!    recorded under katgpt-rs Issue 978 and the primitive stays opt-in.
//! 4. **The cost ledger** — wall-clock(ρ reads + whitening, post-encode) vs
//!    wall-clock(the Procrustes solve) per suite. The laya forwards are the
//!    paired-data cost, common to both paths (the pair-flip class census
//!    embeds first too), disclosed separately.
//!
//! # Posture + hygiene
//!
//! ```text
//! LAYA_DEVICE=metal cargo run --release \
//!   --features modelless,laya-riir,laya-riir-metal,rho_probe \
//!   --example rho_suite_probe
//! ```
//!
//! Metal posture (M5 Ultra); CPU works with `--features
//! modelless,laya-riir,rho_probe` and `LAYA_DEVICE=cpu`, disclosed by the
//! device line. Encodings ride the pinned english checkpoint loader
//! unchanged (`RiirAgent::load`, the harness's own path); the lane reads
//! NO accuracy — geometry only. Deterministic end to end: BTreeMap
//! grouping, sorted shard order, no HashMap iteration.

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

/// Both spaces measure at k=256 (the 4:1 stride-fold of laya's 1024; the
/// modelless embedder natively takes the bucket count from the buffer).
const K: usize = 256;
/// banking77 rows fed to the builder (the mirror's head is label-clustered;
/// 1200 keeps n ≫ k while bounding laya forwards).
const BANKING_ROWS: usize = 1200;
/// emotion / ag_news feed ALL rows on disk (400 each — n ≥ k holds).
const FULL_ROWS: usize = 400;

/// The pinned correct-refusal bars (pre-registered; see the module docs).
const BOUND_TOL: f64 = 1e-3;
const MAX_GAIN_SHARE: f64 = 0.05;

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
        // Re-wrap the merged rows in the envelope shape the builders read.
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

/// The 4:1 stride-fold 1024→256 (fixed, deterministic, disclosed).
fn fold1024(hidden: &[f32]) -> Vec<f32> {
    debug_assert_eq!(hidden.len() % 1024, 0);
    let mut pooled = [0.0f64; 1024];
    // Mean-pool over the sequence: hidden is row-major [seq_len, 1024].
    let seq = hidden.len() / 1024;
    for chunk in hidden.as_chunks::<1024>().0 {
        for (o, v) in pooled.iter_mut().zip(chunk) {
            *o += f64::from(*v);
        }
    }
    for v in &mut pooled {
        *v /= seq as f64;
    }
    (0..K)
        .map(|i| (pooled[i] + pooled[i + K] + pooled[i + 2 * K] + pooled[i + 3 * K]) as f32)
        .collect()
}

/// Population mean + Σ (1/N) + ZCA whitener (formatting_flip's recipe).
/// Returns (mean, whitened centered points) — the mean is informational.
fn whiten(points: &[Vec<f32>], scratch: &mut SymmetricEigScratch) -> (Vec<f64>, Vec<f32>) {
    let n = points.len();
    let mut mean = vec![0.0f64; K];
    for p in points {
        for (m, x) in mean.iter_mut().zip(p) {
            *m += f64::from(*x);
        }
    }
    for m in &mut mean {
        *m /= n as f64;
    }
    let mut sigma = vec![0.0f64; K * K];
    for p in points {
        let cen: Vec<f64> = p.iter().map(|x| f64::from(*x)).collect();
        for i in 0..K {
            let ci = cen[i] - mean[i];
            for (j, cj) in cen.iter().enumerate() {
                sigma[i * K + j] += ci * (cj - mean[j]);
            }
        }
    }
    for v in &mut sigma {
        *v /= n as f64;
    }
    let mut whitener = vec![0.0f64; K * K];
    let mut vals = vec![0.0f64; K];
    let mut vecs = vec![0.0f64; K * K];
    build_zca_whitener(&sigma, K, &mut whitener, &mut vals, &mut vecs, scratch)
        .expect("zca whitener");
    // Whiten the centered points in place of the caller.
    let mut out = Vec::with_capacity(n * K);
    for p in points {
        let mut w = vec![0.0f64; K];
        for (i, x) in p.iter().enumerate() {
            let ci = f64::from(*x) - mean[i];
            for (o, row) in w.iter_mut().zip(whitener[i * K..(i + 1) * K].iter()) {
                *o += ci * row;
            }
        }
        out.extend(w.iter().map(|v| *v as f32));
    }
    (mean, out)
}

fn rms2_identity(a: &[f32], b: &[f32]) -> f64 {
    let n = a.len() / K;
    let mut acc = 0.0f64;
    for i in 0..n {
        let (ar, br) = (&a[i * K..(i + 1) * K], &b[i * K..(i + 1) * K]);
        for (x, y) in ar.iter().zip(br) {
            let d = f64::from(*x) - f64::from(*y);
            acc += d * d;
        }
    }
    acc / n as f64
}

/// The orthogonal Procrustes on whitened centered points: R minimizes
/// ‖a·R − b‖_F; R = M·(MᵀM)^{-1/2}, M = AᵀB (the Issue-978 T3 convention).
/// Returns (R, post-RMS²). The pseudo-inverse floor follows the T3 arm:
/// eigenvalues floored at 1e-10·λ_max before the −1/2 power.
fn procrustes(a: &[f32], b: &[f32], scratch: &mut SymmetricEigScratch) -> (Vec<f64>, f64) {
    let n = a.len() / K;
    // M = Σ_i â_iᵀ · b̂_i  (k×k, row-major; column-vector convention as in
    // the T3 amendment — the element-product polar form is invariant to the
    // reading, which is why the shipped helper uses it).
    let mut m = vec![0.0f64; K * K];
    for i in 0..n {
        let ar = &a[i * K..(i + 1) * K];
        let br = &b[i * K..(i + 1) * K];
        for (x, ax) in ar.iter().enumerate() {
            let axv = f64::from(*ax);
            for (y, by) in br.iter().enumerate() {
                m[x * K + y] += axv * f64::from(*by);
            }
        }
    }
    // S = MᵀM.
    let mut s = vec![0.0f64; K * K];
    for x in 0..K {
        for y in x..K {
            let mut acc = 0.0f64;
            for t in 0..K {
                acc += m[t * K + x] * m[t * K + y];
            }
            s[x * K + y] = acc;
            s[y * K + x] = acc;
        }
    }
    // S = MᵀM; the inverse square root T = S^{-1/2} is EXACTLY the ZCA
    // whitener of S (the shipped builder's eigenvalue-floor + assembly are
    // correctness-proven — a hand-rolled V·diag·Vᵀ mis-pairs the solver's
    // eigenvector storage layout, measured the hard way: the wrong reading
    // put post-RMS² 110× above identity, an impossibility for a true polar
    // factor, caught by the optimal ≤ identity invariant).
    let mut t = vec![0.0f64; K * K];
    {
        let mut vals = vec![0.0f64; K];
        let mut vecs = vec![0.0f64; K * K];
        build_zca_whitener(&s, K, &mut t, &mut vals, &mut vecs, scratch)
            .expect("inverse square root of MᵀM");
    }
    let mut r = vec![0.0f64; K * K];
    for i in 0..K {
        for j in 0..K {
            let mut acc = 0.0f64;
            for t2 in 0..K {
                acc += m[i * K + t2] * t[t2 * K + j];
            }
            r[i * K + j] = acc;
        }
    }
    // post-RMS² = mean ‖â·R − b̂‖².
    let mut acc = 0.0f64;
    for i in 0..n {
        let (ar, br) = (&a[i * K..(i + 1) * K], &b[i * K..(i + 1) * K]);
        for (y, by) in br.iter().enumerate() {
            let mut o = 0.0f64;
            for (x, ax) in ar.iter().enumerate() {
                o += f64::from(*ax) * r[x * K + y];
            }
            let d = o - f64::from(*by);
            acc += d * d;
        }
    }
    (r, acc / n as f64)
}

fn rho_read(
    name: &str,
    points: &[Vec<f32>],
    labels: &[usize],
    scratch: &mut RhoScratch,
) -> Option<f64> {
    // Group by gold label; singleton labels excluded BY NAME (the
    // primitive refuses a single-view group loud — disclosed, never
    // silently filtered).
    let mut groups: BTreeMap<usize, Vec<Vec<f32>>> = BTreeMap::new();
    let mut singles: Vec<usize> = Vec::new();
    for (p, l) in points.iter().zip(labels) {
        groups.entry(*l).or_default().push(p.clone());
    }
    let kept: Vec<usize> = groups
        .keys()
        .copied()
        .filter(|l| groups[l].len() >= 2)
        .collect();
    singles.extend(groups.keys().filter(|l| groups[l].len() < 2));
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
    let n: usize = refs.iter().map(|g| g.len() / K).sum();
    if n < K {
        println!("  {name:>10}: REFUSED loud (rank-deficient: n={n} < k={K})");
        return None;
    }
    match rho_fraction(&refs, scratch) {
        Ok(r) => {
            println!(
                "  {name:>10}: ρ={:.4}  (G={}, k={K}, n={n})",
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

fn run_suite(suite: &Suite, agent: &mut laya_face::Agent, embedder: &Embedder) -> Verdict {
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
    let g_all = labels
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    println!("\n== {name}: {n} questions (k={K}, ≤{g_all} labels) ==");

    // ── laya space (the expensive half; timed separately, common cost) ──
    let t0 = Instant::now();
    let mut laya_pts: Vec<Vec<f32>> = Vec::with_capacity(n);
    let mut forward_refusals = 0usize;
    for (case, _text) in suite.cases.iter().zip(&texts) {
        let qdef = &case_questions(case)[0].1;
        match agent.encode(&case.state, qdef) {
            Ok(enc) => laya_pts.push(fold1024(&enc)),
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
    let landed = laya_pts.len();
    if landed <= K {
        println!(
            "  {name}: too few landed points ({landed}) for k={K} — the ρ read would be rank-deficient; gate skipped"
        );
        return Verdict::Skipped;
    }
    labels.truncate(landed);

    // ── modelless space ──
    let mut mdl_pts: Vec<Vec<f32>> = Vec::with_capacity(landed);
    for text in &texts[..landed] {
        let mut buf = vec![0.0f32; K];
        embedder.embed_into(text.as_bytes(), &mut buf);
        mdl_pts.push(buf);
    }

    // ── ρ per space (label groups) ──
    let mut rho_scratch = RhoScratch::new(K);
    let t_rho = Instant::now();
    let rho_mdl = rho_read("modelless", &mdl_pts, &labels, &mut rho_scratch);
    let rho_laya = rho_read("laya", &laya_pts, &labels, &mut rho_scratch);
    let rho_secs = t_rho.elapsed().as_secs_f64();

    let (Some(ra), Some(rb)) = (rho_mdl, rho_laya) else {
        println!("  {name}: gate skipped — a ρ read refused loud");
        return Verdict::Skipped;
    };

    // ── the gate: floor vs identity residual on whitened centered points ──
    let mut eig = SymmetricEigScratch::new();
    let (_, a_w) = whiten(&mdl_pts, &mut eig);
    let (_, b_w) = whiten(&laya_pts, &mut eig);
    let ident = rms2_identity(&a_w, &b_w);
    let floor = alignment_floor(ra, rb, K);
    let refuse = floor >= ident;
    println!(
        "  gate: floor=2k(1−min ρ)={floor:.4}  RMS²(R=I)={ident:.4}  ⇒ {}",
        match refuse {
            true => "REFUSE the transport (floor ≥ identity — no rotation can gain)",
            false => "permit (floor < identity — a rotation may gain; solve runs anyway)",
        }
    );

    // ── out-of-band adjudication (the Procrustes runs REGARDLESS) ──
    let t_solve = Instant::now();
    let (_r, post) = procrustes(&a_w, &b_w, &mut eig);
    let solve_secs = t_solve.elapsed().as_secs_f64();
    let bound_held = post >= floor * (1.0 - BOUND_TOL);
    let gain = ident - post;
    let gain_share = gain / ident.max(1e-12);
    let verdict = match refuse {
        true if bound_held && gain_share <= MAX_GAIN_SHARE => Verdict::RefusedCorrect,
        true => Verdict::RefusedWrong,
        false => Verdict::Permitted,
    };
    println!(
        "  adjudication: post-RMS²={post:.4} (floor {floor:.4}, bound {}); \
         gain={gain:.4} ({:.1}% of identity) ⇒ {}",
        match bound_held {
            true => "HELD",
            false => "VIOLATED (finite-sample solver noise — disclosed)",
        },
        gain_share * 100.0,
        match verdict {
            Verdict::RefusedCorrect => "REFUSAL CORRECT (doomed transport confirmed)",
            Verdict::RefusedWrong =>
                "REFUSAL WRONG (a wrongly-refused viable transport fails the lane)",
            Verdict::Permitted if gain_share <= MAX_GAIN_SHARE => {
                "permitted but rotation gained ≤5% — a refusal WOULD have been correct"
            }
            Verdict::Permitted =>
                "permitted and the rotation gained — the floor was right to permit",
            Verdict::Skipped => unreachable!(),
        }
    );
    println!(
        "  ledger: ρ+whiten {rho_secs:.3}s | procrustes solve {solve_secs:.3}s | \
         laya forwards (common cost) {laya_secs:.3}s"
    );
    verdict
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
        "suites: {} ({} rows total) | k={K} | pinned bars: bound_tol={BOUND_TOL}, \
         max_gain_share={MAX_GAIN_SHARE}",
        suites.iter().map(|s| s.name).collect::<Vec<_>>().join(", "),
        suites.iter().map(|s| s.cases.len()).sum::<usize>(),
    );

    let mut refused = 0usize;
    let mut refused_wrong = 0usize;
    let mut permitted = 0usize;
    for s in &suites {
        match run_suite(s, &mut agent, &embedder) {
            Verdict::RefusedCorrect => refused += 1,
            Verdict::RefusedWrong => refused_wrong += 1,
            Verdict::Permitted => permitted += 1,
            Verdict::Skipped => {}
        }
    }
    println!(
        "\nlane verdict: {refused} correct refusals / {refused_wrong} wrong refusals / \
         {permitted} permitted — {}",
        match (refused, refused_wrong) {
            (r, 0) if r >= 1 => "PASS (≥1 refused, every refusal correct per the pinned bars)",
            (_, 0) => "FAIL (no refusal — the negative is recorded under katgpt-rs 978)",
            (_, w) if w > 0 => "FAIL (a wrong refusal is worse than no gate)",
            _ => "FAIL",
        }
    );
    println!("REPORT-ONLY: nothing armed here; results land under issue 106 + katgpt-rs 978.");
}

/// The laya face, isolated so the encoder surface stays one call away.
mod laya_face {
    use serde_json::Value;

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

        /// Mean-pooled final hidden states, folded to k by the caller.
        pub fn encode(&mut self, state: &Value, qdef: &Value) -> Result<Vec<f32>, String> {
            self.0
                .encode_question(state, qdef)
                .map(|enc| enc.hidden)
                .map_err(|e| e.to_string())
        }
    }
}
