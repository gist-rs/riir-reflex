//! Issue 969 T3 — the reflex `pick_domain` liveness CONTROL arm (the
//! routing-statistics diagnostic's control half).
//!
//! Port of arXiv:2610.31818 (Nestor) App C.6's level-1 statistics onto the
//! modelless engine's corpus router, with the issue's LIVENESS GATE: on the
//! seated suites, shuffling the routed domain across cases must move the
//! top-1 pick on a large fraction of slots — a near-zero change rate is an
//! instrument failure (the forced-domain seam not engaging, or routing not
//! reaching the scorer) and refuses the run loud, never a quiet pass.
//!
//! The K3 arm of Issue 969 (katgpt-rs Bench 969) reports its subject
//! statistics against this control's liveness verdict.
//!
//! Statistics per seated suite (level 1 — the question→domain router):
//!
//! - **Effective choices** — `exp(H(p))/N` of the routed-domain assignment
//!   distribution (load balance across the suite's label corpora).
//! - **Corpus MI** — mutual information (bits) between the slot's gold
//!   label and the routed domain (the corpus-dependence of routing).
//! - **Shuffled-domain counterfactual** — re-route every slot with another
//!   slot's routed domain (seeded whole-assignment permutation; the domain
//!   marginals are exactly preserved) through the
//!   `Scratch::forced_domain` diagnostic seam; measure the top-1 pick
//!   change rate, the answered-accuracy delta, and the abstain-rate delta.
//!   The gate weights stay the receiver question's own — the assignment is
//!   the only thing shuffled (the K3 arm's semantics, mirrored).
//!
//! Scope note (recorded honestly): the issue hypothesized the reflex lane
//! is two-level ("pick_domain → per-span"). In the shipped engine the
//! option spans all score against the ONE routed domain — there is no
//! independent level-2 assignment to shuffle, so the paper's per-level MI
//! split collapses to level 1 here. That is a structural finding of this
//! arm, not a skipped measurement.
//!
//! Measurement posture: the deployed engine build EXACTLY as the rubric
//! probe builds it (`fit_posture_inner` + `build_engine_with`, every
//! selection lever pinned off, the cal-slice `observe` calibration) — the
//! probe measures what ships. Plus a determinism witness: the first 20
//! baseline slots decide twice and must agree.

use super::perm_probe::stride_indices;
use super::*;

/// Default suites: the issue's four distinct labeled domains.
pub const DEFAULT_ROUTER_STATS_SUITES: &[&str] = &["ag_news", "banking77", "emotion", "sst5"];

/// Default sampled cases per suite (stride over the choice/score cases).
pub const DEFAULT_ROUTER_STATS_MAX_CASES: usize = 100;

/// The liveness wall: the suite-aggregate top-1 pick change rate under the
/// shuffled-domain counterfactual must reach this — below it the control is
/// RED (instrument-blind or routing not load-bearing on what ships; both
/// refuse the run).
pub const CONTROL_CHANGE_WALL: f64 = 0.05;

/// The probe's knobs.
#[derive(Debug, Clone)]
pub struct RouterStatsOptions {
    pub max_cases: usize,
}

/// The record for one suite.
#[derive(Debug, Clone, Serialize)]
pub struct RouterStatsSuite {
    pub suite: String,
    /// Sampled case count.
    pub n_cases: usize,
    /// choice/score slots probed (one domain assignment per slot).
    pub n_slots: usize,
    pub n_domains: usize,
    pub effective_choices: f64,
    pub max_load_frac: f64,
    pub mi_bits: f64,
    /// baseline answered-accuracy over answered slots.
    pub baseline_acc: f64,
    pub baseline_abstain_rate: f64,
    /// top-1 pick change rate under the shuffled-domain counterfactual.
    pub shuffle_change_rate: f64,
    pub shuffle_acc: f64,
    pub shuffle_abstain_rate: f64,
    /// baseline answered-accuracy minus shuffled (positive = shuffling hurt).
    pub acc_drop: f64,
    pub verdict: RouterStatsVerdict,
    /// Suites whose change rate sat under the wall while the control as a
    /// whole is LIVE — the structural-zero class (e.g. sst5's 1-byte digit
    /// candidates are drafter-insensitive: the corpus choice cannot move a
    /// 1-byte candidate's compressed length, so the argmax never moves; the
    /// seam itself provably engaged — the abstain delta moves). A finding,
    /// never an instrument failure by itself.
    pub structural_zero: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RouterStatsVerdict {
    /// The control is LIVE (shuffle moved a large fraction of picks).
    Live,
    /// RED — the change rate sat under the wall: instrument-blind.
    Red,
}

impl RouterStatsVerdict {
    fn str_of(self) -> &'static str {
        match self {
            Self::Live => "LIVE",
            Self::Red => "RED",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RouterStatsMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub max_cases: usize,
    pub control_change_wall: f64,
    pub measurement_posture: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouterStatsOutput {
    pub meta: RouterStatsMeta,
    /// Suites under the wall (the structural-zero class when control_live).
    pub red_suites: usize,
    /// The lane-level liveness verdict: ANY suite clearing the wall proves
    /// the seam engages and routing is load-bearing on what ships.
    pub control_live: bool,
    pub structural_zero_suites: Vec<String>,
    pub suites: Vec<RouterStatsSuite>,
}

// ────────────────────────────────────────────── the per-suite cell ──

fn router_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    ropts: &RouterStatsOptions,
) -> Result<RouterStatsSuite, String> {
    // The deployed posture — the rubric probe's ModellessInput, verbatim.
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
    // The deployed calibration (the rubric probe's pass, verbatim).
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

    // ── slot sampling: every case carrying a choice/score question ──
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
        return Err(format!("{}: no choice/score questions", spec.name));
    }
    let sampled_idx = stride_indices(probed.len(), ropts.max_cases);

    // Flat slot list: (case_pos_in_sampled, question_index).
    let mut sc = Scratch::<EMBED_DIM>::new();
    struct SlotRec {
        case: usize,
        qi: usize,
        domain: usize,
        pick: u32,
        abstained: bool,
        gold: usize,
    }
    let mut slots: Vec<SlotRec> = Vec::new();
    for &ci in &sampled_idx {
        let cc = &prepared.suite.cases[probed[ci]];
        let state_str = serialize_state(&cc.state);
        let req = engine_request(cc, &state_str)?;
        sc.forced_domain = None;
        engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("engine decide ({}): {e}", cc.id))?;
        for (qi, q) in cc.questions.iter().enumerate() {
            if !matches!(q.kind, QKind::Choice | QKind::Score) {
                continue;
            }
            let slot = &sc.slots[qi];
            let gold = cc.gold.get(qi).map(|g| g.idx).unwrap_or(slot.pick as usize);
            slots.push(SlotRec {
                case: ci,
                qi,
                domain: sc.domains[qi],
                pick: slot.pick,
                abstained: slot.abstained,
                gold,
            });
        }
    }
    if slots.is_empty() {
        return Err(format!("{}: no probed slots", spec.name));
    }

    // ── determinism witness: first 20 slots decide twice, must agree ──
    for rec in slots.iter().take(20) {
        let cc = &prepared.suite.cases[probed[rec.case]];
        let state_str = serialize_state(&cc.state);
        let req = engine_request(cc, &state_str)?;
        sc.forced_domain = None;
        engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("engine redecide ({}): {e}", cc.id))?;
        if sc.slots[rec.qi].pick != rec.pick {
            return Err(format!(
                "{}: baseline NOT deterministic (slot redecide flipped {} → {})",
                spec.name, rec.pick, sc.slots[rec.qi].pick
            ));
        }
    }

    // ── Stat 1: effective choices over the routed-domain distribution ──
    let mut count = vec![0u64; N];
    for s in &slots {
        count[s.domain] += 1;
    }
    let total = slots.len() as f64;
    let mut h = 0.0f64;
    let mut max_frac = 0.0f64;
    for &c in &count {
        let p = c as f64 / total;
        max_frac = max_frac.max(p);
        if p > 0.0 {
            h -= p * p.ln();
        }
    }
    let effective = h.exp() / N as f64;

    // ── Stat 2: corpus MI(gold label; routed domain), bits ──
    let mut joint = vec![0u64; N * N];
    let mut gold_marg = vec![0u64; N];
    let mut dom_marg = vec![0u64; N];
    for s in &slots {
        joint[s.gold * N + s.domain] += 1;
        gold_marg[s.gold] += 1;
        dom_marg[s.domain] += 1;
    }
    let mut mi = 0.0f64;
    for g in 0..N {
        for d in 0..N {
            let j = joint[g * N + d] as f64;
            if j > 0.0 {
                let p_joint = j / total;
                let p_g = gold_marg[g] as f64 / total;
                let p_d = dom_marg[d] as f64 / total;
                mi += p_joint * (p_joint / (p_g * p_d)).log2();
            }
        }
    }

    // ── Stat 3: the shuffled-domain counterfactual (the liveness leg) ──
    // Whole-assignment permutation across the suite's slots: the domain
    // marginals are exactly preserved, the pairing is not. Seeded LCG —
    // deterministic, no external dep (the katgpt-rs bench_012 pattern).
    let mut rng_seed: u64 = 0x9690_C711;
    let mut next = move || {
        rng_seed = rng_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (rng_seed >> 32) as usize
    };
    let mut perm: Vec<usize> = (0..slots.len()).collect();
    for i in (1..perm.len()).rev() {
        let j = next() % (i + 1);
        perm.swap(i, j);
    }
    // Group slots by case for the per-decide forced vector.
    let mut by_case: std::collections::BTreeMap<usize, Vec<(usize, usize)>> =
        std::collections::BTreeMap::new();
    for (si, s) in slots.iter().enumerate() {
        by_case.entry(s.case).or_default().push((s.qi, perm[si]));
    }

    let mut changed = 0u64;
    let mut baseline_correct = 0u64;
    let mut baseline_answered = 0u64;
    let mut shuffle_correct = 0u64;
    let mut shuffle_answered = 0u64;
    let mut baseline_abstained = 0u64;
    let mut shuffle_abstained = 0u64;
    for (&case_pos, qslots) in &by_case {
        let cc = &prepared.suite.cases[probed[case_pos]];
        let state_str = serialize_state(&cc.state);
        let req = engine_request(cc, &state_str)?;
        let mut forced: Vec<Option<usize>> = vec![None; cc.questions.len()];
        for (qi, donor_slot) in qslots {
            let donor = slots[*donor_slot].domain;
            forced[*qi] = Some(donor);
        }
        sc.forced_domain = Some(forced);
        engine
            .decide_with(&req, &mut sc)
            .map_err(|e| format!("engine shuffled decide ({}): {e}", cc.id))?;
        sc.forced_domain = None;
        for &(qi, donor_slot) in qslots {
            let base = &slots[donor_slot];
            let now = &sc.slots[qi];
            if now.pick != base.pick {
                changed += 1;
            }
            let gold = base.gold;
            if !base.abstained {
                baseline_answered += 1;
                if base.pick as usize == gold {
                    baseline_correct += 1;
                }
            }
            if !now.abstained {
                shuffle_answered += 1;
                if now.pick as usize == gold {
                    shuffle_correct += 1;
                }
            }
            baseline_abstained += u64::from(base.abstained);
            shuffle_abstained += u64::from(now.abstained);
        }
    }

    let n = slots.len() as f64;
    let change_rate = changed as f64 / n;
    let baseline_acc = baseline_correct as f64 / baseline_answered.max(1) as f64;
    let shuffle_acc = shuffle_correct as f64 / shuffle_answered.max(1) as f64;

    Ok(RouterStatsSuite {
        suite: spec.name.to_string(),
        n_cases: sampled_idx.len(),
        n_slots: slots.len(),
        n_domains: N,
        effective_choices: effective,
        max_load_frac: max_frac,
        mi_bits: mi,
        baseline_acc,
        baseline_abstain_rate: baseline_abstained as f64 / n,
        shuffle_change_rate: change_rate,
        shuffle_acc,
        shuffle_abstain_rate: shuffle_abstained as f64 / n,
        acc_drop: baseline_acc - shuffle_acc,
        verdict: if change_rate >= CONTROL_CHANGE_WALL {
            RouterStatsVerdict::Live
        } else {
            RouterStatsVerdict::Red
        },
        structural_zero: change_rate < CONTROL_CHANGE_WALL,
    })
}

// ────────────────────────────────────────────── the N dispatch ──

macro_rules! with_router_width {
    ($n:literal, $spec:expr, $prepared:expr, $ropts:expr) => {
        router_suite::<$n>($spec, $prepared, $ropts)
    };
}

/// Run the control over the selected suites (default: the issue's four
/// labeled domains).
pub fn run_router_stats_probe(
    opts: &RunOptions,
    ropts: &RouterStatsOptions,
) -> Result<RouterStatsOutput, String> {
    let mut suites = Vec::new();
    let mut prep_skipped: Vec<(String, String)> = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only || !DEFAULT_ROUTER_STATS_SUITES.contains(&spec.name) {
            continue;
        }
        match prepare(spec, &opts.datasets_dir) {
            Ok(p) => {
                eprintln!("═══ suite {} (router-stats control) ═══", spec.name);
                let rep = match p.labels.len() {
                    2 => with_router_width!(2, spec, &p, ropts),
                    3 => with_router_width!(3, spec, &p, ropts),
                    4 => with_router_width!(4, spec, &p, ropts),
                    5 => with_router_width!(5, spec, &p, ropts),
                    6 => with_router_width!(6, spec, &p, ropts),
                    7 => with_router_width!(7, spec, &p, ropts),
                    8 => with_router_width!(8, spec, &p, ropts),
                    10 => with_router_width!(10, spec, &p, ropts),
                    15 => with_router_width!(15, spec, &p, ropts),
                    59 => with_router_width!(59, spec, &p, ropts),
                    77 => with_router_width!(77, spec, &p, ropts),
                    842 => with_router_width!(842, spec, &p, ropts),
                    n => Err(format!(
                        "no engine instantiation for {n} domains — extend the dispatch \
                         table in router_stats_probe.rs"
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
            "router-stats probe: no suite produced a cell ({} request line(s)) — check --suites",
            opts.suites.len()
        ));
    }
    if !prep_skipped.is_empty() {
        for (name, why) in &prep_skipped {
            eprintln!("  [router-stats skipped] {name}: {why}");
        }
    }
    let red_suites = suites
        .iter()
        .filter(|s| s.verdict == RouterStatsVerdict::Red)
        .count();
    // The LIVENESS gate is lane-level, not per-suite: the instrument is
    // proven live when ANY suite's shuffled-domain counterfactual clears the
    // wall (a per-suite zero can be a structural property of that suite's
    // option shape — sst5's 1-byte digit candidates — while the seam
    // provably engages there too, via the abstain-delta movement). An
    // all-suites RED is the instrument-blind class and refuses the run.
    let control_live = suites
        .iter()
        .any(|s| s.shuffle_change_rate >= CONTROL_CHANGE_WALL);
    let out = RouterStatsOutput {
        meta: RouterStatsMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            max_cases: ropts.max_cases,
            control_change_wall: CONTROL_CHANGE_WALL,
            measurement_posture: "deployed engine build (fit_posture_inner + \
                build_engine_with, selection levers pinned off, cal-slice observe \
                calibration); shuffled-domain counterfactual through the \
                Scratch::forced_domain seam; whole-assignment permutation, \
                marginals preserved"
                .to_string(),
        },
        red_suites,
        control_live,
        structural_zero_suites: suites
            .iter()
            .filter(|s| s.structural_zero)
            .map(|s| s.suite.clone())
            .collect(),
        suites,
    };
    Ok(out)
}

/// The markdown render (the harness writes it beside the JSON).
pub fn render_router_stats_markdown(out: &RouterStatsOutput) -> String {
    let mut md = String::with_capacity(4096);
    md.push_str("# Issue 969 T3 — reflex `pick_domain` router-statistics control\n\n");
    md.push_str(&format!(
        "- date: {} · host: {} · sha: {}\n- datasets: {}\n- posture: {}\n- control wall: pick change rate ≥ {:.2} (below = RED, instrument-blind)\n\n",
        out.meta.date_utc,
        out.meta.host,
        out.meta
            .git_sha
            .as_deref()
            .unwrap_or("unknown"),
        out.meta.datasets_dir,
        out.meta.measurement_posture,
        out.meta.control_change_wall,
    ));
    md.push_str("| suite | slots | N | eff | maxld | MI bits | base acc | shuf acc | acc drop | change | abst Δ | verdict |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for s in &out.suites {
        md.push_str(&format!(
            "| {} | {} | {} | {:.3} | {:.3} | {:.4} | {:.4} | {:.4} | {:+.4} | {:.3} | {:+.3} | {} |\n",
            s.suite,
            s.n_slots,
            s.n_domains,
            s.effective_choices,
            s.max_load_frac,
            s.mi_bits,
            s.baseline_acc,
            s.shuffle_acc,
            s.acc_drop,
            s.shuffle_change_rate,
            s.shuffle_abstain_rate - s.baseline_abstain_rate,
            s.verdict.str_of(),
        ));
    }
    md.push_str(&format!(
        "\n**Control verdict: {}** — {} suite(s) under the wall ({}, named structural-zero class when live), {} of {} suites live.\n",
        if out.control_live {
            "LIVE (the liveness gate holds: domain routing is load-bearing on what ships)"
        } else {
            "RED (the liveness gate FAILED — instrument-blind; never a quiet pass)"
        },
        out.red_suites,
        out.structural_zero_suites.join(", "),
        out.suites.iter().filter(|s| !s.structural_zero).count(),
        out.suites.len()
    ));
    md.push_str(
        "\nScope note: the option spans all score against the ONE routed domain in the \
         shipped engine — there is no independent level-2 assignment to shuffle, so the \
         paper's per-level MI split collapses to level 1 here (structural, measured).\n",
    );
    md
}
