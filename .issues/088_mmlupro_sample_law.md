# Issue 88 — the mmlupro_jdi eval sample: stable across population changes, matching NO replicable ordering law

> **Status:** OPEN (filed 2026-10-14, 4090 lane) · severity: correctness-of-record (the numbers are real; the SAMPLING LAW documented in-source is not what runs) · discovered during riir-train 629 T2's bench-143/144 readouts

## The observation (all three empirically demonstrated, this box, 2026-10-14)

The harness's `mmlupro_jdi` eval sample — `cases_digest fnv1a64-6a2d24628ce7b916`, raw test-slice digest `fnv1a64-fd1290994c5a0790`, n=2000 — is **byte-stable across three different on-disk populations**:

1. 103 test pages (10,300 rows; the 08:36 state) → digest X.
2. 121 test pages (12,032 rows; the full universe, top-up at 09:01) → digest X, same run shape (spotcheck 143 fullpop re-read, 15:45; probe 15:53).
3. 121 pages with `test-113.json` EMPTIED (rows[] = []) in a scratch `--datasets-dir` → **digest X again** — the sample provably does not consume test-113's rows, even though test-113 sorts among the population and its rows are well-formed.

## What it is NOT (each replicated independently, python, canonical fnv1a64 verified against known vectors)

The gold (`answer_index`) sequence of the served cases — `[7, 9, 5, 6, 2, 2, 1, 7, 0, 0, 0, 6, …]` — matches NONE of:

- **first-N file order** (the `sampled()` law): `[8, 5, 9, 2, 6, 0, 3, 9, 4, 5, 4, 6, …]`
- **hash-order over `question_id` decimal** (the documented Plan-012 law): `[7, 7, 1, 2, 3, 0, 2, 4, 6, 7, 8, 2, …]`
- hash-order over the full-row serde bytes (both insertion-order and BTreeMap sorted-key serializations)
- hash-order over the 103-file sub-population
- the raw `slice_digest` over first-N / hash-order / pop103 selections (none reproduces `fd1290994c5a0790` under either key order)

## What makes this strange

- `stratified_split`'s hash branch IS hash-ordered **in the compiled binary this session ran** — proven by the new unit test `mmlupro_jdi_sample_is_hash_ordered_not_first_n` (suites.rs, passes) — and `prepare` provably calls `stratified_split(&eval_rows, spec.name, spec.test_cap)` with `test_cap = 2000` over `load_rows(suite_dir, "test")`, which reads every sorted `test-*.json` with no cap.
- So: function-level behavior (hash), call-path wiring (present), end-to-end behavior (neither law, tail-insensitive). One of the three links is not what it reads as. The tail-insensitivity pattern is characteristic of first-N or bucket-head round-robin — both of which the gold sequence rules out.

## Impact (bounded)

- The d3 mmlupro spot check (Bench 143) and any number read over the CURRENT sample is a real measurement of 2,000 real questions — valid with the population caveat.
- The dataset-manifest §15 note ("the eval sample is HASH-ORDERED … sample digest `6a2d24628ce7b916`") is NOT a description of a running law — the digest happens to be stable, so the PIN still functions as a population-identity witness, but the law prose is unverified against behavior.
- Any suite relying on the same JDI hash branch (hellaswag_jdi) inherits the question.

## Next step (whoever picks this up)

Instrument `prepare` end-to-end for mmlupro_jdi (one eprintln of `eval_rows.len()` + the first 12 picks after the split), run the modelless probe, and read which of the three links lies. The probe harness and scratch-dir method that demonstrated tail-insensitivity are in this issue's record (`.raw/mmlupro_probe*`, `.raw/mmlupro_probe_ds` — gitignored, regenerate per the cmds in bench 143/144 provenance).
