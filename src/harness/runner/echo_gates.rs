//! The Issue-064 echo gates — the two mandatory instruments that make a
//! corpus-ab V5 read SAFE to act on (pre-registered in the issue before the
//! density-gated artifact runs; constants are provisional-documented, never
//! run-time knobs).
//!
//! 1. **Abstention-entropy gate** — the calibrated fused gate's thresholds
//!    were fit on the gold corpus's confidence/entropy shape; a synth
//!    corpus that shifts that shape makes the fitted thresholds misfire
//!    downstream. Gate (the issue's own wording): KL(gold ‖ synth) ≤ 0.05
//!    nats over the per-question answer-entropy histogram (10 bins over
//!    [0,1], entropy normalized by ln(#labels), Laplace-smoothed). The
//!    abstain-RATE pair rides as a disclosure column, never a gate leg:
//!    corpus growth raises answer confidence by design, and an absolute
//!    rate band mis-fires at high-abstain baselines (first real reading:
//!    6pp drift at a 0.93 baseline with KL 0.0000 — healthy, see
//!    ABSTENTION_RULE).
//!
//! 2. **OOD corruption rig** — a projected-corpus win that only survives on
//!    the clean (exchangeable) test read is surface echo, not learning: the
//!    synth docs are transplanted spans of train frames, so a win built on
//!    span overlap rides the exact-token channel. Deterministic seeded
//!    word-dropout (p ∈ {0.10, 0.20, 0.30}) breaks that channel while label
//!    semantics partially survive. The gate rung is p = 0.20: on a clean V5
//!    PASS (paired LB95 > 0), a significantly NEGATIVE corrupted delta
//!    (paired LB95 < 0) reads ECHO — recorded NEGATIVE, the lane dies per
//!    the issue's law. A non-negative corrupted delta reads transfer-ok and
//!    the retention ratio (corrupted Δ / clean Δ) is reported.
//!
//!    Stated limits (honest disclosure, not caveats): word-dropout probes
//!    the SURFACE-ECHO channel specifically; it is not a semantic-shift
//!    benchmark. Cross-lingual OOD was rejected as degenerate for a
//!    hashed-bag engine (nothing transfers in either arm — no
//!    discriminative power), and no second same-label-space dataset exists
//!    on disk. Both pulls of massive on this box are byte-identical, so no
//!    replication read exists either — the corruption ladder is the one
//!    buildable discriminator, and its verdict is one axis, not the whole
//!    truth.
//!
//! Both gates run on EVERY corpus-ab pass (modelless reads are µs-class;
//! the gates are mandatory per the issue, not opt-in). The corrupted reads
//! are byte-reproducible: seed = blake3(case_id ‖ ":" ‖ p), splitmix64
//! stream, integer/f64 ops only (the frozen-read law).

use serde::Serialize;

/// Answer-entropy histogram bins over normalized entropy [0,1].
pub(crate) const ENTROPY_BINS: usize = 10;
/// Pre-registered KL ceiling, gold→synth direction (nats). Provisional:
/// revisited only by an issue edit.
pub(crate) const KL_EPS: f64 = 0.05;
/// The word-dropout ladder. The middle rung is the gate rung.
pub(crate) const OOD_RUNGS: [f64; 3] = [0.10, 0.20, 0.30];
/// The pre-registered gate rung (must be one of `OOD_RUNGS`).
pub(crate) const OOD_GATE_RUNG: f64 = 0.20;

/// Deterministic splitmix64 — integer ops only, platform-stable.
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform f64 in [0, 1) — 53 random mantissa bits as a VALUE (never
    /// `from_bits`, which would read the integer as a raw IEEE pattern).
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / 9_007_199_254_740_992.0
    }
}

/// The corruption seed for one (case, rung): blake3 over the case id and
/// the rung spelled at fixed precision — byte-stable across runs/boxes.
pub(crate) fn state_seed(case_id: &str, p: f64) -> u64 {
    let key = format!("{case_id}:p{p:.2}");
    let h = blake3::hash(key.as_bytes());
    let bytes = h.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Word-dropout corruption of one state string: each whitespace token
/// survives with probability 1 − p, consumed in order (one draw per token —
/// the stream is stable). Strings of 0–1 tokens pass through untouched; a
/// draw that would empty the string keeps token 0 (deterministic; the rung
/// still measurably degrades). The kept tokens are a subsequence — order
/// and token text are never altered.
pub(crate) fn corrupt_state(s: &str, p: f64, seed: u64) -> String {
    if p <= 0.0 {
        return s.to_string();
    }
    let toks: Vec<&str> = s.split_whitespace().collect();
    if toks.len() <= 1 {
        return s.to_string();
    }
    let mut rng = SplitMix64::new(seed);
    let kept: Vec<&str> = toks
        .iter()
        .copied()
        .filter(|_| rng.next_f64() >= p)
        .collect();
    if kept.is_empty() {
        toks[0].to_string()
    } else {
        kept.join(" ")
    }
}

/// Normalized answer entropy of one probability vector: H(p̂)/ln(n) ∈ [0,1]
/// (p̂ = p/Σp). Degenerate inputs (empty, non-positive mass, n ≤ 1) read 0
/// (no uncertainty expressible) — documented, deterministic.
fn normalized_entropy(p: &[f64]) -> f64 {
    let n = p.len();
    if n <= 1 {
        return 0.0;
    }
    let sum: f64 = p.iter().sum();
    if !sum.is_finite() || sum <= 0.0 {
        return 0.0;
    }
    let mut h = 0.0f64;
    for &x in p {
        let q = x / sum;
        if q > 0.0 {
            h -= q * q.ln();
        }
    }
    let norm = (n as f64).ln();
    if norm > 0.0 {
        (h / norm).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The smoothed answer-entropy histogram over every question of an eval
/// (10 bins, add-one Laplace, normalized to sum 1) + the abstain rate.
fn entropy_hist_and_rate(eval: &super::Eval) -> (Vec<f64>, f64) {
    let mut counts = [0usize; ENTROPY_BINS];
    let mut n_q = 0usize;
    let mut n_abstain = 0usize;
    for (case_probs, case_abst) in eval.probs.iter().zip(eval.abstained.iter()) {
        for (p, abst) in case_probs.iter().zip(case_abst.iter()) {
            let h = normalized_entropy(p);
            let bin = ((h * ENTROPY_BINS as f64) as usize).min(ENTROPY_BINS - 1);
            counts[bin] += 1;
            n_q += 1;
            if *abst {
                n_abstain += 1;
            }
        }
    }
    let denom = (n_q + ENTROPY_BINS) as f64;
    let hist = counts.iter().map(|&c| (c as f64 + 1.0) / denom).collect();
    let rate = if n_q == 0 { 0.0 } else { n_abstain as f64 / n_q as f64 };
    (hist, rate)
}

/// KL(P ‖ Q) over two normalized histograms (already smoothed — no zeros).
fn kl(p_dist: &[f64], q_dist: &[f64]) -> f64 {
    p_dist
        .iter()
        .zip(q_dist.iter())
        .map(|(&pi, &qi)| if pi > 0.0 { pi * (pi / qi).ln() } else { 0.0 })
        .sum()
}

/// The abstention-entropy gate block (corpus-ab output).
#[derive(Debug, Serialize)]
pub struct AbstentionGateBlock {
    pub abstain_rate_gold: f64,
    pub abstain_rate_synth: f64,
    pub entropy_hist_gold: Vec<f64>,
    pub entropy_hist_synth: Vec<f64>,
    pub entropy_kl_gold_to_synth: f64,
    pub entropy_kl_synth_to_gold: f64,
    pub pass: bool,
    pub rule: &'static str,
}

const ABSTENTION_RULE: &str = "KL(gold‖synth) ≤ 0.05 nats over the 10-bin normalized answer-entropy \
 histogram (Laplace-smoothed) — the issue's own wording gates the KL alone; the abstain-rate \
 drift is a DISCLOSURE column, not a gate leg (first real reading, 2026-10-04: corpus growth \
 raises answer confidence BY DESIGN — a 6pp rate drop at a 0.93 baseline with KL 0.0000 is \
 the system working, not a collapse; an absolute-rate leg would mis-fire at high-abstain \
 baselines and under-bind at low ones)";

/// Compute the abstention-entropy gate from the two clean-read evals.
/// The gate is the KL alone (the issue's rule); the rate pair rides as
/// disclosure.
pub(crate) fn abstention_gate(eval_gold: &super::Eval, eval_synth: &super::Eval) -> AbstentionGateBlock {
    let (hist_a, rate_a) = entropy_hist_and_rate(eval_gold);
    let (hist_b, rate_b) = entropy_hist_and_rate(eval_synth);
    let kl_ab = kl(&hist_a, &hist_b);
    let kl_ba = kl(&hist_b, &hist_a);
    let pass = kl_ab <= KL_EPS;
    AbstentionGateBlock {
        abstain_rate_gold: rate_a,
        abstain_rate_synth: rate_b,
        entropy_hist_gold: hist_a,
        entropy_hist_synth: hist_b,
        entropy_kl_gold_to_synth: kl_ab,
        entropy_kl_synth_to_gold: kl_ba,
        pass,
        rule: ABSTENTION_RULE,
    }
}

/// One rung of the corruption ladder (corrupted-read accuracies + paired
/// statistic).
#[derive(Debug, Serialize)]
pub struct OodRungRow {
    pub dropout_p: f64,
    pub acc_gold: f64,
    pub acc_synth: f64,
    pub delta: f64,
    pub delta_lb95: f64,
}

/// The OOD corruption-rig block (corpus-ab output).
#[derive(Debug, Serialize)]
pub struct OodBlock {
    pub rungs: Vec<OodRungRow>,
    pub gate_rung_p: f64,
    pub clean_delta: f64,
    pub clean_lb95: f64,
    /// Corrupted-gate-rung Δ ÷ clean Δ (None when clean Δ ≤ 0 — no win to
    /// retain).
    pub retention_at_gate: Option<f64>,
    /// "transfer-ok" | "echo-suspect" | "no-clean-win".
    pub echo_verdict: &'static str,
    pub rule: &'static str,
}

const OOD_RULE: &str = "deterministic seeded word-dropout ladder p∈{0.10,0.20,0.30}; gate rung \
 p=0.20; a clean V5 PASS whose gate-rung paired LB95 < 0 reads ECHO (recorded NEGATIVE, the lane \
 dies); a non-negative gate-rung LB95 reads transfer-ok; retention = corrupted Δ / clean Δ";

/// Assemble the OOD block from the ladder rows + the clean-read statistics.
/// Pure — corpus_ab_suite owns the engine reads.
pub(crate) fn ood_block(rungs: Vec<OodRungRow>, clean_delta: f64, clean_lb95: f64) -> OodBlock {
    let gate = rungs
        .iter()
        .find(|r| (r.dropout_p - OOD_GATE_RUNG).abs() < 1e-9)
        .expect("the ladder always carries the gate rung");
    let retention_at_gate = if clean_delta > 0.0 {
        Some(gate.delta / clean_delta)
    } else {
        None
    };
    let echo_verdict = if clean_lb95 <= 0.0 {
        "no-clean-win"
    } else if gate.delta_lb95 < 0.0 {
        "echo-suspect"
    } else {
        "transfer-ok"
    };
    OodBlock {
        rungs,
        gate_rung_p: OOD_GATE_RUNG,
        clean_delta,
        clean_lb95,
        retention_at_gate,
        echo_verdict,
        rule: OOD_RULE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix_stream_is_deterministic_and_diverges() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = SplitMix64::new(43);
        assert_ne!(a.next_u64(), c.next_u64());
    }

    #[test]
    fn corrupt_state_p_zero_and_short_strings_pass_through() {
        assert_eq!(corrupt_state("turn on the lights", 0.0, 7), "turn on the lights");
        assert_eq!(corrupt_state("run", 0.5, 7), "run");
        assert_eq!(corrupt_state("", 0.5, 7), "");
    }

    #[test]
    fn corrupt_state_is_seed_stable_and_order_preserving() {
        let s = "alpha beta gamma delta epsilon zeta eta theta";
        let a = corrupt_state(s, 0.3, state_seed("case-1", 0.3));
        let b = corrupt_state(s, 0.3, state_seed("case-1", 0.3));
        assert_eq!(a, b, "same (case, rung) must be byte-identical");
        let c = corrupt_state(s, 0.3, state_seed("case-2", 0.3));
        // Different seed virtually always differs over 8 tokens at p=0.3.
        assert_ne!(a, c);
        // Kept tokens are a subsequence: same relative order, original text.
        let src: Vec<&str> = s.split_whitespace().collect();
        let out: Vec<&str> = a.split_whitespace().collect();
        let mut it = src.iter();
        for tok in &out {
            assert!(
                it.any(|t| t == tok),
                "output token {tok} must appear in source order"
            );
        }
    }

    #[test]
    fn corrupt_state_empirical_rate_is_near_p() {
        // 2000 tokens, p = 0.2 → kept ≈ 1600 ± 60 (3σ-ish, deterministic).
        let s: String = (0..2000).map(|i| format!("t{i}")).collect::<Vec<_>>().join(" ");
        let out = corrupt_state(&s, 0.2, 99);
        let kept = out.split_whitespace().count();
        assert!((1500..=1700).contains(&kept), "kept {kept} outside tolerance");
    }

    #[test]
    fn corrupt_state_never_empties() {
        for seed in 0..64u64 {
            let out = corrupt_state("one two", 0.99, seed);
            assert!(!out.is_empty(), "seed {seed} emptied the string");
        }
    }

    #[test]
    fn state_seed_is_stable_across_calls_and_rung_sensitive() {
        assert_eq!(state_seed("abc", 0.2), state_seed("abc", 0.2));
        assert_ne!(state_seed("abc", 0.2), state_seed("abc", 0.3));
        assert_ne!(state_seed("abc", 0.2), state_seed("abd", 0.2));
    }

    fn eval_of(hists: &[&[f64]], abstain: &[bool]) -> super::super::Eval {
        // hists: one probability vector per question; abstain parallel.
        super::super::Eval {
            probs: hists.iter().map(|p| vec![p.to_vec()]).collect(),
            picks: hists.iter().map(|_| vec![0]).collect(),
            confs: hists.iter().map(|_| vec![0.5]).collect(),
            abstained: abstain.iter().map(|&a| vec![a]).collect(),
            causes: abstain
                .iter()
                .map(|_| vec![crate::engine::AbstainCause::Answered])
                .collect(),
        }
    }

    #[test]
    fn entropy_extremes_land_in_the_extreme_bins() {
        // One-hot → bin 0; uniform → top bin.
        let one_hot = eval_of(&[&[1.0, 0.0, 0.0, 0.0]], &[false]);
        let uniform = eval_of(&[&[0.25, 0.25, 0.25, 0.25]], &[false]);
        let (h1, r1) = entropy_hist_and_rate(&one_hot);
        let (h2, r2) = entropy_hist_and_rate(&uniform);
        assert_eq!(r1, 0.0);
        assert_eq!(r2, 0.0);
        // The dominant mass sits in bin 0 (one-hot) vs bin 9 (uniform).
        let m1 = h1.iter().enumerate().max_by(|x, y| x.1.total_cmp(y.1)).unwrap().0;
        let m2 = h2.iter().enumerate().max_by(|x, y| x.1.total_cmp(y.1)).unwrap().0;
        assert_eq!(m1, 0);
        assert_eq!(m2, ENTROPY_BINS - 1);
    }

    #[test]
    fn kl_is_zero_on_identical_and_positive_on_shifted() {
        let e = eval_of(&[&[0.9, 0.1]], &[false]);
        let (h, _) = entropy_hist_and_rate(&e);
        assert!(kl(&h, &h).abs() < 1e-12);
        let other = eval_of(&[&[0.5, 0.5]], &[false]);
        let (h2, _) = entropy_hist_and_rate(&other);
        assert!(kl(&h, &h2) > 0.0);
    }

    #[test]
    fn abstention_gate_gates_on_kl_and_discloses_rate() {
        // Identical evals → pass, zero KL.
        let e1 = eval_of(&[&[0.9, 0.1], &[0.6, 0.4]], &[false, true]);
        let e2 = eval_of(&[&[0.9, 0.1], &[0.6, 0.4]], &[false, true]);
        let g = abstention_gate(&e1, &e2);
        assert!(g.pass);
        assert!(g.entropy_kl_gold_to_synth.abs() < 1e-12);
        assert!((g.abstain_rate_synth - 0.5).abs() < 1e-12);

        // A rate drift ALONE (identical answer shapes, different abstain
        // flags) PASSES — the rate is disclosure, never a gate leg (the
        // 2026-10-04 adjudication: corpus growth raises confidence by
        // design).
        let e3 = eval_of(&[&[0.9, 0.1], &[0.6, 0.4]], &[false, false]);
        let g_rate = abstention_gate(&e1, &e3);
        assert!(g_rate.pass);
        assert!((g_rate.abstain_rate_synth - 0.0).abs() < 1e-12);

        // A real SHAPE collapse (all-uniform gold vs all-one-hot synth,
        // 40 questions each) FAILS on the KL leg.
        let mut shape_a = Vec::new();
        let mut shape_b = Vec::new();
        for _ in 0..40 {
            shape_a.push(&[0.5_f64, 0.5_f64][..]);
            shape_b.push(&[1.0_f64, 0.0_f64][..]);
        }
        let uniform_gold = eval_of(&shape_a, &[false; 40]);
        let one_hot_synth = eval_of(&shape_b, &[false; 40]);
        let g2 = abstention_gate(&uniform_gold, &one_hot_synth);
        assert!(!g2.pass);
        assert!(g2.entropy_kl_gold_to_synth > KL_EPS);
    }

    #[test]
    fn ood_block_verdicts_follow_the_pre_registered_law() {
        let rungs = |d20: f64, lb20: f64| {
            vec![
                OodRungRow { dropout_p: 0.10, acc_gold: 0.8, acc_synth: 0.83, delta: 0.03, delta_lb95: 0.01 },
                OodRungRow { dropout_p: 0.20, acc_gold: 0.7, acc_synth: 0.7 + d20, delta: d20, delta_lb95: lb20 },
                OodRungRow { dropout_p: 0.30, acc_gold: 0.6, acc_synth: 0.61, delta: 0.01, delta_lb95: -0.005 },
            ]
        };
        // Clean win + gate rung survives → transfer-ok, retention reported.
        let b = ood_block(rungs(0.02, 0.005), 0.04, 0.011);
        assert_eq!(b.echo_verdict, "transfer-ok");
        assert!((b.retention_at_gate.unwrap() - 0.5).abs() < 1e-12);
        // Clean win + gate rung significantly negative → echo.
        let b2 = ood_block(rungs(-0.03, -0.012), 0.04, 0.011);
        assert_eq!(b2.echo_verdict, "echo-suspect");
        // No clean win → informational.
        let b3 = ood_block(rungs(0.02, 0.005), 0.00, -0.01);
        assert_eq!(b3.echo_verdict, "no-clean-win");
        assert!(b3.retention_at_gate.is_none());
    }
}
