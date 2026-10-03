# Issue 066: Fused-abstain reference-density threshold (LSL gate)

**Status:** OPEN — filed 2026-10-03 from katgpt-rs Research 604 (arXiv:2610.02126 "Local Support Learning"). POC/optimization task; consumer of the katgpt-rs `gmm_support` primitive (Plan 618).

## The gap

The harness's fused gate fits the score-axis threshold on the cal slice (`--gate-fit-calibrated`, Bench 095 class). That score half measures CLASSIFIER CONFIDENCE — one constant per suite — and no constant on it can tell the harness whether a specific input is actually inside the suite corpus's support. The LSL App-E theory (excess/deficit decomposition) names the missing half: a gate's opening on out-of-support inputs (its *excess*) is invisible to any objective fit on in-distribution data, and the fix is a second, DENSITY-side signal — `ℓ(x) = log Φ_suite(x) − log Φ_ref(x)` over a diagonal GMM pair (suite corpus vs generic reference slice). This is a SEPARATE fused input alongside the score half (confidence vs in-support-ness — two axes, never merged); against the one-sided `CorpusDistanceGate` (max-cosine exemplars, fixed `mid`) it is the two-sided parametric sibling.

## Task

- [ ] PRE-CHECK (added at verdict round 1): a diagonal Gaussian is a poor model for sparse hashed-embedding features — run `katgpt-core`'s `sketched_gaussianity` (feature `gaussianity_probe`) on the suite embeddings first; if normality rejects, record "wrong model class for this feature space" and try the JL-projected/whitened variant BEFORE reading any A/B number, so a bad fit never masquerades as a negative result.
- [ ] POC: fit a diagonal GMM on the seat corpus's hashed-embedding space (JL-project), a reference GMM on a generic out-of-suite slice; `sigmoid(ℓ/τ)` as the DENSITY half of the fused gate — a SEPARATE fused input alongside the existing score half (which measures classifier confidence; the density half measures in-support-ness — do not merge the axes), fused exactly as the shipped fused-gate OR/threshold semantics specify.
- [ ] A/B on the frozen harness reads (paired, ONE read): fused-with-density-threshold vs shipped fused gate — accuracy, abstain rate, ECE; Report-the-Floor (conformal-naive) on the confidence output.
- [ ] Expected win surface: suites where the cal-fitted threshold transfers badly (the Bench 061/066 abstain-pathology class); no-regression LB95 ≥ 0 everywhere else.
- [ ] Gate: adopt only if paired improvement is certified per the repo's GOAT discipline; demote/keep-opt-in otherwise.

## Why this repo

The gate consumes only existing signals (suite corpus embeddings + a reference slice); no training, no new deps; aligns with the abstention-first contract. Substrate lands in katgpt-rs Plan 618; this issue is the consumer wiring.
