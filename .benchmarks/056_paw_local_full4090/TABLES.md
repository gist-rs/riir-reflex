# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `634093f` on `shikuwa` (2026-09-26T20:05:28Z) · profile release · laya feature false
- box state (Issue 021): start power=? mode=? load=? swap=?M · end power=? mode=? load=? swap=?M — ⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: on — the SAME compiled programs answered through their LOCAL llama.cpp runtime as a Python subprocess oracle (programasweights, not affiliated; PAW_LOCAL_PYTHON + PAW_LOCAL_SCRIPT), never compiling — the program id comes from the hosted lane's cache so the local-vs-hosted delta isolates the runtime posture on identical artifacts; greedy by construction, determinism re-verified per run (observed-repeat, first 10); latency = in-process round-trip (no network); same mapping law + refusal accounting as --paw; .issues/033
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): OFF (nb_scale 0 — the published baseline posture)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- sampling: test sample = label-STRATIFIED round-robin over the whole test split (budget = the registry test cap; first-appearance label order, dataset order within each label, deterministic, no RNG); cal slice = the same law over the train rows; corpus pool = the train rows MINUS the cal front (excluded by construction, not position); budget-0 suites unchanged (identity split) — Issue 039 T2
- readout: shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side arming lever overfit the narrow suites' cal slice — emotion's test G1 regressed — and coincided with Dispatch on the wide suites it targeted; the wide-label G1 gap was closed by T2's stratified cal slice). The candidate table is recorded per suite (report-only); EngineConfig::readout stays the opt-in knob
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; harness_cache_reuse is LLM-lane only (the modelless lane has no KV cache) and reports a loud SKIPPED absence without the laya-riir feature

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4845 · max_prob 0.2262 · inv_entropy 0.4845

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.4050 | 0.3750 | 0.1465 | 0.7436 | 1.3735 | 0.5223 | 0.4750 | 0.0800 | 0.67/0.39 | 0.3951 | 0.202 ms | 0.416 ms (5) | ✓ | s 0.0001 / d 0.4105 (n 200) |
| paw-local · paw-ft-bs48-20260530 | 400 | 0.8000 | — | — | — | — | — | — | — | — / — | — | 264.0 ms | 1563.8 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **0/400** (0.0%) · answered-acc 0.8000 · quote-stripped 0 · program `35a7963d2d18aac54616` (paw-ft-bs48-20260530, local-subprocess) · compile 11.8 s (cached) · server p50 —

**G1 (modelless readout ECE):** raw 0.4046 · calibrated 0.0800 · conformal-naive floor 0.2282 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1449 · max_prob 0.0245 · inv_entropy 0.1449

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2750 | 0.1345 | 0.1053 | 0.8323 | 1.7886 | 0.7259 | 0.3000 | 0.1297 | 0.58/0.33 | 0.2910 | 0.101 ms | 0.124 ms (5) | ✓ | s 0.0000 / d 0.4265 (n 200) |
| paw-local · paw-ft-bs48-20260530 | 400 | 0.4875 | — | — | — | — | — | — | — | — / — | — | 113.7 ms | 213.4 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **1/400** (0.2%) · answered-acc 0.4887 · quote-stripped 0 · program `703c4b3c3acf668f8ad8` (paw-ft-bs48-20260530, local-subprocess) · compile 11.7 s (cached) · server p50 —
refusal samples: ["sleepiness"]

**G1 (modelless readout ECE):** raw 0.2749 · calibrated 0.1297 · conformal-naive floor 0.2982 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2442 · max_prob 0.0340 · inv_entropy 0.2442

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2017 | 0.1890 | 0.0087 | 0.7986 | 1.6059 | 0.8159 | 0.1833 | 0.0434 | 0.58/0.32 | 0.2044 | 0.087 ms | 0.127 ms (7) | ✓ | s 0.0002 / d 0.4166 (n 200) |
| paw-local · paw-ft-bs48-20260530 | 600 | 0.4167 | — | — | — | — | — | — | — | — / — | — | 118.3 ms | 210.7 ms (7) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **0/600** (0.0%) · answered-acc 0.4167 · quote-stripped 0 · program `a0fb83a0afc58a1ce2b9` (paw-ft-bs48-20260530, local-subprocess) · compile 11.7 s (cached) · server p50 — · score MAE (answered) 0.8600 · within-1 0.8167

**G1 (modelless readout ECE):** raw 0.2009 · calibrated 0.0434 · conformal-naive floor 0.3343 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1781 · within_1 0.4433

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

⛔ **corpus fallback (Issue 039 guard):** 45 option label(s) with NO train docs in the corpus pool — self-doc fallback only: Refund not showing up, activate my card, apple pay or google pay, atm support, balance not updated after bank transfer, beneficiary not allowed, card about to expire, card swallowed, cash withdrawal charge, cash withdrawal not recognised, change pin, compromised card, country support, declined card payment, declined cash withdrawal, declined transfer, direct debit payment not recognised, disposable card limits, exchange charge, failed transfer, get disposable virtual card, getting spare card, lost or stolen phone, order physical card, passcode forgotten, pending card payment, pending transfer, receiving money, request refund, reverted card payment?, terminate account, top up by card charge, top up by cash or cheque, top up failed, topping up by card, transaction charged twice, transfer fee charged, transfer into account, transfer timing, verify my identity, verify source of funds, verify top up, virtual card not working, visa or mastercard, wrong exchange rate for cash withdrawal

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.2600 | 0.2190 | 0.2416 | 0.9808 | 4.1290 | 0.6315 | 0.3560 | 0.2448 | 1.00/0.35 | 0.3354 | 0.276 ms | 0.606 ms (6) | ✓ | s 0.0361 / d 0.4928 (n 200) |
| paw-local · paw-ft-bs48-20260530 | 500 | 0.4120 | — | — | — | — | — | — | — | — / — | — | 131.7 ms | 236.7 ms (6) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **177/500** (35.4%) · answered-acc 0.6378 · quote-stripped 0 · program `ebbfe5a01ae044052526` (paw-ft-bs48-20260530, local-subprocess) · compile 12.0 s (cached) · server p50 —
refusal samples: ["locate my card", "wrong exchange rate for foreign exchange", "top up by day", "exchange currency", "contactless", "pending gas", "cancel transaction", "top up limit"]

**G1 (modelless readout ECE):** raw 0.2416 · calibrated 0.2448 · conformal-naive floor 0.2550 → **FAIL** (does not beat both)

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

