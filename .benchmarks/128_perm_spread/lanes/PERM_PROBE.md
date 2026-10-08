# Perm probe — option-permutation spread (issue 077)

- date 2026-10-08T05:35:42Z · sha 1b8873e · host 4090-windows
- datasets E:\git\riir-reflex\.raw/datasets
- K 5 orderings × ≤ 40 case(s)/suite · seed 125239297
- laya device: not probed
- gate: per lane x suite: MEDIAN top-pick swing across orderings <= 2 pt; max + flip rate disclosed beside. The modelless control holds a stricter contract: flips allowed ONLY on an exact top-2 tie (the deterministic first-position tie-break) and median swing within the L1 normalizer's measured fp envelope (0.01 pt) — strict byte-identity stays a disclosed column (the L1 sum reorders with the presentation and moves ULPs by design). A control red refuses the run verdict.
- sampling: the suite's choice-carrying cases in dataset order, first of every ceil(n/max_cases) stride (deterministic); ALL choice questions of each sampled case. The canary case rides every lane as the liveness cell (never in the suite medians).
- invariance: label-space mapping: option LABELS are the invariant under permutation (indexes move); every comparison happens on labels mapped back to the original criteria order.
- latency: latency columns deliberately absent — this is a distribution-stability probe; the latency/determinism axes live in the det benches (126/127).

| suite | lane | model | n | median pt | max pt | flips | rate | control | verdict |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | modelless | modelless (deployed plain posture) | 40 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| ag_news | drex | drex-dlm | 40 | 0.38 | 13.25 | 1 | 0.025 | — | "PASS" |
| ag_news | agentjev | AgentJev-0.6B@68883998 (cuda:0) | 40 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| emotion | modelless | modelless (deployed plain posture) | 40 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| emotion | drex | drex-dlm | 40 | 14.92 | 44.39 | 9 | 0.225 | — | "RED" |
| emotion | agentjev | AgentJev-0.6B@68883998 (cuda:0) | 40 | 0.00 | 0.00 | 1 | 0.025 | — | "PASS" |
| xnli_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| xnli_en | drex | drex-dlm | 38 | 4.41 | 19.10 | 5 | 0.132 | — | "RED" |
| xnli_en | agentjev | AgentJev-0.6B@68883998 (cuda:0) | 38 | 0.00 | 0.00 | 0 | 0.000 | — | "PASS" |
| massive_intent_en | modelless | modelless (deployed plain posture) | 38 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| massive_intent_en | drex | drex-dlm | 38 | 5.11 | 76.29 | 6 | 0.158 | — | "RED" |
| massive_intent_en | agentjev | AgentJev-0.6B@68883998 (cuda:0) | 38 | 0.27 | 0.86 | 2 | 0.053 | — | "PASS" |
| banking77 | modelless | modelless (deployed plain posture) | 39 | 0.00 | 0.00 | 0 | 0.000 | false | "PASS" |
| banking77 | drex | drex-dlm | 39 | 8.65 | 52.23 | 7 | 0.179 | — | "RED" |
| banking77 | agentjev | AgentJev-0.6B@68883998 (cuda:0) | 39 | 0.24 | 0.77 | 2 | 0.051 | — | "PASS" |

## Canary cell (liveness — an order-biased lane flips it BY DESIGN)

| suite | lane | ref | picks | swing pt | flipped |
|---|---|---|---|---|---|
| ag_news | modelless | alpha | alpha|delta|gamma|alpha|alpha | 0.0 | true |
| ag_news | drex | delta | delta|beta|beta|beta|beta | 27.5 | true |
| ag_news | agentjev | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| emotion | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| emotion | drex | delta | delta|beta|beta|beta|beta | 27.5 | true |
| emotion | agentjev | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| xnli_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| xnli_en | drex | delta | delta|beta|beta|beta|beta | 27.5 | true |
| xnli_en | agentjev | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| massive_intent_en | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| massive_intent_en | drex | delta | delta|beta|beta|beta|beta | 27.5 | true |
| massive_intent_en | agentjev | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |
| banking77 | modelless | beta | beta|beta|beta|beta|beta | 0.0 | false |
| banking77 | drex | delta | delta|beta|beta|beta|beta | 27.5 | true |
| banking77 | agentjev | alpha | alpha|alpha|alpha|alpha|alpha | 0.0 | false |

**RED cells: 4** — disclosed above; a control red refuses the run verdict
