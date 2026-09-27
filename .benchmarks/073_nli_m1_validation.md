# Bench 073 — Issue 047 M1: the xnli validation-slice reopen (pick/confidence separation)

**Status:** DONE 2026-09-28 — **measured NEGATIVE; the run is VOID as an
M1-promotion instrument under the pre-registered S2 gate, and the
negative is overdetermined independently of the void**: the head is
net-negative against the engine on the fresh slice, every promotion leg
fails on clean validation-side numbers, and the primary gate feature is
measured degenerate. Issue 047 closes: no promotable xnli posture exists
in the candidate set. Two instrument findings recorded (the NB
self-reference mis-estimate; the shipped calibrated readout's near-binary
saturation — the review's hole-5 ECE loophole, measured in the wild).

M3, AC (100%, `pmset` AC), load 6.75 at launch (sibling agents active;
accuracy/pick-count claims are load-immune, no latency claim made, no
preflight floor owed — the 069 precedent). Release build at `b8f4ea9`.
Deterministic: the command re-runs byte-identically (a re-run is not a
second adjudication).

```
cargo run --release --bin harness -- --skip-laya --head-select --nb-select \
  --ridge-select --nli-m1 --suites xnli_en_val \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/073_nli_m1_validation
```

Pre-registration: `.plans/006_nli_m1_reopen.md` (committed `6ea4bfbc`,
BEFORE the instrument commit `1e19be7`, BEFORE any run). Two pre-read
launch failures died at data load/label resolution — no eval ran, the
read was unspent (`b8d8f17`, `b8f4ea9`).

## The read

n = 2490 validation items (the full `validation` split, fetched once —
R1; the 300-item test split was never loaded). Fit pool = 19,782 items
(cal front + corpus-excluded train rest; 26 premise-shared rows
excluded). K=10 cross-fit; B=2000 bootstrap, seed 20260928; the exact
pre-registered grids and legs.

| leg | result | verdict |
|---|---|---|
| λ* selection (OOF, pool) | **λ\* = 0** — every blend rung ≤ the engine on the pool ladder | selection refused the mechanism |
| 1. m1_ece ≤ floor | 0.3550 vs 0.1243 | FAIL |
| 2. m1_ece < own raw | 0.3550 vs 0.1398 | FAIL |
| 3. acc ≥ A0 + 5 pt | 0.5410 vs 0.5410 (λ\*=0 ⇒ identical picks) | FAIL |
| 4. bootstrap LB95 > 0 | [0.0000, 0.0000] (degenerate: zero pick deltas) | FAIL |

Sanity gates: S1 ✓, **S2 ✗**, S3 ✓, S4 ✓, S5 ✓ → the plan's VOID flag
fired. Adjudication below.

## The component-level refutation (clean numbers, defect-untouched)

The decisive numbers involve NO pool data and are valid under the
Bench-069 salvage rule (which aggregates the defect provably does not
touch):

- **Head-alone on validation: 0.5205** (the full-pool-fit LDA, ~19.8k
  training rows) **vs A0 0.5410** — the pair-feature head is 2 pt BELOW
  the engine out-of-sample, at 10× its old training size. The head is
  data-saturated at ~52–53% (it scored 53.3% on the old test split from
  just 200 rows) — the 4 lexical features are exhausted.
- **Oracle win/loss: 397 / 448** — the head's unique wins are net-NEGATIVE
  against the engine (McNemar 448/397 discordance, one-sided p ≈ 0.008).
  A component that nets −51 picks cannot power a +5 pt blend. This alone
  refutes every pick-level mechanism in the 047 candidate set (M1's blend
  pick, M2's log-odds, M3's switch): they all ride the same head.
- M2 observation (selection was pool-side/contaminated, so an
  observation, not a result): 0.5578 at λ2=0.125 — +1.68 pt, 3.3 pt
  below the bar. M3: zero switches at the selected (τ=0.5, m=8.0) —
  the empty corner of its grid.

The 068 "+5.67 pt blend harvest" is therefore fully resolved as
selection artifact — in-sample head contamination + λ-grid sensitivity +
n=300 noise — exactly the review's diagnosis, now measured at 8× the
data on a fresh slice.

## Instrument finding 1 — the NB self-reference mis-estimate (the void's cause)

The plan disclosed pool rows' NB-table membership as "measured-negligible
(~1/6600 of table mass)". **Falsified**: `nb-select` chose scale 16, and
the self-boost is 16× the item's OWN tokens — the pool ladder read
**oof_a0 = 0.9108** (self-retrieval, not signal), every λ diluted it
(ladder monotone down to 0.5411 at λ=8), and λ\*=0 was selected *because
of* the contamination. The gate then fit on 91%-positive contaminated
labels (base NLL 0.3006 at the inflated rate; gate NLL 0.2895 — under
the plan's −0.02 gain bar → S2 fired). The plan's void clause applies:
as an M1-promotion instrument, the run is VOID. The salvage rule
preserves the validation-side aggregates above (they involve no pool
read), and they carry the negative regardless — the void and the
negative reach the same consequence.

## Instrument finding 2 — the shipped calibrated readout is near-binary on xnli (R6 payoff)

The engine's shipped CALIBRATED readout confidence on this slice:
**95.5% of items at exactly 0.0, 4.5% at exactly 1.0, zero in between**
— and 1,242 of the 0.0-conf items are RIGHT. Consequences, each
arithmetic-verified from the per-item log:

- The lane's own G1 face (ECE **0.0028** ≤ floor 0.1243 → PASS) is an
  **artifact**: conf-0.0 items fall in NO bin (the `ece_edges`
  left-open convention), so 95.5% of the mass removes itself from the
  binned ECE. This is the review's hole-5 loophole ("a near-constant
  confidence … GAMES binned ECE") measured in the wild, in the shipped
  lane's own published row.
- NLL 13.86 on the same surface (right-at-0 items cost −ln(1e-12) each)
  — the clamp floor, not a meaningful calibration reading.
- AUROC 0.5359, Brier resolution 0.0071 — near-zero discrimination.
- **M1's x1 feature (`logit c_eng`) was therefore a constant step
  function** (logit(1e-4) for 95.5% of items): the gate had no
  continuous signal to learn from. S2's firing is the correct verdict
  on a degenerate input, not a fit failure.
- The RAW max-prob surface is the honest one here: continuous
  (0.334–0.724), AUROC **0.6542**, ECE 0.1398. Any future confidence
  work on xnli must start from it (or repair the calibrator saturation),
  never from the shipped calibrated readout.

## Disposition

- Issue 047 **closed, measured-negative**: D1 + A1–A4 + M1 + R1–R6 all
  exercised; M2/M3 computed in-read and nowhere near the bar; the head
  itself refuted at the component level. The xnli gap (−28.2 pt vs laya)
  stands **accepted** — the pair-feature mechanism family is exhausted.
  ag_news's −6.8 pt stands accepted per the A-series (047).
- The lane (`--nli-m1`, `xnli_en_val`) stays in the tree as the
  instrument that ran the pre-registered protocol; report-only, nothing
  shipped changed. The published xnli_en row (0.5233, test) is
  UNTOUCHED; the validation A0 (0.5410, n=2490) is recorded here as the
  fresh-slice reference.
- A future xnli attempt would need (a) a NEW confirmation surface (this
  read is spent — the plan's void default), (b) fold-engines or
  corpus-excluded NB tables for pool reads, and (c) a continuous
  confidence source. None is planned.

Record: `results.json` (`suites[0].modelless.nli_m1`, per-item log
included — R5) + `TABLES.md`. Plan: `.plans/006_nli_m1_reopen.md`
(status updated to COMPLETE with this adjudication).
