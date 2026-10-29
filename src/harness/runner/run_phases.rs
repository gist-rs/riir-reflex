//! The run-loop phase fns (issue 098 T3b): validation → per-suite
//! orchestration → the meta block. Carved out of `runner.rs` along the
//! seams the T3a in-place split created; every item reaches the runner's
//! substrate through `use super::*` (a child sees the parent's private
//! items), and harness-level paths follow the T1 carve convention
//! (`crate::harness::`, never `super::super`).

use super::*;

/// The run's input validation (issue 098 T3 phase fn): the
/// mutual-exclusion and feature-gate refusals, then the unknown-suite
/// refusal — the same check order as before the split, so the first
/// failing option still names itself.
fn validate_run_options(opts: &RunOptions) -> Result<(), String> {
    if opts.corpus_cap_override != 0 && !opts.cal_select_caps.is_empty() {
        return Err(
            "--corpus-cap and --cal-select-cap are mutually exclusive: one pins the cap, \
             the other selects it on the cal slice"
                .to_string(),
        );
    }
    if opts.head_scale > 0.0 && opts.head_select {
        return Err(
            "--head-scale and --head-select are mutually exclusive: one pins the head \
             scale, the other selects it on the cal slice"
                .to_string(),
        );
    }
    if opts.nb_select && !cfg!(feature = "nb_scope") {
        return Err(
            "--nb-select needs the nb_scope feature (issue 038): rebuild with \
             --features nb_scope"
                .to_string(),
        );
    }
    if opts.oc_select && !cfg!(feature = "option_cond") {
        return Err(
            "--oc-select needs the option_cond feature (issue 038 T7b): rebuild with \
             --features option_cond"
                .to_string(),
        );
    }
    if opts.density_gate && !cfg!(feature = "density_gate") {
        return Err(
            "--density-gate needs the density_gate feature (issue 066): rebuild with \
             --features density_gate"
                .to_string(),
        );
    }
    if opts.ridge_select && !cfg!(feature = "nb_ridge") {
        return Err(
            "--ridge-select needs the nb_ridge feature (issue 038 T7a): rebuild with \
             --features nb_ridge"
                .to_string(),
        );
    }
    if opts.genome_select && !cfg!(feature = "nb_scope") {
        return Err(
            "--genome-select needs the nb_scope feature (issue 038 T5): rebuild with \
             --features nb_scope"
                .to_string(),
        );
    }
    if opts.cascade && opts.skip_laya {
        return Err(
            "--cascade and --skip-laya are mutually exclusive: the cascade escalates \
             to the laya lane — there is nothing to escalate to without it (issue 038 T4′)"
                .to_string(),
        );
    }
    if opts.cascade && !cfg!(feature = "laya-riir") {
        return Err(
            "--cascade needs the laya-riir feature (issue 038 T4′): rebuild with \
             --features laya-riir"
                .to_string(),
        );
    }
    if opts.cascade_worthiness && !opts.cascade {
        return Err(
            "--cascade-worthiness needs --cascade: it gates the cascade lane's \
             escalation per suite (issue 042 lever 3)"
                .to_string(),
        );
    }
    // A requested name matching NO registered spec selects nothing and the
    // run would write a 0-suite results.json over the out-dir's previous
    // contents (measured twice on the default tables: 09-24's 2-suite
    // partial, 10-25's 0-suite probe — both restored from HEAD). Refuse
    // BEFORE any lane machinery or file write.
    for requested in &opts.suites {
        if !SUITES.iter().any(|spec| spec.name == *requested) {
            let known: Vec<&str> = SUITES.iter().map(|s| s.name).collect();
            return Err(format!(
                "unknown suite {requested:?} — registered suites: {}",
                known.join(", ")
            ));
        }
    }
    Ok(())
}

/// Issue 055 / Plan 008 T5: the `--mc-ab` arm's tuning. A pinned knob is a
/// single-row grid (the selection table then carries the pinned row only —
/// the test read stays one). Pure construction from the options — built per
/// modelless-armed suite since the T3 phase split (was once per run; the
/// values are identical and the cost is three knob reads + two tiny grids).
#[cfg(feature = "mc_ensemble")]
fn build_mc_cfg(opts: &RunOptions) -> crate::harness::mc_ab::McAbConfig {
    crate::harness::mc_ab::McAbConfig {
        n_samples: if opts.mc_samples == 0 {
            crate::harness::mc_ab::DEFAULT_MC_SAMPLES
        } else {
            opts.mc_samples
        },
        p_grid: opts.mc_p_drop.map_or_else(
            || crate::harness::mc_ab::DEFAULT_P_GRID.to_vec(),
            |p| vec![p],
        ),
        lambda_grid: opts.mc_lambda.map_or_else(
            || crate::harness::mc_ab::DEFAULT_LAMBDA_GRID.to_vec(),
            |l| vec![l],
        ),
    }
}

/// The modelless lane's per-suite outcome (issue 098 T3). `Failed` means
/// the error is already recorded and the SUITE is aborted — the run loop's
/// `continue` before the split yielded no suite row at all on a modelless
/// failure, and that contract is preserved by construction.
enum ModellessOutcome {
    /// The suite has no modelless lane (the declared-inability skip — dead
    /// over the current registry since Issue 045, kept for any future
    /// `modelless_lane: false` suite).
    NotApplicable,
    Ran {
        result: Box<LaneResult>,
        questions: crate::harness::cascade::ModellessQuestions,
    },
    Failed,
}

/// One suite's laya-lane records (issue 098 T3 phase struct): `results`
/// keyed by checkpoint (`py/` prefix = the python oracle), `questions` the
/// cascade lane's escalation half, `cal_questions` the worthiness probe's
/// escalator half (Issue 042 lever 3, only served when the lever is armed).
struct LayaLanes {
    results: BTreeMap<String, LaneResult>,
    questions: BTreeMap<String, crate::harness::cascade::LayaQuestions>,
    cal_questions: BTreeMap<String, crate::harness::cascade::LayaQuestions>,
}

/// One suite's comparison-lane rows (issue 098 T3 phase struct) — every
/// field `None` = the lane did not run (flag off / server unreachable), an
/// honest absence recorded in the errors list, never a fabricated row.
struct ComparisonLanes {
    clm: Option<LaneResult>,
    gliner: Option<LaneResult>,
    bekko: Option<LaneResult>,
    agentjev: Option<LaneResult>,
    clef: Option<LaneResult>,
    openthai: Option<LaneResult>,
    drex: Option<LaneResult>,
    d1: Option<LaneResult>,
    pplx: Option<LaneResult>,
    d3: Option<LaneResult>,
    jeva: Option<LaneResult>,
    startlux: Option<LaneResult>,
    mammoth: Option<LaneResult>,
    elephant: Option<LaneResult>,
    opendecider: Option<LaneResult>,
    paw: Option<crate::lanes::paw::PawLaneResult>,
    paw_local: Option<crate::lanes::paw::PawLaneResult>,
}

/// One standard-shape comparison lane (issue 098 T3): the `running…`
/// banner, the shared 4-metric success line, and the loud-absence error
/// push. The lanes with nonstandard readouts (clm's token count, paw's
/// refusal accounting, the laya checkpoints) keep their bespoke blocks.
fn run_comparison_lane(
    label: &str,
    suite_name: &str,
    errors: &mut Vec<String>,
    run: impl FnOnce() -> Result<LaneResult, String>,
) -> Option<LaneResult> {
    eprintln!("    {label}: running…");
    match run() {
        Ok(r) => {
            eprintln!(
                "    {label}: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                r.hard.accuracy,
                r.hard.ece,
                r.latency_p50_ms,
                (r.seconds * 10.0).round() / 10.0
            );
            Some(r)
        }
        Err(e) => {
            errors.push(format!("{suite_name} ({label}): {e}"));
            None
        }
    }
}

/// Run the harness. Suite-level failures (missing datasets, engine build
/// errors) are REPORTED in the returned output as errors — never silently
/// dropped — and the run continues with the remaining suites.
pub fn run(opts: &RunOptions) -> Result<(RunOutput, Vec<String>), String> {
    let box_start = crate::harness::box_state::capture();
    validate_run_options(opts)?;

    let mut results: Vec<SuiteResult> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
            continue;
        }
        if opts.suites.is_empty() && spec.named_only {
            continue; // named-only suites never tax a default run
        }
        if let Some(result) = run_suite(spec, opts, &mut errors) {
            results.push(result);
        }
    }

    let meta = build_meta(opts, box_start);
    Ok((
        RunOutput {
            meta,
            suites: results,
        },
        errors,
    ))
}

/// The run loop's per-suite body (issue 098 T3 phase fn). `None` = the
/// suite was skipped (prepare failure / no engine instantiation / modelless
/// failure) — the reason is already recorded in `errors`.
fn run_suite(spec: &SuiteSpec, opts: &RunOptions, errors: &mut Vec<String>) -> Option<SuiteResult> {
    eprintln!("═══ suite {} ═══", spec.name);
    let prepared = match prepare(spec, &opts.datasets_dir) {
        Ok(p) => p,
        Err(e) => {
            errors.push(format!("{}: {e}", spec.name));
            return None;
        }
    };
    let n_questions: usize = prepared.suite.cases.iter().map(|c| c.questions.len()).sum();
    eprintln!(
        "    {} cases / {} questions · {} domains",
        prepared.suite.cases.len(),
        n_questions,
        prepared.labels.len()
    );
    // Issue 058: the slice-integrity line rides the run log AND the
    // tables — a silently moved pool (the Bench-076 class) is visible
    // at run time as a digest/count change, not as an accuracy move.
    if let Some(s) = &prepared.slices {
        eprintln!("    {}", s.summary_line());
    }

    let (leak_block, leak_flags) = scan_slice_leaks(spec, &prepared, errors);
    let leak_flags_ref: Option<&[bool]> = leak_flags.as_deref();

    // Modelless lane (const-generic dispatch over the domain count) — the
    // phase fn returns the outcome; `Failed` aborts the whole suite (the
    // run loop's `continue` before the split yielded no suite row at all).
    let (modelless, modelless_questions) =
        match run_modelless_suite(spec, &prepared, opts, leak_flags_ref, errors) {
            ModellessOutcome::NotApplicable => (None, None),
            ModellessOutcome::Ran { result, questions } => (Some(result), Some(questions)),
            ModellessOutcome::Failed => return None,
        };

    let laya_lanes = run_laya_lanes(spec, &prepared, opts, leak_flags_ref, errors);

    let comparison = run_comparison_lanes(spec, &prepared, opts, leak_flags_ref, errors);

    let cascade = compose_cascade(
        spec,
        &prepared,
        opts,
        modelless_questions.as_ref(),
        &laya_lanes,
        errors,
    );

    let inductive_bias = run_inductive_bias_probes(
        spec,
        &prepared,
        opts,
        modelless.as_deref(),
        &laya_lanes,
        &comparison,
        errors,
    );

    Some(SuiteResult {
        name: spec.name.to_string(),
        n_cases: prepared.suite.cases.len(),
        n_questions,
        cases_digest: cases_digest(spec.name, &prepared.suite.cases),
        modal_floor: crate::harness::modal_floor::census(
            &prepared.suite.cases,
            Some(prepared.cal_cases.as_slice()),
        ),
        modelless: modelless.map(|boxed| *boxed),
        laya: laya_lanes.results,
        cascade,
        clm: comparison.clm,
        gliner: comparison.gliner,
        bekko: comparison.bekko,
        agentjev: comparison.agentjev,
        clef: comparison.clef,
        openthai: comparison.openthai,
        drex: comparison.drex,
        d1: comparison.d1,
        pplx: comparison.pplx,
        d3: comparison.d3,
        jeva: comparison.jeva,
        startlux: comparison.startlux,
        mammoth: comparison.mammoth,
        elephant: comparison.elephant,
        opendecider: comparison.opendecider,
        paw: comparison.paw,
        paw_local: comparison.paw_local,
        leak: leak_block,
        slices: prepared.slices,
        inductive_bias,
    })
}

impl ComparisonLanes {
    /// The standard-shape rows by display name (paw's bespoke row type is
    /// not a `LaneResult` and is excluded — Issue 100's pairing reads
    /// `LaneResult.items`).
    fn standard_rows(&self) -> [(&'static str, Option<&LaneResult>); 15] {
        [
            ("clm", self.clm.as_ref()),
            ("gliner", self.gliner.as_ref()),
            ("bekko", self.bekko.as_ref()),
            ("agentjev", self.agentjev.as_ref()),
            ("clef", self.clef.as_ref()),
            ("openthai", self.openthai.as_ref()),
            ("drex", self.drex.as_ref()),
            ("d1", self.d1.as_ref()),
            ("pplx", self.pplx.as_ref()),
            ("d3", self.d3.as_ref()),
            ("jeva", self.jeva.as_ref()),
            ("startlux", self.startlux.as_ref()),
            ("mammoth", self.mammoth.as_ref()),
            ("elephant", self.elephant.as_ref()),
            ("opendecider", self.opendecider.as_ref()),
        ]
    }
}

/// Issue 100 (phase fn): the report-only inductive-bias probes. Re-reads
/// the SAME armed lanes on probe copies of the prepared suite — T1 on the
/// word-dropout ladder, T2 (modelless only) on the label-smoothed corpus.
/// `None` when neither flag is armed or the suite has no modelless row (the
/// floor every delta is paired against).
fn run_inductive_bias_probes(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &RunOptions,
    modelless: Option<&LaneResult>,
    laya_lanes: &LayaLanes,
    comparison: &ComparisonLanes,
    errors: &mut Vec<String>,
) -> Option<robust_ab::InductiveBiasBlock> {
    if !opts.robust_ab && !opts.label_smooth_ab {
        return None;
    }
    let Some(ml_clean) = modelless else {
        errors.push(format!(
            "{} (inductive-bias): no modelless row — the probes pair every lane against it",
            spec.name
        ));
        return None;
    };
    let clean_enc = robust_ab::encoder_rows(&laya_lanes.results, comparison.standard_rows());

    let robust = opts.robust_ab.then(|| {
        let mut rungs = Vec::with_capacity(echo_gates::OOD_RUNGS.len());
        for &p in echo_gates::OOD_RUNGS.iter() {
            eprintln!("    robust-ab: dropout p={p:.2} — re-reading every armed lane…");
            let cp = prepared.with_eval_suite(robust_ab::corrupt_suite(&prepared.suite, p));
            let ml_corr = match run_modelless_suite(spec, &cp, opts, None, errors) {
                ModellessOutcome::Ran { result, .. } => result,
                _ => {
                    errors.push(format!(
                        "{} (robust-ab p={p:.2}): modelless re-read failed",
                        spec.name
                    ));
                    continue;
                }
            };
            let laya_c = run_laya_lanes(spec, &cp, opts, None, errors);
            let comp_c = run_comparison_lanes(spec, &cp, opts, None, errors);
            let corr_enc = robust_ab::encoder_rows(&laya_c.results, comp_c.standard_rows());
            let mut lanes = vec![robust_ab::lane_degradation(
                "modelless",
                &ml_clean.items,
                &ml_corr.items,
            )];
            let mut cross = Vec::new();
            for (name, clean) in &clean_enc {
                let Some((_, corr)) = corr_enc.iter().find(|(n, _)| n == name) else {
                    continue;
                };
                lanes.push(robust_ab::lane_degradation(name, &clean.items, &corr.items));
                cross.push(robust_ab::cross_lane_delta(
                    name,
                    &ml_clean.items,
                    &ml_corr.items,
                    &clean.items,
                    &corr.items,
                ));
            }
            for l in &lanes {
                eprintln!(
                    "    robust-ab p={p:.2} {}: {:.4} → {:.4} (Δ {:+.4} [{:+.4}, {:+.4}], n={})",
                    l.lane, l.acc_clean, l.acc_corrupted, l.delta, l.delta_lb95, l.delta_ub95, l.n
                );
            }
            for c in &cross {
                eprintln!(
                    "    robust-ab p={p:.2} {} − modelless: ΔΔ {:+.4} [{:+.4}, {:+.4}] (n={})",
                    c.encoder, c.delta_of_deltas, c.dod_lb95, c.dod_ub95, c.n_paired
                );
            }
            rungs.push(robust_ab::RobustRung {
                dropout_p: p,
                lanes,
                cross,
            });
        }
        rungs
    });

    let label_smooth = if opts.label_smooth_ab {
        match ml_clean.corpus_cap.as_ref().map(|c| c.effective) {
            None => {
                errors.push(format!(
                    "{} (label-smooth-ab): the modelless row carries no effective corpus cap",
                    spec.name
                ));
                None
            }
            Some(cap) => {
                let mut rungs = Vec::with_capacity(robust_ab::SMOOTH_ALPHAS.len());
                for &alpha in robust_ab::SMOOTH_ALPHAS.iter() {
                    let (train, moved, n) = robust_ab::smoothed_corpus(&prepared.train, cap, alpha);
                    eprintln!(
                        "    label-smooth-ab α={alpha:.2}: {moved}/{n} working-set docs relabelled — re-reading modelless…"
                    );
                    let sp = prepared.with_train(train);
                    match run_modelless_suite(spec, &sp, opts, None, errors) {
                        ModellessOutcome::Ran { result, .. } => {
                            let d = robust_ab::lane_degradation(
                                "modelless",
                                &ml_clean.items,
                                &result.items,
                            );
                            eprintln!(
                                "    label-smooth-ab α={alpha:.2} modelless: {:.4} → {:.4} (Δ {:+.4} [{:+.4}, {:+.4}])",
                                d.acc_clean, d.acc_corrupted, d.delta, d.delta_lb95, d.delta_ub95
                            );
                            rungs.push(robust_ab::SmoothRung {
                                alpha,
                                working_set: n,
                                relabelled: moved,
                                relabel_rate: moved as f64 / n.max(1) as f64,
                                modelless: d,
                            });
                        }
                        _ => errors.push(format!(
                            "{} (label-smooth-ab α={alpha:.2}): modelless re-read failed",
                            spec.name
                        )),
                    }
                }
                Some(robust_ab::LabelSmoothBlock {
                    effective_cap: cap,
                    k: robust_ab::SMOOTH_K,
                    beta: robust_ab::SMOOTH_BETA,
                    tau: robust_ab::SMOOTH_TAU,
                    encoder_half: "structurally null — every frozen encoder lane reads no \
                                   train rows, so its smoothed read IS its clean read",
                    rungs,
                })
            }
        }
    } else {
        None
    };

    Some(robust_ab::InductiveBiasBlock {
        robust,
        label_smooth,
    })
}

/// The corpus∪cal → eval near-duplicate scan over the slices THIS run
/// serves (issue 098 T3 phase fn; feature `slice_leak`). Reference side =
/// the train split (the corpora draw from train[cal_cap..], the calibration
/// slice IS train[..cal_cap] — the union is the whole split, the probe's
/// exact reference); query side = the built eval cases, whose raw dataset
/// text is recovered by the same rule the corpus side uses. Everything
/// stays None when the feature is off or the suite is out of scope — G3
/// keeps results.json identical.
#[cfg(all(feature = "slice_leak", not(target_arch = "wasm32")))]
fn scan_slice_leaks(
    spec: &SuiteSpec,
    prepared: &Prepared,
    errors: &mut Vec<String>,
) -> (Option<SuiteLeak>, Option<Vec<bool>>) {
    let in_scope = spec.synthetic.is_none() && spec.name != "typed_decisions";
    if !in_scope {
        return (None, None);
    }
    let queries: Option<Vec<String>> = prepared
        .suite
        .cases
        .iter()
        .map(|c| crate::harness::slice_leak::eval_case_text(spec.name, c))
        .collect();
    match queries {
        Some(queries) => {
            let refs: Vec<&str> = prepared.train.iter().map(|d| d.text.as_str()).collect();
            match crate::harness::slice_leak::scan_eval(
                &refs,
                &queries,
                crate::harness::slice_leak::LeakParams::default(),
            ) {
                Some(scan) => {
                    eprintln!(
                        "    leak: exact {} / near {} over {} eval rows \
                         ({} reference rows, J >= {:.2})",
                        scan.exact,
                        scan.near,
                        scan.n_eval,
                        scan.n_reference,
                        crate::harness::slice_leak::LeakParams::default().near_j
                    );
                    (
                        Some(SuiteLeak {
                            threshold: crate::harness::slice_leak::LeakParams::default().near_j,
                            n_reference: scan.n_reference,
                            n_eval: scan.n_eval,
                            exact: scan.exact,
                            near: scan.near,
                        }),
                        Some(scan.flags),
                    )
                }
                None => (None, None),
            }
        }
        None => {
            // A builder key drifted: the scan would silently
            // compare against a shorter query set — surfaced,
            // never read as clean.
            errors.push(format!(
                "{}: leak report SKIPPED — an eval case's dataset text was \
                 not recoverable from its built state (state-shape drift)",
                spec.name
            ));
            (None, None)
        }
    }
}

#[cfg(any(not(feature = "slice_leak"), target_arch = "wasm32"))]
fn scan_slice_leaks(
    _spec: &SuiteSpec,
    _prepared: &Prepared,
    _errors: &mut Vec<String>,
) -> (Option<SuiteLeak>, Option<Vec<bool>>) {
    (None, None)
}

/// The modelless lane for one suite (issue 098 T3 phase fn): const-generic
/// dispatch over the domain count. `Failed` = the error is already recorded
/// and the SUITE is aborted (the run loop's `continue` before the split
/// yielded no suite row at all on a modelless failure).
fn run_modelless_suite(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &RunOptions,
    leak_flags: Option<&[bool]>,
    errors: &mut Vec<String>,
) -> ModellessOutcome {
    if !spec.modelless_lane {
        eprintln!(
            "    modelless: SKIPPED — LLM-lane only (Issue 004 T3: no KV \
             cache in the modelless lane)"
        );
        return ModellessOutcome::NotApplicable;
    }
    #[cfg(feature = "mc_ensemble")]
    let mc_cfg = build_mc_cfg(opts);
    let inp = ModellessInput {
        spec,
        suite: &prepared.suite,
        train: &prepared.train,
        state_strs: &prepared.state_strs,
        cal_cases: &prepared.cal_cases,
        cal_state_strs: &prepared.cal_state_strs,
        labels: &prepared.labels,
        want_by_type: spec.name == "typed_decisions",
        corpus_cap_per_label: if opts.corpus_cap_override != 0 {
            opts.corpus_cap_override
        } else {
            spec.corpus_cap_per_label
        },
        cap_source_base: if opts.corpus_cap_override != 0 {
            "--corpus-cap override"
        } else {
            "registry"
        },
        cal_select_caps: &opts.cal_select_caps,
        pool_rows: &prepared.pool_rows,
        pair_head_ab: opts.pair_head_ab,
        nli_feature_ab: opts.nli_feature_ab,
        nli_m1: opts.nli_m1,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: opts.mc_ab.then_some(&mc_cfg),
        density_gate: opts.density_gate,
        head_scale: opts.head_scale,
        head_select: opts.head_select,
        nb_select: opts.nb_select,
        oc_select: opts.oc_select,
        ridge_select: opts.ridge_select,
        genome_select: opts.genome_select,
        genome_accept_margin: opts.genome_accept_margin,
        leak_flags,
        cascade_worthiness: opts.cascade_worthiness,
        gate_fit_selection: opts.gate_fit_selection,
        gate_distance_only: opts.gate_distance_only,
        gate_fit_calibrated: opts.gate_fit_calibrated,
        kn_route: opts.kn_route,
        drafter_fix: opts.drafter_fix,
        set_refine: {
            #[cfg(feature = "set_refine")]
            {
                opts.set_refine_ab.then_some((
                    if opts.set_refine_gamma > 0.0 {
                        opts.set_refine_gamma
                    } else {
                        1.0
                    },
                    if opts.set_refine_tau > 0.0 {
                        opts.set_refine_tau
                    } else {
                        0.5
                    },
                ))
            }
            #[cfg(not(feature = "set_refine"))]
            {
                None
            }
        },
    };
    macro_rules! dispatch {
        ($n:literal) => {
            run_modelless::<$n>(&inp)
        };
    }
    let modelless = match prepared.labels.len() {
        2 => dispatch!(2),
        3 => dispatch!(3),
        4 => dispatch!(4),
        5 => dispatch!(5),
        6 => dispatch!(6),
        7 => dispatch!(7), // Plan 003: thai_sib200 (the first 7-domain suite)
        8 => dispatch!(8),
        10 => dispatch!(10), // Plan 010: s1mb_score (the presented level-key union)
        15 => dispatch!(15), // Plan 010: s1mb_score headroom (a wider pull widens the union)
        56 => dispatch!(56), // i043: massive_intent_en's deciding front (the stratified REST
                             // covers 56/59 labels — the R1 read exhausted three rare
                             // labels; the dispatch is COUNT-keyed, the 59 arm misses it)
        59 => dispatch!(59),
        77 => dispatch!(77),
        842 => dispatch!(842), // Plan 010: s1mb_choice (the option-key union — the massive by-name posture at breadth)
        n => {
            errors.push(format!(
                "{}: no engine instantiation for {} domains — extend the \
                 dispatch table in runner.rs",
                spec.name, n
            ));
            return ModellessOutcome::Failed;
        }
    };
    match modelless {
        Ok((r, questions)) => {
            eprintln!(
                "    modelless: acc {:.4} · ece(maxp) {:.4} · p50 {:.3} ms · {} s",
                r.hard.accuracy,
                r.hard.ece,
                r.latency_p50_ms,
                (r.seconds * 10.0).round() / 10.0
            );
            ModellessOutcome::Ran {
                result: Box::new(r),
                questions,
            }
        }
        Err(e) => {
            errors.push(format!("{} (modelless): {e}", spec.name));
            ModellessOutcome::Failed
        }
    }
}

/// The riir-laya checkpoints + the laya-python oracle for one suite (issue
/// 098 T3 phase fn).
fn run_laya_lanes(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &RunOptions,
    leak_flags: Option<&[bool]>,
    errors: &mut Vec<String>,
) -> LayaLanes {
    let mut laya = LayaLanes {
        results: BTreeMap::new(),
        questions: BTreeMap::new(),
        cal_questions: BTreeMap::new(),
    };
    if !opts.skip_laya {
        for ck in laya_checkpoints_for(spec.name) {
            eprintln!("    laya[{ck}]: running…");
            let cal_for_probe = if opts.cascade_worthiness {
                Some(prepared.cal_cases.as_slice())
            } else {
                None
            };
            match run_laya_checkpoint(
                &prepared.suite,
                ck,
                opts.laya_max_questions,
                leak_flags,
                cal_for_probe,
            ) {
                Ok((r, notes, lq, cal_lq)) => {
                    eprintln!(
                        "    laya[{ck}]: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0
                    );
                    // The ANE bucket-skip disclosure — named cases,
                    // never a silently smaller served set.
                    errors.extend(notes);
                    laya.results.insert((*ck).to_string(), r);
                    laya.questions.insert((*ck).to_string(), lq);
                    if let Some(cal_lq) = cal_lq {
                        laya.cal_questions.insert((*ck).to_string(), cal_lq);
                    }
                }
                Err(e) => {
                    // Honest absence: weights missing / feature off —
                    // recorded, never silently skipped.
                    errors.push(format!("{} (laya/{ck}): {e}", spec.name));
                }
            }
        }
    }

    // Laya-python lane (opt-in measurement oracle): the ORIGINAL torch
    // reference answering the SAME cases as a subprocess. Keyed `py/`
    // so both backends can appear side by side in the tables.
    if opts.laya_python {
        for ck in laya_checkpoints_for(spec.name) {
            eprintln!("    laya-python[{ck}]: running…");
            match run_laya_python_checkpoint(
                &prepared.suite,
                ck,
                opts.laya_max_questions,
                leak_flags,
            ) {
                Ok(r) => {
                    eprintln!(
                        "    laya-python[{ck}]: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0
                    );
                    laya.results.insert(format!("py/{ck}"), r);
                }
                Err(e) => {
                    // Loud absence, never a silent skip (the same law
                    // as the riir lane's errors).
                    errors.push(format!("{} (laya-python/{ck}): {e}", spec.name));
                }
            }
        }
    }
    laya
}

/// The comparison lanes for one suite (issue 098 T3 phase fn): clm,
/// gliner, bekko, agentjev, clef, openthai, drex, d1, pplx, d3, jeva,
/// startlux, mammoth, elephant, opendecider, paw, paw-local. Every lane is
/// loud-absent (the errors list), never silently skipped.
fn run_comparison_lanes(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &RunOptions,
    leak_flags: Option<&[bool]>,
    errors: &mut Vec<String>,
) -> ComparisonLanes {
    ComparisonLanes {
        // CLM comparison lane (Issue 019 T3 / `.issues/027`): the external
        // Contrastive-LM reference over `/v1/systemone` — their stack
        // serves, our Rust measures. Same cases, their rendering law, the
        // same metrics tail.
        clm: if opts.clm {
            eprintln!("    clm: running…");
            match run_clm_lane(&prepared.suite, leak_flags) {
                Ok((r, input_tokens)) => {
                    eprintln!(
                        "    clm: acc {:.4} · ece(maxp) {:.4} · p50 {:.1} ms · {} s · {} input tokens",
                        r.hard.accuracy,
                        r.hard.ece,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0,
                        input_tokens
                    );
                    Some(r)
                }
                Err(e) => {
                    // Loud absence, never a silent skip (the same law as
                    // the laya lanes' errors).
                    errors.push(format!("{} (clm): {e}", spec.name));
                    None
                }
            }
        } else {
            None
        },
        // GLiNER comparison lane (Issue 029): fastino/GLiNER2.5-Decide as a
        // JSONL subprocess oracle over their gliner2 package.
        gliner: if opts.gliner {
            run_comparison_lane("gliner", spec.name, errors, || {
                run_gliner_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // Bekko comparison lane (Bench 103): hotchpotch/bekko-system-one-v0
        // as a JSONL subprocess oracle over their shared-prefix runtime.
        bekko: if opts.bekko {
            run_comparison_lane("bekko", spec.name, errors, || {
                run_bekko_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // AgentJev comparison lane (Issue 025 amendment 4 / `.issues/027`):
        // their jev_service served on loopback, measured over HTTP.
        agentjev: if opts.agentjev {
            run_comparison_lane("agentjev", spec.name, errors, || {
                run_agentjev_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // Clef comparison lane (plan 011 Phase A): Cloudflare's hosted
        // decision models behind the operator-run loopback forwarder. The
        // no-creds refusal and the spend ceiling are the lane's own gates.
        clef: if opts.clef {
            run_comparison_lane("clef", spec.name, errors, || {
                run_clef_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // OpenThai comparison lane (Plan 003 Phase 2): their
        // openthai_systemone served on loopback, measured over HTTP.
        openthai: if opts.openthai {
            run_comparison_lane("openthai", spec.name, errors, || {
                run_openthai_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // Drex DLM comparison lane (Issue 073): their diffusion-LM server
        // on loopback, measured over HTTP. CC BY-NC weights: measurement
        // only.
        drex: if opts.drex {
            run_comparison_lane("drex", spec.name, errors, || {
                run_drex_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // LiquidAI d1 comparison lane (Issue 078): our reference server
        // over their in-repo D1Model on loopback. lfm1.0 license:
        // measurement only.
        d1: if opts.d1 {
            run_comparison_lane("d1", spec.name, errors, || {
                run_d1_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // pplx-decider comparison lane (Issue 082): Perplexity's
        // pplx-decider-v1.1-27b through their autojev server (Apache-2.0,
        // UNGATED — teacher-eligible, the read decides; never a product
        // lane).
        pplx: if opts.pplx {
            run_comparison_lane("pplx", spec.name, errors, || {
                run_pplx_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // vLLM-SR d3 comparison lane (riir-train 629 T2): the JDI board's
        // d3 entrant through their d3_server.py (Apache-2.0, UNGATED —
        // teacher-eligible; tag v3.1.0).
        d3: if opts.d3 {
            run_comparison_lane("d3", spec.name, errors, || {
                run_d3_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // Jeva comparison lane (Issue 090 B1, feeding riir-train 632):
        // LIQIIIII/Jeva-4B through OUR reference server, measured over
        // HTTP. Weights license UNDECLARED — measurement + teacher-capture
        // posture, adoption fail-closed.
        jeva: if opts.jeva {
            run_comparison_lane("jeva", spec.name, errors, || {
                run_jeva_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // StartLux comparison lane (Issue 095, feeding riir-train 635 T3):
        // board #1 through THEIR startlux-decide engine (weights CC BY-NC
        // 4.0 — DECLARED-NC RESTRICTED posture, adoption fail-closed).
        startlux: if opts.startlux {
            run_comparison_lane("startlux", spec.name, errors, || {
                run_startlux_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // mammoth/elephant hosted comparison lane (rethink Issue 032 T10):
        // Base Labs' board-#1 through Baseten Model APIs behind the loopback
        // TLS forwarder (MIT, MEASUREMENT-ONLY — no RIDT arm).
        mammoth: if opts.mammoth {
            run_comparison_lane("mammoth", spec.name, errors, || {
                run_mammoth_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // elephant-1 self-hosted comparison lane (rethink Issue 032 T10.2 /
        // Research 012 — the LOCAL-FIRST law): Base Labs' 9B via OUR
        // reference server over their qev runtime.
        elephant: if opts.elephant {
            run_comparison_lane("elephant", spec.name, errors, || {
                run_elephant_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // opendecider self-hosted comparison lane (rethink Issue 038 /
        // Research 013): OpenDecider-nano via their `opendecider serve`.
        // The P0 gate (bar, one frozen read, fail classification) is
        // pre-registered in 038 — this block is the rig that produces the
        // read.
        opendecider: if opts.opendecider {
            run_comparison_lane("opendecider", spec.name, errors, || {
                run_opendecider_lane(&prepared.suite, opts.laya_max_questions, leak_flags)
            })
        } else {
            None
        },
        // PAW comparison lane (Issue 033): one program per specced suite
        // (cached), free-text answers under the exact-match law.
        paw: if opts.paw {
            eprintln!("    paw: running…");
            let run = crate::lanes::paw::PawClient::new(crate::lanes::paw::PawConfig::from_env())
                .and_then(|c| {
                    crate::lanes::paw::run_suite(&c, &prepared.suite, opts.laya_max_questions)
                });
            match run {
                Ok(Some(r)) => {
                    eprintln!(
                        "    paw: acc {:.4} · refusals {}/{} · p50 {:.1} ms · {} s",
                        r.accuracy,
                        r.refusals,
                        r.n_questions,
                        r.latency_p50_ms,
                        (r.seconds * 10.0).round() / 10.0
                    );
                    Some(r)
                }
                Ok(None) => {
                    eprintln!(
                        "    paw: ABSENT — no committed spec for {} (scripts/paw_specs/)",
                        spec.name
                    );
                    None
                }
                Err(e) => {
                    errors.push(format!("{} (paw): {e}", spec.name));
                    None
                }
            }
        } else {
            None
        },
        // PAW Posture B lane (Issue 033): the same compiled programs
        // through their LOCAL llama.cpp runtime — the runtime-posture twin
        // of the hosted row above, so the delta isolates the posture.
        paw_local: if opts.paw_local {
            eprintln!("    paw-local: running…");
            let (python, script) = crate::lanes::paw_local::resolve_invocation();
            let cfg = crate::lanes::paw::PawConfig::from_env();
            let run = crate::lanes::paw_local::PawLocalSession::spawn(&python, &script).and_then(
                |mut session| {
                    crate::lanes::paw_local::run_suite(
                        &mut session,
                        &cfg,
                        &prepared.suite,
                        opts.laya_max_questions,
                    )
                },
            );
            match run {
                Ok(Some(r)) => {
                    eprintln!(
                        "    paw-local: acc {:.4} · refusals {}/{} · p50 {:.1} ms · det {} · {} s",
                        r.accuracy,
                        r.refusals,
                        r.n_questions,
                        r.latency_p50_ms,
                        r.determinism_ok.map_or("—".to_string(), |ok| if ok {
                            "✓".into()
                        } else {
                            "✗".into()
                        }),
                        (r.seconds * 10.0).round() / 10.0
                    );
                    Some(r)
                }
                Ok(None) => {
                    eprintln!(
                        "    paw-local: ABSENT — no committed spec for {} (scripts/paw_specs/)",
                        spec.name
                    );
                    None
                }
                Err(e) => {
                    errors.push(format!("{} (paw-local): {e}", spec.name));
                    None
                }
            }
        } else {
            None
        },
    }
}

/// Issue 038 T4′ — the cascade lane (issue 098 T3 phase fn): the modelless
/// answers stand everywhere except the calibrated fused gate's abstains,
/// which escalate to each served riir-laya checkpoint. Composed per
/// checkpoint; a compose refusal is a named absence (the errors list),
/// never a fabricated row.
fn compose_cascade(
    spec: &SuiteSpec,
    prepared: &Prepared,
    opts: &RunOptions,
    modelless_questions: Option<&crate::harness::cascade::ModellessQuestions>,
    laya_lanes: &LayaLanes,
    errors: &mut Vec<String>,
) -> BTreeMap<String, crate::harness::cascade::CascadeReport> {
    let mut composed = BTreeMap::new();
    if !opts.cascade {
        return composed;
    }
    if let Some(mq) = modelless_questions {
        for (ck, lq) in &laya_lanes.questions {
            // Issue 042 lever 3: when armed, the worthiness probe
            // gates THIS checkpoint's escalation on the cal-slice
            // delta. Any missing half reads unprobed (armed + named
            // reason) — never a silent fallthrough.
            let worthiness =
                opts.cascade_worthiness
                    .then(|| crate::harness::cascade::WorthinessInput {
                        gate: mq.cal_gate.as_ref(),
                        escalator: laya_lanes.cal_questions.get(ck),
                        cal_cases: prepared.cal_cases.as_slice(),
                        min_delta: opts.cascade_worthiness_margin,
                        min_probe_lcb: opts.cascade_worthiness_lcb,
                    });
            match crate::harness::cascade::compose(
                &prepared.suite.cases,
                mq,
                lq,
                worthiness.as_ref(),
            ) {
                Ok(rep) => {
                    let disarm_note = rep
                        .worthiness
                        .as_ref()
                        .filter(|w| w.disarmed)
                        .map(|w| {
                            format!(
                                " · DISARMED (cal Δ {:+.3} < min {:+.3}, n {})",
                                w.delta, w.min_delta, w.probe_n
                            )
                        })
                        .unwrap_or_default();
                    eprintln!(
                        "    cascade[{ck}]: acc {:.4} · esc {:.1}% · {} escalated / \
                         {} un-escalated{}",
                        rep.accuracy,
                        rep.escalation_rate * 100.0,
                        rep.n_escalated,
                        rep.n_un_escalated,
                        disarm_note
                    );
                    composed.insert(ck.clone(), rep);
                }
                Err(e) => errors.push(format!("{} (cascade/{ck}): {e}", spec.name)),
            }
        }
    }
    composed
}

/// The run's meta block (issue 098 T3 phase fn): posture prose +
/// provenance — a pure function of the options and the captured box state.
fn build_meta(opts: &RunOptions, box_start: crate::harness::box_state::BoxState) -> RunMeta {
    let laya_feature = cfg!(feature = "laya-riir");
    let box_state = crate::harness::box_state::BoxStateSpan {
        start: box_start,
        end: crate::harness::box_state::capture(),
    };
    RunMeta {
        box_state,
        date_utc: iso8601_utc(),
        git_sha: git_sha().unwrap_or_else(|| "unknown".to_string()),
        host: hostname_refusing_unknown(),
        profile: if cfg!(debug_assertions) {
            "dev".to_string()
        } else {
            "release".to_string()
        },
        laya_feature,
        laya_max_questions: opts.laya_max_questions,
        datasets_dir: opts.datasets_dir.display().to_string(),
        corpus_cap_mode: if opts.corpus_cap_override != 0 {
            format!(
                "override {} (measurement-only --corpus-cap; every dataset suite) — \
                 MEASUREMENT POSTURE, not the registry posture",
                opts.corpus_cap_override
            )
        } else if !opts.cal_select_caps.is_empty() {
            "stratified cap selection: accuracy per candidate cap (+ the registry default) on \
             a label-stratified slice of the pool region, argmax picked on that slice ONLY, \
             test read once at the selected cap (Issue 013 lever-1 protocol)"
                .to_string()
        } else {
            "registry defaults".to_string()
        },
        corpus_protocol: "one engine domain per label; corpus = train-split docs of that \
                          label, capped per label (registry), self-doc fallback for \
                          labels absent from the fetched train rows"
            .to_string(),
        calibration_protocol: {
            let mut p = "sigmoid-gate fit on a train-tail calibration slice (same \
                               builder over the train rows); fused-gate thresholds fitted \
                               per suite at the cal-slice 30th percentile (the T1.6 arena \
                               posture rho=30%; the birth constants do not transfer — \
                               measured: 100% abstain on several suites at defaults); \
                               raw-vs-calibrated readout ECE compared per the G1 gate; \
                               no calibration claim when the calibrator never moved"
                .to_string();
            if opts.gate_fit_selection {
                p.push_str(
                    "; --gate-fit-selection: thresholds fitted on the stratified \
                            selection slice, probe corpus excluding the fit docs \
                            (issue 042 lever 1)",
                );
            }
            if opts.gate_distance_only {
                p.push_str(
                    "; --gate-distance-only: score axis disabled (threshold 0.0), \
                            the fused gate runs on the corpus-distance axis alone \
                            (issue 042 lever 2)",
                );
            }
            if !opts.kn_route {
                p.push_str(
                    "; --no-kn-route: the legacy k==N index-alignment route \
                            binding disabled — route terms arm only via by-name \
                            resolution (issue 079 content-bound posture)",
                );
            }
            if opts.drafter_fix != crate::engine::DrafterFix::Off {
                p.push_str(&format!(
                    "; --drafter-fix {}: the drafter-only correction mode armed \
                     (issue 079 tuning axis; EngineConfig::drafter_fix, issue 036 T2)",
                    opts.drafter_fix.as_str()
                ));
            }
            p
        },
        floor_definition: "conformal-naive floor = split-conformal recalibration of the raw \
                           readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — \
                           the exchangeability-valid baseline (G1 fails unless the \
                           calibrated ECE beats it)"
            .to_string(),
        modal_floor_definition: crate::harness::modal_floor::MODAL_FLOOR_DEFINITION.to_string(),
        determinism_scoping: "the bit-identity claim is the MODELLESS lane's (plan caveat 3); \
                              the laya lane gets the same observed repeat check and it is \
                              reported, not claimed"
            .to_string(),
        laya_device: {
            #[cfg(feature = "laya-riir")]
            {
                use crate::laya::riir::agent::DeviceKind;
                match DeviceKind::from_env() {
                    Ok(DeviceKind::Metal) => {
                        "metal (LAYA_DEVICE or the build's macOS default)".to_string()
                    }
                    Ok(DeviceKind::Cuda) => {
                        "cuda (LAYA_DEVICE or the build's non-macOS CUDA default)".to_string()
                    }
                    Ok(DeviceKind::Cpu) => {
                        "cpu (LAYA_DEVICE or the no-backend default)".to_string()
                    }
                    Ok(DeviceKind::Ane) => {
                        // This harness interprets LAYA_DEVICE=ane itself: it
                        // routes the checkpoint loads to RiirAgent::load_ane
                        // (artifacts: LAYA_ANE_ARTIFACTS_DIR else assets/ane/).
                        // The substrate's plain loader still refuses env-only
                        // ANE — only an explicit consumer choice selects the
                        // lane, never a silent fallback.
                        "ane (LAYA_DEVICE=ane → load_ane; whole-graph CoreML encoder, Plan 002)"
                            .to_string()
                    }
                    Ok(DeviceKind::Cubecl) => {
                        // The harness does NOT interpret cubecl itself: the
                        // env value flows to RiirAgent::load -> from_env,
                        // which refuses loudly when the laya-riir-cubecl
                        // feature is absent (plan 611).
                        "cubecl (LAYA_DEVICE -> load; wgpu portable op layer, plan 611)".to_string()
                    }
                    Err(e) => format!("unknown ({e})"),
                }
            }
            #[cfg(not(feature = "laya-riir"))]
            {
                "n/a (compiled without laya-riir)".to_string()
            }
        },
        laya_python_lane: if opts.laya_python {
            "on — the ORIGINAL torch reference as a JSONL subprocess oracle: same cases, \
             the reference's own rounded-4 probabilities, latency = subprocess round-trip \
             (IPC included)"
                .to_string()
        } else {
            "off (pass --laya-python to add the reference lane)".to_string()
        },
        head_posture: if opts.head_select {
            "ON — cal-selected per suite (ladder 0/0.25/0.5/1, forced cal-slice accuracy, \
             ties → 0 = off; issue 030 lever 4; per-row candidates in the results)"
                .to_string()
        } else if opts.head_scale > 0.0 {
            format!(
                "ON — head_scale {} (fitted per-label one-vs-all logistic, issue 030 lever 4; \
                 MEASUREMENT posture, not the published baseline)",
                opts.head_scale
            )
        } else {
            "OFF (head_scale 0 — the published baseline posture)".to_string()
        },
        nb_posture: if opts.nb_select {
            format!(
                "ON — cal-selected per suite (scale 0/1/4/16/32/64 × α observed-laplace/fixed-1, \
                 promotion bar +5 pt over off on the stratified slice; + noul polarity per \
                 domain on noul suites; + bag/pair view on multi-field states; count tables from \
                 TRAIN rows only, uncapped). Transductive column: {TRANSDUCTIVE_PROTOCOL}"
            )
        } else {
            "OFF (nb_scale 0 — the published baseline posture)".to_string()
        },
        #[cfg(feature = "option_cond")]
        oc_posture: if opts.oc_select {
            "ON — cal-selected per suite (scale 0/0.25/0.5/1/2/4/8, promotion bar +5 pt over off \
             on the stratified slice; one contrastive table per (question id, gold option) \
             from the TRAIN rows, events filtered to the corpus pool; every question kind \
             armed — typed_decisions is the target suite)"
                .to_string()
        } else {
            String::new()
        },
        #[cfg(feature = "nb_ridge")]
        ridge_posture: if opts.ridge_select {
            "ON — cal-selected per suite (scale 0/0.5/1/2/4/8 × λ 10 fixed, promotion bar \
             +5 pt over off on the stratified slice; NBSVM closed-form ridge, k=2048, \
             per-class NB log-count ratios, fit-time self-calibrated margin temperature)"
                .to_string()
        } else {
            String::new()
        },
        sampling_protocol: "test sample = label-STRATIFIED round-robin over the whole test \
                            split (budget = the registry test cap; first-appearance label \
                            order, dataset order within each label, deterministic, no RNG); \
                            cal slice = the same law over the train rows; corpus pool = the \
                            train rows MINUS the cal front (excluded by construction, not \
                            position); budget-0 suites unchanged (identity split) — Issue 039 T2"
            .to_string(),
        readout_posture: "shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side \
                          arming lever overfit the narrow suites' cal slice — emotion's test G1 \
                          regressed — and coincided with Dispatch on the wide suites it targeted; \
                          the wide-label G1 gap was closed by T2's stratified cal slice). The \
                          candidate table is recorded per suite (report-only); \
                          EngineConfig::readout stays the opt-in knob"
            .to_string(),
        clm_lane: if opts.clm {
            "on — the external Contrastive-LM reference over /v1/systemone \
             (clm-serve; comparison lane, Apache-2.0, not affiliated): same cases, \
             their rendering law, latency = client round-trip; determinism = the \
             observed-repeat check"
                .to_string()
        } else {
            "off (pass --clm to add the comparison lane; .issues/027)".to_string()
        },
        gliner_lane: if opts.gliner {
            "on — fastino/GLiNER2.5-Decide over their gliner2 package as a JSONL \
             subprocess oracle (comparison lane, Apache-2.0, not affiliated): same \
             cases, their per-label probability readout (softmax for single-label \
             heads; conf = top-label probability), their fp32 default posture; \
             latency = subprocess round-trip (IPC included) — the honest \
             cross-lane latency comparison is laya-python (the same IPC law), \
             never the in-process riir lane; determinism = the observed-repeat \
             check; .issues/029"
                .to_string()
        } else {
            "off (pass --gliner to add the comparison lane; needs the gliner2 \
             venv — .issues/029)"
                .to_string()
        },
        bekko_lane: if opts.bekko {
            "on — hotchpotch/bekko-system-one-v0 over their BekkoSentenceTransformer \
             runtime as a JSONL subprocess oracle (comparison lane, never a product \
             lane; MIT — verified 2026-10-02): same cases, \
             their softmax-over-candidates readout taken as-is, the state passed AS \
             JSON (their reference law renders JSON states itself), score native \
             when levels are numeric else choice-over-labels (disclosed), noul native \
             with generic meanings; model id + revision ride BEKKO_MODEL/\
             BEKKO_REVISION (default the card's 17M release); latency = subprocess \
             round-trip (IPC included); determinism = the observed-repeat check; \
             Bench 103"
                .to_string()
        } else {
            "off (pass --bekko to add the comparison lane; needs the bekko venv — \
             Bench 103)"
                .to_string()
        },
        agentjev_lane: if opts.agentjev {
            "on — their jev_service (malevrigns/agent-jev @ a965ca8f, Apache-2.0, \
             not affiliated) served on loopback, measured over HTTP (comparison \
             lane, never a product lane): same cases, their decision.v1 contract, \
             gold-label scoring (their published 79.25% is teacher-argmax \
             agreement — this lane's row is the protocol-honest split); latency = \
             client round-trip, their usage.wall_ms recorded beside it; \
             determinism = the observed-repeat check; .issues/025 + .issues/027"
                .to_string()
        } else {
            "off (pass --agentjev to add the comparison lane; needs their \
             jev_service on AGENTJEV_SERVE_URL — .issues/025)"
                .to_string()
        },
        clef_lane: if opts.clef
            && (opts.clef_local
                || std::env::var(crate::lanes::clef::LOCAL_ENV).is_ok_and(|v| v == "1"))
        {
            "on — clef-text-0.6b answered IN-PROCESS on the Apple Neural Engine \
             (plan 619 T3.1) through the riir-infer-clef substrate (feature clef-ane): \
             the T2.2 frozen-capture parity gate is BIT-EXACT vs the official \
             FluidUse runtime; compute units via CLEF_COMPUTE_UNITS (ane default | \
             gpu | cpu); the student-vs-teacher caveat stands (0.6b student of \
             clef-flash); the HTTP arm (plan 011) is untouched; the arm is selected \
             by --clef-local or CLEF_LOCAL=1"
                .to_string()
        } else if opts.clef {
            let url = std::env::var(crate::lanes::clef::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::clef::DEFAULT_SERVE_URL.into());
            format!(
                "on — Cloudflare Clef decision models (Cloudflare/clef + clef-flash, \
                 Apache-2.0, not affiliated) behind a loopback endpoint, measured over \
                 HTTP via {url} (comparison lane, never a product lane): same cases, the \
                 Jev-shaped wire per the vendor (the wire fixture is plan 011 A2.5 — \
                 captured before any published cell), gold-label scoring; the loopback \
                 hop is INCLUDED in the client round-trip; the SERVING PROVENANCE is the \
                 reply envelope's model field (plan 011 A0: the TLS-terminating forwarder \
                 in front of Workers AI, or a local serve — quote the model id + quant in \
                 the record); env: CLEF_SERVE_URL, CLEF_ACCOUNT_ID/CLEF_RUN_PATH, \
                 CLEF_API_TOKEN, CLEF_MODEL, CLEF_TIMEOUT_MS, spend ceiling \
                 CLEF_SMOKE_MAX_CASES (default 50; CLEF_ALLOW_UNCAPPED=1 overrides); \
                 plan 011 Phase A"
            )
        } else {
            "off (pass --clef to add the comparison lane; the hosted posture needs the \
             owner's Workers AI creds behind the loopback forwarder, or serve the open \
             weights locally — plan 011 Phase A + .research/007)"
                .to_string()
        },
        openthai_lane: if opts.openthai {
            "on — openthai_systemone (iapp-technology/OpenThai-SystemOne,              Apache-2.0, not affiliated) served on loopback, measured over              HTTP (comparison lane, never a product lane): same cases,              their /v1/systemone contract, gold-label scoring; liveness =              GET /healthz, the model id read from the response's own model              field (never hardcoded); their usage.input_tokens + the              in-head abstain-slot count reported beside the metrics;              determinism = the observed-repeat check; plan 003 Phase 2"
                .to_string()
        } else {
            "off (pass --openthai to add the comparison lane; needs their \
             openthai service on OPENTHAI_SERVE_URL — plan 003)"
                .to_string()
        },
        drex_lane: if opts.drex {
            let url = std::env::var(crate::lanes::drex::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::drex::DEFAULT_SERVE_URL.into());
            format!(
                "on — Drex DLM (nace-ai/drex-dlm @ 6c63df2, Efficient-DLM-8B backbone; \
                 repo MIT, weights CC BY-NC 4.0 — MEASUREMENT ONLY, never a product \
                 lane, never a distill teacher) served on loopback via {url} \
                 (comparison lane, never a product lane): same cases, their \
                 /v1/systemone contract (the wire pinned from their api.py/serve.py \
                 at the pin), gold-label scoring; liveness = GET /health, the model \
                 id read from the response's own model field (never hardcoded — \
                 their Python serve.py vs the llama.cpp edlm fork is disclosed by \
                 it + the env); their scorer-side latency_ms + billing-style token \
                 counts reported beside the client round-trip; probabilities read \
                 as-is at their 4-decimal serialization (never renormalized); \
                 determinism = the observed-repeat check; Issue 073 + .research/008"
            )
        } else {
            "off (pass --drex to add the comparison lane; needs their service on \
             DREX_SERVE_URL, default http://127.0.0.1:8000 — Issue 073)"
                .to_string()
        },
        d1_lane: if opts.d1 {
            let url = std::env::var(crate::lanes::d1::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::d1::DEFAULT_SERVE_URL.into());
            format!(
                "on — LiquidAI d1-3B (LiquidAI/d1-3B @ 051bcc4, LFM2.5-VL-3B backbone; \
                 license other/lfm1.0 — MEASUREMENT ONLY, never a product lane; the \
                 blog's 'without restrictions' is marketing) served on loopback via \
                 {url} — our reference stdlib server over their in-repo D1Model \
                 (trust_remote_code=True, transformers >=5.14; the repo ships NO HTTP \
                 layer): same cases, their official /decisions/v1/systemone contract \
                 (the wire pinned from their api.py/prompt.py/runner.py at the pin), \
                 gold-label scoring; liveness = GET /health, the model id read from \
                 the response's own model field (never hardcoded); the SERVED DTYPE is \
                 fp16 per their own flip table (fp16 = 0 flips vs fp32; bf16 flips \
                 0.8% text) and rides /health + the boot log — never assumed from the \
                 bf16 checkpoint storage; calibration = the in-repo default (NONE — \
                 the open weights ship no temperature artifact; the 'calibrated' card \
                 claim is what the ECE/Brier cells test); their server latency_ms + \
                 token counts (output ALWAYS 0 — the answer-slot read law) reported \
                 beside the client round-trip; probabilities read as-is (never \
                 renormalized); determinism = the observed-repeat check; Issue 078 + \
                 .research/009"
            )
        } else {
            "off (pass --d1 to add the comparison lane; needs the reference server on \
             D1_SERVE_URL, default http://127.0.0.1:8078 — Issue 078)"
                .to_string()
        },
        pplx_lane: if opts.pplx {
            let url = std::env::var(crate::lanes::pplx::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::pplx::DEFAULT_SERVE_URL.into());
            let device = std::env::var(crate::lanes::pplx::DEVICE_ENV)
                .unwrap_or_else(|_| "cuda:0".to_string());
            format!(
                "on — Perplexity pplx-decider-v1.1-27b (perplexity-ai/pplx-decider-v1.1-27b \
                 @ 3b45dea, Qwen3.8-27B backbone + single-token readout head, \
                 noncausal_full_attention + last pooling; Apache-2.0, UNGATED — \
                 teacher-eligible unlike drex/d1; not affiliated) served through \
                 THEIR autojev server via {url} on {device} (comparison lane, never \
                 a product lane): same cases, their /v1/systemone contract (the wire \
                 pinned from their server.py/types.py/model.py at the pin), \
                 gold-label scoring; liveness = GET /health (refuses while their \
                 lazy first load is 'loading'), the model id + checkpoint basename \
                 read from /health (never hardcoded); the 52GB bf16 checkpoint \
                 exceeds a 24GB card — an offload-patched loader is a numerically \
                 exact bf16 posture, DISCLOSED via PPLX_DEVICE and quoted here; \
                 latency = client round-trip INCLUDING the network hop to the serve \
                 box (the JDI hosted rows' own disclosure class — not comparable to \
                 on-card figures); input tokens reported beside the metrics; the \
                 no-clock law: their body carries no timing, determinism = verbatim \
                 observed-repeat (compute, bf16 wobble included); Issue 082"
            )
        } else {
            "off (pass --pplx to add the comparison lane; serve their autojev \
             server on PPLX_SERVE_URL, default http://127.0.0.1:8793 — Issue 082)"
                .to_string()
        },
        d3_lane: if opts.d3 {
            let url = std::env::var(crate::lanes::d3::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::d3::DEFAULT_SERVE_URL.into());
            let device = std::env::var(crate::lanes::d3::DEVICE_ENV)
                .unwrap_or_else(|_| "cuda:0".to_string());
            format!(
                "on — vLLM-SR d3 27B (tag v3.1.0, Qwen3.8-27B backbone + 256-code \
                 d3-code-readout head, the pplx head class; Apache-2.0, UNGATED — \
                 teacher-eligible; not affiliated) served through THEIR \
                 d3_server.py via {url} on {device} (comparison lane, never \
                 a product lane): same cases, their /v1/systemone contract (the \
                 wire REUSED from the pplx lane — their server is adapted FROM \
                 pplx's autojev server, bounds 255/2-10 identical), gold-label \
                 scoring; liveness = GET /health (refuses while their lazy first \
                 load is 'loading'), the model id read from /health (never \
                 hardcoded — serve version-true with --name d3-v3.1.0, their \
                 health carries no checkpoint field); the 52GB bf16 checkpoint \
                 exceeds a 24GB card — the local serve runs the PATCHED loader \
                 (env-knobbed offload/NF4), and the quant posture is PINNED via \
                 D3_DEVICE and quoted here (the lossy-surface law: which quant \
                 produced the scores gates the captures — riir-train 629 T2's \
                 per-family spot check vs the board's published numbers); \
                 latency = client round-trip INCLUDING the network hop to the \
                 serve box; input tokens reported beside the metrics; the \
                 no-clock law: their body carries no timing, determinism = \
                 verbatim observed-repeat (compute, device wobble included); \
                 riir-train 629 T2"
            )
        } else {
            "off (pass --d3 to add the comparison lane; serve their d3_server.py \
             on D3_SERVE_URL, default http://127.0.0.1:8794 — riir-train 629 T2)"
                .to_string()
        },
        jeva_lane: if opts.jeva {
            let url = std::env::var(crate::lanes::jeva::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::jeva::DEFAULT_SERVE_URL.into());
            let posture = crate::lanes::jeva::JevaLane::default()
                .info()
                .map_or_else(|e| format!("(health unreadable: {e})"), |(p, _)| p);
            format!(
                "on — Jeva-4B (LIQIIIII/Jeva-4B, arXiv:2610.18773; Qwen3.5-4B \
                 backbone + 13.37M decision head — the first decision-native \
                 PAPER model on our split; weights license UNDECLARED → \
                 MEASUREMENT + TEACHER-CAPTURE posture, adoption FAIL-CLOSED, \
                 riir-train 632 T0; not affiliated) served through OUR \
                 reference stdlib server scripts/jeva_server.py over their \
                 in-repo jeva.Jeva via {url} ({posture}): same cases, our \
                 /decide contract (the wire pinned from their types.py + \
                 inference.py at the pin), gold-label scoring; liveness = GET \
                 /health, the model id read from the response's own model \
                 field (never hardcoded); the SERVED DTYPE is the RESOLVED \
                 param dtype off /health (never assumed from the config); \
                 the allow_null posture is server-side (JEVA_ALLOW_NULL, \
                 default 0 = the forced-pick comparison posture; 1 arms \
                 their coverage gate for the A1 probe); probabilities read \
                 as-is (never renormalized); noul reads their CONDITIONAL \
                 q_true (the paper's A3 readout); determinism = the \
                 observed-repeat check over the volatile-stripped body \
                 (latency_ms + usage — their input_tokens moves with the \
                 hybrid option cache, a cache-hit delta is not \
                 non-determinism); reflex Issue 090 B1 + riir-train 632 T2"
            )
        } else {
            "off (pass --jeva to add the comparison lane; serve OUR reference \
             server scripts/jeva_server.py on JEVA_SERVE_URL, default \
             http://127.0.0.1:8095, under the venv python with their repo \
             installed — reflex Issue 090 B1)"
                .to_string()
        },
        startlux_lane: if opts.startlux {
            let url = std::env::var(crate::lanes::startlux::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::startlux::DEFAULT_SERVE_URL.into());
            let posture = crate::lanes::startlux::StartluxLane::default()
                .info()
                .map_or_else(|e| format!("(health unreadable: {e})"), |(p, _)| p);
            format!(
                "on — StartLux-Decision v1.1 27B Instant (startlux-models; board #1 \
                 on the pinned snapshot; Qwen3.8-27B backbone — the pplx/d3 base \
                 family; weights CC BY-NC 4.0 → DECLARED-NC RESTRICTED: \
                 capture-for-MEASUREMENT + teacher-CAPTURE allowed, any ADOPTION \
                 FAIL-CLOSED, riir-train 635 T0; NC-lineage rides every artifact; \
                 not affiliated) served through THEIR Apache-2.0 engine \
                 startlux-decide v1.1.0 over llama.cpp via {url} ({posture}): \
                 same cases, the pplx/d3 /v1/systemone wire (the wire functions \
                 REUSED from the pplx lane), gold-label scoring; the health \
                 handshake GATES the bidirectional Instant posture (kind instant \
                 + prompt_attention bidirectional + engine.bidir_mode server — a \
                 causal/off serve is a different model than the one trained, the \
                 −26 pt class); the quant posture is the STARTLUX_DEVICE stamp \
                 (our 635-T2 arm: Q4_K_M, -ngl 99 — the lossy-surface law); \
                 determinism = verbatim observed-repeat (their timing_ms rides \
                 only with return_details, never set here); answers' \
                 none_probability is read by the parser as unknown-and-ignored \
                 (the RIDT capture carries the license token); reflex Issue 095 \
                 + riir-train 635 T3"
            )
        } else {
            "off (pass --startlux to add the comparison lane; serve their \
             startlux-decide engine on STARTLUX_SERVE_URL, default \
             http://127.0.0.1:8090 — reflex Issue 095, riir-train 635 T3)"
                .to_string()
        },
        mammoth_lane: if opts.mammoth {
            let url = std::env::var(crate::lanes::mammoth::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::mammoth::DEFAULT_SERVE_URL.into());
            let model = std::env::var(crate::lanes::mammoth::MODEL_ENV)
                .ok()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| crate::lanes::mammoth::DEFAULT_MODEL.into());
            let posture = crate::lanes::mammoth::MammothLane::from_env()
                .and_then(|l| l.info())
                .map_or_else(
                    |e| format!("(handshake unreadable: {e})"),
                    |(m, d)| format!("{m} on {d}"),
                );
            format!(
                "on — the hosted mammoth/elephant lane (rethink Issue 032 T10; Base \
                 Labs' mammoth-1 / elephant-1, MIT — the FIRST permissively-licensed \
                 board-#1, no NC posture; not affiliated) on Baseten Model APIs \
                 behind the operator-run loopback TLS forwarder via {url} ({posture}): \
                 catalog id {model} rides the model field (never hardcoded — the \
                 gliner law); same cases, the pplx-family /v1/systemone wire (the \
                 wire functions REUSED from the pplx lane), gold-label scoring; \
                 the handshake is ONE minimal noul probe (the clef law — no \
                 GET /health is assumed on the hosted route); the forwarder hop is \
                 the forwarder hop is \
                 PART of the row's latency claim (a hosted row is never pooled with \
                 on-card figures — the clef A0 disclosure); spend ceiling \
                 MAMMOTH_SMOKE_MAX_CASES default 50 (free week ≠ unmetered; the \
                 determinism reruns TRIPLE the first 10 cases' spend, disclosed; \
                 the compare projects the envelope minus the serve-meta block — \
                 a qev-backed serve stamps wall-clock timing inside the body, \
                 varying per call BY DESIGN; the answers are the content); \
                 MEASUREMENT-ONLY: no RIDT arm — API outputs never enter a capture \
                 artifact (the T10.2 law; teacher lineage stays self-hosted, \
                 riir-train 640); RE-FETCH the Baseten ToS + Model-APIs policy \
                 before the run (the staleness law; free week ends 2026-10-27)"
            )
        } else {
            "off (pass --mammoth to add the hosted comparison lane; run the loopback \
             TLS forwarder to inference.baseten.co on MAMMOTH_SERVE_URL, default \
             http://127.0.0.1:8793, with MAMMOTH_API_TOKEN set — rethink Issue 032 \
             T10; T10.1: the key is an OWNER act; window ends 2026-10-27 23:59 PT)"
                .to_string()
        },
        elephant_lane: if opts.elephant {
            let url = std::env::var(crate::lanes::elephant::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::elephant::DEFAULT_SERVE_URL.into());
            let posture = crate::lanes::elephant::ElephantLane::from_env()
                .and_then(|l| l.info())
                .map_or_else(
                    |e| format!("(handshake unreadable: {e})"),
                    |(m, d)| format!("{m} on {d}"),
                );
            format!(
                "on — the elephant-1 SELF-HOSTED lane (rethink Issue 032 T10.2; the \
                 LOCAL-FIRST law — our reference scripts/elephant_server.py over \
                 their qev runtime @81ce62f + serve.patch, weights MIT, board #4 \
                 public print 72.24; not affiliated) via {url} ({posture}): the \
                 block-bidirectional mask arms from the checkpoint (a causal serve \
                 can never hide — the provenance carries the attention posture, \
                 the 635 lesson); same cases, the pplx-family SystemOne wire (the \
                 wire functions REUSED from the pplx lane); the determinism compare \
                 projects the envelope minus the top-level latency_ms clock (the \
                 mammoth serve-meta class — the answers are the content); the local \
                 row is a DIFFERENT posture from the API row, never pooled (the \
                 clef disclosure law); spend ceiling ELEPHANT_SMOKE_MAX_CASES \
                 default 50 (ELEPHANT_ALLOW_UNCAPPED=1 records the local \
                 zero-marginal-cost posture); latency columns on a loaded MPS box \
                 are characterization, never a claim (the bench-162 quarantine law)"
            )
        } else {
            "off (pass --elephant to add the self-hosted comparison lane; serve our \
             scripts/elephant_server.py under the qev venv on ELEPHANT_SERVE_URL, \
             default http://127.0.0.1:8096 — rethink Issue 032 T10.2, riir-train \
             640 T2(a) records the exact venv + weight pin)"
                .to_string()
        },
        opendecider_lane: if opts.opendecider {
            let url = std::env::var(crate::lanes::opendecider::SERVE_URL_ENV)
                .unwrap_or_else(|_| crate::lanes::opendecider::DEFAULT_SERVE_URL.into());
            let posture = crate::lanes::opendecider::OpenDeciderLane::from_env()
                .and_then(|l| l.info())
                .map_or_else(
                    |e| format!("(handshake unreadable: {e})"),
                    |(m, d)| format!("{m} on {d}"),
                );
            format!(
                "on — the OpenDecider-nano self-hosted lane (rethink Issue 038 / \
                 Research 013; ~400M Ettin-encoder-400m + MLP decision head, \
                 Apache-2.0 with LICENSE file present — the third \
                 encoder-candidate; not affiliated) via their `opendecider serve` \
                 at {url} ({posture}): same cases, the pplx-family /v1/systemone \
                 wire (the wire functions REUSED from the pplx lane), gold-label \
                 scoring; the handshake is ONE minimal noul probe (the clef law); \
                 determinism = verbatim observed-repeat (their server is not the \
                 qev stack — no clock key is known; a disclosed one lands as a \
                 projection BEFORE any measured read); spend ceiling \
                 OPENDECIDER_SMOKE_MAX_CASES default 50 (OPENDECIDER_ALLOW_UNCAPPED=1 \
                 records the local zero-marginal-cost posture — the P0's 400-case \
                 read rides it); THE P0 GATE IS PRE-REGISTERED IN rethink Issue 038 \
                 (bar 0.7825, one frozen read, the wire-smoke on TRAIN rows only, \
                 the selection-on-test caveat lives there); an adoption/seat move \
                 is a separate owner-gated posture (ESC re-arm, Research 002)"
            )
        } else {
            "off (pass --opendecider to add the self-hosted comparison lane; \
             pip install \"opendecider[serve]\" then `opendecider serve --model \
             manjunathshiva/opendecider-nano` on OPENDECIDER_SERVE_URL, default \
             http://127.0.0.1:8000 — rethink Issue 038 / Research 013)"
                .to_string()
        },
        paw_lane: if opts.paw {
            format!(
                "on — ProgramAsWeights (MIT SDK, not affiliated) over their hosted \
                 REST via a curl subprocess, posture {} (anonymous = their free \
                 tier; programs compile PUBLIC); one program per specced suite \
                 (scripts/paw_specs/, cached by (suite, compiler, BLAKE3(spec))); \
                 temperature 0, max_tokens {}; free-text answers mapped by the \
                 exact-match law (one surrounding quote pair stripped, counted), \
                 unparseable = a counted REFUSAL scored wrong, never guessed; NO \
                 confidence/ECE columns (no probability surface — disclosed \
                 divergence); latency = client round-trip incl. network + curl \
                 spawn; hosted inference not promised deterministic (observed-repeat \
                 check); .issues/033",
                crate::lanes::paw::PawConfig::from_env().posture(),
                crate::lanes::paw::MAX_TOKENS
            )
        } else {
            "off (pass --paw to add the comparison lane; PAW_API_KEY optional — \
             .issues/033)"
                .to_string()
        },
        paw_local_lane: if opts.paw_local {
            format!(
                "on — the SAME compiled programs answered through their LOCAL \
                 llama.cpp runtime as a Python subprocess oracle \
                 (programasweights, not affiliated; {} + {}), never compiling — \
                 the program id comes from the hosted lane's cache so the \
                 local-vs-hosted delta isolates the runtime posture on identical \
                 artifacts; greedy by construction, determinism re-verified per \
                 run (observed-repeat, first 10); latency = in-process round-trip \
                 (no network); same mapping law + refusal accounting as --paw; \
                 .issues/033",
                crate::lanes::paw_local::PYTHON_ENV,
                crate::lanes::paw_local::SCRIPT_ENV
            )
        } else {
            "off (pass --paw-local to add the local-runtime twin; needs the \
             programasweights venv + a hosted-lane cache — .issues/033)"
                .to_string()
        },
        divergences: vec![
            "massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — \
             both lanes see byte-identical questions, which is the integrity that matters"
                .to_string(),
            "banking77: mteb/banking77 mirror (the reference's own bench_apps variant); \
             PolyAI/banking77 is script-based and unservable"
                .to_string(),
            "engine context = state + prompt (wire criteria None) — the modelless \
             serving path"
                .to_string(),
            "synthetic fixtures (semantic_defects): in-process fixtures with \
             programmatic gold; the six Issue-004 harness families (2026-10-02) \
             and code_fixtures (2026-10-11) were retired — owner calls — so the \
             modelless lane no longer answers any self-authored suite but the one \
             that reads above chance"
                .to_string(),
        ],
    }
}
