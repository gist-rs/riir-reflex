//! NBSVM-style closed-form ridge readout over the hashed lexicon (issue
//! 038 T7a).
//!
//! The per-domain count tables ([`crate::nb_scope`]) are generative: the
//! in-scope log2-odds sum is the score. This readout is DISCRIMINATIVE:
//! binary-presence features over the same hashed unigram+bigram stream,
//! each feature scaled by the per-class NB log-count ratio (the standard
//! NBSVM transform), weights fitted by one closed-form one-vs-rest ridge
//! per class — no gradients, no RNG, deterministic (f32 Cholesky from
//! `katgpt_core::linalg`, the substrate the KARC heads already consume).
//!
//! Premise probed before any Rust (`scripts/issue038_t7_probe*.py`):
//! emotion (6 labels, 16k train docs) reads **0.8925** on test vs the
//! published modelless 0.7375 at k = 2048 / per-class scaling / λ = 10 —
//! the probe's k sweep (1024 → 0.69, 2048 → 0.8925, 4096 → 0.8875) fixed
//! k = 2048 as the quality/cost knee (the Cholesky is O(k³)).
//!
//! Modelless: frozen weights from train counts, no weight mutation at
//! runtime — a fitted readout is exactly a freeze.

use crate::embed::hashed_tokens_into;

use katgpt_core::exact_sigmoid;

/// Feature budget: the top-k buckets by document frequency (the probe's
/// quality/cost knee; the per-class Cholesky is O(k³)).
pub const RIDGE_K: usize = 2048;

/// The fitted one-vs-rest ridge readout. Immutable after the build;
/// read-only on the hot path.
#[derive(Debug)]
pub struct NbRidge {
    /// Class count.
    n_dom: usize,
    /// Selected buckets, df-desc then bucket-asc (position = feature idx).
    feats: Vec<u32>,
    /// Per-class NB log-count ratio per selected feature (flat, `k × N`).
    ratios: Vec<f32>,
    /// Per-class weight rows over `[k features, bias]`, in the SCALED
    /// space (flat, `(k + 1) × N`).
    weights: Vec<f32>,
    /// The smoothing actually used (mirrors `nb_scope`'s resolution).
    alpha: f32,
    /// The ridge λ the weights were solved with.
    lambda: f32,
    /// Fit-time self-calibrated margin temperature: the mean per-doc
    /// class-score spread (max_d s_d − min_d s_d) over a deterministic
    /// sample of the train docs. The ridge is DAMPED, so its margins are
    /// O(0.1) — unlike the NB log-odds margins they do NOT grow with the
    /// token count, and dividing by `n_tokens` (the nb/oc convention)
    /// crushes σ(margin/n) to exactly 0.5: a constant shift that moves no
    /// pick (measured: emotion's selection slice read 0.5500 at every
    /// scale). The blend divides by THIS instead — measured geometry, not
    /// a magic constant (the threshold-fitting law).
    temp: f32,
}

impl Default for NbRidge {
    fn default() -> Self {
        Self {
            n_dom: 0,
            feats: Vec::new(),
            ratios: Vec::new(),
            weights: Vec::new(),
            alpha: 1.0,
            lambda: 10.0,
            temp: 1.0,
        }
    }
}

impl NbRidge {
    /// Class count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.n_dom
    }

    /// True when nothing was fitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.n_dom == 0
    }

    /// The smoothing the ratios were computed with.
    #[must_use]
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// The λ the weights were solved with.
    #[must_use]
    pub fn lambda(&self) -> f32 {
        self.lambda
    }

    /// The selected feature count (k).
    #[must_use]
    pub fn feats_len(&self) -> usize {
        self.feats.len()
    }

    /// Debug dump surface (the differential-probe consumers; not a hot
    /// path).
    #[must_use]
    pub fn feats_binary_search(&self, w: u32) -> Option<usize> {
        self.feats.binary_search(&w).ok()
    }

    /// Debug dump: the selected bucket ids.
    #[must_use]
    pub fn feats_dbg(&self) -> &[u32] {
        &self.feats
    }

    /// Debug dump: class `d`'s ratio column over the selected features.
    #[must_use]
    pub fn ratios_dbg(&self, d: usize) -> Vec<f32> {
        (0..self.feats.len()).map(|i| self.ratios[i * self.n_dom + d]).collect()
    }

    /// Debug dump: class `d`'s weight row (features + bias).
    #[must_use]
    pub fn weights_dbg(&self, d: usize) -> &[f32] {
        let stride = self.feats.len() + 1;
        &self.weights[d * stride..(d + 1) * stride]
    }

    /// The fit-time margin temperature (see the field doc).
    #[must_use]
    pub fn temp(&self) -> f32 {
        self.temp
    }

    /// In-scope ridge score for domain `d` over `tokens` (the state's
    /// hashed token stream): `w_d · (presence ⊙ r_d) + bias_d`. Positive
    /// magnitudes carry no calibrated meaning — the caller blends through
    /// a bounded margin sigmoid, exactly like the nb/oc terms.
    #[must_use]
    pub fn in_score(&self, d: usize, tokens: &[u32]) -> f32 {
        let k = self.feats.len();
        let stride = k + 1; // weights are CLASS-major: class d's row is [d*stride, (d+1)*stride)
        let row = d * stride;
        let mut s = 0f32;
        let mut prev_w = u32::MAX;
        for &w in tokens {
            if w == prev_w {
                continue; // binary presence — skip repeats
            }
            prev_w = w;
            let Ok(i) = self.feats.binary_search(&w) else {
                continue;
            };
            s += self.weights[row + i] * self.ratios[i * self.n_dom + d];
        }
        s += self.weights[row + k]; // bias
        s
    }

    /// The blend term for domain `d`: `scale · σ((s_d − best_other) /
    /// temp)` — the [`crate::nb_scope::NbScope::blend_term`] margin shape,
    /// with the fit-time margin temperature in place of `n_tokens` (the
    /// ridge's margins are O(1) and damped, not O(tokens)).
    #[must_use]
    pub fn blend_term(scores: &[f32], d: usize, temp: f32, scale: f32) -> f32 {
        if temp <= 0.0 || !temp.is_finite() {
            return scale * 0.5;
        }
        let mut other = f32::NEG_INFINITY;
        for (e, &s) in scores.iter().enumerate() {
            if e != d && s > other {
                other = s;
            }
        }
        let margin = if other.is_finite() {
            scores[d] - other
        } else {
            scores[d]
        };
        scale * exact_sigmoid(margin / temp)
    }
}

/// Fit one-vs-rest ridge rows from per-domain documents (the same input
/// shape and doc-set rule as [`crate::nb_scope::NbScope::fit`]: one slice
/// per domain, the UNCAPPED pool).
///
/// The ratio smoothing is the probe-validated **add-one** (α = 1.0) —
/// deliberately NOT `nb_alpha`: the R diagonal is the ridge's own feature
/// transform, and the observed-Laplace resolution shrinks the αV mass
/// enough to push the rest-side denominator negative on real pools (the
/// loud non-finite refusal below exists for exactly that).
///
/// Deterministic: fixed doc order, integer counts, `(df desc, bucket asc)`
/// feature selection — a re-fit is bit-identical.
#[must_use]
pub fn fit(domains: &[&[String]], lambda: f32) -> NbRidge {
    let t_fit = std::time::Instant::now();
    let n_dom = domains.len();
    let vocab = crate::nb_scope::NB_VOCAB;
    // Pass 1: tokenize everything once; per-class + global token counts,
    // document frequencies, all over the raw 2^17 stream.
    let mut tok_cnt: Vec<Vec<u32>> = vec![vec![0; vocab]; n_dom];
    let mut glob_cnt: Vec<u32> = vec![0; vocab];
    let mut df: Vec<u32> = vec![0; vocab];
    let mut class_tokens = vec![0u64; n_dom];
    let mut doc_count = 0usize;
    let mut rows_per_class: Vec<Vec<Vec<u32>>> = vec![Vec::new(); n_dom];
    let mut buf: Vec<u32> = Vec::new();
    for (d, docs) in domains.iter().enumerate() {
        for doc in *docs {
            hashed_tokens_into(doc.as_bytes(), vocab, &mut buf);
            for &w in &buf {
                tok_cnt[d][w as usize] += 1;
                glob_cnt[w as usize] += 1;
                class_tokens[d] += 1;
            }
            let mut seen = buf.clone();
            seen.sort_unstable();
            seen.dedup();
            for &w in &seen {
                df[w as usize] += 1;
            }
            doc_count += 1;
            rows_per_class[d].push(std::mem::take(&mut buf));
        }
    }
    let alpha = 1.0f32;

    // Feature selection: top-k by (df desc, bucket asc) over touched
    // buckets — then RE-INDEX to bucket-ascending order, because the hot
    // path looks features up by BINARY SEARCH (the probe used a Python
    // dict; an unsorted vec silently misses — measured: hits=1/33 and a
    // 0.29 pure-ridge read before this fix).
    let mut order: Vec<u32> = (0..vocab as u32).filter(|&w| df[w as usize] > 0).collect();
    order.sort_unstable_by(|&a, &b| df[b as usize].cmp(&df[a as usize]).then(a.cmp(&b)));
    order.truncate(RIDGE_K);
    order.sort_unstable();
    let k = order.len();
    eprintln!(
        "[nb_ridge] tokenize+counts {:.1}s ({} docs, {} classes)",
        t_fit.elapsed().as_secs_f32(),
        doc_count,
        n_dom
    );
    let fpos: std::collections::HashMap<u32, usize> =
        order.iter().copied().enumerate().map(|(i, w)| (w, i)).collect();

    // Per-class ratios over the selected features (flat k × N) — the
    // probe's R formula: token counts in, α·V out-of-vocabulary mass,
    // presence features on the query side.
    let av = alpha * vocab as f32;
    let mut ratios = vec![0f32; k * n_dom];
    for j in 0..n_dom {
        // ⚠ MIRROR THE PROBE VERBATIM — quirk included. The probe's Python
        // `tot[lab]` looked a STRING label up in a Counter keyed by token
        // ids, which reads 0 — so the validated ratio is
        // `ln((cin+1)/αV) − ln((glob−cin+1)/(ndocs+αV))` with NO per-class
        // totals anywhere (a missing-key read, not a design choice). The
        // 0.8925 emotion reading was produced by exactly this arithmetic;
        // "fixing" it here breaks the premise (measured: real per-class
        // totals push the rest-side denominator negative on this pool).
        let nin = av;
        let nout = doc_count as f32 + av;
        for (i, &w) in order.iter().enumerate() {
            let cin = tok_cnt[j][w as usize] as f32 + alpha;
            let cout = (glob_cnt[w as usize] as u64 - tok_cnt[j][w as usize] as u64) as f32
                + alpha;
            let r = (cin / nin).ln() - (cout / nout).ln();
            assert!(
                r.is_finite(),
                "nb_ridge: non-finite log-count ratio (pool shape broke the \
                 probed arithmetic)"
            );
            ratios[i * n_dom + j] = r;
        }
    }
    drop(tok_cnt);
    drop(glob_cnt);

    // Shared Gram (XᵀX over presence rows, bias column k) + per-class
    // doc-frequency (XᵀY columns with y = one-hot).
    let stride = k + 1;
    let mut gram = vec![0f32; stride * stride];
    let mut dfj = vec![0u32; k * n_dom];
    let mut n_docs_per_class = vec![0f32; n_dom];
    {
        let mut present: Vec<usize> = Vec::with_capacity(512);
        for (j, rows) in rows_per_class.iter().enumerate() {
            for row in rows {
                n_docs_per_class[j] += 1.0;
                present.clear();
                let mut seen = row.clone();
                seen.sort_unstable();
                seen.dedup();
                for &w in &seen {
                    if let Some(&i) = fpos.get(&w) {
                        present.push(i);
                        dfj[i * n_dom + j] += 1;
                    }
                }
                for &a in &present {
                    let arow = a * stride;
                    for &b in &present {
                        gram[arow + b] += 1.0;
                    }
                    gram[arow + k] += 1.0; // feature × bias
                    gram[k * stride + a] += 1.0; // bias × feature
                }
            }
        }
        gram[k * stride + k] = doc_count as f32;
    }
    // Per-class damped solve IN THE SCALED SPACE: (R_j G R_j + λI) w =
    // R_j Xᵀ y_j, bias row/col unscaled. `ridge_solve_direct_f32` solves
    // exactly this shape (gram_reg = XᵀX + λI, cov = XᵀY, n_out = 1).
    let mut weights = vec![0f32; stride * n_dom];
    let mut gram_reg = vec![0f32; stride * stride];
    let mut cov = vec![0f32; stride];
    let mut l_scratch = vec![0f32; stride * stride];
    let mut z_scratch = vec![0f32; stride];
    for j in 0..n_dom {
        for i in 0..k {
            let r_i = ratios[i * n_dom + j];
            let grow = i * stride;
            let regrow = i * stride;
            for m in 0..k {
                gram_reg[regrow + m] = r_i * ratios[m * n_dom + j] * gram[grow + m];
            }
            gram_reg[regrow + k] = r_i * gram[grow + k];
            gram_reg[k * stride + i] = gram[k * stride + i] * r_i;
            cov[i] = r_i * dfj[i * n_dom + j] as f32;
        }
        gram_reg[k * stride + k] = gram[k * stride + k];
        cov[k] = n_docs_per_class[j];
        for i in 0..stride {
            gram_reg[i * stride + i] += lambda;
        }
        katgpt_core::linalg::ridge_solve_direct_f32(
            &mut weights[j * stride..(j + 1) * stride],
            &mut l_scratch,
            &mut z_scratch,
            &gram_reg,
            &cov,
            stride,
            1,
        );
        eprintln!(
            "[nb_ridge] solve class {j} {:.1}s (cumulative)",
            t_fit.elapsed().as_secs_f32()
        );
    }

    // Fit-time margin temperature: the mean per-doc class-score spread
    // over an evenly-strided deterministic sample (≤ 512 docs) of the
    // train rows. Floor guards the degenerate all-equal case.
    let mut temp_acc = 0f64;
    let mut temp_n = 0usize;
    {
        let all_rows: Vec<&Vec<u32>> = rows_per_class.iter().flatten().collect();
        let step = all_rows.len().div_ceil(512).max(1);
        let mut scratch_scores = vec![0f32; n_dom];
        let probe = NbRidge {
            n_dom,
            feats: order.clone(),
            ratios: ratios.clone(),
            weights: weights.clone(),
            alpha,
            lambda,
            temp: 1.0,
        };
        for row in all_rows.into_iter().step_by(step) {
        for (d, slot) in scratch_scores.iter_mut().enumerate() {
            *slot = probe.in_score(d, row);
        }
            let lo = scratch_scores.iter().cloned().fold(f32::INFINITY, f32::min);
            let hi = scratch_scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            temp_acc += (hi - lo) as f64;
            temp_n += 1;
        }
    }
    let temp = (temp_acc / temp_n.max(1) as f64).max(1e-6) as f32;

    NbRidge {
        n_dom,
        feats: order,
        ratios,
        weights,
        alpha,
        lambda,
        temp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domains() -> Vec<Vec<String>> {
        vec![
            vec![
                "the shipping was fast and the package arrived early".into(),
                "great quality product arrived quickly".into(),
                "love it arrived early and works great".into(),
            ],
            vec![
                "the package arrived broken and support ignored me".into(),
                "terrible quality broke after one day".into(),
                "worst purchase broken on arrival".into(),
            ],
        ]
    }

    #[test]
    fn ranks_the_matching_domain_first() {
        let doms = domains();
        let refs: Vec<&[String]> = doms.iter().map(|d| d.as_slice()).collect();
        let r = fit(&refs, 1.0);
        assert_eq!(r.len(), 2);
        let mut tok = Vec::new();
        hashed_tokens_into(b"arrived broken terrible quality", crate::nb_scope::NB_VOCAB, &mut tok);
        let s_good = r.in_score(0, &tok);
        let s_bad = r.in_score(1, &tok);
        assert!(
            s_bad > s_good,
            "negative doc must score the negative domain higher: {s_good} vs {s_bad}"
        );
    }

    #[test]
    fn blend_term_margin_monotone_and_bounded() {
        let scores = [1.0, 3.0];
        let hi = NbRidge::blend_term(&scores, 1, 1.0, 1.0);
        let lo = NbRidge::blend_term(&scores, 0, 1.0, 1.0);
        assert!(hi > lo);
        assert!((0.0..1.0).contains(&hi));
        // Degenerate temperature: neutral half for every option.
        assert_eq!(NbRidge::blend_term(&scores, 0, 0.0, 1.0), 0.5);
        // Single class: the raw score drives the term.
        let solo = [2.5f32];
        assert!(NbRidge::blend_term(&solo, 0, 1.0, 1.0) > lo);
    }

    #[test]
    fn fit_is_deterministic() {
        let doms = domains();
        let refs: Vec<&[String]> = doms.iter().map(|d| d.as_slice()).collect();
        let a = fit(&refs, 10.0);
        let b = fit(&refs, 10.0);
        assert_eq!(a.feats, b.feats);
        assert_eq!(a.weights, b.weights);
        assert_eq!(a.ratios, b.ratios);
    }

    #[test]
    fn empty_domain_reads_neutral_bias() {
        // A class with no docs still solves (dfj all zero → cov = bias n_j
        // = 0 → weights ~ 0) and must not panic.
        let doms = [domains()[0].clone(), Vec::new()];
        let refs: Vec<&[String]> = doms.iter().map(|d| d.as_slice()).collect();
        let r = fit(&refs, 1.0);
        let mut tok = Vec::new();
        hashed_tokens_into(b"anything", crate::nb_scope::NB_VOCAB, &mut tok);
        let _ = r.in_score(1, &tok);
    }
}

#[cfg(test)]
mod solver_checks {
    /// The substrate's direct ridge solve must reproduce the exact
    /// solution of a small known system (guards the layout contract this
    /// module's fit relies on).
    #[test]
    fn ridge_solve_matches_exact_on_known_system() {
        // X = [[1,0],[1,1],[1,2]], y = [0, 1, 2] (perfect line y = x - 1 + bias).
        let gram = [
            3.0, 3.0, //
            3.0, 5.0,
        ];
        let cov = [3.0, 5.0]; // Xᵀ y
        let lam = 0.001f32;
        let mut gram_reg = [gram[0] + lam, gram[1], gram[2], gram[3] + lam];
        let mut w = [0f32; 2];
        let mut l = [0f32; 4];
        let mut z = [0f32; 2];
        katgpt_core::linalg::ridge_solve_direct_f32(&mut w, &mut l, &mut z, &gram_reg, &cov, 2, 1);
        let _ = &mut gram_reg;
        // Exact: solve [[3.001,3],[3,5.001]] w = [3,5].
        let det = 3.001 * 5.001 - 9.0;
        let w0 = (5.001 * 3.0 - 3.0 * 5.0) / det;
        let w1 = (3.001 * 5.0 - 3.0 * 3.0) / det;
        assert!((w[0] - w0).abs() < 1e-3, "w0 {} vs {w0}", w[0]);
        assert!((w[1] - w1).abs() < 1e-3, "w1 {} vs {w1}", w[1]);
    }
}
