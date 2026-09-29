# Plan 008: Distributional Decision Layer — Seeded MC Bagging over the Deterministic Engine (Issue 055)

**Status:** IN PROGRESS — T1–T4 LANDED (katgpt-rs `8f5291c14` + `3e986fe58`, reflex `aee6404`); T5–T7 DEFERRED to the next session (resume note below)
**Issue:** [055](../.issues/055_distributional_decision_layer.md) (filed from the DRM distill, riir-train Research 462 / Plan 429; arXiv:2609.33803)
**Bench:** `.benchmarks/092_distributional_layer_poc/` (allocated)
**Source paper:** arXiv:2609.33803 "Diffusion Reward Models" (thunlp) — the decision-layer findings only (U_pair/U_BoN rejection +2.81 avg @70% coverage; LCB_λ=0.4 ranking beats mean-only BoN; sample-count reward-axis scaling).

---

## Goal

A seeded Monte-Carlo wrapper perturbs the hashed-feature embedding (Bernoulli
bucket mask under a per-request seed) and runs the deterministic pipeline N
times → an empirical per-question answer/score histogram. The histogram feeds
three DRM decision rules as pure modelless math: uncertainty-aware rejection
(U stats), risk-sensitive LCB ranking, and the reward-axis N knob. No
training, no diffusion head, no new model. **PoC-gated with a pre-registered
null path** (the set_rerank / differential_anchor precedent): marginal lift
≈ 0 over the two-signal fused gate at matched coverage → record the negative,
feature stays opt-in behind an exact-literal kill switch.

## Substrate check (substrate-first skill, Mode 1 — 2026-09-30)

- **Searched for** (vocabulary translation, 3+ variants): `ensemble /
  perturbation / bagging / bootstrap / dropout`, `disagreement / instability /
  variance`, `welford / running_variance / running_moments / incremental_mean`,
  `histogram / answer distribution`, `LCB / best_belief`, `mask draw /
  bucket mask`, across katgpt-core `*.rs` + `.plans/.docs/.issues`.
- **Found (family census — the three-provenance UQ family the issue names):**
  - *inter-member*: `velocity_field_disagreement.rs` + `velocity_field_ensemble.rs`
    (ridge-combined frozen fields — different provenance, not input-perturbation).
  - *steering*: `distributional_steering.rs` (FK-weighted population steering —
    steers a population; does not measure per-instance UQ from perturbation).
  - `guided_width/perturb.rs` — a `Perturbation` TRAIT with the right
    discipline (seeded, deterministic, alloc-free, σ=0 bit-identical) but the
    WRONG contract for this lane: `delta`/transversal-orthogonal noise for
    rollout exploration on dense belief states, not Bernoulli bucket dropout
    on an input feature bag. Kinship recorded, not consumed.
  - `subspace_intervention.rs`, `set_diffusion_schedule.rs`,
    `velocity_field_ensemble.rs` — read; different concepts (probe fitting /
    diffusion schedule / model combination).
  - `diversity::temp::blake3_noise_fill(seed, sigma, out)` — UNGATED, the
    shipped seeded noise source (BLAKE3, one hash per 8 coords, uniform
    [-1,1]·σ). **CONSUMED** (the mask draws derive from the same per-block
    hash stream; a uniform variant joins it in `diversity::temp`).
  - `WelfordVariance` EXISTS at `karc/regime_gate.rs` behind opt-in
    `karc_regime_gate` (implies `karc_forecaster + conformal_predictive_intervals`)
    — feature-entangled for this consumer. **MOVE** to an ungated home
    (`welford.rs`), karc re-exports for compat (the `rating` promotion
    precedent; one definition, two substrate consumers).
  - `act_channel_moments.rs` (raw-sums calibration accumulator) +
    `fitted_anchor_table.rs::StreamingMeanTable` — different shapes; their
    LAWS are consumed as design constraints (f64 streaming stats, alloc-free
    observe, poison control).
- **Decision: BUILD NEW** — no input-perturbation ensemble primitive exists
  (the family ships the other two provenances; this is the missing member).
  Consume `blake3` hashing + the moved `WelfordVariance`; the `Perturbation`
  trait's DISCIPLINE is consumed as convention, its contract is not.
- **Architectural rules checked:** domain classification (histogram + U stats
  = semantic/latent math, sigmoid projections — never softmax for gates ✓);
  sync boundary (pure local compute, no sync surface ✓); bridge pattern
  (zero-alloc, feature-gateable ✓); sigmoid-not-softmax (LCB over sample
  moments is arithmetic, no normalization competition ✓).

## Design

**katgpt-core `perturbation_ensemble` (opt-in feature, module
`src/perturbation_ensemble.rs`):**

1. `blake3_uniform_fill(seed, out)` in `diversity::temp` — the [0,1) uniform
   sibling of `blake3_noise_fill` (same per-block hashing; mask draws +
   any future seeded-uniform consumer).
2. `bucket_dropout_into(q: &[f32], out: &mut [f32], seed: u64, p_drop: f32)`
   — Bernoulli per-bucket mask from the uniform stream; dropped buckets
   zeroed; survivor set re-L2-normalized (the `distance_abstain::unit` law —
   cosine geometry must not see a shrunk vector). `p_drop == 0` bit-identical
   (copies q). Deterministic in `(q, seed, p_drop)`, alloc-free.
3. `WelfordVariance` moved here (ungated `src/welford.rs`), karc re-export.
4. `EnsembleHistogram` — per-question sample accumulator: pick counts per
   option (`usize` counts, caller-sized) + `WelfordVariance` over the picked
   option's score; zero-alloc observe; `majority_share()`, `runnerup_flip_share()`.
5. DRM decision rules as pure fns over the accumulated samples:
   - `u_pair(h) = 1 − |2·p_majority − 1|` (0 = certain, 1 = coin flip);
   - `u_bon(h) = runnerup_flip_share()` (P̂(runner-up outranks top));
   - `lcb_score(mean, sigma, lambda) = mean − lambda·sigma` — ranking key.
   Sigmoid-projection helper for the fused-gate third signal:
   `instability_gate(u) = sigmoid(u / u_temperature)` — never softmax.

**reflex `mc_ensemble` (opt-in feature, kill switch `RIIR_REFLEX_NO_MC_ENSEMBLE`
— exact literal; also default-off engine knob like nb_scale/oc_scale):**

6. `src/mc_ensemble.rs` — the wrapper: embed once (unperturbed, the legacy
   bytes), then N−1 perturbed re-runs through the engine's score→route→abstain
   path with a per-request seed = BLAKE3(request bytes ‖ sample idx); per
   question accumulate `EnsembleHistogram`; exposes the U stats + LCB ranking
   + the majority pick (reporting picks; the SERVED pick stays the legacy
   pipeline's unless the Pareto gate wins). Adaptive-N early exit: stop when
   the majority pick's share crosses a calibrated certainty bound (the
   stopping rule is a deterministic function of the samples — byte-identity
   survives). N=1 + p_drop=0 → the wrapper is not called (unarmed posture).
7. Harness columns (`harness/metrics.rs` + runner): U stats per question,
   coverage–accuracy curve (reject by U desc, accuracy vs coverage), LCB-vs-
   mean ranking delta, paired vs the fused gate at matched coverage.

## Pre-registered gates (from the issue, verbatim intent)

1. **G1 three-part:** unarmed (wrapper absent / N=1) == legacy bytes exactly
   (frozen-pick parity gates untouched); armed same-seed == armed same-seed
   (byte-identity); cross-seed AGGREGATE stability is a separate assertion
   class with its own tolerance — never conflated with the determinism pin.
2. **Non-collapse floor:** ≥ 2 distinct picks on a discrimination fixture
   (the harness discrimination-floor pattern); an all-same histogram on the
   adversarial fixture is a dose-response failure, not a confident answer.
3. **G2:** N=8 within the ~1 ms per-decision-set budget (adaptive-N early
   exit); release bench, `scripts/bench_preflight.sh` PROVENANCE line quoted
   (box-state law — the M3 is under a sibling measurement at authoring time;
   the latency bench WAITS for a quiet window).
4. **G4:** preallocated scratch + Welford accumulation; zero per-run
   allocation (canary-armed counting allocator, the engine's pattern).
5. **Report the Floor:** any calibrated-interval/confidence claim benchmarks
   against the conformal-naive floor (`metrics::conformal_naive_floor`) on
   CRPS / coverage / Winkler.
6. **Pareto vs the two-signal fused gate at matched coverage** on the
   15-suite harness — pre-registered null path: marginal lift ≈ 0 → record
   the negative, feature stays opt-in.

## Tasks

- [x] **T1** Substrate-first Mode 1 gate (this record).
- [x] **T2** katgpt-core: ~~diversity::temp::blake3_uniform_fill~~ **amended in-fixup**: the helper moved INTO the module (`diversity` sits behind `temp_loss_fingerprint`, absent from narrow-feature consumers — the feature stays `[]`-clean) + the `WelfordVariance` move (`src/welford.rs` ungated; karc/regime_gate re-export; karc tests green under the move) + `perturbation_ensemble`
  feature + `src/perturbation_ensemble.rs` (bucket_dropout_into,
  EnsembleHistogram, u_pair/u_bon/lcb_score, instability_gate) + unit tests
  (11 green). Landed `8f5291c14` + fixup `3e986fe58`.
- [x] **T3** katgpt-rs repo duties: README/examples counts 663→664,
  `.docs/09_feature_catalog` opt-in entry, count_features ✓, docs_gate
  35/35.
- [x] **T4** reflex: `mc_ensemble` feature + `src/mc_ensemble.rs` wrapper +
  engine seam (`solve_sample_into` extraction — solve_into delegates with
  `None`, byte-identical by construction; hooks at BOTH embed sites with
  domain-separated seeds) + 4 gates green (G1 parts 1–3 + non-collapse
  floor) + FULL suite green (219 lib + 8 engine_gates + 16 frozen-pick
  game_heads — the refactor's byte-identity proof) + clippy `-D` clean at
  both postures. Landed `aee6404`.
- [ ] **T5** reflex harness: `--mc-ab` arm — per-suite A/B rankers (baseline
  fused-gate confidence desc vs u_pair desc) over the SAME eval loop,
  coverage–accuracy curves at matched coverage + LCB-vs-mean ranking delta
  + paired-vs-fused-gate; the 15-suite PoC run (Bench 092).
  **RESUME NOTE:** follow the pair_head_ab pattern end-to-end — pass fn
  (`pair_head_ab_pass` at runner.rs:867), typed arm struct in the result
  (~:700), flags field (~:5360 `pub pair_head_ab: bool` + :5636 parse +
  :5106 defaults + :3729 None-init), dispatch (~:3147), table render +
  out-dir write. The arm calls `mc_ensemble::solve_mc_into` per test case
  over the engine built by `build_engine_with` (runner.rs:1436), collecting
  per-question (gold, computed pick, confidence, u_pair, u_bon) → the
  coverage–accuracy curve pair. Datasets on-box: `.raw/datasets_t20k`
  (the frozen Bench-005 pool — the arena default) + `.raw/datasets`.
  Box-state: quote `scripts/bench_preflight.sh` PROVENANCE for any latency
  row; correctness/coverage rows are load-insensitive.
- [ ] **T6** Adjudication: Pareto verdict per the null path; docs (HISTORY
  row, issue update); promotion decision (default: stays opt-in).
- [ ] **T7** Verdict ping-pong (research skill §5 discipline) before the
  closing commit.

## Deferred / out of scope

- Reward-axis N scaling beyond the N sweep in T5 (the knob is exposed; the
  sweep is the honest reading, no extrapolation).
- Any laya-lane interaction (the layer is modelless-side only).
