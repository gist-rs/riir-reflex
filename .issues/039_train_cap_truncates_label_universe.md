# Issue 039 — the 4000-row train cap truncates the label universe on label-sorted mirrors (banking77 32/77, massive 42/60): modelless rows were inflated

**Status:** OPEN — filed 2026-09-26 from the Issue 038 T2 run. Measured; the repair (full train pull, T2) is in flight under Issue 038 / Bench 051.

## Finding

`scripts/fetch_datasets.sh` pulls the first 4000 train rows per suite. The
HF mirrors of `mteb/banking77` and `mteb/amazon_massive_intent` are
**label-sorted**, so the pull covers a label PREFIX:

| suite | labels in full train | labels in the 4000-row pull | labels in the harness test sample | test-sample labels inside the pull |
|---|---|---|---|---|
| banking77 | 77 | **32** | 13 (first 500 test rows) | 13 / 13 |
| massive_intent_en | 60 | **42** | 30 (first 300 test rows) | 30 / 30 |
| ag_news / emotion / sst5 / xnli_en | all | all | all | — |

The questions still offer every label, but the labels absent from the pull
get only the self-doc fallback corpus (the label name itself) and almost
never win. The modelless engine was therefore choosing among ~32 (banking77)
/ ~42 (massive) plausible labels, not 77 / 60. **The laya lanes read no train
rows and were unaffected**, so the comparison was tilted in modelless's
favour on exactly the two suites it "won".

Measured under Issue 038 (same code, `--head-select --nb-select`):

| suite | 4000-row pull (inflated) | full pull (fair) | laya best |
|---|---|---|---|
| banking77 | 0.8700 | **0.7700** | 0.4980 |
| massive_intent_en | 0.9267 | **0.8967** | 0.7500 |

Both still beat laya at the fair posture. The previously published Bench
045 / 040 modelless rows (banking77 0.6840, massive 0.7933) carry the same
inflation and should be re-read at the full pull.

A second, lane-fair weakness: the harness test sample is the first N rows
of a label-sorted test split, so banking77's 500 cases span only 13 of 77
labels. Every lane sees the same cases, so the comparison stays fair, but
the number is not a representative banking77 score.

## Plan

- [ ] **T1** — publish at the full train pull (Issue 038 T2: `TRAIN_CAP` /
      `--datasets-dir`), with the manifest (`.docs/02_protocols/dataset_manifest.md`)
      re-recorded for the new pull. The 4000-row posture is withdrawn for
      these two suites.
- [ ] **T2** — a representative test sample for label-sorted splits: a
      seeded stratified sample across the whole test split instead of the
      first N rows. This changes every lane's cases, so it needs a full
      both-hosts re-run of all lanes, laya included.
- [ ] **T3** — a harness guard: refuse (or loudly disclose) a run where a
      label offered by the questions has no train docs in the corpus pool
      (the self-doc fallback count is already known at build time).

## References

- Issue 038 (the T2 run that exposed it), Bench 051.
- `src/harness/runner.rs` `build_engine_with` (self-doc fallback),
  `src/harness/suites.rs` `sampled()` (first-N test rows).
