//! Issue 090 A1–A3 — the decoy-augmentation / null-stability probe (the
//! pure core).
//!
//! Jeva's propositions (arXiv:2610.18773) convert three assumed engine
//! properties into measurable gates. This module is the lane-agnostic
//! measurement: a choice question is re-answered with K decoy options
//! APPENDED (originals keep their positions — the presentation axis is the
//! perm-probe's domain, not this one), and the probe reads how the
//! null/abstention, the pairwise preference, and the noul readout MOVE.
//!
//! Everything here is PURE: the graded decoy pool/sampler, the
//! augmentation builder, the per-arm record math, the verdicts, and the
//! noul invariance check. The lanes live in [`super::runner::null_probe`]
//! behind the [`NullOracle`] seam — each real lane answers through its own
//! production decide path, so the probe measures what ships, never a
//! parallel rendering.
//!
//! Measurement laws:
//! - **A1 (null-stability).** The paper's Prop 2 proves a flat
//!   softmax-with-null null-probability carries an explicit −log K term a
//!   max-based coverage gate removes — but removing the TERM is not
//!   immunity: the MAX of K option scores still rises with K, so
//!   abstention drift remains possible for their gate and ours alike.
//!   The probe CHARACTERIZES the drift curve (abstain rate and confidence
//!   vs K) and gates only the Report-the-Floor leg (the runner half): the
//!   gate's abstention AUROC/ECE must beat the conformal-naive floor.
//! - **A2 (set-preference).** Prop 1: a decider whose option dependence is
//!   only through independent per-option scores cannot REVERSE the A-vs-B
//!   preference when the candidate set changes (the shared denominator
//!   cancels in the ratio; monotone independent scores ⇒ ordering
//!   decoy-invariant a fortiori). The probe pins `sign(p_A − p_B)` across
//!   augmentation arms — the OVERALL pick may legitimately move to a decoy
//!   (not a failure); a preference FLIP is a red (an order/set-sensitive
//!   feature leaked into scoring — structural fix, never smoothing).
//!   Exact ties at the reference arm are exempt (the perm-probe's tie law:
//!   a tied pair's order is the tie structure, not a preference).
//! - **A3 (noul-conditional).** The paper's Noul readout is the positive
//!   candidate's CONDITIONAL probability, never the joint ρ·q (the joint
//!   drifts with coverage, hence with K). Two legs: (a) a paired noul
//!   question on the same state re-answered per arm must be BIT-identical
//!   (our noul wire carries no options by law — anything else would be
//!   cross-request state); (b) the choice-side confidence readout is
//!   classified conditional-shaped (bit-stable across arms) vs
//!   joint-shaped (drifts with K) and the shape is pinned in the record.
//! - **Graded decoys.** Decoys are drawn from the run's suite label
//!   vocabulary (the same namespace the engine scores), ranked by
//!   trigram-Jaccard similarity to the slot's gold label, and sampled as
//!   K evenly-spaced picks across the similarity-sorted candidates (the
//!   linspace law) — the set spans near→far grades deterministically
//!   (byte-reproducible).
//! - **Label space, never position space.** Originals keep their criteria
//!   positions under augmentation (decoys append), so per-arm reads are
//!   position-stable; the A/B pair is tracked by LABEL.

use serde::Serialize;
use serde_json::{Map, Value};

use super::suites::{GoldAnswer, QKind, SuiteCase, SuiteQuestion};

/// Seed constant for the probe's decoy stream (reproducibility is what
/// matters, not the value — the MASSIVE seed law).
pub const NULL_PROBE_SEED: u64 = 0x0900_0001;

/// The default augmentation ladder: the issue's A1 arms (0/8/32/256) plus
/// A2's +64 arm, one ladder for both (each arm is one decide per lane).
pub const DEFAULT_NULL_K: &[usize] = &[0, 8, 32, 64, 256];

/// Default slots (choice questions) sampled per suite — the perm-probe's
/// default, kept so the two probes read the same population shape.
pub const DEFAULT_NULL_MAX_CASES: usize = 40;

/// The paired noul question's fixed instruction. Our noul wire carries NO
/// options by law, so the readout is state-side by construction — the
/// invariance pin (bit-identical across arms) is the point, and the
/// phrasing is disclosed in the meta rather than tuned.
pub const NOUL_PROBE_INSTRUCTIONS: &str =
    "Answer yes if none of the offered answers would be correct; no otherwise.";

/// The A2 gate: preference flips beyond an exact reference tie. The
/// modelless control pre-registers ZERO (independent per-option scoring ⇒
/// the ratio is set-invariant within the normalizer's fp envelope; a flip
/// is an engine defect or a harness bug). Comparison lanes read the same
/// verdict, disclosed rather than run-refusing.
pub const A2_FLIP_GATE: f64 = 0.0;

// ─────────────────────────────────────────────────── graded decoys ──

/// Character-trigram Jaccard similarity in [0, 1] over lowercased strings
/// (strings shorter than 3 chars contribute the whole string as one gram).
#[must_use]
pub fn trigram_jaccard(a: &str, b: &str) -> f64 {
    let grams = |s: &str| -> std::collections::BTreeSet<[u8; 3]> {
        let lower = s.to_lowercase();
        let bytes = lower.as_bytes();
        if bytes.len() < 3 {
            let mut g = [0u8; 3];
            for (i, &b) in bytes.iter().enumerate() {
                g[i] = b;
            }
            if bytes.is_empty() {
                return std::collections::BTreeSet::new();
            }
            return std::collections::BTreeSet::from([g]);
        }
        lower
            .as_bytes()
            .windows(3)
            .map(|w| [w[0], w[1], w[2]])
            .collect()
    };
    let ga = grams(a);
    let gb = grams(b);
    if ga.is_empty() && gb.is_empty() {
        return 1.0;
    }
    if ga.is_empty() || gb.is_empty() {
        return 0.0;
    }
    let inter = ga.intersection(&gb).count() as f64;
    let union = ga.union(&gb).count() as f64;
    inter / union
}

/// The decoy vocabulary: the union of the run's per-suite label spaces,
/// first-seen order (deterministic given the suite selection order).
#[derive(Debug, Clone)]
pub struct DecoyPool {
    labels: Vec<String>,
}

impl DecoyPool {
    #[must_use]
    pub fn build(per_suite_labels: &[Vec<String>]) -> Self {
        let mut labels: Vec<String> = Vec::new();
        for suite_labels in per_suite_labels {
            for l in suite_labels {
                if !labels.iter().any(|x| x == l) {
                    labels.push(l.clone());
                }
            }
        }
        Self { labels }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// K graded decoys for one slot: pool minus the slot's own labels,
    /// ranked by similarity to `gold` (DESC, stable in pool order), then
    /// K evenly-spaced picks across the ranked list (the linspace law:
    /// first + last + k−2 interior positions — exactly `min(k, candidates)`
    /// picks spanning near→far, deterministic in the pool + slot).
    /// Fewer candidates than K returns all of them (the caller discloses
    /// the cap).
    #[must_use]
    pub fn graded_decoys(
        &self,
        slot_labels: &[String],
        gold: Option<&str>,
        k: usize,
        seed: u64,
    ) -> Vec<String> {
        let mut cands: Vec<&String> = self
            .labels
            .iter()
            .filter(|l| !slot_labels.iter().any(|s| s == *l))
            .collect();
        if k == 0 || cands.is_empty() {
            return Vec::new();
        }
        // Rank by similarity to the gold label DESC; the gold is the
        // anchor the paper's decoy construction stresses (near-grade
        // decoys are the ones that can plausibly steal the argmax).
        let anchor = gold.unwrap_or_default();
        cands.sort_by(|a, b| {
            let sa = trigram_jaccard(b, anchor);
            let sb = trigram_jaccard(a, anchor);
            sa.total_cmp(&sb)
        });
        let _ = seed; // the linspace is deterministic without a stream;
                      // the seed is kept in the signature for record parity.
        let n = cands.len();
        if n <= k {
            return cands.into_iter().cloned().collect();
        }
        (0..k)
            .map(|i| {
                // floor(i * (n−1) / (k−1)) — the evenly-spaced k indices
                // over [0, n−1]; the division is exact-index, never a
                // rounding of a float.
                let idx = (i * (n - 1)) / (k - 1).max(1);
                cands[idx].clone()
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────── case building ──

/// The augmented single-question case: the slot's choice question with
/// `decoys` APPENDED to the criteria (original keys keep their positions;
/// decoy values render as the label string itself — the same namespace the
/// engine scores). Gold rides along unmoved (originals' indexes are
/// stable). The arm tag rides the case id (`…/augK`).
///
/// # Errors
/// When the question is not a choice or a decoy collides with an existing
/// key (the sampler excludes slot labels; this is the belt-and-braces
/// refusal).
pub fn augmented_case(
    case: &SuiteCase,
    qi: usize,
    decoys: &[String],
) -> Result<SuiteCase, String> {
    let q = case
        .questions
        .get(qi)
        .ok_or_else(|| format!("case {}: question index {qi} out of range", case.id))?;
    let obj = q
        .criteria
        .as_object()
        .ok_or_else(|| format!("case {}: augmentation needs a choice question", q.qid))?;
    let mut m = Map::new();
    for (k, v) in obj {
        if decoys.iter().any(|d| d == k) {
            return Err(format!(
                "qid {}: decoy key {k:?} collides with an option label",
                q.qid
            ));
        }
        m.insert(k.clone(), v.clone());
    }
    for d in decoys {
        m.insert(d.clone(), Value::String(d.clone()));
    }
    Ok(SuiteCase {
        id: format!("{}#{}/aug{}", case.id, q.qid, decoys.len()),
        state: case.state.clone(),
        questions: vec![SuiteQuestion {
            qid: q.qid.clone(),
            kind: QKind::Choice,
            instructions: q.instructions.clone(),
            criteria: Value::Object(m),
        }],
        gold: vec![case.gold[qi].clone()],
    })
}

/// The paired noul question on the same state (A3 leg a): our noul wire
/// carries NO options by law, so the request bytes are identical across
/// arms — the invariance assert is a real re-answer (catches cross-request
/// state), not a cached copy.
#[must_use]
pub fn noul_case(case: &SuiteCase, qi: usize) -> SuiteCase {
    let q = &case.questions[qi];
    SuiteCase {
        id: format!("{}#{}/noul", case.id, q.qid),
        state: case.state.clone(),
        questions: vec![SuiteQuestion {
            qid: q.qid.clone(),
            kind: QKind::Noul,
            instructions: NOUL_PROBE_INSTRUCTIONS.to_string(),
            criteria: Value::Null,
        }],
        gold: vec![GoldAnswer {
            idx: 0,
            soft: vec![],
            gold_score: None,
        }],
    }
}

// ─────────────────────────────────────────────────── the record ──

/// One lane answer to one arm: probabilities in presented order (originals
/// first, decoys appended — positions stable across arms by construction),
/// the would-be pick, the abstention verdict, the confidence readout, and
/// the modelless engine's scoring REGIME (`drafter_only` — None for lanes
/// without the concept). The regime is load-bearing for A2: the engine's
/// by-name route arms only while k ≤ N, so an arm that pushes the option
/// count past the domain count switches regimes BY DESIGN — preference
/// flips ACROSS regimes are the arming law disclosed, flips WITHIN a
/// regime are the set-sensitivity red.
#[derive(Debug, Clone, PartialEq)]
pub struct AugAnswer {
    pub probs: Vec<f64>,
    pub pick: usize,
    pub abstained: bool,
    pub confidence: f64,
    /// The engine's per-question regime flag (the scratch slot's
    /// `drafter_only`); `None` for comparison lanes.
    pub drafter_only: Option<bool>,
}

/// The paired noul answer (A3 leg a): p(yes), the typed outcome, and the
/// confidence — compared BIT-wise across arms.
#[derive(Debug, Clone, PartialEq)]
pub struct NoulAnswer {
    pub p_yes: f64,
    pub yes: Option<bool>,
    pub confidence: f64,
}

/// The lane-agnostic seam: answer ONE rendered case through the lane's own
/// production decide path (choice arms and the paired noul leg share it).
pub trait NullOracle {
    /// # Errors
    /// Whatever the lane's decide path returns — propagated loud, never
    /// mapped to a fake answer.
    fn answer_null(&mut self, case: &SuiteCase) -> Result<AugAnswer, String>;

    /// The paired noul leg (A3a). Lanes with no noul readout return
    /// `None` per case (disclosed in the cell; never a silent pass).
    fn answer_noul(&mut self, case: &SuiteCase) -> Result<Option<NoulAnswer>, String> {
        let _ = case;
        Ok(None)
    }

    /// Whether the lane can abstain at all (the wire's `outcome: None`).
    /// Comparison deciders cannot — their A1 abstain columns read n/a,
    /// never zero.
    fn can_abstain(&self) -> bool {
        false
    }
}

/// One arm's readout for one slot, in ORIGINAL-label space: the reference
/// pair's probabilities, whether a decoy won the argmax, the abstention,
/// and the confidence.
#[derive(Debug, Clone, Serialize)]
pub struct ArmReadout {
    pub k: usize,
    /// Presented option count at this arm.
    pub n_presented: usize,
    pub p_a: f64,
    pub p_b: f64,
    /// The top pick's label is a DECOY (legitimate under Prop 1's scope —
    /// disclosed, never a failure).
    pub decoy_won: bool,
    pub abstained: bool,
    pub confidence: f64,
    /// The arm's scoring regime: "routed" / "drafter-only" (the modelless
    /// engine's k ≤ N by-name arming law) or "n/a" (comparison lanes).
    pub regime: &'static str,
}

/// The paired noul readouts, arm-aligned (`arms[i]` ↔ `noul[i]`).
#[derive(Debug, Clone, Serialize)]
pub struct NoulLeg {
    pub answered: bool,
    /// p(yes) bits identical across every arm (the structural pin).
    pub bit_identical: bool,
    pub p_yes: Vec<f64>,
}

/// One slot's full record across the augmentation ladder.
#[derive(Debug, Clone, Serialize)]
pub struct NullCaseRecord {
    pub case_id: String,
    pub qid: String,
    pub n_options: usize,
    pub gold: Option<String>,
    /// The reference arm's top-2 ORIGINAL labels (A then B by probability,
    /// first-position tie-break).
    pub a_label: String,
    pub b_label: String,
    /// The reference arm's top-2 probabilities tie EXACTLY — any later
    /// p_A/p_b order swap is the tie structure, not a preference flip.
    pub tie: bool,
    pub arms: Vec<ArmReadout>,
    pub noul: NoulLeg,
}

/// Compute one slot's record from its arm answers. `ladder[i]` is
/// (requested K, actual presented count) — the pool may cap an arm below
/// its request (closed label spaces; the lane discloses the cap).
/// `noul[i]` is the paired noul answer (None ⇒ the leg reads
/// unanswered). The REFERENCE pair (A, B) is fixed by the FIRST arm (the
/// unaugmented question — the ladder must start at K=0).
///
/// # Errors
/// When an answer's width disagrees with its arm's presented-option count
/// or the ladder is empty.
pub fn null_case_record(
    case_id: &str,
    qid: &str,
    labels: &[String],
    ladder: &[(usize, usize)],
    answers: &[AugAnswer],
    noul: &[Option<NoulAnswer>],
    gold_label: Option<&str>,
) -> Result<NullCaseRecord, String> {
    let k_set: Vec<usize> = ladder.iter().map(|(k, _)| *k).collect();
    let presented: Vec<usize> = ladder.iter().map(|(_, p)| *p).collect();
    if ladder.is_empty() || ladder.len() != answers.len() || noul.len() != answers.len() {
        return Err(format!(
            "case {case_id}/{qid}: ladder shape mismatch ({} arms, {} answers, {} noul)",
            ladder.len(),
            answers.len(),
            noul.len()
        ));
    }
    let n = labels.len();
    if answers[0].probs.len() != n {
        return Err(format!(
            "case {case_id}/{qid}: reference arm width {} != option count {n}",
            answers[0].probs.len()
        ));
    }
    // The reference pair: top-2 among the ORIGINALS at the first arm.
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&x, &y| answers[0].probs[y].total_cmp(&answers[0].probs[x]));
    let a = idx[0];
    let b = idx[1];
    let tie = answers[0].probs[a] == answers[0].probs[b];
    let regime_of = |ans: &AugAnswer| -> &'static str {
        match ans.drafter_only {
            Some(true) => "drafter-only",
            Some(false) => "routed",
            None => "n/a",
        }
    };
    let mut arms = Vec::with_capacity(answers.len());
    for (i, ans) in answers.iter().enumerate() {
        let n_pres = presented[i];
        if ans.probs.len() != n_pres {
            return Err(format!(
                "case {case_id}/{qid}: arm {} width {} != presented {n_pres}",
                k_set[i],
                ans.probs.len()
            ));
        }
        if ans.pick >= n_pres {
            return Err(format!(
                "case {case_id}/{qid}: arm {} pick {} out of range {n_pres}",
                k_set[i], ans.pick
            ));
        }
        arms.push(ArmReadout {
            k: k_set[i],
            n_presented: n_pres,
            p_a: ans.probs[a],
            p_b: ans.probs[b],
            decoy_won: ans.pick >= n,
            abstained: ans.abstained,
            confidence: ans.confidence,
            regime: regime_of(ans),
        });
    }
    let answered = noul.iter().any(|x| x.is_some());
    let p_yes: Vec<f64> = noul
        .iter()
        .map(|x| x.as_ref().map(|y| y.p_yes).unwrap_or(f64::NAN))
        .collect();
    let bit_identical = answered && {
        let first = noul.iter().flatten().next().expect("answered");
        noul.iter()
            .flatten()
            .all(|x| x.p_yes.to_bits() == first.p_yes.to_bits())
    };
    Ok(NullCaseRecord {
        case_id: case_id.to_string(),
        qid: qid.to_string(),
        n_options: n,
        gold: gold_label.map(str::to_string),
        a_label: labels[a].clone(),
        b_label: labels[b].clone(),
        tie,
        arms,
        noul: NoulLeg {
            answered,
            bit_identical,
            p_yes,
        },
    })
}

// ─────────────────────────────────────────────────── aggregates ──

/// The gate verdict for a probe cell (A2 flips, A3 bit-identity, the
/// floor leg). A CONTROL red refuses the run; comparison-lane reds are
/// disclosed rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NullVerdict {
    Pass,
    Red,
}

impl NullVerdict {
    #[must_use]
    pub fn str_of(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Red => "RED",
        }
    }
}

/// A1 per arm: the null/coverage drift curve (characterized, not gated —
/// the floor leg owns the gate).
#[derive(Debug, Clone, Serialize)]
pub struct A1ArmStats {
    pub k: usize,
    pub abstain_rate: f64,
    /// Abstain rate minus the reference arm's, in points.
    pub d_abstain_pt: f64,
    /// Median |Δconfidence| vs the reference arm, in points.
    pub median_abs_d_conf_pt: f64,
    /// Fraction of slots whose argmax moved to a decoy at this arm.
    pub decoy_win_rate: f64,
}

/// A2 per arm: the preference-flip census (ties at the reference arm
/// exempt). `flips` splits by REGIME: within-regime flips (the gated
/// count — the scoring function is fixed within a regime, so a
/// preference flip there is set-sensitivity, red) and cross-regime flips
/// (the modelless engine's k ≤ N by-name arming law switching regimes BY
/// DESIGN — disclosed, never red).
#[derive(Debug, Clone, Serialize)]
pub struct A2ArmStats {
    pub k: usize,
    pub n_untied: usize,
    pub flips: usize,
    pub flip_rate: f64,
    /// Flips on slots whose reference arm shares this arm's regime (the
    /// gated count; the verdict keys on this).
    pub within_regime_flips: usize,
    /// Flips on slots whose reference arm is in the OTHER regime (the
    /// arming-law disclosure).
    pub cross_regime_flips: usize,
    pub verdict: NullVerdict,
}

/// A3: the noul-conditional shape.
#[derive(Debug, Clone, Serialize)]
pub struct A3Stats {
    /// Leg (a): the paired noul readout is bit-identical across arms
    /// (None when the lane ships no noul readout — disclosed, never a
    /// pass).
    pub noul_bit_identical: Option<bool>,
    /// Leg (b): the choice-side confidence shape, classified bit-stable
    /// (conditional-shaped) vs drifting (joint-shaped) — pinned, not
    /// gated: a joint-shaped lane is a finding, not a failure.
    pub conf_shape: &'static str,
    /// Median |Δconfidence| across the top arm (the shape's evidence).
    pub median_abs_d_conf_top_pt: f64,
}

/// Aggregate A1/A2 over the slot records. `reference` is the ladder's
/// first arm index (0 by construction).
#[must_use]
pub fn a1_stats(recs: &[NullCaseRecord]) -> Vec<A1ArmStats> {
    let Some(first) = recs.first() else {
        return Vec::new();
    };
    let n = recs.len();
    first
        .arms
        .iter()
        .enumerate()
        .map(|(ai, ref_arm)| {
            let abst = |r: &NullCaseRecord| usize::from(r.arms[ai].abstained);
            let abstain_rate = recs.iter().map(abst).sum::<usize>() as f64 / n as f64;
            let ref_abstain_rate =
                recs.iter().map(|r| usize::from(r.arms[0].abstained)).sum::<usize>() as f64
                    / n as f64;
            let mut dconf: Vec<f64> = recs
                .iter()
                .map(|r| (r.arms[ai].confidence - r.arms[0].confidence).abs() * 100.0)
                .collect();
            dconf.sort_by(|x, y| x.total_cmp(y));
            let median_dconf = if dconf.is_empty() {
                f64::NAN
            } else {
                dconf[dconf.len() / 2]
            };
            let decoy_wins = recs
                .iter()
                .filter(|r| r.arms[ai].decoy_won)
                .count();
            A1ArmStats {
                k: ref_arm.k,
                abstain_rate,
                d_abstain_pt: (abstain_rate - ref_abstain_rate) * 100.0,
                median_abs_d_conf_pt: median_dconf,
                decoy_win_rate: decoy_wins as f64 / n as f64,
            }
        })
        .collect()
}

/// A2 per arm: `sign(p_a − p_b)` flips vs the reference arm, exact-tie
/// reference slots exempt, split by regime (within = gated, cross = the
/// arming-law disclosure).
#[must_use]
pub fn a2_stats(recs: &[NullCaseRecord]) -> Vec<A2ArmStats> {
    let Some(first) = recs.first() else {
        return Vec::new();
    };
    first
        .arms
        .iter()
        .enumerate()
        .map(|(ai, arm)| {
            let untied: Vec<&NullCaseRecord> = recs.iter().filter(|r| !r.tie).collect();
            let n_untied = untied.len();
            let mut within = 0usize;
            let mut cross = 0usize;
            for r in &untied {
                let ref_arm = &r.arms[0];
                let a = &r.arms[ai];
                let s0 = ref_arm.p_a - ref_arm.p_b;
                let s1 = a.p_a - a.p_b;
                // Only a STRICT reference preference can flip; an exact
                // tie at a later arm is movement TOWARD neutral (counted
                // only when the sign actually reverses).
                if s0 > 0.0 && s1 < 0.0 || s0 < 0.0 && s1 > 0.0 {
                    if a.regime == ref_arm.regime {
                        within += 1;
                    } else {
                        cross += 1;
                    }
                }
            }
            let flip_rate = if n_untied == 0 {
                0.0
            } else {
                (within + cross) as f64 / n_untied as f64
            };
            A2ArmStats {
                k: arm.k,
                n_untied,
                flips: within + cross,
                flip_rate,
                within_regime_flips: within,
                cross_regime_flips: cross,
                verdict: if n_untied == 0 || within == 0 {
                    NullVerdict::Pass
                } else {
                    NullVerdict::Red
                },
            }
        })
        .collect()
}

/// A3 aggregate: the noul bit-identity leg + the confidence-shape
/// classification (bit-stable ⇒ conditional-shaped; any drift ⇒
/// joint-shaped — a finding, pinned not gated).
#[must_use]
pub fn a3_stats(recs: &[NullCaseRecord]) -> A3Stats {
    let answered: Vec<&NullCaseRecord> = recs.iter().filter(|r| r.noul.answered).collect();
    let noul_bit_identical = (!answered.is_empty()).then(|| {
        answered
            .iter()
            .all(|r| r.noul.bit_identical)
    });
    let top = recs
        .first()
        .map(|r| r.arms.len().saturating_sub(1))
        .unwrap_or(0);
    let mut dconf: Vec<f64> = recs
        .iter()
        .map(|r| (r.arms[top].confidence - r.arms[0].confidence).abs() * 100.0)
        .collect();
    dconf.sort_by(|x, y| x.total_cmp(y));
    let median = if dconf.is_empty() {
        f64::NAN
    } else {
        dconf[dconf.len() / 2]
    };
    let conf_shape = if recs
        .iter()
        .all(|r| r.arms[top].confidence.to_bits() == r.arms[0].confidence.to_bits())
    {
        "conditional-shaped (bit-stable across arms — state-side readout)"
    } else {
        "joint-shaped (drifts with K — option-dependent readout)"
    };
    A3Stats {
        noul_bit_identical,
        conf_shape,
        median_abs_d_conf_top_pt: median,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(n_opts: usize) -> SuiteCase {
        let mut m = Map::new();
        for l in ["alpha", "beta", "gamma", "delta", "epsilon"]
            .iter()
            .take(n_opts)
        {
            m.insert((*l).to_string(), Value::String(format!("the {l} option")));
        }
        SuiteCase {
            id: "c1".to_string(),
            state: Value::String("state text".to_string()),
            questions: vec![SuiteQuestion {
                qid: "q1".to_string(),
                kind: QKind::Choice,
                instructions: "pick".to_string(),
                criteria: Value::Object(m),
            }],
            gold: vec![GoldAnswer {
                idx: 0,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    fn ans(k: usize, p_a: f64, p_b: f64, conf: f64) -> AugAnswer {
        ans_regime(k, p_a, p_b, conf, Some(false))
    }

    /// `drafter_only` parameterized: the regime axis is A2's split (a
    /// routed reference vs drafter-only arm = the arming-law disclosure).
    fn ans_regime(k: usize, p_a: f64, p_b: f64, conf: f64, dr: Option<bool>) -> AugAnswer {
        let n = 5 + k;
        let mut probs = vec![0.0; n];
        probs[0] = p_a;
        probs[1] = p_b;
        AugAnswer {
            probs,
            pick: 0,
            abstained: false,
            confidence: conf,
            drafter_only: dr,
        }
    }

    #[test]
    fn trigram_jaccard_grades_and_edges() {
        assert!(trigram_jaccard("card arrival", "card arrived") > 0.5);
        assert_eq!(trigram_jaccard("", ""), 1.0);
        assert_eq!(trigram_jaccard("", "abc"), 0.0);
        assert!(trigram_jaccard("sports news", "sports") > trigram_jaccard("sports news", "card arrival"));
    }

    #[test]
    fn decoys_exclude_slot_grade_and_cap() {
        let pool = DecoyPool::build(&[vec![
            "sports".to_string(),
            "business".to_string(),
            "world".to_string(),
            "tech".to_string(),
            "card arrival".to_string(),
            "balance check".to_string(),
        ]]);
        let slot = vec!["sports".to_string(), "business".to_string()];
        let d = pool.graded_decoys(&slot, Some("sports"), 3, NULL_PROBE_SEED);
        assert_eq!(d.len(), 3);
        assert!(!d.iter().any(|x| slot.contains(x)));
        // Deterministic.
        assert_eq!(d, pool.graded_decoys(&slot, Some("sports"), 3, NULL_PROBE_SEED));
        // Fewer candidates than k: capped, no duplicates.
        let d2 = pool.graded_decoys(&slot, Some("sports"), 10, NULL_PROBE_SEED);
        assert_eq!(d2.len(), 4);
        let mut sorted = d2.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), d2.len());
        // Graded: the first pick is at least as similar to the anchor as
        // the last (stride over a similarity-DESC ranking).
        let sim = |s: &str| trigram_jaccard(s, "sports");
        assert!(sim(&d[0]) >= sim(&d[d.len() - 1]));
    }

    #[test]
    fn augmented_case_appends_and_keeps_gold() {
        let c = case(5);
        let decoys = vec!["world".to_string(), "tech".to_string()];
        let a = augmented_case(&c, 0, &decoys).expect("augment");
        let obj = a.questions[0].criteria.as_object().expect("obj");
        let keys: Vec<&String> = obj.keys().collect();
        assert_eq!(keys.len(), 7);
        // Originals keep their positions.
        assert_eq!(keys[0], "alpha");
        assert_eq!(keys[4], "epsilon");
        assert_eq!(keys[5], "world");
        assert_eq!(keys[6], "tech");
        assert_eq!(a.gold[0].idx, 0);
        // Collision refused.
        assert!(augmented_case(&c, 0, &["alpha".to_string()]).is_err());
    }

    #[test]
    fn noul_case_is_option_free() {
        let c = case(5);
        let n = noul_case(&c, 0);
        assert_eq!(n.questions[0].kind, QKind::Noul);
        assert!(n.questions[0].criteria.is_null());
        assert_eq!(n.questions[0].instructions, NOUL_PROBE_INSTRUCTIONS);
    }

    #[test]
    fn record_reads_pair_and_decoy_wins() {
        let c = case(5);
        let labels: Vec<String> = c.questions[0]
            .criteria
            .as_object()
            .expect("obj")
            .keys()
            .cloned()
            .collect();
        let mut a1 = ans(0, 0.5, 0.3, 0.8);
        a1.pick = 0;
        let mut a2 = ans(8, 0.2, 0.1, 0.6);
        a2.probs[6] = 0.9; // a decoy wins
        a2.pick = 6;
        let rec = null_case_record(
            "c1",
            "q1",
            &labels,
            &[(0, 5), (8, 13)],
            &[a1, a2],
            &[Some(NoulAnswer { p_yes: 0.25, yes: Some(false), confidence: 0.5 }),
             Some(NoulAnswer { p_yes: 0.25, yes: Some(false), confidence: 0.5 })],
            Some("alpha"),
        )
        .expect("record");
        assert_eq!(rec.a_label, "alpha");
        assert_eq!(rec.b_label, "beta");
        assert!(!rec.tie);
        assert!(!rec.arms[0].decoy_won);
        assert!(rec.arms[1].decoy_won);
        assert!(rec.noul.answered);
        assert!(rec.noul.bit_identical);
        // Shape mismatch refused.
        assert!(null_case_record("c1", "q1", &labels, &[(0, 5)], &[ans(8, 0.5, 0.3, 0.8)], &[None], None).is_err());
    }

    #[test]
    fn a2_detects_preference_flip_and_exempts_ties() {
        let c = case(5);
        let labels: Vec<String> = c.questions[0]
            .criteria
            .as_object()
            .expect("obj")
            .keys()
            .cloned()
            .collect();
        // Slot 1: strict preference 0.5/0.3 that FLIPS at K=64.
        let flip = null_case_record(
            "c1",
            "q1",
            &labels,
            &[(0, 5), (64, 69)],
            &[ans(0, 0.5, 0.3, 0.7), ans(64, 0.2, 0.4, 0.7)],
            &[None, None],
            None,
        )
        .expect("record");
        // Slot 2: exact reference tie — exempt even though the orders swap.
        let mut t0 = ans(0, 0.4, 0.4, 0.7);
        t0.pick = 0;
        let tie = null_case_record(
            "c2",
            "q1",
            &labels,
            &[(0, 5), (64, 69)],
            &[t0, ans(64, 0.2, 0.5, 0.7)],
            &[None, None],
            None,
        )
        .expect("record");
        assert!(tie.tie);
        let stats = a2_stats(&[flip.clone(), tie.clone()]);
        assert_eq!(stats[1].n_untied, 1);
        assert_eq!(stats[1].flips, 1);
        assert_eq!(stats[1].within_regime_flips, 1);
        assert_eq!(stats[1].cross_regime_flips, 0);
        assert_eq!(stats[1].verdict, NullVerdict::Red);
        // The SAME flip magnitude across regimes (routed reference →
        // drafter-only arm): disclosed as the arming law, never red.
        let mut routed_ref = ans(0, 0.5, 0.3, 0.7);
        routed_ref.drafter_only = Some(false);
        let mut switched = ans(64, 0.2, 0.4, 0.7);
        switched.drafter_only = Some(true);
        let cross = null_case_record(
            "c3",
            "q1",
            &labels,
            &[(0, 5), (64, 69)],
            &[routed_ref, switched],
            &[None, None],
            None,
        )
        .expect("record");
        let cs = a2_stats(&[cross]);
        assert_eq!(cs[1].flips, 1);
        assert_eq!(cs[1].within_regime_flips, 0);
        assert_eq!(cs[1].cross_regime_flips, 1);
        assert_eq!(cs[1].verdict, NullVerdict::Pass);
        // The tie-only population is fully exempt.
        let all_tie = a2_stats(std::slice::from_ref(&tie));
        assert_eq!(all_tie[1].flips, 0);
        assert_eq!(all_tie[1].verdict, NullVerdict::Pass);
    }

    #[test]
    fn a3_classifies_shapes_and_checks_noul_bits() {
        let c = case(5);
        let labels: Vec<String> = c.questions[0]
            .criteria
            .as_object()
            .expect("obj")
            .keys()
            .cloned()
            .collect();
        // Conditional-shaped: conf bit-stable, noul bit-identical.
        let cond = null_case_record(
            "c1",
            "q1",
            &labels,
            &[(0, 5), (256, 261)],
            &[ans(0, 0.5, 0.3, 0.7), ans(256, 0.1, 0.05, 0.7)],
            &[Some(NoulAnswer { p_yes: 0.25, yes: Some(false), confidence: 0.4 }),
             Some(NoulAnswer { p_yes: 0.25, yes: Some(false), confidence: 0.4 })],
            None,
        )
        .expect("record");
        let s = a3_stats(&[cond]);
        assert_eq!(s.noul_bit_identical, Some(true));
        assert!(s.conf_shape.starts_with("conditional"));
        // Joint-shaped: conf drifts; noul bits differ — both surfaced.
        let joint = null_case_record(
            "c2",
            "q1",
            &labels,
            &[(0, 5), (256, 261)],
            &[ans(0, 0.5, 0.3, 0.7), ans(256, 0.1, 0.05, 0.4)],
            &[Some(NoulAnswer { p_yes: 0.25, yes: Some(false), confidence: 0.4 }),
             Some(NoulAnswer { p_yes: 0.26, yes: Some(false), confidence: 0.4 })],
            None,
        )
        .expect("record");
        let s2 = a3_stats(&[joint]);
        assert_eq!(s2.noul_bit_identical, Some(false));
        assert!(s2.conf_shape.starts_with("joint"));
        // No noul readout: disclosed as None, never a pass.
        let noleg = null_case_record(
            "c3",
            "q1",
            &labels,
            &[(0, 5), (8, 13)],
            &[ans(0, 0.5, 0.3, 0.7), ans(8, 0.3, 0.2, 0.7)],
            &[None, None],
            None,
        )
        .expect("record");
        assert_eq!(a3_stats(&[noleg]).noul_bit_identical, None);
    }

    #[test]
    fn a1_reads_drift_curve() {
        let c = case(5);
        let labels: Vec<String> = c.questions[0]
            .criteria
            .as_object()
            .expect("obj")
            .keys()
            .cloned()
            .collect();
        let mk = |k: usize, abst: bool, conf: f64| {
            let mut a = ans(k, 0.5, 0.3, conf);
            a.abstained = abst;
            a
        };
        let r1 = null_case_record(
            "c1",
            "q1",
            &labels,
            &[(0, 5), (64, 69)],
            &[mk(0, false, 0.8), mk(64, true, 0.5)],
            &[None, None],
            None,
        )
        .expect("record");
        let r2 = null_case_record(
            "c2",
            "q1",
            &labels,
            &[(0, 5), (64, 69)],
            &[mk(0, false, 0.6), mk(64, false, 0.6)],
            &[None, None],
            None,
        )
        .expect("record");
        let stats = a1_stats(&[r1, r2]);
        assert_eq!(stats[0].abstain_rate, 0.0);
        assert!((stats[1].abstain_rate - 0.5).abs() < 1e-12);
        assert!((stats[1].d_abstain_pt - 50.0).abs() < 1e-12);
        // median |dconf| over [0, 30] pt sorted = [0, 30]; the len/2
        // index convention picks element 1 = 30 pt (the record discloses
        // the curve; the exact quantile convention is pinned by this
        // test).
        assert!((stats[1].median_abs_d_conf_pt - 30.0).abs() < 1e-12);
    }
}
