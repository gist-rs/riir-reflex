//! `riir-reflex` — decision-engine serving + comparison + contribution.
//!
//! The Proposal 014 Phase 1 deliverable (Plan 603): typed decisions over the
//! LANDED katgpt-rs substrate. The wire contract
//! (`katgpt_core::decision_wire`) defines the request/response vocabulary;
//! this crate is the ENGINE behind it:
//!
//! - [`embed`] — the hashed-feature text → latent head (modelless, fixed
//!   width, deterministic);
//! - [`engine`] — routing (`pick_domain` over corpus centroids),
//!   corpus-is-the-model option scoring (`Lz4FlexDrafter`), sigmoid
//!   normalization (never softmax), calibrated confidence
//!   (`SigmoidGateCalibrator`), fused abstain (score + `CorpusDistanceGate`);
//! - [`readout`] — the confidence readout dispatch (Bench 817's verdict
//!   inherited: label-entropy for narrow option sets, argmax-label-prob for
//!   wide ones; agreement-across-rereads is NOT a signal on deterministic
//!   forwards);
//! - [`serve`] — the localhost HTTP edge (one std-only binary, no daemon
//!   framework; the hexagonal seam is the edge and ONLY the edge).
//! - [`laya`] — the native-Rust comparison lane (Plan 603 T1.4): the pinned
//!   laya checkpoints — MOVED substrate-side (the lane now lives in the
//!   inference substrate's `riir-infer-laya` crate; this is a `pub use`
//!   shim), G5-parity-gated from HERE (fixtures + consumer gate stay) before
//!   any published number.
//!
//! **Default build = ONE foreign code-level dep** (`katgpt-core`, see the
//! root `BOUNDARY.md`) plus the lane-substrate path dep (`riir-infer-laya`,
//! non-optional only because the UNGATED Python-JSON writer moved with the
//! lane — at default features it compiles just that writer); `serde_json`
//! is the HTTP/JSON edge only. No game crate is reachable from ANY feature
//! combination; no Python anywhere. The `laya` lane is opt-in and
//! self-contained (tokenizers + gemm + the metal tree are the SUBSTRATE
//! crate's deps behind its own features, never the modelless lane's).
//!
//! Gate posture (Plan 603 T1.1e): the `[[bin]]`, `[[test]]` and `[[bench]]`
//! rows carry `required-features` in the same commit as the gated code — the
//! silent-`ok. 0 passed` trap is closed at birth.

/// Crate version (the one ungated item — `--no-default-features` compiles
/// an intentionally near-empty lib; this is the tested flag-OFF posture).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Python-JSON byte-format writer (ungated DRY home — the laya lane's
/// sequence rendering and the harness's modelless-lane state strings MUST
/// be the same bytes; see `laya::render` for the lane-local re-export).
/// MOVED substrate-side: the ONE writer now lives in the inference
/// substrate's lane crate; this module re-exports it (same bytes, one
/// implementation).
pub mod pyjson;

/// The `--version` build stamp — compiled feature set vs the shipped
/// release set, with the loud STALE + rebuild line (ungated: the stamp must
/// render in every build, that is its whole point).
pub mod build_stamp;

/// The modelless decision engine (Plan 603 T1.3). Opt-out:
/// `--no-default-features`.
#[cfg(feature = "modelless")]
pub mod embed;

/// Confidence readout dispatch (Plan 603 T1.7 — inherit, don't re-derive).
#[cfg(feature = "modelless")]
pub mod readout;

/// The decision engine itself.
#[cfg(feature = "modelless")]
pub mod engine;

/// The localhost HTTP serving edge.
#[cfg(feature = "modelless")]
pub mod serve;

/// The fitted game head over HTTP (Plan 607's decoded Tetris head served
/// from the digest-pinned oracle fixture — the arena's modelless lane
/// plays; `.plans/001_game_head_serving.md`).
#[cfg(feature = "modelless")]
pub mod game_heads;

/// The laya native-Rust comparison lane (Plan 603 T1.4) — the riir-owned
/// forward (opt-in `laya-riir`). MOVED substrate-side (the consolidation
/// issue's encoder-lane move): the lane lives in the inference substrate's
/// `riir-infer-laya` crate; this module is a `pub use` shim (public API
/// unchanged). One tokenizer, one config parser, one download path, one
/// answer envelope — there now. The G5 parity fixtures + gate stay HERE.
#[cfg(feature = "laya-riir")]
pub mod laya;

/// Phase-1 benchmark harness (Plan 603 T1.5): suite builders, metrics, and
/// the runner that produces the honest per-task tables over both lanes.
/// Ungated: the builders/metrics are pure; the runner degrades loudly when
/// the `laya-riir` feature (or the datasets/weights) are absent.
pub mod harness;

/// The comparison-lane adapters (Issue 019): third-party decision engines
/// measured in the arena's tables over their own wire. Opt-in `clm-lane`
/// today; the module is empty without it. Their stack serves, our Rust
/// measures — no new deps either way (std HTTP + the in-tree serde_json).
#[cfg(feature = "clm-lane")]
pub mod lanes;
