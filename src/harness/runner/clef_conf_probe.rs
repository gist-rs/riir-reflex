//! The clef headful confidence-candidate probe (Issue 094 T3) — the
//! clef-text-0.6b lane's LAST-POSITION ENTROPY read against the two
//! floors, the headful sibling of T2's headless depth probe.
//!
//! Per (case): one `LocalAneLane::decide_with_entropy` pass — the serving
//! answers AND the tied-table vocabulary entropy from the SAME decode
//! (substrate riir-infer `0f876b8`: the final hidden at the record's last
//! real token projected through the tied embedding table, peak-subtracted
//! softmax, `H = peak + ln Z − Σ exp(l−peak)·l / Z` NATS). The signal is
//! CASE-LEVEL by construction (one record = one last position), so every
//! question of a case carries its case's entropy — disclosed on the rows
//! and priced into the AUROC's tie handling (average ranks).
//!
//! G1 (the issue's law): the candidate MUST beat BOTH floors or is
//! recorded NEGATIVE with the miss named —
//! (a) the **conformal floor**: the shipped split-half conformal-naive
//!     floor ECE shape (`assemble_drex_conf_readout`'s: pooled incumbent
//!     confidence pairs, first half calibrates, second half audited,
//!     occupancy floor) over the lane's OWN max-prob confidences — the
//!     gate measures the same way as the lanes the candidate would be
//!     compared against;
//! (b) the **input-keyed modal floor** (T1): priced BOTH ways — the
//!     census ACC as the mimicry bar for the lane itself, and the modal
//!     pick's census SHARE as a confidence through the same `ece_of`
//!     (`census_with_pairs`) — beats conformal but fails modal is
//!     mimicry; beats modal but fails conformal is miscalibration.
//!
//! Entropy→confidence map: `conf = (1 − H/ln V).clamp(0, 1)` over the
//! config vocab — a monotone map (rank-preserving, so the AUROC is
//! map-invariant) into the [0,1] scale `ece_of`'s 15 bins read; raw
//! entropies fall outside every bin and would read a fabricated 0.0.
//!
//! LENS DEPTH is NOT here: the ANE posture ships one output (the final
//! states) — per-layer states never leave the Core ML graph, so a logit
//! lens at depth needs a CPU forward port (a substrate lane of its own).
//! Deferred in the issue with that blocker named; this probe is the
//! last-position candidate only.
//!
//! REPORT-ONLY (the issue's posture): no exit-code coupling beyond the
//! loud refusals, no serving-path edit, macOS ANE seat, capped
//! (default 100 cases/suite).

#![cfg(all(feature = "clef-ane", target_os = "macos"))]

use serde::Serialize;

use super::perm_probe::stride_indices;
use super::*;
use crate::lanes::clef::LocalAneLane;

/// Probe options (the `--clef-conf-probe` flag set).
#[derive(Debug, Clone)]
pub struct ClefConfProbeOptions {
    /// Stride-capped cases per suite (default 100).
    pub max_cases: usize,
}

impl Default for ClefConfProbeOptions {
    fn default() -> Self {
        Self { max_cases: 100 }
    }
}

/// One question's confidence row (the artifact — entropy-vs-correct
/// scatter). `entropy` is the CASE's scalar, repeated across the case's
/// questions by construction.
#[derive(Debug, Clone, Serialize)]
pub struct ClefConfRow {
    pub suite: String,
    pub case_id: String,
    pub qid: String,
    pub pick: usize,
    pub correct: bool,
    /// The incumbent readout: the lane's max-prob confidence.
    pub conf: f64,
    /// The case-level last-position vocabulary entropy (NATS).
    pub entropy: f32,
    /// The entropy→confidence map the ECE reads (monotone in −entropy).
    pub entropy_conf: f64,
}

/// Per-suite summary + the G1 adjudication.
#[derive(Debug, Clone, Serialize)]
pub struct ClefConfSuiteRow {
    pub suite: String,
    pub questions: usize,
    pub accuracy: f64,
    /// AUROC(entropy → correct) with the sign FIXED: the score is
    /// −entropy (lower entropy = more confident = higher rank), ties
    /// averaged. > 0.5 = the candidate discriminates.
    pub auroc_entropy_correct: Option<f64>,
    /// The incumbent readout's own ECE (the honest beside-number).
    pub conf_ece: f64,
    /// The candidate's ECE through the monotone map.
    pub entropy_ece: f64,
    /// Floor (a): the split-half conformal floor ECE over the incumbent
    /// confidence pairs. `None` below the occupancy floor.
    pub conformal_floor_ece: Option<f64>,
    pub conformal_floor_n_cal: usize,
    pub conformal_floor_n_test: usize,
    /// Floor (b), accuracy reading (the lane itself must clear this).
    pub modal_floor_accuracy: Option<f64>,
    /// Floor (b), calibration reading: the modal pick's census share as
    /// a confidence through the same `ece_of`.
    pub modal_conf_ece: Option<f64>,
    pub modal_floor_reference: Option<String>,
    /// Mean wall ms per case (ANE forward + the entropy sweep — the
    /// T4 cost disclosure; the incumbent readout adds zero).
    pub mean_case_ms: f64,
    /// "PASS" | "NEGATIVE(<miss>)" | "UNADJUDICABLE(<why>)".
    pub verdict: String,
}

/// The probe output (json + md render).
#[derive(Debug, Clone, Serialize)]
pub struct ClefConfProbeOutput {
    pub model: String,
    pub posture: String,
    pub vocab: u32,
    pub max_cases: usize,
    pub suites: Vec<ClefConfSuiteRow>,
    pub rows: Vec<ClefConfRow>,
    pub skipped: Vec<(String, String)>,
}

/// The occupancy floor below which the split-half conformal floor is
/// noise — the Drex lane's shared law (40 pairs → 20/20).
const CLEF_CONF_MIN_FLOOR_PAIRS: usize = super::DREX_MIN_FLOOR_PAIRS;

/// Mann–Whitney AUROC over (signal, correct) with average ranks for
/// ties. The score convention is the caller's (`score` higher = more
/// confident); the direction is NOT flipped here.
fn auroc_of(score: impl Fn(&ClefConfRow) -> f64, rows: &[&ClefConfRow]) -> Option<f64> {
    let pos = rows.iter().filter(|r| r.correct).count();
    let neg = rows.len() - pos;
    if pos == 0 || neg == 0 {
        return None;
    }
    let mut keyed: Vec<(f64, bool)> = rows.iter().map(|r| (score(r), r.correct)).collect();
    keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut rank_sum_pos = 0.0f64;
    let mut i = 0usize;
    while i < keyed.len() {
        let mut j = i;
        while j < keyed.len() && keyed[j].0 == keyed[i].0 {
            j += 1;
        }
        let avg_rank = (i + j + 1) as f64 / 2.0;
        for (_, correct) in &keyed[i..j] {
            if *correct {
                rank_sum_pos += avg_rank;
            }
        }
        i = j;
    }
    let u = rank_sum_pos - (pos as f64 * (pos as f64 + 1.0)) / 2.0;
    Some(u / (pos as f64 * neg as f64))
}

/// The G1 adjudication (pure — fixture-tested): the candidate must beat
/// BOTH floors STRICTLY; a missing floor (occupancy) is unadjudicable,
/// never a pass.
fn adjudicate(
    cand: Option<f64>,
    conformal: Option<f64>,
    modal: Option<f64>,
    accuracy: f64,
    modal_acc: Option<f64>,
) -> String {
    let Some(cand) = cand else {
        return "UNADJUDICABLE(no candidate ece)".into();
    };
    let Some(conformal) = conformal else {
        return format!(
            "UNADJUDICABLE(conformal floor below the occupancy floor — candidate ece {cand:.4})"
        );
    };
    let Some(modal) = modal else {
        return format!(
            "UNADJUDICABLE(no modal pairs — candidate ece {cand:.4} vs conformal {conformal:.4})"
        );
    };
    let misses = [
        (
            cand >= conformal,
            format!("conformal {cand:.4} ≥ floor {conformal:.4}"),
        ),
        (cand >= modal, format!("modal {cand:.4} ≥ floor {modal:.4}")),
    ];
    let mut failed: Vec<String> = misses
        .iter()
        .filter(|(bad, _)| *bad)
        .map(|(_, why)| why.clone())
        .collect();
    // The mimicry bar rides beside: a lane below the census accuracy has
    // no input-independent signal at all, whatever its confidence reads.
    if modal_acc.is_some_and(|macc| accuracy <= macc) {
        failed.push(format!(
            "lane acc {accuracy:.4} ≤ modal acc {} (mimicry)",
            modal_acc.unwrap_or(f64::NAN)
        ));
    }
    if failed.is_empty() {
        "PASS".into()
    } else {
        format!("NEGATIVE({})", failed.join("; "))
    }
}

/// The entropy→confidence map (the ECE's scale; the AUROC is
/// map-invariant).
fn entropy_conf(h: f32, vocab: u32) -> f64 {
    let ln_v = (vocab as f64).ln();
    if ln_v <= 0.0 {
        return 0.0;
    }
    (1.0 - f64::from(h) / ln_v).clamp(0.0, 1.0)
}

/// The probe run: the local ANE seat over the stride-capped suites.
pub fn run_clef_conf_probe(
    opts: &RunOptions,
    copts: &ClefConfProbeOptions,
) -> Result<ClefConfProbeOutput, String> {
    let lane = LocalAneLane::from_env()?;
    let (model, posture) = lane.info()?;
    let vocab = {
        // The config vocab the entropy map normalizes by — read off the
        // loaded runtime (the pinned artifact's own config).
        lane.vocab()
    };

    let mut rows: Vec<ClefConfRow> = Vec::new();
    let mut suite_rows: Vec<ClefConfSuiteRow> = Vec::new();
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
        let idx = stride_indices(prepared.suite.cases.len(), copts.max_cases);
        if idx.is_empty() {
            skipped.push((spec.name.to_string(), "no cases after stride".into()));
            continue;
        }
        eprintln!(
            "[clef-conf-probe] {}: {} case(s) × {} question(s)",
            spec.name,
            idx.len(),
            prepared
                .suite
                .cases
                .first()
                .map(|c| c.questions.len())
                .unwrap_or(0)
        );

        let mut case_ms_sum = 0.0f64;
        let mut case_n = 0usize;
        for &ci in &idx {
            let case: &SuiteCase = &prepared.suite.cases[ci];
            let (triples, ms, entropy) = match lane.decide_with_entropy(case) {
                Ok(r) => r,
                Err(e) => {
                    skipped.push((case.id.clone(), e));
                    continue;
                }
            };
            case_ms_sum += ms;
            case_n += 1;
            for (qi, q) in case.questions.iter().enumerate() {
                let Some((probs, pick, conf)) = triples.get(qi) else {
                    break;
                };
                // The row artifact carries the readouts (pick, conf,
                // entropy) — the full probability vector stays on the
                // lane's decide wire, the row file stays small.
                let _ = probs;
                rows.push(ClefConfRow {
                    suite: spec.name.to_string(),
                    case_id: case.id.clone(),
                    qid: q.qid.clone(),
                    pick: *pick,
                    correct: *pick == case.gold[qi].idx,
                    conf: *conf,
                    entropy,
                    entropy_conf: entropy_conf(entropy, vocab),
                });
            }
        }

        // Suite summary.
        let srows: Vec<&ClefConfRow> = rows.iter().filter(|r| r.suite == spec.name).collect();
        if srows.is_empty() {
            skipped.push((spec.name.to_string(), "no rows captured".into()));
            continue;
        }
        let accuracy = srows.iter().filter(|r| r.correct).count() as f64 / srows.len() as f64;
        // The candidate readouts.
        let auroc = auroc_of(|r| -f64::from(r.entropy), &srows);
        let entropy_pairs: Vec<(f64, bool)> =
            srows.iter().map(|r| (r.entropy_conf, r.correct)).collect();
        let entropy_ece = crate::harness::metrics::ece_of(&entropy_pairs);
        // The incumbent readout + floor (a): the Drex shape — pooled
        // incumbent-confidence pairs, case-order split-half.
        let conf_pairs: Vec<(f64, bool)> = srows.iter().map(|r| (r.conf, r.correct)).collect();
        let conf_ece = crate::harness::metrics::ece_of(&conf_pairs);
        let (floor_ece, n_cal, n_test) = if conf_pairs.len() >= CLEF_CONF_MIN_FLOOR_PAIRS {
            let cut = conf_pairs.len() / 2;
            let cal: Vec<crate::harness::metrics::CalibrationPair> = conf_pairs[..cut]
                .iter()
                .map(|&(conf, correct)| crate::harness::metrics::CalibrationPair { conf, correct })
                .collect();
            let test_confs: Vec<f64> = conf_pairs[cut..].iter().map(|&(c, _)| c).collect();
            let floored = crate::harness::metrics::conformal_naive_floor(&cal, &test_confs);
            let floor_pairs: Vec<(f64, bool)> = floored
                .into_iter()
                .zip(conf_pairs[cut..].iter().map(|&(_, ok)| ok))
                .collect();
            (
                Some(crate::harness::metrics::ece_of(&floor_pairs)),
                cut,
                conf_pairs.len() - cut,
            )
        } else {
            (None, 0, 0)
        };
        // Floor (b): the modal census over the prepared cases with the
        // CAL slice as the reference population (the T1 preferred form —
        // the probe loads it; empty cal falls back to the disclosed LOO).
        let modal_ref = if prepared.cal_cases.is_empty() {
            None
        } else {
            Some(prepared.cal_cases.as_slice())
        };
        let (modal_row, modal_pairs) =
            crate::harness::modal_floor::census_with_pairs(&prepared.suite.cases, modal_ref)
                .map_or((None, None), |(r, p)| (Some(r), Some(p)));
        let modal_acc = modal_row.as_ref().map(|r| r.acc);
        let modal_ece = modal_pairs
            .as_ref()
            .filter(|p| !p.is_empty())
            .map(|p| crate::harness::metrics::ece_of(p));
        let verdict = adjudicate(Some(entropy_ece), floor_ece, modal_ece, accuracy, modal_acc);
        eprintln!(
            "[clef-conf-probe] {}: acc {:.4} · entropy ece {:.4} · conf ece {:.4} · \
             conformal {} · modal acc {} / conf ece {} — {}",
            spec.name,
            accuracy,
            entropy_ece,
            conf_ece,
            floor_ece.map_or("n/a".into(), |v| format!("{v:.4}")),
            modal_acc.map_or("n/a".into(), |v| format!("{v:.4}")),
            modal_ece.map_or("n/a".into(), |v| format!("{v:.4}")),
            verdict,
        );
        suite_rows.push(ClefConfSuiteRow {
            suite: spec.name.to_string(),
            questions: srows.len(),
            accuracy,
            auroc_entropy_correct: auroc,
            conf_ece,
            entropy_ece,
            conformal_floor_ece: floor_ece,
            conformal_floor_n_cal: n_cal,
            conformal_floor_n_test: n_test,
            modal_floor_accuracy: modal_acc,
            modal_conf_ece: modal_ece,
            modal_floor_reference: modal_row.map(|r| r.reference.to_string()),
            mean_case_ms: if case_n > 0 {
                case_ms_sum / case_n as f64
            } else {
                0.0
            },
            verdict,
        });
    }

    if suite_rows.is_empty() {
        return Err(format!(
            "clef conf probe: no suite produced rows (skipped: {skipped:?}) — check \
             --suites, the datasets dir, and the clef artifact tree"
        ));
    }

    Ok(ClefConfProbeOutput {
        model,
        posture,
        vocab,
        max_cases: copts.max_cases,
        suites: suite_rows,
        rows,
        skipped,
    })
}

/// The markdown render (the artifact's human half).
pub fn render_clef_conf_probe_markdown(out: &ClefConfProbeOutput) -> String {
    let mut s = String::with_capacity(4096);
    s.push_str("# Clef conf probe — last-position entropy vs the two floors (Issue 094 T3)\n\n");
    s.push_str(&format!(
        "{} · {} · vocab {} · ≤{} case(s)/suite · entropy→conf map `(1 − H/ln V).clamp(0,1)` · \
         case-level signal (one entropy per case, repeated across its questions)\n\n",
        out.model, out.posture, out.vocab, out.max_cases,
    ));
    s.push_str("| suite | q | acc | AUROC(−H→correct) | ECE(H) | ECE(conf) | conformal floor | modal acc | modal ECE | verdict |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for r in &out.suites {
        s.push_str(&format!(
            "| {} | {} | {:.4} | {} | {:.4} | {:.4} | {} | {} | {} | {} |\n",
            r.suite,
            r.questions,
            r.accuracy,
            r.auroc_entropy_correct
                .map_or("n/a".into(), |v| format!("{v:.4}")),
            r.entropy_ece,
            r.conf_ece,
            r.conformal_floor_ece
                .map_or("n/a".into(), |v| format!("{v:.4}")),
            r.modal_floor_accuracy
                .map_or("n/a".into(), |v| format!("{v:.4}")),
            r.modal_conf_ece.map_or("n/a".into(), |v| format!("{v:.4}")),
            r.verdict,
        ));
    }
    s.push_str(
        "\nG1: the candidate must beat BOTH floors strictly; \
`beats conformal but not modal` = mimicry, `beats modal but not conformal` = \
miscalibration, a lane at/below the modal accuracy carries no input-independent \
signal. Lens depth is DEFERRED (the ANE artifact ships one output — a CPU \
forward port is the unblock; recorded in the issue).\n",
    );
    if !out.skipped.is_empty() {
        s.push_str("\n## Skipped\n\n");
        for (id, why) in &out.skipped {
            s.push_str(&format!("- `{id}`: {why}\n"));
        }
    }
    s
}

#[cfg(test)]
mod clef_conf_probe_tests {
    //! The pure halves: the AUROC tie law (case-level entropy repeats —
    //! average ranks must hold), the entropy→confidence map's range, and
    //! the G1 adjudication's verdict split.

    use super::*;

    fn row(conf: f64, entropy: f32, correct: bool) -> ClefConfRow {
        ClefConfRow {
            suite: "s".into(),
            case_id: "c".into(),
            qid: "q".into(),
            pick: 0,
            correct,
            conf,
            entropy,
            entropy_conf: 0.0,
        }
    }

    #[test]
    fn auroc_lower_entropy_ranks_confident() {
        // Four rows: the two correct carry LOWER entropy — a perfect
        // discriminator reads 1.0 under the −entropy score.
        let rows = [
            row(0.9, 1.0, true),
            row(0.9, 2.0, true),
            row(0.9, 8.0, false),
            row(0.9, 9.0, false),
        ];
        let refs: Vec<&ClefConfRow> = rows.iter().collect();
        let a = auroc_of(|r| -f64::from(r.entropy), &refs).expect("both classes");
        assert!((a - 1.0).abs() < 1e-12, "perfect: {a}");
    }

    #[test]
    fn auroc_case_level_ties_average() {
        // The case-level shape: the same entropy on every row → all ties
        // → the AUROC must read exactly 0.5, never 1.0 (a per-group-first
        // rank check would inflate it — the T2 lesson).
        let rows = [
            row(0.9, 3.0, true),
            row(0.9, 3.0, true),
            row(0.9, 3.0, false),
            row(0.9, 3.0, false),
        ];
        let refs: Vec<&ClefConfRow> = rows.iter().collect();
        let a = auroc_of(|r| -f64::from(r.entropy), &refs).expect("both classes");
        assert!((a - 0.5).abs() < 1e-12, "all-tie: {a}");
    }

    #[test]
    fn entropy_conf_maps_into_the_unit_interval() {
        assert!((entropy_conf(0.0, 151_936) - 1.0).abs() < 1e-12);
        assert!((entropy_conf(11.0, 151_936)).abs() >= 0.0);
        assert!(entropy_conf(11.0, 151_936) <= 1.0);
        // An entropy past ln V clamps to 0 (never negative — the bins).
        assert_eq!(entropy_conf(99.0, 4), 0.0);
        // Degenerate vocab → 0.0, never NaN.
        assert_eq!(entropy_conf(1.0, 1), 0.0);
    }

    #[test]
    fn adjudicate_names_the_miss() {
        // Beats both → PASS.
        assert_eq!(
            adjudicate(Some(0.10), Some(0.20), Some(0.30), 0.9, Some(0.3)),
            "PASS"
        );
        // Fails conformal only → miscalibration named.
        let v = adjudicate(Some(0.25), Some(0.20), Some(0.30), 0.9, Some(0.3));
        assert!(v.starts_with("NEGATIVE(") && v.contains("conformal"), "{v}");
        // Fails modal only → mimicry named (cand sits below conformal,
        // at the modal floor).
        let v = adjudicate(Some(0.15), Some(0.20), Some(0.15), 0.9, Some(0.3));
        assert!(
            v.starts_with("NEGATIVE(") && v.contains("modal 0.1500 ≥ floor 0.1500"),
            "{v}"
        );
        // Lane below the census accuracy → the mimicry bar rides beside.
        let v = adjudicate(Some(0.10), Some(0.20), Some(0.30), 0.2, Some(0.3));
        assert!(v.contains("mimicry"), "{v}");
        // Occupancy-missing floor → unadjudicable, never a pass.
        let v = adjudicate(Some(0.10), None, Some(0.30), 0.9, Some(0.3));
        assert!(v.starts_with("UNADJUDICABLE"), "{v}");
        let v = adjudicate(Some(0.10), Some(0.20), None, 0.9, Some(0.3));
        assert!(v.starts_with("UNADJUDICABLE"), "{v}");
    }
}
