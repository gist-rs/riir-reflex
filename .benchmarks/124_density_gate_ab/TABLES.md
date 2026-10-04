# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `1d6932f` on `m3` (2026-10-04T20:09:54Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=3.69 swap=53320M · end power=AC Power mode=high load=3.47 swap=53320M — latency QUOTABLE
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: off (pass --bekko to add the comparison lane; needs the bekko venv — Bench 103)
- agentjev lane: off (pass --agentjev to add the comparison lane; needs their jev_service on AGENTJEV_SERVE_URL — .issues/025)
- clef lane: off (pass --clef to add the comparison lane; the hosted posture needs the owner's Workers AI creds behind the loopback forwarder, or serve the open weights locally — plan 011 Phase A + .research/007)
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
- divergence: synthetic fixtures (semantic_defects, code_fixtures): in-process fixtures with programmatic gold; the six Issue-004 harness families were retired 2026-10-02 (owner call — at-chance on the modelless lane at the honest populations)

## ag_news — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4845 · max_prob 0.2262 · inv_entropy 0.4845

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.4050 | 0.3750 | 0.1465 | 0.7436 | 1.3735 | 0.5223 | 0.4750 | 0.0813 | 1.00/0.39 | 0.3951 | 0.141 ms | 0.239 ms (5) | ✓ | s 0.4397 / d 0.4105 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.4050 | 0.3750 | 0.3075 | 0.1408 | ✓ |


**G1 (modelless readout ECE):** raw 0.4046 · calibrated 0.0813 · conformal-naive floor 0.2282 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1449 · max_prob 0.0245 · inv_entropy 0.1449

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2750 | 0.1345 | 0.1053 | 0.8323 | 1.7886 | 0.7259 | 0.3000 | 0.1297 | 1.00/0.33 | 0.2910 | 0.108 ms | 0.113 ms (5) | ✓ | s 0.1453 / d 0.4265 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.2750 | 0.1345 | 0.2975 | 0.0000 | ✓ |


**G1 (modelless readout ECE):** raw 0.2749 · calibrated 0.1297 · conformal-naive floor 0.2982 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**slice-integrity: test 600 (fnv1a64-67fe9666fa0c3a6f) · cal 200 (fnv1a64-d210e7c6707c3d40) · pool 8333 (fnv1a64-b46434b82644c53b) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2442 · max_prob 0.0340 · inv_entropy 0.2442

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2017 | 0.1890 | 0.0087 | 0.7986 | 1.6059 | 0.8159 | 0.1833 | 0.0658 | 1.00/0.58 | 0.2072 | 0.089 ms | 0.093 ms (7) | ✓ | s 0.2049 / d 0.4166 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 600 | 0.2017 | 0.1890 | 0.2733 | 0.0000 | ✓ |


**G1 (modelless readout ECE):** raw 0.2009 · calibrated 0.0658 · conformal-naive floor 0.3343 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1781 · within_1 0.4433

## xnli_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3196 · max_prob 0.0238 · inv_entropy 0.3196

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3400 | 0.2052 | 0.0055 | 0.6676 | 1.1000 | 0.6937 | 0.3467 | 0.0335 | 1.00/0.56 | 0.3383 | 0.084 ms | 0.089 ms (4) | ✓ | s 0.2999 / d 0.5110 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.3400 | 0.2052 | 0.3333 | 0.0100 | ✓ |


**G1 (modelless readout ECE):** raw 0.3395 · calibrated 0.0335 · conformal-naive floor 0.3137 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK**

**corpus cap:** 48 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.6100 | 0.6035 | 0.5454 | 0.9258 | 2.7756 | 0.2278 | 0.7667 | 0.0614 | 1.00/0.35 | 0.6122 | 0.102 ms | 0.120 ms (4) | ✓ | s 0.2372 / d 0.5259 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.6100 | 0.6035 | 0.0867 | 0.5730 | ✓ |


**G1 (modelless readout ECE):** raw 0.5454 · calibrated 0.0614 · conformal-naive floor 0.1249 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK**

**corpus cap:** 40 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.3940 | 0.3526 | 0.3780 | 0.9822 | 4.1742 | 0.5090 | 0.4840 | 0.0278 | 1.00/0.53 | 0.5485 | 0.339 ms | 0.648 ms (6) | ✓ | s 0.3177 / d 0.5549 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 500 | 0.3940 | 0.3526 | 0.0140 | 0.3854 | ✓ |


**G1 (modelless readout ECE):** raw 0.3780 · calibrated 0.0278 · conformal-naive floor 0.1942 → **PASS** (beats both the uncalibrated output AND the floor)

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

