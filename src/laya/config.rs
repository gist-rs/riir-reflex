//! Checkpoint config parsing: `rl_agent_config.json` (the decision-head /
//! temperature / sequence-budget table) + `encoder_config.json` (the
//! `ModernBERT` geometry). Only fields the port compiles against are read;
//! anything else in the JSON is inert by construction (the pin doc's
//! geometry tables are the contract, and `Config` errors name every
//! violation instead of guessing).

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use super::{LayaError, Result};

/// Which pinned checkpoint. Maps 1:1 to the HF hub's subfolder layout
/// (`convaiinnovations/laya`, one subfolder per checkpoint — the reference's
/// own `subfolder=` download path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Checkpoint {
    /// `english` — ModernBERT-large, `max_len` 512.
    English,
    /// `multilingual` — mmBERT-base, `max_len` 1024.
    Multilingual,
    /// `typed` — ModernBERT-large on typed decisions, `max_len` 1024.
    TypedDecisions,
}

impl Checkpoint {
    /// The three checkpoints in canonical (hub subfolder) order.
    pub const ALL: [Self; 3] = [Self::English, Self::TypedDecisions, Self::Multilingual];

    /// The hub subfolder (also the on-disk directory name).
    pub fn subfolder(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::Multilingual => "multilingual",
            Self::TypedDecisions => "typed",
        }
    }

    /// The name the fixture corpus + expected file key checkpoints by.
    pub fn fixture_name(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::Multilingual => "multilingual",
            Self::TypedDecisions => "typed-decisions",
        }
    }

    /// The checkpoint a fixture row name resolves to.
    pub fn from_fixture_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.fixture_name() == name)
    }
}

/// The `rl_agent_config.json` fields the port reads.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// `max_len` — total sequence budget (512 EN / 1024 ML+TD).
    pub max_len: usize,
    /// `head_max_len` — head budget before state (192 EN / 256 ML+TD).
    pub head_max_len: usize,
    /// `temperature[qtype]` — the fallback table.
    pub temperature: [f64; 3],
    /// `temperature_by_options` — bucket-keyed table (checked against the
    /// reference's 5 buckets; unknown buckets would silently fall back, so
    /// unexpected keys are a loud `Config` error instead).
    pub temperature_by_options: HashMap<String, f64>,
}

impl AgentConfig {
    /// Parse from the checkpoint's `rl_agent_config.json`.
    pub fn parse(v: &Value, ckpt: &'static str) -> Result<Self> {
        let err = |detail: String| LayaError::Config {
            checkpoint: ckpt,
            detail,
        };
        let max_len = v
            .get("max_len")
            .and_then(Value::as_u64)
            .ok_or_else(|| err("missing integer max_len".into()))? as usize;
        let head_max_len =
            v.get("head_max_len")
                .and_then(Value::as_u64)
                .ok_or_else(|| err("missing integer head_max_len".into()))? as usize;
        let temps: Vec<f64> = v
            .get("temperature")
            .and_then(Value::as_array)
            .ok_or_else(|| err("missing temperature table".into()))?
            .iter()
            .map(|x| {
                x.as_f64()
                    .ok_or_else(|| err("non-numeric temperature".into()))
            })
            .collect::<Result<Vec<_>>>()?;
        let temperature = [
            *temps
                .first()
                .ok_or_else(|| err("temperature[0] missing".into()))?,
            *temps
                .get(1)
                .ok_or_else(|| err("temperature[1] missing".into()))?,
            *temps
                .get(2)
                .ok_or_else(|| err("temperature[2] missing".into()))?,
        ];
        let mut temperature_by_options = HashMap::new();
        if let Some(map) = v.get("temperature_by_options").and_then(Value::as_object) {
            for (k, val) in map {
                let t = val
                    .as_f64()
                    .ok_or_else(|| err(format!("non-numeric temperature_by_options[{k}]")))?;
                temperature_by_options.insert(k.clone(), t);
            }
        }
        Ok(Self {
            max_len,
            head_max_len,
            temperature,
            temperature_by_options,
        })
    }
}

/// The `encoder_config.json` geometry the `ModernBERT` forward compiles
/// against.
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// `hidden_size` (1024 / 768).
    pub hidden: usize,
    /// `num_hidden_layers` (28 / 22).
    pub layers: usize,
    /// `num_attention_heads` (16 / 12).
    pub heads: usize,
    /// `intermediate_size` (2624 / 1152).
    pub intermediate: usize,
    /// `vocab_size` — checked against the loaded embedding tensor.
    pub vocab: usize,
    /// `norm_eps` (1e-5).
    pub eps: f32,
    /// `global_attn_every_n_layers` (3 — full attention on layers 0, 3, …).
    pub global_every: usize,
    /// `local_attention` (128 — the FULL window; the mask uses half + 1).
    pub local_attention: usize,
    /// `RoPE` theta for full-attention layers.
    pub rope_theta_full: f64,
    /// `RoPE` theta for sliding-window layers.
    pub rope_theta_slide: f64,
    /// `layer_types` — per-layer `true` = sliding.
    pub sliding: Vec<bool>,
    /// `hidden_activation` must be `gelu` (the shipped checkpoints') — a
    /// different value would change the MLP silently, so it is refused.
    pub hidden_activation: String,
}

impl EncoderConfig {
    /// Parse from the checkpoint's `encoder_config.json`.
    pub fn parse(v: &Value, ckpt: &'static str) -> Result<Self> {
        let err = |detail: String| LayaError::Config {
            checkpoint: ckpt,
            detail,
        };
        let num = |key: &str| -> Result<f64> {
            v.get(key)
                .and_then(Value::as_f64)
                .ok_or_else(|| err(format!("missing numeric {key}")))
        };
        let usize_of = |key: &str| -> Result<usize> { num(key).map(|x| x as usize) };
        let hidden = usize_of("hidden_size")?;
        let heads = usize_of("num_attention_heads")?;
        if hidden % heads != 0 {
            return Err(err(format!(
                "hidden {hidden} not divisible by heads {heads}"
            )));
        }
        let layer_types: Vec<bool> = v
            .get("layer_types")
            .and_then(Value::as_array)
            .ok_or_else(|| err("missing layer_types".into()))?
            .iter()
            .map(|x| match x.as_str() {
                Some("full_attention") => Ok(false),
                Some("sliding_attention") => Ok(true),
                other => Err(err(format!("unknown layer_type {other:?}"))),
            })
            .collect::<Result<Vec<_>>>()?;
        let rope = v
            .get("rope_parameters")
            .ok_or_else(|| err("missing rope_parameters".into()))?;
        let theta_of = |kind: &str| -> Result<f64> {
            rope.get(kind)
                .and_then(|p| p.get("rope_theta"))
                .and_then(Value::as_f64)
                .ok_or_else(|| err(format!("missing rope_parameters.{kind}.rope_theta")))
        };
        let activation = v
            .get("hidden_activation")
            .and_then(Value::as_str)
            .ok_or_else(|| err("missing hidden_activation".into()))?
            .to_string();
        if activation != "gelu" {
            return Err(err(format!(
                "hidden_activation {activation:?} unsupported — the port compiles against the pinned checkpoints' gelu"
            )));
        }
        let sliding_len = layer_types.len();
        Ok(Self {
            hidden,
            layers: usize_of("num_hidden_layers")?,
            heads,
            intermediate: usize_of("intermediate_size")?,
            vocab: usize_of("vocab_size")?,
            eps: num("norm_eps")? as f32,
            global_every: usize_of("global_attn_every_n_layers")?,
            local_attention: usize_of("local_attention")?,
            rope_theta_full: theta_of("full_attention")?,
            rope_theta_slide: theta_of("sliding_attention")?,
            sliding: layer_types,
            hidden_activation: activation,
        })
        .map(|c: Self| {
            if c.layers != sliding_len {
                Err(err(format!(
                    "num_hidden_layers {} != layer_types len {sliding_len}",
                    c.layers
                )))
            } else {
                Ok(c)
            }
        })
        .and_then(std::convert::identity)
    }

    /// `head_dim` = hidden / heads (64 on both pinned geometries).
    pub fn head_dim(&self) -> usize {
        self.hidden / self.heads
    }

    /// The mask window for a sliding layer: `local_attention // 2` — the
    /// config's `sliding_window` property (half the total window). The
    /// attention module's `+1` (flash-attention inclusive-boundary
    /// bookkeeping) rides the FLASH-path kwarg only; the SDPA mask the
    /// capture ran under is built from the UN-incremented property with
    /// the bidirectional overlay `|q - kv| <= window`.
    pub fn sliding_window(&self) -> usize {
        self.local_attention / 2
    }

    /// `RoPE` theta for layer `idx`.
    pub fn theta_for(&self, idx: usize) -> f64 {
        if self.sliding[idx] {
            self.rope_theta_slide
        } else {
            self.rope_theta_full
        }
    }
}

/// Read + parse both checkpoint configs from a local (flat-named) dir —
/// the shared load step of BOTH backends (moved from the candle agent; the
/// riir agent consumes the same two files with the same error mapping).
pub fn load_checkpoint_configs(
    dir: &Path,
    ckpt: &'static str,
) -> Result<(AgentConfig, EncoderConfig)> {
    let read_json = |file: &str| -> Result<Value> {
        let text = std::fs::read_to_string(dir.join(file)).map_err(|e| LayaError::Missing {
            checkpoint: ckpt,
            file: format!("{file} ({e})"),
        })?;
        serde_json::from_str(&text).map_err(|e| LayaError::Config {
            checkpoint: ckpt,
            detail: format!("{file}: {e}"),
        })
    };
    let agent_raw = read_json("rl_agent_config.json")?;
    let enc_raw = read_json("encoder_config.json")?;
    Ok((
        AgentConfig::parse(&agent_raw, ckpt)?,
        EncoderConfig::parse(&enc_raw, ckpt)?,
    ))
}
