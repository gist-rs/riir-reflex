//! Issue 086 T1 — the perm-pair presentation-subspace certificate (pure core).
//!
//! Source: `../katgpt-rs/.research/618_SplitJEPA_Invariant_Variant_Block_Identifiability.md`
//! (SplitJEPA, arXiv:2610.12349); the primitive is katgpt-core `basis_split`
//! (Plan 625). The permutation probe (issue 077) answers robustness with a
//! black-box aggregate (median swing ≤ 2 pt); this certificate says WHERE the
//! ordering energy lives: it routes the probe's own perm pairs (content-
//! identical, presentation-differing) through the hashed-feature embed and
//! fits the **presentation-variant basis** — the top eigenspace of the
//! whitened pair-difference second moment — under the primitive's
//! refuse-loud identifiability verdict.
//!
//! # What each output means (the naming law)
//!
//! - `verdict` — does a rank-gated presentation subspace EXIST in the embed
//!   at all? `unidentified_*` is a RESULT, not a failure: the modelless
//!   hashed-feature embed is order-insensitive in its ctx path by
//!   construction, and the certificate saying so honestly is the point
//!   (Issue 086: "`Unidentified` postures are results, not failures").
//! - `pair_leakage` / `agreement_residual` — held-out generalization
//!   residuals of the fitted presentation null space over the holdout
//!   ordering pairs. Generalization residuals, NEVER purity (the
//!   katgpt-core naming law binds here too).
//! - `casing_null_residual` — the T2 cross-family arm's plumbing: the same
//!   basis evaluated on casing-permutation pairs. The embed's tokenizer is
//!   case-insensitive by construction (the embed crate's
//!   `case_and_punctuation_insensitive` test), so casing pairs carry Δ ≡ 0
//!   and are skipped; when every casing pair is null the arm reports
//!   `None` honestly rather than a fabricated 0.0.
//!
//! # The rendering law
//!
//! The presentation rendering of a permuted question is the full text a
//! bag-of-words reader of the presented question sees: `{state}\n
//! {instructions}\n` then one `{label}: {description}` line per option IN
//! PRESENTED ORDER. Order enters the hashed embed through the boundary
//! bigrams between consecutive option lines (unigrams are a set —
//! order-free; the FNV bigram hashes span line boundaries). This is a
//! certificate-side rendering, deliberately NOT the engine's decide-path
//! input (the modelless engine embeds state+prompt into its ctx and scores
//! options per-option — order-equivariant there by construction). The
//! certificate measures the presentation geometry of the QUESTION TEXT;
//! whether that geometry rides into a consumer's scores is the consumer's
//! business (T2 records the score-side agreement beside it).
//!
//! # Whitening + pair budget (the katgpt-core contracts)
//!
//! The whitener is ZCA over the **text population the pipeline consumes**:
//! the slots' identity renderings PLUS the suite's train-corpus documents
//! (deterministic stride cap) — d=256 needs more support than the ≤40 slot
//! identities alone can give the covariance positive-definiteness (the
//! population must cover every hash bucket — a structured small-vocabulary
//! population lands rank-deficient and the whitener refuses, as designed).
//! **The two whitening arms are EXCLUSIVE** (measured the hard way): the
//! moment is accumulated through `accumulate_whitened(W, …)` — the
//! `Whitener(W)` arm — and [`split`] receives that same arm; the
//! `TotalCovariance(Σ)` arm expects a RAW moment and would whiten AGAIN
//! (W²MW² grading → the QL phase fails to converge). Pair budget
//! (Research 618 measured facts): the null space identifies with ≈1k
//! pairs; [`MIN_FIT_PAIRS`] (= d) is the structural floor below which the
//! moment is rank-deficient, the rank gate degenerates, and the
//! certificate refuses (`unidentified_insufficient_pairs`) — the runner
//! samples its own slot budget to clear it (the probe's default 40-slot
//! sample is far below). The moment carries a disclosed shrinkage floor
//! (`SHRINKAGE_REL · tr(M)/d` on the diagonal): katgpt-core's QL phase
//! panics on an exactly-singular moment (measured), and the floor makes
//! the rank gate read "dimension of above-floor variation" instead of the
//! sample-size cutoff. Everything here is PURE; the embed is f32,
//! conversion happens at accumulate. The runner half lives in
//! `runner::basis_cert` behind the embed seam (no `ChoiceOracle` — the
//! certificate never runs an engine).

use katgpt_core::basis_split::{held_out_leakage, split, Whitening, MAX_DIM};

/// The embed width this certificate runs at (the hashed-feature embed's
/// own width, mirrored so the pure core stays decoupled from the
/// embedder type; the runner asserts the two agree).
pub const CERT_DIM: usize = 256;

/// `τ_gap` for the rank gate — the boundary must clear a 4× eigengap to
/// certify (the planted-recovery tests' posture).
pub const TAU_GAP: f64 = 4.0;

/// The shrinkage floor's relative size: `δ = SHRINKAGE_REL · tr(M)/d` is
/// added to the moment diagonal before the eigensolve. Numerical guard —
/// katgpt-core's QL phase panics on an exactly-singular moment (measured),
/// and the floor makes the rank gate read "dimension of above-floor
/// variation" instead of the sample-size cutoff. 1e-8 sits far below any
/// real structure and far above the f64 roundoff of the solve.
pub const SHRINKAGE_REL: f64 = 1e-8;

/// Train-corpus cap for the whitening population (deterministic stride;
/// the runner picks WHICH docs via the shared stride helper).
pub const WHITEN_POP_CAP: usize = 512;

/// The minimum fit-half pair count for the split to be meaningful: the
/// moment of `n` pairs has rank ≤ n, and a rank-deficient moment makes
/// the eigengap gate tautological (the largest gap lands at the rank
/// cutoff, "certifying" k = n_pairs — a statement about the sample size,
/// not the geometry). The Research-618 budget is ≈1k pairs for
/// identification; this floor is the structural minimum (rank d).
pub const MIN_FIT_PAIRS: usize = CERT_DIM;

/// One question's presentation inputs, order-carried: labels and their
/// descriptions IN PRESENTED ORDER (parallel slices; the runner fills
/// `descs` from the criteria map, empty strings when a label carries none).
#[derive(Debug, Clone)]
pub struct PresentationQuestion<'a> {
    pub state: &'a str,
    pub instructions: &'a str,
    pub labels: &'a [String],
    pub descs: &'a [String],
}

/// Renders the presentation text: `{state}\n{instructions}\n` + one
/// `{label}: {description}` line per option in presented order. The ONLY
/// place order enters the certified text.
#[must_use]
pub fn render_presentation(q: &PresentationQuestion<'_>) -> String {
    let mut s = String::with_capacity(128 + q.labels.len() * 48);
    s.push_str(q.state);
    s.push('\n');
    s.push_str(q.instructions);
    for (i, label) in q.labels.iter().enumerate() {
        s.push('\n');
        s.push_str(label);
        s.push_str(": ");
        let desc = q.descs.get(i).map(String::as_str).unwrap_or("");
        s.push_str(desc);
    }
    s
}

/// Which perturbation family a pair belongs to: `Ordering` pairs are the
/// fit/holdout population; `Casing` pairs (case flips) and `Rename` pairs
/// (option-label renames) are the cross-family checks — DIFFERENT
/// presentation-perturbation families the fitted ordering basis must NOT
/// capture (Issue 086 T2's holdout law: the true cross-family arm is a
/// different presentation change).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairFamily {
    Ordering,
    Casing,
    Rename,
}

/// A collected pair: the f64 embed difference (variant − identity
/// rendering). The runner builds these; the core consumes them.
#[derive(Debug, Clone)]
pub struct CertPair {
    pub family: PairFamily,
    pub delta: Vec<f64>,
}

/// The certificate's report for one lane × suite (serde-additive; the
/// perm-probe record carries it beside its own lane rows).
#[derive(Debug, Clone, serde::Serialize)]
pub struct BasisCertReport {
    pub lane: String,
    pub n_fit_pairs: usize,
    pub n_holdout_pairs: usize,
    pub n_whiten_samples: usize,
    /// `"sufficient_variation"` (+ `k`/`gap_ratio`), `unidentified_*`
    /// (+ the reason spelled into the string), or `"error: <message>"`.
    pub verdict: String,
    pub k: Option<usize>,
    pub gap_ratio: Option<f64>,
    /// Held-out leakage of the presentation null space (fit-half basis,
    /// holdout-half ordering pairs). `None` unless the verdict certified.
    pub pair_leakage: Option<f64>,
    /// The non-squared sibling.
    pub agreement_residual: Option<f64>,
    /// The casing-null cross-check residual (`None` when no casing pair
    /// carried a nonzero Δ — the expected posture for a case-insensitive
    /// tokenizer).
    pub casing_null_residual: Option<f64>,
    /// The label-rename cross-family residual (the ordering basis
    /// evaluated on renamed-label pairs — low = the families are distinct,
    /// high = the ordering basis generalizes to rename churn). `None` when
    /// no rename pair carried a nonzero Δ.
    pub rename_residual: Option<f64>,
    /// The shrinkage floor added to the moment diagonal
    /// (`δ = SHRINKAGE_REL · tr(M)/d`) — the numerical guard that keeps
    /// the eigensolve convergent on a (near-)rank-deficient moment and
    /// makes the rank gate read "dimension of above-floor variation".
    /// Disclosed, never silent; `None` on the refusal postures.
    pub shrinkage_eps: Option<f64>,
    pub tau_gap: f64,
}

/// The certificate over collected pairs. `whiten_pop` is the whitening
/// population (identity renderings + train docs, already embedded, f32 —
/// converted here). Deterministic in call order.
///
/// Contract: `fit_pairs`/`holdout_pairs` carry `PairFamily::Ordering`
/// deltas and the caller has already split the slots fit/holdout
/// (deterministic stride); `cross_pairs` carry the off-family checks
/// (`PairFamily::Casing` / `PairFamily::Rename`).
pub fn certify(
    fit_pairs: &[CertPair],
    holdout_pairs: &[CertPair],
    cross_pairs: &[CertPair],
    whiten_pop: &[[f32; CERT_DIM]],
) -> BasisCertReport {
    match certify_inner(fit_pairs, holdout_pairs, cross_pairs, whiten_pop) {
        Ok(r) => r,
        Err(msg) => error_report(fit_pairs.len(), holdout_pairs.len(), whiten_pop.len(), &msg),
    }
}

fn error_report(n_fit: usize, n_hold: usize, n_pop: usize, msg: &str) -> BasisCertReport {
    BasisCertReport {
        lane: String::new(),
        n_fit_pairs: n_fit,
        n_holdout_pairs: n_hold,
        n_whiten_samples: n_pop,
        verdict: format!("error: {msg}"),
        k: None,
        gap_ratio: None,
        pair_leakage: None,
        agreement_residual: None,
        casing_null_residual: None,
        rename_residual: None,
        shrinkage_eps: None,
        tau_gap: TAU_GAP,
    }
}

fn certify_inner(
    fit_pairs: &[CertPair],
    holdout_pairs: &[CertPair],
    cross_pairs: &[CertPair],
    whiten_pop: &[[f32; CERT_DIM]],
) -> Result<BasisCertReport, String> {
    let d = CERT_DIM;
    if d > MAX_DIM {
        return Err(format!("cert dim {d} over the primitive's MAX_DIM {MAX_DIM}"));
    }
    for (idx, p) in fit_pairs.iter().chain(holdout_pairs.iter()).enumerate() {
        if p.delta.len() != d {
            return Err(format!("pair[{idx}] width {} != {d}", p.delta.len()));
        }
    }
    // The designed null, checked FIRST (any n): every ordering Δ ≡ 0 (an
    // order-insensitive embed) — refuse with the honest verdict spelled.
    if fit_pairs.iter().all(|p| p.delta.iter().all(|&v| v == 0.0)) {
        return Ok(BasisCertReport {
            lane: String::new(),
            n_fit_pairs: fit_pairs.len(),
            n_holdout_pairs: holdout_pairs.len(),
            n_whiten_samples: whiten_pop.len(),
            verdict: "unidentified_zero_variation".to_string(),
            k: None,
            gap_ratio: None,
            pair_leakage: None,
            agreement_residual: None,
            casing_null_residual: None,
            rename_residual: None,
            shrinkage_eps: None,
            tau_gap: TAU_GAP,
        });
    }
    // The structural floor: a moment from fewer than d pairs is
    // rank-deficient and the rank gate degenerates (see MIN_FIT_PAIRS).
    if fit_pairs.len() < MIN_FIT_PAIRS {
        return Ok(BasisCertReport {
            lane: String::new(),
            n_fit_pairs: fit_pairs.len(),
            n_holdout_pairs: holdout_pairs.len(),
            n_whiten_samples: whiten_pop.len(),
            verdict: "unidentified_insufficient_pairs".to_string(),
            k: None,
            gap_ratio: None,
            pair_leakage: None,
            agreement_residual: None,
            casing_null_residual: None,
            rename_residual: None,
            shrinkage_eps: None,
            tau_gap: TAU_GAP,
        });
    }

    // ── whitening: build the ZCA whitener from the population Σ ──
    let mut sigma = vec![0.0f64; d * d];
    for v in whiten_pop {
        for i in 0..d {
            let xi = f64::from(v[i]);
            if xi == 0.0 {
                continue;
            }
            for j in i..d {
                let add = xi * f64::from(v[j]);
                sigma[i * d + j] += add;
                if i != j {
                    sigma[j * d + i] += add;
                }
            }
        }
    }
    let n_pop = whiten_pop.len();
    if n_pop > 0 {
        let inv = 1.0 / n_pop as f64;
        for v in sigma.iter_mut() {
            *v *= inv;
        }
    }
    let mut eigvals = vec![0.0f64; d];
    let mut eigvecs = vec![0.0f64; d * d];
    let mut whitener_out = vec![0.0f64; d * d];
    let mut sym_scratch = katgpt_core::linalg::symmetric_eig::SymmetricEigScratch::default();
    katgpt_core::basis_split::build_zca_whitener(
        &sigma,
        d,
        &mut whitener_out,
        &mut eigvals,
        &mut eigvecs,
        &mut sym_scratch,
    )
    .map_err(|e| format!("whitening: {e}"))?;

    // ── the fit-half moment, accumulated WITH the same whitener ──
    let mut moment = vec![0.0f64; d * d];
    {
        let mut acc = katgpt_core::basis_split::PairDifferenceMoment::new(&mut moment, d)
            .map_err(|e| format!("moment: {e}"))?;
        let mut w_scratch = vec![0.0f64; d];
        for (idx, p) in fit_pairs.iter().enumerate() {
            for v in &p.delta {
                if !v.is_finite() {
                    return Err(format!("fit pair[{idx}] non-finite"));
                }
            }
            acc.accumulate_whitened(&whitener_out, &p.delta, &mut w_scratch)
                .map_err(|e| format!("accumulate[{idx}]: {e}"))?;
        }
        acc.finish();
    }
    // The shrinkage floor (disclosed): δ on the diagonal before the solve.
    let trace: f64 = (0..d).map(|i| moment[i * d + i]).sum();
    let shrinkage_eps = SHRINKAGE_REL * trace / d as f64;
    if shrinkage_eps > 0.0 {
        for i in 0..d {
            moment[i * d + i] += shrinkage_eps;
        }
    }

    // ── the split: the moment is ALREADY whitened (accumulated through
    // accumulate_whitened with the same W) — the Whitener arm solves it
    // raw. TotalCovariance here would whiten AGAIN (the contract's two
    // arms are exclusive) — the double-whitened grading is what QL
    // cannot converge on.
    let mut basis = vec![0.0f64; d * d];
    let mut moment_scratch = vec![0.0f64; d * d];
    let mut jacobi_scratch = vec![0.0f64; d * d];
    // The whitener_out param is only written by the TotalCovariance arm —
    // the Whitener arm never touches it; a scratch dummy satisfies the
    // signature without aliasing the real whitener.
    let mut whitener_unused = vec![0.0f64; d * d];
    let (verdict, mut report) = split(
        &moment,
        d,
        Some(Whitening::Whitener(&whitener_out)),
        None,
        TAU_GAP,
        &mut eigvals,
        &mut eigvecs,
        &mut basis,
        &mut sym_scratch,
        &mut jacobi_scratch,
        &mut moment_scratch,
        &mut whitener_unused,
    )
    .map_err(|e| format!("split: {e}"))?;

    let mut out = BasisCertReport {
        lane: String::new(),
        n_fit_pairs: fit_pairs.len(),
        n_holdout_pairs: holdout_pairs.len(),
        n_whiten_samples: n_pop,
        verdict: match &verdict {
            katgpt_core::basis_split::Verdict::SufficientVariation { .. } => {
                "sufficient_variation".to_string()
            }
            katgpt_core::basis_split::Verdict::Unidentified(_) => "unidentified_no_gap".to_string(),
            katgpt_core::basis_split::Verdict::Unwhitened => "unwhitened".to_string(),
        },
        k: None,
        gap_ratio: None,
        pair_leakage: None,
        agreement_residual: None,
        casing_null_residual: None,
        rename_residual: None,
        shrinkage_eps: Some(shrinkage_eps),
        tau_gap: TAU_GAP,
    };
    if let katgpt_core::basis_split::Verdict::SufficientVariation { k, gap_ratio } = &verdict {
        out.k = Some(*k);
        out.gap_ratio = Some(*gap_ratio);
        // Held-out leakage over the holdout ordering pairs.
        let hold_deltas: Vec<&[f64]> =
            holdout_pairs.iter().map(|p| p.delta.as_slice()).collect();
        let mut proj_scratch = vec![0.0f64; d];
        let mut bx_scratch = vec![0.0f64; *k];
        held_out_leakage(
            &basis,
            *k,
            d,
            &hold_deltas,
            &mut proj_scratch,
            &mut bx_scratch,
            &mut report,
        )
        .map_err(|e| format!("leakage: {e}"))?;
        out.pair_leakage = report.pair_leakage;
        out.agreement_residual = report.agreement_residual;

        // ── the cross-family checks (off-family residuals, nonzero pairs only) ──
        // The fitted ordering basis evaluated on DIFFERENT presentation
        // changes (Issue 086 T2's holdout law). Each family reports its own
        // residual; all-null families report None (honest absence).
        let mut proj = vec![0.0f64; d];
        for family in [PairFamily::Casing, PairFamily::Rename] {
            let mut sq_sum = 0.0;
            let mut counted = 0usize;
            for p in cross_pairs.iter().filter(|p| p.family == family) {
                if p.delta.iter().all(|&v| v == 0.0) {
                    continue;
                }
                katgpt_core::basis_split::project_invariant_into(
                    &basis, *k, d, &p.delta, &mut bx_scratch, &mut proj,
                )
                .map_err(|e| format!("{family:?} project: {e}"))?;
                let (mut num, mut den) = (0.0, 0.0);
                for (i, pv) in proj.iter().enumerate() {
                    num += pv * pv;
                    den += p.delta[i] * p.delta[i];
                }
                if den > 0.0 {
                    sq_sum += num / den;
                    counted += 1;
                }
            }
            if counted > 0 {
                let residual = sq_sum / counted as f64;
                match family {
                    PairFamily::Casing => out.casing_null_residual = Some(residual),
                    PairFamily::Rename => out.rename_residual = Some(residual),
                    PairFamily::Ordering => {}
                }
            }
        }
    } else if let katgpt_core::basis_split::Verdict::Unidentified(
        katgpt_core::basis_split::UnidentifiedReason::GapBelowTau { k, gap_ratio, .. },
    ) = &verdict
    {
        out.k = Some(*k);
        out.gap_ratio = Some(*gap_ratio);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embed_of(text: &str) -> [f32; CERT_DIM] {
        let mut v = [0.0f32; CERT_DIM];
        crate::embed::Embedder.embed_into(text.as_bytes(), &mut v);
        v
    }

    fn render(state: &str, instructions: &str, options: &[(&str, &str)]) -> String {
        let labels: Vec<String> = options.iter().map(|(l, _)| (*l).to_string()).collect();
        let descs: Vec<String> = options.iter().map(|(_, d)| (*d).to_string()).collect();
        render_presentation(&PresentationQuestion {
            state,
            instructions,
            labels: &labels,
            descs: &descs,
        })
    }

    fn pair_from(identity: &str, variant: &str, family: PairFamily) -> CertPair {
        let a = embed_of(identity);
        let b = embed_of(variant);
        let delta: Vec<f64> = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| f64::from(*x) - f64::from(*y))
            .collect();
        CertPair { family, delta }
    }

    /// A whitening population with dense bucket coverage: pseudo-random
    /// hex tokens (deterministic LCG) hit every one of the 256 hash
    /// buckets many times — measured: structured small-vocabulary
    /// populations leave buckets uncovered and the covariance lands
    /// rank-deficient (the primitive refuses, as designed).
    pub(crate) fn whiten_pop(n: usize, salt: &str) -> Vec<[f32; CERT_DIM]> {
        let mut lcg: u64 = 0x9E37_79B9_7F4A_7C15 ^ (salt.len() as u64);
        let next = |lcg: &mut u64| {
            *lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *lcg
        };
        (0..n)
            .map(|i| {
                let words: Vec<String> = (0..16)
                    .map(|_| format!("{:016x}", next(&mut lcg)))
                    .collect();
                embed_of(&format!("{salt} doc{i} {}", words.join(" ")))
            })
            .collect()
    }

    /// A planted presentation direction: every variant rendering carries a
    /// shared marker token the identity never has — the pair differences
    /// share one dominant direction, the rank gate must certify it, and
    /// the holdout leakage must stay low (the direction generalizes to
    /// unseen tags). Scaled past [`MIN_FIT_PAIRS`] (the structural floor).
    #[test]
    fn planted_marker_direction_certifies_with_low_holdout_leakage() {
        let opts: [(&str, &str); 3] = [("alpha", "ship the release"), ("beta", "roll back"), ("gamma", "hold")];
        let mk = |marker: Option<&str>, state: String| {
            let owned: Vec<(String, String)> = opts
                .iter()
                .map(|(l, d)| {
                    let desc = match marker {
                        Some(m) => format!("{d} {m}"),
                        None => (*d).to_string(),
                    };
                    ((*l).to_string(), desc)
                })
                .collect();
            let labels: Vec<String> = owned.iter().map(|(l, _)| l.clone()).collect();
            let descs: Vec<String> = owned.iter().map(|(_, d)| d.clone()).collect();
            render_presentation(&PresentationQuestion {
                state: &state,
                instructions: "pick an action",
                labels: &labels,
                descs: &descs,
            })
        };
        // The marker's HASH POSITION varies with the tag (a different token
        // per pair) but the STRUCTURE is shared: every variant carries
        // exactly one extra token against an otherwise-identical identity.
        // 300 fit + 300 holdout pairs (the floor is 256).
        let mut fit_pairs = Vec::new();
        for i in 0..300usize {
            let identity = mk(None, format!("cluster green case {i} status nominal"));
            let variant = mk(Some(&format!("mtag{}", 10_000 + i)), format!("cluster green case {i} status nominal"));
            fit_pairs.push(pair_from(&identity, &variant, PairFamily::Ordering));
        }
        let mut holdout_pairs = Vec::new();
        for i in 0..300usize {
            let identity = mk(None, format!("cluster green holdout {i} status nominal"));
            let variant = mk(Some(&format!("mtag{}", 20_000 + i)), format!("cluster green holdout {i} status nominal"));
            holdout_pairs.push(pair_from(&identity, &variant, PairFamily::Ordering));
        }
        let pop = whiten_pop(320, "planted");
        let rep = certify(&fit_pairs, &holdout_pairs, &[], &pop);
        assert_eq!(
            rep.verdict, "sufficient_variation",
            "a planted common direction must certify, got {rep:?}"
        );
        let leakage = rep.pair_leakage.expect("certified → leakage measured");
        assert!(
            leakage < 0.9,
            "holdout leakage must stay well below 1 (the direction generalizes), got {leakage}"
        );
    }

    /// The designed null: ordering pairs whose Δ ≡ 0 (an order-insensitive
    /// embed) must refuse `unidentified_zero_variation`, never certify.
    #[test]
    fn zero_variation_pairs_refuse_unidentified() {
        let opts: [(&str, &str); 3] = [("alpha", "ship"), ("beta", "hold"), ("gamma", "rollback")];
        let mut fit = Vec::new();
        let mut hold = Vec::new();
        for i in 0..12usize {
            let state = format!("state {i}");
            let id = render(&state, "pick", &opts);
            let variant = render(&state, "pick", &opts);
            let p = pair_from(&id, &variant, PairFamily::Ordering);
            if i % 2 == 0 {
                fit.push(p);
            } else {
                hold.push(p);
            }
        }
        let pop = whiten_pop(320, "null");
        let rep = certify(&fit, &hold, &[], &pop);
        assert_eq!(
            rep.verdict, "unidentified_zero_variation",
            "zero Δ must refuse, got {rep:?}"
        );
        assert!(rep.pair_leakage.is_none(), "no basis → no leakage claim");
    }

    /// The casing cross-check: a case-flipped variant embeds IDENTICALLY
    /// under the case-insensitive tokenizer → no nonzero casing pair →
    /// `casing_null_residual` stays `None` (honest absence, never a fake
    /// 0.0). Scaled past [`MIN_FIT_PAIRS`] so the certified branch runs.
    #[test]
    fn casing_pairs_are_null_and_report_none() {
        let identity = render("s", "pick", &[("Alpha", "ship it"), ("BETA", "hold")]);
        let flipped = render("s", "pick", &[("ALPHA", "SHIP IT"), ("beta", "HOLD")]);
        let p = pair_from(&identity, &flipped, PairFamily::Casing);
        assert!(
            p.delta.iter().all(|&v| v == 0.0),
            "case-flip must embed identically (the tokenizer law)"
        );
        // Fit pairs WITH a planted direction so the certified branch runs
        // and evaluates the casing arm.
        let opts: [(&str, &str); 2] = [("alpha", "ship it"), ("beta", "hold")];
        let mk = |marker: &str, state: String| {
            let owned: Vec<(String, String)> = opts
                .iter()
                .map(|(l, d)| ((*l).to_string(), format!("{d} {marker}")))
                .collect();
            let labels: Vec<String> = owned.iter().map(|(l, _)| l.clone()).collect();
            let descs: Vec<String> = owned.iter().map(|(_, d)| d.clone()).collect();
            render_presentation(&PresentationQuestion {
                state: &state,
                instructions: "pick",
                labels: &labels,
                descs: &descs,
            })
        };
        let mut fit = Vec::new();
        for i in 0..300usize {
            let state = format!("casing case {i} status nominal");
            fit.push(pair_from(
                &mk("plain", state.clone()),
                &mk("ctag_x", state),
                PairFamily::Ordering,
            ));
        }
        let pop = whiten_pop(320, "casing");
        let rep = certify(&fit, &fit, &[p], &pop);
        assert_eq!(rep.verdict, "sufficient_variation", "got {rep:?}");
        assert_eq!(
            rep.casing_null_residual, None,
            "all-null casing arm → None"
        );
    }

    /// The rendering law: order reaches the text, and through the boundary
    /// bigrams the embed.
    #[test]
    fn rendering_carries_presented_order() {
        let a = render("st", "ins", &[("one", "d1"), ("two", "d2")]);
        let b = render("st", "ins", &[("two", "d2"), ("one", "d1")]);
        assert_ne!(a, b, "presented order must reach the text");
        assert!(a.contains("one: d1\ntwo: d2"));
        assert!(b.contains("two: d2\none: d1"));
        let ea = embed_of(&a);
        let eb = embed_of(&b);
        assert!(
            ea.iter().zip(eb.iter()).any(|(x, y)| x != y),
            "boundary bigrams must move the embed (or the certificate is vacuous)"
        );
    }

    /// The T2 cross-family law: the rename arm carries NONZERO deltas (a
    /// real token change — unlike the casing arm's null) and the ordering
    /// basis must NOT capture it wholesale (the families are distinct;
    /// the residual is reported, its size is the finding).
    #[test]
    fn rename_pairs_carry_nonzero_delta_and_report() {
        let identity = render("s", "pick", &[("alpha", "ship"), ("beta", "hold")]);
        let renamed = render("s", "pick", &[("alpha_2", "ship"), ("beta_2", "hold")]);
        let p = pair_from(&identity, &renamed, PairFamily::Rename);
        assert!(
            p.delta.iter().any(|&v| v != 0.0),
            "a rename must move the embed (new tokens)"
        );
        // Fit pairs with a planted direction so the certified branch runs.
        let opts: [(&str, &str); 2] = [("alpha", "ship"), ("beta", "hold")];
        let mk = |marker: &str, state: String| {
            let owned: Vec<(String, String)> = opts
                .iter()
                .map(|(l, d)| ((*l).to_string(), format!("{d} {marker}")))
                .collect();
            let labels: Vec<String> = owned.iter().map(|(l, _)| l.clone()).collect();
            let descs: Vec<String> = owned.iter().map(|(_, d)| d.clone()).collect();
            render_presentation(&PresentationQuestion {
                state: &state,
                instructions: "pick",
                labels: &labels,
                descs: &descs,
            })
        };
        let mut fit = Vec::new();
        for i in 0..300usize {
            let state = format!("rename case {i} status nominal");
            fit.push(pair_from(
                &mk("plain", state.clone()),
                &mk("rtag_x", state),
                PairFamily::Ordering,
            ));
        }
        let pop = whiten_pop(320, "rename");
        let rep = certify(&fit, &fit, &[p], &pop);
        assert_eq!(rep.verdict, "sufficient_variation", "got {rep:?}");
        let residual = rep
            .rename_residual
            .expect("a nonzero rename pair must report a residual");
        assert!(
            (0.0..=1.0).contains(&residual),
            "residual is a fraction of ‖Δ‖², got {residual}"
        );
    }
}
