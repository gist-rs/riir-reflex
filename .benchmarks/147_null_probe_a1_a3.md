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

## Addendum — the n-firming re-read (2026-10-16, same instrument, 10× n)

`--null-probe --null-max-cases 500` (stride law samples 300–600 slots/suite vs the
first read's 38–60) — the A1-floor verdicts at full population, CPU-only, host
shikuwa, sibling LOWO window unchanged:

| suite | first read (n 38–60) | n-firmed (n 300–600) | gate AUROC vs floor | gate ECE vs floor |
|---|---|---|---|---|
| typed | RED | **PASS** | 0.5726 vs 0.5716 (tie) | 0.1679 vs 0.4350 (win) |
| ag_news | RED | **RED — CONFIRMED** | 0.5588 vs 0.5846 (**−0.026**) | 0.0813 vs 0.2047 (win) |
| emotion | RED | **RED — CONFIRMED** | 0.4745 vs 0.5208 (**−0.046**) | 0.1297 vs 0.2980 (win) |
| xnli | PASS | **RED** (Δ 0.0024) | 0.4767 vs 0.4791 (tie-zone) | 0.0335 vs 0.3201 (win) |
| massive | RED | **PASS** | 0.7156 vs 0.7153 (tie) | 0.0614 vs 0.1249 (win) |
| banking77 | PASS | PASS | 0.6322 vs 0.6321 (tie) | 0.0278 vs 0.1942 (win) |

The finding FIRMS, with the small-n verdicts split into signal and noise:

- **The AUROC-axis weakness is REAL on ag_news (−0.026) and emotion (−0.046) at
  10× n** — the deployed confidence readout ranks correctness worse than raw
  max-prob on exactly these two suites. typed/massive flip to PASS (their
  first-read REDs were n 38–60 artifacts); xnli flips RED on a Δ 0.0024 that is
  deep inside noise — the strict lose-no-axis rule fires on a tie-zone delta
  (disclosed as such; a noise band on the AUROC axis would be an instrument
  refinement, not a verdict change).
- **The ECE axis wins HUGE and consistently everywhere (0.03–0.17 vs 0.12–0.44)**
  — the calibrator does its job; the ranking axis is the weak one.
- Because the floor's conformal calibration is MONOTONE in the raw max-prob,
  floor AUROC ≡ AUROC(raw max-prob): the comparison is really
  AUROC(deployed readout) vs AUROC(raw max-prob). The deployed dispatch (the
  Bench-817-inherited readout) is not a monotone function of max-prob — that is
  where the ranking loss lives, and any fix is a readout-dispatch re-derivation
  (explicitly gated on evidence exactly like this — the AGENTS "never
  re-derived" law; this addendum is that evidence base, not the fix).

A2/A3 at 10× n: zero within-regime flips everywhere (e.g. typed 0/295 untied
slots); the cross-regime flip counts scale with n (ag 114/400, banking 130/500,
massive 117/300 — the arming-law shape unchanged from the first read); noul
bit-identity holds on every suite — the structural pins are n-independent, as
expected.
