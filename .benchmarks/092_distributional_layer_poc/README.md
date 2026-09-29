# Bench 092 — the Issue-055 distributional-layer PoC (`--mc-ab`)

**Status:** RECORD (Bench closed 2026-09-30) — **the pre-registered null path FIRED, and the null is STRONGER than first recorded; the feature stays opt-in** (verdict round-1 revision applied: the initial banking77 carve-out was an artifact of the calibrated baseline key — see the calibration finding)
**Plan:** [`.plans/008_issue055_distributional_decision_layer.md`](../.plans/008_issue055_distributional_decision_layer.md) T5+T6
**Issue:** 055 (closed by this bench — record in `HISTORY.md`); **spun off Issue 056** (the calibrated-vs-raw ranking regression this bench's baseline columns exposed)
**Source paper:** arXiv:2609.33803 (DRM) — the decision-layer findings only.

## What ran

15-suite harness at the **deployed seat posture** (`--head-select --nb-select
--oc-select --ridge-select`), frozen Bench-005 pool (`.raw/datasets_t20k`),
`--mc-ab` with N=8, cal-side selection p ∈ {0.05, 0.10, 0.20, 0.30} /
λ ∈ {0.2, 0.4, 0.8} (ties → smallest), test read once. The arm wraps the
FITTED engine; sample 0 of every MC pass IS the legacy solve; the baseline
rankers are the deployed CALIBRATED fused-gate confidence and the RAW
readout confidence over the SAME forced picks — the arms differ only in
the ranking key. Binary `be5c564`, host `m3`, 2026-09-29T18:58Z. Full
tables: `results.json` (per-suite `modelless.mc_ab`).

The founding defect, caught and fixed in-run: the first PoC pass read the
mean-score rule in ENGINE space against gold (`prompt_injections` mean-pick
0.2328 = exactly 1 − 0.7672 — the complement signature). Root cause: the
noul `[yes,no]`→`[no,yes]` flip was applied to majority/LCB but missed on
mean. Fixed + pinned by `noul_mean_pick_is_not_the_inverted_legacy_pick`
(at N=1 every histogram is a point mass — mean/majority/LCB must ALL equal
legacy exactly; any index-space break reads as the complement).

**Posture scope:** the pre-registered gate compares against the DEPLOYED
gate, so the adjudication ran only at the deployed posture. A default-
posture run was made first (wiring smoke + the complement-signature catch)
and is NOT adjudicated — its engine is materially weaker (e.g. banking77
0.394 vs 0.826) and it carries no comparable baseline.

## Verdict — the pre-registered null path FIRED (stronger than first recorded)

**Pareto vs the fused gate at matched coverage (gate 6), dataset suites
with n ≥ 100 (8 of 9; `code_fixtures` n=32 is context-only — the same
size class as the harness families):**

- vs the **raw** readout confidence: u_pair loses **8 of 8**; mean ΔAUC
  **−0.091** (worst massive_intent_en −0.147, xnli_en −0.142).
- vs the **deployed calibrated** confidence: u_pair loses **7 of 8**; mean
  ΔAUC **−0.041** (the single apparent win, banking77 +0.082, is the
  calibration artifact below).
- LCB-λ vs mean/legacy: **null everywhere** (max +0.012).

→ marginal lift ≈ 0 (uniformly negative) over the fused gate at matched
coverage ⇒ **record the negative, keep the feature opt-in behind
`RIIR_REFLEX_NO_MC_ENSEMBLE` + the default-off knob** (the `set_rerank` /
`differential_anchor` precedent).

| suite | n | AUC cal | AUC raw | AUC u_pair | u−cal | u−raw | legacy acc |
|---|---|---|---|---|---|---|---|
| ag_news | 400 | 0.9335 | 0.9494 | 0.8983 | −0.035 | −0.051 | 0.8825 |
| emotion | 400 | 0.8967 | 0.9604 | 0.8826 | −0.014 | −0.078 | 0.8850 |
| sst5 | 600 | 0.4014 | 0.4538 | 0.3821 | −0.019 | −0.072 | 0.3967 |
| prompt_injections | 116 | 0.8538 | 0.9051 | 0.7840 | −0.070 | −0.121 | 0.7672 |
| xnli_en | 300 | 0.6161 | 0.6927 | 0.5506 | −0.066 | −0.142 | 0.5233 |
| massive_intent_en | 300 | 0.9447 | 0.9447 | 0.7982 | −0.147 | −0.147 | 0.7800 |
| typed_decisions | 2000 | 0.5699 | 0.5871 | 0.5099 | −0.060 | −0.077 | 0.4655 |
| banking77 | 500 | 0.8125 | 0.9350 | 0.8945 | **+0.082** | **−0.040** | 0.8260 |
| code_fixtures (context) | 32 | 0.2062 | 0.2062 | 0.5960 | +0.390 | +0.390 | 0.3750 |

## The real finding this bench surfaced — Issue 056

The first draft of this record carved banking77 out as a "wide-label
perturbation-UQ win (+0.082 AUC, +8 pt @70% coverage)". The verdict round-1
review caught the contradiction in the baseline columns: **the RAW
confidence key beats the MC layer on banking77 too (0.9350 vs 0.8945)** —
the +0.082 was a win against a DEGRADED key, not against the engine's
information. Worse, the calibrated key's AUC there (0.8125) sits BELOW the
suite's own accuracy (0.8260) — worse than a random ordering — and the
calibrated key ranks strictly worse than raw on **7 of 9** dataset suites,
never better (up to −0.122 AUC). A monotone calibrator cannot change a
ranking at all, so the deployed (windowed) calibrator is either non-
monotone by construction or mis-mapped — **filed as Issue 056** with the
per-suite table and the discriminating evidence to collect. The gate's own
threshold-wise abstain decisions are NOT implicated (pointwise, order-free);
every RANKING consumer of the calibrated conf is.

The "wide label space" re-open lead is WITHDRAWN with the carve-out: it was
defined post-hoc from the single win, and the widest-label suite that
should have benefited (massive_intent_en, 60 intents) is the worst loss.
Any future re-open must pre-register the population rule and a held-out
wide-label suite.

## Gates

1. **G1 (three-part)** — unarmed N=1 == legacy bytes: the full pre-existing
   suite green at both feature postures (219 lib + 8 engine_gates + 16
   frozen-pick `game_heads` at default; 230 lib at the feature posture
   incl. the 7 `mc_ab` unit tests). Armed same-seed byte-identity: two
   independent `banking77` runs produce **byte-identical `mc_ab` records**
   (timings excluded — the load-sensitive axis, below), and the main-run
   record matches both. Cross-seed aggregate stability: not asserted per
   the pre-registration (separate class).
2. **Non-collapse floor** — T4's straddle fixture (`u_pair > 0` at p=0.3);
   at the deployed posture the PoC histograms are live (banking77
   `lcb_flipped=7`, emotion's λ table populated, `u_pair` AUCs
   well-formed everywhere).
3. **G2 (latency, PROVISIONAL — load-disclosed)** — per-case whole-MC-pass
   p50 at N=8: **639 µs (prompt_injections) → 3 222 µs (banking77) →
   3 985 µs (typed_decisions)**. `PROVENANCE: power=AC load=12.95–19.95
   (sibling measurements on the box) powermode=2(high)` — preflight
   REFUSED; figures are upper bounds. The conclusion has margin load
   cannot cross: the ~1 ms budget at N=8 fails on heavy suites
   STRUCTURALLY (banking77's base solve p50 is 0.338 ms; 8 × = 2.7 ms —
   load would have to inflate the base solve ~2.7× to make the breach
   load-made). **Adaptive-N early exit remains the recorded remedy path**
   (deferred with the reward-axis N scaling, per the plan). No quiet-window
   re-quote is owed for the verdict; one is cosmetic.
4. **G4** — the harness arm is report-side (allocating by design); the
   engine wrapper's zero-alloc core was gated at T4.
5. **Report the Floor** — no calibrated-interval claim is made by this
   record (rejection ranking + decision rules only), so the conformal
   floor comparison does not bind; the readout-ECE G1 columns of the host
   lanes are unchanged (the arm adds no readout).
6. **Pareto (the headline)** — fired above.

## Honest limits

- One test read per suite at the cal-selected (p*, λ*) — the selection
  tables ride `results.json`; no post-hoc re-selection was done.
- `p_drop` landed on the TOP of the grid (0.3) on 4 of 9 suites — the grid
  may be truncated on the high side; the null verdict does not turn on it
  (the losses are at every p).
- Latency provisional (above); the structural-breach conclusion stands at
  any load.
- The default-posture run is unadjudicated (scope line above).
