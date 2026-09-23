//! The laya lane: a native-Rust parity port of the pinned laya reference
//! (`github.com/NandhaKishorM/laya` @ `573e5b62…`, release 0.3.5), for the
//! Plan 603 arena. Canonical record: `.docs/laya_reference_pin.md`.
//!
//! Law of this module: **parity over elegance**. Where the reference's math
//! is softmax, the port stays softmax (the house sigmoid rule governs OUR
//! modelless primitives, not a faithful port whose G5 gate is
//! top-1 ≥ 99.9% + p-drift ≤ 1e-3). Where the reference's shipped temperature
//! would distort confidence (`choice:11+` = 0.1006), the port mirrors the
//! reference's own clamp — it does not "fix" the checkpoint.
//!
//! ONE BACKEND (`.issues/006`, owner directive 2026-09-22: candle leaves this
//! repo entirely): `laya-riir` — the riir-OWNED forward (`riir/`), the same
//! math over our own flat-`Vec<f32>` tensor code, **no candle anywhere** (the
//! ONE kernel dep is `gemm`, `.issues/002`; the MSL Metal arm is
//! `laya-riir-metal`, `.issues/005`). The candle reference lane that birthed
//! the goldens was deleted after the 005 T5 three-way chart — the frozen
//! expected captures in `tests/fixtures/` ARE the reference now, and the G5
//! gate replays the riir forward against them (candle-independent, verified
//! at the removal: the parity test loads fixtures + corpus digest only).
//!
//! Layout: [`render`] (the Python-JSON byte format), [`config`] (checkpoint
//! configs), [`tokenize`] (pinned BPE + `build_sequence`), [`types`] (the
//! answer/forward envelopes), [`temps`] (the temperature law), [`weights`]
//! (locate / download / verify), [`lang`] + [`router`] (the script-detector),
//! [`riir`] (our tensors, encoder, head, agent + the MSL backend).
//!
//! No Python anywhere (owner directive). Weights are runtime-downloaded from
//! HF and SHA-256-verified — never bundled.

pub mod config;
pub mod lang;
pub mod render;
pub mod router;
pub mod temps;
pub mod tokenize;
pub mod types;
pub mod weights;

/// The riir-owned backend (no candle) — the lane.
#[cfg(feature = "laya-riir")]
pub mod riir;

use std::fmt;

/// The lane's error type: every failure names the checkpoint and the file or
/// stage, so a red G5 is diagnosable without a debugger.
#[derive(Debug)]
pub enum LayaError {
    /// A file required by a checkpoint is missing and could not be fetched.
    Missing {
        /// Which checkpoint.
        checkpoint: &'static str,
        /// Which file, relative to the checkpoint dir.
        file: String,
    },
    /// A downloaded or on-disk file failed its pin verification.
    Pin {
        /// Which checkpoint.
        checkpoint: &'static str,
        /// Which file, relative to the checkpoint dir.
        file: String,
        /// What failed (hash family + detail).
        detail: String,
    },
    /// A config field is missing or out of the shape the port compiles
    /// against (the pin doc's geometry).
    Config {
        /// Which checkpoint.
        checkpoint: &'static str,
        /// What is wrong.
        detail: String,
    },
    /// The tokenizers crate or the tensor backend failed.
    Runtime(String),
    /// The question could not be served (e.g. options exceed the head
    /// budget — the reference raises the same class).
    Question(String),
}

impl fmt::Display for LayaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { checkpoint, file } => {
                write!(f, "laya checkpoint {checkpoint:?}: missing {file}")
            }
            Self::Pin {
                checkpoint,
                file,
                detail,
            } => write!(
                f,
                "laya checkpoint {checkpoint:?}: {file} failed pin: {detail}"
            ),
            Self::Config { checkpoint, detail } => {
                write!(f, "laya checkpoint {checkpoint:?}: bad config: {detail}")
            }
            Self::Runtime(s) => write!(f, "laya runtime: {s}"),
            Self::Question(s) => write!(f, "laya question: {s}"),
        }
    }
}

impl std::error::Error for LayaError {}

/// Shorthand for the lane's result type.
pub type Result<T> = std::result::Result<T, LayaError>;
