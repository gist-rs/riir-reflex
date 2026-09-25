# Issue 024 — `slice_leak`: an in-harness near-duplicate leak report (Issue 007 P2's decided route)

**Status:** OPEN — T1 + T2 LANDED 2026-09-24 (the index + its G1/G2 oracle gate, both green; see "T1/T2 landed" below). **T3 LANDED 2026-09-24** (runner wiring + the `results.json` `leak` block; record below). **T4 DONE 2026-09-25** (site columns rode the 027 window publish — see the task row; the old status line here predates that landing). **T5's 4090-modelless half LANDED 2026-09-26 (Bench 044, `.benchmarks/044_headsel_4090_leak/`): the 4090's head-select run taken WITH `--features slice_leak` — 7/7 in-scope suites carry leak blocks + `acc_deleaked`, counts byte-identical to the M3's T3 oracle values and all at-or-below the probe bound (verified); de-leaked deltas ≤ 1.2 pt (massive), banking77's promotion stands at 0.6832 de-leaked.** T5 (the Bench write-up) still waits on the M3 leak-enabled run (queued behind the sibling's needle run — take it with `--head-select --features slice_leak` alongside Issue 032's M3 leg) + the laya lanes' acc_deleaked columns from the next full multi-lane run.

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
- [x] T3: runner wiring over the real slices, plus the `leak` block in
  `results.json`.
- [x] T4: site columns in `bench.json`. These ride the NEXT Issue 018
  two-host publish; do not force a separate one. **DONE 2026-09-25** —
  rode the 027 window publish (bench 034 post-renumber, site `ae79a87`): the 7 dataset
  suites carry `leak` blocks (exact/near counts) + `acc_deleaked` on
  every lane row; `publish_bench.py` carries them through update docs
  (a doc without a block never erases a previous scan) and the bench
  page renders the per-suite disclosure line.
- [ ] T5: a Bench write-up: per-suite leak and `acc_deleaked` delta for
  each lane.

## T3 landed (2026-09-24)

- **Runner wiring** (`src/harness/runner.rs`, all under
  `cfg(all(feature = "slice_leak", not(wasm32)))` at the run() boundary):
  after `prepare()`, the scan runs over the slices the run ACTUALLY
  serves — reference = the train split (the corpora draw from
  `train[cal_cap..]`, the calibration slice IS `train[..cal_cap]`; the
  union is the whole split — the probe's exact reference side), query =
  the BUILT eval cases whose raw dataset text is recovered by
  `eval_case_text` (the per-suite builder-key rule: `article` / `text` /
  `message` / `utterance` / premise+"\n"+hypothesis — the same field
  `train_docs` reads corpus-side, so the scan compares text-to-text).
  A missing key (state-shape drift) is a LOUD `errors.push` + no leak
  block, never a clean scan. Scope: dataset suites only —
  `typed_decisions` (templated) and the synthetic/code families are out;
  the scan prints one `leak:` line per suite. Semantics = the probe's
  (EXACT over all eval rows, NEAR over the first `near_cap`=2000), so
  the runner counts sit at or below the probe's train-vs-test bound by
  construction (the registry test caps only shrink the query side).
- **Schema** (additive, both `skip_serializing_if = None`):
  `SuiteResult.leak: Option<SuiteLeak>` (`threshold` / `n_reference` /
  `n_eval` / `exact` / `near` — counts, never a rate) and
  `LaneResult.acc_deleaked: Option<f64>` on every lane (modelless +
  laya riir + laya-python — the assembler threads it; the ANE bucket
  skips remap the flags through the served original indices so the
  served slice and the mask stay index-aligned, asserted).
- **`subset_accuracy`** (`metrics.rs`, pure): the hard walk restricted
  to the unflagged cases, `flagged_case` semantics (true = drop — ONE
  meaning shared with `scan_eval`'s flags so no call site can invert
  it), alignment asserted both ways, `None` when every row is flagged.
- **The live arithmetic check caught a real inversion bug.** The first
  wiring passed the flags straight into a keep-semantics parameter:
  prompt_injections measured `acc_deleaked = 0.5` over `kept=2` — the
  scan had KEPT exactly the two LEAKED rows. The headline's own numbers
  refuted it (51/116 = 0.4397 cannot yield 57/114): kept-correct can
  never exceed total-correct, so the flag semantics had to be inverted
  somewhere. Fixed by making the API carry the flag semantics directly
  (the drop-semantics rename + the negate inside). Post-fix:
  `acc_deleaked = 50/114 = 0.43860` — one flagged row was correct, one
  wrong; the de-leaked read is honestly LOWER than the headline on this
  suite. Lesson: a subset metric must reconcile arithmetically against
  its own headline before it is believed.
- **New primitive `scan_eval`** (`slice_leak.rs`): the per-row-flag
  form of `leak_counts` (`EvalScan` = counts + flags), pinned equal to
  `leak_counts` by a unit arm on the same fixture; rows beyond the NEAR
  window are UNSCANNED (kept in the de-leaked set, conservatively),
  never counted as clean.
- **Gates (M3, AC, release):** the two new oracle arms —
  `t3_runner_wiring_reproduces_the_probe_over_built_cases` (all 7
  in-scope suites, built via the pub builders at max_rows=0 = the
  probe's query set: counts EQUAL the probe exactly — ag_news 1/26,
  banking77 4/61, massive 13/49, prompt 0/2, sst5 1/0, emotion 0/0,
  xnli 0/0; per-suite scan 3.6–54 ms against the 1 s budget) and
  `t3_registry_cap_keeps_the_report_at_or_below_the_probe_bound`
  (ag_news at the registry cap 400: exact 1 ≤ 1, near 26 ≤ 26). Lib
  67/0 (`--features slice_leak`), full workspace suite green at BOTH
  default and slice_leak postures, clippy `-D warnings` clean at
  default / `slice_leak` / `--no-default-features --features
  slice_leak` `--all-targets`. **G3 verified empirically**: feature OFF
  vs ON runs of `--suites prompt_injections` — the OFF output carries
  no leak keys, and stripping the two leak fields from the ON output
  leaves the suites arrays equal except the timing fields (which vary
  between any two runs).
- ⚠ **Not validated here**: the laya-lane `acc_deleaked` path compiled
  but could not be clippy-verified at landing — the riir-infer sibling
  checkout carries live ANE-lane WIP (uncommitted type churn in
  `riir-infer-laya`), so the reflex `--features laya-riir` clippy lane
  is red on the SIBLING'S tree, not on this change (reflex-side files
  are clean at every posture; the laya plumbing mirrors the modelless
  path through the same `subset_accuracy`). Re-run the laya clippy
  postures once the sibling lane lands.

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
