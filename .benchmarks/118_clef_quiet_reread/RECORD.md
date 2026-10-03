# Bench 118 — the Clef lane's quiet-box latency re-read (plan 011 C1's deferred item)

**Status:** MEASURED — both full-N cells re-read on a preflight-clean box; latency QUOTABLE both ends;
the site cells graduate from `acc-only`.

## Why

Bench 113's full runs landed under a dirty box (load 11.03 → 18.10; `bench_preflight` REFUSED at
load 6.36 — the run proceeded as accuracy-only with every ms figure PROVISIONAL). The published
clef cells carried `clef:acc-only` and the site's p50 presence-row pin read "acc-only re-read
pending". This is that re-read.

## The runs

- PROVENANCE (the harness's own Issue-021 spans, quoted): b77 start load **2.57** → end **1.07**
  (swap 29.4 → 15.5 GB, drained during the run); typed start **1.22** → end **2.80** — **latency
  QUOTABLE both ends, both suites**; AC Power, powermode 2 (high) throughout.
- Posture unchanged from 113: `mlx-community/clef-flash-4bit` (community 4-bit 9B) served by the
  standing local rig (`.raw/clef_srv/clef_lane_server.py` @ `127.0.0.1:8793`,
  `CLEF_RUN_PATH=/v1/clef-lane` — the flat local route), the harness's clef lane over the same
  wire. Full-N: banking77 500 cases / 500 q · typed 400 cases / 2000 q
  (`CLEF_SMOKE_MAX_CASES=1000`, the local no-spend ceiling raise).
- Binary: HEAD `91dd0cb` (release). Host m3.

## Rows (the tables carry the full metric tail)

| suite | acc | macro F1 | ECE(maxp) | p50 | p99 | det |
|---|---|---|---|---|---|---|
| banking77 | **0.9540** | 0.9536 | 0.0349 | **3368 ms** | 4371 | ✓ |
| typed_decisions | **0.6955** | 0.6742 | 0.0215 | **1690 ms** | 2554 | ✓ |

**Determinism witnesses:** accuracy, ECE, macro-F1 and the population digests reproduce Bench
113's full runs EXACTLY (b77 `fnv1a64-dd8ab35333abb82a` · typed `fnv1a64-6e37760ee2a5b6c9` — the
same pins the crosswalk asserts). The lane's byte-deterministic reply shape held across both runs.

**Latency vs the provisional cells:** b77 p50 3901 → **3368 ms** (−14%), typed 2377 → **1690 ms**
(−29%) — the loaded-box figures were inflated exactly as the Issue-021 law predicts; the quiet
numbers are the quotable ones (and remain 9B-4-bit-on-M3 loopback HTTP — never pooled with
hosted rows).

## What this unblocks

- The site's clef cells publish with timing (dropping `acc-only`); the p50 presence-row pin
  retires (clef wears the row).
- Plan 011 C1's deferred latency caveat closes; C4's deciding cell remains the hosted 27B
  (owner-gated, A6).
