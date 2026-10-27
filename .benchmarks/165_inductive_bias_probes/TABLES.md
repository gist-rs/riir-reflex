# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `53e5dc0` on `amo` (2026-10-27T04:33:19Z) · profile release · laya feature true
- box state (Issue 021): start power=AC Power mode=? load=9.79 swap=4986M · end power=AC Power mode=? load=6.74 swap=4986M — ⛔ latency NOT QUOTABLE — load 9.79 > 6 — a sibling job is on the box; load 6.74 > 6 — a sibling job is on the box
- laya device posture: metal (LAYA_DEVICE or the build's macOS default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: off (pass --bekko to add the comparison lane; needs the bekko venv — Bench 103)
- agentjev lane: off (pass --agentjev to add the comparison lane; needs their jev_service on AGENTJEV_SERVE_URL — .issues/025)
- clef lane: off (pass --clef to add the comparison lane; the hosted posture needs the owner's Workers AI creds behind the loopback forwarder, or serve the open weights locally — plan 011 Phase A + .research/007)
- drex lane: off (pass --drex to add the comparison lane; needs their service on DREX_SERVE_URL, default http://127.0.0.1:8000 — Issue 073)
- d1 lane: off (pass --d1 to add the comparison lane; needs the reference server on D1_SERVE_URL, default http://127.0.0.1:8078 — Issue 078)
- pplx lane: off (pass --pplx to add the comparison lane; serve their autojev server on PPLX_SERVE_URL, default http://127.0.0.1:8793 — Issue 082)
- d3 lane: off (pass --d3 to add the comparison lane; serve their d3_server.py on D3_SERVE_URL, default http://127.0.0.1:8794 — riir-train 629 T2)
- startlux lane: off (pass --startlux to add the comparison lane; serve their startlux-decide engine on STARTLUX_SERVE_URL, default http://127.0.0.1:8090 — reflex Issue 095, riir-train 635 T3)
- mammoth lane: off (pass --mammoth to add the hosted comparison lane; run the loopback TLS forwarder to inference.baseten.co on MAMMOTH_SERVE_URL, default http://127.0.0.1:8793, with MAMMOTH_API_TOKEN set — rethink Issue 032 T10; T10.1: the key is an OWNER act; window ends 2026-10-27 23:59 PT)
- elephant lane: off (pass --elephant to add the self-hosted comparison lane; serve our scripts/elephant_server.py under the qev venv on ELEPHANT_SERVE_URL, default http://127.0.0.1:8096 — rethink Issue 032 T10.2, riir-train 640 T2(a) records the exact venv + weight pin)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16/32/64 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
- option-conditioned tables (issue 038 T7b): ON — cal-selected per suite (scale 0/0.25/0.5/1/2/4/8, promotion bar +5 pt over off on the stratified slice; one contrastive table per (question id, gold option) from the TRAIN rows, events filtered to the corpus pool; every question kind armed — typed_decisions is the target suite)
- NBSVM ridge readout (issue 038 T7a): ON — cal-selected per suite (scale 0/0.5/1/2/4/8 × λ 10 fixed, promotion bar +5 pt over off on the stratified slice; NBSVM closed-form ridge, k=2048, per-class NB log-count ratios, fit-time self-calibrated margin temperature)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved; --drafter-fix per_byte: the drafter-only correction mode armed (issue 079 tuning axis; EngineConfig::drafter_fix, issue 036 T2)
- sampling: test sample = label-STRATIFIED round-robin over the whole test split (budget = the registry test cap; first-appearance label order, dataset order within each label, deterministic, no RNG); cal slice = the same law over the train rows; corpus pool = the train rows MINUS the cal front (excluded by construction, not position); budget-0 suites unchanged (identity split) — Issue 039 T2
- readout: shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side arming lever overfit the narrow suites' cal slice — emotion's test G1 regressed — and coincided with Dispatch on the wide suites it targeted; the wide-label G1 gap was closed by T2's stratified cal slice). The candidate table is recorded per suite (report-only); EngineConfig::readout stays the opt-in knob
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- modal floor: input-keyed modal-answer floor = the cal-split (reference) modal gold over questions sharing the input key, pooled by option identity and mapped through each case's own option order (first-4 normalized input tokens × qid × option-count; ties → global prior → case order; arXiv:2610.17495's anti-mimicry census; LOO over eval only when no cal slice exists) — a lane at/below it carries no input-independent signal (issue 094)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: synthetic fixtures (semantic_defects): in-process fixtures with programmatic gold; the six Issue-004 harness families (2026-10-02) and code_fixtures (2026-10-11) were retired — owner calls — so the modelless lane no longer answers any self-authored suite but the one that reads above chance

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

**modal floor (issue 094):** acc `0.2625` over 400 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (195 keys, 398 fallbacks, reference-modal share `0.2500`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8825 | 0.8742 | 0.4783 | 0.5053 | 0.9630 | 0.0469 | 0.9400 | 0.0459 | 1.00/0.49 | 0.9510 | 0.148 ms | 0.301 ms (5) | ✓ | s 0.8359 / d 0.4105 (n 200) |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.0316 | 0.0808 | 0.1637 | 0.0067 | 1.0000 | 0.1412 | — / — | — | 10.0 ms | 16.0 ms (5) | ✓ | — |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.8825 | 0.8742 | 0.3075 | 0.8303 | ✓ |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.3075 | 0.9278 | ✓ |


**G1 (modelless readout ECE):** raw 0.8305 · calibrated 0.0459 · conformal-naive floor 0.2482 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK**

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

**transductive column (NOT the headline):** acc 0.8875 vs honest 0.8850 (+0.2 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.7589 · max_prob 0.5274 · inv_entropy 0.7589

**modal floor (issue 094):** acc `0.3125` over 400 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (162 keys, 267 fallbacks, reference-modal share `0.1700`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8850 | 0.8014 | 0.6182 | 0.6593 | 1.3643 | 0.0360 | 0.9750 | 0.0488 | 1.00/0.46 | 0.9349 | 0.093 ms | 0.111 ms (5) | ✓ | s 0.7631 / d 0.4265 (n 200) |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.3088 | 0.6963 | 2.1840 | 0.2657 | 0.7150 | 0.2571 | — / — | — | 9.0 ms | 11.0 ms (5) | ✓ | — |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.8850 | 0.8014 | 0.2975 | 0.8363 | ✓ |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.2975 | 0.4199 | ✓ |


**G1 (modelless readout ECE):** raw 0.8621 · calibrated 0.0488 · conformal-naive floor 0.2781 → **PASS** (beats both the uncalibrated output AND the floor)

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

**modal floor (issue 094):** acc `0.3333` over 300 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (198 keys, 288 fallbacks, reference-modal share `0.3350`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.5233 | 0.5061 | 0.1383 | 0.6178 | 1.0287 | 0.3120 | 0.6400 | 0.1119 | 1.00/0.54 | 0.6087 | 0.067 ms | 0.082 ms (4) | ✓ | s 0.5569 / d 0.5110 (n 200) |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.0685 | 0.2204 | 0.3987 | 0.0381 | 0.9800 | 0.1044 | — / — | — | 10.0 ms | 12.0 ms (4) | ✓ | — |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.5233 | 0.5061 | 0.3333 | 0.2850 | ✓ |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.3333 | 0.7900 | ✓ |


**G1 (modelless readout ECE):** raw 0.5133 · calibrated 0.1119 · conformal-naive floor 0.1480 → **PASS** (beats both the uncalibrated output AND the floor)

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

**modal floor (issue 094):** acc `0.0460` over 500 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (148 keys, 311 fallbacks, reference-modal share `0.0150`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.8420 | 0.8362 | 0.8194 | 0.9688 | 3.8215 | 0.0599 | 0.9720 | 0.0517 | 1.00/0.48 | 0.9349 | 0.298 ms | 0.566 ms (6) | ✓ | s 0.7478 / d 0.5549 (n 200) |
| laya-riir · english | 500 | 0.4220 | 0.3765 | 0.3813 | 0.9327 | 6.1219 | 0.4066 | 0.6000 | 0.3940 | — / — | — | 20.0 ms | 21.0 ms (6) | ✓ | — |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 500 | 0.8420 | 0.8362 | 0.0140 | 0.8398 | ✓ |
| laya-riir · english | 500 | 0.4220 | 0.3765 | 0.0140 | 0.4138 | ✓ |


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

