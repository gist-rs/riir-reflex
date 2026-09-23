# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `0d956be` on `m3` (2026-09-23T17:26:29Z) · profile release · laya feature true
- laya posture: PRE-MOVE BASELINE — `src/laya` is scheduled to move to the riir-infer repo (008 T4); the laya columns pin the pre-move tree at the sha above
- laya cap: 100 questions per checkpoint — PARTIAL run; the laya `n` columns reflect the cap and are NOT comparable row-wise against the modelless `n` columns
- laya device posture: metal (LAYA_DEVICE or the build's macOS default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: harness families (Issue 004, Research 579): in-process synthetic fixtures with programmatic gold; harness_cache_reuse is LLM-lane only (the modelless lane has no KV cache) and reports a loud SKIPPED absence without the laya-riir feature

## ag_news — 400 cases / 400 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.5100 | 0.4725 | 0.2493 | 0.7419 | 1.3702 | 0.4899 | 0.5350 | 0.3797 | 0.49/0.26 | 0.5436 | 0.147 ms | 0.164 ms (5) | ✓ | s 0.0001 / d 0.3709 (n 200) |
| laya-riir · english | 100 | 0.9500 | 0.9313 | 0.0377 | 0.0850 | 0.1615 | 0.0069 | 1.0000 | 0.1286 | — / — | — | 38.0 ms | 68.0 ms (2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.5093 · calibrated 0.3797 · conformal-naive floor 0.2506 → **FAIL** (does not beat both)

## banking77 — 500 cases / 500 questions

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.4460 | 0.1515 | 0.4275 | 0.9778 | 4.0279 | 0.4253 | 0.5680 | 0.3640 | 1.00/1.00 | 0.0000 | 0.313 ms | 0.670 ms (6) | ✓ | s 0.0402 / d 0.4730 (n 200) |
| laya-riir · english | 100 | 0.5200 | 0.1062 | 0.3149 | 0.7511 | 5.1654 | 0.2522 | 0.8400 | 0.3468 | — / — | — | 99.0 ms | 124.0 ms (2) | ✓ | — |

**G1 (modelless readout ECE):** raw 0.4275 · calibrated 0.3640 · conformal-naive floor 0.4410 → **PASS** (beats both the uncalibrated output AND the floor)

