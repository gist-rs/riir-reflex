# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `777f6b6` on `m3` (2026-09-23T16:00:13Z) · profile release · laya feature true
- laya posture: PRE-MOVE BASELINE — `src/laya` is scheduled to move to the riir-infer repo (008 T4); the laya columns pin the pre-move tree at the sha above
- laya device posture: metal (LAYA_DEVICE or the build's macOS default)
- laya-python lane: on — the ORIGINAL torch reference as a JSONL subprocess oracle: same cases, the reference's own rounded-4 probabilities, latency = subprocess round-trip (IPC included)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; harness_cache_reuse is LLM-lane only (the modelless lane has no KV cache) and reports a loud SKIPPED absence without the laya-riir feature

## typed_decisions — 400 cases / 2000 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3190 | 0.2270 | 0.0527 | 0.6924 | 1.2331 | 0.5525 | 0.4400 | 0.0805 | 0.82/0.93 | 0.3493 | 0.512 ms | 1.819 ms (5) | ✓ | s 0.0139 / d 0.9213 (n 100) |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2113 | 0.7684 | 1.3593 | 0.5627 | 0.4290 | 0.1918 | — / — | — | 1440.0 ms | 2416.0 ms (5) | ✓ | — |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.3228 | 0.9057 | 1.9437 | 0.5548 | 0.4040 | 0.2567 | — / — | — | 682.0 ms | 1181.0 ms (5) | ✓ | — |
| laya-python · english | 2000 | 0.3575 | 0.3262 | 0.2113 | 0.7684 | 1.3593 | 0.5627 | 0.4290 | 0.1918 | — / — | — | 350.0 ms | 581.0 ms (5) | ✓ | — |
| laya-python · multilingual | 2000 | 0.3490 | 0.3499 | 0.3228 | 0.9057 | 1.9437 | 0.5548 | 0.4040 | 0.2567 | — / — | — | 156.0 ms | 335.0 ms (5) | ✓ | — |
| laya-python · typed | 2000 | 0.7445 | 0.7349 | 0.1924 | 0.4099 | 0.7185 | 0.1263 | 0.8590 | 0.3953 | — / — | — | 352.0 ms | 703.0 ms (5) | ✓ | — |
| laya-riir · typed | 2000 | 0.7445 | 0.7349 | 0.1924 | 0.4099 | 0.7185 | 0.1263 | 0.8590 | 0.3953 | — / — | — | 1359.0 ms | 3056.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2933 · calibrated 0.0805 · conformal-naive floor 0.3087 → **PASS** (beats both of the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3166 · brier_soft 0.2459 · score MAE 0.7394 · within_1 0.7037

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1867 | 0.0828 | 0.2695 |
| modelless | noul | 600 | 0.5317 | 0.0250 | 0.5067 |
| modelless | score | 800 | 0.2587 | 0.0540 | 0.3096 |
| laya-riir·english | choice | 600 | 0.2883 | 0.1681 | 0.4527 |
| laya-riir·english | noul | 600 | 0.4733 | 0.2808 | 0.7542 |
| laya-riir·english | score | 800 | 0.3225 | 0.2065 | 0.5168 |
| laya-riir·multilingual | choice | 600 | 0.2950 | 0.3623 | 0.6559 |
| laya-riir·multilingual | noul | 600 | 0.4867 | 0.4084 | 0.8919 |
| laya-riir·multilingual | score | 800 | 0.2863 | 0.2313 | 0.5110 |
| laya-python·english | choice | 600 | 0.2883 | 0.1681 | 0.4527 |
| laya-python·english | noul | 600 | 0.4733 | 0.2808 | 0.7542 |
| laya-python·english | score | 800 | 0.3225 | 0.2065 | 0.5168 |
| laya-python·multilingual | choice | 600 | 0.2950 | 0.3623 | 0.6559 |
| laya-python·multilingual | noul | 600 | 0.4867 | 0.4084 | 0.8919 |
| laya-python·multilingual | score | 800 | 0.2863 | 0.2313 | 0.5110 |
| laya-python·typed | choice | 600 | 0.7333 | 0.2548 | 0.4785 |
| laya-python·typed | noul | 600 | 0.7850 | 0.1294 | 0.6624 |
| laya-python·typed | score | 800 | 0.7225 | 0.1987 | 0.5246 |
| laya-riir·typed | choice | 600 | 0.7333 | 0.2548 | 0.4785 |
| laya-riir·typed | noul | 600 | 0.7850 | 0.1294 | 0.6624 |
| laya-riir·typed | score | 800 | 0.7225 | 0.1987 | 0.5246 |
**typed-decisions extras (laya-riir·english):** soft_acc 0.3346 · brier_soft 0.3348 · score MAE 0.6937 · within_1 0.7525
**typed-decisions extras (laya-riir·multilingual):** soft_acc 0.3271 · brier_soft 0.4737 · score MAE 0.7604 · within_1 0.7000
**typed-decisions extras (laya-python·english):** soft_acc 0.3346 · brier_soft 0.3348 · score MAE 0.6937 · within_1 0.7525
**typed-decisions extras (laya-python·multilingual):** soft_acc 0.3271 · brier_soft 0.4737 · score MAE 0.7604 · within_1 0.7000
**typed-decisions extras (laya-python·typed):** soft_acc 0.4668 · brier_soft 0.0677 · score MAE 0.2424 · within_1 0.9950
**typed-decisions extras (laya-riir·typed):** soft_acc 0.4668 · brier_soft 0.0677 · score MAE 0.2424 · within_1 0.9950

## ag_news — 400 cases / 400 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5100 | 0.4725 | 0.2493 | 0.7419 | 1.3702 | 0.4899 | 0.5350 | 0.3797 | 0.49/0.26 | 0.5436 | 0.153 ms | 0.192 ms (5) | ✓ | s 0.0001 / d 0.3709 (n 200) |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.0316 | 0.0808 | 0.1637 | 0.0067 | 1.0000 | 0.1412 | — / — | — | 107.0 ms | 177.0 ms (5) | ✓ | — |
| laya-python · english | 400 | 0.9500 | 0.9439 | 0.0316 | 0.0808 | 0.1637 | 0.0067 | 1.0000 | 0.1412 | — / — | — | 32.0 ms | 84.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.3797 · conformal-naive floor 0.2506 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2825 | 0.1511 | 0.1128 | 0.8319 | 1.7874 | 0.7236 | 0.2850 | 0.0226 | 0.62/0.36 | 0.2773 | 0.117 ms | 0.149 ms (5) | ✓ | s 0.0000 / d 0.4153 (n 200) |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.3088 | 0.6963 | 2.1840 | 0.2657 | 0.7150 | 0.2571 | — / — | — | 73.0 ms | 185.0 ms (5) | ✓ | — |
| laya-python · english | 400 | 0.5925 | 0.4709 | 0.3088 | 0.6963 | 2.1840 | 0.2657 | 0.7150 | 0.2571 | — / — | — | 25.0 ms | 154.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2824 · calibrated 0.0226 · conformal-naive floor 0.2930 → **PASS** (beats both of the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2167 | 0.2044 | 0.0065 | 0.7988 | 1.6063 | 0.7849 | 0.2067 | 0.0165 | 0.57/0.32 | 0.2068 | 0.101 ms | 0.127 ms (7) | ✓ | s 0.0002 / d 0.3973 (n 200) |
| laya-riir · english | 600 | 0.3717 | 0.3292 | 0.2478 | 0.8172 | 1.7291 | 0.5279 | 0.4567 | 0.0811 | — / — | — | 86.0 ms | 103.0 ms (7) | ✓ | — |
| laya-python · english | 600 | 0.3717 | 0.3292 | 0.2478 | 0.8172 | 1.7291 | 0.5279 | 0.4567 | 0.0811 | — / — | — | 26.0 ms | 62.0 ms (7) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2159 · calibrated 0.0165 · conformal-naive floor 0.3300 → **PASS** (beats both of the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1774 · within_1 0.4450

## prompt_injections — 116 cases / 116 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4397 | 0.4396 | 0.1070 | 0.5154 | 0.7086 | 0.5713 | 0.3966 | 0.1466 | 0.56/0.88 | 0.2143 | 0.056 ms | 0.067 ms (2) | ✓ | s 0.0004 / d 0.4079 (n 100) |
| laya-riir · english | 116 | 0.6983 | 0.6751 | 0.2620 | 0.5259 | 3.1497 | 0.1073 | 0.9655 | 0.2620 | — / — | — | 84.0 ms | 181.0 ms (2) | ✓ | — |
| laya-python · english | 116 | 0.6983 | 0.6751 | 0.2620 | 0.5259 | 3.1497 | 0.1073 | 0.9655 | 0.2620 | — / — | — | 29.0 ms | 76.0 ms (2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4333 · calibrated 0.1466 · conformal-naive floor 0.3757 → **PASS** (beats both of the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3467 | 0.2151 | 0.0012 | 0.6674 | 1.0997 | 0.6898 | 0.3533 | 0.0965 | 0.64/0.38 | 0.3459 | 0.092 ms | 0.142 ms (4) | ✓ | s 0.0001 / d 0.5124 (n 200) |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.0685 | 0.2204 | 0.3987 | 0.0381 | 0.9800 | 0.1044 | — / — | — | 90.0 ms | 120.0 ms (4) | ✓ | — |
| laya-python · english | 300 | 0.8600 | 0.8612 | 0.0685 | 0.2204 | 0.3987 | 0.0381 | 0.9800 | 0.1044 | — / — | — | 27.0 ms | 56.0 ms (4) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3461 · calibrated 0.0965 · conformal-naive floor 0.3174 → **PASS** (beats both of the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.0767 | 0.0748 | 0.0200 | 0.9459 | 2.9549 | 0.9564 | 0.0467 | 0.0337 | 0.56/0.35 | 0.0825 | 0.079 ms | 0.108 ms (4) | ✓ | s 0.0560 / d 0.6319 (n 200) |
| laya-riir · english | 300 | 0.7500 | 0.7512 | 0.2047 | 0.4324 | 2.9079 | 0.1022 | 0.9333 | 0.2076 | — / — | — | 136.0 ms | 159.0 ms (4) | ✓ | — |
| laya-python · english | 300 | 0.7500 | 0.7512 | 0.2047 | 0.4324 | 2.9079 | 0.1022 | 0.9333 | 0.2076 | — / — | — | 38.0 ms | 61.0 ms (4) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.0200 · calibrated 0.0337 · conformal-naive floor 0.4894 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.4460 | 0.1515 | 0.4275 | 0.9778 | 4.0279 | 0.4253 | 0.5680 | 0.3640 | 1.00/1.00 | 0.0000 | 0.324 ms | 0.666 ms (6) | ✓ | s 0.0402 / d 0.4730 (n 200) |
| laya-riir · english | 500 | 0.4980 | 0.1519 | 0.3158 | 0.7996 | 5.3751 | 0.3141 | 0.6880 | 0.3314 | — / — | — | 206.0 ms | 242.0 ms (6) | ✓ | — |
| laya-python · english | 500 | 0.4980 | 0.1519 | 0.3158 | 0.7996 | 5.3751 | 0.3141 | 0.6880 | 0.3314 | — / — | — | 62.0 ms | 114.0 ms (6) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4275 · calibrated 0.3640 · conformal-naive floor 0.4410 → **PASS** (beats both of the uncalibrated output AND the floor)

## code_fixtures — 14 cases / 28 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 28 | 0.2143 | 0.0779 | 0.1188 | 0.6945 | 1.4111 | 0.5596 | 0.4286 | 0.2148 | 0.29/0.25 | 0.2381 | 0.079 ms | 0.186 ms (1) | ✓ | s 0.0001 / d 0.2303 (n 40) |
| laya-riir · english | 28 | 0.5357 | 0.4133 | 0.2314 | 0.6123 | 1.4189 | 0.3251 | 0.7143 | 0.2125 | — / — | — | 276.0 ms | 956.0 ms (1) | ✓ | — |
| laya-python · english | 28 | 0.5357 | 0.4133 | 0.2314 | 0.6123 | 1.4189 | 0.3251 | 0.7143 | 0.2125 | — / — | — | 99.0 ms | 268.0 ms (1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2099 · calibrated 0.2148 · conformal-naive floor 0.4127 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## harness_visibility — 16 cases / 16 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.009 ms | 0.011 ms (1) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| laya-riir · english | 16 | 0.3125 | 0.2738 | 0.3731 | 0.7005 | 1.2409 | 0.4633 | 0.6250 | 0.2218 | — / — | — | 75.0 ms | 109.0 ms (1) | ✓ | — |
| laya-python · english | 16 | 0.3125 | 0.2738 | 0.3731 | 0.7005 | 1.2409 | 0.4633 | 0.6250 | 0.2218 | — / — | — | 19.0 ms | 118.0 ms (1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## harness_permissions — 12 cases / 12 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.007 ms | 0.009 ms (1) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| laya-riir · english | 12 | 0.4167 | 0.3111 | 0.4358 | 0.8133 | 1.2925 | 0.8356 | 0.1667 | 0.4770 | — / — | — | 74.0 ms | 125.0 ms (1) | ✓ | — |
| laya-python · english | 12 | 0.4167 | 0.3111 | 0.4358 | 0.8133 | 1.2925 | 0.8356 | 0.1667 | 0.4770 | — / — | — | 36.0 ms | 110.0 ms (1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## harness_tool_fit — 12 cases / 12 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.008 ms | 0.009 ms (1) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| laya-riir · english | 12 | 0.5000 | 0.4111 | 0.4619 | 0.7146 | 1.7865 | 0.4104 | 0.5000 | 0.3852 | — / — | — | 57.0 ms | 121.0 ms (1) | ✓ | — |
| laya-python · english | 12 | 0.5000 | 0.4111 | 0.4619 | 0.7146 | 1.7865 | 0.4104 | 0.5000 | 0.3852 | — / — | — | 28.0 ms | 102.0 ms (1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## harness_routing — 16 cases / 16 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.009 ms | 0.011 ms (1) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| laya-riir · english | 16 | 0.6250 | 0.5970 | 0.2251 | 0.5411 | 1.0299 | 0.1734 | 0.7500 | 0.4866 | — / — | — | 85.0 ms | 135.0 ms (1) | ✓ | — |
| laya-python · english | 16 | 0.6250 | 0.5970 | 0.2251 | 0.5411 | 1.0299 | 0.1734 | 0.7500 | 0.4866 | — / — | — | 89.0 ms | 234.0 ms (1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## harness_sensitivity — 15 cases / 15 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.010 ms | 0.012 ms (1) | ✓ | s 0.0111 / d 0.4172 (n 20) |
| laya-riir · english | 15 | 0.6000 | 0.6000 | 0.3761 | 0.5874 | 1.0559 | 0.4451 | 0.7143 | 0.5525 | — / — | — | 92.0 ms | 139.0 ms (1) | ✓ | — |
| laya-python · english | 15 | 0.6000 | 0.6000 | 0.3761 | 0.5874 | 1.0559 | 0.4451 | 0.7143 | 0.5525 | — / — | — | 38.0 ms | 117.0 ms (1) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3816 · calibrated 0.3816 · conformal-naive floor 0.3429 → **FAIL** (does not beat both of the uncalibrated output AND the floor)

## harness_cache_reuse — 12 cases / 12 questions

> modelless lane: SKIPPED — LLM-lane only (the modelless lane has no KV cache, so it has no honest answer for this family); the laya lane answered below.

| lane · model | n | acc | ECE(maxp) | readout-ECE | p50 | p99 (support) | det |
|---|---|---|---|---|---|---|---|
| laya-riir · english | 12 | 0.5000 | 0.3618 | 0.3618 | 88.0 ms | 148.0 ms (1) | ✓ |
| laya-python · english | 12 | 0.5000 | 0.3618 | 0.3618 | 41.0 ms | 122.0 ms (1) | ✓ |

