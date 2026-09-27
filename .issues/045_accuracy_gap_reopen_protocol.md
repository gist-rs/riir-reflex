# Issue 045 — accuracy-gap reopen protocol: xnli mechanism + ag_news n-gram view

**Status:** OPEN — filed from Bench 069's Claude verdict review (rounds
1–2); the negative verdict is confirmed and overdetermined, this issue
owns the BINDING rules and candidate mechanisms for any future attempt.
No read of any xnli slice may happen until the primary-posture rule
below is exercised.

## Why

Bench 069 measured the G1-constrained NLI blend NEGATIVE
(UNSATISFIABLE, λ*=0) and the review confirmed it is overdetermined:
even under a raw-surface feasibility object, the cal-best rung λ=0.125
reads +3.33 pt on test — below the 5 pt arming bar
(`.benchmarks/069_nli_g1_constrained.md`, verdict-review section). But
the review also established the search was UNDERPOWERED and
CONTAMINATED: in-sample head on cal, a 100-pair screen with a ~50%
split-noise bar, and a spent 300-item test set (three reads). The gap
is real (xnli −28.2 pt, ag_news −6.8 pt vs laya); the next attempt must
be designed against the recorded failure modes, not repeat them.

## Binding rules for ANY reopen (from the verdict review)

- [ ] R1 — **Fresh slice only.** Never read the 300-item xnli_en test
  split again for this question. The confirmation surface is the xnli_en
  VALIDATION slice (~2490 items), read once, pre-registered.
- [ ] R2 — **One primary posture named BEFORE any read** of the
  validation slice. The four candidate mechanisms below must not each
  get their own look. Secondaries are reported WITHOUT promotion, or
  the slice is split up front.
- [ ] R3 — **Cross-fit the head.** `NliLda` must fit K-fold and score
  out-of-fold deltas on cal — the Bench-069 screen was invalidated by
  in-sample head contamination (closed-form, cheap).
- [ ] R4 — **Bootstrap bars, not point comparisons.** Feasibility/
  promotion legs carry a paired-bootstrap interval on ECE differences
  (the 069 bar moved ~50% between splits at n=100).
- [ ] R5 — **Per-item pick logging.** Record
  `(engine_pick, head_pick, blend_pick, gold)` for every item, so
  McNemar, the win/loss override split and the bootstrap are computable
  AFTER the run (Bench 069 could not compute McNemar from its
  aggregates).
- [ ] R6 — **Sharpness beside ECE.** Any G1 claim reports AUROC
  (confidence vs correctness) or the Brier resolution term beside ECE —
  binned ECE at n=300 is at/below sampling noise and a near-constant
  confidence at the base rate games it (the 0.0067 vs 0.0964 comparison
  is not meaningful without this).

## Owed diagnostic (cheap, cal-side only — no slice read)

- [ ] D1 — Dump `(w, c)` per rung + held-out NLL/Brier for the Bench-069
  recal windows (env-gated stderr in `build_g1_plan`). If held-out NLL
  improves while ECE worsens, the "Platt minimizes NLL, not binned ECE"
  story holds. If NLL ALSO worsens, the suspect is the shared
  katgpt-core `SigmoidGateCalibrator::refit` solver (undamped f32
  Newton, no line search, unnormalized Hessian, absolute
  `det < f32::EPSILON` guard) — file a katgpt-rs issue with the
  reproduction, not a reflex finding.

## Candidate mechanisms (xnli — pick ONE primary before any read)

- [ ] M1 — **Pick/confidence separation with an agreement gate** (the
  review's top pick): keep the blend's pick; take confidence from a
  3–4 parameter logistic on (logit of the lane's calibrated readout
  confidence, the head's LDA margin δ₁−δ₂, engine/head agreement).
  Agreement is the strongest correctness signal given the 56/53
  decorrelation. Substrate exists: `CalibratedGateSet<2>` in
  `katgpt-core/src/sigmoid_calibration.rs` (direction = agree/disagree,
  no new type).
- [ ] M2 — **Log-odds combination** σ(δ_e + λ·δ_h) instead of
  max-normalized probability sums — LDA deltas are already log-
  likelihood ratios, so the sum is a coherent naive-Bayes combination;
  the max-normalized surface is the one that broke G1.
- [ ] M3 — **Gated switch instead of a blend**: override with the head
  pick only when engine calibrated confidence < τ AND head margin > m
  (two cal-selected thresholds); most picks keep the shipped calibrated
  surface; only the switched subset needs its own gate.
- [ ] M4 — **More cal data**: grow n_cal 200 → ~2000 with K=10
  cross-fit; a closed-form fit on more data is not training.

## ag_news (−6.8 pt) — diagnose before building

- [ ] A1 — **Confusion matrix first**: if the gap concentrates in
  Business↔Sci/Tech, the disambiguation hypothesis stands; if spread
  across classes, it does not (record either way before building).
- [ ] A2 — **Hashed n-gram view on the nb lane's existing view axis**
  (bag/pair already selected per suite): hash `(w_i, w_{i+1})` into the
  same count table — the cheapest order signal; bigrams are the usual
  NB win on topic classification.
- [ ] A3 — **Title-weighted + case-preserving hashing**: weight early
  tokens more (headlines are ag_news's discriminative surface); keep
  case so capitalized tokens behave like entities.
- [ ] A4 — **G1 path**: stay on the lane's readout surface and Platt-
  recalibrate it — correlated n-gram features double-count and produce
  MONOTONE overconfidence, which is exactly the case the shipped
  calibrator fixes well.

## Numbering

`.issues/.highwater` 044 → 045 (044 removed at its closure, Bench 069 /
commit `7d46414`; record in HISTORY.md 2026-09-27).
