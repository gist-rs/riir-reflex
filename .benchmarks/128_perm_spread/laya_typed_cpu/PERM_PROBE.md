# Perm probe — option-permutation spread (issue 077)

- date 2026-10-08T05:45:41Z · sha 1b8873e · host 4090-windows
- datasets E:\git\riir-reflex\.raw/datasets
- K 5 orderings × ≤ 20 case(s)/suite · seed 125239297
- laya device: Cpu
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control holds a stricter contract: flips allowed ONLY on an exact top-2 tie (the deterministic first-position tie-break) and median swing within the L1 normalizer's measured fp envelope (0.01 pt) — strict byte-identity stays a disclosed column (the L1 sum reorders with the presentation and moves ULPs by design). A control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | modelless | modelless (deployed plain posture) | 30 | 6.55 | 13.62 | 21 | 0.700 | false | "RED" |
| typed_decisions | laya/typed | laya/typed | 30 | 4.12 | 9.23 | 4 | 0.133 | — | "RED" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| typed_decisions | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| typed_decisions | laya/typed | alpha | alpha|alpha|alpha|alpha|alpha | 5.0 | false |

**RED cells: 2** — disclosed above; a control red refuses the run verdict
