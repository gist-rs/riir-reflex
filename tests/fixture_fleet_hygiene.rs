//! The fixture-FLEET hygiene audit (the four-time recurrence wall).
//!
//! WHY (2026-10-02): the workspace measured the same contamination class
//! four times — dataset near-duplicates (Issue 024), the frozen t20k sst5
//! duplicates (slice_guard KNOWN_DIRTY), code_fixtures' template-shared
//! eval (instinct 008 T8), and the harness families' corpus-overlap
//! inflation (measured at the 059 wide-eval landing: 0.68–0.78 mean
//! unigram overlap, per-case maxes 1.00) — because every fixture set was
//! authored ungated and every prior instrument was private to the set
//! that got burned. This audit closes the loop mechanically:
//!
//! - its population is DERIVED (`runner::synthetic_suite_names()`), so a
//!   new synthetic suite joins by EXISTING and reds here until it carries
//!   hygiene facts — no 12th-instance silence;
//! - `semantic_defects` re-asserts the authoring law's overlap bounds
//!   (the gates file owns its label ban + digest pin).
//!
//! (`code_fixtures` was pinned here at its measured triple — the fleet's
//! WORST contamination, 0.9494/1.0000/15 — until its 2026-10-11
//! retirement; the pin is history now, record in HISTORY.md.)
//!
//! Every run prints the fleet table — the numbers the records cite.

#![cfg(feature = "modelless")]

use riir_reflex::harness::fixture_hygiene;
use riir_reflex::harness::runner;

const WIDE_MAX_MEAN_OVERLAP: f64 = 0.78;
const WIDE_MAX_CASE_OVERLAP: f64 = 0.92;

/// The wide-law population (`semantic_defects` — the gates file owns the
/// label ban + the digest pin).
const WIDE_FAMILIES: &[&str] = &["semantic_defects"];

fn slice_strings(seat: &runner::seat::Seat) -> (Vec<String>, Vec<String>, Vec<String>) {
    let corpus: Vec<String> = seat.train.iter().map(|d| d.text.clone()).collect();
    let cal: Vec<String> = seat
        .cal_cases
        .iter()
        .map(|c| riir_reflex::pyjson::serialize_state(&c.state))
        .collect();
    let eval: Vec<String> = seat.state_strs.clone();
    (corpus, cal, eval)
}

#[test]
fn fixture_fleet_hygiene_is_measured_and_walled() {
    let registry = runner::synthetic_suite_names();

    // The COVERAGE WALL: every synthetic suite in the registry must be
    // adjudicated below. A new synthetic suite reds HERE — the audit's
    // arms must name it before it can land ungated.
    let mut covered: Vec<&str> = WIDE_FAMILIES.to_vec();
    covered.sort_unstable();
    let mut derived = registry.clone();
    derived.sort_unstable();
    assert_eq!(
        covered, derived,
        "the synthetic registry moved — every suite needs a hygiene arm in \
         this audit (fixture_hygiene::audit_slices), never an ungated landing"
    );

    println!("── fixture fleet hygiene ──");
    // Pass 1: measure + print the WHOLE fleet (the bootstrap measurement
    // must see every suite, never just the first refuser).
    let mut facts: Vec<(&str, fixture_hygiene::SliceHygiene)> = Vec::new();
    for name in registry {
        let seat = runner::seat::prepare_seat(name, std::path::Path::new(".raw/datasets"))
            .unwrap_or_else(|e| panic!("prepare {name}: {e}"));
        let (corpus, cal, eval) = slice_strings(&seat);
        let corpus: Vec<&str> = corpus.iter().map(String::as_str).collect();
        let cal: Vec<&str> = cal.iter().map(String::as_str).collect();
        let eval: Vec<&str> = eval.iter().map(String::as_str).collect();
        let h = fixture_hygiene::audit_slices(&corpus, &cal, &eval);
        println!(
            "{:>24}: eval {} · mean_ovl {:.4} · max_ovl {:.4} · trigram hits {}",
            name,
            h.eval_n,
            h.mean_unigram_overlap,
            h.max_unigram_overlap,
            h.trigram_hit_cases
        );
        facts.push((name, h));
    }
    // Pass 2: the per-suite arms.
    for (name, h) in facts {
        if WIDE_FAMILIES.contains(&name) {
            // The wide authoring law's overlap bounds (the gates file owns
            // the label bans + the digest pins for these five).
            assert!(
                h.mean_unigram_overlap <= WIDE_MAX_MEAN_OVERLAP
                    && h.max_unigram_overlap <= WIDE_MAX_CASE_OVERLAP
                    && h.trigram_hit_cases == 0,
                "{name}: hygiene regressed past the wide law (mean {:.4} max \
                 {:.4} hits {}) — re-author or consciously re-pin the gates",
                h.mean_unigram_overlap,
                h.max_unigram_overlap,
                h.trigram_hit_cases
            );
        }
    }
}
