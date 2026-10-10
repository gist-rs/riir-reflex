//! probe_delta — the FlyBy paired counterfactual ΔV certification
//! (Plan 621 Phase 5 T5.1; Research 609 §2, arXiv:2609.34327).
//!
//! Offline paired certification that an intervention class moves value,
//! over ONE frozen read: for each decision state (question) two arms —
//! control `A` and intervention `B` — produce a paired outcome, and the
//! ΔV distribution over the states is the evidence. This is the protocol
//! FlyBy used to DISCOVER the execution/knowledge split; here it is the
//! certification instrument any paired A/B lane consumes.
//!
//! The certification consumes the Plan-621 probe kernel
//! ([`katgpt_core::state_probe`]): the DISCORDANT pairs are the ensemble
//! (`pass_count` = B-wins, `n` = wins + losses), the win-rate over them is
//! V̂, and `classify(ε = 0.5)` is the sign-test decision:
//!
//! * `MovesValue` (wilson_lo ≥ 0.5) — B wins discordant pairs decisively:
//!   the intervention moves value on this population.
//! * `NoValue` (wilson_hi < 0.5) — B loses decisively even optimistically:
//!   the intervention provably does not move value here.
//! * `Undetermined` — the interval straddles break-even: more states
//!   needed at this ε.
//!
//! The adaptive prefix walk feeds cumulative estimates (first N states,
//! question order fixed) through the kernel's `AdaptiveClassifier` —
//! [`ProbeDeltaCert::settled_at_n`] is the smallest N at which the
//! certification WOULD have settled (the online reading: how many states
//! this intervention needs), with every class flip disclosed WITH its
//! interval width (T2.1's disclosure duty). The walk stops at the first
//! decisive prefix; **the certification of record is the FULL frozen
//! read** (the one-frozen-test-read law) — a settled prefix that disagrees
//! with the full-set verdict is disclosed by the pair of fields, never
//! hidden.
//!
//! AUGMENTS, never replaces: the consuming lane's own gate law (the
//! corpus-ab V5 paired-LB95 verdict) is unchanged — this record is
//! additive disclosure. h_mm is disclosure context only (the outcome
//! concentration); classification reads only the Wilson bounds, and the
//! histogram feeding it clamps at u16::MAX (suites are thousands, not
//! millions — the clamp is unreachable in practice and never gate-bearing).
//!
//! Zero-alloc where the kernel is: the walk's flip log rides the stack;
//! the returned record allocates by design (report-side, not hot path —
//! the Phase-4 eval instrument's posture).

use katgpt_core::state_probe::{
    AdaptiveClassifier, AdaptiveDecision, ClassFlip, ProbeClass, ProbeEstimate, ProbeInput,
    classify, probe,
};

use serde::Serialize;

/// The certification bar: break-even over the discordant pairs. The sign
/// test's decision rule — an intervention "moves value" iff its win-rate
/// over the pairs it could change is decisively above a coin flip.
pub const EPSILON: f64 = 0.5;

/// The two-sided 95% Wilson critical value (the kernel docs' own spelling).
pub const Z95: f64 = 1.959963984540054;

/// Prefix sizes for the adaptive walk (the kernel's n_min-doubling law,
/// 8 → 8192; the full population always rides as the last rung).
const PREFIX_RUNGS: [u32; 11] = [
    8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
];

/// The reflex-side verdict vocabulary (the wire shape for the kernel's
/// `ProbeClass` — a Copy POD substrate type carries no serde; the mapping
/// is disclosed here, one direction, total).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProbeDeltaVerdict {
    /// `ProbeClass::Productive` — the intervention moves value.
    MovesValue,
    /// `ProbeClass::KnowledgeLike` — the intervention provably does not.
    NoValue,
    /// `ProbeClass::Undetermined` — more states needed at this ε.
    Undetermined,
}

impl ProbeDeltaVerdict {
    fn of(class: ProbeClass) -> Self {
        match class {
            ProbeClass::Productive => Self::MovesValue,
            ProbeClass::KnowledgeLike => Self::NoValue,
            ProbeClass::Undetermined => Self::Undetermined,
        }
    }
}

/// One disclosed class flip in the adaptive walk (the kernel's `ClassFlip`
/// with the reflex-side verdict vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ProbeDeltaFlip {
    /// The prefix size at which the class flipped.
    pub at_n: u32,
    /// The class held before this observation.
    pub from: ProbeDeltaVerdict,
    /// The class held after it.
    pub to: ProbeDeltaVerdict,
    /// `wilson_hi − wilson_lo` at the flip — the width that decided it.
    pub interval_width: f64,
}

/// The certification record over one paired A/B read (the wire shape —
/// plain fields, no substrate types).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProbeDeltaCert {
    /// Total paired states (ties included).
    pub n_pairs: usize,
    /// B right where A was wrong (the intervention's wins).
    pub wins: usize,
    /// B wrong where A was right (the intervention's losses).
    pub losses: usize,
    /// Both arms agreed (these never enter the ensemble).
    pub ties: usize,
    /// V̂ — the win-rate over the DISCORDANT pairs.
    pub v_hat: f64,
    /// The Wilson lower bound on V̂ (95%, two-sided).
    pub wilson_lo: f64,
    /// The Wilson upper bound on V̂.
    pub wilson_hi: f64,
    /// Miller–Madow outcome entropy over [losses, wins] — disclosure
    /// context only (all-wins concentrates to 0; a spread read widens it).
    pub h_mm: f64,
    /// The certification of record: the FULL frozen read's class.
    pub verdict: ProbeDeltaVerdict,
    /// The smallest prefix N at which the adaptive walk settled (the
    /// online reading: how many states this certification needs). `None`
    /// when the walk never settled — including the empty read.
    pub settled_at_n: Option<u32>,
    /// Every class flip before the settle, in observation order, each
    /// with its interval width.
    pub flips: Vec<ProbeDeltaFlip>,
}

/// Feed one adaptive rung: advance the cumulative (wins, losses) over the
/// prefix `0..upto`, observe through the classifier, return whether the
/// class settled. `upto` never exceeds the population (callers clip).
fn feed_rung(
    walker: &mut AdaptiveClassifier<'_>,
    arm_a: &[bool],
    arm_b: &[bool],
    upto: u32,
    idx: &mut usize,
    cum: &mut (usize, usize),
) -> bool {
    let clip = (upto as usize).min(arm_a.len());
    while *idx < clip {
        match (arm_b[*idx], arm_a[*idx]) {
            (true, false) => cum.0 += 1,
            (false, true) => cum.1 += 1,
            _ => {}
        }
        *idx += 1;
    }
    let e = discordant_estimate(cum.0 as u64, cum.1 as u64);
    matches!(
        walker.observe(&e, clip as u32),
        AdaptiveDecision::Settled(_)
    )
}

/// The discordant-set probe estimate (the kernel entry — plain counts in,
/// estimates out; the histogram bins are [losses, wins], clamped at
/// u16::MAX — disclosure context only, never gate-bearing).
fn discordant_estimate(wins: u64, losses: u64) -> ProbeEstimate {
    let n = wins.saturating_add(losses);
    let hist = [
        losses.min(u64::from(u16::MAX)) as u16,
        wins.min(u64::from(u16::MAX)) as u16,
    ];
    probe(
        &ProbeInput {
            pass_count: wins.min(u64::from(u32::MAX)) as u32,
            n: n.min(u64::from(u32::MAX)) as u32,
            histogram: &hist,
        },
        Z95,
    )
}

/// Certify one paired A/B read: `arm_a` = control, `arm_b` = intervention,
/// index-aligned over the SAME states in a FIXED order (the walk is
/// order-sensitive by design — the consumer's canonical question order).
///
/// Panics when the arms disagree in length — a mis-paired read is a bug in
/// the caller, never a certification.
pub fn certify(arm_a: &[bool], arm_b: &[bool]) -> ProbeDeltaCert {
    assert_eq!(
        arm_a.len(),
        arm_b.len(),
        "probe_delta::certify: arm lengths disagree ({} vs {}) — the read is mis-paired",
        arm_a.len(),
        arm_b.len()
    );
    assert!(
        arm_a.len() <= u32::MAX as usize,
        "probe_delta::certify: population overflows the kernel's u32 N"
    );
    let mut wins = 0usize;
    let mut losses = 0usize;
    for (a, b) in arm_a.iter().zip(arm_b.iter()) {
        match (*b, *a) {
            (true, false) => wins += 1,
            (false, true) => losses += 1,
            _ => {}
        }
    }
    let ties = arm_a.len() - wins - losses;

    // The certification of record: the FULL frozen read.
    let est = discordant_estimate(wins as u64, losses as u64);
    let verdict = ProbeDeltaVerdict::of(classify(&est, EPSILON));

    // The adaptive prefix walk (disclosure): stop at the first decisive
    // prefix — the online reading of the certification cost. The full
    // population always rides as the last rung; `CappedUndetermined` there
    // is the honest "undetermined at the full read", which the verdict
    // field already carries.
    let mut flips_buf = [ClassFlip {
        at_n: 0,
        from: ProbeClass::Undetermined,
        to: ProbeClass::Undetermined,
        interval_width: 0.0,
    }; 8];
    let mut walker = AdaptiveClassifier::new(EPSILON, arm_a.len() as u32, &mut flips_buf);
    let mut settled_at_n: Option<u32> = None;
    let mut idx = 0usize;
    let mut cum = (0usize, 0usize); // (wins, losses) over the fed prefix
    for r in PREFIX_RUNGS {
        if r as usize >= arm_a.len() {
            break;
        }
        if feed_rung(&mut walker, arm_a, arm_b, r, &mut idx, &mut cum) {
            settled_at_n = Some(r);
            break;
        }
    }
    if settled_at_n.is_none() && idx < arm_a.len() {
        feed_rung(&mut walker, arm_a, arm_b, arm_a.len() as u32, &mut idx, &mut cum);
    }

    let flips: Vec<ProbeDeltaFlip> = walker
        .flips()
        .iter()
        .map(|f| ProbeDeltaFlip {
            at_n: f.at_n,
            from: ProbeDeltaVerdict::of(f.from),
            to: ProbeDeltaVerdict::of(f.to),
            interval_width: f.interval_width,
        })
        .collect();

    ProbeDeltaCert {
        n_pairs: arm_a.len(),
        wins,
        losses,
        ties,
        v_hat: est.v_hat,
        wilson_lo: est.wilson_lo,
        wilson_hi: est.wilson_hi,
        h_mm: est.h_mm,
        verdict,
        settled_at_n,
        flips,
    }
}

#[cfg(test)]
mod tests {
    use super::{EPSILON, ProbeDeltaVerdict, certify};

    /// A win = B right where A wrong; a loss = the reverse; a tie = agree.
    fn arms(spec: &[(bool, bool)]) -> (Vec<bool>, Vec<bool>) {
        (
            spec.iter().map(|s| s.0).collect(),
            spec.iter().map(|s| s.1).collect(),
        )
    }

    #[test]
    fn all_wins_certify_moves_value() {
        let spec = [(false, true); 64];
        let (a, b) = arms(&spec);
        let c = certify(&a, &b);
        assert_eq!(c.verdict, ProbeDeltaVerdict::MovesValue);
        assert_eq!((c.wins, c.losses, c.ties), (64, 0, 0));
        assert_eq!(c.v_hat, 1.0);
        // The first rung (8) is already decisive all-wins — no flip, no
        // earlier observation.
        assert_eq!(c.settled_at_n, Some(8));
        assert!(c.flips.is_empty());
        // All-wins concentrates the outcome histogram: h_mm at its floor.
        assert!(c.h_mm.abs() < 1e-9);
    }

    #[test]
    fn all_losses_certify_no_value() {
        let spec = [(true, false); 64];
        let (a, b) = arms(&spec);
        let c = certify(&a, &b);
        assert_eq!(c.verdict, ProbeDeltaVerdict::NoValue);
        assert_eq!(c.v_hat, 0.0);
        assert_eq!(c.settled_at_n, Some(8));
    }

    #[test]
    fn coin_flip_stays_undetermined_at_the_full_read() {
        // Perfectly balanced discordant set: the interval straddles 0.5 at
        // EVERY rung, including the full population — never settles.
        let spec: Vec<(bool, bool)> = (0..128)
            .map(|i| {
                if i % 2 == 0 {
                    (false, true)
                } else {
                    (true, false)
                }
            })
            .collect();
        let (a, b) = arms(&spec);
        let c = certify(&a, &b);
        assert_eq!(c.verdict, ProbeDeltaVerdict::Undetermined);
        assert_eq!(c.settled_at_n, None);
        assert_eq!(c.v_hat, 0.5);
        assert!(c.wilson_lo < EPSILON && c.wilson_hi > EPSILON);
    }

    #[test]
    fn ties_never_enter_the_ensemble() {
        // 12 wins / 3 losses / 380 ties: v̂ reads over the 15 discordant
        // pairs ONLY (0.8 — wilson_lo ≈ 0.548 clears ε).
        let mut spec = vec![(false, true); 12];
        spec.extend(vec![(true, false); 3]);
        spec.extend(vec![(false, false); 380]);
        let (a, b) = arms(&spec);
        let c = certify(&a, &b);
        assert_eq!((c.n_pairs, c.wins, c.losses, c.ties), (395, 12, 3, 380));
        assert!((c.v_hat - 0.8).abs() < 1e-12);
        assert_eq!(c.verdict, ProbeDeltaVerdict::MovesValue);
    }

    #[test]
    fn settled_prefix_that_disagrees_with_the_full_read_is_disclosed_not_hidden() {
        // First 8 states all-wins (the walk settles MovesValue at 8), then
        // the population reverses: the FULL read certifies NoValue. Both
        // fields stand — settled_at_n is the online disclosure, verdict is
        // the certification of record.
        let mut spec = vec![(false, true); 8];
        spec.extend(vec![(true, false); 24]);
        let (a, b) = arms(&spec);
        let c = certify(&a, &b);
        assert_eq!(c.settled_at_n, Some(8));
        assert_eq!(c.verdict, ProbeDeltaVerdict::NoValue);
        assert_eq!((c.wins, c.losses), (8, 24));
    }

    #[test]
    fn undetermined_then_decisive_flips_are_disclosed_with_width() {
        // First 8 discordant pairs split 4/4 (undetermined at rung 8), the
        // next 8 all wins (decisive at 16): exactly one flip,
        // Undetermined → MovesValue, carrying a positive interval width.
        let mut spec: Vec<(bool, bool)> = (0..8)
            .map(|i| {
                if i % 2 == 0 {
                    (false, true)
                } else {
                    (true, false)
                }
            })
            .collect();
        spec.extend(vec![(false, true); 8]);
        spec.extend(vec![(false, false); 100]); // ties pad past rung 16
        let (a, b) = arms(&spec);
        let c = certify(&a, &b);
        assert_eq!(c.settled_at_n, Some(16));
        assert_eq!(c.verdict, ProbeDeltaVerdict::MovesValue);
        assert_eq!(c.flips.len(), 1, "one flip: undetermined → decisive");
        let f = c.flips[0];
        assert_eq!(f.at_n, 16);
        assert_eq!(f.from, ProbeDeltaVerdict::Undetermined);
        assert_eq!(f.to, ProbeDeltaVerdict::MovesValue);
        assert!(f.interval_width > 0.0, "the flip carries the deciding width");
        assert!((f.interval_width - (c.wilson_hi - c.wilson_lo)).abs() < 1e-9,
            "the final flip's width is the full read's interval width (same counts)");
    }

    #[test]
    fn mispaired_arms_panic() {
        let a = vec![false; 4];
        let b = vec![false; 5];
        let r = std::panic::catch_unwind(|| certify(&a, &b));
        assert!(r.is_err(), "a length disagreement is a caller bug, never a record");
    }

    #[test]
    fn empty_read_certifies_undetermined() {
        let c = certify(&[], &[]);
        assert_eq!(c.n_pairs, 0);
        assert_eq!(c.verdict, ProbeDeltaVerdict::Undetermined);
        assert_eq!(c.settled_at_n, None);
        assert!(c.flips.is_empty());
    }
}
