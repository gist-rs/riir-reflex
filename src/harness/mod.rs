//! Phase-1 benchmark harness (Plan 603 T1.5): suite builders, metrics, and
//! the runner that produces the honest per-task tables over both lanes.
//! Builders + metrics are pure (ungated); the runner consumes the modelless
//! engine, so it rides the same feature.
// Box-state provenance stamped into every run's meta (Issue 021 T7).
pub mod box_state;
// Issue 038 T4′ — the cascade lane's pure composer (ungated: no engine dep).
pub mod cascade;
pub mod families;
// (families_eval_wide removed 2026-10-02 with the six Issue-004 families'
// retirement — owner call; only `semantic_defects` remains, whose wide
// eval lives in its own sibling module.)
// Issue 061: the semantic_defects code-defect family (perch-derived
// taxonomy; original Rust fixtures — data-only, families-shaped).
pub mod families_semantic_defects;
pub mod families_semantic_defects_eval;
// The ONE fixture-hygiene checker (tokens/trigrams/overlap/digest) —
// shared by every in-process fixture authoring; see the module doc for
// the four-time recurrence record that forced it out of private copies.
pub mod fixture_hygiene;
// Issue 072 — measurement-only per-item outcome records (`--dump-items`;
// rethink 028 T1's instrument). Ungated + pure: reads materialized eval
// vectors, never runs an engine.
pub mod item_dump;
// Issue 077 — the option-PERMUTATION spread probe (pure core): orderings,
// permutation application, label-space mapping, spread math, the ≤ 2 pt
// gate, and the canary. Ungated + pure; the lanes live in the runner half
// (`runner/perm_probe.rs`) behind the `ChoiceOracle` seam.
pub mod permutation;
// Issue 090 A1–A3 — the decoy-augmentation / null-stability probe (pure
// core): graded decoy pool/sampler, augmentation + paired-noul builders,
// per-arm record math, and the A1/A2/A3 verdicts. Ungated + pure; the
// lanes live in the runner half (`runner/null_probe.rs`) behind the
// `NullOracle` seam.
pub mod null_stability;
// Issue 090 C1 — the rubric-covariance + route-agreement probe (pure
// core): the Σq·v identity math (I1 scale / I2 shift / I3 value-blind
// argmax / I4 normalization residual) + the route-agreement record.
// Ungated + pure; the lane lives in the runner half
// (`runner/rubric_probe.rs`).
pub mod rubric;
// Issue 086 T1 — the perm-pair presentation-subspace certificate (pure
// core): rendering, pair collection types, and the certify() call into
// katgpt-core `basis_split`. Gated with the lane (the feature forwards
// katgpt-core/basis_split); the runner half is `runner/basis_cert.rs`.
#[cfg(feature = "basis_cert")]
pub mod basis_cert;
// Issue 094 T1 — the input-keyed modal-answer floor (pure core): the
// leave-one-out anti-mimicry census the harness tables quote every lane
// row against. Ungated + pure (no engine dep); the render half lives in
// `runner.rs::render_markdown`.
pub mod modal_floor;
// Per-lane latency extremes: first / max / argmax case (Issue 020 T8).
pub mod latency;
// Issue 055 / Plan 008 T5 — the `--mc-ab` distributional-layer A/B arm
// (report-only; rides the wrapper's feature).
#[cfg(feature = "mc_ensemble")]
pub mod mc_ab;
pub mod metrics;
pub mod pair_heads;
// Issue 066 — the `--density-gate` fused-abstain density-half A/B arm
// (report-only; rides the engine feature).
#[cfg(feature = "density_gate")]
pub mod density_ab;
// Plan 627 / Issue 090 C2 — the `--set-refine-ab` set-conditioned state
// refinement A/B arm (report-only; rides the engine feature).
#[cfg(feature = "set_refine")]
pub mod set_refine_ab;
// Plan 621 Phase 5 T5.1 — the FlyBy paired counterfactual ΔV
// certification (the `--corpus-ab --probe-delta-ab` instrument; consumes
// katgpt-core `state_probe`). Pure over paired outcome slices; the
// corpus-ab lane is its first consumer.
#[cfg(feature = "probe_delta_ab")]
pub mod probe_delta;
// Issue 058 — the slice-integrity guard: hard assertions over the
// test/cal/pool slices (overlaps, test-sample label coverage) + the raw
// slice-identity digests the board publish path compares. Ungated + pure.
pub mod slice_guard;
// Near-duplicate leak report (Issue 024): opt-in, zero-dep, native-only.
#[cfg(feature = "modelless")]
pub mod runner;
#[cfg(all(feature = "slice_leak", not(target_arch = "wasm32")))]
pub mod slice_leak;
pub mod suites;
// Warm-tier persistence via the released `ndb` binary (Issue 007 P1):
// subprocess-only, zero storage-leaf cargo deps; NATIVE-ONLY by
// construction (std::process) — never join a wasm32 combo.
#[cfg(all(
    feature = "corpus_db",
    feature = "modelless",
    not(target_arch = "wasm32")
))]
pub mod corpus_db;
