//! The laya lane — MOVED substrate-side (the consolidation issue's
//! encoder-lane move): the pinned-checkpoint lane now lives in the
//! inference substrate repo (`../riir-infer`, crate `riir-infer-laya`)
//! and this module is the compatibility shim. A glob re-export keeps
//! every `crate::laya::*` / `riir_reflex::laya::*` path resolving — the
//! public API is unchanged (config / tokenize / weights / types / temps /
//! lang / router / render / `riir` forward / `LayaError` / `Result`).
//!
//! What stayed HERE (consumer-side, deliberately): the G5 parity gate
//! (`tests/laya_riir_parity.rs`) with the frozen fixture corpus + expected
//! capture — a lane number is only ever published from a posture this
//! gate has greened — and the harness/pyjson consumers of the byte-format
//! writer. The lane's own op-level gate (`metal_ops_smoke`) moved with
//! the lane (promoted into the lane crate).

pub use riir_infer_laya::laya::*;
