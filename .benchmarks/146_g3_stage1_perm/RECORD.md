# Bench 146 — plan 436 G3 Stage 1: the round-2 option-shuffled re-read (perm-probe, frozen cell)

**Date:** 2026-10-15T16:41:38Z · **Host:** 4090-windows · **Reflex sha:** `da447b5` ·
**Dataset cell:** `.raw/datasets` (the default dir — the same cell Bench 112 read)

The plan-436 pre-registered Stage-1 probe (thresholds fixed BEFORE any probe ran;
verdict ping-pong round 2 AGREE, recorded in `../riir-train/.plans/436_laya_s1mb_round2_replay.md`):
the round-2 checkpoint re-read under K = 5 deterministic option orderings on the frozen
typed test cell.

## Posture + gates that ran first

- **G5 parity GREEN pre-run** (the publish law): `LAYA_DEVICE=cuda cargo test --release
  --features laya-riir-cuda --test laya_riir_parity` — 2/2 checkpoints, 7.72 s.
- **Weights = the round-2 artifact, UNPINNED research lane**: `LAYA_WEIGHTS_DIR` →
  `riir-train/.raw/plan435/laya_weights_r2` + `LAYA_ALLOW_UNPINNED_WEIGHTS=1` (the
  loud-disclosure downgrade; every file mismatches the serving pins by construction —
  this is a different training run, plan 436's round-2 checkpoint, blake3-unpinned).
- Box state at launch: GPU idle (19 % transient desktop util, 611 MiB used, 23.5 GiB
  free), no trainer/cargo/python consumers — the 623 pre-arm runbook check.

```sh
LAYA_WEIGHTS_DIR="E:/git/riir-train/.raw/plan435/laya_weights_r2" \
LAYA_ALLOW_UNPINNED_WEIGHTS=1 LAYA_DEVICE=cuda REFLEX_BENCH_HOST=4090-windows \
  target/release/harness --perm-probe --suites typed_decisions \
  --perm-max-cases 400 --out .benchmarks/146_g3_stage1_perm/laya_typed_round2_cuda
```

## The Stage-1 metric (median pooled acc across the K orderings)

Per-ordering pooled accuracy over the cell's 600 choice questions (400 cases — the full
typed test cell's choice-carrying questions; computed from `perm_probe.json`
`pick_labels[k] == gold`):

| lane | ordering accs (K=5) | median | identity |
|---|---|---|---|
| laya/typed · round-2 | 0.7500 / 0.7450 / 0.7433 / 0.7467 / 0.7483 | **0.7467** | 0.7500 |
| modelless (control) | 0.1400 / 0.1333 / 0.1517 / 0.1333 / 0.1367 | 0.1367 | 0.1400 |

## The pre-registered gates

| gate | threshold | read | verdict |
|---|---|---|---|
| **PROMOTE to serving behind the kill-switch** | median ≥ 0.7445 (the round-0 incumbent) | **0.7467** | **PASS** (+0.22 pt) |
| **publish the "+1.05 vs agentjev" board row** | median ≥ 0.7715 (the agentjev bar) | 0.7467 | **FAIL** — the row reads "parity within order noise"; NO reflex-site republish carries the claim |

Promotion is NOT claimed here: Stage 2 (the 4 LOWO arms, min held-out pooled-acc ≥ 0.60)
must also pass — either stage fails → the artifact stays record-only at 0.7825
(plan 436's law). Stage 2 chain launched the same night (GPU-exclusive, ~30 h).

## Disclosures

- **Comparability caveat:** the 0.7445 incumbent is the round-0 FULL-CELL read
  (2,000 questions incl. noul/score); this probe's cell is the 600 choice questions.
  The thresholds were pre-registered naming THIS instrument (the Bench-128 probe) —
  the gates are judged as pre-registered, the cell difference recorded here.
- **Swing (disclosed, not gated):** laya/typed median top-pick swing 4.83 pt · max
  25.21 pt · flip rate 0.112 (67/600) — RED under the ≤ 2 pt comparison-lane gate, the
  standing position-sensitivity of the laya/typed lane (bench 128/131/140 lineage,
  known since before the round-2 training; the ≤ 2 pt bar is the comparison-lane
  gate, never a Stage-1 gate). The whole reason the staged probes exist.
- **Modelless control PASS:** median swing 0.00 pt; 288 flips, every one an exact
  top-2 tie under the tie-only contract — the issue-079 content-bound fix holds at
  this sha.
- The canary cell: modelless `beta|beta|beta|beta|beta` (0.0 swing); laya
  `alpha|alpha|alpha|alpha|alpha` (3.1 pt swing, no flip) — liveness only.
- Determinism note: `p_ref` quantized at 4 decimals in the record (the bench-140 law).

## Files

- `laya_typed_round2_cuda/perm_probe.json` + `PERM_PROBE.md` (the instrument output)
- Computed verdict: `../riir-train/.raw/stage1_verdict.txt` (the per-ordering acc calc)
