# Bench 069 — the G1-constrained NLI blend λ (issue 044's pre-registered reopen path)

**Status:** DONE 2026-09-27 — **measured NEGATIVE for promotion**; the
pre-registered protocol executed faithfully and refused every rung
(UNSATISFIABLE → λ*=0 → zero gain). The pair-feature head's xnli signal
stays report-only. Two instrument findings recorded (the void first run;
mini-G1 screen non-transferability at n_cal=200). **Verdict CONFIRMED by
Claude review (see the closing section): the negative is overdetermined —
it survives the raw-surface feasibility object too — and the record
carries seven amended holes.**

M3, AC, release at the working tree on `7b4b9e8`+ (modelless lane only —
no laya number is published by this bench, G5 not implicated; the claims
are accuracy/pick-counts and load-immune; load 14.5 class noted for
provenance, no latency claim made, no preflight floor owed).

## Why

Bench 068 (issue 044 T3) measured the modelless NLI pair-feature head on
xnli_en: the additive blend at the cal-selected λ=0.5 harvested **+5.67 pt**
(0.5233 → 0.5800, above the house 5 pt arming bar) but the blended readout
carried ECE **0.1596 against the conformal-naive floor 0.1351 → G1 FAIL**
(the Bench-064 massive-row law binding a second time). 068's recorded
reopen path: a pre-registered cal-side G1-constrained λ selection, or a
calibrated blend readout. This bench implements BOTH as one posture and
reads the test ONCE.

## The pre-registered protocol

Implementation: `src/harness/runner/nli_lane.rs` (`blend_g1_constrained`
row; `--nli-feature-ab` grows the posture — same ONE test read, no new
flag). Selection is CAL-SIDE ONLY:

1. **Interleaved halves.** The cal slice splits by index parity: even
   indices = the fit half, odd = the held-out eval half. Interleaving (not
   first/second) keeps a label-grouped cal slice balanced on both sides.
   A fit half never scores its own fit.
2. **ONE floor, engine-derived.** `conformal_naive_floor` fit on the fit
   half's ENGINE readout pairs, read on the held-out half → `floor_b` —
   the test-side G1 construction miniaturized. The engine's own RAW
   held-out ECE is disclosed beside it.
3. **Ladder.** λ ∈ {0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0} — the
   unconstrained ladder extended DOWN so the constraint has somewhere to
   stand; rung 0 is the unarmed engine (the guaranteed no-op fallback).
   Per rung, over the FULL cal slice: the blend pick accuracy (the
   existing ladder's convention); on the held-out half: the raw blend ECE
   and the RECALIBRATED blend ECE (the lane's own `SigmoidGateCalibrator`
   family — same window config as the lane's calibrator — fit on the fit
   half's blend pairs, read on the held-out half).
4. **Feasibility** (mirrors `g1_verdict_of`, strict):
   `recal_b < raw_b  &&  recal_b < floor_b  &&  cal_acc >= cal_acc(λ=0)`.
   The third leg is the no-regression rule at selection time — the
   constraint can never select a blend that loses accuracy on cal.
5. **Selection.** λ* = max cal accuracy among feasible rungs; ties → the
   smaller λ. No feasible rung → λ* = 0 + `constraint_unsatisfiable: true`
   (a refusal is a finding, never a silent worse posture).
6. **The ONE test read at λ\***: blend accuracy + BOTH G1 triples against
   the lane's TEST-side floor — `g1_raw` (the raw blend surface) and
   `g1_recalibrated` (the same calibrator family fit on the FULL cal
   slice's blend pairs at λ*, read on test — the lane's own
   fit-cal-read-test convention).

## The promotion criterion (fixed before the read)

The posture is promotable IFF, on the single test read:

- `g1_recalibrated.pass == true` (recalibrated blend ECE ≤ the
  test-side conformal floor — the lane always ships its calibrator, so
  the recalibrated readout IS the promotable surface), AND
- `acc >= baseline_acc + 0.05` (the house 5 pt arming bar 068 applied).

Anything else: the blend stays report-only, issue 044's xnli follow-up
closes measured (whichever way the numbers read), and NO posture changes.
A pass does NOT silently bump the shipped lane — promotion wiring is its
own follow-up review against this record.

## Pre-registered invariants (must hold in the run, else the run is void)

The two existing postures' code paths are untouched (the blend arithmetic
was extracted verbatim into one helper — same ops, same order), and the
published posture flags reproduce 068's lane exactly. Therefore:

- `baseline_acc == 0.5233` (xnli_en, n=300 — 068 read it twice);
- `floor ≈ 0.1351`, engine calibrated readout ECE ≈ 0.0067;
- `blend_additive.selected == 0.5, acc == 0.58`, its `blend_ece ≈ 0.1596,
  pass == false` (068's numbers byte-reproduced);
- `head_alone_acc == 0.5333`, oracle 56/53.

## Run

M3, AC, release, `datasets_t20k`, modelless lane only (no laya number is
published by this bench — G5 not implicated; accuracy columns are
pick-counts and load-immune by construction; no latency claim is made, so
no preflight floor is owed — load noted for provenance).

```
cargo run --release --bin harness -- --skip-laya \
  --head-select --nb-select --ridge-select \
  --nli-feature-ab --suites xnli_en \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/069_nli_g1_constrained
```

## Results

### Run 1 — VOID (instrument defect, recorded not hidden)

The first run selected λ*=0.125 (acc 0.5567, +3.33 pt) — and its ladder
showed `raw_b == recal_b` at every rung to 4 decimals. Root cause: the
miniature calibrator **never fitted** — `SigmoidGateCalibrator::observe`
only RECORDS; the Platt solve is `refit()`, which the first implementation
never called. `apply` stayed identity, and the strict `recal_b < raw_b`
feasibility leg decided rungs by f64→f32 **rounding noise** (which side of
the bit a widened f32 landed on), not by calibration. A selection decided
by a rounding coin-flip voids the protocol's premise; the run is recorded
here (its results.json was overwritten by run 2; key rows quoted from the
session log) and NOT adjudicated. Fixes: call `refit()` on both legs; read
the held-out half through the SAME f32 lens on both legs so an identity
fit compares bit-equal, never to a widening artifact; and treat a
BEHAVIORALLY identity recalibration (apply returns every input unchanged —
this also covers a constant-confidence window whose Platt solve is
collinear and keeps the identity parameters, a case where `refit()`
reports moved while returning identity) as the equality leg, with the
floor leg staying strict. Module tests extended for the machinery; the
synthetic e2e now shows the recalibration working (0.2917 → 0.0005).

### Run 2 — the protocol read (single test read)

Pre-registered invariants: ALL HELD — baseline 0.5233 (n=300);
floor 0.13514096185737973 and engine calibrated readout ECE 0.0067
identical to 068; `blend_additive` byte-reproduced (selected 0.5,
acc 0.58, blend_ece 0.1596442288662537 — the same 16 digits as 068; the
blend arithmetic extraction into one helper is provably byte-identical);
head_alone 0.5333.

**The cal-side screen refused every rung — UNSATISFIABLE → λ*=0:**

| λ | cal_acc | raw_b | recal_b | floor_b | feasible |
|---|---|---|---|---|---|
| 0.0 | 0.590 | 0.0876 | 0.2838 | 0.2045 | false |
| 0.125 | **0.630** | 0.1175 | 0.2376 | 0.2045 | false |
| 0.25 | 0.625 | 0.1105 | 0.2062 | 0.2045 | false |
| 0.5 | 0.610 | 0.1109 | 0.1931 | 0.2045 | false |
| 1.0 | 0.600 | 0.1234 | 0.1584 | 0.2045 | false |
| 2.0 | 0.585 | 0.2008 | 0.1951 | 0.2045 | false |
| 4.0 | 0.590 | 0.1438 | 0.1472 | 0.2045 | false |
| 8.0 | 0.585 | 0.1205 | 0.1451 | 0.2045 | false |

(engine_raw_b = 0.4374 — the engine's own readout-confidence surface on
the held-out half; the blend's max-prob surface starts far better
calibrated than the lane's readout surface on cal.)

The unsatisfiable fallback is the DESIGNED behavior exercised for real:
λ*=0 → acc 0.5233 = baseline, zero gain. **The promotion criterion fails
on its accuracy leg** (needs ≥ +5 pt; measured +0.0). Verdict: the
G1-constrained blend is NOT promotable on xnli_en; the blend stays
report-only; the lane's published xnli row is UNCHANGED (0.5233).

### The two findings a negative result still pays

1. **The mini-G1 screen is not a transferable predictor at n_cal=200.**
   The cal held-out half said recalibration HURTS the blend surface at
   every rung (0.0876 → 0.2838 at λ=0); the test side said the FULL-cal
   refit at λ=0 PASSES the floor (recal ECE 0.0964 ≤ 0.1351 — the row's
   `g1_recalibrated`, reported for completeness). Same construction,
   opposite verdicts, two splits. A 100-pair Platt refit on this surface
   is split-unstable in ECE terms (Platt minimizes NLL, not binned ECE;
   the two disagree sharply here). Corollary: a cal-side G1 screen needs
   either a bigger cal slice or a different constraint surface before it
   can gate a promotion — do not reuse this screen at n_cal=200 as a
   verdict instrument. (The FIRST order finding stands regardless: the
   test-side recal at every pick-changing λ fails or the λ gains nothing
   — see 2.)
2. **Nothing promotable was FOUND — and the search was underpowered.** The
   only passing surface (λ=0 recal, 0.0964) is the engine's OWN picks with
   a re-fitted confidence — zero accuracy delta — and it reads 14× the
   lane's shipped calibrated ECE (0.0067) — a comparison that itself needs
   a sharpness companion (see the review section: binned ECE 0.0067 at
   n=300 is at/below sampling noise; the 14× is not yet a meaningful
   ranking). The pick-level harvest only exists on λ where the surface
   fails the floor. Closing shape: the pair-feature signal is REAL
   (decorrelated, 56/53) but no promotable surface was found by the ONE
   family tried, under a screen later shown underpowered and
   head-in-sample-contaminated. A future mechanism must bring its OWN
   calibrated surface (or ride the lane's readout confidence), never the
   max-prob blend one.

### Run

```
cargo run --release --bin harness -- --skip-laya \
  --head-select --nb-select --ridge-select \
  --nli-feature-ab --suites xnli_en \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/069_nli_g1_constrained
```

Record: `results.json` (`suites[0].modelless.nli_feature_ab`) + `TABLES.md`.

## Issue 044 disposition

The xnli follow-up is CLOSED (measured negative, this bench). Issue 044
is removed per the noise-reduction rule; the remaining gap (ag_news −6.8
pt, no shipped lever — measured three times now) and the routing/
sensitivity eval-slice note carry forward as reopen paths in HISTORY.md,
not as open questions of a closed issue.

## Verdict review (Claude, rounds 1–2, post-landing) — verdict AGREE, record amended

An independent Claude review confirmed the negative verdict and supplied
the closure this record was missing, plus seven holes (all accepted and
written in):

**The verdict is overdetermined — it survives the RAW feasibility object.**
Under a raw-surface screen (feasibility on `raw_b` instead of
`recal_b`), every rung's raw_b (0.0876–0.2008) already sits below
floor_b (0.2045) — so the screen picks the cal-best rung λ=0.125, and
the VOID run 1 already measured that rung's test PICK COUNT validly
(picks are calibrator-independent; the void defect only touched the
confidence surfaces): **0.5567 = +3.33 pt, below the 5 pt bar.** Raw
object or recal object, the accuracy leg fails. The split-instability
finding cannot flip the verdict either — the only test-passing rung
(λ=0) changes zero picks.

**Holes accepted into the record:**

1. **The head is scored IN-SAMPLE on cal.** `NliLda::fit` uses the FULL
   cal slice and the cal blend pairs are scored by that same head —
   `cal_acc(λ>0)` is inflated, and the mini-calibrator learns the
   confidence map of an in-sample head: a ready reason the screen did
   not transfer to test. Any reopen must CROSS-FIT the head (K-fold,
   out-of-fold deltas — closed-form, cheap).
2. **"Platt minimizes NLL, not binned ECE" is an UNTESTED explanation.**
   A 2-parameter NLL fit on an already-calibrated surface (raw_b 0.0876)
   should land near identity; tripling held-out ECE to 0.2838 is not the
   expected failure. The diagnostic is owed: dump `(w, c)` per rung plus
   held-out NLL/Brier. If held-out NLL improves while ECE worsens, the
   metric story holds; if NLL also worsens, the suspect is the shared
   katgpt-core `refit` solver (undamped f32 Newton, no line search,
   unnormalized Hessian, absolute `det < f32::EPSILON` guard) — a
   shared-substrate issue to file in katgpt-rs, not a reflex finding.
3. **The bar itself is noisy.** floor_b reads 0.2045 on the 100-pair cal
   half against 0.1351 on test — the floor moves ~50% between splits,
   and binned-ECE noise at n=100 is larger than most gaps in the
   ladder. Feasibility should carry a paired-bootstrap interval on ECE
   differences, not point comparisons.
4. **The floor is built on a different surface.** `engine_raw_b` (0.4374)
   vs the blend's raw_b (0.0876 at λ=0) are different confidence
   surfaces. Judging the blend against an engine-derived floor mirrors
   the test-side construction and was pre-registered — defensible, but a
   blend readout's floor should ultimately come from the blend's own
   pick outcomes.
5. **The lane's 0.0067 needs a sharpness check.** Binned ECE that low at
   n=300 is at/below the sampling noise of a perfectly calibrated
   predictor — consistent with near-constant confidence at the base
   rate, which GAMES binned ECE (a G1 loophole: a constant predictor
   passes). The 14× comparison is not meaningful until an AUROC
   (confidence vs correctness) or Brier-resolution term is reported
   beside it.
6. **The 068 harvest is λ-grid-sensitive.** On the finer ladder the
   cal-best λ (0.125) reads +3.33 pt on test, not +5.67. The harvest is
   **3–6 pt, depending on the λ grid, with no confidence interval**.
   The win/loss override split was not recorded, so McNemar is not
   computable from the aggregates; the reopen must carry it.
7. **The spent test set is the binding constraint.** 068, void run 1 and
   run 2 all read the same 300 test items. Any reopen reads a FRESH,
   pre-registered confirmation slice (xnli_en validation, ~2490 items)
   with a cross-fitted head and bootstrap bars — never another pass at
   the test split.

**Reopen rules contributed by the review (binding on the next attempt):**
name ONE primary posture before any read of the validation slice (the
four candidate mechanisms — pick/confidence logistic, log-odds
combination, gated switch, more cal data — must not each get its own
look; secondaries are reported without promotion, or the slice is split
up front); and log PER-ITEM picks `(engine_pick, head_pick, blend_pick,
gold)` so McNemar, the win/loss split and bootstrap intervals are
computable after the run instead of planned before it.

These rules + the mechanism candidates are filed in
`.issues/045_accuracy_gap_reopen_protocol.md`.
