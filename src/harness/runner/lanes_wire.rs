//! The wire/comparison lanes carved from `runner.rs` (issue 098 T1): the
//! laya lane (the riir-owned backend since the candle deletion + its
//! python-protocol oracles), the CLM twins, the jsonl-oracle lane, and the
//! GLiNER/Bekko/AgentJev/clef hosted lanes. Behavior-preserving carve - the
//! bodies moved verbatim; call sites in `runner.rs` keep the bare names via
//! the glob re-import.

#![allow(clippy::too_many_lines)]

use super::*;
// ── the laya lane (the riir-owned backend since .issues/006 — the candle
// reference lane was deleted; `RiirAgent` serves the same envelope surface
// `system_one`, G5-proven against the same frozen captures) ────────────────

/// The checkpoint's agent at the ENV-selected posture. `LAYA_DEVICE=ane`
/// (feature `laya-riir-ane`) routes to `RiirAgent::load_ane` with the
/// artifact root from `LAYA_ANE_ARTIFACTS_DIR` (else `assets/ane/`) — the
/// HARNESS is the explicit consumer choosing the constructor from the env
/// (the substrate's own plain loader still refuses env-only ANE; an env
/// value can never silently demote an explicitly requested lane). A
/// missing artifact tree errors LOUD naming the remedy, never a CPU
/// number wearing an ANE label.
/// `target_os = "macos"` — the SAME scope the substrate gates
/// `RiirAgent::load_ane` with (Core ML artifacts); elsewhere this variant
/// compiles to nothing and the plain fallback below serves (its substrate
/// loader refuses an env `ane` LOUD — an ANE number can never silently
/// wear a CPU label either way).
#[cfg(all(feature = "laya-riir", feature = "laya-riir-ane", target_os = "macos"))]
pub(super) fn load_laya_agent(
    ckpt: &str,
    ck: crate::laya::config::Checkpoint,
) -> Result<crate::laya::riir::RiirAgent, String> {
    use crate::laya::riir::agent::DeviceKind;
    use crate::laya::weights::weights_root;
    use std::path::PathBuf;
    if DeviceKind::from_env().map_err(|e| e.to_string())? == DeviceKind::Ane {
        let ane_root = std::env::var_os("LAYA_ANE_ARTIFACTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("assets/ane"));
        let manifest = ane_root.join("manifest.json");
        if !manifest.exists() {
            return Err(format!(
                "LAYA_DEVICE=ane: no manifest at {} — run scripts/ane_convert.py first \
                 (artifacts are local-only) or set LAYA_ANE_ARTIFACTS_DIR",
                manifest.display()
            ));
        }
        return crate::laya::riir::RiirAgent::load_ane(&weights_root(), ck, &ane_root, &manifest)
            .map_err(|e| format!("laya ane load ({ckpt}): {e}"));
    }
    crate::laya::riir::RiirAgent::load(&weights_root(), ck)
        .map_err(|e| format!("laya load ({ckpt}): {e}"))
}

/// The no-ANE-build form: plain load (an env `ane` value is refused loud
/// by the substrate — fail loud, never a silent fallback). Also the form
/// on a non-macOS box with the `laya-riir-ane` feature ON: the substrate's
/// `load_ane` is macOS-scoped (Core ML), so the plain loader serves and
/// refuses the env selection loud — the same fail-loud law.
#[cfg(all(
    feature = "laya-riir",
    not(all(feature = "laya-riir-ane", target_os = "macos"))
))]
pub(super) fn load_laya_agent(
    ckpt: &str,
    ck: crate::laya::config::Checkpoint,
) -> Result<crate::laya::riir::RiirAgent, String> {
    use crate::laya::riir::RiirAgent;
    use crate::laya::weights::weights_root;
    RiirAgent::load(&weights_root(), ck).map_err(|e| format!("laya load ({ckpt}): {e}"))
}

#[cfg(feature = "laya-riir")]
pub(super) fn run_laya_checkpoint(
    suite: &Suite,
    ckpt: &'static str,
    laya_max_questions: usize,
    // Issue 024 T3: one leak flag per suite.cases entry (the caller's full
    // eval scan). None = feature off / suite out of scope.
    leak_flags: Option<&[bool]>,
    // Issue 042 lever 3: when armed, ALSO serve the suite's CAL slice with
    // this checkpoint — the worthiness probe's escalator half (the cal
    // slice is pool-region selection data; the probe never reads test).
    // Untimed, excluded from the lane's latency/metrics; bucket skips are
    // named and simply shrink the probe set.
    cal_cases: Option<&[SuiteCase]>,
) -> Result<
    (
        LaneResult,
        Vec<String>,
        crate::harness::cascade::LayaQuestions,
        Option<crate::harness::cascade::LayaQuestions>,
    ),
    String,
> {
    use crate::laya::config::Checkpoint;

    let t_start = Instant::now();
    let ck = match ckpt {
        "english" => Checkpoint::English,
        "multilingual" => Checkpoint::Multilingual,
        "typed" => Checkpoint::TypedDecisions,
        other => return Err(format!("unknown checkpoint {other}")),
    };
    let agent = load_laya_agent(ckpt, ck)?;

    // Cap the eval cases when asked (runtime trim; documented in the table
    // when used).
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // GPU pre-ramp (Issue 020): the first measured case after a fresh load
    // pays the Metal pipeline compile (the python oracle pays its MPS graph
    // compile the same way), which is the recorded cold-first-suite
    // signature — the process's first suite inflated across runs. One
    // unmeasured warmup case per (suite, checkpoint) load compiles the
    // exact pipelines the timed loop then uses. `LAYA_HARNESS_NO_WARMUP=1`
    // restores the cold posture (the measurement, not the default).
    // The warmup is the first SERVABLE case: an ANE bucket refusal moves
    // on to the next one (a refused case warms nothing — Bench 042 found
    // prompt_injections' case 0 over the bucket, so its first timed case
    // was the cold one).
    if laya_gpu_preramp_enabled() {
        for case in cases {
            match agent.system_one(&case.state, &case_questions(case)) {
                Ok(_) => {
                    eprintln!(
                        "  [laya {ckpt}] gpu pre-ramp: 1 unmeasured warmup case ({})",
                        case.id
                    );
                    break;
                }
                Err(crate::laya::LayaError::Bucket { .. }) => continue,
                Err(e) => return Err(format!("laya warmup ({ckpt}): {e}")),
            }
        }
    }

    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT — incremented on every actual
    // re-run (bucket-skipped cases `continue` before this, so the count is
    // the SERVED re-runs, the verdict's true support).
    let mut determinism_n: usize = 0;
    // The ANE lane's bucket refusals are a coverage LIMIT, not a compute
    // failure: the case is skipped LOUDLY (named + counted into the suite's
    // disclosure) and the lane continues with the servable cases. Real
    // failures still abort the lane (the ?-path below is untouched).
    // `served_cases` is the slice the metrics tail walks — Eval's vectors
    // are per-SERVED-case, so the assembler must never see the full set.
    let mut served_cases: Vec<&SuiteCase> = Vec::with_capacity(cases.len());
    let mut served_orig: Vec<usize> = Vec::with_capacity(cases.len());
    let mut bucket_skipped_cases: Vec<&str> = Vec::new();

    for (ci, case) in cases.iter().enumerate() {
        let questions = case_questions(case);
        let t0 = Instant::now();
        let answers = match agent.system_one(&case.state, &questions) {
            Ok(a) => a,
            Err(crate::laya::LayaError::Bucket { seq, max, .. }) => {
                bucket_skipped_cases.push(&case.id);
                eprintln!(
                    "  [laya {ckpt}] case {} skipped: seq {seq} > max ANE bucket {max} \
                     (named + counted, not a failure)",
                    case.id
                );
                continue;
            }
            Err(e) => return Err(format!("laya forward ({}, case {ci}): {e}", case.id)),
        };
        durs_ms.push(t0.elapsed().as_millis() as u64);

        if ci < 10 {
            determinism_n += 1;
            let answers2 = agent
                .system_one(&case.state, &questions)
                .map_err(|e| format!("laya determinism rerun (case {ci}): {e}"))?;
            let render = |as_: &[crate::laya::types::Answer]| {
                as_.iter()
                    .map(|a| {
                        format!(
                            "{}|{}|{:.6}|{:.6}|{:.6}",
                            a.choice.as_deref().unwrap_or(""),
                            a.score.unwrap_or(-1.0),
                            a.noul.unwrap_or(-1.0),
                            a.confidence,
                            a.act_probability
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";")
            };
            if render(&answers) != render(&answers2) {
                *determinism_ok.get_or_insert(true) = false;
                // The divergent pair's rendered fields (6dp) print at the
                // fire — Issue 021's hunt was blind until this named the
                // flipping FIELD (act_probability) and its magnitude. The
                // flag is already a recorded outcome; this is disclosure.
                eprintln!(
                    "  [laya {ckpt}] DETERMINISM case {} q{}:\n    a1 = {}\n    a2 = {}",
                    case.id,
                    answers.len(),
                    render(&answers),
                    render(&answers2)
                );
            }
        }

        let mut cprobs = Vec::with_capacity(questions.len());
        let mut cpicks = Vec::with_capacity(questions.len());
        let mut cconfs = Vec::with_capacity(questions.len());
        for (q, ans) in case.questions.iter().zip(answers.iter()) {
            let (p, pick) = laya_prob_pick(q, ans);
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(ans.confidence);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
        served_cases.push(case);
        // The case walk is over `cases` — a PREFIX of suite.cases when the
        // question cap trimmed it — so the enumerate index IS the
        // suite.cases index the leak flags are keyed on.
        served_orig.push(ci);
    }

    // Issue 042 lever 3: the worthiness probe's escalator half — the SAME
    // checkpoint over the suite's CAL slice. Rendering law shared with the
    // timed loop ([`laya_prob_pick`]); NO timing, NO determinism rerun, NO
    // lane metrics — the probe set is selection-slice instrumentation. A
    // bucket skip shrinks the probe set (named); a REAL forward failure is
    // disclosed and the probe reports unprobed — a cal-serving failure must
    // not destroy the checkpoint's already-served test row.
    let cal_questions = match cal_cases {
        None => None,
        Some(cal) => {
            let mut cal_served_orig: Vec<usize> = Vec::new();
            let mut cal_picks: Vec<Vec<usize>> = Vec::new();
            let mut cal_failed: Option<String> = None;
            for (ci, case) in cal.iter().enumerate() {
                let questions = case_questions(case);
                match agent.system_one(&case.state, &questions) {
                    Ok(answers) => {
                        let mut cpicks = Vec::with_capacity(questions.len());
                        for (q, ans) in case.questions.iter().zip(answers.iter()) {
                            let (_, pick) = laya_prob_pick(q, ans);
                            cpicks.push(pick);
                        }
                        cal_served_orig.push(ci);
                        cal_picks.push(cpicks);
                    }
                    Err(crate::laya::LayaError::Bucket { seq, max, .. }) => {
                        eprintln!(
                            "  [laya {ckpt}] cal probe case {} skipped: seq {seq} > max ANE \
                             bucket {max} (probe set shrinks, named)",
                            case.id
                        );
                    }
                    Err(e) => {
                        cal_failed = Some(format!("case {} ({ci}): {e}", case.id));
                        break;
                    }
                }
            }
            if let Some(fail) = cal_failed {
                eprintln!(
                    "  [laya {ckpt}] cal probe FAILED ({fail}) — the worthiness probe \
                     reports unprobed for this checkpoint; the test-side row stands"
                );
            }
            Some(crate::harness::cascade::LayaQuestions {
                served_orig: cal_served_orig,
                picks: cal_picks,
            })
        }
    };

    // Every case over the bucket limit = the suite has NO servable cases:
    // an honest ABSENCE (the caller's errors list), never an empty metrics
    // row (the metrics tail indexes per-served-case vectors). Only the ANE
    // lane can produce skips, so the bucket-max read is gated on the SAME
    // cfg the agent's `ane_bucket_max()` carries (macos + feature) — a
    // narrower gate here would fail to compile on the non-macos ane arm.
    if probs.is_empty() {
        #[cfg(all(target_os = "macos", feature = "laya-riir-ane"))]
        let max_bucket = agent.ane_bucket_max().unwrap_or(0);
        #[cfg(not(all(target_os = "macos", feature = "laya-riir-ane")))]
        let max_bucket = 0;
        return Err(format!(
            "all {} case(s) exceed the ANE buckets (max {}) — no servable cases; \
             this device row is absent for this suite (named, not fabricated)",
            cases.len(),
            max_bucket,
        ));
    }

    // Latency percentiles are computed ONCE in the shared tail
    // (assemble_laya_lane_result) — both lanes' metrics must not diverge.
    // The leak flags are remapped through the bucket skips so the served
    // slice and the keep-mask stay index-aligned (subset_accuracy asserts
    // exactly that).
    let served_flags: Option<Vec<bool>> =
        leak_flags.map(|f| served_orig.iter().map(|&i| f[i]).collect());
    // Issue 038 T4′: the escalator's per-question record (served case
    // indices + picks) — the cascade lane's escalation half.
    let laya_questions = crate::harness::cascade::LayaQuestions {
        served_orig: served_orig.clone(),
        picks: picks.clone(),
    };
    let result = assemble_laya_lane_result(
        "laya-riir",
        ckpt,
        suite.name,
        served_cases.as_slice(),
        served_flags.as_deref(),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    );
    // The bucket-skip disclosure rides back to the caller (who pushes it
    // into the absences section — the named-not-silent channel). `n_cases`
    // / `n_questions` already count the SERVED set only, so the served-vs-
    // corpus delta is visible in the table row itself.
    let notes = if bucket_skipped_cases.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "{} (laya/{ckpt}): {} case(s) skipped over the ANE bucket limit — {:?} \
             (a coverage limit, not a failure; the served row is the smaller set)",
            suite.name,
            bucket_skipped_cases.len(),
            bucket_skipped_cases
        )]
    };
    Ok((result, notes, laya_questions, cal_questions))
}

/// One laya answer → (probability vector in the question's option order,
/// label-space pick). THE one rendering law for the riir laya backend —
/// shared by the timed eval loop and the Issue-042 worthiness probe, so
/// the two can never diverge on what a pick means.
#[cfg(feature = "laya-riir")]
pub(super) fn laya_prob_pick(
    q: &crate::harness::suites::SuiteQuestion,
    ans: &crate::laya::types::Answer,
) -> (Vec<f64>, usize) {
    match q.kind {
        QKind::Choice => {
            let keys: Vec<String> = q
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            let probs: Vec<f64> = ans.probabilities.iter().map(|(_, v)| *v).collect();
            let idx = ans
                .choice
                .as_ref()
                .and_then(|c| keys.iter().position(|k| k == c))
                .unwrap_or_else(|| {
                    probs
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .map_or(0, |(i, _)| i)
                });
            (probs, idx)
        }
        QKind::Score => {
            let probs: Vec<f64> = ans.probabilities.iter().map(|(_, v)| *v).collect();
            let pick = probs
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map_or(0, |(i, _)| i);
            (probs, pick)
        }
        QKind::Noul => {
            let p_true = ans.noul.unwrap_or(0.5);
            (vec![1.0 - p_true, p_true], usize::from(p_true >= 0.5))
        }
    }
}

/// Shared tail of BOTH laya backends (the in-process riir backend and the
/// python subprocess oracle): the answer vectors → metrics, the optional
/// typed-decisions extras, and the [`LaneResult`] envelope. The two lanes
/// differ only in HOW answers are produced — the metrics must not.
#[allow(clippy::too_many_arguments)]
pub(super) fn assemble_laya_lane_result(
    lane: &'static str,
    model: &str,
    suite_name: &str,
    cases: &[impl AsRef<SuiteCase>],
    // Issue 024 T3: one leak flag per SERVED case (aligned with `cases` —
    // the caller maps the full-suite flags through bucket skips / trims).
    // None = feature off / suite out of scope.
    leak_flags: Option<&[bool]>,
    probs: Vec<Vec<Vec<f64>>>,
    picks: Vec<Vec<usize>>,
    confs: Vec<Vec<f64>>,
    durs_ms: Vec<u64>,
    determinism_ok: Option<bool>,
    determinism_n: Option<usize>,
    seconds: f64,
) -> LaneResult {
    let ev = Eval {
        probs,
        picks,
        confs,
        // laya cannot abstain (Research-562 flaw) — the selective metrics
        // report None rather than a fake zero; this field stays empty.
        abstained: Vec::new(),
        causes: Vec::new(),
    };
    let forced = ev.forced_rows(cases);
    let hard = hard_metrics(&forced);
    // Plan 011 §B3: the JDI reference columns, derived from the same
    // forced rows `hard` read (never folded into `hard.accuracy`).
    let jdi_chance = chance_majority(&forced.iter().map(|(g, _)| *g).collect::<Vec<usize>>());
    let jdi_skill = chance_corrected_skill(hard.accuracy, jdi_chance);
    let acc_deleaked = leak_flags.and_then(|keep| {
        let per_case: Vec<usize> = cases.iter().map(|c| c.as_ref().questions.len()).collect();
        subset_accuracy(&forced, keep, &per_case)
    });
    let readout: Vec<(f64, bool)> = {
        let mut pairs = Vec::new();
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, _q) in case.questions.iter().enumerate() {
                pairs.push((ev.confs[ci][qi], ev.picks[ci][qi] == case.gold[qi].idx));
            }
        }
        pairs
    };

    // Issue 072 (`--dump-items`): per-item records from the same vectors the
    // metrics read. laya/oracle lanes cannot abstain (Research-562 flaw) —
    // the gate fields stay `None`, never a fake false.
    let items = {
        let mut out = Vec::with_capacity(ev.n_questions());
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, q) in case.questions.iter().enumerate() {
                out.push(crate::harness::item_dump::ItemOutcome {
                    lane,
                    model: model.to_string(),
                    case_id: case.id.clone(),
                    case_idx: ci,
                    q_idx: qi,
                    kind: q.kind.as_str().to_string(),
                    gold: case.gold[qi].idx,
                    pick: ev.picks[ci][qi],
                    ok: ev.picks[ci][qi] == case.gold[qi].idx,
                    conf: ev.confs[ci][qi],
                    conf_cal: None,
                    abstain_raw: None,
                    abstain_cal: None,
                    cause_cal: None,
                });
            }
        }
        out
    };

    // Optional extras for typed_decisions.
    let mut by_question_type: Option<BTreeMap<String, HardMetrics>> = None;
    let mut soft_acc: Option<f64> = None;
    let mut brier_soft: Option<f64> = None;
    let mut score_mae: Option<f64> = None;
    let mut within_1: Option<f64> = None;
    if suite_name == "typed_decisions" {
        let mut buckets: BTreeMap<String, Vec<(usize, Vec<f64>)>> = BTreeMap::new();
        let (mut ssum, mut sn, mut bsum) = (0.0f64, 0usize, 0.0f64);
        let (mut msum, mut wsum, mut mn) = (0.0f64, 0.0f64, 0usize);
        for (ci, case) in cases.iter().enumerate() {
            let case = case.as_ref();
            for (qi, q) in case.questions.iter().enumerate() {
                buckets
                    .entry(q.kind.as_str().to_string())
                    .or_default()
                    .push((case.gold[qi].idx, ev.probs[ci][qi].clone()));
                if let Some(sm) = soft_metrics(&ev.probs[ci][qi], &case.gold[qi].soft) {
                    ssum += sm.soft_acc;
                    bsum += sm.brier_soft;
                    sn += 1;
                }
                if let Some(gs) = case.gold[qi].gold_score {
                    let sm = score_metrics(&ev.probs[ci][qi], gs);
                    msum += sm.mae;
                    wsum += sm.within_1;
                    mn += 1;
                }
            }
        }
        by_question_type = Some(
            buckets
                .into_iter()
                .map(|(k, rows)| (k, hard_metrics(&rows)))
                .collect(),
        );
        if sn > 0 {
            soft_acc = Some(ssum / sn as f64);
            brier_soft = Some(bsum / sn as f64);
        }
        if mn > 0 {
            score_mae = Some(msum / mn as f64);
            within_1 = Some(wsum / mn as f64);
        }
    }

    let (p50, p99, support) = percentile_us(&durs_ms);
    LaneResult {
        lane,
        model: model.to_string(),
        hard,
        readout_ece: Some(ece_of(&readout)),
        readout_brier: Some(brier_of(&readout)),
        raw_abstain: None,
        calibrated_abstain: None,
        // laya cannot abstain — no causes to attribute (never a fake zero).
        abstain_causes: None,
        readout_ece_raw: None,
        readout_ece_calibrated: None,
        floor_ece: None,
        g1_pass: None,
        g1_verdict: None,
        drex_conf_readout: None,
        by_question_type,
        soft_acc,
        brier_soft,
        score_mae,
        within_1,
        latency_p50_ms: p50 as f64,
        latency_p99_ms: p99 as f64,
        latency_tail_support: support,
        latency_extremes: LatencyExtremes::of(&durs_ms, 1.0),
        determinism_ok,
        determinism_n,
        seconds,
        n_cases: cases.len(),
        n_questions: ev.n_questions(),
        acc_deleaked,
        jdi_chance: Some(jdi_chance),
        jdi_skill: Some(jdi_skill),
        score_threshold: f32::NAN, // laya exposes no abstain knob — N/A
        distance_threshold: f32::NAN,
        threshold_recommendation: None,
        corpus_cap: None,     // the laya lanes have no corpus
        head_selection: None, // no fitted heads on the laya lanes
        nb_selection: None,
        #[cfg(feature = "option_cond")]
        oc_selection: None,
        #[cfg(feature = "nb_ridge")]
        ridge_selection: None,
        genome_selection: None, // the laya lanes run no joint selection
        transductive: None,
        confusion: None, // the pair probe is the modelless lane's instrument
        items,
        pair_head_ab: None,
        nli_feature_ab: None,
        nli_m1: None,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: None,
        #[cfg(feature = "density_gate")]
        density_ab: None, // the laya lanes run no fused gate
        #[cfg(feature = "set_refine")]
        set_refine_ab: None, // the laya lanes run no modelless seam
        readout_report: None,
        corpus_fallbacks: Vec::new(), // laya reads no train rows (Issue 039)
        corpus_digest: None,          // laya reads no train rows (Issue 057)
    }
}

/// The laya-python lane: the ORIGINAL torch reference driven as a JSONL
/// subprocess oracle (measurement-only — the product lane stays the riir
/// backend; the owner's "no Python anywhere" directive governs the shipped
/// binary, not the bench reference, per the probe_orig_laya_latency
/// precedent). SAME cases, SAME answer mapping, SAME metrics tail as the
/// riir lane ([`assemble_laya_lane_result`]) — the only differences are the
/// forward's executor and the reference's own rounded-4 probabilities.
/// Latency is the subprocess round-trip per case (IPC included) and is
/// disclosed as such in the run meta.
pub(super) fn run_laya_python_checkpoint(
    suite: &Suite,
    ckpt: &str,
    laya_max_questions: usize,
    // Issue 024 T3: one leak flag per suite.cases entry. The served set is
    // a PREFIX of suite.cases (the trim law), so the flags stay aligned.
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{ChildStdin, Command, Stdio};

    let script = std::env::var("LAYA_PYTHON_LANE_SCRIPT")
        .unwrap_or_else(|_| "scripts/laya_python_lane.py".to_string());
    if !Path::new(&script).is_file() {
        return Err(format!(
            "oracle script not found at {script} — run from the repo root or set \
             LAYA_PYTHON_LANE_SCRIPT (the lane is opt-in measurement tooling)"
        ));
    }
    let python = std::env::var("LAYA_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let device = std::env::var("LAYA_PY_DEVICE").unwrap_or_else(|_| "mps".to_string());

    let t_start = Instant::now();
    let mut child = Command::new(&python)
        .arg(&script)
        .arg(ckpt)
        .arg(&device)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // stderr inherits: the reference's load warnings stay visible.
        .spawn()
        .map_err(|e| format!("spawn {python} {script} ({ckpt}): {e}"))?;
    let mut stdin: ChildStdin = child
        .stdin
        .take()
        .ok_or_else(|| "oracle stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "oracle stdout unavailable".to_string())?;
    let mut reader = BufReader::new(stdout);

    let send_case = |stdin: &mut ChildStdin, case: &SuiteCase| -> Result<(), String> {
        let mut qs = Vec::with_capacity(case.questions.len());
        for q in &case.questions {
            let mut def = serde_json::Map::new();
            def.insert("type".into(), Value::String(q.kind.as_str().into()));
            def.insert("instructions".into(), Value::String(q.instructions.clone()));
            if !q.criteria.is_null() {
                def.insert("criteria".into(), q.criteria.clone());
            }
            qs.push(serde_json::json!({"qid": q.qid, "def": Value::Object(def)}));
        }
        let line = serde_json::json!({"state": case.state, "questions": qs});
        writeln!(stdin, "{line}").map_err(|e| format!("oracle stdin ({ckpt}): {e}"))?;
        stdin
            .flush()
            .map_err(|e| format!("oracle stdin flush ({ckpt}): {e}"))
    };

    let read_line = |reader: &mut BufReader<std::process::ChildStdout>,
                     buf: &mut String|
     -> Result<(), String> {
        buf.clear();
        let n = reader
            .read_line(buf)
            .map_err(|e| format!("oracle stdout ({ckpt}): {e}"))?;
        if n == 0 {
            return Err(format!(
                "oracle stream ended early ({ckpt}) — the reference process died; \
                 its stderr above names the cause"
            ));
        }
        Ok(())
    };

    // Handshake: one line once the checkpoint is loaded.
    let mut line = String::new();
    read_line(&mut reader, &mut line)?;
    let ready: Value = serde_json::from_str(line.trim())
        .map_err(|e| format!("oracle handshake ({ckpt}): {e} (got: {})", line.trim()))?;
    if ready.get("ready") != Some(&Value::Bool(true)) {
        return Err(format!(
            "oracle handshake ({ckpt}): expected {{\"ready\":true}}, got {line}"
        ));
    }

    // Cap the eval cases when asked — the SAME trim law as the riir lane.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // GPU pre-ramp (Issue 020), the oracle half: the first measured case
    // pays the reference's MPS graph compile — the same cold-first-suite
    // signature the riir lane's warmup addresses. One unmeasured case per
    // (suite, checkpoint) subprocess; `LAYA_HARNESS_NO_WARMUP=1` restores
    // the cold posture.
    if laya_gpu_preramp_enabled()
        && let Some(first) = cases.first()
    {
        send_case(&mut stdin, first)?;
        let mut line = String::new();
        read_line(&mut reader, &mut line)?;
        eprintln!("  [laya-python {ckpt}] gpu pre-ramp: 1 unmeasured warmup case");
    }

    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        send_case(&mut stdin, case)?;
        let mut line = String::new();
        read_line(&mut reader, &mut line)?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        let resp: Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("oracle response ({ckpt}, case {ci}): {e}"))?;

        if ci < 10 {
            determinism_n += 1;
            send_case(&mut stdin, case)?;
            let mut line2 = String::new();
            read_line(&mut reader, &mut line2)?;
            if line != line2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        for q in case.questions.iter() {
            let answers = resp
                .get("answers")
                .and_then(|a| a.as_object())
                .ok_or_else(|| format!("oracle response ({ckpt}, case {ci}): no answers object"))?;
            let a = answers.get(&q.qid).ok_or_else(|| {
                format!("oracle response ({ckpt}, case {ci}): missing qid {}", q.qid)
            })?;
            let (p, pick, conf) = parse_python_answer(q, a)
                .map_err(|e| format!("oracle response ({ckpt}, case {ci}): {e}"))?;
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    drop(stdin);
    let _ = child.wait();

    Ok(assemble_laya_lane_result(
        "laya-python",
        ckpt,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// Map one oracle answer object (`{"p": [...], "conf": c, "choice"?,
/// "noul"?}`) to the SAME (probs, pick, confidence) triple the riir lane's
/// answer mapping produces — the metrics must not see a different shape.
pub fn parse_python_answer(
    q: &crate::harness::suites::SuiteQuestion,
    a: &Value,
) -> Result<(Vec<f64>, usize, f64), String> {
    let p: Vec<f64> = a
        .get("p")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| format!("qid {}: missing/invalid p", q.qid))?;
    let conf = a
        .get("conf")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("qid {}: missing/invalid conf", q.qid))?;
    let pick = match q.kind {
        QKind::Choice => {
            let keys: Vec<String> = q
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            a.get("choice")
                .and_then(Value::as_str)
                .and_then(|c| keys.iter().position(|k| k == c))
                .unwrap_or_else(|| {
                    p.iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .map_or(0, |(i, _)| i)
                })
        }
        QKind::Score => p
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i),
        QKind::Noul => {
            if p.len() != 2 {
                return Err(format!("qid {}: noul p must be [1-n, n]", q.qid));
            }
            usize::from(p[1] >= 0.5)
        }
    };
    Ok((p, pick, conf))
}

#[cfg(not(feature = "laya-riir"))]
pub(super) fn run_laya_checkpoint(
    _suite: &Suite,
    _ckpt: &str,
    _laya_max_questions: usize,
    _leak_flags: Option<&[bool]>,
    _cal_cases: Option<&[SuiteCase]>,
) -> Result<
    (
        LaneResult,
        Vec<String>,
        crate::harness::cascade::LayaQuestions,
        Option<crate::harness::cascade::LayaQuestions>,
    ),
    String,
> {
    Err("laya-riir feature off".to_string())
}

/// GPU pre-ramp switch (Issue 020): default ON; `LAYA_HARNESS_NO_WARMUP=1`
/// is the explicit cold-posture opt-out (only the literal truthy spelling
/// disables — a typo must not silently restore the cold lane).
pub(super) fn laya_gpu_preramp_enabled() -> bool {
    std::env::var("LAYA_HARNESS_NO_WARMUP").ok().as_deref() != Some("1")
}

// ── the CLM comparison lane (Issue 019 T3 / .issues/027) ─────────────
//
// PARITY LAW (what reaches THEIR encoder, byte-for-byte their reference
// protocol — `src/lanes/clm.rs` pins the law copies):
//   * state  = their `to_text(state)` prose (our copy is byte-pinned, so
//     the pre-rendered string IS what their server would render from the
//     raw object — `to_text(str)` is identity on their side);
//   * choice candidates = the criterion DESCRIPTION when one is given,
//     else the key (their `candidates` law: "a candidate reaches the
//     encoder exactly as the caller wrote it");
//   * score candidates  = the rubric level texts;
//   * noul candidates   = their default law (per-side descriptions are
//     inexpressible on our wire — the documented adapter divergence).
// Each lane applies its OWN pinned rendering law over the same case
// content (the laya lane prefixes `key: desc`; CLM embeds the bare
// description — the systems are compared, not their prompt formats).

/// The `/v1/systemone` request for one harness case, built under THEIR
/// rendering law (see the parity note above). The question ORDER and the
/// option ORDER are the harness criteria's insertion order — the same
/// label order every lane scores against.
#[cfg(feature = "clm-lane")]
pub(super) fn clm_request(
    case: &SuiteCase,
) -> Result<katgpt_core::decision_wire::DecisionRequest, String> {
    use crate::lanes::clm;
    use katgpt_core::decision_wire::Question;
    let mut questions = Vec::with_capacity(case.questions.len());
    for q in &case.questions {
        let question = match q.kind {
            QKind::Choice => {
                let obj = q
                    .criteria
                    .as_object()
                    .ok_or_else(|| format!("case {}: choice criteria must be an object", q.qid))?;
                // THEIR candidates law: the description when present, else
                // the key (their `crit[k] in (None, "")` fallback).
                let options: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| match v {
                        Value::Null => k.clone(),
                        Value::String(s) if s.is_empty() => k.clone(),
                        Value::String(s) => s.clone(),
                        other => clm::to_text(other),
                    })
                    .collect();
                Question::choice(q.qid.as_str(), q.instructions.as_str(), options, None)
            }
            QKind::Score => {
                let levels: Vec<String> = q
                    .criteria
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect()
                    })
                    .ok_or_else(|| format!("case {}: score criteria must be an array", q.qid))?;
                Question::score(q.qid.as_str(), q.instructions.as_str(), levels)
            }
            QKind::Noul => Question::noul(q.qid.as_str(), q.instructions.as_str()),
        };
        questions.push(question);
    }
    Ok(katgpt_core::decision_wire::DecisionRequest {
        state: clm::to_text(&case.state),
        questions,
    })
}

/// One rendered answer row for the observed-repeat determinism check —
/// the SAME law as the laya lane's `render` (6-decimal probabilities,
/// pick, confidence), so a nondeterministic server shows up identically
/// in both lanes' `det` columns.
#[cfg(feature = "clm-lane")]
pub(super) fn clm_repeat_render(answers: &[katgpt_core::decision_wire::Answer]) -> String {
    answers
        .iter()
        .map(|a| format!("{:?}|{:?}|{:.6}", a.outcome, a.probabilities, a.confidence))
        .collect::<Vec<_>>()
        .join(";")
}

/// Run the CLM comparison lane over one suite: every case → one
/// `/v1/systemone` round trip, latency = CLIENT round trip (the
/// cross-lane consistency law — the same wall-clock every other lane's
/// p50 reports), `usage.input_tokens` summed for the run log.
#[cfg(feature = "clm-lane")]
pub(super) fn run_clm_lane(
    suite: &Suite,
    leak_flags: Option<&[bool]>,
) -> Result<(LaneResult, u64), String> {
    use crate::lanes::clm::{ClmLane, DEFAULT_MODEL};
    use katgpt_core::decision_wire::Outcome;

    let t_start = Instant::now();
    let lane = ClmLane::default();
    let cases = &suite.cases;
    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut input_tokens: u64 = 0;

    // WARMUP (measured 2026-09-25, the determinism pin's cold-start
    // finding): the very FIRST request after a clm-serve boot answers
    // correctly but reports usage.input_tokens = 0 — a server-side
    // first-request accounting quirk. One throwaway FIXED request (never
    // a case's — no cache pollution of measured latencies) absorbs the
    // cold path before the first measured case.
    {
        use katgpt_core::decision_wire::Question;
        let warm = katgpt_core::decision_wire::DecisionRequest {
            state: "warmup: the lane's cold-path probe (discarded; not a \
                    measured case)"
                .to_string(),
            questions: vec![Question::noul("warm", "Is this the warmup?")],
        };
        if let Err(e) = lane.decide(&warm) {
            return Err(format!(
                "clm warmup: {e} — is their stack serving at \
                 CLM_SERVE_URL? (scripts/clm_serve_4090.sh status)"
            ));
        }
    }
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;

    for (ci, case) in cases.iter().enumerate() {
        let req = clm_request(case)?;
        let t0 = Instant::now();
        let (resp, usage) = lane
            .decide(&req)
            .map_err(|e| format!("clm round trip ({}): {e}", case.id))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        input_tokens += usage.input_tokens;

        // Observed-repeat check, first 10 cases (the laya lane's law): a
        // lane that cannot repeat byte-identically flags its det column.
        if ci < 10 {
            determinism_n += 1;
            let (resp2, _) = lane
                .decide(&req)
                .map_err(|e| format!("clm determinism rerun ({}): {e}", case.id))?;
            if clm_repeat_render(&resp.answers) != clm_repeat_render(&resp2.answers) {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        for (q, ans) in case.questions.iter().zip(resp.answers.iter()) {
            let (p, pick) = match q.kind {
                // `map_response` returns probabilities in OUR option order
                // — the criteria insertion order — which IS the label
                // order the gold indexes score against.
                QKind::Choice => {
                    let idx = match ans.outcome.as_ref() {
                        Some(Outcome::Choice { index }) => *index as usize,
                        _ => 0,
                    };
                    (
                        ans.probabilities.iter().map(|p| f64::from(*p)).collect(),
                        idx,
                    )
                }
                QKind::Score => {
                    let lvl = match ans.outcome.as_ref() {
                        Some(Outcome::Score { level }) => *level as usize,
                        _ => 0,
                    };
                    (
                        ans.probabilities.iter().map(|p| f64::from(*p)).collect(),
                        lvl,
                    )
                }
                // LABEL space: [p_no, p_yes], gold idx 1 = true (the same
                // convention every lane's noul row reports).
                QKind::Noul => {
                    let p_yes = f64::from(ans.probabilities.first().copied().unwrap_or(0.5));
                    (vec![1.0 - p_yes, p_yes], usize::from(p_yes >= 0.5))
                }
            };
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(f64::from(ans.confidence));
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }

    let result = assemble_laya_lane_result(
        "clm",
        DEFAULT_MODEL,
        suite.name,
        cases,
        leak_flags,
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    );
    Ok((result, input_tokens))
}

/// The feature-off stub: an explicit `--clm` is a LOUD refusal naming the
/// rebuild, never a silent absence (the build-stamp law).
#[cfg(not(feature = "clm-lane"))]
pub(super) fn run_clm_lane(
    _suite: &Suite,
    _leak_flags: Option<&[bool]>,
) -> Result<(LaneResult, u64), String> {
    Err(
        "clm-lane feature off — rebuild with --features clm-lane to measure the CLM reference"
            .to_string(),
    )
}

/// The shared JSONL-subprocess-oracle handle — spawn + handshake + the
/// send/ask wire, ONE implementation for every external oracle that speaks
/// the laya-python line protocol over stdin/stdout: the GLiNER/Bekko
/// comparison lanes ([`run_jsonl_oracle_lane`]) AND the distill teacher's
/// bekko arm (riir-train Issue 608) — the wire is the point of the
/// protocol, never a per-consumer copy. `ask` returns the RAW response
/// line: the comparison lane's determinism check is byte-exact on it, and
/// each consumer applies its own parse.
pub(crate) struct JsonlOracle {
    lane: &'static str,
    child: std::process::Child,
    stdin: Option<std::process::ChildStdin>,
    reader: std::io::BufReader<std::process::ChildStdout>,
    /// The advertised model id — the handshake stamps it, never a
    /// hardcoded name (an env override shows up as itself).
    pub(crate) model: String,
    /// The advertised device (the handshake's own disclosure).
    pub(crate) device: String,
}

impl JsonlOracle {
    /// Spawn the oracle subprocess and read its ready handshake. `extra_env`
    /// overrides inherited env for the CHILD only (the teacher's model pins
    /// — the parent process never mutates its own env).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn spawn(
        lane: &'static str,
        script_env: &str,
        script_default: &str,
        python_env: &str,
        device_env: &str,
        default_device: &str,
        fail_hint: &str,
        extra_env: &[(&str, &str)],
    ) -> Result<Self, String> {
        let script = std::env::var(script_env).unwrap_or_else(|_| script_default.to_string());
        if !Path::new(&script).is_file() {
            return Err(format!(
                "{lane} lane script not found at {script} — run from the repo root or set \
                 {script_env} ({fail_hint})"
            ));
        }
        let python = std::env::var(python_env).unwrap_or_else(|_| "python3".to_string());
        let device = std::env::var(device_env).unwrap_or_else(|_| default_device.to_string());

        let mut child = std::process::Command::new(&python)
            .arg(&script)
            .arg(&device)
            .envs(extra_env.iter().copied())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            // stderr inherits: their loader's warnings stay visible.
            .spawn()
            .map_err(|e| format!("spawn {python} {script}: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| format!("{lane} oracle stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| format!("{lane} oracle stdout unavailable"))?;
        let mut reader = std::io::BufReader::new(stdout);

        // Handshake: one ready line once the checkpoint is loaded; the
        // advertised model id is stamped into the row (never hardcoded).
        let mut line = String::new();
        read_raw_line(lane, &mut reader, &mut line)?;
        let ready: Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("{lane} handshake: {e} (got: {})", line.trim()))?;
        if ready.get("ready") != Some(&Value::Bool(true)) {
            return Err(format!(
                "{lane} handshake: expected {{\"ready\":true}}, got {line}"
            ));
        }
        let model = ready
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string();
        let device = ready
            .get("device")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string();
        Ok(Self {
            lane,
            child,
            stdin: Some(stdin),
            reader,
            model,
            device,
        })
    }

    /// One request/response round trip — returns the RAW response line
    /// (the caller parses; the comparison lane's determinism check is
    /// byte-exact on the raw bytes).
    pub(crate) fn ask(&mut self, case: &SuiteCase) -> Result<String, String> {
        use std::io::Write;
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| format!("{} oracle stdin closed", self.lane))?;
        let questions = case_questions(case);
        let qs: Vec<Value> = questions
            .iter()
            .map(|(qid, def)| serde_json::json!({"qid": qid, "def": def}))
            .collect();
        let line = serde_json::json!({"state": case.state, "questions": qs});
        writeln!(stdin, "{line}").map_err(|e| format!("{} stdin: {e}", self.lane))?;
        stdin
            .flush()
            .map_err(|e| format!("{} stdin flush: {e}", self.lane))?;
        let mut buf = String::new();
        read_raw_line(self.lane, &mut self.reader, &mut buf)?;
        Ok(buf)
    }
}

impl Drop for JsonlOracle {
    fn drop(&mut self) {
        // Closing stdin is the script's exit signal (its loop reads to
        // EOF); wait so the subprocess never outlives the run silently.
        drop(self.stdin.take());
        let _ = self.child.wait();
    }
}

pub(super) fn read_raw_line(
    lane: &str,
    reader: &mut std::io::BufReader<std::process::ChildStdout>,
    buf: &mut String,
) -> Result<(), String> {
    use std::io::BufRead;
    buf.clear();
    let n = reader
        .read_line(buf)
        .map_err(|e| format!("{lane} stdout: {e}"))?;
    if n == 0 {
        return Err(format!(
            "{lane} oracle stream ended early — the subprocess died; its \
             stderr above names the cause"
        ));
    }
    Ok(())
}

/// The shared JSONL-subprocess-oracle lane runner — ONE implementation for
/// every external oracle that speaks the laya-python line protocol (the
/// GLiNER and Bekko lanes today), over the [`JsonlOracle`] wire. The
/// per-lane identity (env names, script default, device default, failure
/// hint) rides the parameters; the trim law, the pre-ramp, the determinism
/// check and the metrics tail are shared verbatim.
///
/// Env contract per lane: `<script_env>` (default `script_default`) ·
/// `<python_env>` (default `python3`) · `<device_env>` (default
/// `default_device`). The handshake advertises the loaded model id and
/// device; the lane stamps THOSE into the row, never a hardcoded name (an
/// env override shows up as itself).
#[allow(clippy::too_many_arguments)]
pub(super) fn run_jsonl_oracle_lane(
    lane: &'static str,
    script_env: &str,
    script_default: &str,
    python_env: &str,
    device_env: &str,
    default_device: &str,
    fail_hint: &str,
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    let t_start = Instant::now();
    let mut oracle = JsonlOracle::spawn(
        lane,
        script_env,
        script_default,
        python_env,
        device_env,
        default_device,
        fail_hint,
        &[],
    )?;
    let model = oracle.model.clone();
    eprintln!(
        "    [{lane}] oracle up: {} on {}",
        oracle.model, oracle.device
    );

    // The trim law: the SAME question cap as the laya lanes, so a capped
    // run never compares a full-N row against a capped row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // GPU pre-ramp (Issue 020), the same law as both laya lanes: one
    // unmeasured warmup case absorbs the CUDA graph/kernel-compile cold
    // path before the first measured case. `LAYA_HARNESS_NO_WARMUP=1`
    // restores the cold posture for every GPU lane at once.
    if laya_gpu_preramp_enabled()
        && let Some(first) = cases.first()
    {
        oracle.ask(first)?;
        eprintln!("  [{lane}] gpu pre-ramp: 1 unmeasured warmup case");
    }

    let mut probs = Vec::with_capacity(cases.len());
    let mut picks = Vec::with_capacity(cases.len());
    let mut confs = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let line = oracle.ask(case)?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        let resp: Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("{lane} response (case {ci}): {e}"))?;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column.
        if ci < 10 {
            determinism_n += 1;
            let line2 = oracle.ask(case)?;
            if line != line2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(case.questions.len());
        let mut cpicks = Vec::with_capacity(case.questions.len());
        let mut cconfs = Vec::with_capacity(case.questions.len());
        for q in case.questions.iter() {
            let answers = resp
                .get("answers")
                .and_then(|a| a.as_object())
                .ok_or_else(|| format!("{lane} response (case {ci}): no answers object"))?;
            let a = answers
                .get(&q.qid)
                .ok_or_else(|| format!("{lane} response (case {ci}): missing qid {}", q.qid))?;
            let (p, pick, conf) = parse_python_answer(q, a)
                .map_err(|e| format!("{lane} response (case {ci}): {e}"))?;
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    drop(oracle);

    Ok(assemble_laya_lane_result(
        lane,
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The GLiNER comparison lane (Issue 029) — fastino/GLiNER2.5-Decide
/// as a JSONL subprocess oracle over THEIR gliner2 package
/// (`scripts/gliner_lane.py`) — the laya-python lane's protocol and answer
/// mapping (SAME cases, SAME metrics tail via [`assemble_laya_lane_result`];
/// the only differences are the forward's executor and their probability
/// readout). Latency is the subprocess round-trip per case (IPC included),
/// the same measurement law as the laya-python oracle — the honest
/// cross-lane comparison for it is laya-python, NOT the in-process riir
/// lane; the table's posture line says so.
pub(super) fn run_gliner_lane(
    suite: &Suite,
    laya_max_questions: usize,
    // Issue 024 T3: one leak flag per suite.cases entry; the served set is
    // a PREFIX (the trim law), so the flags stay aligned.
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    run_jsonl_oracle_lane(
        "gliner",
        "GLINER_LANE_SCRIPT",
        "scripts/gliner_lane.py",
        "GLINER_PYTHON",
        "GLINER_PY_DEVICE",
        "cuda",
        "the lane is opt-in measurement tooling; the venv needs gliner2 + \
         torch + transformers + peft + accelerate",
        suite,
        laya_max_questions,
        leak_flags,
    )
}

/// The Bekko comparison lane (Bench 103, owner call 2026-10-01) —
/// hotchpotch/bekko-system-one-v0 (17M/68M/400M shared-prefix encoders,
/// the same choice/noul/score decision vocabulary as the wire) as a JSONL
/// subprocess oracle over THEIR `BekkoSentenceTransformer` runtime
/// (`scripts/bekko_lane.py`) — the same protocol, the same metrics tail.
/// Latency is the subprocess round-trip per case (IPC included — the
/// laya-python measurement law). The model id rides `BEKKO_MODEL` (default
/// the card's 17M release; the bench-record runs pin 68m explicitly) and
/// `BEKKO_REVISION` pins the card's release commit. License: MIT (verified
/// 2026-10-02 — distill/teacher use license-clear with attribution).
pub(super) fn run_bekko_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    run_jsonl_oracle_lane(
        "bekko",
        "BEKKO_LANE_SCRIPT",
        "scripts/bekko_lane.py",
        "BEKKO_PYTHON",
        "BEKKO_PY_DEVICE",
        "cpu",
        "the lane is opt-in measurement tooling; the venv needs torch + \
         transformers + sentence-transformers (the card's runtime pins; MIT \
         license — verified 2026-10-02)",
        suite,
        laya_max_questions,
        leak_flags,
    )
}

/// The AgentJev comparison lane (Issue 025 amendment 4 / `.issues/027`):
/// their `jev_service` served on loopback, measured over HTTP — the
/// MEASURE-vs-SERVE split (their Apache-2.0 Python service runs on the
/// 4090, our Rust harness measures; the riir-infer port stays DEFERRED in
/// `.issues/025`). Same cases, their contract (`src/lanes/agentjev.rs`
/// pins the mapping), the same metrics tail via
/// [`assemble_laya_lane_result`]. Latency = client round-trip per case
/// (their `usage.wall_ms` recorded beside it — the 025 amendment-4 law).
///
/// Env: `AGENTJEV_SERVE_URL` (default `http://127.0.0.1:8149`). The
/// `/api/info` handshake advertises the loaded checkpoint + device; the
/// lane stamps THOSE into the row, never a hardcoded name.
pub(super) fn run_agentjev_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::agentjev::AgentJevLane;

    let t_start = Instant::now();
    let lane = AgentJevLane::default();
    let (model, device) = lane.info()?;
    eprintln!("    [agentjev] service up: {model} on {device}");

    // WARMUP (the clm lane's cold-start law): one FIXED throwaway request
    // (never a case's — no cache pollution of measured latencies) absorbs
    // the service's cold path before the first measured case.
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm)
            .map_err(|e| format!("agentjev warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N agentjev row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut server_wall_ms: f64 = 0.0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, wall_ms) = lane
            .decide(case)
            .map_err(|e| format!("agentjev lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        if let Some(w) = wall_ms {
            server_wall_ms += w;
        }

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("agentjev determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("agentjev determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [agentjev] server-side wall sum: {server_wall_ms:.0} ms");

    Ok(assemble_laya_lane_result(
        "agentjev",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The Cloudflare Clef comparison lane (plan 011 Phase A, from
/// `.research/005_…`): their decision models served on Workers AI behind
/// an operator-run loopback TLS forwarder, measured over HTTP — the
/// agentjev lane's shape (same cases, the Jev-shaped wire via
/// `src/lanes/clef.rs`, the same metrics tail through
/// [`assemble_laya_lane_result`]). Latency = client round-trip per case,
/// INCLUDING the forwarder hop (plan 011 A0: disclosed, part of the
/// claim).
///
/// Env contract: `CLEF_SERVE_URL` (default `http://127.0.0.1:8791`),
/// `CLEF_ACCOUNT_ID` or `CLEF_RUN_PATH` (missing creds refuse at
/// construction — the A5 gate), `CLEF_API_TOKEN`, `CLEF_MODEL`,
/// `CLEF_TIMEOUT_MS`, and the spend ceiling `CLEF_SMOKE_MAX_CASES`
/// (default 50 cases; `CLEF_ALLOW_UNCAPPED=1` overrides — pricing is
/// undisclosed, no unbounded spend).
pub(super) fn run_clef_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::clef::ClefArm;

    let t_start = Instant::now();
    let lane = ClefArm::from_env()?;
    let (model, posture) = lane.info()?;
    eprintln!("    [clef] service up: {model} via {posture}");

    // WARMUP (the clm lane's cold-start law): one FIXED throwaway request
    // (never a case's — no cache pollution of measured latencies) absorbs
    // the hosted route's cold path before the first measured case.
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm)
            .map_err(|e| format!("clef warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N clef row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // The spend ceiling (plan 011 A6): pricing is undisclosed, so a run
    // above the default smoke cap REFUSES unless explicitly overridden.
    // LOCAL-arm law: the ceiling is an HTTP-arm law (undisclosed HOSTED
    // pricing) — the local arm runs on our own hardware, so it is
    // exempt; the full-N frozen splits stay the T3.2 posture.
    if !lane.is_local() {
        let max_cases = std::env::var(crate::lanes::clef::MAX_CASES_ENV)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(crate::lanes::clef::DEFAULT_MAX_CASES);
        let uncapped = std::env::var(crate::lanes::clef::UNCAPPED_ENV).is_ok_and(|v| v == "1");
        if cases.len() > max_cases && !uncapped {
            return Err(format!(
                "refusing: {} cases exceeds the {CAP} spend ceiling ({}, default) — \
                 pricing is undisclosed; raise {CAP} or set {UNC}=1 to run uncapped",
                cases.len(),
                crate::lanes::clef::DEFAULT_MAX_CASES,
                CAP = crate::lanes::clef::MAX_CASES_ENV,
                UNC = crate::lanes::clef::UNCAPPED_ENV,
            ));
        }
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, _wall) = lane
            .decide(case)
            .map_err(|e| format!("clef lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("clef determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("clef determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }

    Ok(assemble_laya_lane_result(
        "clef",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}
