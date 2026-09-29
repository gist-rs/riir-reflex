# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `85d452d` on `m3` (2026-09-29T05:05:56Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=12.71 swap=404M · end power=AC Power mode=high load=14.99 swap=404M — ⛔ latency NOT QUOTABLE — load 12.71 > 6 — a sibling job is on the box; load 14.99 > 6 — a sibling job is on the box
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: on — the SAME compiled programs answered through their LOCAL llama.cpp runtime as a Python subprocess oracle (programasweights, not affiliated; PAW_LOCAL_PYTHON + PAW_LOCAL_SCRIPT), never compiling — the program id comes from the hosted lane's cache so the local-vs-hosted delta isolates the runtime posture on identical artifacts; greedy by construction, determinism re-verified per run (observed-repeat, first 10); latency = in-process round-trip (no network); same mapping law + refusal accounting as --paw; .issues/033
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

## typed_decisions — 400 cases / 2000 questions

**corpus cap:** 48 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3523 · max_prob 0.0598 · inv_entropy 0.3523

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3345 | 0.2456 | 0.0488 | 0.6895 | 1.2301 | 0.5429 | 0.4690 | 0.0675 | 0.80/0.93 | 0.3469 | 0.926 ms | 2.265 ms (5) | ✓ | s 0.0071 / d 0.9122 (n 100) |
| paw-local · paw-ft-bs48-20260530 | 2000 | 0.5955 | — | — | — | — | — | — | — | — / — | — | 79.5 ms | 176.7 ms (21) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **2/2000** (0.1%) · answered-acc 0.5961 · quote-stripped 0 · program `73c963ebfd40a8d25288,f1d08b30f8c84c2fec6c,04a3b05698d9f7afc8cd,3f8d28006aa102c9df04,0189d0989f5222454bea,5c74a4fca552e2471669,0c366580689869c1f689,0522b1a4023281b00c19,c4bf7c8af208cf4374af,19114692867411036ed5,3466ab136a4aa4ae6bdf,84d32e31a96917d47a0b,eeecbb0dbfd3bba96bdb,d8564cd360417f43590c,492c5669e60b7b07a7db` (paw-ft-bs48-20260530, local-subprocess) · compile 2645.5 s (cached) · server p50 — · score MAE (answered) 0.6387 · within-1 0.7963
refusal samples: ["data", "observe"]

**G1 (modelless readout ECE):** raw 0.3293 · calibrated 0.0675 · conformal-naive floor 0.3578 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3169 · brier_soft 0.2430 · score MAE 0.7046 · within_1 0.7275

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1717 | 0.0927 | 0.2644 |
| modelless | noul | 600 | 0.5650 | 0.0568 | 0.5082 |
| modelless | score | 800 | 0.2838 | 0.0442 | 0.2935 |

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4999 · max_prob 0.0055 · inv_entropy 0.4999

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5005 | 0.6937 | 0.3980 | 0.5690 | 0.0172 | 0.39/0.39 | 0.4507 | 0.095 ms | 0.133 ms (2) | ✓ | s 0.0001 / d 0.4619 (n 100) |
| paw-local · paw-ft-bs48-20260530 | 116 | 0.6379 | — | — | — | — | — | — | — | — / — | — | 32.9 ms | 52.8 ms (2) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **0/116** (0.0%) · answered-acc 0.6379 · quote-stripped 0 · program `d9cfba100af1e46f97cd` (paw-ft-bs48-20260530, local-subprocess) · compile 77.6 s (cached) · server p50 —

**G1 (modelless readout ECE):** raw 0.4827 · calibrated 0.0172 · conformal-naive floor 0.2491 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3196 · max_prob 0.0238 · inv_entropy 0.3196

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3400 | 0.2052 | 0.0055 | 0.6676 | 1.1000 | 0.6937 | 0.3467 | 0.0199 | 0.56/0.32 | 0.3448 | 0.126 ms | 0.165 ms (4) | ✓ | s 0.0001 / d 0.5110 (n 200) |
| paw-local · paw-ft-bs48-20260530 | 300 | 0.7133 | — | — | — | — | — | — | — | — / — | — | 50.4 ms | 71.0 ms (4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **0/300** (0.0%) · answered-acc 0.7133 · quote-stripped 0 · program `1949b801cf9746028d56` (paw-ft-bs48-20260530, local-subprocess) · compile 121.0 s (cached) · server p50 —

**G1 (modelless readout ECE):** raw 0.3395 · calibrated 0.0199 · conformal-naive floor 0.3137 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

⛔ **corpus fallback (Issue 039 guard):** 29 option label(s) with NO train docs in the corpus pool — self-doc fallback only: email_querycontact, play_radio, social_post, email_addcontact, transport_taxi, lists_remove, qa_definition, cooking_recipe, email_query, calendar_set, recommendation_locations, social_query, lists_createoradd, recommendation_movies, qa_currency, play_audiobook, play_podcasts, qa_maths, calendar_remove, transport_query, qa_stock, play_game, transport_traffic, email_sendemail, recommendation_events, calendar_query, qa_factoid, transport_ticket, lists_query

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3533 | 0.3495 | 0.2873 | 0.9367 | 2.8786 | 0.4430 | 0.5133 | 0.6033 | 0.50/0.39 | 0.3825 | 0.126 ms | 0.224 ms (4) | ✓ | s 0.0633 / d 0.6283 (n 200) |
| paw-local · paw-ft-bs48-20260530 | 300 | 0.5133 | — | — | — | — | — | — | — | — / — | — | 43.8 ms | 67.0 ms (4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **118/300** (39.3%) · answered-acc 0.8462 · quote-stripped 0 · program `1b5d4b654f60b508602c` (paw-ft-bs48-20260530, local-subprocess) · compile 88.6 s (cached) · server p50 —
refusal samples: ["qa_factoid", "iot_hue_lightchange", "iot_hue_sound", "screen_query", "iot_hue_lightdown", "iot_hue_lightoff", "iot_hub_power", "toaster_query"]

**G1 (modelless readout ECE):** raw 0.2873 · calibrated 0.6033 · conformal-naive floor 0.2015 → **FAIL** (does not beat both)

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

