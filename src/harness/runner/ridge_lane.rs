//! Issue 038 T7a — the harness half of the NBSVM ridge lever
//! (`nb_ridge`): the cal-side scale selection.
//!
//! PROTOCOL (the shared no-cheat law): corpora from TRAIN rows only, the
//! scale selected on the stratified selection slice under the heads'
//! promotion bar, test read once. The premise was probed before any Rust
//! (`scripts/issue038_t7_probe_k.py`: emotion 0.8925 at k=2048
//! per-class λ=10 vs the published 0.7375). λ is FIXED at the probe's
//! selected 10.0 and the fit is O(k³) per class, so the ladder is
//! deliberately SHORT (3 scale candidates + off) — each candidate is a
//! full engine build including the fit.

use super::*;

/// One scale candidate's selection-slice accuracy.
#[derive(Debug, Clone, Serialize)]
pub struct RidgeCandidate {
    pub scale: f32,
    pub lambda: f32,
    pub cal_acc: f64,
}

/// The cal-selected ridge posture for a suite: accuracy per candidate on
/// the stratified selection slice, argmax there ONLY under the heads'
/// promotion bar (nothing clears → off, the byte-identical baseline).
#[derive(Debug, Clone, Serialize)]
pub struct RidgeSelection {
    pub selected_scale: f32,
    pub selected_lambda: f32,
    pub candidates: Vec<RidgeCandidate>,
}

/// The blend ladder (short on purpose — every candidate pays a full
/// O(k³)-per-class fit). 2.0 lets the discriminative rows dominate the
/// σ-drafter term; the probe's pure-argmax read bounds the regime.
#[cfg(feature = "nb_ridge")]
pub(super) const RIDGE_SCALE_LADDER: [f32; 3] = [0.5, 1.0, 2.0];

/// The fixed λ (the probe's selection; changing it is a NEW probe first —
/// a λ ladder would multiply the fit cost by its length).
#[cfg(feature = "nb_ridge")]
pub(super) const RIDGE_LAMBDA: f32 = 10.0;

/// Cal-side selection of the ridge scale at the already-selected cap +
/// head + nb + oc posture. `events_for_pool` supplies the oc events for a
/// pool (typed arms oc BEFORE ridge runs — an oc-armed base config must
/// build through the event-carrying path, else the engine refuses).
#[cfg(feature = "nb_ridge")]
pub(super) fn build_ridge_selection<const N: usize>(
    inp: &ModellessInput<'_>,
    effective_cap: usize,
    base_cfg: &EngineConfig,
    events_for_pool: impl Fn(&[TrainDoc]) -> Vec<crate::option_cond::OcEvent>,
) -> Result<RidgeSelection, String> {
    let spec = inp.spec;
    let SelSlice {
        cases,
        state_strs,
        pool,
    } = selection_slice(inp, "ridge-select")?;
    let oc_events = events_for_pool(&pool);
    let eval_at = |scale: f32| -> Result<f64, String> {
        let cfg = EngineConfig {
            ridge_scale: scale,
            ridge_lambda: RIDGE_LAMBDA,
            ..base_cfg.clone()
        };
        let (mut engine, _) = (|| {
            #[cfg(feature = "option_cond")]
            if !oc_events.is_empty() {
                return build_engine_oc_with::<N>(
                    spec.name,
                    &pool,
                    inp.labels,
                    effective_cap,
                    cfg,
                    &[],
                    &oc_events,
                );
            }
            #[cfg(not(feature = "option_cond"))]
            let _ = &oc_events;
            build_engine::<N>(spec.name, &pool, inp.labels, effective_cap, cfg)
        })()?;
        let (ev, _) = eval_engine(&mut engine, &cases, &state_strs, false)?;
        Ok(hard_metrics(&ev.forced_rows(&cases)).accuracy)
    };
    let base = eval_at(0.0)?;
    eprintln!("    ridge-select: scale 0 (off) → sel-slice acc {base:.4}");
    let mut rows = vec![RidgeCandidate {
        scale: 0.0,
        lambda: RIDGE_LAMBDA,
        cal_acc: base,
    }];
    let (mut selected, mut best) = (0.0f32, base);
    for &scale in &RIDGE_SCALE_LADDER {
        let cal_acc = eval_at(scale)?;
        eprintln!("    ridge-select: scale {scale} → sel-slice acc {cal_acc:.4}");
        if cal_acc >= base + HEAD_SELECT_MARGIN && cal_acc > best {
            best = cal_acc;
            selected = scale;
        }
        rows.push(RidgeCandidate {
            scale,
            lambda: RIDGE_LAMBDA,
            cal_acc,
        });
    }
    eprintln!(
        "    ridge-select: selected {selected} — the suite's row below is the single \
         test read, at this posture"
    );
    Ok(RidgeSelection {
        selected_scale: selected,
        selected_lambda: RIDGE_LAMBDA,
        candidates: rows,
    })
}
