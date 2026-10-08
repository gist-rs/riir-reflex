# Perm probe — option-permutation spread (issue 077)

- date 2026-10-08T07:07:42Z · sha 97b2189 · host 4090-windows
- datasets E:\git\riir-reflex\.raw/datasets
- K 5 orderings × ≤ 40 case(s)/suite · seed 125239297
- laya device: not probed
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control holds a stricter contract: flips allowed ONLY on an exact top-2 tie (the deterministic first-position tie-break) and median swing within the L1 normalizer's measured fp envelope (0.01 pt) — strict byte-identity stays a disclosed column (the L1 sum reorders with the presentation and moves ULPs by design). A control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | modelless | modelless (deployed plain posture) | 60 | 4.70 | 18.81 | 41 | 0.683 | false | "RED" |
| typed_decisions | d1 | LiquidAI/d1-3B @ 051bcc4 | 60 | 7.70 | 26.08 | 8 | 0.133 | — | "RED" |
| ag_news | modelless | modelless (deployed plain posture) | 40 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| ag_news | d1 | LiquidAI/d1-3B @ 051bcc4 | 40 | 0.51 | 13.87 | 0 | 0.000 | — | "PASS" |
| emotion | modelless | modelless (deployed plain posture) | 40 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| emotion | d1 | LiquidAI/d1-3B @ 051bcc4 | 40 | 4.92 | 20.76 | 5 | 0.125 | — | "RED" |
| xnli_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| xnli_en | d1 | LiquidAI/d1-3B @ 051bcc4 | 38 | 1.76 | 7.53 | 0 | 0.000 | — | "PASS" |
| massive_intent_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| massive_intent_en | d1 | LiquidAI/d1-3B @ 051bcc4 | 38 | 0.99 | 42.16 | 1 | 0.026 | — | "PASS" |
| banking77 | modelless | modelless (deployed plain posture) | 39 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| banking77 | d1 | LiquidAI/d1-3B @ 051bcc4 | 39 | 5.58 | 38.75 | 4 | 0.103 | — | "RED" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| typed_decisions | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| typed_decisions | d1 | alpha | alpha|beta|beta|alpha|alpha | 11.6 | true |
| ag_news | modelless | alpha | alpha|delta|gamma|alpha|alpha | 0.0 | true |
| ag_news | d1 | alpha | alpha|beta|beta|alpha|alpha | 11.6 | true |
| emotion | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| emotion | d1 | alpha | alpha|beta|beta|alpha|alpha | 11.6 | true |
| xnli_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| xnli_en | d1 | alpha | alpha|beta|beta|alpha|alpha | 11.6 | true |
| massive_intent_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| massive_intent_en | d1 | alpha | alpha|beta|beta|alpha|alpha | 11.6 | true |
| banking77 | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| banking77 | d1 | alpha | alpha|beta|beta|alpha|alpha | 11.6 | true |

**RED cells: 4** — disclosed above; a control red refuses the run verdict
