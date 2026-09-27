//! Issue 038 T7b — the harness half of the option-conditioned lever
//! (`option_cond`): the cal-side scale selection for the per-(qid, option)
//! count tables.
//!
//! PROTOCOL (the same no-cheat law as [`super::nb_lane`]): events come from
//! TRAIN rows only, filtered to the corpus pool's texts (the selection
//! cases never score against their own text), the scale selected on the
//! stratified selection slice under the heads' promotion bar, test read
//! once. The premise was probed before any Rust
//! (`scripts/issue038_t7_probe.py`: per-(qid, option) counts 0.4475 vs the
//! published 0.32 on typed_decisions).

use super::*;
#[cfg(feature = "option_cond")]
use crate::option_cond::OcEvent;

/// One scale candidate's selection-slice accuracy.
#[derive(Debug, Clone, Serialize)]
pub struct OcCandidate {
    pub scale: f32,
    pub cal_acc: f64,
}

/// The cal-selected option-conditioned posture for a suite: accuracy per
/// candidate on the stratified selection slice, argmax there ONLY under
/// the heads' promotion bar (must clear scale 0 by [`HEAD_SELECT_MARGIN`];
/// nothing clears → off, the byte-identical baseline). Test read once.
#[derive(Debug, Clone, Serialize)]
pub struct OcSelection {
    pub selected_scale: f32,
    pub candidates: Vec<OcCandidate>,
}

/// The blend ladder: 0 = off (the baseline arm), then the same
/// sigmoid-term regime the nb ladder spans — 0.25 (a nudge beside the
/// drafter term) to 2.0 (the tables dominate).
#[cfg(feature = "option_cond")]
pub(super) const OC_SCALE_LADDER: [f32; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0];

/// Cal-side selection of the option-conditioned scale at the
/// already-selected cap, head scale and nb posture. `events_for_pool`
/// filters the suite's gold events to the corpus pool's texts (the
/// selection slice must never score against its own text — the shared
/// selection-slice law).
#[cfg(feature = "option_cond")]
pub(super) fn build_oc_selection<const N: usize>(
    inp: &ModellessInput<'_>,
    effective_cap: usize,
    _head_scale: f32,
    base_cfg: &EngineConfig,
    events_for_pool: impl Fn(&[TrainDoc]) -> Vec<OcEvent>,
) -> Result<OcSelection, String> {
    let spec = inp.spec;
    let SelSlice {
        cases,
        state_strs,
        pool,
    } = selection_slice(inp, "oc-select")?;
    let events = events_for_pool(&pool);
    if events.is_empty() {
        return Err(format!(
            "{}: oc-select produced no fit events for the selection pool — the suite \
             carries no per-question gold to condition on",
            spec.name
        ));
    }
    let eval_at = |scale: f32| -> Result<f64, String> {
        let cfg = EngineConfig {
            oc_scale: scale,
            ..base_cfg.clone()
        };
        let (mut engine, _) = build_engine_oc_with::<N>(
            spec.name,
            &pool,
            inp.labels,
            effective_cap,
            cfg,
            &[],
            &events,
        )?;
        let (ev, _) = eval_engine(&mut engine, &cases, &state_strs, false)?;
        Ok(hard_metrics(&ev.forced_rows(&cases)).accuracy)
    };
    let base = eval_at(0.0)?;
    eprintln!("    oc-select: scale 0 (off) → sel-slice acc {base:.4}");
    let mut rows = vec![OcCandidate { scale: 0.0, cal_acc: base }];
    let (mut selected, mut best) = (0.0f32, base);
    for &scale in &OC_SCALE_LADDER {
        let cal_acc = eval_at(scale)?;
        eprintln!("    oc-select: scale {scale} → sel-slice acc {cal_acc:.4}");
        if cal_acc >= base + HEAD_SELECT_MARGIN && cal_acc > best {
            best = cal_acc;
            selected = scale;
        }
        rows.push(OcCandidate { scale, cal_acc });
    }
    eprintln!(
        "    oc-select: selected {selected} — the suite's row below is the single \
         test read, at this posture"
    );
    Ok(OcSelection {
        selected_scale: selected,
        candidates: rows,
    })
}
