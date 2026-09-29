# Issue 056 — the calibrated readout confidence ranks WORSE than raw as a rejection key (up to −0.12 AUC; banking77 below random)

**Status:** OPEN (owner gate pending — SIMPLIFIED by Bench 094) — filed from Bench 092 (`--mc-ab`'s baseline columns), 2026-09-30; landed with the measuring arm at `460f5f1`, the discriminating evidence at `788cf5b`, the severity elevation at `2fb2ae4`, repair direction 0 (`--gate-fit-calibrated`, Bench 093) at `2641477`, and **repairs 2 + 3 substrate-side in katgpt-rs Issue 909 (closed, `74e9d192d`) + the full 15-suite re-baseline [Bench 094](../.benchmarks/094_substrate_repaired_rebaseline/README.md): the AUC regression is GONE 15/15 AND the gate abstain distortion collapsed at the DEFAULT posture (0.31–0.48 band, was 0.90–0.99) — the remaining owner decision is whether `--gate-fit-calibrated` is promoted as belt-and-suspenders scale coherence or stays opt-in; the substrate defect itself is fully repaired.**

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

- ⚠ **Superseded by the severity-elevation section below:** the first pass
  held the gate's threshold-wise abstain decisions unaffected ("pointwise,
  order-free") — the second pass REFUTED that: the thresholds are fit on
  the raw scale and applied to the saturated calibrated scale, so the
  deployed abstain IS distorted (90–99% on the saturate-at-0 suites,
  score-axis disarm on banking77). See below.
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

## SEVERITY ELEVATION (2026-09-30, second pass): the deployed GATE is distorted, not just ranking consumers — and this IS the Issue-042 over-escalation class

> **⚠ SUPERSEDED by the substrate repair + Bench 094 (same day):** the
> table below records the DISTORTION against the stalled substrate fits.
> With katgpt-rs Issue 909 landed, the honest constant maps place the
> calibrated confs at sane levels and the deployed abstain collapsed to the
> 0.31–0.48 band at the DEFAULT posture (see
> [Bench 094](../.benchmarks/094_substrate_repaired_rebaseline/README.md)
> Finding 2). The table stays as the defect record.

The gate's score axis thresholds are fit on the **RAW** scale but applied
 to the **CALIBRATED** conf: the threshold-fit probe engine is built fresh
 (`default_cfg`) with an IDENTITY calibrator, so `score_obs` reads raw
 confidences (`runner.rs` gate-fit block, `slot.confidence` of the probe);
at test time `solve` computes `conf = calibrator.apply(raw)` and compares
 THAT against the fitted threshold. Under near-identity calibration the
 mismatch is benign; under saturation it is catastrophic, in both
directions:

| suite | cal abstain (deployed) | raw abstain | distortion |
|---|---|---|---|
| sst5 | **0.9883** | 0.5317 | saturate-at-0 → abstain ~everything |
| emotion | **0.9725** | 0.4625 | same |
| typed_decisions | **0.9670** | 0.6020 | same — **the 96.7% that IS Issue 042's recorded typed escalation figure** |
| ag_news | **0.9300** | 0.4900 | same (sel-acc 1.0 on 7% kept — looks perfect while refusing 93% of traffic) |
| xnli_en | **0.9333** | 0.5333 | same |
| prompt_injections | 0.9052 | 0.6810 | same |
| banking77 | **0.3220** | 0.4680 | saturate-at-1.0 → score axis DISARMED (the residual abstain is the distance axis alone) |
| massive_intent_en | 0.3100 | 0.3133 | sane fit — the control |

**The unification:** Issue 042's cascade work measured "the shipped fused
posture's typed 96.7% escalation fails this window" and built levers
around it (gate-fit-selection, distance-only) — treating the symptom
(over-abstention) without the root cause (calibration saturation + the
fit-scale/apply-scale mismatch). The distance-only lever "worked"
mechanically because it ZEROED the mis-matched score axis entirely. This
issue owns the root cause; the 042 levers are the palliative record.

**Repair directions now include (in deliberation order):**
0. **Fit-scale/apply-scale coherence (reflex-side, before any substrate
   change):** the gate-fit probe must observe on the SAME scale the gate
   applies — either fit the threshold on the CALIBRATED probe confs (the
   probe observes the cal pairs then reads calibrated confidences), or
   apply the raw conf at the gate and calibrate only the REPORTED
   confidence. Either closes the mismatch class without touching the
   substrate. **→ LANDED 2026-09-30 as `--gate-fit-calibrated` (Bench 093,
   opt-in default-off):** the former form — the probe's own per-question
   pairs reconstruct the deployed calibrator exactly; the six saturate-at-0
   suites' calibrated abstain collapses 90.5–98.8% → 31.5–47.5% (honest
   inert score axis), banking77 unchanged (was already disarmed), massive
   (sane-fit control) unchanged within 0.3 pt, forced accuracy unchanged
   everywhere. Promotion is owner-gated WITH the substrate guard (the two
   repairs compose; this bench is the evidence).
1. **Consumer-side (ranking): rank on the RAW readout conf** — unchanged
   from below.
2. **Substrate-side saturation guard** (katgpt-rs `sigmoid_calibration`)
   — unchanged from below; note it fixes the saturation but NOT the
   scale-mismatch by itself (an unsaturated but non-identity calibration
   still shifts the threshold's meaning).
3. The Newton early-break audit — unchanged.

## SUBSTRATE REPAIRS LANDED (2026-09-30, katgpt-rs Issue 909 — closed; HISTORY.md there): the audit verdict, the fallback, the guard, and the end-to-end fix

The two substrate repairs landed in katgpt-rs `sigmoid_calibration`, with
the REAL production cal windows (banking77 / xnli_en / massive_intent_en,
200 pairs each — dumped via the new `RIIR_DEBUG_CAL_WINDOW` env instrument
in `runner.rs`, same posture) committed as replay fixtures. The replay
reproduces the measured fits EXACTLY (w=1.2337 ↔ T=0.811; w=556468 ↔
T≈1.8e-6; w=3.6342 ↔ T=0.275).

**The audit verdict refuted the recorded early-break theory.** An f64
mirror of the production solve stalls at the SAME extreme point with the
SAME loss (gap 0.0000 on all three windows) — not an f32-precision defect.
The true mechanism: on a narrow-z window the FIRST Newton Hessian is
near-singular (det ≈ (Σr)²·var z), the first step lands the iterate in a
saturated corner where every `r = p(1−p)` collapses, and the loop breaks
at a point **10.8×** (banking77: loss 1051.02 vs 97.29) and **23×**
(xnli_en: 3151.38 vs 135.35) above a constant-at-base-rate map. The
extreme (w, c) values were never the smoothed MLE — a degenerate-init
stall.

**The landed repairs** (both in `refit`, off-hot-path, deterministic):
1. `fit_window` — the refit is the loss-argmin over {Newton-from-identity,
   near-constant base-rate `(W_MIN, c*)`, identity}. Sane windows keep the
   Newton result verbatim (massive: bit-identical, T=0.275); degenerate
   windows get the loss-optimal constant-at-base-rate map (T = 1/W_MIN =
   1000 — the calibrated conf lands at the window's base rate: ECE-honest,
   ranking-preserving).
2. `window_keeps_resolution` — a fit whose mapped window collapses below
   0.5× the raw window's distinct f32 values is refused to identity (the
   output-side analogue of `W_MIN`): monotone in the reals, tied in the
   floats breaks G3's ranking promise; the guard enforces it in f32
   arithmetic.

**End-to-end confirmation** (reflex rebuilt against the repaired
substrate, the deployed posture, same command as below):

| suite | T (was → now) | auc_cal (was → now) | auc_raw | verdict |
|---|---|---|---|---|
| banking77 | 0.811 → **1000** (constant 0.81) | 0.8125 → **0.93500** | 0.93496 | **regression GONE** (the 3.5e-5 residual = one pair-unit of the engine's pre-existing f64→f32 conf cast) |
| xnli_en | 1.8e-6 → **1000** (constant 0.59) | 0.6161 → **0.69271** | 0.69271 | **regression GONE** (exactly equal) |

Forced accuracy unchanged on both (the calibrator never touches picks).
**Bench 092's `auc_baseline_cal_conf` columns are superseded at substrate
HEAD** — the cal ranking key now equals the raw one wherever the fit was a
stall, and remains the (sane) fit's own order elsewhere (massive: T=0.275
unchanged, 300/300 distinct).

**What remains (the owner gate — simplified by Bench 094):** Bench 094
measured that the substrate repair ALONE restores honest gates at the
default posture (the abstain distortion collapsed 0.90–0.99 → 0.31–0.48
without the lever), so `--gate-fit-calibrated`'s promotion is now a
belt-and-suspenders scale-coherence decision, not a defect repair. The
consumer-side ranking rule (repair 1, "rank on raw") is MOOT at substrate
HEAD — the calibrated key no longer degrades anywhere — but stays the
conservative rule for any consumer pinned to an older substrate.

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

# The EXACT cal-window pairs the deployed calibrator fit on (the katgpt-rs
# Issue-909 fixture source — same pairs → same FIFO window → same params):
RIIR_DEBUG_CAL_WINDOW=1 cargo run --release --bin harness -- \
  --skip-laya --datasets-dir .raw/datasets_t20k \
  --head-select --nb-select --oc-select --ridge-select --suites banking77 --out /tmp/cal_window
# [cal-window] lines: header (suite + n) then one `conf correct` pair per line
```

## References

- Bench 092 (`.benchmarks/092_distributional_layer_poc/`) — the measuring
  instrument + the founding observation (the verdict round-1 review caught
  the column contradiction; the carve-out was reframed accordingly).
- Issue 055 (closed, HISTORY 2026-09-30) — the parent PoC.
