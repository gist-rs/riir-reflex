//! The candle-free answer/forward envelopes of the laya lane — shared by
//! both backends (`laya` candle / `laya-riir` our tensors). One copy, zero
//! divergence.

use serde::Serialize;

use super::tokenize::InternalQuestion;
use super::{LayaError, Result};

/// One question's forward outputs at the parity seam (raw — the agent turns
/// these into answers; the G5 tests replay them against the capture).
#[derive(Debug, Clone)]
pub struct Forward {
    /// Raw scorer logits (length = option count).
    pub logits: Vec<f32>,
    /// Temperature-scaled probabilities.
    pub probs: Vec<f32>,
    /// Entropy confidence over `max(k, 2)`, clipped [0, 1].
    pub confidence: f64,
    /// `softmax(act_logits)` [2].
    pub act_probabilities: Vec<f32>,
    /// The resolved (clamped) temperature.
    pub temperature_used: f64,
    /// The bucket key the temperature resolved through.
    pub bucket: String,
    /// Sequence length (structural parity check).
    pub seq_len: usize,
    /// Marker positions (structural parity check).
    pub markers: Vec<usize>,
}

/// One answered question (the reference's answer envelope).
#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    /// The question id this answer belongs to.
    pub qid: String,
    /// `type`.
    pub t: &'static str,
    /// `choice`: the winning option key.
    pub choice: Option<String>,
    /// `score`: Σ i·pᵢ.
    pub score: Option<f64>,
    /// `noul`: p(true).
    pub noul: Option<f64>,
    /// Per-option probabilities (option keys in label order, rounded 4).
    pub probabilities: Vec<(String, f64)>,
    /// Calibrated confidence (entropy readout; noul uses max(p, 1−p);
    /// rounded 4 — the reference's exact definitions).
    pub confidence: f64,
    /// `softmax(act_logits)[0]` — the escalate-slot probability.
    pub act_probability: f64,
    /// The resolved (clamped) temperature actually applied.
    pub temperature_used: f64,
}

/// The option keys in label order (choice: criteria keys; score: "0", "1",
/// …; noul: false/true) — the reference's answer-envelope vocabulary. Shared
/// by both backends (moved from the candle agent; the riir agent must not
/// reach into the cfg(`laya`)-gated module).
pub(crate) fn option_keys(q: &InternalQuestion) -> Result<Vec<String>> {
    match q.t {
        "choice" => q
            .crit
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .ok_or_else(|| LayaError::Question("choice criteria must be an object".into())),
        "score" => {
            let n = q.crit.as_array().map_or(0, Vec::len);
            Ok((0..n).map(|i| i.to_string()).collect())
        }
        _ => Ok(vec!["false".into(), "true".into()]),
    }
}

/// First-argmax over probabilities (ties keep the earliest index — the
/// reference's `np.argmax`). Shared by both backends.
pub(crate) fn argmax_of(p: &[f32]) -> usize {
    let mut best = 0usize;
    for (i, v) in p.iter().enumerate() {
        if *v > p[best] {
            best = i;
        }
    }
    best
}

/// `confidence_from_probs`: `1 − H(p)/ln(k)`, k < 2 → 1.0, clipped [0, 1]
/// (f32 compute like the reference's numpy path). Shared by both backends.
pub(crate) fn confidence_from_probs(p: &[f32], k: usize) -> f64 {
    if k < 2 {
        return 1.0;
    }
    let mut ent = 0f32;
    for pi in p.iter().take(k) {
        let cl = pi.max(1e-12);
        ent -= cl * cl.ln();
    }
    let conf = 1.0 - ent / (k as f32).ln();
    (conf as f64).clamp(0.0, 1.0)
}
