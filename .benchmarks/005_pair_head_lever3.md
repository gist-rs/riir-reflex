# Bench 005 — Issue 013 lever 3: the confusion probe and the fitted pair-head A/B (both REFUTED)

**Status:** COMPLETE 2026-09-24 · run commit = this landing · M3 (macOS,
release, modelless lane only `--skip-laya`) · all 14 suites, fixtures
`.raw/datasets/` · DETERMINISTIC: the full A/B run re-ran byte-identical
(confusion rows + both A/B records, every suite).

## The question

Lever 3 asked whether **fitted heads** (the Bench 881 game-head precedent)
can lift modelless accuracy on the dataset suites, candidate form: "heads
for the suites' most-confused label pairs." Two instruments answer it, in
protocol order:

1. **The confusion probe** (`results.json` `modelless.confusion`, always
   on): per-suite categorical (gold → pred) confusion over the forced raw
   eval — Choice + Score questions, Noul excluded (binary wire). Is the
   error pair-structured at all?
2. **The pair-head A/B** (`harness --pair-head-ab`, report-only): arm the
   top-4 confusion pairs per suite from **CAL-slice** mispredictions
   (≥ 8 support — a pair picked on the test split would be a
   test-set-selected hyperparameter, the exact hole Bench 004 closed for
   caps), fit a **diagonal-LDA head** per pair from the pair's corpus docs
   (the same capped train docs the engine's domains consume; closed-form
   moments, no training — the modelless fit law; σ² shrink
   `+1% mean + 1e-12 floor`), then re-decide questions from the state
   embedding under **two firing gates**: `top2` (the engine's own top-2 IS
   the pair — the canonical form) and `pred` (the engine's pick is in the
   pair — the variant that can see black-hole errors whose gold ranked
   ≥ 3; overlapping pairs resolve to the highest-cal-support pair). The
   record splits every subset into gold-in-pair (the only questions a head
   can move) and gold-outside (locked wrong under both arms). With the
   flag off, every byte of the default posture is unchanged.

## Probe finding — the error mass IS concentrated, into one "black hole" class

| suite | acc | top flow | share of ALL errors |
|---|---|---|---|
| xnli_en | .3467 | → neutral (94) + → neutral (92) | **94.9%** |
| emotion | .2825 | sadness→joy (98), anger→joy (49), fear→joy (32), love→joy (31) | **73.2%** |
| ag_news | .5100 | sci_tech→world (48), sports→world (35), business→world (24) | **54.6%** |
| banking77 | .4460 | 4× → card linking (84 of 277) | 30.3% |
| sst5 | .2167 | spread (top pair 12.3%) | — |
| massive_intent_en | .0767 | spread (top pair 6.1%) | — |

The mechanism behind a black hole is centroid collapse: the broad class's
unit centroid sits near the data mean and attracts everything (the route
term is the state-alone cosine to that centroid). Pair-structured on its
face — which is what justified building the A/B.

## A/B verdict — REFUTED, both gates, no GOAT cell anywhere

Suite-level forced accuracy, counted (Choice + Score) population
(full per-pair tables in `results.json` `pair_head_ab` =
`[top2-gated, pred-anchored]`):

| suite | top2 Δ | pred Δ |
|---|---|---|
| typed_decisions | −0.43 pp | −0.36 pp |
| ag_news | −2.25 pp | −3.25 pp |
| emotion | −9.00 pp | **+1.75 pp** |
| sst5 | −1.50 pp | **+0.67 pp** |
| xnli_en | −0.33 pp | −3.33 pp |
| banking77 / code_fixtures | 0 (0 overrides) | 0 (0 overrides) |

Global net ≈ **−19 questions**; the two positive pred cells are
outweighed by the negative cells on the same arm. The per-pair
gold-in-pair detail is the mechanism, stated as three measured facts:

1. **The engine is not at chance on the fired subsets** — on gold-in-pair
   questions its blend reads 46–81% (e.g. world-vs-sci_tech 65%,
   sports-vs-sci_tech 81%, world-vs-business 80%). The black-hole errors
   are mostly NOT inside the fired pairs: gold-outside is 55–71% of every
   big subset (typed 449/1131 in-pair, ag_news 128/184, emotion 160/356,
   sst5 181/396, xnli 211/300).
2. **Where the engine IS at chance (~50%), the hashed-bag LDA head is at
   chance too** (sadness-vs-joy 49→54%, negative-vs-neutral 48→49%,
   neutral-vs-positive 68→68%) — bag-of-words carries no pairwise signal
   beyond the engine's on these decisions. Its two genuine wins (xnli
   neutral-vs-contradiction 46→55% top2; world-vs-business 74→84% pred)
   are isolated cells, outweighed on their own suites.
3. **Where the engine is strong, the head is strictly worse** (joy-vs-fear
   68→43%, joy-vs-love 68→43%, world-vs-sci_tech 65→54%, entailment-vs-
   neutral 51→46% top2 / 200-sample 51→46% pred) — which is why the
   pred-anchored variant, ungated by construction, bleeds.

**Selection honesty note:** banking77 armed 2 pairs from the cal slice
that fired **zero** times on test — the cal slice is the train-tail prefix
and the mirrors store rows label-grouped (the Bench 004 finding), so its
confusion structure does not transfer to the test slice's routing
behavior. Derived-universe suites' pair selection is additionally weakened
by that clustering; disclosed, not tuned around.

## Verdict

- **Lever 3's dataset candidate is DEAD**: fitted pair heads over the
  engine's own hashed-bag features cannot beat the engine's blend on the
  confused pairs, in either firing form. The `--pair-head-ab` instrument
  and the confusion readout STAY as standing harness instruments
  (report-only, flag-gated, default posture byte-identical).
- **The three levers of Issue 013 are now all measured** (1: cap = per-
  suite tuning knob, promotion refused by protocol; 2: route_scale flat,
  declined; 3: pair heads refuted). The modelless lane sits at its
  **feature-class ceiling** on these suites: every lever that re-weights
  or re-fits the SAME hashed-bag features reads at-or-below the shipped
  blend. The remaining honest lever is the feature class itself (the
  256-dim bag embedder) — recorded in the issue as lever 4, unstarted,
  a different and bigger lift.

## What landed

- `src/harness/metrics.rs`: `ConfusionRow` + `confusion_top` (pure,
  unit-pinned in `tests/harness_units.rs`).
- `src/harness/runner.rs`: the confusion readout (always on, modelless),
  `confusion_rows`, the A/B pass (`--pair-head-ab`): cal-slice arming →
  `PairHead::fit` per pair → both gated walks → the two A/B records
  (`LaneResult.pair_head_ab` = `(top2, pred)`), display-key resolution,
  `n_counted` empty-denominator disclosure.
- `src/harness/pair_heads.rs`: `PairHead` (diagonal LDA, deterministic),
  `select_pairs`, `ArmedPair` / `PairHeadAb` / `PairSubsetRow` records,
  module tests (selection order/floor, separable-signal fit, the
  NaN-floor canary that caught the zero-variance defect before any run).
