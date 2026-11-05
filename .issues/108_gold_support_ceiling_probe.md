# Issue 108 — Gold-support ceiling probe for harness suites (+ the adjudication contract)

**Status:** OPEN — Gain (reflex Research 015 §2.1; ungoat discipline: no plan until a proof path exists)
**Claimed:** zed-4090-idle-gs108, epoch 1793895876, expires 2026-11-07T23:24:36Z

## Problem

Reflex's harness reads suite scores as gold-absolute: a lane scoring 0.42 on sst5 is read as model failure against the suite's ceiling of 1.0. But benchmark gold itself carries noise — surface-form ambiguity, option imprecision, labels the fixture text does not support as typed. OntoSmith's flagship record (public MIT repo, `docs/whitepaper.md` §5.3–5.4) measured this on a KG benchmark: under a ruled adjudication contract, only **ĝ_B = 0.5244** of 17,284 gold triples were text-supported as typed, and it keeps TWO recall ceilings separate — pipeline reachability vs the gold's own as-typed support ceiling (their per-gold-item fate census: matched / emission_blocked / none_judged / never_enumerated / roster_missed, with per-stratum adjudicated support rates + CIs).

Reflex has no gold-quality axis at all: our ceilings are spend ceilings (`CLEF_SMOKE_MAX_CASES`) and body-size ceilings; the existing gold-adjacent check — the keymap per-row gold pin (Bench 141, fix `724b5df`) — verifies **label-mapping integrity** (doc label ↔ presented key), a different axis from whether the fixture's own text supports the label. Before calling a lane score (ours OR a JDI entrant's) "low", we cannot say how much of the gap is the benchmark's own gold. This also feeds the decision-index-beat lane (B5 law's neighbor): board and our-suite reads both quote gold-absolute numbers.

## The ask

A report-only harness arm — **`--ceiling-probe --suite <name>`** — that machine-adjudicates a stratified sample of the suite's gold items under a WRITTEN adjudication contract and reports:

1. **Gold-support ceiling estimate** ĝ with Wilson-style 95% CI, per suite (as-typed support: does the fixture's own context/options text support the gold label as typed?).
2. **The harness fate census** (the closed set, per gold item sampled — exactly one fate):
   - `correct ∧ supported` — lane right, gold text-supported
   - `correct ∧ not_supported` — lane agrees with gold the text does not support — **the shared-bias / memorization signal; the strongest new cell**
   - `wrong ∧ supported` — lane error on supported gold (real model failure)
   - `wrong ∧ not_supported` — lane disagrees with unsupported gold (defensible error)
   - `abstained` — the gate's abstention (separate from wrong — reflex's first-class answer)
   - `unknown` — quote-verification failed or genuinely unclear; excluded from every denominator, count published
3. **Two ceilings, harness versions, never pooled:**
   - **Coverage ceiling** (reachability analog): the suite share the gate SERVES (answered, not abstained) at the operating point — for classification suites there is no enumeration stage; the gate's abstention IS the reachability cut.
   - **Support ceiling** ĝ: the adjudicated as-typed gold-support estimate.
4. **Ceiling-adjusted score = a per-item JOIN, never a division:** the lane's score computed on the `supported` subset only, reported beside the raw score — raw stays the headline (comparability law). Dividing raw accuracy by ĝ is wrong by construction here: lanes can agree with unsupported gold, and a division would credit that agreement. The `correct ∧ not_supported` count rides beside the adjusted score as its own disclosure.
5. **Per-stratum support rates** where suites have natural strata.

## The adjudication contract (required BEFORE the probe runs — never mid-run)

Riding OntoSmith's proven arc (their contract scored 0.9765, then a pre-registered cross-family recheck failed its 95% bar, localized six seam classes, re-judged, re-priced 0.9765 → 0.9110 — and the ruled contract's own cross-family κ was re-measured in an addendum: 0.4271 three-category / 0.4723 decisive-only, both reported):

- **Blind:** the adjudicating judge never sees lane scores, gold provenance, or stratum.
- **As-typed:** the question is whether the fixture's context + option set supports the gold label AS TYPED, not whether a nearby reading is plausible.
- **Verbatim-quote verification:** a `supported` verdict counts only with a quote from the fixture text; quote-failed supported verdicts are recorded `unknown`, excluded from every denominator, counts published.
- **Controls beside-not-inside:** known-clean and known-flipped items shuffled in blind; reported beside the estimate, never inside it.
- **Verdict vocabulary:** `supported / not_supported / unclear` + evidence quote.
- **Seam rulings:** ambiguity classes discovered during the first pass are settled as NAMED rulings with machine-readable codes (`seam_applied`), so the seam population of future corrections is measured, not guessed. The contract text changes by pre-registered discipline, never mid-run.
- **Judge-validity gate (pre-registered, BEFORE any ĝ is reported):** the CI covers sampling error only — judge noise is a separate, potentially dominant error source (OntoSmith's ruled contract still measured κ ≈ 0.43; their planted-control catch 29/40 is the published failure example). The probe therefore runs a planted-flip control set first and reports **control-catch recall against the pre-registered bar: ≥ 0.85** (pinned above the 29/40 = 0.725 failure example); the written contract must carry this number before the first cell runs, and the probe refuses to start without a contract-stamped bar. A judge below the bar does not get to price gold, whatever its CI.
- **Stratified design effect:** the sample is stratified, so the report carries the design effect (deff) alongside ĝ; a deff > 1 raises the effective sample-size requirement above the n ≥ 400 floor (n_eff = n / deff must still clear the half-width bar).
- **Cross-family recheck:** a pre-registered agreement bar; a judge from a different model family re-judges a stratified sample; on a failed bar, localize → rulings → re-judge → re-price. Report κ both three-category AND decisive-only (raw agreement overstates at skewed base rates).

## Hard laws

- **grammar-not-answers applies:** adjudication outputs are eval-side only — never enter corpora, retrieval, or training at any proportion.
- **No self-judging:** the adjudicating judge seat must NOT be a scored entrant on the suite it adjudicates — a hosted comparison lane can resolve to a JDI entrant (jev included) that is itself scored on these suites; a judge grading an exam it sits is disqualified, loud.
- **Two ceilings never pooled:** coverage-at-operating-point vs gold-support. A report that merges them is a bug, not a summary.
- **Class prior art must be cited** in any output doc (Northcutt arXiv:2103.14749; Tan/ReDocRED arXiv:2205.12696; Klie arXiv:2405.11919; Nahum arXiv:2410.18889 — machine-adjudication of gold exists published; our composition: sampled CI'd ceiling + two-ceiling accounting + per-item fate census, in the harness shapes above).
- **Judge seat:** use an existing lane posture (hosted comparison lane or laya) subject to the no-self-judging law — the probe prices gold, it is not a new model claim. The judge's identity + contract version stamp every ceiling number (calibration identity law: a ceiling estimated under another judge/contract never answers for this one).

## Proof path (pre-registered)

First cell: ONE suite (sst5 or massive_intent_en), stratified sample **n ≥ 400** (the bar is a Wilson half-width ≤ 0.05; at p ≈ 0.5 that needs n ≈ 385 — n = 200 caps out at ≈ ±0.069 and cannot meet the bar; OntoSmith's own ĝ_B interval ±0.049 is what n ≈ 400 buys), one frozen read, judge-validity gate green first. **Bar:** ĝ lands with CI half-width ≤ 0.05 AND control-catch recall ≥ its pre-registered bar (else the sample or the judge is too weak to read scores against — either outcome is a result). Secondary consumer noted: the same probe shape prices the clippy corpus's labeled eval fixtures (riir-refine) before retrieval-floor reads — file there separately if the first cell lands.

## References

- reflex Research 015 (the distill; prior-art record §5, signal-diff §6)
- OntoSmith @ `1412da4d` (MIT): `docs/whitepaper.md` §5.3–5.6, `docs/ADJUDICATION.md`, `pipeline/identity_seat.py` (calibration-identity ledger key), `pipeline/operating_points.py` (vacuity guard)
- reflex grammar-not-answers skill (the ingestion-side twin)
- reflex Bench 141 + fix `724b5df` (the keymap gold pins — label-mapping axis, the closest existing cousin)
