# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `afacc3a` on `m3-ane` (2026-09-24T07:16:02Z) · profile release · laya feature true
- laya posture: PRE-MOVE BASELINE — `src/laya` is scheduled to move to the riir-infer repo (008 T4); the laya columns pin the pre-move tree at the sha above
- box state (Issue 021): start power=AC Power mode=high load=7.08 swap=2564M · end power=AC Power mode=high load=5.92 swap=2564M — ⛔ latency NOT QUOTABLE — load 7.08 > 6 — a sibling job is on the box
- laya device posture: ane (LAYA_DEVICE=ane → load_ane; whole-graph CoreML encoder, Plan 002)
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

## Absences / errors (honest — never silently dropped)

- typed_decisions (laya/typed): all 400 case(s) exceed the ANE buckets (max 128) — no servable cases; this device row is absent for this suite (named, not fabricated)
- typed_decisions (laya/english): all 400 case(s) exceed the ANE buckets (max 128) — no servable cases; this device row is absent for this suite (named, not fabricated)
- typed_decisions (laya/multilingual): all 400 case(s) exceed the ANE buckets (max 128) — no servable cases; this device row is absent for this suite (named, not fabricated)
- ag_news (laya/english): 35 case(s) skipped over the ANE bucket limit — ["ag_news:1", "ag_news:5", "ag_news:6", "ag_news:7", "ag_news:10", "ag_news:23", "ag_news:27", "ag_news:28", "ag_news:29", "ag_news:35", "ag_news:62", "ag_news:71", "ag_news:88", "ag_news:89", "ag_news:97", "ag_news:117", "ag_news:124", "ag_news:166", "ag_news:170", "ag_news:184", "ag_news:194", "ag_news:196", "ag_news:198", "ag_news:200", "ag_news:215", "ag_news:243", "ag_news:256", "ag_news:258", "ag_news:290", "ag_news:301", "ag_news:305", "ag_news:333", "ag_news:369", "ag_news:372", "ag_news:380"] (a coverage limit, not a failure; the served row is the smaller set)
- prompt_injections (laya/english): 9 case(s) skipped over the ANE bucket limit — ["prompt_injections:0", "prompt_injections:8", "prompt_injections:37", "prompt_injections:45", "prompt_injections:57", "prompt_injections:61", "prompt_injections:84", "prompt_injections:107", "prompt_injections:109"] (a coverage limit, not a failure; the served row is the smaller set)
- xnli_en (laya/english): 2 case(s) skipped over the ANE bucket limit — ["xnli_en:4", "xnli_en:5"] (a coverage limit, not a failure; the served row is the smaller set)
- massive_intent_en (laya/english): all 300 case(s) exceed the ANE buckets (max 128) — no servable cases; this device row is absent for this suite (named, not fabricated)
- banking77 (laya/english): all 500 case(s) exceed the ANE buckets (max 128) — no servable cases; this device row is absent for this suite (named, not fabricated)
- code_fixtures (laya/english): 8 case(s) skipped over the ANE bucket limit — ["code:embed.rs:0", "code:embed.rs:1", "code:engine.rs:1", "code:readout.rs:0", "code:readout.rs:1", "code:serve.rs:0", "code:harness/metrics.rs:0", "code:harness/metrics.rs:1"] (a coverage limit, not a failure; the served row is the smaller set)

## typed_decisions — 400 cases / 2000 questions

**corpus cap:** 48 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3190 | 0.2270 | 0.0527 | 0.6924 | 1.2331 | 0.5525 | 0.4400 | 0.0805 | 0.82/0.93 | 0.3493 | 0.487 ms | 1.726 ms (5) | ✓ | s 0.0139 / d 0.9213 (n 100) |
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

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5100 | 0.4725 | 0.2493 | 0.7419 | 1.3702 | 0.4899 | 0.5350 | 0.3797 | 0.49/0.26 | 0.5436 | 0.148 ms | 0.177 ms (5) | ✓ | s 0.0001 / d 0.3709 (n 200) |
| laya-riir · english | 365 | 0.9452 | 0.9374 | 0.0387 | 0.0864 | 0.1734 | 0.0075 | 1.0000 | 0.1421 | — / — | — | 28.0 ms | 34.0 ms (4) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.3797 · conformal-naive floor 0.2506 → **FAIL** (does not beat both)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2825 | 0.1511 | 0.1128 | 0.8319 | 1.7874 | 0.7236 | 0.2850 | 0.0226 | 0.62/0.36 | 0.2773 | 0.113 ms | 0.138 ms (5) | ✓ | s 0.0000 / d 0.4153 (n 200) |
| laya-riir · english | 400 | 0.5975 | 0.4777 | 0.3040 | 0.6964 | 2.1820 | 0.2659 | 0.7150 | 0.2547 | — / — | — | 19.0 ms | 26.0 ms (5) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2824 · calibrated 0.0226 · conformal-naive floor 0.2931 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2167 | 0.2044 | 0.0065 | 0.7988 | 1.6063 | 0.7849 | 0.2067 | 0.0165 | 0.57/0.32 | 0.2068 | 0.099 ms | 0.120 ms (7) | ✓ | s 0.0002 / d 0.3973 (n 200) |
| laya-riir · english | 600 | 0.3733 | 0.3303 | 0.2461 | 0.8179 | 1.7314 | 0.5290 | 0.4533 | 0.0807 | — / — | — | 24.0 ms | 27.0 ms (7) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2159 · calibrated 0.0165 · conformal-naive floor 0.3300 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1774 · within_1 0.4450

## prompt_injections — 116 cases / 116 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4397 | 0.4396 | 0.1070 | 0.5154 | 0.7086 | 0.5713 | 0.3966 | 0.1466 | 0.56/0.88 | 0.2143 | 0.056 ms | 0.067 ms (2) | ✓ | s 0.0004 / d 0.4079 (n 100) |
| laya-riir · english | 107 | 0.6916 | 0.6522 | 0.2668 | 0.5192 | 2.8819 | 0.1046 | 0.9811 | 0.2668 | — / — | — | 23.0 ms | 652.0 ms (2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4333 · calibrated 0.1466 · conformal-naive floor 0.3757 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**corpus cap:** 64 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3467 | 0.2151 | 0.0012 | 0.6674 | 1.0997 | 0.6898 | 0.3533 | 0.0965 | 0.64/0.38 | 0.3459 | 0.088 ms | 0.109 ms (4) | ✓ | s 0.0001 / d 0.5124 (n 200) |
| laya-riir · english | 298 | 0.8591 | 0.8603 | 0.0680 | 0.2222 | 0.4017 | 0.0385 | 0.9799 | 0.1039 | — / — | — | 28.0 ms | 34.0 ms (3) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3461 · calibrated 0.0965 · conformal-naive floor 0.3174 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.6900 | 0.6908 | 0.6227 | 0.9196 | 2.7260 | 0.1560 | 0.8667 | 0.0750 | 0.41/0.35 | 0.7268 | 0.091 ms | 0.118 ms (4) | ✓ | s 0.0621 / d 0.6319 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.6227 · calibrated 0.0750 · conformal-naive floor 0.0854 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.4460 | 0.1515 | 0.4275 | 0.9778 | 4.0279 | 0.4253 | 0.5680 | 0.1810 | 1.00/0.23 | 0.5325 | 0.322 ms | 0.680 ms (6) | ✓ | s 0.0360 / d 0.4730 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4275 · calibrated 0.1810 · conformal-naive floor 0.4410 → **PASS** (beats both the uncalibrated output AND the floor)

## code_fixtures — 12 cases / 24 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 24 | 0.2917 | 0.1088 | 0.0481 | 0.6889 | 1.3912 | 0.5512 | 0.4167 | 0.1530 | 0.29/0.29 | 0.3529 | 0.148 ms | 0.236 ms (1, max@case8) | ✓ | s 0.0001 / d 0.2673 (n 36) |
| laya-riir · english | 8 | 0.5000 | 0.4000 | 0.3518 | 0.6183 | 1.3504 | 0.2390 | 0.7500 | 0.2937 | — / — | — | 695.0 ms | 730.0 ms (1, max@case3) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.2875 · calibrated 0.1530 · conformal-naive floor 0.3510 → **PASS** (beats both the uncalibrated output AND the floor)

## harness_visibility — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.009 ms | 0.011 ms (1, max@case0) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| laya-riir · english | 16 | 0.3125 | 0.2738 | 0.3717 | 0.6989 | 1.2381 | 0.4841 | 0.6250 | 0.2218 | — / — | — | 23.0 ms | 701.0 ms (1, max@case3) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_permissions — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.006 ms | 0.007 ms (1, max@case0) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| laya-riir · english | 12 | 0.4167 | 0.3111 | 0.4355 | 0.8125 | 1.2923 | 0.8356 | 0.1667 | 0.4764 | — / — | — | 23.0 ms | 668.0 ms (1, max@case0) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_tool_fit — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.008 ms | 0.010 ms (1, max@case4) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| laya-riir · english | 12 | 0.5000 | 0.4111 | 0.4627 | 0.7158 | 1.7786 | 0.4104 | 0.5000 | 0.3875 | — / — | — | 19.0 ms | 691.0 ms (1, max@case0) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_routing — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.009 ms | 0.011 ms (1, max@case9) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| laya-riir · english | 16 | 0.6250 | 0.5970 | 0.2252 | 0.5420 | 1.0325 | 0.1734 | 0.7500 | 0.4862 | — / — | — | 27.0 ms | 659.0 ms (1, max@case0) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_sensitivity — 15 cases / 15 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.008 ms | 0.010 ms (1, max@case8) | ✓ | s 0.0111 / d 0.4172 (n 20) |
| laya-riir · english | 15 | 0.6667 | 0.6691 | 0.3768 | 0.5846 | 1.0511 | 0.4375 | 0.7143 | 0.5510 | — / — | — | 25.0 ms | 644.0 ms (1, max@case0) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.3816 · calibrated 0.3816 · conformal-naive floor 0.3429 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_cache_reuse — 12 cases / 12 questions

> modelless lane: SKIPPED — LLM-lane only (the modelless lane has no KV cache, so it has no honest answer for this family); the laya lane answered below.

| lane · model | n | acc | ECE(maxp) | readout-ECE | p50 | p99 (support) | det |
|---|---|---|---|---|---|---|---|
| laya-riir · english | 12 | 0.5000 | 0.3622 | 0.3622 | 26.0 ms | 649.0 ms (1, max@case0) | ✓ |

