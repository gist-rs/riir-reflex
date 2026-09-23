//! The checkpoint router (port of the pure half of `laya/router.py`).
//!
//! Decision precedence, the reference's exact order: explicit model >
//! explicit task > detected typed-decisions workflow (OPT-IN) > explicit
//! lang > detected script/language > default.
//!
//! Why routing exists (the reference's own measured table): the English
//! checkpoint does not degrade gracefully off English — it collapses
//! (Hindi 0.100 / Korean 0.103 at 20 options vs 0.050 random, ECE 0.855
//! on Hindi) — so script detection is the primary signal.
//! `typed-decisions` is NEVER auto-selected unless opted in: it is
//! fine-tuned on four synthetic workflows and must not be a silent
//! default.
//!
//! Deviation from the reference, structural: the Python `Router` also
//! owns checkpoint LOADING (LRU cache, preload, eviction — ~1.16B params
//! do not fit the same way here). In this lane the caller holds loaded
//! [`Agent`]s and hands the chosen one the routed request; the router
//! itself stays pure decision logic, which is what the plan's "pure
//! logic" scope names.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::config::Checkpoint;
use super::lang;

/// The hub repo (all three checkpoints; the pin's subfolder packaging).
pub const BUNDLE_REPO: &str = super::weights::HF_REPO;

/// The typed-decisions workflow question-id signatures. AUTO task
/// detection requires an EXACT id-set match — an unrelated schema that
/// happens to contain `urgency` is never captured.
const TYPED_DECISION_WORKFLOWS: &[(&str, &[&str])] = &[
    (
        "agent_trace_observability",
        &["action", "needs_review", "outcome", "risk", "urgency"],
    ),
    (
        "customer_service",
        &["action", "category", "churn_risk", "needs_human", "urgency"],
    ),
    (
        "invoice_processing",
        &[
            "discrepancy_severity",
            "disposition",
            "duplicate",
            "matches_order",
            "urgency",
        ],
    ),
    (
        "security_incidents",
        &[
            "credential_compromise",
            "disposition",
            "severity",
            "true_positive",
            "urgency",
        ],
    ),
];

/// Aliases people are likely to type.
fn normalise_name(name: &str) -> Result<Checkpoint, String> {
    let key = name.trim().to_lowercase();
    let key = match key.as_str() {
        "en" | "laya" | "default" => "english",
        "multi" | "ml" | "laya-multilingual" => "multilingual",
        "typed" | "typed_decisions" | "laya-typed-decisions" | "decisions" => "typed",
        k => k,
    };
    match key {
        "english" => Ok(Checkpoint::English),
        "multilingual" => Ok(Checkpoint::Multilingual),
        "typed" | "typed-decisions" => Ok(Checkpoint::TypedDecisions),
        other => Err(format!(
            "unknown model {other:?}; choose one of english/multilingual/typed-decisions (or an alias)"
        )),
    }
}

/// Name of the typed-decisions workflow whose question ids these are,
/// else `None` (exact id-set match, both directions).
pub fn match_typed_decisions_workflow(question_ids: &[String]) -> Option<&'static str> {
    let ids: BTreeSet<&String> = question_ids.iter().collect();
    for (wf, sig) in TYPED_DECISION_WORKFLOWS {
        let sig_set: BTreeSet<&str> = sig.iter().copied().collect();
        if ids.len() == sig_set.len() && ids.iter().all(|i| sig_set.contains(i.as_str())) {
            return Some(wf);
        }
    }
    None
}

/// The routing outcome: which checkpoint, why, and what was detected.
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// The checkpoint to serve with.
    pub model: Checkpoint,
    /// Human-readable hub id (`repo` or `repo/subfolder`).
    pub repo: String,
    /// Why this checkpoint — recorded into the routing envelope.
    pub reason: String,
    /// The full detection payload (`None` for explicit overrides).
    pub detection: Option<lang::Detection>,
    /// The matched typed-decisions workflow, if any.
    pub workflow: Option<&'static str>,
}

/// Explicit override inputs (the reference's `model`/`task`/`lang`
/// keyword arguments).
#[derive(Debug, Default, Clone)]
pub struct RouteOverrides {
    pub model: Option<String>,
    pub task: Option<String>,
    /// Auto task detection is OPT-IN (the reference's
    /// `auto_task_detection`).
    pub auto_task_detection: bool,
    pub lang: Option<String>,
}

/// Decide which checkpoint to use — no loading, no forward. `questions`
/// are the question ids only (the workflow signature is the id SET).
pub fn route(
    state: &Value,
    question_ids: &[String],
    ov: &RouteOverrides,
) -> Result<RouteDecision, String> {
    if let Some(model) = &ov.model {
        let key = normalise_name(model)?;
        return Ok(RouteDecision {
            model: key,
            repo: repo_str(key),
            reason: format!("explicit model={model:?}"),
            detection: None,
            workflow: None,
        });
    }
    if let Some(task) = &ov.task {
        let key = if task.to_lowercase().replace(['-', '_'], "") == "typeddecisions" {
            Checkpoint::TypedDecisions
        } else {
            normalise_name(task)?
        };
        return Ok(RouteDecision {
            model: key,
            repo: repo_str(key),
            reason: format!("explicit task={task:?}"),
            detection: None,
            workflow: None,
        });
    }

    let workflow = match_typed_decisions_workflow(question_ids);
    if let (Some(wf), true) = (workflow, ov.auto_task_detection) {
        return Ok(RouteDecision {
            model: Checkpoint::TypedDecisions,
            repo: repo_str(Checkpoint::TypedDecisions),
            reason: format!("question ids match the {wf:?} typed-decisions workflow"),
            detection: None,
            workflow: Some(wf),
        });
    }

    if let Some(l) = &ov.lang {
        let base = l.to_lowercase().split('-').next().unwrap_or("").to_string();
        let key = if base == "en" || base == "eng" || base == "english" {
            Checkpoint::English
        } else {
            Checkpoint::Multilingual
        };
        return Ok(RouteDecision {
            model: key,
            repo: repo_str(key),
            reason: format!("explicit lang={l:?}"),
            detection: None,
            workflow,
        });
    }

    let det = lang::analyse(state);
    let decision = if det.script == "unknown" {
        RouteDecision {
            model: Checkpoint::English,
            repo: repo_str(Checkpoint::English),
            reason: "no letters detected in state; using default (english)".into(),
            detection: Some(det),
            workflow,
        }
    } else if det.script != "latin" {
        let pct = (det.non_latin_fraction * 100.0) as i64;
        RouteDecision {
            model: Checkpoint::Multilingual,
            repo: repo_str(Checkpoint::Multilingual),
            reason: format!(
                "non-Latin script ({}, {pct}% of letters); the English checkpoint cannot read it",
                det.script
            ),
            detection: Some(det),
            workflow,
        }
    } else if !det.is_english {
        let reason = if let Some(lg) = &det.language {
            format!("Latin script but language looks like {lg:?}, not English")
        } else {
            let pct = (det.diacritic_rate * 100.0) as i64;
            format!(
                "Latin script, language not identified but {pct}% non-English letters; \
                 not safe for the English checkpoint"
            )
        };
        RouteDecision {
            model: Checkpoint::Multilingual,
            repo: repo_str(Checkpoint::Multilingual),
            reason,
            detection: Some(det),
            workflow,
        }
    } else {
        RouteDecision {
            model: Checkpoint::English,
            repo: repo_str(Checkpoint::English),
            reason: "English Latin text".into(),
            detection: Some(det),
            workflow,
        }
    };
    Ok(decision)
}

fn repo_str(ckpt: Checkpoint) -> String {
    match ckpt {
        Checkpoint::English => BUNDLE_REPO.to_string(),
        other => format!("{BUNDLE_REPO}/{}", other.subfolder()),
    }
}

/// Convenience: the routing half of `predict` — route, then hand the
/// chosen checkpoint to the caller's loaded agent (the Rust `Router`
/// owns no checkpoints; the harness holds them).
pub fn route_json(state: &Value, questions: &Value, ov: &RouteOverrides) -> Result<Value, String> {
    let ids: Vec<String> = questions
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let d = route(state, &ids, ov)?;
    let det = d.detection.as_ref().map(|det| {
        json!({
            "script": det.script,
            "script_profile": det.script_profile,
            "language": det.language,
            "is_english": det.is_english,
            "language_undecided": det.language_undecided,
            "diacritic_rate": det.diacritic_rate,
            "non_latin_fraction": det.non_latin_fraction,
        })
    });
    Ok(json!({
        "model": d.model.fixture_name(),
        "repo": d.repo,
        "reason": d.reason,
        "detection": det,
        "workflow": d.workflow,
    }))
}

/// Routing-decision pins mirrored from the reference's
/// `tests/test_router.py` (pure logic, no weights).
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ids(v: Value) -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn workflow_signatures_match_exactly() {
        for (wf, sig) in TYPED_DECISION_WORKFLOWS {
            let v: Vec<Value> = sig.iter().map(|s| json!(s)).collect();
            assert_eq!(
                match_typed_decisions_workflow(&ids(Value::Array(v))),
                Some(*wf)
            );
        }
        assert_eq!(
            match_typed_decisions_workflow(&ids(json!(["urgency", "category"]))),
            None,
            "partial overlap"
        );
        assert_eq!(
            match_typed_decisions_workflow(&ids(json!([
                "action",
                "category",
                "churn_risk",
                "needs_human",
                "urgency",
                "extra"
            ]))),
            None,
            "superset"
        );
        assert_eq!(match_typed_decisions_workflow(&[]), None, "empty");
    }

    #[test]
    fn name_normalisation() {
        for (alias, want) in [
            ("en", Checkpoint::English),
            ("laya", Checkpoint::English),
            ("default", Checkpoint::English),
            ("multi", Checkpoint::Multilingual),
            ("ML", Checkpoint::Multilingual),
            ("typed", Checkpoint::TypedDecisions),
            ("typed_decisions", Checkpoint::TypedDecisions),
            ("English", Checkpoint::English),
        ] {
            assert_eq!(normalise_name(alias).unwrap(), want, "alias/{alias}");
        }
        assert!(normalise_name("nope").is_err(), "unknown raises");
    }

    fn q_generic_ids() -> Vec<String> {
        vec!["dept".into()]
    }
    fn q_td_ids() -> Vec<String> {
        ids(json!([
            "action",
            "category",
            "churn_risk",
            "needs_human",
            "urgency"
        ]))
    }

    #[test]
    fn routing_decisions_match_reference_suite() {
        let cases: &[(&str, Value, &str)] = &[
            (
                "english text",
                json!({"body": "I was charged twice, please refund."}),
                "english",
            ),
            ("armenian text", json!({"body": "Հայերեն"}), "multilingual"),
            (
                "hindi text",
                json!({"body": "मुझसे दो बार शुल्क लिया गया"}),
                "multilingual",
            ),
            (
                "japanese text",
                json!({"body": "二重に請求されました"}),
                "multilingual",
            ),
            (
                "korean text",
                json!({"body": "두 번 청구되었습니다"}),
                "multilingual",
            ),
            (
                "arabic text",
                json!({"body": "تم خصم المبلغ مرتين"}),
                "multilingual",
            ),
            (
                "german text",
                json!({"body": "Der Kunde wurde zweimal belastet und moechte eine Rueckerstattung fuer die Rechnung die nicht korrekt ist"}),
                "multilingual",
            ),
            ("empty state", json!({}), "english"),
            ("none state", Value::Null, "english"),
        ];
        for (label, state, want) in cases {
            let d = route(state, &q_generic_ids(), &RouteOverrides::default()).unwrap();
            assert_eq!(d.model.fixture_name(), *want, "route/{label}");
        }
    }

    #[test]
    fn explicit_overrides_beat_detection() {
        let armenian = json!({"body": "Հայերեն"});
        let d = route(
            &armenian,
            &q_generic_ids(),
            &RouteOverrides {
                model: Some("english".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            d.model,
            Checkpoint::English,
            "explicit model overrides script"
        );
        assert_eq!(
            d.repo, "convaiinnovations/laya",
            "english repo is the bundle root"
        );

        let d = route(
            &json!({"body": "मुझसे दो बार"}),
            &q_generic_ids(),
            &RouteOverrides {
                model: Some("english".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(d.model, Checkpoint::English);

        let d = route(
            &json!({"body": "x"}),
            &q_generic_ids(),
            &RouteOverrides {
                task: Some("typed_decisions".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(d.model, Checkpoint::TypedDecisions, "explicit task");

        let d = route(
            &json!({"body": "मुझसे दो बार"}),
            &q_generic_ids(),
            &RouteOverrides {
                lang: Some("en".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(d.model, Checkpoint::English, "explicit lang en");

        let d = route(
            &json!({"body": "hello there"}),
            &q_generic_ids(),
            &RouteOverrides {
                lang: Some("de".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(d.model, Checkpoint::Multilingual, "explicit lang de");
    }

    #[test]
    fn auto_task_detection_is_opt_in() {
        let charged = json!({"body": "I was charged twice"});
        let d = route(&charged, &q_td_ids(), &RouteOverrides::default()).unwrap();
        assert_eq!(d.model, Checkpoint::English, "td workflow, auto OFF");

        let d = route(
            &charged,
            &q_td_ids(),
            &RouteOverrides {
                auto_task_detection: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(d.model, Checkpoint::TypedDecisions, "td workflow, auto ON");
        assert_eq!(d.workflow, Some("customer_service"));

        let d = route(
            &charged,
            &q_generic_ids(),
            &RouteOverrides {
                auto_task_detection: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            d.model,
            Checkpoint::English,
            "auto ON but generic questions"
        );

        let d = route(
            &json!({"body": "x"}),
            &q_td_ids(),
            &RouteOverrides {
                auto_task_detection: true,
                model: Some("multilingual".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(d.model, Checkpoint::Multilingual, "explicit beats workflow");
    }

    #[test]
    fn decision_payload_shape() {
        let d = route(
            &json!({"body": "मुझसे दो बार शुल्क लिया गया"}),
            &q_generic_ids(),
            &RouteOverrides::default(),
        )
        .unwrap();
        assert_eq!(d.repo, "convaiinnovations/laya/multilingual");
        let j = route_json(
            &json!({"body": "I was charged twice"}),
            &json!({"dept": {}}),
            &RouteOverrides::default(),
        )
        .unwrap();
        assert_eq!(j["model"], "english");
        assert!(!j["reason"].as_str().unwrap().is_empty());
    }
}
