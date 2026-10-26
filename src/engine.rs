//! The modelless decision engine (Plan 603 T1.3).
//!
//! Pipeline per question, ALL questions answered in one call (the wire's
//! "all questions in the call" law):
//!
//! 1. **embed** — state + prompt (+ criteria) → `[f32; D]` (the [`crate::embed`]
//!    head);
//! 2. **route** — `pick_domain` argmax over per-domain unit centroids
//!    (the same argmax-over-centroids math as the KernelExpertRouter
//!    pattern). Day-one note: `CalibratedActionBridge`'s fixed-`A` wrapper
//!    cannot select over REQUEST-TIME option spaces (`A` is const, options
//!    are dynamic) — the bridge's substance, the `SigmoidGateCalibrator`,
//!    is wired on the answer confidence instead, and `pick_domain` is the
//!    routing head. The bridge itself stays forwarded for fixed-A lanes.
//! 3. **score** — corpus-is-the-model, TWO signals per option:
//!    the `Lz4FlexDrafter` scores each option's bytes against the routed
//!    domain's corpus with the request context prepended (an `i32`
//!    compressed-length delta — deterministic by construction), AND — when
//!    domains double as option classes (`k == N`, every lane this engine
//!    serves) — the query's cosine to that option's corpus centroid. The
//!    blend is load-bearing, not decorative: a 4-byte option string never
//!    moves the compressed length of a ~300-byte context, so the drafter
//!    delta alone scored every option identically and the argmax tie broke
//!    to index 0 — a constant, input-independent pick (the degenerate-lane
//!    lesson, Issue 004 T7); the centroid cosine is the signal that
//!    actually ranks options;
//! 4. **normalize** — per-option sigmoid at `score_temperature`, then L1
//!    normalization (SIGMOID, never softmax — the house law; a bounded
//!    monotone map needs no temperature competition);
//! 5. **calibrate + readout** — the [`crate::readout`] dispatch produces
//!    the raw confidence; `SigmoidGateCalibrator::apply` is identity until
//!    outcome evidence refits it (bit-identical cold start, the bench-808
//!    law);
//! 6. **abstain** — the fused gate: calibrated confidence below the score
//!    threshold OR the domain's `CorpusDistanceGate` says the state is
//!    off-corpus — and, behind the `density_gate` feature (Issue 066), a
//!    third DENSITY half: the routed domain's two-density support gate
//!    (per-domain diagonal GMM vs the pooled reference, JL-projected per
//!    the fit-space law) reads below the density threshold —
//!    in-support-ness beside classifier confidence and exemplar distance,
//!    three axes never merged. Abstention is a FIRST-CLASS answer
//!    (outcome `None`, the distribution still rides for the risk–coverage
//!    tables).
//!
//! **Allocation law:** [`DecisionEngine::solve_into`] is the zero-alloc
//! core — fixed-size arrays + caller-owned [`Scratch`] end to end (the
//! G4 gate counts, with a canary proving the counter live). Wire
//! materialization ([`DecisionEngine::to_response`]) allocates BY DESIGN:
//! the wire types own their payload; that is the boundary, not the hot path.

use crate::embed::Embedder;
use crate::label_heads::LabelHeads;
#[cfg(feature = "nb_ridge")]
use crate::nb_ridge::NbRidge;
#[cfg(feature = "nb_scope")]
use crate::nb_scope::{NbAlpha, NbScope, NbView, view_tokens_into};
#[cfg(feature = "option_cond")]
use crate::option_cond::OptionCond;
use crate::readout;
use katgpt_core::compression_drafter::Lz4FlexDrafter;
use katgpt_core::decision_wire::{
    Answer, Calibration, DecisionRequest, DecisionResponse, Lane, Outcome, QuestionKind, Routing,
    WireError,
};
use katgpt_core::distance_abstain::CorpusDistanceGate;
use katgpt_core::exact_sigmoid;
#[cfg(feature = "density_gate")]
use katgpt_core::gmm_support::{EmConfig, JlProjector};
#[cfg(feature = "density_gate")]
use katgpt_core::gmm_support::{SupportGate, fit_diag_gmm};
use katgpt_core::sigmoid_calibration::SigmoidGateCalibrator;
use katgpt_core::variable_rank_domain_expert::pick_domain;

/// The threshold-recommendation surface (Issue 009 — the jimothy steal:
/// per-task abstention cutoffs as fit-time engine metadata, null on thin
/// support, support-disclosed, posture-cited).
pub mod threshold;

pub use threshold::{
    FusedGateRecommendation, GateObservation, Posture, RecommendationStatus, THIN_SUPPORT_FLOOR,
    ThresholdRecommendation, ThresholdSupport, recommend_fused_gate, threshold_recommendation,
};

/// Sigmoid temperature on raw LZ4 score deltas (i32-scale diffs; /24 puts
/// the useful slope around the corpus-scale deltas measured at birth).
pub const DEFAULT_SCORE_TEMPERATURE: f32 = 24.0;
/// Fused-gate score threshold on the CALIBRATED confidence.
pub const DEFAULT_SCORE_THRESHOLD: f32 = 0.35;
/// Fused-gate distance threshold on the corpus-distance confidence.
pub const DEFAULT_DISTANCE_THRESHOLD: f32 = 0.5;
/// Corpus-distance gate sigmoid midpoint (max-cosine axis). Birth
/// measurement: in-corpus max-cos ≈ 0.68–0.70, off-corpus ≈ 0.00–0.02 —
/// the midpoint sits in the desert between the two populations.
/// `pub` for the diagnostics that measure drift AGAINST the birth
/// boundary (Issue 093) — the constant itself is load-bearing and never
/// edited by a measurement (any compensation ships behind a lever).
pub const GATE_MID: f32 = 0.35;
/// Corpus-distance gate sigmoid SLOPE (the substrate semantic is
/// `sigmoid(scale · (max_sim − mid))` — scale multiplies the cosine
/// distance from the midpoint). At slope 8 the two measured populations
/// (in-corpus cos ≈ 0.68–0.70, off-corpus cos ≈ 0.00–0.02) pin to ≈0.94 / ≈0.06.
pub const GATE_SCALE: f32 = 8.0;

/// The gate's confidence formula for a raw max-cosine reading — ONE home
/// for the birth shape (`sigmoid(GATE_SCALE · (x − GATE_MID))`), shared by
/// the diagnostics that must recompute a boundary from a max-similarity
/// distribution (Issue 093) instead of re-deriving the constants.
/// Bit-identical to [`katgpt_core::distance_abstain::CorpusDistanceGate::abstain_confidence`]
/// by construction (the same two constants, the same delegated sigmoid).
#[must_use]
pub fn gate_confidence(max_sim: f32) -> f32 {
    katgpt_core::exact_sigmoid(GATE_SCALE * (max_sim - GATE_MID))
}
/// Issue 066: JL output dim for the density half — the fit-space law
/// (raw hashed-bag space REJECTS Gaussianity universally, 0/154
/// per-label pools; JL k=64 accepted on 253/254, k-stable across
/// 64/32/16 — the PRE-CHECK's measured verdict). The projection is part
/// of the gate's frozen definition.
#[cfg(feature = "density_gate")]
pub const DENSITY_E: usize = 64;
/// Issue 066: GMM components per domain — the substrate bench's
/// request-posture shape (Bench 908, E=64/K=16 on an 800-sample
/// fixture), measured against K=4 at the first A/B (Bench 124): K=4
/// holds the two many-label winners (+0.043/+0.027) but flips ag_news
/// negative (−0.010 vs +0.018) and degrades sst5/xnli — the wider
/// mixture's extra components earn their keep on the modal structure
/// even at 40-doc pools, with the raised variance floor
/// ([`DENSITY_VAR_FLOOR`]) doing the spike-guarding instead.
#[cfg(feature = "density_gate")]
pub const DENSITY_K: usize = 16;
/// Issue 066: fused-gate threshold on the density confidence (the
/// `sigmoid(ℓ/τ)` scalar). Birth constant; the harness A/B fits it at
/// the cal-slice ρ=30 percentile (the T1.6 posture shared with the
/// other two axes).
#[cfg(feature = "density_gate")]
pub const DEFAULT_DENSITY_THRESHOLD: f32 = 0.5;
/// Issue 066: the JL projector seed — fixed (the gate's definition is
/// frozen; a different projector is a different gate).
#[cfg(feature = "density_gate")]
const DENSITY_JL_SEED: u64 = 7;
/// Issue 066: EM variance floor for the density fits. Unit-norm inputs
/// through the JL projection have per-coordinate variance ≈ 1/E = 0.016
/// (σ ≈ 0.125). The first A/B reading (Bench 124) measured the K=16
/// positives on 40–64-doc pools collapsing onto narrow clusters (floored
/// at the substrate default 1e-6 → σ = 0.001): QUESTION embeddings
/// (state+prompt — a different region than the doc embeddings the GMMs
/// were fit on) scored ℓ ≈ −100s against them, >30% of cal confidences
/// underflowed sigmoid to exactly 0, and the ρ=30 fit landed on a
/// denormal — abstention without discrimination. The floor at 0.01
/// (σ = 0.1, ~80% of the pool's per-coordinate spread) makes every
/// component a near-pool-width blob: the PRE-CHECK's per-label
/// near-Gaussian verdict says that IS the honest shape, and the
/// mixture's job reverts to covering the pool (and the question region)
/// rather than carving 16 spikes into 40 points.
#[cfg(feature = "density_gate")]
const DENSITY_VAR_FLOOR: f32 = 1e-2;

/// One domain's authored corpus — the builder input to
/// [`DecisionEngine::build_specs`].
#[derive(Clone, Debug)]
pub struct ExpertSpec {
    /// Domain name (Routing-reason material).
    pub name: String,
    /// The domain's documents (the corpus IS the model).
    pub docs: Vec<String>,
    /// Issue 038: an optional WIDER document set for the count tables
    /// (`nb_scope`) — the drafter's corpus stays capped for latency
    /// (drafter cost grows with corpus bytes), while a count table's
    /// scoring cost is independent of how many docs built it. `None` ⇒ the
    /// tables read `docs`.
    #[cfg(feature = "nb_scope")]
    pub nb_docs: Option<Vec<String>>,
}

impl ExpertSpec {
    /// Spec from a name + document list.
    pub fn new(name: impl Into<String>, docs: &[String]) -> Self {
        Self {
            name: name.into(),
            docs: docs.to_vec(),
            #[cfg(feature = "nb_scope")]
            nb_docs: None,
        }
    }

    /// Attach the wider count-table document set (issue 038).
    #[cfg(feature = "nb_scope")]
    #[must_use]
    pub fn with_nb_docs(mut self, docs: Vec<String>) -> Self {
        self.nb_docs = Some(docs);
        self
    }
}

/// One corpus expert: a domain's documents, its drafter (the corpus IS the
/// model), its exemplar gate, and its routing direction (the unit centroid
/// of the document embeddings).
pub struct DomainExpert<const D: usize> {
    name: String,
    drafter: Lz4FlexDrafter,
    gate: CorpusDistanceGate<D>,
    direction: [f32; D],
    /// Issue 066 (`density_gate` feature): the domain's two-density
    /// support gate — positive fit on THIS domain's projected rows,
    /// negative shared across domains (the pooled projected rows —
    /// distribution WIDTH, not identity). `None` when the knob is off
    /// (the byte-identical posture) or on raw-expert builds.
    #[cfg(feature = "density_gate")]
    density: Option<SupportGate<DENSITY_E, DENSITY_K>>,
}

impl<const D: usize> DomainExpert<D> {
    /// Build from the domain's documents. Empty corpora are refused by
    /// [`DecisionEngine::build`] (an empty gate abstains at every
    /// threshold — a domain that answers nothing must not exist).
    pub fn new(name: impl Into<String>, docs: &[String], embedder: &Embedder) -> Self {
        assert!(
            !docs.is_empty(),
            "DomainExpert requires at least one document"
        );
        let mut rows: Vec<[f32; D]> = Vec::with_capacity(docs.len());
        for doc in docs {
            let mut v = [0.0f32; D];
            embedder.embed_into(doc.as_bytes(), &mut v);
            rows.push(v);
        }
        let corpus = docs.join("\n");
        Self::from_embedded(name, corpus.into_bytes(), &rows)
    }

    /// Assemble from ALREADY-EMBEDDED rows (the build_specs path embeds
    /// once and shares the rows with the head fit — one pass over the
    /// corpus, never two).
    fn from_embedded(name: impl Into<String>, corpus: Vec<u8>, rows: &[[f32; D]]) -> Self {
        let mut acc = [0.0f32; D];
        for r in rows {
            for (a, x) in acc.iter_mut().zip(r.iter()) {
                *a += x;
            }
        }
        // Unit centroid = the routing direction.
        let k = rows.len() as f32;
        let mut direction = acc;
        for x in direction.iter_mut() {
            *x /= k;
        }
        let n = direction.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n > 0.0 {
            for x in direction.iter_mut() {
                *x /= n;
            }
        }
        Self {
            name: name.into(),
            drafter: Lz4FlexDrafter::new(corpus),
            gate: CorpusDistanceGate::new(rows, GATE_MID, GATE_SCALE),
            direction,
            #[cfg(feature = "density_gate")]
            density: None,
        }
    }

    /// The domain's name (Routing reason material).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The corpus-distance gate (exposed read-only for harness calibration —
    /// the self-calibrating gate tests read it to set thresholds from
    /// measured geometry, never from magic numbers).
    pub fn gate(&self) -> &CorpusDistanceGate<D> {
        &self.gate
    }
}

/// Engine operating policy. `Default` is the birth-tuned posture; every
/// threshold is a knob the harness (T1.5) sweeps, never a silent constant.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineConfig {
    /// Sigmoid temperature on raw LZ4 score deltas.
    pub score_temperature: f32,
    /// Fused-gate threshold on the calibrated confidence.
    pub score_threshold: f32,
    /// Fused-gate threshold on the corpus-distance confidence.
    pub distance_threshold: f32,
    /// Calibrator evidence-window capacity.
    pub cal_capacity: usize,
    /// Calibrator occupancy floor before a refit can move anything.
    pub cal_min_obs: usize,
    /// Option-rank blend scale (Issue 004 T7, sweep lever in issue 013):
    /// the state-to-centroid cosine is sigmoid-squashed at this scale before
    /// it ranks options. Default 8.0 is the landing value; a promoted change
    /// needs the measured sweep at parity everywhere else (the GOAT shape).
    pub route_scale: f32,
    /// Resolve each option to its domain BY NAME (Issue 023): when every
    /// option string names a domain, the option's route term is the state's
    /// cosine to THAT domain's centroid, whatever the option order or
    /// arity. Off (or any option unresolved) → the legacy rule: route terms
    /// only when `k == N`, aligned by INDEX. Without it a sampled-distractor
    /// suite (massive: 20 of 59 labels per question) never sees the centroid
    /// signal and ranks options by the drafter delta alone — measured
    /// ~1.5x chance.
    pub option_name_route: bool,
    /// Issue 079: the index-anchored route arm. When by-name resolution
    /// fails, a SCORE question with k == N whose domains ARE named by the
    /// option indices ("0".."N-1" — sst5's ordinal level corpora; the
    /// noul-precedent class) resolves option i to domain i BY NAME, so
    /// position ↔ level ↔ corpus is the scale's own semantics. A CHOICE
    /// question NEVER takes this arm (a choice option's identity is its
    /// content, never its position — the old blind `k == N` fill was
    /// typed's position-driven defect: bench 128, 4.70 pt median swing,
    /// 68.3% flips). `false` = strict by-name only (the A/B instrument;
    /// the harness's `--no-kn-route`). Default `true` = the shipped
    /// behavior, byte-identical for every suite except typed (bench 130).
    pub legacy_kn_route: bool,
    /// Fitted per-label head blend scale (issue 030 lever 4). When > 0,
    /// [`DecisionEngine::build_specs`] fits one-vs-all logistic heads over
    /// the hashed-bag features at build time (deterministic, [`crate::label_heads`])
    /// and every option WHEREVER ROUTE TERMS ARE ACTIVE gains the blend term
    /// `0.5 + head_scale·(σ(logit) − 0.5)` — the fitted model's deviation
    /// from neutral, joining the σ-shaped drafter + route terms. Noul NEVER
    /// takes the term (issue 030: its `[yes, no]` pair is question
    /// vocabulary, never a label list). 0.0 = OFF — byte-identical to the
    /// pre-head posture, and the default until the GOAT verdict. A
    /// [`DecisionEngine::build`] (raw experts, no corpora) refuses a
    /// non-zero scale — a head that silently never fires is the bug class
    /// this engine refuses.
    pub head_scale: f32,
    /// Naive-Bayes count-table blend scale (issue 038 T1, opt-in
    /// `nb_scope`). When > 0, [`DecisionEngine::build_specs`] fits one-vs-rest
    /// `ContrastiveScoreTable`s per domain ([`crate::nb_scope`]) and every
    /// option WHEREVER ROUTE TERMS ARE ACTIVE gains
    /// `nb_scale · σ(margin / n_tokens)` — the same legality guard as the
    /// heads (noul never takes it). 0.0 = OFF, byte-identical to the
    /// pre-038 posture.
    #[cfg(feature = "nb_scope")]
    pub nb_scale: f32,
    /// Smoothing policy for the count tables (issue 038).
    #[cfg(feature = "nb_scope")]
    pub nb_alpha: NbAlpha,
    /// Noul polarity for the count tables (issue 038): `Some(d)` means a
    /// `noul` question's "yes" reads as domain `d` (yes term
    /// `nb_scale·σ(margin_d/n)`, no term `nb_scale·(1 − σ(margin_d/n))`).
    /// `None` = noul takes no count-table term (the issue-030 posture). The
    /// wire cannot carry this — `[yes, no]` is question vocabulary — so it
    /// is caller configuration, selected on a LABELLED cal slice by the
    /// harness, never read off test.
    #[cfg(feature = "nb_scope")]
    pub nb_noul_domain: Option<usize>,
    /// Count-table event view (issue 038 T3): bag, or the sentence-pair
    /// view (last field read against the earlier ones).
    #[cfg(feature = "nb_scope")]
    pub nb_view: NbView,
    /// PAIRWISE count-table blend scale (issue 081 T2c, opt-in `nb_scope`).
    /// When > 0, a by-name-resolved choice question's option `i` gains
    /// `nb_pair_scale · σ(in_{d_i} − max_{j≠i} in_{d_j})` — the margin over
    /// the QUESTION'S OWN option domains only, in BITS, with NO per-token
    /// normalization. Measured law (the v8 probe): on O(1000)-token states
    /// a single discriminative token contributes O(10) bits to the table
    /// margins — cleanly signed — but `σ(margin/n_tokens)` (the nb/oc
    /// convention) averages it to ±0.02 in σ-space and the ridge's
    /// top-k-by-df selection drops class-exclusive low-df buckets
    /// outright; under L1-normalized scores the averaged family is also
    /// scale-self-limiting (byte-identical p at scale 4k vs 64k — the
    /// common σ component cancels). The pairwise form has no common
    /// component (a 2-option pair is complementary: the terms sum to the
    /// scale), so its L1 leverage is `2σ(Δ)−1` — full range at any scale.
    /// 0.0 = OFF, byte-identical.
    #[cfg(feature = "nb_scope")]
    pub nb_pair_scale: f32,
    /// Option-conditioned count-table blend scale (issue 038 T7b, opt-in
    /// `option_cond`). When > 0, the event-carrying constructor
    /// ([`DecisionEngine::build_specs_oc`]) fits one (qid, option) table
    /// per gold event group ([`crate::option_cond`]) and every option of
    /// EVERY question kind gains `oc_scale · σ(margin / n_tokens)` —
    /// independent of `route_active` (the typed questions this lever
    /// targets are drafter_only today). Options whose key has no table
    /// take NO term (never a fabricated neutral). 0.0 = OFF,
    /// byte-identical to the pre-T7b posture.
    #[cfg(feature = "option_cond")]
    pub oc_scale: f32,
    /// NBSVM ridge readout blend scale (issue 038 T7a, opt-in
    /// `nb_ridge`). When > 0, [`DecisionEngine::build_specs`] fits the
    /// closed-form per-domain ridge ([`crate::nb_ridge`]) over the same
    /// wide doc sets the nb tables read, and every option WHEREVER ROUTE
    /// TERMS ARE ACTIVE gains the margin term (the same legality guard as
    /// the heads/nb — the class IS the domain). The fit is O(k³) per
    /// class (k = [`crate::nb_ridge::RIDGE_K`], build-time only; scoring
    /// is O(|x| log k)). 0.0 = OFF, byte-identical.
    #[cfg(feature = "nb_ridge")]
    pub ridge_scale: f32,
    /// The ridge λ (Tikhonov damping) the weights solve with. Fixed by
    /// the caller when the readout arms; the probe selected 10.0.
    #[cfg(feature = "nb_ridge")]
    pub ridge_lambda: f32,
    /// Confidence-readout functional (Issue 039 T4). Default is the
    /// shipped Bench-817 dispatch; the harness may arm a per-suite mode
    /// selected on the cal slice (in-sample calibrated ECE, margin over
    /// Dispatch). Readout-only — never moves the pick or the distribution.
    pub readout: crate::readout::ReadoutMode,
    /// Drafter-delta correction for the DRAFTER-ONLY path (Issue 036 T2):
    /// choice/score questions where neither route path armed, whose option
    /// score is the raw LZ4 compressed-length delta — a quantity that
    /// carries an option-length prior (appending a short literal grows the
    /// stream less) and collapses onto the shortest option on dynamic
    /// option spaces. Engaged ONLY when the question is drafter-only, so
    /// every route-armed suite stays byte-identical by construction (the
    /// issue's no-regression gate). Off = the shipped law.
    pub drafter_fix: DrafterFix,
    /// Issue 066 (`density_gate` feature): arm the fused gate's DENSITY
    /// half — the per-domain two-density support gate over the
    /// JL-projected embedding. `false` (the default) never fits, never
    /// evaluates, and is byte-identical to a feature-off build (the
    /// `contrastive_scope` posture, pinned by test).
    #[cfg(feature = "density_gate")]
    pub density_gate: bool,
    /// Issue 066: fused-gate threshold on the density confidence
    /// `sigmoid(ℓ/τ)`; meaningful only when `density_gate` is armed. The
    /// harness A/B fits it at the cal-slice ρ=30 percentile.
    #[cfg(feature = "density_gate")]
    pub density_threshold: f32,
    /// Issue 066: the density confidence's sigmoid temperature τ (baked
    /// into the fitted gates at build time — a different τ is a rebuild,
    /// the POC sweep posture). POC-scale default (the substrate's
    /// `DEFAULT_TAU`).
    #[cfg(feature = "density_gate")]
    pub density_tau: f32,
    /// Plan 627 / Issue 090 C2 (`set_refine` feature): set-conditioned
    /// state refinement. When > 0, every ROUTE-ACTIVE question's state
    /// embedding `x` (the state-alone vector that feeds the route terms —
    /// and the head terms when fitted) is first refined against the
    /// question's OWN option set through the katgpt-core `set_refine`
    /// composition: `x' = x + (γ/n)·Σ wᵢ(kᵢ−x)`, `wᵢ = σ(dot(x,kᵢ)/τ)`,
    /// where the option keys `kᵢ` are the options' resolved domain
    /// DIRECTIONS (this engine's option model — "each label's corpus
    /// centroid IS that option's model", the option-rank law). The route
    /// and head terms are then recomputed from `x'` per question — the
    /// set-dependence an independent per-option scorer structurally
    /// lacks (Jeva Prop 1, arXiv:2610.18773; the A2 exemption is the
    /// issue's own law — decoy-invariance is deliberately BROKEN, never
    /// "fixed"). Scope: noul and drafter-only questions never take route
    /// terms (issue 030 legality) and are byte-identical; the count-table
    /// families (nb/oc/ridge) are token-keyed, not embedding-keyed, and
    /// are structurally unaffected. 0.0 = OFF, byte-identical to the
    /// pre-627 posture — and the default (NOT wired into any shipped
    /// posture; the harness `--set-refine-ab` A/B arm is the only armer).
    #[cfg(feature = "set_refine")]
    pub set_refine_gamma: f32,
    /// Plan 627 (`set_refine` feature): the refinement gate temperature
    /// τ in `wᵢ = σ(dot(x,kᵢ)/τ)`. The kernel-side working envelope
    /// measured at katgpt-rs Plan 627 T1: τ 0.4–0.7 × γ 0.8–1.5.
    /// Default 0.5 (the S1 fixture center).
    #[cfg(feature = "set_refine")]
    pub set_refine_tau: f32,
}

/// The [`EngineConfig::drafter_fix`] candidates (Issue 036 T2). All are
/// corrections to the drafter term `s = compressed_len(ctx) −
/// compressed_len(ctx+cand)` (higher = more compressible = more likely).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrafterFix {
    /// The shipped law: raw delta.
    #[default]
    Off,
    /// Length-normalized delta: `s / candidate_byte_len` — cancels the
    /// prior that short literals win by growing the stream less.
    PerByte,
    /// The NCD-style conditional term: `s − standalone(candidate)`, where
    /// standalone = the delta against an EMPTY context (the candidate's own
    /// compressed cost under the corpus). What remains is the context-
    /// CONDITIONED signal only.
    Ncd,
    /// Score only the candidate SUFFIX past the options' longest common
    /// prefix (shared prefixes like `fill ` cancel; the row-unique value is
    /// what distinguishes). Candidate-byte rewrite.
    SharedPrefix,
    /// Score only the candidate KEY: the bytes up to the first `: `
    /// (strips the row-unique value after it — `fill Given name: Isla`
    /// scores as `fill Given name`). Options without `: ` score whole.
    /// Candidate-byte rewrite.
    KeyOnly,
}

impl DrafterFix {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DrafterFix::Off => "off",
            DrafterFix::PerByte => "per_byte",
            DrafterFix::Ncd => "ncd",
            DrafterFix::SharedPrefix => "shared_prefix",
            DrafterFix::KeyOnly => "key_only",
        }
    }

    /// The inverse of [`DrafterFix::as_str`] (unknown → `None`).
    #[must_use]
    pub fn from_spelling(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "per_byte" => Some(Self::PerByte),
            "ncd" => Some(Self::Ncd),
            "shared_prefix" => Some(Self::SharedPrefix),
            "key_only" => Some(Self::KeyOnly),
            _ => None,
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            score_temperature: DEFAULT_SCORE_TEMPERATURE,
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            distance_threshold: DEFAULT_DISTANCE_THRESHOLD,
            cal_capacity: 512,
            cal_min_obs: 64,
            route_scale: 8.0,
            option_name_route: true,
            legacy_kn_route: true,
            head_scale: 0.0,
            #[cfg(feature = "nb_scope")]
            nb_scale: 0.0,
            #[cfg(feature = "nb_scope")]
            nb_alpha: NbAlpha::ObservedLaplace,
            #[cfg(feature = "nb_scope")]
            nb_noul_domain: None,
            #[cfg(feature = "nb_scope")]
            nb_view: NbView::Bag,
            #[cfg(feature = "nb_scope")]
            nb_pair_scale: 0.0,
            #[cfg(feature = "option_cond")]
            oc_scale: 0.0,
            #[cfg(feature = "nb_ridge")]
            ridge_scale: 0.0,
            #[cfg(feature = "nb_ridge")]
            ridge_lambda: 10.0,
            readout: crate::readout::ReadoutMode::Dispatch,
            drafter_fix: DrafterFix::Off,
            #[cfg(feature = "density_gate")]
            density_gate: false,
            #[cfg(feature = "density_gate")]
            density_threshold: DEFAULT_DENSITY_THRESHOLD,
            #[cfg(feature = "density_gate")]
            density_tau: 1.0,
            #[cfg(feature = "set_refine")]
            set_refine_gamma: 0.0,
            #[cfg(feature = "set_refine")]
            set_refine_tau: 0.5,
        }
    }
}

/// Issue 066 (`density_gate` feature): fit the density half's gate family
/// — one shared JL projector, one shared NEGATIVE GMM over the POOLED
/// projected rows (every domain: distribution WIDTH, not identity — the
/// PRE-CHECK's whole-corpus rejection is the negative being BROAD, which
/// is the v1 reference posture; the out-of-suite arm is the A/B's second
/// posture if this one underperforms), and one POSITIVE GMM per domain
/// over its own projected rows (the per-label pools are the near-Gaussian
/// side post-JL, 253/254 accepted). Knob off ⇒ `(None, [])` — no fit,
/// no cost, byte-identical build.
#[cfg(feature = "density_gate")]
fn fit_density_gates<const D: usize>(
    rows_all: &[Vec<[f32; D]>],
    armed: bool,
    tau: f32,
) -> Result<
    (
        Option<JlProjector<D, DENSITY_E>>,
        Vec<SupportGate<DENSITY_E, DENSITY_K>>,
    ),
    EngineError,
> {
    if !armed {
        return Ok((None, Vec::new()));
    }
    let proj = JlProjector::<D, DENSITY_E>::new(DENSITY_JL_SEED);
    let em = EmConfig {
        var_floor: DENSITY_VAR_FLOOR,
        ..EmConfig::default()
    };
    let project_into = |rows: &[[f32; D]], out: &mut Vec<[f32; DENSITY_E]>| {
        out.reserve(rows.len());
        for r in rows {
            let mut x = [0.0f32; DENSITY_E];
            proj.project(r, &mut x);
            out.push(x);
        }
    };
    // Negative: pooled rows across every domain, fit ONCE, shared by all
    // domain gates (the paper's `-43% gate count` mechanism).
    let pooled_len: usize = rows_all.iter().map(|r| r.len()).sum();
    let mut pooled: Vec<[f32; DENSITY_E]> = Vec::with_capacity(pooled_len);
    for rows in rows_all {
        project_into(rows, &mut pooled);
    }
    let pooled_refs: Vec<&[f32]> = pooled.iter().map(|x| x.as_slice()).collect();
    let neg = fit_diag_gmm::<DENSITY_E, DENSITY_K>(&pooled_refs, &em)
        .map_err(|_| EngineError::DensityFit)?;
    // Positives: one per domain over its own projected rows.
    let mut gates = Vec::with_capacity(rows_all.len());
    for rows in rows_all {
        let mut projected: Vec<[f32; DENSITY_E]> = Vec::with_capacity(rows.len());
        project_into(rows, &mut projected);
        let refs: Vec<&[f32]> = projected.iter().map(|x| x.as_slice()).collect();
        let pos = fit_diag_gmm::<DENSITY_E, DENSITY_K>(&refs, &em)
            .map_err(|_| EngineError::DensityFit)?;
        gates.push(SupportGate::new(pos, neg.clone(), tau).map_err(|_| EngineError::DensityFit)?);
    }
    Ok((Some(proj), gates))
}

/// First offset of `needle` in `haystack` (None when absent) — the
/// Issue-036 KeyOnly rewrite's `: ` finder. Byte-level: candidates are
/// `&[u8]`, and `[u8]::split_once` is unstable.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// The `[ridge-dbg]` flag, read ONCE per process (issue 075). The
/// per-question `env::var_os` lookup this replaced was a G4 violation at
/// the nb_ridge posture: on Windows a lookup of a MISSING variable
/// ALLOCATES every call (measured 1000 missing-var lookups → 1000
/// allocations), so every route-active question allocated once and the
/// bench's 200-rep G4 loop counted exactly 4 × 200 = 800. The lookup is
/// also a real hot-path cost (env lock + OS call per question). Init
/// rides the first solve (warmup); a var set AFTER the first solve is
/// not observed — a debug flag, never a config surface.
#[cfg(feature = "nb_ridge")]
fn ridge_debug_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RIIR_DEBUG_RIDGE").is_some())
}

/// The longest prefix (≥ [`SHARED_PREFIX_MIN`], the measured floor: a word)
/// carried by at least half the options, and its byte length — the
/// Issue-036 SharedPrefix rewrite. Deterministic; `(0, None)` when no
/// family qualifies (including k < 2).
const SHARED_PREFIX_MIN: usize = 4;

fn shared_prefix_of(options: &[String]) -> usize {
    if options.len() < 2 {
        return 0;
    }
    let need = options.len().div_ceil(2);
    let min_len = options.iter().map(|o| o.len()).min().unwrap_or(0);
    let mut best = 0usize;
    for l in (SHARED_PREFIX_MIN..=min_len).rev() {
        // Count L-byte prefixes at this length; a majority carrying ONE
        // identical prefix is the family.
        let mut seen: std::collections::HashMap<&[u8], usize> = std::collections::HashMap::new();
        for o in options {
            let n = seen.entry(&o.as_bytes()[..l]).or_insert(0);
            *n += 1;
        }
        if let Some((_, count)) = seen.iter().max_by_key(|(_, c)| **c)
            && *count >= need
        {
            best = l;
            break; // longest first — the first qualifying length wins
        }
    }
    best
}

/// Fail-closed engine errors. `Wire` wraps the wire contract's own
/// validation verdicts (position-tagged).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineError {
    /// The request failed `DecisionRequest::validate`.
    Wire(WireError),
    /// `build` received the wrong number of domain specs for the engine's
    /// const `N`.
    DomainCount {
        /// specs provided
        have: usize,
        /// domains required
        want: usize,
    },
    /// `build_specs` received a domain with NO documents (an empty gate
    /// abstains at every threshold — a domain that answers nothing must
    /// not exist).
    EmptyCorpus {
        /// offending domain index
        domain: usize,
    },
    /// `build` (raw experts — the corpora are already consumed) was asked
    /// for fitted heads (`head_scale > 0`). The fit needs the document
    /// texts; refusing is fail-closed — a head that silently never fires
    /// is exactly the flag-does-nothing bug class.
    HeadsNeedCorpora,
    /// `build` (raw experts) was asked for count tables (`nb_scale > 0`) —
    /// the same fail-closed law as [`EngineError::HeadsNeedCorpora`].
    #[cfg(feature = "nb_scope")]
    NbNeedsCorpora,
    /// `oc_scale > 0` was set but no (qid, option, doc) events were
    /// supplied to the event-carrying constructor — the same fail-closed
    /// law: a scale that silently never fires is the flag-does-nothing bug
    /// class this engine refuses.
    #[cfg(feature = "option_cond")]
    OcNeedsEvents,
    /// `ridge_scale > 0` on [`DecisionEngine::build`] (raw experts) — the
    /// fit needs the document texts (the [`EngineError::NbNeedsCorpora`]
    /// law).
    #[cfg(feature = "nb_ridge")]
    RidgeNeedsCorpora,
    /// [`DecisionEngine::set_blend_scales`] (issue 038 T5 genome lane) was
    /// handed a positive scale whose fitted tables this build never
    /// produced — the same fail-closed never-silently-fires law as the
    /// build-time gates: the search builds once with every lever fitted,
    /// so a positive scale over absent tables can only be a caller bug.
    ScaleNotFitted {
        /// which blend scale ("head" / "nb" / "oc" / "ridge")
        scale: &'static str,
    },
    /// `build` (raw experts) was asked for the density half
    /// (`density_gate == true`, Issue 066) — the GMM fits need the
    /// document texts (the [`EngineError::HeadsNeedCorpora`] law: a
    /// gate that silently never fires is the flag-does-nothing bug
    /// class).
    #[cfg(feature = "density_gate")]
    DensityNeedsCorpora,
    /// A density-half GMM fit failed (Issue 066). Unreachable by
    /// construction on `build_specs` paths — empty corpora are refused
    /// upstream and the dimensions are compile-fixed — so seeing this
    /// is a substrate contract break, never a corpus-shape issue.
    #[cfg(feature = "density_gate")]
    DensityFit,
    /// Plan 627 / Issue 090 C2 (`set_refine` feature): `set_refine_gamma > 0`
    /// with a non-positive `set_refine_tau` — the gate
    /// `w_i = σ(dot(x,k_i)/τ)` is undefined at τ ≤ 0. Refusing is the
    /// fail-closed law (never a silent NaN-producing knob).
    #[cfg(feature = "set_refine")]
    BadSetRefineTau,
    /// Plan 627 (`set_refine` feature): the refinement kernel refused the
    /// shapes — a caller-bug surface (the engine sizes its own buffers;
    /// the kernel's own error is preserved verbatim).
    #[cfg(feature = "set_refine")]
    SetRefineKernel(katgpt_core::set_refine::SetRefineError),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wire(e) => write!(f, "request invalid: {e}"),
            Self::DomainCount { have, want } => {
                write!(
                    f,
                    "domain count mismatch: {have} specs for a {want}-domain engine"
                )
            }
            Self::EmptyCorpus { domain } => {
                write!(f, "domain {domain} has an empty corpus")
            }
            Self::HeadsNeedCorpora => write!(
                f,
                "head_scale > 0 needs document corpora: use build_specs (the fit reads the docs)"
            ),
            #[cfg(feature = "nb_scope")]
            Self::NbNeedsCorpora => write!(
                f,
                "nb_scale > 0 needs document corpora: use build_specs (the tables read the docs)"
            ),
            #[cfg(feature = "option_cond")]
            Self::OcNeedsEvents => write!(
                f,
                "oc_scale > 0 needs (qid, option, doc) events: use build_specs_oc"
            ),
            #[cfg(feature = "nb_ridge")]
            Self::RidgeNeedsCorpora => write!(
                f,
                "ridge_scale > 0 needs document corpora: use build_specs (the fit reads the docs)"
            ),
            Self::ScaleNotFitted { scale } => write!(
                f,
                "{scale}_scale > 0 needs the fitted tables this build never produced — \
                 build once with the lever armed (a positive placeholder scale), then move scales"
            ),
            #[cfg(feature = "density_gate")]
            Self::DensityNeedsCorpora => write!(
                f,
                "density_gate needs document corpora: use build_specs (the GMM fits read the docs)"
            ),
            #[cfg(feature = "density_gate")]
            Self::DensityFit => write!(
                f,
                "density-half GMM fit failed (a substrate contract break — not a corpus shape)"
            ),
            #[cfg(feature = "set_refine")]
            Self::BadSetRefineTau => write!(
                f,
                "set_refine_gamma > 0 needs set_refine_tau > 0 — the σ(dot/τ) gate \
                 is undefined otherwise"
            ),
            #[cfg(feature = "set_refine")]
            Self::SetRefineKernel(e) => {
                write!(f, "set_refine kernel rejected the shapes: {e:?}")
            }
        }
    }
}

impl std::error::Error for EngineError {}

impl From<WireError> for EngineError {
    fn from(e: WireError) -> Self {
        Self::Wire(e)
    }
}

/// Why a slot abstained — Issue 060's closed taxonomy. The score gate is
/// evaluated first, so `DistanceGate` means the state PASSED the score
/// threshold and failed only the corpus-distance gate (the marginal cause;
/// per-cause shares sum to the abstain total). `GrammarInvalid` is the
/// serve-lane fall-through arm (game heads) and never occurs on the wire
/// path — every wire question is grammar-valid by the wire contract — so
/// the engine never sets it; the harness results field carries it as the
/// reserved third key of the closed set. `DensityGate` (Issue 066) is the
/// density half's marginal arm: the state passed score AND distance and
/// failed only the in-support-ness axis — the engine emits it only behind
/// the `density_gate` feature with the knob armed; every other build
/// carries it as the reserved fourth key (the `GrammarInvalid` posture,
/// fixed wire shape across feature postures).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AbstainCause {
    #[default]
    Answered,
    ScoreGate,
    DistanceGate,
    GrammarInvalid,
    DensityGate,
}

impl AbstainCause {
    /// True when the fused gate withheld the answer.
    pub fn abstained(self) -> bool {
        !matches!(self, AbstainCause::Answered)
    }

    /// Stable wire name — the Issue-060 closed-taxonomy keys, exposed for
    /// the Issue-072 per-item dump (never a display string).
    pub fn as_str(self) -> &'static str {
        match self {
            AbstainCause::Answered => "answered",
            AbstainCause::ScoreGate => "score_gate",
            AbstainCause::DistanceGate => "distance_gate",
            AbstainCause::GrammarInvalid => "grammar_invalid",
            AbstainCause::DensityGate => "density_gate",
        }
    }
}

/// One answered slot — the zero-alloc core's per-question verdict. The
/// probabilities live flat in [`Scratch::probs`] at `[prob_lo, prob_lo +
/// prob_len)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// True when the fused gate abstained (wire outcome `None`).
    pub abstained: bool,
    /// WHY the fused gate abstained (Issue 060's closed taxonomy; `Answered`
    /// when it did not) — `abstained == cause.abstained()` by construction.
    pub cause: AbstainCause,
    /// Winner index (option order; `noul` picks 0="yes" / 1="no").
    pub pick: u32,
    /// Calibrated confidence (the readout dispatch, then the calibrator).
    pub confidence: f32,
    /// Flat-probability range start.
    pub prob_lo: usize,
    /// Flat-probability range length (the question's arity).
    pub prob_len: usize,
    /// Issue 036 T1: the question ran its WEAKEST scorer — a choice/score
    /// with neither route path armed (options neither all resolve to
    /// domains by name nor `k == N`), so the score is the byte-level
    /// drafter delta alone, which carries an option-length prior and often
    /// ranks by accident on dynamic option spaces. Noul never counts (its
    /// no-route posture is by design, issue 030). Surfaced via
    /// [`DecisionEngine::routing_reason`] so any wire caller can see it.
    pub drafter_only: bool,
}

/// Caller-owned scratch: pre-allocated once, cleared and reused per call —
/// the zero-alloc core's whole working set.
#[derive(Debug)]
pub struct Scratch<const D: usize> {
    /// The embedded state+prompt of the question being solved.
    pub q: [f32; D],
    /// Per-question chosen domain (harness/introspection surface).
    pub domains: Vec<usize>,
    /// Per-question verdicts.
    pub slots: Vec<Slot>,
    /// Flat per-question probability vectors.
    pub probs: Vec<f32>,
    ctx: Vec<u8>,
    cand: Vec<u8>,
    scores: Vec<f32>,
    /// Hashed state tokens for the count tables (issue 038).
    #[cfg(feature = "nb_scope")]
    nb_tok: Vec<u32>,
    /// Option-conditioned per-option in-scores + terms (issue 038 T7b).
    #[cfg(feature = "option_cond")]
    oc_in: Vec<Option<f32>>,
    #[cfg(feature = "option_cond")]
    oc_terms: Vec<f32>,
    /// Ridge readout state tokens (bag view always) + per-domain scores
    /// (issue 038 T7a). The scores buffer is per-question scratch sized to
    /// the domain count at first use (the engine hands it back each call).
    #[cfg(feature = "nb_ridge")]
    ridge_tok: Vec<u32>,
    #[cfg(feature = "nb_ridge")]
    ridge_in: Vec<f32>,
    #[cfg(feature = "nb_ridge")]
    ridge_terms: Vec<f32>,
}

impl<const D: usize> Default for Scratch<D> {
    fn default() -> Self {
        Self {
            q: [0.0; D],
            domains: Vec::new(),
            slots: Vec::new(),
            probs: Vec::new(),
            ctx: Vec::new(),
            cand: Vec::new(),
            scores: Vec::new(),
            #[cfg(feature = "nb_scope")]
            nb_tok: Vec::new(),
            #[cfg(feature = "option_cond")]
            oc_in: Vec::new(),
            #[cfg(feature = "option_cond")]
            oc_terms: Vec::new(),
            #[cfg(feature = "nb_ridge")]
            ridge_tok: Vec::new(),
            #[cfg(feature = "nb_ridge")]
            ridge_in: Vec::new(),
            #[cfg(feature = "nb_ridge")]
            ridge_terms: Vec::new(),
        }
    }
}

impl<const D: usize> Scratch<D> {
    /// Empty scratch; call [`Scratch::prepare`] once the question count is
    /// known (reserve = one-time growth, then stable capacity).
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserve for `n_questions` (idempotent).
    pub fn prepare(&mut self, n_questions: usize) {
        self.domains.reserve(n_questions);
        self.slots.reserve(n_questions);
        // The option-conditioned buffers are per-QUESTION (cleared and
        // refilled per question), so they size to the max arity, not the
        // question count; 32 covers every suite's option set (one-time
        // growth, then stable capacity — the G4 law).
        #[cfg(feature = "option_cond")]
        {
            self.oc_in.reserve(32);
            self.oc_terms.reserve(32);
        }
        // The ridge token buffer likewise (one-time growth).
        #[cfg(feature = "nb_ridge")]
        self.ridge_tok.reserve(512);
        self.reset();
    }

    fn reset(&mut self) {
        self.domains.clear();
        self.slots.clear();
        self.probs.clear();
        self.ctx.clear();
        self.cand.clear();
        self.scores.clear();
        #[cfg(feature = "option_cond")]
        {
            self.oc_in.clear();
            self.oc_terms.clear();
        }
        #[cfg(feature = "nb_ridge")]
        self.ridge_tok.clear();
    }
}

/// The decision engine: `N` corpus experts over `D`-dim latents.
pub struct DecisionEngine<const N: usize, const D: usize> {
    experts: [DomainExpert<D>; N],
    cfg: EngineConfig,
    calibrator: SigmoidGateCalibrator,
    embedder: Embedder,
    calibrated: bool,
    /// Fitted per-label heads (issue 030 lever 4) — `Some` only when the
    /// config asked for them AND the build had the corpora to fit from
    /// (`build_specs`). `None` + `head_scale > 0` is impossible: `build`
    /// refuses that combination (fail-closed, never a silent no-op).
    heads: Option<LabelHeads<N, D>>,
    /// Count tables (issue 038) — `Some` exactly when `nb_scale > 0` and
    /// the build had corpora (`build_specs`); `build` refuses otherwise.
    #[cfg(feature = "nb_scope")]
    nb: Option<NbScope>,
    /// Option-conditioned tables (issue 038 T7b) — `Some` exactly when
    /// `oc_scale > 0` and the build had events (`build_specs_oc`); every
    /// other path leaves it `None`.
    #[cfg(feature = "option_cond")]
    oc: Option<OptionCond>,
    /// NBSVM ridge rows (issue 038 T7a) — `Some` exactly when
    /// `ridge_scale > 0` and the build had corpora.
    #[cfg(feature = "nb_ridge")]
    ridge: Option<NbRidge>,
    /// Issue 066: the shared JL projector for the density half (one
    /// projection matrix serves every domain's gate — same frozen
    /// definition, one commitment). `None` when the knob is off.
    #[cfg(feature = "density_gate")]
    density_proj: Option<JlProjector<D, DENSITY_E>>,
    /// Plan 627 / Issue 090 C2 (`set_refine` feature): the refinement
    /// substrate — built exactly when the knob is armed
    /// (`set_refine_gamma > 0`) at this engine's `D`; `None` = the
    /// byte-identical posture (no allocation, no cost, no branch taken
    /// on the hot path beyond the `is_some` test).
    #[cfg(feature = "set_refine")]
    sr: Option<katgpt_core::set_refine::SetRefineScratch>,
    /// The flat option-key buffer (k rows of D — the options' resolved
    /// domain directions), reserved once at build for `N` options,
    /// cleared/refilled per question (steady-state zero-alloc — the G4
    /// law; `refine_state_into` itself never allocates with a
    /// pre-ensured scratch).
    #[cfg(feature = "set_refine")]
    sr_keys: Vec<f32>,
    /// The kernel config snapshot (d = D at build; tau/gamma from cfg).
    #[cfg(feature = "set_refine")]
    sr_cfg: katgpt_core::set_refine::SetRefineConfig,
}

impl<const N: usize, const D: usize> DecisionEngine<N, D> {
    /// Build from domain specs (documents per domain). Refuses the wrong
    /// domain count (`EngineError::DomainCount`) — the const-generic shape
    /// keeps every routing array on the stack.
    pub fn build(specs: Vec<DomainExpert<D>>, cfg: EngineConfig) -> Result<Self, EngineError> {
        if specs.len() != N {
            return Err(EngineError::DomainCount {
                have: specs.len(),
                want: N,
            });
        }
        if cfg.head_scale > 0.0 {
            return Err(EngineError::HeadsNeedCorpora);
        }
        #[cfg(feature = "density_gate")]
        if cfg.density_gate {
            return Err(EngineError::DensityNeedsCorpora);
        }
        #[cfg(feature = "nb_scope")]
        if cfg.nb_scale > 0.0 || cfg.nb_pair_scale > 0.0 {
            return Err(EngineError::NbNeedsCorpora);
        }
        #[cfg(feature = "option_cond")]
        if cfg.oc_scale > 0.0 {
            return Err(EngineError::OcNeedsEvents);
        }
        #[cfg(feature = "nb_ridge")]
        if cfg.ridge_scale > 0.0 {
            return Err(EngineError::RidgeNeedsCorpora);
        }
        Self::build_with_heads(specs, cfg, None)
    }

    /// The shared build tail: experts + config + an OPTIONAL fitted head
    /// set (the caller owns the fit-vs-none decision).
    fn build_with_heads(
        specs: Vec<DomainExpert<D>>,
        cfg: EngineConfig,
        heads: Option<LabelHeads<N, D>>,
    ) -> Result<Self, EngineError> {
        let have = specs.len();
        let experts: [DomainExpert<D>; N] = specs
            .try_into()
            .map_err(|_| EngineError::DomainCount { have, want: N })?;
        // Plan 627 (`set_refine` feature): fail-closed tau validation —
        // the σ(dot/τ) gate is undefined at τ ≤ 0; an armed knob with a
        // bad tau refuses at BUILD, never a NaN at solve time.
        #[cfg(feature = "set_refine")]
        if cfg.set_refine_gamma > 0.0 && cfg.set_refine_tau <= 0.0 {
            return Err(EngineError::BadSetRefineTau);
        }
        // Plan 627 (`set_refine` feature): snapshot the knob into kernel
        // config + substrate BEFORE `cfg` moves into the engine struct.
        #[cfg(feature = "set_refine")]
        let sr_scratch = (cfg.set_refine_gamma > 0.0)
            .then(|| katgpt_core::set_refine::SetRefineScratch::new(D, N));
        #[cfg(feature = "set_refine")]
        let sr_cfg = katgpt_core::set_refine::SetRefineConfig::new(
            cfg.set_refine_tau,
            cfg.set_refine_gamma,
            D,
        );
        let calibrator = SigmoidGateCalibrator::new(cfg.cal_capacity, cfg.cal_min_obs);
        Ok(Self {
            experts,
            cfg,
            calibrator,
            embedder: Embedder,
            calibrated: false,
            heads,
            #[cfg(feature = "nb_scope")]
            nb: None,
            #[cfg(feature = "option_cond")]
            oc: None,
            #[cfg(feature = "nb_ridge")]
            ridge: None,
            #[cfg(feature = "density_gate")]
            density_proj: None,
            #[cfg(feature = "set_refine")]
            sr: sr_scratch,
            #[cfg(feature = "set_refine")]
            sr_keys: Vec::with_capacity(N * D),
            #[cfg(feature = "set_refine")]
            sr_cfg,
        })
    }

    /// Build from authored corpus specs (the friendly constructor):
    /// embeds each domain's documents, derives routing centroids, refuses
    /// wrong domain counts and empty corpora — fail-closed before any
    /// request is ever served.
    pub fn build_specs(specs: Vec<ExpertSpec>, cfg: EngineConfig) -> Result<Self, EngineError> {
        Self::build_specs_impl(specs, cfg, &[])
    }

    /// [`DecisionEngine::build_specs`] plus the option-conditioned events
    /// (issue 038 T7b): when `oc_scale > 0`, one (qid, option) contrastive
    /// table per gold event group is fitted and every option of every
    /// question kind gains the blend term. An armed scale with EMPTY
    /// events refuses (fail-closed); an unarmed scale ignores the events
    /// entirely (byte-identical build).
    #[cfg(feature = "option_cond")]
    pub fn build_specs_oc(
        specs: Vec<ExpertSpec>,
        cfg: EngineConfig,
        events: &[crate::option_cond::OcEvent],
    ) -> Result<Self, EngineError> {
        Self::build_specs_impl(specs, cfg, events)
    }

    /// Overwrite the scoring-time blend scales in place (issue 038 T5
    /// genome lane): a coordinate move between sel-slice evals without an
    /// engine rebuild. The search builds ONCE with every lever's tables
    /// FITTED (positive placeholder scales force the fits), so any scale
    /// the genome walks is scoreable here. Fail-closed: a positive scale
    /// whose fitted tables are absent refuses ([`EngineError::ScaleNotFitted`])
    /// — never a silent no-op; scale 0 is always legal (each blend term is
    /// constant-or-zero there, argmax-neutral by construction).
    pub fn set_blend_scales(
        &mut self,
        route_scale: f32,
        head_scale: f32,
        nb_scale: f32,
        oc_scale: f32,
        ridge_scale: f32,
    ) -> Result<(), EngineError> {
        if head_scale > 0.0 && self.heads.is_none() {
            return Err(EngineError::ScaleNotFitted { scale: "head" });
        }
        #[cfg(feature = "nb_scope")]
        if nb_scale > 0.0 && self.nb.is_none() {
            return Err(EngineError::ScaleNotFitted { scale: "nb" });
        }
        #[cfg(not(feature = "nb_scope"))]
        if nb_scale > 0.0 {
            return Err(EngineError::ScaleNotFitted { scale: "nb" });
        }
        #[cfg(feature = "option_cond")]
        if oc_scale > 0.0 && self.oc.is_none() {
            return Err(EngineError::ScaleNotFitted { scale: "oc" });
        }
        #[cfg(not(feature = "option_cond"))]
        if oc_scale > 0.0 {
            return Err(EngineError::ScaleNotFitted { scale: "oc" });
        }
        #[cfg(feature = "nb_ridge")]
        if ridge_scale > 0.0 && self.ridge.is_none() {
            return Err(EngineError::ScaleNotFitted { scale: "ridge" });
        }
        #[cfg(not(feature = "nb_ridge"))]
        if ridge_scale > 0.0 {
            return Err(EngineError::ScaleNotFitted { scale: "ridge" });
        }
        self.cfg.route_scale = route_scale;
        self.cfg.head_scale = head_scale;
        // The gated fields only exist behind their features; the guards
        // above already refused every off-feature scale > 0, so skipping
        // the write here is behavior-identical to writing 0 (the value is
        // always 0 at this point on a gated-off feature).
        #[cfg(feature = "nb_scope")]
        {
            self.cfg.nb_scale = nb_scale;
        }
        #[cfg(feature = "option_cond")]
        {
            self.cfg.oc_scale = oc_scale;
        }
        #[cfg(feature = "nb_ridge")]
        {
            self.cfg.ridge_scale = ridge_scale;
        }
        Ok(())
    }

    /// The shared build tail of the spec constructors.
    fn build_specs_impl(
        specs: Vec<ExpertSpec>,
        cfg: EngineConfig,
        #[cfg(feature = "option_cond")] oc_events: &[crate::option_cond::OcEvent],
        #[cfg(not(feature = "option_cond"))] _oc_events: &[()],
    ) -> Result<Self, EngineError> {
        if specs.len() != N {
            return Err(EngineError::DomainCount {
                have: specs.len(),
                want: N,
            });
        }
        let mut experts: Vec<DomainExpert<D>> = Vec::with_capacity(N);
        let mut rows_all: Vec<Vec<[f32; D]>> = Vec::with_capacity(N);
        // Count tables fit BEFORE the specs are consumed (they read the
        // wider `nb_docs` set when present, `docs` otherwise).
        #[cfg(feature = "nb_scope")]
        let nb = (cfg.nb_scale > 0.0 || cfg.nb_pair_scale > 0.0).then(|| {
            let sets: Vec<&[String]> = specs
                .iter()
                .map(|s| s.nb_docs.as_deref().unwrap_or(&s.docs))
                .collect();
            NbScope::fit(&sets, cfg.nb_alpha, cfg.nb_view)
        });
        #[cfg(feature = "option_cond")]
        let oc = if cfg.oc_scale > 0.0 {
            if oc_events.is_empty() {
                return Err(EngineError::OcNeedsEvents);
            }
            Some(OptionCond::fit(oc_events, cfg.nb_alpha, cfg.nb_view))
        } else {
            None
        };
        // The ridge fit (issue 038 T7a) reads the SAME wide doc sets as
        // the nb tables — the pools are captured above as `specs`' nb_docs
        // — so fit from those sets (docs when nb_docs is absent).
        #[cfg(feature = "nb_ridge")]
        let ridge = if cfg.ridge_scale > 0.0 {
            let sets: Vec<&[String]> = specs
                .iter()
                .map(|s| s.nb_docs.as_deref().unwrap_or(&s.docs))
                .collect();
            Some(crate::nb_ridge::fit(&sets, cfg.ridge_lambda))
        } else {
            None
        };
        for (i, s) in specs.into_iter().enumerate() {
            if s.docs.is_empty() {
                return Err(EngineError::EmptyCorpus { domain: i });
            }
            // Embed once; the rows feed BOTH the expert (gate + centroid)
            // and — when asked — the head fit.
            let mut rows: Vec<[f32; D]> = Vec::with_capacity(s.docs.len());
            for doc in &s.docs {
                let mut v = [0.0f32; D];
                Embedder.embed_into(doc.as_bytes(), &mut v);
                rows.push(v);
            }
            let corpus = s.docs.join("\n");
            experts.push(DomainExpert::from_embedded(
                s.name,
                corpus.into_bytes(),
                &rows,
            ));
            rows_all.push(rows);
        }
        let heads = (cfg.head_scale > 0.0).then(|| LabelHeads::fit(&rows_all));
        // Issue 066: the density half's gate family — fitted only when the
        // knob is armed (zero cost and byte-identical otherwise).
        #[cfg(feature = "density_gate")]
        let (density_proj, density_gates) =
            fit_density_gates::<D>(&rows_all, cfg.density_gate, cfg.density_tau)?;
        #[cfg(feature = "density_gate")]
        for (e, g) in experts.iter_mut().zip(density_gates) {
            e.density = Some(g);
        }
        #[allow(unused_mut)]
        let mut engine = Self::build_with_heads(experts, cfg, heads)?;
        #[cfg(feature = "nb_scope")]
        {
            engine.nb = nb;
        }
        #[cfg(feature = "option_cond")]
        {
            engine.oc = oc;
        }
        #[cfg(feature = "nb_ridge")]
        {
            engine.ridge = ridge;
        }
        #[cfg(feature = "density_gate")]
        {
            engine.density_proj = density_proj;
        }
        Ok(engine)
    }

    /// The zero-alloc core: answer every question, writing verdicts into
    /// `sc`. Fails closed on an invalid request BEFORE any work.
    pub fn solve_into(
        &mut self,
        req: &DecisionRequest,
        sc: &mut Scratch<D>,
    ) -> Result<(), EngineError> {
        self.solve_sample_into(req, sc, None, None)
    }

    /// The single-sample core with an optional input-perturbation hook —
    /// [`Self::solve_into`] is this with `perturb == None` (byte-identical
    /// by construction: the hook is the only difference and `None` skips
    /// it entirely).
    ///
    /// When `perturb` is `Some`, BOTH embeddings the pipeline consumes are
    /// Bernoulli-bucket-dropped in place under the sample's derived seed
    /// (the context embedding `sc.q` that drives `pick_domain` routing, and
    /// the state-alone embedding that drives the route/head/count-table
    /// terms) — the input-perturbation ensemble's per-sample form (reflex
    /// Issue 055 / katgpt-core `perturbation_ensemble`). The byte-level
    /// drafter delta is UNAFFECTED by embedding perturbation — a
    /// drafter-only question has zero sample variance by construction, and
    /// the honest readout is a degenerate histogram, never a silent reroute.
    ///
    /// `additive` is the Thm-3 perturbation-certificate hook (Issue 086
    /// T3, `basis_cert`): `Some((direction, epsilon))` adds
    /// `epsilon * direction[i]` to BOTH embeddings in place, after embed,
    /// before every consumer. `None` skips it — byte-identical by
    /// construction (the `perturb` law one layer up). The direction is a
    /// shared-ref array reference, so the hook is allocation-free and
    /// applies unchanged to the per-request state embed and the
    /// per-question context embed.
    #[cfg_attr(
        not(all(feature = "mc_ensemble", feature = "basis_cert")),
        allow(unused_variables)
    )]
    pub(crate) fn solve_sample_into(
        &mut self,
        req: &DecisionRequest,
        sc: &mut Scratch<D>,
        perturb: Option<(u32, f32, u64)>,
        additive: Option<(&[f32; D], f32)>,
    ) -> Result<(), EngineError> {
        req.validate()?;
        sc.reset();
        // Stack-local routing directions (N×D floats, no allocation).
        let mut dirs = [[0.0f32; D]; N];
        for (o, e) in dirs.iter_mut().zip(self.experts.iter()) {
            *o = e.direction;
        }
        // Case-level state derivation (the open-jev-fast prefix-root shape,
        // issue 054's transferable technique — reflex Research 006): every
        // quantity below is a pure function of `req.state`, the shared
        // prefix of the request, and the pipeline used to re-derive it once
        // per QUESTION. Derived once per case here, the per-question path
        // maps through its own option→domain view. Outputs are bit-
        // identical (same bytes into the same pure functions — the hoist
        // changes WORK, never VALUES); the state hash/token pass runs once
        // per request instead of once per question, which is the modelless
        // engine's exact analogue of the source's root-node sharing (the
        // laya encoder could not take it — bidirectional attention couples
        // the shared span; a hashed bag has no coupling at all).
        let mut state_q = [0.0f32; D];
        self.embedder.embed_into(req.state.as_bytes(), &mut state_q);
        // The Thm-3 additive certificate hook (issue 086 T3): perturb the
        // state embedding BEFORE its route/head consumers derive from it —
        // every state-embedding consumer downstream sees the perturbed
        // vector, exactly as it would see any other input change.
        #[cfg(feature = "basis_cert")]
        if let Some((dir, eps)) = additive {
            for (qv, dv) in state_q.iter_mut().zip(dir.iter()) {
                *qv += eps * dv;
            }
        }
        // Unperturbed cosine terms — state alone (the route/heads shared
        // input). MC perturbation is per-question BY DESIGN (its seed folds
        // `qi`), so the perturbed arm re-derives from this base below — the
        // identical bytes the pre-hoist form embedded fresh per question.
        let mut route_base = [0.0f32; N];
        for (rt, dir) in route_base.iter_mut().zip(dirs.iter()) {
            let mut dot = 0.0f32;
            for (qv, dv) in state_q.iter().zip(dir.iter()) {
                dot += qv * dv;
            }
            *rt = exact_sigmoid(dot * self.cfg.route_scale);
        }
        // Per-domain fitted-head blends on the unperturbed state; a
        // question's head term is its resolved domain's entry.
        let mut head_base = [0.0f32; N];
        if let Some(heads) = self.heads.as_ref() {
            for (d, hb) in head_base.iter_mut().enumerate() {
                *hb = heads.blend_term(d, &state_q, self.cfg.head_scale);
            }
        }
        // Count-table state tokens + per-domain in-scores, UNPERTURBED by
        // construction: the nb family always re-hashed `req.state` fresh
        // (the perturbation hit the cosine state alone) — the hoist keeps
        // exactly that. One fill serves three consumers that each hashed
        // the state with the same view into this same buffer: the
        // route-nb arm, the noul polarity arm and the option-conditioned
        // arm.
        #[cfg(feature = "nb_scope")]
        let mut nb_in_base = [0.0f32; N];
        #[cfg(feature = "nb_scope")]
        let nb_state_tokens = self.nb.is_some();
        #[cfg(all(feature = "nb_scope", feature = "option_cond"))]
        let nb_state_tokens = nb_state_tokens || self.oc.is_some();
        #[cfg(feature = "nb_scope")]
        let nb_n_tok = if nb_state_tokens {
            view_tokens_into(self.cfg.nb_view, req.state.as_bytes(), &mut sc.nb_tok);
            if let Some(nb) = self.nb.as_ref() {
                nb.in_scores(&sc.nb_tok, &mut nb_in_base);
            }
            sc.nb_tok.len()
        } else {
            0
        };
        // Ridge state tokens + per-domain scores (bag view, unperturbed —
        // the same law as the count tables).
        #[cfg(feature = "nb_ridge")]
        if let Some(ridge) = self.ridge.as_ref() {
            crate::embed::hashed_tokens_into(
                req.state.as_bytes(),
                crate::nb_scope::NB_VOCAB,
                &mut sc.ridge_tok,
            );
            sc.ridge_in.clear();
            for d in 0..N {
                sc.ridge_in.push(ridge.in_score(d, &sc.ridge_tok));
            }
        }
        #[cfg(feature = "nb_ridge")]
        let r_temp = self.ridge.as_ref().map(NbRidge::temp).unwrap_or(0.0);
        for (qi, q) in req.questions.iter().enumerate() {
            // Context = state + prompt (+ criteria).
            sc.ctx.clear();
            sc.ctx.extend_from_slice(req.state.as_bytes());
            sc.ctx.push(b'\n');
            sc.ctx.extend_from_slice(q.prompt.as_bytes());
            if let Some(c) = &q.criteria {
                sc.ctx.push(b'\n');
                sc.ctx.extend_from_slice(c.as_bytes());
            }
            self.embedder.embed_into(&sc.ctx, &mut sc.q);
            // The Thm-3 additive certificate hook's second surface: the
            // per-question context embedding (drives `pick_domain` + the
            // distance gate). Applied AFTER embed, BEFORE every consumer —
            // byte-identical when the hook is `None`.
            #[cfg(feature = "basis_cert")]
            if let Some((dir, eps)) = additive {
                for (qv, dv) in sc.q.iter_mut().zip(dir.iter()) {
                    *qv += eps * dv;
                }
            }
            #[cfg(feature = "mc_ensemble")]
            if let Some((sample, p_drop, seed)) = perturb {
                let sseed = seed
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add((sample as u64) << 32)
                    .wrapping_add(qi as u64);
                let mut tmp = [0.0f32; D];
                katgpt_core::perturbation_ensemble::bucket_dropout_into(
                    &sc.q, &mut tmp, sseed, p_drop,
                );
                sc.q = tmp;
            }

            // Route to the corpus expert.
            let di = pick_domain::<N, D>(&sc.q, &dirs);

            // Score every candidate against the corpus.
            let k = match q.kind {
                QuestionKind::Noul => 2usize,
                QuestionKind::Choice | QuestionKind::Score => q.options.len(),
            };
            // Option-rank terms from the routing geometry: when domains
            // double as option classes (`k == N`), each label's corpus
            // centroid IS that option's model, and the STATE's cosine to it
            // ranks the options. The STATE alone, not state+prompt+criteria:
            // the per-family prompt and criteria carry every label word on
            // every case — shared mass that swamps a short state and tilts
            // all three cosines toward one accidental centroid (the
            // permissions residue of the degenerate-lane lesson). The corpus
            // docs are embedded alone, so state-alone is the apples-to-
            // apples comparison. The byte-level drafter delta cannot rank
            // options at all: a 4-byte option string never moves the
            // compressed length of a ~300-byte context, so drafter-only
            // scoring made every option identical and the argmax tie broke
            // to index 0 — a constant pick (Issue 004 T7). The blend
            // scale is a config knob (`route_scale`, sweep lever in
            // issue 013), not a silent constant. Stack-local;
            // zero-alloc law holds.
            // Case-level cosine terms (the unperturbed state base —
            // derived once per case above); the MC hook re-derives per
            // question from the perturbed base when armed.
            #[cfg(not(any(feature = "mc_ensemble", feature = "set_refine")))]
            let route_terms = route_base;
            #[cfg(any(feature = "mc_ensemble", feature = "set_refine"))]
            let mut route_terms = route_base;
            // Option → domain index. By NAME first (Issue 023): exact
            // byte-equality of the option string with a domain name, on a
            // stack array (k ≤ N is necessary for every option to resolve
            // to a distinct domain; duplicates are refused by `validate`).
            // Only a FULL resolution arms it — a partial map would give some
            // options a centroid term and others none, a bias not a signal.
            let mut opt_dom = [0usize; N];
            let by_name = self.cfg.option_name_route
                && !matches!(q.kind, QuestionKind::Noul)
                && k <= N
                && q.options.iter().zip(opt_dom.iter_mut()).all(|(o, slot)| {
                    match self.experts.iter().position(|e| e.name == *o) {
                        Some(d) => {
                            *slot = d;
                            true
                        }
                        None => false,
                    }
                });
            if !by_name && self.cfg.legacy_kn_route && k == N {
                // Index-anchored resolution (issue 079, bench 130's G3
                // finding): the blind `k == N` identity fill is REPLACED by
                // a NAME check — the binding is legal only when the domains
                // ARE the option indices ("0".."N-1"): the ordinal-score
                // corpora, sst5 the incumbent case (its domains are the
                // level-index strings, its questions Score, so position i
                // ↔ level i ↔ corpus i is the scale's own semantics — the
                // noul-precedent class), s1mb_score by its Plan-010 law.
                // SCORE ONLY: a choice option's identity is its CONTENT,
                // never its position — typed's exact defect (bench 128:
                // 4.70 pt median swing, 68.3% flips — option 0 of an
                // observability question read the customer_service
                // centroid). Byte-preserving everywhere: where the old
                // blind fill legitimately armed, an index-named domain set
                // resolves to the same identity map (sst5 digit-identical,
                // bench 130 G3 re-diff); where the names are not indices
                // (typed), nothing arms and the question reads drafter-
                // only + option_cond — the content-bound posture, perm-
                // invariant (bench 130 acceptance: 0.00/0.00 pt, ties
                // only). [`EngineConfig::legacy_kn_route`] gates this arm
                // (off = strict by-name only, the A/B instrument).
                let by_index = matches!(q.kind, QuestionKind::Score)
                    && self
                        .experts
                        .iter()
                        .enumerate()
                        .all(|(i, e)| e.name == i.to_string());
                if by_index {
                    for (i, slot) in opt_dom.iter_mut().enumerate() {
                        *slot = i;
                    }
                }
            }
            // Noul never takes route terms (issue 030): its `[yes, no]`
            // pair is question-semantic vocabulary, never a label list, so
            // the legacy `k == N` index alignment would map it onto label
            // corpora by arbitrary index. On prompt_injections (N == 2:
            // domain 0 = benign, domain 1 = injection) that alignment scored
            // "yes, injection" against the BENIGN centroid — an anti-signal
            // by construction, measured 0.4397 below the 0.50 chance floor
            // (the T7 addendum's recorded −4.3 pt regression). The by-name
            // path already excludes noul; the index-anchored arm excludes
            // it by kind.
            let route_active = !matches!(q.kind, QuestionKind::Noul)
                && (by_name
                    || (self.cfg.legacy_kn_route
                        && k == N
                        && matches!(q.kind, QuestionKind::Score)
                        && self
                            .experts
                            .iter()
                            .enumerate()
                            .all(|(i, e)| e.name == i.to_string())));
            // Issue 036 T1/T2: the weakest-scorer path — a choice/score
            // question whose option score is the raw drafter delta. Both
            // the disclosure flag and the [`EngineConfig::drafter_fix`]
            // corrections key on exactly this shape (noul's no-route
            // posture is by design, issue 030, and never counts).
            let drafter_only = !route_active && !matches!(q.kind, QuestionKind::Noul);
            // Issue 036 T2 (SharedPrefix): the LONGEST byte-prefix shared
            // by a MAJORITY of the options (≥ half, ≥ 4 bytes = a word) —
            // the dominant option family's shared head (cua: `fill ` across
            // ~26 of 29 options). Stripped from the options that carry it;
            // the rest score whole. The all-options common prefix is the
            // wrong rule on mixed option sets (check/click/skip share
            // nothing with the fills — the global prefix is empty) and a
            // global strip would be a no-op exactly where the fix targets.
            // Zero when the fix is off or no family qualifies.
            let common_prefix = if drafter_only && self.cfg.drafter_fix == DrafterFix::SharedPrefix
            {
                shared_prefix_of(&q.options)
            } else {
                0
            };
            // Fitted per-label heads (issue 030 lever 4): the same legality
            // guard as the route terms — a head row belongs to a LABEL's
            // corpus, and noul's `[yes, no]` is question vocabulary, never
            // a label list. `Some` + scale 0 is unrepresentable (the build
            // refuses it), so `heads.is_some()` IS the armed test.
            let head_on = route_active && self.heads.is_some();
            let mut head_terms = [0.0f32; N];
            #[cfg(feature = "nb_scope")]
            let nb_on = route_active && self.nb.is_some();
            #[cfg(feature = "nb_scope")]
            let mut nb_terms = [0.0f32; N];
            if route_active {
                // The case-level state base stands in for the per-question
                // fresh embed (bit-identical: same bytes, same pure
                // function). The MC hook perturbs it per (sample, question)
                // — the seed folds `qi`, unchanged — and the cosine + head
                // families re-run on the perturbed state, exactly as the
                // pre-hoist order. The set_refine hook composes AFTER it
                // (refine the EFFECTIVE state — perturbed when MC is armed,
                // the shared state otherwise).
                #[cfg(any(feature = "mc_ensemble", feature = "set_refine"))]
                let mut eff_state: Option<[f32; D]> = None;
                #[cfg(feature = "mc_ensemble")]
                if let Some((sample, p_drop, seed)) = perturb {
                    let sseed = seed
                        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        .wrapping_add((sample as u64) << 32)
                        .wrapping_add(0x5DEE_CE66_D000_0000 ^ (qi as u64));
                    let mut q_state = [0.0f32; D];
                    katgpt_core::perturbation_ensemble::bucket_dropout_into(
                        &state_q,
                        &mut q_state,
                        sseed,
                        p_drop,
                    );
                    for (rt, dir) in route_terms.iter_mut().zip(dirs.iter()) {
                        let mut dot = 0.0f32;
                        for (qv, dv) in q_state.iter().zip(dir.iter()) {
                            dot += qv * dv;
                        }
                        *rt = exact_sigmoid(dot * self.cfg.route_scale);
                    }
                    eff_state = Some(q_state);
                }
                // Plan 627 / Issue 090 C2 (`set_refine` feature): the
                // set-conditioned state refinement — THE option-interaction
                // seam. The question's OWN option set (resolved to domain
                // directions, `opt_dom`) refines the effective state before
                // the route/head terms derive from it, making every
                // state-geometry term set-dependent: `x' = x +
                // (γ/n)·Σ σ(dot(x,kᵢ)/τ)·(kᵢ−x)` (sigmoid, never softmax —
                // per-option independent gates; katgpt-core `set_refine`,
                // the Plan-354 composition). Per-QUESTION by necessity —
                // the option set is question state, not request state, so
                // the hoisted `route_base` cannot serve this arm and the
                // route terms re-derive here. Unarmed (`sr == None`: knob 0
                // or feature off) the block is skipped and `route_base`
                // stands — byte-identical (G3). Scope: this arm lives
                // INSIDE `route_active`, so noul (issue 030 — `[yes, no]`
                // is question vocabulary, never a label list) and
                // drafter-only questions never take it, by the same
                // legality guard as the route terms themselves.
                #[cfg(feature = "set_refine")]
                if let Some(sr) = self.sr.as_mut() {
                    #[cfg(feature = "mc_ensemble")]
                    let base: &[f32; D] = eff_state.as_ref().unwrap_or(&state_q);
                    #[cfg(not(feature = "mc_ensemble"))]
                    let base: &[f32; D] = &state_q;
                    // Keys = the options' resolved directions (rows of D).
                    // k ≤ N on every route-active shape (by-name needs a
                    // full resolve; the index arm is k == N), and the
                    // kernel `ensure`s its own scratch regardless.
                    self.sr_keys.clear();
                    for &od in opt_dom.iter().take(k) {
                        self.sr_keys.extend_from_slice(&dirs[od]);
                    }
                    let mut refined = [0.0f32; D];
                    katgpt_core::set_refine::refine_state_into(
                        base,
                        &self.sr_keys,
                        &mut refined,
                        &self.sr_cfg,
                        sr,
                    )
                    .map_err(EngineError::SetRefineKernel)?;
                    for (rt, dir) in route_terms.iter_mut().zip(dirs.iter()) {
                        let mut dot = 0.0f32;
                        for (qv, dv) in refined.iter().zip(dir.iter()) {
                            dot += qv * dv;
                        }
                        *rt = exact_sigmoid(dot * self.cfg.route_scale);
                    }
                    eff_state = Some(refined);
                }
                if let Some(heads) = self.heads.as_ref() {
                    for (ht, &d) in head_terms.iter_mut().zip(opt_dom.iter()).take(k) {
                        #[cfg(any(feature = "mc_ensemble", feature = "set_refine"))]
                        if let Some(qs) = eff_state.as_ref() {
                            *ht = heads.blend_term(d, qs, self.cfg.head_scale);
                            continue;
                        }
                        *ht = head_base[d];
                    }
                }
                // Count-table terms (issue 038): the STATE's hashed tokens
                // (the same state-alone rule as route + heads), one
                // in-scope score per domain, one term per offered option.
                #[cfg(feature = "nb_scope")]
                if self.nb.is_some() {
                    // The count-table terms read the case-level in-scores
                    // (the same unperturbed state scores the pre-hoist
                    // form recomputed per question).
                    for (nt, &d) in nb_terms.iter_mut().zip(opt_dom.iter()).take(k) {
                        *nt = NbScope::blend_term(&nb_in_base, d, nb_n_tok, self.cfg.nb_scale);
                    }
                    // Pairwise terms (issue 081 T2c): the option-set-scoped
                    // margin, NO per-token normalization — the O(1)-token
                    // signal the averaged family cannot express (the field
                    // doc on `nb_pair_scale` records the measured law).
                    // Only when armed (> 0) and every option resolved by
                    // name (route_active is this block's guard).
                    if self.cfg.nb_pair_scale > 0.0 && k >= 2 {
                        for i in 0..k {
                            let di = opt_dom[i];
                            // Max over the QUESTION'S OWN other option
                            // domains — never the whole corpus (the
                            // pin-steal law: a big unrelated domain wins
                            // max_other and saturates the margin).
                            let mut other = f32::NEG_INFINITY;
                            for j in 0..k {
                                if j != i {
                                    other = other.max(nb_in_base[opt_dom[j]]);
                                }
                            }
                            nb_terms[i] +=
                                self.cfg.nb_pair_scale * exact_sigmoid(nb_in_base[di] - other);
                        }
                    }
                }
            }
            // Noul count-table polarity (issue 038): only when the caller
            // configured which domain "yes" means.
            #[cfg(feature = "nb_scope")]
            let noul_nb: Option<(f32, f32)> =
                match (q.kind, self.nb.as_ref(), self.cfg.nb_noul_domain) {
                    (QuestionKind::Noul, Some(_), Some(d)) if d < N => {
                        let yes = NbScope::blend_term(&nb_in_base, d, nb_n_tok, self.cfg.nb_scale);
                        Some((yes, self.cfg.nb_scale - yes))
                    }
                    _ => None,
                };
            // Option-conditioned terms (issue 038 T7b): keyed by the
            // question id + the option's own string, armed for EVERY
            // question kind — the typed questions this lever targets are
            // drafter_only (route-geometry terms never reach them). A key
            // with no table contributes no term (a 0.0 add — bit-identical
            // to absent).
            #[cfg(feature = "option_cond")]
            let oc_present = self.oc.is_some();
            #[cfg(feature = "option_cond")]
            if let Some(oc) = self.oc.as_ref() {
                // State tokens read the case-level fill (the same view the
                // route-nb and noul arms consume — one hash pass per case,
                // identical bytes per question).
                let n_tok = sc.nb_tok.len();
                sc.oc_in.clear();
                for i in 0..k {
                    let opt: &str = if matches!(q.kind, QuestionKind::Noul) {
                        if i == 0 { "yes" } else { "no" }
                    } else {
                        q.options[i].as_str()
                    };
                    sc.oc_in.push(oc.in_score(q.id.as_str(), opt, &sc.nb_tok));
                }
                sc.oc_terms.clear();
                for (i, slot) in sc.oc_in.iter().enumerate() {
                    let term = match slot {
                        Some(s) => crate::option_cond::oc_blend_term(
                            *s,
                            crate::option_cond::best_other(&sc.oc_in, i),
                            n_tok,
                            self.cfg.oc_scale,
                        ),
                        None => 0.0,
                    };
                    sc.oc_terms.push(term);
                }
            }
            // NBSVM ridge terms (issue 038 T7a): per-DOMAIN scores on the
            // bag token stream, option-mapped through `opt_dom` — the same
            // legality guard as the heads/nb (the class IS the domain), so
            // route_active is required.
            #[cfg(feature = "nb_ridge")]
            let ridge_on = route_active && self.ridge.is_some();
            #[cfg(feature = "nb_ridge")]
            if ridge_on {
                // Tokens + per-domain scores come from the case-level
                // fill; the per-question work is only the option mapping.
                if ridge_debug_enabled() {
                    eprintln!(
                        "[ridge-dbg] n_tok={} ridge_in={:?}",
                        sc.ridge_tok.len(),
                        sc.ridge_in
                    );
                }
                sc.ridge_terms.clear();
                sc.ridge_terms.resize(k, 0.0);
                for (i, &dom) in opt_dom.iter().take(k).enumerate() {
                    sc.ridge_terms[i] =
                        NbRidge::blend_term(&sc.ridge_in, dom, r_temp, self.cfg.ridge_scale);
                }
            }
            sc.scores.clear();
            // Each option's route term is its RESOLVED domain's cosine
            // (`opt_dom`: by name, or the index-anchored identity map —
            // score questions on index-named domains, gated by
            // [`EngineConfig::legacy_kn_route`], issue 079).
            let mut terms = opt_dom[..k.min(N)].iter().map(|&d| route_terms[d]);
            // k can EXCEED N when heads are disarmed (noul k=2 on an N=1
            // engine; a choice with more options than domains) — the loop
            // must run over the OPTIONS, never over the N-length head-term
            // array. clippy's needless_range_loop rewrite (iterate the
            // array) is the k>N truncation bug; the allow is the fix, not
            // the suppression.
            #[allow(clippy::needless_range_loop)]
            for i in 0..k {
                sc.cand.clear();
                match q.kind {
                    QuestionKind::Noul => {
                        sc.cand
                            .extend_from_slice(if i == 0 { b"yes" as &[u8] } else { b"no" })
                    }
                    _ => {
                        // Issue 036 T2: the drafter-only candidate-byte
                        // rewrites (SharedPrefix strips the options' common
                        // head; KeyOnly keeps the key before `: `). The
                        // route-armed path never sees either (the rewrites
                        // key on drafter_only).
                        let full = q.options[i].as_bytes();
                        let stripped =
                            if drafter_only && self.cfg.drafter_fix == DrafterFix::SharedPrefix {
                                &full[common_prefix..]
                            } else if drafter_only && self.cfg.drafter_fix == DrafterFix::KeyOnly {
                                // The bytes up to the first `: ` (the key);
                                // options without one score whole.
                                match find_subslice(full, b": ") {
                                    Some(pos) => &full[..pos],
                                    None => full,
                                }
                            } else {
                                full
                            };
                        sc.cand.extend_from_slice(stripped);
                    }
                }
                // The zero-alloc hot scorer (substrate: katgpt-core
                // `score_into`, landed for THIS lane's G4).
                let s_raw = self.experts[di].drafter.score_into(&sc.ctx, &sc.cand);
                // Issue 036 T2: the drafter-only delta corrections
                // (PerByte cancels the option-length prior; Ncd subtracts
                // the candidate's own standalone compressed cost — the
                // context-CONDITIONED term remains).
                let s = if drafter_only {
                    match self.cfg.drafter_fix {
                        DrafterFix::PerByte => (s_raw as f32 / sc.cand.len().max(1) as f32) as i32,
                        DrafterFix::Ncd => {
                            s_raw - self.experts[di].drafter.score_into(&[], &sc.cand)
                        }
                        _ => s_raw,
                    }
                } else {
                    s_raw
                };
                let mut score = exact_sigmoid(s as f32 / self.cfg.score_temperature);
                if route_active {
                    score += terms
                        .next()
                        .expect("route terms resolve for every option when route_active");
                }
                if head_on {
                    // head_on ⇒ route_active ⇒ (by_name with k ≤ N)
                    // or (k == N): the armed head terms cover every
                    // option index.
                    score += head_terms[i];
                }
                // nb_on ⇒ route_active: the same coverage argument.
                #[cfg(feature = "nb_scope")]
                if nb_on {
                    score += nb_terms[i];
                }
                // Engine noul order is [yes, no].
                #[cfg(feature = "nb_scope")]
                if let Some((yes, no)) = noul_nb {
                    score += if i == 0 { yes } else { no };
                }
                // Option-conditioned term (issue 038 T7b): present ⇒ the
                // per-option terms were computed for this question; a
                // missing key already folded to the no-op 0.0.
                #[cfg(feature = "option_cond")]
                if oc_present {
                    score += sc.oc_terms[i];
                }
                // NBSVM ridge term (issue 038 T7a): the option's domain
                // row; only when route terms are active (ridge_on ⇒
                // route_active ⇒ every option index < k has a resolved
                // domain).
                #[cfg(feature = "nb_ridge")]
                if ridge_on {
                    score += sc.ridge_terms[i];
                }
                sc.scores.push(score);
            }

            // Sigmoid scores → L1-normalized distribution (never softmax).
            let lo = sc.probs.len();
            let sum: f32 = sc.scores.iter().sum();
            if sum > 0.0 {
                sc.probs.extend(sc.scores.iter().map(|p| p / sum));
            } else {
                let u = 1.0 / k as f32;
                sc.probs.extend(std::iter::repeat_n(u, k));
            }

            // Winner + calibrated confidence + fused abstain.
            let mut best = 0usize;
            for i in 1..k {
                if sc.probs[lo + i] > sc.probs[lo + best] {
                    best = i;
                }
            }
            let raw = readout::confidence_with(self.cfg.readout, &sc.probs[lo..lo + k]);
            let conf = self.calibrator.apply(raw);
            // Issue 060: classify WHY the fused gate abstains — same
            // predicate as the fused OR, with score-gate precedence. The
            // short-circuit is preserved (the distance gate's reference-row
            // scan is skipped when the score gate already fired), so
            // `DistanceGate` means the state PASSED the score threshold and
            // failed only the corpus-distance gate — the marginal cause; the
            // shares sum to the abstain total.
            let score_declined = conf < self.cfg.score_threshold;
            // Issue 066: the density half — the LAST marginal arm of the
            // chain, so its projection + pair eval only run when score AND
            // distance passed (the Issue-060 short-circuit law). See
            // [`Self::density_declined`] — unarmed/unfitted/feature-off all
            // read `false` (never abstains; the `contrastive_scope`
            // posture).
            let cause = if score_declined {
                AbstainCause::ScoreGate
            } else if self.experts[di].gate.abstain_confidence(&sc.q) < self.cfg.distance_threshold
            {
                AbstainCause::DistanceGate
            } else if self.density_declined(di, &sc.q) {
                AbstainCause::DensityGate
            } else {
                AbstainCause::Answered
            };
            let abstained = cause.abstained();
            sc.slots.push(Slot {
                abstained,
                cause,
                pick: best as u32,
                confidence: conf,
                prob_lo: lo,
                prob_len: k,
                drafter_only: !route_active && !matches!(q.kind, QuestionKind::Noul),
            });
            sc.domains.push(di);
        }
        Ok(())
    }

    /// Wire materialization: build the response (allocating by design —
    /// the wire types own their payload) and validate it against the
    /// request (fail-closed before it ever leaves the process).
    pub fn to_response(&self, req: &DecisionRequest, sc: &Scratch<D>) -> DecisionResponse {
        let mut answers = Vec::with_capacity(sc.slots.len());
        for (q, slot) in req.questions.iter().zip(sc.slots.iter()) {
            // `noul` carries exactly ONE probability on the wire (p_yes) —
            // the internal yes/no candidate pair stays internal (the
            // confidence readout still sees the full 2-distribution).
            let probabilities = if q.kind == QuestionKind::Noul {
                vec![sc.probs[slot.prob_lo]]
            } else {
                sc.probs[slot.prob_lo..slot.prob_lo + slot.prob_len].to_vec()
            };
            let outcome = if slot.abstained {
                None
            } else {
                Some(match q.kind {
                    QuestionKind::Noul => Outcome::Noul {
                        yes: slot.pick == 0,
                    },
                    QuestionKind::Choice => Outcome::Choice { index: slot.pick },
                    QuestionKind::Score => Outcome::Score { level: slot.pick },
                })
            };
            answers.push(Answer {
                question_id: q.id.clone(),
                outcome,
                probabilities,
                confidence: slot.confidence,
            });
        }
        let (temperature, _bias) = self.calibrator.params();
        let (method, temperature) = if self.calibrated {
            ("sigmoid-gate".to_string(), temperature)
        } else {
            ("none".to_string(), 1.0)
        };
        let response = DecisionResponse {
            answers,
            routing: Routing {
                lane: Lane::Modelless,
                reason: Some(self.routing_reason(sc)),
            },
            calibration: Calibration {
                method,
                temperature,
            },
        };
        debug_assert!(
            response.validate_against(req).is_ok(),
            "engine output must satisfy the wire contract"
        );
        response
    }

    /// Convenience: [`Self::solve_into`] + [`Self::to_response`] with a
    /// fresh scratch (the cold-path shape; hot callers reuse scratch).
    pub fn decide(&mut self, req: &DecisionRequest) -> Result<DecisionResponse, EngineError> {
        let mut sc = Scratch::<D>::new();
        sc.prepare(req.questions.len());
        self.solve_into(req, &mut sc)?;
        Ok(self.to_response(req, &sc))
    }

    /// Hot-path convenience with a CALLER-OWNED scratch.
    pub fn decide_with(
        &mut self,
        req: &DecisionRequest,
        sc: &mut Scratch<D>,
    ) -> Result<DecisionResponse, EngineError> {
        self.solve_into(req, sc)?;
        Ok(self.to_response(req, sc))
    }

    /// Outcome feedback: observe `(reported confidence, was correct)`.
    /// Returns `true` when the observation moved the calibration (the
    /// response's `Calibration` metadata flips to `sigmoid-gate`).
    pub fn observe(&mut self, p: f32, outcome: bool) -> bool {
        self.calibrator.observe(p, outcome);
        let moved = self.calibrator.refit();
        self.calibrated |= moved;
        moved
    }

    /// The `Calibration` surface the arena tables read.
    pub fn calibration(&self) -> Calibration {
        let (temperature, _bias) = self.calibrator.params();
        if self.calibrated {
            Calibration {
                method: "sigmoid-gate".to_string(),
                temperature,
            }
        } else {
            Calibration::none()
        }
    }

    /// Domain names in routing order.
    pub fn domain_names(&self) -> Vec<&str> {
        self.experts.iter().map(|e| e.name()).collect()
    }

    /// Issue 090 C1 (route-agreement probe): the SEMANTIC route of an
    /// arbitrary text under this engine's routing table — the same
    /// unit-centroid argmax (`pick_domain`) the decide path uses, over
    /// the text embedded ALONE (the apples-to-apples surface: corpus
    /// docs are embedded alone). Read-only instrumentation: no engine
    /// state touched, no allocation beyond the stack-local embedding.
    pub fn semantic_route(&self, text: &str) -> usize {
        let mut dirs = [[0.0f32; D]; N];
        for (o, e) in dirs.iter_mut().zip(self.experts.iter()) {
            *o = e.direction;
        }
        let mut q = [0.0f32; D];
        self.embedder.embed_into(text.as_bytes(), &mut q);
        pick_domain::<N, D>(&q, &dirs)
    }

    /// Read-only gate access (harness calibration — thresholds from
    /// measured geometry, never magic numbers).
    pub fn gate(&self, domain: usize) -> &CorpusDistanceGate<D> {
        &self.experts[domain].gate
    }

    /// Issue 066 (`density_gate` feature): the density half's admission
    /// predicate — `sigmoid(ℓ(x)/τ) < density_threshold` on the routed
    /// domain's two-density gate. Unarmed, unfitted, or feature-off all
    /// read `false`: the gate never abstains on a build that did not ask
    /// for it (the `contrastive_scope` posture, pinned by test).
    #[cfg(feature = "density_gate")]
    fn density_declined(&self, di: usize, q: &[f32; D]) -> bool {
        self.cfg.density_gate
            && match (
                self.experts[di].density.as_ref(),
                self.density_proj.as_ref(),
            ) {
                (Some(gate), Some(proj)) => {
                    let mut x = [0.0f32; DENSITY_E];
                    proj.project(q, &mut x);
                    gate.confidence(&x) < self.cfg.density_threshold
                }
                _ => false,
            }
    }

    /// Feature-off spelling of [`Self::density_declined`] — the chain in
    /// [`Self::solve_sample_into`] compiles identically at every posture
    /// and the reserved variant is simply never emitted.
    #[cfg(not(feature = "density_gate"))]
    fn density_declined(&self, _di: usize, _q: &[f32; D]) -> bool {
        false
    }

    /// Issue 066 (`density_gate` feature): the density half's confidence
    /// `sigmoid(ℓ(x)/τ)` at a domain for an ALREADY-EMBEDDED question —
    /// the harness's threshold-fit observation axis (the ρ=30 percentile
    /// posture shared with the other two axes). `None` when the lane is
    /// unarmed or unfitted (the observation is then simply absent, never
    /// fabricated).
    #[cfg(feature = "density_gate")]
    #[must_use]
    pub fn density_confidence(&self, dom: usize, q: &[f32; D]) -> Option<f32> {
        let gate = self.experts.get(dom)?.density.as_ref()?;
        let proj = self.density_proj.as_ref()?;
        let mut x = [0.0f32; DENSITY_E];
        proj.project(q, &mut x);
        Some(gate.confidence(&x))
    }

    /// The fitted count tables (issue 038), when armed — the seat for the
    /// hybrid fusion terms (riir-instinct Issue 005 H2: the per-option
    /// in-scope margin + the evidence count). Read-only; the tables are
    /// frozen at build time.
    #[cfg(feature = "nb_scope")]
    #[must_use]
    pub fn nb_scope(&self) -> Option<&crate::nb_scope::NbScope> {
        self.nb.as_ref()
    }

    /// The option-conditioned tables (issue 038 T7b), when armed — the
    /// seat for the riir-instinct issue-005 hybrid's OPTION-space margin:
    /// one [`crate::option_cond::OptionCond`] table per (question id,
    /// option key), so a suite whose presented options are state-field
    /// values (typed_decisions) gets a per-option margin the domain
    /// tables' [`Self::nb_scope`] margin can never produce there. The
    /// [`Self::nb_scope`] accessor's exact sibling; read-only, frozen at
    /// build time.
    #[cfg(feature = "option_cond")]
    #[must_use]
    pub fn oc(&self) -> Option<&crate::option_cond::OptionCond> {
        self.oc.as_ref()
    }

    fn routing_reason(&self, sc: &Scratch<D>) -> String {
        let mut counts = [0usize; N];
        for d in &sc.domains {
            counts[*d] += 1;
        }
        let mut s = String::from("modelless corpus routing:");
        for (i, e) in self.experts.iter().enumerate() {
            s.push_str(&format!(" {}={}", e.name(), counts[i]));
        }
        s.push_str("; fused abstain (score+distance) armed");
        // Issue 036 T1: disclose the weakest-scorer posture — a caller whose
        // request mints per-request options (neither all resolving to domain
        // names nor k == N) must be able to SEE that the engine answered
        // from the drafter delta alone.
        let drafter_only = sc.slots.iter().filter(|s| s.drafter_only).count();
        if drafter_only > 0 {
            s.push_str(&format!("; drafter-only={drafter_only}"));
        }
        s
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;

#[cfg(all(test, feature = "density_gate"))]
mod density_tests {
    use super::*;
    use crate::embed::EMBED_DIM;
    use katgpt_core::decision_wire::Question;

    fn corpus_specs() -> Vec<ExpertSpec> {
        vec![
            ExpertSpec::new(
                "deploy",
                &[
                    "deploy the server to staging and verify the rollout; the cluster is ready"
                        .to_string(),
                ],
            ),
            ExpertSpec::new(
                "weather",
                &["rain forecast sunny cloudy temperature tomorrow umbrella".to_string()],
            ),
            ExpertSpec::new(
                "music",
                &["play the song album playlist artist track shuffle".to_string()],
            ),
            ExpertSpec::new(
                "billing",
                &["refund the customer invoice balance billing account".to_string()],
            ),
        ]
    }

    fn choice_request(state: &str) -> DecisionRequest {
        DecisionRequest {
            state: state.to_string(),
            questions: vec![Question::choice(
                "q0",
                "which intent?",
                ["deploy", "weather", "music", "billing"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                None,
            )],
        }
    }

    /// The pairing premise the Issue-066 A/B stands on: the density half
    /// NEVER touches scoring — identical probabilities and picks armed vs
    /// unarmed; only the abstain flags may differ.
    #[test]
    fn density_gate_never_moves_picks_or_probs() {
        let mut off =
            DecisionEngine::<4, EMBED_DIM>::build_specs(corpus_specs(), EngineConfig::default())
                .unwrap();
        let mut on = DecisionEngine::<4, EMBED_DIM>::build_specs(
            corpus_specs(),
            EngineConfig {
                density_gate: true,
                ..EngineConfig::default()
            },
        )
        .unwrap();
        let req = choice_request("play the song album playlist");
        let (mut a, mut b) = (Scratch::new(), Scratch::new());
        off.solve_into(&req, &mut a).unwrap();
        on.solve_into(&req, &mut b).unwrap();
        assert_eq!(
            a.probs, b.probs,
            "the gate must never touch the distribution"
        );
        assert_eq!(
            a.slots[0].pick, b.slots[0].pick,
            "the gate must never touch the pick"
        );
        assert_eq!(a.domains, b.domains, "the gate must never touch routing");
    }

    /// The knob-off half of the byte-identity posture: an unarmed build
    /// never fits (the accessor reads `None`) and answers exactly as the
    /// feature-off build does (same code path — short-circuited at the
    /// first cfg read).
    #[test]
    fn density_unarmed_never_fits_and_never_abstains() {
        let mut eng =
            DecisionEngine::<4, EMBED_DIM>::build_specs(corpus_specs(), EngineConfig::default())
                .unwrap();
        assert!(
            eng.density_confidence(0, &[0.0f32; EMBED_DIM]).is_none(),
            "an unarmed build must never expose a density reading"
        );
        let req = choice_request("rain forecast tomorrow");
        let mut sc = Scratch::new();
        eng.solve_into(&req, &mut sc).unwrap();
        // The shipped birth thresholds on a 1-doc corpus may fire the
        // score gate — this test pins the DENSITY posture only, so pin
        // via the armed-thresholds-0 twin below instead of the cause.
        let _ = sc.slots[0].cause;
    }

    /// The marginal-cause classification: thresholds 0 on score+distance
    /// (they never fire — isolating the chain's tail) and a density
    /// threshold ABOVE every possible confidence (sigmoid ≤ 1 < 2) fires
    /// the density arm exactly on the questions that reached it.
    #[test]
    fn density_gate_classifies_the_marginal_cause() {
        let mut eng = DecisionEngine::<4, EMBED_DIM>::build_specs(
            corpus_specs(),
            EngineConfig {
                density_gate: true,
                density_threshold: 2.0,
                score_threshold: 0.0,
                distance_threshold: 0.0,
                ..EngineConfig::default()
            },
        )
        .unwrap();
        let req = choice_request("refund the customer invoice balance");
        let mut sc = Scratch::new();
        eng.solve_into(&req, &mut sc).unwrap();
        let slot = &sc.slots[0];
        assert!(
            slot.abstained,
            "threshold 2.0 must abstain on everything that reaches the arm"
        );
        assert_eq!(slot.cause, AbstainCause::DensityGate);
        // And with a threshold of 0.0 (nothing fires), the same request
        // answers — the arm is really the threshold's, not a constant.
        let mut open = DecisionEngine::<4, EMBED_DIM>::build_specs(
            corpus_specs(),
            EngineConfig {
                density_gate: true,
                density_threshold: 0.0,
                score_threshold: 0.0,
                distance_threshold: 0.0,
                ..EngineConfig::default()
            },
        )
        .unwrap();
        let mut sc2 = Scratch::new();
        open.solve_into(&req, &mut sc2).unwrap();
        assert_eq!(sc2.slots[0].cause, AbstainCause::Answered);
    }

    /// The fail-closed law: raw-expert `build` (corpora already consumed)
    /// refuses an armed density knob — a gate that silently never fires is
    /// the flag-does-nothing bug class.
    #[test]
    fn build_refuses_density_without_corpora() {
        let built = DecisionEngine::<4, EMBED_DIM>::build(
            (0..4).map(one_domain_raw).collect(),
            EngineConfig {
                density_gate: true,
                ..EngineConfig::default()
            },
        );
        assert!(matches!(built, Err(EngineError::DensityNeedsCorpora)));
    }

    fn one_domain_raw(i: usize) -> DomainExpert<EMBED_DIM> {
        let docs = [
            "deploy the server to staging and verify the rollout",
            "rain forecast sunny cloudy temperature tomorrow",
            "play the song album playlist artist track",
            "refund the customer invoice balance billing account",
        ];
        DomainExpert::new(format!("d{i}"), &[docs[i].to_string()], &Embedder)
    }

    /// The exposed observation axis: deterministic, in (0, 1), and
    /// domain-discriminative enough to observe (an in-corpus embedding
    /// reads HIGHER than a disjoint one — the win surface's primitive
    /// form; the exact value is corpus-shape-dependent, never pinned).
    #[test]
    fn density_confidence_is_deterministic_and_bounded() {
        let eng = DecisionEngine::<4, EMBED_DIM>::build_specs(
            corpus_specs(),
            EngineConfig {
                density_gate: true,
                ..EngineConfig::default()
            },
        )
        .unwrap();
        let mut q = [0.0f32; EMBED_DIM];
        Embedder.embed_into(b"refund the customer invoice balance", &mut q);
        let a = eng
            .density_confidence(3, &q)
            .expect("armed build must expose a reading");
        let b = eng.density_confidence(3, &q).expect("same again");
        assert_eq!(a.to_bits(), b.to_bits(), "deterministic by construction");
        assert!((0.0..=1.0).contains(&a), "sigmoid output");
        // An out-of-domain index still reads (routing picks something for
        // every input; the gate is per-domain).
        assert!(eng.density_confidence(0, &q).is_some());
        assert!(
            eng.density_confidence(99, &q).is_none(),
            "out-of-range domain is None, never a panic"
        );
    }
}
