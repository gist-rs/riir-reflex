//! The riir-OWNED laya backend (feature `laya-riir`) — **no candle**.
//!
//! Owner directive (2026-09-22): the prod-path lane must build without
//! candle; say OUR name, not candle's. The model math is ripped from the
//! candle port (`crate::laya::encoder` / `crate::laya::head`) onto our own
//! flat-`Vec<f32>` tensor code; the ONE kernel dep is `gemm` 0.18 — the
//! exact crate + version candle's CPU backend calls, version-matched so
//! both lanes share one build (`.issues/002`, BOUNDARY.md row).
//!
//! Layout:
//! - [`weights`] — the minimal safetensors reader (F16→F32 bit-exact
//!   widening, F32 passthrough, F64 cast, every other dtype refused loud);
//! - [`ops`] — the ONE `gemm::` seam + the flat elementwise/reduction ops
//!   (LN, softmax, `gelu_erf`, rope, head split/merge) — candle's exact call
//!   shapes, so matching GEMMs are bit-identical;
//! - [`backend`] — the device seam (`.issues/005`): the ~15 ops the forward
//!   needs, ONE forward body in [`encoder`] / [`head`], `Cpu` delegating 1:1
//!   to [`ops`];
//! - [`metal`] — the MSL backend (feature `laya-riir-metal`, macOS;
//!   the default posture on those builds, `LAYA_DEVICE=cpu` opts out —
//!   fail loud when the feature/platform is absent) — the same op semantics
//!   over Metal kernels, candle's METAL erf transcribed verbatim;
//! - [`encoder`] / [`head`] — the forwards, ported in the candle port's
//!   op order (the normative math the G5 gate was measured against);
//! - [`agent`] — `RiirAgent`, the candle agent's envelope semantics over
//!   the shared substrate (`types` / `temps` / `tokenize` / `weights`-
//!   download / `render` — one copy for both backends); the device is
//!   chosen at load from `LAYA_DEVICE` (Metal default on macOS metal
//!   builds — Plan 001 T4; CPU elsewhere; explicit env always wins).
//!
//! G5 parity: `tests/laya_riir_parity.rs` gates the SAME fixture corpus and
//! expected capture as the candle lane (top-1 ≥ 99.9 %, p-drift ≤ 1e-3, per
//! checkpoint). No number from this lane is published before that gate is
//! green AT THAT POSTURE — the CPU posture was the landing gate; the Metal
//! posture was greened by `.issues/005` T4 before any riir-Metal number
//! was published.
//!
//! Deps (both version-matched to candle's own calls, `.issues/002` +
//! `.issues/003`, both CLOSED at the lane landing): `gemm` (the GEMM) and
//! `libm` (`erff` for the CPU gelu) — the lane itself stays candle-free;
//! `metal` + `objc2` ride `laya-riir-metal` only (`.issues/005`,
//! version-matched to candle's Metal lane, macOS-only).

pub mod agent;
pub mod backend;
pub mod encoder;
pub mod head;
pub mod ops;
pub mod weights;

/// The Apple Metal backend (`laya-riir-metal`, macOS) — compiles to nothing
/// everywhere else.
#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
pub mod metal;

pub use agent::RiirAgent;
