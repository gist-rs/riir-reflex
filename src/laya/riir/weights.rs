//! Minimal safetensors reader for the riir-owned forward — candle-free by
//! construction (`.issues/002`): the 8-byte LE u64 header length → the JSON
//! header (`{name: {dtype, shape, data_offsets}}`, offsets relative to the
//! data section at `8 + header_len`) → absolute slices of the buffer.
//!
//! Widening: F16 → F32 is BIT-EXACT (every f16 value is representable in
//! f32 — normals `(sign<<31) | ((e−15+127)<<23) | (m<<13)`, subnormals
//! `sign · m · 2⁻²⁴`, ±0/±inf/NaN mapped bit-for-bit), F32 passes through,
//! F64 casts down (the same rounding candle's `to_dtype(F32)` applies).
//! Every other dtype is refused LOUD — the pinned checkpoints are F16
//! throughout except the unused `temperature` tensor (F32 english/
//! multilingual, F16 typed-decisions — the reader widens whatever each
//! tensor declares and assumes no uniform dtype).
//!
//! The file is read whole and widened tensor-by-tensor (the english
//! checkpoint is ~890 MB f16 → ~1.7 GB f32; the transient peak is fine on
//! any machine this lane targets, and the map is consumed via `remove` so
//! encoder + head split it without a second copy).

use std::collections::HashMap;
use std::path::Path;

use super::super::{LayaError, Result};

/// One widened tensor: the header's shape + the f32 data (row-major, the
/// safetensors storage order).
#[derive(Debug)]
pub struct Weights {
    /// The declared shape (e.g. `[3d, d]` for a fused Wqkv).
    pub shape: Vec<usize>,
    /// The widened f32 payload (the `shape` product of elements).
    pub data: Vec<f32>,
}

/// Parse + widen a safetensors file from disk.
pub fn load(path: &Path, ckpt: &'static str) -> Result<HashMap<String, Weights>> {
    let bytes = std::fs::read(path).map_err(|e| LayaError::Missing {
        checkpoint: ckpt,
        file: format!("model.safetensors ({e})"),
    })?;
    from_bytes(&bytes, ckpt)
}

/// Parse + widen an in-memory safetensors buffer (the test seam).
pub fn from_bytes(bytes: &[u8], ckpt: &'static str) -> Result<HashMap<String, Weights>> {
    let bad = |detail: String| LayaError::Pin {
        checkpoint: ckpt,
        file: "model.safetensors".to_string(),
        detail,
    };
    if bytes.len() < 8 {
        return Err(bad(format!("{} bytes: no header length", bytes.len())));
    }
    let header_len = u64::from_le_bytes(bytes[..8].try_into().expect("8 bytes")) as usize;
    let data_start = 8usize
        .checked_add(header_len)
        .ok_or_else(|| bad("header length overflow".into()))?;
    if bytes.len() < data_start {
        return Err(bad(format!(
            "header claims {header_len} bytes but the file has {}",
            bytes.len()
        )));
    }
    let header: serde_json::Value =
        serde_json::from_slice(&bytes[8..data_start]).map_err(|e| bad(format!("header: {e}")))?;
    let entries = header
        .as_object()
        .ok_or_else(|| bad("header is not a JSON object".into()))?;

    let mut out: HashMap<String, Weights> = HashMap::with_capacity(entries.len());
    for (name, entry) in entries {
        if name == "__metadata__" {
            continue;
        }
        let dtype = entry["dtype"]
            .as_str()
            .ok_or_else(|| bad(format!("{name}: missing dtype")))?;
        let shape: Vec<usize> = entry["shape"]
            .as_array()
            .ok_or_else(|| bad(format!("{name}: missing shape")))?
            .iter()
            .map(|d| {
                d.as_u64()
                    .map(|v| v as usize)
                    .ok_or_else(|| bad(format!("{name}: non-integer shape dim")))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let offsets = entry["data_offsets"]
            .as_array()
            .ok_or_else(|| bad(format!("{name}: missing data_offsets")))?;
        if offsets.len() != 2 {
            return Err(bad(format!("{name}: data_offsets must be [begin, end]")));
        }
        let begin = offsets[0].as_u64().unwrap_or(u64::MAX) as usize;
        let end = offsets[1].as_u64().unwrap_or(0) as usize;
        let width = dtype_width(dtype, name, ckpt)?;
        let numel: usize = shape.iter().product();
        if end < begin || end - begin != numel * width {
            return Err(bad(format!(
                "{name}: data span {} bytes != {numel} × {width}",
                end.saturating_sub(begin)
            )));
        }
        let abs_begin = data_start
            .checked_add(begin)
            .ok_or_else(|| bad(format!("{name}: data offset overflow")))?;
        let abs_end = data_start
            .checked_add(end)
            .ok_or_else(|| bad(format!("{name}: data offset overflow")))?;
        if bytes.len() < abs_end {
            return Err(bad(format!(
                "{name}: data ends at {abs_end} but the file has {} bytes",
                bytes.len()
            )));
        }
        let data = widen(dtype, &bytes[abs_begin..abs_end], name, ckpt)?;
        out.insert(name.to_string(), Weights { shape, data });
    }
    Ok(out)
}

/// The storage byte width per element of `dtype`.
fn dtype_width(dtype: &str, name: &str, ckpt: &'static str) -> Result<usize> {
    match dtype {
        "F32" => Ok(4),
        "F16" => Ok(2),
        "F64" => Ok(8),
        other => Err(LayaError::Pin {
            checkpoint: ckpt,
            file: name.to_string(),
            detail: format!(
                "unsupported dtype {other:?} — the riir reader widens F16/F32/F64 only"
            ),
        }),
    }
}

/// Widen a tensor's storage bytes to f32 (little-endian throughout).
fn widen(dtype: &str, bytes: &[u8], name: &str, ckpt: &'static str) -> Result<Vec<f32>> {
    let bad = |detail: String| LayaError::Pin {
        checkpoint: ckpt,
        file: name.to_string(),
        detail,
    };
    let data: Vec<f32> = match dtype {
        "F32" => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect(),
        "F16" => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| f16_bits_to_f32(u16::from_le_bytes(*c)))
            .collect(),
        "F64" => bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|c| f64::from_le_bytes(*c) as f32)
            .collect(),
        other => {
            return Err(bad(format!(
                "unsupported dtype {other:?} — the riir reader widens F16/F32/F64 only"
            )));
        }
    };
    Ok(data)
}

/// Exact F16 → F32 widening (the spec's bit laws, including subnormals —
/// `sign · m · 2⁻²⁴` is exact in f32: a 10-bit mantissa times a power of
/// two).
#[must_use]
pub fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exp = (bits & 0x7C00) >> 10;
    let man = bits & 0x03FF;
    let bits32: u32 = match (exp, man) {
        (0, 0) => sign, // ±0
        (0, m) => {
            // Subnormal: value = m · 2⁻²⁴ — exact f32 product of an integer
            // ≤ 1023 with a power of two (10 significant bits < 24).
            let mag = (m as f32) * f32::from_bits((127 - 24) << 23);
            return if bits & 0x8000 != 0 { -mag } else { mag };
        }
        (0x1F, m) => sign | 0x7F80_0000 | (u32::from(m) << 13), // ±inf / NaN
        (e, m) => sign | ((u32::from(e) + 112) << 23) | (u32::from(m) << 13),
    };
    f32::from_bits(bits32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One F16 tensor + one F32 tensor + `__metadata__`, serialized by hand
    /// into the exact safetensors layout — validates header length, JSON
    /// parsing, offset math and widening end to end.
    #[test]
    fn parses_a_hand_built_safetensors_buffer() {
        // F16 payload: [0x3C00 (1.0), 0x4000 (2.0), 0x0000 (0.0)]
        let f16_data: [u8; 6] = [0x00, 0x3C, 0x00, 0x40, 0x00, 0x00];
        // F32 payload: [−1.5f32]
        let f32_data = (-1.5f32).to_le_bytes();
        let header = r#"{"__metadata__":{"hub":"test"},"a":{"dtype":"F16","shape":[3],"data_offsets":[0,6]},"b":{"dtype":"F32","shape":[1,1],"data_offsets":[6,10]}}"#;
        let mut buf = Vec::new();
        buf.extend_from_slice(&(header.len() as u64).to_le_bytes());
        buf.extend_from_slice(header.as_bytes());
        buf.extend_from_slice(&f16_data);
        buf.extend_from_slice(&f32_data);

        let map = from_bytes(&buf, "test").expect("parses");
        assert_eq!(map["a"].shape, vec![3]);
        assert_eq!(map["a"].data, vec![1.0, 2.0, 0.0]);
        assert_eq!(map["b"].shape, vec![1, 1]);
        assert_eq!(map["b"].data, vec![-1.5]);
    }

    /// The F16 widening bit laws on known values, including the subnormal /
    /// infinity / sign edges.
    #[test]
    fn f16_widening_is_exact_on_known_values() {
        assert_eq!(f16_bits_to_f32(0x0000), 0.0);
        assert!(f16_bits_to_f32(0x8000).is_sign_negative());
        assert_eq!(f16_bits_to_f32(0x8000), -0.0);
        assert_eq!(f16_bits_to_f32(0x3C00), 1.0);
        assert_eq!(f16_bits_to_f32(0xBC00), -1.0);
        assert_eq!(f16_bits_to_f32(0x4000), 2.0);
        // Max subnormal 0x03FF = 1023 · 2⁻²⁴; min normal 0x0400 = 2⁻¹⁴.
        let max_sub = f16_bits_to_f32(0x03FF);
        let expect_sub = 1023.0f32 * 2.0f32.powi(-24);
        assert_eq!(max_sub, expect_sub);
        assert_eq!(f16_bits_to_f32(0x0400), 2.0f32.powi(-14));
        assert!(f16_bits_to_f32(0x7C00).is_infinite());
        assert!(f16_bits_to_f32(0xFC00).is_infinite());
        assert!(f16_bits_to_f32(0x7E00).is_nan());
        // A normal with a fractional mantissa: 0x3555 = (1 + 341/1024)·2⁻².
        let v = f16_bits_to_f32(0x3555);
        let expect = (1.0 + 341.0 / 1024.0) * 2.0f32.powi(-2);
        assert!((v - expect).abs() < 1e-9);
    }

    /// An unknown dtype is refused LOUD (naming the tensor), never
    /// mis-widened.
    #[test]
    fn unknown_dtype_is_refused_loud() {
        let header = r#"{"t":{"dtype":"BF16","shape":[1],"data_offsets":[0,2]}}"#;
        let mut buf = Vec::new();
        buf.extend_from_slice(&(header.len() as u64).to_le_bytes());
        buf.extend_from_slice(header.as_bytes());
        buf.extend_from_slice(&[0x00; 2]);
        let err = from_bytes(&buf, "test").expect_err("refused");
        let msg = err.to_string();
        assert!(msg.contains("BF16") && msg.contains("t"), "{msg}");
    }
}
