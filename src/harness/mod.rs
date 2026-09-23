//! Phase-1 benchmark harness (Plan 603 T1.5): suite builders, metrics, and
//! the runner that produces the honest per-task tables over both lanes.
//! Builders + metrics are pure (ungated); the runner consumes the modelless
//! engine, so it rides the same feature.
pub mod families;
pub mod metrics;
pub mod suites;
#[cfg(feature = "modelless")]
pub mod runner;
