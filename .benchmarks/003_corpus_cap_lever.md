# Bench 003 — Issue 013 lever 1: acc vs per-label corpus cap (the `--corpus-cap` sweep)

**Status:** COMPLETE 2026-09-24 · run commit `307a10b` + the instrument
(`--corpus-cap` plumb) in this commit · M3 (macOS, release, modelless lane
only `--skip-laya`) · fixtures `.raw/datasets/` ag_news + banking77 ·
DETERMINISTIC: every reading byte-reproduced on re-run (fixed fixtures, no
RNG in corpus selection — single readings are exact, not samples).

## The instrument

`harness --corpus-cap N` (0 = registry default) overrides every dataset
suite's `corpus_cap_per_label` (the per-label cap on train docs building the
engine's routing corpora). Measurement-only: a published table must state
the override or read against the registry posture, never mix the two.

## ag_news (4 labels; pool ≈ 3800 train docs post-cal-slice ≈ 950/label)

| cap | 8 | 16 | 32 | **64 (default)** | 128 | 256 | 512 |
|---|---|---|---|---|---|---|---|
| acc | .4050 | .4200 | .4275 | **.5100** | .4800 | .4675 | .4625 |
| p50 ms | 0.034 | 0.052 | 0.083 | 0.144 | 0.313 | 0.589 | 1.138 |

Peak AT the registry default. Rising 8→64 (+10.5 pp), declining after
(512 is −4.75 pp vs peak) while p50 grows ~linearly with corpus size.
Bigger corpora HURT this suite.

## banking77 (77 labels; pool ≈ 3800 train docs ≈ 49/label average, imbalanced)

| cap | 8 | 16 | 32 | 64 (default) | **128** | ≥256 (saturated) |
|---|---|---|---|---|---|---|
| acc | .3700 | .4100 | .3980 | .4380 | **.5040** | .4560 (identical 256/512/1024/4000) |
| p50 ms | 0.126 | 0.178 | 0.289 | 0.500 | 0.875 | ~0.85 |

Interior peak at 128; "use everything" (≥256, saturation) is −4.8 pp vs the
peak. The default 64 is −6.6 pp below the peak.

## Full modelless-lane row, banking77 cap 64 → 128

| metric | 64 | 128 | Δ |
|---|---|---|---|
| accuracy | .4380 | .5040 | +6.6 pp |
| macro_f1 | .1400 | .1730 | +3.3 pp |
| ECE(maxp) | .4195 | .4855 | WORSE +6.6 pp |
| Brier | .9779 | .9778 | flat |
| NLL | 4.0297 | 4.0281 | flat |
| AURC | .4280 | .3752 | BETTER −5.3 pp (lower = better ranking) |
| acc@50cov | .5840 | .6520 | +6.8 pp |
| acc@80cov | .5025 | .5725 | +7.0 pp |

## Verdict (Issue 013 lever 1)

**NOT flat, NOT monotone — and the premise inverts: the cap is a real
per-suite tuning knob, not a "bigger is better" lever.**

- ag_news: the registry default 64 IS the measured peak — no change.
- banking77: +6.6 pp available at cap 128 (with acc@50cov +6.8, AURC −5.3,
  macro-F1 +3.3) at p50 0.50→0.875 ms (harness lane only; still ~20–400×
  faster than the laya lane) and ECE +6.6 pp worse (Brier/NLL flat —
  confidence inflates while ranking improves).

**PROMOTION (banking77 64→128) DEFERRED on a protocol hole, not on the
gain:** this sweep reads the TEST split, so promoting 128 would ship a
test-set-selected hyperparameter — exactly the "gain on a selection-biased
answer" class the GOAT discipline refuses. The honest unblock path is a
cal-slice selection protocol (pick the cap on calibration-slice acc, report
test once), which needs a small runner plumb (cal-acc per cap) before any
registry constant moves. Recorded as Issue 013's lever-1 remainder.
