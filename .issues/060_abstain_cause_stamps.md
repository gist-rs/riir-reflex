# Issue 060 — record the abstain CAUSE per case (the abstention-reason mining unblock)

Status: OPEN — unblocks reflex-site plan 001 task 6

## The ask

The harness serves every decision with abstention as a first-class answer, and the
site publishes the abstain RATE (`calibrated_abstain.abstain_rate` /
`raw_abstain` on the modelless cells). What it never publishes is WHY an answer
was withheld — and the distinction is load-bearing for the board's honesty story:

- **score-gate abstain** — the calibrated fused gate's confidence threshold
  declined (the selective-accuracy story: right when it answers, silent when it
  can't);
- **corpus-distance-gate abstain** — the `CorpusDistanceGate` found the input too
  far from every reference corpus (the out-of-domain story);
- **grammar-invalid / foreign-question abstain** — the question shape itself fell
  outside the lane's contract (the game-heads lane's fall-through arm).

Three different claims, one number today. The Jev Decision Index distill
(reflex-site `.plans/001_jev_decision_index_distill.md`) mined exactly this
mechanic from their `answer_gaps.py` (refusal-as-wrong + per-cause shares); our
version is the abstention-first-class story's missing half.

## The shape

Harness side (this repo — `src/harness/`):

1. Record the abstain CAUSE per case in the hard-metrics block (a new structured
   field beside `calibrated_abstain`, e.g. `abstain_causes: {score_gate: n,
   distance_gate: n, grammar_invalid: n}` per suite) — never a prose string, the
   `cases_digest` pattern.
2. Byte-stable: a re-run with no code change must produce identical cause counts
   (the determinism law extends to the new field).
3. `--skip-laya` and every suite shape covered; the counts are per suite, summed
   by nothing else.

Site side (reflex-site, NOT this issue's work):

4. `publish_bench.py` aggregates the per-suite causes into the areas block;
   `bench/index.html` renders an answered-rate bar + per-cause shares beside the
   existing abstain column.

## Constraints

- No new model candidates (reflex-site plan 001 constraint 2 stands).
- The cause taxonomy is CLOSED: adding a fourth cause is a wire change with its
  own record, not a publisher-side fallback string.
- Cells that predate the field publish as unattributed (`causes: null` → the
  page renders "not recorded for this run"), never guessed.

## Refs

- reflex-site `.plans/001_jev_decision_index_distill.md` task 6 (the Claude
  verdict deferred exactly this half: "publishing a cause share the harness
  never recorded would be a made-up number").
- `.docs/02_protocols/laya_bench_protocols.md` (the metrics the block extends).
