# Issue 039 — the 4000-row train cap truncates the label universe on label-sorted mirrors (banking77 32/77, massive 42/60): modelless rows were inflated

**Status:** CLOSED 2026-09-26 — T1 done (Bench 051, full pull); T2 + T3 done and T4 adjudicated at Bench 052 (`.benchmarks/052_stratified_readout/`, stratified sample + complement pool + fallback guard; the readout-selection lever measured NEGATIVE and demoted to a report-only table — the wide-label G1 gap closed via the stratified cal slice instead; emotion's G1 honestly regresses, its 051 pass was a sampling artifact). Both-host note: the M3 metal run is published here; the 4090 re-run is queued as follow-up work (same protocol, `datasets_t20k` bytes already on that box).

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

Measured at the full pull (Bench 051), the **baseline** (no count tables)
is banking77 0.5940 and massive **0.7367, below laya's 0.7500**. The
published massive "win" was this artifact.

**Calibration too:** at the full pull the baseline FAILS G1 on massive
(cal 0.2600 vs floor 0.0964), banking77 (0.3938 vs 0.1689) and ag_news
(0.3797 vs 0.2506). The truncated universe had flattered the G1 verdicts.
The count tables flip ag_news to PASS; massive and banking77 stay FAIL.

## Plan

- [x] **T1** — DONE: published at the full pull (Bench 051, both hosts bit-identical, reflex-site `9b8223d`); full-pull digests recorded in `.docs/02_protocols/dataset_manifest.md`. Original: (Issue 038 T2: `TRAIN_CAP` /
      `--datasets-dir`), with the manifest (`.docs/02_protocols/dataset_manifest.md`)
      re-recorded for the new pull. The 4000-row posture is withdrawn for
      these two suites.
- [x] **T2** — DONE (Bench 052): a label-STRATIFIED round-robin test sample
      across the whole test split (`suites.rs::stratified_split` —
      first-appearance label order, dataset order within each label,
      deterministic, no RNG; budget-0 = identity so uncapped suites never
      move), applied to the cal slice too, and the corpus pool switched to
      the split's REST envelope (cal rows excluded by construction, not
      position — the positional cut was orphaning whole label blocks on
      clustered mirrors). M3 run published in `.benchmarks/052_stratified_readout/`
      (modelless + laya metal, all 15 suites): modelless ≥ laya on 5/8
      dataset suites (was 4/8) — banking77 0.8260 vs 0.4220, massive 0.7800
      vs 0.6933, emotion 0.7375 vs 0.5925, sst5 0.3967 vs 0.3717, prompt
      0.7672 vs 0.6983. Accuracy at the representative sample is HONESTLY
      harder on some suites (massive −11.7 vs the 30-label-prefix sample;
      prompt +28.5 — the old first-116 sample was label-clustered too).
- [x] **T4** — DONE AS MEASURED NEGATIVE (Bench 052): per-suite readout
      selection over {dispatch, max_prob, inv_entropy} by in-sample
      calibrated ECE on the cal slice (margin 0.005 over dispatch, min 32
      cal pairs). On the wide suites it targets, max_prob IS the dispatch
      wide arm (candidates coincide) and inv_entropy measured WORSE
      (massive 0.5519 vs 0.4618; banking77 0.8070 vs 0.7860) — no
      functional beats the shipped law where the gap was. Where a candidate
      won on cal (narrow suites, entropy→maxp), arming it overfit the cal
      slice and regressed emotion's test G1 → the lever is DEMOTED to a
      report-only candidate table (`readout_report`, never armed;
      `EngineConfig::readout` stays the opt-in knob). The wide-label G1
      gap CLOSED anyway via T2's stratified cal slice: 7/8 dataset suites
      PASS G1 at dispatch — massive 0.0796 vs floor 0.1209, banking77
      0.1740 vs 0.2887, ag_news 0.0025 vs 0.2482. The one honest
      regression: emotion FAILS G1 at the representative sample (cal
      0.2274 vs floor 0.1371; the 051 pass was a first-400 label-subset
      artifact; maxp would read 0.1506 — closer, still failing). The
      temperature-on-L1-scores candidate stays UNTESTED (`- [-]` defer
      below).
- [-] **T4' (deferred)** — the temperature candidate: a τ on the
      L1-normalized scores before the readout, selected on the cal slice.
      Unblocks only if emotion's (or another suite's) G1 failure matters
      for a published claim; the demoted selection machinery (`EngineConfig::readout`)
      is the wiring it would ride.
- [x] **T3** — DONE (Bench 052): the harness guard — `build_engine_with`
      returns the labels whose pool docs are EMPTY (self-doc fallback), the
      main suite build discloses them loud (stderr `[issue-039 corpus
      guard]` + `LaneResult.corpus_fallbacks` + a ⛔ markdown line).
      First-run findings: typed_decisions 1 starved label (a workflow whose
      rows all sit in the cal front), code_fixtures 4 (by construction —
      modules with <10 fns self-doc); all dataset suites otherwise clean at
      the full pull. Selection-slice builds ignore the report (the
      starvation there is by construction).
- [ ] **T5** — 4090 re-run at the Bench 052 protocol (both-hosts
      bit-identity for the new sampling law; `datasets_t20k` bytes already
      copied there; laya lanes included — every lane's cases changed).
      Queued as follow-up; the M3 metal run published above is the record
      of note.

## References

- Issue 038 (the T2 run that exposed it), Bench 051, Bench 052
  (`.benchmarks/052_stratified_readout/`).
- `src/harness/suites.rs` `stratified_split` (T2), `src/harness/runner.rs`
  `build_engine_with` fallback return (T3) + `readout_report_on_cal` (T4),
  `src/readout.rs` `ReadoutMode`.
- `.docs/02_protocols/dataset_manifest.md` §"Sampling law".
