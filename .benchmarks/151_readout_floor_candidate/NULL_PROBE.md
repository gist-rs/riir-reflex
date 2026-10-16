# Null/K-stability probe — 2026-10-16T01:29:13Z (host shikuwa, datasets E:\git\riir-reflex\.raw/datasets) — ladder [0, 8, 32, 64, 256], ≤500 slot(s)/suite

- decoys: pool = the union of the run's suite label vocabularies (first-seen order); per slot: pool minus the slot's own labels, ranked by trigram-Jaccard to the GOLD label DESC, then K evenly-spaced picks across the ranked list (the linspace law: near→far graded, deterministic); decoys APPEND (originals keep positions); a pool smaller than K caps the arm (disclosed per suite).
- gates: CONTROL gates (a red refuses the run): A2 — ZERO WITHIN-REGIME preference flips beyond exact reference ties (the engine's by-name route arms only while k ≤ N, so an arm past the domain count switches regimes BY DESIGN — those flips are the arming law, disclosed as cross-regime, never red); A3a — the paired noul readout BIT-identical across arms; A1-floor (Report-the-Floor, Plan 340 shape) — the DEPLOYED (cal-slice-calibrated) confidence's abstention AUROC/ECE must beat the conformal-naive floor over the raw max-prob on the cal/test split (≥1 axis better, none worse). A1's drift curve itself is CHARACTERIZED (the issue's own ⚠: max-of-K drift is the measured quantity), never gated on an unstudied ε. The Issue-092 2-feature candidate leg (σ(a·maxp + b·ent + c), fit on the same cal slice) is REPORT-ONLY: it never gates the run and never arms anything. Comparison-lane reds are disclosed rows.
- laya: laya not wired in this unit: the perm-probe seam discards the answer's confidence field and A1/A3 need it — the lane joins when its oracle exposes a confidence readout.

## typed_decisions (pool 153 labels) — 1 lane cell(s)
### modelless — 600 slot(s) · CONTROL · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 100.0 | +0.0 | 0.00 | 0.0 | 295 | 0 (0/0) | PASS |
| 8 | 100.0 | +0.0 | 0.10 | 19.8 | 295 | 0 (0/0) | PASS |
| 32 | 100.0 | +0.0 | 0.08 | 60.2 | 295 | 0 (0/0) | PASS |
| 64 | 100.0 | +0.0 | 0.07 | 79.3 | 295 | 0 (0/0) | PASS |
| 256 | 100.0 | +0.0 | 0.05 | 83.7 | 295 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 0.05 pt)
- floor (Report-the-Floor): gate AUROC 0.5726 / ECE 0.1679 vs floor AUROC 0.5716 / ECE 0.4350 (n_cal 150, n_test 600) — **PASS**
- candidate (Issue 092, REPORT-ONLY): σ(-2.013·maxp +0.016·ent -1.271) AUROC 0.4296 / ECE 0.0066 vs floor 0.5716/0.4350 · shipped gate 0.5726/0.1679 (n_cal 150, n_test 600) — **RED**
- slice: slice-integrity: test 400 (fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100 (fnv1a64-a1679de56e9278b1) · OK

## ag_news (pool 153 labels) — 1 lane cell(s)
### modelless — 400 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 39.2 | +0.0 | 0.00 | 0.0 | 400 | 0 (0/0) | PASS |
| 8 | 39.2 | +0.0 | 32.28 | 0.0 | 400 | 114 (0/114) | PASS |
| 32 | 39.2 | +0.0 | 27.78 | 43.5 | 400 | 114 (0/114) | PASS |
| 64 | 39.2 | +0.0 | 25.08 | 74.2 | 400 | 114 (0/114) | PASS |
| 256 | 39.2 | +0.0 | 21.29 | 99.8 | 400 | 114 (0/114) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 21.29 pt)
- floor (Report-the-Floor): gate AUROC 0.5588 / ECE 0.0813 vs floor AUROC 0.5846 / ECE 0.2047 (n_cal 200, n_test 400) — **RED**
- candidate (Issue 092, REPORT-ONLY): σ(+3.826·maxp +0.269·ent -1.050) AUROC 0.5845 / ECE 0.0797 vs floor 0.5846/0.2047 · shipped gate 0.5588/0.0813 (n_cal 200, n_test 400) — **RED**
- slice: slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK

## emotion (pool 153 labels) — 1 lane cell(s)
### modelless — 400 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 33.0 | +0.0 | 0.00 | 0.0 | 400 | 0 (0/0) | PASS |
| 8 | 33.0 | +0.0 | 0.09 | 0.0 | 400 | 53 (0/53) | PASS |
| 32 | 33.0 | +0.0 | 0.07 | 0.0 | 400 | 53 (0/53) | PASS |
| 64 | 33.0 | +0.0 | 0.07 | 0.0 | 400 | 53 (0/53) | PASS |
| 256 | 33.0 | +0.0 | 0.06 | 0.0 | 400 | 53 (0/53) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 0.06 pt)
- floor (Report-the-Floor): gate AUROC 0.4745 / ECE 0.1297 vs floor AUROC 0.5208 / ECE 0.2980 (n_cal 200, n_test 400) — **RED**
- candidate (Issue 092, REPORT-ONLY): σ(-0.191·maxp -0.008·ent -1.742) AUROC 0.4791 / ECE 0.1300 vs floor 0.5208/0.2980 · shipped gate 0.4745/0.1297 (n_cal 200, n_test 400) — **RED**
- slice: slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK

## xnli_en (pool 153 labels) — 1 lane cell(s)
### modelless — 300 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 55.7 | +0.0 | 0.00 | 0.0 | 300 | 0 (0/0) | PASS |
| 8 | 32.3 | -23.3 | 18.84 | 0.0 | 300 | 13 (0/13) | PASS |
| 32 | 32.3 | -23.3 | 15.22 | 100.0 | 300 | 13 (0/13) | PASS |
| 64 | 32.3 | -23.3 | 13.24 | 100.0 | 300 | 13 (0/13) | PASS |
| 256 | 32.3 | -23.3 | 10.73 | 100.0 | 300 | 13 (0/13) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 10.73 pt)
- floor (Report-the-Floor): gate AUROC 0.4767 / ECE 0.0335 vs floor AUROC 0.4791 / ECE 0.3201 (n_cal 200, n_test 300) — **RED**
- candidate (Issue 092, REPORT-ONLY): σ(+0.875·maxp +0.083·ent -1.055) AUROC 0.4801 / ECE 0.0196 vs floor 0.4791/0.3201 · shipped gate 0.4767/0.0335 (n_cal 200, n_test 300) — **PASS**
- slice: slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK

## massive_intent_en (pool 153 labels) — 1 lane cell(s)
### modelless — 300 slot(s) · CONTROL · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 34.7 | +0.0 | 0.00 | 0.0 | 300 | 0 (0/0) | PASS |
| 8 | 100.0 | +65.3 | 57.44 | 63.3 | 300 | 117 (0/117) | PASS |
| 32 | 100.0 | +65.3 | 57.83 | 100.0 | 300 | 117 (0/117) | PASS |
| 64 | 100.0 | +65.3 | 57.83 | 100.0 | 300 | 117 (0/117) | PASS |
| 256 | 100.0 | +65.3 | 57.83 | 100.0 | 300 | 117 (0/117) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 57.83 pt)
- floor (Report-the-Floor): gate AUROC 0.7156 / ECE 0.0614 vs floor AUROC 0.7153 / ECE 0.1249 (n_cal 200, n_test 300) — **PASS**
- candidate (Issue 092, REPORT-ONLY): σ(+8.444·maxp +2.485·ent -0.809) AUROC 0.7117 / ECE 0.1729 vs floor 0.7153/0.1249 · shipped gate 0.7156/0.0614 (n_cal 200, n_test 300) — **RED**
- slice: slice-integrity: test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK

## banking77 (pool 153 labels) — 1 lane cell(s)
### modelless — 500 slot(s) · CONTROL · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 52.6 | +0.0 | 0.00 | 0.0 | 500 | 0 (0/0) | PASS |
| 8 | 39.0 | -13.6 | 7.11 | 85.0 | 500 | 130 (0/130) | PASS |
| 32 | 100.0 | +47.4 | 20.80 | 100.0 | 500 | 130 (0/130) | PASS |
| 64 | 100.0 | +47.4 | 32.76 | 100.0 | 500 | 130 (0/130) | PASS |
| 256 | 100.0 | +47.4 | 34.39 | 100.0 | 500 | 130 (0/130) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 34.39 pt)
- floor (Report-the-Floor): gate AUROC 0.6322 / ECE 0.0278 vs floor AUROC 0.6321 / ECE 0.1942 (n_cal 200, n_test 500) — **PASS**
- candidate (Issue 092, REPORT-ONLY): σ(+1.116·maxp +0.467·ent -0.445) AUROC 0.6171 / ECE 0.0010 vs floor 0.6321/0.1942 · shipped gate 0.6322/0.0278 (n_cal 200, n_test 500) — **RED**
- slice: slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK

red cells: 3 — control reds refuse the run (exit 1)
