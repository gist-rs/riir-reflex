# Bench 108 — the families wide-eval BOARD REFRESH (the main /bench/ board stops showing the old 16-case rows)

**Date:** 2026-10-02 · **Host:** m3 (m3-max-metal) · **Profile:** release · **reflex HEAD:** `8ca8770`

## Why

Plan 009 / issue 059 (2026-10-02) widened the five harness families' evals to
96–100-case template-disjoint populations and landed the quarantined
`/families/` site section with the frozen read (Bench 105) — but the MAIN
bench board (`reflex.gist.rs/bench`, `data/bench.json`) kept rendering the
OLD 12–16-case family rows (visibility modelless 0.5625 @ n=16) because the
last modelless harness doc feeding the board predated the rewire
(`19cb043`, 2026-10-01) and the 10-02 republish carried only the instinct
hybrid doc. Owner call: "i need new 96 cases not outdate 16 one" — this run
is that refresh. The old accuracies were corpus-overlap inflation anyway
(Bench 105's finding); the board now carries the honest reads.

## The run

```
LAYA_DEVICE=metal LAYA_PY_DEVICE=mps ./target/release/harness --laya-python \
  --suites harness_visibility,harness_permissions,harness_tool_fit,harness_routing,harness_sensitivity \
  --datasets-dir .raw/datasets --out .benchmarks/108_families_wide_board_refresh
```

PASSED — 5 suites, no absences. All three board lanes re-measured on the
wide populations (laya-riir metal + laya-python mps over system python3
torch 2.11; rust == py per suite, the lanes' own repeat check ✓).

| suite | n | modelless | laya-en (rust) | laya-en (py) |
|---|---|---|---|---|
| harness_visibility | 96 | 0.3125 | 0.6563 | 0.6563 |
| harness_permissions | 96 | 0.3125 | 0.2813 | 0.2813 |
| harness_tool_fit | 96 | 0.3021 | 0.5104 | 0.5104 |
| harness_routing | 96 | 0.2813 | 0.6042 | 0.6042 |
| harness_sensitivity | 100 | 0.2200 | 0.6000 | 0.6000 |

`harness_cache_reuse` is NOT re-run — its eval is the frozen T3 12-fixture
record (the documented divergence, Plan 009); its board cells are current.

Modelless visibility 0.3125 == Bench 105's frozen read exactly (the frozen
read reproduces; the harness path and the /families/ readout agree).

## Box state (the Issue-021 disclosure)

AC power, 100% charged, power mode high — but **load 7.79 → 9.50** (sibling
jobs): `latency_quotable: false` in the doc's own box_state, refusals named.
Accuracy and determinism are load-independent; the latency cells publish by
acknowledgement (`PUBLISH_BENCH_ALLOW_UNQUOTABLE=m3`) — the same posture as
the incumbent cells (the 10-01 modelless lane measured at load 6.31/6.97).
Re-measure latency on a quiet box before quoting any of these p50/p99.

## The publish mechanics (what made this non-trivial)

- The publisher's population guard correctly refused a plain lane-update:
  `excluded_suites: harness_visibility (nq 96 vs primary 16)` … — a
  population change is not a lane-update. The documented escape is the
  **`PUBLISH_BENCH_POPULATION_RESET`** ack (reflex issue 044 T4): it
  re-pinned the five rows to the wide populations, dropped every
  stale-population lane slot the update did not re-declare (the 4090-win
  family lanes, the hybrid/encoder family cells — they measured the OLD
  questions; the /families/ page carries the hybrid wide numbers from
  instinct Bench 0051), and landed the fresh modelless + laya + py lanes.
- The ack is staleness-adjudicated: the SECOND publish attempt with the
  same env refused with "a reset ack cannot outlive the population change"
  — the guard working as designed (the first attempt had already landed).
- En-route fixes in reflex-site (same landing):
  - `scripts/republish_bench.sh`: `PUBLISH_BENCH_POPULATION_RESET` joins
    the self-test's `env -u` list (the ack leaked into the self-test's
    stale-ack case and killed step 1 — the exact Issue-057/058 leak class,
    one env over).
  - `scripts/publish_bench.py` DISCLOSURES for the six families: the stale
    "n=12–16 template-shared eval" wording replaced with the wide-eval
    posture (Plan 009 / issue 059; instinct re-baseline Bench 0051; the
    encoder lane's honest `not run` owner call).

## Numbers moved (modelless, old → new)

| suite | old (n) | new (n) |
|---|---|---|
| harness_visibility | 0.5625 (16) | **0.3125** (96) |
| harness_permissions | 0.6667 (12) | **0.3125** (96) |
| harness_tool_fit | 0.9167 (12) | **0.3021** (96) |
| harness_routing | 0.7500 (16) | **0.2813** (96) |
| harness_sensitivity | 0.8000 (15) | **0.2200** (100) |

The drops are the point: the old numbers were corpus-overlap inflation
(Bench 105: the old evals shared 68–78% of their vocabulary with
corpus∪cal). The board's family rows now measure the same honest fixture
distribution the /families/ caveat page describes.

## Cross-refs

- reflex Plan 009 (`REVISED-2`) / issue 059 — the wide eval + the quarantined section
- Bench 105 — the frozen read the /families/ page serves
- instinct Bench 0051 — the hybrid A0 wide-eval re-baseline
- reflex-site: the board publish commit (see the site repo's log)
- `.benchmarks/.highwater`: 106 → 108 (dir `107_bekko_v0_400m_board` is a
  sibling session's in-flight allocation, on disk at write time)
