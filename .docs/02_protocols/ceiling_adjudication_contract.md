# The gold-support ceiling adjudication contract

**Contract version:** `CEILING_CONTRACT_V2` (the stamp the `--ceiling-probe`
arm refuses to start without; changes land here first, by pre-registered
discipline — never mid-run, the OntoSmith meta-rule). V1 stands as the
stamp of the two frozen V1 cells (`.benchmarks/108_ceiling_first_cell/`,
`.benchmarks/108_ceiling_cross_family/`); V2 = V1 + the §V2 amendment
below, committed BEFORE any V2 judge call.

**Status:** the Issue-108 contract of record. **V2 cell LANDED 2026-11-06 — κ re-price FAIL (0.4496 < 0.70)**; per the V2 clause below this is the recorded negative: the local seat ladder cannot price sst5 gold at the bar, every ĝ is a seat-conditioned reading only, and the next rung (hosted seat or a third local family) needs a V3 edit first (`.benchmarks/108_ceiling_v2_rejudge/README.md`). Issue 108 closed. The probe arm
(`--ceiling-probe`) reads this document's pinned constants
(`CONTRACT_VERSION`, the judge-validity bar, the sampling floor) from code
(`src/harness/runner/ceiling_probe.rs`) — the document is the source, the
consts are the machine-readable projection; they move in ONE commit.

## Purpose (the one-paragraph statement)

Reflex harness suite scores are gold-absolute: a lane scoring 0.42 on sst5
reads as model failure against a ceiling of 1.0. But benchmark gold itself
carries noise — labels the fixture text does not support as typed. Before
calling any lane score (ours or an entrant's) "low", this contract prices
how much of the gap is the benchmark's own gold: a machine-adjudicated,
CI-bounded **gold-support ceiling** ĝ, with a closed-set per-item fate
census and two ceilings (coverage, support) that are never pooled.
Adjudication outputs are **eval-side only** — they never enter corpora,
retrieval, or training at any proportion (the grammar-not-answers law).

## The judge seat

**No self-judging:** the judge must NOT be a scored entrant on the suite it
adjudicates. Every reflex lane (modelless, laya, the hosted comparison
lanes that resolve to JDI entrants) is disqualified where it sits the
suite. The seat ladder (first legal rung wins; the identity + contract
version stamp every ceiling number — a ceiling estimated under another
judge/contract never answers for this one):

1. **PRIMARY — Bonsai-27B (Ternary-Bonsai-2-27B-PQ2_0, our trained ternary
   SFT)** served over loopback HTTP (`CEILING_JUDGE_URL`,
   OpenAI-compatible `/v1/chat/completions` shape; on-box proven server:
   the league fork `llama-server` build 838 @ `d7b2896`, full offload).
   Owner-preferred model; ours (no license entanglement); never a scored
   entrant on any harness suite. Measure-side only.
2. **FALLBACK — Qwen3-8B** (on-box, `.raw/models/Qwen3-8B`, Apache-2.0,
   standard causal arch), same client, same prompt. A different model
   FAMILY from the primary — it doubles as the cross-family recheck seat.
3. **HOSTED seats are owner acts** — not this contract's operative path
   (keys + ToS re-fetch are owner-gated); named here only as the future
   escalation if both local rungs fail their validity gates.

**Cross-family recheck (pre-registered):** the primary's ceiling is
re-priced by the OTHER family's seat on a stratified sub-sample; agreement
bar: κ ≥ 0.70 three-category (decisive-only κ reported beside, both
conventions — the OntoSmith dual-κ law; their ruled contract measured
0.4271/0.4721, so both numbers always travel). On a failed bar: localize
→ named seam rulings → re-judge → re-price; the correction changes THIS
document first, never mid-run.

**Judge noise is effect-sized:** the CI covers sampling error only. The
validity gate below bounds judge noise before any ĝ is reported; a judge
below the bar does not get to price gold, whatever its CI (the OntoSmith
failure example: planted-control catch 29/40 = 0.725 — our bar sits above
it pinned).

## The adjudication question (as-typed law)

For one gold item (suite case + its gold label): **does the fixture's own
context + option-set text support the gold label AS TYPED** — not whether
a nearby reading is plausible. The judge is BLIND: it never sees lane
scores, gold provenance, sample strata, or the suite name's benchmark
identity beyond what the fixture text itself carries.

## Verdict vocabulary + verbatim-quote verification

- Verdicts: `supported` / `not_supported` / `unclear`, plus the evidence
  quote (JSON: `{"verdict": "...", "quote": "..."}`).
- A `supported` verdict counts ONLY with a quote that occurs VERBATIM in
  the fixture text (exact substring after whitespace normalization —
  multiple internal spaces collapse, case-sensitive, no other edits).
  Quote-failed `supported` verdicts are recorded `unknown` — excluded
  from every denominator, counts published (the denominator floor line
  prints every run).
- `not_supported` requires no quote (absence is not quotable);
  `unclear` requires no quote. Both stay in the support denominator as
  non-supported readings of the item — `unclear` is disclosed separately,
  never pooled into `not_supported`.

## Judge-validity gate (runs FIRST — pre-registered bar)

Planted controls shuffled in blind beside the sample (never inside it) —
the TWIN design, self-contained per suite (no hand-authored fixtures):

- **known-supported control**: a real sampled item presented with its TRUE
gold — if the text supports the gold as typed, the correct verdict is
`supported`. (The twin pair's supported-half doubles as the positive
class: no separate hand-authored fixtures needed.)
- **known-flipped control**: the SAME item with the gold SWAPPED to a wrong
option (sentiment/far-class flips only — the text cannot support the wrong
label as typed); any `supported` verdict on a flipped control is a false
positive.

**Bar: control-catch recall ≥ 0.85** on the flipped set (pinned above the
0.725 failure example), reported beside the known-supported catch rate.
The probe REFUSES to start its sample pass without the validity gate green
in the same run (the gate is part of every run — controls are cheap, the
discipline is not optional). The control counts ride the report beside the
estimate, never inside it. Flip pairs are drawn seeded from the sample's
own items (far-class flip: max gold distance under the suite's option
order), never from the test-side answer key of another suite.

## Sampling law

- First cell: ONE suite (sst5 — the issue's named candidate; the honest
  seat target whose board row reads 0.4217 gold-absolute), stratified
  sample **n ≥ 400** (Wilson half-width ≤ 0.05 at p ≈ 0.5 needs n ≈ 385;
  n = 200 caps at ±0.069 and cannot meet the bar). The sample draws from
  the suite's stratified eval front via the harness stride law — the
  front is already label-stratified, so the stride keeps proportionality.
- ONE frozen read: the cell runs once; no threshold iteration against the
  result; a re-run under a changed contract is a NEW record, never a
  replacement.

## The fate census (closed set, exactly one fate per sampled item)

- `correct ∧ supported` — lane right, gold text-supported.
- `correct ∧ not_supported` — lane agrees with gold the text does not
  support: the shared-bias / memorization cell (the strongest new signal).
- `wrong ∧ supported` — lane error on supported gold (real model failure).
- `wrong ∧ not_supported` — lane disagrees with unsupported gold
  (defensible error).
- `abstained` — the gate's own abstention (reflex's first-class answer;
  separate from wrong, never folded into it).
- `unknown` — quote verification failed or the judge returned
  `unclear`/malformed; excluded from every denominator, counted loud.

`supported`/`not_supported` read from the ADJUDICATED verdict, never from
the gold's own claim.

## Two ceilings, never pooled

- **Coverage ceiling**: the share of the suite front the gate SERVES at
  its operating point (answered, not abstained) — reachability.
- **Support ceiling ĝ**: the adjudicated as-typed support rate with
  Wilson 95% CI over the adjudicated denominators; the stratified design
  effect (deff) rides beside it — deff > 1 raises the effective-sample
  requirement (n_eff = n/deff must still clear the half-width bar, else
  the report says so).
- **Ceiling-adjusted score = a per-item JOIN, never a division**: the
  lane's accuracy computed on the `supported` subset only, reported
  beside the raw score — raw stays the headline (comparability law).
  Dividing by ĝ is wrong by construction (lanes can agree with
  unsupported gold; a division would credit that agreement). The
  `correct ∧ not_supported` count rides beside the adjusted score.

## Seam rulings (named, machine-readable — the registry starts here)

| code | ruling |
|---|---|
| `seam_quote_whitespace_normalized` | quote matching collapses internal whitespace runs before the exact-substring test (case-sensitive; no other normalization) |
| `seam_option_text_in_context` | the option-set text counts as fixture text for quote verification (the label words often live in the options, not the context) |
| `seam_unclear_not_supported` | `unclear` items are non-supported in every denominator, disclosed separately, never pooled with judged `not_supported` |
| `seam_controls_beside_not_inside` | control verdicts never enter ĝ's numerator or denominator |
| `seam_served_answer_counts_once` | an abstained item has ONE fate (`abstained`) — the judge never runs on it (coverage ceiling owns it) |

A first-pass disagreement class not settled here lands as a NEW named row
before any re-price — measured, not guessed.

## V2 amendment (2026-11-06 — the LOCALIZE-derived rulings, pre-registered)

**Why:** the V1 cross-family recheck FAILED its bar (κ 0.6387 < 0.70,
`.benchmarks/108_ceiling_cross_family/KAPPA.md`); the LOCALIZE cell
(`LOCALIZE.md` there) named the disagreement mass: 12/22 grade-threshold
(very-X vs X — both seats polarity-correct, on OPPOSITE sides of the grade
line by pole), 4 ambiguous neutrals, 4 Qwen3-8B errors (2 strict, 2
lenient), 3 Bonsai seat-starved re-reads. The contract's arc is
localize → rulings → re-judge → re-price; these are the rulings.

**Delegated-owner verdict (recorded per the owner's standing delegation):**
the rulings below are ACCEPTED as the V2 edit; the κ bar **does NOT move**
(0.70 stays — moving a bar after reading a miss is the forbidden shape;
the precedent is OntoSmith's own 95% line held through their re-price);
the seat ladder stays (Bonsai PRIMARY — owner-preferred; Qwen3-8B FALLBACK
+ cross-family seat). The S-B-STRICT / S-B-LENIENT items are SEAT errors —
no ruling can fix a seat error, and none is attempted (no ruling is
written to move a specific item).

New seam rows (in the prompt the seat reads — a ruling the seat never sees
cannot change its reading):

| code | ruling |
|---|---|
| `seam_ordinal_grade_contradiction_required` | ordered intensity scales: judge polarity first; an intensity grade (very-X or plain X) is `supported` when the text's polarity matches and the text carries NO explicit marker of a different same-side intensity (a hedge/mild wording against very-X; an explicit superlative/intensifier against plain X) — never rejected merely because intensity is unstated. Resolves S-GRADE-THRESHOLD to one shared line instead of two seat-private ones. |
| `seam_ordinal_middle_mixed_supported` | a middle (neutral) label is `supported` when the text is mixed, descriptive, or only weakly polar; `not_supported` only when one polarity clearly dominates. Resolves S-AMBIGUOUS-NEUTRAL. |
| `seam_seat_starved_unknown` | seat-ops: `max_tokens` 256 → **1024** (a reasoning-channel seat starved `content` at finish=length on ~14% of re-reads); every item records the seat's `finish_reason`; an empty-content reply at finish=length stays `unknown` in every denominator and is DISCLOSED as `unknown_seat_starved` (the seat cost, never gold quality). |

Both new rows apply only where the options form an ordered scale (the
prompt says so conditionally) — a nominal suite (intents) reads as V1.

**Direction disclosure (before the run):** both ordinal rulings resolve the
boundary toward `supported`, so V2's ĝ is expected to rise vs V1 on both
seats. That is the rulings' stated semantics (as-typed on a graded scale =
polarity-consistent and uncontradicted), not a lane credit — the
ceiling-adjusted score is still the per-item JOIN, never a division.

**The V2 re-judge cell (pre-registered):** BOTH seats re-judge the same
sst5 posture as the V1 cells (whole 600-item front, 40 twin controls,
default seed, the lane half unchanged — bit-exactness of the lane half
across boxes is the precondition and is re-asserted from the thresholds).
Validity gate first per seat (bar 0.85, unchanged). Re-price = κ
three-category over the both-adjudicated set via the committed
`ceiling_kappa.py` (decisive-only beside it). **Bar: κ ≥ 0.70.** PASS →
the V2 ĝ of the PRIMARY seat is the ceiling of record, the fallback's ĝ
beside it. FAIL → recorded negative: the local seat ladder cannot price
sst5 gold at the pre-registered agreement; the only remaining rung is a
HOSTED seat (an owner act, §judge seat); both V2 ĝs stand as
judge-conditioned readings only. Diagnostic beside (never the bar): the
V2 readings on the 22 V1-disagreement items.

## Machine-projection constants (the arm reads these; ONE commit with this doc)

```text
CONTRACT_VERSION     = CEILING_CONTRACT_V2
CONTROL_CATCH_BAR    = 0.85
MIN_SAMPLE_N         = 400
CROSS_FAMILY_KAPPA_BAR = 0.70
JUDGE_MAX_TOKENS     = 1024   (V2; V1 = 256)
```

## Prior art (cited per the issue's hard law)

Northcutt et al. arXiv:2103.14749 (label errors distort benchmarks);
Tan/ReDocRED arXiv:2205.12696 (~65% missing gold); Klie et al.
arXiv:2405.11919 (CI'd annotation-error sampling); Nahum et al.
arXiv:2410.18889 (machine-adjudicated gold + rescoring); OntoSmith
@ `1412da4d` (MIT) `docs/whitepaper.md` §5.3–5.6 + `docs/ADJUDICATION.md`
(the contract's ancestor; its self-correction arc is the pre-registration
template). Our composition: sampled CI'd ceiling + two-ceiling accounting +
per-item fate census in the harness shapes (the issue's novelty record).
