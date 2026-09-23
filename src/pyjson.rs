//! The Python-JSON byte format — the crate's ONE writer for it (DRY home).
//!
//! Parity-critical (the pinned repo's `tests/test_criteria.py` pins these
//! bytes): `json.dumps(v, ensure_ascii=False, separators=(", ", ": "))` for
//! criterion + state rendering, and plain `json.dumps(v)` (ASCII-escaped,
//! default separators) for non-string `instructions`. `serde_json` alone
//! does NOT reproduce the separators (`{"desc":"phishing"}` vs
//! `{"desc": "phishing"}`) — one writer here serves every call site: the
//! laya lane's sequence rendering (via the `laya::render` re-export) AND the
//! ungated harness's modelless-lane state strings (Plan 603 T1.5), which
//! must be the SAME bytes.
//!
//! Python-`round(x, 4)` is half-even, and Python float repr is shortest
//! round-trip with `e+NN`-style exponents — both mirrored here so answer
//! envelopes and state strings match the reference byte-for-byte.

use serde_json::Value;

/// Render one JSON value as Python's `json.dumps(v, ensure_ascii=<flag>,
/// separators=(", ", ": "))`. Any `serde_json::Value` is serializable, so
/// the reference's `default=str` fallback is unreachable from a JSON
/// question definition and not ported.
#[must_use]
pub fn py_json(v: &Value, ensure_ascii: bool) -> String {
    let mut out = String::new();
    write_value(&mut out, v, ensure_ascii);
    out
}

fn write_value(out: &mut String, v: &Value, ascii: bool) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => write_number(out, n),
        Value::String(s) => write_string(out, s, ascii),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(out, item, ascii);
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            for (i, (k, val)) in map.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_string(out, k, ascii);
                out.push_str(": ");
                write_value(out, val, ascii);
            }
            out.push('}');
        }
    }
}

fn write_number(out: &mut String, n: &serde_json::Number) {
    if let Some(i) = n.as_i64() {
        out.push_str(&i.to_string());
    } else if let Some(u) = n.as_u64() {
        out.push_str(&u.to_string());
    } else if let Some(f) = n.as_f64() {
        out.push_str(&py_float_repr(f));
    } else {
        // Arbitrary-precision big ints only land here with serde_json's
        // `arbitrary_precision` feature, which this crate does not enable.
        out.push_str(&n.to_string());
    }
}

/// Python `json.dumps` string escaping. Control chars < 0x20 → `\uXXXX`
/// (except the named escapes); `ensure_ascii=true` also escapes every
/// non-ASCII code point to `\uXXXX`, BMP+ as surrogate pairs (lowercase hex,
/// exactly `CPython`).
fn write_string(out: &mut String, s: &str, ascii: bool) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c if ascii && (c as u32) > 0x7E => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Python `repr(float)` / `json.dumps(float)` — shortest round-trip decimal,
/// integral floats carry `.0`, exponent form is `e+NN` / `e-NN` (sign always,
/// at least two exponent digits) and kicks in outside `[1e-4, 1e16)`.
#[must_use]
pub fn py_float_repr(f: f64) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    let a = f.abs();
    if f != 0.0 && !(1e-4..1e16).contains(&a) {
        // exponent form: `format!("{:e}")` gives e.g. "1.5e-5" / "1e16";
        // Python wants "1.5e-05" / "1e+16".
        let s = format!("{f:e}");
        let (mant, exp) = s.split_once('e').expect("e-form contains 'e'");
        let exp: i32 = exp.parse().expect("e-form exponent parses");
        // Python keeps an integral mantissa bare ("1e+16", not "1.0e+16").
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{mant}e{sign}{:02}", exp.abs())
    } else {
        // decimal range: Rust's shortest repr is already Python-compatible
        // EXCEPT it may switch to exponent form earlier (1e15 → "1e15").
        let s = format!("{f:?}");
        if !s.contains('e') {
            s
        } else {
            expand_decimal(f)
        }
    }
}

/// Expand a float in the decimal range to full decimal notation by finding
/// the shortest round-tripping precision, then trimming trailing zeros
/// (keeping at least one fractional digit, as Python does for `49.0`).
fn expand_decimal(f: f64) -> String {
    for prec in 0..=17 {
        let s = format!("{f:.prec$}");
        if s.parse::<f64>() == Ok(f) {
            let trimmed = s.trim_end_matches('0');
            let trimmed = trimmed.trim_end_matches('.');
            let trimmed = if trimmed.contains('.') {
                trimmed.to_string()
            } else {
                format!("{trimmed}.0")
            };
            return trimmed;
        }
    }
    // Unreachable for finite f64 (17 significant digits round-trip).
    format!("{f}")
}

/// Port of `serialize_state` (`laya/common.py`): strings pass through;
/// dict/list states render through the Python-JSON writer with
/// `ensure_ascii=False` (Python's `json.dumps` default separators are
/// exactly `(", ", ": ")`).
#[must_use]
pub fn serialize_state(state: &Value) -> String {
    match state {
        Value::String(s) => s.clone(),
        other => py_json(other, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn criterion_bytes_match_python() {
        assert_eq!(
            py_json(&json!({"desc": "phishing"}), false),
            r#"{"desc": "phishing"}"#
        );
        assert_eq!(py_json(&json!(["a", "b"]), false), r#"["a", "b"]"#);
        assert_eq!(py_json(&json!(3), false), "3");
        assert_eq!(py_json(&json!(false), false), "false");
        assert_eq!(py_json(&json!(null), false), "null");
        assert_eq!(
            py_json(&json!({"d": "münchen"}), false),
            r#"{"d": "münchen"}"#
        );
        assert_eq!(
            py_json(&json!({"d": "münchen"}), true),
            r#"{"d": "m\u00fcnchen"}"#
        );
    }

    #[test]
    fn string_state_passes_through() {
        let s = json!("raw state text");
        assert_eq!(serialize_state(&s), "raw state text");
    }

    #[test]
    fn object_state_uses_python_separators() {
        let s = json!({"article": "hello", "id": 7, "ok": true, "meta": {"a": [1, 2]}});
        assert_eq!(
            serialize_state(&s),
            "{\"article\": \"hello\", \"id\": 7, \"ok\": true, \"meta\": {\"a\": [1, 2]}}"
        );
    }

    #[test]
    fn key_order_is_insertion_order() {
        // preserve_order serde_json: json! keeps literal order.
        let s = json!({"zebra": 1, "alpha": 2});
        assert_eq!(serialize_state(&s), "{\"zebra\": 1, \"alpha\": 2}");
    }

    #[test]
    fn non_ascii_passes_through_raw() {
        let s = json!({"t": "épée — ok"});
        assert_eq!(serialize_state(&s), "{\"t\": \"épée — ok\"}");
    }

    #[test]
    fn control_chars_escape_like_python() {
        let s = json!({"t": "a\u{1}b\n"});
        assert_eq!(serialize_state(&s), "{\"t\": \"a\\u0001b\\n\"}");
    }

    #[test]
    fn float_repr_matches_python() {
        assert_eq!(py_float_repr(49.0), "49.0");
        assert_eq!(py_float_repr(1e16), "1e+16");
        assert_eq!(py_float_repr(1.5e-5), "1.5e-05");
        assert_eq!(py_float_repr(0.1), "0.1");
    }
}
