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
  Population-robust: the dump is BYTE-IDENTICAL on `.raw/datasets` and
  `.raw/datasets_t20k` (same xnli cal slice in both pulls) — re-verified
  2026-09-28. Verdict per this issue's fork: (a) **λ=8 is the literal
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

- [x] A1 — **MEASURED 2026-09-28 from the already-committed
  published-posture artifact (`.benchmarks/044_close_gaps/results.json`,
  the Bench-068 byte-exact 0.8825 re-confirmation, n=400, 47 errors) —
  no new test read consumed. The disambiguation hypothesis STANDS:**
  business↔sci_tech is 18 of 47 errors (**38.3% of ALL errors**, both
  directions: business→sci_tech 10 / 21.3% + sci_tech→business 8 /
  17.0% — the top-2 rows, 2.6× the next directed row), while world's
  16 errors SPREAD across three targets (business 7 / sports 6 /
  sci_tech 3). The boundary between the two classes is blurred BOTH
  ways — a mutual disambiguation failure, consistent with the T1
  residual (encoder-owned word-order) and exactly the surface A2's
  bigrams + A3's case/title weighting target. A2–A4 are GO to design
  against, with the recorded bar: the gap to win is 6.8 pt; the
  pair holds 38.3% of the error mass, so even a strong pair-specialist
  lever must recover well over half of the pair's errors to move the
  suite — volume (Bench 065) and joint blends (064/065) already
  measured NEGATIVE on the shipped view.
- [x] A2 — **STALE — already shipped substrate.** The bag view's
  count-table events ARE unigram + bigram:
  `hashed_tokens_into` (`src/embed.rs`) pushes `(w_i, w_{i+1})`
  FNV-hashed (BIGRAM_SALT) into the SAME table — exactly this task's
  prescription — and the published ag_news posture SELECTED that view
  (bag, scale 4, observed-laplace; `044_close_gaps` nb_selection).
  ag_news further presents a SINGLE text field (0 pair candidates — the
  title+description are merged upstream), so the pair view never armed.
  The 38.3% pair error mass therefore stands DESPITE shipped bigrams:
  the order signal is already in the counts and is not the binding
  constraint.
- [x] A3 — **REFUSED BY DESIGN, both legs.** (a) Case-preserving
  hashing conflicts with the ONE-LEXICON law — `fnv1a_word` lowercases
  in-hash and `embed.rs` pins "the SAME tokenizer and FNV-1a
  word/bigram hashes as Embedder (one lexicon, two projections)"; a
  case-preserving variant doubles vocabulary pressure and forks the
  lexicon. (b) Positional/title weighting conflicts with "no weights: a
  count table wants integer events, and bigram evidence is one event
  like a word" — weighted events are a different substrate. Either leg
  is a deliberate design change (owner-scale), not a bench lever. Note
  the pair half of the title surface is ALSO structurally absent (see
  A2: single text field).
- [x] A4 — **STALE — already the shipped posture.** The lane's readout
  is ALREADY Platt-recalibrated per suite in the published row (the
  fused gate + `SigmoidGateCalibrator` family; `readout_ece_calibrated`
  + floor in every published record). Any future count-side change
  inherits this path automatically — nothing to build.
  **A-series conclusion: the cheap ag_news mechanism surface is
  EXHAUSTED** — shipped-bigrams (A2) + shipped-calibration (A4) are in
  the measured 0.8825 row, and the remaining idea (A3) breaks two
  pinned design laws. The residual −6.8 pt needs a NEW modelless
  mechanism (044 T1's own conclusion, now diagnosed: it must move the
  business↔sci_tech pair, 38.3% of errors, where volume and joint
  blends measurably do not) — or acceptance.

## Numbering

044 (closed, removed) → **045 taken by
`045_answer_harness_cache_reuse_modelless.md`** (sibling `7b5f0ba`,
landed between this session's commits — dual allocation, this file
renumbered) → 046 in flight by the same sibling session → **this file
allocated 047** (`.highwater` written back in the same commit).
