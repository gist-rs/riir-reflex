# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `5587bda` on `m3` (2026-10-01T17:00:05Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=13.87 swap=2683M · end power=AC Power mode=high load=22.65 swap=2675M — ⛔ latency NOT QUOTABLE — load 13.87 > 6 — a sibling job is on the box; load 22.65 > 6 — a sibling job is on the box
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: on — hotchpotch/bekko-system-one-v0 over their BekkoSentenceTransformer runtime as a JSONL subprocess oracle (comparison lane, never a product lane; the card assigns NO license yet — measurement-only): same cases, their softmax-over-candidates readout taken as-is, the state passed AS JSON (their reference law renders JSON states itself), score native when levels are numeric else choice-over-labels (disclosed), noul native with generic meanings; model id + revision ride BEKKO_MODEL/BEKKO_REVISION (default the card's 17M release); latency = subprocess round-trip (IPC included); determinism = the observed-repeat check; Bench 103
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16/32/64 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
- option-conditioned tables (issue 038 T7b): 
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

## ag_news — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.3900 |
| 1 | observed-laplace | — | bag | 0.7000 |
| 4 ← selected | observed-laplace | — | bag | 0.7450 |
| 16 | observed-laplace | — | bag | 0.7450 |
| 32 | observed-laplace | — | bag | 0.7450 |
| 64 | observed-laplace | — | bag | 0.7450 |
| 1 | fixed-1 | — | bag | 0.7000 |
| 4 | fixed-1 | — | bag | 0.7450 |
| 16 | fixed-1 | — | bag | 0.7450 |
| 32 | fixed-1 | — | bag | 0.7450 |
| 64 | fixed-1 | — | bag | 0.7450 |

**transductive column (NOT the headline):** acc 0.8825 vs honest 0.8825 (+0.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.8204 · max_prob 0.4857 · inv_entropy 0.8204

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8825 | 0.8742 | 0.4783 | 0.5053 | 0.9630 | 0.0469 | 0.9400 | 0.0459 | 1.00/0.49 | 0.9510 | 0.180 ms | 0.567 ms (5) | ✓ | s 0.8359 / d 0.4105 (n 200) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 400 | 0.9000 | 0.8899 | 0.0274 | 0.1344 | 0.2453 | 0.0172 | 0.9900 | 0.0274 | — / — | — | 87.0 ms | 120.0 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8305 · calibrated 0.0459 · conformal-naive floor 0.2482 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15800 (fnv1a64-a05205a8dc7160cb) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.1950 |
| 1 | observed-laplace | — | bag | 0.5350 |
| 4 ← selected | observed-laplace | — | bag | 0.5500 |
| 16 | observed-laplace | — | bag | 0.5500 |
| 32 | observed-laplace | — | bag | 0.5500 |
| 64 | observed-laplace | — | bag | 0.5500 |
| 1 | fixed-1 | — | bag | 0.4650 |
| 4 | fixed-1 | — | bag | 0.4850 |
| 16 | fixed-1 | — | bag | 0.4850 |
| 32 | fixed-1 | — | bag | 0.4850 |
| 64 | fixed-1 | — | bag | 0.4850 |

**transductive column (NOT the headline):** acc 0.8850 vs honest 0.8850 (+0.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.7590 · max_prob 0.5277 · inv_entropy 0.7590

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8850 | 0.8014 | 0.6185 | 0.6597 | 1.3652 | 0.0360 | 0.9750 | 0.0513 | 1.00/0.46 | 0.9349 | 0.141 ms | 0.297 ms (5) | ✓ | s 0.7632 / d 0.4265 (n 200) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 400 | 0.4825 | 0.4159 | 0.0969 | 0.6635 | 1.4239 | 0.3489 | 0.6450 | 0.0969 | — / — | — | 65.0 ms | 92.0 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8622 · calibrated 0.0513 · conformal-naive floor 0.2780 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**slice-integrity: test 600 (fnv1a64-67fe9666fa0c3a6f) · cal 200 (fnv1a64-d210e7c6707c3d40) · pool 8344 (fnv1a64-cf933108b9825392) · VIOLATIONS**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 16 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.2400 |
| 1 | observed-laplace | — | bag | 0.3350 |
| 4 | observed-laplace | — | bag | 0.3550 |
| 16 ← selected | observed-laplace | — | bag | 0.3600 |
| 32 | observed-laplace | — | bag | 0.3600 |
| 64 | observed-laplace | — | bag | 0.3600 |
| 1 | fixed-1 | — | bag | 0.3150 |
| 4 | fixed-1 | — | bag | 0.3400 |
| 16 | fixed-1 | — | bag | 0.3400 |
| 32 | fixed-1 | — | bag | 0.3350 |
| 64 | fixed-1 | — | bag | 0.3350 |

**transductive column (NOT the headline):** acc 0.3967 vs honest 0.3967 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2754 · max_prob 0.0464 · inv_entropy 0.2754

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.3967 | 0.3211 | 0.1393 | 0.7604 | 1.5156 | 0.5361 | 0.4700 | 0.1086 | 1.00/0.53 | 0.4057 | 0.114 ms | 0.188 ms (7) | ✓ | s 0.2813 / d 0.4166 (n 200) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 600 | 0.4050 | 0.3345 | 0.0877 | 0.6704 | 1.2816 | 0.4389 | 0.5233 | 0.0877 | — / — | — | 66.0 ms | 97.0 ms (7) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3849 · calibrated 0.1086 · conformal-naive floor 0.2220 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1170 · within_1 0.5400

## prompt_injections — 116 cases / 116 questions

**slice-integrity: test 116 (fnv1a64-a6a375f0273e0bef) · cal 100 (fnv1a64-a45e6561b2a98ff1) · pool 446 (fnv1a64-8b32b961bbd47195) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.5000 |
| 1 | observed-laplace | 0 | bag | 0.3900 |
| 4 | observed-laplace | 0 | bag | 0.3900 |
| 16 | observed-laplace | 0 | bag | 0.3900 |
| 32 | observed-laplace | 0 | bag | 0.3900 |
| 64 | observed-laplace | 0 | bag | 0.3900 |
| 1 | fixed-1 | 0 | bag | 0.5000 |
| 4 | fixed-1 | 0 | bag | 0.4900 |
| 16 | fixed-1 | 0 | bag | 0.4900 |
| 32 | fixed-1 | 0 | bag | 0.4900 |
| 64 | fixed-1 | 0 | bag | 0.4900 |
| 1 ← selected | observed-laplace | 1 | bag | 0.6100 |
| 4 | observed-laplace | 1 | bag | 0.6100 |
| 16 | observed-laplace | 1 | bag | 0.6100 |
| 32 | observed-laplace | 1 | bag | 0.6100 |
| 64 | observed-laplace | 1 | bag | 0.6100 |
| 1 | fixed-1 | 1 | bag | 0.5100 |
| 4 | fixed-1 | 1 | bag | 0.5100 |
| 16 | fixed-1 | 1 | bag | 0.5100 |
| 32 | fixed-1 | 1 | bag | 0.5100 |
| 64 | fixed-1 | 1 | bag | 0.5100 |

**transductive column (NOT the headline):** acc 0.7672 vs honest 0.7672 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.6831 · max_prob 0.1165 · inv_entropy 0.6831

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.7672 | 0.7643 | 0.1336 | 0.3365 | 0.5194 | 0.0949 | 0.8966 | 0.0949 | 1.00/0.68 | 0.9459 | 0.086 ms | 0.109 ms (2) | ✓ | s 0.7991 / d 0.4619 (n 100) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 116 | 0.5000 | 0.3743 | 0.3691 | 0.7454 | 1.1184 | 0.4140 | 0.6034 | 0.3691 | — / — | — | 80.0 ms | 117.0 ms (2) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.6745 · calibrated 0.0949 · conformal-naive floor 0.3622 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 4 α fixed-1 view pair (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.3650 |
| 1 | observed-laplace | — | bag | 0.3100 |
| 4 | observed-laplace | — | bag | 0.3400 |
| 16 | observed-laplace | — | bag | 0.3250 |
| 32 | observed-laplace | — | bag | 0.3250 |
| 64 | observed-laplace | — | bag | 0.3200 |
| 1 | fixed-1 | — | bag | 0.3100 |
| 4 | fixed-1 | — | bag | 0.3300 |
| 16 | fixed-1 | — | bag | 0.3250 |
| 32 | fixed-1 | — | bag | 0.3300 |
| 64 | fixed-1 | — | bag | 0.3300 |
| 1 | observed-laplace | — | pair | 0.5600 |
| 4 | observed-laplace | — | pair | 0.5500 |
| 16 | observed-laplace | — | pair | 0.5500 |
| 32 | observed-laplace | — | pair | 0.5500 |
| 64 | observed-laplace | — | pair | 0.5500 |
| 1 | fixed-1 | — | pair | 0.5600 |
| 4 ← selected | fixed-1 | — | pair | 0.5800 |
| 16 | fixed-1 | — | pair | 0.5700 |
| 32 | fixed-1 | — | pair | 0.5650 |
| 64 | fixed-1 | — | pair | 0.5550 |

**transductive column (NOT the headline):** acc 0.5033 vs honest 0.5233 (-2.0 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.5868 · max_prob 0.2135 · inv_entropy 0.5868

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.5233 | 0.5061 | 0.1383 | 0.6178 | 1.0287 | 0.3120 | 0.6400 | 0.1119 | 1.00/0.54 | 0.6087 | 0.102 ms | 0.126 ms (4) | ✓ | s 0.5569 / d 0.5110 (n 200) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 300 | 0.6767 | 0.6755 | 0.0430 | 0.4577 | 0.8200 | 0.2033 | 0.7933 | 0.0430 | — / — | — | 81.0 ms | 146.0 ms (4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.5133 · calibrated 0.1119 · conformal-naive floor 0.1480 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK**

**corpus cap:** 48 (registry)

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4500 |
| 1 | observed-laplace | — | bag | 0.5850 |
| 4 ← selected | observed-laplace | — | bag | 0.6050 |
| 16 | observed-laplace | — | bag | 0.5950 |
| 32 | observed-laplace | — | bag | 0.5950 |
| 64 | observed-laplace | — | bag | 0.5950 |
| 1 | fixed-1 | — | bag | 0.5500 |
| 4 | fixed-1 | — | bag | 0.5850 |
| 16 | fixed-1 | — | bag | 0.5650 |
| 32 | fixed-1 | — | bag | 0.5650 |
| 64 | fixed-1 | — | bag | 0.5650 |

**transductive column (NOT the headline):** acc 0.7833 vs honest 0.7800 (+0.3 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.7800 | 0.7698 | 0.6352 | 0.7856 | 2.0599 | 0.0553 | 0.9667 | 0.0796 | 0.94/0.31 | 0.7670 | 0.138 ms | 0.274 ms (4) | ✓ | s 0.2008 / d 0.5259 (n 200) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 300 | 0.8667 | 0.8559 | 0.0495 | 0.1880 | 0.3979 | 0.0204 | 1.0000 | 0.0495 | — / — | — | 132.0 ms | 164.0 ms (4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.6352 · calibrated 0.0796 · conformal-naive floor 0.1209 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK**

**corpus cap:** 40 (registry)

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4650 |
| 1 ← selected | observed-laplace | — | bag | 0.8100 |
| 4 | observed-laplace | — | bag | 0.8050 |
| 16 | observed-laplace | — | bag | 0.8050 |
| 32 | observed-laplace | — | bag | 0.8050 |
| 64 | observed-laplace | — | bag | 0.8000 |
| 1 | fixed-1 | — | bag | 0.7950 |
| 4 | fixed-1 | — | bag | 0.7750 |
| 16 | fixed-1 | — | bag | 0.7700 |
| 32 | fixed-1 | — | bag | 0.7600 |
| 64 | fixed-1 | — | bag | 0.7600 |

**transductive column (NOT the headline):** acc 0.8360 vs honest 0.8420 (-0.6 pt; 500 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.8420 | 0.8362 | 0.8194 | 0.9688 | 3.8215 | 0.0599 | 0.9720 | 0.0517 | 1.00/0.48 | 0.9349 | 0.462 ms | 0.978 ms (6) | ✓ | s 0.7478 / d 0.5549 (n 200) |
| bekko · hotchpotch/bekko-system-one-v0-68m | 500 | 0.7380 | 0.7250 | 0.0611 | 0.3604 | 0.9622 | 0.0878 | 0.9480 | 0.0611 | — / — | — | 251.0 ms | 424.0 ms (6) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.8194 · calibrated 0.0517 · conformal-naive floor 0.2933 → **PASS** (beats both the uncalibrated output AND the floor)

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

