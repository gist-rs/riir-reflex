# Issue 109 — Gate-fit threshold-stability audit (bootstrap t* + held-out flatness)

**Status:** OPEN — Gain (reflex Research 015 §2.2; offline, rides the deterministic calibration pass — per-case observations are in-memory today, no persistence path exists yet)
**Claimed:** zed-4090-idle-walk24sibling, epoch 1793893997, expires 2026-11-07T22:46:37Z

## Problem

Reflex fits fused-gate thresholds per suite at calibration ρ=30 (`--gate-fit-calibrated` is the DEFAULT since Issue 056), and rethink pins per-suite `ESC_GATE_RHOS` (the Bench-0057/0078 fits). Neither surface audits whether the fitted operating point is:

1. **Stable** — would resampling the calibration slice have produced a very different threshold?
2. **On a plateau** — is held-out performance flat around t*, or does everything hinge on a knife's edge that the next re-fit could fall off?

OntoSmith ships exactly this audit as a product surface (`pipeline/threshold_stability.py`): bootstrap the tuning procedure on dev (200 resamples) → distribution of t* (mean/median/SD/p5/p95 — tight distribution = stable selection), then a held-out F1(t) curve at fixed points around t* ("robust optimum = flat region, not a knife's edge"). Their `operating_points.py` adds the **vacuous-precision guard**: an operating point with `n_accept < max(10, 5% of dev)` is invalid, never a green zero — the same population-floor class as our green-zero laws.

## The ask

A report-only harness arm — **`--gate-stability`** — over stored calibration records:

1. Bootstrap the gate-fit threshold per suite (seeded, deterministic; iterations + seed pinned in the output). The substrate is the per-case `GateObservation` (`src/engine/threshold.rs` — one observation = the gate score + whether the engine's answer was correct), but it is built IN MEMORY inside the harness runner today (`runner.rs`, `density_ab.rs`, `gate_drift.rs` — no persistence path writes per-case observations to disk). So the arm **re-runs the deterministic calibration pass per suite** (cheap, no model calls — the same pass the gate fit already runs), collects the observations, and bootstraps them in memory; an optional first task may persist the observations beside each fitted record if a stored-record posture is wanted later. If a future posture stores only fitted constants and no re-run seam exists, the arm refuses loud rather than bootstrapping constants.
2. Report t* distribution (mean / SD / p5–p95) per suite.
3. Report held-out metric curve at fixed offsets around t*; flag suites whose curve is single-point-sensitive (e.g. ≥2× metric drop at the nearest fixed neighbor).
4. Vacuity guard: a resample yielding too few accepted cases at t* counts as unstable, never silently passes.

## Hard laws

- Report-only: no exit-code coupling, no auto-retune (a demotion or re-fit is a separate measured act — the Issue-056 default posture is not reopened by this audit).
- Seeded + deterministic: same records + seed → byte-identical report (the workspace determinism law).
- A knife-edge finding does NOT demote anything by itself — it names the suite and the risk; the re-fit decision is a separate gate with its own pre-registered bar.

## Proof path

First cell: run over the existing calibrated suites' stored records; bar = the report reproduces byte-identically across two runs and surfaces at least the suites whose t* SD > 0.05 (if none, that is a result, not a failure — the population floor is the number of calibrated suites audited, printed every run).

## References

- reflex Research 015 §2.2 (distill + prior-art note: threshold-selection robustness is standard statistics — no novelty claim, pure hygiene)
- OntoSmith @ `1412da4d` (MIT): `pipeline/threshold_stability.py`, `pipeline/operating_points.py`
- reflex Issue 056 lineage (gate-fit-calibrated default), rethink Bench 0057/0078 (`ESC_GATE_RHOS` fits)
