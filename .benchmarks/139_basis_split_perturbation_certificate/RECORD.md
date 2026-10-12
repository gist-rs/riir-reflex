# Bench 139 — basis_split Thm-3 perturbation certificate (reflex Issue 086 T3)

**Date:** 2026-10-12 · **Host:** shikuwa (4090 box, Windows) · **Reflex sha:** `1d396d1` + the T3 lane in this commit
**Katgpt-core:** origin/develop `3de81792e` (the 931-fixed `basis_split`) — this box's `../katgpt-rs` checkout was 21 behind with a sibling lane's WIP, so the run rode a worktree cluster (`wt-t3-*`); the recorded `git_sha` (`1d396d1`) is the reflex HEAD, the katgpt pin is disclosed here.
**Suites:** typed_decisions · banking77 · ag_news · emotion (bench 138's four, for continuity)

**Command** (release, basis_cert):

```sh
cargo run --release --features basis_cert --bin harness -- --perm-probe --basis-split \
  --skip-laya --suites typed_decisions,banking77,ag_news,emotion --perm-max-cases 60 \
  --datasets-dir .raw/datasets --out .benchmarks/139_basis_split_perturbation_certificate
```

Deterministic (fixed probe seed, fixed stride sampling, single-threaded f64 linear
algebra); outputs are bit-identical at any load. Not a latency claim — no preflight bar.

## The T3 numbers (the Thm-3 certificate table)

| suite | verdict | cases | dirs (of k) | route-active | median ‖Δ‖ | gate flips | gate max Δp | first-flip rung |
|---|---|---|---|---|---|---|---|---|
| typed_decisions | vacuous_no_response | 31 | 32 (of 202) | 0/31 | 0.0819 | 0 | 0.00 | — |
| ag_news | pass | 31 | 32 (of 111) | 31/31 | 0.1200 | 0 | 5.96e-8 | — |
| emotion | pass | 31 | 32 (of 124) | 31/31 | 0.2312 | 0 | 2.98e-8 | — |
| banking77 | pass | 32 | 32 (of 255) | 32/32 | 0.1435 | 0 | 3.73e-9 | — |

- **The gate rung is `fp_1x`** — one norm-ULP (‖y‖₂ = 1 on the L2-normalized hashed
  embed; `f32::EPSILON` IS the norm-ULP anchor): the amplitude any summation reorder
  could produce. Discrete outputs (pick + abstain) bit-identical REQUIRED, scored
  outputs within 1e-3.
- **ag_news / emotion / banking77 PASS with maximal margin**: zero discrete flips at
  EVERY rung (7 rungs: fp ×{1,16,256} ULP, then {1e-3,1e-2,1e-1,1}× median real
  ordering-delta norm — the top rung perturbs by a FULL ordering-flip amplitude),
  max probability drift 3.7e-9…6.0e-8 (five+ orders under the 1e-3 bar). The scorer's
  discrete surface is FLAT along the presentation directions.
- **typed_decisions reads `vacuous_no_response`** — 0/31 route-active cases (its
  options do not resolve by-name to domains; the deployed posture is drafter-only +
  option-conditioned), and even the full-amplitude rung moved NOTHING (Δp exactly 0).
  The loud vacuity verdict is the absent-work defense doing its job: an inert posture
  is DISCLOSED, never passed. The embedding drives only domain routing there, and the
  argmax was flat at these amplitudes.
- Comparisons per suite: 1984–2048 = cases × 32 directions (top-eigenvalue cap, of the
  certified k) × 2 signs × 7 rungs + base solves.

## The T1/T2 re-record (same run, corrected metric — supersedes bench 138's numbers)

Bench 138 (m3, 10-11) was recorded under katgpt-core's **defective** ZCA whitener
(katgpt-rs Issue 931, fixed `be9b2cbd2` the next day). This run is the first
presentation-certificate read under the CORRECTED whitener, with one semantic fix
found and landed with T3:

- **The leakage metric moved to the population metric.** `held_out_leakage` was
  projecting RAW pair deltas onto WHITENED-space eigenvectors — a metric mismatch
  (a low-pop-variance delta whitens to a direction near-orthogonal to itself under
  the identity metric; measured on the planted fixture: raw-metric absorption 0.006
  against a clean rank-1 whitened moment at gap 1700×). The reflex core now whitens
  holdout + cross-family deltas through the same `W` before projection, and the
  cross-family residuals likewise. The T3 perturbation directions are the raw-space
  images `v_raw = normalize(Σ^{1/2} u)` (`W⁻¹u`), so the perturbation moves the raw
  embedding along the block in the SAME metric the certificate measured it in.
- **The k structure survived the fix exactly** (202/111/124/255 — same as bench 138 on
  all four suites) and the leakage moved little (typed 0.192→0.231, ag_news 0.542→0.579,
  emotion 0.536→0.500, banking77 0.004→0.005): the 931 close-out's "qualitative
  verdicts likely stand" prediction holds for the certificate.
- The planted unit-test fixtures were rewritten to the LIVE structure (permuted option
  order — same token multiset, adjacency-only deltas): the old token-ADDITION fixture's
  "generalization to unseen tags" property was an artifact of the defective whitener
  (a hashed embed maps unseen tokens to unseen buckets — orthogonal by construction),
  and its delta is dominated by the identity-direction rescale the L1 normalizer
  induces. The transfer bar now lives in the bench record, not the fixture.

## Verdicts

- **T3 PASS** on the three by-name suites (max-margin: no flip at any rung).
- **typed_decisions**: honest inert-posture disclosure (`vacuous_no_response`) — the
  T4 opt-in projection is a no-op consideration there by construction (no embedding
  term rides the scored path).
- **T4 disposition input**: on the three passing suites the certificate shows zero
  sensitivity up to FULL ordering amplitude — there is no ordering leakage riding the
  decide path to project out. T4 stays unarmed pending a lane that shows real swings
  (the issue's own gate).
