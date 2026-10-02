//! Phase-1 benchmark harness (Plan 603 T1.5): suite builders, metrics, and
//! the runner that produces the honest per-task tables over both lanes.
//! Builders + metrics are pure (ungated); the runner consumes the modelless
//! engine, so it rides the same feature.
// Box-state provenance stamped into every run's meta (Issue 021 T7).
pub mod box_state;
// Issue 038 T4′ — the cascade lane's pure composer (ungated: no engine dep).
pub mod cascade;
// Issue 044 T4 — the FROZEN code_fixtures population (digest-pinned
// committed fixture + the original extractor; consumed by the runner).
#[cfg(feature = "modelless")]
pub mod code_frozen;
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
// Per-lane latency extremes: first / max / argmax case (Issue 020 T8).
pub mod latency;
// Issue 055 / Plan 008 T5 — the `--mc-ab` distributional-layer A/B arm
// (report-only; rides the wrapper's feature).
#[cfg(feature = "mc_ensemble")]
pub mod mc_ab;
pub mod metrics;
pub mod pair_heads;
// Issue 058 — the slice-integrity guard: hard assertions over the
// test/cal/pool slices (overlaps, test-sample label coverage) + the raw
// slice-identity digests the board publish path compares. Ungated + pure.
pub mod slice_guard;
// Near-duplicate leak report (Issue 024): opt-in, zero-dep, native-only.
#[cfg(all(feature = "slice_leak", not(target_arch = "wasm32")))]
pub mod slice_leak;
#[cfg(feature = "modelless")]
pub mod runner;
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
