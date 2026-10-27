# Bench 165 — Issue 100 inductive-bias probes: the robustness axis is UNIFORM (the encoder retains less above-floor skill under word dropout on every suite and every rung), the label-irregularity axis is FLAT — neither separates the board's winners; T3 screen NOT written (recorded negative)

**Status:** RECORD — executed 2026-10-27 (amo, M5 Ultra), commit `bb39c3f` (probe code `936b014` + T2 fix `bb39c3f`). Closes reflex Issue 100.

## Posture + provenance

```
CARGO_TARGET_DIR=/tmp/reflex-100 LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --bin harness -- \
  --suites banking77,emotion,ag_news,xnli_en --nb-select --oc-select --ridge-select --drafter-fix per_byte \
  --robust-ab --label-smooth-ab --out <dir>
```

- `PROVENANCE: power=AC Power load=5.03 swap=4985.94M canary=skipped powermode=0(auto)` (`scripts/bench_preflight.sh`). Box shared with the elephant window-3 capture tail (Python MPS serve). Every number below is an ACCURACY; no latency is quoted.
- Modelless at the site posture (Bench 131's flags) reproduces Bench 131 digit-for-digit: ag_news 0.8825, emotion 0.8850, xnli_en 0.5233, banking77 0.8420. ⛔ A first run WITHOUT the posture flags read ag_news modelless 0.4050 — discarded (wrong floor); the posture is part of the claim.
- Encoder = laya `english` checkpoint (the only locally runnable encoder; the hosted lanes were not armed). Suite coverage meets the issue's rule: DEAD-encoder suites banking77 + emotion, seated-encoder suites ag_news + xnli_en.
- Artifacts: `165_inductive_bias_probes/results.json` (`suites[].inductive_bias`), `TABLES.md`.

## T1 — `--robust-ab` (word dropout on every string leaf of the case state; identical corrupted text for both lanes)

Raw ΔΔ = (enc_corr − enc_clean) − (ml_corr − ml_clean), paired over (case_id, q_idx), 95% normal CI. `ret` = above-floor retention (corr − f)/(clean − f), f = the suite's modal floor (`SuiteResult.modal_floor.acc`).

| suite | f | p | ml clean → ret | enc clean → ret | ret gap (enc−ml) | raw ΔΔ [95% CI] |
|---|---|---|---|---|---|---|
| ag_news | 0.263 | 0.1 | 0.8825 → 1.004 | 0.9500 → 0.993 | −0.011 | −0.0075 [−0.0330, +0.0180] |
| ag_news | | 0.2 | → 1.020 | → 0.985 | −0.035 | −0.0225 [−0.0469, +0.0019] |
| ag_news | | 0.3 | → 0.972 | → 0.949 | −0.023 | −0.0175 [−0.0489, +0.0139] |
| emotion | 0.312 | 0.1 | 0.8850 → 0.917 | 0.5950 → 0.903 | −0.014 | +0.0200 [−0.0146, +0.0546] |
| emotion | | 0.2 | → 0.843 | → 0.832 | −0.011 | +0.0425 [−0.0036, +0.0886] |
| emotion | | 0.3 | → 0.747 | → 0.726 | −0.021 | **+0.0675 [+0.0167, +0.1183]** |
| xnli_en | 0.333 | 0.1 | 0.5233 → 1.035 | 0.8600 → 0.867 | −0.168 | **−0.0767 [−0.1303, −0.0230]** |
| xnli_en | | 0.2 | → 0.825 | → 0.747 | −0.078 | **−0.1000 [−0.1638, −0.0362]** |
| xnli_en | | 0.3 | → 0.719 | → 0.462 | −0.257 | **−0.2300 [−0.3044, −0.1556]** |
| banking77 | 0.046 | 0.1 | 0.8420 → 0.935 | 0.4220 → 0.910 | −0.025 | +0.0180 [−0.0150, +0.0510] |
| banking77 | | 0.2 | → 0.854 | → 0.835 | −0.019 | **+0.0540 [+0.0087, +0.0993]** |
| banking77 | | 0.3 | → 0.771 | → 0.596 | −0.176 | +0.0300 [−0.0172, +0.0772] |

**Reading.** (1) The raw ΔΔ sign is a HEADROOM artifact: in all four suites the lane with the higher clean accuracy loses more raw points (emotion/banking77 → modelless loses more; xnli_en/ag_news → laya loses more). Read raw, the probe just re-states who wins — circular as a screen. (2) Normalised above the modal floor, the gap is **negative in 12 of 12 cells**: per unit of above-chance skill, the hashed-bag kNN floor is MORE robust to uninformative-token removal than the encoder on every suite — DEAD and seated alike. This is the finding-2 analog, and it holds; but it holds UNIFORMLY, and the magnitude does not order the winners (encoder-wins ag_news −0.011..−0.035 sits inside modelless-wins emotion −0.011..−0.021 / banking77 −0.019..−0.176). ⚠ The ret column is a point estimate (no CI on the ratio); the separation verdict does not need one — the sign is uniform and the magnitudes interleave.

## T2 — `--label-smooth-ab` (working-set labels graph-smoothed toward the k=10 hashed-bag neighbours, weights sigmoid(20·(cos−0.30)), own label wins ties; full pool kept, only working-set labels change)

| suite | α 0.60 relabel / Δ | α 0.75 relabel / Δ [CI] | α 0.90 relabel / Δ [CI] |
|---|---|---|---|
| ag_news | 3/256 · 0 | 27/256 · −0.0025 [−0.0074, +0.0024] | 89/256 · 0.0000 [−0.0069, +0.0069] |
| emotion | 0/384 · 0 | 52/384 · −0.0025 [−0.0135, +0.0085] | 230/384 · −0.0050 [−0.0170, +0.0070] |
| xnli_en | 4/192 · 0 | 37/192 · −0.0033 [−0.0180, +0.0113] | 91/192 · −0.0033 [−0.0230, +0.0163] |
| banking77 | 9/3071 · 0 | 137/3071 · −0.0100 [−0.0241, +0.0041] | 668/3071 · 0.0000 [−0.0266, +0.0266] |

**Reading.** Flat everywhere — relabelling up to 60% of the drafter working set moves forced accuracy by ≤ 1 pt, every CI crosses 0. At the site posture the decision is carried by the UNCAPPED nb/oc/ridge count tables over the full pool, so the drafter working-set label is not load-bearing: the probe as specified measures a non-load-bearing input and cannot express label irregularity. Design notes recorded: α ≤ 0.5 is inert by construction (own term 1−α ≥ α·m); the encoder half is structurally null (frozen lanes read no train rows). A future irregularity axis would have to smooth the count-table views too — not pursued (T3's gate already fails on T1).

## T3 verdict — NOT SEPARABLE → negative recorded, no screen written

The separation rule (issue 100 T3) needs an axis whose values order the board's known winners beyond the noise envelope. T1-normalised is uniform in sign with interleaved magnitudes; T1-raw is circular (headroom); T2 is flat. No `.docs/` screen is written; the run-to-run noise envelope was not re-measured because no candidate separation exists to adjudicate (the modelless lane is deterministic; laya's repeat check rides `determinism_ok`). Research 012's theory section is left as-is per the issue; its finding-2 extrapolation is now MEASURED in the normalised form (holds on 4/4 suites) and its lane-seating-screen ambition is MEASURED negative.
