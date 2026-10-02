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
//! - the five wide families re-assert the authoring law's overlap bounds
//!   (the gates file owns their label bans + digest pins);
//! - `harness_cache_reuse` is pinned at its EXACT measured history
//!   (the documented divergence — it keeps the inflated T3 record ON
//!   PURPOSE, so its numbers are a history pin, never a ceiling);
//! - `code_fixtures` is pinned at its measured numbers, whatever they
//!   are — a re-baseline of its frozen population must re-pin here
//!   consciously, with the overlap stats moving in the record.
//!
//! Every run prints the fleet table — the numbers the records cite.

#![cfg(feature = "modelless")]

use riir_reflex::harness::fixture_hygiene;
use riir_reflex::harness::runner;

const WIDE_MAX_MEAN_OVERLAP: f64 = 0.78;
const WIDE_MAX_CASE_OVERLAP: f64 = 0.92;

/// The five widened families (the wide authoring law's population —
/// `harness_cache_reuse` is the documented divergence, pinned below at
/// its history, not held to the wide bounds).
const WIDE_FAMILIES: &[&str] = &[
    "harness_visibility",
    "harness_permissions",
    "harness_tool_fit",
    "harness_routing",
    "harness_sensitivity",
];

/// `harness_cache_reuse`'s EXACT measured hygiene (the 059 audit — the
/// frozen T3 record kept on purpose). Drift reds: a moved number means
/// the frozen fixtures were edited without their divergence note.
const CACHE_REUSE_PIN: (f64, f64, usize) = (0.8145, 0.8750, 11);

/// `code_fixtures`' measured hygiene (the 059 audit, 2026-10-02): mean
/// 0.9494 · max 1.0000 · 15 of 16 eval cases share a 3-gram with the
/// corpus pool — the fleet's WORST contamination, the measured basis of
/// instinct issue 008 T8's unfalsifiable-memorization refusal (its eval
/// snippets ARE the corpus's source vocabulary). RECORDED, not gated at
/// the wide bounds: the population is FROZEN + digest-pinned
/// (`code_frozen`), and a re-baseline is the same owner-visible decision
/// the families' wide eval was. A re-baseline re-pins this triple with
/// the reason; any silent drift reds here.
const CODE_FIXTURES_PIN: (f64, f64, usize) = (0.9494, 1.0000, 15);

// (No bootstrap placeholders remain — both history pins carry their
// measured triples. A future pin edit goes through a measured run.)

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 5e-5
}

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
    covered.push("harness_cache_reuse");
    covered.push("code_fixtures");
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
        } else if name == "harness_cache_reuse" {
            // History pin: the frozen T3 record's exact measured shape.
            assert!(
                approx(h.mean_unigram_overlap, CACHE_REUSE_PIN.0)
                    && approx(h.max_unigram_overlap, CACHE_REUSE_PIN.1)
                    && h.trigram_hit_cases == CACHE_REUSE_PIN.2,
                "cache_reuse's frozen-record hygiene moved (measured {:.4}/{:.4}/{} \
                 vs the history pin {:?}) — the T3 fixtures were edited without \
                 their documented-divergence note",
                h.mean_unigram_overlap,
                h.max_unigram_overlap,
                h.trigram_hit_cases,
                CACHE_REUSE_PIN
            );
        } else if name == "code_fixtures" {
            // Standing disclosure, pinned at the measured triple (see the
            // const's doc for the recorded-not-gated decision).
            assert!(
                approx(h.mean_unigram_overlap, CODE_FIXTURES_PIN.0)
                    && approx(h.max_unigram_overlap, CODE_FIXTURES_PIN.1)
                    && h.trigram_hit_cases == CODE_FIXTURES_PIN.2,
                "code_fixtures' hygiene moved (measured {:.4}/{:.4}/{} vs pin \
                 {:?}) — a frozen-population re-baseline re-pins here with \
                 the reason",
                h.mean_unigram_overlap,
                h.max_unigram_overlap,
                h.trigram_hit_cases,
                CODE_FIXTURES_PIN
            );
        }
    }
}
