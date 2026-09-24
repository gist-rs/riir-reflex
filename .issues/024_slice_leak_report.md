# Issue 024 — `slice_leak`: an in-harness near-duplicate leak report (Issue 007 P2's decided route)

**Status:** OPEN — filed 2026-09-24 from Issue 007 P2's decision. The owner delegated the route call. The trigger was measured and has fired. No Rust code has landed.

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

- [ ] T1: `slice_leak.rs` index plus unit tests. Arms: a planted paraphrase
  is flagged; a disjoint topic is not; the rare-cap skip holds; the
  exact-normalisation path works.
- [ ] T2: the G1 oracle test against `scripts/slice_leak_probe.py`, with a
  loud skip when `.raw/datasets` is absent (UNSEEN, never a pass).
- [ ] T3: runner wiring over the real slices, plus the `leak` block in
  `results.json`.
- [ ] T4: site columns in `bench.json`. These ride the NEXT Issue 018
  two-host publish; do not force a separate one.
- [ ] T5: a Bench write-up: per-suite leak and `acc_deleaked` delta for
  each lane.

## Notes

- Sequence after Issue 023 T5. That publish is already queued behind the
  4090 run, and a schema change landing mid-publish is exactly the
  two-host disagreement Issue 018's publisher refuses.
- massive's 2 exact label conflicts are label noise in the source dataset.
  Report them; do not fix them here.
