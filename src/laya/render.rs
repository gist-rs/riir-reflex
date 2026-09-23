//! The Python-JSON byte format the reference renders sequences through.
//!
//! Parity-critical (`tests/test_criteria.py` of the pinned repo pins these
//! bytes). The WRITER itself lives ungated in [`crate::pyjson`] (one DRY
//! home — the harness's modelless-lane state strings must be the SAME
//! bytes); this module re-exports it for the laya lane and keeps the
//! lane-local rendering (`render_options`, `py_round4`).

pub use crate::pyjson::{py_float_repr, py_json, serialize_state};

use serde_json::Value;

/// Python `round(x, 4)`: half-even on the decimal value. Rust's
/// `round_ties_even` operates on the binary value, which is what the
/// reference's `round(float(v), 4)` sees too (`float(np.float32)` first,
/// then half-even) — identical semantics at the 4th decimal for the
/// magnitudes probabilities live in.
pub fn py_round4(x: f64) -> f64 {
    (x * 10_000.0).round_ties_even() / 10_000.0
}

// ── option/state rendering (`laya/common.py`) ───────────────────────────────

use crate::laya::tokenize::InternalQuestion;

/// Port of `render_criterion`: strings pass through; everything structured
/// renders as compact Python-JSON (the reference's own docstring: a rubric
/// reads as JSON, never a Python repr).
fn render_criterion(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => py_json(other, false),
    }
}

fn is_none_or_empty(v: &Value) -> bool {
    v.is_null() || v.as_str().is_some_and(str::is_empty)
}

/// Port of `render_options`: the option TEXTS in label-index order. Noul is
/// always `[false, true]`. For `choice` only `null` / `""` mean "no
/// description" — `0` and `false` are real criterion values and render as
/// such. The option ORDER is the JSON object's insertion order
/// (`serde_json`'s `preserve_order` feature — parity-critical: it IS the
/// label order the head scores against).
///
/// Deviations from the reference, both loud instead of degenerate: `score`
/// criteria must be a JSON array (the reference's `enumerate` over a dict
/// would render its keys as levels — a shape the checkpoints were never
/// trained on), and `noul` criteria must be an object or null.
pub fn render_options(q: &InternalQuestion) -> Vec<String> {
    match q.t {
        "choice" => {
            let mut out = Vec::new();
            if let Some(map) = q.crit.as_object() {
                for (k, v) in map {
                    if is_none_or_empty(v) {
                        out.push(k.clone());
                    } else {
                        out.push(format!("{k}: {}", render_criterion(v)));
                    }
                }
            }
            out
        }
        "score" => {
            let crit = q.crit.as_array().cloned().unwrap_or_default();
            crit.iter()
                .enumerate()
                .map(|(i, c)| format!("level {i}: {}", render_criterion(c)))
                .collect()
        }
        _ => {
            let get = |side: &str| -> Option<Value> {
                q.crit.as_object().and_then(|m| m.get(side).cloned())
            };
            let false_crit = get("false");
            let true_crit = get("true");
            let render_side = |name: &str, v: &Option<Value>, default: &str| -> String {
                let body = match v {
                    Some(v) if !is_none_or_empty(v) => render_criterion(v),
                    _ => default.to_string(),
                };
                format!("{name}: {body}")
            };
            vec![
                render_side("false", &false_crit, "no, the statement does not hold"),
                render_side("true", &true_crit, "yes, the statement holds"),
            ]
        }
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
        // nested composite
        assert_eq!(
            py_json(&json!({"code": 500, "path": ["pay", 1]}), false),
            r#"{"code": 500, "path": ["pay", 1]}"#
        );
        // 0 and false are REAL criterion values
        assert_eq!(py_json(&json!(0), false), "0");
        assert_eq!(py_json(&json!(Value::Null), true), "null");
    }

    #[test]
    fn float_repr_matches_python() {
        assert_eq!(py_float_repr(49.0), "49.0");
        assert_eq!(py_float_repr(0.1), "0.1");
        assert_eq!(py_float_repr(0.0), "0.0");
        assert_eq!(py_float_repr(-2.5), "-2.5");
        assert_eq!(py_float_repr(1e16), "1e+16");
        assert_eq!(py_float_repr(1.5e-5), "1.5e-05");
        assert_eq!(py_float_repr(1e15), "1000000000000000.0");
        assert_eq!(py_float_repr(1e-4), "0.0001");
    }

    #[test]
    fn round4_is_half_even() {
        assert_eq!(py_round4(0.20505), 0.205); // ties-to-even on the 4th decimal
        assert_eq!(py_round4(0.1234499999), 0.1234);
        assert_eq!(py_round4(0.99995), 1.0);
    }
}
