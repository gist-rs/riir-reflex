//! Plan 426 T5 — the coverage-directed corpus synthesis lane (V5/N2/N3).
//!
//! WHAT: grow the modelless lane's corpus with DETERMINISTIC
//! template×slot expansions of the suite's own TRAIN rows, aimed at the
//! measured coverage holes, filtered by the strongest teacher's agreement
//! — never labeled by it.
//!
//! The generator is CROSS-FRAME SPAN TRANSPLANTATION. Mining attests
//! slot frames per label: two rows sharing a non-empty prefix + suffix
//! (the frame) whose differing middle spans are ≤ `max_span_len` tokens —
//! e.g. "wake me up at ⟦nine am|six pm⟧ on friday". Each frame carries
//! its own attested spans; every span attested ANYWHERE in the label is a
//! transplant candidate into every OTHER frame of that label. A candidate
//! is a frame attested in the label composed with a span attested in the
//! label — locally natural by construction, occasionally odd in
//! composition, which is exactly what the veto is FOR (N3): the teacher
//! removes what reads wrong; it never writes anything.
//!
//! The three plan clauses this lane implements:
//! - **N2 — E0-directed synthesis.** Per-intent evidence density (the E0
//!   rumor fraction over each label's OWN count tables, bag view) weights
//!   the per-label synthesis budget — targeted coverage, never blind
//!   scaling. Without the `nb_scope` feature the weights degrade to
//!   uniform, disclosed (the artifact meta names the rule actually used).
//! - **N3 — teacher as veto, never as labeler.** The teacher generates
//!   nothing (openthai is a discriminative slot head). Gold labels are
//!   CARRIED from the source label; a candidate survives iff the
//!   teacher's argmax over the FULL label universe equals the carried
//!   gold. A teacher forward failure aborts the run loud — never a
//!   half-vetoed artifact.
//! - **C1/C2/C3.** Seeds come ONLY from the train-split pool (the same
//!   corpus pool the engine builds from — the cal front is excluded
//!   twice: never seeded from, and generated texts equal to a cal text
//!   are dropped); gold is never replaced (the veto only removes); every
//!   artifact row carries `provenance:"synth"` and the A/B lane reports
//!   gold-only vs +synth separately.
//!
//! Artifact (corpus-artifact generation v2 — v1 is the fetched dataset
//! envelope): one JSONL per suite, header line (magic `SYNT`, version 2,
//! full provenance + per-label stats), then one row per accepted
//! candidate (`{"text","label","provenance","src","span"}`), with a
//! `.blake3` sidecar (the RIDT discipline). The loader verifies magic,
//! version, provenance, row count, and the sidecar digest — a corpus
//! whose bytes do not match its seal is refused, never loaded.
//!
//! Scope v1: single-question Choice/Score suites whose state is an
//! object with exactly ONE string field (massive is the board; the lane
//! refuses anything else loud). `--synth-plan` runs the whole pipeline
//! except the veto and writes nothing — inspect the allocation before
//! spending teacher calls.
//!
//! The rescue pre-filter (riir-refine Issue 156 T4 — the FlyBy state-level
//! predicate, arXiv:2609.34327 Algorithm 1, synth-lane face): OPT-IN ahead
//! of the veto, the local modelless engine answers each candidate as the
//! `b0` (no-tool) continuation — the hybrid's free arm, read at the FORCED
//! pick (the hard-metric convention: `forced accuracy, abstain ignored`;
//! the deployed gate's abstention is a serve-confidence posture the
//! per-suite fit owns, not a knowledge axis). A candidate whose scoring
//! already ranks gold top-1 is execution-reachable (FlyBy's `V>0` law) and
//! is rejected WITHOUT a teacher forward: not a knowledge bottleneck, not
//! worth corpus budget. The `bd` (rescue-evidence) leg stays the VETO's own
//! job — one cross-model teacher agreement is stronger evidence than the
//! paper's ≥3 same-model continuations, so the veto remains the acceptance
//! AUTHORITY and this filter can only NARROW what enters it (the augment
//! law). The canonical predicate home is riir-refine `src/rescue_mine.rs`
//! (private repo, paper-derived); this public port is pinned to the same
//! FlyBy truth table by tests — the copies cannot drift.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use super::{
    density_pilot::{DensityGate, DensityRung},
    distill::{argmax_lowest_pos, construct_teacher},
    git_sha, hostname_refusing_unknown, iso8601_utc, percentile_us, prepare, Prepared,
    RunOptions, SuiteSpec, SUITES,
};
use crate::harness::suites::{GoldAnswer, QKind, Suite, SuiteCase, SuiteQuestion, TrainDoc};
#[cfg(feature = "nb_scope")]
use super::e0::RUMOR_N;
#[cfg(feature = "nb_scope")]
use crate::nb_scope::{NbAlpha, NbScope, NbView, view_tokens_into};

/// The corpus-artifact v2 magic + version (see the module doc — v1 is the
/// fetched dataset envelope, unversioned; v2 is the provenance-row
/// synthesis generation).
pub const CORPUS_MAGIC: &str = "SYNT";
pub const CORPUS_VERSION: u32 = 2;

/// The provenance token every artifact row must carry (C3).
pub const SYNTH_PROVENANCE: &str = "synth";

/// Spans considered per frame, in sorted order, beyond the frame's own
/// attested set — the transplant bound that keeps a label with one huge
/// span pool from flooding its buckets (the allocation caps what the
/// veto forwards; this caps what generation holds).
const MAX_SPANS_PER_FRAME: usize = 48;

/// The synth mode's knobs (the bin parses; the lane consumes).
#[derive(Debug, Clone)]
pub struct SynthOptions {
    /// The veto teacher (`openthai` default — the V2-qualified 0.92
    /// teacher; `laya` needs the `laya-riir` feature, refused by
    /// `construct_teacher`'s own law).
    pub teacher: String,
    /// Global cap on ACCEPTED rows (the veto early-stops there).
    pub max_accepted: usize,
    /// Per-label cap on accepted rows (the allocation's ceiling).
    pub max_per_label: usize,
    /// Longest substitutable span, in tokens.
    pub max_span_len: usize,
    /// Output directory for the artifact + sidecar.
    pub out_dir: std::path::PathBuf,
    /// The Issue-064 ascent leg, OPT-IN: the minimal-deviation density gate
    /// AND-ed with the teacher veto (acceptance can only REJECT, never
    /// inject). `None` = the shipped veto-only posture, byte-identical.
    pub density_gate: Option<DensityRung>,
    /// The rescue pre-filter (riir-refine Issue 156 T4), OPT-IN: the FlyBy
    /// `b0` leg — the LOCAL modelless engine answers each candidate BEFORE
    /// the teacher forward (forced pick, the hard-metric convention), and a
    /// candidate whose scoring already ranks gold top-1 is rejected without
    /// spending the veto call. AUGMENTS the veto, never replaces it: the
    /// veto stays the acceptance authority and the pre-filter can only
    /// NARROW the candidate set entering it. `false` = the shipped posture,
    /// byte-identical.
    pub rescue_prefilter: bool,
}

/// One generated candidate (pre-veto). `src` is the LABEL-LOCAL pool
/// row index whose pair first attested the FRAME (the provenance anchor —
/// label-local because mining iterates the label's own rows); `span` is
/// `(prefix_len, span_len)` in tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SynthCand {
    pub text: String,
    pub label: String,
    pub src: usize,
    pub span: (usize, usize),
}

/// Per-label synthesis outcome (the report + the artifact header's
/// per-label block).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SynthPerLabel {
    pub candidates: usize,
    pub allocated: usize,
    pub accepted: usize,
    pub veto_rejected: usize,
    /// Rows the minimal-deviation density gate rejected BEFORE the teacher
    /// forward (density-first, the order-invariant AND: a rejected row never
    /// spends a teacher call; the accepted set is identical either order).
    /// 0 unless the gate is armed — Issue 064 task 2.
    #[serde(default)]
    pub density_rejected: usize,
    /// Rows the rescue pre-filter rejected BEFORE the teacher forward (the
    /// `b0` leg: the local modelless engine already serves the candidate —
    /// answered, not abstained, and correct). 0 unless the pre-filter is
    /// armed — riir-refine Issue 156 T4.
    #[serde(default)]
    pub rescue_rejected: usize,
    pub weight: f64,
}

/// One suite's synthesis record (the run report row).
#[derive(Debug, serde::Serialize)]
pub struct SynthSuite {
    pub name: String,
    pub teacher: String,
    pub teacher_provenance: String,
    pub weighting_rule: String,
    pub pool_rows: usize,
    pub candidates_total: usize,
    pub accepted: usize,
    pub veto_rejected: usize,
    pub dedup_pool: usize,
    pub dedup_cal: usize,
    pub dedup_dup: usize,
    pub outside_universe: usize,
    pub veto_forwards: usize,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub seconds: f64,
    pub path: String,
    /// First 16 hex of the artifact's BLAKE3 (the sidecar carries the
    /// full one).
    pub blake3: String,
    /// The ascent-leg disclosure (mirrors the artifact meta; `None` =
    /// veto-only).
    pub density_rule: Option<String>,
    /// Rows the rescue pre-filter rejected before the teacher forward,
    /// summed over labels. 0 unless the pre-filter is armed.
    pub rescue_rejected: usize,
    /// The rescue pre-filter disclosure (mirrors the artifact meta;
    /// `None` = veto-only).
    pub rescue_rule: Option<String>,
    pub per_label: BTreeMap<String, SynthPerLabel>,
}

#[derive(Debug, serde::Serialize)]
pub struct SynthMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub host: String,
    pub datasets_dir: String,
    pub max_accepted: usize,
    pub max_per_label: usize,
    pub max_span_len: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct SynthOutput {
    pub meta: SynthMeta,
    pub suites: Vec<SynthSuite>,
    pub skipped: Vec<String>,
}

/// The artifact header, typed for the loader (the A/B lane reads the
/// suite + provenance off it).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SynthArtifactMeta {
    pub magic: String,
    pub version: u32,
    pub suite: String,
    pub teacher: String,
    pub teacher_provenance: String,
    pub weighting_rule: String,
    pub created_utc: String,
    pub git_sha: String,
    pub host: String,
    pub accepted: usize,
    pub pool_rows: usize,
    /// The Issue-064 ascent-leg disclosure: `None` = veto-only (the
    /// shipped posture); `Some(rule)` = the density gate was armed.
    /// Additive + `serde(default)` — version stays 2 (the ROWS' semantics
    /// are unchanged; the field says how the set was filtered, the way
    /// `weighting_rule` already does).
    #[serde(default)]
    pub density_rule: Option<String>,
    /// The rescue pre-filter disclosure (riir-refine Issue 156 T4): `None`
    /// = veto-only (the shipped posture); `Some(rule)` = the FlyBy `b0`
    /// pre-filter was armed ahead of the teacher veto. Additive +
    /// `serde(default)` — version stays 2 for the same reason as
    /// [`Self::density_rule`].
    #[serde(default)]
    pub rescue_rule: Option<String>,
}

// ── the veto shape ───────────────────────────────────────────────────────

/// Everything a synthetic veto case needs, cloned once from the suite's
/// first case: the state's single string FIELD name, the question's
/// qid/instructions/kind, and (Score suites) the verbatim criteria array.
struct VetoShape {
    field: String,
    qid: String,
    instructions: String,
    kind: QKind,
    score_criteria: Option<serde_json::Value>,
}

impl VetoShape {
    fn from_suite(suite: &Suite) -> Result<Self, String> {
        let case = suite
            .cases
            .first()
            .ok_or_else(|| "suite has no cases — cannot clone the question shape".to_string())?;
        let q = case.questions.first().ok_or_else(|| {
            format!("{}: no questions for its gold — not a single-question suite", case.id)
        })?;
        if suite.cases.iter().any(|c| c.questions.len() != 1) {
            return Err("multi-question suite — the synth lane speaks one gold per row".into());
        }
        let kind = match q.kind {
            k @ (QKind::Choice | QKind::Score) => k,
            QKind::Noul => {
                return Err("noul suite — authored-corpus lane, synthesis does not apply".into());
            }
        };
        let score_criteria = match kind {
            QKind::Score => Some(q.criteria.clone()),
            _ => None,
        };
        Ok(Self {
            field: single_text_field(&case.state).ok_or_else(|| {
                "state is not a single-string-field object — unsupported v1 (loud)".to_string()
            })?,
            qid: q.qid.clone(),
            instructions: q.instructions.clone(),
            kind,
            score_criteria,
        })
    }
}

/// The state object's single string field name — `None` unless the state
/// is an object with EXACTLY one field and that field is a string.
fn single_text_field(state: &serde_json::Value) -> Option<String> {
    let m = state.as_object()?;
    if m.len() != 1 {
        return None;
    }
    let (k, v) = m.iter().next()?;
    v.is_string().then(|| k.clone())
}

/// Build one synthetic veto case: the shape's question with the candidate
/// text in the state field and the FULL label universe as criteria (the
/// deterministic presentation — no arbitrary distractor draw). Returns
/// the case + the gold KEY STRING the accept test compares (order-free)
/// + the gold POSITION (index-space, the b0 judge's compare).
fn veto_case(
    shape: &VetoShape,
    labels_sorted: &[String],
    gold_label: &str,
    text: &str,
    id: String,
) -> Result<(SuiteCase, String, usize), String> {
    let (criteria, gold_pos, n_keys) = match shape.kind {
        QKind::Choice => {
            let pos = labels_sorted
                .iter()
                .position(|l| l == gold_label)
                .ok_or_else(|| format!("gold label {gold_label:?} outside the universe"))?;
            let mut m = serde_json::Map::new();
            for l in labels_sorted {
                m.insert(l.clone(), serde_json::Value::String(l.clone()));
            }
            (serde_json::Value::Object(m), pos, labels_sorted.len())
        }
        QKind::Score => {
            let arr = shape
                .score_criteria
                .as_ref()
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| "score criteria is not an array".to_string())?
                .clone();
            let pos: usize = gold_label
                .parse()
                .map_err(|_| format!("score gold label {gold_label:?} is not an index"))?;
            let n = arr.len();
            if pos >= n {
                return Err(format!("score gold {pos} outside {n} levels"));
            }
            (serde_json::Value::Array(arr), pos, n)
        }
        QKind::Noul => unreachable!("VetoShape refuses noul"),
    };
    let case = SuiteCase {
        id,
        state: serde_json::json!({ shape.field.as_str(): text }),
        questions: vec![SuiteQuestion {
            qid: shape.qid.clone(),
            kind: shape.kind,
            instructions: shape.instructions.clone(),
            criteria,
        }],
        gold: vec![GoldAnswer {
            idx: gold_pos,
            soft: vec![0.0; n_keys],
            gold_score: None,
        }],
    };
    Ok((case, gold_label.to_string(), gold_pos))
}

/// The veto predicate (N3): the teacher's argmax KEY must equal the gold
/// label string — position-free, so the criteria iteration order can
/// never flip a verdict. Ties break to the lowest position (the engine
/// argmax law).
fn veto_accept(keyed: &[(String, f64)], gold_key: &str) -> bool {
    let best = argmax_lowest_pos(keyed);
    keyed.get(best).is_some_and(|(k, _)| k == gold_key)
}

// ── the rescue pre-filter (riir-refine Issue 156 T4) ─────────────────────

/// The tool-assisted success bar — `FlyBy`'s `bd ≥ 3` of 4 (a 3-of-4 rescue
/// majority; `(0, 2)` is REJECT: the rescue is undemonstrated).
pub(crate) const RESCUE_BD_KEEP_MIN: u32 = 3;

/// The `FlyBy` probe's per-arm continuation count (the paper's 4/4 design —
/// documentation + the honest denominator for yield math). The synth lane's
/// `b0` leg is ONE continuation because the modelless engine is
/// DETERMINISTIC: one sample is the complete enumeration of its continuation
/// distribution, not an estimate of it.
#[allow(dead_code)] // pinned by the truth-table test; documents the paper denominator the b0 leg collapses
pub(crate) const FLYBY_CONTINUATIONS_PER_ARM: u32 = 4;

/// The `FlyBy` rescue predicate on success COUNTS: zero no-tool successes
/// AND at least [`RESCUE_BD_KEEP_MIN`] tool-assisted successes. Ported from
/// riir-refine `src/rescue_mine.rs` (the canonical home, paper-derived);
/// the truth-table test below pins both copies to the same law.
///
/// Synth-lane mapping (the augment law): the `b0` leg is measured LOCALLY
/// (the modelless engine, one deterministic continuation); the `bd` leg is
/// the VETO's own job — one cross-model teacher agreement is stronger
/// rescue evidence than ≥3 same-model continuations, and the veto stays
/// the acceptance authority. The pre-filter therefore enforces only the
/// `b0 = 0` half (see [`B0Judge::already_serves`]) and can only NARROW the
/// candidate set entering the veto — never widen acceptance.
#[must_use]
#[allow(dead_code)] // pinned by the truth-table test; the production pre-filter enforces the b0 half — this full predicate ships for a future LOCAL bd leg (refine's arm-H consumes its twin)
#[allow(clippy::similar_names)] // b0/bd ARE the FlyBy vocabulary — the near-identical names are the domain's own, not a slip
pub(crate) fn rescue_keep(b0_successes: u32, bd_successes: u32) -> bool {
    b0_successes == 0 && bd_successes >= RESCUE_BD_KEEP_MIN
}

/// The `b0` (no-tool) judge: the local modelless engine over the suite's
/// own train pool (the registry corpus cap — the incumbent serving
/// posture's cap, `specs_corpus_extended` with an empty extension so the
/// construction is the SAME seam the corpus A/B lane measured), at the
/// FORCED posture (`score/distance thresholds = 2.0` — conf ≤ 1 < 2, the
/// `build_head_scale_selection` precedent: never abstains). One
/// continuation per candidate; the forced pick comes back through the
/// harness's own `eval_engine` (one source of truth for the decide law —
/// never a parallel scoring path).
///
/// Why the FORCED pick and not the deployed gate's served/abstained split:
/// the bottleneck the corpus lane mines is a KNOWLEDGE gap — whether the
/// engine's scoring already ranks gold top-1 (the hard-metric convention,
/// `forced accuracy, abstain ignored`, every lane table reports). The
/// deployed gate's abstention is a serve-CONFIDENCE posture the per-suite
/// fit owns (`fit_posture`'s calibrated thresholds); a cold judge cannot
/// reproduce it faithfully, and a mis-fit judge would only blur the
/// filter — while the forced pick is deterministic, posture-free, and
/// exactly the axis FlyBy's `V>0` law names (the no-tool continuation
/// SUCCEEDS = the task is execution-reachable, period).
type ContinuationFn = Box<dyn FnMut(&SuiteCase, &str) -> Result<usize, String>>;

pub(crate) struct B0Judge {
    run: ContinuationFn,
}

impl B0Judge {
    /// One `b0` continuation: the FORCED pick, INDEX-space (criteria order)
    /// — the same convention the eval lanes' hard metrics read.
    fn continuation(&mut self, case: &SuiteCase, state_str: &str) -> Result<usize, String> {
        (self.run)(case, state_str)
    }

    /// The `b0` success test — the no-tool arm's scoring already ranks gold
    /// top-1 (`FlyBy`'s `V>0`: execution-reachable). `true` = the
    /// pre-filter rejects the candidate without a teacher forward.
    fn already_serves(
        &mut self,
        case: &SuiteCase,
        state_str: &str,
        gold_pos: usize,
    ) -> Result<bool, String> {
        Ok(self.continuation(case, state_str)? == gold_pos)
    }
}

/// Build the `b0` judge for one suite — the run() const-generic dispatch
/// table over the domain count (the run() law; a new label count extends
/// THIS table and that one together).
fn build_b0_judge(suite: &str, corpus_cap: usize, prepared: &Prepared) -> Result<B0Judge, String> {
    macro_rules! dispatch {
        ($n:literal) => {{
            let (specs, fallback, _) = super::specs_corpus_extended(
                &prepared.train,
                &[],
                &prepared.labels,
                corpus_cap,
                0,
                None,
            );
            if !fallback.is_empty() {
                eprintln!(
                    "  [synth {suite}] rescue pre-filter: {} starved label(s) serve their \
                     own label text as corpus (the Issue-039 disclosure)",
                    fallback.len()
                );
            }
            let mut engine = crate::engine::DecisionEngine::<$n, { crate::embed::EMBED_DIM }>::
                build_specs(
                    specs,
                    crate::engine::EngineConfig {
                        // Forced: never abstain (conf ≤ 1 < threshold) — the
                        // b0 leg reads the scoring axis, the hard-metric
                        // convention (see the B0Judge doc).
                        score_threshold: 2.0,
                        distance_threshold: 2.0,
                        ..crate::engine::EngineConfig::default()
                    },
                )
                .map_err(|e| format!("rescue pre-filter engine build ({suite}): {e}"))?;
            Ok(B0Judge {
                run: Box::new(move |case: &SuiteCase, state_str: &str| {
                    let strs = [state_str.to_string()];
                    let (ev, _) = super::eval_engine::<$n>(
                        &mut engine,
                        std::slice::from_ref(case),
                        &strs,
                        false,
                    )?;
                    Ok(ev.picks[0][0])
                }),
            })
        }};
    }
    match prepared.labels.len() {
        2 => dispatch!(2),
        3 => dispatch!(3),
        4 => dispatch!(4),
        5 => dispatch!(5),
        6 => dispatch!(6),
        7 => dispatch!(7),
        8 => dispatch!(8),
        10 => dispatch!(10),
        15 => dispatch!(15),
        59 => dispatch!(59),
        77 => dispatch!(77),
        842 => dispatch!(842),
        n => Err(format!(
            "synth rescue pre-filter: no engine instantiation for {n} domains — extend the \
             dispatch table (the run() law)",
        )),
    }
}

// ── mining + generation ─────────────────────────────────────────────────

/// Lowercase whitespace-normalized key (the dedup identity for texts).
fn norm_key(text: &str) -> String {
    text.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Common prefix length + common suffix length (case-insensitive token
/// compare; original case is preserved for reconstruction).
fn common_affix(a: &[String], b: &[String]) -> (usize, usize) {
    let mut p = 0;
    while p < a.len() && p < b.len() && a[p].eq_ignore_ascii_case(&b[p]) {
        p += 1;
    }
    let mut s = 0;
    while s < a.len() - p && s < b.len() - p && a[a.len() - 1 - s].eq_ignore_ascii_case(&b[b.len() - 1 - s])
    {
        s += 1;
    }
    (p, s)
}

/// Mine span slots: context frame `(prefix, suffix)` → the spans attested
/// in it (sorted set) + the pool index of the row whose pair first
/// attested the frame (the provenance anchor). Deterministic: BTree
/// iteration everywhere.
type SlotMap = BTreeMap<(Vec<String>, Vec<String>), (BTreeSet<Vec<String>>, usize)>;

fn mine_slots(rows: &[Vec<String>], max_span: usize) -> SlotMap {
    let mut slots: SlotMap = BTreeMap::new();
    for i in 0..rows.len() {
        for j in (i + 1)..rows.len() {
            let (a, b) = (&rows[i], &rows[j]);
            let (p, s) = common_affix(a, b);
            // Both middle spans must be non-empty, distinct, and within
            // the span bound (a shared prefix+suffix covering a row
            // leaves an empty span — not a slot).
            let sa = &a[p..a.len() - s];
            let sb = &b[p..b.len() - s];
            if sa.is_empty() || sb.is_empty() || sa == sb {
                continue;
            }
            if sa.len() > max_span || sb.len() > max_span {
                continue;
            }
            let frame = (a[..p].to_vec(), a[a.len() - s..].to_vec());
            let entry = slots.entry(frame).or_insert_with(|| (BTreeSet::new(), i));
            entry.0.insert(sa.to_vec());
            entry.0.insert(sb.to_vec());
        }
    }
    slots
}

/// The dedup counters (named — the report discloses each).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct DedupCounts {
    pool: usize,
    cal: usize,
    dup: usize,
    outside: usize,
}

/// The per-suite plan: candidate buckets, evidence weights, allocation.
pub(crate) struct SuitePlan {
    pub buckets: BTreeMap<String, Vec<SynthCand>>,
    pub alloc: BTreeMap<String, usize>,
    pub weights: BTreeMap<String, f64>,
    pub weighting_rule: &'static str,
    pub counts: DedupCounts,
}

/// Plan one suite: mine + generate + weight + allocate. `prepared` is the
/// standard suite preparation (the SAME pool/cal/test the eval lanes
/// build — C1: seeds come only from the train pool, never cal or test).
pub(crate) fn plan_suite(
    prepared: &super::Prepared,
    sopts: &SynthOptions,
) -> Result<SuitePlan, String> {
    let pool = &prepared.train;
    if pool.is_empty() {
        return Err("empty corpus pool".into());
    }
    // Cal exclusion keys (C1): a generated text equal to a cal text must
    // never enter the corpus.
    let cal_keys: HashSet<String> = prepared
        .cal_cases
        .iter()
        .filter_map(|c| {
            single_text_field(&c.state).and_then(|f| {
                c.state.get(&f).and_then(serde_json::Value::as_str).map(norm_key)
            })
        })
        .collect();
    if cal_keys.len() != prepared.cal_cases.len() {
        return Err(
            "cal cases are not all single-string-field states — unsupported v1 (loud)".into(),
        );
    }
    let pool_keys: HashSet<String> = pool.iter().map(|d| norm_key(&d.text)).collect();

    // Per-label pool row indices (sorted label iteration = determinism).
    let mut by_label: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (idx, d) in pool.iter().enumerate() {
        by_label.entry(d.label.clone()).or_default().push(idx);
    }
    let tokens: Vec<Vec<String>> = pool
        .iter()
        .map(|d| d.text.split_whitespace().map(str::to_string).collect())
        .collect();

    let universe: BTreeSet<&str> = prepared.labels.iter().map(String::as_str).collect();
    let mut buckets: BTreeMap<String, Vec<SynthCand>> = BTreeMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut counts = DedupCounts::default();

    for (label, idxs) in &by_label {
        let rows: Vec<Vec<String>> = idxs.iter().map(|&i| tokens[i].clone()).collect();
        let slots = mine_slots(&rows, sopts.max_span_len);
        if slots.is_empty() {
            continue;
        }
        // The label's global span pool — every span attested in ANY frame.
        let mut all_spans: BTreeSet<Vec<String>> = BTreeSet::new();
        for (spans, _) in slots.values() {
            all_spans.extend(spans.iter().cloned());
        }
        for ((prefix, suffix), (own_spans, src)) in &slots {
            // Cross-frame transplantation: the frame's OWN spans would
            // only regenerate the attested rows (they dedup out anyway) —
            // the NEW text comes from every OTHER span of the label,
            // sorted, bounded per frame.
            let mut taken = 0usize;
            for span in &all_spans {
                if taken >= MAX_SPANS_PER_FRAME {
                    break;
                }
                if own_spans.contains(span) {
                    continue;
                }
                taken += 1;
                let mut toks = prefix.clone();
                toks.extend(span.iter().cloned());
                toks.extend(suffix.iter().cloned());
                let text = toks.join(" ");
                let key = norm_key(&text);
                if pool_keys.contains(&key) {
                    counts.pool += 1;
                    continue;
                }
                if cal_keys.contains(&key) {
                    counts.cal += 1;
                    continue;
                }
                if !seen.insert(key) {
                    counts.dup += 1;
                    continue;
                }
                if !universe.contains(label.as_str()) {
                    counts.outside += 1;
                    continue;
                }
                buckets.entry(label.clone()).or_default().push(SynthCand {
                    text,
                    label: label.clone(),
                    src: *src,
                    span: (prefix.len(), span.len()),
                });
            }
        }
    }

    let candidates_total: usize = buckets.values().map(Vec::len).sum();
    if candidates_total == 0 {
        return Err(
            "no span slots mined — the pool rows carry no near-duplicate pairs to expand".into(),
        );
    }

    // N2 — per-intent evidence weights (the E0 rumor fraction over each
    // label's OWN count tables), uniform fallback without nb_scope.
    #[cfg(feature = "nb_scope")]
    let (weights, weighting_rule) = {
        let mut w = per_intent_rumor(prepared)?;
        let measured: Vec<f64> = w.values().copied().collect();
        let mean = if measured.is_empty() {
            0.0
        } else {
            measured.iter().sum::<f64>() / measured.len() as f64
        };
        for l in buckets.keys() {
            w.entry(l.clone()).or_insert(mean);
        }
        let rule: &'static str = "per-intent E0 rumor fraction (bag view, n<4, own-tables; \
                                  unmeasured labels take the mean)";
        (w, rule)
    };
    #[cfg(not(feature = "nb_scope"))]
    let (weights, weighting_rule) = {
        let mut w = BTreeMap::new();
        for l in buckets.keys() {
            w.insert(l.clone(), 1.0);
        }
        let rule: &'static str =
            "uniform (nb_scope feature off — the E0 weighting is unavailable)";
        (w, rule)
    };

    let order: Vec<String> = buckets.keys().cloned().collect();
    let alloc = allocate(&order, &weights, &buckets, sopts.max_accepted, sopts.max_per_label);

    Ok(SuitePlan { buckets, alloc, weights, weighting_rule, counts })
}

/// Proportional apportionment by weight with per-label availability +
/// cap, the leftover redistributed round-robin in label order (the
/// caps/availability slack never goes to chance). Integer-scaled weights —
/// float drift cannot reorder the allocation.
fn allocate(
    order: &[String],
    weights: &BTreeMap<String, f64>,
    available: &BTreeMap<String, Vec<SynthCand>>,
    max_accepted: usize,
    max_per_label: usize,
) -> BTreeMap<String, usize> {
    let room = |l: &str| -> usize {
        available.get(l).map_or(0, |v| v.len().min(max_per_label))
    };
    let eff: Vec<(String, u128)> = order
        .iter()
        .filter(|l| room(l) > 0)
        .map(|l| {
            let w = weights.get(l).copied().unwrap_or(0.0).max(0.0);
            (l.clone(), (w * 1_000_000.0) as u128)
        })
        .collect();
    let total_w: u128 = eff.iter().map(|(_, w)| *w).sum();
    let mut alloc: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0usize;
    for (l, w) in &eff {
        // checked_div: a zero total weight (all labels unmeasured) takes
        // the uniform share of 1 per label — the leftover loop does the
        // real filling.
        let share = (max_accepted as u128)
            .checked_mul(*w)
            .and_then(|v| v.checked_div(total_w))
            .unwrap_or(1) as usize;
        let give = share.min(room(l));
        alloc.insert(l.clone(), give);
        total += give;
    }
    // Leftover round-robin (deterministic label order) until the budget
    // is met or every label is at its room.
    while total < max_accepted {
        let mut progressed = false;
        for (l, _) in &eff {
            if total >= max_accepted {
                break;
            }
            let used = alloc.get_mut(l).expect("seeded above");
            if *used < room(l) {
                *used += 1;
                total += 1;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    alloc
}

/// N2 — the per-intent E0 evidence: each label's rumor fraction over its
/// OWN count tables (bag view, ObservedLaplace — scale/α never move
/// seen-ness), measured on that label's CAL states against its own pool
/// docs. Train-region only (C1): cal states are budget INPUTS, never
/// corpus content.
#[cfg(feature = "nb_scope")]
fn per_intent_rumor(prepared: &super::Prepared) -> Result<BTreeMap<String, f64>, String> {
    // Cal states grouped by their gold label.
    let mut states_by_label: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for case in &prepared.cal_cases {
        let Some(q) = case.questions.first() else {
            continue;
        };
        let keys: Vec<String> = match q.kind {
            QKind::Choice => q
                .criteria
                .as_object()
                .ok_or_else(|| format!("{}: choice criteria not an object", case.id))?
                .keys()
                .cloned()
                .collect(),
            QKind::Score => (0..q.criteria.as_array().map_or(0, Vec::len))
                .map(|i| i.to_string())
                .collect(),
            QKind::Noul => {
                return Err(format!("{}: noul cal case in the synth lane", case.id));
            }
        };
        let Some(gold) = keys.get(case.gold[0].idx) else {
            return Err(format!("{}: gold idx out of the presented keys", case.id));
        };
        if let Some(t) = single_text_field(&case.state)
            .and_then(|f| case.state.get(&f))
            .and_then(serde_json::Value::as_str)
        {
            states_by_label.entry(gold.clone()).or_default().push(t);
        }
    }
    // Pool docs per label.
    let mut docs_by_label: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for d in &prepared.train {
        docs_by_label.entry(d.label.clone()).or_default().push(&d.text);
    }
    let mut out = BTreeMap::new();
    for (label, states) in &states_by_label {
        let Some(docs) = docs_by_label.get(label) else {
            continue;
        };
        if docs.is_empty() || states.is_empty() {
            continue;
        }
        let sets: Vec<Vec<String>> = docs
            .iter()
            .map(|d| d.split_whitespace().map(str::to_string).collect())
            .collect();
        let refs: Vec<&[String]> = sets.iter().map(Vec::as_slice).collect();
        let nb = NbScope::fit(&refs, NbAlpha::ObservedLaplace, NbView::Bag);
        let mut toks: Vec<u32> = Vec::new();
        let mut rumor = 0usize;
        for s in states {
            view_tokens_into(NbView::Bag, s.as_bytes(), &mut toks);
            if nb.seen_count(&toks) < RUMOR_N {
                rumor += 1;
            }
        }
        out.insert(label.clone(), rumor as f64 / states.len() as f64);
    }
    Ok(out)
}

// ── the artifact ────────────────────────────────────────────────────────

/// One artifact row (C3: provenance on every row).
#[derive(Debug, serde::Serialize)]
struct SynthRow<'a> {
    text: &'a str,
    label: &'a str,
    provenance: &'static str,
    src: usize,
    span: [usize; 2],
}

fn write_artifact(
    out_dir: &Path,
    suite: &str,
    meta: &SynthArtifactMeta,
    per_label: &BTreeMap<String, SynthPerLabel>,
    accepted: &[SynthCand],
) -> Result<(std::path::PathBuf, String), String> {
    let mut header = serde_json::to_value(meta).map_err(|e| e.to_string())?;
    header["per_label"] = serde_json::to_value(per_label).map_err(|e| e.to_string())?;
    let mut bytes: Vec<u8> = Vec::with_capacity(1 << 16);
    bytes
        .extend_from_slice(serde_json::to_string(&header).map_err(|e| e.to_string())?.as_bytes());
    bytes.push(b'\n');
    for c in accepted {
        let row = SynthRow {
            text: &c.text,
            label: &c.label,
            provenance: SYNTH_PROVENANCE,
            src: c.src,
            span: [c.span.0, c.span.1],
        };
        bytes
            .extend_from_slice(serde_json::to_string(&row).map_err(|e| e.to_string())?.as_bytes());
        bytes.push(b'\n');
    }
    std::fs::create_dir_all(out_dir).map_err(|e| format!("create {}: {e}", out_dir.display()))?;
    let path = out_dir.join(format!("{suite}_synth.jsonl"));
    std::fs::write(&path, &bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    let seal = blake3::hash(&bytes);
    let sidecar = out_dir.join(format!("{suite}_synth.jsonl.blake3"));
    std::fs::write(&sidecar, format!("{}\n", seal.to_hex()))
        .map_err(|e| format!("write {}: {e}", sidecar.display()))?;
    Ok((path, seal.to_hex().to_string()))
}

/// Load a synth corpus artifact: magic/version/provenance/row-count
/// validation + the sidecar digest check (a corpus whose bytes do not
/// match its seal is refused, never loaded). Label semantics (membership
/// in an engine universe) are the CONSUMER's check — the loader owns
/// integrity only. Returns (meta, docs, the sidecar-verified digest hex)
/// — the digest rides the same read that verified it (no second read, no
/// TOCTOU window between the check and the identity).
pub fn load_synth_corpus(
    path: &Path,
) -> Result<(SynthArtifactMeta, Vec<TrainDoc>, String), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let sidecar = std::path::PathBuf::from(format!("{}.blake3", path.display()));
    let expect = std::fs::read_to_string(&sidecar).map_err(|e| {
        format!(
            "read {}: {e} — the digest sidecar is mandatory (write one with harness \
             --synth-corpus)",
            sidecar.display()
        )
    })?;
    let expect = expect.trim();
    let got = blake3::hash(&bytes).to_hex().to_string();
    if got != expect {
        return Err(format!(
            "{}: BLAKE3 mismatch (file {}… vs sidecar {expect}) — refusing a corpus \
             whose bytes do not match its seal",
            path.display(),
            &got[..16]
        ));
    }
    let text =
        String::from_utf8(bytes.clone())
            .map_err(|e| format!("{}: not utf-8: {e}", path.display()))?;
    let mut lines = text.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| format!("{}: empty artifact", path.display()))?;
    let meta: SynthArtifactMeta = serde_json::from_str(header_line)
        .map_err(|e| format!("{}: header parse: {e}", path.display()))?;
    if meta.magic != CORPUS_MAGIC {
        return Err(format!(
            "{}: magic {:?} != {CORPUS_MAGIC:?}",
            path.display(),
            meta.magic
        ));
    }
    if meta.version != CORPUS_VERSION {
        return Err(format!(
            "{}: version {} != {CORPUS_VERSION} — a format change is a new version, never \
             an in-place reinterpretation",
            path.display(),
            meta.version
        ));
    }
    let mut docs = Vec::new();
    for (ln, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("{}: row {}: {e}", path.display(), ln + 2))?;
        let provenance = v
            .get("provenance")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{}: row {} missing provenance", path.display(), ln + 2))?;
        if provenance != SYNTH_PROVENANCE {
            return Err(format!(
                "{}: row {} provenance {provenance:?} != {SYNTH_PROVENANCE:?} — the artifact \
                 carries synth rows only",
                path.display(),
                ln + 2
            ));
        }
        let label = v
            .get("label")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{}: row {} missing label", path.display(), ln + 2))?;
        let t = v
            .get("text")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{}: row {} missing text", path.display(), ln + 2))?;
        if t.trim().is_empty() {
            return Err(format!("{}: row {} empty text", path.display(), ln + 2));
        }
        docs.push(TrainDoc { label: label.to_string(), text: t.to_string() });
    }
    if docs.len() != meta.accepted {
        return Err(format!(
            "{}: header claims {} accepted rows, file carries {}",
            path.display(),
            meta.accepted,
            docs.len()
        ));
    }
    Ok((meta, docs, got))
}

// ── the run entries ─────────────────────────────────────────────────────

/// The synth mode's suites: explicit `--suites`, else the V5 board suite.
pub(crate) fn synth_suites(opts: &RunOptions) -> (Vec<&'static SuiteSpec>, bool) {
    let defaulted = opts.suites.is_empty();
    let names: Vec<&str> = if defaulted {
        vec!["massive_intent_en"]
    } else {
        opts.suites.iter().map(String::as_str).collect()
    };
    let specs = names
        .into_iter()
        .filter_map(|n| SUITES.iter().find(|s| s.name == n))
        .collect();
    (specs, defaulted)
}

/// Run the synthesis lane over the requested suites (empty `--suites` =
/// the V5 board suite `massive_intent_en`). A suite that fails is a named
/// loud skip; ALL skipped is the mode's error. Zero accepted rows after a
/// real veto is ALSO loud (a teacher that agrees with nothing is a broken
/// teacher, not a clean run).
pub fn run_synth_corpus(opts: &RunOptions, sopts: &SynthOptions) -> Result<SynthOutput, String> {
    let (specs, defaulted) = synth_suites(opts);
    if defaulted {
        eprintln!(
            "harness --synth-corpus: no --suites — defaulting to the V5 board suite \
             (massive_intent_en)"
        );
    }
    let mut suites = Vec::new();
    let mut skipped = Vec::new();
    for spec in specs {
        match synth_one(spec, &opts.datasets_dir, sopts) {
            Ok(s) => suites.push(s),
            Err(e) => skipped.push(format!("{}: {e}", spec.name)),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "synth-corpus: no suite ran ({} skip/failure line(s), first: {})",
            skipped.len(),
            skipped.first().map_or("none", String::as_str)
        ));
    }
    Ok(SynthOutput {
        meta: SynthMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            max_accepted: sopts.max_accepted,
            max_per_label: sopts.max_per_label,
            max_span_len: sopts.max_span_len,
        },
        suites,
        skipped,
    })
}

fn synth_one(spec: &SuiteSpec, dir: &Path, sopts: &SynthOptions) -> Result<SynthSuite, String> {
    let t_start = std::time::Instant::now();
    let prepared = prepare(spec, dir)?;
    let plan = plan_suite(&prepared, sopts)?;
    let shape = VetoShape::from_suite(&prepared.suite)?;
    let labels_sorted = {
        let mut v = prepared.labels.clone();
        v.sort();
        v
    };

    let mut teacher = construct_teacher(&sopts.teacher, spec)?;
    if let Some(case) = prepared.suite.cases.first() {
        teacher.warmup(case)?;
    }

    // The Issue-064 ascent leg, OPT-IN: per-label density gates built from
    // the SAME pool the veto scores against (the Embedder's own projection;
    // ε at the label's control-quantile |Δ| — Bench 120's instrument).
    // `None` = the shipped posture: no kernel is built, nothing changes.
    let embedder = crate::embed::Embedder;
    let density_gates: Option<BTreeMap<String, DensityGate>> = sopts.density_gate.map(|rung| {
        let mut pool_emb: Vec<[f32; crate::embed::EMBED_DIM]> = prepared
            .train
            .iter()
            .map(|d| {
                let mut v = [0.0f32; crate::embed::EMBED_DIM];
                embedder.embed_into(d.text.as_bytes(), &mut v);
                v
            })
            .collect();
        let mut by_label: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (idx, d) in prepared.train.iter().enumerate() {
            by_label.entry(d.label.clone()).or_default().push(idx);
        }
        let mut gates = BTreeMap::new();
        for label in plan.buckets.keys() {
            if let Some(idxs) = by_label.get(label) {
                let pool: Vec<[f32; crate::embed::EMBED_DIM]> =
                    idxs.iter().map(|&g| pool_emb[g]).collect();
                gates.insert(label.clone(), DensityGate::new(pool, rung));
            }
        }
        // Free the scratch embeddings (moved out of above; a no-op drop).
        pool_emb.clear();
        if !gates.is_empty() {
            let mut bands: Vec<f64> = gates.values().map(|g| g.eps()).collect();
            bands.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
            eprintln!(
                "  [synth {}] density gates: {} label(s) · ε_nat band {:.3}..{:.3} (median \
                 {:.3})",
                spec.name,
                gates.len(),
                bands[0],
                bands[bands.len() - 1],
                bands[bands.len() / 2]
            );
        }
        gates
    });
    let density_rule: Option<String> = sopts.density_gate.map(|r| r.rule());
    // The rescue pre-filter (Issue 156 T4): built ONCE per suite — the
    // default-posture engine over the same train pool the veto's universe
    // came from. `None` = the shipped veto-only posture.
    let mut rescue_judge = if sopts.rescue_prefilter {
        Some(build_b0_judge(spec.name, spec.corpus_cap_per_label, &prepared)?)
    } else {
        None
    };
    let rescue_rule: Option<String> = sopts
        .rescue_prefilter
        .then(|| format!("flyby-b0-local-veto-augment-v1 (bd≥{RESCUE_BD_KEEP_MIN} stays the veto's authority)"));
    eprintln!(
        "  [synth {}] teacher {}/{} · {} candidate(s) across {} label(s) · budget {}{}{}",
        spec.name,
        teacher.name(),
        teacher.provenance(),
        plan.buckets.values().map(Vec::len).sum::<usize>(),
        plan.buckets.len(),
        plan.alloc.values().sum::<usize>(),
        if let Some(r) = &density_rule { format!(" · density gate: {r}") } else { String::new() },
        if rescue_rule.is_some() { " · rescue pre-filter: b0 local, veto authority".to_string() } else { String::new() },
    );

    let mut accepted: Vec<SynthCand> = Vec::new();
    let mut durs_ms: Vec<u64> = Vec::new();
    let mut per_label: BTreeMap<String, SynthPerLabel> = BTreeMap::new();
    for (label, cands) in &plan.buckets {
        per_label.insert(
            label.clone(),
            SynthPerLabel {
                candidates: cands.len(),
                allocated: plan.alloc.get(label).copied().unwrap_or(0),
                accepted: 0,
                veto_rejected: 0,
                density_rejected: 0,
                rescue_rejected: 0,
                weight: plan.weights.get(label).copied().unwrap_or(0.0),
            },
        );
    }
    'labels: for (label, cands) in &plan.buckets {
        let alloc = plan.alloc.get(label).copied().unwrap_or(0);
        if alloc == 0 {
            continue;
        }
        for c in cands {
            if accepted.len() >= sopts.max_accepted {
                break 'labels;
            }
            let slot = per_label.get_mut(label).expect("seeded above");
            if slot.accepted >= alloc {
                continue 'labels;
            }
            // The ascent leg (Issue 064), DENSITY-FIRST: the minimal-deviation
            // gate runs BEFORE the teacher forward. Both filters are pure
            // functions of the candidate, so the accepted SET is
            // order-invariant (the first N candidates passing BOTH, in
            // iteration order — identical artifact bytes either order), but
            // the cost is not: the loop forwards until the budget fills, so
            // a post-veto gate would spend a teacher call on every
            // density-rejected row (~2.5× the forwards at the p50 pass rate
            // — measured ~10 h vs ~4 h). Density-first also mirrors the
            // paper's own operator (the learner-density check runs on the
            // proposal; the privileged constraint is the second gate). The
            // gate can only reject, never rescue — unchanged.
            if let Some(gates) = &density_gates
                && !gates
                    .get(label)
                    .is_none_or(|g| g.accepts(&embedder, &c.text, c.src))
            {
                let slot = per_label.get_mut(label).expect("seeded above");
                slot.density_rejected += 1;
                continue;
            }
            // The rescue pre-filter (Issue 156 T4), after the density gate
            // and BEFORE the teacher forward — both local filters are pure
            // functions of the candidate (the engine is deterministic), so
            // the accepted SET stays order-invariant; only the COST moves:
            // a no-tool-solved candidate never spends the veto call. The
            // filter can only NARROW (a candidate passing local but failing
            // the veto stays rejected — the veto is the acceptance
            // authority); the b0 leg reads the FORCED pick (the hard-metric
            // convention — see the B0Judge doc for why not the gate).
            let (case, gold_key, gold_pos) = veto_case(
                &shape,
                &labels_sorted,
                label,
                &c.text,
                format!("synth:{}:{label}:{}", spec.name, accepted.len()),
            )?;
            if let Some(judge) = rescue_judge.as_mut() {
                let state_str = crate::pyjson::serialize_state(&case.state);
                if judge.already_serves(&case, &state_str, gold_pos)? {
                    let slot = per_label.get_mut(label).expect("seeded above");
                    slot.rescue_rejected += 1;
                    continue;
                }
            }
            let (keyed, ms) = teacher.forward(&case)?;
            durs_ms.push(ms);
            let is_accept = veto_accept(&keyed, &gold_key);
            {
                let slot = per_label.get_mut(label).expect("seeded above");
                if is_accept {
                    accepted.push(c.clone());
                    slot.accepted += 1;
                } else {
                    slot.veto_rejected += 1;
                }
            }
            if durs_ms.len().is_multiple_of(200) {
                let vetoed: usize = per_label.values().map(|p| p.veto_rejected).sum::<usize>();
                eprintln!(
                    "  [synth {}] {} forwards · {} accepted · {} vetoed",
                    spec.name,
                    durs_ms.len(),
                    accepted.len(),
                    vetoed
                );
            }
        }
    }
    let veto_rejected: usize = per_label.values().map(|p| p.veto_rejected).sum();
    let rescue_rejected: usize = per_label.values().map(|p| p.rescue_rejected).sum();
    if accepted.is_empty() {
        return Err(format!(
            "veto accepted 0 of {} forwarded candidate(s) — teacher {:?} agrees with \
             nothing; refusing to write an empty artifact",
            durs_ms.len(),
            sopts.teacher
        ));
    }
    let (p50, p99, _support) = percentile_us(&durs_ms);

    let meta = SynthArtifactMeta {
        magic: CORPUS_MAGIC.to_string(),
        version: CORPUS_VERSION,
        suite: spec.name.to_string(),
        teacher: teacher.name().to_string(),
        teacher_provenance: teacher.provenance().to_string(),
        weighting_rule: plan.weighting_rule.to_string(),
        created_utc: iso8601_utc(),
        git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
        host: hostname_refusing_unknown(),
        accepted: accepted.len(),
        pool_rows: prepared.train.len(),
        density_rule: density_rule.clone(),
        rescue_rule: rescue_rule.clone(),
    };
    let (path, seal_hex) =
        write_artifact(sopts.out_dir.as_path(), spec.name, &meta, &per_label, &accepted)?;
    eprintln!(
        "  [synth {}] done: {} accepted / {} forwarded · {} rescue-rejected · p50 {p50} ms · \
         {:.1}s → {}",
        spec.name,
        accepted.len(),
        durs_ms.len(),
        rescue_rejected,
        t_start.elapsed().as_secs_f64(),
        path.display()
    );
    Ok(SynthSuite {
        name: spec.name.to_string(),
        teacher: teacher.name().to_string(),
        teacher_provenance: teacher.provenance().to_string(),
        weighting_rule: plan.weighting_rule.to_string(),
        pool_rows: prepared.train.len(),
        candidates_total: plan.buckets.values().map(Vec::len).sum(),
        accepted: accepted.len(),
        veto_rejected,
        dedup_pool: plan.counts.pool,
        dedup_cal: plan.counts.cal,
        dedup_dup: plan.counts.dup,
        outside_universe: plan.counts.outside,
        veto_forwards: durs_ms.len(),
        latency_p50_ms: p50 as f64,
        latency_p99_ms: p99 as f64,
        seconds: t_start.elapsed().as_secs_f64(),
        path: path.display().to_string(),
        blake3: seal_hex[..16].to_string(),
        density_rule,
        rescue_rejected,
        rescue_rule,
        per_label,
    })
}

/// The plan-only report (`--synth-plan`): the whole pipeline except the
/// veto, nothing written — inspect the allocation before spending teacher
/// calls. Returns the markdown.
pub fn run_synth_plan(opts: &RunOptions, sopts: &SynthOptions) -> Result<String, String> {
    let (specs, defaulted) = synth_suites(opts);
    if defaulted {
        eprintln!(
            "harness --synth-plan: no --suites — defaulting to the V5 board suite \
             (massive_intent_en)"
        );
    }
    let mut md = String::new();
    md.push_str(&format!(
        "# synth plan — teacher {:?} · max {} accepted / {} per label · span ≤ {}{}\n\n",
        sopts.teacher,
        sopts.max_accepted,
        sopts.max_per_label,
        sopts.max_span_len,
        if sopts.rescue_prefilter {
            " · rescue pre-filter: b0 local, veto authority"
        } else {
            ""
        }
    ));
    let mut any = false;
    let mut errors = Vec::new();
    for spec in specs {
        let prepared = match prepare(spec, &opts.datasets_dir) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!("{}: {e}", spec.name));
                continue;
            }
        };
        let plan = match plan_suite(&prepared, sopts) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!("{}: {e}", spec.name));
                continue;
            }
        };
        any = true;
        md.push_str(&format!(
            "## {} — {} candidate(s) / {} label(s) · pool {} rows\n\n",
            spec.name,
            plan.buckets.values().map(Vec::len).sum::<usize>(),
            plan.buckets.len(),
            prepared.train.len(),
        ));
        md.push_str(&format!("weighting: {}\n\n", plan.weighting_rule));
        md.push_str("| label | weight | candidates | allocated | sample |\n|---|---|---|---|---|\n");
        for (label, cands) in &plan.buckets {
            let alloc = plan.alloc.get(label).copied().unwrap_or(0);
            let sample = cands
                .first()
                .map(|c| c.text.replace('|', "\\|"))
                .unwrap_or_default();
            md.push_str(&format!(
                "| {label} | {:.3} | {} | {alloc} | {sample} |\n",
                plan.weights.get(label).copied().unwrap_or(0.0),
                cands.len()
            ));
        }
        md.push_str(&format!(
            "\ndedup: pool {} · cal {} · dup {} · outside-universe {}\n\ntotal allocated: \
             {}\n\n",
            plan.counts.pool,
            plan.counts.cal,
            plan.counts.dup,
            plan.counts.outside,
            plan.alloc.values().sum::<usize>(),
        ));
    }
    for e in &errors {
        md.push_str(&format!("\n- SKIPPED: {e}\n"));
    }
    if !any {
        return Err(format!(
            "synth-plan: no suite planned ({} failure line(s), first: {})",
            errors.len(),
            errors.first().map_or("none", String::as_str)
        ));
    }
    Ok(md)
}

/// The run report markdown (the bin writes SYNTH.md).
pub fn render_synth_markdown(out: &SynthOutput) -> String {
    let mut s = String::new();
    s.push_str("# synth corpus report (Plan 426 T5)\n\n");
    s.push_str(&format!(
        "- budget: ≤ {} accepted / {} per label · span ≤ {} · datasets {}\n- host {} @ {} \
         ({})\n",
        out.meta.max_accepted,
        out.meta.max_per_label,
        out.meta.max_span_len,
        out.meta.datasets_dir,
        out.meta.host,
        &out.meta.git_sha[..8.min(out.meta.git_sha.len())],
        out.meta.date_utc,
    ));
    s.push_str("\n| suite | teacher | cands | accepted | vetoed | forwards | p50 ms | s | \
                blake3 |\n|---|---|---|---|---|---|---|---|---|\n");
    for v in &out.suites {
        s.push_str(&format!(
            "| {} | {}/{} | {} | {} | {} | {} | {:.0} | {:.0} | {}… |\n",
            v.name,
            v.teacher,
            v.teacher_provenance,
            v.candidates_total,
            v.accepted,
            v.veto_rejected,
            v.veto_forwards,
            v.latency_p50_ms,
            v.seconds,
            v.blake3
        ));
    }
    s.push_str("\n| suite | weighting | dedup pool/cal/dup/out |\n|---|---|---|\n");
    for v in &out.suites {
        s.push_str(&format!(
            "| {} | {} | {}/{}/{}/{} |\n",
            v.name,
            v.weighting_rule,
            v.dedup_pool,
            v.dedup_cal,
            v.dedup_dup,
            v.outside_universe
        ));
    }
    // The rescue pre-filter disclosure (Issue 156 T4) — only when armed on
    // any suite (the shipped posture prints nothing, byte-identical).
    if let Some(v) = out.suites.iter().find(|s| s.rescue_rule.is_some()) {
        s.push_str(&format!(
            "\nrescue pre-filter: {} — the local `b0` leg rejects what the modelless \
             arm already serves; the teacher veto stays the acceptance authority.\n\n| \
             suite | rescue-rejected |\n|---|---|\n",
            v.rescue_rule.clone().unwrap_or_default()
        ));
        for v in &out.suites {
            s.push_str(&format!("| {} | {} |\n", v.name, v.rescue_rejected));
        }
    }
    s.push_str("\nPer-label allocation (top 20 by accepted):\n\n| label | weight | cands | \
                alloc | accepted | vetoed |\n|---|---|---|---|---|---|\n");
    for v in &out.suites {
        let mut rows: Vec<_> = v.per_label.iter().collect();
        rows.sort_by(|a, b| b.1.accepted.cmp(&a.1.accepted).then(a.0.cmp(b.0)));
        for (label, p) in rows.iter().take(20) {
            s.push_str(&format!(
                "| {}/{label} | {:.3} | {} | {} | {} | {} |\n",
                v.name, p.weight, p.candidates, p.allocated, p.accepted, p.veto_rejected
            ));
        }
    }
    for e in &out.skipped {
        s.push_str(&format!("\n- SKIPPED: {e}\n"));
    }
    s
}

// ── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tks(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }

    fn doc(label: &str, text: &str) -> TrainDoc {
        TrainDoc { label: label.to_string(), text: text.to_string() }
    }

    #[test]
    fn mining_finds_the_attested_span_slots() {
        let rows: Vec<Vec<String>> = [
            "wake me up at nine am on friday",
            "wake me up at six pm on friday",
            "set an alarm for two hours from now",
            "set an alarm for one hour from now",
        ]
        .iter()
        .map(|s| tks(s))
        .collect();
        let slots = mine_slots(&rows, 4);
        let frame1 = (tks("wake me up at"), tks("on friday"));
        let (spans, _) = slots.get(&frame1).expect("frame 1 mined");
        assert!(spans.contains(&tks("nine am")));
        assert!(spans.contains(&tks("six pm")));
        assert_eq!(spans.len(), 2);
        let frame2 = (tks("set an alarm for"), tks("from now"));
        let (spans2, _) = slots.get(&frame2).expect("frame 2 mined");
        assert!(spans2.contains(&tks("two hours")));
        assert!(spans2.contains(&tks("one hour")));
    }

    #[test]
    fn mining_respects_the_span_bound_and_identical_rows() {
        // A 5-token diff exceeds the default bound of 4; identical rows
        // leave an empty span.
        let rows: Vec<Vec<String>> = [
            "a b c one two three four five",
            "a b c six seven eight nine ten",
            "a b c d e",
            "a b c d e",
        ]
        .iter()
        .map(|s| tks(s))
        .collect();
        let slots = mine_slots(&rows, 4);
        assert!(
            slots.is_empty(),
            "5-token spans must refuse; identical rows are not slots"
        );
        let slots5 = mine_slots(&rows, 5);
        assert_eq!(slots5.len(), 1, "the span bound is the knob");
    }

    fn fixture_prepared() -> super::super::Prepared {
        let train = vec![
            doc("alarm_set", "wake me up at nine am on friday"),
            doc("alarm_set", "wake me up at six pm on friday"),
            doc("alarm_set", "set an alarm for two hours from now"),
            doc("alarm_set", "set an alarm for one hour from now"),
            doc("audio_volume_other", "make it quieter please"),
        ];
        let cal_cases = vec![SuiteCase {
            id: "cal:0".into(),
            state: serde_json::json!({ "utterance": "wake me up at six pm on saturday" }),
            questions: vec![],
            gold: vec![],
        }];
        super::super::Prepared {
            slices: None,
            suite: Suite {
                name: "fixture",
                cases: vec![],
                option_counts_note: "",
            },
            train,
            state_strs: vec![],
            cal_cases,
            cal_state_strs: vec![],
            labels: vec!["alarm_set".to_string(), "audio_volume_other".to_string()],
            pool_rows: serde_json::Value::Null,
        }
    }

    fn plan_fixture(sopts: &SynthOptions) -> SuitePlan {
        plan_suite(&fixture_prepared(), sopts).expect("plan")
    }

    fn sopts(max_accepted: usize, max_per_label: usize) -> SynthOptions {
        SynthOptions {
            teacher: "openthai".into(),
            max_accepted,
            max_per_label,
            max_span_len: 4,
            out_dir: std::env::temp_dir().join(format!("rfx_synth_test_{}", std::process::id())),
            density_gate: None,
            rescue_prefilter: false,
        }
    }

    #[test]
    fn generation_transplants_across_frames_and_respects_exclusions() {
        let sopts = sopts(64, 64);
        let plan1 = plan_fixture(&sopts);
        let plan2 = plan_fixture(&sopts);
        let texts: Vec<String> = plan1
            .buckets
            .values()
            .flat_map(|v| v.iter().map(|c| c.text.clone()))
            .collect();
        let texts2: Vec<String> = plan2
            .buckets
            .values()
            .flat_map(|v| v.iter().map(|c| c.text.clone()))
            .collect();
        assert_eq!(texts, texts2, "two plans must be identical");
        assert!(!texts.is_empty(), "the fixture mines transplantable slots");
        // Cross-frame products present; the frame's OWN spans regenerate
        // only attested rows, which dedup out.
        assert!(
            texts.contains(&"wake me up at two hours on friday".to_string()),
            "frame1 × span-from-frame2 present, got {texts:?}"
        );
        assert!(
            texts.contains(&"set an alarm for nine am from now".to_string()),
            "frame2 × span-from-frame1 present, got {texts:?}"
        );
        // Cal text and pool rows never appear.
        assert!(!texts.iter().any(|t| t.contains("saturday")), "cal texts excluded");
        assert!(
            !texts.contains(&"wake me up at six pm on friday".to_string()),
            "pool rows excluded"
        );
    }

    #[test]
    fn allocation_is_capped_and_deterministic_with_leftover_round_robin() {
        let order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut weights = BTreeMap::new();
        weights.insert("a".to_string(), 0.6);
        weights.insert("b".to_string(), 0.3);
        weights.insert("c".to_string(), 0.1);
        let mut buckets = BTreeMap::new();
        buckets.insert("a".to_string(), make_cands("a", 5));
        buckets.insert("b".to_string(), make_cands("b", 50));
        buckets.insert("c".to_string(), make_cands("c", 50));
        // Capacity: room(a) = min(5, cap 3) = 3, room(b) = 3, room(c) = 3
        // → total 11? No: 3+3+3 = 9 < budget 20 — the leftover loop must
        // fill every label to its room and stop.
        let alloc = allocate(&order, &weights, &buckets, 20, 3);
        assert_eq!(alloc["a"], 3, "per-label cap binds for a too");
        assert_eq!(alloc["b"], 3, "per-label cap binds");
        assert_eq!(alloc["c"], 3, "per-label cap binds");
        assert_eq!(alloc.values().sum::<usize>(), 9);

        // Roomy buckets: the budget splits evenly (equal weights).
        let mut buckets2 = BTreeMap::new();
        buckets2.insert("a".to_string(), make_cands("a", 30));
        buckets2.insert("b".to_string(), make_cands("b", 30));
        let order2 = vec!["a".to_string(), "b".to_string()];
        let alloc2 = allocate(&order2, &weights_for_uniform(), &buckets2, 40, 128);
        assert_eq!(alloc2["a"], 20);
        assert_eq!(alloc2["b"], 20);
        assert_eq!(alloc2.values().sum::<usize>(), 40);

        // A zero-capacity label takes nothing.
        let mut buckets3 = buckets2.clone();
        buckets3.insert("c".to_string(), vec![]);
        let order3 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let alloc3 = allocate(&order3, &weights_for_uniform(), &buckets3, 10, 128);
        assert!(!alloc3.contains_key("c"), "no room → no allocation");
    }

    fn weights_for_uniform() -> BTreeMap<String, f64> {
        let mut w = BTreeMap::new();
        w.insert("a".to_string(), 1.0);
        w.insert("b".to_string(), 1.0);
        w.insert("c".to_string(), 1.0);
        w
    }

    fn make_cands(label: &str, n: usize) -> Vec<SynthCand> {
        (0..n)
            .map(|i| SynthCand {
                text: format!("{label} candidate number {i}"),
                label: label.to_string(),
                src: i,
                span: (0, 1),
            })
            .collect()
    }

    #[test]
    fn the_veto_predicate_is_key_string_free_of_order() {
        let keyed = vec![
            ("alpha".to_string(), 0.2),
            ("beta".to_string(), 0.9),
            ("gamma".to_string(), 0.9),
        ];
        // beta and gamma tie at 0.9 — the LOWEST position wins, so gold
        // "gamma" must NOT pass, gold "beta" must.
        assert!(veto_accept(&keyed, "beta"));
        assert!(!veto_accept(&keyed, "gamma"));
        assert!(!veto_accept(&keyed, "alpha"));
        assert!(!veto_accept(&[], "anything"));
    }

    #[test]
    fn artifact_roundtrips_and_refuses_tampering() {
        let dir = std::env::temp_dir().join(format!("rfx_synth_art_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let accepted = vec![
            SynthCand {
                text: "wake me up at nine pm on friday".into(),
                label: "alarm_set".into(),
                src: 3,
                span: (4, 2),
            },
            SynthCand {
                text: "set an alarm for one hour from now".into(),
                label: "alarm_set".into(),
                src: 1,
                span: (4, 2),
            },
        ];
        let mut per_label = BTreeMap::new();
        per_label.insert(
            "alarm_set".to_string(),
            SynthPerLabel {
                candidates: 10,
                allocated: 8,
                accepted: 2,
                veto_rejected: 1,
                density_rejected: 0,
                rescue_rejected: 0,
                weight: 0.5,
            },
        );
        let meta = SynthArtifactMeta {
            magic: CORPUS_MAGIC.to_string(),
            version: CORPUS_VERSION,
            suite: "fixture".into(),
            teacher: "openthai".into(),
            teacher_provenance: "openthai:openthai-systemone".into(),
            weighting_rule: "uniform".into(),
            created_utc: "2026-09-28T00:00:00Z".into(),
            git_sha: "test".into(),
            host: "test".into(),
            accepted: accepted.len(),
            pool_rows: 4,
            density_rule: None,
            rescue_rule: None,
        };
        let (path, _hex) =
            write_artifact(&dir, "fixture", &meta, &per_label, &accepted).expect("write");
        let (loaded_meta, docs, _digest) = load_synth_corpus(&path).expect("load");
        assert_eq!(loaded_meta.suite, "fixture");
        assert_eq!(
            docs,
            accepted.iter().map(|c| doc(&c.label, &c.text)).collect::<Vec<_>>()
        );

        // Tamper: rewrite a text byte → the sidecar must refuse it.
        let raw = std::fs::read_to_string(&path).unwrap();
        let tampered = raw.replace("nine pm", "ten pm");
        std::fs::write(&path, tampered).unwrap();
        let err = load_synth_corpus(&path).unwrap_err();
        assert!(err.contains("BLAKE3 mismatch"), "got: {err}");

        // Provenance drift: a hand-written gold row must refuse.
        let raw = raw.replace("\"provenance\":\"synth\"", "\"provenance\":\"gold\"");
        std::fs::write(&path, &raw).unwrap();
        let seal = blake3::hash(raw.as_bytes()).to_hex();
        std::fs::write(format!("{}.blake3", path.display()), format!("{seal}\n")).unwrap();
        let err = load_synth_corpus(&path).unwrap_err();
        assert!(err.contains("provenance"), "got: {err}");

        // Magic drift.
        let raw = raw.replace(CORPUS_MAGIC, "XXXX");
        std::fs::write(&path, &raw).unwrap();
        let seal = blake3::hash(raw.as_bytes()).to_hex();
        std::fs::write(format!("{}.blake3", path.display()), format!("{seal}\n")).unwrap();
        let err = load_synth_corpus(&path).unwrap_err();
        assert!(err.contains("magic"), "got: {err}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn veto_case_presents_the_full_universe_and_keys_gold() {
        let shape = VetoShape {
            field: "utterance".into(),
            qid: "intent".into(),
            instructions: "What is the user asking for in `utterance`?".into(),
            kind: QKind::Choice,
            score_criteria: None,
        };
        let labels: Vec<String> = ["alarm_set", "audio_volume_other"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (case, gold_key, gold_pos) = veto_case(
            &shape,
            &labels,
            "audio_volume_other",
            "make it louder",
            "synth:t:0".into(),
        )
        .expect("case");
        let q = &case.questions[0];
        let keys: Vec<&String> = q.criteria.as_object().unwrap().keys().collect();
        assert_eq!(keys.len(), 2);
        assert_eq!(case.gold[0].idx, 1, "gold position follows the sorted universe");
        assert_eq!(gold_pos, 1, "the returned position matches the gold index");
        assert_eq!(gold_key, "audio_volume_other");
        assert_eq!(case.state["utterance"], "make it louder");
    }

    #[test]
    fn rescue_predicate_pins_the_flyby_truth_table() {
        // The same truth table riir-refine's G1 pins (Issue 156 T5) — the
        // two ports cannot drift: (0,4)/(0,3) keep · (1,4) reject (the
        // control succeeded — execution-reachable, FlyBy's V>0 law) ·
        // (0,2) reject (rescue undemonstrated).
        assert!(rescue_keep(0, 4));
        assert!(rescue_keep(0, 3));
        assert!(!rescue_keep(1, 4));
        assert!(!rescue_keep(0, 2));
        assert_eq!(RESCUE_BD_KEEP_MIN, 3);
        assert_eq!(FLYBY_CONTINUATIONS_PER_ARM, 4);
    }

    #[test]
    fn b0_judge_serves_pool_replicas_and_is_deterministic() {
        // The engine's decide path is stack-hungry in debug builds (the
        // serve lane boots 64 MiB-stack threads for the same reason) — the
        // test owns its stack, never the runner's environment.
        let handoff = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(b0_judge_probe)
            .expect("spawn");
        let (replica_served, far_served) = handoff.join().expect("join");
        assert!(
            replica_served,
            "a pool replica is home turf — the scoring picks its own label"
        );
        assert!(
            !far_served,
            "a text off every corpus must not read as served — the veto decides it"
        );
    }

    /// The test body on its own thread: returns
    /// (replica already-served?, far-text already-served?).
    fn b0_judge_probe() -> (bool, bool) {
        // The judge over the 2-label fixture: an alarm corpus replica is
        // the no-tool arm's home turf — the forced pick must select its own
        // label, so the pre-filter would reject it without a teacher call.
        // A text far from every corpus must not read as served.
        let prepared = fixture_prepared();
        let mut judge = build_b0_judge("fixture", 64, &prepared).expect("judge");
        let shape = VetoShape {
            field: "utterance".into(),
            qid: "intent".into(),
            instructions: "What is the user asking for in `utterance`?".into(),
            kind: QKind::Choice,
            score_criteria: None,
        };
        let labels = vec!["alarm_set".to_string(), "audio_volume_other".to_string()];
        let (case, _gold_key, gold_pos) = veto_case(
            &shape,
            &labels,
            "alarm_set",
            "wake me up at nine am on friday",
            "synth-b0:t:0".into(),
        )
        .expect("case");
        let state_str = crate::pyjson::serialize_state(&case.state);
        // Deterministic: two continuations of the same request agree (one
        // continuation IS the full distribution — FLYBY_CONTINUATIONS_PER_ARM
        // documents the stochastic denominator the deterministic engine
        // collapses to 1).
        let p1 = judge.continuation(&case, &state_str).expect("cont 1");
        let p2 = judge.continuation(&case, &state_str).expect("cont 2");
        assert_eq!(p1, p2, "the engine is deterministic");
        assert_eq!(p1, gold_pos, "the pool replica picks its own label");
        let replica_served = judge
            .already_serves(&case, &state_str, gold_pos)
            .expect("serve");

        // A text the no-tool arm does NOT solve: gold claims audio_volume_other
        // but the corpora say alarm — the scoring must not rank gold top-1
        // (the veto decides it, never the pre-filter).
        let (far, _k, far_gold) = veto_case(
            &shape,
            &labels,
            "audio_volume_other",
            "zzz qqq barely words at all",
            "synth-b0:t:1".into(),
        )
        .expect("case");
        let far_state = crate::pyjson::serialize_state(&far.state);
        let far_pick = judge.continuation(&far, &far_state).expect("cont far");
        assert_ne!(far_pick, far_gold, "the off-corpus text must not pick the far gold");
        let far_served = judge
            .already_serves(&far, &far_state, far_gold)
            .expect("serve far");
        (replica_served, far_served)
    }

    #[test]
    fn artifact_roundtrips_with_the_rescue_rule_field() {
        // The additive-field law (the density_rule precedent): an artifact
        // written with the rescue disclosure set loads back with it intact,
        // and version stays 2 — old artifacts (no field) load via
        // serde(default), new ones are still v2 rows.
        let dir = std::env::temp_dir().join(format!("rfx_synth_resc_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let accepted = vec![SynthCand {
            text: "wake me up at seven am on friday".into(),
            label: "alarm_set".into(),
            src: 0,
            span: (4, 2),
        }];
        let mut per_label = BTreeMap::new();
        per_label.insert(
            "alarm_set".to_string(),
            SynthPerLabel {
                candidates: 5,
                allocated: 4,
                accepted: 1,
                veto_rejected: 1,
                density_rejected: 0,
                rescue_rejected: 2,
                weight: 1.0,
            },
        );
        let meta = SynthArtifactMeta {
            magic: CORPUS_MAGIC.to_string(),
            version: CORPUS_VERSION,
            suite: "fixture".into(),
            teacher: "openthai".into(),
            teacher_provenance: "openthai:openthai-systemone".into(),
            weighting_rule: "uniform".into(),
            created_utc: "2026-10-08T00:00:00Z".into(),
            git_sha: "test".into(),
            host: "test".into(),
            accepted: accepted.len(),
            pool_rows: 4,
            density_rule: None,
            rescue_rule: Some("flyby-b0-local-veto-augment-v1".into()),
        };
        let (path, _hex) =
            write_artifact(&dir, "fixture_r", &meta, &per_label, &accepted).expect("write");
        let (loaded, docs, _digest) = load_synth_corpus(&path).expect("load");
        assert_eq!(loaded.version, CORPUS_VERSION);
        assert_eq!(
            loaded.rescue_rule.as_deref(),
            Some("flyby-b0-local-veto-augment-v1")
        );
        assert_eq!(docs.len(), 1);
        // The shipped posture's artifacts (field absent) still load: strip
        // the field from the header, re-seal, load — serde(default) tolerates
        // the absence.
        let raw = std::fs::read_to_string(&path).unwrap();
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let mut v: serde_json::Value = serde_json::from_str(header).unwrap();
        let obj = v.as_object_mut().unwrap();
        obj.remove("rescue_rule");
        let mut bytes = serde_json::to_string(&v).unwrap();
        bytes.push('\n');
        for l in lines {
            bytes.push_str(l);
            bytes.push('\n');
        }
        std::fs::write(&path, &bytes).unwrap();
        let seal = blake3::hash(bytes.as_bytes()).to_hex();
        std::fs::write(format!("{}.blake3", path.display()), format!("{seal}\n")).unwrap();
        let (old_meta, _docs, _d) = load_synth_corpus(&path).expect("old artifact loads");
        assert_eq!(old_meta.rescue_rule, None, "absent field decodes None");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
