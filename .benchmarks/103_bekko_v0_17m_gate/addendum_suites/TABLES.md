# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `a2545b9` on `m3` (2026-10-02T02:16:23Z) · profile release · laya feature true
- box state (Issue 021): start power=AC Power mode=high load=14.06 swap=2619M · end power=AC Power mode=high load=9.06 swap=2619M — ⛔ latency NOT QUOTABLE — load 14.06 > 6 — a sibling job is on the box; load 9.06 > 6 — a sibling job is on the box
- laya device posture: cpu (LAYA_DEVICE or the no-backend default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: on — hotchpotch/bekko-system-one-v0 over their BekkoSentenceTransformer runtime as a JSONL subprocess oracle (comparison lane, never a product lane; the card assigns NO license yet — measurement-only): same cases, their softmax-over-candidates readout taken as-is, the state passed AS JSON (their reference law renders JSON states itself), score native when levels are numeric else choice-over-labels (disclosed), noul native with generic meanings; model id + revision ride BEKKO_MODEL/BEKKO_REVISION (default the card's 17M release); latency = subprocess round-trip (IPC included); determinism = the observed-repeat check; Bench 103
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16/32/64 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
- option-conditioned tables (issue 038 T7b): ON — cal-selected per suite (scale 0/0.25/0.5/1/2/4/8, promotion bar +5 pt over off on the stratified slice; one contrastive table per (question id, gold option) from the TRAIN rows, events filtered to the corpus pool; every question kind armed — typed_decisions is the target suite)
- NBSVM ridge readout (issue 038 T7a): ON — cal-selected per suite (scale 0/0.5/1/2/4/8 × λ 10 fixed, promotion bar +5 pt over off on the stratified slice; NBSVM closed-form ridge, k=2048, per-class NB log-count ratios, fit-time self-calibrated margin temperature)
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

**slice-integrity: test 400 (fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100 (fnv1a64-a1679de56e9278b1) · OK**

**corpus cap:** 48 (registry)

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.3600 |
| 1 | observed-laplace | — | bag | 0.3600 |
| 4 | observed-laplace | — | bag | 0.3600 |
| 16 | observed-laplace | — | bag | 0.3600 |
| 32 | observed-laplace | — | bag | 0.3600 |
| 64 | observed-laplace | — | bag | 0.3600 |
| 1 | fixed-1 | — | bag | 0.3600 |
| 4 | fixed-1 | — | bag | 0.3600 |
| 16 | fixed-1 | — | bag | 0.3600 |
| 32 | fixed-1 | — | bag | 0.3600 |
| 64 | fixed-1 | — | bag | 0.3600 |

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4823 · max_prob 0.1156 · inv_entropy 0.4823

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.5725 | 0.5429 | 0.0993 | 0.5634 | 0.9993 | 0.2967 | 0.6760 | 0.0496 | 1.00/0.72 | 0.6537 | 0.802 ms | 1.744 ms (5) | ✓ | s 0.5785 / d 0.9122 (n 100) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 2000 | 0.4840 | 0.4925 | 0.1476 | 0.6388 | 1.1055 | 0.3687 | 0.6010 | 0.1476 | — / — | — | 195.0 ms | 313.0 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4837 · calibrated 0.0496 · conformal-naive floor 0.1818 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3827 · brier_soft 0.1676 · score MAE 0.5718 · within_1 0.8163

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.5483 | 0.1482 | 0.4001 |
| modelless | noul | 600 | 0.7300 | 0.0682 | 0.6618 |
| modelless | score | 800 | 0.4725 | 0.0974 | 0.3865 |
**typed-decisions extras (bekko·hotchpotch/bekko-system-one-v0-68m):** soft_acc 0.4144 · brier_soft 0.2468 · score MAE 0.6564 · within_1 0.7562
| bekko·hotchpotch/bekko-system-one-v0-68m | choice | 600 | 0.4933 | 0.0512 | 0.5163 |
| bekko·hotchpotch/bekko-system-one-v0-68m | noul | 600 | 0.5633 | 0.2205 | 0.7839 |
| bekko·hotchpotch/bekko-system-one-v0-68m | score | 800 | 0.4175 | 0.1681 | 0.5844 |

## code_fixtures — 16 cases / 32 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4602 · max_prob 0.1479 · inv_entropy 0.4602

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 32 | 0.3750 | 0.1640 | 0.0948 | 0.6858 | 1.3870 | 0.4073 | 0.6250 | 0.0938 | 1.00/0.34 | 0.2857 | 0.231 ms | 0.392 ms (1, max@case9) | ✓ | s 0.4607 / d 0.3365 (n 64) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 32 | 0.4062 | 0.2872 | 0.2680 | 0.7144 | 1.4910 | 0.3933 | 0.6875 | 0.2680 | — / — | — | 91.0 ms | 189.0 ms (1, max@case9) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3742 · calibrated 0.0938 · conformal-naive floor 0.3510 → **PASS** (beats both the uncalibrated output AND the floor)

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

