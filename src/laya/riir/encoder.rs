//! The riir-owned `ModernBERT` encoder forward — the candle port
//! (`crate::laya::encoder`) is the normative math; this module carries the
//! same op order, same shapes and the same GEMM call shapes onto the
//! flat-`Vec<f32>` ops in [`super::ops`], so the G5 drift budget is spent
//! only on the reduction-order differences, not the matmuls or the gelu.
//!
//! Geometry facts the port compiles against (`.docs/laya_reference_pin.md`):
//! `RoPE` is the ONLY position signal; NO bias tensors anywhere in the
//! encoder; layer 0 has `mlp_norm` only (the embeddings norm feeds attention
//! directly); full attention every `global_attn_every_n_layers`-th layer,
//! sliding window otherwise; per-layer-type `RoPE` theta.

use std::collections::HashMap;

use super::super::config::EncoderConfig;
use super::super::{LayaError, Result};
use super::backend::{AttnScratch, Backend};
use super::ops;
use super::weights::Weights;

/// One encoder layer's weights (all f32, bias-free — taken out of the
/// parsed map, never cloned).
struct Layer {
    /// `attn_norm` — `None` for layer 0 (the reference's `nn.Identity`).
    attn_norm: Option<Vec<f32>>,
    /// `attn.Wqkv.weight` [3d, d].
    wqkv: Vec<f32>,
    /// `attn.Wo.weight` [d, d].
    wo: Vec<f32>,
    /// `mlp.Wi.weight` [2I, d] — fused (input, gate).
    wi: Vec<f32>,
    /// `mlp.Wo.weight` [d, I].
    mlp_wo: Vec<f32>,
    /// `mlp_norm.weight` [d].
    mlp_norm: Vec<f32>,
    /// Sliding-window layer?
    sliding: bool,
}

/// The loaded `ModernBERT` encoder (our tensors).
pub struct Encoder {
    cfg: EncoderConfig,
    /// The checkpoint this encoder serves (error rendering).
    ckpt: &'static str,
    /// `embeddings.tok_embeddings.weight` [vocab, d].
    tok_emb: Vec<f32>,
    /// `embeddings.norm.weight` [d].
    emb_norm: Vec<f32>,
    layers: Vec<Layer>,
    /// `final_norm.weight` [d].
    final_norm: Vec<f32>,
}

/// Per-forward scratch — allocated once at the sequence's sizes, reused
/// across all layers (the layer loop must not allocate).
struct Scratch {
    x: Vec<f32>,
    qkv: Vec<f32>,
    attn: AttnScratch,
    merged: Vec<f32>,
    attn_out: Vec<f32>,
    xn: Vec<f32>,
    fused: Vec<f32>,
    act: Vec<f32>,
    mlp_out: Vec<f32>,
    sq: Vec<f32>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            x: Vec::new(),
            qkv: Vec::new(),
            attn: AttnScratch::default(),
            merged: Vec::new(),
            attn_out: Vec::new(),
            xn: Vec::new(),
            fused: Vec::new(),
            act: Vec::new(),
            mlp_out: Vec::new(),
            sq: Vec::new(),
        }
    }

    fn reset(&mut self, n: usize) {
        self.x.resize(n, 0.0);
        self.merged.resize(n, 0.0);
        self.attn_out.resize(n, 0.0);
        self.xn.resize(n, 0.0);
        self.mlp_out.resize(n, 0.0);
        // qkv (seq·3d) and the attention scratch (split thirds + the score
        // parent) are sized inside the backend's `attention_forward`.
    }
}

impl Encoder {
    /// Assemble from a parsed safetensors map (weights are REMOVED — the
    /// map is split between encoder and head with no second copy; the
    /// unused `temperature` tensor is what legitimately remains).
    pub fn from_map(
        map: &mut HashMap<String, Weights>,
        cfg: EncoderConfig,
        ckpt: &'static str,
    ) -> Result<Self> {
        let missing = |name: &str| LayaError::Pin {
            checkpoint: ckpt,
            file: name.to_string(),
            detail: "tensor missing from checkpoint".into(),
        };
        // The embedding first — its declared shape is checked against the
        // config vocab (rows AND width both pinned by the shape + the
        // hidden-dependent takes below).
        let vocab_name = "encoder.embeddings.tok_embeddings.weight";
        let tok_w = map.remove(vocab_name).ok_or_else(|| missing(vocab_name))?;
        if tok_w.shape.first().copied() != Some(cfg.vocab) {
            return Err(LayaError::Config {
                checkpoint: ckpt,
                detail: format!(
                    "tok_embeddings rows {:?} != config vocab {}",
                    tok_w.shape.first(),
                    cfg.vocab
                ),
            });
        }
        let tok_emb = tok_w.data;
        let mut take = |name: &str| -> Result<Vec<f32>> {
            map.remove(name)
                .map(|w| w.data)
                .ok_or_else(|| missing(name))
        };
        let mut layers = Vec::with_capacity(cfg.layers);
        for idx in 0..cfg.layers {
            let attn_norm = if idx == 0 {
                None
            } else {
                Some(take(&format!("encoder.layers.{idx}.attn_norm.weight"))?)
            };
            layers.push(Layer {
                attn_norm,
                wqkv: take(&format!("encoder.layers.{idx}.attn.Wqkv.weight"))?,
                wo: take(&format!("encoder.layers.{idx}.attn.Wo.weight"))?,
                wi: take(&format!("encoder.layers.{idx}.mlp.Wi.weight"))?,
                mlp_wo: take(&format!("encoder.layers.{idx}.mlp.Wo.weight"))?,
                mlp_norm: take(&format!("encoder.layers.{idx}.mlp_norm.weight"))?,
                sliding: cfg.sliding[idx],
            });
        }
        Ok(Self {
            cfg,
            ckpt,
            tok_emb,
            emb_norm: take("encoder.embeddings.norm.weight")?,
            layers,
            final_norm: take("encoder.final_norm.weight")?,
        })
    }

    /// Forward `input_ids` (one unpadded sequence — the reference capture's
    /// batch shape) to `last_hidden_state` as a flat `[seq, d]` row-major
    /// buffer. ONE forward body for every backend (`.issues/005`): the op
    /// order lives here and nowhere else. Under the Metal backend the
    /// returned Vec is a HANDLE — ops run device-side and its host bytes
    /// are stale until [`Backend::download_into`] syncs; every consumer
    /// below reads it only through backend ops.
    pub fn forward(&self, b: &dyn Backend, input_ids: &[u32]) -> Result<Vec<f32>> {
        let seq = input_ids.len();
        let d = self.cfg.hidden;
        let hd = self.cfg.head_dim();
        let heads = self.cfg.heads;
        let scale = 1.0f32 / (hd as f32).sqrt();
        let i_sz = self.cfg.intermediate;
        let eps = self.cfg.eps;

        // Embeddings: host-side token gather (a ~seq·d memcpy off the
        // agent-owned table — cheap, and it keeps the vocab-sized table out
        // of the device flow entirely) + LayerNorm (the norm feeds layer 0's
        // attention directly — the layer-0 quirk).
        let mut gathered = vec![0f32; seq * d];
        for (s, id) in input_ids.iter().enumerate() {
            let row = (*id as usize) * d;
            let Some(src) = self.tok_emb.get(row..row + d) else {
                return Err(LayaError::Config {
                    checkpoint: self.ckpt,
                    detail: format!("token id {id} outside the embedding table"),
                });
            };
            gathered[s * d..s * d + d].copy_from_slice(src);
        }
        let mut h = vec![0f32; seq * d];
        let mut sq = Vec::new();
        b.layer_norm_nobias_into(&gathered, &self.emb_norm, eps, d, &mut sq, &mut h);

        let mut sc = Scratch::new();
        sc.reset(seq * d);
        let mut rope_full: Option<(Vec<f32>, Vec<f32>)> = None;
        let mut rope_slide: Option<(Vec<f32>, Vec<f32>)> = None;

        // Sliding-window additive mask [seq, seq] — built once, added per
        // sliding layer. Skipped when the window covers the whole sequence
        // (the reference's mask-skip: same math, no mask tensor).
        let window = self.cfg.sliding_window();
        let mask: Option<Vec<f32>> = if seq > 1 && window < seq - 1 {
            let mut m = vec![f32::MIN; seq * seq];
            for qi in 0..seq {
                let lo = qi.saturating_sub(window);
                let hi = (qi + window).min(seq - 1);
                for kv in lo..=hi {
                    m[qi * seq + kv] = 0.0;
                }
            }
            Some(m)
        } else {
            None
        };

        for layer in &self.layers {
            // x = attn_norm(h) — or h itself on layer 0 (the identity path,
            // copied DEVICE-side: the residual stream is device-current
            // under Metal and a host copy would read stale bytes).
            match &layer.attn_norm {
                Some(w) => {
                    b.layer_norm_nobias_into(&h, w, eps, d, &mut sc.sq, &mut sc.x);
                }
                None => b.copy_into(&h, &mut sc.x),
            }

            // Attention: Wqkv → the backend's fused attention block over the
            // packed projection (rope + q-scale + score + mask + softmax +
            // value mix + head merge in ONE op — the op contract and its CPU
            // op sequence live on the `Backend` trait, the Metal lane's
            // flash-attention form in [`super::metal`]).
            sc.qkv.resize(seq * 3 * d, 0.0);
            b.matmul_w(&sc.x, seq, d, &layer.wqkv, 3 * d, &mut sc.qkv);
            let rope = if layer.sliding {
                rope_slide
                    .get_or_insert_with(|| ops::rope_tables(seq, hd, self.cfg.rope_theta_slide))
            } else {
                rope_full.get_or_insert_with(|| ops::rope_tables(seq, hd, self.cfg.rope_theta_full))
            };
            b.attention_forward(
                &sc.qkv,
                &rope.0,
                &rope.1,
                scale,
                seq,
                heads,
                hd,
                if layer.sliding { window } else { usize::MAX },
                if layer.sliding { mask.as_deref() } else { None },
                &mut sc.attn,
                &mut sc.merged,
            );
            b.matmul_w(&sc.merged, seq, d, &layer.wo, d, &mut sc.attn_out);
            let h_len = h.len();
            b.add(&mut h, 0, &sc.attn_out, 0, h_len);

            // MLP: fused Wi → gelu(input) · gate → Wo.
            b.layer_norm_nobias_into(&h, &layer.mlp_norm, eps, d, &mut sc.sq, &mut sc.xn);
            sc.fused.resize(seq * 2 * i_sz, 0.0);
            b.matmul_w(&sc.xn, seq, d, &layer.wi, 2 * i_sz, &mut sc.fused);
            sc.act.resize(seq * i_sz, 0.0);
            b.glu_gelu_gate(&sc.fused, seq, i_sz, &mut sc.act);
            b.matmul_w(&sc.act, seq, i_sz, &layer.mlp_wo, d, &mut sc.mlp_out);
            let h_len = h.len();
            b.add(&mut h, 0, &sc.mlp_out, 0, h_len);
        }

        let mut out = vec![0f32; seq * d];
        b.layer_norm_nobias_into(&h, &self.final_norm, eps, d, &mut sc.sq, &mut out);
        Ok(out)
    }
}
