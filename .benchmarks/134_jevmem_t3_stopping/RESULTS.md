# Bench 134 — Jev-Mem stopping-controller sweep: the calibration target (Issue 081 T3)

**Status:** COMPLETE — the threshold-regime sweep over the full LoCoMo-10
matrix (1540 QAs, cats 1-4). The finding: at the published config the
traversal collapses to depth ≤ 1 on every query in every arm — the stopping
controller's only real decision is a **binary expand-once-or-not gate**, and
that is precisely the surface a fitted sigmoid should own.

## The cells (same graphs as Bench 133 — retrieval tuning reuses the cached graphs per their own `validate_reuse_memory` law)

| cell | thresholds (suff/cont) | pooled recall | cat1 | cat2 | cat3 | cat4 | mean edges | mean jev_calls | depth |
|---|---|---|---|---|---|---|---|---|---|
| mock base | 0.95 / 0.15 (published) | 0.7847 | .5445 | .8377 | .5298 | .8730 | 2109 | 3.97 | 1 |
| mock stop-1 | 0.85 / 0.40 (their dataclass posture) | **0.7847 — identical on every QA** | .5445 | .8377 | .5298 | .8730 | **1484 (−30%)** | 3.97 | 1 |
| reflex base | 0.95 / 0.15 (published) | **0.7921** | .5451 | .8598 | .5091 | .8801 | 1670 | 3.97 | 1 |
| reflex stop-0 | 0.48 / 0.53 (under the flat values) | 0.7503 (−4.2pt) | .4933 | .8113 | .4718 | .8436 | **0** | **1.99** | 0 |

## Findings

1. **The mock's extra traversal work buys exactly nothing.** Stopping after
   one expansion round (0.85/0.40) reproduces the base recall **bit-for-bit
   on all 1540 QAs** — same per-category means, same per-QA recall — while
   the base config's round-2 sweep burns +625 edges/query for zero gold
   evidence. Their own dataclass thresholds were already right; the published
   0.95/0.15 posture pays 30% more retrieval work for nothing.
2. **The traversal is depth-1 everywhere.** Every arm, every threshold
   setting, all 1540 QAs: `retrieval_depth ≤ 1` (anchors → one expansion →
   stop; the beam/graph budgets saturate after round 1). The multi-round
   adaptive traversal their controller design implies does not occur at this
   config — the real control surface is a single binary decision: expand the
   anchors once, or serve anchors-only.
3. **That binary is exactly where the value is.** reflex stop-0 (anchors
   only) halves decision traffic (1.99 calls) and zeroes edge sweeps but
   costs 4.2pt recall (cat1 −5.2pt, cat2 −4.9pt) — the one expansion round is
   load-bearing. A fitted gate that says "expand" only when the anchors look
   insufficient keeps the 4.2pt where it matters and skips the round (and the
   second stopping call) where it doesn't.
4. **Why no sigmoid fit ran (the honest T3 answer):** reflex's stopping
   values are domain-blind and flat — `evidence_sufficient` 0.489–0.495
   (stdev 0.001), `continue_useful` exactly 0.495 across 304 calls — there
   is no variance to calibrate. With flat values only two regimes exist
   (fire-at-0 / never-fire), both measured above. The mock's heuristic
   variance (0.3 at depth 0, 0.9 at depth ≥ 1 — a step function wearing
   threshold clothes) is what its 0.85 posture exploits.

## Report-the-Floor (the stopping decision)

- The floor is **mock stop-1**: full recall at 1484 edges — any fitted
  reflex stopping gate must match 0.7847@1484 or beat it (reflex base's
  0.7921@1670 sets the recall ceiling; the fitted target is
  **0.7921@~1200** — reflex recall at mock-stop-1-class cost).
- The gate shape to fit (one sigmoid, not per-round thresholds):
  `p(expand | anchor evidence)` over the anchor set's coverage signal —
  the corpus lever (T2's LoCoMo-derived supervision) is what gives it
  labels: anchors-sufficient QAs (recall@0 ≈ recall@1) → 0, the rest → 1.

## Determinism

All 20 sweep runs reproduced their printed means exactly on re-run (the
first sweep pass hit an out-dir layout bug that destroyed per-QA files after
printing; the re-run under the fixed layout reproduced every number — same
cached graphs, query-only passes).

## Reproduce

```sh
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm reflex --sample <N> \
    --jev-config .raw/jev-mem/config/jev_mem.json \
    --stop-threshold 0.48 --continue-threshold 0.53   # stop-at-0 regime
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm mock --sample <N> \
    --jev-config .raw/jev-mem/config/jev_mem.json \
    --stop-threshold 0.85 --continue-threshold 0.40   # stop-after-1 regime
```
