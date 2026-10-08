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
// Issue 068: the lanes' shared std-only HTTP/1.1 micro-client — the
// extraction the four TcpStream lanes (openthai/agentjev/clef/clm) each
// deferred at their own landing. Ungated (std-only, the agentjev/clef
// import surface). paw is NOT a consumer (curl subprocess transport).
pub(crate) mod http_mini;
// Plan 011 Phase A: the Cloudflare Clef comparison lane — agentjev-shaped
// (ungated: imports only `crate::harness::suites` + the std-only
// `crate::lanes::http_mini` + std + serde_json, the G-ISO-4 import law as
// amended by Issue 068); the hosted Workers-AI route behind an operator-run
// loopback TLS forwarder (plan 011 A0). The no-creds refusal is the lane's
// own construction gate.
pub mod clef;
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
// Issue 073 (from .research/008): the Drex DLM comparison lane — the
// openthai shape on the same TypeSafe `/v1/systemone` wire (ungated: the
// same import law). CC BY-NC weights: measurement only, never a product
// lane, never a distill teacher (the license law in the module doc).
pub mod drex;
// Issue 078 (from .research/009): the LiquidAI d1 comparison lane — the
// drex shape on their official `/decisions/v1/systemone` wire (ungated:
// the same import law). License `other`/lfm1.0: measurement only (the
// license law in the module doc).
pub mod d1;
// Issue 082: the pplx-decider comparison lane — the drex/TypeSafe
// `/v1/systemone` shape against Perplexity's autojev server (their source
// ships IN the model repo; Apache-2.0 UNGATED — teacher-eligible, unlike
// drex/d1's license-restricted lanes). Ungated: the same import law
// (`crate::harness::suites` + the std-only `crate::lanes::http_mini` +
// std + serde_json).
pub mod pplx;
