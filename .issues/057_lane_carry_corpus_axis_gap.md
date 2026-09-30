# Issue 057 — LANE_CARRY has no corpus-axis escape: a corpus change can never refresh published timing

**Status:** OPEN — **owner-gate DECIDED 2026-09-30 (Claude verdict round 1: REVISE — all fixes folded)**; **harness half LANDED** (riir-reflex, this commit); publisher half SPEC'D below for the reflex-site session (outside this session's writable workspace). The specimen is live on reflex.gist.rs today.

## The specimen (measured, quotable both sides)

The site's `typed_decisions` m3 modelless latency cell reads **p50 0.517 ms**,
LANE_CARRIED (published cell `source_run d4051c8` 2026-09-29; the original
measure is Bench 076 `7e1bb07`, 2026-09-28) — measured on the **800-row train
pool**, before Issue 052's cap lift. Since Bench 078 the accuracy cells beside
it measure the **1200-row pool** (hard 0.5725): the oc/nb count tables the
per-question matvec reads grew ~50%, and every quotable read on the new pool
prices the same hot path at:

| read | p50 ms | box state |
|---|---|---|
| Bench 078 (the lift's own run) | 0.815 | quotable both spans |
| Bench 096 (2026-09-30, load 3.92→4.16) | 0.764 | quotable both spans |

The published cell understates the current corpus by **~35–48%**, and has
since 078's publish. It is disclosed on the cell (`latency_provenance` =
LANE_CARRY note) — the number is not silently wrong, but it is wrong.

## Why there is no path (the gap, code-verified 2026-09-30)

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

The cell's `corpus_cap` field records the per-label cap registry value
(`{"effective": 48, "source": "registry"}`) — identical before and after the
lift (the pre-lift pool simply under-filled the cap; the post-lift pool
fills it). It is a config echo, not a corpus identity.

## The amendment — owner-gate VERDICT (2026-09-30, round 1: REVISE)

Proposed: harness records a corpus pool digest per suite row; publisher
skips the carry when the digests differ. **Verdict: REVISE** — the direction
is right, with four load-bearing corrections (all folded):

1. **BLOCKING — the original "missing digest on either side → carry" rule
   protects the defect it was filed for**: the incumbent cell has no digest,
   so it carries forever and the stale 0.517 never refreshes. FIX: a
   one-time ack, `PUBLISH_BENCH_CORPUS_RESET=<suite>`, in the
   population-reset style — exempts the named suite from the carry, loud
   note, and the same stale-ack refusal (refuses unless the update cell
   carries a `corpus_digest` the incumbent lacks or contradicts). The
   republish comes from a NEW post-landing run on a fit box (Bench 096's doc
   has no digest).
2. **BLOCKING — the publish wall would miss exactly these slots**:
   `_latency_slots` skips a carried-class slot on the
   `carry_beats_incumbent` predicate, assuming the carry will replace the
   timing. An exemption living only in `apply_lane_carry` lets an
   unquotable update on a changed corpus dodge the wall as "will be
   carried" and then not be carried — unjudged timing reaches the page.
   FIX: ONE shared predicate (`carry_applies(src_lane, cell,
   update_quotable)`) covering equal source runs, unfit incumbents, AND
   corpus mismatch; BOTH call sites use it; a test arm pins the wall still
   refusing an unquotable update when the corpus differs.
3. **SHAPE — compare per cell, not per suite row**: `extra_host_lanes`
   slots can come from runs on a different corpus; a row-level comparison
   answers wrong for every non-primary host. FIX: stamp `corpus_digest`
   onto the CELL (the `cases_digest` stamping pattern) and compare
   `target["corpus_digest"]` vs `src_lane["corpus_digest"]` inside the
   per-slot loop.
4. **SHAPE — the field lives on the modelless `LaneResult`, not
   `SuiteResult`**: modelless is the only class in
   `LANE_CARRY["latency"]`; on SuiteResult it would be meaningless for
   LLM-only families. `Option<String>` + `skip_serializing_if`. And hash
   the pool ACTUALLY TIMED — the capped drafter docs (+ self-doc fallback)
   plus the uncapped count-table sets when the posture arms them — not the
   full pool plus a cap number.

Agreed as-is: both-same-corpus stays a carry; unfit-incumbent and
population-reset paths untouched; the same-texts-same-counts residual blind
spot accepted (then the timing identity genuinely did not move); when the
exemption fires on an unquotable update the cell keeps its own
`latency_quotable: false` stamp and the wall must refuse it.

## Tasks

- [x] Owner call on the amendment — **DECIDED** (verdict round 1 above;
      REVISE folded).
- [x] Harness corpus digest per lane (the `cases_digest` pattern, per the
      verdict's shape): `LaneResult.corpus_digest: Option<String>` —
      FNV-1a 64 (ungated-safe; blake3 is not in the tree at
      `--no-default-features`) over a shape tag, the suite name, and the
      consumed corpus per label (capped drafter docs, self-doc fallback
      included, plus the uncapped nb table sets when armed). The timed
      docs come from `per_label_corpus_docs` — the ONE construction
      `specs_from_pool` itself consumes, so the digest provably hashes
      what the engine builds on. Laya lanes carry `None`. Tests: 6 arms
      in `corpus_digest_tests` (determinism; pool-growth sensitivity at a
      fixed cap — the 057 specimen axis; cap sensitivity; nb-arm
      sensitivity; content + suite-name sensitivity; the starved-label
      fallback digested + disclosed). lib 225/225; harness_units 33/0 +
      seat 5/0 + families 7/0; clippy `-D` clean at default AND
      `--no-default-features`.
- [-] Publisher: the `apply_lane_carry` corpus-mismatch exemption + the
      shared `carry_applies` predicate at BOTH call sites
      (`apply_lane_carry` + the `_latency_slots` publish wall) + per-cell
      stamping + self-test arms (including the wall-refuses-unquotable
      arm) + the one-time `PUBLISH_BENCH_CORPUS_RESET` ack with its
      stale-ack refusal. **Deferred to the reflex-site session** — that
      repo is outside this session's writable workspace; the full patch
      spec is this issue's next section.
- [-] Re-publish typed timing (0.764-class) from a NEW post-landing run
      on a fit box with the ack. **Deferred with the publisher half** —
      Bench 096's doc carries no digest; the reproducing run must be
      post-landing (box state recorded beside the number).

## Publisher patch spec (for the reflex-site session — the verdict's corrected shape)

File: `scripts/publish_bench.py` + `scripts/test_publish_bench.py`.

1. **One shared predicate** (beside `carry_beats_incumbent`):

```python
def carry_applies(src_lane, target_cell, update_quotable, suite_name=""):
    """True when the incumbent timing must replace the update's, per the
    LANE_CARRY law. The ONE vocabulary both the carry loop
    (apply_lane_carry) and the publish wall (_latency_slots) read, so an
    exempted carry can never leak an unjudged slot past the wall.
    Exemptions, checked in order:
    - same source run (re-stamp would fabricate provenance);
    - unfit incumbent + quotable update (Issue-003 T2);
    - corpus mismatch: BOTH cells carry `corpus_digest` and they differ —
      carrying would re-attach stale timing by construction (Issue 057).
    Missing digests on either side are NOT an exemption (adoption-stable;
    the one-time ack below retires the stale incumbent)."""
```

2. **`apply_lane_carry`**: replace the inline `source_run`-equal check and
   the `carry_beats_incumbent` check with `carry_applies(...)`, reading
   the digests from the CELL pairs (`target` / `src_lane`) the per-slot
   loop already holds. On a corpus-mismatch exemption print the loud note:
   the population-reset rationale verbatim + "when the exemption fires on
   an unquotable update, the cell keeps its own `latency_quotable: false`
   stamp and the publish wall refuses it — a stale timing that looks
   quotable must not quietly become a fresh one that is unjudged".

3. **`_latency_slots` (the publish wall)**: skip a carried-class slot iff
   `carry_applies(...)` — the same predicate, so the corpus-mismatch slot
   is JUDGED by the wall instead of assumed-carried.

4. **Per-cell stamping**: wherever `stamp_cell` copies `cases_digest` /
   `source_run` / `latency_quotable`, also copy `corpus_digest` when the
   raw suite's modelless lane carries one (the results.json field is
   `suites[].modelless.corpus_digest`, `None` on laya lanes — skip
   `None`). The snapshot/merge paths that `setdefault` the
   `cases_digest` (≈L519 and ≈L644) get the modelless-digest analogue.

5. **The one-time ack** (population-reset style, same laws):

```python
# env PUBLISH_BENCH_CORPUS_RESET=<suite>[,<suite>…]
# - a named suite is exempt from the carry regardless of digest state
#   (its incumbent has no digest — that is the point);
# - STALE-ACK REFUSAL: a named suite whose update cell carries NO
#   corpus_digest, or whose digest EQUALS the incumbent's, exits 1 — an
#   ack cannot outlive the corpus change it was written for;
# - loud note per exempted suite.
```

6. **Self-test arms** (`test_publish_bench.py`): digest-equal → carry;
   digest-differs → no carry + update timing stands; either side missing
   → carry (adoption-stable); corpus-differs + unquotable update → the
   WALL refuses (the exemption must not create an unjudged slot); the ack
   exempts a digest-less incumbent; the ack refuses stale (no digest /
   equal digest) with exit 1.

7. **Then the republish**: run `typed_decisions` fresh on a fit box
   (`scripts/bench_preflight.sh` provenance line quoted beside it), publish
   with `PUBLISH_BENCH_CORPUS_RESET=typed_decisions`, verify the cell
   reads the new run's timing with `corpus_digest` present. reflex's
   `.benchmarks/.highwater` discipline applies to the new run's record.

## Why the amendment needed an owner call

The carry law is an owner call (Issue 032, 2026-09-26). Narrowing it is a
publish-semantics change affecting every future lane-scoped update. The
verdict session adjudicated per the standing "ask Claude for verdict on
owner-gated" rule; the folded shape above is the decision of record.
