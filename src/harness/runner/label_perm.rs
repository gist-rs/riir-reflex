//! riir-train Issue 657 R8 — the label-PERMUTATION accuracy probe (the
//! runner half).
//!
//! Lane-agnostic: every probed lane answers the SAME cases through its OWN
//! production decide path (the [`crate::harness::permutation::ChoiceOracle`]
//! seam) — the identity presentation and the σ-renamed presentation — and
//! the probe reads the PAIRED per-item accuracy delta (the
//! [`super::robust_ab::lane_degradation`] statistic: mean per-item delta +
//! 95% CI over the items both arms answered). The pure math lives in
//! [`crate::harness::label_perm`]; this module owns the lanes, the seeds,
//! the output record, and the render.
//!
//! Postures (owner binding, 657 R8):
//! - **modelless (DEFAULT)** — full-consistent relabel: train docs + engine
//!   domains + question keys through the SAME σ (the invariance arm;
//!   expected ≈ 0 delta — a large delta IS the finding).
//! - **`--label-perm-keep-corpus`** — by-name-binding control: questions
//!   renamed, corpus + domains UNRENAMED. Renamed keys miss every by-name
//!   domain, the route terms disarm, and the question runs the drafter-only
//!   scorer (the weakest-scorer fallback) — a STRUCTURAL collapse, disclosed
//!   in the record, never a gate.
//! - **laya** (feature `laya-riir`, honors `--skip-laya`) — keys renamed
//!   through σ, descriptions INTACT (the lane exposes them; the head was
//!   trained on the original names — that OOD-ness is the measurement).
//!   One checkpoint per suite (the primary), one agent load, both arms.
//!
//! Seeds: per-suite base = [`crate::harness::label_perm::LABEL_PERM_SEED`]
//! ^ FNV-1a(suite); the probe runs base, base+1, base+2 — per-suite and
//! per-seed delta rows, byte-reproducible.
//!
//! REPORT-ONLY: there is no engine invariant being asserted here (unlike
//! the perm-probe's control), so no exit-code gate — the delta IS the
//! measurement. Identity-arm absences (laya bucket skips are state-side,
//! rename-independent) shrink the paired join and are named in the record.

use super::perm_probe::stride_indices;
use super::*;
use crate::harness::item_dump::ItemOutcome;
use crate::harness::label_perm::{
    LABEL_PERM_SEED, label_bijection, relabel_labels, relabel_train_docs, rename_case,
    single_question_case, suite_seed_base,
};
use crate::harness::permutation::ChoiceOracle;

/// Cases sampled per suite by deterministic stride over the suite's
/// choice-carrying cases in dataset order. `0` (the default) = the FULL
/// eval slice — accuracy deltas want the whole population, and the paired
/// join keeps the cost at one extra pass per seed.
pub const DEFAULT_LABEL_PERM_MAX_CASES: usize = 0;
/// The default suite set (the owner's four; `--suites` overrides).
const DEFAULT_LABEL_PERM_SUITES: &[&str] = &["banking77", "ag_news", "xnli_en", "emotion"];
/// The spec-of-record line every record carries (the run meta AND the
/// markdown render share this const, so neither can lose the citation).
const SPEC_OF_RECORD: &str = "riir-train .issues/657 R8 — label-STRING bijection \
                              probe (options+gold follow σ). NOT Bench 119 \
                              (formatting-flip): Bench 119's axis is formatting \
                              churn of the state text; this axis is the label \
                              identity itself.";
/// The rename-rule line (shared by the meta and the tests' fixture).
const RENAME_RULE: &str = "Sattolo derangement over the suite's option-key \
                           alphabet (every label moves); option ORDER preserved, \
                           criteria keys replaced in place, descriptions intact; \
                           gold rides by position (idx unchanged, string renamed \
                           through σ); per-suite base = seed ^ FNV-1a(suite), \
                           seeds base+0/+1/+2";

/// The probe's knobs (the CLI's `--label-perm-max-cases` /
/// `--label-perm-keep-corpus`).
#[derive(Debug, Clone)]
pub struct LabelPermOptions {
    /// Cases sampled per suite; `0` = the full eval slice.
    pub max_cases: usize,
    /// The by-name-binding control posture: rename the questions ONLY
    /// (corpus + engine domains stay original). Disclosed, never a gate.
    pub keep_corpus: bool,
}

// ────────────────────────────────────────────────────────── the record ──

/// One seed's σ over one suite's alphabet, serialized for the record.
#[derive(Debug, Clone, Serialize)]
pub struct LabelPermSeedMap {
    pub seed_index: usize,
    pub seed: u64,
    /// The bijection (alphabet order — BTreeMap iteration is sorted, so the
    /// record is byte-stable).
    pub mapping: Vec<(String, String)>,
}

/// One lane × suite × seed cell: the paired accuracy delta vs identity.
#[derive(Debug, Clone, Serialize)]
pub struct LabelPermLaneRow {
    pub lane: String,
    /// The arm's provenance (which corpus answered which presentation).
    pub model: String,
    pub seed_index: usize,
    /// Paired slots (both arms answered).
    pub n: usize,
    /// Slots the identity arm answered.
    pub n_identity: usize,
    /// Slots the renamed arm answered (bucket skips shrink the join).
    pub n_perm: usize,
    pub acc_identity: f64,
    pub acc_perm: f64,
    /// perm − identity (mean per-item paired delta; ≤ 0 = degrades).
    pub delta: f64,
    pub delta_lb95: f64,
    pub delta_ub95: f64,
    /// Paired items correct@identity → wrong@σ.
    pub flipped_to_wrong: usize,
    /// Paired items wrong@identity → correct@σ.
    pub flipped_to_right: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelPermSuiteReport {
    pub suite: String,
    /// Sampled choice slots (the probe's population before lane absences).
    pub n_slots: usize,
    /// The suite's option-key alphabet (σ's domain = its codomain).
    pub alphabet: Vec<String>,
    pub permutations: Vec<LabelPermSeedMap>,
    pub lanes: Vec<LabelPermLaneRow>,
    /// Loud absences: preparation failures, choice-less suites, lane
    /// failures, bucket-limited slots.
    pub skipped: Vec<String>,
    /// The slice-integrity summary line (None on in-process paths).
    pub slice_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelPermMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    /// `"modelless"` (full-consistent relabel) or
    /// `"modelless_keep_corpus"` (the by-name-binding control).
    pub posture: String,
    /// 0 = the full eval slice.
    pub max_cases: usize,
    pub keep_corpus: bool,
    /// The seed CONSTANT; per-suite bases are `seed ^ FNV-1a(suite)` and
    /// every suite record carries its absolute per-seed values.
    pub seed: u64,
    pub rename_rule: String,
    pub corpus_rule: String,
    pub spec_of_record: String,
    pub laya_device: Option<String>,
    pub box_start: crate::harness::box_state::BoxState,
    pub box_end: crate::harness::box_state::BoxState,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelPermOutput {
    pub meta: LabelPermMeta,
    pub suites: Vec<LabelPermSuiteReport>,
}

type Items = Vec<ItemOutcome>;

// ──────────────────────────────────────────────────────── the machinery ──

fn fresh_scratch() -> Scratch<EMBED_DIM> {
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(1);
    sc
}

/// Answer every case through the oracle and materialize per-item outcomes.
/// `bucket:`-prefixed errors are STATE-side skips (rename-independent) —
/// named and dropped, never fabricated; anything else fails the lane loud.
fn run_items(
    oracle: &mut dyn ChoiceOracle,
    lane: &'static str,
    model: &str,
    cases: &[SuiteCase],
    skips: &mut Vec<String>,
) -> Result<Items, String> {
    let mut items = Vec::with_capacity(cases.len());
    for (idx, case) in cases.iter().enumerate() {
        match oracle.answer(case) {
            Ok(a) => {
                let gold = case.gold[0].idx;
                let conf = a.probs.iter().copied().fold(0.0_f64, f64::max);
                items.push(ItemOutcome {
                    lane,
                    model: model.to_string(),
                    case_id: case.id.clone(),
                    case_idx: idx,
                    q_idx: 0,
                    kind: QKind::Choice.as_str().to_string(),
                    gold,
                    pick: a.pick,
                    ok: a.pick == gold,
                    conf,
                    conf_cal: None,
                    abstain_raw: None,
                    abstain_cal: None,
                    cause_cal: None,
                });
            }
            Err(e) if e.starts_with("bucket:") => {
                skips.push(format!(
                    "{}: {e} (bucket-limited — state-side, rename-independent)",
                    case.id
                ));
            }
            Err(e) => return Err(format!("{}: {e}", case.id)),
        }
    }
    Ok(items)
}

/// Paired flip counts over the items BOTH sides answered (the same join
/// [`super::robust_ab::lane_degradation`] pairs with).
fn flip_counts(clean: &[ItemOutcome], perm: &[ItemOutcome]) -> (usize, usize) {
    let mut c: BTreeMap<(&str, usize), bool> = BTreeMap::new();
    for i in clean {
        c.insert((i.case_id.as_str(), i.q_idx), i.ok);
    }
    let (mut wrong, mut right) = (0usize, 0usize);
    for i in perm {
        if let Some(&okc) = c.get(&(i.case_id.as_str(), i.q_idx)) {
            match (okc, i.ok) {
                (true, false) => wrong += 1,
                (false, true) => right += 1,
                _ => {}
            }
        }
    }
    (wrong, right)
}

/// One lane's paired row: the house degradation statistic + the flip counts.
fn assemble_row(
    lane: &str,
    model: &str,
    seed_index: usize,
    identity_items: &[ItemOutcome],
    perm_items: &[ItemOutcome],
) -> LabelPermLaneRow {
    let deg = robust_ab::lane_degradation(lane, identity_items, perm_items);
    let (flipped_to_wrong, flipped_to_right) = flip_counts(identity_items, perm_items);
    LabelPermLaneRow {
        lane: deg.lane,
        model: model.to_string(),
        seed_index,
        n: deg.n,
        n_identity: identity_items.len(),
        n_perm: perm_items.len(),
        acc_identity: deg.acc_clean,
        acc_perm: deg.acc_corrupted,
        delta: deg.delta,
        delta_lb95: deg.delta_lb95,
        delta_ub95: deg.delta_ub95,
        flipped_to_wrong,
        flipped_to_right,
    }
}

// ─────────────────────────────────────────────────────────────── lanes ──

/// One suite's slot-level probe inputs, shared by every lane (the runner's
/// argument-bundle pattern — [`ModellessInput`]'s law, one layer down).
struct SlotContext<'a> {
    /// The identity single-question cases (shared by every lane; both arms
    /// key on the SAME synthetic ids).
    identity_cases: &'a [SuiteCase],
    /// The per-seed bijections over the suite's option-key alphabet.
    sigmas: &'a [(usize, u64, BTreeMap<String, String>)],
    popts: &'a LabelPermOptions,
    kn_route: bool,
}

/// The modelless cell for one suite: the deployed engine build
/// (`fit_posture_inner` + `build_engine_with`, every selection lever pinned
/// off — the perm-probe's plain-posture law), the identity arm answered
/// ONCE, then one renamed arm per seed. The identity fit's EngineConfig and
/// effective cap are REUSED by every renamed build so the ONLY variable
/// across arms is the label identity itself.
fn label_perm_modelless<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    ctx: &SlotContext<'_>,
    lanes: &mut Vec<LabelPermLaneRow>,
    skipped: &mut Vec<String>,
) -> Result<(), String> {
    let SlotContext {
        identity_cases,
        sigmas,
        popts,
        kn_route,
    } = *ctx;
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        gate_fit_selection: false,
        gate_distance_only: false,
        gate_fit_calibrated: true,
        gate_rho: None,
        kn_route,
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
    // The probe pins the PLAIN deployed posture (the perm-probe's guard): a
    // nonzero scale here means a selection lever reached the fitted config.
    #[cfg(feature = "option_cond")]
    if default_cfg.oc_scale > 0.0 {
        return Err(format!(
            "the fitted config arms option-conditioned tables (oc_scale {}) — \
             the probe pins the plain posture",
            default_cfg.oc_scale
        ));
    }
    #[cfg(feature = "nb_scope")]
    if default_cfg.nb_scale > 0.0 {
        return Err(format!(
            "the fitted config arms count tables (nb_scale {}) — the probe \
             pins the plain posture",
            default_cfg.nb_scale
        ));
    }
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };
    let (mut identity_engine, fallbacks) = build_engine_with::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        cfg.clone(),
        &[],
    )?;
    if !fallbacks.is_empty() {
        eprintln!(
            "  [issue-039 corpus guard] {}: {} option label(s) with NO train docs \
             → self-doc fallback: {}",
            spec.name,
            fallbacks.len(),
            fallbacks.join(", ")
        );
    }
    // The identity arm — answered ONCE, paired against every seed.
    let mut identity_skips = Vec::new();
    let identity_items = {
        let mut oracle = super::perm_probe::ModellessOracle {
            engine: &mut identity_engine,
            sc: fresh_scratch(),
        };
        run_items(
            &mut oracle,
            "modelless",
            "modelless",
            identity_cases,
            &mut identity_skips,
        )?
    };
    skipped.extend(identity_skips);
    eprintln!(
        "    label-perm[modelless]: identity arm {} item(s) · posture {}",
        identity_items.len(),
        if popts.keep_corpus {
            "keep-corpus control"
        } else {
            "full-consistent relabel"
        }
    );
    for &(seed_index, _seed, ref sigma) in sigmas {
        let renamed_cases: Vec<SuiteCase> = identity_cases
            .iter()
            .map(|c| rename_case(c, sigma))
            .collect();
        let mut seed_skips = Vec::new();
        let (perm_items, model): (Items, String) = if popts.keep_corpus {
            // The control: the IDENTITY corpus answers the RENAMED keys. By
            // name-resolution misses on every option, the route terms
            // disarm, and the question runs the drafter-only scorer.
            let items = {
                let mut oracle = super::perm_probe::ModellessOracle {
                    engine: &mut identity_engine,
                    sc: fresh_scratch(),
                };
                run_items(
                    &mut oracle,
                    "modelless",
                    "modelless",
                    &renamed_cases,
                    &mut seed_skips,
                )?
            };
            (
                items,
                "identity corpus · renamed keys (by-name misses → drafter-only)".to_string(),
            )
        } else {
            // The invariance arm: corpus + domains + keys through the SAME σ.
            let (renamed_train, untouched) = relabel_train_docs(&prepared.train, sigma);
            if untouched != 0 {
                eprintln!(
                    "    label-perm[modelless]: {untouched} corpus doc(s) outside σ — \
                     kept verbatim (disclosed)"
                );
            }
            let renamed_labels = relabel_labels(&prepared.labels, sigma);
            let (mut renamed_engine, fb) = build_engine_with::<N>(
                spec.name,
                &renamed_train,
                &renamed_labels,
                effective_cap,
                cfg.clone(),
                &[],
            )?;
            if !fb.is_empty() {
                eprintln!(
                    "  [issue-039 corpus guard] {} (σ): {} option label(s) with NO train \
                     docs → self-doc fallback: {}",
                    spec.name,
                    fb.len(),
                    fb.join(", ")
                );
            }
            let items = {
                let mut oracle = super::perm_probe::ModellessOracle {
                    engine: &mut renamed_engine,
                    sc: fresh_scratch(),
                };
                run_items(
                    &mut oracle,
                    "modelless",
                    "modelless",
                    &renamed_cases,
                    &mut seed_skips,
                )?
            };
            (items, "full-consistent σ (corpus+domains+keys)".to_string())
        };
        skipped.extend(seed_skips);
        let row = assemble_row(
            "modelless",
            &model,
            seed_index,
            &identity_items,
            &perm_items,
        );
        eprintln!(
            "    label-perm[modelless] seed#{seed_index}: acc {:.4} → {:.4} \
             (Δ {:+.4} [{:+.4}, {:+.4}]) · flips −{}/+{} · n {}",
            row.acc_identity,
            row.acc_perm,
            row.delta,
            row.delta_lb95,
            row.delta_ub95,
            row.flipped_to_wrong,
            row.flipped_to_right,
            row.n
        );
        lanes.push(row);
    }
    Ok(())
}

/// The laya cell for one suite — its PRIMARY checkpoint only (english for
/// every default-set suite), ONE agent load answering both arms; bucket
/// skips shrink the paired join and are named.
#[cfg(feature = "laya-riir")]
fn label_perm_laya(
    spec: &SuiteSpec,
    identity_cases: &[SuiteCase],
    sigmas: &[(usize, u64, BTreeMap<String, String>)],
    lanes: &mut Vec<LabelPermLaneRow>,
    skipped: &mut Vec<String>,
) -> Result<(), String> {
    use crate::laya::config::Checkpoint;
    let ck_name = laya_checkpoints_for(spec.name)[0];
    let ck = match ck_name {
        "english" => Checkpoint::English,
        "multilingual" => Checkpoint::Multilingual,
        "typed" => Checkpoint::TypedDecisions,
        other => return Err(format!("unknown checkpoint {other}")),
    };
    let agent = load_laya_agent(ck_name, ck)?;
    let device = crate::laya::riir::agent::DeviceKind::from_env()
        .map(|d| format!("{d:?}"))
        .unwrap_or_else(|_| "default".to_string());
    eprintln!("    label-perm[laya/{ck_name}]: device {device}");
    let mut oracle = super::perm_probe::LayaOracle { agent };
    let mut skips = Vec::new();
    let identity_items = run_items(&mut oracle, "laya", ck_name, identity_cases, &mut skips)?;
    eprintln!(
        "    label-perm[laya/{ck_name}]: identity arm {} item(s)",
        identity_items.len()
    );
    for &(seed_index, _seed, ref sigma) in sigmas {
        let renamed_cases: Vec<SuiteCase> = identity_cases
            .iter()
            .map(|c| rename_case(c, sigma))
            .collect();
        let items = run_items(&mut oracle, "laya", ck_name, &renamed_cases, &mut skips)?;
        let row = assemble_row(
            &format!("laya/{ck_name}"),
            &format!("laya/{ck_name} (keys renamed through σ; descriptions intact)"),
            seed_index,
            &identity_items,
            &items,
        );
        eprintln!(
            "    label-perm[laya/{ck_name}] seed#{seed_index}: acc {:.4} → {:.4} \
             (Δ {:+.4} [{:+.4}, {:+.4}]) · flips −{}/+{} · n {}",
            row.acc_identity,
            row.acc_perm,
            row.delta,
            row.delta_lb95,
            row.delta_ub95,
            row.flipped_to_wrong,
            row.flipped_to_right,
            row.n
        );
        lanes.push(row);
    }
    skipped.extend(skips);
    Ok(())
}

// ───────────────────────────────────────────────────────── the suite run ──

fn empty_suite_report(spec: &SuiteSpec, skipped: Vec<String>) -> LabelPermSuiteReport {
    LabelPermSuiteReport {
        suite: spec.name.to_string(),
        n_slots: 0,
        alphabet: Vec::new(),
        permutations: Vec::new(),
        lanes: Vec::new(),
        skipped,
        slice_summary: None,
    }
}

fn label_perm_suite(
    spec: &SuiteSpec,
    dir: &Path,
    popts: &LabelPermOptions,
    laya_enabled: bool,
    kn_route: bool,
) -> LabelPermSuiteReport {
    let mut lanes: Vec<LabelPermLaneRow> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    eprintln!("═══ suite {} (label-perm) ═══", spec.name);
    let prepared = match prepare(spec, dir) {
        Ok(p) => p,
        Err(e) => {
            skipped.push(format!("prepare: {e}"));
            return empty_suite_report(spec, skipped);
        }
    };
    let slice_summary = prepared.slices.as_ref().map(|s| s.summary_line());
    let alphabet = prepared.labels.clone();
    // Choice slots: the sampled case indices × their choice questions.
    let choice_cases: Vec<usize> = prepared
        .suite
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c.questions.iter().any(|q| q.kind == QKind::Choice))
        .map(|(i, _)| i)
        .collect();
    if choice_cases.is_empty() {
        skipped.push(
            "no choice questions — the probe is choice-only (score levels are \
             ordinal and noul's [false, true] order is semantic)"
                .to_string(),
        );
        return LabelPermSuiteReport {
            suite: spec.name.to_string(),
            n_slots: 0,
            alphabet,
            permutations: Vec::new(),
            lanes,
            skipped,
            slice_summary,
        };
    }
    let sampled: Vec<usize> = if popts.max_cases == 0 {
        (0..choice_cases.len()).collect()
    } else {
        stride_indices(choice_cases.len(), popts.max_cases)
    };
    let slots: Vec<(usize, usize)> = sampled
        .iter()
        .flat_map(|&si| {
            let ci = choice_cases[si];
            let case = &prepared.suite.cases[ci];
            (0..case.questions.len()).filter_map(move |qi| {
                (case.questions[qi].kind == QKind::Choice).then_some((ci, qi))
            })
        })
        .collect();
    eprintln!(
        "    label-perm: {} choice slot(s) over {} sampled case(s) · 3 seeds",
        slots.len(),
        sampled.len()
    );
    // The per-seed bijections over the suite's option-key alphabet.
    let base = suite_seed_base(spec.name);
    let sigmas: Vec<(usize, u64, BTreeMap<String, String>)> = (0..3usize)
        .map(|k| {
            let seed = base + k as u64;
            (k, seed, label_bijection(&prepared.labels, seed))
        })
        .collect();
    let permutations = sigmas
        .iter()
        .map(|(k, seed, m)| LabelPermSeedMap {
            seed_index: *k,
            seed: *seed,
            mapping: m.iter().map(|(a, b)| (a.clone(), b.clone())).collect(),
        })
        .collect();
    // The identity single-question cases (shared by every lane; both arms
    // key on the SAME synthetic ids so the paired join lines up).
    let identity_cases: Vec<SuiteCase> = slots
        .iter()
        .enumerate()
        .map(|(idx, &(ci, qi))| {
            single_question_case(&prepared.suite.cases[ci], qi)
                .unwrap_or_else(|e| panic!("label-perm slot {idx}: {e}"))
        })
        .collect();

    // The control (modelless) — the const-generic dispatch, the run()'s
    // table (56 = massive_intent_en's deciding front, the run_phases arm).
    let ctx = SlotContext {
        identity_cases: &identity_cases,
        sigmas: &sigmas,
        popts,
        kn_route,
    };
    macro_rules! dispatch {
        ($n:literal) => {
            label_perm_modelless::<$n>(spec, &prepared, &ctx, &mut lanes, &mut skipped)
        };
    }
    let modelless = match prepared.labels.len() {
        2 => dispatch!(2),
        3 => dispatch!(3),
        4 => dispatch!(4),
        5 => dispatch!(5),
        6 => dispatch!(6),
        7 => dispatch!(7),
        8 => dispatch!(8),
        10 => dispatch!(10),
        15 => dispatch!(15),
        56 => dispatch!(56),
        59 => dispatch!(59),
        77 => dispatch!(77),
        842 => dispatch!(842),
        n => Err(format!(
            "no engine instantiation for {n} domains — extend the dispatch \
             table in runner/label_perm.rs"
        )),
    };
    if let Err(e) = modelless {
        skipped.push(format!("modelless: {e}"));
    }

    // The laya lane (feature-gated, primary checkpoint).
    #[cfg(feature = "laya-riir")]
    if laya_enabled
        && let Err(e) = label_perm_laya(spec, &identity_cases, &sigmas, &mut lanes, &mut skipped)
    {
        skipped.push(format!("laya: {e}"));
    }
    #[cfg(not(feature = "laya-riir"))]
    let _ = laya_enabled;

    LabelPermSuiteReport {
        suite: spec.name.to_string(),
        n_slots: slots.len(),
        alphabet,
        permutations,
        lanes,
        skipped,
        slice_summary,
    }
}

/// Run the probe over the requested suites. Report-only: the run never
/// fails on a lane delta — the record carries every verdict-relevant number.
pub fn run_label_perm_probe(
    opts: &RunOptions,
    popts: &LabelPermOptions,
) -> Result<LabelPermOutput, String> {
    let box_start = crate::harness::box_state::capture();
    let laya_enabled = !opts.skip_laya;
    let laya_device: Option<String> = if cfg!(feature = "laya-riir") && laya_enabled {
        #[cfg(feature = "laya-riir")]
        {
            Some(
                crate::laya::riir::agent::DeviceKind::from_env()
                    .map(|d| format!("{d:?}"))
                    .unwrap_or_else(|_| "default".to_string()),
            )
        }
        #[cfg(not(feature = "laya-riir"))]
        {
            None
        }
    } else {
        None
    };
    let mut suites = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() {
            if !opts.suites.iter().any(|s| s == spec.name) {
                continue;
            }
        } else if spec.named_only || !DEFAULT_LABEL_PERM_SUITES.contains(&spec.name) {
            continue;
        }
        suites.push(label_perm_suite(
            spec,
            &opts.datasets_dir,
            popts,
            laya_enabled,
            opts.kn_route,
        ));
    }
    if suites.is_empty() {
        return Err(format!(
            "label-perm probe: no suite measured ({} request line(s)) — check \
             --suites against the registry",
            opts.suites.len()
        ));
    }
    let posture = if popts.keep_corpus {
        "modelless_keep_corpus"
    } else {
        "modelless"
    };
    let corpus_rule = if popts.keep_corpus {
        "keep_corpus control: questions + engine domains renamed, corpus docs \
         and centroids UNRENAMED — renamed keys miss every by-name domain, the \
         route terms disarm, the question runs the drafter-only scorer \
         (structural, DISCLOSED — never a gate)"
    } else {
        "full-consistent relabel: train docs + engine domains + question keys \
         through the SAME σ (the invariance arm; expected ≈ 0 delta — a large \
         delta IS the finding)"
    };
    Ok(LabelPermOutput {
        meta: LabelPermMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            posture: posture.to_string(),
            max_cases: popts.max_cases,
            keep_corpus: popts.keep_corpus,
            seed: LABEL_PERM_SEED,
            rename_rule: RENAME_RULE.to_string(),
            corpus_rule: corpus_rule.to_string(),
            spec_of_record: SPEC_OF_RECORD.to_string(),
            laya_device,
            box_start,
            box_end: crate::harness::box_state::capture(),
        },
        suites,
    })
}

/// The record's markdown render.
pub fn render_label_perm_markdown(out: &LabelPermOutput) -> String {
    let m = &out.meta;
    let mut s = String::new();
    s.push_str(
        "# Label-perm probe — seeded bijective label-STRING rename \
         (riir-train 657 R8)\n\n",
    );
    s.push_str(&format!(
        "- date {} · sha {} · host {}\n- datasets {} · posture {} · ≤{} \
         case(s)/suite (0 = full slice)\n- seed {} · per-suite base = seed ^ \
         FNV-1a(suite) · seeds base+0/+1/+2\n- laya device: {}\n",
        m.date_utc,
        m.git_sha.as_deref().unwrap_or("unknown"),
        m.host,
        m.datasets_dir,
        m.posture,
        m.max_cases,
        m.seed,
        m.laya_device.as_deref().unwrap_or("not probed"),
    ));
    for (name, rule) in [
        ("spec", &m.spec_of_record),
        ("rename", &m.rename_rule),
        ("corpus", &m.corpus_rule),
    ] {
        s.push_str(&format!("- {name}: {rule}\n"));
    }
    s.push_str(
        "\n| suite | lane | model | seed# | n | acc@id | acc@σ | Δ | 95% CI \
         | flips→wrong/→right |\n",
    );
    s.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for suite in &out.suites {
        if suite.lanes.is_empty() {
            s.push_str(&format!(
                "| {} | — | — | — | — | — | — | — | — | — |\n",
                suite.suite
            ));
            continue;
        }
        for l in &suite.lanes {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {:.4} | {:.4} | {:+.4} | [{:+.4}, \
                 {:+.4}] | {}/{} |\n",
                suite.suite,
                l.lane,
                l.model,
                l.seed_index,
                l.n,
                l.acc_identity,
                l.acc_perm,
                l.delta,
                l.delta_lb95,
                l.delta_ub95,
                l.flipped_to_wrong,
                l.flipped_to_right,
            ));
        }
    }
    for suite in &out.suites {
        s.push_str(&format!(
            "\n- {}: {} slot(s) · seeds [{:x}, {:x}, {:x}] · alphabet {}",
            suite.suite,
            suite.n_slots,
            suite
                .permutations
                .first()
                .map(|p| p.seed)
                .unwrap_or_default(),
            suite
                .permutations
                .get(1)
                .map(|p| p.seed)
                .unwrap_or_default(),
            suite
                .permutations
                .get(2)
                .map(|p| p.seed)
                .unwrap_or_default(),
            suite.alphabet.len(),
        ));
        if let Some(sl) = &suite.slice_summary {
            s.push_str(&format!(" · {sl}"));
        }
        s.push('\n');
        for sk in &suite.skipped {
            s.push_str(&format!("  - SKIP: {sk}\n"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::permutation::OrderingAnswer;
    use crate::harness::suites::{GoldAnswer, SuiteQuestion};

    fn sg_case(id: &str, keys: &[&str], gold: usize) -> SuiteCase {
        let mut m = serde_json::Map::new();
        for k in keys {
            m.insert((*k).to_string(), serde_json::Value::Null);
        }
        SuiteCase {
            id: id.to_string(),
            state: serde_json::Value::String("state".to_string()),
            questions: vec![SuiteQuestion {
                qid: "q".to_string(),
                kind: QKind::Choice,
                instructions: "pick".to_string(),
                criteria: serde_json::Value::Object(m),
            }],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    fn keys_of(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    /// The label-STRING memorizer: always picks the position of one fixed
    /// label string, wherever σ moved it. Under rename its four target-gold
    /// cases (gold at the ORIGINAL target position) all flip wrong — the
    /// degradation is guaranteed by the derangement, seed-independent.
    struct LabelMemorizingOracle {
        target: String,
    }
    impl ChoiceOracle for LabelMemorizingOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            let keys: Vec<String> = case.questions[0]
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            let pick = keys
                .iter()
                .position(|k| *k == self.target)
                .expect("target label present (σ maps the alphabet onto itself)");
            let mut probs = vec![0.025; keys.len()];
            probs[pick] = 0.9;
            Ok(OrderingAnswer { probs, pick })
        }
    }

    /// A position reader: always the first presented option. Order is the
    /// invariant under label rename → exactly zero delta.
    struct FixedPositionOracle;
    impl ChoiceOracle for FixedPositionOracle {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            let n = case.questions[0]
                .criteria
                .as_object()
                .map(serde_json::Map::len)
                .unwrap_or(0);
            let mut probs = vec![0.0; n];
            probs[0] = 1.0;
            Ok(OrderingAnswer { probs, pick: 0 })
        }
    }

    #[test]
    fn label_memorizing_oracle_degrades_under_rename() {
        let keys = keys_of(&["alpha", "beta", "gamma", "delta"]);
        // Golds skew hard toward the target label — the memorization shape.
        let golds = [0usize, 0, 0, 0, 1, 2];
        let cases: Vec<SuiteCase> = golds
            .iter()
            .enumerate()
            .map(|(i, &g)| sg_case(&format!("c{i}"), &["alpha", "beta", "gamma", "delta"], g))
            .collect();
        let sigma = label_bijection(&keys, suite_seed_base("t_mem"));
        let renamed: Vec<SuiteCase> = cases.iter().map(|c| rename_case(c, &sigma)).collect();
        let mut skips = Vec::new();
        let mut oracle = LabelMemorizingOracle {
            target: "alpha".to_string(),
        };
        let identity =
            run_items(&mut oracle, "modelless", "m", &cases, &mut skips).expect("identity pass");
        let perm =
            run_items(&mut oracle, "modelless", "m", &renamed, &mut skips).expect("renamed pass");
        assert!(skips.is_empty());
        let row = assemble_row("modelless", "memorizing", 0, &identity, &perm);
        // Identity: the four gold-0 cases hit "alpha" at position 0 → 4/6.
        assert!((row.acc_identity - 4.0 / 6.0).abs() < 1e-9);
        // Renamed: "alpha" moved (derangement) → the four gold-0 cases flip
        // wrong; at most the two other golds can flip right → ≤ 2/6.
        assert!(
            row.delta < 0.0,
            "memorizer must degrade, got Δ {}",
            row.delta
        );
        assert!(row.acc_perm < row.acc_identity);
        assert!(row.flipped_to_wrong >= 4);
    }

    #[test]
    fn fixed_position_oracle_has_exactly_zero_delta() {
        let keys = keys_of(&["w", "x", "y", "z"]);
        let cases: Vec<SuiteCase> = (0..4)
            .map(|i| sg_case(&format!("c{i}"), &["w", "x", "y", "z"], i))
            .collect();
        let sigma = label_bijection(&keys, suite_seed_base("t_pos"));
        let renamed: Vec<SuiteCase> = cases.iter().map(|c| rename_case(c, &sigma)).collect();
        let mut skips = Vec::new();
        let mut oracle = FixedPositionOracle;
        let identity = run_items(&mut oracle, "modelless", "m", &cases, &mut skips).expect("id");
        let perm = run_items(&mut oracle, "modelless", "m", &renamed, &mut skips).expect("perm");
        let row = assemble_row("modelless", "position", 0, &identity, &perm);
        assert_eq!(
            row.delta, 0.0,
            "order-preserving rename cannot move a position reader"
        );
        assert_eq!(row.acc_identity, row.acc_perm);
        assert_eq!((row.flipped_to_wrong, row.flipped_to_right), (0, 0));
    }

    struct BucketOnSkip;
    impl ChoiceOracle for BucketOnSkip {
        fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
            if case.id.contains("skip") {
                Err("bucket: laya seq 9999 > max bucket 512".to_string())
            } else {
                Ok(OrderingAnswer {
                    probs: vec![0.5, 0.5],
                    pick: case.gold[0].idx,
                })
            }
        }
    }

    #[test]
    fn run_items_skips_buckets_and_the_pair_join_shrinks() {
        let cases = vec![
            sg_case("c0", &["a", "b"], 0),
            sg_case("c1skip", &["a", "b"], 1),
            sg_case("c2", &["a", "b"], 0),
            sg_case("c3", &["a", "b"], 1),
        ];
        let mut skips = Vec::new();
        let mut oracle = BucketOnSkip;
        let items = run_items(&mut oracle, "modelless", "m", &cases, &mut skips).expect("pass");
        assert_eq!(
            items.len(),
            3,
            "the bucket case is named + dropped, never fabricated"
        );
        assert_eq!(skips.len(), 1);
        // A complete identity side vs the shrunken perm side pairs 3.
        let identity: Vec<ItemOutcome> = cases
            .iter()
            .map(|c| ItemOutcome {
                lane: "modelless",
                model: "m".to_string(),
                case_id: c.id.clone(),
                case_idx: 0,
                q_idx: 0,
                kind: "choice".to_string(),
                gold: c.gold[0].idx,
                pick: c.gold[0].idx,
                ok: true,
                conf: 1.0,
                conf_cal: None,
                abstain_raw: None,
                abstain_cal: None,
                cause_cal: None,
            })
            .collect();
        let row = assemble_row("modelless", "m", 0, &identity, &items);
        assert_eq!(row.n, 3, "the join pairs only the commonly-answered slots");
        assert_eq!((row.n_identity, row.n_perm), (4, 3));
    }

    #[test]
    fn output_serializes_and_render_carries_the_spec() {
        let alphabet = keys_of(&["a", "b", "c"]);
        let out = LabelPermOutput {
            meta: LabelPermMeta {
                date_utc: "2026-10-31T00:00:00Z".to_string(),
                git_sha: Some("deadbeef".to_string()),
                host: "test".to_string(),
                datasets_dir: ".raw/datasets".to_string(),
                posture: "modelless".to_string(),
                max_cases: 0,
                keep_corpus: false,
                seed: LABEL_PERM_SEED,
                rename_rule: RENAME_RULE.to_string(),
                corpus_rule: "corpus".to_string(),
                spec_of_record: SPEC_OF_RECORD.to_string(),
                laya_device: None,
                box_start: crate::harness::box_state::capture(),
                box_end: crate::harness::box_state::capture(),
            },
            suites: vec![LabelPermSuiteReport {
                suite: "ag_news".to_string(),
                n_slots: 2,
                alphabet: alphabet.clone(),
                permutations: vec![LabelPermSeedMap {
                    seed_index: 0,
                    seed: 1,
                    mapping: alphabet
                        .iter()
                        .cloned()
                        .zip(alphabet.iter().cloned())
                        .collect(),
                }],
                lanes: vec![LabelPermLaneRow {
                    lane: "modelless".to_string(),
                    model: "full-consistent σ".to_string(),
                    seed_index: 0,
                    n: 2,
                    n_identity: 2,
                    n_perm: 2,
                    acc_identity: 0.5,
                    acc_perm: 0.5,
                    delta: 0.0,
                    delta_lb95: 0.0,
                    delta_ub95: 0.0,
                    flipped_to_wrong: 0,
                    flipped_to_right: 0,
                }],
                skipped: Vec::new(),
                slice_summary: None,
            }],
        };
        let json = serde_json::to_string_pretty(&out).expect("label-perm serialize");
        assert!(json.contains("permutations"));
        assert!(!json.contains("modelless_keep_corpus"));
        let md = render_label_perm_markdown(&out);
        assert!(md.contains("657 R8"));
        assert!(md.contains("NOT Bench 119"));
        assert!(md.contains("acc@id"));
        assert!(md.contains("full-consistent σ"));
    }

    #[test]
    fn default_label_perm_suites_are_registered() {
        for name in DEFAULT_LABEL_PERM_SUITES {
            assert!(
                SUITES.iter().any(|s| s.name == *name),
                "{name} must be registered in SUITES"
            );
        }
    }
}
