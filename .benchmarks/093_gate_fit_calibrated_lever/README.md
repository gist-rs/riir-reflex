# Bench 093 — `--gate-fit-calibrated` (Issue 056 repair direction 0): fit the score-axis threshold on the scale the gate applies

**Status:** RECORD (2026-09-30) — **the lever works as designed; stays opt-in** (promotion is owner-gated alongside the substrate saturation guard — the two repairs compose)
**Issue:** `056` (severity-elevation section — the fit-on-raw / apply-on-calibrated scale mismatch)
**Baseline:** Bench 092's run (identical posture, lever off)

## What ran

The same 15-suite deployed-posture run as Bench 092
(`--head-select --nb-select --oc-select --ridge-select`, frozen t20k pool,
`--mc-ab` on) with `--gate-fit-calibrated` added: the gate-fit probe's own
per-question (conf, correct) pairs — byte-identical to the deployed
calibrator's `cal_pairs` in the default posture — are observed into a
`SigmoidGateCalibrator` at the engine's `(cal_capacity, cal_min_obs)`,
refit, and the score-axis observations are mapped through `apply` BEFORE
the ρ=30 percentile fit. Fit scale == apply scale. Binary one commit past
`2fb2ae4`, host `m3`.

## The A/B (lever OFF = Bench 092's row)

| suite | abst(cal) OFF → ON | sel-acc OFF → ON | threshold OFF → ON | forced acc |
|---|---|---|---|---|
| typed_decisions | 0.9670 → **0.4750** | 0.8636 → 0.5314 | 0.0165 → 0.0000 | unchanged |
| ag_news | 0.9300 → **0.3925** | 1.0000* → 0.9053 | 0.0111 → 0.0000 | unchanged |
| emotion | 0.9725 → **0.3300** | 1.0000* → 0.8806 | 0.0080 → 0.0000 | unchanged |
| sst5 | 0.9883 → **0.3150** | 0.5714 → 0.3893 | 0.0047 → 0.0000 | unchanged |
| prompt_injections | 0.9052 → **0.3879** | 1.0000* → 0.8310 | 0.0675 → 0.0000 | unchanged |
| xnli_en | 0.9333 → **0.3233** | 0.9000 → 0.5271 | 0.0029 → 0.0000 | unchanged |
| massive_intent_en (control) | 0.3100 → 0.3133 | 0.7681 → 0.7670 | 0.0581 → 0.2008 | unchanged |
| banking77 | 0.3220 → 0.3220 | 0.8555 → 0.8555 | 0.0205 → 1.0000 | unchanged |

\* the fake cells: selective accuracy 1.0000 on 3–7% kept traffic — the
saturated gate refusing nearly everything and being right about the sliver
it served.

## Reading (honest)

- **The six saturate-at-0 suites**: the calibrated abstain collapses from
  90.5–98.8% to 31.5–47.5% — the score axis becomes HONESTLY inert
  (threshold 0.0000 = the saturated fit-slice floor) and abstain rides the
  distance axis + the exact-0.0 tie band. The published "selective
  accuracy" cells stop being the 3%-kept artifact.
- **banking77 (saturate-at-1)**: threshold 1.0 formally disarms the score
  axis — which was ALREADY the deployed reality (conf 1.0 never abstains);
  the row is unchanged. A saturated scale carries no information and NO
  threshold on it recovers any — restoring the signal is the substrate
  saturation guard's job (Issue 056 repair 2).
- **massive (sane fit) — the control**: the lever is sound where
  calibration is sound (real threshold 0.2008, abstain within 0.3 pt).
- **Forced accuracy unchanged everywhere** (the gate never changes picks).

## Verdict + posture

The lever closes the scale-mismatch class exactly as designed and composes
with the future substrate guard (which would restore a live score axis on
the saturated suites — at which point this lever's calibrated thresholds
become informative rather than degenerate). **Stays opt-in, default off**:
promotion changes the deployed abstain semantics on six suites (a
published-row change) and belongs with the substrate-guard decision — the
owner's call, with this bench as the evidence.

The seat stays on the shipped posture (`fit_posture` pins all three gate
levers off — the arena's published face is unchanged by construction).
