# Bench 138 — basis_split presentation-subspace certificate, per-suite numbers (reflex Issue 086 T2)

**Date:** 2026-10-11T23:17Z · **Host:** m3 · **Reflex sha:** `e3b8d72` (T1 landed) + the T2 rename arm in this commit
**Suites:** typed_decisions · banking77 · ag_news · emotion (the issue's three minimum + emotion for continuity)
**Command:**

```sh
scripts/bench_preflight.sh   # PROVENANCE: power=AC Power load=7.68 swap=0.00M canary=136.7us/best5 powermode=2(high) — REFUSED (load > 6)
CARGO_TARGET_DIR=/tmp/reflex-basis cargo run --release -p riir-reflex --features basis_cert \
  --bin harness -- --perm-probe --basis-split --skip-laya \
  --suites typed_decisions,banking77,ag_news,emotion --perm-max-cases 60 \
  --out .benchmarks/138_basis_split_presentation_certificate
```

**Determinism disclosure:** the preflight REFUSED at load 7.68 — that bar exists for LATENCY
claims, and this record makes none. The certificate is fully deterministic (fixed probe seed,
fixed stride sampling, single-threaded f64 linear algebra): its outputs are bit-identical at any
load, and the JSON beside this file is the artifact. Re-run the command to reproduce byte-for-byte.
High Power Mode (powermode=2) was on for the run — clock state, another latency-only concern.

## The numbers (the certificate table, verbatim from the run)

| suite | verdict | k | gap | holdout leakage | agreement residual | casing residual | rename residual | fit+hold pairs | whitening pop |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | sufficient_variation | 202 | 4.86 | 0.1920 | 0.4327 | none (all null) | 0.1857 | 600+600 | 667 |
| ag_news | sufficient_variation | 111 | 1879.38 | 0.5417 | 0.7186 | none (all null) | 0.6117 | 400+400 | 708 |
| emotion | sufficient_variation | 124 | 380.26 | 0.5358 | 0.7307 | none (all null) | 0.4475 | 400+400 | 710 |
| banking77 | sufficient_variation | 255 | 18.27 | 0.0040 | 0.0536 | none (all null) | 0.0010 | 500+500 | 740 |

The perm-probe half of the same run: **all four modelless controls PASS at 0.00 pt median swing,
zero flips** (`PERM_PROBE.md` beside this file) — while the certificate finds 111–255-dimensional
presentation subspaces in the embed. That contrast IS the finding: the ordering energy exists in
the presentation rendering (boundary bigrams between consecutive option lines) and does NOT ride
into the modelless decide path (order-equivariant by construction).

## Reads (honest, per suite)

- **banking77** (77 options per question): the cleanest certificate. Holdout leakage **0.004** —
  the fitted presentation null space generalizes almost perfectly to unseen slots; rename
  residual **0.001** — near-total family separation (the ordering basis captures essentially no
  rename churn). k=255 ≈ d: the presentation variation spans the full embed width on this suite
  (every option-boundary bigram pair appears somewhere across 77 options).
- **ag_news / emotion**: leakage ≈ 0.54 — about half the ordering energy generalizes across
  slots (the rest is slot-specific bigram detail). Rename residual 0.45–0.61 — PARTIAL family
  overlap, honestly disclosed: a rename also shifts the same boundary bigrams the ordering does,
  so the two families share mechanism-level structure. The cross-family arm is doing its job —
  it is measuring exactly that overlap.
- **typed_decisions**: k=202 of 256 with the weakest gap (4.86× vs τ 4.0) — the presentation
  moment dominates this suite (rich per-option descriptions → many boundary bigrams) and the
  spectrum has no sharp structural cutoff. Rename residual **0.186** is LOW — distinct families
  here. The leakage 0.192 says the fitted subspace generalizes well despite the weak gap.
- **The casing arm is all-null on every suite** — the tokenizer's case-insensitivity law held at
  scale (each suite contributes ~200–300 casing pairs, every Δ ≡ 0). The arm reports `None`
  honestly rather than a fabricated 0.0.

## Gate posture

- No promotion decision rides this record: `basis_split` stays opt-in in katgpt-core, the
  certificate stays report-only behind reflex `basis_cert`. The T3 lane (projected invariant
  KNN) is NOT armed by these numbers — nothing here shows formatting/ordering churn hurting a
  real retrieval path; it shows where the energy lives.
- The score-side agreement column (correlating certificate projection with the probe's score
  swings) stays open — the probe's control had ZERO untied flips to correlate against, so the
  column would be vacuous on these suites; it becomes measurable the day a lane shows real
  swings (the laya cell, T2's follow-up if the encoder lane ever enters the probe).

## Reproduction

The run is deterministic; the full per-case rows are in `perm_probe.json` (176 KB). Box state
quoted above; datasets at `.raw/datasets` (canonical, reflex issue-013 posture).
