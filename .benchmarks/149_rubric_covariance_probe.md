# Bench 149 — Rubric-covariance + route-agreement probe (Issue 090 C1): the score-readout algebra pinned

**Status:** RECORD — Issue 090 C1 LANDED (the `--rubric-probe` harness lane); 6/6 contributing suites PASS every identity at every surface, exit 0. The route-agreement half is characterization with two recorded findings.

**Provenance:** host shikuwa (4090 box), 2026-10-15T22:57Z · git `c6fedbb` + this landing commit (the lane code is THIS commit — the meta's sha predates it, the Bench-147 convention) · release build · datasets `.raw/datasets` · GPU 100%/11.7 GiB = the sibling LOWO finetune window (this probe is CPU-only — the modelless decide path; no latency claims) · artifacts `.benchmarks/149_rubric_covariance_probe/` (`rubric_probe.json` + `RUBRIC_PROBE.md`; the probe is deterministic — stride sampling, no RNG).

## The lane

`cargo run --release --bin harness -- --rubric-probe [--rubric-max-cases N] [--suites …]` — Issue 090 C1: the Jeva paper's score readout is `S(v) = Σ q_i·v_i` — the consumer-side algebra every rubric-scoring client performs over our returned `probabilities`. The probe pins it as measurable identities at THREE surfaces of the same decision, each SELF-consistent (the wire renders its own request bytes through the serve edge's own `translate → decide → map_answers` path — cross-surface pick equality is deliberately NOT an identity):

- **I1 scale covariance** `S(10v) = 10·S(v)` — reds a value-dependent (clamped/weighted) readout smuggled into the read.
- **I2 shift covariance** `S(v + c·1) = S(v) + c·Σq` — pins distribution completeness: a truncated or re-normalized-on-the-wire vector breaks it.
- **I3 value-blind argmax** — the shipped pick == argmax(shipped distribution, ties→first). Values never ride our wire, so value-blindness is by construction; the identity pins it stays that way, and specifically catches the **round6 near-tie collision**: an engine pick of index 1 in a pair that rounds to a visible tie reads as index 0 to any consumer recomputing the argmax from the wire. A unit test in the pure core reproduces the collision shape (0.4999995 vs 0.5000004 → both round to 0.5).
- **I4 normalization residual** `|Σq − 1|` per surface, measured against the ±1e-3 TypeSafe wire budget.

Value families: the unit ramp `(i+1)` and golden-ratio scatter `frac((i+1)·φ)·100` (deterministic, no semantic meaning — the identities must hold for ANY v by linearity). Pure core `src/harness/rubric.rs`; lane `src/harness/runner/rubric_probe.rs`; the deployed posture verbatim (the null-probe's `fit_posture_inner` + `build_engine_with` + the cal-slice `observe` calibration). New engine accessor: `DecisionEngine::semantic_route(&self, text)` — the same unit-centroid `pick_domain` argmax the decide path uses, over the text embedded ALONE (read-only instrumentation; no behavior change — 423 lib tests green).

## Findings (first read, ≤40 sampled cases/suite, 129 answered slots over 6 suites)

**Identities: ALL PASS, with three orders of headroom.** I1 max 4.0e-13 (banking77 wire), I2 max 4.3e-14, I3 zero disagreements at every surface, I4 max 6.0e-6 (banking77 wire — the round6 residue at 77 options, `77 × ~1e-7`, exactly the predicted magnitude; the f32 L1 raw residue is 2.5e-7; the engine-norm residue is f64-exact at ~1e-16). The consumer-side expected-value algebra is sound end-to-end: engine f32 → f64 L1 → round6 wire.

**Route agreement (characterization, no bar):**
- **context == state-alone route: 59.7%** — the prompt+criteria move the routed domain on 40% of slots, a direct population-scale measurement of the prompt-pollution effect the engine's route-terms comment describes (why route terms read the STATE ALONE).
- **option-route == context: 4.7%** — short option strings (1–6 words) mostly route elsewhere than the full context; expected — an option text is not a state.
- **name == semantic: 43.4% of named options** — an option string that exactly names a domain routes to its OWN domain under semantic routing only 43% of the time. The population-scale justification for the by-name route-arming law (Issue 023): semantic routing of bare label strings is unreliable, which is exactly why `k == N` + all-options-name-domains resolves by NAME instead.

**typed_decisions: 0 identity slots (loud skip, disclosed)** — every slot abstains at the deployed posture (its abstention-heavy shape, the Bench-147 A1 record's "typed 100%→100% flat"); abstained slots carry no distribution so the identities cannot run. The gate ran on the six suites that answer.

## Dispositions

- Comparison lanes not wired (their servers down at this window — the loud-refusal law); the lane is modelless-control-only, the seam follows null_probe's oracle law when one joins.
- C2 (`set_refine`) stays OUT per the issue's own law (do-not-wire-blind — needs its own probe suite + GOAT first, and katgpt-rs is sibling-wedged this window).
- The exit-1 on control reds is the designed loud refusal; none fired.

Refs: Issue 090 C1 · arXiv:2610.18773 · Bench 147 (the null-probe lineage + the deployed-posture convention this lane mirrors) · Bench 128 (the perm-probe's stride-sampling law).
