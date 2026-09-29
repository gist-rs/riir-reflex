//! The input-perturbation ensemble wrapper (Issue 055 / Plan 008).
//!
//! A seeded Monte-Carlo bag over the deterministic engine: sample 0 is the
//! UNPERTURBED legacy run (its bytes are the served answer — the wrapper is
//! an analysis/risk surface, never a silent re-route), and samples `1..n`
//! re-run the pipeline under Bernoulli bucket-dropout of the embeddings
//! (katgpt-core `perturbation_ensemble::bucket_dropout_into`). Per question,
//! the samples accumulate into an [`EnsembleHistogram`] (pick counts +
//! per-option score moments) and the DRM decision rules read off it:
//!
//! - **U_pair / U_BoN** — uncertainty-aware rejection keys (rank decisions
//!   by disagreement, reject the most uncertain first);
//! - **LCB-λ** — risk-sensitive option ranking `μ − λσ` over the per-option
//!   score moments;
//! - **instability** — the sigmoid-projected third signal a fused gate can
//!   consume beside score + corpus distance.
//!
//! # Determinism contract (G1, three-part — pre-registered in the issue)
//!
//! - **Unarmed (N=1) == legacy bytes exactly**: [`McConfig::disabled`] (or
//!   simply not calling this wrapper) changes nothing; with `n_samples == 1`
//!   the wrapper runs exactly one unperturbed sample and the histogram is a
//!   point mass (u_pair = 0) — the frozen-pick parity gates are untouched.
//! - **Armed same-seed == armed same-seed (byte-identity)**: the per-request
//!   seed is `BLAKE3(state ‖ per-question prompt bytes ‖ salt)` and every
//!   sample draw derives from it deterministically — two runs at the same
//!   config produce bit-identical histograms.
//! - **Cross-seed AGGREGATE stability is a separate assertion class** with
//!   its own tolerance (the harness owns it) — never conflated with the
//!   per-seed determinism pin.
//!
//! # Honest limits (recorded, not hidden)
//!
//! - The byte-level drafter delta is unaffected by embedding perturbation —
//!   a `drafter_only` question has ZERO sample variance by construction and
//!   reads as a point-mass histogram. The non-collapse floor must use
//!   route-active fixtures.
//! - The wrapper leaves `sc` holding the LEGACY (sample-0) state on return
//!   (samples `1..n` run FIRST, the unperturbed run LAST) so a caller can
//!   `to_response` immediately after and get the unarmed answer.

use crate::engine::{DecisionEngine, Scratch};
use katgpt_core::perturbation_ensemble::EnsembleHistogram;

/// The MC wrapper's config. Default is DISABLED (`n_samples: 1`, `p_drop:
/// 0.0`) — the byte-identical posture; arming is explicit, per the
/// `nb_scale`/`oc_scale` knob discipline.
#[derive(Clone, Copy, Debug)]
pub struct McConfig {
    /// Total samples INCLUDING the unperturbed legacy run. `1` = disabled
    /// (one legacy run, histogram is a point mass).
    pub n_samples: usize,
    /// Per-bucket drop probability for the perturbed samples (the issue's
    /// dose-response sweep is cal-side over p ∈ [0.05, 0.3]).
    pub p_drop: f32,
    /// Domain-separation salt folded into the per-request seed (two harness
    /// arms at different salts = independent draw families).
    pub salt: u64,
}

impl Default for McConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

impl McConfig {
    /// The unarmed posture — byte-identical to not calling the wrapper.
    pub const fn disabled() -> Self {
        Self {
            n_samples: 1,
            p_drop: 0.0,
            salt: 0,
        }
    }

    /// Whether the config actually perturbs anything.
    pub const fn armed(&self) -> bool {
        self.n_samples > 1 && self.p_drop > 0.0
    }
}

/// Run the ensemble for `req`, accumulating per-question histograms.
///
/// `hists` must be caller-prepared (`hists.len() == req.questions.len()`,
/// each `prepare`d for its question's option count — the harness allocates,
/// the loop does not). On return `sc` holds the LEGACY sample-0 solve (the
/// unarmed answer; `to_response` composes cleanly after this).
pub fn solve_mc_into<const N: usize, const D: usize>(
    eng: &mut DecisionEngine<N, D>,
    req: &katgpt_core::decision_wire::DecisionRequest,
    sc: &mut Scratch<D>,
    cfg: &McConfig,
    hists: &mut [EnsembleHistogram],
) -> Result<(), crate::engine::EngineError> {
    debug_assert_eq!(
        hists.len(),
        req.questions.len(),
        "one histogram per question"
    );
    // Per-request seed over the request's identity bytes.
    let mut hasher = blake3::Hasher::new();
    hasher.update(req.state.as_bytes());
    for q in &req.questions {
        hasher.update(&[0]);
        hasher.update(q.prompt.as_bytes());
    }
    hasher.update(&cfg.salt.to_le_bytes());
    let seed = u64::from_le_bytes(
        hasher.finalize().as_bytes()[..8]
            .try_into()
            .expect("8 bytes"),
    );

    let n = cfg.n_samples.max(1);
    // Perturbed samples FIRST (1..n), legacy run LAST — sc holds the unarmed
    // answer on return.
    for sample in 1..n {
        let perturb = Some((sample as u32, cfg.p_drop, seed));
        eng.solve_sample_into(req, sc, perturb)?;
        observe_sample(req, sc, hists);
    }
    eng.solve_sample_into(req, sc, None)?;
    observe_sample(req, sc, hists);
    Ok(())
}

/// Read one solved sample out of the scratch into the histograms.
///
/// The pick observed is the pipeline's COMPUTED winner (`slot.pick`) — NOT
/// gated by the fused abstain: an abstained question's underlying answer
/// distribution is exactly the risk signal this surface exists to measure
/// (the engine's own "the distribution still rides for the risk–coverage
/// tables" law). The abstain axis is a separate per-sample dimension the
/// harness summarizes; folding it into the pick histogram (as `None`)
/// would collapse every histogram on abstain-heavy lanes — a lossy readout,
/// not a conservative one.
fn observe_sample<const D: usize>(
    req: &katgpt_core::decision_wire::DecisionRequest,
    sc: &Scratch<D>,
    hists: &mut [EnsembleHistogram],
) {
    for (qi, _q) in req.questions.iter().enumerate() {
        let slot = &sc.slots[qi];
        let probs = &sc.probs[slot.prob_lo..slot.prob_lo + slot.prob_len];
        hists[qi].observe(Some(slot.pick as usize), probs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{EMBED_DIM, Embedder};
    use katgpt_core::decision_wire::{DecisionRequest, Question};

    fn four_topics() -> Vec<crate::engine::DomainExpert<EMBED_DIM>> {
        vec![
            crate::engine::DomainExpert::new(
                "billing",
                &["my card was charged twice and the refund never arrived".to_string()],
                &Embedder,
            ),
            crate::engine::DomainExpert::new(
                "tech",
                &["the app crashes on launch and the login page will not load".to_string()],
                &Embedder,
            ),
            crate::engine::DomainExpert::new(
                "music",
                &["add this song to my playlist and shuffle the album".to_string()],
                &Embedder,
            ),
            crate::engine::DomainExpert::new(
                "weather",
                &["will it rain today and how warm will the afternoon be".to_string()],
                &Embedder,
            ),
        ]
    }

    fn topic_req(state: &str) -> DecisionRequest {
        DecisionRequest {
            state: state.to_string(),
            questions: vec![Question::choice(
                "q0",
                "which team handles this?",
                vec![
                    "billing".to_string(),
                    "tech".to_string(),
                    "music".to_string(),
                    "weather".to_string(),
                ],
                None,
            )],
        }
    }

    /// G1 part 1: N=1 (disabled) == legacy bytes exactly.
    #[test]
    fn n1_is_the_legacy_point_mass() {
        let mut eng = crate::engine::DecisionEngine::<4, EMBED_DIM>::build(
            four_topics(),
            crate::engine::EngineConfig::default(),
        )
        .unwrap();
        let req = topic_req("my card was charged twice and no refund arrived");
        let mut sc = Scratch::<EMBED_DIM>::new();
        sc.prepare(req.questions.len());
        eng.solve_into(&req, &mut sc).unwrap();
        let legacy_pick = sc.slots[0].pick;
        let legacy_probs: Vec<f32> =
            sc.probs[sc.slots[0].prob_lo..sc.slots[0].prob_lo + sc.slots[0].prob_len].to_vec();

        let mut hists = [EnsembleHistogram::default()];
        hists[0].prepare(4);
        solve_mc_into(&mut eng, &req, &mut sc, &McConfig::disabled(), &mut hists).unwrap();
        assert_eq!(hists[0].n_samples(), 1);
        assert_eq!(hists[0].majority_pick(), Some(legacy_pick as usize));
        assert_eq!(hists[0].u_pair(), Some(0.0), "point mass: zero uncertainty");
        // sc holds the legacy solve on return — same pick, same probs.
        assert_eq!(sc.slots[0].pick, legacy_pick);
        let probs: Vec<f32> =
            sc.probs[sc.slots[0].prob_lo..sc.slots[0].prob_lo + sc.slots[0].prob_len].to_vec();
        assert_eq!(probs.len(), legacy_probs.len());
        for (a, b) in probs.iter().zip(legacy_probs.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "N=1 must be byte-identical");
        }
    }

    /// G1 part 2: armed same-seed == armed same-seed (bit-identity of the
    /// histogram's integers AND its moments).
    #[test]
    fn armed_same_seed_is_bit_identical() {
        let mut eng = crate::engine::DecisionEngine::<4, EMBED_DIM>::build(
            four_topics(),
            crate::engine::EngineConfig::default(),
        )
        .unwrap();
        let req = topic_req("the app crashes whenever I open the billing page");
        let mut run = |hists: &mut [EnsembleHistogram; 1]| {
            let mut sc = Scratch::<EMBED_DIM>::new();
            sc.prepare(1);
            let cfg = McConfig {
                n_samples: 12,
                p_drop: 0.2,
                salt: 99,
            };
            solve_mc_into(&mut eng, &req, &mut sc, &cfg, hists).unwrap();
        };
        let mut a = [EnsembleHistogram::default()];
        let mut b = [EnsembleHistogram::default()];
        a[0].prepare(4);
        b[0].prepare(4);
        run(&mut a);
        run(&mut b);
        // Compare the readouts (counts + moments through the public surface;
        // identical readouts at every option ⇒ bit-identical state, since
        // every field feeds a readout).
        for i in 0..4 {
            assert_eq!(
                a[0].option_mean(i).to_bits(),
                b[0].option_mean(i).to_bits(),
                "option {i} mean"
            );
            assert_eq!(
                a[0].option_sigma(i).to_bits(),
                b[0].option_sigma(i).to_bits(),
                "option {i} sigma"
            );
        }
        assert_eq!(a[0].n_samples(), b[0].n_samples());
        assert_eq!(a[0].majority_pick(), b[0].majority_pick());
        assert_eq!(
            a[0].u_pair().unwrap().to_bits(),
            b[0].u_pair().unwrap().to_bits()
        );
    }

    /// Non-collapse floor: a route-active fixture at p=0.2 must produce ≥ 2
    /// distinct picks across samples (the harness discrimination-floor
    /// pattern) — an all-same histogram here would be a dose-response
    /// failure.
    #[test]
    fn route_active_fixture_does_not_collapse() {
        let mut eng = crate::engine::DecisionEngine::<4, EMBED_DIM>::build(
            four_topics(),
            crate::engine::EngineConfig::default(),
        )
        .unwrap();
        // A precisely balanced straddle — 2 topical words per side
        // (billing: card, refund / tech: app, crash), prompt neutral — the
        // discrimination fixture the non-collapse floor requires.
        let req = topic_req("card refund app crash");
        let mut sc = Scratch::<EMBED_DIM>::new();
        sc.prepare(1);
        let mut hists = [EnsembleHistogram::default()];
        hists[0].prepare(4);
        let cfg = McConfig {
            n_samples: 32,
            p_drop: 0.3,
            salt: 7,
        };
        solve_mc_into(&mut eng, &req, &mut sc, &cfg, &mut hists).unwrap();
        // Distinct picks: majority + runner-up shares sum > 0 ⇒ ≥ 2 picks
        // observed (u_pair > 0 proves non-collapse on its own).
        let u = hists[0].u_pair().expect("samples picked");
        assert!(
            u > 0.0,
            "non-collapse floor: u_pair must be > 0 on the straddling fixture (got {u})"
        );
    }

    /// G1 part 3 (the SEPARATE class): a different salt is a different draw
    /// family — histograms MAY differ; this records the class boundary, not
    /// a determinism claim.
    #[test]
    fn different_salt_is_a_different_draw_family() {
        let mut eng = crate::engine::DecisionEngine::<4, EMBED_DIM>::build(
            four_topics(),
            crate::engine::EngineConfig::default(),
        )
        .unwrap();
        let req = topic_req("the app crashes whenever I open the billing page");
        let mut sc = Scratch::<EMBED_DIM>::new();
        sc.prepare(1);
        let mut a = [EnsembleHistogram::default()];
        let mut b = [EnsembleHistogram::default()];
        a[0].prepare(4);
        b[0].prepare(4);
        solve_mc_into(
            &mut eng,
            &req,
            &mut sc,
            &McConfig {
                n_samples: 12,
                p_drop: 0.2,
                salt: 1,
            },
            &mut a,
        )
        .unwrap();
        solve_mc_into(
            &mut eng,
            &req,
            &mut sc,
            &McConfig {
                n_samples: 12,
                p_drop: 0.2,
                salt: 2,
            },
            &mut b,
        )
        .unwrap();
        // No assertion on agreement — cross-seed stability is the harness's
        // aggregate class. This test pins that both families RUN and produce
        // well-formed histograms (n matches, readouts finite).
        assert_eq!(a[0].n_samples(), 12);
        assert_eq!(b[0].n_samples(), 12);
        for h in [&a[0], &b[0]] {
            assert!(h.u_pair().unwrap().is_finite());
            assert!(h.majority_pick().is_some());
        }
    }
}
