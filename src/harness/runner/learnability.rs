//! Issue 104 — the learnability-gate class for the synth-corpus lane.
//!
//! RT-J (arXiv:2610.17867 §3.3–3.4) filters generated forecasting tasks by
//! margin over trivial historical baselines: a task a trivial solver
//! already aces carries no signal. The modelless transport scores each
//! synth row with closed-form trivial-solver margins and derives gated
//! variants by dropping the too-easy POLE (margin ≥ quantile threshold),
//! keeping the rest:
//!
//! - **prior-frequency** — `prior(gold) / max_prior`: rows of the
//!   majority-adjacent labels are the rows an ICL corpus's class prior
//!   already solves;
//! - **lexical-overlap** — gold-label token coverage of the text minus the
//!   best other-label coverage (clamped ≥ 0): rows whose signature word
//!   uniquely names the label are solved by lexical matching;
//! - **longest-option** — 1.0 iff gold is the UNIQUE longest label (the
//!   classic multiple-choice heuristic; degenerate for intents, kept for
//!   the family's completeness);
//! - `trivial_margin = max` of the three.
//!
//! Report-first (T1): if the margin histograms show no too-easy pole, the
//! gate class is vacuous here and closes with that measurement. Gated
//! artifacts (T2) are DERIVED from the sealed artifact — rows verbatim,
//! fresh header (`accepted` = kept count, `learnability_rule` = the
//! derivation), fresh blake3 sidecar — and verified by re-loading through
//! the same sealed loader before any read can consume them. No teacher
//! calls, no engine changes.
//!
//! Quantile convention (the percentile-index law): `idx = floor(q·n)` on
//! the ascending-sorted margins, clamped; `tail_support = n − idx` is
//! reported beside every threshold (a tail under 10 is weak support).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;

use super::synth::{CORPUS_MAGIC, CORPUS_VERSION, SynthArtifactMeta, load_synth_corpus};
use crate::harness::suites::TrainDoc;

/// One row's trivial-solver margins (row order = artifact order).
#[derive(Debug, Clone, Serialize)]
pub struct MarginRow {
    pub idx: usize,
    pub label: String,
    pub prior: f64,
    pub lexical: f64,
    pub longest: f64,
    pub trivial: f64,
}

/// A histogram bucket over [0, 1] (the last bucket is closed).
#[derive(Debug, Clone, Serialize)]
pub struct MarginBucket {
    pub lo: f64,
    pub hi: f64,
    pub count: usize,
}

/// Per-solver margin distribution + the at-ceiling count.
#[derive(Debug, Clone, Serialize)]
pub struct SolverBlock {
    pub name: &'static str,
    pub at_ceiling: usize,
    pub histogram: Vec<MarginBucket>,
}

/// A quantile threshold with its index + tail support.
#[derive(Debug, Clone, Serialize)]
pub struct LearnabilityQuantile {
    pub q: f64,
    pub threshold: f64,
    pub idx: usize,
    pub tail_support: usize,
}

/// Report-only difficulty terciles over a rung's KEPT margins.
#[derive(Debug, Clone, Serialize)]
pub struct Tiers {
    pub easy: usize,
    pub medium: usize,
    pub hard: usize,
    pub t1: f64,
    pub t2: f64,
}

/// One gated rung: the filter summary + the derived artifact's seal.
#[derive(Debug, Clone, Serialize)]
pub struct RungSummary {
    pub name: String,
    pub q: f64,
    pub threshold: f64,
    pub tail_support: usize,
    pub kept: usize,
    pub dropped: usize,
    /// kept == 0 — the rung drops everything; no artifact written.
    pub starved: bool,
    /// dropped == 0 — the rung drops nothing; no artifact written.
    pub saturated: bool,
    pub artifact: Option<String>,
    pub blake3_16: Option<String>,
    pub tiers: Option<Tiers>,
    pub per_label_kept: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnabilityOutput {
    pub artifact: String,
    pub source_blake3_16: String,
    pub rows: usize,
    pub solvers: Vec<SolverBlock>,
    pub trivial_histogram: Vec<MarginBucket>,
    pub quantiles: Vec<LearnabilityQuantile>,
    pub rungs: Vec<RungSummary>,
    /// Full per-row margin table (the report-only sidecar a future lane
    /// can reuse without gating).
    pub rows_report: Vec<MarginRow>,
}

const SOLVER_FIELDS: [(&str, &str); 3] = [
    ("prior_frequency", "prior"),
    ("lexical_overlap", "lexical"),
    ("longest_option", "longest"),
];

/// Lowercase alphanumeric runs.
fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

fn label_tokens(label: &str) -> BTreeSet<String> {
    tokens(label).into_iter().collect()
}

fn histogram(values: impl Iterator<Item = f64>) -> Vec<MarginBucket> {
    let mut buckets: Vec<MarginBucket> = (0..10)
        .map(|i| MarginBucket {
            lo: i as f64 / 10.0,
            hi: (i + 1) as f64 / 10.0,
            count: 0,
        })
        .collect();
    for v in values {
        let v = v.clamp(0.0, 1.0);
        let idx = ((v * 10.0).floor() as usize).min(9);
        buckets[idx].count += 1;
    }
    buckets
}

/// Score every row with the three closed-form trivial-solver margins.
pub fn score_corpus(docs: &[TrainDoc]) -> Vec<MarginRow> {
    let n = docs.len();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut label_set: BTreeSet<&str> = BTreeSet::new();
    for d in docs {
        counts
            .entry(d.label.as_str())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        label_set.insert(d.label.as_str());
    }
    let max_prior = counts.values().copied().max().unwrap_or(1) as f64 / n.max(1) as f64;

    // Gold + other label token sets, one row of work per distinct label.
    let label_tks: BTreeMap<&str, BTreeSet<String>> =
        label_set.iter().map(|l| (*l, label_tokens(l))).collect();
    let max_len = label_set
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let longest_owners = label_set
        .iter()
        .filter(|l| l.chars().count() == max_len)
        .count();

    docs.iter()
        .enumerate()
        .map(|(idx, d)| {
            let prior = counts.get(d.label.as_str()).copied().unwrap_or(1) as f64 / n.max(1) as f64;
            let prior = if max_prior > 0.0 {
                prior / max_prior
            } else {
                0.0
            };

            let tks: BTreeSet<String> = tokens(&d.text).into_iter().collect();
            let cov = |l: &str| -> f64 {
                let lt = label_tks.get(l);
                match lt {
                    Some(lt) if lt.is_empty() => 0.0,
                    Some(lt) => {
                        lt.iter().filter(|t| tks.contains(*t)).count() as f64 / lt.len() as f64
                    }
                    None => 0.0,
                }
            };
            let cov_gold = cov(d.label.as_str());
            let best_other = label_set
                .iter()
                .filter(|l| **l != d.label.as_str())
                .map(|l| cov(l))
                .fold(0.0_f64, f64::max);
            let lexical = (cov_gold - best_other).clamp(0.0, 1.0);

            let longest = if longest_owners == 1 && d.label.chars().count() == max_len {
                1.0
            } else {
                0.0
            };
            MarginRow {
                idx,
                label: d.label.clone(),
                prior,
                lexical,
                longest,
                trivial: prior.max(lexical).max(longest),
            }
        })
        .collect()
}

/// The quantile convention: `idx = floor(q·n)` clamped onto the ascending
/// sorted slice; returns (idx, value, tail_support = n − idx).
pub fn margin_quantile(sorted: &[f64], q: f64) -> (usize, f64, usize) {
    let n = sorted.len();
    let idx = (((n as f64) * q).floor() as usize).min(n.saturating_sub(1));
    (idx, sorted[idx], n - idx)
}

fn rung_name(q: f64) -> String {
    format!("p{}", (q * 100.0).round() as usize)
}

fn terciles(kept_margins: &[f64]) -> Tiers {
    let mut s = kept_margins.to_vec();
    s.sort_by(f64::total_cmp);
    let (_, t1, _) = margin_quantile(&s, 1.0 / 3.0);
    let (_, t2, _) = margin_quantile(&s, 2.0 / 3.0);
    let easy = s.iter().filter(|&&v| v < t1).count();
    let medium = s.iter().filter(|&&v| v >= t1 && v < t2).count();
    let hard = s.len() - easy - medium;
    Tiers {
        easy,
        medium,
        hard,
        t1,
        t2,
    }
}

/// Run the T1 report + T2 gated-artifact derivation over one sealed synth
/// artifact. Every derived artifact is re-loaded through the sealed loader
/// before being reported, so a sidecar/header inconsistency dies HERE.
pub fn run_learnability_gate(
    artifact: &Path,
    out_dir: &Path,
    rungs: &[f64],
) -> Result<LearnabilityOutput, String> {
    if rungs.is_empty() {
        return Err("learnability gate: at least one rung is required".into());
    }
    for &q in rungs {
        if q <= 0.0 || q >= 1.0 {
            return Err(format!("learnability gate: rung {q} outside (0, 1)"));
        }
    }
    let (source_meta, docs, digest) = load_synth_corpus(artifact)?;
    if docs.is_empty() {
        return Err(format!(
            "{}: artifact carries no rows — nothing to score",
            artifact.display()
        ));
    }
    let text = std::fs::read_to_string(artifact)
        .map_err(|e| format!("read {}: {e}", artifact.display()))?;
    let margins = score_corpus(&docs);
    let mut sorted: Vec<f64> = margins.iter().map(|m| m.trivial).collect();
    sorted.sort_by(f64::total_cmp);

    let quantiles: Vec<LearnabilityQuantile> = rungs
        .iter()
        .map(|&q| {
            let (idx, threshold, tail_support) = margin_quantile(&sorted, q);
            LearnabilityQuantile {
                q,
                threshold,
                idx,
                tail_support,
            }
        })
        .collect();

    // The source header's per-label weights (carried into each derived
    // artifact's fresh per_label block).
    let header_line = text
        .lines()
        .next()
        .ok_or_else(|| format!("{}: empty artifact", artifact.display()))?;
    let source_header: serde_json::Value = serde_json::from_str(header_line)
        .map_err(|e| format!("{}: header parse: {e}", artifact.display()))?;
    let weights: BTreeMap<String, f64> = source_header
        .get("per_label")
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| {
                    let w = v.get("weight").and_then(serde_json::Value::as_f64)?;
                    Some((k.clone(), w))
                })
                .collect()
        })
        .unwrap_or_default();

    // Rows verbatim: the loader skips only blank lines, so doc i aligns
    // with non-empty source line i (line 0 is the header).
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();

    let mut rungs_out = Vec::with_capacity(rungs.len());
    for &q in rungs {
        let (_, threshold, tail) = margin_quantile(&sorted, q);
        let kept_idx: Vec<usize> = margins
            .iter()
            .filter(|m| m.trivial < threshold)
            .map(|m| m.idx)
            .collect();
        let dropped = margins.len() - kept_idx.len();
        let starved = kept_idx.is_empty();
        let saturated = dropped == 0;
        let name = rung_name(q);

        let mut per_label_kept: BTreeMap<String, usize> = BTreeMap::new();
        for &i in &kept_idx {
            *per_label_kept.entry(margins[i].label.clone()).or_insert(0) += 1;
        }
        let tiers = (!starved).then(|| {
            let kept_m: Vec<f64> = kept_idx.iter().map(|&i| margins[i].trivial).collect();
            terciles(&kept_m)
        });

        let (artifact_path, blake3_16) = if starved || saturated {
            eprintln!(
                "  [learnability {name}] kept {} dropped {} — no artifact written ({})",
                kept_idx.len(),
                dropped,
                if starved { "STARVED" } else { "SATURATED" },
            );
            (None, None)
        } else {
            let (p, seal) = write_gated_artifact(
                out_dir,
                &name,
                &source_meta,
                &weights,
                &lines,
                &kept_idx,
                &per_label_kept,
                q,
                threshold,
                &digest,
            )?;
            // Round-trip through the sealed loader — the derived artifact
            // must satisfy the SAME integrity bar as the source.
            let (lm, ld, _) = load_synth_corpus(&p)?;
            if ld.len() != kept_idx.len() || lm.accepted != kept_idx.len() {
                return Err(format!(
                    "{}: derived artifact carries {} rows, header/keep-set says {}",
                    p.display(),
                    ld.len(),
                    kept_idx.len()
                ));
            }
            eprintln!(
                "  [learnability {name}] wrote {} ({} kept / {dropped} dropped, threshold {threshold:.4})",
                p.display(),
                kept_idx.len(),
            );
            (Some(p.display().to_string()), Some(seal[..16].to_string()))
        };

        rungs_out.push(RungSummary {
            name,
            q,
            threshold,
            tail_support: tail,
            kept: kept_idx.len(),
            dropped,
            starved,
            saturated,
            artifact: artifact_path,
            blake3_16,
            tiers,
            per_label_kept,
        });
    }

    let solvers = SOLVER_FIELDS
        .into_iter()
        .map(|(name, field)| {
            let at_ceiling = margins
                .iter()
                .filter(|m| match field {
                    "prior" => m.prior >= 1.0 - 1e-9,
                    "lexical" => m.lexical >= 1.0 - 1e-9,
                    _ => m.longest >= 1.0 - 1e-9,
                })
                .count();
            let hist = histogram(margins.iter().map(|m| match field {
                "prior" => m.prior,
                "lexical" => m.lexical,
                _ => m.longest,
            }));
            SolverBlock {
                name,
                at_ceiling,
                histogram: hist,
            }
        })
        .collect::<Vec<_>>();

    Ok(LearnabilityOutput {
        artifact: artifact.display().to_string(),
        source_blake3_16: digest[..16].to_string(),
        rows: margins.len(),
        trivial_histogram: histogram(margins.iter().map(|m| m.trivial)),
        quantiles,
        rungs: rungs_out,
        rows_report: margins,
        solvers,
    })
}

#[allow(clippy::too_many_arguments)]
fn write_gated_artifact(
    out_dir: &Path,
    rung: &str,
    source_meta: &SynthArtifactMeta,
    weights: &BTreeMap<String, f64>,
    lines: &[&str],
    kept_idx: &[usize],
    per_label_kept: &BTreeMap<String, usize>,
    q: f64,
    threshold: f64,
    source_digest: &str,
) -> Result<(std::path::PathBuf, String), String> {
    let suite = source_meta.suite.clone();
    let rule = format!(
        "drop trivial_margin >= {threshold:.4} at q={q} (solvers: prior-frequency/\
         lexical-overlap/longest-option; source blake3 {}; kept {}/{}; this artifact is a \
         DERIVED filter of a sealed artifact, not a fresh synthesis run)",
        &source_digest[..16],
        kept_idx.len(),
        lines.len() - 1,
    );
    let meta = SynthArtifactMeta {
        magic: CORPUS_MAGIC.to_string(),
        version: CORPUS_VERSION,
        suite: suite.clone(),
        teacher: source_meta.teacher.clone(),
        teacher_provenance: source_meta.teacher_provenance.clone(),
        weighting_rule: source_meta.weighting_rule.clone(),
        created_utc: super::iso8601_utc(),
        git_sha: super::git_sha().unwrap_or_else(|| "unknown".into()),
        host: super::hostname_refusing_unknown(),
        accepted: kept_idx.len(),
        pool_rows: source_meta.pool_rows,
        density_rule: source_meta.density_rule.clone(),
        rescue_rule: source_meta.rescue_rule.clone(),
        learnability_rule: Some(rule),
    };
    let mut header = serde_json::to_value(&meta).map_err(|e| e.to_string())?;
    let per_label: serde_json::Map<String, serde_json::Value> = per_label_kept
        .iter()
        .map(|(label, kept)| {
            (
                label.clone(),
                serde_json::json!({
                    "candidates": kept,
                    "allocated": kept,
                    "accepted": kept,
                    "veto_rejected": 0,
                    "density_rejected": 0,
                    "rescue_rejected": 0,
                    "weight": weights.get(label).copied().unwrap_or(0.0),
                }),
            )
        })
        .collect();
    header["per_label"] = serde_json::Value::Object(per_label);

    let mut bytes: Vec<u8> = Vec::with_capacity(1 << 16);
    bytes.extend_from_slice(
        serde_json::to_string(&header)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    );
    bytes.push(b'\n');
    for &i in kept_idx {
        bytes.extend_from_slice(lines[i + 1].as_bytes());
        bytes.push(b'\n');
    }
    std::fs::create_dir_all(out_dir).map_err(|e| format!("create {}: {e}", out_dir.display()))?;
    let dir = out_dir.join(rung);
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let path = dir.join(format!("{suite}_synth.jsonl"));
    std::fs::write(&path, &bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    let seal = blake3::hash(&bytes).to_hex().to_string();
    let sidecar = dir.join(format!("{suite}_synth.jsonl.blake3"));
    std::fs::write(&sidecar, format!("{seal}\n"))
        .map_err(|e| format!("write {}: {e}", sidecar.display()))?;
    Ok((path, seal))
}

/// The T1/T2 report markdown (the example writes LEARNABILITY.md).
pub fn render_learnability_markdown(out: &LearnabilityOutput) -> String {
    let mut s = String::new();
    s.push_str("# learnability gate report (Issue 104 — trivial-solver margins)\n\n");
    s.push_str(&format!(
        "- artifact {} (blake3 {}…) · {} rows\n- solvers: prior-frequency · lexical-overlap · \
         longest-option; trivial_margin = max of the three\n\n",
        out.artifact, out.source_blake3_16, out.rows
    ));

    s.push_str("## At-ceiling counts (rows the solver aces outright)\n\n");
    s.push_str("| solver | at ceiling | share |\n|---|---|---|\n");
    for sv in &out.solvers {
        s.push_str(&format!(
            "| {} | {} | {:.4} |\n",
            sv.name,
            sv.at_ceiling,
            sv.at_ceiling as f64 / out.rows.max(1) as f64
        ));
    }

    s.push_str("\n## Quantile thresholds (trivial_margin)\n\n");
    s.push_str("| q | threshold | idx | tail support |\n|---|---|---|---|\n");
    for qt in &out.quantiles {
        s.push_str(&format!(
            "| {:.2} | {:.4} | {} | {} |\n",
            qt.q, qt.threshold, qt.idx, qt.tail_support
        ));
    }

    s.push_str("\n## Gated rungs\n\n");
    s.push_str(
        "| rung | threshold | kept | dropped | verdict | artifact |\n|---|---|---|---|---|---|\n",
    );
    for r in &out.rungs {
        let verdict = if r.starved {
            "STARVED (kept 0)"
        } else if r.saturated {
            "SATURATED (dropped 0)"
        } else {
            "derived"
        };
        let art = r.artifact.as_deref().unwrap_or("-");
        s.push_str(&format!(
            "| {} | {:.4} | {} | {} | {} | {} |\n",
            r.name, r.threshold, r.kept, r.dropped, verdict, art
        ));
    }

    s.push_str("\n## Trivial-margin histogram (10 buckets of 0.1)\n\n");
    s.push_str("| bucket | count |\n|---|---|\n");
    for b in &out.trivial_histogram {
        s.push_str(&format!(
            "| {:.1}–{:.1}{} | {} |\n",
            b.lo,
            b.hi,
            if (b.hi - 1.0).abs() < 1e-9 { "]" } else { ")" },
            b.count
        ));
    }

    s.push_str("\n## Per-solver histograms\n\n");
    for sv in &out.solvers {
        s.push_str(&format!(
            "\n### {}\n\n| bucket | count |\n|---|---|\n",
            sv.name
        ));
        for b in &sv.histogram {
            s.push_str(&format!(
                "| {:.1}–{:.1}{} | {} |\n",
                b.lo,
                b.hi,
                if (b.hi - 1.0).abs() < 1e-9 { "]" } else { ")" },
                b.count
            ));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::TrainDoc;

    fn fixture() -> Vec<TrainDoc> {
        vec![
            TrainDoc {
                label: "alarm_set".into(),
                text: "set an alarm for eight".into(),
            },
            TrainDoc {
                label: "alarm_query".into(),
                text: "when is my alarm".into(),
            },
            TrainDoc {
                label: "weather_query".into(),
                text: "is it raining outside".into(),
            },
            TrainDoc {
                label: "weather_query".into(),
                text: "weather today".into(),
            },
        ]
    }

    fn doc(label: &str, text: &str) -> TrainDoc {
        TrainDoc {
            label: label.into(),
            text: text.into(),
        }
    }

    #[test]
    fn margins_match_the_hand_computation() {
        let m = score_corpus(&fixture());
        assert_eq!(m.len(), 4);
        // Row 0: prior 1/4 ÷ 1/2 = 0.5; "alarm"+"set" both in text → gold cov
        // 1.0, best other (alarm_query) 0.5 → lexical 0.5; not the longest.
        assert!((m[0].prior - 0.5).abs() < 1e-9);
        assert!((m[0].lexical - 0.5).abs() < 1e-9);
        assert_eq!(m[0].longest, 0.0);
        assert!((m[0].trivial - 0.5).abs() < 1e-9);
        // Row 1: lexical ties (both alarm_* labels cover 0.5) → 0; prior 0.5.
        assert_eq!(m[1].lexical, 0.0);
        assert!((m[1].trivial - 0.5).abs() < 1e-9);
        // Row 2: majority label (prior 1.0) AND the unique longest label.
        assert!((m[2].prior - 1.0).abs() < 1e-9);
        assert_eq!(m[2].longest, 1.0);
        assert!((m[2].trivial - 1.0).abs() < 1e-9);
        // Row 3: lexical separable ("weather" names it) + majority + longest.
        assert!((m[3].lexical - 0.5).abs() < 1e-9);
        assert!((m[3].trivial - 1.0).abs() < 1e-9);
    }

    #[test]
    fn quantile_convention_reports_tail_support() {
        let s = vec![1.0, 2.0, 3.0, 4.0];
        let (idx, v, tail) = margin_quantile(&s, 0.5);
        assert_eq!((idx, v, tail), (2, 3.0, 2));
        let (_, _, tail90) = margin_quantile(&s, 0.9);
        assert_eq!(tail90, 1); // disclosed, never hidden
        let (idx1, v1, _) = margin_quantile(&s, 0.25);
        assert_eq!((idx1, v1), (1, 2.0));
    }

    #[test]
    fn keep_sets_shrink_monotonically_as_q_rises() {
        let docs = fixture();
        let m = score_corpus(&docs);
        let mut sorted: Vec<f64> = m.iter().map(|x| x.trivial).collect();
        sorted.sort_by(f64::total_cmp);
        let mut prev: Option<Vec<usize>> = None;
        for q in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let (_, threshold, _) = margin_quantile(&sorted, q);
            let keep: Vec<usize> = m
                .iter()
                .filter(|x| x.trivial < threshold)
                .map(|x| x.idx)
                .collect();
            if let Some(p) = &prev {
                assert!(p.iter().all(|i| keep.contains(i)), "q={q} lost rows");
            }
            prev = Some(keep);
        }
    }

    fn write_source_artifact(dir: &Path) -> std::path::PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let rows = [
            ("set an alarm for eight", "alarm_set"),
            ("when is my alarm", "alarm_query"),
            ("is it raining outside", "weather_query"),
            ("weather today", "weather_query"),
        ];
        let header = serde_json::json!({
            "magic": "SYNT", "version": 2, "suite": "fixture",
            "teacher": "t", "teacher_provenance": "t:t",
            "weighting_rule": "uniform", "created_utc": "2026-01-01T00:00:00Z",
            "git_sha": "test", "host": "test", "accepted": rows.len(),
            "pool_rows": 4,
            "per_label": {
                "alarm_set": {"weight": 1.0},
                "alarm_query": {"weight": 0.5},
                "weather_query": {"weight": 0.25}
            }
        });
        let mut bytes = serde_json::to_string(&header).unwrap().into_bytes();
        bytes.push(b'\n');
        for (text, label) in rows {
            bytes.extend_from_slice(
                format!(
                    "{{\"text\":\"{text}\",\"label\":\"{label}\",\"provenance\":\"synth\",\
                     \"src\":0,\"span\":[0,1]}}\n"
                )
                .as_bytes(),
            );
        }
        let path = dir.join("fixture_synth.jsonl");
        std::fs::write(&path, &bytes).unwrap();
        std::fs::write(
            dir.join("fixture_synth.jsonl.blake3"),
            format!("{}\n", blake3::hash(&bytes).to_hex()),
        )
        .unwrap();
        path
    }

    #[test]
    fn derived_artifact_roundtrips_through_the_sealed_loader() {
        let base = std::env::temp_dir().join(format!("learn104_{}", std::process::id()));
        let src = write_source_artifact(&base);
        let out = base.join("derived");
        let out_json = run_learnability_gate(&src, &out, &[0.5]).expect("gate run");
        let rung = out_json
            .rungs
            .iter()
            .find(|r| r.name == "p50")
            .expect("p50 rung");
        let path = rung.artifact.as_ref().expect("p50 writes an artifact");
        let (meta, docs, _) = load_synth_corpus(Path::new(path)).expect("round-trip");
        // Fixture: margins {0.5, 0.5, 1.0, 1.0} → p50 threshold 1.0 keeps the
        // two 0.5 rows, drops the two too-easy 1.0 rows.
        assert_eq!(docs.len(), 2);
        assert_eq!(meta.accepted, 2);
        assert!(meta.learnability_rule.as_deref().unwrap().contains("q=0.5"));
        assert_eq!(docs[0].label, "alarm_set");
        assert_eq!(docs[1].label, "alarm_query");
        // The report carries the full margin table + at-ceiling counts.
        assert_eq!(out_json.rows_report.len(), 4);
        let prior = out_json
            .solvers
            .iter()
            .find(|s| s.name == "prior_frequency")
            .unwrap();
        assert_eq!(prior.at_ceiling, 2);
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn an_all_too_easy_artifact_starves_every_rung() {
        let base = std::env::temp_dir().join(format!("learn104_starve_{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let rows = [
            ("weather today", "weather_query"),
            ("weather tomorrow", "weather_query"),
            ("weather now", "weather_query"),
        ];
        let header = serde_json::json!({
            "magic": "SYNT", "version": 2, "suite": "starve",
            "teacher": "t", "teacher_provenance": "t:t",
            "weighting_rule": "uniform", "created_utc": "x",
            "git_sha": "test", "host": "test", "accepted": rows.len(),
            "pool_rows": 3
        });
        let mut bytes = serde_json::to_string(&header).unwrap().into_bytes();
        bytes.push(b'\n');
        for (text, label) in rows {
            bytes.extend_from_slice(
                format!(
                    "{{\"text\":\"{text}\",\"label\":\"{label}\",\"provenance\":\"synth\",\
                     \"src\":0,\"span\":[0,1]}}\n"
                )
                .as_bytes(),
            );
        }
        let src = base.join("starve_synth.jsonl");
        std::fs::write(&src, &bytes).unwrap();
        std::fs::write(
            base.join("starve_synth.jsonl.blake3"),
            format!("{}\n", blake3::hash(&bytes).to_hex()),
        )
        .unwrap();
        // Every row: prior 1.0 (sole label) → trivial 1.0 → p50 keeps nothing.
        let out = run_learnability_gate(&src, &base.join("d"), &[0.5]).expect("gate run");
        let rung = &out.rungs[0];
        assert!(rung.starved);
        assert_eq!(rung.kept, 0);
        assert!(rung.artifact.is_none());
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn rungs_outside_the_unit_interval_refuse() {
        let base = std::env::temp_dir().join(format!("learn104_bad_{}", std::process::id()));
        let src = write_source_artifact(&base);
        let err = run_learnability_gate(&src, &base.join("d"), &[0.0]).unwrap_err();
        assert!(err.contains("outside (0, 1)"));
        let err = run_learnability_gate(&src, &base.join("d"), &[]).unwrap_err();
        assert!(err.contains("at least one rung"));
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn tiers_partition_the_kept_rows() {
        let t = terciles(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]);
        assert_eq!(t.easy + t.medium + t.hard, 9);
        assert!(t.t1 <= t.t2);
        assert!(t.easy > 0 && t.hard > 0);
    }

    #[test]
    fn single_label_corpus_scores_full_prior_and_no_lexical() {
        let docs = vec![doc("a", "x y"), doc("a", "z w")];
        let m = score_corpus(&docs);
        assert!((m[0].prior - 1.0).abs() < 1e-9);
        assert_eq!(m[0].lexical, 0.0); // no other label to beat
        assert!((m[0].trivial - 1.0).abs() < 1e-9);
    }

    #[test]
    fn label_tie_for_longest_gives_no_option_signal() {
        let docs = vec![
            doc("aa_bb", "aa here"),  // cov gold 0.5, best other 0.5 → 0
            doc("bb_aa", "aa there"), // same length as aa_bb → longest tie
            doc("aa_bb", "bb now"),
        ];
        let m = score_corpus(&docs);
        assert_eq!(m.iter().map(|x| x.longest).sum::<f64>(), 0.0);
    }
}
