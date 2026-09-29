# Issue 055 — distributional decision layer for the deterministic engine (seeded MC bagging + rejection + LCB)

**Status:** OPEN — fusion idea, novelty TBD; PoC-gated with a pre-registered null path

Filed from the DRM distill (riir-train Research 462 / riir-train Plan 429, 2026-09-29). Source: [arXiv:2609.33803](https://arxiv.org/abs/2609.33803) "Diffusion Reward Models" (thunlp).

## What

The engine today emits **one answer + one calibrated confidence** (deterministic, G1 bit-identity gated). DRM's decision-layer finding: per-instance reward **distributions** beat point scores downstream —

- **Uncertainty-aware rejection:** rank decisions by `U_pair = 1 − |2p − 1|` (p = P̂(r_chosen > r_rejected)) or `U_BoN = P̂(r_runner > r_top)`; rejecting the most uncertain first gains **+2.81 avg at 70% coverage** (PPE Correctness) and **+4.56…+7.31** (RMB).
- **Risk-sensitive full-coverage ranking:** `LCB_λ = μ − λσ` over sampled rewards beats mean-only Best-of-N at every candidate count (λ=0.4).
- **Reward-axis scaling:** more samples tighten the estimate (56.5 → 65.6 on RewardBench v2, N=1 → 32) — a test-time knob a deterministic engine structurally lacks.

**The modelless transfer:** a **seeded Monte-Carlo wrapper** perturbs the hashed-feature embedding (bucket mask / feature dropout under a per-request seed) and runs the deterministic pipeline N times → an empirical per-instance answer/score histogram. That histogram is the sample source for all three DRM decision rules — no training, no diffusion head, no new model. Corpus routing is discrete, so mask draws that reroute the corpus pick produce natively **multimodal answer histograms** — the paper's headline property, without a generative head.

## Why this is an issue and not a Super-GOAT (honest)

- **Q1 partial:** perturbation ensembles are canonical (MC-dropout arXiv:1506.02142, deep ensembles arXiv:1612.01474, bagging/TTA); pessimistic-LCB ranking (arXiv:2206.02593) and reward-ensemble WCO/UWO (Coste et al., ICLR 2024, arXiv:2310.02743) are canonical consumers. The *specific compositions* (runner-up-flip rejection; seeded-hash bagging feeding LCB BoN inside a modelless serving engine) read unpublished per the §4 sweep — but unproven here.
- **Q3 unproven:** the lift over the existing fused gate (calibration + corpus distance) is plausibly **null** — instability under masking is likely correlated with corpus distance (far-from-corpus inputs flip under any perturbation). Pre-registered null path below; a recorded negative is an acceptable landing (`set_rerank` / `differential_anchor` precedent).

## Where

- Primitive (at implementation time, behind substrate-first): katgpt-core vocabulary beside `distributional_steering.rs` — input-perturbation ensemble → empirical answer distribution. NOTE the three-provenance UQ family: **inter-member** (VFD, `velocity_field_disagreement.rs`), **intra-model sampling** (DRM's own head), **input-perturbation** (this) — VFD's UQ-floor *test pattern* is the reusable piece, its signal is NOT.
- Consumer: reflex engine wrapper + the fused gate's third signal (sigmoid-projected instability, never softmax) + harness columns (U stats, coverage-accuracy curves, LCB-vs-mean ranking delta).

## Gates (all pre-registered)

1. **G1 three-part:** unarmed (N=1) == legacy bytes exactly (the frozen-pick parity gates untouched); armed same-seed == armed same-seed (byte-identity); cross-seed **aggregate** stability is a separate assertion class with its own tolerance — never conflated with the determinism pin.
2. **Non-collapse floor:** ≥ 2 distinct picks on a discrimination fixture (the harness's discrimination-floor pattern); an all-same histogram is a dose-response failure, not a confident answer.
3. **G2:** N=8 within the ~1 ms budget (adaptive-N early exit; the stopping rule is a deterministic function of the samples so byte-identity survives).
4. **G4:** preallocated scratch + Welford accumulation; zero per-run allocation.
5. **Report the Floor:** any calibrated-interval claim benchmarks against the conformal-naive floor on CRPS / coverage / Winkler.
6. **Pareto vs the two-signal fused gate at matched coverage** on the 15-suite harness — with the pre-registered null path: marginal lift ≈ 0 → record the negative, keep the feature opt-in behind an exact-literal kill switch.

## Risks

- **Dose-response:** the mask probability has no training signal — sweep p ∈ {0.05..0.3} cal-side; center the induced mean shift cal-side (a deterministic correction, modelless-legal).
- **σ̂ at small N is noisy** (~25% relative error at N=8) — N sweep before any LCB claim; λ swept cal-side.
- **Latency multiplication** is bounded by the engine's µs-class cost (unlike the laya lane), but the gate measures, never assumes.

## Secondary consumer (deferred)

Per-suite **distributional specialist heads** in the instinct hybrid lane (σ for abstention + a fusion that consumes μ): deferred until suites grow — overfit risk at n=64–500, H2 fusion math assumes point probabilities, and the T2 paired-LB95 + escalation-window gates would apply if tried. The trained-head version of this idea lives in riir-train Plan 429 (the Tetris critic lane), which is the priced instance.

## References

- riir-train `.research/462_DRM_Diffusion_Reward_Models.md` + `.plans/429_diffusion_distributional_critic_tetris.md`
- katgpt-core: `velocity_field_disagreement.rs` (+ its UQ-floor test), `distributional_steering.rs`, `best_belief_score`
- Prior art: arXiv:1506.02142 (MC-dropout), arXiv:1612.01474 (deep ensembles), arXiv:2206.02593 (pessimistic LTR), arXiv:2310.02743 (RM ensembles), arXiv:2609.33803 (DRM)
