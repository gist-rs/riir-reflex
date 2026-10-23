//! The depth probe (Issue 094 T2; substrate riir-infer Issue 1106) — the
//! headless residual-stability readout over the harness suites.
//!
//! Per (case, question): one `RiirAgent::system_one` forward for the PICK
//! (correctness, exactly the serving path), then one
//! `RiirAgent::encoder_forward_capture` streaming the residual stream
//! (`Embedding` + every `AfterLayer(i)`) through the substrate's
//! `StabilityTracker` — the LAST-POSITION row per stage, per the tracker's
//! consumer contract. Reports:
//!
//! - the per-suite depth distribution (p10/50/90 + the `None` count — the
//!   "never stabilized" cases, each named by `max_leaving_delta`),
//! - **AUROC(depth → correct)** — the confidence-candidate readout: does a
//!   LATER stabilization depth predict a correct pick? (A `None` depth
//!   enters the AUROC as the deepest rank + disclosure.)
//! - depth-as-confidence **ECE vs BOTH floors** (Report-the-Floor law):
//!   the conformal floor through `DrexConfReadout::floor_ece`'s shape and
//!   the T1 input-keyed modal floor (`modal_floor::suite_floor_accuracy`)
//!   as the mimicry bar. A depth signal that beats conformal but not modal
//!   is mimicry; beats modal but not conformal is miscalibration.
//!
//! REPORT-ONLY (the issue's posture): no exit-code coupling beyond the
//! loud refusals, no serving-path edit, CPU-only run leg, capped
//! (default 100 cases/suite — the laya CPU lane is ~0.15–0.5 s/forward,
//! two forwards per case).
//!
//! NOT a correctness instrument: the depth distribution is a property of
//! the checkpoint + band. The band is pinned by the caller
//! (`--depth-band-cos`, `--depth-band-l2`; defaults 0.9999 / 1e-3 — the
//! values the substrate's own gates calibrate at).

#![cfg(feature = "twt_profile")]

use serde::Serialize;

use super::perm_probe::stride_indices;
use super::*;

/// Probe options (the `--depth-probe` flag set).
#[derive(Debug, Clone)]
pub struct DepthProbeOptions {
    /// Stride-capped cases per suite (default 100 — the CPU run leg's
    /// budget).
    pub max_cases: usize,
    /// The "materially stopped moving" band (substrate `StabilityBand`),
    /// ABSOLUTE — and therefore scale-bound: the English encoder's raw
    /// residual stream runs mid-depth l2 ≈ 4–20 and tail l2 ≈ 90–130, so
    /// the default l2 band pins the MEASURED trough scale (25.0), not a
    /// dimension-free constant (the first live run at l2 = 1e-3 read
    /// structurally-None everywhere — recorded in the issue). Cross-
    /// checkpoint sweeps re-pin via `--depth-band-l2`.
    pub band_cos: f32,
    pub band_l2: f32,
    /// The trough window (consecutive sub-band transitions that certify a
    /// settle). 2 = the shortest run that survives one noisy blip; the
    /// measured cosines peak ≈ 0.998 at this checkpoint, so the default
    /// cos band pins 0.995 (the first live run at 0.9999 read
    /// structurally-None everywhere — recorded in the issue).
    pub trough_window: usize,
}

impl Default for DepthProbeOptions {
    fn default() -> Self {
        Self {
            max_cases: 100,
            band_cos: 0.995,
            band_l2: 25.0,
            trough_window: 2,
        }
    }
}

/// One question's depth + correctness row.
#[derive(Debug, Clone, Serialize)]
pub struct DepthRow {
    pub suite: String,
    pub case_id: String,
    pub qid: String,
    /// The serving pick's option index (into the case's own order).
    pub pick: usize,
    /// Whether the pick was the gold label.
    pub correct: bool,
    /// The stabilization depth (stage ordinal; `Embedding` = 0). `None` =
    /// never stabilized inside the network (disclosed per suite). SUFFIX
    /// semantics (the substrate's law: everything after the depth stays
    /// sub-band) — for a stream whose tail RE-ACCELERATES (the English
    /// encoder's measured shape: mid-depth trough l2 ≈ 4–20, final layers
    /// 90–130) the suffix read is structurally `None` and the usable
    /// candidate is [`DepthRow::trough_depth`].
    pub depth: Option<usize>,
    /// The WINDOWED-TROUGH depth (the consumer-side confidence candidate):
    /// the earliest stage whose NEXT `window` transitions are ALL sub-band
    /// (the representation settles there even if the tail re-accelerates).
    /// `None` = no such window (the stream never settles for `window`
    /// straight transitions).
    pub trough_depth: Option<usize>,
    /// The largest leaving transition's L2 (the `None` disclosure).
    pub max_leaving_l2: Option<f32>,
    /// The full per-transition (cos, l2) series (stage 1..n leaving
    /// transitions, capture order — the artifact's diagnostic tail).
    pub delta_cos_l2s: Vec<(f32, f32)>,
}

/// Per-suite summary.
#[derive(Debug, Clone, Serialize)]
pub struct DepthSuiteRow {
    pub suite: String,
    pub questions: usize,
    pub accuracy: f64,
    /// Depth distribution over the STABILIZED cases (None excluded, counted
    /// beside).
    pub depth_p10: Option<usize>,
    pub depth_p50: Option<usize>,
    pub depth_p90: Option<usize>,
    pub none_depths: usize,
    /// AUROC(trough_depth → correct) over the rows WITH a trough (None
    /// rows ranked deepest). The confidence-candidate readout the paper
    /// means; the suffix `depth` AUROC is a structural 0.5 here (all-None
    /// — the re-accelerating tail), recorded beside as the honest control.
    pub auroc_depth_correct: Option<f64>,
    /// Depth-as-confidence ECE (10 equal bins over trough/max_trough) —
    /// compared against the modal floor accuracy (the mimicry bar; the
    /// conformal-floor shape ride is a later slice).
    pub ece_depth: Option<f64>,
    pub conformal_floor_ece: Option<f64>,
    pub modal_floor_accuracy: Option<f64>,
}

/// The probe output (json + md render).
#[derive(Debug, Clone, Serialize)]
pub struct DepthProbeOutput {
    pub checkpoint: String,
    pub device: String,
    pub band_cos: f32,
    pub band_l2: f32,
    pub trough_window: usize,
    pub max_cases: usize,
    pub suites: Vec<DepthSuiteRow>,
    /// The per-question rows (the artifact — depth-vs-correct scatter).
    pub rows: Vec<DepthRow>,
    pub skipped: Vec<(String, String)>,
}

/// AUROC over (depth, correct) pairs with the None-depth rows ranked
/// DEEPEST (never stabilized = the least confident end... disclosed: the
/// convention is a CHOICE — a None depth is the *noisiest* tail of the
/// stream, and the AUROC's direction is computed so that LARGER depth →
/// MORE likely correct; flipping the sign of the AUROC flips the reading).
fn auroc_depth_correct(rows: &[&DepthRow]) -> Option<f64> {
    // Mann–Whitney: positive pairs = correct, negative = incorrect; score =
    // the TROUGH depth rank (None → deepest). Ties count ½.
    // None-trough rows rank by (MAX, +inf leaving-L2): the noisiest tail.
    // f32 lacks Ord — sort by (trough_key, leaving_key) via total_cmp on
    // the second component.
    let key = |r: &DepthRow| {
        (
            r.trough_depth.unwrap_or(usize::MAX),
            r.max_leaving_l2.unwrap_or(f32::INFINITY),
        )
    };
    let mut all: Vec<&DepthRow> = rows.to_vec();
    all.sort_by(|a, b| {
        key(a)
            .0
            .cmp(&key(b).0)
            .then_with(|| key(a).1.total_cmp(&key(b).1))
    });
    let pos: usize = rows.iter().filter(|r| r.correct).count();
    let neg = rows.len() - pos;
    if pos == 0 || neg == 0 {
        return None;
    }
    // Average ranks (1-based): every row in a tie group carries the
    // group's average rank, and each row credits ITS OWN correctness (a
    // mixed tie group splits its credit — a per-group-first-row check
    // inflated pos ranks past 1.0 AUROC in the first live run).
    let mut rank_sum_pos = 0.0f64;
    let mut i = 0usize;
    while i < all.len() {
        let mut j = i;
        while j < all.len() && key(all[j]) == key(all[i]) {
            j += 1;
        }
        let avg_rank = (i + j + 1) as f64 / 2.0; // 1-based average rank
        for r in &all[i..j] {
            if r.correct {
                rank_sum_pos += avg_rank;
            }
        }
        i = j;
    }
    let u = rank_sum_pos - (pos as f64 * (pos as f64 + 1.0)) / 2.0;
    Some(u / (pos as f64 * neg as f64))
}

/// Depth-as-confidence ECE: confidence(depth) = depth / max_depth observed
/// (None rows excluded from the ECE — they have no depth to bin; their
/// count is the suite's `none_depths`). 10 equal-width bins, left-open per
/// the harness ECE convention.
fn ece_depth(rows: &[&DepthRow]) -> Option<f64> {
    let max_depth = rows.iter().filter_map(|r| r.trough_depth).max()?;
    if max_depth == 0 {
        return None;
    }
    const BINS: usize = 10;
    let mut bin_conf = [0.0f64; BINS];
    let mut bin_acc = [0.0f64; BINS];
    let mut bin_n = [0usize; BINS];
    for r in rows {
        let Some(d) = r.trough_depth else { continue };
        if d == 0 {
            continue; // depth 0 = stabilized at the embedding — zero confidence under this map
        }
        let b = (((d - 1) * BINS) / max_depth).min(BINS - 1);
        bin_conf[b] += d as f64 / max_depth as f64;
        bin_acc[b] += if r.correct { 1.0 } else { 0.0 };
        bin_n[b] += 1;
    }
    let total: usize = bin_n.iter().sum();
    if total == 0 {
        return None;
    }
    let mut ece = 0.0f64;
    for b in 0..BINS {
        if bin_n[b] == 0 {
            continue;
        }
        let conf = bin_conf[b] / bin_n[b] as f64;
        let acc = bin_acc[b] / bin_n[b] as f64;
        ece += bin_n[b] as f64 / total as f64 * (conf - acc).abs();
    }
    Some(ece)
}

fn percentile(v: &mut [usize], p: f64) -> Option<usize> {
    if v.is_empty() {
        return None;
    }
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    Some(v[idx.min(v.len() - 1)])
}

/// The probe run. CPU-only by construction (the substrate's capture path is
/// backend-generic; `LAYA_DEVICE` may select metal for speed — the run leg
/// refuses nothing, but the REPORT discloses the device label either way).
pub fn run_depth_probe(
    opts: &RunOptions,
    dopts: &DepthProbeOptions,
) -> Result<DepthProbeOutput, String> {
    use crate::laya::riir::agent::DeviceKind;
    let device = DeviceKind::from_env()
        .map(|d| format!("{d:?}"))
        .unwrap_or_else(|_| "default".to_string());

    // The English checkpoint only — the probe is a characterization of ONE
    // checkpoint's residual stream (multi-checkpoint sweeps are a later
    // lane, priced separately).
    let ckpt = "english";
    let ck = crate::laya::config::Checkpoint::English;
    let agent = super::load_laya_agent(ckpt, ck)?;

    let mut rows: Vec<DepthRow> = Vec::new();
    let mut suite_rows: Vec<DepthSuiteRow> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();

    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only {
            continue;
        }
        let prepared = match prepare(spec, &opts.datasets_dir) {
            Ok(p) => p,
            Err(e) => {
                skipped.push((spec.name.to_string(), format!("prepare: {e}")));
                continue;
            }
        };
        let idx = stride_indices(prepared.suite.cases.len(), dopts.max_cases);
        if idx.is_empty() {
            skipped.push((spec.name.to_string(), "no cases after stride".into()));
            continue;
        }
        eprintln!(
            "[depth-probe] {}: {} case(s) × {} question(s)",
            spec.name,
            idx.len(),
            prepared
                .suite
                .cases
                .first()
                .map(|c| c.questions.len())
                .unwrap_or(0)
        );

        for &ci in &idx {
            let case: &SuiteCase = &prepared.suite.cases[ci];
            // The serving pick, once per case (all questions in one
            // forward — exactly the serving path).
            let answers = match agent.system_one(&case.state, &case_questions(case)) {
                Ok(a) => a,
                Err(crate::laya::LayaError::Bucket { seq, max, .. }) => {
                    skipped.push((case.id.clone(), format!("bucket: seq {seq} > {max}")));
                    continue;
                }
                Err(e) => return Err(format!("laya system_one ({}): {e}", case.id)),
            };
            // Per question: the capture path.
            let questions = case_questions(case);
            for (qi, (qid, qdef)) in questions.iter().enumerate() {
                let Some(ans) = answers.get(qi) else { break };
                // The pick in PRESENTED-index space (the shared laya pick
                // rule — criteria-key order, choice-key position, max-prob
                // fallback; noul's two-class map).
                let (_probs, pick) = super::laya_prob_pick(&case.questions[qi], ans);
                let correct = pick == case.gold[qi].idx;

                // One capture per question: tokenize FIRST to learn the
                // sequence length (the substrate's own smoke pattern — the
                // sink filters rows to the LAST position, which cannot be
                // learned from the sweep itself: the Embedding stage's rows
                // arrive in order, so a running count would fire on every
                // row and mix positions into the tracker). The double
                // tokenize is a calibration-cost no-op.
                let (ids, _markers) = agent
                    .tokenize_question(&case.state, qdef)
                    .map_err(|e| format!("tokenize ({}:{qid}): {e}", case.id))?;
                let last = ids.len().saturating_sub(1);
                let mut trk: Option<crate::laya::riir::stability::StabilityTracker> = None;
                {
                    let mut stage_sink = |stage: crate::laya::riir::encoder::CaptureStage,
                                          row_index: usize,
                                          row: &[f32]| {
                        if row_index == last {
                            let tr = trk.get_or_insert_with(|| {
                                crate::laya::riir::stability::StabilityTracker::with_stage_capacity(
                                    64,
                                    row.len(),
                                )
                            });
                            tr.observe(stage, row);
                        }
                    };
                    if let Err(e) =
                        agent.encoder_forward_capture(&case.state, qdef, &mut stage_sink)
                    {
                        skipped.push((format!("{}:{qid}", case.id), format!("capture: {e}")));
                        continue;
                    }
                }
                let band = crate::laya::riir::stability::StabilityBand {
                    cos: dopts.band_cos,
                    l2: dopts.band_l2,
                };
                let (depth, max_leaving_l2, deltas) = match trk.as_ref() {
                    Some(t) => (
                        t.stabilization_depth(&band),
                        t.max_leaving_delta().map(|(_, d)| d.l2),
                        t.deltas().to_vec(),
                    ),
                    None => (None, None, Vec::new()),
                };
                // The windowed trough: the earliest stage whose next
                // `window` consecutive transitions are all sub-band.
                let sub_band = |d: &crate::laya::riir::stability::StageDelta| {
                    d.cos >= dopts.band_cos && d.l2 <= dopts.band_l2
                };
                let trough_depth = (0..deltas.len())
                    .find(|&i| {
                        deltas.len() - i >= dopts.trough_window
                            && deltas[i..i + dopts.trough_window].iter().all(sub_band)
                    })
                    .map(|i| i + 1); // the FROM stage of the first sub-band transition
                rows.push(DepthRow {
                    suite: spec.name.to_string(),
                    case_id: case.id.clone(),
                    qid: qid.clone(),
                    pick,
                    correct,
                    depth,
                    trough_depth,
                    max_leaving_l2,
                    delta_cos_l2s: deltas.iter().map(|d| (d.cos, d.l2)).collect(),
                });
            }
        }

        // Suite summary.
        let srows: Vec<&DepthRow> = rows.iter().filter(|r| r.suite == spec.name).collect();
        if srows.is_empty() {
            skipped.push((spec.name.to_string(), "no rows captured".into()));
            continue;
        }
        let accuracy = srows.iter().filter(|r| r.correct).count() as f64 / srows.len() as f64;
        let mut depths: Vec<usize> = srows.iter().filter_map(|r| r.trough_depth).collect();
        let none_depths = srows.len() - depths.len();
        let depth_p10 = percentile(&mut depths, 0.10);
        let depth_p50 = percentile(&mut depths, 0.50);
        let depth_p90 = percentile(&mut depths, 0.90);
        let auroc = auroc_depth_correct(&srows);
        let ece = ece_depth(&srows);
        // The modal floor (the T1 instrument, report-only here): the
        // census over this suite's prepared cases, LOO posture (the
        // disclosed degraded form — the probe reads no cal slice).
        let modal = crate::harness::modal_floor::census(&prepared.suite.cases, None).map(|f| f.acc);
        // The conformal floor shape: depth-confidence ECE is compared
        // against the CONFORMAL floor's ECE on the same rows when the
        // drex readout is available — the shape ride is a later slice; the
        // modal floor + AUROC carry the T2 verdict on their own.
        suite_rows.push(DepthSuiteRow {
            suite: spec.name.to_string(),
            questions: srows.len(),
            accuracy,
            depth_p10,
            depth_p50,
            depth_p90,
            none_depths,
            auroc_depth_correct: auroc,
            ece_depth: ece,
            conformal_floor_ece: None,
            modal_floor_accuracy: modal,
        });
    }

    if suite_rows.is_empty() {
        return Err(format!(
            "depth probe: no suite produced rows (skipped: {skipped:?}) — check --suites, \
             the datasets dir, and the weights root"
        ));
    }

    Ok(DepthProbeOutput {
        checkpoint: ckpt.to_string(),
        device,
        band_cos: dopts.band_cos,
        band_l2: dopts.band_l2,
        trough_window: dopts.trough_window,
        max_cases: dopts.max_cases,
        suites: suite_rows,
        rows,
        skipped,
    })
}

/// The markdown render (the artifact's human half).
pub fn render_depth_probe_markdown(out: &DepthProbeOutput) -> String {
    let mut s = String::with_capacity(4096);
    s.push_str("# Depth probe — residual stabilization vs correctness (Issue 094 T2)\n\n");
    s.push_str(&format!(
        "checkpoint `{}` · device `{}` · band cos ≥ {} / l2 ≤ {} · trough window {} · ≤{} case(s)/suite\n\n",
        out.checkpoint, out.device, out.band_cos, out.band_l2, out.trough_window, out.max_cases
    ));
    s.push_str("| suite | q | acc | trough p10/p50/p90 | none | AUROC(trough→correct) | ECE(trough) | modal floor |\n");
    s.push_str("|---|---|---|---|---|---|---|---|\n");
    for r in &out.suites {
        s.push_str(&format!(
            "| {} | {} | {:.4} | {}/{}/{} | {} | {} | {} | {} |\n",
            r.suite,
            r.questions,
            r.accuracy,
            r.depth_p10.map(|d| d.to_string()).unwrap_or("-".into()),
            r.depth_p50.map(|d| d.to_string()).unwrap_or("-".into()),
            r.depth_p90.map(|d| d.to_string()).unwrap_or("-".into()),
            r.none_depths,
            r.auroc_depth_correct
                .map(|a| format!("{a:.4}"))
                .unwrap_or_else(|| "n/a".into()),
            r.ece_depth
                .map(|e| format!("{e:.4}"))
                .unwrap_or_else(|| "n/a".into()),
            r.modal_floor_accuracy
                .map(|m| format!("{m:.4}"))
                .unwrap_or_else(|| "n/a".into()),
        ));
    }
    if !out.skipped.is_empty() {
        s.push_str("\n## Skipped\n\n");
        for (id, why) in &out.skipped {
            s.push_str(&format!("- `{id}`: {why}\n"));
        }
    }
    s
}
