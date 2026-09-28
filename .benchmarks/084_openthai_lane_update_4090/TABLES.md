# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `bee92d3` on `shikuwa` (2026-09-28T13:59:51Z) · profile release · laya feature false
- box state (Issue 021): start power=? mode=? load=? swap=?M · end power=? mode=? load=? swap=?M — ⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
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
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; all six modelless at default features since Issue 045 (cache_reuse's LLM-only carve-out REVERSED — Bench 072: modelless 0.9167 vs the frozen LLM-lane 0.5000)

## typed_decisions — 400 cases / 2000 questions

**corpus cap:** 48 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3523 · max_prob 0.0598 · inv_entropy 0.3523

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3345 | 0.2456 | 0.0488 | 0.6895 | 1.2301 | 0.5429 | 0.4690 | 0.0675 | 0.80/0.93 | 0.3469 | 0.942 ms | 2.632 ms (5) | ✓ | s 0.0071 / d 0.9122 (n 100) |
| openthai · openthai-systemone | 2000 | 0.5360 | 0.5392 | 0.2275 | 0.6465 | 1.1769 | 0.3245 | 0.6340 | 0.1566 | — / — | — | 79.0 ms | 187.0 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3293 · calibrated 0.0675 · conformal-naive floor 0.3578 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3169 · brier_soft 0.2430 · score MAE 0.7046 · within_1 0.7275

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1717 | 0.0927 | 0.2644 |
| modelless | noul | 600 | 0.5650 | 0.0568 | 0.5082 |
| modelless | score | 800 | 0.2838 | 0.0442 | 0.2935 |

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4845 · max_prob 0.2262 · inv_entropy 0.4845

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.4050 | 0.3750 | 0.1465 | 0.7436 | 1.3735 | 0.5223 | 0.4750 | 0.0800 | 0.67/0.39 | 0.3951 | 0.203 ms | 0.493 ms (5) | ✓ | s 0.0001 / d 0.4105 (n 200) |
| openthai · openthai-systemone | 400 | 0.8900 | 0.8826 | 0.0474 | 0.1550 | 0.2814 | 0.0202 | 0.9850 | 0.0679 | — / — | — | 56.0 ms | 174.0 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4046 · calibrated 0.0800 · conformal-naive floor 0.2282 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1449 · max_prob 0.0245 · inv_entropy 0.1449

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2750 | 0.1345 | 0.1053 | 0.8323 | 1.7886 | 0.7259 | 0.3000 | 0.1297 | 0.58/0.33 | 0.2910 | 0.100 ms | 0.136 ms (5) | ✓ | s 0.0000 / d 0.4265 (n 200) |
| openthai · openthai-systemone | 400 | 0.5950 | 0.4877 | 0.2124 | 0.6034 | 1.3951 | 0.2206 | 0.7750 | 0.1138 | — / — | — | 50.0 ms | 148.0 ms (5) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2749 · calibrated 0.1297 · conformal-naive floor 0.2982 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2442 · max_prob 0.0340 · inv_entropy 0.2442

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2017 | 0.1890 | 0.0087 | 0.7986 | 1.6059 | 0.8159 | 0.1833 | 0.0434 | 0.58/0.32 | 0.2044 | 0.087 ms | 0.138 ms (7) | ✓ | s 0.0002 / d 0.4166 (n 200) |
| openthai · openthai-systemone | 600 | 0.4333 | 0.4144 | 0.1738 | 0.6930 | 1.3199 | 0.4602 | 0.5233 | 0.0825 | — / — | — | 59.0 ms | 158.0 ms (7) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2009 · calibrated 0.0434 · conformal-naive floor 0.3343 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1781 · within_1 0.4433

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4999 · max_prob 0.0055 · inv_entropy 0.4999

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5005 | 0.6937 | 0.3980 | 0.5690 | 0.0172 | 0.39/0.39 | 0.4507 | 0.065 ms | 0.100 ms (2) | ✓ | s 0.0001 / d 0.4619 (n 100) |
| openthai · openthai-systemone | 116 | 0.6293 | 0.5821 | 0.2118 | 0.4788 | 0.6803 | 0.1592 | 0.8621 | 0.2118 | — / — | — | 50.0 ms | 118.0 ms (2) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4827 · calibrated 0.0172 · conformal-naive floor 0.2491 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3196 · max_prob 0.0238 · inv_entropy 0.3196

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3400 | 0.2052 | 0.0055 | 0.6676 | 1.1000 | 0.6937 | 0.3467 | 0.0199 | 0.56/0.32 | 0.3448 | 0.083 ms | 0.134 ms (4) | ✓ | s 0.0001 / d 0.5110 (n 200) |
| openthai · openthai-systemone | 300 | 0.9000 | 0.9000 | 0.0453 | 0.1660 | 0.3077 | 0.0233 | 0.9867 | 0.2504 | — / — | — | 52.0 ms | 143.0 ms (4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3395 · calibrated 0.0199 · conformal-naive floor 0.3137 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

⛔ **corpus fallback (Issue 039 guard):** 29 option label(s) with NO train docs in the corpus pool — self-doc fallback only: email_querycontact, play_radio, social_post, email_addcontact, transport_taxi, lists_remove, qa_definition, cooking_recipe, email_query, calendar_set, recommendation_locations, social_query, lists_createoradd, recommendation_movies, qa_currency, play_audiobook, play_podcasts, qa_maths, calendar_remove, transport_query, qa_stock, play_game, transport_traffic, email_sendemail, recommendation_events, calendar_query, qa_factoid, transport_ticket, lists_query

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3533 | 0.3495 | 0.2873 | 0.9367 | 2.8786 | 0.4430 | 0.5133 | 0.6033 | 0.50/0.39 | 0.3825 | 0.072 ms | 0.094 ms (4) | ✓ | s 0.0633 / d 0.6283 (n 200) |
| openthai · openthai-systemone | 300 | 0.9200 | 0.9142 | 0.0444 | 0.1070 | 0.2889 | 0.0090 | 1.0000 | 0.0383 | — / — | — | 95.0 ms | 213.0 ms (4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2873 · calibrated 0.6033 · conformal-naive floor 0.2015 → **FAIL** (does not beat both)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

⛔ **corpus fallback (Issue 039 guard):** 45 option label(s) with NO train docs in the corpus pool — self-doc fallback only: Refund not showing up, activate my card, apple pay or google pay, atm support, balance not updated after bank transfer, beneficiary not allowed, card about to expire, card swallowed, cash withdrawal charge, cash withdrawal not recognised, change pin, compromised card, country support, declined card payment, declined cash withdrawal, declined transfer, direct debit payment not recognised, disposable card limits, exchange charge, failed transfer, get disposable virtual card, getting spare card, lost or stolen phone, order physical card, passcode forgotten, pending card payment, pending transfer, receiving money, request refund, reverted card payment?, terminate account, top up by card charge, top up by cash or cheque, top up failed, topping up by card, transaction charged twice, transfer fee charged, transfer into account, transfer timing, verify my identity, verify source of funds, verify top up, virtual card not working, visa or mastercard, wrong exchange rate for cash withdrawal

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.2600 | 0.2190 | 0.2416 | 0.9808 | 4.1290 | 0.6315 | 0.3560 | 0.2448 | 1.00/0.35 | 0.3354 | 0.276 ms | 0.520 ms (6) | ✓ | s 0.0361 / d 0.4928 (n 200) |
| openthai · openthai-systemone | 500 | 0.6540 | 0.6331 | 0.0895 | 0.5155 | 1.9273 | 0.1682 | 0.8840 | 0.1141 | — / — | — | 178.0 ms | 306.0 ms (6) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2416 · calibrated 0.2448 · conformal-naive floor 0.2550 → **FAIL** (does not beat both)

## code_fixtures — 16 cases / 32 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4602 · max_prob 0.1479 · inv_entropy 0.4602

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 32 | 0.3750 | 0.1640 | 0.0948 | 0.6858 | 1.3870 | 0.4073 | 0.6250 | 0.0938 | 0.34/0.34 | 0.2857 | 0.242 ms | 0.459 ms (1, max@case15) | ✓ | s 0.0001 / d 0.3365 (n 64) |
| openthai · openthai-systemone | 32 | 0.5938 | 0.3146 | 0.1933 | 0.5772 | 1.4916 | 0.1576 | 0.8750 | 0.1632 | — / — | — | 86.0 ms | 125.0 ms (1, max@case13) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3742 · calibrated 0.0938 · conformal-naive floor 0.3510 → **PASS** (beats both the uncalibrated output AND the floor)

## harness_visibility — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.008 ms | 0.010 ms (1, max@case0) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| openthai · openthai-systemone | 16 | 0.4375 | 0.4249 | 0.2194 | 0.7038 | 1.3007 | 0.5024 | 0.5000 | 0.3182 | — / — | — | 62.0 ms | 95.0 ms (1, max@case4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_permissions — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.006 ms | 0.007 ms (1, max@case0) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| openthai · openthai-systemone | 12 | 0.5833 | 0.5524 | 0.3147 | 0.6818 | 1.2055 | 0.5037 | 0.6667 | 0.5352 | — / — | — | 55.0 ms | 73.0 ms (1, max@case3) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_tool_fit — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.007 ms | 0.008 ms (1, max@case1) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| openthai · openthai-systemone | 12 | 0.9167 | 0.9111 | 0.1963 | 0.1988 | 0.4471 | 0.0229 | 1.0000 | 0.2813 | — / — | — | 54.0 ms | 65.0 ms (1, max@case10) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_routing — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.008 ms | 0.009 ms (1, max@case0) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| openthai · openthai-systemone | 16 | 0.4375 | 0.3843 | 0.4883 | 0.8866 | 1.5076 | 0.3854 | 0.5000 | 0.3445 | — / — | — | 57.0 ms | 116.0 ms (1, max@case4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_sensitivity — 15 cases / 15 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.007 ms | 0.009 ms (1, max@case1) | ✓ | s 0.0111 / d 0.4172 (n 20) |
| openthai · openthai-systemone | 15 | 0.7333 | 0.6548 | 0.1926 | 0.3895 | 0.7253 | 0.1101 | 1.0000 | 0.3224 | — / — | — | 52.0 ms | 57.0 ms (1, max@case4) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3816 · calibrated 0.3816 · conformal-naive floor 0.3429 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_cache_reuse — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.3333 | 0.0054 | 0.5000 | 0.6932 | 0.5410 | 0.5000 | 0.4999 | 0.67/0.67 | 0.2500 | 0.008 ms | 0.011 ms (1, max@case6) | ✓ | s 0.0001 / d 0.9649 (n 20) |
| openthai · openthai-systemone | 12 | 0.5000 | 0.3333 | 0.4058 | 0.6710 | 0.8804 | 0.7960 | 0.1667 | 0.4058 | — / — | — | 53.0 ms | 58.0 ms (1, max@case2) | ✓ | — |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4999 · calibrated 0.4999 · conformal-naive floor 0.2778 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

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

