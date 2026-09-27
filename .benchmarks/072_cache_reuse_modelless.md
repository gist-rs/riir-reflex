# Bench 072 — `harness_cache_reuse` answers modelless (Issue 045 T1–T3 reflex half)

**Verdict: POSITIVE — the modelless lane flips the suite.** Forced accuracy
**0.9167 (11/12)** vs the LLM lane's published **0.5000** (Bench 001 Addendum
6 — chance with a constant confident collapse); single test read, predictions
in `results.json`. G2 p50 **0.009 ms**; determinism: answers/probabilities
bit-identical across three runs (the envelope's wall clock + box load differ —
the volatile fields, not the lane).

## The lane

The T3 carve-out ("LLM-lane ONLY — the modelless lane has no KV cache") is
reversed on Issue 045's two premises, both verified here: the family is
text-decidable (every fixture's gold is programmatic from the described
coverage relation), and the LLM lane had no winner to protect (0.5000 =
chance). The 12 eval fixtures and their gold are UNCHANGED — the reversal
adds only what the other five families always had: a per-class corpus and a
cal front.

The lever is the **noul count-table polarity** (issue 038's
`nb_noul_domain`), because noul never takes route terms or heads by the
issue-030 law (its `[yes, no]` pair is question vocabulary — arming route
there measured 0.4397, below chance). The corpus docs therefore reach the
decision through the NB tables, not the drafter, and the polarity is
SELECTED on the authored cal front, never fixed:

- cal front (20 authored cases, 10/class): scale 0 (off) 0.5000 · polarity
  yes→domain 1 **0.7500** at every scale/α · polarity yes→domain 0 0.2500
  (inverted, below chance — the tables genuinely separate the classes).
- Selection arms **scale 1, observed-laplace, yes→domain 1** (+25 pt over
  off, clearing the +5 pt promotion bar; the flat across scales says the
  signal saturates at one σ-term — the minimum dose).
- Test read ONCE at the selected posture: **0.9167**.

## Enabling changes (reflex-side, Issue 045)

1. `synth_cache_reuse` ships the authored corpus (12 docs, 6/class) + cal
   front (20, 10/class), pairwise-disjoint from the eval texts (the
   self-inclusion law, gate-asserted). `CACHE_REUSE` joins `FAMILY_DEFS`.
2. `harness_cache_reuse` → `modelless_lane: true` (the T3 skip stays as
   declared-inability machinery; unreachable over the current registry).
3. `selection_slice` carries a synthetic fallback: with no pool rows, the
   labelled selection slice IS the authored cal front (the same labelled
   data the fused-gate thresholds fit), pool = the family docs minus any
   cal text. Before this, ANY select knob on ANY synthetic suite errored
   with "empty stratified slice" — the fallback enables the selection
   machinery suite-wide without touching any dataset path.
4. NB-selection eligibility extends to synthetic suites (heads stay
   dataset-only — the families' choice-route baseline rows are the harness
   sanity pins; cache_reuse's lever is the polarity, not heads).

## Gates

`tests/harness_families_gates.rs` 7/7 — cache_reuse joins the registry /
count / disjointness / gold gates; the engine smoke is noul-aware (one wire
probability, p_yes ∈ [0,1], `Outcome::Noul` pick); the LLM-only gate is
REPLACED by `cache_reuse_grounded_posture_discriminates`, which runs the
PRODUCTION seat path (`prepare_seat` → `fit_posture{nb_select}` →
`build_seat_engine`) and asserts: the selection arms yes→domain 1, and the
grounded engine discriminates (≥2 distinct picks over eval). Grounded forced
accuracy at the gate: 11/12. `tests/harness_seat_gates.rs` 5/5 — cache_reuse
seats as synthetic, byte-identical questions, cal + corpus present.

## G1 power statement (the 045 T2 convention)

n = 12 eval questions: the 95% CI on 0.9167 (Wilson) is [0.646, 0.985] — the
row cannot statistically separate from 0.75, and the fused-gate cal→test
transfer is asserted on a 20-case cal front. The row is published as a
flipped suite with wide error bars; the suite has no accuracy claim to
defend (the gates pin discrimination + grounding, never the accuracy). ECE
(maxp) 0.2868 raw — the 11/12 confidence mass concentrates above the
empirical rate; no calibration claim (the calibrated-vs-floor comparison is
meaningless at this n and is not asserted).

## Box state (Issue 021)

M3 Max, AC power, High Power mode, load 3.45–3.62, release profile. Quotable
per the harness's own disclosure (the p50 is three orders under the bar;
load swings cannot reach it).

## What remains (owed)

- **Lane-scoped publish** (045 T3): this record + `results.json` are the
  reflex-side input; the reflex-site half (bench.json lane row + the arena
  TL;DR's 14th row via `pick()`) belongs to the session holding the
  reflex-site checkout — same standing as 044 T5's pending py publish.
- **Instinct follow** (consumer): `harness_cache_reuse` now seats — the
  instinct arena's synthetic population can add it (the seat refusal it
  recorded in Bench 011 is gone upstream).
