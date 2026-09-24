//! Phase-1 benchmark harness (Plan 603 T1.5): suite builders, metrics, and
//! the runner that produces the honest per-task tables over both lanes.
//! Builders + metrics are pure (ungated); the runner consumes the modelless
//! engine, so it rides the same feature.
// Box-state provenance stamped into every run's meta (Issue 021 T7).
pub mod box_state;
pub mod families;
// Per-lane latency extremes: first / max / argmax case (Issue 020 T8).
pub mod latency;
pub mod metrics;
pub mod pair_heads;
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
