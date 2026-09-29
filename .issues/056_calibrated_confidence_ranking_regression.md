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

Discriminating evidence to collect: per-question scatter of (raw, cal)
pairs ordered by raw — monotonicity violations counted, not eyeballed; the
calibrator's window configuration per suite; whether the identical-column
suites (massive) are no-reorder or never-fitted.

## Reproduce

```sh
cargo run --release --features mc_ensemble --bin harness -- --skip-laya --mc-ab \
  --datasets-dir .raw/datasets_t20k --head-select --nb-select --oc-select --ridge-select \
  --out /tmp/mc_ab_repro
# read suites[*].modelless.mc_ab.auc_baseline_{cal_conf,raw_conf} vs hard.accuracy
```

## References

- Bench 092 (`.benchmarks/092_distributional_layer_poc/`) — the measuring
  instrument + the founding observation (the verdict round-1 review caught
  the column contradiction; the carve-out was reframed accordingly).
- Issue 055 (closed, HISTORY 2026-09-30) — the parent PoC.
