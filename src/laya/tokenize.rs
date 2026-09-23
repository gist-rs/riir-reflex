//! The pinned BPE tokenizers + `build_sequence`.
//!
//! Both pinned tokenizers are HF fast-tokenizer JSON (BPE — the plan's
//! "WordPiece-class" guess was corrected by the pin). The `tokenizers`
//! crate loads the same file the reference's `transformers` fast backend
//! drives, so an encode with `add_special_tokens=false` matches by
//! construction. **tokenizers v1 was attempted and measured NEGATIVE
//! (`.issues/006` T3, 2026-09-22):** 1.0.0-rc.2 refuses the pinned
//! english/typed tokenizers at load — their GPT-2-family vocabs genuinely
//! lack 14 ByteLevel byte atoms, which 0.22 tolerates lazily and v1
//! validates up front (plus the v1-format files need tk-convert's in-memory
//! v1→v2 pass). Stays on 0.22 until 1.0.0 stable relaxes that; the G5 gate
//! re-runs on any future bump (verified, never assumed).
//!
//! `build_sequence` is a line-faithful port of `laya/common.py` at the
//! pinned sha — every budget, floor, clamp and marker rule is the
//! reference's own, including the ones that look odd (`head_ids[:max(8,
//! opt_budget)]` keeps 8 tokens even when the budget went negative; the
//! marker filter `m < max_len` can DROP markers, and the caller treats a
//! dropped marker as an error exactly like the reference does).

use std::path::Path;

use serde_json::Value;
use tokenizers::Tokenizer;

use super::render::{py_json, render_options};
use super::{LayaError, Result};

/// One question in the reference's internal form (`_to_internal` output):
/// `t` ∈ {choice, score, noul}, the instruction string, and the raw
/// criteria value (dict for choice, list for score, dict-or-null for noul).
#[derive(Debug, Clone)]
pub struct InternalQuestion {
    /// Question type key (`choice` / `score` / `noul`).
    pub t: &'static str,
    /// The question-type index (0/1/2 — the head's qtype).
    pub qtype: usize,
    /// Instructions (already stringified — non-str instructions render
    /// through the ASCII-escaped Python-JSON writer, as `json.dumps` does).
    pub ins: String,
    /// Raw criteria value.
    pub crit: Value,
}

/// `choice` criteria given as a LIST become `{opt: null}` per element
/// (bare keys, order preserved) — `Agent._to_internal`. A non-list crit
/// passes through unchanged.
fn choice_list_criteria_to_map(crit: Value) -> Result<Value> {
    let Some(items) = crit.as_array() else {
        return Ok(crit);
    };
    let mut map = serde_json::Map::new();
    for item in items {
        let key = item
            .as_str()
            .ok_or_else(|| LayaError::Question("choice criteria list has a non-string key".into()))?
            .to_string();
        map.insert(key, Value::Null);
    }
    Ok(Value::Object(map))
}

/// Map a question definition (JSON) to the internal form. Port of
/// `Agent._to_internal`: a `choice` question whose `criteria` is a LIST
/// becomes `{opt: null}` per element (bare keys), preserving order.
pub fn to_internal(qdef: &Value) -> Result<InternalQuestion> {
    let t = qdef
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| LayaError::Question("missing question type".into()))?;
    let (qtype, t): (usize, &'static str) = match t {
        "choice" => (0, "choice"),
        "score" => (1, "score"),
        "noul" => (2, "noul"),
        other => {
            return Err(LayaError::Question(format!(
                "unknown question type {other:?}"
            )));
        }
    };
    let mut crit = qdef.get("criteria").cloned().unwrap_or(Value::Null);
    if t == "choice" {
        crit = choice_list_criteria_to_map(crit)?;
    }
    let ins_raw = qdef
        .get("instructions")
        .ok_or_else(|| LayaError::Question("missing instructions".into()))?;
    let ins = match ins_raw.as_str() {
        Some(s) => s.to_string(),
        None => py_json(ins_raw, true),
    };
    Ok(InternalQuestion {
        t,
        qtype,
        ins,
        crit,
    })
}

/// The tokenizer + special-token table one checkpoint loads.
pub struct Tok {
    inner: Tokenizer,
    /// `[CLS]` id (the sequence's first token).
    pub cls: u32,
    /// `[SEP]` id (the sequence's separators).
    pub sep: u32,
    /// `[MASK]` id (one before every option text).
    pub mask: u32,
    /// `[PAD]` id (only used by batched collation; the port runs one
    /// question per forward, matching the reference capture).
    pub pad: u32,
    /// The mask token STRING — the reference neutralizes literal mask
    /// markers in instruction/option/state text with
    /// `text.replace(mask_token, " ")`.
    pub mask_str: String,
}

impl Tok {
    /// Load `tokenizer.json` + the special-token names from
    /// `tokenizer_config.json` in `dir`.
    pub fn from_dir(dir: &Path, ckpt: &'static str) -> Result<Self> {
        let path = dir.join("tokenizer.json");
        if !path.is_file() {
            return Err(LayaError::Missing {
                checkpoint: ckpt,
                file: "tokenizer.json".into(),
            });
        }
        let inner = Tokenizer::from_file(&path)
            .map_err(|e| LayaError::Runtime(format!("tokenizer.json load: {e}")))?;
        let cfg: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("tokenizer_config.json")).map_err(|e| {
                LayaError::Missing {
                    checkpoint: ckpt,
                    file: format!("tokenizer_config.json ({e})"),
                }
            })?,
        )
        .map_err(|e| LayaError::Config {
            checkpoint: ckpt,
            detail: format!("tokenizer_config.json: {e}"),
        })?;
        let name_of = |key: &str| -> Result<String> {
            cfg.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| LayaError::Config {
                    checkpoint: ckpt,
                    detail: format!("tokenizer_config.json missing {key}"),
                })
        };
        let id_of = |name: &str| -> Result<u32> {
            inner.token_to_id(name).ok_or_else(|| LayaError::Config {
                checkpoint: ckpt,
                detail: format!("special token {name:?} not in vocab"),
            })
        };
        let cls = id_of(&name_of("cls_token")?)?;
        let sep = id_of(&name_of("sep_token")?)?;
        let mask_str = name_of("mask_token")?;
        let mask = id_of(&mask_str)?;
        let pad = id_of(&name_of("pad_token")?)?;
        Ok(Self {
            inner,
            cls,
            sep,
            mask,
            pad,
            mask_str,
        })
    }

    /// `tok(text, add_special_tokens=False)["input_ids"]`.
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let enc = self
            .inner
            .encode(text, false)
            .map_err(|e| LayaError::Runtime(format!("encode: {e}")))?;
        Ok(enc.get_ids().to_vec())
    }

    /// Replace every literal mask marker with a space (Python `str.replace`
    /// semantics: all occurrences).
    fn neutralize<'a>(&self, s: &'a str) -> std::borrow::Cow<'a, str> {
        if s.contains(self.mask_str.as_str()) {
            std::borrow::Cow::Owned(s.replace(self.mask_str.as_str(), " "))
        } else {
            std::borrow::Cow::Borrowed(s)
        }
    }
}

/// Port of `build_sequence` (state already a JSON value; serialization is
/// the caller's Python-JSON law).
///
/// Returns `(ids, markers)` — the marker positions are the `[MASK]` index
/// before each option, filtered to `< max_len` exactly like the reference.
pub fn build_sequence(
    tok: &Tok,
    state: &Value,
    q: &InternalQuestion,
    max_len: usize,
    head_max_len: usize,
) -> Result<(Vec<u32>, Vec<usize>)> {
    let opts = render_options(q);
    let ins = str::to_string(&tok.neutralize(&q.ins));
    let head_ids: Vec<u32> = tok.encode(&format!("{} question: {}", q.t, ins))?;

    let mut opt_ids: Vec<Vec<u32>> = Vec::with_capacity(opts.len());
    for opt in &opts {
        let text = tok.neutralize(opt);
        let mut ids = tok.encode(&format!(" {text}"))?;
        ids.truncate(48);
        let mut full = Vec::with_capacity(ids.len() + 1);
        full.push(tok.mask);
        full.extend(ids);
        opt_ids.push(full);
    }

    let opt_budget = head_max_len.saturating_sub(opt_ids.iter().map(Vec::len).sum::<usize>());
    let opt_budget = if opt_budget < 16 {
        let per = std::cmp::max(4, (head_max_len - 16) / std::cmp::max(1, opt_ids.len()));
        for o in &mut opt_ids {
            o.truncate(per);
        }
        head_max_len.saturating_sub(opt_ids.iter().map(Vec::len).sum::<usize>())
    } else {
        opt_budget
    };

    let head_keep = std::cmp::max(8, opt_budget);
    let mut ids: Vec<u32> = Vec::with_capacity(max_len.min(head_max_len + 8));
    ids.push(tok.cls);
    ids.extend(head_ids.into_iter().take(head_keep));
    ids.push(tok.sep);

    let mut markers = Vec::with_capacity(opt_ids.len());
    for o in &opt_ids {
        markers.push(ids.len());
        ids.extend_from_slice(o);
    }
    ids.push(tok.sep);

    let room = max_len.saturating_sub(ids.len() + 1);
    let serialized = super::render::serialize_state(state);
    let state_str = tok.neutralize(&serialized);
    let mut st = tok.encode(&state_str)?;
    st.truncate(room);

    ids.extend(st);
    ids.push(tok.sep);
    ids.truncate(max_len);
    markers.retain(|m| *m < max_len);
    Ok((ids, markers))
}
