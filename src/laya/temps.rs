//! The temperature law + f32 softmax — candle-free, shared by both backends
//! (extracted verbatim from the candle-era head.rs; one copy, zero
//! divergence).

use super::Result;

/// Stable softmax over f32 (the reference's numpy path: `exp(z - max)`,
/// then normalize — float32 throughout).
#[must_use]
pub fn softmax32(z: &[f32]) -> Vec<f32> {
    let max = z.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = z.iter().map(|v| (v - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.into_iter().map(|v| v / sum).collect()
}

/// `temp_bucket(qtype, k)` — the reference's exact bucket spelling.
#[must_use]
pub fn temp_bucket(qtype: usize, k: usize) -> String {
    let name = ["choice", "score", "noul"][qtype];
    let size = if k <= 2 {
        "2"
    } else if k <= 5 {
        "3-5"
    } else if k <= 10 {
        "6-10"
    } else {
        "11+"
    };
    format!("{name}:{size}")
}

/// `clamp_temperature`: [0.5, 5.0], non-finite → 1.0 (the reference's own
/// guard; the shipped `choice:11+` = 0.1006 resolves to 0.5 — the port
/// mirrors the clamp, it does not "fix" the checkpoint).
#[must_use]
pub fn clamp_temperature(t: f64) -> f64 {
    if !t.is_finite() {
        return 1.0;
    }
    t.clamp(0.5, 5.0)
}

/// The resolved temperature tables for one checkpoint (clamped at load —
/// exactly when the reference clamps).
pub struct Temperatures {
    base: [f64; 3],
    by_options: std::collections::HashMap<String, f64>,
}

impl Temperatures {
    /// Build from the raw config tables (clamping here = the reference's
    /// load-time clamp).
    #[must_use]
    pub fn from_config(cfg: &super::config::AgentConfig) -> Self {
        Self {
            base: cfg.temperature.map(clamp_temperature),
            by_options: cfg
                .temperature_by_options
                .iter()
                .map(|(k, v)| (k.clone(), clamp_temperature(*v)))
                .collect(),
        }
    }

    /// The temperature for `qtype` at true option count `k` (bucket table
    /// first, `temperature[qtype]` fallback — the reference's exact order).
    #[must_use]
    pub fn for_question(&self, qtype: usize, k: usize) -> f64 {
        let bucket = temp_bucket(qtype, k);
        *self.by_options.get(&bucket).unwrap_or(&self.base[qtype])
    }
}

/// Convenience for callers building question definitions inline.
#[must_use]
pub fn question(t: &str, instructions: &str, criteria: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": t, "instructions": instructions, "criteria": criteria })
}

/// Re-exported Result so backend modules can share the alias without
/// importing the parent twice.
pub type LayaResult<T> = Result<T>;
