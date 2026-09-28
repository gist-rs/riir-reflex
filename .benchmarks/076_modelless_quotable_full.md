# Bench 076+077 — the m3 modelless re-run at HEAD: every unfit published latency cell becomes quotable (reflex-site Issue 003 T1)

**Status:** COMPLETE 2026-09-28 — two coordinated runs, one engine state:
076 = m3 (fresh QUOTABLE timing + HEAD accuracy), 077 = 4090-windows
(HEAD accuracy, cross-host bit-identity re-proven). The publish
(`reflex-site`, lane-scoped update) replaced every carried-unfit m3
modelless timing cell via `carry_beats_incumbent` (Issue 003 T2's landed
path). m3 modelless unfit cells on the site: 26 → 11 (the residual 11 are
the m3-max-ane laya:english cells — Issue 003 T3, deferred: ane_prefill is
sibling WIP and the drop option is owner-gated).

## Why a second host had to run (the constraint T1's note anticipated)

The m3 modelless accuracy cells had moved at HEAD (the 072-era
count-table-ladder changes re-roll the cal-selection on several suites), and
the modelless lane's cross-host bit-identity claim (Issue 018 T7) is gated on
the FINAL merged state — publishing fresh m3 numbers beside stale 4090-win
numbers REFUSES. The 4090 re-run is the resolution the publisher's own gate
names: same sha, same flags, same datasets on both hosts.

## PROVENANCE (the Issue-021 box-state law)

- 076 (m3): preflight PASSED at the run window — load 5.17→5.82 across the
  run, `latency_quotable: true` BOTH spans. Tree at `c464a8a` (HEAD).
- 077 (4090-windows): the host has no box probes — `box state UNJUDGED`,
  disclosed per lane (the standing 4090 posture). Its TIMING publishes as
  unjudged; only the ACCURACY half is load-bearing here.
- Scout-run lesson recorded: a first 076 attempt was discarded — the
  `cargo run` build itself pushed load past the ceiling mid-run (box read
  8.62 at both spans, NOT QUOTABLE) AND the flags were bare (missing the
  published posture). Pre-build, preflight, then run — build churn is a
  box-state hazard, never part of the timed window.

## The exact posture (the published one — 066's lane posture, cascade absent)

```
cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --out .benchmarks/<dir>
```

Bare flags were measured first (the scout) and read ag_news 0.405 vs the
published 0.8825-posture — the selection ladders ARE the published modelless
posture (instinct Bench 005's finding, re-confirmed here at HEAD).

## Results — m3 (076): fresh quotable timing on all 15 suites

| suite | p50 ms (was, carried-unfit) | acc (new) | acc (was) |
|---|---|---|---|
| typed_decisions (2000 q) | 0.517 (0.539) | 0.4655 | 0.4655 |
| ag_news | 0.154 (0.167) | 0.8625 | 0.8825 |
| emotion | 0.119 (0.118) | 0.77 | 0.885 |
| sst5 | 0.097 (0.097) | 0.2017 | 0.3967 |
| prompt_injections | 0.076 (0.056) | 0.7672 | 0.7672 |
| xnli_en | 0.090 (0.088) | 0.5033 | 0.5233 |
| massive_intent_en | 0.092 (0.088) | 0.4067 | 0.78 |
| banking77 | 0.350 (0.353) | 0.402 | 0.826 |
| code_fixtures | 0.226 (0.226) | 0.375 | 0.375 |
| harness_visibility | 0.010 (0.009) | 0.5625 | 0.375 |
| harness_permissions | 0.007 (0.006) | 0.6667 | 0.4167 |
| harness_tool_fit | 0.008 (0.008) | 0.9167 | 0.5 |
| harness_routing | 0.009 (0.008) | 0.75 | 0.4375 |
| harness_sensitivity | 0.008 (0.008) | 0.8 | 0.4 |
| harness_cache_reuse | 0.009 (0.009) | 0.9167 | 0.9167 |

⚠ **The accuracy deltas are an ENGINE MOVE, not a re-measurement claim.**
The published cells pre-dated the 072-era count-table-ladder changes; at
HEAD the cal-selection picks different winners on 11 suites (datasets moved
DOWN: ag_news −2.0 pt, emotion −11.5, sst5 −19.5, massive −37.3, banking77
−42.4 — the ladder now selects postures that calibrated better on the cal
slice and generalize worse on test; the harness FAMILIES moved UP — the
count-table lever now arms there: visibility +18.8, permissions +25.0,
tool_fit +41.7, routing +31.3, sensitivity +40.0). The published page shows
the HEAD truth on both hosts with per-cell `source_run c464a8a` stamps.
The timing is the point of T1 and it is quotable everywhere; the accuracy
movement is disclosed, never hidden.

## Results — 4090-windows (077): 15/15 cross-host bit-identity at HEAD

Every suite's modelless accuracy is byte-identical between 076 (m3) and 077
(4090) — the Issue 018 T7 claim re-proven on the current engine. Two
box-state/dataset defects were found and fixed en route (both the
"measure twice" class — the FIRST 077 run could not see them):

1. **xnli_en absence**: the 4090's `.raw/datasets/xnli_en` had no usable
   train pages (3 of 40) → "the train split produced no corpus docs". The
   datasets dir the harness actually reads is `.raw/datasets` (the DEFAULT
   — `runner.rs` `DEFAULT_DATASETS_DIR`), not `datasets_t20k`; a first
   repair copied xnli into `datasets_t20k` and the absence persisted.
   Copied the M3's `.raw/datasets/xnli_en` → PASSED.
2. **massive_intent_en 1-pick divergence** (0.41333 vs 0.40667): the 4090's
   massive train pool was an OLD partial fetch — 26 train pages vs the
   M3's 40 (first 6 hashes identical, truncated set). Corpus pool differs →
   one boundary pick flips. Copied the M3's train pages → bit-identical.
   The test pages were already identical (30/30 MD5s) — only the TRAIN half
   (the corpus pool's source) had drifted, which is why the eval-side
   digest checks never caught it.

3. **`meta.host` = "unknown"** on the first 077: `uname -n` returns nothing
   on the Windows box, so the host label fell to "unknown" without
   `REFLEX_BENCH_HOST`. Re-ran with the override; the publisher's alias map
   (`4090-windows` → `4090-win`) filed the lanes into the existing host
   container.

## Republish (reflex-site, lane-scoped)

`scripts/republish_bench.sh data/bench.json <076>/results.json <077>/results.json`
— current published doc as PRIMARY (the wholesale-drop guard demands it:
fresh modelless-only docs would silently drop 122 carried lane slots),
the two new docs as lane-scoped updates. All gates green: self-test 51/51,
chart smoke, the cross-host drift gate (now satisfiable — both hosts at
HEAD), pairing gate, mirror parity, bench-page smoke ("11 unfit cells in
the data", all T3's ANE rows).
