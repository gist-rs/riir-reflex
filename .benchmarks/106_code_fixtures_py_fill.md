# Bench 106 — code_fixtures laya-python fill (the frontier-coverage closure)

**Status:** COMPLETE — the cell landed and is published on reflex.gist.rs.

## Why

The /bench/ efficiency-frontier chart drew `laya (python)` at cc index 56.4%
vs `laya (rust)` 50.3% — a "quality win" for the torch reference over its own
riir port. Root cause: the python lane never ran `code_fixtures` (its
source run predated the suite joining the board), so its index was a mean
over 8 suites while rust's covered 9 — and code_fixtures is laya's worst
suite (cc 0.136). Same model, G5-identical outputs; the entire 6.1-point gap
was missing coverage. The chart's Pareto ring then made the unopposed
partial lane look frontier-worthy (renderer round-2/3 verdicts + a
shared-basis tooltip disclosure landed on the site side the same day).

## Run

- `REFLEX_BENCH_HOST=m3 LAYA_PYTHON=python3 LAYA_PY_DEVICE=mps` — the
  reference torch runtime (system python3, torch 2.11.0, MPS), JSONL
  subprocess lane, 16 cases / 32 questions.
- PROVENANCE: power=AC Power load=4.03 swap=2587.25M canary=118.7us/best5
  powermode=2(high) — start load 4.96 / end 4.73, **latency QUOTABLE**
  (the 09-27 attempt `044_code_fixtures_py2` measured the same cell but at
  load 10.52, NOT QUOTABLE — its accuracy matches, its timing could not
  publish).
- git sha `450a81c` @ 2026-10-02T09:03:17Z.

## Result

- laya-python english code_fixtures: **acc 0.4062** — byte-identical to the
  09-27 run AND to laya-riir english (third-posture G5 parity witness).
- p50 75.0 ms (subprocess round-trip, IPC included).
- Published (reflex-site `PUBLISH_BENCH_LANES="laya"` lane-scoped update):
  python lane index 56.4% → **50.3% == laya (rust) exactly**, coverage 9/9
  complete, timing geomean 47.8 → 50.2 ms over 9/9 quotable.

## En-route

- The harness built WITHOUT `laya-riir` (plain `--release --bin harness`);
  the doc's `laya_feature: false` does not clobber the host row (the merge
  keeps the original run's facts) and the py lane is subprocess-only — no
  effect on the cell.
