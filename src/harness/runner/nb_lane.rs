//! Issue 038 — the harness half of the count-table lever (`nb_scope`):
//! the cal-side (scale × α) selection, and the SEPARATE transductive
//! column.
//!
//! PROTOCOL (the no-cheat law, issue 038 §Protocol rule): the headline
//! `acc` is the honest read — corpora and count tables from TRAIN rows
//! only, the posture selected on the stratified selection slice (never
//! test), test read once. The transductive column answers the owner's
//! "full vocab + grammar" concern without touching gold: it may read the
//! TEST TEXT (never its labels), pseudo-labelled by the honest engine, and
//! it is published beside `acc`, never in it.

use super::*;
#[cfg(feature = "nb_scope")]
use crate::nb_scope::{NbAlpha, NbView};

/// One (scale, α) candidate's selection-slice accuracy.
#[derive(Debug, Clone, Serialize)]
pub struct NbCandidate {
    pub scale: f32,
    pub alpha: &'static str,
    /// Noul polarity candidate (`Some(d)`: "yes" reads as domain `d`);
    /// `None` on choice/score suites.
    pub noul_domain: Option<usize>,
    /// Event view (`bag` / `pair`).
    pub view: &'static str,
    pub cal_acc: f64,
}

/// The cal-selected count-table posture for a suite: accuracy per
/// candidate on the stratified selection slice, argmax there ONLY under
/// the heads' promotion bar (must clear scale 0 by `HEAD_SELECT_MARGIN`;
/// nothing clears → off, the byte-identical baseline). Test read once.
#[derive(Debug, Clone, Serialize)]
pub struct NbSelection {
    pub selected_scale: f32,
    pub selected_alpha: &'static str,
    /// The selected noul polarity (noul suites only).
    pub selected_noul_domain: Option<usize>,
    /// The selected event view (`bag` / `pair`).
    pub selected_view: &'static str,
    pub candidates: Vec<NbCandidate>,
}

/// The transductive column (issue 038): a DIFFERENT protocol, so a
/// different number — never folded into `acc`.
#[derive(Debug, Clone, Serialize)]
pub struct TransductiveReport {
    /// Forced accuracy under the transductive protocol.
    pub accuracy: f64,
    /// The honest headline accuracy of the same suite row (for the delta).
    pub honest_accuracy: f64,
    /// Pseudo-labelled test docs that joined the count tables.
    pub n_pseudo: usize,
    /// The protocol, verbatim, on every row.
    pub protocol: &'static str,
}

/// The transductive protocol text (rides every row + the run meta).
pub const TRANSDUCTIVE_PROTOCOL: &str = "TRANSDUCTIVE — unlabeled TEST text joins the count \
     tables, labelled by the honest engine's own forced picks (gold never read); 2-fold \
     cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores \
     against its own text); drafter corpora, route, heads and posture unchanged. Not \
     comparable to the headline acc";

#[cfg(feature = "nb_scope")]
const NB_SCALE_LADDER: [f32; 3] = [1.0, 4.0, 16.0];
#[cfg(feature = "nb_scope")]
const NB_ALPHA_LADDER: [(NbAlpha, &str); 2] = [
    (NbAlpha::ObservedLaplace, "observed-laplace"),
    (NbAlpha::Fixed(1.0), "fixed-1"),
];

/// Cal-side selection of the count-table posture at the already-selected
/// cap and head scale. The ladder spans the blend regime: 1 ≈ one more
/// σ-term beside drafter/route/head, 16 ≈ the tables decide alone.
#[cfg(feature = "nb_scope")]
pub(super) fn build_nb_selection<const N: usize>(
    inp: &ModellessInput<'_>,
    effective_cap: usize,
    head_scale: f32,
) -> Result<NbSelection, String> {
    let spec = inp.spec;
    let SelSlice {
        cases,
        state_strs,
        pool,
    } = selection_slice(inp, "nb-select")?;
    // A noul suite (every selection question is noul) needs a polarity:
    // each domain is a candidate for "yes", scored on the LABELLED slice.
    let noul_suite = !cases.is_empty()
        && cases
            .iter()
            .all(|c| c.questions.iter().all(|q| matches!(q.kind, QKind::Noul)));
    let polarities: Vec<Option<usize>> = if noul_suite {
        (0..N).map(Some).collect()
    } else {
        vec![None]
    };
    // A sentence-pair suite (states carry ≥ 2 string fields) also tries the
    // pair view (issue 038 T3).
    let pair_suite = cases.first().is_some_and(|c| {
        c.state
            .as_object()
            .is_some_and(|m| m.values().filter(|v| v.is_string()).count() >= 2)
    });
    let views: &[(NbView, &str)] = if pair_suite {
        &[(NbView::Bag, "bag"), (NbView::Pair, "pair")]
    } else {
        &[(NbView::Bag, "bag")]
    };
    let eval_at =
        |scale: f32, alpha: NbAlpha, noul: Option<usize>, view: NbView| -> Result<f64, String> {
            let mut engine = build_engine::<N>(
                spec.name,
                &pool,
                inp.labels,
                effective_cap,
                EngineConfig {
                    head_scale,
                    nb_scale: scale,
                    nb_alpha: alpha,
                    nb_noul_domain: noul,
                    nb_view: view,
                    // Forced: never abstain (conf ≤ 1 < threshold).
                    score_threshold: 2.0,
                    distance_threshold: 2.0,
                    ..EngineConfig::default()
                },
            )?;
            let (ev, _) = eval_engine(&mut engine, &cases, &state_strs, false)?;
            Ok(hard_metrics(&ev.forced_rows(&cases)).accuracy)
        };
    let base = eval_at(0.0, NbAlpha::ObservedLaplace, None, NbView::Bag)?;
    eprintln!("    nb-select: scale 0 (off) → sel-slice acc {base:.4}");
    let mut rows = vec![NbCandidate {
        scale: 0.0,
        alpha: "off",
        noul_domain: None,
        view: "bag",
        cal_acc: base,
    }];
    let (mut selected_scale, mut selected_alpha, mut selected_noul, mut selected_view, mut best) =
        (0.0f32, "off", None, "bag", base);
    for &(view, view_name) in views {
        for &noul in &polarities {
            for &(alpha, name) in &NB_ALPHA_LADDER {
                for &scale in &NB_SCALE_LADDER {
                    let cal_acc = eval_at(scale, alpha, noul, view)?;
                    eprintln!(
                        "    nb-select: scale {scale} α {name} noul-yes {noul:?} view {view_name} → \
                     sel-slice acc {cal_acc:.4}"
                    );
                    if cal_acc >= base + HEAD_SELECT_MARGIN && cal_acc > best {
                        best = cal_acc;
                        selected_scale = scale;
                        selected_alpha = name;
                        selected_noul = noul;
                        selected_view = view_name;
                    }
                    rows.push(NbCandidate {
                        scale,
                        alpha: name,
                        noul_domain: noul,
                        view: view_name,
                        cal_acc,
                    });
                }
            }
        }
    }
    eprintln!(
        "    nb-select: selected scale {selected_scale} α {selected_alpha} noul-yes \
         {selected_noul:?} — the suite's row below is the single test read, at this posture"
    );
    Ok(NbSelection {
        selected_scale,
        selected_alpha,
        selected_noul_domain: selected_noul,
        selected_view,
        candidates: rows,
    })
}

/// The α a selected label maps back to.
#[cfg(feature = "nb_scope")]
pub(super) fn alpha_of(name: &str) -> NbAlpha {
    NB_ALPHA_LADDER
        .iter()
        .find(|(_, n)| *n == name)
        .map_or(NbAlpha::ObservedLaplace, |(a, _)| *a)
}

/// The view a selected label maps back to.
#[cfg(feature = "nb_scope")]
pub(super) fn view_of(name: &str) -> NbView {
    if name == "pair" {
        NbView::Pair
    } else {
        NbView::Bag
    }
}

/// Raw text of a case state: every string leaf, newline-joined (the
/// train-doc shape — no JSON keys, which would plant a constant token in
/// one label's table).
#[cfg(feature = "nb_scope")]
fn state_text(v: &Value, out: &mut String) {
    match v {
        Value::String(s) => {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(s);
        }
        Value::Array(a) => a.iter().for_each(|x| state_text(x, out)),
        Value::Object(m) => m.values().for_each(|x| state_text(x, out)),
        _ => {}
    }
}

/// Pseudo-docs from the honest engine's forced picks over `range`: one
/// doc per choice question whose pick resolves to a label by the ENGINE's
/// own rule — the option key by name first (Issue 023), else the legacy
/// index alignment when the option count equals the label count (the
/// ag_news/emotion display-name shape). Reads no gold.
#[cfg(feature = "nb_scope")]
fn pseudo_docs(
    cases: &[SuiteCase],
    eval: &Eval,
    labels: &[String],
    range: std::ops::Range<usize>,
) -> Vec<TrainDoc> {
    let mut out = Vec::new();
    for ci in range {
        let case = &cases[ci];
        for (qi, q) in case.questions.iter().enumerate() {
            if !matches!(q.kind, QKind::Choice) {
                continue;
            }
            let Some(m) = q.criteria.as_object() else {
                continue;
            };
            let pick = eval.picks[ci][qi];
            let by_name = m.keys().all(|k| labels.iter().any(|l| l == k));
            let label = if by_name {
                m.keys().nth(pick).cloned()
            } else if m.len() == labels.len() {
                labels.get(pick).cloned()
            } else {
                None
            };
            if let Some(label) = label {
                let mut text = String::new();
                state_text(&case.state, &mut text);
                out.push(TrainDoc { label, text });
            }
        }
    }
    out
}

/// The transductive column (see [`TRANSDUCTIVE_PROTOCOL`]). `None` when
/// the count tables are off (nothing for the test text to join).
#[allow(clippy::too_many_arguments)]
pub(super) fn transductive_pass<const N: usize>(
    inp: &ModellessInput<'_>,
    honest: &Eval,
    honest_accuracy: f64,
    cfg: &EngineConfig,
    corpus_pool: &[TrainDoc],
    effective_cap: usize,
) -> Result<Option<TransductiveReport>, String> {
    #[cfg(not(feature = "nb_scope"))]
    {
        let _ = (
            inp,
            honest,
            honest_accuracy,
            cfg,
            corpus_pool,
            effective_cap,
        );
        Ok(None)
    }
    #[cfg(feature = "nb_scope")]
    {
        if cfg.nb_scale <= 0.0 {
            return Ok(None);
        }
        let cases = &inp.suite.cases;
        let n = cases.len();
        if n < 2 {
            return Ok(None);
        }
        let half = n / 2;
        let mut rows = Vec::with_capacity(n);
        let mut n_pseudo = 0usize;
        for (score, source) in [(half..n, 0..half), (0..half, half..n)] {
            let extra = pseudo_docs(cases, honest, inp.labels, source);
            n_pseudo += extra.len();
            let mut engine = build_engine_with::<N>(
                inp.spec.name,
                corpus_pool,
                inp.labels,
                effective_cap,
                cfg.clone(),
                &extra,
            )?;
            let (ev, _) = eval_engine(
                &mut engine,
                &cases[score.clone()],
                &inp.state_strs[score.clone()],
                false,
            )?;
            rows.extend(ev.forced_rows(&cases[score]));
        }
        Ok(Some(TransductiveReport {
            accuracy: hard_metrics(&rows).accuracy,
            honest_accuracy,
            n_pseudo,
            protocol: TRANSDUCTIVE_PROTOCOL,
        }))
    }
}
