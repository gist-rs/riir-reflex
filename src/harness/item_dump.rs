//! Measurement-only per-item outcome dump (riir-rethink Issue 028 T1's
//! instrument; reflex issue `.issues/072`, filed in the same commit).
//!
//! `--dump-items` persists ONE JSONL record per answered question per lane
//! to `<out>/items/<suite>.jsonl`, following the M1ItemLog precedent
//! (per-item pick/conf/gate records carried beside the aggregate row —
//! but written to their OWN file, never into `results.json`, which stays
//! byte-identical whether or not the flag is armed).
//!
//! The records are the raw material for population-size scaling analysis
//! (rethink 028): coverage over arm subsets `v_cov(S)`, gate-realized
//! ordered-ladder value `v_ladder(order)` at the deployed worthiness
//! margin, and exact Shapley attribution — all computed OFFLINE from
//! these lines, never by re-running the lanes.
//!
//! Cross-lane item identity is `(case_id, q_idx)`: `SuiteCase::id` is
//! stable per suite build, so lanes that served a trimmed prefix or a
//! bucket-skipped subset still join against the full-suite modelless
//! read. `case_idx` (the index within the lane's OWN served order) rides
//! along for debugging, never as a join key.
//!
//! Purity law: this module only READS already-materialized eval vectors
//! (probs/picks/confs/abstain/cause) — it never runs an engine, never
//! mutates lane state, and costs one Vec walk against a lane that just
//! ran an encoder. The record is ALWAYS built (the same walk the metrics
//! read); the CLI flag only chooses whether to PERSIST it.

use serde::Serialize;

/// One answered question's outcome on one lane.
#[derive(Debug, Clone, Serialize)]
pub struct ItemOutcome {
    /// Lane id (`"modelless"`, `"laya"`, `"laya-python"`, oracle names).
    pub lane: &'static str,
    /// Checkpoint/model name (`"modelless"` on the modelless lane).
    pub model: String,
    /// Stable per-suite case identity (the join key across lanes).
    pub case_id: String,
    /// Index within THIS lane's served order (debug only — never a key).
    pub case_idx: usize,
    /// Question index within the case.
    pub q_idx: usize,
    /// Question kind (`choice` / `score` / `noul`).
    pub kind: String,
    /// Gold label index (label space; noul: 1 = true).
    pub gold: usize,
    /// Forced pick in label space (the row `hard` metrics score).
    pub pick: usize,
    pub ok: bool,
    /// The posture readout confidence that produced `pick` (modelless:
    /// the RAW readout; laya/oracles: their entropy/answer confidence).
    pub conf: f64,
    /// Modelless only — the CALIBRATED-scale readout confidence.
    pub conf_cal: Option<f64>,
    /// Modelless only — the RAW (unfitted) gate's abstain verdict.
    pub abstain_raw: Option<bool>,
    /// Modelless only — the deployed CALIBRATED gate's abstain verdict
    /// (the escalation trigger of the shipped ESC lane).
    pub abstain_cal: Option<bool>,
    /// Modelless only — the calibrated gate's cause when it abstained.
    pub cause_cal: Option<String>,
}
