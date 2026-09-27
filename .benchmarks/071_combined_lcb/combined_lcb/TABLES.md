# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `eedfe24` on `m3` (2026-09-27T13:40:46Z) · profile release · laya feature true
- laya posture: PRE-MOVE BASELINE — `src/laya` is scheduled to move to the riir-infer repo (008 T4); the laya columns pin the pre-move tree at the sha above
- box state (Issue 021): start power=AC Power mode=high load=6.73 swap=1882M · end power=AC Power mode=high load=13.65 swap=1882M — ⛔ latency NOT QUOTABLE — load 6.73 > 6 — a sibling job is on the box; load 13.65 > 6 — a sibling job is on the box
- laya device posture: metal (LAYA_DEVICE or the build's macOS default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16/32/64 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
- option-conditioned tables (issue 038 T7b): ON — cal-selected per suite (scale 0/0.25/0.5/1/2/4/8, promotion bar +5 pt over off on the stratified slice; one contrastive table per (question id, gold option) from the TRAIN rows, events filtered to the corpus pool; every question kind armed — typed_decisions is the target suite)
- NBSVM ridge readout (issue 038 T7a): ON — cal-selected per suite (scale 0/0.5/1/2/4/8 × λ 10 fixed, promotion bar +5 pt over off on the stratified slice; NBSVM closed-form ridge, k=2048, per-class NB log-count ratios, fit-time self-calibrated margin temperature)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved; --gate-fit-selection: thresholds fitted on the stratified selection slice, probe corpus excluding the fit docs (issue 042 lever 1); --gate-distance-only: score axis disabled (threshold 0.0), the fused gate runs on the corpus-distance axis alone (issue 042 lever 2)
- sampling: test sample = label-STRATIFIED round-robin over the whole test split (budget = the registry test cap; first-appearance label order, dataset order within each label, deterministic, no RNG); cal slice = the same law over the train rows; corpus pool = the train rows MINUS the cal front (excluded by construction, not position); budget-0 suites unchanged (identity split) — Issue 039 T2
- readout: shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side arming lever overfit the narrow suites' cal slice — emotion's test G1 regressed — and coincided with Dispatch on the wide suites it targeted; the wide-label G1 gap was closed by T2's stratified cal slice). The candidate table is recorded per suite (report-only); EngineConfig::readout stays the opt-in knob
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; harness_cache_reuse is LLM-lane only (the modelless lane has no KV cache) and reports a loud SKIPPED absence without the laya-riir feature

## typed_decisions — 400 cases / 2000 questions

**corpus cap:** 48 (registry)

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.3700 |
| 1 | observed-laplace | — | bag | 0.3700 |
| 4 | observed-laplace | — | bag | 0.3700 |
| 16 | observed-laplace | — | bag | 0.3700 |
| 32 | observed-laplace | — | bag | 0.3700 |
| 64 | observed-laplace | — | bag | 0.3700 |
| 1 | fixed-1 | — | bag | 0.3700 |
| 4 | fixed-1 | — | bag | 0.3700 |
| 16 | fixed-1 | — | bag | 0.3700 |
| 32 | fixed-1 | — | bag | 0.3700 |
| 64 | fixed-1 | — | bag | 0.3700 |

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4708 · max_prob 0.1033 · inv_entropy 0.4708

⛔ **corpus fallback (Issue 039 guard):** 1 option label(s) with NO train docs in the corpus pool — self-doc fallback only: security_incidents

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.4655 | 0.4455 | 0.0750 | 0.6190 | 1.0998 | 0.3865 | 0.5680 | 0.0055 | 0.50/0.50 | 0.5283 | 0.565 ms | 1.727 ms (5) | ✓ | s 0.0261 / d 0.9316 (n 100) |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2113 | 0.7684 | 1.3593 | 0.5627 | 0.4290 | 0.1918 | — / — | — | 214.0 ms | 312.0 ms (5) | ✓ | — |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.3228 | 0.9057 | 1.9437 | 0.5548 | 0.4040 | 0.2567 | — / — | — | 80.0 ms | 139.0 ms (5) | ✓ | — |
| laya-riir · typed | 2000 | 0.7445 | 0.7349 | 0.1924 | 0.4099 | 0.7185 | 0.1263 | 0.8590 | 0.3953 | — / — | — | 193.0 ms | 350.0 ms (5) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 2000 | 0.0% | 0.4655 | 0.4655 | +0.0000 | — | — | 993 (disarmed by worthiness) |
| cascade · multilingual | 2000 | 0.0% | 0.4655 | 0.4655 | +0.0000 | — | — | 993 (disarmed by worthiness) |
| cascade · typed | 2000 | 49.6% | 0.6340 | 0.4655 | +0.1685 | 736 / 993 (0.7412) | 399 / 993 (0.4018) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 160 | 0.3438 | 0.5625 | -0.2188 | -0.3081 | +0.1600 | — |
| cascade · multilingual | DISARMED | 160 | 0.3063 | 0.5625 | -0.2562 | -0.3443 | +0.1600 | — |
| cascade · typed | armed | 160 | 0.8375 | 0.5625 | +0.2750 | +0.1946 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.4238 · calibrated 0.0055 · conformal-naive floor 0.2072 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3545 · brier_soft 0.1912 · score MAE 0.6710 · within_1 0.7462

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.4383 | 0.1013 | 0.3371 |
| modelless | noul | 600 | 0.6200 | 0.0930 | 0.5923 |
| modelless | score | 800 | 0.3700 | 0.0496 | 0.3204 |
| laya-riir·english | choice | 600 | 0.2883 | 0.1681 | 0.4527 |
| laya-riir·english | noul | 600 | 0.4733 | 0.2808 | 0.7542 |
| laya-riir·english | score | 800 | 0.3225 | 0.2065 | 0.5168 |
| laya-riir·multilingual | choice | 600 | 0.2950 | 0.3623 | 0.6559 |
| laya-riir·multilingual | noul | 600 | 0.4867 | 0.4084 | 0.8919 |
| laya-riir·multilingual | score | 800 | 0.2863 | 0.2313 | 0.5110 |
| laya-riir·typed | choice | 600 | 0.7333 | 0.2548 | 0.4785 |
| laya-riir·typed | noul | 600 | 0.7850 | 0.1294 | 0.6624 |
| laya-riir·typed | score | 800 | 0.7225 | 0.1987 | 0.5246 |
**typed-decisions extras (laya-riir·english):** soft_acc 0.3346 · brier_soft 0.3348 · score MAE 0.6937 · within_1 0.7525
**typed-decisions extras (laya-riir·multilingual):** soft_acc 0.3271 · brier_soft 0.4737 · score MAE 0.7604 · within_1 0.7000
**typed-decisions extras (laya-riir·typed):** soft_acc 0.4668 · brier_soft 0.0677 · score MAE 0.2424 · within_1 0.9950

## ag_news — 400 cases / 400 questions

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
| modelless | 400 | 0.8825 | 0.8742 | 0.4783 | 0.5053 | 0.9630 | 0.0469 | 0.9400 | 0.0025 | 0.38/0.38 | 0.9040 | 0.161 ms | 0.283 ms (5) | ✓ | s 0.0089 / d 0.4033 (n 200) |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.0316 | 0.0808 | 0.1637 | 0.0067 | 1.0000 | 0.1412 | — / — | — | 17.0 ms | 27.0 ms (5) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 400 | 37.5% | 0.9125 | 0.8825 | +0.0300 | 139 / 150 (0.9267) | 127 / 150 (0.8467) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 53 | 0.9811 | 0.7736 | +0.2075 | +0.1081 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.8305 · calibrated 0.0025 · conformal-naive floor 0.2482 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

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
| modelless | 400 | 0.8850 | 0.8014 | 0.6185 | 0.6597 | 1.3652 | 0.0360 | 0.9750 | 0.0000 | 0.32/0.32 | 0.8828 | 0.126 ms | 0.145 ms (5) | ✓ | s 0.0084 / d 0.4201 (n 200) |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.3088 | 0.6963 | 2.1840 | 0.2657 | 0.7150 | 0.2571 | — / — | — | 12.0 ms | 15.0 ms (5) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 400 | 0.0% | 0.8850 | 0.8850 | +0.0000 | — | — | 127 (disarmed by worthiness) |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 57 | 0.5263 | 0.8596 | -0.3333 | -0.4659 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.8622 · calibrated 0.0000 · conformal-naive floor 0.2780 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

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
| modelless | 600 | 0.3967 | 0.3211 | 0.1393 | 0.7604 | 1.5156 | 0.5361 | 0.4700 | 0.0050 | 0.40/0.40 | 0.3760 | 0.099 ms | 0.104 ms (7) | ✓ | s 0.0050 / d 0.4464 (n 200) |
| laya-riir · english | 600 | 0.3717 | 0.3292 | 0.2478 | 0.8172 | 1.7291 | 0.5279 | 0.4567 | 0.0811 | — / — | — | 14.0 ms | 18.0 ms (7) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 600 | 0.0% | 0.3967 | 0.3967 | +0.0000 | — | — | 241 (disarmed by worthiness) |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 82 | 0.3902 | 0.2561 | +0.1341 | +0.0152 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.3849 · calibrated 0.0050 · conformal-naive floor 0.2220 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1170 · within_1 0.5400

## prompt_injections — 116 cases / 116 questions

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
| modelless | 116 | 0.7672 | 0.7643 | 0.1336 | 0.3365 | 0.5194 | 0.0949 | 0.8966 | 0.0348 | 0.34/0.34 | 0.7922 | 0.089 ms | 0.121 ms (2) | ✓ | s 0.0258 / d 0.4409 (n 100) |
| laya-riir · english | 116 | 0.6983 | 0.6751 | 0.2620 | 0.5259 | 3.1497 | 0.1073 | 0.9655 | 0.2620 | — / — | — | 16.0 ms | 33.0 ms (2) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 116 | 0.0% | 0.7672 | 0.7672 | +0.0000 | — | — | 39 (disarmed by worthiness) |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 23 | 0.5652 | 0.8261 | -0.2609 | -0.4749 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.6745 · calibrated 0.0348 · conformal-naive floor 0.3622 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

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
| modelless | 300 | 0.5233 | 0.5061 | 0.1383 | 0.6178 | 1.0287 | 0.3120 | 0.6400 | 0.0716 | 0.16/0.16 | 0.5178 | 0.110 ms | 0.134 ms (4) | ✓ | s 0.0016 / d 0.4292 (n 200) |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.0685 | 0.2204 | 0.3987 | 0.0381 | 0.9800 | 0.1044 | — / — | — | 21.0 ms | 33.0 ms (4) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 300 | 15.7% | 0.5833 | 0.5233 | +0.0600 | 44 / 47 (0.9362) | 26 / 47 (0.5532) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 37 | 0.9459 | 0.5946 | +0.3514 | +0.2052 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.5133 · calibrated 0.0716 · conformal-naive floor 0.1480 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

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
| modelless | 300 | 0.7800 | 0.7698 | 0.6352 | 0.7856 | 2.0599 | 0.0553 | 0.9667 | 0.0796 | 0.25/0.25 | 0.7679 | 0.129 ms | 0.199 ms (4) | ✓ | s 0.0619 / d 0.5126 (n 200) |
| laya-riir · english | 300 | 0.6933 | 0.6936 | 0.2483 | 0.5405 | 3.5500 | 0.1444 | 0.9133 | 0.2441 | — / — | — | 31.0 ms | 38.0 ms (4) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 300 | 0.0% | 0.7800 | 0.7800 | +0.0000 | — | — | 76 (disarmed by worthiness) |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 47 | 0.7447 | 0.5957 | +0.1489 | -0.0086 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.6352 · calibrated 0.0796 · conformal-naive floor 0.1209 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

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
| modelless | 500 | 0.8420 | 0.8362 | 0.8194 | 0.9688 | 3.8215 | 0.0599 | 0.9720 | 0.1580 | 0.35/0.35 | 0.8765 | 0.469 ms | 14.919 ms (6) | ✓ | s 0.0205 / d 0.5768 (n 200) |
| laya-riir · english | 500 | 0.4220 | 0.3765 | 0.3813 | 0.9327 | 6.1219 | 0.4066 | 0.6000 | 0.3940 | — / — | — | 48.0 ms | 58.0 ms (6) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 500 | 0.0% | 0.8420 | 0.8420 | +0.0000 | — | — | 176 (disarmed by worthiness) |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 70 | 0.4143 | 0.7714 | -0.3571 | -0.4844 | +0.1600 | — |


**G1 (modelless readout ECE):** raw 0.8194 · calibrated 0.1580 · conformal-naive floor 0.2933 → **PASS** (beats both the uncalibrated output AND the floor)

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

