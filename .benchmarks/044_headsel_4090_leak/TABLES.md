# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `d727196` on `4090-windows` (2026-09-25T19:47:18Z) · profile release · laya feature false
- box state (Issue 021): start power=? mode=? load=? swap=?M · end power=? mode=? load=? swap=?M — ⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- corpus cap posture: registry defaults
- label heads: ON — cal-selected per suite (ladder 0/0.25/0.5/1, forced cal-slice accuracy, ties → 0 = off; issue 030 lever 4; per-row candidates in the results)
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

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3190 | 0.2270 | 0.0527 | 0.6924 | 1.2331 | 0.5525 | 0.4400 | 0.0805 | 0.82/0.93 | 0.3493 | 0.490 ms | 2.741 ms (5) | ✓ | s 0.0139 / d 0.9213 (n 100) |
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

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5100 | 0.4725 | 0.2493 | 0.7419 | 1.3702 | 0.4899 | 0.5350 | 0.3797 | 0.49/0.26 | 0.5436 | 0.284 ms | 2.371 ms (5) | ✓ | s 0.0001 / d 0.3709 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.3797 · conformal-naive floor 0.2506 → **FAIL** (does not beat both)

## emotion — 400 cases / 400 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 ← selected | 0.1850 |
| 0.25 | 0.2050 |
| 0.5 | 0.2050 |
| 1 | 0.2100 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2825 | 0.1511 | 0.1128 | 0.8319 | 1.7874 | 0.7236 | 0.2850 | 0.0226 | 0.62/0.36 | 0.2773 | 0.178 ms | 0.492 ms (5) | ✓ | s 0.0000 / d 0.4153 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2824 · calibrated 0.0226 · conformal-naive floor 0.2931 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**corpus cap:** 64 (registry)

**label heads:** cal-selected scale 0.5 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.2050 |
| 0.25 | 0.2450 |
| 0.5 ← selected | 0.2550 |
| 1 | 0.2500 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2167 | 0.2164 | 0.0020 | 0.7992 | 1.6073 | 0.7985 | 0.1767 | 0.0215 | 0.54/0.32 | 0.2044 | 0.161 ms | 0.783 ms (7) | ✓ | s 0.0004 / d 0.3973 (n 200) |
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

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5004 | 0.6936 | 0.3981 | 0.6552 | 0.3663 | 0.41/0.41 | 0.4118 | 0.087 ms | 0.243 ms (2) | ✓ | s 0.0001 / d 0.4079 (n 100) |
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

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3467 | 0.2151 | 0.0012 | 0.6674 | 1.0997 | 0.6898 | 0.3533 | 0.0965 | 0.64/0.38 | 0.3459 | 0.142 ms | 0.744 ms (4) | ✓ | s 0.0001 / d 0.5124 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3461 · calibrated 0.0965 · conformal-naive floor 0.3174 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**corpus cap:** 48 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.5150 |
| 0.25 | 0.5850 |
| 0.5 | 0.5950 |
| 1 ← selected | 0.6100 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.7933 | 0.7955 | 0.7111 | 0.8910 | 2.5413 | 0.0519 | 0.9667 | 0.1134 | 0.45/0.35 | 0.8505 | 0.074 ms | 0.178 ms (4) | ✓ | s 0.0679 / d 0.6319 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.7111 · calibrated 0.1134 · conformal-naive floor 0.2177 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**corpus cap:** 40 (registry)

**label heads:** cal-selected scale 1 (ladder 0/0.25/0.5/1, forced cal accuracy, ties → 0)

| head scale | cal acc |
|---|---|
| 0 | 0.4900 |
| 0.25 | 0.6250 |
| 0.5 | 0.6500 |
| 1 ← selected | 0.6800 |

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.6840 | 0.2773 | 0.6612 | 0.9697 | 3.8373 | 0.1050 | 0.9400 | 0.4689 | 1.00/0.23 | 0.7766 | 0.486 ms | 1.050 ms (6) | ✓ | s 0.0390 / d 0.4730 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.6612 · calibrated 0.4689 · conformal-naive floor 0.6790 → **PASS** (beats both the uncalibrated output AND the floor)

## code_fixtures — 12 cases / 24 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 24 | 0.3333 | 0.2242 | 0.0928 | 0.6853 | 1.3719 | 0.6940 | 0.4167 | 0.1944 | 0.29/0.29 | 0.4118 | 0.300 ms | 0.588 ms (1, max@case0) | ✓ | s 0.0001 / d 0.2543 (n 36) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3297 · calibrated 0.1944 · conformal-naive floor 0.4092 → **PASS** (beats both the uncalibrated output AND the floor)

## harness_visibility — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.3750 | 0.3083 | 0.1994 | 0.7330 | 1.3525 | 0.4415 | 0.5000 | 0.3723 | 0.50/0.50 | 0.5000 | 0.016 ms | 0.021 ms (1, max@case0) | ✓ | s 0.0012 / d 0.7100 (n 20) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3723 · calibrated 0.3723 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_permissions — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.4167 | 0.3016 | 0.2232 | 0.6260 | 1.0380 | 0.3217 | 0.6667 | 0.3971 | 0.42/0.42 | 0.5714 | 0.011 ms | 0.016 ms (1, max@case8) | ✓ | s 0.0066 / d 0.3835 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.3971 · calibrated 0.3971 · conformal-naive floor 0.2500 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_tool_fit — 12 cases / 12 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 12 | 0.5000 | 0.5032 | 0.3273 | 0.7920 | 1.6724 | 0.2040 | 0.8333 | 0.4937 | 0.42/0.42 | 0.7143 | 0.013 ms | 0.022 ms (1, max@case1) | ✓ | s 0.0042 / d 0.1777 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4937 · calibrated 0.4937 · conformal-naive floor 0.1754 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_routing — 16 cases / 16 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 16 | 0.4375 | 0.3208 | 0.1523 | 0.7225 | 1.3314 | 0.3161 | 0.6250 | 0.4296 | 0.44/0.44 | 0.4444 | 0.015 ms | 0.017 ms (1, max@case0) | ✓ | s 0.0028 / d 0.4689 (n 16) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4296 · calibrated 0.4296 · conformal-naive floor 0.3125 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## harness_sensitivity — 15 cases / 15 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 15 | 0.4000 | 0.3089 | 0.1557 | 0.7623 | 1.5144 | 0.4692 | 0.4286 | 0.3816 | 0.47/0.47 | 0.3750 | 0.014 ms | 0.017 ms (1, max@case12) | ✓ | s 0.0111 / d 0.4172 (n 20) |
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

