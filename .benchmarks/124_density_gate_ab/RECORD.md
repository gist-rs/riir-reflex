# Bench 124 — Issue 066: the fused-abstain density half's paired A/B (POC)

**Status: MEASURED — the POC mechanism is LANDED and the A/B verdict is
NOT CERTIFIED at the ρ=30 per-axis posture: LCB95 < 0 on all six suites.
The lever stays opt-in/report-only (no adoption flip, no default change,
no serving change — the published rows in this very record ARE the shipped
posture's). The directional signal is real and mechanistically aligned
with the PRE-CHECK: the two many-label suites (massive 60 lbl, banking77
77 lbl — the PRE-CHECK's whole-corpus-REJECTED class, where the label
mixture is multimodal and the per-label-vs-pooled density ratio carries
information) gain +0.034/+0.041 selective accuracy with marginal slices
12–18 points WORSE than base (the gate removes genuine errors); the
few-label near-unimodal suites (whole-corpus ACCEPTED post-JL → per-label
≈ pooled → ratio ≈ noise) are flat-to-negative.**

Date: 2026-10-05 · host `m3` · reflex tree at the issue-066 wiring commit
(`--features density_gate`, release build, `--skip-laya --density-gate`,
frozen t20k pool `.raw/datasets_t20k`).
PROVENANCE: `power=AC Power load=3.34 swap=53319.94M canary=140.1us/best5
powermode=2(high)` (preflight PASSED).

Substrate: katgpt-rs `gmm_support` (Plan 618, `1d8da4862`, Bench 908 GOAT).
Consumer shape: `SupportGate<E=64, K=16>` per domain (positive = the
domain's JL-projected rows; negative = the POOLED projected rows, shared)
+ `JlProjector<256, 64>` seed 7 + `EmConfig.var_floor = 1e-2` (re-pinned
consumer scale — see the calibration section).

## The A/B design (one frozen read, paired per question)

`--density-gate` arms a REPORT-ONLY pass (`harness/density_ab.rs`): it
builds the density-armed engine at the SAME fitted posture the deployed
gate runs (score + distance thresholds verbatim; the calibrator re-fit on
the same cal pairs in the same order — the ONLY delta is the density
half), fits the density threshold at the cal-slice ρ=30 percentile (the
T1.6 posture shared with the other two axes), reads the SAME frozen test
slice ONCE, and pairs per-question against the shipped calibrated gate's
read. The pairing law: the gate NEVER touches scoring (pinned by
`density_gate_never_moves_picks_or_probs`) — so the density arm's abstain
set is a strict SUPERSET, and `pick_disagreements` must be 0 (it is, on
every suite — a nonzero reading voids the record).

## The board (the record's `density_ab` blocks; `results.json`)

| suite | labels | Δabstain | Δselacc | LCB95 | marginal (correct-rate vs base) |
|---|---|---|---|---|---|
| massive_intent_en | 60 | +16.3pp | **+0.0408** | −0.0392 | 24/49 = 49% vs base 61% ✓ |
| banking77 | 77 | +7.6pp | **+0.0344** | −0.0405 | 14/38 = 37% vs base 55% ✓ |
| ag_news | 4 | +17.7pp | +0.0177 | −0.0555 | 25/71 = 35% vs 40% |
| sst5 | 5 | +13.0pp | +0.0009 | −0.0586 | 16/78 = 21% vs 21% (neutral) |
| emotion | 6 | +17.0pp | −0.0110 | −0.0752 | 22/68 = 32% vs 29% ✗ |
| xnli_en | 3 | +15.0pp | −0.0202 | −0.1149 | 17/45 = 38% vs 34% ✗ |

Hard accuracy is arm-invariant BY CONSTRUCTION (the gate never moves
picks — pinned), so the trade lives entirely in the selective columns.

## Calibration (the en-route measurement the first red run forced)

The substrate-default `var_floor = 1e-6` (σ = 0.001) let the K=16
positives on 40–64-doc pools collapse onto narrow clusters: QUESTION
embeddings (state+prompt — a different region than the doc embeddings
the pair was fit on) read ℓ ≈ −100s, >30% of cal confidences underflowed
`sigmoid` to exactly 0.0, and the ρ=30 fit landed on a denormal —
abstention without discrimination (ag_news Δselacc +0.0004 at that
posture). The sweep:

| config | massive | banking77 | ag_news | emotion | sst5 | xnli |
|---|---|---|---|---|---|---|
| floor 1e-3 (σ=0.032) | +0.0004 (ag_news probe) | — | +0.0004 | — | — | — |
| **floor 1e-2 (σ=0.1) — LANDED** | **+0.0408** | **+0.0344** | +0.0177 | −0.0110 | +0.0009 | −0.0202 |
| floor 2e-2 (σ=0.141) | +0.0311 | +0.0272 | — | −0.0053 | — | −0.0480 |
| K=4, floor 1e-2 | +0.0427 | +0.0266 | −0.0100 | −0.0022 | −0.0127 | −0.0307 |

Readings: the two winners are ROBUST across every config probed (the
label-mixture signal is not a knob artifact); the wider floor
monotonically helps until it swamps the structure (2e-2 degrades both
winners and xnli); K=4 holds the winners but flips the middle board —
the 16 components earn their keep with the floor doing the spike-guarding.
`DENSITY_VAR_FLOOR = 1e-2` and `DENSITY_K = 16` are the landed pins.

## τ is PROVABLY inert under percentile threshold fitting (the sweep task, closed by argument)

The issue's τ sweep {0.5, 1, 2} is a no-op at this posture, provably:
`sigmoid(ℓ/τ)` is strictly monotone in `ℓ` for every τ > 0, and a
percentile threshold on a monotone transform of a variable selects the
SAME rank — `sigmoid(ℓ/τ) < sigmoid(ℓ_ρ/τ) ⟺ ℓ < ℓ_ρ` for any τ. The
abstain SET is τ-independent; τ only matters for a fixed threshold or
for the confidence's calibration semantics (neither is this posture).
Recorded as closed-by-argument rather than three redundant runs.

## Report-the-Floor (the issue's A/B item)

The G1 floor comparison is arm-invariant at this posture by the same
superset law: picks and confidences never move, so the all-questions
readout ECE and the conformal-naive floor (fit on the same cal pairs
against the same test confidences) are IDENTICAL between arms. The
record carries each arm's ANSWERED-set ECE instead (`answered_ece`) —
the selective-calibration read that actually differs. The G1 verdict for
the density POSTURE as a whole remains the shipped row's (the density
half is not a confidence recalibration; it is an admission filter).

## Verdict

**NOT CERTIFIED — the lever stays opt-in/report-only.** No suite's
Δselacc LCB95 crosses 0; the union abstention cost (+8–18pp at the ρ=30
per-axis posture) is not paid back certifiably anywhere. The recorded
win surface (many-label suites, marginal slices well below base) is the
honest signal for any RE-OPEN: a per-suite arming rule (the cascade-
worthiness shape — arm where the CAL-side marginal slice reads
meaningfully below base) is the natural next design, and the PRE-CHECK's
whole-corpus verdict is the free predictor of which suites those are.
Re-open triggers: (a) a per-suite arming design with cal-side selection,
(b) an out-of-suite negative reference (the A/B's deferred second
posture — the pooled-in-suite negative is the broad-but-own-suite
shape), (c) a consumer whose pools are large enough that the substrate
default floor stops spiking (the 800-sample regime the substrate bench
validated).

## Artifacts

- `results.json` — the six suites' full rows + `density_ab` blocks; the
  shipped rows' `abstain_causes` keep the three-key closed-taxonomy wire
  shape (the `density_gate` key serializes only when nonzero — pinned by
  `abstain_causes_wire_shape_is_the_closed_taxonomy`).
- Engine unit tests (`engine::density_tests`, 5): the pairing premise
  (picks/probs/domains never move), the unarmed posture (no fit, no
  reading), the marginal-cause classification, the fail-closed
  `DensityNeedsCorpora`, and the accessor's determinism/bounds.
