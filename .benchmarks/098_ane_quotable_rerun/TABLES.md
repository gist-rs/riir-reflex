# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `2f6b58c` on `m3-ane` (2026-09-30T10:35:35Z) · profile release · laya feature true
- box state (Issue 021): start power=AC Power mode=high load=5.04 swap=388M · end power=AC Power mode=high load=4.64 swap=388M — latency QUOTABLE
- laya device posture: ane (LAYA_DEVICE=ane → load_ane; whole-graph CoreML encoder, Plan 002)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): OFF (nb_scale 0 — the published baseline posture)
- option-conditioned tables (issue 038 T7b): 
- NBSVM ridge readout (issue 038 T7a): 
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- sampling: test sample = label-STRATIFIED round-robin over the whole test split (budget = the registry test cap; first-appearance label order, dataset order within each label, deterministic, no RNG); cal slice = the same law over the train rows; corpus pool = the train rows MINUS the cal front (excluded by construction, not position); budget-0 suites unchanged (identity split) — Issue 039 T2
- readout: shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side arming lever overfit the narrow suites' cal slice — emotion's test G1 regressed — and coincided with Dispatch on the wide suites it targeted; the wide-label G1 gap was closed by T2's stratified cal slice). The candidate table is recorded per suite (report-only); EngineConfig::readout stays the opt-in knob
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; all six modelless at default features since Issue 045 (cache_reuse's LLM-only carve-out REVERSED — Bench 072: modelless 0.9167 vs the frozen LLM-lane 0.5000)

## Absences / errors (honest — never silently dropped)

- ag_news (laya/english): 35 case(s) skipped over the ANE bucket limit — ["ag_news:1", "ag_news:5", "ag_news:6", "ag_news:7", "ag_news:10", "ag_news:23", "ag_news:27", "ag_news:28", "ag_news:29", "ag_news:35", "ag_news:62", "ag_news:71", "ag_news:88", "ag_news:89", "ag_news:97", "ag_news:117", "ag_news:124", "ag_news:166", "ag_news:170", "ag_news:184", "ag_news:194", "ag_news:196", "ag_news:198", "ag_news:200", "ag_news:215", "ag_news:243", "ag_news:256", "ag_news:258", "ag_news:290", "ag_news:301", "ag_news:305", "ag_news:333", "ag_news:369", "ag_news:372", "ag_news:380"] (a coverage limit, not a failure; the served row is the smaller set)
- prompt_injections (laya/english): 9 case(s) skipped over the ANE bucket limit — ["prompt_injections:0", "prompt_injections:8", "prompt_injections:37", "prompt_injections:45", "prompt_injections:57", "prompt_injections:61", "prompt_injections:84", "prompt_injections:107", "prompt_injections:109"] (a coverage limit, not a failure; the served row is the smaller set)
- xnli_en (laya/english): 2 case(s) skipped over the ANE bucket limit — ["xnli_en:4", "xnli_en:5"] (a coverage limit, not a failure; the served row is the smaller set)

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4845 · max_prob 0.2262 · inv_entropy 0.4845

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.4050 | 0.3750 | 0.1465 | 0.7436 | 1.3735 | 0.5223 | 0.4750 | 0.0813 | 1.00/0.39 | 0.3951 | 0.147 ms | 0.265 ms (5) | ✓ | s 0.4397 / d 0.4105 (n 200) |
| laya-riir · english | 365 | 0.9452 | 0.9374 | 0.0387 | 0.0864 | 0.1734 | 0.0075 | 1.0000 | 0.1421 | — / — | — | 28.0 ms | 34.0 ms (4) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4046 · calibrated 0.0813 · conformal-naive floor 0.2282 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1449 · max_prob 0.0245 · inv_entropy 0.1449

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2750 | 0.1345 | 0.1053 | 0.8323 | 1.7886 | 0.7259 | 0.3000 | 0.1297 | 1.00/0.33 | 0.2910 | 0.109 ms | 0.125 ms (5) | ✓ | s 0.1453 / d 0.4265 (n 200) |
| laya-riir · english | 400 | 0.5975 | 0.4777 | 0.3040 | 0.6964 | 2.1820 | 0.2659 | 0.7150 | 0.2547 | — / — | — | 19.0 ms | 24.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2749 · calibrated 0.1297 · conformal-naive floor 0.2982 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2442 · max_prob 0.0340 · inv_entropy 0.2442

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2017 | 0.1890 | 0.0087 | 0.7986 | 1.6059 | 0.8159 | 0.1833 | 0.0658 | 1.00/0.58 | 0.2072 | 0.090 ms | 0.107 ms (7) | ✓ | s 0.2049 / d 0.4166 (n 200) |
| laya-riir · english | 600 | 0.3733 | 0.3303 | 0.2461 | 0.8179 | 1.7314 | 0.5290 | 0.4533 | 0.0807 | — / — | — | 23.0 ms | 27.0 ms (7) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2009 · calibrated 0.0658 · conformal-naive floor 0.3343 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1781 · within_1 0.4433

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4999 · max_prob 0.0055 · inv_entropy 0.4999

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5005 | 0.6937 | 0.3980 | 0.5690 | 0.0172 | 1.00/1.00 | 0.0000 | 0.067 ms | 0.076 ms (2) | ✓ | s 0.5100 / d 0.4619 (n 100) |
| laya-riir · english | 107 | 0.6916 | 0.6522 | 0.2668 | 0.5192 | 2.8819 | 0.1046 | 0.9811 | 0.2668 | — / — | — | 23.0 ms | 30.0 ms (2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4827 · calibrated 0.0172 · conformal-naive floor 0.2491 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3196 · max_prob 0.0238 · inv_entropy 0.3196

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3400 | 0.2052 | 0.0055 | 0.6676 | 1.1000 | 0.6937 | 0.3467 | 0.0335 | 1.00/0.56 | 0.3383 | 0.084 ms | 0.103 ms (4) | ✓ | s 0.2999 / d 0.5110 (n 200) |
| laya-riir · english | 298 | 0.8591 | 0.8603 | 0.0680 | 0.2222 | 0.4017 | 0.0385 | 0.9799 | 0.1039 | — / — | — | 28.0 ms | 32.0 ms (3) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3395 · calibrated 0.0335 · conformal-naive floor 0.3137 → **PASS** (beats both the uncalibrated output AND the floor)

## Landscape — published specialist rows (NOT measured by this harness)

External "System One" typed-decision models on the same 400-case /
2000-question Typed Decisions official test split, quoted AS PUBLISHED
(issue 025; pin `malevrigns/agent-jev` @ `a965ca8f`, Apache-2.0). Their
protocol differs from ours — the footnotes are part of the row; no number
here is comparable without them.

| lane · model | source | acc | bool·noul / choice / score | p50 case |
|---|---|---|---|---|
| AgentJev-0.6B (598M, Qwen3-0.6B backbone) | published (their run) | **0.7925** | 88.83 / 75.33 / 75.00 | ~60–70 ms, their cuda box |
| reflex · agentjev (their service, gold-label) | **MEASURED** (bench 039, 4090) | **0.7715** | — | 88 ms (loopback HTTP) |
| Laya (published checkpoint, 421M ModernBERT) | their table — card-copied, not re-scored | 0.7700 | — | 41.53 ms (their box) |
| TypeSafe Jev 1.13.0 | their table — zero-shot generalist | 0.727 | — | — |
| reflex · laya-riir·typed | MEASURED — the typed_decisions table above | 0.7445 (baseline `aa37823`) | 78.50 / 73.33 / 72.25 | 1312 ms (m3 metal, that baseline) |
| reflex · modelless | MEASURED — the typed_decisions table above | 0.3190 (baseline `aa37823`) | 53.17 / 18.67 / 25.87 | 0.472 ms |

Footnotes: (1) their accuracy is agreement with the public TEACHER argmax;
ours is gold-label under the standard harness protocol — different
references of truth. (1b) the MEASURED agentjev row (bench 039, Issue 025
amendment 4) closes that gap for AgentJev: **0.7715 gold-label** on this
harness's split (teacher-argmax 0.7925 → gold −2.1pt) — the published
ranking SURVIVES the protocol change (+2.7pt over our measured laya-typed
0.7445; their teacher-protocol gap was +2.25). (2) their run held out 120 dev + 120 cal cases and
selected the step-600 checkpoint on dev soft-CE before opening test; ours
fits no per-benchmark head. (3) their wide-load figure (shared-prefix
298.91 ms vs unshared 609.65 ms at 66 paths / 33,547 tokens, backbone
token-ops 33,547 → 2,551 = 92.4% reduction, max prob delta 5.08e-4) is
their box and their load — not re-measured here. (4) on SHORT inputs their
own table reads Laya faster (41.53 ms vs ~60–70 ms p50/case); AgentJev's
latency win is wide candidate loads only. (5) the bool·noul column maps
their boolean primitive to our noul primitive — the closest analogue, not
a wire match; neither of their lanes carries an abstention primitive (the
wire's first-class abstention is ours alone).

## Landscape — fast-decisions (vendor suite; NOT measured by this harness)

fastino's `fast-decisions` suite — 17 English operational-decision
domains (commerce support intent/topic, ticket routing, product feedback,
banking intent, document type, review sentiment, assistant handoff,
email triage, clinic request, travel request, news topic, paper field,
sports recap, restaurant review, benefits request, screen tags), 300
held-out test examples per domain, exact-match accuracy, the same text
and candidate labels for every model. Quoted AS PUBLISHED (issue 029).

| model | avg exact-match |
|---|---|
| GLiNER2.5-Decide (340M, DeBERTa-v3-large) | **60.2%** |
| GLiNER2.5-Decide-1B (their dataset card: "GLiNER2 XL (1B)") | 59.6% |
| JevK5 | 57.6% |
| GLiNER2.5-multi-Decide (287M) | 56.7% |
| SemIf (Qwen3.5-4B) | 56.4% |
| GLiFormer large-v1 | 49.0% |
| Laya Router | 46.6% |

Footnotes: (1) the scored split is private — their public repo carries
only the 100/domain development split with an explicit "do not report a
score computed on the files in this repo" note, so no row here can be
re-scored outside fastino. (2) their metric is per-head exact match
(single-label string equality; multi-label heads compared as sets),
averaged over the 17 domains. (3) their "Laya Router" row names no
checkpoint variant — neither the base router nor the typed specialist;
our measured laya-riir cells are the port checkpoints under OUR protocol,
so the 60.2-vs-46.6 gap is their-suite/their-checkpoint/their-protocol.
(4) our measured comparison lives in the 15-suite tables (bench 037,
`.benchmarks/037_gliner_lane_4090`): the DIRECTION is confirmed on
decision-style suites — gliner beats the laya base checkpoint 9/15
(banking77 +20.8pt, massive_intent +7.3pt, all five harness families) —
and honestly refuted on classic NLU (ag_news −24.8pt, xnli_en −38.3pt;
their card's own "not a general-purpose model" framing). The
typed_decisions headline stays laya's: the `typed` specialist 0.7445 vs
gliner 0.5280.

