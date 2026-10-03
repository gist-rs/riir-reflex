# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `3905b17` on `4090-windows` (2026-10-03T04:19:50Z) · profile release · laya feature true
- box state (Issue 021): start power=? mode=? load=? swap=?M · end power=? mode=? load=? swap=?M — ⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state
- laya device posture: cuda (LAYA_DEVICE or the build's non-macOS CUDA default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: off (pass --bekko to add the comparison lane; needs the bekko venv — Bench 103)
- agentjev lane: off (pass --agentjev to add the comparison lane; needs their jev_service on AGENTJEV_SERVE_URL — .issues/025)
- clef lane: off (pass --clef to add the comparison lane; needs the owner's Workers AI creds behind the loopback forwarder — plan 011 Phase A)
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

## typed_decisions — 400 cases / 2000 questions

**slice-integrity: test 400 (fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100 (fnv1a64-a1679de56e9278b1) · OK**

**corpus cap:** 48 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3523 · max_prob 0.0598 · inv_entropy 0.3523

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3345 | 0.2456 | 0.0488 | 0.6895 | 1.2301 | 0.5429 | 0.4690 | 0.0234 | 1.00/0.99 | 0.2857 | 0.929 ms | 2.703 ms (5) | ✓ | s 0.3619 / d 0.9122 (n 100) |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2113 | 0.7684 | 1.3593 | 0.5627 | 0.4290 | 0.1918 | — / — | — | 84.0 ms | 130.0 ms (5) | ✓ | — |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.3228 | 0.9057 | 1.9437 | 0.5548 | 0.4040 | 0.2567 | — / — | — | 45.0 ms | 74.0 ms (5) | ✓ | — |
| laya-riir · typed | 2000 | 0.7825 | 0.7744 | 0.1477 | 0.3426 | 0.5991 | 0.0878 | 0.9180 | 0.3319 | — / — | — | 83.0 ms | 155.0 ms (5) | ✓ | — |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 2000 | 0.3345 | 0.2456 | 0.2925 | 0.0594 | ✓ |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2925 | 0.0919 | ✓ |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.2925 | 0.0799 | ✓ |
| laya-riir · typed | 2000 | 0.7825 | 0.7744 | 0.2925 | 0.6926 | ✓ |


**G1 (modelless readout ECE):** raw 0.3293 · calibrated 0.0234 · conformal-naive floor 0.3578 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3169 · brier_soft 0.2430 · score MAE 0.7046 · within_1 0.7275

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1717 | 0.0927 | 0.2644 |
| modelless | noul | 600 | 0.5650 | 0.0568 | 0.5082 |
| modelless | score | 800 | 0.2838 | 0.0442 | 0.2935 |
| laya-riir·english | choice | 600 | 0.2883 | 0.1681 | 0.4527 |
| laya-riir·english | noul | 600 | 0.4733 | 0.2808 | 0.7542 |
| laya-riir·english | score | 800 | 0.3225 | 0.2065 | 0.5168 |
| laya-riir·multilingual | choice | 600 | 0.2950 | 0.3623 | 0.6559 |
| laya-riir·multilingual | noul | 600 | 0.4867 | 0.4084 | 0.8919 |
| laya-riir·multilingual | score | 800 | 0.2863 | 0.2313 | 0.5110 |
| laya-riir·typed | choice | 600 | 0.7500 | 0.1579 | 0.5973 |
| laya-riir·typed | noul | 600 | 0.8267 | 0.1148 | 0.7118 |
| laya-riir·typed | score | 800 | 0.7738 | 0.1662 | 0.6075 |
**typed-decisions extras (laya-riir·english):** soft_acc 0.3346 · brier_soft 0.3348 · score MAE 0.6937 · within_1 0.7525
**typed-decisions extras (laya-riir·multilingual):** soft_acc 0.3271 · brier_soft 0.4737 · score MAE 0.7604 · within_1 0.7000
**typed-decisions extras (laya-riir·typed):** soft_acc 0.5134 · brier_soft 0.0565 · score MAE 0.2135 · within_1 0.9925

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

