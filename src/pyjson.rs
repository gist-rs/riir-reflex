//! The Python-JSON byte-format writer — MOVED substrate-side (the lane
//! move): the ONE writer now lives in the inference substrate crate
//! (`riir-infer-laya`, ungated there as here) and this module re-exports
//! it. The ungated harness consumer (modelless-lane state strings) and
//! the lane's sequence rendering keep producing the SAME bytes from ONE
//! implementation — the DRY home moved with the lane, this file is the
//! compatibility shim.

pub use riir_infer_laya::pyjson::{py_float_repr, py_json, serialize_state};
