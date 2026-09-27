# Issue 047 — accuracy-gap reopen protocol: xnli mechanism + ag_news n-gram view

**Status:** OPEN — filed from Bench 069's Claude verdict review (rounds
1–2); the negative verdict is confirmed and overdetermined, this issue
owns the BINDING rules and candidate mechanisms for any future attempt.
No read of any xnli slice may happen until the primary-posture rule
below is exercised.

> **Renumbered 045 → 047 (dual allocation).** 045 was allocated twice:
> this file (allocator read a stale session-start highwater) and
> `045_answer_harness_cache_reuse_modelless.md` (sibling commit
> `7b5f0ba`, landed between this session's two commits). First
> allocation keeps the number; the latecomer moves. The write-time
> re-check (`ls` + re-read `.highwater`) was skipped — the exact
> stale-read class the numbering discipline exists for.

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

- [x] D1 — **MEASURED 2026-09-28 — the story HOLDS; the katgpt-rs solver
  suspect is CLEARED.** Instrument: `RIIR_REFLEX_G1_DIAG=1` on the
  `--nli-feature-ab` lane (`src/harness/runner/nli_lane.rs`) — per-rung
  `(w, c)` (`params_raw()`) + held-out and **fit-half** NLL/Brier beside
  the ECEs, dumped inside `build_g1_plan` (cal-side only by
  construction), then the lane loud-skips BEFORE the ONE test read (R1:
  the 300-item test split is never read for this question; the
  harness's standard modelless row upstream is previously published
  posture work, unspent for the nli question). New metric helpers:
  `metrics::{nll_of, brier_of}`. xnli_en, n_eval_half = 100 (fit half
  = 100), default `.raw/datasets` (the Bench 068/069 population):

  | λ | raw ECE | recal ECE | raw NLL | recal NLL | fit NLL raw→recal |
  |---|---|---|---|---|---|
  | 0 | 0.0443 | 0.0788 | 0.6149 | 0.6189 | 0.6400 → 0.6362 |
  | 0.125 | 0.2058 | 0.1045 | 0.7709 | 0.6982 | 0.8044 → 0.6331 |
  | 0.5 | 0.1616 | 0.1179 | 0.7259 | 0.6526 | 0.7531 → 0.6074 |
  | 8 | 0.1158 | 0.1326 | 0.6583 | 0.6465 | 0.6692 → 0.6169 |

  (0.25/1/2/4 in the same shape; full table in the landing commit.)
  Verdict per this issue's fork: (a) **λ=8 is the literal
  "NLL improves while ECE worsens" cell** — the "Platt minimizes NLL,
  not binned ECE" story holds; (b) **fit-half NLL improves at ALL 8
  rungs incl. λ=0** — the undamped `refit` solver descends its own
  objective everywhere; λ=0's joint held-out worsening (raw ECE already
  0.0443, the w=7.9 sharpening overfits n=100) is generalization, not a
  broken descent → **no katgpt-rs issue**. Bonus diagnosis the dump
  bought: at every blend rung (0.125–4) recal improves BOTH held-out
  NLL and ECE — Bench 069's UNSATISFIABLE is the recalibrated blend ECE
  (~0.10–0.14) still sitting ABOVE the engine-derived conformal floor
  (the engine is near-calibrated, so its floor is low), i.e. the
  constraint fails on the FLOOR leg, never on the recal leg. Any M1–M4
  attempt inherits that bar, not a recalibration problem.

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

044 (closed, removed) → **045 taken by
`045_answer_harness_cache_reuse_modelless.md`** (sibling `7b5f0ba`,
landed between this session's commits — dual allocation, this file
renumbered) → 046 in flight by the same sibling session → **this file
allocated 047** (`.highwater` written back in the same commit).
