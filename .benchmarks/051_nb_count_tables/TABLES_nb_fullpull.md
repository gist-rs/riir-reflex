# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `754eca0` on `m3` (2026-09-26T13:33:45Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=3.20 swap=2543M · end power=AC Power mode=high load=3.44 swap=2543M — latency QUOTABLE
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

## Absences / errors (honest — never silently dropped)

- harness_cache_reuse: harness_cache_reuse: SKIPPED — LLM-lane only (Issue 004 T3): the modelless lane has no KV cache, so a modelless answer here would be a fake task; compile with --features laya-riir (or clm-lane) to run this family

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
| modelless | 2000 | 0.3190 | 0.2270 | 0.0527 | 0.6924 | 1.2331 | 0.5525 | 0.4400 | 0.0805 | 0.82/0.93 | 0.3493 | 0.536 ms | 1.786 ms (5) | ✓ | s 0.0139 / d 0.9213 (n 100) |
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

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4800 |
| 1 ← selected | observed-laplace | — | bag | 0.8450 |
| 4 | observed-laplace | — | bag | 0.8450 |
| 16 | observed-laplace | — | bag | 0.8450 |
| 1 | fixed-1 | — | bag | 0.8450 |
| 4 | fixed-1 | — | bag | 0.8450 |
| 16 | fixed-1 | — | bag | 0.8400 |

**transductive column (NOT the headline):** acc 0.8800 vs honest 0.8825 (-0.2 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8825 | 0.8757 | 0.5679 | 0.6383 | 1.1803 | 0.0494 | 0.9350 | 0.0025 | 0.33/0.88 | 1.0000 | 0.154 ms | 0.195 ms (5) | ✓ | s 0.0014 / d 0.3709 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8727 · calibrated 0.0025 · conformal-naive floor 0.1550 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.1850 |
| 0.25 | 0.2050 |
| 0.5 | 0.2050 |
| 1 | 0.2100 |

**count tables:** cal-selected scale 16 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.1850 |
| 1 | observed-laplace | — | bag | 0.5750 |
| 4 | observed-laplace | — | bag | 0.5800 |
| 16 ← selected | observed-laplace | — | bag | 0.5850 |
| 1 | fixed-1 | — | bag | 0.4900 |
| 4 | fixed-1 | — | bag | 0.5100 |
| 16 | fixed-1 | — | bag | 0.5150 |

**transductive column (NOT the headline):** acc 0.7300 vs honest 0.7375 (-0.8 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.7375 | 0.6124 | 0.4686 | 0.6759 | 1.4046 | 0.1199 | 0.9000 | 0.0000 | 0.52/0.99 | 1.0000 | 0.115 ms | 0.140 ms (5) | ✓ | s 0.0098 / d 0.4153 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.7055 · calibrated 0.0000 · conformal-naive floor 0.1933 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0.5 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.2050 |
| 0.25 | 0.2450 |
| 0.5 ← selected | 0.2550 |
| 1 | 0.2500 |

**count tables:** cal-selected scale 16 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.2550 |
| 1 | observed-laplace | — | bag | 0.3000 |
| 4 | observed-laplace | — | bag | 0.3150 |
| 16 ← selected | observed-laplace | — | bag | 0.3350 |
| 1 | fixed-1 | — | bag | 0.2800 |
| 4 | fixed-1 | — | bag | 0.3000 |
| 16 | fixed-1 | — | bag | 0.3200 |

**transductive column (NOT the headline):** acc 0.3917 vs honest 0.3917 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.3917 | 0.3189 | 0.1368 | 0.7621 | 1.5194 | 0.5378 | 0.4767 | 0.0050 | 0.55/0.99 | 0.8000 | 0.097 ms | 0.120 ms (7) | ✓ | s 0.0044 / d 0.3973 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3810 · calibrated 0.0050 · conformal-naive floor 0.2038 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1198 · within_1 0.5433

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
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5004 | 0.6936 | 0.3981 | 0.6552 | 0.3663 | 0.41/0.41 | 0.4118 | 0.056 ms | 0.069 ms (2) | ✓ | s 0.0001 / d 0.4079 (n 100) |
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
| 1 | observed-laplace | — | bag | 0.3150 |
| 4 | observed-laplace | — | bag | 0.3400 |
| 16 | observed-laplace | — | bag | 0.3500 |
| 1 | fixed-1 | — | bag | 0.3100 |
| 4 | fixed-1 | — | bag | 0.3250 |
| 16 | fixed-1 | — | bag | 0.3550 |
| 1 | observed-laplace | — | pair | 0.5800 |
| 4 | observed-laplace | — | pair | 0.5850 |
| 16 | observed-laplace | — | pair | 0.5900 |
| 1 | fixed-1 | — | pair | 0.5850 |
| 4 ← selected | fixed-1 | — | pair | 0.6000 |
| 16 | fixed-1 | — | pair | 0.6000 |

**transductive column (NOT the headline):** acc 0.5067 vs honest 0.5233 (-1.7 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.5233 | 0.5045 | 0.1381 | 0.6176 | 1.0283 | 0.3121 | 0.6467 | 0.0567 | 0.58/0.38 | 0.5297 | 0.087 ms | 0.096 ms (4) | ✓ | s 0.0014 / d 0.5124 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.5132 · calibrated 0.0567 · conformal-naive floor 0.1204 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.4300 |
| 0.25 | 0.4850 |
| 0.5 | 0.4850 |
| 1 ← selected | 0.5050 |

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.5050 |
| 1 ← selected | observed-laplace | — | bag | 0.5950 |
| 4 | observed-laplace | — | bag | 0.5800 |
| 16 | observed-laplace | — | bag | 0.5800 |
| 1 | fixed-1 | — | bag | 0.5700 |
| 4 | fixed-1 | — | bag | 0.5600 |
| 16 | fixed-1 | — | bag | 0.5400 |

**transductive column (NOT the headline):** acc 0.9033 vs honest 0.8967 (+0.7 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.8967 | 0.8883 | 0.7970 | 0.8575 | 2.3509 | 0.0194 | 0.9800 | 0.3316 | 0.26/0.26 | 0.8919 | 0.117 ms | 0.138 ms (4) | ✓ | s 0.0577 / d 0.5258 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.7970 · calibrated 0.3316 · conformal-naive floor 0.1932 → **FAIL** (does not beat both)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.3900 |
| 0.25 | 0.5200 |
| 0.5 | 0.5350 |
| 1 ← selected | 0.5400 |

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.5400 |
| 1 | observed-laplace | — | bag | 0.8150 |
| 4 ← selected | observed-laplace | — | bag | 0.8200 |
| 16 | observed-laplace | — | bag | 0.8150 |
| 1 | fixed-1 | — | bag | 0.7400 |
| 4 | fixed-1 | — | bag | 0.7800 |
| 16 | fixed-1 | — | bag | 0.7700 |

**transductive column (NOT the headline):** acc 0.7680 vs honest 0.7700 (-0.2 pt; 500 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.7700 | 0.2148 | 0.7282 | 0.9355 | 3.3560 | 0.0552 | 0.9760 | 0.1394 | 0.27/0.29 | 0.8697 | 0.363 ms | 0.685 ms (6) | ✓ | s 0.0232 / d 0.4798 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.7282 · calibrated 0.1394 · conformal-naive floor 0.0420 → **FAIL** (does not beat both)

## code_fixtures — 12 cases / 24 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 24 | 0.2500 | 0.1517 | 0.0920 | 0.6881 | 1.3864 | 0.7148 | 0.3333 | 0.2785 | 0.38/0.38 | 0.2667 | 0.279 ms | 0.347 ms (1, max@case8) | ✓ | s 0.0001 / d 0.3169 (n 35) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2457 · calibrated 0.2785 · conformal-naive floor 0.3867 → **FAIL** (does not beat both)

## harness_visibility — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.009 ms | 0.010 ms (1, max@case0) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_permissions — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.006 ms | 0.007 ms (1, max@case0) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_tool_fit — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.008 ms | 0.008 ms (1, max@case0) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_routing — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.008 ms | 0.010 ms (1, max@case0) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_sensitivity — 15 cases / 15 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.008 ms | 0.009 ms (1, max@case1) | ✓ | s 0.0111 / d 0.4172 (n 20) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3816 · calibrated 0.3816 · conformal-naive floor 0.3429 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

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

