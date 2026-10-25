//! The OpenDecider SELF-HOSTED lane (rethink Issue 038 T1 — the
//! OpenDecider-nano P0 admission rig; rethink Research 013 is the candidate
//! note of record): `manjunathshiva/opendecider-nano` (~400M,
//! Ettin-encoder-400m backbone + MLP decision head, Apache-2.0 with LICENSE
//! file present) served by THEIR production server (`opendecider serve`,
//! pip `opendecider[serve]`), which speaks TypeSafe Jev's `/v1/systemone`
//! API natively — the same wire the mammoth/elephant/pplx lanes drive.
//!
//! WHY THIS LANE EXISTS: nano is the third encoder-candidate (the 034 hold
//! lane) and its P0 is a MEASUREMENT, not a training run — one frozen
//! `typed_decisions` read through THIS lane against the pre-registered bar
//! 0.7825 (Issue 038; the gate, postures, fail-classification and
//! selection-on-test caveats are pinned THERE, before any read). The lane
//! is measurement+comparison only; an adoption/seat move is a separate
//! owner-gated posture (ESC re-arm, Research 002).
//!
//! LICENSE (verified 2026-10-25 via the HF API + the card): weights
//! Apache-2.0, LICENSE FILE PRESENT in-repo (the mammoth/elephant T0
//! caveat does not apply), base Ettin-encoder-400m MIT. Permissive; no
//! RESTRICTED posture.
//!
//! Wire: the pplx-family SystemOne grammar — the wire functions are
//! REUSED from the pplx lane ([`crate::lanes::pplx::build_body_for_model`]
//! /`map_answers`/`input_tokens_of`), one wire, many oracles. Their
//! server answers keyed per question id: choice keyed by OPTION LABEL,
//! score by level, noul as a probability — the family shape. THE T1
//! WIRE-SMOKE (038: train rows or synthetic rows ONLY, never test rows)
//! verifies the dialect on the live serve before any measured read; a
//! divergence lands as a recorded parser delta, never a silent remap.
//!
//! Determinism: the compare is VERBATIM (the pplx posture) — their server
//! is not the qev stack, and no timing key is known in its envelope. If
//! the smoke's repeat discloses a clock key, the mammoth precedent applies
//! (a projection + a test, committed before the measured read).
//!
//! Env: `OPENDECIDER_SERVE_URL` (default `http://127.0.0.1:8000` — their
//! `opendecider serve` default), `OPENDECIDER_API_TOKEN` (optional bearer;
//! their server supports auth, default off), `OPENDECIDER_MODEL` (the
//! provenance label, default `manjunathshiva/opendecider-nano`),
//! `OPENDECIDER_DEVICE` (the posture stamp, default
//! `opendecider-serve-local`), `OPENDECIDER_TIMEOUT_MS` (default 120 s),
//! `OPENDECIDER_SMOKE_MAX_CASES` — spend ceiling, default 50 cases; a
//! larger run refuses unless `OPENDECIDER_ALLOW_UNCAPPED=1` (the clef A6
//! law; a local serve is zero-marginal-cost, which is exactly what the
//! `=1` posture records for the P0's 400-case read).

use std::time::{Duration, Instant};

use serde_json::{Map, Value};

use crate::harness::suites::SuiteCase;
use crate::lanes::http_mini;
use crate::lanes::pplx::{AnswerTriple, build_body_for_model, input_tokens_of, map_answers};

/// The env var naming the serve base URL.
pub const SERVE_URL_ENV: &str = "OPENDECIDER_SERVE_URL";
/// The default posture: their `opendecider serve` default port.
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:8000";
/// The optional bearer token (their server's auth; unset = the default
/// no-auth posture).
pub const TOKEN_ENV: &str = "OPENDECIDER_API_TOKEN";
/// The provenance label (the model field + the record's lane name).
pub const MODEL_ENV: &str = "OPENDECIDER_MODEL";
pub const DEFAULT_MODEL: &str = "manjunathshiva/opendecider-nano";
/// The posture disclosure stamp.
pub const DEVICE_ENV: &str = "OPENDECIDER_DEVICE";
pub const DEFAULT_DEVICE: &str = "opendecider-serve-local";
/// Per-request timeout, milliseconds (a CPU-serve worst case is ~0.7 s per
/// batched case; generous by design — the capture loop is not
/// latency-bound).
pub const TIMEOUT_ENV: &str = "OPENDECIDER_TIMEOUT_MS";
pub const DEFAULT_TIMEOUT_MS: u64 = 120_000;
/// The spend ceiling (cases per suite), the clef A6 law.
pub const MAX_CASES_ENV: &str = "OPENDECIDER_SMOKE_MAX_CASES";
pub const DEFAULT_MAX_CASES: usize = 50;
/// The explicit uncapped override (`=1`) — the P0's 400-case read rides
/// this (a local serve is zero-marginal-cost; the posture is recorded).
pub const UNCAPPED_ENV: &str = "OPENDECIDER_ALLOW_UNCAPPED";
/// The order-canonicalization lever (`=1`): sort CHOICE criteria entries
/// by option key in the WIRE body, so every presentation of the same
/// question collapses to one canonical order (the bench-163 rule-(d)
/// lever — nano's choice answers are option-order-sensitive, 5/6 choice
/// suites RED; the name-keyed answer mapping needs no inverse remap).
/// DEFAULT OFF = the incumbent posture byte-identical; the fresh read
/// rides `=1` and stamps the lever in its record (the render-spec law:
/// a wire change re-registers).
pub const OPTION_SORT_ENV: &str = "OPENDECIDER_OPTION_SORT";

/// The lane: a thin plaintext-HTTP/1.1 client to their production serve.
/// std-only (the shared `crate::lanes::http_mini` micro-client; no new
/// deps).
#[derive(Debug, Clone)]
pub struct OpenDeciderLane {
    host: String,
    port: u16,
    token: Option<String>,
    model: String,
    timeout: Duration,
}

/// The order-canonicalization applied when [`OPTION_SORT_ENV`]`=1`:
/// every CHOICE question's criteria object is rebuilt in ascending option-
/// key order (the wire's presentation order IS the object's insertion
/// order — serde_json preserve_order; the perm-probe's
/// [`crate::harness::permutation`] relies on the same fact in the
/// opposite direction). Score arrays are ORDERED LEVELS — permuting them
/// changes the question, never the presentation (the permutation law) —
/// and noul's `[false, true]` is fixed; both pass through untouched. The
/// response join is NAME-keyed (map_answers reads the response dict per
/// OUR criteria key), so no inverse remap exists or is needed.
fn canonicalize_option_order(body: &mut Value) {
    if !(std::env::var(OPTION_SORT_ENV).is_ok_and(|v| v == "1")) {
        return;
    }
    let Some(questions) = body.get_mut("questions").and_then(Value::as_object_mut) else {
        return;
    };
    for q in questions.values_mut() {
        if q.get("type").and_then(Value::as_str) != Some("choice") {
            continue;
        }
        let Some(criteria) = q.get_mut("criteria").and_then(Value::as_object_mut) else {
            continue;
        };
        let sorted: Map<String, Value> = criteria
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
            .into_iter()
            .collect();
        *criteria = sorted;
    }
}

impl OpenDeciderLane {
    /// Build from the env contract. No creds gate (a local serve needs
    /// none by default); the liveness gate is [`OpenDeciderLane::info`]'s
    /// handshake — an unreachable serve is a loud error there.
    pub fn from_env() -> Result<Self, String> {
        let url = std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into());
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(&url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(80)),
            None => (rest.to_string(), 80),
        };
        Ok(Self {
            host,
            port,
            token: std::env::var(TOKEN_ENV).ok().filter(|t| !t.is_empty()),
            model: std::env::var(MODEL_ENV)
                .ok()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| DEFAULT_MODEL.into()),
            timeout: Duration::from_millis(
                std::env::var(TIMEOUT_ENV)
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS),
            ),
        })
    }

    /// The served model label (the env-selected provenance — never a
    /// hardcoded name, the gliner law).
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The posture disclosure stamp ([`DEVICE_ENV`]).
    pub fn device(&self) -> String {
        std::env::var(DEVICE_ENV).unwrap_or_else(|_| DEFAULT_DEVICE.into())
    }

    /// Build directly (the test seam — the env contract is pinned by the
    /// constants + the env tests; the wire tests never touch process env).
    #[cfg(test)]
    pub(crate) fn for_test(addr: std::net::SocketAddr, token: Option<&str>, model: &str) -> Self {
        Self {
            host: addr.ip().to_string(),
            port: addr.port(),
            token: token.map(str::to_string),
            model: model.to_string(),
            timeout: Duration::from_secs(10),
        }
    }

    /// The handshake (the clef law): ONE minimal `noul` probe. A 200 with
    /// a well-formed `answers` object carrying the probe's qid proves the
    /// wire shape end to end — no GET /health is assumed (their server's
    /// health route is their own; the probe is the wire truth). The
    /// reply's own `model` field is the provenance when the envelope
    /// provides it. Returns `(model_echo, device)`.
    pub fn info(&self) -> Result<(String, String), String> {
        let probe = SuiteCase {
            id: "handshake".into(),
            state: Value::String(
                "handshake: the lane's minimal noul probe (the clef A2 law; \
                 discarded, never a measured case)"
                    .into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "probe".into(),
                kind: crate::harness::suites::QKind::Noul,
                instructions: "Is this the handshake probe?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let body = build_body_for_model(&probe, &self.model)?.to_string();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("opendecider handshake: {e}"))?;
        if status != 200 {
            return Err(format!(
                "opendecider handshake -> HTTP {status}: {} — is `opendecider \
                 serve --model {DEFAULT_MODEL}` up at {SERVE_URL_ENV} (default \
                 {DEFAULT_SERVE_URL})? (404 = the route; a refused criteria-less \
                 noul is the 038-T1 wrapper case — re-run the wire-smoke)",
                String::from_utf8_lossy(&raw),
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw).map_err(|e| {
            format!(
                "opendecider handshake parse: {e} — the reply is not the \
                 SystemOne shape; check the serve route"
            )
        })?;
        // STRICT: the answers object must exist and carry the probe's qid —
        // the wire proof, never a half-check (the clef A2 law).
        let answers = parsed
            .get("answers")
            .and_then(Value::as_object)
            .ok_or("opendecider handshake: no answers object — not the SystemOne wire")?;
        if !answers.contains_key("probe") {
            return Err(
                "opendecider handshake: answers carries no qid for the probe — not the \
                 SystemOne wire shape this lane pins"
                    .to_string(),
            );
        }
        let echo = parsed
            .get("model")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .unwrap_or(&self.model)
            .to_string();
        Ok((echo, self.device()))
    }

    /// One decision round: build the body (the pplx wire grammar), POST it,
    /// map the answers back (the pplx parser, verbatim). Returns `(answer
    /// triples, client_ms, input_tokens)` — the client round trip is the
    /// cross-lane latency measure and is DISCLOSED as a local-serve row,
    /// never pooled with on-card figures.
    pub fn decide(&self, case: &SuiteCase) -> Result<(Vec<AnswerTriple>, f64, u64), String> {
        let mut body = build_body_for_model(case, &self.model)?;
        canonicalize_option_order(&mut body);
        let body = body.to_string();
        let t0 = Instant::now();
        let (status, _headers, raw) = self
            .post(&body)
            .map_err(|e| format!("opendecider round trip ({}): {e}", case.id))?;
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if status != 200 {
            return Err(format!(
                "opendecider round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        let parsed: Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("opendecider response ({}): {e}", case.id))?;
        let answers = map_answers(case, &parsed)?;
        let input_tokens = input_tokens_of(&parsed).unwrap_or(0);
        Ok((answers, client_ms, input_tokens))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// in-memory compare input. Nothing is persisted from the measured
    /// read (the 038 grammar-not-answers law: no test row enters any
    /// artifact).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let mut body = build_body_for_model(case, &self.model)?;
        canonicalize_option_order(&mut body);
        let body = body.to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "opendecider round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    /// The determinism-compare projection: VERBATIM passthrough (the pplx
    /// posture). Their server is not the qev stack — no timing key is
    /// known in the envelope; if the 038-T1 smoke's repeat discloses one,
    /// the mammoth precedent applies (a named projection + a test,
    /// committed BEFORE the measured read — never a silent compare change).
    pub fn determinism_view(raw: &str) -> Result<String, String> {
        serde_json::from_str::<Value>(raw)
            .map(|v| v.to_string())
            .map_err(|e| format!("determinism view parse: {e}"))
    }

    fn post(&self, body: &str) -> Result<http_mini::HttpReply, String> {
        let bearer = self.token.as_deref().map(|t| format!("Bearer {t}"));
        let headers: &[(&str, &str)] = match &bearer {
            Some(b) => &[("authorization", b.as_str())],
            None => &[],
        };
        http_mini::request(
            &self.host,
            self.port,
            "POST",
            "/v1/systemone",
            Some(body.as_bytes()),
            self.timeout,
            headers,
        )
        .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::QKind;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::sync::Mutex;

    /// The env contract is process-global; every env-touching test takes
    /// the guard (serializes them — the shared-temp-path lesson, in-process
    /// form).
    static ENV_GUARD: Mutex<()> = Mutex::new(());

    /// Set/clear env vars for one test, restoring on drop (the
    /// sibling-agents lesson: never leak env into other tests). The CALLER
    /// wraps the constructors in `unsafe { }` — edition 2024 marks the
    /// process-env mutation unsafe; ENV_GUARD is the safety invariant.
    struct EnvPatch(Vec<(String, Option<String>)>);
    impl EnvPatch {
        fn set(k: &str, v: &str) -> Self {
            let old = std::env::var(k).ok();
            // SAFETY: ENV_GUARD held by the caller — no concurrent env reader.
            unsafe { std::env::set_var(k, v) };
            Self(vec![(k.into(), old)])
        }
        fn clear(k: &str) -> Self {
            let old = std::env::var(k).ok();
            // SAFETY: ENV_GUARD held by the caller — no concurrent env reader.
            unsafe { std::env::remove_var(k) };
            Self(vec![(k.into(), old)])
        }
    }
    impl Drop for EnvPatch {
        fn drop(&mut self) {
            for (k, old) in &self.0 {
                // SAFETY: ENV_GUARD held by the test that owns this patch.
                match old {
                    Some(v) => unsafe { std::env::set_var(k, v) },
                    None => unsafe { std::env::remove_var(k) },
                }
            }
        }
    }

    fn sample_case() -> SuiteCase {
        SuiteCase {
            id: "c1".into(),
            state: Value::String("state".into()),
            questions: vec![
                crate::harness::suites::SuiteQuestion {
                    qid: "q_choice".into(),
                    kind: QKind::Choice,
                    instructions: "pick".into(),
                    criteria: json!({"team": "a", "billing": "b", "sales": "c"}),
                },
                crate::harness::suites::SuiteQuestion {
                    qid: "q_noul".into(),
                    kind: QKind::Noul,
                    instructions: "yes?".into(),
                    criteria: Value::Null,
                },
            ],
            gold: vec![],
        }
    }

    /// A one-shot stub server (the d3/startlux/mammoth stub, verbatim —
    /// one shape, many lanes): serves `canned` with `status` for EVERY
    /// request. Each served request reads the request IN FULL.
    fn serve_with(status: u16, canned: String) -> std::net::SocketAddr {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || loop {
            let (stream, _) = match listener.accept() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut writer = match stream.try_clone() {
                Ok(w) => w,
                Err(_) => continue,
            };
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue;
            }
            let mut content_length = 0usize;
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end();
                        if trimmed.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = trimmed.split_once(':')
                            && k.eq_ignore_ascii_case("content-length")
                        {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                }
            }
            let mut body = vec![0u8; content_length];
            if reader.read_exact(&mut body).is_err() {
                continue;
            }
            let resp = format!(
                "HTTP/1.1 {status} ST\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                canned.len(),
                canned
            );
            let _ = writer.write_all(resp.as_bytes());
            let _ = writer.flush();
        });
        addr
    }

    fn answers_body(model: &str) -> String {
        json!({
            "model": model,
            "answers": {
                "probe": {"type": "noul", "noul": 0.9},
                "q_choice": {
                    "type": "choice",
                    "choice": "billing",
                    "confidence": 0.9948,
                    "probabilities": {"team": 0.003, "billing": 0.9948, "sales": 0.0022},
                },
                "q_noul": {"type": "noul", "noul": 0.7449},
            },
            "usage": {"input_tokens": 191, "output_tokens": 0},
        })
        .to_string()
    }

    #[test]
    fn from_env_defaults_match_the_constants() {
        let _g = ENV_GUARD.lock().unwrap();
        let _u = EnvPatch::clear(SERVE_URL_ENV);
        let _t = EnvPatch::clear(TOKEN_ENV);
        let _m = EnvPatch::clear(MODEL_ENV);
        let _d = EnvPatch::clear(DEVICE_ENV);
        // from_env reads the URL env; assert the contract constants instead
        // of the env (the for_test seam keeps the wire tests env-free).
        assert_eq!(DEFAULT_SERVE_URL, "http://127.0.0.1:8000");
        assert_eq!(DEFAULT_MODEL, "manjunathshiva/opendecider-nano");
        assert_eq!(DEFAULT_DEVICE, "opendecider-serve-local");
        assert_eq!(DEFAULT_MAX_CASES, 50);
        let _lane = OpenDeciderLane::from_env().expect("no creds gate — local serve");
    }

    #[test]
    fn token_and_model_envs_are_honored() {
        let _g = ENV_GUARD.lock().unwrap();
        let _u = EnvPatch::set(SERVE_URL_ENV, "http://127.0.0.1:8123");
        let _t = EnvPatch::set(TOKEN_ENV, "tok-1");
        let _m = EnvPatch::set(MODEL_ENV, "other/model");
        let _d = EnvPatch::set(DEVICE_ENV, "cuda");
        let lane = OpenDeciderLane::from_env().unwrap();
        assert_eq!(lane.model(), "other/model");
        assert_eq!(lane.device(), "cuda");
        assert_eq!(lane.port, 8123);
    }

    #[test]
    fn stub_listener_round_trip() {
        let addr = serve_with(200, answers_body("manjunathshiva/opendecider-nano"));
        let lane = OpenDeciderLane::for_test(addr, None, "manjunathshiva/opendecider-nano");

        let (model_echo, device) = lane.info().unwrap();
        assert_eq!(model_echo, "manjunathshiva/opendecider-nano");
        assert_eq!(device, "opendecider-serve-local");

        let c = sample_case();
        let (answers, _ms, tokens) = lane.decide(&c).unwrap();
        assert_eq!(answers.len(), 2);
        let (probs, pick, conf) = &answers[0];
        assert_eq!(probs.len(), 3);
        assert_eq!(*pick, 1, "billing carries the max prob");
        assert!((conf - 0.9948).abs() < 1e-12);
        assert_eq!(tokens, 191);

        let raw = lane.decide_raw(&c).unwrap();
        assert!(raw.contains("\"probabilities\""), "{raw}");

        // The determinism view is the VERBATIM posture (canonical JSON
        // round trip, content-equal).
        let view = OpenDeciderLane::determinism_view(&raw).unwrap();
        assert_eq!(view, raw);
    }

    #[test]
    fn option_sort_default_off_is_byte_identical() {
        let _g = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        let _patch = EnvPatch::clear(OPTION_SORT_ENV);
        let mut body = json!({
            "model": "m",
            "state": "s",
            "questions": {
                "q": {"type": "choice", "instructions": "i",
                       "criteria": {"zeta": "Z", "alpha": "A", "mid": "M"}}
            }
        });
        let before = body.to_string();
        canonicalize_option_order(&mut body);
        assert_eq!(body.to_string(), before, "default-off must be byte-identical");
    }

    #[test]
    fn option_sort_canonicalizes_choice_criteria_only() {
        let _g = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        let _patch = EnvPatch::set(OPTION_SORT_ENV, "1");
        let mut body = json!({
            "model": "m",
            "state": "s",
            "questions": {
                "c": {"type": "choice", "instructions": "i",
                      "criteria": {"zeta": "Z", "alpha": "A", "mid": "M"}},
                "s": {"type": "score", "instructions": "i",
                      "criteria": ["very negative", "negative", "neutral"]},
                "n": {"type": "noul", "instructions": "i"}
            }
        });
        canonicalize_option_order(&mut body);
        let qs = body.get("questions").unwrap();
        let choice_keys: Vec<&str> = qs["c"]["criteria"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(choice_keys, vec!["alpha", "mid", "zeta"], "choice sorted by key");
        let score_levels: Vec<&str> = qs["s"]["criteria"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            score_levels,
            vec!["very negative", "negative", "neutral"],
            "score levels are ordered semantics — never permuted"
        );
    }

    #[test]
    fn permuted_presentation_collapses_to_the_same_wire_body() {
        // The bench-163 lever's actual claim: two presentations of the same
        // question (original order vs permuted insertion order) produce the
        // IDENTICAL wire body under the lever.
        let _g = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        let _patch = EnvPatch::set(OPTION_SORT_ENV, "1");
        let q_orig = json!({"type": "choice", "instructions": "i",
                            "criteria": {"a": "A", "b": "B", "c": "C"}});
        let q_perm = json!({"type": "choice", "instructions": "i",
                            "criteria": {"c": "C", "a": "A", "b": "B"}});
        let mut b1 = json!({"model": "m", "state": "s", "questions": {"q": q_orig}});
        let mut b2 = json!({"model": "m", "state": "s", "questions": {"q": q_perm}});
        canonicalize_option_order(&mut b1);
        canonicalize_option_order(&mut b2);
        assert_eq!(b1.to_string(), b2.to_string(), "canonical order collapses presentations");
    }

    #[test]
    fn handshake_refusal_names_the_serve_and_the_env() {
        let addr = serve_with(404, "{\"error\":\"no route\"}".into());
        let lane = OpenDeciderLane::for_test(addr, None, "manjunathshiva/opendecider-nano");
        let err = lane.info().unwrap_err();
        assert!(err.contains("opendecider serve"), "{err}");
        assert!(err.contains(SERVE_URL_ENV), "{err}");
    }

    #[test]
    fn handshake_rejects_a_non_systemone_body() {
        let addr = serve_with(200, "{\"hello\":\"world\"}".into());
        let lane = OpenDeciderLane::for_test(addr, None, "m");
        let err = lane.info().unwrap_err();
        assert!(err.contains("no answers object"), "{err}");

        let addr = serve_with(200, "{\"answers\":{\"other\":1}}".into());
        let lane = OpenDeciderLane::for_test(addr, None, "m");
        let err = lane.info().unwrap_err();
        assert!(err.contains("no qid for the probe"), "{err}");
    }

    #[test]
    fn bearer_token_rides_the_request_when_set() {
        // The stub records the headers it saw; the lane must attach
        // `authorization: Bearer tok-9` when OPENDECIDER_API_TOKEN is set.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let canned = answers_body("m");
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut saw_auth = false;
            let mut content_length = 0usize;
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end();
                        if trimmed.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = trimmed.split_once(':') {
                            if k.eq_ignore_ascii_case("authorization") {
                                saw_auth = v.trim() == "Bearer tok-9";
                            }
                            if k.eq_ignore_ascii_case("content-length") {
                                content_length = v.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }
            }
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).unwrap();
            let resp = format!(
                "HTTP/1.1 200 ST\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                canned.len(),
                canned
            );
            let _ = writer.write_all(resp.as_bytes());
            let _ = writer.flush();
            saw_auth
        });
        let lane = OpenDeciderLane::for_test(addr, Some("tok-9"), "m");
        lane.info().unwrap();
        assert!(handle.join().unwrap(), "the bearer must ride the request");
    }
}
