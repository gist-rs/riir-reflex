//! The laya runtime, riir-owned backend: load a pinned checkpoint once
//! (our own safetensors reader — candle-free), answer typed questions.
//!
//! `forward_question` is the parity seam (the `tests/laya_riir_parity` gate
//! replays it against the SAME captured reference forwards the candle lane
//! gates on); `system_one` is the envelope surface the harness consumes.
//! Both share ONE forward path with the candle agent's exact semantics —
//! the shared envelope helpers (`option_keys` / `argmax_of` /
//! `confidence_from_probs`) live in [`super::super::types`], one copy for
//! both backends.
//!
//! `LAYA_DEVICE` is HONORED here (`.issues/005`): unset → the build's
//! default posture (Metal on macOS with `laya-riir-metal` compiled — the
//! Plan 001 T4 watchability default; CPU elsewhere), explicit `cpu`/`metal`
//! is honored verbatim, anything else fails loud.
//! `Cpu` backend (the lane's original posture); `metal` → the MSL backend
//! ([`super::metal`], feature `laya-riir-metal`, macOS) — fail loud when
//! the feature or platform is absent, never a silent CPU fallback (the
//! candle lane's `device_from_env` precedent). The gate law is unchanged:
//! G5 parity must be green at WHICHEVER posture a number is published
//! from.

use serde_json::Value;

use super::super::config::{AgentConfig, Checkpoint, load_checkpoint_configs};
use super::super::render::{py_round4, render_options};
use super::super::temps::{Temperatures, softmax32, temp_bucket};
use super::super::tokenize::{InternalQuestion, Tok, build_sequence, to_internal};
use super::super::types::{Answer, Forward, argmax_of, confidence_from_probs, option_keys};
use super::super::weights::ensure_checkpoint;
use super::super::{LayaError, Result};
use super::backend::{Backend, Cpu};
use super::encoder::Encoder;
use super::head::{Head, HeadOutput};

/// The device the riir forward runs on, chosen at load from `LAYA_DEVICE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    /// The flat-`Vec` CPU backend (the lane's original posture).
    Cpu,
    /// The MSL backend (`laya-riir-metal`, macOS).
    Metal,
}

impl DeviceKind {
    /// Resolve `LAYA_DEVICE` — unset/empty → [`Self::default_device`]
    /// (Metal when this build SHIPS the Metal backend on macOS, CPU
    /// everywhere else — the measured ~2× forward is the arena
    /// watchability default, Plan 001 T4), `cpu` → [`DeviceKind::Cpu`]
    /// (the explicit opt-out), `metal` → [`DeviceKind::Metal`], anything
    /// else is an error (an env typo must fail loud, never fall back).
    pub fn from_env() -> Result<Self> {
        match std::env::var("LAYA_DEVICE").as_deref() {
            Ok("") | Err(_) => Ok(Self::default_device()),
            Ok("cpu") => Ok(Self::Cpu),
            Ok("metal") => Ok(Self::Metal),
            Ok(other) => Err(LayaError::Config {
                checkpoint: "riir",
                detail: format!(
                    "unknown LAYA_DEVICE {other:?} — expected unset, \"cpu\" or \"metal\""
                ),
            }),
        }
    }

    /// The no-env posture: Metal where the backend is compiled and exists
    /// (macOS + `laya-riir-metal`), CPU everywhere else. An explicit env
    /// value is always honored verbatim — only the ABSENT choice defaults.
    pub fn default_device() -> Self {
        #[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
        {
            Self::Metal
        }
        #[cfg(not(all(target_os = "macos", feature = "laya-riir-metal")))]
        {
            Self::Cpu
        }
    }
}

/// One autoreleasepool spanning one forward. The Metal backend creates
/// autoreleased command buffers per op; on a thread with no Cocoa runloop
/// they would otherwise accumulate until thread exit. With the metal
/// feature this is `objc2::rc::autoreleasepool`; without it, identity
/// (one code path for both postures).
#[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
fn pass_pool<T>(f: impl FnOnce() -> T) -> T {
    objc2::rc::autoreleasepool(|_| f())
}

/// [`pass_pool`] identity form when no Metal backend is compiled.
#[cfg(not(all(target_os = "macos", feature = "laya-riir-metal")))]
fn pass_pool<T>(f: impl FnOnce() -> T) -> T {
    f()
}

/// A loaded checkpoint (riir backend): tokenizer + encoder + head +
/// temperature tables, plus the device backend the forward runs on.
pub struct RiirAgent {
    tok: Tok,
    enc: Encoder,
    head: Head,
    backend: Box<dyn Backend>,
    temps: Temperatures,
    cfg: AgentConfig,
    ckpt: &'static str,
}

impl RiirAgent {
    /// Load one checkpoint from the weights root (downloading + verifying
    /// against the pins first — never bundled). The safetensors file is
    /// parsed ONCE and split between encoder and head (weights are removed
    /// from the map, no second copy).
    pub fn load(root: &std::path::Path, ckpt: Checkpoint) -> Result<Self> {
        let dir = ensure_checkpoint(root, ckpt)?;
        let name = ckpt.subfolder();
        let (agent_cfg, enc_cfg) = load_checkpoint_configs(&dir, name)?;

        let tok = Tok::from_dir(&dir, name)?;
        let mut raw = super::weights::load(&dir.join("model.safetensors"), name)?;
        let enc = Encoder::from_map(&mut raw, enc_cfg.clone(), name)?;
        let head = Head::from_map(&mut raw, name, enc_cfg.hidden, enc_cfg.eps)?;

        let backend: Box<dyn Backend> = match DeviceKind::from_env()? {
            DeviceKind::Cpu => Box::new(Cpu),
            #[cfg(all(target_os = "macos", feature = "laya-riir-metal"))]
            DeviceKind::Metal => Box::new(super::metal::Metal::new()?),
            #[cfg(not(all(target_os = "macos", feature = "laya-riir-metal")))]
            DeviceKind::Metal => {
                return Err(LayaError::Config {
                    checkpoint: name,
                    detail: "LAYA_DEVICE=metal needs --features laya-riir-metal on macOS — \
                             this build has no Metal backend (fail loud, never a silent \
                             CPU fallback)"
                        .into(),
                });
            }
        };

        let temps = Temperatures::from_config(&agent_cfg);
        Ok(Self {
            tok,
            enc,
            head,
            backend,
            temps,
            cfg: agent_cfg,
            ckpt: name,
        })
    }

    /// The checkpoint this agent serves.
    pub fn checkpoint(&self) -> &'static str {
        self.ckpt
    }

    /// The backend posture this agent runs (`"cpu"` / `"metal"`) — gate
    /// lines and timing labels print it so a reading can never be mistaken
    /// for the other posture.
    pub fn device(&self) -> &'static str {
        self.backend.name()
    }

    /// Forward one question against `state` — one unpadded sequence, the
    /// reference capture's exact batch shape and the G5 parity seam. Errors
    /// when options exceed the head budget (the reference raises the same
    /// class) or when a marker was truncated away.
    pub fn forward_question(&self, state: &Value, qdef: &Value) -> Result<Forward> {
        let q = to_internal(qdef)?;
        self.forward_internal(state, &q)
    }

    /// The internal-typed variant (avoids re-parsing per row in the test).
    pub fn forward_internal(&self, state: &Value, q: &InternalQuestion) -> Result<Forward> {
        let opts = render_options(q);
        let (ids, markers) =
            build_sequence(&self.tok, state, q, self.cfg.max_len, self.cfg.head_max_len)?;
        if opts.is_empty() || markers.len() != opts.len() {
            return Err(LayaError::Question(format!(
                "options exceed the head budget: {} rendered, {} markers survived",
                opts.len(),
                markers.len()
            )));
        }
        // One autoreleasepool per forward: the Metal backend's per-op
        // autoreleased command buffers drain here instead of accumulating
        // on a thread with no Cocoa runloop (a no-op wrapper without the
        // metal feature — one code path for both postures).
        let out = pass_pool(|| -> Result<HeadOutput> {
            self.backend.begin_pass();
            let mut hidden = self.enc.forward(self.backend.as_ref(), &ids)?;
            self.head
                .forward(self.backend.as_ref(), &mut hidden, q.qtype, &markers)
        })?;
        let k = markers.len();
        let t = self.temps.for_question(q.qtype, k);
        let z: Vec<f32> = out.logits.iter().map(|l| l / t as f32).collect();
        let probs = softmax32(&z);
        let confidence = confidence_from_probs(&probs, k);
        Ok(Forward {
            logits: out.logits,
            probs,
            confidence,
            act_probabilities: out.act_probabilities,
            temperature_used: t,
            bucket: temp_bucket(q.qtype, k),
            seq_len: ids.len(),
            markers,
        })
    }

    /// `system_one` over multiple questions (one forward each — the
    /// capture's posture; the reference batches, which is numerically
    /// equivalent modulo padding).
    pub fn system_one(&self, state: &Value, questions: &[(String, Value)]) -> Result<Vec<Answer>> {
        let mut out = Vec::with_capacity(questions.len());
        for (qid, qdef) in questions {
            let q = to_internal(qdef)?;
            let f = self.forward_internal(state, &q)?;
            let keys = option_keys(&q)?;
            let argmax = argmax_of(&f.probs);
            let act_probability = py_round4(f.act_probabilities[0] as f64);
            let probabilities = keys
                .iter()
                .zip(f.probs.iter())
                .map(|(k, p)| (k.clone(), py_round4(*p as f64)))
                .collect();
            let (choice, score, noul) = match q.t {
                "choice" => (Some(keys[argmax].clone()), None, None),
                "score" => {
                    let exp: f64 = f
                        .probs
                        .iter()
                        .enumerate()
                        .map(|(i, p)| i as f64 * *p as f64)
                        .sum();
                    (None, Some(py_round4(exp)), None)
                }
                _ => (None, None, Some(py_round4(f.probs[1] as f64))),
            };
            let confidence = if q.t == "noul" {
                let p1 = f.probs[1] as f64;
                py_round4(p1.max(1.0 - p1))
            } else {
                py_round4(f.confidence)
            };
            out.push(Answer {
                qid: qid.clone(),
                t: q.t,
                choice,
                score,
                noul,
                probabilities,
                confidence,
                act_probability,
                temperature_used: f.temperature_used,
            });
        }
        Ok(out)
    }
}
