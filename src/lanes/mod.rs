//! The comparison-lane adapters (Issues 019/029/025): third-party decision
//! engines measured in the arena's tables over their own wire — their
//! stack serves, our Rust measures. `agentjev` is ungated (std-HTTP/JSON
//! only, zero new deps — Issue 025 amendment 4); `clm` needs its feature
//! (its law copies pin katgpt-core's decision_wire surface, which
//! `--no-default-features` does not activate).

pub mod agentjev;
#[cfg(feature = "clm-lane")]
pub mod clm;
