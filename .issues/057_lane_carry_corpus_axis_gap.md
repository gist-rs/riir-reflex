# Issue 057 — LANE_CARRY has no corpus-axis escape: a corpus change can never refresh published timing

**Status:** OPEN — owner-gated law amendment (Issue 032's owner call) + a harness-side corpus-identity field; the specimen is live on reflex.gist.rs today.

## The specimen (measured, quotable both sides)

The site's `typed_decisions` m3 modelless latency cell reads **p50 0.517 ms**,
LANE_CARRIED from Bench 076 (`source_run 7e1bb07`, 2026-09-28) — measured on
the **800-row train pool**, before Issue 052's cap lift. Since Bench 078 the
accuracy cells beside it measure the **1200-row pool** (hard 0.5725): the
oc/nb count tables the per-question matvec reads grew ~50%, and every
quotable read on the new pool prices the same hot path at:

| read | p50 ms | box state |
|---|---|---|
| Bench 078 (the lift's own run) | 0.815 | quotable both spans |
| Bench 096 (2026-09-30, load 3.92→4.16) | 0.764 | quotable both spans |

The published cell understates the current corpus by **~35–48%**, and has
since 078's publish. It is disclosed on the cell (`latency_provenance` =
LANE_CARRY note) — the number is not silently wrong, but it is wrong.

## Why there is no path (the gap)

`reflex-site/scripts/publish_bench.py::apply_lane_carry` replaces a
refreshed carried-class lane's timing with the host incumbent's values
**unconditionally**, except `carry_beats_incumbent` (incumbent verdict
`False` AND update quotable — Issue 003 T2). Both sides quotable → the
incumbent wins, by design (timing moves only through dedicated timing
publishes or unfitness replacement).

The one escape, `PUBLISH_BENCH_POPULATION_RESET`, fires only on a
`(n_questions, n_cases)` mismatch — the QUESTION axis. The typed corpus lift
changed the **CORPUS axis** (train pool → count-table size → per-question
compute) with the question set byte-identical, so:

- the reset cannot fire (typing the ack is refused as stale — no population
  mismatch exists);
- the carry is mandatory;
- therefore **no sequence of lane-scoped publishes can ever refresh the
  typed timing cell** while the incumbent stays quotable.

## Why no trigger exists today

The cell's `corpus_cap` field records the per-label cap registry value
(`{"effective": 48, "source": "registry"}`) — identical before and after the
lift (the pre-lift pool simply under-filled the cap; the post-lift pool
fills it). It is a config echo, not a corpus identity. There is no per-cell
digest of the train pool the corpus actually consumed.

## The proposed amendment (owner-gated — two halves)

1. **Harness (riir-reflex)**: record a corpus pool digest per suite row
   (BLAKE3 over the train pages/rows the corpus consumed, or the
   per-label effective doc counts — cheap, deterministic) into the
   results doc, carried onto the cell like `cases_digest`.
2. **Publisher (reflex-site)**: in `apply_lane_carry`, skip the carry for a
   cell whose corpus digest differs from the incumbent's — the population
   reset's own rationale verbatim ("carrying it would re-attach stale
   timing by construction; the update's own timing stands"), moved to the
   axis it was missing from.

Both-quotable-same-corpus stays a carry (the law's stability intent is
untouched); unfit-incumbent behavior untouched (Issue 003 T2); only the
corpus-changed cell's timing refreshes, and only when the update's own
timing is quotable.

## Why this is owner-gated

The carry law is an owner call (Issue 032, 2026-09-26). Narrowing it is a
publish-semantics change affecting every future lane-scoped update; the
amendment above is the minimal shape but the call is the owner's.

## Tasks

- [ ] Owner call on the amendment (this issue).
- [ ] If GO: harness corpus digest per suite (the `cases_digest` pattern).
- [ ] If GO: the `apply_lane_carry` corpus-mismatch skip + self-test arms.
- [ ] If GO: re-publish typed timing from Bench 096's quotable doc (0.764)
      — the reproducing evidence is already committed.
