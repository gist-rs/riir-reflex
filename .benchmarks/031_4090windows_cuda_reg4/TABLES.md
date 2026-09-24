# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `3668b6b` on `4090-windows` (2026-09-24T18:54:23Z) · profile release · laya feature true
- laya posture: PRE-MOVE BASELINE — `src/laya` is scheduled to move to the riir-infer repo (008 T4); the laya columns pin the pre-move tree at the sha above
- box state (Issue 021): start power=? mode=? load=? swap=?M · end power=? mode=? load=? swap=?M — ⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state
- laya device posture: cuda (LAYA_DEVICE or the build's non-macOS CUDA default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- corpus cap posture: registry defaults
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

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3190 | 0.2270 | 0.0527 | 0.6924 | 1.2331 | 0.5525 | 0.4400 | 0.0805 | 0.82/0.93 | 0.3493 | 0.546 ms | 2.861 ms (5) | ✓ | s 0.0139 / d 0.9213 (n 100) |
| laya-riir · english | 2000 | 0.3570 | 0.3255 | 0.2117 | 0.7685 | 1.3595 | 0.5627 | 0.4290 | 0.1914 | — / — | — | 91.0 ms | 139.0 ms (5) | ✗ | — |
| laya-riir · multilingual | 2000 | 0.3465 | 0.3473 | 0.3258 | 0.9078 | 1.9509 | 0.5557 | 0.4030 | 0.2555 | — / — | — | 52.0 ms | 82.0 ms (5) | ✓ | — |
| laya-riir · typed | 2000 | 0.7400 | 0.7307 | 0.1880 | 0.4165 | 0.7344 | 0.1418 | 0.8510 | 0.3897 | — / — | — | 92.0 ms | 168.0 ms (5) | ✗ | — |

**G1 (modelless readout ECE):** raw 0.2933 · calibrated 0.0805 · conformal-naive floor 0.3087 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3166 · brier_soft 0.2459 · score MAE 0.7394 · within_1 0.7037

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1867 | 0.0828 | 0.2695 |
| modelless | noul | 600 | 0.5317 | 0.0250 | 0.5067 |
| modelless | score | 800 | 0.2587 | 0.0540 | 0.3096 |
| laya-riir·english | choice | 600 | 0.2883 | 0.1681 | 0.4527 |
| laya-riir·english | noul | 600 | 0.4733 | 0.2808 | 0.7542 |
| laya-riir·english | score | 800 | 0.3212 | 0.2068 | 0.5165 |
| laya-riir·multilingual | choice | 600 | 0.2950 | 0.3623 | 0.6559 |
| laya-riir·multilingual | noul | 600 | 0.4867 | 0.4084 | 0.8919 |
| laya-riir·multilingual | score | 800 | 0.2800 | 0.2388 | 0.5123 |
| laya-riir·typed | choice | 600 | 0.7333 | 0.2548 | 0.4785 |
| laya-riir·typed | noul | 600 | 0.7850 | 0.1294 | 0.6624 |
| laya-riir·typed | score | 800 | 0.7113 | 0.1890 | 0.5265 |
**typed-decisions extras (laya-riir·english):** soft_acc 0.3346 · brier_soft 0.3348 · score MAE 0.6937 · within_1 0.7525
**typed-decisions extras (laya-riir·multilingual):** soft_acc 0.3265 · brier_soft 0.4757 · score MAE 0.7681 · within_1 0.6937
**typed-decisions extras (laya-riir·typed):** soft_acc 0.4651 · brier_soft 0.0720 · score MAE 0.2566 · within_1 0.9888

## ag_news — 400 cases / 400 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5100 | 0.4725 | 0.2493 | 0.7419 | 1.3702 | 0.4899 | 0.5350 | 0.3797 | 0.49/0.26 | 0.5436 | 0.164 ms | 0.237 ms (5) | ✓ | s 0.0001 / d 0.3709 (n 200) |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.0316 | 0.0808 | 0.1637 | 0.0067 | 1.0000 | 0.1412 | — / — | — | 12.0 ms | 16.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.3797 · conformal-naive floor 0.2506 → **FAIL** (does not beat both)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2825 | 0.1511 | 0.1128 | 0.8319 | 1.7874 | 0.7236 | 0.2850 | 0.0226 | 0.62/0.36 | 0.2773 | 0.101 ms | 0.125 ms (5) | ✓ | s 0.0000 / d 0.4153 (n 200) |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.3088 | 0.6963 | 2.1840 | 0.2657 | 0.7150 | 0.2571 | — / — | — | 10.0 ms | 12.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2824 · calibrated 0.0226 · conformal-naive floor 0.2931 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2167 | 0.2044 | 0.0065 | 0.7988 | 1.6063 | 0.7849 | 0.2067 | 0.0165 | 0.57/0.32 | 0.2068 | 0.085 ms | 0.118 ms (7) | ✓ | s 0.0002 / d 0.3973 (n 200) |
| laya-riir · english | 600 | 0.3717 | 0.3292 | 0.2478 | 0.8172 | 1.7291 | 0.5279 | 0.4567 | 0.0811 | — / — | — | 11.0 ms | 12.0 ms (7) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2159 · calibrated 0.0165 · conformal-naive floor 0.3300 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1774 · within_1 0.4450

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4397 | 0.4396 | 0.1070 | 0.5154 | 0.7086 | 0.5713 | 0.3966 | 0.1466 | 0.56/0.88 | 0.2143 | 0.048 ms | 0.058 ms (2) | ✓ | s 0.0004 / d 0.4079 (n 100) |
| laya-riir · english | 116 | 0.6983 | 0.6751 | 0.2620 | 0.5259 | 3.1497 | 0.1073 | 0.9655 | 0.2620 | — / — | — | 11.0 ms | 17.0 ms (2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4333 · calibrated 0.1466 · conformal-naive floor 0.3757 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3467 | 0.2151 | 0.0012 | 0.6674 | 1.0997 | 0.6898 | 0.3533 | 0.0965 | 0.64/0.38 | 0.3459 | 0.074 ms | 0.113 ms (4) | ✓ | s 0.0001 / d 0.5124 (n 200) |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.0685 | 0.2204 | 0.3987 | 0.0381 | 0.9800 | 0.1044 | — / — | — | 12.0 ms | 14.0 ms (4) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3461 · calibrated 0.0965 · conformal-naive floor 0.3174 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.6900 | 0.6908 | 0.6227 | 0.9196 | 2.7260 | 0.1560 | 0.8667 | 0.0750 | 0.41/0.35 | 0.7268 | 0.072 ms | 0.101 ms (4) | ✓ | s 0.0621 / d 0.6319 (n 200) |
| laya-riir · english | 300 | 0.7500 | 0.7512 | 0.2047 | 0.4324 | 2.9079 | 0.1022 | 0.9333 | 0.2076 | — / — | — | 15.0 ms | 17.0 ms (4) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.6227 · calibrated 0.0750 · conformal-naive floor 0.0854 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.4460 | 0.1515 | 0.4275 | 0.9778 | 4.0279 | 0.4253 | 0.5680 | 0.1810 | 1.00/0.23 | 0.5325 | 0.279 ms | 0.604 ms (6) | ✓ | s 0.0360 / d 0.4730 (n 200) |
| laya-riir · english | 500 | 0.4980 | 0.1519 | 0.3158 | 0.7996 | 5.3751 | 0.3141 | 0.6880 | 0.3314 | — / — | — | 24.0 ms | 25.0 ms (6) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4275 · calibrated 0.1810 · conformal-naive floor 0.4410 → **PASS** (beats both the uncalibrated output AND the floor)

## code_fixtures — 12 cases / 24 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 24 | 0.2500 | 0.0930 | 0.0806 | 0.6852 | 1.3719 | 0.6986 | 0.4167 | 0.2777 | 0.33/0.33 | 0.3750 | 0.153 ms | 0.238 ms (1, max@case8) | ✓ | s 0.0001 / d 0.2463 (n 36) |
| laya-riir · english | 24 | 0.5833 | 0.4874 | 0.2440 | 0.5718 | 1.3758 | 0.3161 | 0.7500 | 0.2175 | — / — | — | 26.0 ms | 60.0 ms (1, max@case3) | ✗ | — |

**G1 (modelless readout ECE):** raw 0.2464 · calibrated 0.2777 · conformal-naive floor 0.4863 → **FAIL** (does not beat both)

## harness_visibility — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.008 ms | 0.009 ms (1, max@case0) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| laya-riir · english | 16 | 0.3125 | 0.2738 | 0.3731 | 0.7005 | 1.2409 | 0.4633 | 0.6250 | 0.2218 | — / — | — | 11.0 ms | 12.0 ms (1, max@case1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_permissions — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.006 ms | 0.007 ms (1, max@case0) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| laya-riir · english | 12 | 0.4167 | 0.3111 | 0.4358 | 0.8133 | 1.2925 | 0.8356 | 0.1667 | 0.4770 | — / — | — | 12.0 ms | 12.0 ms (1, max@case2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_tool_fit — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.008 ms | 0.009 ms (1, max@case1) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| laya-riir · english | 12 | 0.5000 | 0.4111 | 0.4619 | 0.7146 | 1.7865 | 0.4104 | 0.5000 | 0.3852 | — / — | — | 10.0 ms | 11.0 ms (1, max@case3) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_routing — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.008 ms | 0.010 ms (1, max@case0) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| laya-riir · english | 16 | 0.6250 | 0.5970 | 0.2251 | 0.5411 | 1.0299 | 0.1734 | 0.7500 | 0.4866 | — / — | — | 12.0 ms | 13.0 ms (1, max@case1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_sensitivity — 15 cases / 15 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.007 ms | 0.009 ms (1, max@case0) | ✓ | s 0.0111 / d 0.4172 (n 20) |
| laya-riir · english | 15 | 0.6000 | 0.6000 | 0.3761 | 0.5874 | 1.0559 | 0.4451 | 0.7143 | 0.5525 | — / — | — | 12.0 ms | 13.0 ms (1, max@case0) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3816 · calibrated 0.3816 · conformal-naive floor 0.3429 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_cache_reuse — 12 cases / 12 questions

> modelless lane: SKIPPED — LLM-lane only (the modelless lane has no KV cache, so it has no honest answer for this family); the laya lane answered below.

| lane · model | n | acc | ECE(maxp) | readout-ECE | p50 | p99 (support) | det |
|---|---|---|---|---|---|---|---|
| laya-riir · english | 12 | 0.5000 | 0.3618 | 0.3618 | 12.0 ms | 13.0 ms (1, max@case2) | ✓ |

## Landscape — published specialist rows (NOT measured by this harness)

External "System One" typed-decision models on the same 400-case /
2000-question Typed Decisions official test split, quoted AS PUBLISHED
(issue 025; pin `malevrigns/agent-jev` @ `a965ca8f`, Apache-2.0). Their
protocol differs from ours — the footnotes are part of the row; no number
here is comparable without them.

| lane · model | source | acc | bool·noul / choice / score | p50 case |
|---|---|---|---|---|
| AgentJev-0.6B (598M, Qwen3-0.6B backbone) | published (their run) | **0.7925** | 88.83 / 75.33 / 75.00 | ~60–70 ms, their cuda box |
| Laya (published checkpoint, 421M ModernBERT) | their table — card-copied, not re-scored | 0.7700 | — | 41.53 ms (their box) |
| TypeSafe Jev 1.13.0 | their table — zero-shot generalist | 0.727 | — | — |
| reflex · laya-riir·typed | MEASURED — the typed_decisions table above | 0.7445 (baseline `aa37823`) | 78.50 / 73.33 / 72.25 | 1312 ms (m3 metal, that baseline) |
| reflex · modelless | MEASURED — the typed_decisions table above | 0.3190 (baseline `aa37823`) | 53.17 / 18.67 / 25.87 | 0.472 ms |

Footnotes: (1) their accuracy is agreement with the public TEACHER argmax;
ours is gold-label under the standard harness protocol — different
references of truth. (2) their run held out 120 dev + 120 cal cases and
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

