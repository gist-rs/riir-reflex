# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `de67f50` on `4090-windows` (2026-09-27T02:02:29Z) · profile release · laya feature false
- box state (Issue 021): start power=? mode=? load=? swap=?M · end power=? mode=? load=? swap=?M — ⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: ON — cal-selected per suite (ladder 0/0.25/0.5/1, forced cal-slice accuracy, ties → 0 = off; issue 030 lever 4; per-row candidates in the results)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
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

## Absences / errors (honest — never silently dropped)

- harness_cache_reuse: harness_cache_reuse: SKIPPED — LLM-lane only (Issue 004 T3): the modelless lane has no KV cache, so a modelless answer here would be a fake task; compile with --features laya-riir (or clm-lane) to run this family

## typed_decisions — 400 cases / 2000 questions

**corpus cap:** 48 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.3700 |
| 0.25 | 0.3700 |
| 0.5 | 0.3700 |
| 1 | 0.3700 |

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.3700 |
| 1 | observed-laplace | — | bag | 0.3700 |
| 4 | observed-laplace | — | bag | 0.3700 |
| 16 | observed-laplace | — | bag | 0.3700 |
| 1 | fixed-1 | — | bag | 0.3700 |
| 4 | fixed-1 | — | bag | 0.3700 |
| 16 | fixed-1 | — | bag | 0.3700 |

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4708 · max_prob 0.1033 · inv_entropy 0.4708

⛔ **corpus fallback (Issue 039 guard):** 1 option label(s) with NO train docs in the corpus pool — self-doc fallback only: security_incidents

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.4655 | 0.4455 | 0.0750 | 0.6190 | 1.0998 | 0.3865 | 0.5680 | 0.0055 | 0.60/0.97 | 0.8636 | 1.113 ms | 5.066 ms (5) | ✓ | s 0.0165 / d 0.9294 (n 100) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4238 · calibrated 0.0055 · conformal-naive floor 0.2072 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3545 · brier_soft 0.1912 · score MAE 0.6710 · within_1 0.7462

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.4383 | 0.1013 | 0.3371 |
| modelless | noul | 600 | 0.6200 | 0.0930 | 0.5923 |
| modelless | score | 800 | 0.3700 | 0.0496 | 0.3204 |

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.3900 |
| 0.25 | 0.4150 |
| 0.5 | 0.4100 |
| 1 | 0.4050 |

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.3900 |
| 1 | observed-laplace | — | bag | 0.7000 |
| 4 ← selected | observed-laplace | — | bag | 0.7450 |
| 16 | observed-laplace | — | bag | 0.7450 |
| 1 | fixed-1 | — | bag | 0.7000 |
| 4 | fixed-1 | — | bag | 0.7450 |
| 16 | fixed-1 | — | bag | 0.7450 |

**transductive column (NOT the headline):** acc 0.8825 vs honest 0.8825 (+0.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.8204 · max_prob 0.4857 · inv_entropy 0.8204

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8825 | 0.8742 | 0.4783 | 0.5053 | 0.9630 | 0.0469 | 0.9400 | 0.0025 | 0.49/0.93 | 1.0000 | 0.322 ms | 0.818 ms (5) | ✓ | s 0.0111 / d 0.4105 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8305 · calibrated 0.0025 · conformal-naive floor 0.2482 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.1950 |
| 0.25 | 0.2100 |
| 0.5 | 0.2300 |
| 1 | 0.2350 |

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.1950 |
| 1 | observed-laplace | — | bag | 0.5350 |
| 4 ← selected | observed-laplace | — | bag | 0.5500 |
| 16 | observed-laplace | — | bag | 0.5500 |
| 1 | fixed-1 | — | bag | 0.4650 |
| 4 | fixed-1 | — | bag | 0.4850 |
| 16 | fixed-1 | — | bag | 0.4850 |

**transductive column (NOT the headline):** acc 0.8475 vs honest 0.8475 (+0.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.7053 · max_prob 0.4922 · inv_entropy 0.7053

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8475 | 0.7354 | 0.6053 | 0.7025 | 1.4580 | 0.0556 | 0.9450 | 0.0000 | 0.45/0.98 | 1.0000 | 0.182 ms | 0.359 ms (5) | ✓ | s 0.0047 / d 0.4265 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8324 · calibrated 0.0000 · conformal-naive floor 0.2040 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.2400 |
| 0.25 | 0.2350 |
| 0.5 | 0.2350 |
| 1 | 0.2300 |

**count tables:** cal-selected scale 16 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.2400 |
| 1 | observed-laplace | — | bag | 0.3350 |
| 4 | observed-laplace | — | bag | 0.3550 |
| 16 ← selected | observed-laplace | — | bag | 0.3600 |
| 1 | fixed-1 | — | bag | 0.3150 |
| 4 | fixed-1 | — | bag | 0.3400 |
| 16 | fixed-1 | — | bag | 0.3400 |

**transductive column (NOT the headline):** acc 0.3967 vs honest 0.3967 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2754 · max_prob 0.0464 · inv_entropy 0.2754

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.3967 | 0.3211 | 0.1393 | 0.7604 | 1.5156 | 0.5361 | 0.4700 | 0.0050 | 0.53/0.99 | 0.5714 | 0.158 ms | 0.609 ms (7) | ✓ | s 0.0047 / d 0.4166 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3849 · calibrated 0.0050 · conformal-naive floor 0.2220 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1170 · within_1 0.5400

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.5000 |
| 0.25 | 0.5000 |
| 0.5 | 0.5000 |
| 1 | 0.5000 |

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.5000 |
| 1 | observed-laplace | 0 | bag | 0.3900 |
| 4 | observed-laplace | 0 | bag | 0.3900 |
| 16 | observed-laplace | 0 | bag | 0.3900 |
| 1 | fixed-1 | 0 | bag | 0.5000 |
| 4 | fixed-1 | 0 | bag | 0.4900 |
| 16 | fixed-1 | 0 | bag | 0.4900 |
| 1 ← selected | observed-laplace | 1 | bag | 0.6100 |
| 4 | observed-laplace | 1 | bag | 0.6100 |
| 16 | observed-laplace | 1 | bag | 0.6100 |
| 1 | fixed-1 | 1 | bag | 0.5100 |
| 4 | fixed-1 | 1 | bag | 0.5100 |
| 16 | fixed-1 | 1 | bag | 0.5100 |

**transductive column (NOT the headline):** acc 0.7672 vs honest 0.7672 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.6831 · max_prob 0.1165 · inv_entropy 0.6831

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.7672 | 0.7643 | 0.1336 | 0.3365 | 0.5194 | 0.0949 | 0.8966 | 0.0348 | 0.68/0.91 | 1.0000 | 0.117 ms | 0.270 ms (2) | ✓ | s 0.0675 / d 0.4619 (n 100) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.6745 · calibrated 0.0348 · conformal-naive floor 0.3622 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.3650 |
| 0.25 | 0.3850 |
| 0.5 | 0.4000 |
| 1 ← selected | 0.4250 |

**count tables:** cal-selected scale 16 α fixed-1 view pair (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4250 |
| 1 | observed-laplace | — | bag | 0.3900 |
| 4 | observed-laplace | — | bag | 0.3400 |
| 16 | observed-laplace | — | bag | 0.3100 |
| 1 | fixed-1 | — | bag | 0.3950 |
| 4 | fixed-1 | — | bag | 0.3500 |
| 16 | fixed-1 | — | bag | 0.3200 |
| 1 | observed-laplace | — | pair | 0.4700 |
| 4 | observed-laplace | — | pair | 0.5600 |
| 16 | observed-laplace | — | pair | 0.5700 |
| 1 | fixed-1 | — | pair | 0.4800 |
| 4 | fixed-1 | — | pair | 0.5550 |
| 16 ← selected | fixed-1 | — | pair | 0.5850 |

**transductive column (NOT the headline):** acc 0.5067 vs honest 0.5233 (-1.7 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.5735 · max_prob 0.1904 · inv_entropy 0.5735

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.5233 | 0.5070 | 0.1162 | 0.6007 | 1.0047 | 0.3070 | 0.6667 | 0.0067 | 0.53/0.93 | 0.9000 | 0.152 ms | 0.366 ms (4) | ✓ | s 0.0029 / d 0.5110 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.5027 · calibrated 0.0067 · conformal-naive floor 0.1351 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.4500 |
| 0.25 | 0.4900 |
| 0.5 | 0.4950 |
| 1 | 0.4950 |

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4500 |
| 1 | observed-laplace | — | bag | 0.5850 |
| 4 ← selected | observed-laplace | — | bag | 0.6050 |
| 16 | observed-laplace | — | bag | 0.5950 |
| 1 | fixed-1 | — | bag | 0.5500 |
| 4 | fixed-1 | — | bag | 0.5850 |
| 16 | fixed-1 | — | bag | 0.5650 |

**transductive column (NOT the headline):** acc 0.7833 vs honest 0.7800 (+0.3 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.7800 | 0.7698 | 0.6352 | 0.7856 | 2.0599 | 0.0553 | 0.9667 | 0.0796 | 0.31/0.31 | 0.7681 | 0.183 ms | 0.256 ms (4) | ✓ | s 0.0581 / d 0.5259 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.6352 · calibrated 0.0796 · conformal-naive floor 0.1209 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.4650 |
| 0.25 | 0.5400 |
| 0.5 | 0.5650 |
| 1 ← selected | 0.6200 |

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.6200 |
| 1 ← selected | observed-laplace | — | bag | 0.8200 |
| 4 | observed-laplace | — | bag | 0.8000 |
| 16 | observed-laplace | — | bag | 0.8050 |
| 1 | fixed-1 | — | bag | 0.8000 |
| 4 | fixed-1 | — | bag | 0.7800 |
| 16 | fixed-1 | — | bag | 0.7650 |

**transductive column (NOT the headline):** acc 0.8300 vs honest 0.8260 (+0.4 pt; 500 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.8260 | 0.8210 | 0.8015 | 0.9653 | 3.7538 | 0.0650 | 0.9720 | 0.1740 | 0.47/0.32 | 0.8555 | 0.634 ms | 2.391 ms (6) | ✓ | s 0.0205 / d 0.5549 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8015 · calibrated 0.1740 · conformal-naive floor 0.2887 → **PASS** (beats both the uncalibrated output AND the floor)

## code_fixtures — 12 cases / 24 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.5498 · max_prob 0.2221 · inv_entropy 0.5498

⛔ **corpus fallback (Issue 039 guard):** 4 option label(s) with NO train docs in the corpus pool — self-doc fallback only: embed, readout, laya::agent, laya::router

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 24 | 0.2500 | 0.0936 | 0.0936 | 0.6896 | 1.3870 | 0.6764 | 0.3333 | 0.3026 | 0.12/0.12 | 0.2381 | 0.307 ms | 0.556 ms (1, max@case8) | ✓ | s 0.0001 / d 0.2851 (n 38) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2457 · calibrated 0.3026 · conformal-naive floor 0.5103 → **FAIL** (does not beat both)

## harness_visibility — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.008 ms | 0.009 ms (1, max@case0) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_permissions — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.006 ms | 0.007 ms (1, max@case7) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_tool_fit — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.007 ms | 0.008 ms (1, max@case1) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_routing — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.008 ms | 0.017 ms (1, max@case3) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_sensitivity — 15 cases / 15 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.007 ms | 0.008 ms (1, max@case0) | ✓ | s 0.0111 / d 0.4172 (n 20) |
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

