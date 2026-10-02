//! The harness families module — scaffolding + Issue 061's
//! `semantic_defects` code-defect family (its own module).
//!
//! **RETIREMENT (owner call, 2026-10-02):** the six Issue-004
//! decision-point families (`harness_visibility`, `harness_permissions`,
//! `harness_tool_fit`, `harness_routing`, `harness_sensitivity`,
//! `harness_cache_reuse`) are REMOVED — home-made synthetic evals the
//! modelless engine reads at chance on (wide-eval populations, 0.22–0.31
//! vs ~0.2–0.33 chance; the small-n template-shared reads that looked
//! strong were the artifact). Nobody benches them anymore: the registry
//! rows, the wide evals, the gates, and the GOAT bench are gone with
//! them. Final readings live in git history + the instinct/reflex-site
//! records. `semantic_defects` stays: it is authored the same way but
//! reads above chance and its wide eval discriminates.
//!
//! Pure data + builders (ungated), the `code_fixtures` shape: in-process
//! synthetic fixtures with programmatic gold, self-split into three
//! PAIRWISE-DISJOINT slices — corpus (the routing/reference texts), cal
//! (the calibration slice, ≥16 so the fused-gate thresholds fit), eval —
//! because a cal case scoring cos 1.0 against ITSELF inflates every
//! cal-slice quantile (the measured self-inclusion leak, runner docs).
//!
//! Fixture-design law (the degenerate-lane lesson, 2026-09-22): states
//! are multi-sentence and class-distinctive by VOCABULARY — and they
//! deliberately do NOT contain the gold label token (no "— hide.", ": allow.",
//! "level 4"), which would make any future accuracy claim a label-leak
//! claim. The corpus docs DO carry the label words: they are what teaches
//! the corpus-conditioned drafter which option string fits.
//!
//! No engine imports here; the runner consumes [`SynthData`].

use serde_json::{Map, Value};

use crate::harness::suites::{GoldAnswer, QKind, Suite, SuiteCase, SuiteQuestion, TrainDoc};

/// One in-process suite, prepared for the runner: the eval suite, its
/// calibration slice, the per-label corpus docs, and the domain labels
/// (domain order = label order = option-key order for Choice).
#[derive(Debug, Clone)]
pub struct SynthData {
    pub suite: Suite,
    pub cal_cases: Vec<SuiteCase>,
    pub docs: Vec<TrainDoc>,
    pub labels: Vec<String>,
}

/// One authored fixture: the state text and its programmatic gold index
/// (the option/level index, in `labels` order).
#[derive(Debug)]
pub struct FamilyText {
    pub text: &'static str,
    pub gold: usize,
}

/// One family definition: question shape, option universe, and the three
/// authored slices.
pub struct FamilyDef {
    pub name: &'static str,
    pub qid: &'static str,
    pub kind: QKind,
    /// Domain labels = option keys (Choice) = level indices (Score).
    pub labels: &'static [&'static str],
    pub instructions: &'static str,
    /// Score-level display strings (Score families; empty for Choice).
    pub score_levels: &'static [&'static str],
    pub note: &'static str,
    /// (gold index, text) — the routing/reference corpus.
    pub corpus: &'static [(usize, &'static str)],
    pub cal: &'static [FamilyText],
    pub eval: &'static [FamilyText],
}

#[must_use]
pub fn synth_semantic_defects() -> SynthData {
    build_family(&super::families_semantic_defects::DEFECTS)
}

// ── the shared builder ─────────────────────────────────────────────────────

fn build_family(def: &'static FamilyDef) -> SynthData {
    let criteria: Value = match def.kind {
        QKind::Choice => {
            let mut m = Map::new();
            for l in def.labels {
                m.insert((*l).to_string(), Value::Null);
            }
            Value::Object(m)
        }
        QKind::Score => Value::Array(
            def.score_levels
                .iter()
                .map(|s| Value::String((*s).to_string()))
                .collect(),
        ),
        QKind::Noul => Value::Null,
    };
    let case = |slice: &str, i: usize, t: &FamilyText| SuiteCase {
        id: format!("{}:{slice}:{i}", def.name),
        state: Value::String(t.text.to_string()),
        questions: vec![SuiteQuestion {
            qid: def.qid.to_string(),
            kind: def.kind,
            instructions: def.instructions.to_string(),
            criteria: criteria.clone(),
        }],
        gold: vec![GoldAnswer {
            idx: t.gold,
            soft: vec![0.0; def.labels.len()],
            gold_score: if def.kind == QKind::Score {
                Some(t.gold as f64)
            } else {
                None
            },
        }],
    };
    let cal_cases: Vec<SuiteCase> = def
        .cal
        .iter()
        .enumerate()
        .map(|(i, t)| case("cal", i, t))
        .collect();
    let cases: Vec<SuiteCase> = def
        .eval
        .iter()
        .enumerate()
        .map(|(i, t)| case("eval", i, t))
        .collect();
    let docs: Vec<TrainDoc> = def
        .corpus
        .iter()
        .map(|(g, text)| TrainDoc {
            label: def.labels[*g].to_string(),
            text: (*text).to_string(),
        })
        .collect();
    SynthData {
        suite: Suite {
            name: def.name,
            cases,
            option_counts_note: def.note,
        },
        cal_cases,
        docs,
        labels: def.labels.iter().map(|s| (*s).to_string()).collect(),
    }
}

/// The remaining modelless family definition (registry order = suite order).
pub const FAMILY_DEFS: &[&FamilyDef] = &[&super::families_semantic_defects::DEFECTS];

/// Lookup by suite name.
#[must_use]
pub fn family_def(name: &str) -> Option<&'static FamilyDef> {
    FAMILY_DEFS.iter().copied().find(|d| d.name == name)
}

/// Build a family's [`SynthData`] by suite name.
#[must_use]
pub fn synth_by_name(name: &str) -> Option<SynthData> {
    family_def(name).map(build_family)
}
