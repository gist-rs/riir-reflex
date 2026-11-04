//! Issue 107 — the routing-state probe + pinned-path replay PoC
//! (Research 014 / arXiv:2611.03526, the Library-of-Layers distill).
//!
//! Two closed-form runtime halves of a trained router, transferred to
//! reflex's `pick_domain` engine, both MEASUREMENT-ONLY — **no serving-path
//! change anywhere in this issue** (the forced-domain seam stays disarmed
//! on production paths exactly as Bench 167 left it; T2.3 is a hard
//! no-promotion gate — a worthwhile replay posture files a NEW issue):
//!
//! **T1 — the supervised probe over routing-state features** (the
//! supervised bound completing Bench 167's unsupervised MI finding). Two
//! arms with different status:
//!
//! - **(a) score-vector arm — MANDATORY.** The per-domain scores the
//!   router computes before argmax. These are RAW DOT PRODUCTS
//!   (`dot(embed(ctx), direction)`), so the probe re-derives them
//!   bit-exactly from two existing surfaces — the public `Scratch.q`
//!   (the ctx embed the routing site consumed) and the read-only
//!   `DecisionEngine::routing_directions()` accessor — with
//!   `pick_domain`'s exact sequential f32 fold. A per-slot consistency
//!   gate asserts the re-derived argmax equals the engine's routed
//!   domain (`sc.domains`); any mismatch is an instrument bug and
//!   refuses the run loud. This arm can carry information BEYOND Bench
//!   167's MI (a one-hot routed domain makes the probe a lookup table
//!   whose accuracy is already bounded by the measured MI).
//! - **(b) one-hot arm — consistency check only.** It MUST agree with
//!   the MI-implied picture (a linear readout over one-hot inputs
//!   reproduces the per-domain class rates); agreement is the
//!   deliverable, not a finding.
//!
//! Closed-form ridge one-vs-rest (the `nb_ridge` solve shape: Gram + bias
//! column + λI, solved per class by katgpt-core
//! `ridge_solve_direct_f32`), fit on the suite's CAL slice (disjoint from
//! the test split by construction — different source splits), evaluated
//! held-out on the stride-sampled test slots, ONE frozen read. Pre-
//! registered discipline:
//!
//! - λ is swept over a fixed grid, but the pre-registered GATES read the
//!   PRE-NOMINATED [`GATE_LAMBDA`] row (`nb_ridge`'s own default — the
//!   house ridge strength); the full grid is disclosure, and its
//!   argmax-λ row is the OPTIMISTIC bound, never a gate.
//! - AUROC = macro one-vs-rest with per-class tail support printed; a
//!   class under [`TAIL_SUPPORT_FLOOR`] positives or negatives is
//!   UNDECIDABLE, excluded from the macro mean, named.
//! - banking77 gate: probe accuracy at the gate λ lands within 2σ of its
//!   routed accuracy (the MI already predicts near-perfect — the gate
//!   proves the probe doesn't UNDERREAD the MI signal).
//! - ag_news / emotion: AUROC reported with tail support; thin support
//!   is UNDECIDABLE, never folded into a pass.
//! - Degenerate-feature loud-fail: a suite whose routing features are
//!   near-constant FAILS the probe loudly, never scores ~0.5 silently.
//! - T1.3: the default claim is AUROC-only (discrimination, not
//!   calibration); no threshold/abstention consumer is proposed by this
//!   PoC, so the conformal-naive floor does not apply.
//! - T1.4: verdict per suite — does the MI number PREDICT the probe
//!   result? HIGH-MI suites ([`HIGH_MI_BITS`]) should land the probe in
//!   the 2σ band; LOW-MI suites should stay under [`OVERREAD_AUROC`]. A
//!   LOW-MI suite whose score-vector probe reads ABOVE that bar is the
//!   OVERREAD finding (routing state carries label-relevant signal
//!   beyond the argmax — the T3 trigger). Internal prior negative
//!   carried: reflex Bench 157 recorded depth-as-CONFIDENCE NEGATIVE on
//!   the residual stream — a DIFFERENT feature source; this probe is
//!   router-state-keyed and is not a Bench 157 re-run.
//!
//! **T2 — mode-path extraction + report-only pinned replay.** The engine
//! has ONE routing level (Bench 167's structural finding), so the LoL
//! mode path degenerates to a single domain per key. Extraction reads
//! the FIT slice only (cal cases): per key — the suite (a CLASS-BLIND
//! constant pin, named baseline shape), T1's predicted class, and the
//! gold label (ORACLE CEILING ONLY, leakage-bound, never a
//! non-inferiority result) — the argmax of the mean selection → a frozen
//! table, BLAKE3-digested; bit-identical re-extraction is a gate. The
//! table's key is inference-available information ONLY for the two
//! inference-keyed variants (suite id / probe-predicted class) — never
//! the gold label (a gold-keyed pin leaks the label at inference).
//! Replay forces each eval case's domain through the existing
//! `Scratch::forced_domain` seam with paired readouts that are NEVER
//! pooled: paired accuracy live-vs-pinned over both-answered slots with
//! the house paired-LB95 non-inferiority bar (`paired_lb95` > −
//! [`NONINFER_DELTA`]) + adherence (the seam engagement gate: the routed
//! domain MUST equal the pin, else the run refuses) + abstain-rate
//! deltas. The predicted-class pin's ceiling IS the probe's accuracy —
//! named in the readout, never claimed beyond.
//!
//! Measurement posture: the deployed engine build EXACTLY as Bench 167's
//! control (`fit_posture_inner` + `build_engine_with`, every selection
//! lever pinned off, the cal-slice `observe` calibration). Witnesses:
//! capture determinism (the first fit slots re-decide bit-identically) +
//! extraction digest equality.

use std::collections::BTreeMap;

use serde::Serialize;

use super::ensemble::paired_lb95;
use super::perm_probe::stride_indices;
use super::{
    FittedPosture, ModellessInput, Prepared, RunOptions, SUITES, SuiteSpec, answer_probs_pick,
    build_engine_with, engine_request, fit_posture_inner, git_sha, hostname_refusing_unknown,
    iso8601_utc, prepare,
};
use crate::embed::EMBED_DIM;
use crate::engine::{DecisionEngine, EngineConfig, Scratch};
use crate::harness::suites::{QKind, SuiteCase};
use crate::pyjson::serialize_state;
use katgpt_core::linalg::ridge_solve_direct_f32;
use katgpt_core::variable_rank_domain_expert::pick_domain;

/// Default suites: Bench 167's four labeled domains (the MI numbers this
/// probe completes).
pub const DEFAULT_ROUTING_STATE_PROBE_SUITES: &[&str] =
    &["ag_news", "banking77", "emotion", "sst5"];

/// Default sampled eval cases per suite (stride over the choice/score
/// test cases; the fit slice is the suite's whole cal slice).
pub const DEFAULT_ROUTING_STATE_PROBE_MAX_CASES: usize = 300;

/// The fixed λ grid (disclosure rows; the gates read [`GATE_LAMBDA`]).
pub const RIDGE_LAMBDA_GRID: [f32; 6] = [1e-3, 1e-2, 1e-1, 1.0, 10.0, 100.0];

/// The PRE-NOMINATED gate λ — `nb_ridge`'s own default (the house ridge
/// strength). The λ sweep is disclosure; this row carries the gates.
pub const GATE_LAMBDA: f32 = 10.0;

/// Per-class OVR tail-support floor: fewer positives or negatives than
/// this and the class is UNDECIDABLE, excluded from the macro mean,
/// named.
pub const TAIL_SUPPORT_FLOOR: usize = 10;

/// The paired non-inferiority bar (1 pp floor): the pin passes only when
/// the paired LB95 of (pinned − live) sits above minus this.
pub const NONINFER_DELTA: f64 = 0.01;

/// A suite whose MI clears this many bits is HIGH-MI (routing ≈ a
/// classifier there — banking77's Bench-167 read is 5.59; the LOW-MI
/// trio reads 0.14–0.28). This is the T1.4 expectation split.
pub const HIGH_MI_BITS: f64 = 1.0;

/// A LOW-MI suite whose score-vector probe macro-AUROC reads above this
/// is the OVERREAD finding (signal beyond the argmax — the T3 trigger).
pub const OVERREAD_AUROC: f64 = 0.70;

/// Near-constant feature std under which the score-vector arm is
/// DEGENERATE (the loud-fail class).
const DEGENERATE_STD: f32 = 1e-6;

// ────────────────────────────────────────────────────────── records ──

/// The T1.4 verdict vocabulary (per suite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProbeVerdict {
    /// The MI number predicted the probe result.
    AgreeWithMi,
    /// HIGH-MI suite whose probe landed BELOW the 2σ band — the linear
    /// probe underreads the lookup signal at this fit size (a finding).
    Underread,
    /// The probe reads signal the MI says should not be there (HIGH-MI:
    /// above the band; LOW-MI: macro AUROC > [`OVERREAD_AUROC`]) — the
    /// T3 trigger class.
    Overread,
    /// No included classes at the tail-support floor — no macro AUROC
    /// statement exists.
    Undecidable,
    /// Near-constant routing features — the loud-fail class.
    Degenerate,
}

impl ProbeVerdict {
    fn str_of(self) -> &'static str {
        match self {
            Self::AgreeWithMi => "AGREE-WITH-MI",
            Self::Underread => "UNDERREAD",
            Self::Overread => "OVERREAD",
            Self::Undecidable => "UNDECIDABLE",
            Self::Degenerate => "DEGENERATE",
        }
    }
}

/// One arm's headline row (at [`GATE_LAMBDA`]).
#[derive(Debug, Clone, Serialize)]
pub struct ProbeArmRow {
    /// `"score_vector"` (the mandatory arm) or `"one_hot"` (the MI
    /// consistency check).
    pub arm: String,
    pub acc: f64,
    pub macro_auroc: Option<f64>,
    pub n_included: usize,
    pub n_excluded: usize,
    /// Up to 6 named exclusions: `label(n_pos/n_neg)`.
    pub excluded_examples: Vec<String>,
    pub degenerate: bool,
}

/// One λ disclosure row (score-vector arm).
#[derive(Debug, Clone, Serialize)]
pub struct LambdaGridRow {
    pub lambda: f32,
    pub acc: f64,
    pub macro_auroc: Option<f64>,
    pub n_included: usize,
}

/// One pinned-replay readout (T2.2). The three kinds are never pooled.
#[derive(Debug, Clone, Serialize)]
pub struct PinRow {
    /// `"suite_constant"` | `"predicted_class"` | `"gold_oracle"`.
    pub kind: String,
    pub n_slots: usize,
    pub n_paired: usize,
    pub acc_live: f64,
    pub acc_pinned: f64,
    pub delta: f64,
    pub lb95: f64,
    /// Paired LB95 > −[`NONINFER_DELTA`] with ≥ 2 paired slots. `false`
    /// on an UNDECIDABLE pairing too (n < 2 renders as such).
    pub noninferior: bool,
    pub abstain_live: f64,
    pub abstain_pinned: f64,
    /// Eval keys with no fit-table entry (routed to the suite modal).
    pub fallback_count: usize,
    pub ceiling_note: String,
}

/// Per-suite record.
#[derive(Debug, Clone, Serialize)]
pub struct RoutingProbeSuite {
    pub suite: String,
    pub n_fit: usize,
    pub n_eval: usize,
    pub n_domains: usize,
    /// MI(gold; routed domain), bits — the eval-slot recomputation of
    /// Bench 167's Stat 2 (comparable read).
    pub mi_bits: f64,
    /// H(gold), bits — the MI's ceiling.
    pub label_entropy_bits: f64,
    /// Routed-domain-as-class accuracy over ALL eval slots (the forced
    /// reading — abstention ignored, named in the record).
    pub routed_forced_acc: f64,
    /// The 2σ half-width around `routed_forced_acc` (n = eval slots).
    pub two_sigma: f64,
    pub arms: Vec<ProbeArmRow>,
    pub lambda_grid_rows: Vec<LambdaGridRow>,
    pub verdict: ProbeVerdict,
    pub verdict_note: String,
    pub pins: Vec<PinRow>,
    /// One BLAKE3 digest over the canonical serialization of all three
    /// frozen tables (key-ascending).
    pub mode_path_digest: String,
    pub reextract_digest_match: bool,
    pub capture_witness_ok: bool,
    /// The class-blind constant's domain (the suite modal over the fit
    /// slice).
    pub suite_modal_domain: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutingProbeMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub max_cases: usize,
    pub gate_lambda: f32,
    pub lambda_grid: Vec<f32>,
    pub tail_support_floor: usize,
    pub noninferior_delta: f64,
    pub measurement_posture: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutingStateProbeOutput {
    pub meta: RoutingProbeMeta,
    pub suites: Vec<RoutingProbeSuite>,
    /// Lane-level gate: at least one suite is live (score-vector arm
    /// non-degenerate, capture witness green, digest re-extraction
    /// green). An all-degenerate lane refuses the run.
    pub lane_probe_live: bool,
}

#[derive(Debug, Clone)]
pub struct RoutingStateProbeOptions {
    pub max_cases: usize,
}

// ────────────────────────────────────────────────── pure helpers ──

/// The router's pre-argmax score vector — `pick_domain`'s exact
/// sequential f32 fold (same order, same adds → bit-identical scores).
fn router_scores_of<const N: usize, const D: usize>(
    q: &[f32; D],
    dirs: &[[f32; D]; N],
) -> [f32; N] {
    let mut scores = [0.0f32; N];
    for (d, dir) in dirs.iter().enumerate() {
        let mut s = 0.0f32;
        for i in 0..D {
            s += q[i] * dir[i];
        }
        scores[d] = s;
    }
    scores
}

/// Argmax with first-index-wins ties. Pin-verified against
/// `pick_domain`'s tie rule by
/// `first_index_argmax_matches_pick_domain_tie_rule`.
fn first_index_argmax(scores: &[f32]) -> usize {
    let mut best = 0usize;
    for (d, &s) in scores.iter().enumerate().skip(1) {
        if s > scores[best] {
            best = d;
        }
    }
    best
}

/// Score-vector degenerate: EVERY feature's std across the eval rows is
/// under [`DEGENERATE_STD`] — the router sees no spread at all.
fn score_arm_degenerate(x: &[Vec<f32>]) -> bool {
    if x.is_empty() {
        return true;
    }
    let k = x[0].len();
    let n = x.len() as f32;
    (0..k).all(|a| {
        let mean = x.iter().map(|r| r[a]).sum::<f32>() / n;
        let var = x.iter().map(|r| (r[a] - mean) * (r[a] - mean)).sum::<f32>() / n;
        var.sqrt() < DEGENERATE_STD
    })
}

/// One-hot degenerate: the routed domain is constant across eval slots.
fn one_hot_degenerate(domains: &[usize]) -> bool {
    let first = match domains.first() {
        Some(&d) => d,
        None => return true,
    };
    domains.iter().all(|&d| d == first)
}

/// Closed-form one-vs-rest ridge, the `nb_ridge` solve shape: augmented
/// feature row `[x, 1]` (bias), Gram `XᵀX` + λI on the whole diagonal,
/// per-class cov `Xᵀy` (bias cov = Σy), solved by katgpt-core
/// `ridge_solve_direct_f32`. f32 — the same arithmetic space the house
/// lane solves in.
struct RidgeOvr {
    stride: usize,
    n_classes: usize,
    weights: Vec<f32>, // n_classes × stride
}

fn fit_ridge_ovr(x: &[Vec<f32>], golds: &[usize], lambda: f32, n_classes: usize) -> RidgeOvr {
    debug_assert_eq!(x.len(), golds.len());
    let k = x.first().map(|r| r.len()).unwrap_or(0);
    let stride = k + 1;
    // Gram + bias cross terms (computed once — λ only touches the
    // diagonal).
    let mut gram = vec![0f32; stride * stride];
    for row in x {
        for a in 0..k {
            let xa = row[a];
            if xa == 0.0 {
                continue;
            }
            let arow = a * stride;
            for b in 0..k {
                gram[arow + b] += xa * row[b];
            }
            gram[arow + k] += xa;
            gram[k * stride + a] += xa;
        }
        gram[k * stride + k] += 1.0;
    }
    let mut weights = vec![0f32; stride * n_classes.max(1)];
    let mut gram_reg = vec![0f32; stride * stride];
    let mut l_scratch = vec![0f32; stride * stride];
    let mut z_scratch = vec![0f32; stride];
    for c in 0..n_classes {
        let mut cov = vec![0f32; stride];
        for (row, &g) in x.iter().zip(golds.iter()) {
            if g != c {
                continue;
            }
            for a in 0..k {
                cov[a] += row[a];
            }
            cov[k] += 1.0;
        }
        gram_reg.copy_from_slice(&gram);
        for i in 0..stride {
            gram_reg[i * stride + i] += lambda;
        }
        ridge_solve_direct_f32(
            &mut weights[c * stride..(c + 1) * stride],
            &mut l_scratch,
            &mut z_scratch,
            &gram_reg,
            &cov,
            stride,
            1,
        );
    }
    RidgeOvr {
        stride,
        n_classes,
        weights,
    }
}

impl RidgeOvr {
    /// Per-class scores for one feature row (`w·x + bias`).
    fn predict(&self, x: &[f32]) -> Vec<f32> {
        let k = self.stride - 1;
        (0..self.n_classes)
            .map(|c| {
                let w = &self.weights[c * self.stride..(c + 1) * self.stride];
                let mut s = w[k];
                for (a, &xa) in x.iter().enumerate() {
                    s += w[a] * xa;
                }
                s
            })
            .collect()
    }
}

/// Per-class one-vs-rest AUROC with average ranks over ties (f64).
/// `None` when either support side is under [`TAIL_SUPPORT_FLOOR`] —
/// UNDECIDABLE, never folded into a pass.
fn auroc_ovr(scores: &[f32], golds: &[usize], c: usize) -> Option<(f64, usize, usize)> {
    let n_pos = golds.iter().filter(|&&g| g == c).count();
    let n_neg = golds.len() - n_pos;
    if n_pos < TAIL_SUPPORT_FLOOR || n_neg < TAIL_SUPPORT_FLOOR {
        return None;
    }
    let mut ranked: Vec<(f64, usize)> = scores
        .iter()
        .enumerate()
        .map(|(i, &s)| (f64::from(s), i))
        .collect();
    ranked.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    // Average ranks over tie groups (1-based).
    let mut rank_of = vec![0.0f64; ranked.len()];
    let mut i = 0usize;
    while i < ranked.len() {
        let mut j = i;
        while j + 1 < ranked.len() && ranked[j + 1].0 == ranked[i].0 {
            j += 1;
        }
        let avg = (i + j + 2) as f64 / 2.0; // mean of 1-based positions i+1..=j+1
        for r in rank_of[i..=j].iter_mut() {
            *r = avg;
        }
        i = j + 1;
    }
    let rank_sum_pos: f64 = ranked
        .iter()
        .zip(rank_of.iter())
        .filter(|((_, idx), _)| golds[*idx] == c)
        .map(|(_, r)| *r)
        .sum();
    let auroc = (rank_sum_pos - (n_pos * (n_pos + 1)) as f64 / 2.0) / ((n_pos * n_neg) as f64);
    Some((auroc, n_pos, n_neg))
}

/// Macro OVR AUROC + the per-class support bookkeeping (labels name the
/// exclusions).
fn macro_auroc(
    scores_by_class: &[Vec<f32>],
    golds: &[usize],
    labels: &[String],
) -> (Option<f64>, usize, usize, Vec<String>) {
    let n_classes = scores_by_class.len();
    let mut included = Vec::with_capacity(n_classes);
    let mut n_excluded = 0usize;
    let mut examples = Vec::new();
    for (c, sc) in scores_by_class.iter().enumerate() {
        match auroc_ovr(sc, golds, c) {
            Some((a, _, _)) => included.push(a),
            None => {
                n_excluded += 1;
                if examples.len() < 6 {
                    let n_pos = golds.iter().filter(|&&g| g == c).count();
                    let name = labels.get(c).map(String::as_str).unwrap_or("?");
                    examples.push(format!("{name}({n_pos}/{})", golds.len() - n_pos));
                }
            }
        }
    }
    let macro_v = if included.is_empty() {
        None
    } else {
        Some(included.iter().sum::<f64>() / included.len() as f64)
    };
    (macro_v, included.len(), n_excluded, examples)
}

/// MI(gold; domain) in bits + H(gold) in bits (the Bench-167 shapes).
fn mi_and_entropy(slots: &[CapSlot], n_domains: usize) -> (f64, f64) {
    let total = slots.len() as f64;
    let max_label = slots.iter().map(|s| s.gold).max().unwrap_or(0);
    let n_label = max_label + 1;
    let mut j = vec![0u64; n_label * n_domains];
    let mut gm = vec![0u64; n_label];
    let mut dm = vec![0u64; n_domains];
    for s in slots {
        j[s.gold * n_domains + s.domain] += 1;
        gm[s.gold] += 1;
        dm[s.domain] += 1;
    }
    let mut mi = 0.0f64;
    for g in 0..n_label {
        for d in 0..n_domains {
            let v = j[g * n_domains + d] as f64;
            if v > 0.0 {
                let pj = v / total;
                let pg = gm[g] as f64 / total;
                let pd = dm[d] as f64 / total;
                mi += pj * (pj / (pg * pd)).log2();
            }
        }
    }
    let mut h = 0.0f64;
    for &c in &gm {
        let p = c as f64 / total;
        if p > 0.0 {
            h -= p * p.log2();
        }
    }
    (mi, h)
}

// ──────────────────────────────────────────────── capture ──

/// One captured slot: the routing state (pre-argmax scores + the routed
/// domain) beside the engine's verdict and the gold label.
#[derive(Debug, Clone)]
struct CapSlot {
    scores: Vec<f32>,
    domain: usize,
    pick: usize,
    abstained: bool,
    gold: usize,
}

/// Capture the routing state of every single-question choice/score case.
/// The pre-argmax score vector is re-derived from `sc.q` (the ctx embed
/// the routing site consumed — valid because every case carries exactly
/// ONE question, which this refuses to relax) with `pick_domain`'s exact
/// fold; a per-slot consistency gate asserts the re-derived argmax equals
/// the engine's routed domain. Returns (slots, skipped_count).
fn capture_slots<const N: usize, const D: usize>(
    engine: &mut DecisionEngine<N, D>,
    sc: &mut Scratch<D>,
    cases: &[SuiteCase],
    dirs: &[[f32; D]; N],
    slice_name: &str,
    suite_name: &str,
) -> Result<(Vec<CapSlot>, usize), String> {
    let mut slots = Vec::new();
    let mut skipped = 0usize;
    for cc in cases {
        if cc.questions.len() != 1 {
            return Err(format!(
                "{suite_name}: case {} carries {} questions — the probe reads \
                 `sc.q` after the decide, which is valid ONLY for single-question \
                 cases; the multi-question shape needs its own capture surface",
                cc.id,
                cc.questions.len()
            ));
        }
        if !matches!(cc.questions[0].kind, QKind::Choice | QKind::Score) {
            skipped += 1;
            continue;
        }
        let gold = match cc.gold.first() {
            Some(g) => g.idx,
            None => {
                skipped += 1;
                continue;
            }
        };
        let state_str = serialize_state(&cc.state);
        let req = engine_request(cc, &state_str)?;
        sc.forced_domain = None;
        engine
            .decide_with(&req, sc)
            .map_err(|e| format!("engine {slice_name} decide ({}): {e}", cc.id))?;
        let domain = sc.domains[0];
        let scores_vec = router_scores_of::<N, D>(&sc.q, dirs);
        // Instrument-liveness gate: the re-derived router must reproduce
        // the engine's own pick — the seam's pick_domain AND the plain
        // scan agree with the engine's routed domain.
        let rederived = pick_domain::<N, D>(&sc.q, dirs);
        let scanned = first_index_argmax(&scores_vec);
        if rederived != domain || scanned != domain {
            return Err(format!(
                "{suite_name}: routing-state re-derivation MISMATCH (engine {domain}, \
                 pick_domain {rederived}, scan {scanned}) — the probe's score vector \
                 is not what the router computed; refusing loud",
            ));
        }
        let slot = &sc.slots[0];
        slots.push(CapSlot {
            scores: scores_vec.to_vec(),
            domain,
            pick: slot.pick as usize,
            abstained: slot.abstained,
            gold,
        });
    }
    Ok((slots, skipped))
}

// ──────────────────────────────────────────────── extraction ──

/// Frozen mode-path tables keyed by: suite (single key 0 — the
/// CLASS-BLIND constant), T1's predicted class, and the gold label
/// (oracle only).
#[derive(Debug, Clone)]
struct ModeTables {
    suite: BTreeMap<i64, usize>,
    predicted: BTreeMap<i64, usize>,
    gold: BTreeMap<i64, usize>,
}

fn modal_domain(counts: &[u64]) -> usize {
    let mut best = 0usize;
    for (d, &c) in counts.iter().enumerate().skip(1) {
        if c > counts[best] {
            best = d;
        }
    }
    best
}

/// Extract from the FIT slice only. `fit_preds` are the score-arm gate-λ
/// IN-SAMPLE predictions (extraction keys, disclosed as such).
fn extract_mode_tables(
    fit: &[CapSlot],
    fit_preds: &[usize],
    n_domains: usize,
    n_classes: usize,
) -> ModeTables {
    let mut suite_counts = vec![0u64; n_domains];
    for s in fit {
        suite_counts[s.domain] += 1;
    }
    let suite = BTreeMap::from([(0i64, modal_domain(&suite_counts))]);
    let mut pred_counts = vec![vec![0u64; n_domains]; n_classes];
    let mut gold_counts = vec![vec![0u64; n_domains]; n_classes];
    for (i, s) in fit.iter().enumerate() {
        pred_counts[fit_preds[i].min(n_classes - 1)][s.domain] += 1;
        gold_counts[s.gold.min(n_classes - 1)][s.domain] += 1;
    }
    let mut predicted = BTreeMap::new();
    for (c, counts) in pred_counts.iter().enumerate() {
        if counts.iter().sum::<u64>() > 0 {
            predicted.insert(c as i64, modal_domain(counts));
        }
    }
    let mut gold = BTreeMap::new();
    for (g, counts) in gold_counts.iter().enumerate() {
        if counts.iter().sum::<u64>() > 0 {
            gold.insert(g as i64, modal_domain(counts));
        }
    }
    ModeTables {
        suite,
        predicted,
        gold,
    }
}

/// One BLAKE3 digest over the canonical serialization of all three tables
/// (key-ascending; i64 key + u64 value, little-endian, concatenated).
fn tables_digest(t: &ModeTables) -> String {
    let mut bytes = Vec::new();
    for map in [&t.suite, &t.predicted, &t.gold] {
        for (k, v) in map.iter() {
            bytes.extend_from_slice(&k.to_le_bytes());
            bytes.extend_from_slice(&(*v as u64).to_le_bytes());
        }
    }
    blake3::hash(&bytes).to_hex().to_string()
}

// ──────────────────────────────────────────────── replay ──

/// Force each case's routed domain to `pin_of(gold, predicted_class)` —
/// report-only, through the existing `Scratch::forced_domain` seam.
/// Adherence (the routed domain equals the pin) is the seam-engagement
/// gate: a mismatch refuses the run loud. Paired accuracy reads the
/// both-answered slots only; abstention is reported separately; the two
/// arms are never pooled.
#[allow(clippy::too_many_arguments)]
fn replay_pin<const N: usize, const D: usize>(
    engine: &mut DecisionEngine<N, D>,
    sc: &mut Scratch<D>,
    cases: &[SuiteCase],
    live: &[CapSlot],
    preds: &[usize],
    mut pin_of: impl FnMut(usize, usize) -> (usize, bool),
    kind: &str,
    ceiling_note: &str,
    suite_name: &str,
) -> Result<PinRow, String> {
    if cases.len() != live.len() || cases.len() != preds.len() {
        return Err(format!(
            "{suite_name}: pinned[{kind}] slice misalignment ({} cases, {} live, {} \
             preds) — capture skipped a case; refusing loud",
            cases.len(),
            live.len(),
            preds.len()
        ));
    }
    let mut live_ok: Vec<bool> = Vec::new();
    let mut pinned_ok: Vec<bool> = Vec::new();
    let mut live_abstain = 0usize;
    let mut pinned_abstain = 0usize;
    let mut fallback_count = 0usize;
    for (i, cc) in cases.iter().enumerate() {
        let base = &live[i];
        let (pin, fell_back) = pin_of(base.gold, preds[i]);
        if fell_back {
            fallback_count += 1;
        }
        let state_str = serialize_state(&cc.state);
        let req = engine_request(cc, &state_str)?;
        sc.forced_domain = Some(vec![Some(pin)]);
        engine
            .decide_with(&req, sc)
            .map_err(|e| format!("engine pinned[{kind}] decide ({}): {e}", cc.id))?;
        sc.forced_domain = None;
        // Adherence — the seam-engagement gate: the routed domain MUST be
        // the pin (a mismatch means the override did not reach the
        // router).
        if sc.domains[0] != pin {
            return Err(format!(
                "{suite_name}: pinned[{kind}] case {} routed to {} under pin {pin} — \
                 the forced_domain seam did not engage; refusing loud",
                cc.id, sc.domains[0]
            ));
        }
        let slot = &sc.slots[0];
        if base.abstained {
            live_abstain += 1;
        }
        if slot.abstained {
            pinned_abstain += 1;
        }
        if !base.abstained && !slot.abstained {
            live_ok.push(base.pick == base.gold);
            pinned_ok.push(slot.pick as usize == base.gold);
        }
    }
    let n = cases.len();
    let n_paired = live_ok.len();
    let acc_live = live_ok.iter().filter(|&&b| b).count() as f64 / n_paired.max(1) as f64;
    let acc_pinned = pinned_ok.iter().filter(|&&b| b).count() as f64 / n_paired.max(1) as f64;
    let lb95 = paired_lb95(&pinned_ok, &live_ok);
    Ok(PinRow {
        kind: kind.to_string(),
        n_slots: n,
        n_paired,
        acc_live,
        acc_pinned,
        delta: acc_pinned - acc_live,
        lb95,
        noninferior: n_paired >= 2 && lb95 > -NONINFER_DELTA,
        abstain_live: live_abstain as f64 / n.max(1) as f64,
        abstain_pinned: pinned_abstain as f64 / n.max(1) as f64,
        fallback_count,
        ceiling_note: ceiling_note.to_string(),
    })
}

// ───────────────────────────────────────────── the per-suite cell ──

#[allow(clippy::too_many_lines)]
fn routing_probe_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &RoutingStateProbeOptions,
) -> Result<RoutingProbeSuite, String> {
    // The deployed posture — Bench 167's control ModellessInput, verbatim.
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        gate_fit_selection: false,
        gate_distance_only: false,
        gate_fit_calibrated: true,
        gate_rho: None,
        kn_route: false,
        drafter_fix: crate::engine::DrafterFix::Off,
        suite: &prepared.suite,
        train: &prepared.train,
        state_strs: &prepared.state_strs,
        cal_cases: &prepared.cal_cases,
        cal_state_strs: &prepared.cal_state_strs,
        labels: &prepared.labels,
        want_by_type: false,
        corpus_cap_per_label: spec.corpus_cap_per_label,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        oc_select: false,
        ridge_select: false,
        genome_select: false,
        genome_accept_margin: 0.0,
        cap_source_base: "registry",
        cal_select_caps: &[],
        pool_rows: &prepared.pool_rows,
        pair_head_ab: false,
        nli_feature_ab: false,
        nli_m1: false,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: None,
        density_gate: false,
        set_refine: None,
        leak_flags: None,
    };
    let FittedPosture {
        effective_cap,
        default_cfg,
        score_threshold,
        distance_threshold,
        ..
    } = fit_posture_inner::<N>(&inp)?;
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };
    let (mut engine, fallbacks) = build_engine_with::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        cfg,
        &[],
    )?;
    if !fallbacks.is_empty() {
        eprintln!(
            "  [issue-039 corpus guard] {}: {} option label(s) self-doc fallback",
            spec.name,
            fallbacks.len()
        );
    }
    // The deployed calibration (the control's pass, verbatim).
    {
        let mut raw_scratch = Scratch::<EMBED_DIM>::new();
        raw_scratch.prepare(1);
        let mut pairs: Vec<(f32, bool)> = Vec::new();
        for cc in &prepared.cal_cases {
            let state_str = serialize_state(&cc.state);
            let req = engine_request(cc, &state_str)?;
            let resp = engine
                .decide_with(&req, &mut raw_scratch)
                .map_err(|e| format!("engine cal decide ({}): {e}", cc.id))?;
            for (qi, ans) in resp.answers.iter().enumerate() {
                let mut tmp: Vec<f64> = Vec::with_capacity(ans.probabilities.len());
                let pick = answer_probs_pick(cc.questions[qi].kind, ans, &mut tmp);
                let correct = cc.gold.get(qi).map(|g| pick == g.idx).unwrap_or(false);
                pairs.push((ans.confidence, correct));
            }
        }
        for (conf, correct) in pairs {
            engine.observe(conf, correct);
        }
    }

    let dirs = engine.routing_directions();
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(1);

    // ── FIT slice: the suite's cal cases (disjoint from the test split
    // by construction — different source splits; `prepare` guarantees
    // it). ──
    let (fit, _fit_skipped) = capture_slots(
        &mut engine,
        &mut sc,
        &prepared.cal_cases,
        &dirs,
        "fit",
        spec.name,
    )?;
    if fit.is_empty() {
        return Err(format!(
            "{}: no fit slots (cal slice empty of gold choice/score)",
            spec.name
        ));
    }

    // ── EVAL slice: stride over the choice/score test cases ──
    let probed: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.questions
                .iter()
                .any(|q| matches!(q.kind, QKind::Choice | QKind::Score))
        })
        .map(|(i, _)| i)
        .collect();
    if probed.is_empty() {
        return Err(format!("{}: no choice/score test questions", spec.name));
    }
    let sampled = stride_indices(probed.len(), opts.max_cases);
    let mut eval_cases: Vec<SuiteCase> = Vec::with_capacity(sampled.len());
    for &ci in &sampled {
        eval_cases.push(prepared.suite.cases[probed[ci]].clone());
    }
    let (eval, _eval_skipped) =
        capture_slots(&mut engine, &mut sc, &eval_cases, &dirs, "eval", spec.name)?;
    if eval.is_empty() {
        return Err(format!("{}: no eval slots", spec.name));
    }

    // ── capture-determinism witness: re-decide the first fit slots ──
    let witness_n = fit.len().min(10);
    let (wit, _) = capture_slots(
        &mut engine,
        &mut sc,
        &prepared.cal_cases[..witness_n],
        &dirs,
        "witness",
        spec.name,
    )?;
    let capture_witness_ok = wit.len() == witness_n
        && wit.iter().zip(fit.iter()).all(|(a, b)| {
            a.domain == b.domain
                && a.pick == b.pick
                && a.abstained == b.abstained
                && a.gold == b.gold
                && a.scores.len() == b.scores.len()
                && a.scores
                    .iter()
                    .zip(b.scores.iter())
                    .all(|(x, y)| x.to_bits() == y.to_bits())
        });

    // ── T1: the two arms at the gate λ + the score-arm λ grid ──
    let fit_x_score: Vec<Vec<f32>> = fit.iter().map(|s| s.scores.clone()).collect();
    let fit_x_hot: Vec<Vec<f32>> = fit
        .iter()
        .map(|s| {
            let mut v = vec![0.0f32; N];
            v[s.domain] = 1.0;
            v
        })
        .collect();
    let fit_gold: Vec<usize> = fit.iter().map(|s| s.gold).collect();
    let eval_x_score: Vec<Vec<f32>> = eval.iter().map(|s| s.scores.clone()).collect();
    let eval_x_hot: Vec<Vec<f32>> = eval
        .iter()
        .map(|s| {
            let mut v = vec![0.0f32; N];
            v[s.domain] = 1.0;
            v
        })
        .collect();
    let eval_gold: Vec<usize> = eval.iter().map(|s| s.gold).collect();

    let mut lambda_grid_rows = Vec::with_capacity(RIDGE_LAMBDA_GRID.len());
    let mut gate_preds: Option<Vec<usize>> = None;
    for &lambda in RIDGE_LAMBDA_GRID.iter() {
        let m = fit_ridge_ovr(&fit_x_score, &fit_gold, lambda, N);
        let mut by_class = vec![Vec::with_capacity(eval.len()); N];
        let mut preds = Vec::with_capacity(eval.len());
        for row in &eval_x_score {
            let scores = m.predict(row);
            preds.push(first_index_argmax(&scores));
            for (c, s) in scores.iter().enumerate() {
                by_class[c].push(*s);
            }
        }
        let acc = preds
            .iter()
            .zip(eval_gold.iter())
            .filter(|(p, g)| p == g)
            .count() as f64
            / eval.len() as f64;
        let (macro_v, included, _, _) = macro_auroc(&by_class, &eval_gold, &prepared.labels);
        if lambda == GATE_LAMBDA {
            gate_preds = Some(preds);
        }
        lambda_grid_rows.push(LambdaGridRow {
            lambda,
            acc,
            macro_auroc: macro_v,
            n_included: included,
        });
    }
    let eval_preds = gate_preds.unwrap_or_default();

    // ── the two arms' headline rows at the gate λ ──
    let mut arms = Vec::new();
    let mut score_row: Option<ProbeArmRow> = None;
    for (arm_name, x_fit, x_eval) in [
        ("score_vector", &fit_x_score, &eval_x_score),
        ("one_hot", &fit_x_hot, &eval_x_hot),
    ] {
        let m = fit_ridge_ovr(x_fit, &fit_gold, GATE_LAMBDA, N);
        let mut by_class = vec![Vec::with_capacity(eval.len()); N];
        let mut correct = 0usize;
        for (i, row) in x_eval.iter().enumerate() {
            let scores = m.predict(row);
            if first_index_argmax(&scores) == eval_gold[i] {
                correct += 1;
            }
            for (c, s) in scores.iter().enumerate() {
                by_class[c].push(*s);
            }
        }
        let (macro_v, included, n_excluded, examples) =
            macro_auroc(&by_class, &eval_gold, &prepared.labels);
        let degenerate = if arm_name == "score_vector" {
            score_arm_degenerate(x_eval)
        } else {
            one_hot_degenerate(&eval.iter().map(|s| s.domain).collect::<Vec<_>>())
        };
        let row = ProbeArmRow {
            arm: arm_name.to_string(),
            acc: correct as f64 / eval.len() as f64,
            macro_auroc: macro_v,
            n_included: included,
            n_excluded,
            excluded_examples: examples,
            degenerate,
        };
        if arm_name == "score_vector" {
            score_row = Some(row.clone());
        }
        arms.push(row);
    }
    let score_row = score_row.expect("score arm row exists");

    // ── Bench-167-comparable routing stats over the eval slots ──
    let (mi_bits, label_entropy_bits) = mi_and_entropy(&eval, N);
    let routed_forced_acc =
        eval.iter().filter(|s| s.domain == s.gold).count() as f64 / eval.len() as f64;
    let two_sigma =
        2.0 * (routed_forced_acc * (1.0 - routed_forced_acc) / eval.len() as f64).sqrt();

    // ── T1.4 verdict ──
    let degenerate = score_row.degenerate || arms[1].degenerate;
    let high_mi = mi_bits >= HIGH_MI_BITS;
    let (verdict, verdict_note) = if degenerate {
        (
            ProbeVerdict::Degenerate,
            "near-constant routing features — the loud-fail class; the probe row is \
             refused, never scored ~0.5 silently"
                .to_string(),
        )
    } else if high_mi {
        let d = score_row.acc - routed_forced_acc;
        if d.abs() <= two_sigma {
            (
                ProbeVerdict::AgreeWithMi,
                format!(
                    "HIGH-MI ({mi_bits:.4} bits): probe acc within 2σ of routed acc \
                     ({d:+.4} vs ±{two_sigma:.4}) — the linear probe recovers the \
                     lookup signal the MI predicts"
                ),
            )
        } else if d < 0.0 {
            (
                ProbeVerdict::Underread,
                format!(
                    "HIGH-MI ({mi_bits:.4} bits): probe acc {d:+.4} BELOW the 2σ band \
                     (±{two_sigma:.4}) — the linear probe underreads the lookup at \
                     this fit size (fit-slice capacity, not absent signal)"
                ),
            )
        } else {
            (
                ProbeVerdict::Overread,
                format!(
                    "HIGH-MI ({mi_bits:.4} bits): probe acc {d:+.4} ABOVE the 2σ band — \
                     the score vector carries label signal beyond the argmax (the T3 \
                     trigger class)"
                ),
            )
        }
    } else {
        match score_row.macro_auroc {
            None => (
                ProbeVerdict::Undecidable,
                format!(
                    "LOW-MI ({mi_bits:.4} bits): 0 classes clear the tail-support \
                     floor ({TAIL_SUPPORT_FLOOR}) — no macro AUROC statement exists"
                ),
            ),
            Some(a) if a > OVERREAD_AUROC => (
                ProbeVerdict::Overread,
                format!(
                    "LOW-MI ({mi_bits:.4} bits) but macro AUROC {a:.4} > \
                     {OVERREAD_AUROC:.2} — the score vector carries label-relevant \
                     signal BEYOND the routed argmax (the T3 trigger class)"
                ),
            ),
            Some(a) => (
                ProbeVerdict::AgreeWithMi,
                format!(
                    "LOW-MI ({mi_bits:.4} bits): macro AUROC {a:.4} ≤ \
                     {OVERREAD_AUROC:.2} — no hidden signal; the MI picture holds"
                ),
            ),
        }
    };

    // ── T2: extraction (fit slice only) + digest determinism ──
    let m_gate = fit_ridge_ovr(&fit_x_score, &fit_gold, GATE_LAMBDA, N);
    let fit_preds: Vec<usize> = fit_x_score
        .iter()
        .map(|r| first_index_argmax(&m_gate.predict(r)))
        .collect();
    let tables = extract_mode_tables(&fit, &fit_preds, N, N);
    let digest = tables_digest(&tables);
    let tables2 = extract_mode_tables(&fit, &fit_preds, N, N);
    let reextract_digest_match = tables_digest(&tables2) == digest;
    let suite_modal = tables.suite[&0i64];

    // ── T2.2: the three report-only pins (never pooled) ──
    let mut pins = Vec::with_capacity(3);
    {
        let modal = suite_modal;
        pins.push(replay_pin(
            &mut engine,
            &mut sc,
            &eval_cases,
            &eval,
            &eval_preds,
            |_, _| (modal, false),
            "suite_constant",
            "class-blind constant pin — every case forced to one domain (a named \
             baseline shape, not a result)",
            spec.name,
        )?);
    }
    {
        let predicted = tables.predicted.clone();
        let modal = suite_modal;
        let probe_acc = score_row.acc;
        pins.push(replay_pin(
            &mut engine,
            &mut sc,
            &eval_cases,
            &eval,
            &eval_preds,
            move |_, pred| match predicted.get(&(pred as i64)) {
                Some(&d) => (d, false),
                None => (modal, true),
            },
            "predicted_class",
            &format!(
                "ceiling = T1 probe accuracy ({probe_acc:.4}) — the pin replays what \
                 the probe predicts, never beyond it"
            ),
            spec.name,
        )?);
    }
    {
        let gold = tables.gold.clone();
        let modal = suite_modal;
        pins.push(replay_pin(
            &mut engine,
            &mut sc,
            &eval_cases,
            &eval,
            &eval_preds,
            move |g, _| match gold.get(&(g as i64)) {
                Some(&d) => (d, false),
                None => (modal, true),
            },
            "gold_oracle",
            "ORACLE CEILING ONLY — the key IS the gold label (leakage-bound); bounds \
             what a perfect classifier-keyed pin could achieve, never a \
             non-inferiority result",
            spec.name,
        )?);
    }

    Ok(RoutingProbeSuite {
        suite: spec.name.to_string(),
        n_fit: fit.len(),
        n_eval: eval.len(),
        n_domains: N,
        mi_bits,
        label_entropy_bits,
        routed_forced_acc,
        two_sigma,
        arms,
        lambda_grid_rows,
        verdict,
        verdict_note,
        pins,
        mode_path_digest: digest,
        reextract_digest_match,
        capture_witness_ok,
        suite_modal_domain: suite_modal,
    })
}

// ────────────────────────────────────────────── the N dispatch ──

macro_rules! with_probe_width {
    ($n:literal, $spec:expr, $prepared:expr, $opts:expr) => {
        routing_probe_suite::<$n>($spec, $prepared, $opts)
    };
}

/// Run the probe over the selected suites (default: Bench 167's four).
pub fn run_routing_state_probe(
    opts: &RunOptions,
    popts: &RoutingStateProbeOptions,
) -> Result<RoutingStateProbeOutput, String> {
    let mut suites = Vec::new();
    let mut prep_skipped: Vec<(String, String)> = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only || !DEFAULT_ROUTING_STATE_PROBE_SUITES.contains(&spec.name) {
            continue;
        }
        match prepare(spec, &opts.datasets_dir) {
            Ok(p) => {
                eprintln!("═══ suite {} (routing-state probe) ═══", spec.name);
                let rep = match p.labels.len() {
                    2 => with_probe_width!(2, spec, &p, popts),
                    3 => with_probe_width!(3, spec, &p, popts),
                    4 => with_probe_width!(4, spec, &p, popts),
                    5 => with_probe_width!(5, spec, &p, popts),
                    6 => with_probe_width!(6, spec, &p, popts),
                    7 => with_probe_width!(7, spec, &p, popts),
                    8 => with_probe_width!(8, spec, &p, popts),
                    10 => with_probe_width!(10, spec, &p, popts),
                    15 => with_probe_width!(15, spec, &p, popts),
                    59 => with_probe_width!(59, spec, &p, popts),
                    77 => with_probe_width!(77, spec, &p, popts),
                    n => Err(format!(
                        "no engine instantiation for {n} domains — extend the dispatch \
                         table in routing_state_probe.rs"
                    )),
                };
                match rep {
                    Ok(r) => suites.push(r),
                    Err(e) => prep_skipped.push((spec.name.to_string(), e)),
                }
            }
            Err(e) => prep_skipped.push((spec.name.to_string(), format!("prepare: {e}"))),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "routing-state probe: no suite produced a cell ({} request line(s)) — \
             check --suites",
            opts.suites.len()
        ));
    }
    if !prep_skipped.is_empty() {
        for (name, why) in &prep_skipped {
            eprintln!("  [routing-state probe skipped] {name}: {why}");
        }
    }
    let lane_probe_live = suites.iter().any(|s| {
        s.arms.first().map(|a| !a.degenerate).unwrap_or(false)
            && s.capture_witness_ok
            && s.reextract_digest_match
    });
    Ok(RoutingStateProbeOutput {
        meta: RoutingProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            max_cases: popts.max_cases,
            gate_lambda: GATE_LAMBDA,
            lambda_grid: RIDGE_LAMBDA_GRID.to_vec(),
            tail_support_floor: TAIL_SUPPORT_FLOOR,
            noninferior_delta: NONINFER_DELTA,
            measurement_posture: "deployed engine build (fit_posture_inner + \
                build_engine_with, selection levers pinned off, cal-slice observe \
                calibration); routing-state features re-derived at the routing site \
                (dot(embed, direction), the pick_domain fold) via the read-only \
                routing_directions accessor; ridge OVR closed-form (katgpt-core \
                ridge_solve_direct_f32, bias column, λ grid, gates at the \
                pre-nominated λ = 10.0); pins replay through the Scratch::forced_domain \
                seam (report-only; production paths disarm it)"
                .to_string(),
        },
        suites,
        lane_probe_live,
    })
}

/// The markdown render (the harness writes it beside the JSON).
#[allow(clippy::too_many_lines)]
pub fn render_routing_state_probe_markdown(out: &RoutingStateProbeOutput) -> String {
    let mut md = String::with_capacity(8192);
    md.push_str("# Issue 107 — reflex routing-state probe + pinned-path replay\n\n");
    md.push_str(&format!(
        "- date: {} · host: {} · sha: {}\n- datasets: {}\n- posture: {}\n- gates: λ = {} \
         (pre-nominated; the grid is disclosure) · tail floor {} · non-inferiority LB95 \
         > −{:.2}\n\n",
        out.meta.date_utc,
        out.meta.host,
        out.meta.git_sha.as_deref().unwrap_or("unknown"),
        out.meta.datasets_dir,
        out.meta.measurement_posture,
        out.meta.gate_lambda,
        out.meta.tail_support_floor,
        out.meta.noninferior_delta,
    ));
    md.push_str(
        "## T1 — the supervised probe (score-vector arm MANDATORY; one-hot = MI consistency)\n\n",
    );
    md.push_str("| suite | fit | eval | N | MI bits | H(label) | routed acc | probe acc (score) | Δ | 2σ | macro AUROC (score) | incl/excl | verdict |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for s in &out.suites {
        let score = s
            .arms
            .iter()
            .find(|a| a.arm == "score_vector")
            .expect("score arm row");
        md.push_str(&format!(
            "| {} | {} | {} | {} | {:.4} | {:.4} | {:.4} | {:.4} | {:+.4} | {:.4} | {} | {}/{} | {} |\n",
            s.suite,
            s.n_fit,
            s.n_eval,
            s.n_domains,
            s.mi_bits,
            s.label_entropy_bits,
            s.routed_forced_acc,
            score.acc,
            score.acc - s.routed_forced_acc,
            s.two_sigma,
            score
                .macro_auroc
                .map(|a| format!("{a:.4}"))
                .unwrap_or_else(|| "—".to_string()),
            score.n_included,
            score.n_excluded,
            s.verdict.str_of(),
        ));
    }
    md.push_str("\n### Arms at the gate λ\n\n| suite | arm | acc | macro AUROC | included | excluded | degenerate |\n|---|---|---|---|---|---|---|\n");
    for s in &out.suites {
        for a in &s.arms {
            md.push_str(&format!(
                "| {} | {} | {:.4} | {} | {} | {} | {} |\n",
                s.suite,
                a.arm,
                a.acc,
                a.macro_auroc
                    .map(|v| format!("{v:.4}"))
                    .unwrap_or_else(|| "—".to_string()),
                a.n_included,
                a.n_excluded,
                if a.degenerate { "YES" } else { "no" },
            ));
        }
    }
    md.push_str("\n### λ grid (score-vector arm — disclosure; gates never read the argmax-λ row)\n\n| suite | λ | acc | macro AUROC | included |\n|---|---|---|---|---|\n");
    for s in &out.suites {
        for r in &s.lambda_grid_rows {
            md.push_str(&format!(
                "| {} | {} | {:.4} | {} | {} |\n",
                s.suite,
                fmt_lambda(r.lambda),
                r.acc,
                r.macro_auroc
                    .map(|v| format!("{v:.4}"))
                    .unwrap_or_else(|| "—".to_string()),
                r.n_included,
            ));
        }
    }
    md.push_str("\n### T1.4 verdict notes\n\n");
    for s in &out.suites {
        md.push_str(&format!("- **{}**: {}\n", s.suite, s.verdict_note));
    }
    md.push_str("\n## T2 — mode-path pins (report-only; the forced_domain seam stays production-disarmed)\n\n");
    md.push_str("| suite | pin | paired | live | pinned | Δ | LB95 | verdict | abst Δ | fallback |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for s in &out.suites {
        for p in &s.pins {
            md.push_str(&format!(
                "| {} | {} | {} | {:.4} | {:.4} | {:+.4} | {:+.4} | {} | {:+.3} | {} |\n",
                s.suite,
                p.kind,
                p.n_paired,
                p.acc_live,
                p.acc_pinned,
                p.delta,
                p.lb95,
                pin_verdict_display(p),
                p.abstain_pinned - p.abstain_live,
                p.fallback_count,
            ));
        }
    }
    md.push_str("\n### Pin ceilings\n\n");
    for s in &out.suites {
        for p in &s.pins {
            md.push_str(&format!(
                "- **{} / {}**: {}\n",
                s.suite, p.kind, p.ceiling_note
            ));
        }
    }
    md.push_str("\n### Witnesses\n\n");
    for s in &out.suites {
        md.push_str(&format!(
            "- **{}**: capture determinism {} · extraction digest {} (`{}`) · suite modal \
             domain {}\n",
            s.suite,
            bool_word(s.capture_witness_ok),
            bool_word(s.reextract_digest_match),
            s.mode_path_digest,
            s.suite_modal_domain,
        ));
    }
    md.push_str(&format!(
        "\n**Lane verdict: {}** — the probe lane is live when any suite's score-vector arm \
         is non-degenerate with green witnesses.\n",
        if out.lane_probe_live {
            "LIVE"
        } else {
            "RED (instrument-blind — refuse)"
        }
    ));
    md.push_str(
        "\nScope notes: the score vector is re-derived with `pick_domain`'s exact fold from \
         `Scratch.q` + `routing_directions()` and per-slot asserted against the engine's \
         routed domain. The one-hot arm MUST agree with the MI picture (a linear readout \
         over one-hot inputs is the per-domain lookup) — agreement is the deliverable. \
         Bench 157's depth-as-confidence NEGATIVE is a different feature source (residual \
         stream, not routing state) and is not re-run here. NO promotion: the seam stays \
         disarmed on production paths exactly as Bench 167 left it (T2.3).\n",
    );
    md
}

fn pin_verdict_display(p: &PinRow) -> &'static str {
    if p.n_paired < 2 {
        "UNDECIDABLE"
    } else if p.noninferior {
        "NON-INFERIOR"
    } else {
        "INFERIOR"
    }
}

fn fmt_lambda(lambda: f32) -> String {
    if lambda >= 1.0 {
        format!("{lambda:.0}")
    } else {
        format!("{lambda}")
    }
}

fn bool_word(b: bool) -> &'static str {
    if b { "GREEN" } else { "RED" }
}

// ─────────────────────────────────────────────────────────── tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ridge_separates_a_linearly_separable_synthetic() {
        let mut x = Vec::new();
        let mut gold = Vec::new();
        for _ in 0..20 {
            x.push(vec![1.0, 0.0, 0.0]);
            gold.push(0);
        }
        for _ in 0..20 {
            x.push(vec![0.0, 1.0, 0.0]);
            gold.push(1);
        }
        for _ in 0..20 {
            x.push(vec![-1.0, -1.0, 0.0]);
            gold.push(2);
        }
        let m = fit_ridge_ovr(&x, &gold, 10.0, 3);
        for (row, &g) in x.iter().zip(&gold) {
            assert_eq!(first_index_argmax(&m.predict(row)), g);
        }
    }

    #[test]
    fn auroc_perfect_flipped_and_undecidable() {
        let golds = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let scores: Vec<f32> = (0..20).map(|i| i as f32).collect();
        let (a, np, nn) = auroc_ovr(&scores, &golds, 1).expect("decidable");
        assert_eq!((np, nn), (10, 10));
        assert!((a - 1.0).abs() < 1e-9);
        let flipped: Vec<f32> = (0..20).map(|i| (19 - i) as f32).collect();
        let (a2, _, _) = auroc_ovr(&flipped, &golds, 1).expect("decidable");
        assert!((a2 - 0.0).abs() < 1e-9);
        // No positives → UNDECIDABLE (None), never a fake 0.5.
        let thin = [1usize; 20];
        assert!(auroc_ovr(&scores, &thin, 0).is_none());
    }

    #[test]
    fn mode_path_digest_is_deterministic_and_input_sensitive() {
        let fit = vec![
            CapSlot {
                scores: vec![],
                domain: 2,
                pick: 0,
                abstained: false,
                gold: 0,
            },
            CapSlot {
                scores: vec![],
                domain: 2,
                pick: 0,
                abstained: false,
                gold: 0,
            },
            CapSlot {
                scores: vec![],
                domain: 2,
                pick: 0,
                abstained: false,
                gold: 1,
            },
            CapSlot {
                scores: vec![],
                domain: 5,
                pick: 0,
                abstained: false,
                gold: 1,
            },
        ];
        let preds = vec![0, 0, 1, 1];
        let t1 = extract_mode_tables(&fit, &preds, 6, 2);
        let t2 = extract_mode_tables(&fit, &preds, 6, 2);
        assert_eq!(tables_digest(&t1), tables_digest(&t2));
        // Suite modal: domains {2, 2, 2, 5} → 2.
        assert_eq!(t1.suite[&0i64], 2);
        // Predicted/gold keyed modes follow the same argmax rule.
        assert_eq!(t1.predicted[&0i64], 2);
        assert_eq!(t1.gold[&1i64], 2); // gold 1 saw domains {2, 5} — 2 first-wins
        // A different input that MOVES A MODAL OUTCOME must move the digest
        // (gold 1's domains {2,5} → {5,5}: its modal flips 2 → 5; the suite
        // modal can legitimately stay 2 — the argmax collapses non-modal
        // count changes, and that is by design).
        let fit_shift = vec![
            CapSlot {
                scores: vec![],
                domain: 2,
                pick: 0,
                abstained: false,
                gold: 0,
            },
            CapSlot {
                scores: vec![],
                domain: 2,
                pick: 0,
                abstained: false,
                gold: 0,
            },
            CapSlot {
                scores: vec![],
                domain: 5,
                pick: 0,
                abstained: false,
                gold: 1,
            },
            CapSlot {
                scores: vec![],
                domain: 5,
                pick: 0,
                abstained: false,
                gold: 1,
            },
        ];
        let t3 = extract_mode_tables(&fit_shift, &preds, 6, 2);
        assert_eq!(t3.predicted[&1i64], 5);
        assert_eq!(t3.gold[&1i64], 5);
        assert_ne!(tables_digest(&t3), tables_digest(&t1));
    }

    #[test]
    fn first_index_argmax_matches_pick_domain_tie_rule() {
        const A: usize = 4;
        // Exact tie between domains 0 and 1 — BOTH spellings must pick 0.
        let q = [1.0f32, 0.5, 0.0, 0.0];
        let dirs: [[f32; A]; 3] = [
            [1.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
        ];
        assert_eq!(first_index_argmax(&[1.0, 1.0, 0.5]), 0);
        assert_eq!(pick_domain::<3, A>(&q, &dirs), 0);
        // Distinct winners agree too.
        let dirs2: [[f32; A]; 3] = [
            [0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        assert_eq!(pick_domain::<3, A>(&q, &dirs2), 1);
        assert_eq!(first_index_argmax(&router_scores_of(&q, &dirs2)), 1);
        // The fold reproduces pick_domain's own scores bit-exactly on a
        // no-tie input: re-deriving its argmax from OUR scores picks the
        // same domain.
        let ours = router_scores_of(&q, &dirs2);
        let mut theirs = [0.0f32; 3];
        for (d, dir) in dirs2.iter().enumerate() {
            let mut s = 0.0f32;
            for i in 0..A {
                s += q[i] * dir[i];
            }
            theirs[d] = s;
        }
        assert_eq!(ours.len(), theirs.len());
        assert!(
            ours.iter()
                .zip(theirs.iter())
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
    }

    #[test]
    fn degenerate_detectors_fire() {
        let x = vec![vec![2.0, 2.0], vec![2.0, 2.0], vec![2.0, 2.0]];
        assert!(score_arm_degenerate(&x));
        let x2 = vec![vec![2.0, 0.0], vec![0.0, 2.0]];
        assert!(!score_arm_degenerate(&x2));
        assert!(one_hot_degenerate(&[3, 3, 3]));
        assert!(!one_hot_degenerate(&[3, 3, 4]));
        assert!(one_hot_degenerate(&[]));
    }
}
