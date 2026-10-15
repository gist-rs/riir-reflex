# Null/K-stability probe — 2026-10-15T22:25:54Z (host m3, datasets /Users/katopz/git/riir-reflex/.raw/datasets) — ladder [0, 8, 32, 64, 256], ≤40 slot(s)/suite

- decoys: pool = the union of the run's suite label vocabularies (first-seen order); per slot: pool minus the slot's own labels, ranked by trigram-Jaccard to the GOLD label DESC, then K evenly-spaced picks across the ranked list (the linspace law: near→far graded, deterministic); decoys APPEND (originals keep positions); a pool smaller than K caps the arm (disclosed per suite).
- gates: CONTROL gates (a red refuses the run): A2 — ZERO WITHIN-REGIME preference flips beyond exact reference ties (the engine's by-name route arms only while k ≤ N, so an arm past the domain count switches regimes BY DESIGN — those flips are the arming law, disclosed as cross-regime, never red); A3a — the paired noul readout BIT-identical across arms; A1-floor (Report-the-Floor, Plan 340 shape) — the DEPLOYED (cal-slice-calibrated) confidence's abstention AUROC/ECE must beat the conformal-naive floor over the raw max-prob on the cal/test split (≥1 axis better, none worse). A1's drift curve itself is CHARACTERIZED (the issue's own ⚠: max-of-K drift is the measured quantity), never gated on an unstudied ε. Comparison-lane reds are disclosed rows.
- laya: laya not wired in this unit: the perm-probe seam discards the answer's confidence field and A1/A3 need it — the lane joins when its oracle exposes a confidence readout.

## typed_decisions (pool 153 labels) — 2 lane cell(s)
### modelless — 60 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 100.0 | +0.0 | 0.00 | 0.0 | 29 | 0 (0/0) | PASS |
| 8 | 100.0 | +0.0 | 0.10 | 16.7 | 29 | 0 (0/0) | PASS |
| 32 | 100.0 | +0.0 | 0.08 | 58.3 | 29 | 0 (0/0) | PASS |
| 64 | 100.0 | +0.0 | 0.07 | 76.7 | 29 | 0 (0/0) | PASS |
| 256 | 100.0 | +0.0 | 0.05 | 85.0 | 29 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 0.05 pt)
- floor (Report-the-Floor): gate AUROC 0.5144 / ECE 0.1745 vs floor AUROC 0.6070 / ECE 0.4345 (n_cal 150, n_test 60) — **RED**
### jeva — 60 slot(s) · comparison · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 85.0 | +0.0 | 0.00 | 0.0 | 60 | 0 (0/0) | PASS |
| 8 | 91.7 | +6.7 | 0.17 | 8.3 | 60 | 0 (0/0) | PASS |
| 32 | 95.0 | +10.0 | 0.44 | 35.0 | 60 | 0 (0/0) | PASS |
| 64 | 96.7 | +11.7 | 1.24 | 53.3 | 60 | 0 (0/0) | PASS |
| 256 | 96.7 | +11.7 | 4.98 | 75.0 | 60 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 4.98 pt)
- skipped: jeva: arm K=256: pool capped (60/60 slots) — presented 157..158 vs requested
- slice: slice-integrity: test 400 (fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100 (fnv1a64-a1679de56e9278b1) · OK

## ag_news (pool 153 labels) — 2 lane cell(s)
### modelless — 40 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 37.5 | +0.0 | 0.00 | 0.0 | 40 | 0 (0/0) | PASS |
| 8 | 37.5 | +0.0 | 33.15 | 0.0 | 40 | 14 (0/14) | PASS |
| 32 | 37.5 | +0.0 | 28.55 | 30.0 | 40 | 14 (0/14) | PASS |
| 64 | 37.5 | +0.0 | 25.80 | 75.0 | 40 | 14 (0/14) | PASS |
| 256 | 37.5 | +0.0 | 22.09 | 97.5 | 40 | 14 (0/14) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 22.09 pt)
- floor (Report-the-Floor): gate AUROC 0.5680 / ECE 0.1363 vs floor AUROC 0.5840 / ECE 0.2953 (n_cal 200, n_test 40) — **RED**
### jeva — 40 slot(s) · comparison · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 57.5 | +0.0 | 0.00 | 0.0 | 40 | 0 (0/0) | PASS |
| 8 | 52.5 | -5.0 | 2.84 | 17.5 | 40 | 0 (0/0) | PASS |
| 32 | 67.5 | +10.0 | 6.88 | 27.5 | 40 | 0 (0/0) | PASS |
| 64 | 62.5 | +5.0 | 14.50 | 42.5 | 40 | 0 (0/0) | PASS |
| 256 | 70.0 | +12.5 | 19.52 | 55.0 | 40 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 19.52 pt)
- skipped: jeva: arm K=256: pool capped (40/40 slots) — presented 153..153 vs requested
- slice: slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK

## emotion (pool 153 labels) — 2 lane cell(s)
### modelless — 40 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 40.0 | +0.0 | 0.00 | 0.0 | 40 | 0 (0/0) | PASS |
| 8 | 40.0 | +0.0 | 0.09 | 0.0 | 40 | 1 (0/1) | PASS |
| 32 | 40.0 | +0.0 | 0.07 | 0.0 | 40 | 1 (0/1) | PASS |
| 64 | 40.0 | +0.0 | 0.07 | 0.0 | 40 | 1 (0/1) | PASS |
| 256 | 40.0 | +0.0 | 0.06 | 0.0 | 40 | 1 (0/1) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 0.06 pt)
- floor (Report-the-Floor): gate AUROC 0.5253 / ECE 0.1547 vs floor AUROC 0.7560 / ECE 0.3127 (n_cal 200, n_test 40) — **RED**
### jeva — 40 slot(s) · comparison · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 80.0 | +0.0 | 0.00 | 0.0 | 40 | 0 (0/0) | PASS |
| 8 | 82.5 | +2.5 | 0.03 | 7.5 | 40 | 0 (0/0) | PASS |
| 32 | 82.5 | +2.5 | 0.56 | 32.5 | 40 | 0 (0/0) | PASS |
| 64 | 85.0 | +5.0 | 1.82 | 37.5 | 40 | 0 (0/0) | PASS |
| 256 | 82.5 | +2.5 | 5.33 | 42.5 | 40 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 5.33 pt)
- skipped: jeva: arm K=256: pool capped (40/40 slots) — presented 153..153 vs requested
- slice: slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK

## xnli_en (pool 153 labels) — 2 lane cell(s)
### modelless — 38 slot(s) · CONTROL · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 63.2 | +0.0 | 0.00 | 0.0 | 38 | 0 (0/0) | PASS |
| 8 | 36.8 | -26.3 | 19.24 | 0.0 | 38 | 2 (0/2) | PASS |
| 32 | 36.8 | -26.3 | 15.55 | 100.0 | 38 | 2 (0/2) | PASS |
| 64 | 36.8 | -26.3 | 13.60 | 100.0 | 38 | 2 (0/2) | PASS |
| 256 | 36.8 | -26.3 | 11.09 | 100.0 | 38 | 2 (0/2) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 11.09 pt)
- floor (Report-the-Floor): gate AUROC 0.6435 / ECE 0.0772 vs floor AUROC 0.6261 / ECE 0.2646 (n_cal 200, n_test 38) — **PASS**
### jeva — 38 slot(s) · comparison · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 31.6 | +0.0 | 0.00 | 0.0 | 38 | 0 (0/0) | PASS |
| 8 | 34.2 | +2.6 | 0.18 | 0.0 | 38 | 0 (0/0) | PASS |
| 32 | 39.5 | +7.9 | 0.79 | 0.0 | 38 | 0 (0/0) | PASS |
| 64 | 39.5 | +7.9 | 3.25 | 2.6 | 38 | 0 (0/0) | PASS |
| 256 | 42.1 | +10.5 | 5.81 | 2.6 | 38 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 5.81 pt)
- skipped: jeva: arm K=256: pool capped (38/38 slots) — presented 153..153 vs requested
- slice: slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK

## massive_intent_en (pool 153 labels) — 2 lane cell(s)
### modelless — 38 slot(s) · CONTROL · RED
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 47.4 | +0.0 | 0.00 | 0.0 | 38 | 0 (0/0) | PASS |
| 8 | 100.0 | +52.6 | 57.21 | 57.9 | 38 | 18 (0/18) | PASS |
| 32 | 100.0 | +52.6 | 57.67 | 100.0 | 38 | 18 (0/18) | PASS |
| 64 | 100.0 | +52.6 | 57.67 | 100.0 | 38 | 18 (0/18) | PASS |
| 256 | 100.0 | +52.6 | 57.67 | 100.0 | 38 | 18 (0/18) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 57.67 pt)
- floor (Report-the-Floor): gate AUROC 0.6506 / ECE 0.1924 vs floor AUROC 0.6474 / ECE 0.1618 (n_cal 200, n_test 38) — **RED**
### jeva — 38 slot(s) · comparison · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 36.8 | +0.0 | 0.00 | 0.0 | 38 | 0 (0/0) | PASS |
| 8 | 36.8 | +0.0 | 0.03 | 2.6 | 38 | 0 (0/0) | PASS |
| 32 | 36.8 | +0.0 | 0.08 | 5.3 | 38 | 0 (0/0) | PASS |
| 64 | 36.8 | +0.0 | 0.19 | 10.5 | 38 | 0 (0/0) | PASS |
| 256 | 39.5 | +2.6 | 0.46 | 10.5 | 38 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 0.46 pt)
- skipped: jeva: arm K=256: pool capped (38/38 slots) — presented 153..153 vs requested
- slice: slice-integrity: test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK

## banking77 (pool 153 labels) — 2 lane cell(s)
### modelless — 39 slot(s) · CONTROL · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 46.2 | +0.0 | 0.00 | 0.0 | 39 | 0 (0/0) | PASS |
| 8 | 25.6 | -20.5 | 5.81 | 84.6 | 39 | 11 (0/11) | PASS |
| 32 | 100.0 | +53.8 | 21.84 | 100.0 | 39 | 11 (0/11) | PASS |
| 64 | 100.0 | +53.8 | 31.57 | 100.0 | 39 | 11 (0/11) | PASS |
| 256 | 100.0 | +53.8 | 33.06 | 100.0 | 39 | 11 (0/11) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 33.06 pt)
- floor (Report-the-Floor): gate AUROC 0.6028 / ECE 0.0953 vs floor AUROC 0.6014 / ECE 0.3054 (n_cal 200, n_test 39) — **PASS**
### jeva — 39 slot(s) · comparison · PASS
| K | abstain% | Δabstain pt | med\|Δconf\| pt | decoy-win% | A2 untied | flips (in-regime/cross) | A2 |
|---|---|---|---|---|---|---|---|
| 0 | 38.5 | +0.0 | 0.00 | 0.0 | 39 | 0 (0/0) | PASS |
| 8 | 38.5 | +0.0 | 0.00 | 0.0 | 39 | 0 (0/0) | PASS |
| 32 | 38.5 | +0.0 | 0.01 | 0.0 | 39 | 0 (0/0) | PASS |
| 64 | 38.5 | +0.0 | 0.02 | 0.0 | 39 | 0 (0/0) | PASS |
| 256 | 38.5 | +0.0 | 0.02 | 0.0 | 39 | 0 (0/0) | PASS |
- A3: noul bit-identity **BIT-IDENTICAL** · confidence joint-shaped (drifts with K — option-dependent readout) (med |Δconf| top arm 0.02 pt)
- skipped: jeva: arm K=256: pool capped (39/39 slots) — presented 153..153 vs requested
- slice: slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK

red cells: 4 — control reds refuse the run (exit 1)
