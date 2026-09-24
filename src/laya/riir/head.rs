//! The riir-owned decision head — the candle port (`crate::laya::head`) is
//! the normative math: +`type_emb[qtype`], two pre-norm encoder layers (torch
//! `in_proj` Q|K|V rows, `need_weights` 1/√hd q-scaling, `ReLU` FF), marker
//! gather, scorer LN→Linear→GELU→Linear, act head from the CLS row +
//! [top1, top1−top2, entropy/ln(max(k,2)), k/255], softmax32.
//!
//! ⚠ The head FF activation is `ReLU` (torch's `nn.TransformerEncoderLayer`
//! default), the scorer's own GELU is genuinely gelu — the same split the
//! candle port records.
//!
//! Temperatures/`softmax32` come from the shared substrate
//! ([`super::super::temps`]) — NOT copied.

use std::collections::HashMap;

use super::super::temps::softmax32;
use super::super::{LayaError, Result};
use super::backend::Backend;
use super::weights::Weights;

/// One pre-norm transformer-encoder layer of the head (torch
/// `nn.TransformerEncoderLayer`, `norm_first`, `batch_first`, `ReLU` FF).
struct HeadLayer {
    /// `self_attn.in_proj_weight` [3d, d] — torch layout: Q rows, then K
    /// rows, then V rows.
    in_proj_w: Vec<f32>,
    in_proj_b: Vec<f32>,
    /// `self_attn.out_proj`.
    out_w: Vec<f32>,
    out_b: Vec<f32>,
    n1w: Vec<f32>,
    n1b: Vec<f32>,
    n2w: Vec<f32>,
    n2b: Vec<f32>,
    /// `linear1` [4d, d] (`ReLU` between).
    l1w: Vec<f32>,
    l1b: Vec<f32>,
    /// `linear2` [d, 4d].
    l2w: Vec<f32>,
    l2b: Vec<f32>,
}

/// The decision head (our tensors).
pub struct Head {
    layers: Vec<HeadLayer>,
    /// `type_emb.weight` [3, d].
    type_emb: Vec<f32>,
    /// `scorer.0` `LayerNorm`.
    s0w: Vec<f32>,
    s0b: Vec<f32>,
    /// `scorer.1` Linear(d→d) + GELU.
    s1w: Vec<f32>,
    s1b: Vec<f32>,
    /// `scorer.3` Linear(d→1).
    s3w: Vec<f32>,
    s3b: Vec<f32>,
    /// `act_head.0` Linear(d+4→256) + GELU.
    a0w: Vec<f32>,
    a0b: Vec<f32>,
    /// `act_head.2` Linear(256→2).
    a2w: Vec<f32>,
    a2b: Vec<f32>,
    eps: f32,
    /// The head's hidden size (the encoder's `d` — the pinned checkpoints
    /// share one geometry for both stacks).
    d: usize,
}

/// Per-question forward outputs (raw — the agent turns these into answers).
pub struct HeadOutput {
    /// Per-marker scorer logits (length = marker count, all valid — the
    /// port runs one question per forward, so no −1e4 fill occurs).
    pub logits: Vec<f32>,
    /// `softmax(act_logits)` [2].
    pub act_probabilities: Vec<f32>,
}

impl Head {
    /// Assemble from a parsed safetensors map (weights are REMOVED — the
    /// encoder consumed its names first).
    pub fn from_map(
        map: &mut HashMap<String, Weights>,
        ckpt: &'static str,
        d: usize,
        eps: f32,
    ) -> Result<Self> {
        let missing = |name: &str| LayaError::Pin {
            checkpoint: ckpt,
            file: name.to_string(),
            detail: "head tensor missing from checkpoint".into(),
        };
        let mut take = |name: &str| -> Result<Vec<f32>> {
            map.remove(name)
                .map(|w| w.data)
                .ok_or_else(|| missing(name))
        };
        let mut layers = Vec::with_capacity(2);
        for idx in 0..2 {
            layers.push(HeadLayer {
                in_proj_w: take(&format!("head.layers.{idx}.self_attn.in_proj_weight"))?,
                in_proj_b: take(&format!("head.layers.{idx}.self_attn.in_proj_bias"))?,
                out_w: take(&format!("head.layers.{idx}.self_attn.out_proj.weight"))?,
                out_b: take(&format!("head.layers.{idx}.self_attn.out_proj.bias"))?,
                n1w: take(&format!("head.layers.{idx}.norm1.weight"))?,
                n1b: take(&format!("head.layers.{idx}.norm1.bias"))?,
                n2w: take(&format!("head.layers.{idx}.norm2.weight"))?,
                n2b: take(&format!("head.layers.{idx}.norm2.bias"))?,
                l1w: take(&format!("head.layers.{idx}.linear1.weight"))?,
                l1b: take(&format!("head.layers.{idx}.linear1.bias"))?,
                l2w: take(&format!("head.layers.{idx}.linear2.weight"))?,
                l2b: take(&format!("head.layers.{idx}.linear2.bias"))?,
            });
        }
        Ok(Self {
            layers,
            type_emb: take("type_emb.weight")?,
            s0w: take("scorer.0.weight")?,
            s0b: take("scorer.0.bias")?,
            s1w: take("scorer.1.weight")?,
            s1b: take("scorer.1.bias")?,
            s3w: take("scorer.3.weight")?,
            s3b: take("scorer.3.bias")?,
            a0w: take("act_head.0.weight")?,
            a0b: take("act_head.0.bias")?,
            a2w: take("act_head.2.weight")?,
            a2b: take("act_head.2.bias")?,
            eps,
            d,
        })
    }

    /// Forward the encoded sequence through the head.
    ///
    /// `h` is the encoder's `last_hidden_state` flat `[seq, d]` (consumed
    /// in place — the residual stream starts there); `qtype` selects the
    /// type-embedding row; `markers` are the option `[MASK]` positions.
    /// ONE forward body for every backend (`.issues/005`). Host reads of
    /// device results go through [`Backend::download_into`] — exactly
    /// three per forward (logits, CLS row, act logits); everything else
    /// stays device-side under Metal.
    pub fn forward(
        &self,
        b: &dyn Backend,
        h: &mut [f32],
        qtype: usize,
        markers: &[usize],
    ) -> Result<HeadOutput> {
        let d = self.d;
        let seq = h.len() / d;
        let heads = d / 64; // both pinned geometries: head_dim 64
        let hd = d / heads;
        let scale = 1.0f32 / (hd as f32).sqrt();

        // x = h + type_emb[qtype] (broadcast over positions) — in place on
        // the encoder's residual stream.
        let x: &mut [f32] = h;
        let trow = self
            .type_emb
            .get(qtype * d..(qtype + 1) * d)
            .ok_or_else(|| LayaError::Config {
                checkpoint: "head",
                detail: format!("qtype {qtype} outside the 3-row type embedding"),
            })?;
        b.add_bias_row(x, d, trow);
        // The bias-free-LN scratch — one buffer, reused across all layers
        // (the layer loop must not allocate it).
        let mut sq = Vec::new();

        for layer in &self.layers {
            // Pre-norm MHA block (no key-padding mask: one unpadded row).
            // ops::layer_norm's op order = nobias LN + per-row bias add.
            let mut nx = vec![0f32; seq * d];
            b.layer_norm_nobias_into(x, &layer.n1w, self.eps, d, &mut sq, &mut nx);
            b.add_bias_row(&mut nx, d, &layer.n1b);
            let mut qkv = vec![0f32; seq * 3 * d];
            b.matmul_w(&nx, seq, d, &layer.in_proj_w, 3 * d, &mut qkv);
            b.add_bias_row(&mut qkv, 3 * d, &layer.in_proj_b);
            let mut q = vec![0f32; seq * d];
            let mut k = vec![0f32; seq * d];
            let mut v = vec![0f32; seq * d];
            b.split_heads(&qkv, 3 * d, 0, seq, heads, hd, &mut q);
            b.split_heads(&qkv, 3 * d, d, seq, heads, hd, &mut k);
            b.split_heads(&qkv, 3 * d, 2 * d, seq, heads, hd, &mut v);
            // torch's need_weights path scales q by sqrt(1/head_dim) before
            // the matmul. All heads in ONE backend op (one dispatch under
            // Metal; the CPU lane loops per head identically to v1).
            b.scale(&mut q, scale);
            let mut scores = vec![0f32; heads * seq * seq];
            b.matmul_kt_heads(&q, &k, heads, seq, hd, &mut scores);
            b.softmax_rows(&mut scores, seq);
            let mut ctx = vec![0f32; heads * seq * hd];
            b.matmul_heads(&scores, &v, heads, seq, seq, hd, &mut ctx);
            let mut merged = vec![0f32; seq * d];
            b.merge_heads(&ctx, seq, heads, hd, &mut merged);
            let mut attn_out = vec![0f32; seq * d];
            b.matmul_w(&merged, seq, d, &layer.out_w, d, &mut attn_out);
            b.add_bias_row(&mut attn_out, d, &layer.out_b);
            b.add(x, 0, &attn_out, 0, x.len());

            // Pre-norm FF block — ReLU (torch's default activation).
            let mut nx2 = vec![0f32; seq * d];
            b.layer_norm_nobias_into(x, &layer.n2w, self.eps, d, &mut sq, &mut nx2);
            b.add_bias_row(&mut nx2, d, &layer.n2b);
            let mut ff = vec![0f32; seq * 4 * d];
            b.matmul_w(&nx2, seq, d, &layer.l1w, 4 * d, &mut ff);
            b.add_bias_row(&mut ff, 4 * d, &layer.l1b);
            b.relu(&mut ff);
            let mut ff2 = vec![0f32; seq * d];
            b.matmul_w(&ff, seq, 4 * d, &layer.l2w, d, &mut ff2);
            b.add_bias_row(&mut ff2, d, &layer.l2b);
            b.add(x, 0, &ff2, 0, x.len());
        }

        // Gather the marker rows: [k, d].
        let k_opts = markers.len();
        let mut rows = vec![0f32; k_opts * d];
        b.gather_rows(x, d, markers, &mut rows);

        // scorer: LN → Linear(d,d) → GELU → Linear(d,1).
        let mut s = vec![0f32; k_opts * d];
        b.layer_norm_nobias_into(&rows, &self.s0w, self.eps, d, &mut sq, &mut s);
        b.add_bias_row(&mut s, d, &self.s0b);
        let mut s1 = vec![0f32; k_opts * d];
        b.matmul_w(&s, k_opts, d, &self.s1w, d, &mut s1);
        b.add_bias_row(&mut s1, d, &self.s1b);
        b.gelu_erf(&mut s1);
        let mut logits_buf = vec![0f32; k_opts]; // [k, 1] row-major IS [k]
        b.matmul_w(&s1, k_opts, d, &self.s3w, 1, &mut logits_buf);
        b.add_bias_row(&mut logits_buf, 1, &self.s3b);
        let mut logits = vec![0f32; k_opts];
        b.download_into(&logits_buf, &mut logits);

        // act head feats — detached probs of the raw logits (inference: the
        // detach is a no-op numerically), entropy over max(k, 2).
        let p = softmax32(&logits);
        let k_eff = std::cmp::max(k_opts, 2) as f64;
        let mut ent = 0f64;
        for pi in &p {
            let cl = (*pi as f64).max(1e-9);
            ent -= cl * cl.ln();
        }
        ent /= k_eff.ln();
        let mut sorted = p.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let top1 = *sorted.first().unwrap_or(&0.0);
        let top2 = sorted.get(1).copied().unwrap_or(0.0);
        let feats = [top1, top1 - top2, ent as f32, k_eff as f32 / 255.0];

        // pooled = CLS row (position 0) + feats → act logits → softmax.
        // The CLS row is a DEVICE result — sync it out; the features were
        // computed from the already-downloaded logits.
        let mut cls = vec![0f32; d];
        b.download_into(&x[..d], &mut cls);
        let mut act_in = Vec::with_capacity(d + 4);
        act_in.extend_from_slice(&cls);
        act_in.extend_from_slice(&feats);
        let mut a = vec![0f32; 256];
        b.matmul_w(&act_in, 1, d + 4, &self.a0w, 256, &mut a);
        b.add_bias_row(&mut a, 256, &self.a0b);
        b.gelu_erf(&mut a);
        let mut act_logits_buf = vec![0f32; 2];
        b.matmul_w(&a, 1, 256, &self.a2w, 2, &mut act_logits_buf);
        b.add_bias_row(&mut act_logits_buf, 2, &self.a2b);
        let mut act_logits = vec![0f32; 2];
        b.download_into(&act_logits_buf, &mut act_logits);
        let act_probabilities = softmax32(&act_logits);

        Ok(HeadOutput {
            logits,
            act_probabilities,
        })
    }
}
