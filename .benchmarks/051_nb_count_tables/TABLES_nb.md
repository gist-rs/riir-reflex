# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `d66ef21` on `m3` (2026-09-26T12:48:12Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=4.11 swap=2551M · end power=AC Power mode=high load=3.81 swap=2551M — latency QUOTABLE
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- corpus cap posture: registry defaults
- label heads: ON — cal-selected per suite (ladder 0/0.25/0.5/1, forced cal-slice accuracy, ties → 0 = off; issue 030 lever 4; per-row candidates in the results)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; harness_cache_reuse is LLM-lane only (the modelless lane has no KV cache) and reports a loud SKIPPED absence without the laya-riir feature

## typed_decisions — 400 cases / 2000 questions

**corpus cap:** 48 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.3380 |
| 0.25 | 0.3380 |
| 0.5 | 0.3380 |
| 1 | 0.3380 |

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.3380 |
| 1 | observed-laplace | — | bag | 0.3380 |
| 4 | observed-laplace | — | bag | 0.3380 |
| 16 | observed-laplace | — | bag | 0.3380 |
| 1 | fixed-1 | — | bag | 0.3380 |
| 4 | fixed-1 | — | bag | 0.3380 |
| 16 | fixed-1 | — | bag | 0.3380 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3190 | 0.2270 | 0.0527 | 0.6924 | 1.2331 | 0.5525 | 0.4400 | 0.0805 | 0.82/0.93 | 0.3493 | 0.569 ms | 1.756 ms (5) | ✓ | s 0.0139 / d 0.9213 (n 100) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2933 · calibrated 0.0805 · conformal-naive floor 0.3087 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3166 · brier_soft 0.2459 · score MAE 0.7394 · within_1 0.7037

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1867 | 0.0828 | 0.2695 |
| modelless | noul | 600 | 0.5317 | 0.0250 | 0.5067 |
| modelless | score | 800 | 0.2587 | 0.0540 | 0.3096 |

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.4800 |
| 0.25 | 0.4950 |
| 0.5 | 0.5000 |
| 1 | 0.5050 |

**count tables:** cal-selected scale 16 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4800 |
| 1 | observed-laplace | — | bag | 0.8050 |
| 4 | observed-laplace | — | bag | 0.8200 |
| 16 ← selected | observed-laplace | — | bag | 0.8300 |
| 1 | fixed-1 | — | bag | 0.7950 |
| 4 | fixed-1 | — | bag | 0.8050 |
| 16 | fixed-1 | — | bag | 0.8150 |

**transductive column (NOT the headline):** acc 0.8750 vs honest 0.8750 (+0.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8750 | 0.8661 | 0.3947 | 0.4182 | 0.8240 | 0.0537 | 0.9400 | 0.0125 | 0.37/0.79 | 0.9651 | 0.159 ms | 0.205 ms (5) | ✓ | s 0.0151 / d 0.3709 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.7503 · calibrated 0.0125 · conformal-naive floor 0.1857 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.1850 |
| 0.25 | 0.2050 |
| 0.5 | 0.2050 |
| 1 | 0.2100 |

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.1850 |
| 1 | observed-laplace | — | bag | 0.4000 |
| 4 ← selected | observed-laplace | — | bag | 0.4100 |
| 16 | observed-laplace | — | bag | 0.4000 |
| 1 | fixed-1 | — | bag | 0.2800 |
| 4 | fixed-1 | — | bag | 0.2800 |
| 16 | fixed-1 | — | bag | 0.2900 |

**transductive column (NOT the headline):** acc 0.5550 vs honest 0.5950 (-4.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5950 | 0.4132 | 0.3634 | 0.7452 | 1.5594 | 0.2507 | 0.7500 | 0.0000 | 0.58/0.36 | 0.5820 | 0.120 ms | 0.144 ms (5) | ✓ | s 0.0068 / d 0.4153 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.5814 · calibrated 0.0000 · conformal-naive floor 0.1419 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0.5 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.2050 |
| 0.25 | 0.2450 |
| 0.5 ← selected | 0.2550 |
| 1 | 0.2500 |

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.2550 |
| 1 | observed-laplace | — | bag | 0.2550 |
| 4 | observed-laplace | — | bag | 0.2700 |
| 16 | observed-laplace | — | bag | 0.2750 |
| 1 | fixed-1 | — | bag | 0.2650 |
| 4 | fixed-1 | — | bag | 0.2750 |
| 16 | fixed-1 | — | bag | 0.2800 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2167 | 0.2164 | 0.0020 | 0.7992 | 1.6073 | 0.7985 | 0.1767 | 0.0215 | 0.54/0.32 | 0.2044 | 0.102 ms | 0.126 ms (7) | ✓ | s 0.0004 / d 0.3973 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2155 · calibrated 0.0215 · conformal-naive floor 0.3462 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1784 · within_1 0.4317

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.5000 |
| 0.25 | 0.5000 |
| 0.5 | 0.5000 |
| 1 | 0.5000 |

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.5000 |
| 1 | observed-laplace | 0 | bag | 0.5200 |
| 4 | observed-laplace | 0 | bag | 0.5200 |
| 16 | observed-laplace | 0 | bag | 0.5200 |
| 1 | fixed-1 | 0 | bag | 0.5000 |
| 4 | fixed-1 | 0 | bag | 0.5000 |
| 16 | fixed-1 | 0 | bag | 0.5000 |
| 1 | observed-laplace | 1 | bag | 0.4800 |
| 4 | observed-laplace | 1 | bag | 0.4800 |
| 16 | observed-laplace | 1 | bag | 0.4800 |
| 1 | fixed-1 | 1 | bag | 0.5000 |
| 4 | fixed-1 | 1 | bag | 0.5000 |
| 16 | fixed-1 | 1 | bag | 0.5000 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5004 | 0.6936 | 0.3981 | 0.6552 | 0.3663 | 0.41/0.41 | 0.4118 | 0.058 ms | 0.077 ms (2) | ✓ | s 0.0001 / d 0.4079 (n 100) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4827 · calibrated 0.3663 · conformal-naive floor 0.5073 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.3400 |
| 0.25 | 0.3550 |
| 0.5 | 0.3300 |
| 1 | 0.3600 |

**count tables:** cal-selected scale 4 α fixed-1 view pair (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.3400 |
| 1 | observed-laplace | — | bag | 0.3450 |
| 4 | observed-laplace | — | bag | 0.3600 |
| 16 | observed-laplace | — | bag | 0.3600 |
| 1 | fixed-1 | — | bag | 0.3200 |
| 4 | fixed-1 | — | bag | 0.3450 |
| 16 | fixed-1 | — | bag | 0.3450 |
| 1 | observed-laplace | — | pair | 0.4800 |
| 4 | observed-laplace | — | pair | 0.4900 |
| 16 | observed-laplace | — | pair | 0.4950 |
| 1 | fixed-1 | — | pair | 0.5350 |
| 4 ← selected | fixed-1 | — | pair | 0.5400 |
| 16 | fixed-1 | — | pair | 0.5350 |

**transductive column (NOT the headline):** acc 0.4933 vs honest 0.5200 (-2.7 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.5200 | 0.5026 | 0.1338 | 0.6183 | 1.0299 | 0.3049 | 0.6733 | 0.0033 | 0.62/0.94 | 0.9444 | 0.090 ms | 0.109 ms (4) | ✓ | s 0.0019 / d 0.5124 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.0033 · conformal-naive floor 0.1463 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.5150 |
| 0.25 | 0.5850 |
| 0.5 | 0.5950 |
| 1 ← selected | 0.6100 |

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.6100 |
| 1 ← selected | observed-laplace | — | bag | 0.7000 |
| 4 | observed-laplace | — | bag | 0.6900 |
| 16 | observed-laplace | — | bag | 0.7000 |
| 1 | fixed-1 | — | bag | 0.6850 |
| 4 | fixed-1 | — | bag | 0.6550 |
| 16 | fixed-1 | — | bag | 0.6350 |

**transductive column (NOT the headline):** acc 0.9300 vs honest 0.9267 (+0.3 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.9267 | 0.9211 | 0.8153 | 0.8299 | 2.2083 | 0.0086 | 1.0000 | 0.1218 | 0.42/0.35 | 0.9433 | 0.097 ms | 0.129 ms (4) | ✓ | s 0.0884 / d 0.6319 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8153 · calibrated 0.1218 · conformal-naive floor 0.3226 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.4900 |
| 0.25 | 0.6250 |
| 0.5 | 0.6500 |
| 1 ← selected | 0.6800 |

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.6800 |
| 1 ← selected | observed-laplace | — | bag | 0.8700 |
| 4 | observed-laplace | — | bag | 0.8650 |
| 16 | observed-laplace | — | bag | 0.8650 |
| 1 | fixed-1 | — | bag | 0.8300 |
| 4 | fixed-1 | — | bag | 0.8350 |
| 16 | fixed-1 | — | bag | 0.8150 |

**transductive column (NOT the headline):** acc 0.8640 vs honest 0.8700 (-0.6 pt; 500 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.8700 | 0.4882 | 0.8379 | 0.9518 | 3.5070 | 0.0224 | 0.9880 | 0.0000 | 0.97/0.99 | 1.0000 | 0.375 ms | 0.753 ms (6) | ✓ | s 0.0444 / d 0.4730 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8379 · calibrated 0.0000 · conformal-naive floor 0.8403 → **PASS** (beats both the uncalibrated output AND the floor)

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

