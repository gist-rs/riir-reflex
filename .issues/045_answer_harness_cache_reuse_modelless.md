# Issue 045 — answer `harness_cache_reuse` from the modelless lane (retire the LLM-only carve-out)

**Status:** OPEN — filed 2026-09-27, out of the 044 close-out conversation ("can't we at least use an in-mem LRU cache on modelless?"). No lever work yet.

## Why

The arena renders `harness_cache_reuse · Reflex · modelless — not run`. That
is Issue 004 T3's recorded law: the family was declared LLM-lane-only
("the modelless lane has no KV cache … never a fake modelless answer"), so
the runner skips it modelless-side and the site shows an honest absence.

Re-reading the family against the shipped engine, the law's premise no
longer holds:

1. **The family is text-decidable, like the other five.** Every fixture is a
   third-person scenario — "The transcript holds the file read from turn 3;
   the user now asks about a symbol inside that same file. Reuse the cached
   prefix?" — whose gold (`reuse` vs `rebuild`) is programmatic from the
   described situation: does the described cached content still cover the
   described next turn. The lane never needs to OWN a cache to classify a
   described one, exactly as it answers "is this fn `pub`?" without being a
   fn. The self-referential reading (T3's "no KV cache") is weaker than the
   fixtures themselves.
2. **The family currently has no winner to protect.** The published
   laya·english row is **0.5000 at n=12 — chance**, with a constant
   confident one-class collapse (macro F1 0.333, ECE ≈ 0.36; Bench 001
   Addendum 6 / HISTORY). A modelless lane answering above chance flips a
   suite that today reads as a tie.
3. **The Lexical-pair machinery from 044 T3 applies directly.** The decision
   is a relevance/coverage judgment between a "prefix" side and a "coming
   turn" side — the `--nli-feature-ab` overlap/containment feature set and
   the pair-view count tables are the same shape. The expected modelless
   accuracy on lexically this distinctive a pattern (the five families'
   authored-fixture lesson: class-distinctive vocabulary carries these
   suites) is well above the 0.50 the lane stands at.

## What an in-mem LRU would and would not buy

An LRU answers **"what gets evicted from a bounded cache"** — a different
question from this family's **"does the prefix still match the task"**
(semantic coverage, not recency). An in-mem decision LRU on the modelless
SERVE path (memoize identical canonical decision requests — sub-µs repeats,
bounded memory) is a reasonable perf idea on its own merits, but it grounds
none of these 12 fixtures and must not be conflated with this issue. If
wanted, it is its own issue with its own G2/G4 gates.

## Plan

- [ ] T1 — lift the carve-out in code: give `harness_cache_reuse`
  `modelless_lane: true`, author the per-class corpus docs (the family
  ships none — the other half of the T3 exclusion), and let the standard
  noul machinery answer it. The fixture-design law applies (multi-sentence,
  class-distinctive vocabulary, no label-token leak — the existing 12 eval
  texts already comply).
- [ ] T2 — gates: G1 (the fused gate's cal→test transfer on a 12-fixture
  eval is thin — state the power explicitly, the Bench-001-Addendum-6
  convention), G2 sub-ms, determinism; the runner's loud-skip path becomes
  dead for this suite and its gate rows update.
- [ ] T3 — both lanes re-run, lane-scoped publish; the arena TL;DR regains
  its 14th row (`pick()` needs all three lanes; code_fixtures regains its
  py/english cell the same way — see the 044 close-out's pending py
  publish).
- [ ] T4 — HISTORY: the T3 law's reversal recorded with the measured
  verdict either way (if the modelless lane cannot beat 0.50 with grounding,
  that is the honest result and the family stays published as a no-winner
  suite).

## Non-goals

- No serve-path decision cache here (separate concern, separate gates).
- No change to `harness_cache_reuse`'s gold or fixture text (the 12 eval
  fixtures + their programmatic gold stay exactly as authored).
