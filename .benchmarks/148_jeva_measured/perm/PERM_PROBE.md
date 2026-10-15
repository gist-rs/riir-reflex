# Perm probe — option-permutation spread (issue 077)

- date 2026-10-15T21:24:48Z · sha 83031d2 · host m3
- datasets /Users/katopz/git/riir-reflex/.raw/datasets
- K 5 orderings × ≤ 40 case(s)/suite · seed 125239297
- laya device: not probed
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control holds a stricter contract: flips allowed ONLY on an exact top-2 tie (the deterministic first-position tie-break) and median swing within the L1 normalizer's measured fp envelope (0.01 pt) — strict byte-identity stays a disclosed column (the L1 sum reorders with the presentation and moves ULPs by design). A control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- modelless route: by-name + index-anchored (score, k == N, index-named domains; issue 079)
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | modelless | modelless (deployed plain posture) | 60 | 0.00 | 0.00 | 31 | 0.517 | false | "PASS" |
| typed_decisions | jeva | LIQIIIII/Jeva-4B (LIQIIIII/Jeva-4B dtype=torch.bfloat16) | 60 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| ag_news | modelless | modelless (deployed plain posture) | 40 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| ag_news | jeva | LIQIIIII/Jeva-4B (LIQIIIII/Jeva-4B dtype=torch.bfloat16) | 40 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| emotion | modelless | modelless (deployed plain posture) | 40 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| emotion | jeva | LIQIIIII/Jeva-4B (LIQIIIII/Jeva-4B dtype=torch.bfloat16) | 40 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| xnli_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| xnli_en | jeva | LIQIIIII/Jeva-4B (LIQIIIII/Jeva-4B dtype=torch.bfloat16) | 38 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| massive_intent_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| massive_intent_en | jeva | LIQIIIII/Jeva-4B (LIQIIIII/Jeva-4B dtype=torch.bfloat16) | 38 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| banking77 | modelless | modelless (deployed plain posture) | 39 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| banking77 | jeva | LIQIIIII/Jeva-4B (LIQIIIII/Jeva-4B dtype=torch.bfloat16) | 39 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| typed_decisions | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| typed_decisions | jeva | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| ag_news | modelless | alpha | alpha|delta|gamma|alpha|alpha | 0.0 | true |
| ag_news | jeva | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| emotion | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| emotion | jeva | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| xnli_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| xnli_en | jeva | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| massive_intent_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| massive_intent_en | jeva | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| banking77 | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| banking77 | jeva | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |

**RED cells: 0** — every probed lane holds the median gate (or is the control holding byte-identity)
