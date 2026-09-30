//! Plan 426 T5 — the corpus A/B lane (the V5 gate).
//!
//! One frozen test read, two engine builds, ONE changed variable — the
//! corpus:
//! - **Arm A (gold-only):** exactly the published posture — the registry
//!   corpus pool at the registry per-label cap (gold docs first).
//! - **Arm B (+synth):** the same pool, plus the synth artifact's rows
//!   appended BEYOND the gold cap up to `extra_cap` per label (gold-first
//!   ordering is preserved inside each label, so the cap still saturates
//!   with gold everywhere gold is plentiful; thin labels — the N2
//!   targets — are where synth docs enter).
//!
//! The comparison is FORCED-PICK hard accuracy (the published convention:
//! thresholds/calibration steer abstention, never the pick — so no gate
//! fit runs here and the two arms differ ONLY in corpus content). The
//! gate is the arena's own statistic: paired LB95 of (synth − gold) over
//! per-case correctness > 0. C3: both arms are reported separately, next
//! to each other, forever. V6 rides the same read: both arms' p50/p99
//! latency — the modelless lane must stay sub-ms after corpus growth.
//!
//! Instrument aliveness (the ensemble-gate law): arm A must reproduce
//! the published massive number (0.7800) — it reads the same 300 frozen
//! rows through the same build path. A drift there means the instrument
//! is mis-built, and the verdict is void before it is read.
//!
//! G-ISO: harness-only; the serve path is untouched.

use std::collections::BTreeMap;
use std::path::Path;

use super::{
    ensemble::paired_lb95, eval_engine, git_sha, iso8601_utc, percentile_us, prepare, seat,
    RunOptions, SuiteSpec, SUITES,
};
use crate::embed::EMBED_DIM;
use crate::engine::{DecisionEngine, ExpertSpec};
use crate::harness::suites::{QKind, SuiteCase, TrainDoc};
use super::synth::load_synth_corpus;

/// The published massive modelless row (Bench 052 / the arena re-baseline
/// — the A/B's arm-A reproduction anchor; massive-only).
const PUBLISHED_MASSIVE_A0: f64 = 0.7800;

/// Per-suite A/B record.
#[derive(Debug, serde::Serialize)]
pub struct CorpusAbSuite {
    pub suite: String,
    pub synth_artifact: String,
    pub synth_rows_total: usize,
    pub synth_rows_in_scope: usize,
    pub gold_corpus_docs: usize,
    pub synth_corpus_docs: usize,
    pub acc_gold: f64,
    pub acc_synth: f64,
    pub delta_lb95: f64,
    /// Per-label flip counts (the N2 attribution view).
    pub flips: Vec<CorpusAbFlip>,
    pub b_only_cases: Vec<String>,
    pub a_only_cases: Vec<String>,
    pub lat_p50_gold_us: u64,
    pub lat_p99_gold_us: u64,
    pub lat_p50_synth_us: u64,
    pub lat_p99_synth_us: u64,
    pub determinism_ok: bool,
    /// The instrument-aliveness note (the 0.7800 reproduction).
    pub aliveness: String,
    pub verdict: String,
    pub seconds: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct CorpusAbFlip {
    pub label: String,
    pub gained: usize,
    pub lost: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct CorpusAbMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub datasets_dir: String,
    pub corpus_cap_per_label: usize,
    pub synth_extra_cap: usize,
    pub read_rule: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct CorpusAbOutput {
    pub meta: CorpusAbMeta,
    pub suites: Vec<CorpusAbSuite>,
}

/// Flip tallies per gold label (gained = synth-only correct, lost =
/// gold-only correct).
#[derive(Default)]
struct Flip {
    gained: usize,
    lost: usize,
}

/// The eval case's gold LABEL string (the per-label flip attribution).
fn gold_label_of_case(case: &SuiteCase) -> String {
    let Some(q) = case.questions.first() else {
        return String::new();
    };
    let keys: Vec<String> = match q.kind {
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
        QKind::Score => (0..q.criteria.as_array().map_or(0, Vec::len))
            .map(|i| i.to_string())
            .collect(),
        QKind::Noul => vec![],
    };
    keys.get(case.gold[0].idx).cloned().unwrap_or_default()
}

/// The A/B's per-label spec builder: gold docs first (up to `cap`), then
/// the label's synth docs (up to `extra_cap`) — the self-doc fallback law
/// mirrors `specs_from_pool` (a label with neither gold nor synth docs
/// falls back to its own label text, disclosed). When the seat posture
/// arms the count tables (`nb_sets`), each spec carries them — arm A's
/// tables read the uncapped pool ([`nb_doc_sets`]); arm B's read the same
/// pool PLUS the synth docs (the corpus extension, tables included).
#[allow(clippy::too_many_arguments)]
fn arm_specs(
    train: &[TrainDoc],
    extra: &[TrainDoc],
    labels: &[String],
    cap: usize,
    extra_cap: usize,
    nb_sets: Option<&[Vec<String>]>,
) -> (Vec<ExpertSpec>, Vec<String>, usize) {
    // Delegated to the seat's shared constructor (Plan 426 T6): the seat
    // and this lane must not be able to drift apart — the V5 measurement
    // and the served seat are the SAME build by construction now.
    super::specs_corpus_extended(train, extra, labels, cap, extra_cap, nb_sets)
}

/// Run the corpus A/B for the synth artifact's suite. `opts.suites` (when
/// non-empty) must agree with the artifact's suite — a mismatch is a loud
/// refusal, never a silent suite swap.
pub fn run_corpus_ab(
    opts: &RunOptions,
    synth_path: &Path,
    extra_cap: usize,
) -> Result<CorpusAbOutput, String> {
    let (header, docs, _digest) = load_synth_corpus(synth_path)?;
    let Some(spec) = SUITES.iter().find(|s| s.name == header.suite) else {
        return Err(format!(
            "artifact suite {:?} is not a registered suite",
            header.suite
        ));
    };
    if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
        return Err(format!(
            "--corpus-ab artifact suite {:?} is not in --suites {:?} — pass one story",
            spec.name, opts.suites
        ));
    }
    let prepared = prepare(spec, &opts.datasets_dir)?;
    let labels = &prepared.labels;
    let n_labels = labels.len();
    let in_scope: Vec<TrainDoc> = docs
        .iter()
        .filter(|d| labels.iter().any(|l| l == &d.label))
        .cloned()
        .collect();
    let dropped = docs.len() - in_scope.len();
    if dropped > 0 {
        eprintln!(
            "  [corpus-ab {}] {} synth row(s) outside the engine's {}-label universe — \
             dropped (counted, never silent)",
            spec.name, dropped, n_labels
        );
    }

    macro_rules! dispatch {
        ($n:literal) => {
            corpus_ab_suite::<$n>(spec, &prepared, &in_scope, docs.len(), synth_path, extra_cap, opts)
        };
    }
    let suite_row = match n_labels {
        2 => dispatch!(2)?,
        3 => dispatch!(3)?,
        4 => dispatch!(4)?,
        5 => dispatch!(5)?,
        6 => dispatch!(6)?,
        7 => dispatch!(7)?,
        8 => dispatch!(8)?,
        59 => dispatch!(59)?,
        77 => dispatch!(77)?,
        n => {
            return Err(format!(
                "corpus-ab: no engine instantiation for {n} domains — extend the dispatch \
                 table in runner.rs (the run() law)"
            ));
        }
    };
    Ok(CorpusAbOutput {
        meta: CorpusAbMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            datasets_dir: opts.datasets_dir.display().to_string(),
            corpus_cap_per_label: spec.corpus_cap_per_label,
            synth_extra_cap: extra_cap,
            read_rule: "one frozen test read; two engine builds differing ONLY in corpus \
                        content; forced-pick hard accuracy; paired LB95 (synth − gold) > 0 \
                        is the V5 gate; both arms reported (C3)",
        },
        suites: vec![suite_row],
    })
}

fn corpus_ab_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &super::Prepared,
    synth_docs: &[TrainDoc],
    synth_rows_total: usize,
    synth_path: &Path,
    extra_cap: usize,
    opts: &RunOptions,
) -> Result<CorpusAbSuite, String> {
    let t_start = std::time::Instant::now();
    let labels = &prepared.labels;
    let cap = spec.corpus_cap_per_label;

    // The seat posture (the anchor-void fix, 61d4a94d's filed repair): arm A
    // is the DEPLOYED build — the same `fit_posture` prologue the seat and
    // `run_modelless` use, selected knobs included (massive: nb_scale 4.0 +
    // the fitted fused-gate thresholds) — never `EngineConfig::default()`.
    // The default posture measured gold 0.6100 against the published 0.7800
    // anchor and voided the lane's own A/B (bench 089). Both arms share ONE
    // fitted posture: the A/B varies ONLY the corpus content.
    let seat = seat::prepare_seat(spec.name, &opts.datasets_dir)?;
    let posture = seat::fit_posture::<N>(
        spec.name,
        &seat,
        &seat::PostureKnobs {
            head_select: opts.head_select,
            nb_select: opts.nb_select,
            #[cfg(feature = "option_cond")]
            oc_select: opts.oc_select,
            #[cfg(feature = "nb_ridge")]
            ridge_select: opts.ridge_select,
            genome_select: opts.genome_select,
            genome_accept_margin: opts.genome_accept_margin,
            cal_select_caps: opts.cal_select_caps.clone(),
        },
    )?;
    if posture.cfg.oc_scale > 0.0 {
        return Err(format!(
            "corpus-ab {}: the seat posture arms the option-conditioned tables — the \\
             +synth corpus arm cannot carry them; the lane refuses rather than mis-build",
            spec.name
        ));
    }
    let cfg = posture.cfg.clone();
    let effective_cap = posture.effective_cap;
    let _ = cap;

    // Arm A: the seat build at the fitted posture (gold pool, the selected
    // cap) — the deployed engine, byte-equal to run()'s.
    let (mut eng_a, fb_a) =
        seat::build_seat_engine::<N>(spec.name, &seat, effective_cap, cfg.clone())?;
    if !fb_a.is_empty() {
        eprintln!(
            "  [corpus-ab {}] gold arm: {} label(s) with NO pool docs → self-doc fallback",
            spec.name,
            fb_a.len()
        );
    }
    // Arm B: the SAME posture, corpus = gold cap + synth beyond it — the
    // retrieval specs AND the count tables (arm A's tables read the
    // uncapped pool; arm B's read that pool plus the synth docs — the
    // corpus extension is the only difference between the arms).
    #[cfg(feature = "nb_scope")]
    let nb_sets_b = (cfg.nb_scale > 0.0)
        .then(|| super::nb_doc_sets(&prepared.train, labels, synth_docs));
    #[cfg(not(feature = "nb_scope"))]
    let nb_sets_b: Option<Vec<Vec<String>>> = None;
    let (specs_b, fb_b, extra_used) = arm_specs(
        &prepared.train,
        synth_docs,
        labels,
        effective_cap,
        extra_cap,
        nb_sets_b.as_deref(),
    );
    let mut eng_b = DecisionEngine::<N, EMBED_DIM>::build_specs(specs_b, cfg)
        .map_err(|e| format!("engine build ({} +synth): {e}", spec.name))?;
    if !fb_b.is_empty() {
        eprintln!(
            "  [corpus-ab {}] synth arm: {} label(s) with NO docs at all → self-doc fallback",
            spec.name,
            fb_b.len()
        );
    }

    let (eval_a, lat_a) =
        eval_engine(&mut eng_a, &prepared.suite.cases, &prepared.state_strs, true)?;
    let (eval_b, lat_b) =
        eval_engine(&mut eng_b, &prepared.suite.cases, &prepared.state_strs, true)?;

    // Per-case forced-pick correctness (single-question suites only — the
    // synth lane's own scope law).
    let mut ok_a = Vec::with_capacity(prepared.suite.cases.len());
    let mut ok_b = Vec::with_capacity(prepared.suite.cases.len());
    let mut b_only_cases = Vec::new();
    let mut a_only_cases = Vec::new();
    let mut per_label: BTreeMap<String, Flip> = BTreeMap::new();
    for (ci, case) in prepared.suite.cases.iter().enumerate() {
        let a = eval_a.picks[ci][0] == case.gold[0].idx;
        let b = eval_b.picks[ci][0] == case.gold[0].idx;
        ok_a.push(a);
        ok_b.push(b);
        let gold_label = gold_label_of_case(case);
        match (b, a) {
            (true, false) => {
                b_only_cases.push(case.id.clone());
                per_label.entry(gold_label).or_default().gained += 1;
            }
            (false, true) => {
                a_only_cases.push(case.id.clone());
                per_label.entry(gold_label).or_default().lost += 1;
            }
            _ => {}
        }
    }
    let acc_a = ok_a.iter().filter(|&&b| b).count() as f64 / ok_a.len() as f64;
    let acc_b = ok_b.iter().filter(|&&b| b).count() as f64 / ok_b.len() as f64;
    let lb95 = paired_lb95(&ok_b, &ok_a);

    // Instrument aliveness: arm A must reproduce the published massive
    // row — same 300 frozen rows, same build path. (Other suites: the
    // anchor is whatever run() publishes; no hard pin here.)
    let aliveness = if spec.name == "massive_intent_en" {
        if (acc_a - PUBLISHED_MASSIVE_A0).abs() <= 1e-6 {
            format!(
                "arm A reproduces the published {PUBLISHED_MASSIVE_A0} exactly (the seat \\
                 posture, selected knobs + fitted gate) — the instrument is alive"
            )
        } else {
            format!(
                "WARN: arm A reads {acc_a:.4} vs the published {PUBLISHED_MASSIVE_A0} — the \
                 instrument is NOT reproducing the anchor; investigate before reading the \
                 verdict"
            )
        }
    } else {
        "no published anchor for this suite — arm A is its own baseline".to_string()
    };

    let verdict = if lb95 > 0.0 && acc_b > acc_a {
        "V5 PASS — synthesis lifts the modelless row (paired LB95 > 0)"
    } else {
        "V5 FAIL — synthesis does not lift the modelless row (the corpus stays gold-only; \
         the artifact stays recorded)"
    };
    let (p50a, p99a, _) = percentile_us(&lat_a.durs_us);
    let (p50b, p99b, _) = percentile_us(&lat_b.durs_us);
    let mut flips: Vec<CorpusAbFlip> = per_label
        .into_iter()
        .map(|(label, f)| CorpusAbFlip { label, gained: f.gained, lost: f.lost })
        .collect();
    flips.sort_by(|a, b| {
        let na = a.gained as i64 - a.lost as i64;
        let nb = b.gained as i64 - b.lost as i64;
        nb.cmp(&na).then(a.label.cmp(&b.label))
    });

    eprintln!(
        "  [corpus-ab {}] gold {acc_a:.4} → synth {acc_b:.4} · LB95 {lb95:+.4} · +{} synth \
         doc(s) · verdict: {verdict}",
        spec.name, extra_used,
    );
    Ok(CorpusAbSuite {
        suite: spec.name.to_string(),
        synth_artifact: synth_path.display().to_string(),
        synth_rows_total,
        synth_rows_in_scope: synth_docs.len(),
        gold_corpus_docs: prepared.train.len(),
        synth_corpus_docs: prepared.train.len() + extra_used,
        acc_gold: acc_a,
        acc_synth: acc_b,
        delta_lb95: lb95,
        flips,
        b_only_cases,
        a_only_cases,
        lat_p50_gold_us: p50a,
        lat_p99_gold_us: p99a,
        lat_p50_synth_us: p50b,
        lat_p99_synth_us: p99b,
        determinism_ok: lat_a.determinism_ok.unwrap_or(false)
            && lat_b.determinism_ok.unwrap_or(false),
        aliveness,
        verdict: verdict.to_string(),
        seconds: t_start.elapsed().as_secs_f64(),
    })
}

/// The A/B report markdown (the bin writes CORPUS_AB.md).
pub fn render_corpus_ab_markdown(out: &CorpusAbOutput) -> String {
    let mut s = String::new();
    s.push_str("# corpus A/B (Plan 426 T5 — the V5 gate)\n\n");
    s.push_str(&format!(
        "- cap {}/label · synth extra-cap {} · datasets {} · read @ {}\n- {}\n\n",
        out.meta.corpus_cap_per_label,
        out.meta.synth_extra_cap,
        out.meta.datasets_dir,
        &out.meta.git_sha[..8.min(out.meta.git_sha.len())],
        out.meta.read_rule,
    ));
    for v in &out.suites {
        s.push_str(&format!(
            "## {} — gold {:.4} → synth {:.4} · paired LB95 {:+.4}\n\n",
            v.suite, v.acc_gold, v.acc_synth, v.delta_lb95
        ));
        s.push_str(&format!("- {}\n- {}\n", v.aliveness, v.verdict));
        s.push_str(&format!(
            "- corpus: gold {} doc(s) → synth {} doc(s) ({} in scope of {} artifact rows)\n",
            v.gold_corpus_docs,
            v.synth_corpus_docs,
            v.synth_rows_in_scope,
            v.synth_rows_total
        ));
        s.push_str(&format!(
            "- latency: gold p50 {} µs / p99 {} µs · synth p50 {} µs / p99 {} µs (V6: the \
             lane stays sub-ms)\n",
            v.lat_p50_gold_us, v.lat_p99_gold_us, v.lat_p50_synth_us, v.lat_p99_synth_us
        ));
        s.push_str(&format!(
            "- flips: {} synth-only wins, {} gold-only wins · determinism {}\n\n",
            v.b_only_cases.len(),
            v.a_only_cases.len(),
            v.determinism_ok
        ));
        if !v.flips.is_empty() {
            s.push_str("| label | gained | lost | net |\n|---|---|---|---|\n");
            for f in &v.flips {
                s.push_str(&format!(
                    "| {} | {} | {} | {:+} |\n",
                    f.label,
                    f.gained,
                    f.lost,
                    f.gained as i64 - f.lost as i64
                ));
            }
            s.push('\n');
        }
        if !v.b_only_cases.is_empty() {
            s.push_str(&format!("synth-only wins: {}\n\n", v.b_only_cases.join(", ")));
        }
        if !v.a_only_cases.is_empty() {
            s.push_str(&format!("gold-only wins: {}\n\n", v.a_only_cases.join(", ")));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn td(label: &str, text: &str) -> TrainDoc {
        TrainDoc { label: label.to_string(), text: text.to_string() }
    }

    #[test]
    fn arm_specs_gold_first_then_synth_beyond_the_cap() {
        let train = vec![
            td("L1", "gold a1"),
            td("L1", "gold a2"),
            td("L1", "gold a3"),
            td("L2", "gold b1"),
        ];
        let extra = vec![
            td("L1", "synth a4"),
            td("L1", "synth a5"),
            td("L1", "synth a6"),
            td("L9", "synth z9"),
        ];
        let labels = vec!["L1".to_string(), "L2".to_string()];
        let (specs, fb, used) = arm_specs(&train, &extra, &labels, 2, 2, None);
        assert_eq!(specs.len(), 2);
        assert_eq!(used, 2, "L1 takes 2 of its 3 extras (extra cap); L2/L9 take none");
        // Inspect the built specs through the public name field.
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["L1", "L2"]);
        assert!(fb.is_empty());
    }

    #[test]
    fn arm_specs_fallback_law_matches_specs_from_pool() {
        let train = vec![td("L1", "gold a1")];
        let labels = vec!["L1".to_string(), "L_empty".to_string()];
        let (specs, fb, _used) = arm_specs(&train, &[], &labels, 48, 128, None);
        assert_eq!(fb, vec!["L_empty".to_string()], "the empty label discloses");
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn gold_label_reads_the_presented_keys() {
        let case = SuiteCase {
            id: "t".into(),
            state: serde_json::json!({"utterance": "x"}),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "intent".into(),
                kind: QKind::Choice,
                instructions: "i".into(),
                criteria: {
                    let mut m = serde_json::Map::new();
                    m.insert("beta".to_string(), serde_json::Value::String("b".into()));
                    m.insert("alpha".to_string(), serde_json::Value::String("a".into()));
                    serde_json::Value::Object(m)
                },
            }],
            gold: vec![crate::harness::suites::GoldAnswer {
                idx: 0,
                soft: vec![0.0; 2],
                gold_score: None,
            }],
        };
        // serde_json::Map preserves INSERTION order here (preserve_order):
        // beta inserted first → keys [beta, alpha] → idx 0 = "beta". The
        // veto path is order-agnostic (key-string accept), and gold_pos is
        // computed from the same map the lane builds (sorted insertion).
        assert_eq!(gold_label_of_case(&case), "beta");
    }
}
