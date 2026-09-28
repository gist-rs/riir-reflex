# Bench 075 — the Thai-board latency re-run: openthai p50 becomes quotable (Plan 003 T3.5)

**Status:** COMPLETE 2026-09-28 — T3.5 measured; the one command was
`scripts/thai_rerun.sh` (landed `70993ab`). This is a RE-RUN on a fit box,
never a re-judgment of 074's verdict (that run's start-load 6.95 state
stands in its own record).

## PROVENANCE (the Issue-021 box-state law)

- `scripts/bench_preflight.sh` pre-gate at launch:
  **`PROVENANCE: power=AC Power load=5.38 swap=1866.19M canary=129.6us/best5 powermode=2(high)`**
  — preflight PASSED (load 5.38 < ceiling 6.0; on AC 58 min; powermode 2 =
  High Power; canary 129.6 µs vs ref 141 µs).
- The harness's own `meta.box_state`: start load **4.94**, end load **5.43**,
  `latency_quotable: true` on BOTH spans, refusals `[]`. The timing cells
  below are QUOTABLE — the first Thai latency numbers the arena publishes.
- Reflex tree at `70993ab` (the T3.5 one-command runner itself).

## Results — accuracy reproduces 074 exactly; latency is the new content

| suite | modelless | laya-multilingual | openthai |
|---|---|---|---|
| thai_wisesight (400) | 0.1225 · p50 **0.113 ms** | 0.4075 · p50 **7.0 ms** | 0.4750 · p50 **111.0 ms** |
| thai_sib200 (204) | 0.1422 · p50 **0.185 ms** | 0.7843 · p50 **8.0 ms** | 0.8382 · p50 **115.0 ms** |

- **Accuracy: byte-identical to 074** on every lane/suite cell (deterministic
  sampling law + the specialist's `permutations=1` pin) — the re-run changed
  NOTHING about the verdicts, only their quotability. ECE also reproduces
  (wisesight openthai 0.3612, laya 0.2932; sib200 openthai 0.0671, laya
  0.1589, modelless 0.0213).
- **Latency (the payload of this run):** openthai p50 **111 / 115 ms** vs
  laya-multilingual p50 **7.0 / 8.0 ms** — the specialist answers Thai at
  ~14–16× the specialist's latency (111/7.0 ≈ 15.9, 115/8.0 ≈ 14.4), and the
  modelless lane's p50 (0.11–0.19 ms) is ~600× under openthai's. 074's
  order-of-magnitude guess ("~11×, 110 vs 10 ms") was right in shape; the
  quotable split is now on the chart.
- Verdict shape unchanged from 074: openthai leads Thai accuracy (0.475 /
  0.838), laya-multilingual is the best OUR stack answers (0.408 / 0.784),
  modelless abstains by the pinned empty-bag law. Plan 003 stays
  research-sake — no engine work re-opens (T4.2's owner trigger did not
  fire).

## Determinism

The openthai lane's pin (`permutations=1`) green; the accuracy/ECE
byte-identity with 074 across a different box-load window is itself the
strongest repeat evidence the board could carry. Modelless bit-identity
green. Laya observed-repeat ✓ (reported, not claimed).

## Artifacts

- `075_openthai_thai_rerun/TABLES.md` + `results.json` — the re-run board.

## Verdict

- T3.5: **MEASURED, latency QUOTABLE** (box_state both spans true, zero
  refusals). The thai p50 bars publish to reflex-site via
  `PUBLISH_BENCH_LANES=openthai ./scripts/republish_bench.sh` (+ deploy).
- The research clone `.raw/openthai-systemone` is removed after landing (the
  global rule) — the runner re-stages it from the pinned sha if a future
  re-run ever needs it.
