# Plan 009 — wide template-disjoint eval for the six harness families

Status: IN_PROGRESS

Instinct Issue 008 T8 dropped the six harness decision-point families from the
specialist covered set: each eval is n = 12–16, template siblings of any
trainable corpus, so a specialist win is unfalsifiable memorization, and two
families sit at reflex's own near-ceiling (no headroom to certify against).
The recorded re-open condition: **a larger template-disjoint reflex-side
eval**. This plan lands exactly that, so a riir-train specialist can be
trained and certified (or honestly refuted) on a decisive read.

## Design

- **Replace** each family's `eval` slice with a wide population
  (target n = 96 per family, class-balanced, 88–104 accepted). One eval per
  family — the runner, seat, arena, and trainer all consume the same
  population, so certification stays coherent (pick − A0 on the SAME frozen
  read). The old 12–16-case evals remain in git history; `cache_reuse_eval()`
  keeps returning the frozen T3 12-fixture record (its gate pin stays true as
  a history pin — documented divergence).
- **corpus/cal UNCHANGED.** They are the trainable substrate and the
  calibration front; touching them would confound the A0 re-baseline with a
  second variable. Train-substrate expansion is the recorded follow-up lever
  if the first specialist starves.
- **Template-disjointness is a GATE, not an aspiration** (new test in
  `harness_families_gates.rs`):
  - content-token unigram overlap of each eval case vs the family's
    corpus∪cal vocabulary, pinned per-family ceiling (ceilings measured
    against the OLD eval's overlap stats — the new population must be
    measurably more disjoint than the one it replaces);
  - shared content 3-grams between each eval case and any single
    corpus/cal text, walled at a small pin;
  - label-token bans per family (the existing fixture law, now enforced);
  - BLAKE3 digest pin over all six wide populations (the non-goal law —
    gold and fixture text never move — made enforceable).
- **A0 re-baseline recorded, nothing served breaks.** The six families'
  A0 rows are record-only (`a0_stands`; the site cells were already dropped),
  so the re-baseline is low-risk. Instinct's arena re-derives both sides of
  its A0 drift pin by construction. ACC_FLOORS (0.8× chance anti-pathology)
  re-checked on the new eval; re-pinned only if a family reads below its
  floor (which would mean anti-correlation, a finding, not a nuisance).

## Re-open chain (what this unblocks)

1. This plan (reflex): the decisive eval. ← this work
2. riir-train: extend the code_fixtures trainer lane to prose families
   (per-head NBSVM-style over corpus+cal, winner mint, holdout read on the
   new eval) — filed as a riir-train issue, handoff at landing.
3. instinct: the winner_bridge/seat path re-seats a family the day a
   certified winner exists (the normal lanes; no instinct code change needed
   for the eval itself).

## Tasks

- [ ] Numbering: allocate `.issues/059` (reflex anchor issue), write
      `.plans/.highwater` = 9 (found stale at 7 while plan 008 exists on
      disk — real max 8, measured `ls` + git log)
- [ ] Author wide eval populations: 6 families × ~96 cases, class-balanced,
      label-token-free, template-disjoint (delegated to per-family
      subagents; assembled + reviewed here)
- [ ] New module `src/harness/families_eval_wide.rs`; rewire all six
      `FamilyDef.eval`; update CACHE_REUSE_NOTE + the T3 gate comment
      (documented divergence: the T3 static stays as the pinned history
      record, the consumed eval is the wide population)
- [ ] Gates: disjointness ceiling + 3-gram wall + label-token bans + digest
      pin in `harness_families_gates.rs` (ceilings set from measured old vs
      new overlap stats)
- [ ] Re-measure A0 on the wide eval; re-pin ACC_FLOORS / sens within-1 only
      if a floor reds; record cache_reuse grounded acc
- [ ] Full lane green: `cargo test` (families gates + lib) + `cargo clippy
      --all-targets -- -D warnings` at default and all-features postures
- [ ] Records: `.benchmarks/104_families_wide_eval_a0.md` (measured A0 rows
      + box state + overlap stats), reflex HISTORY.md entry
- [ ] Commit + push (develop); file the riir-train trainer-extension issue
      and reference this plan + the landing hash

## Non-goals

- No corpus/cal changes (single-variable re-baseline).
- No engine changes — A0 behavior is whatever it is on better data.
- No instinct code changes (its lanes consume reflex's synth as-is).
- No specialist training here (boundary law: riir-train trains).
