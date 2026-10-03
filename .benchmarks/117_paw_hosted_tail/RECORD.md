# Bench 117 — paw hosted tail: re-attempt #3, the hub recovered — all 6 remaining suites quotable (lane 9/9)

**Status: MEASURED (COMPLETE) — `scripts/paw_hub_probe.sh` exited 0 (their inference backend healthy again,
~03:2x +0700), preflight PASSED on a settled AC box (load 2.05, canary 120.9 µs, swap 16.7 GB disclosed), and
the 6 remaining hosted paw suites ran as per-suite docs, every one quotable at BOTH ends. The paw lane is now
9/9 `latency_quotable: true` (p50 geomean 1057.24 ms published) — 065 T2's hosted tail is closed.**

Date: 2026-10-04 (attempts #1/#2 same night; this window ~03:35–04:35 +0700) · host m3-max-metal · reflex `e45c828`

## Launch protocol (the re-attempt #2 law, executed)

1. `scripts/paw_hub_probe.sh` → exit 0 (HEALTHY; the faithful real-request probe, not the 422 validation-layer
   false-healthy that launched attempt #1 into the outage).
2. `scripts/bench_preflight.sh` → PASSED after the AC settle (first read REFUSED at 2 min < SETTLE_MIN=5 —
   a second power transition at 03:26:39 reset the clock; waited, re-ran, passed at 5+ min).
3. Per-suite docs, quick→slow (bank the fast cells while the hub is up; typed last — the longest exposure):
   massive_intent_en → emotion → ag_news → sst5 → banking77 → typed_decisions.

`PAW_COMPILER=paw-ft-bs48-20260530 PAW_COMPILE_ASYNC=1` + `--skip-laya --paw --nb-select --oc-select
--ridge-select --datasets-dir .raw/datasets_t20k` (the bench-116 flags verbatim; all programs cache-hit —
zero compile traffic). The failed attempt-#2 massive doc is preserved at
`per_suite/massive_intent_en_attempt2_failed/` (the outage's second-window witness).

## The 6 cells

| suite | n | acc | refusals | p50 | p99 | server p50 | wall | vs prior readings |
|---|---|---|---|---|---|---|---|---|
| massive_intent_en | 300 | 0.5133 | 118/300 | 1049.4 ms | 2345.3 | 91.0 ms | 344.8 s | == bench-116 local 0.5133 (identical picks); board hosted 0.51 |
| emotion | 400 | 0.4950 | 0/400 | 1047.6 ms | 1448.6 | 76.4 ms | 437.9 s | local 0.4900 (posture delta, +2 q) |
| ag_news | 400 | 0.7900 | 0/400 | 1049.5 ms | 1644.3 | 94.0 ms | 451.2 s | == board hosted 0.79; local 0.7925 |
| sst5 | 600 | 0.3983 | 0/600 | 1048.6 ms | 2102.5 | 76.4 ms | 663.7 s | == bench-116 local 0.3983 |
| banking77 | 500 | 0.4180 | 173/500 | 1059.2 ms | 1570.8 | 112.2 ms | 566.2 s | == local 0.4180; refusals 173 vs local 177 (the sampled class) |
| typed_decisions | 2000 | 0.5925 | 2/2000 | 1123.1 ms | 2108.0 | 94.6 ms | 2361.9 s | == bench-089 hosted 0.5925, refusals 2 — byte-consistent |

Every doc's modelless control == the published board (the drift-guard premise held on all six). Box state
judged quotable at both ends on every doc (loads 2.04–2.35; one sibling blip to 4.35 mid-banking77 stayed
under the 6.0 ceiling and both verdicts held).

## The one disclosure: massive_intent_en `determinism_ok: false`

The observed-repeat check (re-ask the same input, first 10 served) got a DIFFERENT raw output on a re-ask —
the paw lane's first recorded det ✗ (bench 116's three cells and this window's other five are all ✓). Their
hosted inference is "not promised deterministic" by the lane's own method string; their server-side p50 was
91 ms with a ~1 s queue/round-trip, so a re-ask landing on a different serving instance is the likely shape.
The doc's det column carries it inline; accuracy is unaffected (the FIRST answer scores, the re-ask is only
the witness).

## Publish (065 T4)

`PUBLISH_BENCH_LANES="paw"`, current `data/bench.json` as primary + the 6 docs (the update path; the 12
out-of-scope modelless/laya control slots dropped loudly, by design). Landed:

- `areas.timing.paw` = `{n_used: 9, n_unquotable: 0, n_unjudged: 0, p50_geomean_ms: 1057.24}` — was
  3/2/4 at 1046.65. The lane's hosted timing is fully plottable; the Issue-021 home summary now plots it.

## Smoke fallout (two real smoke defects fixed, not re-pinned around)

1. **`bench_page_smoke` hero break-sign derivation** — the hero renders ONE bar per (suite, lane KEY, host)
   picked by accuracy (`pickHost`), and the paw key matches BOTH postures. Counting raw cells over-counted:
   ag_news + typed_decisions pick paw LOCAL on accuracy (0.7925/0.5955 beat hosted 0.79/0.5925), so their
   ~1.05 s hosted p50s count in the per-suite tables (15 break signs) but never render a hero bar (13). The
   derivation now mirrors the page's pick law (same field order, same accOf, same key grouping); the tables
   check keeps the raw variant count — two derivations, two surfaces.
2. **`bench_page_smoke` unfit-cells enumeration** — missed the `clef`/`encoder` lane fields; the DOM marked 2
   encoder cells † (Rethink's typed + thai reads, `latency_quotable: false`) while the data count read 0. It
   passed before only because the 2 then-unfit paw cells padded the tolerance budget. Enumeration extended
   (clef + encoder, both host levels).
3. **`public_copy_gate`** — red on `data/changes.json` rows carrying "bench 118"/"bench 113" (internal ids in
   public copy; committed after that session's gate run). Reworded per the site's no-typed-record-ids law —
   the gate's case-insensitive ID_RE was correct all along; the rows were the defect.

Publisher self-test 84/84; `bench_page_smoke`, `chart_render_smoke` (p50 now 14 lanes), `home_page_smoke`,
`arena_demo_smoke`, `public_copy_gate` all PASS after the fixes.

## What remains (065)

- T3 (the 4090-hosted lanes' judgability) — owner-gated as recorded (road (a) has nothing to point at; road
  (b) needs the owner's Windows probe pass).
- T5 close-out — every row of the issue's table is now 100% quotable EXCEPT the 4090-hosted lanes (T3's
  owner call) and clef's remaining 13 acc-only cells (plan 011's time-not-money tail).
