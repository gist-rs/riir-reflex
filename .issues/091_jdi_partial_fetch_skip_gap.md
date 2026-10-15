# Issue 091: jdi_tranche1_fetched_count_floors reds on a PARTIAL local fetch — skip logic covers absent, not short

**Status:** OPEN → fixed in this session (see HISTORY.md; closed by the same commit that lands the fix)

**Claimed:** 4090-lowo-window-s2, epoch 1792095445, expires 2026-10-17T20:17:25Z

## Symptom (measured, this box)

`cargo test --test harness_units jdi_tranche1_fetched_count_floors` FAILS on
the 4090 box: `hellaswag_jdi` builds 6400 cases against the pinned 8000
(`assert_eq!(suite.cases.len(), 8000)`). The local
`.raw/datasets/hellaswag_jdi` is a PARTIAL fetch — datasets-server returned
8 failed requests during the re-fetch attempt (prior session record); the
other three JDI suites are complete.

## The gap

The test's `envelope()` closure skips LOUD only when the suite DIR is
**absent**. A dir that is **present but short** (a flaked fetch) flows into
the count assert and reads as a code failure — indistinguishable from a real
builder regression, and it leaves a standing red on any box with a partial
fetch (this one).

## The fix (direction split, both stay honest)

- **Short envelope** (`test_rows.len() < pinned`): SKIP loud with the partial
  count disclosed (`fetched/pinned`), naming the environment cause — the
  suite was NOT measured, this is NOT a pass for it. `JDI_REQUIRE_DATA=1`
  escalates this skip to a failure, same as the absent-dir arm (the dev lane
  keeps its enforcement posture).
- **Full envelope, short builder output**: the `assert_eq!` stays a HARD
  red — that direction is a builder regression (a parse/filter change
  dropping rows), which is exactly what this floor exists to catch. The fix
  must not mask it.
- `fetched > pinned` (upstream growth): falls through to the count assert —
  a pin-drift red, which is a re-pin decision, not a skip.

## Files

- `tests/harness_units.rs` — `jdi_tranche1_fetched_count_floors` only.
