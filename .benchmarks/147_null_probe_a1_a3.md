# Bench 147 — Null/K-stability probe (Issue 090 A1–A3): first measurement

**Status:** RECORD — Issue 090 A1/A2/A3 LANDED (the `--null-probe` harness lane); the control gates fire as designed (4/6 suites RED, exit 1 — loud, recorded, never a quiet pass).

**Provenance:** host shikuwa (4090 box), 2026-10-15T18:41Z · git `64df382` + the probe landing commit (the lane code is THIS commit — the meta's sha predates it by the claim row) · release build · datasets `.raw/datasets` · AC power, power_mode high, load 2.64, GPU 97%/11.7 GiB = the sibling LOWO finetune (this probe is CPU-only — the modelless decide path; no latency claims made, latency_quotable true in meta) · artifacts `.benchmarks/147_null_probe/` (`null_probe.json` + `NULL_PROBE.md`, cell-level byte-deterministic across reruns — floor + A2 cells verified identical).

## The lane

`cargo run --release --bin harness -- --null-probe [--null-k 0,8,32,64,256] [--null-max-cases N] [--suites …]` — Issue 090 A1–A3: every slot is answered across the augmentation ladder (0/8/32/64/256 graded decoys APPENDED; originals keep positions), through the deployed decide path. Pure core `src/harness/null_stability.rs`; lanes `src/harness/runner/null_probe.rs`. Decoys: pool = the run's suite label-vocabulary union, ranked by trigram-Jaccard to the slot's gold, K evenly-spaced picks (the linspace law); a pool smaller than K caps the arm (all six suites cap at K=256 here — pool 153; disclosed in-cell).

The modelless CONTROL is the deployed posture verbatim: `fit_posture_inner` + `build_engine_with` + the cal-slice calibrator (`engine.observe(conf, correct)` over the whole cal split — run_modelless's `fitted` engine shape; the plain run's `calibrated_abstain` column). Without that calibration pass the engine's confidence is the raw readout and the score gate abstains on 100% of rows — the plain run's own `raw_abstain: 1.0` disclosure; measuring THAT gate would measure a gate nothing ships (caught live: the first probe run read 100% abstain at K=0 everywhere; fixed by the observe pass, ag_news K=0 abstain now 37.5% = the plain run's 39.25% posture ±sample).

## Findings (first read, ≤40 slots/suite, n_test 38–60)

**A2 — every preference flip is the k ≤ N route-arming law, zero within-regime.** All measured flips (ag 14/40, massive 18/38, banking 11/39, xnli 2/38, emotion 1/40) are CROSS-regime: the engine's by-name route arms only while `k ≤ N`, so an arm that pushes the option count past the domain count switches scoring regimes by design (route terms → drafter-only). Within a regime: ZERO flips on all six suites — the pre-registered decoy-invariance (independent per-option scores ⇒ the ratio survives the L1 normalizer's shared denominator) HOLDS. A2's verdict keys on within-regime flips (PASS everywhere here); cross-regime counts are the disclosed arming-law column. typed_decisions is all-regime drafter-only (its per-question options are not domain names) — the suite where A2 has full teeth.

**A3 — the paired noul readout is BIT-IDENTICAL across arms on every suite** (the wire's noul carries no options by law; re-answered per arm, so the pin also proves no cross-request state). The choice-side confidence is joint-shaped everywhere (drifts with K) — the readout reads the diluting distribution. Drift magnitude is suite-spread: typed ≈0.05 pt vs massive 57.7 pt at the top arm.

**A1 — the abstention drift curve is real, bidirectional, and non-monotone.** Reference → top arm: typed 100%→100% (flat, its abstention-heavy posture); ag_news 37.5%→37.5%, emotion 40%→40% (flat); **xnli 63.2%→36.8% (−26.3 pt, abstention DECREASES)**; **massive 47.4%→100% (+52.6, INCREASES to total)**; **banking77 46.2%→25.6% at K=8 then →100% at K≥32 (non-monotone, both directions)**. Exactly the paper's Prop-2 ⚠: removing the −log K term is not immunity — the max-of-K score drift moves the gate either way. Characterized, pinned, not gated (the issue's own law).

**A1-floor (Report-the-Floor) — the deployed confidence does NOT uniformly beat the conformal-naive floor: RED on 4/6.** PASS: xnli (gate AUROC 0.6435/ECE 0.0772 vs floor 0.6261/0.2646), banking77 (0.6028/0.0953 vs 0.6014/0.3054). RED: ag_news (AUROC 0.5680 < 0.5840, ECE better), emotion (AUROC 0.5253 ≪ 0.7560, ECE better), massive (ECE 0.1924 > 0.1618), typed (AUROC 0.5144 < 0.6070, ECE better). Pattern: the calibrated gate wins ECE nearly everywhere (the calibrator works) but LOSES the AUROC axis — the dispatch readout's correctness-discrimination underperforms raw max-prob's ranking on 4 suites at n_test 38–60. That is a real G1-axis finding for the readout/calibration lanes (the floor leg is the instrument, not the fix); n_test is disclosed per cell.

## Dispositions

- laya not wired (the perm-probe seam discards its confidence field; A1/A3 need it) — disclosed in-meta.
- Comparison lanes (`--drex/--d1/--agentjev`) wired as A2+confidence cells; not run here (their servers down — the loud-refusal law).
- B1 (`--jeva`) stays gated on the HF weights license (contact-gated, unverified 2026-10-15).
- The exit-1 on control reds is the designed loud refusal: the record is on disk; A2-within and A3 green, the reds are the floor leg (a finding, not a crash).

Refs: Issue 090 · arXiv:2610.18773 Props 1–2 · Plan 340 (the floor shape) · Bench 128 (the perm-probe lineage this lane mirrors).
