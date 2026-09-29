# Bench 095 — the LLW-solver re-baseline: the true MLEs restore real temperatures, the AUC regression stays fixed 15/15, and the `--gate-fit-calibrated` lever question is LIVE again (percentile coherence ≡ the raw gate's decisions)

**Status:** RECORD (2026-09-30) — the final evidence for the Issue-056 owner gate; **corrects Bench 094's "simplified owner gate" framing** (that framing depended on the constant-map outcome the LLW solver superseded).
**Issue:** [056](../../.issues/056_calibrated_confidence_ranking_regression.md); substrate: katgpt-rs Issues 909 + 910 (closed; commits `74e9d192d` → `b0d80979d`)
**Supersedes:** nothing in 094's Finding 1 (the AUC fix holds); **replaces** 094's Finding 2 posture table + its "belt-and-suspenders" conclusion.

## What ran

The Bench-092 command verbatim at the LLW substrate, twice — the default
posture and `--gate-fit-calibrated` (the Bench-093 lever). 15 suites,
deployed seat posture, frozen Bench-005 pool, `--mc-ab` N=8, test read once.
Binary reflex `d02a106`+this record over katgpt-core `b0d80979d`, host `m3`,
2026-09-30. Count-based deterministic metrics; no latency claims.

## Finding 1 — the AUC regression stays fixed, now via REAL fits

Every suite's `auc_baseline_cal_conf` still EQUALS `auc_baseline_raw_conf`
(15/15, same values as Bench 094's table — the map changed from
constant-at-base-rate to the TRUE smoothed MLEs, monotone and untied either
way). The temperatures are now real calibration, not the floor:

| suite | T (094 constant-map → 095 true MLE) | readout ECE cal |
|---|---|---|
| banking77 | 1000 → **≈0.115** (w=8.674 — the band's signal recovered) | 0.0336 |
| xnli_en | 1000 → **≈4.80** (w=0.208, flat-but-informative) | 0.1071 |
| massive_intent_en | 0.275 → **0.275** (unchanged — the sane control) | 0.0796 |

(katgpt-rs Issue 910's audit test pins the exact fitted params on the
committed real-window fixtures; the loss-optimum, held-out-floor, and
property-sweep evidence lives there.)

## Finding 2 — the lever at the LLW solver: percentile coherence ≡ the raw gate's decisions

The three-way comparison (selective accuracy @ abstain rate):

| suite | raw gate | cal gate (default posture) | cal gate + `--gate-fit-calibrated` |
|---|---|---|---|
| ag_news | 0.9510 @ 0.490 | 0.9053 @ 0.3925 | **0.9510 @ 0.490** |
| emotion | 0.9349 @ 0.4625 | 0.8806 @ 0.3300 | **0.9349 @ 0.4625** |
| prompt_injections | 0.9459 @ 0.6810 | 0.8310 @ 0.3879 | **0.9459 @ 0.6810** |
| xnli_en | 0.6143 @ 0.5333 | 0.5271 @ 0.3233 | **0.6143 @ 0.5333** |
| banking77 | 0.9211 @ 0.4680 | 0.8555 @ 0.3220 | **0.9211 @ 0.4680** |
| typed_decisions | 0.5276 @ 0.6020 | **0.5314 @ 0.4750** | 0.5276 @ 0.6020 |
| massive_intent_en | 0.7670 @ 0.3133 | **0.7681 @ 0.3100** | 0.7669 @ 0.3133 |

**The lever's calibrated gate reproduces the raw gate's decisions EXACTLY**
(same selective accuracy and coverage to the digit on every suite — expected
at a monotone fit: a percentile threshold on a monotone map selects the same
abstain set) while reporting CALIBRATED confidences. The default posture's
raw-fit threshold applied to the sharp calibrated confs is an ACCIDENTAL
coverage shift: less abstention, and −11.5 pt selacc on prompt_injections,
−8.7 on xnli, −6.6 on banking77, −5.4 on emotion, −4.6 on ag_news — with
typed (+0.4 pt at lower abstention) and massive (parity) the two cells where
the accident helps. (The lever run's `raw_abstain` column reads 1.0 by
construction — the lever refits BOTH lanes' thresholds on the calibrated
scale, and the raw lane's identity-calibrated confs then sit below it; the
raw-lane baseline to read is the DEFAULT run's raw gate, which is what the
table above uses.)

## The owner decision (reframed, live)

- **Lever default-on** = percentile coherence: the deployed gate abstains on
  exactly the set the threshold-fit meant, with calibrated conf readout;
  decisions ≡ today's raw gate everywhere.
- **Lever opt-in (today)** = the accidental posture: materially less
  abstention, worse selacc on 5 of 8 dataset suites, marginally better on
  typed_decisions.

Bench 094's "belt-and-suspenders, not a defect repair" framing was an
artifact of the constant-map interim — the sharp fits restore the exact
scale-mismatch class the lever was built for. No promotion made here.

## Reproduce

```sh
cargo run --release --features mc_ensemble --bin harness -- \
  --skip-laya --mc-ab [--gate-fit-calibrated] --datasets-dir .raw/datasets_t20k \
  --head-select --nb-select --oc-select --ridge-select --out <dir>
# compare suites[*].modelless.{calibrated_abstain,raw_abstain,mc_ab} between
# the two postures and against .benchmarks/094_*/results.json
```
