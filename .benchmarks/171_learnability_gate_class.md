# Bench 171 — the Issue-104 learnability-gate class: NEGATIVE at every rung; the too-easy pole IS where the lift lives

**Status:** COMPLETE — measured 2026-10-30 (m3, datasets `.raw/datasets_t20k`, read @ this
record's commit). Pre-registration (primary rung p75, V5 law unchanged, honest-null posture)
filed in the issue claim **before any read** (`3596fca`, the claim push is the arbiter).

## The question (Issue 104, the one gate class Benches 120–122 never tested)

RT-J (arXiv:2610.17867 §3.3–3.4) filters generated tasks by margin over trivial baselines —
a task a trivial solver already aces carries no signal. Does the same gate class transfer to
the synth-corpus lane: does dropping the too-easy pole from the sealed ungated artifact make
the corpus lift the modelless row MORE than the ungated artifact does on the same frozen rows?

## T1 — the margin scorer (report-first; NOT vacuous)

Closed-form solvers over the sealed artifact (`.raw/corpus_synth/massive_intent_en_synth.jsonl`,
2,048 rows, blake3 `8fe10f8fabde6471…`), one pass each, no tuning:
**prior-frequency** (`prior(gold)/max_prior`), **lexical-overlap** (gold-token coverage − best
other-label coverage, clamped ≥ 0), **longest-option** (unique-longest label), trivial = max.

- At-ceiling: prior 512 (25.0% — exactly the four 128-per-label majority labels),
  longest 91 (4.4% — the one unique-longest label, `recommendation_events`; the degenerate
  solver being degenerate), lexical 0 (417 rows carry partial margins ≥ 0.1, none full).
- **603 rows (29.4%) sit at trivial = 1.0 — a real too-easy pole.** The gate class is live
  here, not vacuous. Full histograms: `.raw/corpus_synth_learn104/LEARNABILITY.md` +
  `learnability_report.json` (per-row margins — the report-only sidecar).

## T2 — the gated variants (derived filters, sealed)

Rows verbatim from the sealed artifact; fresh header (`accepted` = kept, `learnability_rule`
= the derivation; additive v2 field, the `density_rule` precedent); fresh blake3 sidecar;
each re-loaded through the sealed loader before any read. Threshold = quantile of the
trivial-margin distribution; drop rows at margin ≥ threshold, keep the rest:

| rung | threshold | kept | dropped | artifact |
|---|---|---|---|---|
| p50 | 0.7188 | 974 | 1,074 | `.raw/corpus_synth_learn104/p50/` |
| **p75 (primary)** | 1.0000 | 1,445 | 603 | `.raw/corpus_synth_learn104/p75/` |
| p90 | 1.0000 | 1,445 | 603 | — (identical keep-set to p75 ⇒ identical rows ⇒ identical read; skipped as redundant, not as unmeasured) |

## T3 — the frozen paired reads (one frozen test read per arm; echo gates both sides)

Instrument aliveness: the ungated re-read (`.raw/corpus_synth_learn104_ref/`) reproduces the
published row exactly (gold 0.7800 → synth 0.8133, LB95 +0.0110, the same 11/1 flips and the
same synth-only win IDs as Bench 0029's seated read) — byte-deterministic across the day,
same 300 frozen rows through the same seat posture.

| arm | rows | acc | LB95 vs gold | V5 | KL | OOD gate p=0.20 |
|---|---|---|---|---|---|---|
| ungated (comparator) | 2,048 | **0.8133** | **+0.0110** | PASS | 0.0000 | +0.0130 transfer-ok |
| p75-gated (PRIMARY) | 1,445 | 0.8067 | +0.0062 | PASS | 0.0000 | +0.0084 transfer-ok |
| p50-gated | 974 | 0.7833 | −0.0080 | **FAIL** | 0.0000 | +0.0016 no-clean-win |

## The class verdict (the pre-registered gate: paired LB95 gated − ungated > 0 at the primary rung)

**NEGATIVE at every rung.** Paired per-case synth-arm-vs-synth-arm over the same 300 frozen
rows (`scripts/corpus_ab_pair.py`; the gold arms are identical across reads — determinism
cross-checked):

| pair | gated-only wins | ungated-only wins | acc delta | paired LB95 | verdict |
|---|---|---|---|---|---|
| p75 − ungated | **0** | 2 (`:7`, `:256`) | −0.0067 | **−0.0159** | NEGATIVE |
| p50 − ungated | **0** | 9 | −0.0300 | **−0.0493** | NEGATIVE |

The gated corpus wins ZERO cases at any dose. The 11 ungated synth-only wins (Bench 0029's
list) live in exactly the rows the gate drops or dilutes — **the too-easy pole is not dead
weight; it is where the modelless engine's lift comes from.** The mechanism is visible in
the scorer's own output: the pole is the majority-label mass (prior-frequency at-ceiling =
the four 128-row labels) and the lexically-canonical rows — for an ICL retrieval engine,
a lexically-obvious row teaches the retriever the label's signature vocabulary, which then
generalizes to the eval rows the trivial solver canNOT solve. Dropping it removes signal,
not redundancy (RT-J's filter served gradient-descent pretraining, where trivial rows are
gradient noise; ICL retrieval wants them — the issue's own "does NOT assume RT-J transfers"
clause, resolved negative).

## Verdict

- **Learnability gate class: CLOSED NEGATIVE for this lane** — same posture as Benches
  120–122 for the density gates. The ungated sealed artifact stays the corpus of record and
  the SEATED production corpus is UNCHANGED. Re-open trigger: a lane whose learner is
  trained (not retrieved-against) — the class's premise holds there, per RT-J's own domain.
- Difficulty tiers survive as REPORT-ONLY: the p75 keep-set terciles split easy 306 /
  medium 599 / hard 540 (`learnability_report.json` carries per-row margins) — usable by a
  future lane for stratified reads without gating.
- Echo gates PASS on every rung that passed V5 (KL 0.0000 all; transfer-ok at p75) — the
  class does not echo, it simply subtracts value.

## Provenance + honesty

- `PROVENANCE: power=AC Power load=56.53 swap=1334.44M canary=skipped powermode=2(high)`
  — `bench_preflight.sh REFUSED` (MAX_LOAD=6.0; the Batch-320 kernel_opt mining session was
  running). **The verdict is accuracy-only**: the reads are deterministic engine evals over
  frozen rows (the ungated re-read reproducing 0.8133/+0.0110 under this same load is the
  aliveness proof), but the LATENCY columns are CONTENDED and quoted nowhere — re-measure
  any latency on a quiet box. Determinism true on all three reads.
- Leak scan (grammar-not-answers, run before the reads): content assertions PASS (registry
  cap bound holds; `massive_intent_en exact=30 near=77/1980` — the sealed era's own scale;
  the derived artifacts are pure subsets, so the leak surface only shrinks). Two timing-bar
  tests failed BY LOAD (`s1mb_choice` G2/T3 budget 1 s, took ~2.9–3.1 s at load 56.5) — the
  documented load-sensitive-bar class, not a leak finding; re-run the bars on a quiet box.
- Lane code: `src/harness/runner/learnability.rs` (9 in-module tests) +
  `examples/learnability_gate.rs`; the corpus-ab lane gained one additive disclosure
  (`synth_ok_cases`) so two reads pair without a third binary. p90 skipped as redundant
  (identical keep-set), disclosed above.
- Commands: `./target/release/examples/learnability_gate
  .raw/corpus_synth/massive_intent_en_synth.jsonl --out .raw/corpus_synth_learn104`;
  `./target/release/harness --corpus-ab <artifact> --datasets-dir .raw/datasets_t20k
  --synth-out <dir> --suites massive_intent_en` (×3);
  `python3 scripts/corpus_ab_pair.py <gated corpus_ab.json> <ref corpus_ab.json>` (×2).
