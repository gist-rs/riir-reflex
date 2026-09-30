# Bench 098 — the m3-ane laya:english re-measure: every published ANE latency cell becomes quotable (reflex-site Issue 003 T3)

**Status:** COMPLETE 2026-09-30 — the deferred T3 re-measure, unblocked by two
prerequisites clearing: (a) the `ane_prefill` sibling WIP landed (riir-infer
tree clean), and (b) the ANE lane itself was found BROKEN at HEAD and fixed
first (see the regression record below). Fresh run under a PASSING preflight;
all five published m3-max-ane `laya/english` cells now carry quotable timing
with **bit-identical accuracy** at HEAD.

## PROVENANCE (the Issue-021 box-state law)

```
PROVENANCE: power=AC Power load=5.24 swap=388.06M canary=117.1us/best5 powermode=2(high)
```

Run spans (results.json `meta.box_state`): start load 5.04 / end 4.64 —
`latency_quotable: true` BOTH spans. Tree at `2f6b58c` (HEAD). Serialized
(ANE-only process; no Metal instance shared). The original 2026-09-24 run
(`afacc3a`, load 7.08/5.92) predated the publish wall — its cells were
stamped not-quotable by the 2026-09-27 backfill and carried unfit through
every LANE_CARRY update since.

## The en-route regression (found + fixed before the run)

The G5-ANE parity gate — run first per the lane-change law — was RED at
HEAD: `ane manifest multilingual/table_e8: missing bucket_L`. Root cause:
reflex `6535b75` (2026-09-26, the KV-table lane's e8 table converter,
Plan 612 Phase 1) added `<model>/table_e8` sidecar rows to the SAME
`assets/ane/manifest.json` the ANE lane parses, and riir-infer's
`AneManifest::load` required the artifact schema (`bucket_L`, `outputs`,
`placement`) on EVERY row — violating its own documented "the conversion
tool can grow the schema freely" contract. **The ANE lane had been
unloadable at HEAD for four days** (nothing ran the gate in that window).
Fixed substrate-side (riir-infer `7eae0e7`): the loader parses only
`<model>/L<n>` artifact rows, skips foreign rows; regression pin
`manifest_load_skips_foreign_non_bucket_rows`. G5-ANE parity re-run GREEN
after the fix (231 s, all three checkpoints).

## Results — m3-ane laya:english (fresh vs the carried-unfit incumbent)

| suite | p50 ms (was) | p99 ms (was) | acc (was) | n |
|---|---|---|---|---|
| ag_news | 28.0 (28.0) | 34.0 (34.0) | 0.94521 (0.94521) | 365 |
| emotion | 19.0 (19.0) | 24.0 (26.0) | 0.5975 (0.5975) | 400 |
| sst5 | 23.0 (24.0) | 27.0 (27.0) | 0.37333 (0.37333) | 600 |
| prompt_injections | 23.0 (23.0) | 30.0 (652.0) | 0.69159 (0.69159) | 107 |
| xnli_en | 28.0 (28.0) | 32.0 (34.0) | 0.85906 (0.85906) | 298 |

Readings:
- **Accuracy is bit-identical on all five suites** — the ANE lane's
  decisions did not move at HEAD; only the box verdict changed. The
  published accuracy cells were never wrong, only their timing provenance.
- Timing is stable (p50 within 1 ms everywhere); the one visible p99
  improvement (prompt_injections 652 → 30) is the original run's
  cold-compile outlier — this run's compiled-model cache was warm.
- Bucket-limit absences (35/9/2 cases on ag_news/prompt/xnli) unchanged —
  the lane's known coverage limit, listed per case in TABLES.md.

## Posture

- Suites: exactly the five with published m3-ane cells
  (`--suites ag_news,emotion,sst5,prompt_injections,xnli_en`); the board
  shrink (reflex-site `7eeb936`) dropped the other served suites' ANE
  cells — re-adding them is a board decision, not this bench's.
- Modelless byproduct (bare flags) NOT published — the update is
  lane-scoped (`PUBLISH_BENCH_LANES=laya`), the paw-lane law.
- Publish: reflex-site update path (current bench.json as primary, this
  doc as extra) — `carry_beats_incumbent` (Issue 003 T2) replaces the
  unfit incumbent's carried timing with this quotable run.
