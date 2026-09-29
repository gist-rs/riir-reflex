# Issue 056 — the calibrated readout confidence ranks WORSE than raw as a rejection key (up to −0.12 AUC; banking77 below random)

**Status:** OPEN — filed from Bench 092 (`--mc-ab`'s baseline columns), 2026-09-30; landed with the measuring arm at `460f5f1`

## The finding

Bench 092's A/B record carries both baseline ranking keys per suite — the
RAW readout confidence (`raw_eval.confs`) and the DEPLOYED calibrated one
(`cal_eval_test.confs`, the fitted engine's readout). A monotone calibrator
cannot change a ranking; the measured columns say the deployed one DOES,
always in the harmful direction:

| suite | AUC cal | AUC raw | cal − raw | suite acc |
|---|---|---|---|---|
| banking77 | 0.8125 | **0.9350** | **−0.1225** | 0.8260 |
| xnli_en | 0.6161 | 0.6927 | −0.0766 | 0.5233 |
| emotion | 0.8967 | 0.9604 | −0.0637 | 0.8850 |
| prompt_injections | 0.8538 | 0.9051 | −0.0513 | 0.7672 |
| sst5 | 0.4014 | 0.4538 | −0.0524 | 0.3967 |
| typed_decisions | 0.5699 | 0.5871 | −0.0172 | 0.4655 |
| ag_news | 0.9335 | 0.9494 | −0.0159 | 0.8825 |
| massive_intent_en | 0.9447 | 0.9447 | 0 (identical order) | 0.7800 |
| code_fixtures + 6 families | identical | identical | 0 (calibrator never fitted — thin cal / NoClaim) | |

**7 of 9 dataset suites: the calibrated key ranks strictly worse than raw,
never better.** On banking77 the calibrated key's mean-prefix-accuracy AUC
(0.8125) sits BELOW the suite's own forced accuracy (0.8260) — it ranks
worse than a random ordering of the same answers.

## Why this matters (and what it does NOT mean)

- The deployed fused gate consumes the calibrated conf **threshold-wise**
  (abstain decisions at `score_threshold`) — a pointwise comparison, where
  order does not apply. The abstain/selective-accuracy columns are NOT
  implicated by this finding.
- Every **ranking** consumer inherits the degradation: any AURC-style
  readout keyed on the calibrated conf, escalation ordering, and Bench
  092's own baseline arm (the `--mc-ab` record's `auc_baseline_cal_conf` —
  which is how this was found: the MC layer "beat" the calibrated key on
  banking77 while the raw key beat both).

## The two candidate causes (unadjudicated — this issue's work)

1. **Non-monotone calibrator by construction.** The calibrator is windowed
   (`cal_capacity` / `cal_min_obs`); a windowed empirical map need not be
   monotone in the raw conf. If this is the cause, the finding is a
   documented tradeoff (calibration-for-ECE vs ranking) and the repair is a
   consumer-side rule: *rank on raw, threshold on calibrated*.
2. **A mapping defect** (the index-space class Bench 092 already caught
   once in the same arm — the noul flip). Same-sign symptoms: reordering
   that tracks suite shape; identical columns where the calibrator is the
   identity.

## DISCRIMINATING EVIDENCE COLLECTED (2026-09-30, `RIIR_DEBUG_CAL_RANK=1` — both causes REFUTED; the mechanism is f32 SATURATION under a monotone-but-steep refit)

The pair-dump instrument (env-gated, in `mc_ab.rs`, the `RIIR_DEBUG_RIDGE`
precedent) dumped per-question (raw, calibrated) test confidences + the
fitted temperature at the deployed posture:

| suite | T = 1/w | distinct cal / n | saturation | monotonicity violations | cal−raw AUC |
|---|---|---|---|---|---|
| banking77 | 0.811 (w≈1.2) | **1** / 500 | 100% @ exactly 1.0 | **0** | −0.123 |
| xnli_en | **1.8e-6** (w≈5.6e5) | **2** / 300 | 93% @ 0.0 | **0** | −0.077 |
| ag_news | 6.6e-5 (w≈1.5e4) | **3** / 400 | 86% @ 0.0 | **0** | −0.016 |
| massive_intent_en | 0.275 (w≈3.6) | 300 / 300 | none | **0** | 0 (identical order) |

**Both candidate causes are refuted; the mechanism is a third thing:**

- **The monotonicity guard HOLDS** — zero violations on any suite. Cause 1
  (non-monotone windowed map) is wrong for THIS calibrator (it is a Platt
  refit with a `w > 0` projection, not a windowed empirical map). Cause 2
  (index bug) is wrong — the raw ordering is never inverted.
- **The kill is f32 SATURATION.** The refit is monotone but STEEP, and the
  wide-label confidence bands are narrow and LOW-valued (banking77's
  77-way maxprob readouts span [0.016, 0.044] — the numbers are small
  because K is large, not because the engine is unconfident). A steep map
  over a narrow band lands every test conf in the same f32 bucket — 1
  distinct calibrated value on banking77 (all exactly 1.0), 2 on xnli_en,
  3 on ag_news. Order is preserved over the reals and DESTROYED in the
  floats: ranking consumers tie-break to index order, which is why
  banking77's calibrated AUC (0.8125) sits below its own accuracy (0.826)
  — the worse-than-random signature is a fully-tied key.
- **massive_intent_en (K=60, also wide) escapes** — its raw band is wider
  ([0.057, 0.251]) and its fit is sane (T=0.275, 300 distinct values,
  identical order to raw). Saturation is band-narrowness × fit-steepness,
  not label-count alone.
- **A second, separate suspect surfaced: the fit parameters themselves.**
  xnli_en's w ≈ 5.6e5 on a ~52%-accurate, non-separable window is not a
  plausible smoothed-target MLE — the Newton loop's early-break
  (`det.abs() < f32::EPSILON` on a narrow-z window) can stop at a
  non-stationary iterate (runaway w). banking77's implied intercept
  (c ≳ 21 — derived from the all-@1.0 saturation bound at w≈1.2) is
  likewise suspicious. Whether each extreme is a legitimate smoothed-MLE
  or an early-broken iteration is one deterministic substrate-side check
  (same window → same params → re-solve at f64) — the recorded follow-up.

**Repair directions (in deliberation order):**
1. **Consumer-side (immediate, modelless): rank on the RAW readout conf;
   threshold on the calibrated one.** The harness's own AURC column already
   reads maxp (unaffected); the mc-ab baseline and any other ranking
   consumer should follow. Zero substrate change.
2. **Substrate-side (katgpt-rs `sigmoid_calibration`): a saturation guard**
   — the W_MIN analogy for the OUTPUT side: reject/attenuate a refit whose
   mapped window span collapses below f32 resolution (e.g. fewer distinct
   mapped values than a resolution floor over the window), falling back to
   identity. Preserves the calibration-for-ECE purpose on sane windows,
   refuses the tie-collapse class.
3. The Newton early-break audit (f64 re-solve comparison) before trusting
   any extreme (w, c).

## Reproduce

```sh
# The A/B record (the finding):
cargo run --release --features mc_ensemble --bin harness -- --skip-laya --mc-ab \
  --datasets-dir .raw/datasets_t20k --head-select --nb-select --oc-select --ridge-select \
  --out /tmp/mc_ab_repro
# read suites[*].modelless.mc_ab.auc_baseline_{cal_conf,raw_conf} vs hard.accuracy

# The mechanism probe (the pair dump + fitted temperature):
RIIR_DEBUG_CAL_RANK=1 cargo run --release --features mc_ensemble --bin harness -- \
  --skip-laya --mc-ab --datasets-dir .raw/datasets_t20k \
  --head-select --nb-select --oc-select --ridge-select --suites banking77 --out /tmp/cal_probe
# [cal-rank] lines: one per question (raw, cal) + the calibration header
```

## References

- Bench 092 (`.benchmarks/092_distributional_layer_poc/`) — the measuring
  instrument + the founding observation (the verdict round-1 review caught
  the column contradiction; the carve-out was reframed accordingly).
- Issue 055 (closed, HISTORY 2026-09-30) — the parent PoC.
