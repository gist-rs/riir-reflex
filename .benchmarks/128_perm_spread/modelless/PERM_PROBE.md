# Perm probe — option-permutation spread (issue 077)

- date 2026-10-08T04:55:04Z · sha 1b8873e · host 4090-windows
- datasets E:\git\riir-reflex\.raw/datasets
- K 5 orderings × ≤ 40 case(s)/suite · seed 125239297
- laya device: not probed
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control additionally asserts BYTE-IDENTITY: every ordering's label-space probabilities are bit-identical to the identity ordering's — a control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | modelless | modelless (deployed plain posture) | 60 | 4.70 | 18.81 | 41 | 0.683 | false | "RED" |
| ag_news | modelless | modelless (deployed plain posture) | 40 | 0.73 | 4.18 | 38 | 0.950 | false | "RED" |
| emotion | modelless | modelless (deployed plain posture) | 40 | 0.10 | 1.36 | 14 | 0.350 | false | "RED" |
| xnli_en | modelless | modelless (deployed plain posture) | 38 | 0.52 | 4.45 | 14 | 0.368 | false | "RED" |
| massive_intent_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "RED" |
| banking77 | modelless | modelless (deployed plain posture) | 39 | 0.00 | 0.00 | 0 | 0.000 | false | "RED" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| typed_decisions | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| ag_news | modelless | alpha | alpha|delta|gamma|alpha|alpha | 0.0 | true |
| emotion | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| xnli_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| massive_intent_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| banking77 | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |

**RED cells: 6** — disclosed above; a control red refuses the run verdict
