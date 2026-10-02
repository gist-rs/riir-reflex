//! Issue 061 — the `semantic_defects` family: a code-defect question
//! family over an ORIGINAL Rust fixture set, seeded from the perch
//! taxonomy (`lakeday-org/perch @ 07d38cac`, MIT — the distill verdict is
//! MARGINAL corpus / LANE INTEL HIGH in riir-refine `.distill/001`;
//! perch ships NO labelled eval set, verified at the pin, so every gold
//! here is ours).
//!
//! The question: which defect class does this routine exhibit — perch's
//! production scan shape (class × reachability), narrowed to the class
//! axis as a single Choice. Six labels: `clean` (the faithful control) +
//! five perch classes in Rust-native spelling (`off_by_one`,
//! `inverted_condition`, `unwrapped_none` ← unhandled_null,
//! `swapped_lookup` ← wrong_lookup, `swallowed_error` ← error_ignored).
//!
//! Authoring law (the wide gates own the enforcement):
//! - dual grounding at authoring time — each fixture makes its gold
//!   class derivable AND the five rivals refutable (the scan.yaml
//!   true/false-criteria bar, per issue 061);
//! - eval AND cal carry no label token (the text never names its own
//!   answer — a `None` literal, the word "unwrap" in prose, or the class
//!   names would leak);
//! - the code SHOWS the defect; the prose never pronounces the verdict;
//! - 6 × 17 eval balance, corpus 3 docs per class (corpus MAY name the
//!   classes — that is what teaches the corpus-conditioned drafter).

use crate::harness::families::FamilyDef;
use crate::harness::suites::QKind;

static DEFECTS_INSTR: &str = "This routine was audited for a single defect \
class. clean = the code does exactly what its contract states; off_by_one = \
a bound or index steps a single position past (or falls a single position \
short of) the intended range; inverted_condition = a guard or comparison \
tests the opposite of the documented intent; unwrapped_none = a value that \
may be absent is force-decoded without handling the absent case; \
swapped_lookup = the wrong key, field, or index is read where a different \
target was intended; swallowed_error = a failing result is discarded \
silently instead of propagated or surfaced.";

pub static DEFECTS: FamilyDef = FamilyDef {
    name: "semantic_defects",
    qid: "defect_class",
    kind: QKind::Choice,
    labels: &[
        "clean",
        "off_by_one",
        "inverted_condition",
        "unwrapped_none",
        "swapped_lookup",
        "swallowed_error",
    ],
    instructions: DEFECTS_INSTR,
    score_levels: &[],
    note: "authored Rust scan fixtures (issue 061; taxonomy from lakeday-org/perch \
           @ 07d38cac, which ships no labelled set — gold is ours); gold = the audited \
           defect class; dual-grounding authoring bar (true-criteria + false-criteria \
           derivable per fixture); eval/cal label-token-free; 6x17 balance",
    corpus: &[
        // ── clean (0) ──
        (
            0,
            "A faithful routine needs no patch: its bounds match the declared range, \
             absence is handled where absence can happen, the intended field is read, \
             and failures travel to the caller through the return type.",
        ),
        (
            0,
            "The walk covers exactly the documented window — the final element is \
             visited and nothing beyond it is touched, including at the last tile, \
             the final page, and the wrap point.",
        ),
        (
            0,
            "Where a table read can miss, the fallback runs at the read site: a \
             default is supplied, and a failing sub-call returns its Result upward \
             instead of being reduced to a bare flag.",
        ),
        // ── off_by_one (1) ──
        (
            1,
            "off_by_one: the loop or slice bound reaches a single element beyond \
             the buffer's end, or stops a single element before the last item the \
             contract promises to cover — inclusive `..=` on a count, `<=` on a \
             length, a stride tile whose end index outruns the allocation.",
        ),
        (
            1,
            "The classic shape is a count used as an inclusive endpoint, so index \
             `count` is touched although valid slots run `0 .. count` exclusive; \
             the cousin starts the walk at 1 and silently skips the first slot.",
        ),
        (
            1,
            "Window arithmetic that adds a width to a start position without \
             re-checking the sum against the length reads past the end on the \
             final window; wrap math with `% (capacity + 1)` claims a slot the \
             array never reserved.",
        ),
        // ── inverted_condition (2) ──
        (
            2,
            "inverted_condition: the guard tests the opposite sense of its \
             documented purpose — the branch meant for the rare case runs for the \
             common case, and the normal path is skipped. Every token is spelled \
             correctly; only the sense is wrong.",
        ),
        (
            2,
            "A negation in the wrong place flips eligibility: members are refused \
             while guests pass, fresh keys never recompute while cached ones do, \
             the retry triggers exactly when it should not.",
        ),
        (
            2,
            "Equality intended, inequality written (or the boolean literal flipped): \
             the comparison passes precisely when it should fail, so the alarm \
             fires on agreement and sleeps through divergence.",
        ),
        // ── unwrapped_none (3) ──
        (
            3,
            "unwrapped_none: a value that can legitimately be absent gets \
             force-decoded at the read site — a missing entry panics instead of \
             taking the documented fallback.",
        ),
        (
            3,
            "The absence path exists in the type but the code steps around it: \
             `.unwrap()` or a reassuring `.expect(..)` replaces the match the \
             signature asks for, and the panic lands on data the docs call \
             optional — an omitted config key, a truncated file, a raced pop.",
        ),
        (
            3,
            "Optional input decoded with unwrap at boot or at the API boundary: \
             any deployment or request that omits the optional piece dies before \
             the service is reachable, though the type system offered the \
             fallback arm.",
        ),
        // ── swapped_lookup (4) ──
        (
            4,
            "swapped_lookup: the read targets the wrong key, field, or axis — an \
             identifier used where a name was intended, the adjacent getter \
             called, rows and columns transposed — so the value returned belongs \
             to a different row.",
        ),
        (
            4,
            "Copy-paste leaves the neighboring variable in the index expression; \
             the two tables have compatible value types, so the compiler stays \
             silent and only the data notices.",
        ),
        (
            4,
            "Twin keys from the same table are the usual culprit: `verbose` bound \
             into `quiet`, `p95` recorded as `p99`, the stable slot read where \
             the rollout candidate was wanted, `en_US` served for `en_GB`.",
        ),
        // ── swallowed_error (5) ──
        (
            5,
            "swallowed_error: the failure half of a Result is discarded at the \
             call site — `let _ =`, `.ok()`, a wildcard match arm mapping every \
             failure to a default — so the routine reports success regardless.",
        ),
        (
            5,
            "The diagnostic payload vanishes with the outcome: no log line, no \
             counter, no poisoned-batch signal. Outages masquerade as ordinary \
             empty results, and the SLO summary reads a clean run that never \
             happened.",
        ),
        (
            5,
            "Durable-write contracts are the common victim: a dropped append, an \
             ignored flush, a torn config that leaves yesterday's policy running \
             — each visible only after the data is gone.",
        ),
    ],
    cal: &[
        crate::harness::families::FamilyText {
            text: "Iterates `0 .. window_count` collecting exactly the windows the \
                   header declares; the collector's length matches the count field.",
            gold: 0,
        },
        crate::harness::families::FamilyText {
            text: "Cache misses resolve via `unwrap_or(Loadout::default())`, spelled \
                   at the single read site; callers always see a value.",
            gold: 0,
        },
        crate::harness::families::FamilyText {
            text: "The report selects `leads[lead_uuid]`, matching the column the CSV \
                   header names; each figure lands in its own column.",
            gold: 0,
        },
        crate::harness::families::FamilyText {
            text: "Reads `&tape[pos ..= pos + frame]` while stepping forward, touching \
                   the frame after the window at every step but the last.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`if !throttled { throttle(); }` sits beneath a comment promising \
                   the reverse policy for hot paths.",
            gold: 2,
        },
        crate::harness::families::FamilyText {
            text: "`ports.get(\"https\").unwrap()` inside a library whose docs \
                   promise a plain-HTTP fallback when TLS is unavailable.",
            gold: 3,
        },
        crate::harness::families::FamilyText {
            text: "`ledgers[acct_alias]` fetched where the balance table keys \
                   `acct_uuid`; aliases repeat across regions and merge balances.",
            gold: 4,
        },
        crate::harness::families::FamilyText {
            text: "`let _ = mirror.flush();` precedes the checkpoint line; a dropped \
                   flush still prints the checkpoint as written.",
            gold: 5,
        },
        crate::harness::families::FamilyText {
            text: "The stepper walks `0 ..= steps.len()`, advancing through the \
                   sentinel position past the final plan step.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`metrics[\"rps\"]` is written into the struct field named \
                   `latency_ms` in the exported rollup.",
            gold: 4,
        },
        crate::harness::families::FamilyText {
            text: "`queue.front().unwrap()` sits in the tick handler, which the \
                   design doc says drains to empty between ticks.",
            gold: 3,
        },
        crate::harness::families::FamilyText {
            text: "The purger tests `entry.age() < horizon` where policy says \
                   entries older than the horizon are the purge set.",
            gold: 2,
        },
        crate::harness::families::FamilyText {
            text: "The guard reads `if user.banned { welcome(user); }` beneath a \
                   docstring about enforcement at the door.",
            gold: 2,
        },
        crate::harness::families::FamilyText {
            text: "Wrap arithmetic `% (slots + 1)` lets the producer claim a \
                   position the ring array never allocated.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`retry(attempt + 1)` loops `0 ..= max_attempts`, issuing a \
                   further attempt past the documented policy count.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`tx.send(batch).await.ok();` sits in the shutdown path; queued \
                   work evaporates under load with the exit code still zero.",
            gold: 5,
        },
        crate::harness::families::FamilyText {
            text: "`tiles[y][x]` is read in the pathfinder's neighbor probe where \
                   every other site reads `tiles[x][y]`.",
            gold: 4,
        },
        crate::harness::families::FamilyText {
            text: "Anchor math adds `span` to the cursor with no length re-check; \
                   the final call reads past the segment end.",
            gold: 1,
        },
    ],
    eval: &crate::harness::families_semantic_defects_eval::DEFECTS_WIDE_EVAL,
};
