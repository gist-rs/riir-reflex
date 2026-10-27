# Bench 167 — Issue 969 T3: reflex `pick_domain` router-statistics liveness CONTROL — CONTROL LIVE

**Date:** 2026-10-27
**Issue:** katgpt-rs [`Issue 969`](../../katgpt-rs/.issues/969_kimi_k3_router_statistics_diagnostic.md) (Research 639 / arXiv:2610.31818 Nestor App C.6) — this is the control half (T3); the K3 subject arm is katgpt-rs [Bench 969](../../katgpt-rs/.benchmarks/969_issue969_router_stats.md).
**Instrument:** `harness --router-stats` (`src/harness/runner/router_stats_probe.rs` + the `Scratch::forced_domain` diagnostic seam in `src/engine.rs`); report + GATE (a lane-level RED refuses the run).

## Verdict

**CONTROL LIVE** — the shuffled-domain counterfactual clears the wall on 3 of 4 suites; domain routing is load-bearing on what ships. The K3 arm's subject statistics are read against a live instrument.

| suite | slots | N | eff | maxld | MI bits | base acc | shuf acc | acc drop | change | abst Δ |
|---|---|---|---|---|---|---|---|---|---|---|
| ag_news | 100 | 4 | 0.836 | 0.390 | 0.1399 | 0.3818 | 0.2308 | +0.1510 | 0.590 | +0.030 |
| emotion | 100 | 6 | 0.375 | 0.790 | 0.2808 | 0.3846 | 0.3548 | +0.0298 | 0.300 | +0.030 |
| sst5 | 100 | 5 | 0.594 | 0.650 | 0.1761 | 0.1250 | 0.1639 | −0.0389 | 0.000 | +0.030 |
| banking77 | 100 | 77 | 0.729 | 0.040 | 5.5910 | 0.5510 | 0.0000 | +0.5510 | 0.980 | +0.470 |

Posture: deployed engine build (fit_posture_inner + build_engine_with, selection levers pinned off, cal-slice `observe` calibration); whole-assignment permutation across each suite's slots (domain marginals exactly preserved); 100 stride-sampled cases/suite; host amo, sha `3025434`. Full meta in [`router_stats.json`](router_stats.json).

## Findings

- **The liveness gate (the control's purpose):** pick change under re-routing = 0.59 / 0.30 / 0.98 on ag_news / emotion / banking77 — domain routing provably reaches the scorer and moves answers. Near-zero on ALL suites would have been the instrument-blind refusal; it did not happen.
- **sst5 is the structural-zero class, not an instrument failure:** its candidates are 1-byte digit strings ("0".."4") — a candidate that short cannot carry a corpus-conditional compressed-length difference, so shuffling the routed corpus cannot move the drafter argmax. The seam itself provably engages even there: the distance gate keys on the forced domain and the abstain rate moves (+0.030). The lane-level gate keys on ANY suite clearing the wall; per-suite structural zeros are findings, named as such. (This is the engine's own Issue-004-T7 lesson — short options never move compressed length — now measured from the routing side.)
- **banking77's MI is 5.59 bits** — with 77 domains the routed domain nearly determines the gold label (H(gold) ≤ 6.27 bits); routing there is almost a classifier. Its 98% change rate + accuracy 0.55 → 0.00 under shuffle is the same fact from the counterfactual side.
- **Two-level scope note (the issue's hypothesis, measured):** the option spans all score against the ONE routed domain in the shipped engine — there is no independent level-2 assignment to shuffle, so the paper's per-level MI split collapses to level 1 here. Structural, recorded in the render.
- **Determinism witness:** the first 20 baseline slots of every suite decide twice and must agree — green on all four suites.

## What landed (reflex-side diff)

- `src/engine.rs` — `Scratch::forced_domain: Option<Vec<Option<usize>>>` + the routing-site override (byte-identical when disarmed; production paths never arm it).
- `src/harness/runner/router_stats_probe.rs` (+ `runner.rs` wiring) — the probe: level-1 effective choices / corpus MI / shuffled-domain counterfactual + the lane-level liveness gate + the determinism witness.
- `src/bin/harness.rs` — the `--router-stats` / `--router-stats-max-cases` exclusive early-exit mode.

## Cross-repo conclusion (Issue 969 T4)

Both arms measured on 2026-10-27: reflex control LIVE (this bench); K3's single routed level reads CORPUS_INDEPENDENT_BUT_LOAD_BEARING with a bimodal layer split (katgpt-rs Bench 969). The issue's verdict rows carry both arms' scope caveats verbatim.
