//! The comparison-lane adapters (Issues 019/029/025): third-party decision
//! engines measured in the arena's tables over their own wire — their
//! stack serves, our Rust measures. `agentjev` is ungated (std-HTTP/JSON
//! only, zero new deps — Issue 025 amendment 4); `clm` needs its feature
//! (its law copies pin katgpt-core's decision_wire surface, which
//! `--no-default-features` does not activate). `paw` (Issue 033) needs no
//! feature of its own: it rides `modelless` (its cache key is the
//! in-tree blake3) and is native-only (a `curl` subprocess transport).
//! `paw_local` (Issue 033 Posture B) shares those gates and adds the
//! Python-oracle subprocess (the gliner-lane shape).

pub mod agentjev;
#[cfg(feature = "clm-lane")]
pub mod clm;
#[cfg(all(feature = "modelless", not(target_arch = "wasm32")))]
pub mod paw;
#[cfg(all(feature = "modelless", not(target_arch = "wasm32")))]
pub mod paw_local;
// Plan 003 Phase 2: the OpenThai comparison lane — agentjev-shaped
// (ungated: imports only `crate::harness::suites` + std + serde_json, the
// G-ISO-4 import law; its tripwire test fires the recorded
// `openthai-lane` feature trigger if `decision_wire` is ever needed).
#[cfg(all(feature = "modelless", not(target_arch = "wasm32")))]
pub mod openthai;
