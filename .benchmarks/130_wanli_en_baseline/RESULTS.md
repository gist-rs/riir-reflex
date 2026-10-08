# Bench 130 — wanli_en baseline (the ESC re-source NLI lane)

**Status:** RECORD — the suite's modelless-floor cells; the rethink-side ESC
re-fit + GOAT is tracked in riir-rethink Issue 024 (reflex-side lane:
reflex issue 080).

## What ran

First harness measurement of the new `wanli_en` suite (alisawuffles/WANLI —
the CC BY re-source for the licence-barred xnli_en niche; licence verified at
source 2026-10-08, see dataset_manifest.md §Licences). Modelless lane ONLY
(`--skip-laya`): no wanli specialist exists yet — the A1/H2 specialist +
fusion cells are the rethink-side ESC re-fit's product, not a reflex-side
deliverable.

```
cargo run --release --bin harness -- --skip-laya --suites wanli_en \
  --out .benchmarks/130_wanli_en_baseline
```

Posture: registry defaults — test 300 (stratified round-robin over the whole
5,000-row split), cal 200, corpus cap 64/label, pool floor 16,000 (measured
pool 19,800 = 20,000 train − 200 cal).

## Cells (results.json)

| cell | value |
|---|---|
| **A0 modelless acc** | **0.3100** (300 cases) |
| macro F1 | 0.2042 |
| ECE(maxp) | 0.0384 (raw readout ECE 0.3078 → calibrated 0.0871) |
| Brier / NLL | 0.6689 / 1.1019 |
| acc@50 coverage | 0.3000 |
| G1 (readout ECE vs conformal-naive floor) | **PASS** — calibrated 0.0871 < floor 0.2921, and < raw 0.3078 |
| abstain raw / calibrated | 1.00 / 0.60 (sel-acc 0.3417 @ 120 answered) |
| gate-fit (ρ=.30) | score 0.2836 / distance 0.5656 (n 200) |
| p50 / p99 latency | 0.098 ms / 0.134 ms (support 4) |
| determinism | ✓ |
| A1 / H2 | **did not run** — no wanli specialist/fusion posture exists; that cell is the rethink-side ESC re-fit (riir-rethink Issue 024) |

## Honest caveats

- **A0 reads slightly below 3-way chance (0.3333).** WANLI's gold
  distribution is skewed (test: 2,397 neutral / 1,858 entailment / 745
  contradiction) while the corpus caps flatten per-label docs to 64 each —
  the confusion is dominated by gold-neutral and gold-contradiction
  collapsing into a neutral prediction (43.0% + 42.5% of all errors). This
  is the honest modelless floor, not a defect: the ESC GOAT compares the
  specialist against exactly this bar.
- **Latency NOT QUOTABLE beyond the shape**: the box carried load 13.01
  (sibling job; the run's own PROVENANCE line says so). The accuracy cell is
  deterministic and load-insensitive (det ✓); the latency figures are
  recorded for shape only.
- The probe's leak census over the full splits
  (`slice_leak_probe.py wanli_en`): exact 4/5,000 (0.08%, zero label
  conflicts), near 1/1,999 — normalization-level twins only; the raw-row
  dedupe at fetch found none, and the run's slice-integrity audit passed
  (pool∩test hard check clean).

## Slices

test 300 `fnv1a64-290572092bd750e4` · cal 200 `fnv1a64-c6aab3376a8f57c2` ·
pool 19,800 `fnv1a64-21ccb421c47809e7` — recorded for the board-diff
"same pool?" check (Issue 058 law).
