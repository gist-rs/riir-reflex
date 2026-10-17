//! Issue 093 — the `CorpusDistanceGate` abstention-drift diagnostic (the
//! capacity-law measurement lane; REPORT-ONLY runner half).
//!
//! The gate compares the max cosine similarity of a query against its
//! routed domain's corpus rows with a FIXED absolute midpoint
//! ([`crate::engine::GATE_MID`] = 0.35). The issue's drift carrier is the
//! OFF-corpus population: an unrelated query gets M chances at an
//! accidental near-neighbour, so its max-similarity creeps UP toward the
//! midpoint as the corpus grows — past the midpoint the gate stops
//! abstaining on exactly the inputs it exists to reject (false-accept
//! drift toward never-abstain). The in-corpus population moves the OTHER
//! way (adding rows can only raise a corpus query's max), so the two
//! sides of the birth desert close from opposite ends.
//!
//! The pre-registered instrument (issue T1/T1a, recorded here so the
//! numbers carry their own construction):
//!
//! - **Corpus sizes 1× / 2× / 4×** by SUITE CONCATENATION over
//!   `[ag_news, emotion, sst5, xnli_en]` (disjoint label vocabularies —
//!   concatenation grows the engine's domain count 4 → 10 → 18 and the
//!   total corpus rows 256 → 640 → 1152 at the registry cap 64/label).
//!   The engine per size is the deployed build path
//!   ([`build_engine_with`], `EngineConfig::default()`).
//! - **Off-corpus probes** drawn from sources HELD OUT FROM EVERY family
//!   suite at EVERY size (T1a-i): `[banking77, massive_intent_en]` test
//!   rows — none of them ever joins any corpus at any size, so the two
//!   populations never mix (the issue's round-3 condition, recorded
//!   verbatim in the issue).
//! - **In-corpus probes**: each family suite's stratified eval front at
//!   the registry test cap — corpus-distribution queries, never scoring
//!   against their own rows (test rows, corpora are train rows).
//! - **Two read surfaces, never pooled:**
//!   - *corpus-shaped* (both populations): the probe TEXT embedded ALONE
//!     (the exact shape a corpus row is embedded at build), routed via
//!     [`DecisionEngine::semantic_route`] (the shipped routing read) —
//!     the routed domain's gate max + the WHOLE-corpus max (max over all
//!     N domains' gates — the issue's M-chances statistic, bit-identical
//!     rows to the shipped gates by construction).
//!   - *engine-shaped* (in-corpus only; off-corpus probes have no honest
//!     request shape and are NOT wrapped synthetically): real eval
//!     requests through `solve_into`, the routed domain's gate on the
//!     CONTEXT embedding `sc.q` — what the deployed fused gate actually
//!     consumes.
//! - **Both postures, separately** (the issue's confounder law): (i) the
//!   FIXED birth boundary — `max_sim < GATE_MID` (exactly
//!   `gate_conf < 0.5` at [`GATE_SCALE`]; the birth-constant fallback
//!   posture); (ii) the RE-FIT posture — the ρ=30 percentile distance
//!   threshold fitted on the concatenated engine's pooled cal slice (the
//!   T1.6 arena fit, re-derived at each size — absorbs drift BY
//!   CONSTRUCTION, so the two are never pooled). A third, disclosed
//!   ADDITION: the size-1 fitted threshold HELD while the corpus grows
//!   (the aging-deployment posture — fit once at birth, corpus grows
//!   later).
//! - **Tail support disclosed** (T1a-ii): every quantile row carries its
//!   rank and sample support (`n − rank`); the max is reported
//!   separately, never laundered through a percentile label (the repo's
//!   p99-at-small-n trap).
//! - **Null overlay**: the labeled isotropic reference
//!   `√(2·ln M / d_k)` at M = total corpus rows, d_k = 256. A mismatch
//!   rules out the NULL, never the drift (the measured claim rests on
//!   the off-corpus tail itself — the issue's null-model law; hashed-bag
//!   embeddings are clustered and anisotropic).
//!
//! Verdict inputs are COMPUTED, never gated: the probe is report-only
//! (the issue's own law — no lever lands without a measured curve, and
//! T3/T4 are the conditional follow-ups). T2's close condition (off-corpus
//! upper tail clear of the midpoint at 4× in BOTH postures) is evaluated
//! on the 4× cell and printed for the bench record to cite. The gate
//! constants are load-bearing at birth — nothing here edits them.

use serde::Serialize;

use super::perm_probe::stride_indices;
use super::*;
use crate::embed::{EMBED_DIM, Embedder};
use crate::engine::{EngineConfig, GATE_MID, GATE_SCALE, threshold_recommendation};

/// The concatenation family, in order. Sizes 1×/2×/4× take prefixes of
/// length 1/2/4. All four are default-run dataset suites with disjoint
/// label vocabularies and registry corpus caps of 64/label (asserted at
/// load — a registry cap change breaks the uniform-cap premise and the
/// probe REFUSES rather than silently mis-scale).
pub const GATE_DRIFT_FAMILY: &[&str] = &["ag_news", "emotion", "sst5", "xnli_en"];

/// Off-corpus probe sources: test rows of suites that never join any
/// corpus at any size (T1a-i's held-out law).
pub const GATE_DRIFT_OFF_SOURCES: &[&str] = &["banking77", "massive_intent_en"];

/// The uniform per-label corpus cap (the four family suites' registry
/// caps — asserted, never assumed).
const FAMILY_CAP_PER_LABEL: usize = 64;

/// Default off-corpus probes per source (stride-sampled over the full
/// test pool).
pub const DEFAULT_MAX_OFF_PER_SOURCE: usize = 200;

/// Default in-corpus probes per family suite per surface (stride-sampled
/// over the stratified eval front).
pub const DEFAULT_MAX_IN_PER_SUITE: usize = 100;

/// The probe's knobs.
#[derive(Debug, Clone, Copy)]
pub struct GateDriftOptions {
    pub max_off_per_source: usize,
    pub max_in_per_suite: usize,
}

// ───────────────────────────────────────────── records ──

/// One quantile read with its tail support (T1a-ii): `rank` is the index
/// the percentile law lands on over the sorted sample; `support` is how
/// many observations sit at or above it (`n − rank`) — under 10 is weak
/// and the record says so.
#[derive(Debug, Clone, Serialize)]
pub struct QuantileRow {
    pub label: &'static str,
    pub value: f32,
    pub rank: usize,
    pub support: usize,
}

/// One population's max-similarity distribution (one surface).
#[derive(Debug, Clone, Serialize)]
pub struct DistSummary {
    pub n: usize,
    /// Probes dropped for empty/whitespace text (a zero vector reads as
    /// mid-corpus distance — never measured, always counted).
    pub dropped_empty: usize,
    pub min: f32,
    pub max: f32,
    pub p05: QuantileRow,
    pub p50: QuantileRow,
    pub p90: QuantileRow,
    pub p99: QuantileRow,
}

/// Abstain decisions for one population under one posture.
#[derive(Debug, Clone, Serialize)]
pub struct AbstainSplit {
    pub n: usize,
    pub abstains: usize,
    pub rate: f64,
    /// The threshold applied (`GATE_MID` spelled out for the fixed
    /// posture — its confidence boundary IS the midpoint at any scale).
    pub threshold: f32,
}

/// One population read on the corpus-shaped surface (text embedded
/// alone): whole-corpus max + routed-gate max + the abstain splits.
#[derive(Debug, Clone, Serialize)]
pub struct CorpusShapedRead {
    /// Max over ALL N domains' gates (the issue's M-chances statistic).
    pub whole: DistSummary,
    /// `semantic_route` → the routed domain's gate max (the shipped
    /// routing read on the corpus-shaped embedding).
    pub routed: DistSummary,
    pub abstain_fixed: AbstainSplit,
    pub abstain_refit: AbstainSplit,
    /// The size-1 fitted threshold held at this size (the aging-deployment
    /// posture; identical to `abstain_refit` on the 1× cell).
    pub abstain_held1x: AbstainSplit,
}

/// The engine-shaped in-corpus read (real requests, context embedding).
#[derive(Debug, Clone, Serialize)]
pub struct EngineShapedRead {
    pub n: usize,
    pub max_sim: DistSummary,
    pub conf: DistSummary,
    pub abstain_fixed: AbstainSplit,
    pub abstain_refit: AbstainSplit,
    pub abstain_held1x: AbstainSplit,
}

/// One corpus size's full cell.
#[derive(Debug, Clone, Serialize)]
pub struct GateDriftSizeCell {
    pub size_label: String,
    pub suites: Vec<String>,
    pub n_domains: usize,
    /// Total corpus rows across all domains (the M of the null overlay).
    pub corpus_rows: usize,
    pub cap_per_label: usize,
    /// Per-domain gate row counts, domain order.
    pub per_domain_rows: Vec<usize>,
    /// The isotropic null `√(2·ln M / d_k)` at M = `corpus_rows`.
    pub null_reference: f64,
    pub off_corpus: CorpusShapedRead,
    /// Per-source off-corpus counts, source order.
    pub off_per_source: Vec<(String, usize)>,
    pub in_corpus: CorpusShapedRead,
    pub in_per_suite: Vec<(String, usize)>,
    pub in_engine_shaped: EngineShapedRead,
    /// The ρ=30 distance threshold fitted at THIS size (None = thin
    /// support — the birth constant is the fallback posture).
    pub refit_threshold: Option<f32>,
    pub refit_cal_observations: usize,
    /// The desert on the corpus-shaped routed read: positive
    /// `clearance_off` = the off-corpus p99 still below the midpoint;
    /// positive `in_floor_above_mid` = the in-corpus p05 still above it.
    pub clearance_off: f64,
    pub in_floor_above_mid: f64,
    /// T2's close-condition inputs at THIS size (the record cites 4×):
    /// off-corpus upper tail (p99 AND max) clear of the midpoint on the
    /// whole-corpus and routed reads.
    pub off_p99_clear_fixed: bool,
    pub off_max_clear_fixed: bool,
    pub off_p99_clear_routed: bool,
    /// The re-fit posture's false-accept rate on the off-corpus probes
    /// (1 − abstain rate) — the drift carrier under the absorbing fit.
    pub refit_off_false_accept_rate: f64,
    /// The fixed posture's false-abstain rate on the in-corpus probes.
    pub fixed_in_false_abstain_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateDriftMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub family: Vec<String>,
    pub off_sources: Vec<String>,
    pub cap_per_label: usize,
    pub gate_mid: f32,
    pub gate_scale: f32,
    pub max_off_per_source: usize,
    pub max_in_per_suite: usize,
    /// T1a-i disclosure: how the off-corpus probe set was built.
    pub off_probe_construction: String,
    /// The issue's confounder law, restated for the record.
    pub posture_note: String,
    pub box_start: crate::harness::box_state::BoxState,
    pub box_end: crate::harness::box_state::BoxState,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateDriftOutput {
    pub meta: GateDriftMeta,
    pub cells: Vec<GateDriftSizeCell>,
}

// ───────────────────────────────────────────── pure helpers ──

/// The isotropic-null reference for the off-corpus max statistic:
/// `√(2·ln M / d_k)` — the expected max cosine of a random unit vector
/// against M unrelated unit keys. A labeled REFERENCE, never a law (the
/// issue's null-model caveat).
#[must_use]
pub fn isotropic_null_reference(m: usize, d_k: usize) -> f64 {
    if m == 0 || d_k == 0 {
        return 0.0;
    }
    (2.0 * (m as f64).ln() / d_k as f64).sqrt()
}

/// Sort + index the sample once; every quantile reads the sorted copy.
struct Sorted(f32);
fn sorted_sample(values: &[f32]) -> Vec<Sorted> {
    let mut v: Vec<Sorted> = values.iter().map(|&x| Sorted(x)).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v
}

/// The percentile row under the repo's percentile law
/// (`idx = ⌊n·p⌋`, clamped to n−1) WITH its tail support.
#[must_use]
fn quantile_row(sorted: &[Sorted], p: f64, label: &'static str) -> QuantileRow {
    let n = sorted.len();
    let idx = ((n as f64) * p) as usize;
    let rank = idx.min(n.saturating_sub(1));
    QuantileRow {
        label,
        value: sorted[rank].0,
        rank,
        support: n - rank,
    }
}

#[must_use]
fn dist_summary(values: &[f32], dropped_empty: usize) -> DistSummary {
    let v = sorted_sample(values);
    let n = v.len();
    let min = v.first().map(|s| s.0).unwrap_or(f32::NAN);
    let max = v.last().map(|s| s.0).unwrap_or(f32::NAN);
    DistSummary {
        n,
        dropped_empty,
        min,
        max,
        p05: quantile_row(&v, 0.05, "p05"),
        p50: quantile_row(&v, 0.50, "p50"),
        p90: quantile_row(&v, 0.90, "p90"),
        p99: quantile_row(&v, 0.99, "p99"),
    }
}

/// The fixed posture's boundary IS the midpoint: at any scale,
/// `sigmoid(scale·(x − mid)) < 0.5 ⇔ x < mid`.
#[must_use]
fn abstain_split(values: &[f32], boundary: f32) -> AbstainSplit {
    let abstains = values.iter().filter(|&&x| x < boundary).count();
    let n = values.len();
    AbstainSplit {
        n,
        abstains,
        rate: if n == 0 {
            0.0
        } else {
            abstains as f64 / n as f64
        },
        threshold: boundary,
    }
}

/// The refit/held postures' boundary on the confidence axis:
/// `sigmoid(GATE_SCALE·(max_sim − GATE_MID)) < threshold`.
#[must_use]
fn abstain_split_conf(values: &[f32], threshold: f32) -> AbstainSplit {
    let abstains = values
        .iter()
        .filter(|&&x| crate::engine::gate_confidence(x) < threshold)
        .count();
    let n = values.len();
    AbstainSplit {
        n,
        abstains,
        rate: if n == 0 {
            0.0
        } else {
            abstains as f64 / n as f64
        },
        threshold,
    }
}

/// Per-suite label vocabulary in family order (the engine-domain order).
fn family_labels(take: usize) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    for name in &GATE_DRIFT_FAMILY[..take] {
        let keys: &[&str] = match *name {
            "ag_news" => &crate::harness::suites::AG_NEWS_KEYS,
            "emotion" => &crate::harness::suites::EMOTION_KEYS,
            "sst5" => &["0", "1", "2", "3", "4"],
            "xnli_en" => &crate::harness::suites::XNLI_KEYS,
            other => unreachable!("gate-drift family registry drifted: {other}"),
        };
        labels.extend(keys.iter().map(|k| (*k).to_string()));
    }
    labels
}

/// The disjointness premise: no label may appear twice across the family
/// (a collision would merge two corpora into one domain and break every
/// per-domain read).
#[must_use]
pub fn labels_disjoint(labels: &[String]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    labels.iter().all(|l| seen.insert(l.as_str()))
}

/// One probe's gate reads on the corpus-shaped surface.
struct CorpusShapedProbe {
    whole_max: f32,
    routed_max: f32,
    routed_conf: f32,
}

fn read_corpus_shaped<const N: usize>(
    engine: &crate::engine::DecisionEngine<N, EMBED_DIM>,
    text: &str,
    q: &mut [f32; EMBED_DIM],
) -> CorpusShapedProbe {
    Embedder.embed_into(text.as_bytes(), q);
    let mut whole = -1.0f32;
    for d in 0..N {
        whole = whole.max(engine.gate(d).max_similarity(q));
    }
    let route = engine.semantic_route(text);
    let routed = engine.gate(route);
    CorpusShapedProbe {
        whole_max: whole,
        routed_max: routed.max_similarity(q),
        routed_conf: routed.abstain_confidence(q),
    }
}

fn corpus_shaped_read(
    probes: &[(Vec<CorpusShapedProbe>, usize)],
    refit_threshold: Option<f32>,
    held_threshold: Option<f32>,
) -> (CorpusShapedRead, Vec<(String, usize)>) {
    let mut whole: Vec<f32> = Vec::new();
    let mut routed: Vec<f32> = Vec::new();
    let mut conf: Vec<f32> = Vec::new();
    let mut per_group: Vec<(String, usize)> = Vec::new();
    for (group, (probes, _)) in probes.iter().enumerate() {
        per_group.push((format!("group{group}"), probes.len()));
        for p in probes {
            whole.push(p.whole_max);
            routed.push(p.routed_max);
            conf.push(p.routed_conf);
        }
    }
    let dropped = dropped_total(probes);
    let fixed = abstain_split(&routed, GATE_MID);
    let refit = refit_threshold
        .map(|t| abstain_split_conf(&conf, t))
        .unwrap_or_else(|| fixed.clone());
    let held = held_threshold
        .map(|t| abstain_split_conf(&conf, t))
        .unwrap_or_else(|| refit.clone());
    (
        CorpusShapedRead {
            whole: dist_summary(&whole, dropped),
            routed: dist_summary(&routed, dropped),
            abstain_fixed: fixed,
            abstain_refit: refit,
            abstain_held1x: held,
        },
        per_group,
    )
}

fn dropped_total(probes: &[(Vec<CorpusShapedProbe>, usize)]) -> usize {
    probes.iter().map(|(_, d)| *d).sum()
}

// ───────────────────────────────────────────── per-size cell ──

#[allow(clippy::too_many_lines)]
fn run_size<const N: usize>(
    take: usize,
    size_label: &str,
    arts: &[SuiteArtefacts],
    off_groups: &[(String, Vec<String>, usize)],
    _gopts: GateDriftOptions,
    held_threshold: Option<f32>,
) -> Result<GateDriftSizeCell, String> {
    let family = &arts[..take];
    let labels = family_labels(take);
    assert_eq!(labels.len(), N, "gate-drift: domain-count premise broke");
    assert!(
        labels_disjoint(&labels),
        "gate-drift: family labels collided"
    );

    // ── the concatenated engine (the deployed build path) ──
    let mut train: Vec<TrainDoc> = Vec::new();
    for a in family {
        train.extend(a.corpus_pool.iter().cloned());
    }
    let (mut engine, fallbacks) = build_engine_with::<N>(
        "gate_drift",
        &train,
        &labels,
        FAMILY_CAP_PER_LABEL,
        EngineConfig::default(),
        &[],
    )?;
    if !fallbacks.is_empty() {
        eprintln!(
            "  [gate-drift] issue-039 self-doc fallback for {} starved label(s) — disclosed",
            fallbacks.len()
        );
    }
    let per_domain_rows: Vec<usize> = (0..N).map(|d| engine.gate(d).len()).collect();
    let corpus_rows: usize = per_domain_rows.iter().sum();
    let null_reference = isotropic_null_reference(corpus_rows, EMBED_DIM);

    // ── the ρ=30 refit on the pooled cal slice (the concatenated engine) ──
    let mut distance_obs: Vec<GateObservation> = Vec::new();
    {
        let mut sc = Scratch::<EMBED_DIM>::new();
        for a in family {
            for (case, state) in a.cal_cases.iter().zip(&a.cal_state_strs) {
                let req = engine_request(case, state).map_err(|e| {
                    format!("gate-drift cal request ({}, case {}): {e}", a.name, case.id)
                })?;
                sc.prepare(case.questions.len());
                engine.solve_into(&req, &mut sc).map_err(|e| {
                    format!("gate-drift cal solve ({}, case {}): {e}", a.name, case.id)
                })?;
                let (Some(slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) else {
                    continue;
                };
                let correct = case
                    .gold
                    .last()
                    .is_some_and(|g| g.idx == slot.pick as usize);
                distance_obs.push(GateObservation {
                    score: engine.gate(*dom).abstain_confidence(&sc.q),
                    correct,
                });
            }
        }
    }
    let refit = threshold_recommendation(&distance_obs, Posture::Percentile { rho: 0.30 });
    let refit_threshold = refit.map(|r| r.threshold);
    let refit_cal_observations = distance_obs.len();

    // ── off-corpus probes (corpus-shaped surface) ──
    let mut q = [0.0f32; EMBED_DIM];
    let mut off_groups_read: Vec<(Vec<CorpusShapedProbe>, usize)> = Vec::new();
    let mut off_counts: Vec<(String, usize)> = Vec::new();
    for (source, texts, dropped) in off_groups {
        let mut reads = Vec::with_capacity(texts.len());
        for t in texts {
            reads.push(read_corpus_shaped(&engine, t, &mut q));
        }
        off_counts.push((source.clone(), reads.len()));
        off_groups_read.push((reads, *dropped));
    }
    let (off_read, _) = corpus_shaped_read(&off_groups_read, refit_threshold, held_threshold);

    // ── in-corpus probes, corpus-shaped surface ──
    let mut in_groups_read: Vec<(Vec<CorpusShapedProbe>, usize)> = Vec::new();
    let mut in_counts: Vec<(String, usize)> = Vec::new();
    for a in family {
        let mut reads = Vec::with_capacity(a.a_texts.len());
        for t in &a.a_texts {
            reads.push(read_corpus_shaped(&engine, t, &mut q));
        }
        in_counts.push((a.name.clone(), reads.len()));
        in_groups_read.push((reads, a.a_dropped));
    }
    let (in_read, _) = corpus_shaped_read(&in_groups_read, refit_threshold, held_threshold);

    // ── in-corpus probes, engine-shaped surface (real requests) ──
    let mut b_max: Vec<f32> = Vec::new();
    let mut b_conf: Vec<f32> = Vec::new();
    let mut b_n = 0usize;
    {
        let mut sc = Scratch::<EMBED_DIM>::new();
        for a in family {
            for (case, state) in a.b_cases.iter().zip(&a.b_state_strs) {
                let req = engine_request(case, state).map_err(|e| {
                    format!(
                        "gate-drift eval request ({}, case {}): {e}",
                        a.name, case.id
                    )
                })?;
                sc.prepare(case.questions.len());
                engine.solve_into(&req, &mut sc).map_err(|e| {
                    format!("gate-drift eval solve ({}, case {}): {e}", a.name, case.id)
                })?;
                let (Some(_slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) else {
                    continue;
                };
                b_max.push(engine.gate(*dom).max_similarity(&sc.q));
                b_conf.push(engine.gate(*dom).abstain_confidence(&sc.q));
                b_n += 1;
            }
        }
    }
    let fixed_b = abstain_split(&b_max, GATE_MID);
    let refit_b = refit_threshold
        .map(|t| abstain_split_conf(&b_conf, t))
        .unwrap_or(fixed_b.clone());
    let held_b = held_threshold
        .map(|t| abstain_split_conf(&b_conf, t))
        .unwrap_or_else(|| refit_b.clone());
    let in_engine_shaped = EngineShapedRead {
        n: b_n,
        max_sim: dist_summary(&b_max, 0),
        conf: dist_summary(&b_conf, 0),
        abstain_fixed: fixed_b,
        abstain_refit: refit_b,
        abstain_held1x: held_b,
    };

    // ── desert + verdict inputs (corpus-shaped routed statistics) ──
    let clearance_off = f64::from(GATE_MID) - f64::from(in_read_routed_p99(&off_read));
    let in_floor_above_mid = f64::from(in_read_routed_p05(&in_read)) - f64::from(GATE_MID);
    let cell = GateDriftSizeCell {
        size_label: size_label.to_string(),
        suites: family.iter().map(|a| a.name.clone()).collect(),
        n_domains: N,
        corpus_rows,
        cap_per_label: FAMILY_CAP_PER_LABEL,
        per_domain_rows,
        null_reference,
        off_p99_clear_fixed: off_read.whole.p99.value < GATE_MID,
        off_max_clear_fixed: off_read.whole.max < GATE_MID,
        off_p99_clear_routed: off_read.routed.p99.value < GATE_MID,
        refit_off_false_accept_rate: 1.0 - off_read.abstain_refit.rate,
        fixed_in_false_abstain_rate: in_read.abstain_fixed.rate,
        off_corpus: off_read,
        off_per_source: off_counts,
        in_corpus: in_read,
        in_per_suite: in_counts,
        in_engine_shaped,
        refit_threshold,
        refit_cal_observations,
        clearance_off,
        in_floor_above_mid,
    };
    Ok(cell)
}

fn in_read_routed_p99(r: &CorpusShapedRead) -> f32 {
    r.routed.p99.value
}

fn in_read_routed_p05(r: &CorpusShapedRead) -> f32 {
    r.routed.p05.value
}

// ───────────────────────────────────────────── loading ──

struct SuiteArtefacts {
    name: String,
    /// The corpus pool: stratified train REST (cal front excluded by
    /// construction — the deployed `prepare` law).
    corpus_pool: Vec<TrainDoc>,
    cal_cases: Vec<SuiteCase>,
    cal_state_strs: Vec<String>,
    /// Corpus-shaped in-corpus probe texts (stride-sampled).
    a_texts: Vec<String>,
    a_dropped: usize,
    /// Engine-shaped in-corpus cases (stride-sampled, same front).
    b_cases: Vec<SuiteCase>,
    b_state_strs: Vec<String>,
}

fn spec_of(name: &str) -> Result<&'static SuiteSpec, String> {
    SUITES
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("gate-drift: suite {name} is not in the registry"))
}

fn stride_texts(docs: Vec<TrainDoc>, max: usize) -> (Vec<String>, usize) {
    let mut texts: Vec<String> = Vec::with_capacity(docs.len());
    let mut dropped = 0usize;
    for d in docs {
        if d.text.trim().is_empty() {
            dropped += 1;
        } else {
            texts.push(d.text);
        }
    }
    let n = texts.len();
    let picked: Vec<usize> = stride_indices(n, max);
    (
        picked.into_iter().map(|i| texts[i].clone()).collect(),
        dropped,
    )
}

fn load_suite_artefacts(
    spec: &SuiteSpec,
    datasets_dir: &std::path::Path,
    gopts: GateDriftOptions,
) -> Result<SuiteArtefacts, String> {
    if spec.corpus_cap_per_label != FAMILY_CAP_PER_LABEL {
        return Err(format!(
            "gate-drift: {} registry cap is {} — the uniform-cap premise ({} per label) \
             broke; re-derive the instrument before running",
            spec.name, spec.corpus_cap_per_label, FAMILY_CAP_PER_LABEL
        ));
    }
    let dir = datasets_dir.join(spec.dataset_dir);
    // Corpus pool + cal slice (the deployed prepare construction).
    let train_rows = load_rows(&dir, "train").map_err(|e| format!("{}: {e}", spec.name))?;
    let cal_split = stratified_split(&train_rows, spec.name, spec.cal_cap);
    let corpus_pool = train_docs(&cal_split.rest, spec.name);
    if corpus_pool.is_empty() {
        return Err(format!(
            "gate-drift: {} train pool is empty — fetch train rows first",
            spec.name
        ));
    }
    let cal_suite = (spec.build)(&cal_split.front, 0);
    let cal_state_strs: Vec<String> = cal_suite
        .cases
        .iter()
        .map(|c| serialize_state(&c.state))
        .collect();
    // Eval front (the registry test sample) — both in-corpus surfaces.
    let eval_rows = load_rows(&dir, spec.eval_split)?;
    let eval_front = stratified_split(&eval_rows, spec.name, spec.test_cap).front;
    let eval_docs = train_docs(&eval_front, spec.name);
    let (a_texts, a_dropped) = stride_texts(eval_docs, gopts.max_in_per_suite);
    let eval_suite = (spec.build)(&eval_front, 0);
    let b_all = eval_suite.cases.len();
    let picked: Vec<usize> = stride_indices(b_all, gopts.max_in_per_suite);
    let b_cases: Vec<SuiteCase> = picked
        .iter()
        .map(|&i| eval_suite.cases[i].clone())
        .collect();
    let b_state_strs: Vec<String> = b_cases.iter().map(|c| serialize_state(&c.state)).collect();
    Ok(SuiteArtefacts {
        name: spec.name.to_string(),
        corpus_pool,
        cal_cases: cal_suite.cases,
        cal_state_strs,
        a_texts,
        a_dropped,
        b_cases,
        b_state_strs,
    })
}

fn load_off_source(
    name: &str,
    datasets_dir: &std::path::Path,
    gopts: GateDriftOptions,
) -> Result<(String, Vec<String>, usize), String> {
    let spec = spec_of(name)?;
    let dir = datasets_dir.join(spec.dataset_dir);
    let rows = load_rows(&dir, "test")?;
    let docs = train_docs(&rows, spec.name);
    let (texts, dropped) = stride_texts(docs, gopts.max_off_per_source);
    if texts.is_empty() {
        return Err(format!(
            "gate-drift: off-corpus source {name} produced no probes"
        ));
    }
    Ok((name.to_string(), texts, dropped))
}

// ───────────────────────────────────────────── entry ──

/// Run the diagnostic. Sizes in order so the held-1× posture carries the
/// 1× fit forward; report-only (no gate, no exit-code coupling).
pub fn run_gate_drift(
    opts: &RunOptions,
    gopts: GateDriftOptions,
) -> Result<GateDriftOutput, String> {
    if gopts.max_off_per_source == 0 || gopts.max_in_per_suite == 0 {
        return Err("gate-drift: probe budgets must be > 0".to_string());
    }
    let box_start = crate::harness::box_state::capture();

    let mut arts: Vec<SuiteArtefacts> = Vec::new();
    for name in GATE_DRIFT_FAMILY {
        let spec = spec_of(name)?;
        arts.push(load_suite_artefacts(spec, &opts.datasets_dir, gopts)?);
    }
    let mut off_groups: Vec<(String, Vec<String>, usize)> = Vec::new();
    for name in GATE_DRIFT_OFF_SOURCES {
        off_groups.push(load_off_source(name, &opts.datasets_dir, gopts)?);
    }

    let mut cells: Vec<GateDriftSizeCell> = Vec::new();
    let mut held: Option<f32> = None;
    let size_labels = ["1x_ag_news", "2x_+emotion", "4x_+sst5+xnli"];
    for (si, &take) in [1usize, 2, 4].iter().enumerate() {
        let cell = match take {
            1 => run_size::<4>(take, size_labels[si], &arts, &off_groups, gopts, held)?,
            2 => run_size::<10>(take, size_labels[si], &arts, &off_groups, gopts, held)?,
            4 => run_size::<18>(take, size_labels[si], &arts, &off_groups, gopts, held)?,
            other => return Err(format!("gate-drift: unknown size take {other}")),
        };
        if held.is_none() {
            held = cell.refit_threshold;
        }
        cells.push(cell);
    }

    let box_end = crate::harness::box_state::capture();
    Ok(GateDriftOutput {
        meta: GateDriftMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            family: GATE_DRIFT_FAMILY.iter().map(|s| (*s).to_string()).collect(),
            off_sources: GATE_DRIFT_OFF_SOURCES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            cap_per_label: FAMILY_CAP_PER_LABEL,
            gate_mid: GATE_MID,
            gate_scale: GATE_SCALE,
            max_off_per_source: gopts.max_off_per_source,
            max_in_per_suite: gopts.max_in_per_suite,
            off_probe_construction: "test rows of banking77 + massive_intent_en \
             (stride-sampled, text-field rule per suite) — sources held out from \
             EVERY family suite at EVERY size, so the off-corpus and in-corpus \
             populations never mix (issue T1a-i). Empty/whitespace texts dropped \
             and counted, never measured."
                .to_string(),
            posture_note: "fixed = the birth boundary (max_sim < GATE_MID, exactly \
             gate_conf < 0.5); refit = the rho=30 percentile distance threshold \
             re-fitted on THIS size's pooled cal slice (absorbs drift by \
             construction — never pooled with fixed); held1x = the 1x fitted \
             threshold applied unchanged at later sizes (the aging-deployment \
             posture, a disclosed addition)."
                .to_string(),
            box_start,
            box_end,
        },
        cells,
    })
}

// ───────────────────────────────────────────── render ──

/// `rate` as a fixed-percent string.
fn pct(rate: f64) -> String {
    format!("{:.1}%", rate * 100.0)
}

/// One quantile row cell with support.
fn qs(q: &QuantileRow) -> String {
    format!("{:.4} (sup {})", q.value, q.support)
}

/// The markdown record (the bench doc quotes this; nothing here is a
/// serving-posture number in code — the numbers live in `.benchmarks/`).
#[must_use]
pub fn render_gate_drift_markdown(out: &GateDriftOutput) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# Issue 093 — CorpusDistanceGate drift vs corpus growth (gate-drift probe)"
    );
    let _ = writeln!(
        s,
        "\n- date: {} · host: {} · sha: {}",
        out.meta.date_utc,
        out.meta.host,
        out.meta.git_sha.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        s,
        "- family: {:?} · off sources: {:?} · cap/label {} · GATE_MID {} · GATE_SCALE {}",
        out.meta.family,
        out.meta.off_sources,
        out.meta.cap_per_label,
        out.meta.gate_mid,
        out.meta.gate_scale
    );
    let _ = writeln!(
        s,
        "- datasets: {} · budgets: off {}/source · in {}/suite",
        out.meta.datasets_dir, out.meta.max_off_per_source, out.meta.max_in_per_suite
    );
    let _ = writeln!(s, "- box start: {:?}", out.meta.box_start);
    let _ = writeln!(s, "- box end:   {:?}", out.meta.box_end);
    let _ = writeln!(
        s,
        "\n## Off-corpus probe construction\n\n{}",
        out.meta.off_probe_construction
    );
    let _ = writeln!(s, "\n## Postures\n\n{}", out.meta.posture_note);

    for c in &out.cells {
        let _ = writeln!(
            s,
            "\n## {} — domains {} · corpus rows {} · null √(2·ln M/256) = {:.4}",
            c.size_label, c.n_domains, c.corpus_rows, c.null_reference
        );
        let _ = writeln!(
            s,
            "\nPer-domain rows: {:?} · refit threshold: {} (cal n = {})",
            c.per_domain_rows,
            c.refit_threshold
                .map_or("THIN SUPPORT — birth fallback".to_string(), |t| format!(
                    "{t:.4}"
                )),
            c.refit_cal_observations
        );
        let _ = writeln!(
            s,
            "\n| population · surface | n | min | p05 | p50 | p90 | p99 | max | fixed abstain | refit abstain | held1x abstain |"
        );
        let _ = writeln!(s, "|---|---|---|---|---|---|---|---|---|---|---|");
        let off = &c.off_corpus;
        let _ = writeln!(
            s,
            "| off · whole-corpus | {} | {:.4} | {} | {} | {} | {} | {:.4} | — | — | — |",
            off.whole.n,
            off.whole.min,
            qs(&off.whole.p05),
            qs(&off.whole.p50),
            qs(&off.whole.p90),
            qs(&off.whole.p99),
            off.whole.max,
        );
        let _ = writeln!(
            s,
            "| off · routed gate | {} | {:.4} | {} | {} | {} | {} | {:.4} | {} ({}) | {} ({:.3}) | {} ({:.3}) |",
            off.routed.n,
            off.routed.min,
            qs(&off.routed.p05),
            qs(&off.routed.p50),
            qs(&off.routed.p90),
            qs(&off.routed.p99),
            off.routed.max,
            pct(off.abstain_fixed.rate),
            off.abstain_fixed.abstains,
            pct(off.abstain_refit.rate),
            off.abstain_refit.threshold,
            pct(off.abstain_held1x.rate),
            off.abstain_held1x.threshold,
        );
        let inc = &c.in_corpus;
        let _ = writeln!(
            s,
            "| in · whole-corpus | {} | {:.4} | {} | {} | {} | {} | {:.4} | — | — | — |",
            inc.whole.n,
            inc.whole.min,
            qs(&inc.whole.p05),
            qs(&inc.whole.p50),
            qs(&inc.whole.p90),
            qs(&inc.whole.p99),
            inc.whole.max,
        );
        let _ = writeln!(
            s,
            "| in · routed gate | {} | {:.4} | {} | {} | {} | {} | {:.4} | {} ({}) | {} ({:.3}) | {} ({:.3}) |",
            inc.routed.n,
            inc.routed.min,
            qs(&inc.routed.p05),
            qs(&inc.routed.p50),
            qs(&inc.routed.p90),
            qs(&inc.routed.p99),
            inc.routed.max,
            pct(inc.abstain_fixed.rate),
            inc.abstain_fixed.abstains,
            pct(inc.abstain_refit.rate),
            inc.abstain_refit.threshold,
            pct(inc.abstain_held1x.rate),
            inc.abstain_held1x.threshold,
        );
        let b = &c.in_engine_shaped;
        let _ = writeln!(
            s,
            "| in · ENGINE (real requests, ctx embed) | {} | {:.4} | {} | {} | {} | {} | {:.4} | {} ({}) | {} ({:.3}) | {} ({:.3}) |",
            b.n,
            b.max_sim.min,
            qs(&b.max_sim.p05),
            qs(&b.max_sim.p50),
            qs(&b.max_sim.p90),
            qs(&b.max_sim.p99),
            b.max_sim.max,
            pct(b.abstain_fixed.rate),
            b.abstain_fixed.abstains,
            pct(b.abstain_refit.rate),
            b.abstain_refit.threshold,
            pct(b.abstain_held1x.rate),
            b.abstain_held1x.threshold,
        );
        let _ = writeln!(
            s,
            "\nDesert (routed reads): off p99 → mid clearance {:+.4} · in p05 above mid {:+.4} · \
             refit off-corpus false-accept {} · fixed in-corpus false-abstain {}",
            c.clearance_off,
            c.in_floor_above_mid,
            pct(c.refit_off_false_accept_rate),
            pct(c.fixed_in_false_abstain_rate),
        );
        let _ = writeln!(
            s,
            "\nOff counts per source: {:?} · in counts per suite: {:?}",
            c.off_per_source, c.in_per_suite
        );
    }

    if let Some(last) = out.cells.last() {
        let _ = writeln!(
            s,
            "\n## T2 close-condition inputs (the {} cell)\n",
            last.size_label
        );
        let _ = writeln!(
            s,
            "- off-corpus p99 clear of mid (whole-corpus read, fixed boundary): {}",
            last.off_p99_clear_fixed
        );
        let _ = writeln!(
            s,
            "- off-corpus MAX clear of mid (whole-corpus read, fixed boundary): {}",
            last.off_max_clear_fixed
        );
        let _ = writeln!(
            s,
            "- off-corpus p99 clear of mid (routed read, fixed boundary): {}",
            last.off_p99_clear_routed
        );
        let _ = writeln!(
            s,
            "- refit false-accept rate on off-corpus: {}",
            pct(last.refit_off_false_accept_rate)
        );
        let _ = writeln!(
            s,
            "\nT2 closes ONLY if the tail stays clear in BOTH postures; cite the support columns beside every quantile (the p99 support shrinks with n)."
        );
    }
    s
}

// ───────────────────────────────────────────── tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantile_rows_carry_support_and_clamp() {
        let v: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let s = sorted_sample(&v);
        let p99 = quantile_row(&s, 0.99, "p99");
        assert_eq!(p99.rank, 99);
        assert_eq!(p99.support, 1);
        assert_eq!(p99.value, 99.0);
        let p50 = quantile_row(&s, 0.50, "p50");
        assert_eq!(p50.rank, 50);
        assert_eq!(p50.support, 50);
        // n = 20 at p99 lands on the MAX (the repo's percentile trap) —
        // the support column is what discloses it.
        let v20: Vec<f32> = (0..20).map(|i| i as f32).collect();
        let s20 = sorted_sample(&v20);
        let p99_20 = quantile_row(&s20, 0.99, "p99");
        assert_eq!(p99_20.rank, 19);
        assert_eq!(p99_20.value, 19.0);
        assert_eq!(p99_20.support, 1);
        // p0 = the min with full support.
        let p0 = quantile_row(&s20, 0.0, "p05");
        assert_eq!(p0.rank, 0);
        assert_eq!(p0.support, 20);
        assert_eq!(p0.value, 0.0);
    }

    #[test]
    fn dist_summary_reports_min_max_and_support() {
        let s = dist_summary(&[0.5, 0.1, 0.9, 0.3, 0.7], 2);
        assert_eq!(s.n, 5);
        assert_eq!(s.dropped_empty, 2);
        assert_eq!(s.min, 0.1);
        assert_eq!(s.max, 0.9);
        assert_eq!(s.p50.rank, 2);
        assert_eq!(s.p50.value, 0.5);
    }

    #[test]
    fn null_reference_monotone_in_m_and_spot_checked() {
        let m_small = isotropic_null_reference(256, EMBED_DIM);
        let m_big = isotropic_null_reference(1152, EMBED_DIM);
        assert!(m_small < m_big);
        // √(2·ln 1152 / 256) ≈ 0.2347 — well below the 0.35 midpoint: under
        // the ISOTROPIC null the desert stays open; the measured curve is
        // the question (clustered/anisotropic hashed bags).
        assert!((m_big - 0.234_7).abs() < 5e-4);
        assert_eq!(isotropic_null_reference(0, EMBED_DIM), 0.0);
    }

    #[test]
    fn fixed_boundary_is_the_midpoint_at_any_scale() {
        // sigmoid(scale·(x − mid)) < 0.5 ⇔ x < mid — the exact equivalence
        // the fixed posture's abstain split rides.
        let grid: Vec<f32> = (-60..=60).map(|i| GATE_MID + i as f32 / 100.0).collect();
        for &x in &grid {
            let conf = crate::engine::gate_confidence(x);
            assert_eq!(conf < 0.5, x < GATE_MID, "x = {x}");
        }
        let split = abstain_split(&grid, GATE_MID);
        assert_eq!(split.abstains, 60); // the 60 points strictly below mid
        // x == mid itself: conf == 0.5, not < 0.5 — passes, and the
        // boundary predicate (`x < mid`) agrees, so the equivalence held
        // for every grid point including the boundary itself.
    }

    #[test]
    fn abstain_split_conf_matches_the_gate_predicate() {
        let values = [0.30f32, 0.34, 0.35, 0.36, 0.40];
        // threshold 0.5 on conf == the fixed boundary on max_sim.
        let a = abstain_split(&values, GATE_MID);
        let b = abstain_split_conf(&values, 0.5);
        assert_eq!(a.abstains, b.abstains);
        assert_eq!(a.abstains, 2); // 0.30, 0.34
    }

    #[test]
    fn family_labels_are_disjoint_and_counted() {
        let all = family_labels(4);
        assert_eq!(all.len(), 18);
        assert!(labels_disjoint(&all));
        assert!(!labels_disjoint(&[
            "world".to_string(),
            "world".to_string()
        ]));
        let two = family_labels(2);
        assert_eq!(two.len(), 10);
    }
}
