//! Issue 049 T1+T2+T3's seat gates: the remaining synthetic family
//! `semantic_defects` + `code_fixtures` seat through `prepare_seat`
//! (marked `synthetic`), the seated path reproduces the runner's own
//! build byte-for-byte, and unknown suites refuse. (The six Issue-004
//! families that used to seat here were retired 2026-10-02.)
//!
//! Pins:
//! - **marker** — every synthetic seat carries `synthetic: true` (the
//!   posture-fork defence: the consumer cannot mistake it for a dataset
//!   seat); a dataset seat (when datasets are on disk) carries `false`.
//! - **byte identity** — a family's seated questions are `==` its
//!   `synth_by_name` questions (the seat adds no derivation of its own).
//! - **accuracy equality** — the seat posture's engine answers a family's
//!   eval cases with EXACTLY the accuracy of the direct manual build (the
//!   `harness_families_gates` engine path): the seat is the runner's code,
//!   not a parallel implementation.
//! - **unknown suites refuse** (never a silent fallback).
//! - **dataset posture unchanged** — a dataset suite still seats with
//!   `synthetic: false` when its files are on disk (skips loud otherwise;
//!   `REFLEX_SEAT_REQUIRE_DATA=1` makes the skip a failure).

use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, ExpertSpec};
use riir_reflex::harness::families;
use riir_reflex::harness::runner::seat::{PostureKnobs, fit_posture, prepare_seat};

const FAMILIES: [(&str, usize); 1] = [("semantic_defects", 6)];

#[test]
fn synthetic_families_seat_marked_and_byte_identical() {
    for (name, n) in FAMILIES {
        let dir = std::path::Path::new(".raw/datasets"); // never touched by synthetic seats
        let seat = prepare_seat(name, dir).unwrap_or_else(|e| panic!("{name}: seat failed: {e}"));
        assert!(
            seat.synthetic,
            "{name}: synthetic seat must carry the marker"
        );
        let d = families::synth_by_name(name).expect("family synth");
        assert_eq!(
            seat.suite.cases, d.suite.cases,
            "{name}: seated questions diverge from the family's own build"
        );
        assert_eq!(seat.labels, d.labels, "{name}: label order drifted");
        assert_eq!(seat.cal_cases, d.cal_cases, "{name}: cal front drifted");
        // The pool is null on the synthetic path (selection-ineligible).
        assert!(
            seat.pool_rows.is_null(),
            "{name}: synthetic pool must be null"
        );
        let _ = n;
    }
}

#[test]
fn code_fixtures_seats_as_synthetic_the_recorded_decision() {
    // Issue 049 T2's explicit decision: code_fixtures is in-process
    // generated with its own programmatic cal slice — the same nature as
    // the synthetic families — so it joins T1's branch, marked synthetic.
    let seat = prepare_seat("code_fixtures", std::path::Path::new(".raw/datasets")).expect("seat");
    assert!(
        seat.synthetic,
        "code_fixtures seat must carry the synthetic marker"
    );
    assert!(
        !seat.suite.cases.is_empty(),
        "code_fixtures seat must carry its cases"
    );
    assert!(
        !seat.cal_cases.is_empty(),
        "code_fixtures seat must carry its cal slice"
    );
}

#[test]
fn seat_engine_accuracy_equals_the_manual_build() {
    // T3's guarantee: the seat path IS the runner's code. One family
    // end-to-end: fit the seat posture, build the seat engine, eval, and
    // compare forced-pick accuracy against the direct manual build (the
    // same construction `harness_families_gates::family_engine` uses) on
    // the same cases. EXACT equality — any drift is a fork.
    const NAME: &str = "semantic_defects";
    const N: usize = 6;
    let seat = prepare_seat(NAME, std::path::Path::new(".raw/datasets")).expect("seat");
    let posture = fit_posture::<N>(NAME, &seat, &PostureKnobs::default()).expect("posture");
    let (mut seat_engine, _) = riir_reflex::harness::runner::seat::build_seat_engine::<N>(
        NAME,
        &seat,
        posture.effective_cap,
        posture.cfg.clone(),
    )
    .expect("seat engine");
    let eval = riir_reflex::harness::runner::seat::eval_seat::<N>(
        &mut seat_engine,
        &seat.suite.cases,
        &seat.state_strs,
    )
    .expect("seat eval");

    // The manual build: the family's own docs at the SAME posture (the
    // fitted thresholds are part of the deployed posture — the default
    // config has no cal fit, so the two engines differ exactly there by
    // construction). The pin: given the posture, the seat's build adds
    // nothing of its own — answers are bit-identical.
    let d = families::synth_by_name(NAME).expect("family synth");
    let specs: Vec<ExpertSpec> = d
        .labels
        .iter()
        .map(|l| {
            let docs: Vec<String> = d
                .docs
                .iter()
                .filter(|doc| &doc.label == l)
                .map(|doc| doc.text.clone())
                .collect();
            ExpertSpec::new(l, &docs)
        })
        .collect();
    let mut manual = DecisionEngine::<N, EMBED_DIM>::build_specs(specs, posture.cfg.clone())
        .expect("manual engine");
    let manual_eval = riir_reflex::harness::runner::seat::eval_seat::<N>(
        &mut manual,
        &seat.suite.cases,
        &seat.state_strs,
    )
    .expect("manual eval");

    assert_eq!(
        eval.cases.len(),
        manual_eval.cases.len(),
        "case count drifted"
    );
    let mut seat_hits = 0usize;
    let mut manual_hits = 0usize;
    for (i, (sc, mc)) in eval.cases.iter().zip(&manual_eval.cases).enumerate() {
        let (s, m) = (&sc[0], &mc[0]);
        assert_eq!(s.pick, m.pick, "{NAME} case {i}: seat pick != manual pick");
        assert_eq!(
            s.abstained, m.abstained,
            "{NAME} case {i}: abstention drifted"
        );
        assert!(
            s.probs.len() == m.probs.len()
                && s.probs
                    .iter()
                    .zip(&m.probs)
                    .all(|(a, b)| a.to_bits() == b.to_bits()),
            "{NAME} case {i}: probabilities drifted"
        );
        assert!(
            s.conf.to_bits() == m.conf.to_bits(),
            "{NAME} case {i}: confidence drifted"
        );
        let gold = seat.suite.cases[i].gold[0].idx;
        if s.pick == gold {
            seat_hits += 1;
        }
        if m.pick == gold {
            manual_hits += 1;
        }
    }
    assert_eq!(
        seat_hits, manual_hits,
        "{NAME}: accuracy drifted between paths"
    );
    println!(
        "[recorded] {NAME}: seat accuracy {}/{} == manual build",
        seat_hits,
        seat.suite.cases.len()
    );
}

#[test]
fn synthetic_family_seats_cal_and_corpus_and_unknown_suites_refuse() {
    // The family seats like code_fixtures — the synthetic marker, the
    // pool-null rule, and the byte-identity pins apply to it unchanged
    // (the FAMILIES loop above covers them; here the docs/cal presence
    // is pinned alongside the refusal arm).
    let seat = prepare_seat("semantic_defects", std::path::Path::new(".raw/datasets"))
        .expect("semantic_defects must seat");
    assert!(
        seat.synthetic,
        "semantic_defects seat must carry the synthetic marker"
    );
    assert!(seat.pool_rows.is_null(), "synthetic pool stays null");
    assert!(
        !seat.cal_cases.is_empty() && !seat.train.is_empty(),
        "semantic_defects must ship its cal front and corpus"
    );
    let err = prepare_seat("not_a_suite", std::path::Path::new(".raw/datasets"))
        .err()
        .expect("unknown suite must refuse");
    assert!(err.contains("unknown suite"), "got: {err}");
}

#[test]
fn dataset_suites_seat_unmarked_when_data_is_present() {
    // The dataset posture is unchanged: `synthetic: false`, byte-identical
    // with the on-disk rows. Skips loud without the fetch (the
    // slice_leak pattern; REFLEX_SEAT_REQUIRE_DATA=1 fails the skip).
    let dir = std::path::Path::new(".raw/datasets_t20k");
    if !dir.join("ag_news").exists() {
        let require = std::env::var("REFLEX_SEAT_REQUIRE_DATA").is_ok_and(|v| v == "1");
        println!("SKIP: .raw/datasets_t20k/ag_news absent — dataset-seat pin not run");
        if require {
            panic!("REFLEX_SEAT_REQUIRE_DATA=1 but the datasets are absent");
        }
        return;
    }
    let seat = prepare_seat("ag_news", dir).expect("ag_news seat");
    assert!(
        !seat.synthetic,
        "dataset seat must NOT carry the synthetic marker"
    );
    assert!(
        !seat.pool_rows.is_null(),
        "dataset seat carries its selection pool"
    );
    assert!(
        !seat.suite.cases.is_empty(),
        "dataset seat carries its cases"
    );
}

#[test]
fn eval_case_into_matches_eval_seat_case_by_case() {
    // Reflex issue 070 lead 2's byte-parity contract: the scratch face
    // (`eval_case_into`) answers EXACTLY `eval_seat`'s per-case slice —
    // probabilities, picks, confidences, abstain flags — across the whole
    // synthetic family, through ONE frame REUSED across cases (the refill
    // path, not the fresh-build path: a stale-slot or capacity bug shows
    // up from the second case on).
    const NAME: &str = "semantic_defects";
    const N: usize = 6;
    let seat = prepare_seat(NAME, std::path::Path::new(".raw/datasets")).expect("seat");
    let posture = fit_posture::<N>(NAME, &seat, &PostureKnobs::default()).expect("posture");
    let (mut engine, _) = riir_reflex::harness::runner::seat::build_seat_engine::<N>(
        NAME,
        &seat,
        posture.effective_cap,
        posture.cfg.clone(),
    )
    .expect("seat engine");
    let eval = riir_reflex::harness::runner::seat::eval_seat::<N>(
        &mut engine,
        &seat.suite.cases,
        &seat.state_strs,
    )
    .expect("seat eval");

    let mut frame = riir_reflex::harness::runner::seat::CaseEvalScratch::new();
    for (ci, (case, state)) in seat.suite.cases.iter().zip(&seat.state_strs).enumerate() {
        riir_reflex::harness::runner::seat::eval_case_into::<N>(
            &mut engine,
            case,
            state,
            &mut frame,
        )
        .unwrap_or_else(|e| panic!("{NAME} case {ci}: scratch eval failed: {e}"));
        let outs = &eval.cases[ci];
        assert_eq!(
            outs.len(),
            frame.abstained.len(),
            "{NAME} case {ci}: question count drifted"
        );
        for (qi, qo) in outs.iter().enumerate() {
            assert_eq!(qo.probs, frame.probs[qi], "{NAME} case {ci} q{qi}: probs drifted");
            assert_eq!(qo.pick, frame.picks[qi], "{NAME} case {ci} q{qi}: pick drifted");
            assert_eq!(qo.conf, frame.confs[qi], "{NAME} case {ci} q{qi}: conf drifted");
            assert_eq!(
                qo.abstained, frame.abstained[qi],
                "{NAME} case {ci} q{qi}: abstention drifted"
            );
        }
    }
}
