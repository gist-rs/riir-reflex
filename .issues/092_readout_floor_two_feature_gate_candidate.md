# Issue 092 — the 2-feature readout-gate candidate: can a (maxp, entropy) logistic gate dominate the conformal floor where the shipped posture is RED?

**Status:** OPEN — filed 2026-10-16 from Bench 147 + its 10× addendum. REPORT-ONLY
instrumentation lane (no deployed-path change, no arming); the promotion half (T3) is
conditional on T2 dominance and rides its own GOAT + verdict.

**Claimed:** 4090-watch session, 2026-10-16T11:5x +07, expires 2026-10-18T12:00Z

## Why

Bench 147 (Issue 090 A1's floor leg) measured the deployed confidence readout against
the conformal-naive floor and came back **RED on 4/6 suites**, n-firmed at 10× n
(`--null-max-cases 500`; addendum in [147](../.benchmarks/147_null_probe_a1_a3.md)):

- ag_news: gate AUROC 0.5680 < floor 0.5840 (gate ECE better)
- emotion: gate AUROC 0.5253 ≪ floor 0.7560 (gate ECE better) — the worst cell, on the
  suite where the engine's pick accuracy is the board's highest bar (0.8850)
- massive: gate ECE 0.1924 > floor 0.1618
- typed: gate AUROC 0.5144 < floor 0.6070 (gate ECE better)
- PASS: xnli (AUROC 0.6435 > 0.6261), banking77 (ECE 0.0953 ≪ 0.3054)

The verdict rule is weak dominance on BOTH axes + strict on one (`floor_cell`'s `beats`).

## The adjudicated history this lane must respect

1. **Issue 047 (closed measured-negative, Bench 073):** the xnli −28.2 pt / ag_news
   −6.8 pt confidence gaps stand ACCEPTED; its own conclusion — "honest confidence work
   starts from the RAW max-prob surface, AUROC 0.6542" — is the diagnosis Bench 147
   n-firmed at population scale.
2. **Bench 052 demotion (the per-suite readout arming lever):** arming the best cal-ECE
   `ReadoutMode` candidate per suite OVERFIT the cal slice — on the narrow suites where
   a candidate won (entropy→maxp) arming it REGRESSED emotion's test G1 to a FAIL; on
   the wide suites the candidates coincide with Dispatch. `EngineConfig::readout` stays
   the opt-in knob, never armed by the harness (the doc block at
   `harness/runner.rs::readout_report_on_cal`).
3. **The Bench-817 dispatch law (`src/readout.rs`):** narrow (≤8 options) → inverted
   normalized entropy, wide → argmax-label-prob; the TABLE is the pinned contract.

The physics after 052: the two SINGLE functionals trade the axes — the shipped
(narrow=entropy) readout wins ECE and loses AUROC; maxp wins AUROC and loses calibrated
ECE. Neither single dominates the floor. That is exactly the shape a 2-feature
combination can break: a logistic gate over BOTH functionals,

    conf₂ = σ(a·maxp + b·(1 − H/ln K) + c)

subsumes each single as a restriction (a=0 or b=0) and can rank by the linear
combination while its fit calibrates ECE. Whether it DOMINATES the floor on the red
suites — without regressing the green ones — is an open measurement, not a foregone
conclusion (the 052 lesson is precisely that cal-side wins do not transfer to test).

## Tasks

- [ ] **T1 — the candidate leg in the floor instrument (report-only).** In the
  `--null-probe` modelless cell: collect `(maxp, inv_entropy, correct)` per CHOICE slot
  at BOTH existing collection sites (the cal pass at `runner/null_probe.rs` line ~519
  and the reference-arm test triples at ~622 — the probs vector is already materialized
  at both); fit `conf₂` on the CAL slice with the deterministic sequential-SGD logistic
  law of `src/label_heads.rs` at D=2 single-head (harness-local, zero deps, citing the
  law — do NOT generalize LabelHeads for one instrumentation site); evaluate on the TEST
  slots: AUROC + ECE for {shipped gate, floor, conf₂} in a `FloorCell`-shaped extra cell
  with its own dominance verdict vs the floor. Unit tests: the fit recovers each single
  (a=0 / b=0 sanity), the dominance verdict arithmetic, determinism (seeded order). The
  lane CHANGES NO deployed path, arms nothing, and never touches the run verdicts of the
  existing legs.
- [ ] **T2 — the bench run (Bench 151).** The 147 posture verbatim:
  `cargo run --release --bin harness -- --null-probe --null-max-cases 500 --suites
  ag_news,banking77,emotion,massive_intent_en,typed_decisions,xnli_en --out
  .benchmarks/151_readout_floor_candidate`. The honest verdict either way: dominance on
  the 4 red suites + no regression on the 2 green ones = the reopen evidence base;
  anything less = the recorded negative that keeps the adjudicated posture standing.
- [ ] **T3 — (conditional on T2 dominance) the engine lane + GOAT + verdict ping-pong.**
  Any deployed-path change (a 2-feature gate calibration in the engine) is its own lane:
  G1 the full floor gate at 10× n, G3 flag-off hash-identity, G2/G4 `--release`, the
  Bench-817 dispatch-table law untouched (the candidate layers ON the readouts, it does
  not replace them), owner-gated promotion. If T2 does not dominate: record the negative
  and CLOSE — the 047 acceptance stands.

## Non-goals

- No reopening of the pair-feature head lane (044/047 — measured FAIL, closed).
- No `ReadoutMode` per-suite arming (Bench 052 demotion stands).
- No dispatch-table edit (the Bench-817 contract; `NARROW_MAX_OPTIONS` is not touched).
