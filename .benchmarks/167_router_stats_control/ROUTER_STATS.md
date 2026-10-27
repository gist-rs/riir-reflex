# Issue 969 T3 — reflex `pick_domain` router-statistics control

- date: 2026-10-27T17:50:08Z · host: amo · sha: 3025434
- datasets: /Users/katopz/git/riir-reflex/.raw/datasets
- posture: deployed engine build (fit_posture_inner + build_engine_with, selection levers pinned off, cal-slice observe calibration); shuffled-domain counterfactual through the Scratch::forced_domain seam; whole-assignment permutation, marginals preserved
- control wall: pick change rate ≥ 0.05 (below = RED, instrument-blind)

| suite | slots | N | eff | maxld | MI bits | base acc | shuf acc | acc drop | change | abst Δ | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ag_news | 100 | 4 | 0.836 | 0.390 | 0.1399 | 0.3818 | 0.2308 | +0.1510 | 0.590 | +0.030 | LIVE |
| emotion | 100 | 6 | 0.375 | 0.790 | 0.2808 | 0.3846 | 0.3548 | +0.0298 | 0.300 | +0.030 | LIVE |
| sst5 | 100 | 5 | 0.594 | 0.650 | 0.1761 | 0.1250 | 0.1639 | -0.0389 | 0.000 | +0.030 | RED |
| banking77 | 100 | 77 | 0.729 | 0.040 | 5.5910 | 0.5510 | 0.0000 | +0.5510 | 0.980 | +0.470 | LIVE |

**Control verdict: LIVE (the liveness gate holds: domain routing is load-bearing on what ships)** — 1 suite(s) under the wall (sst5, named structural-zero class when live), 3 of 4 suites live.

Scope note: the option spans all score against the ONE routed domain in the shipped engine — there is no independent level-2 assignment to shuffle, so the paper's per-level MI split collapses to level 1 here (structural, measured).
