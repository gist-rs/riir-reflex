# Perm probe — option-permutation spread (issue 077)

- date 2026-10-08T05:42:39Z · sha 1b8873e · host 4090-windows
- datasets E:\git\riir-reflex\.raw/datasets
- K 5 orderings × ≤ 20 case(s)/suite · seed 125239297
- laya device: Cpu
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control holds a stricter contract: flips allowed ONLY on an exact top-2 tie (the deterministic first-position tie-break) and median swing within the L1 normalizer's measured fp envelope (0.01 pt) — strict byte-identity stays a disclosed column (the L1 sum reorders with the presentation and moves ULPs by design). A control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | modelless | modelless (deployed plain posture) | 20 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| ag_news | laya/english | laya/english | 20 | 0.68 | 16.84 | 0 | 0.000 | — | "PASS" |
| emotion | modelless | modelless (deployed plain posture) | 20 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| emotion | laya/english | laya/english | 20 | 1.53 | 44.50 | 1 | 0.050 | — | "PASS" |
| xnli_en | modelless | modelless (deployed plain posture) | 20 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| xnli_en | laya/english | laya/english | 20 | 0.39 | 39.26 | 0 | 0.000 | — | "PASS" |
| massive_intent_en | modelless | modelless (deployed plain posture) | 20 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| massive_intent_en | laya/english | laya/english | 20 | 0.01 | 99.97 | 6 | 0.300 | — | "PASS" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| ag_news | modelless | alpha | alpha|delta|gamma|alpha|alpha | 0.0 | true |
| ag_news | laya/english | beta | beta|alpha|alpha|alpha|alpha | 4.2 | true |
| emotion | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| emotion | laya/english | beta | beta|alpha|alpha|alpha|alpha | 4.2 | true |
| xnli_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| xnli_en | laya/english | beta | beta|alpha|alpha|alpha|alpha | 4.2 | true |
| massive_intent_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| massive_intent_en | laya/english | beta | beta|alpha|alpha|alpha|alpha | 4.2 | true |

**RED cells: 0** — every probed lane holds the median gate (or is the control holding byte-identity)
