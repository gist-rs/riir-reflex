# Perm probe — option-permutation spread (issue 077)

- date 2026-10-12T10:42:49Z · sha 1d396d1 · host shikuwa
- datasets E:/git/riir-reflex/.raw/datasets
- K 5 orderings × ≤ 60 case(s)/suite · seed 125239297
- laya device: not probed
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control holds a stricter contract: flips allowed ONLY on an exact top-2 tie (the deterministic first-position tie-break) and median swing within the L1 normalizer's measured fp envelope (0.01 pt) — strict byte-identity stays a disclosed column (the L1 sum reorders with the presentation and moves ULPs by design). A control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- modelless route: by-name + index-anchored (score, k == N, index-named domains; issue 079)
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | modelless | modelless (deployed plain posture) | 87 | 0.00 | 0.00 | 44 | 0.506 | false | "PASS" |
| ag_news | modelless | modelless (deployed plain posture) | 58 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| emotion | modelless | modelless (deployed plain posture) | 58 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| banking77 | modelless | modelless (deployed plain posture) | 56 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| typed_decisions | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| ag_news | modelless | alpha | alpha|delta|gamma|alpha|alpha | 0.0 | true |
| emotion | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| banking77 | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |

## Basis-split certificate (issue 086 T1 — presentation subspace, hashed-feature embed)

Fit/holdout = even/odd slots; whitening = identity renderings + train docs (stride-capped); τ_gap 4.0. `unidentified_*` is a RESULT (order-insensitive embed), not a failure; `pair_leakage` is a generalization residual of the fitted presentation null space, never purity.

| suite | verdict | k | gap | holdout leakage | agreement residual | casing residual | rename residual | fit+hold pairs | whitening pop |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | sufficient_variation | 202 | 68.19 | 0.2312 | 0.4720 | none (all null) | 0.1803 | 600+600 | 667 |
| ag_news | sufficient_variation | 111 | 3880.56 | 0.5789 | 0.7433 | none (all null) | 0.5317 | 400+400 | 708 |
| emotion | sufficient_variation | 124 | 1268.62 | 0.5000 | 0.7057 | none (all null) | 0.4668 | 400+400 | 710 |
| banking77 | sufficient_variation | 255 | 145.76 | 0.0053 | 0.0608 | none (all null) | 0.0009 | 500+500 | 740 |

## Basis-perturb certificate (issue 086 T3 — Thm-3, the scorer under ε·v_k along the presentation basis)

GATE rung = `fp_1x` (one norm-ULP — ‖y‖ = 1 on the L2-normalized embed: the amplitude any summation reorder could produce). Discrete outputs (pick + abstain) must be bit-identical there and scored outputs within 1e-3; the higher rungs measure margin, they do not gate. `vacuous_no_response` = even a full median-delta perturbation moved nothing (an inert posture, honestly disclosed — never a pass).

| suite | verdict | cases | dirs (of k) | route-active | median ‖Δ‖ | gate flips | gate max Δp | first-flip rung |
|---|---|---|---|---|---|---|---|---|
| typed_decisions | vacuous_no_response | 31 | 32 (of 202) | 0/31 | 0.0819 | 0 | 0.00e0 | — |
| ag_news | pass | 31 | 32 (of 111) | 31/31 | 0.1200 | 0 | 5.96e-8 | — |
| emotion | pass | 31 | 32 (of 124) | 31/31 | 0.2312 | 0 | 2.98e-8 | — |
| banking77 | pass | 32 | 32 (of 255) | 32/32 | 0.1435 | 0 | 3.73e-9 | — |

**RED cells: 0** — every probed lane holds the median gate (or is the control holding byte-identity)
