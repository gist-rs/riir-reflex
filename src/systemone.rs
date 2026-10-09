//! Issue 081: the TypeSafe `/v1/systemone` dialect on the serve edge —
//! reflex as a System-One BACKEND (Jev-Mem's controller slot, and any
//! `typesafe_sdk` client generally).
//!
//! Direction is the INVERSE of the comparison lanes: there our harness
//! calls THEIR servers (agentjev/clef/pplx); here THEIR SDK calls us.
//! Jev-Mem plugs in with three env vars and zero forks —
//! `decision_backend=jev` (their default), `TYPESAFE_BASE_URL=<our loopback>`,
//! `TYPESAFE_API_KEY=<dummy>` — because `JevMemConfig.load` honors
//! `TYPESAFE_BASE_URL` and their `JevClient.evaluate` posts through the
//! real SDK (`typesafe_sdk` 0.7.2, `SYSTEM_ONE_PATH = "/v1/systemone"`).
//!
//! Wire (read from the SDK wheel, not guessed):
//! - request  `{state: <json>, model: <str>, questions: {qid: qdef}}`
//! - qdef     `{type: "noul"|"choice"|"score", instructions?: <json>,
//!             criteria?: noul `{true,false}` | choice `{label: desc|null}`
//!             (ordered) | score `[desc, ...]` (from zero)}`
//! - response `{model, usage, answers: {qid: answer}}`
//! - NoulAnswer   `{type:"noul", noul: p_yes}`
//! - ChoiceAnswer `{type:"choice", choice, confidence, probabilities}`
//!   (probabilities sum ≈ 1 ± 1e-3 AND `choice` must be a highest-prob
//!   option — Jev-Mem's `_validate` refuses otherwise)
//! - ScoreAnswer  `{type:"score", score, confidence, legend, probabilities}`
//!   (legend/probabilities keyed by integer level)
//!
//! The TypeSafe wire has NO abstention: every answer is a number. The
//! engine's abstain maps honestly — noul → `0.5` (their own "uncertainty"
//! semantics), choice/score → the carried distribution when present, else
//! a uniform one with a deterministic argmax (ties → first). usage is `{}`
//! — the modelless engine counts no tokens (the `generated_tokens: 0`
//! posture: we never generate).

use crate::engine::DecisionEngine;
use katgpt_core::decision_wire::{
    DecisionRequest, DecisionResponse, Question, QuestionKind,
};
use serde_json::{Map, Value};

/// What the response's `model` field names: what WE are, never an echo of
/// the requested model (the real API answers with the model that answered;
/// a modelless engine that echoed `jev-latest` would be a false claim).
pub const MODEL_NAME: &str = "reflex-modelless";

/// The engine's embedding dimension is a serve-level constant; the route is
/// generic over the corpus width `N` exactly like the rest of the edge.
#[derive(Debug)]
pub(crate) struct SystemOneBuilt {
    pub request: DecisionRequest,
    /// The translated questions, echoed-aligned with the response's answers
    /// (label lookup for choice, rubric for score).
    questions: Vec<Question>,
    /// Wire-shape record: `true` where the WIRE question was `noul` — the
    /// answer must carry the `{type: noul, noul: p}` shape whatever the
    /// engine kind ended up being (see the noul arm in [`translate`]).
    wire_noul: Vec<bool>,
}

/// Render a TypeSafe `JSONContent` (text | object | array) into the bytes
/// the engine's embedder and drafter consume: text as-is, structured values
/// as compact canonical JSON (with `preserve_order`, byte-deterministic).
fn render_json_content(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// The spaced-rendering experiment flag (issue 081 T2c): read ONCE per
/// process (the G4 env law — the `ridge_debug_enabled` precedent).
fn spaced_state_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RIIR_REFLEX_SYSTEMONE_SPACED").is_some())
}

/// Render the request state as Python-`json.dumps`-default-shaped JSON
/// (`", "` / `": "` separators), recursively, preserving key order —
/// byte-deterministic. WHY (the measured law, issue 081 T2c): their client
/// serializes the state with ALPHABETICAL keys and compact separators, so
/// the `depth` digit is glued into the first whitespace token
/// (`{"depth":0,"evidence":[{"content":"[Audrey]: …`), a per-state-
/// unique chunk the count tables can never see on held-out states. The
/// spaced rendering separates `"depth":` and `0,` into clean standalone
/// tokens — the O(1)-token signal becomes reachable. Opt-in (the standing
/// compact rendering is the measured posture of Benches 133-135; every
/// ctx consumer — drafter, embedder, distance gate — shifts under spacing,
/// so adoption is a re-baseline decision, not a default).
fn render_state_spaced(v: &Value) -> String {
    match v {
        Value::String(s) => serde_json::to_string(s).unwrap_or_default(),
        Value::Array(a) => {
            let mut out = String::from("[");
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&render_state_spaced(item));
            }
            out.push(']');
            out
        }
        Value::Object(m) => {
            let mut out = String::from("{");
            for (i, (k, val)) in m.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&serde_json::to_string(k).unwrap_or_default());
                out.push_str(": ");
                out.push_str(&render_state_spaced(val));
            }
            out.push('}');
            out
        }
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Render noul/choice criteria into the engine's `Question::criteria`
/// string — both branches' descriptions must reach the drafter context
/// (`ctx = prompt + "\n" + criteria`), because the yes/no delta the engine
/// scores is semantic only when the descriptions are in the bytes.
fn render_criteria_map(criteria: &Map<String, Value>) -> String {
    let mut out = String::new();
    for (label, desc) in criteria {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(label);
        out.push_str(": ");
        out.push_str(&render_json_content(desc));
    }
    out
}

/// Translate a TypeSafe request into a `DecisionRequest`. Errors are the
/// route's 400 body — a malformed question is named, never guessed past.
pub(crate) fn translate(
    state: &Value,
    questions: &Map<String, Value>,
) -> Result<SystemOneBuilt, String> {
    if questions.is_empty() {
        return Err("questions must be a nonempty object keyed by question id"
            .into());
    }
    let mut built = Vec::with_capacity(questions.len());
    let mut wire_noul = Vec::with_capacity(questions.len());
    for (qid, def) in questions {
        let mut is_wire_noul = false;
        let Some(obj) = def.as_object() else {
            return Err(format!("question {qid:?} must be an object"));
        };
        let Some(qtype) = obj.get("type").and_then(Value::as_str) else {
            return Err(format!("question {qid:?} carries no string \"type\""));
        };
        let instructions = obj
            .get("instructions")
            .filter(|v| !v.is_null())
            .map(render_json_content)
            .unwrap_or_default();
        let criteria_val = obj.get("criteria").filter(|v| !v.is_null());
        let question = match qtype {
            "noul" => {
                // The SDK's noul criteria `{true: desc, false: desc}` are
                // OPTIONAL descriptions. WITH the canonical {true, false}
                // pair the question rides the CRITERIA-AS-OPTIONS arm: the
                // engine sees a 2-option Choice whose options ARE the two
                // descriptions, so the drafter scores semantic strings
                // against the routed domain's corpus (the banking77-class
                // competency) and p(yes) = probabilities[0]. The bare
                // engine-Noul kind scores the literal bytes `yes`/`no` —
                // measured flat ~0.495 for EVERY input (Bench 133: the
                // corpus is invisible to that arm; a 2-3 byte literal is a
                // constant compression offset, not a semantic decision).
                // Absent or non-canonical criteria keep the bare arm.
                let cmap = criteria_val.and_then(Value::as_object);
                let canonical = cmap.is_some_and(|c| {
                    c.len() == 2 && c.contains_key("true") && c.contains_key("false")
                });
                let question = if let (Some(c), true) = (cmap, canonical) {
                    let options = vec![
                        render_json_content(c.get("true").expect("checked")),
                        render_json_content(c.get("false").expect("checked")),
                    ];
                    // criteria: NONE on this arm — the descriptions ARE the
                    // options now, and rendering them into the ctx as well
                    // would let each candidate match the request itself
                    // (measured: the two scores converge and p pins at 0.5
                    // for every input — the criteria-in-ctx symmetricization).
                    // The dictionary, not the request, must decide.
                    Question {
                        id: qid.clone(),
                        kind: QuestionKind::Choice,
                        prompt: instructions,
                        options,
                        criteria: None,
                    }
                } else {
                    let criteria = cmap.map(render_criteria_map).unwrap_or_default();
                    Question {
                        id: qid.clone(),
                        kind: QuestionKind::Noul,
                        prompt: instructions,
                        options: Vec::new(),
                        criteria: if criteria.is_empty() {
                            None
                        } else {
                            Some(criteria)
                        },
                    }
                };
                is_wire_noul = true;
                question
            }
            "choice" => {
                let Some(criteria) = criteria_val
                    .and_then(Value::as_object)
                    .filter(|c| !c.is_empty())
                else {
                    return Err(format!(
                        "choice question {qid:?} requires nonempty criteria \
                         {{label: description}}"
                    ));
                };
                // Insertion order is preserved (serde_json preserve_order) —
                // the wire's option order is the answer's option order.
                let options: Vec<String> =
                    criteria.keys().cloned().collect();
                Question {
                    id: qid.clone(),
                    kind: QuestionKind::Choice,
                    prompt: instructions,
                    options,
                    criteria: Some(render_criteria_map(criteria)),
                }
            }
            "score" => {
                let Some(rubric) = criteria_val.and_then(Value::as_array).filter(
                    |a| !a.is_empty(),
                ) else {
                    return Err(format!(
                        "score question {qid:?} requires nonempty criteria \
                         [level descriptions, lowest first]"
                    ));
                };
                let options: Vec<String> =
                    rubric.iter().map(render_json_content).collect();
                Question {
                    id: qid.clone(),
                    kind: QuestionKind::Score,
                    prompt: instructions,
                    options,
                    criteria: None,
                }
            }
            other => {
                return Err(format!(
                    "question {qid:?} carries unknown type {other:?} \
                     (supported: noul, choice, score)"
                ))
            }
        };
        wire_noul.push(is_wire_noul);
        if let Err(e) = question.validate(built.len()) {
            return Err(format!("question {qid:?}: {e}"));
        }
        built.push(question);
    }
    let request = DecisionRequest {
        // The state crosses as canonical JSON (objects keep their wire
        // order — the same bytes their own `semantic()` would render); the
        // SPACED arm (issue 081 T2c) renders Python-default separators so
        // structural scalars (the `depth` digit) become standalone tokens —
        // see `render_state_spaced` for the measured law.
        state: if spaced_state_enabled() {
            render_state_spaced(state)
        } else {
            serde_json::to_string(state).unwrap_or_default()
        },
        questions: built.clone(),
    };
    if let Err(e) = request.validate() {
        return Err(e.to_string());
    }
    Ok(SystemOneBuilt {
        request,
        questions: built,
        wire_noul,
    })
}

/// Normalize a carried distribution to sum exactly 1 in f64 (the engine's
/// L1 is f32 — the rounding residue is far inside the ±1e-3 wire budget,
/// but exact is free and keeps the argmax relation untouched). Empty
/// input reads empty (the caller decides the abstention posture).
fn normalized(probs: &[f32]) -> Vec<f64> {
    let sum: f64 = probs.iter().map(|&p| p as f64).sum();
    if sum <= 0.0 || !sum.is_finite() {
        return Vec::new();
    }
    probs.iter().map(|&p| p as f64 / sum).collect()
}

/// The full distribution an abstention-capable engine answer maps onto the
/// wire: a carried, exactly-normalized distribution when the arity matches;
/// uniform when the engine abstained before scoring (the TypeSafe wire has
/// no abstention — a full distribution and an argmax pick are forced).
fn wire_distribution(probs: &[f32], k: usize) -> Vec<f64> {
    if probs.is_empty() {
        return vec![1.0 / k as f64; k];
    }
    let n = normalized(probs);
    if n.len() == k {
        n
    } else {
        vec![1.0 / k as f64; k]
    }
}

/// Round to 6 decimal places — the f32→f64 widening artifact (0.7f32 →
/// 0.69999998807…) never reaches a client's logs.
fn round6(v: f64) -> f64 {
    (v.max(0.0) * 1e6).round() / 1e6
}

/// Map the engine's answers back to TypeSafe answer objects, one per
/// question id (their `answers` is keyed, order irrelevant).
pub(crate) fn map_answers(
    built: &SystemOneBuilt,
    resp: &DecisionResponse,
) -> Map<String, Value> {
    use katgpt_core::decision_wire::Outcome;
    let mut out = Map::new();
    for ((q, a), &wire_noul) in built.questions.iter().zip(resp.answers.iter()).zip(built.wire_noul.iter()) {
        let probs = &a.probabilities;
        let mut answer = Map::new();
        // A wire-noul question answers `{type: noul, noul: p}` whatever the
        // engine kind: the criteria-as-options arm (engine Choice) reads
        // probabilities[0] = p(TRUE description); the bare arm (engine
        // Noul) reads probabilities[0] = p(yes) — the same slot. Abstained
        // (empty probs) reads 0.5 — the dialect's own "uncertainty"
        // semantics. A carried p passes through UNTOUCHED: it is a single
        // value, not a distribution — normalizing it would erase it
        // (p/p ≡ 1).
        if wire_noul {
            answer.insert("type".into(), "noul".into());
            let p = probs
                .first()
                .map(|&p| round6(p as f64))
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            answer.insert("noul".into(), Value::from(p));
            out.insert(q.id.clone(), Value::Object(answer));
            continue;
        }
        match q.kind {
            QuestionKind::Noul => {
                answer.insert("type".into(), "noul".into());
                // Abstained noul (empty probs) reads 0.5 — the dialect's own
                // "uncertainty" semantics. A carried p(yes) passes through
                // UNTOUCHED: it is a single value, not a distribution —
                // normalizing it would erase it (p/p ≡ 1).
                let p = probs
                    .first()
                    .map(|&p| round6(p as f64))
                    .unwrap_or(0.5)
                    .clamp(0.0, 1.0);
                answer.insert("noul".into(), Value::from(p));
            }
            QuestionKind::Choice => {
                let k = q.options.len().max(1);
                let dist = wire_distribution(probs, k);
                let pick = match a.outcome {
                    Some(Outcome::Choice { index }) => index as usize,
                    _ => argmax_by(&dist),
                };
                let mut probabilities = Map::new();
                for (label, p) in q.options.iter().zip(dist.iter()) {
                    probabilities.insert(label.clone(), Value::from(round6(*p)));
                }
                let choice = q
                    .options
                    .get(pick)
                    .cloned()
                    .unwrap_or_else(|| q.options[0].clone());
                answer.insert("type".into(), "choice".into());
                answer.insert("choice".into(), choice.into());
                answer.insert(
                    "confidence".into(),
                    Value::from(round6(a.confidence as f64)),
                );
                answer.insert("probabilities".into(), Value::Object(probabilities));
            }
            QuestionKind::Score => {
                let k = q.options.len().max(1);
                let dist = wire_distribution(probs, k);
                let level = match a.outcome {
                    Some(Outcome::Score { level }) => level as usize,
                    _ => argmax_by(&dist),
                };
                let mut legend = Map::new();
                let mut probabilities = Map::new();
                for (i, (desc, p)) in
                    q.options.iter().zip(dist.iter()).enumerate()
                {
                    legend.insert(i.to_string(), Value::String(desc.clone()));
                    probabilities.insert(
                        i.to_string(),
                        Value::from((p.max(0.0) * 1e6).round() / 1e6),
                    );
                }
                answer.insert("type".into(), "score".into());
                answer.insert("score".into(), Value::from(level as u64));
                answer.insert(
                    "confidence".into(),
                    Value::from(round6(a.confidence as f64)),
                );
                answer.insert("legend".into(), Value::Object(legend));
                answer.insert(
                    "probabilities".into(),
                    Value::Object(probabilities),
                );
            }
        }
        out.insert(q.id.clone(), Value::Object(answer));
    }
    out
}

/// f64 argmax over the normalized distribution (ties → first).
fn argmax_by(dist: &[f64]) -> usize {
    let mut best = 0usize;
    let mut bp = f64::MIN;
    for (i, &p) in dist.iter().enumerate() {
        if i == 0 || p > bp {
            best = i;
            bp = p;
        }
    }
    best.min(dist.len().saturating_sub(1))
}

/// The full route body: translate → engine → map → the TypeSafe response
/// JSON. `Ok` is ALWAYS a 200-shaped response; every `Err` is a 400 with
/// the named defect (the SDK surfaces it as `TypeSafeAPIError` and their
/// client fails loud — the correct posture for a malformed question).
pub(crate) fn respond<const N: usize, const D: usize>(
    engine: &mut DecisionEngine<N, D>,
    body: &[u8],
) -> Result<String, String> {
    let parsed: Value = serde_json::from_slice(body)
        .map_err(|e| format!("request body is not JSON: {e}"))?;
    let Some(state) = parsed.get("state") else {
        return Err("request carries no \"state\"".into());
    };
    let Some(questions) = parsed.get("questions") else {
        return Err("request carries no \"questions\" object".into());
    };
    let Some(questions) = questions.as_object() else {
        return Err("\"questions\" must be an object keyed by question id".into());
    };
    let built = translate(state, questions)?;
    let resp = engine
        .decide(&built.request)
        .map_err(|e| format!("engine refused the translated request: {e}"))?;
    let answers = map_answers(&built, &resp);
    let mut out = Map::new();
    out.insert("model".into(), MODEL_NAME.into());
    out.insert("usage".into(), Value::Object(Map::new()));
    out.insert("answers".into(), Value::Object(answers));
    serde_json::to_string(&Value::Object(out))
        .map_err(|e| format!("response serialization failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn built_from(questions: &Value) -> SystemOneBuilt {
        translate(&json!({"observation": "Alice prefers concise answers."}), questions
            .as_object()
            .unwrap())
            .expect("translate")
    }

    #[test]
    fn noul_translates_instructions_and_criteria() {
        // Canonical {true, false} criteria ride the CRITERIA-AS-OPTIONS arm:
        // the engine sees a 2-option Choice whose options ARE the two
        // descriptions (the drafter scores semantic strings — the bare
        // yes/no arm is measured flat for every input, Bench 133).
        let built = built_from(&json!({
            "should_store": {
                "type": "noul",
                "instructions": "Is this worth retaining?",
                "criteria": {"true": "A fact worth recall.", "false": "Only filler."}
            }
        }));
        let q = &built.questions[0];
        assert_eq!(q.id, "should_store");
        assert_eq!(q.kind, QuestionKind::Choice);
        assert_eq!(q.options, vec!["A fact worth recall.", "Only filler."]);
        assert_eq!(q.prompt, "Is this worth retaining?");
        // criteria is NONE on this arm — the descriptions ARE the options;
        // rendering them into the ctx too would let each candidate match
        // the request itself (p pins at 0.5 — measured).
        assert!(q.criteria.is_none());
        assert!(built.wire_noul[0]);
        // The state crosses as canonical JSON.
        assert!(built.request.state.contains("Alice prefers"));
    }

    #[test]
    fn noul_without_criteria_keeps_the_bare_arm() {
        let built = built_from(&json!({
            "q": {"type": "noul", "instructions": "Done?"}
        }));
        assert!(built.questions[0].criteria.is_none());
        assert_eq!(built.questions[0].kind, QuestionKind::Noul);
        assert!(built.questions[0].options.is_empty());
        assert!(built.wire_noul[0]);
    }

    #[test]
    fn noul_with_non_canonical_criteria_keeps_the_bare_arm() {
        // Criteria that are not exactly {true, false} (the SDK's canonical
        // noul shape) do not guess an option order — bare Noul arm.
        let built = built_from(&json!({
            "q": {
                "type": "noul",
                "instructions": "Done?",
                "criteria": {"yes": "a", "no": "b"}
            }
        }));
        assert_eq!(built.questions[0].kind, QuestionKind::Noul);
        assert!(built.questions[0].options.is_empty());
        assert!(built.questions[0].criteria.is_some());
        assert!(built.wire_noul[0]);
    }

    #[test]
    fn choice_options_keep_wire_order() {
        let built = built_from(&json!({
            "representation": {
                "type": "choice",
                "instructions": "Which representation fits?",
                "criteria": {
                    "keep_separate": "Contradictory accounts.",
                    "merge": "Compatible accounts.",
                    "promote": "A repeated pattern.",
                    "uncertain": "Insufficient evidence."
                }
            }
        }));
        let q = &built.questions[0];
        assert_eq!(q.kind, QuestionKind::Choice);
        assert_eq!(
            q.options,
            vec!["keep_separate", "merge", "promote", "uncertain"]
        );
        assert!(q.criteria.as_deref().unwrap().contains("merge: Compatible"));
    }

    #[test]
    fn choice_without_criteria_refuses() {
        let err = translate(
            &json!("s"),
            json!({"q": {"type": "choice", "instructions": "pick"}})
                .as_object()
                .unwrap(),
        )
        .unwrap_err();
        assert!(err.contains("requires nonempty criteria"), "{err}");
    }

    #[test]
    fn score_rubric_orders_from_zero() {
        let built = built_from(&json!({
            "urgency": {
                "type": "score",
                "instructions": "How urgent?",
                "criteria": ["low", "mid", "high"]
            }
        }));
        let q = &built.questions[0];
        assert_eq!(q.kind, QuestionKind::Score);
        assert_eq!(q.options, vec!["low", "mid", "high"]);
    }

    #[test]
    fn unknown_type_and_empty_questions_refuse() {
        let e1 = translate(
            &json!("s"),
            json!({"q": {"type": "emoji"}}).as_object().unwrap(),
        )
        .unwrap_err();
        assert!(e1.contains("unknown type"), "{e1}");
        let e2 =
            translate(&json!("s"), json!({}).as_object().unwrap()).unwrap_err();
        assert!(e2.contains("nonempty"), "{e2}");
    }

    #[test]
    fn instructions_as_object_render_canonically() {
        let built = built_from(&json!({
            "q": {"type": "noul", "instructions": {"note": "a", "x": 1}}
        }));
        assert_eq!(built.questions[0].prompt, r#"{"note":"a","x":1}"#);
    }

    fn answer_for(built: &SystemOneBuilt, a: katgpt_core::decision_wire::Answer) -> Value {
        let one = DecisionResponse {
            answers: vec![a],
            routing: katgpt_core::decision_wire::Routing {
                lane: katgpt_core::decision_wire::Lane::Modelless,
                reason: None,
            },
            calibration: katgpt_core::decision_wire::Calibration::none(),
        };
        map_answers(built, &one)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
    }

    #[test]
    fn noul_answer_carries_p_yes_and_abstain_reads_half() {
        let built = built_from(&json!({
            "q": {"type": "noul", "instructions": "worth storing?"}
        }));
        let a = answer_for(
            &built,
            katgpt_core::decision_wire::Answer::noul("q", true, 0.87, 0.6),
        );
        assert_eq!(a["type"], "noul");
        assert_eq!(a["noul"], json!(0.87));

        let abstained = katgpt_core::decision_wire::Answer {
            question_id: "q".into(),
            outcome: None,
            probabilities: Vec::new(),
            confidence: 0.1,
        };
        let a2 = answer_for(&built, abstained);
        assert_eq!(a2["noul"], json!(0.5));
    }

    #[test]
    fn criteria_options_noul_maps_the_choice_distribution_to_p() {
        // The criteria-as-options arm: the WIRE question is noul, the ENGINE
        // question is a 2-option Choice — the answer must still carry the
        // noul shape, p = probabilities[0] (the TRUE description's share).
        let built = built_from(&json!({
            "q": {
                "type": "noul",
                "instructions": "worth storing?",
                "criteria": {"true": "A fact.", "false": "Filler."}
            }
        }));
        assert_eq!(built.questions[0].kind, QuestionKind::Choice);
        let a = answer_for(
            &built,
            katgpt_core::decision_wire::Answer::choice(
                "q",
                0,
                vec![0.72, 0.28],
                0.6,
            ),
        );
        assert_eq!(a["type"], "noul");
        assert_eq!(a["noul"], json!(0.72));
        // Abstained reads 0.5 (the uniform 2-option share).
        let abstained = katgpt_core::decision_wire::Answer {
            question_id: "q".into(),
            outcome: None,
            probabilities: Vec::new(),
            confidence: 0.1,
        };
        let a2 = answer_for(&built, abstained);
        assert_eq!(a2["type"], "noul");
        assert_eq!(a2["noul"], json!(0.5));
    }

    #[test]
    fn choice_answer_satisfies_their_validator_laws() {
        let built = built_from(&json!({
            "rep": {
                "type": "choice",
                "instructions": "Which fits?",
                "criteria": {"keep_separate": "a", "merge": "b", "promote": "c", "uncertain": "d"}
            }
        }));
        let a = answer_for(
            &built,
            katgpt_core::decision_wire::Answer::choice(
                "rep",
                2,
                vec![0.1, 0.2, 0.55, 0.15],
                0.7,
            ),
        );
        assert_eq!(a["type"], "choice");
        assert_eq!(a["choice"], "promote");
        let probs = a["probabilities"].as_object().unwrap();
        // Jev-Mem's `_validate` laws, encoded: sum ≈ 1 ...
        let sum: f64 = probs.values().filter_map(|v| v.as_f64()).sum();
        assert!((sum - 1.0).abs() < 1e-3, "sum {sum}");
        // ... and the selected choice is a highest-probability option.
        let picked = probs["promote"].as_f64().unwrap();
        for v in probs.values().filter_map(|v| v.as_f64()) {
            assert!(picked + 1e-6 >= v);
        }
        assert_eq!(a["confidence"], json!(0.7));
    }

    #[test]
    fn choice_abstain_reads_uniform_with_argmax_pick() {
        let built = built_from(&json!({
            "rep": {
                "type": "choice",
                "instructions": "Which?",
                "criteria": {"a": "x", "b": "y"}
            }
        }));
        let abstained = katgpt_core::decision_wire::Answer {
            question_id: "rep".into(),
            outcome: None,
            probabilities: Vec::new(),
            confidence: 0.0,
        };
        let a = answer_for(&built, abstained);
        let probs = a["probabilities"].as_object().unwrap();
        assert_eq!(probs.len(), 2);
        let sum: f64 = probs.values().filter_map(|v| v.as_f64()).sum();
        assert!((sum - 1.0).abs() < 1e-3);
        let picked = probs[a["choice"].as_str().unwrap()].as_f64().unwrap();
        for v in probs.values().filter_map(|v| v.as_f64()) {
            assert!(picked + 1e-6 >= v);
        }
    }

    #[test]
    fn score_answer_keys_by_integer_level() {
        let built = built_from(&json!({
            "urgency": {
                "type": "score",
                "instructions": "How urgent?",
                "criteria": ["low", "mid", "high"]
            }
        }));
        let a = answer_for(
            &built,
            katgpt_core::decision_wire::Answer::score(
                "urgency",
                2,
                vec![0.1, 0.3, 0.6],
                0.5,
            ),
        );
        assert_eq!(a["type"], "score");
        assert_eq!(a["score"], json!(2));
        assert_eq!(a["legend"]["2"], json!("high"));
        assert_eq!(a["probabilities"]["2"], json!(0.6));
    }

    #[test]
    fn respond_round_trips_through_the_demo_engine() {
        let mut engine = crate::serve::demo_engine();
        let body = json!({
            "state": {"observation": "Alice started the Jev-Mem project in Dallas."},
            "model": "jev-latest",
            "questions": {
                "should_store": {
                    "type": "noul",
                    "instructions": "Does `observation` contain a specific detail worth retaining?",
                    "criteria": {"true": "A fact attributable to a participant.", "false": "Only filler."}
                },
                "representation": {
                    "type": "choice",
                    "instructions": "Which representation fits?",
                    "criteria": {"merge": "Compatible.", "keep_separate": "Contradictory."}
                }
            }
        });
        let out = respond(&mut engine, body.to_string().as_bytes()).expect("respond");
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["model"], MODEL_NAME);
        assert_eq!(parsed["usage"], json!({}));
        let noul = &parsed["answers"]["should_store"];
        assert_eq!(noul["type"], "noul");
        let p = noul["noul"].as_f64().unwrap();
        assert!((0.0..=1.0).contains(&p));
        let choice = &parsed["answers"]["representation"];
        assert_eq!(choice["type"], "choice");
        let probs = choice["probabilities"].as_object().unwrap();
        let sum: f64 = probs.values().filter_map(|v| v.as_f64()).sum();
        assert!((sum - 1.0).abs() < 1e-3);
        let picked = probs[choice["choice"].as_str().unwrap()].as_f64().unwrap();
        for v in probs.values().filter_map(|v| v.as_f64()) {
            assert!(picked + 1e-6 >= v);
        }
    }

    #[test]
    fn respond_is_byte_deterministic() {
        let mut engine = crate::serve::demo_engine();
        let body = json!({
            "state": "Alice presented the Jev-Mem results on Friday.",
            "questions": {
                "importance": {
                    "type": "noul",
                    "instructions": "Is this a meaningful life event?",
                    "criteria": {"true": "Yes.", "false": "No."}
                }
            }
        })
        .to_string();
        let a = respond(&mut engine, body.as_bytes()).expect("first");
        let b = respond(&mut engine, body.as_bytes()).expect("second");
        assert_eq!(a, b, "the same body must answer byte-identically");
    }

    #[test]
    fn malformed_bodies_refuse_with_named_errors() {
        let mut engine = crate::serve::demo_engine();
        let no_state = json!({"questions": {}}).to_string();
        assert!(respond(&mut engine, no_state.as_bytes()).is_err());
        let not_json = b"{not json";
        assert!(respond(&mut engine, not_json).is_err());
    }

    #[test]
    fn spaced_state_rendering_separates_structural_scalars() {
        // Issue 081 T2c: the burial law — a compact state with alphabetical
        // keys glues the `depth` digit into the first whitespace token (a
        // per-state-unique chunk); the spaced rendering must isolate `0,`
        // as a standalone token so the count tables can see it.
        let state = json!({"depth": 0, "evidence": [{"content": "[Audrey]: hello"}]});
        let spaced = super::render_state_spaced(&state);
        assert_eq!(
            spaced,
            "{\"depth\": 0, \"evidence\": [{\"content\": \"[Audrey]: hello\"}]}"
        );
        // The token stream: the digit is its own token (trimmed `0`), not
        // glued into the first chunk.
        let mut toks = Vec::new();
        crate::embed::hashed_tokens_into(
            spaced.as_bytes(),
            crate::nb_scope::NB_VOCAB,
            &mut toks,
        );
        let mut probe = Vec::new();
        crate::embed::hashed_tokens_into(
            b"0",
            crate::nb_scope::NB_VOCAB,
            &mut probe,
        );
        assert!(toks.contains(&probe[0]), "the depth digit must be a standalone token");
    }
}
