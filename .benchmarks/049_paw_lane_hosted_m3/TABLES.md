# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `20c949f` on `m3` (2026-09-26T06:35:16Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=18.71 swap=1452M · end power=AC Power mode=high load=38.11 swap=1420M — ⛔ latency NOT QUOTABLE — load 18.71 > 6 — a sibling job is on the box; load 38.11 > 6 — a sibling job is on the box
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: on — ProgramAsWeights (MIT SDK, not affiliated) over their hosted REST via a curl subprocess, posture hosted-anonymous (anonymous = their free tier; programs compile PUBLIC); one program per specced suite (scripts/paw_specs/, cached by (suite, compiler, BLAKE3(spec))); temperature 0, max_tokens 48; free-text answers mapped by the exact-match law (one surrounding quote pair stripped, counted), unparseable = a counted REFUSAL scored wrong, never guessed; NO confidence/ECE columns (no probability surface — disclosed divergence); latency = client round-trip incl. network + curl spawn; hosted inference not promised deterministic (observed-repeat check); .issues/033
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; harness_cache_reuse is LLM-lane only (the modelless lane has no KV cache) and reports a loud SKIPPED absence without the laya-riir feature

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5100 | 0.4725 | 0.2493 | 0.7419 | 1.3702 | 0.4899 | 0.5350 | 0.3797 | 0.49/0.26 | 0.5436 | 0.154 ms | 0.194 ms (5) | ✓ | s 0.0001 / d 0.3709 (n 200) |
| paw · paw-4b-qwen3-0.6b-20260407 | 400 | 0.7825 | — | — | — | — | — | — | — | — / — | — | 981.9 ms | 3775.9 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **0/400** (0.0%) · answered-acc 0.7825 · quote-stripped 0 · program `06f283857d30b77a18fc` (paw-4b-qwen3-0.6b-20260407, hosted-anonymous) · compile 4.6 s · server p50 94.5 ms

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.3797 · conformal-naive floor 0.2506 → **FAIL** (does not beat both)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2825 | 0.1511 | 0.1128 | 0.8319 | 1.7874 | 0.7236 | 0.2850 | 0.0226 | 0.62/0.36 | 0.2773 | 0.122 ms | 0.158 ms (5) | ✓ | s 0.0000 / d 0.4153 (n 200) |
| paw · paw-4b-qwen3-0.6b-20260407 | 400 | 0.4750 | — | — | — | — | — | — | — | — / — | — | 1046.1 ms | 4299.9 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **2/400** (0.5%) · answered-acc 0.4774 · quote-stripped 393 · program `25f883f21290e923a627` (paw-4b-qwen3-0.6b-20260407, hosted-anonymous) · compile 4.1 s · server p50 112.8 ms
refusal samples: ["\"hope\"", "\"sex\""]

**G1 (modelless readout ECE):** raw 0.2824 · calibrated 0.0226 · conformal-naive floor 0.2931 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2167 | 0.2044 | 0.0065 | 0.7988 | 1.6063 | 0.7849 | 0.2067 | 0.0165 | 0.57/0.32 | 0.2068 | 0.099 ms | 0.120 ms (7) | ✓ | s 0.0002 / d 0.3973 (n 200) |
| paw · paw-4b-qwen3-0.6b-20260407 | 600 | 0.3283 | — | — | — | — | — | — | — | — / — | — | 1044.8 ms | 2189.5 ms (7) | ✗ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **0/600** (0.0%) · answered-acc 0.3283 · quote-stripped 581 · program `0e0968ebe6a700f0f4ad` (paw-4b-qwen3-0.6b-20260407, hosted-anonymous) · compile 0.8 s · server p50 112.8 ms · score MAE (answered) 0.8450 · within-1 0.8517

**G1 (modelless readout ECE):** raw 0.2159 · calibrated 0.0165 · conformal-naive floor 0.3300 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1774 · within_1 0.4450

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.4460 | 0.1515 | 0.4275 | 0.9778 | 4.0279 | 0.4253 | 0.5680 | 0.1810 | 1.00/0.23 | 0.5325 | 0.397 ms | 1.529 ms (6) | ✓ | s 0.0360 / d 0.4730 (n 200) |
| paw · paw-4b-qwen3-0.6b-20260407 | 500 | 0.1400 | — | — | — | — | — | — | — | — / — | — | 1048.2 ms | 2068.9 ms (6) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**PAW lane:** refusals **365/500** (73.0%) · answered-acc 0.5185 · quote-stripped 0 · program `f2c6df69e759026ade01` (paw-4b-qwen3-0.6b-20260407, hosted-anonymous) · compile 4.3 s · server p50 94.8 ms
refusal samples: ["locate card", "new card not received", "card status", "card status", "track new card", "still waiting on that card", "new card wait", "track card"]

**G1 (modelless readout ECE):** raw 0.4275 · calibrated 0.1810 · conformal-naive floor 0.4410 → **PASS** (beats both the uncalibrated output AND the floor)

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

