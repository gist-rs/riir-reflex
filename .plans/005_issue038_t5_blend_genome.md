# Plan 005 — issue 038 T5 steps 2+: the joint blend-genome selection

**Status:** COMPLETE — 2 adopted (banking77 +3.60, sst5 +0.50, both G1-pass), 2 rejected (massive G1-fail, emotion −53.5 accuracy collapse); RRF declined. Record: `.benchmarks/064_issue038_t5_genome.md`.

## Why

The shipped modelless posture was composed by four one-at-a-time greedy
ladders in a fixed order (head → nb → oc → ridge), each selected at the
then-current posture. Greedy order effects are exactly the interaction
class the joint search fixes: e.g. emotion selected ridge@8 at the
then-current nb posture; the joint walk can find nb@lower + ridge@8 (or
ridge off + nb@higher) beating it on the same cal slice. T7(d) (Hebbian
xnli) reopens when this lands.

## Design

- Genome = the scoring-time blend scales `{route, head, nb(+α,+view),
  oc, ridge}`. λ stays fixed (build-time, `RIDGE_LAMBDA=10.0`); noul
  polarity stays fixed (already cal-selected).
- Build ONCE per (α, view) with every lever FITTED (positive placeholder
  scales force the fits), then move scales at scoring time via the new
  `DecisionEngine::set_blend_scales` — a coordinate eval pays one
  sel-slice scoring pass, not an engine rebuild. Rebuilds happen only
  for the discrete α/view coordinates.
- Search: coordinate descent, ≤ 2 passes, each candidate must beat the
  coordinate's current best by `GENOME_MOVE_MARGIN = 0.01` (≈2 questions
  on a 200-case slice — the noise floor for a MOVE); ties hold current.
- Acceptance: the walk end must clear the seed by `HEAD_SELECT_MARGIN`
  (0.05, the lanes' own promotion bar) or the posture is HELD — the
  published posture stands, byte-identical (G3 by construction: the
  final engine is the plain rebuild at the untouched `default_cfg`).
- Protocol: cal slice only; test read once (the run's existing single
  read at the final posture). Genome-off runs byte-identical
  (`skip_serializing_if` on the new row fields).
- RRF (issue 038 T5 "optionally"): verdict DEFERRED to the genome
  results — the blend terms are already sigmoid-squashed onto a shared
  scale, which riir-rag's own RRF doc names as the case where additive
  fusion is strictly more informative. Decide from the walk; record the
  decision either way.

## Tasks

- [x] engine: `set_blend_scales` + `EngineError::ScaleNotFitted` (fail-closed) + neutrality test (fitted-but-zero picks == unfitted picks)
- [x] `genome_lane.rs`: GenomePoint/GenomeMove/GenomeSelection + pure `descend` + `build_genome_selection`
- [x] pure `descend` test (synthetic surface: converges to the ladder optimum; a sub-margin improvement holds)
- [x] runner plumbing: ModellessInput/RunOptions/fit_posture_inner/FittedPosture/LaneResult/seat + loud feature error
- [x] CLI `--genome-select` + `--genome-accept-margin` + TABLES.md disclosure (skip when None)
- [x] run genome-select over the dataset suites; compare vs published rows (052/057/062) — round 1 at the house bar 0.05: ALL HELD, held rows byte-identical to published (G3 proven); round 2 at the pre-registered refinement bar 0.03: 4 accepted
- [x] verdicts: banking77 +3.60 / sst5 +0.50 ADOPTED (G1 pass); massive +5.33 REJECTED (G1 fail — ECE .238 vs floor .136); emotion REJECTED (0.35, accuracy collapse — the house bar would have refused); bench record `.benchmarks/064_issue038_t5_genome.md`
- [x] RRF decision recorded — DECLINED (the blend terms share a σ scale; additive wins there by riir-rag's own RRF doc; emotion's collapse is posture overfitting, not scale incomparability)
- [x] issue 038 status update + HISTORY row + plan checkboxes; commit + push (4090 cross-host verify rides the post-041 whole re-run)

## Non-goals

- No drafter-scale coordinate (engine surgery; the drafter is the base σ
  score — route/head/nb/oc/ridge already reshape around it).
- No λ ladder (the ridge lane's documented O(k³) cost law).
- No noul-polarity re-search (discrete, already cal-selected; the α/view
  coordinates cover the table-shape axis).
