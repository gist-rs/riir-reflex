# Bench 094 — the substrate-repaired re-baseline: the Issue-056 calibrated-key regression is GONE on all 15 suites, and the gate abstain distortion collapsed WITHOUT the lever

**Status:** RECORD (2026-09-30) — **SUPERSEDED same-day by [Bench 095](../095_llw_solver_rebaseline/README.md) in its Finding-2 posture table and owner-gate framing** (the constant-map fits below were the three-candidate-fallback interim; the LLW solver's true MLEs restore real temperatures and with them the scale-mismatch class the lever was built for — 094's "belt-and-suspenders" conclusion no longer holds). Finding 1 (the AUC fix, 15/15) stands unchanged. Evidence for the Issue-056 owner gate at its interim state; no promotion made here.
**Issue:** [056](../../.issues/056_calibrated_confidence_ranking_regression.md) (repairs 2+3 landed substrate-side: katgpt-rs Issue 909, closed, `74e9d192d`)
**Supersedes:** Bench 092's `auc_baseline_cal_conf` columns + Issue 056's severity-elevation abstain table (both recorded against the STALLED substrate fits).

## What ran

The Bench-092 command verbatim at the repaired substrate — 15-suite harness,
deployed seat posture (`--head-select --nb-select --oc-select --ridge-select`),
frozen Bench-005 pool (`.raw/datasets_t20k`), `--mc-ab` N=8, cal-side selection
p ∈ {0.05, 0.10, 0.20, 0.30} / λ ∈ {0.2, 0.4, 0.8}, test read once. Binary
reflex `00f5316` over katgpt-core `74e9d192d` (the `fit_window` loss-argmin
fallback + the output-side saturation guard), host `m3`, 2026-09-30, box under
sibling-arena load — **all metrics here are count-based and deterministic
(AUC, abstain rates, accuracy); no latency claims**. Full tables:
`results.json` (per-suite `modelless.mc_ab`).

## Finding 1 — the ranking regression is GONE, 15 of 15

`auc_baseline_cal_conf` now EQUALS `auc_baseline_raw_conf` on every suite
(the monotone map no longer loses information in f32):

| suite | AUC cal (092 → 094) | AUC raw | recovered to |
|---|---|---|---|
| banking77 | 0.8125 → **0.9350** | 0.9350 | raw (exactly) |
| emotion | 0.8967 → **0.9604** | 0.9604 | raw |
| xnli_en | 0.6161 → **0.6927** | 0.6927 | raw |
| prompt_injections | 0.8538 → **0.9051** | 0.9051 | raw |
| sst5 | 0.4014 → **0.4538** | 0.4538 | raw |
| ag_news | 0.9335 → **0.9494** | 0.9494 | raw |
| typed_decisions | 0.5699 → **0.5872** | 0.5871 | raw (1-pair float-cast unit) |
| massive_intent_en | 0.9447 → **0.9447** | 0.9447 | unchanged — the sane control's Newton fit survives verbatim |
| code_fixtures + 6 families | — | — | cal == raw (thin/NoClaim windows, as before) |

Mechanism (katgpt-rs Issue 909, measured on the committed real-window
fixtures): the seven degraded suites' fits were DEGENERATE-INIT STALLS of the
identity-init Newton (near-singular first Hessian on a narrow-z band →
saturated corner → Hessian collapse → break) sitting 10.8×/23× above the
achievable loss — not the smoothed MLE. `fit_window` replaces them with the
loss-argmin candidate: the near-constant base-rate map (T = 1/W_MIN = 1000),
which is monotone, UNTIED in f32, and ECE-honest. The u_pair MC-layer columns
are byte-identical to Bench 092 (seeded, untouched) — **the Bench-092 null
verdict (u_pair loses to the fused gate at matched coverage, 8 of 8) stands
unchanged**; only the baseline key was repaired.

## Finding 2 — the GATE abstain distortion collapsed at the DEFAULT posture

The Issue-056 severity table (thresholds fit on raw, applied to the saturated
calibrated scale — 90–99% abstain on the saturate-at-0 suites, the
Issue-042 96.7% typed escalation root cause) — at the repaired substrate,
WITHOUT `--gate-fit-calibrated`:

| suite | cal abstain (was → 094) | raw abstain |
|---|---|---|
| sst5 | 0.9883 → **0.3150** | 0.5317 |
| emotion | 0.9725 → **0.3300** | 0.4625 |
| typed_decisions | 0.9670 → **0.4750** | 0.6020 |
| ag_news | 0.9300 → **0.3925** | 0.4900 |
| xnli_en | 0.9333 → **0.3233** | 0.5333 |
| prompt_injections | 0.9052 → **0.3879** | 0.6810 |
| banking77 | 0.3220 → **0.3220** | 0.4680 (unchanged — was already disarmed) |
| massive_intent_en | 0.3100 → **0.3100** | 0.3133 (unchanged — sane control) |

Every suite now lands in the 0.31–0.48 band (vs raw 0.46–0.68): the honest
constant maps place calibrated confs at sane levels the raw-fit thresholds
interpret reasonably. **The owner-gate question simplifies**: the substrate
repair alone restores honest gates at the default posture;
`--gate-fit-calibrated` (Bench 093) remains available as the explicit
scale-coherence posture — its promotion is now a belt-and-suspenders decision,
not a defect repair. The readout ECE on the previously-stalled suites lands
0.016–0.11 (banking77 0.0162, ag_news 0.0277, prompt 0.0320, xnli 0.0666).

## Reproduce

```sh
cargo run --release --features mc_ensemble --bin harness -- \
  --skip-laya --mc-ab --datasets-dir .raw/datasets_t20k \
  --head-select --nb-select --oc-select --ridge-select \
  --out <dir>
# compare suites[*].modelless.mc_ab.auc_baseline_{cal_conf,raw_conf} +
# .calibrated_abstain.abstain_rate against .benchmarks/092_*/results.json
```
