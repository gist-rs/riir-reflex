//! Issue 090 C1 — the rubric-covariance + route-agreement probe (the pure
//! core).
//!
//! The Jeva paper's `score` readout is an expected value over the option
//! distribution, `S(v) = Σ q_i·v_i` — the consumer-side algebra every
//! rubric-scoring client of `decision_wire` performs over our returned
//! `probabilities`. This module pins the ALGEBRA as measurable identities
//! (the issue's own wording, "to fp tolerance"):
//!
//! - **I1 (scale covariance).** `S(10·v) == 10·S(v)` — scaling the rubric
//!   values scales the score. Exact for any fixed read vector by
//!   linearity; a violation means the readout path is non-linear in v
//!   (a value-weighted or clamped readout smuggled into the read).
//! - **I2 (shift covariance).** `S(v + c·1) == S(v) + c·Σq_i` — shifting
//!   every value by c shifts the score by c times the EXPOSED mass. The
//!   identity pins the distribution's completeness: a truncated or
//!   re-normalized-on-the-wire vector breaks it (Σq on the wire is not
//!   the Σq the consumer derives).
//! - **I3 (value-blind argmax).** The shipped pick is the argmax of the
//!   shipped distribution (ties → first, the determinism law's tie rule).
//!   Values never ride our wire, so the pick is value-blind BY
//!   CONSTRUCTION — the identity pins it stays that way: the outcome and
//!   the probabilities must agree WITHIN each surface (the engine's f32
//!   answer; the serve edge's wire rendering), catching a near-tie
//!   rounding collision where the engine picks index 1 of a pair that
//!   rounds to a visible tie (a consumer computing argmax from the wire
//!   would pick index 0). Surfaces are SELF-consistent by construction of
//!   the probe — never cross-compared (the wire renders its own request
//!   bytes; cross-surface pick equality is not an identity).
//! - **I4 (normalization residual, measured).** `|Σq − 1|` per surface:
//!   the engine's raw f32 sum (the L1 normalizer's own residue), the
//!   wire-normalized sum (exact to f64), and the wire ROUNDED sum (the
//!   round6 residue, bounded by n·5e-7). Reported, never gated beyond the
//!   TypeSafe dialect's ±1e-3 budget (the contract the serve edge states).
//!
//! Everything here is PURE: the value families, the transforms, the dot
//! product, the per-slot record math, and the verdict. The lane, the
//! posture, and the wire rendering live in
//! [`super::runner::rubric_probe`].
//!
//! Route-agreement half (C1's second clause): the per-slot agreement record
//! between the CONTEXT route (what the engine routed to: state+prompt+
//! criteria) and each OPTION's semantic route (`pick_domain` over the
//! option text embedded alone) — plus the by-name resolution column (an
//! option string that names a domain resolves to it BY NAME when armed;
//! the probe discloses where the name-resolved and semantic routes
//! disagree). Characterization only: no pre-registered bar exists, and the
//! route geometry deliberately differs between the two surfaces (corpus
//! docs are embedded ALONE — the apples-to-apples note in the engine's
//! route-terms comment).

use serde::Serialize;

/// The fp tolerance for I1/I2: relative, per-slot, generous enough for the
/// f32→f64 widening and the round6 wire residue, tight enough that a
/// non-linear or truncating readout reds.
pub const IDENTITY_TOL: f64 = 1e-9;

/// The TypeSafe dialect's stated probability-sum budget (systemone's wire
/// contract: "probabilities sum ≈ 1 ± 1e-3").
pub const WIRE_SUM_BUDGET: f64 = 1e-3;

/// The shift constant (I2) — arbitrary, fixed, disclosed in the meta.
pub const SHIFT_C: f64 = 7.0;

/// The scale constant (I1).
pub const SCALE: f64 = 10.0;

/// The golden-ratio constant for the scattered value family (irrational
/// multiples exercise non-uniform spacing without semantic meaning).
const PHI: f64 = std::f64::consts::GOLDEN_RATIO;

/// Value family A: the unit ramp `v_i = (i+1)` — the natural rubric of a
/// score question read as 1-based levels.
pub fn ramp_v(k: usize) -> Vec<f64> {
    (1..=k).map(|i| i as f64).collect()
}

/// Value family B: golden-ratio scattered — `v_i = frac((i+1)·φ)·100`,
/// deterministic irrational spacing (no two values share a scale relation
/// the identities could hide behind).
pub fn phi_scatter_v(k: usize) -> Vec<f64> {
    (1..=k)
        .map(|i| ((i as f64) * PHI).fract() * 100.0)
        .collect()
}

/// The fixed-order dot product `Σ q_i·v_i` (sequential fold — the
/// determinism law; never a parallel/SIMD reorder).
pub fn s_dot(q: &[f64], v: &[f64]) -> f64 {
    let mut s = 0.0f64;
    for (&qi, &vi) in q.iter().zip(v.iter()) {
        s += qi * vi;
    }
    s
}

/// Round to 6 decimals — the wire's own rendering (systemone::round6);
/// duplicated here so the pure core measures the same surface the serve
/// edge emits.
pub fn round6(v: f64) -> f64 {
    (v.max(0.0) * 1e6).round() / 1e6
}

/// f64 argmax with the determinism law's tie rule (ties → FIRST).
pub fn argmax_first(q: &[f64]) -> usize {
    let mut best = 0usize;
    let mut bp = f64::MIN;
    for (i, &p) in q.iter().enumerate() {
        if i == 0 || p > bp {
            best = i;
            bp = p;
        }
    }
    best
}

/// One surface's identity readout (engine raw / engine normalized / wire
/// rounded — the runner half instantiates one per surface).
#[derive(Debug, Clone, Serialize)]
pub struct SurfaceIdentities {
    /// Which surface: "engine-raw" (f32 widened), "engine-norm"
    /// (wire_distribution's exact f64 L1), or "wire-round6".
    pub surface: &'static str,
    pub n: usize,
    /// I1 residual: |S(10v) − 10·S(v)| max over the value families.
    pub scale_residual: f64,
    /// I2 residual: |S(v+c) − S(v) − c·Σq| max over the value families.
    pub shift_residual: f64,
    /// |Σq − 1| (the normalization residual, max-abs over slots).
    pub sum_residual: f64,
    /// I3: the shipped pick vs argmax(this surface's distribution,
    /// ties→first) — disagreements counted per surface.
    pub argmax_disagreements: usize,
}

impl SurfaceIdentities {
    /// The structural verdict for this surface: I1/I2 within tolerance
    /// AND no argmax disagreement. The sum residual is NOT structural
    /// here (measured against [`WIRE_SUM_BUDGET`] by the caller).
    pub fn structural_ok(&self) -> bool {
        self.scale_residual <= IDENTITY_TOL
            && self.shift_residual <= IDENTITY_TOL
            && self.argmax_disagreements == 0
    }
}

/// The two value families, one pass: max residuals over both. The surface
/// label names WHICH distribution was fed (engine raw / engine normalized
/// / wire rounded) — the runner half instantiates one per surface.
pub fn identity_residuals(surface: &'static str, q: &[f64], pick: usize) -> SurfaceIdentities {
    let families = [ramp_v(q.len()), phi_scatter_v(q.len())];
    let mut scale_res = 0.0f64;
    let mut shift_res = 0.0f64;
    let sum: f64 = q.iter().sum();
    for v in &families {
        let s = s_dot(q, v);
        let scaled: Vec<f64> = v.iter().map(|x| x * SCALE).collect();
        let shifted: Vec<f64> = v.iter().map(|x| x + SHIFT_C).collect();
        scale_res = scale_res.max((s_dot(q, &scaled) - SCALE * s).abs());
        shift_res = shift_res.max((s_dot(q, &shifted) - s - SHIFT_C * sum).abs());
    }
    SurfaceIdentities {
        surface,
        n: q.len(),
        scale_residual: scale_res,
        shift_residual: shift_res,
        sum_residual: (sum - 1.0).abs(),
        argmax_disagreements: usize::from(argmax_first(q) != pick),
    }
}

/// The route-agreement record for one question slot (C1's second clause).
#[derive(Debug, Clone, Serialize)]
pub struct RouteAgreement {
    /// The engine's CONTEXT route index (state+prompt+criteria →
    /// pick_domain) — `sc.domains[qi]` after a decide.
    pub context_route: usize,
    /// The STATE-ALONE route index (state embedded alone — the route-terms
    /// surface; discloses how far the prompt/criteria move the route).
    pub state_route: usize,
    /// Per-option semantic route indices (option text embedded alone).
    pub option_routes: Vec<usize>,
    /// Per-option by-name resolution: `Some(di)` where the option string
    /// equals a domain name (None = not a domain name).
    pub by_name: Vec<Option<usize>>,
    /// option_routes entries equal to context_route.
    pub option_context_agree: usize,
    /// by_name-resolved entries whose NAME route equals their SEMANTIC
    /// route (the two resolutions of the same option agreeing).
    pub name_semantic_agree: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn ramp_and_phi_families_are_deterministic_and_distinct() {
        let r = ramp_v(5);
        let p = phi_scatter_v(5);
        assert_eq!(r, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(p, phi_scatter_v(5));
        assert_ne!(r, p);
        let mut sorted = p.clone();
        sorted.sort_by(f64::total_cmp);
        for w in sorted.windows(2) {
            assert_ne!(w[0], w[1], "phi family must not repeat values");
        }
    }

    #[test]
    fn identities_hold_on_a_clean_distribution() {
        let q = vec![0.1, 0.2, 0.05, 0.6, 0.05];
        let pick = argmax_first(&q);
        assert_eq!(pick, 3);
        let rec = identity_residuals("engine-raw", &q, pick);
        assert!(rec.scale_residual <= IDENTITY_TOL);
        assert!(rec.shift_residual <= IDENTITY_TOL);
        assert!(rec.structural_ok());
        assert!(rec.sum_residual < 1e-12);
    }

    #[test]
    fn scale_identity_reds_a_clamped_readout() {
        // A clamped readout (min(v, 5)) breaks I1 once scaled past the
        // clamp — the class of value-dependent read the identity exists
        // to catch.
        let q = vec![0.25, 0.75];
        let v = ramp_v(2); // [1, 2]
        let s = s_dot(&q, &v);
        let scaled: Vec<f64> = v.iter().map(|x| (x * SCALE).min(5.0)).collect();
        let s_scaled_clamped = s_dot(&q, &scaled);
        assert!(!close(s_scaled_clamped, SCALE * s, IDENTITY_TOL));
    }

    #[test]
    fn shift_identity_exposes_dropped_mass() {
        // A TRUNCATED distribution is self-consistent under shift, but its
        // Σq disagrees with the wire's claimed mass — the cross-surface
        // difference IS the finding I2/I4 surface.
        let q = [0.2, 0.3, 0.1, 0.25, 0.15];
        let truncated = &q[..4];
        let v = ramp_v(5);
        let shifted: Vec<f64> = v.iter().map(|x| x + SHIFT_C).collect();
        let s_trunc = s_dot(truncated, &v);
        let s_trunc_shifted = s_dot(truncated, &shifted);
        let sum_trunc: f64 = truncated.iter().sum();
        let residual = (s_trunc_shifted - s_trunc - SHIFT_C * sum_trunc).abs();
        assert!(residual <= IDENTITY_TOL, "self-consistent within itself");
        assert!((sum_trunc - 1.0).abs() > 0.14, "dropped mass is the signal");
    }

    #[test]
    fn argmax_ties_break_first() {
        assert_eq!(argmax_first(&[0.3, 0.4, 0.3]), 1);
        assert_eq!(argmax_first(&[0.5, 0.5]), 0);
        assert_eq!(argmax_first(&[0.2, 0.5, 0.5]), 1);
    }

    #[test]
    fn round6_near_tie_collision_is_detectable() {
        // The engine picks index 1 of a near-tie; both round to the same
        // wire value → a wire-side argmax computes index 0 — the I3
        // collision the probe counts.
        let a = 0.499_999_5;
        let b = 0.500_000_4;
        let q = vec![a, b];
        let wire: Vec<f64> = q.iter().map(|x| round6(*x)).collect();
        assert_eq!(wire[0], wire[1], "near-tie rounds to a visible tie");
        assert_eq!(argmax_first(&q), 1, "engine picked the second");
        assert_eq!(argmax_first(&wire), 0, "wire consumer picks the first");
        let rec = identity_residuals("wire-round6", &wire, 1);
        assert_eq!(rec.argmax_disagreements, 1);
        assert!(!rec.structural_ok());
    }

    #[test]
    fn wire_budget_law() {
        // A round6-normalized 8-arity distribution's sum residual stays
        // far inside the ±1e-3 wire budget.
        let q = [0.125; 8];
        let wire: Vec<f64> = q.iter().map(|x| round6(*x)).collect();
        let rec = identity_residuals("wire-round6", &wire, 0);
        assert!(rec.sum_residual <= WIRE_SUM_BUDGET);
    }
}
