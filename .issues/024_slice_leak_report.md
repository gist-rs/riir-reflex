# Issue 024 — `slice_leak`: an in-harness near-duplicate leak report (Issue 007 P2's decided route)

**Status:** OPEN — T1 + T2 LANDED 2026-09-24 (the index + its G1/G2 oracle gate, both green; see "T1/T2 landed" below). T3 (runner wiring + the `results.json` `leak` block) is next, and stays sequenced after Issue 023 T5 per the Notes.

## Finding (measured, `scripts/slice_leak_probe.py`, 2.3 s, M3 at load ~6, AC)

For each test row, the probe asks whether the fetched train slice has an
exact twin (after normalisation) or a near twin (char-4-gram Jaccard >= 0.8).
Eval rows come from the test split, and corpus and cal rows from the train
split, so this bounds every corpus∪cal → eval leak from above.

| suite | exact | near >= 0.8 | same label |
|---|---|---|---|
| ag_news | 1 / 400 | 26 / 399 (6.5%) | 25 / 26 |
| banking77 | 4 / 3076 | 61 / 1996 (3.1%) | 60 / 61 |
| massive_intent_en | 13 / 2974 (2 label-conflicting) | 49 / 1987 (2.5%) | 48 / 49 |
| prompt_injections | 0 / 116 | 2 / 116 | 2 / 2 |
| emotion, sst5, xnli_en | ≤ 1 | 0 | n/a |
| typed_decisions | 0 | 51 / 400 (TEMPLATE artifact: one fixed JSON schema) | n/a |

Specimens: banking77 `What do you base your exchange rates on?` in test and
`On what do you base your exchange rates?` in train. massive `olly turn the
lights off in the bedroom` in test and `turn the lights off in the bedroom`
in train. ag_news carries the same AP story under two outlets' headlines.

The existing self-inclusion law (`tests/harness_families_gates.rs`) is
EXACT-string, so none of these rows trips it.

## Why this matters, and why it is a column and not a filter

- The published references are measured on these same public test splits,
  so the leak is shared, and a comparison against them is still apples to
  apples. Dropping leaked rows from OUR numbers only would make that
  comparison unfair in the other direction.
- It does inflate absolute accuracy, most for a retrieval or centroid lane,
  which is what the modelless engine is. A near twin of the gold label
  sitting in the corpus is a free answer.
- So the site discloses it. It shows a per-suite leak rate, plus accuracy on
  the de-leaked eval subset beside the headline number. The headline stays
  on the full split so it stays comparable.

## Design

- Feature `slice_leak`, opt-in and default-off. It has no deps (the 4-gram
  shingles need no hashing crate). Native-only. The code goes in
  `src/harness/slice_leak.rs`, next to `families.rs`.
- A modelless shingle index: normalise, char 4-grams, an inverted index that
  skips shingles held by ≥ 200 rows, top-5 candidates, Jaccard. Build once
  per suite and pre-allocate. It is O(rows × shingles) and runs in seconds,
  and the harness corpus fits in memory. This is why Issue 007 rejected the
  neuron-db routes.
- Output per suite: `leak.exact`, `leak.near`, `leak.threshold`, and
  `acc_deleaked`, which is accuracy recomputed over the eval rows not flagged.
  It lives under a new `leak` object in `results.json`, additive, so older
  readers ignore it.
- Adjudicate over the ACTUAL corpus/cal/eval slices the runner builds, not
  over train vs test. The probe's train-vs-test numbers are an upper bound,
  and the Rust report must come in at or below them.
- typed_decisions is marked `not_applicable` (templated rows), never
  reported as a rate.

## Gates

- G1 (correctness): run on the same `.raw/datasets/` checkout with
  train-vs-test scope, the Rust index reproduces the probe's per-suite exact
  and near counts EXACTLY. The probe is the known-answer oracle.
- G2 (perf, `--release`, box state recorded): ≤ 1 s per suite on the M3,
  with no allocation inside the per-row candidate loop.
- G3 (no regression): with the feature off, every existing gate stays green
  and `results.json` is byte-identical.
- Promotion: this is a REPORT, not a runtime primitive, so it never joins
  the release set. Wire it into the harness bench run only.

## Tasks

- [x] T1: `slice_leak.rs` index plus unit tests. Arms: a planted paraphrase
  is flagged; a disjoint topic is not; the rare-cap skip holds; the
  exact-normalisation path works.
- [x] T2: the G1 oracle test against `scripts/slice_leak_probe.py`, with a
  loud skip when `.raw/datasets` is absent (UNSEEN, never a pass).
- [ ] T3: runner wiring over the real slices, plus the `leak` block in
  `results.json`.
- [ ] T4: site columns in `bench.json`. These ride the NEXT Issue 018
  two-host publish; do not force a separate one.
- [ ] T5: a Bench write-up: per-suite leak and `acc_deleaked` delta for
  each lane.

## T1/T2 landed (2026-09-24)

- `src/harness/slice_leak.rs` behind `slice_leak = []` (zero deps,
  native-only, also builds at `--no-default-features`). Flat CSR arrays for
  the per-row shingle sets and the inverted index, a `u64`-packed shingle
  (≤ 8 chars), and a sorted-run top-c. `ShingleIndex::classify` returns
  `Exact` / `Near` / `Clean`, so T3 can flag individual eval rows for
  `acc_deleaked`. `leak_counts` returns `None` for an empty side, never a
  zero rate. 9 unit arms: a planted paraphrase, a disjoint topic, the rare
  cap, the exact path, short and empty text, the Python-`isspace` gap
  (`\x1c..\x1f`), collapse-before-strip, the tie-break, and scratch reset.
- The probe's candidate order was hash-seeded. `Counter.most_common(5)`
  breaks ties by insertion order, which follows `str`-hash set iteration,
  so "reproduce EXACTLY" was not well defined. The probe now sorts
  `(count desc, index asc)` and Rust does the same. Its output is
  byte-identical to the old one under PYTHONHASHSEED 1, 2, 3 and 7, so no
  count in the Finding table moves.
- `tests/slice_leak_oracle.rs` (`required-features = ["slice_leak"]`) runs
  the probe live and parses its lines, so the oracle is the script itself
  and not a copy of its numbers.
  - **G1 ✅:** all 8 suites match on all 7 fields (typed_decisions
    included: the test transcribes Python's `json.dumps` for the string
    `state`).
  - **G2 ✅:** 3.7–52 ms per suite against a 1 s budget, `--release`. Zero
    allocations over 3076 warm `classify` calls on banking77, counted by a
    per-thread global allocator.
  - Box: M3 Max, AC, powermode 2, load 6.2, 87% memory free, sibling
    sessions active.
  - Without `.raw/datasets` or python3 the test prints UNSEEN, and
    `SLICE_LEAK_REQUIRE_DATA=1` turns that into a failure.
- Both gates were checked to fire. `DEFAULT_RARE_CAP` 200 → 100 fails G1
  on massive_intent_en. An allocation inside `classify_near` fails G2 with
  3072 allocations (one per non-exact row). The first G2 probe did NOT
  fire because LLVM elided an unused `Vec::with_capacity`; the probe that
  counts wraps it in `black_box`.
- Clippy `-D warnings` is clean with `--features slice_leak --all-targets`,
  with default `--all-targets`, and with `--no-default-features --features
  slice_leak`. G3 holds by construction for T1/T2: nothing outside the
  feature-gated module is touched.

## Notes

- Sequence after Issue 023 T5. That publish is already queued behind the
  4090 run, and a schema change landing mid-publish is exactly the
  two-host disagreement Issue 018's publisher refuses.
- massive's 2 exact label conflicts are label noise in the source dataset.
  Report them; do not fix them here.
