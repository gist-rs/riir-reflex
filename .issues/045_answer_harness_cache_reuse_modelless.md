# Issue 045 — answer `harness_cache_reuse` from the modelless lane (retire the LLM-only carve-out)

**Status:** REFLEX HALF LANDED 2026-09-28 — T1+T2 measured POSITIVE (Bench
072, `.benchmarks/072_cache_reuse_modelless.md`): the modelless lane reads
**0.9167 (11/12)** on the unchanged 12 fixtures vs the LLM lane's 0.5000 —
the suite flips. Lever = the noul count-table polarity (issue 038),
cal-selected on the authored cal front (yes→domain 1 at +25 pt over off);
noul never takes route terms (issue-030 law), so the corpus reaches the
decision through the NB tables. Enabling changes: the family now ships
corpus + cal (pairwise-disjoint, gate-asserted); `modelless_lane: true`; the
`selection_slice` synthetic fallback (cal front = the labelled selection
slice — before this, ANY select knob on ANY synthetic suite errored); NB
selection extended to synthetic suites (heads stay dataset-only). Gates:
families 7/7 (the LLM-only gate replaced by the production-seat grounding
gate `cache_reuse_grounded_posture_discriminates`), seat 5/5, clippy clean
at default + selective + the new posture. G1 power statement: n=12 → Wilson
CI [0.646, 0.985] — published as a flipped suite with wide bars, no accuracy
claim. **OPEN: T3's reflex-site publish half** (bench.json row + the arena
TL;DR's 14th row) — belongs to the session holding the reflex-site checkout,
same standing as 044 T5's pending py publish. T4: the HISTORY row + this
record carry the T3-law reversal.

Filed 2026-09-27, out of the 044 close-out conversation ("can't we at least
use an in-mem LRU cache on modelless?").

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

- [x] T1 — lift the carve-out in code: give `harness_cache_reuse`
  `modelless_lane: true`, author the per-class corpus docs (12, 6/class —
  the coverage-vs-divergence vocabulary) + the cal front (20, 10/class —
  the other half of the T3 exclusion), and let the standard noul
  machinery answer it — concretely, the issue-038 count-table polarity
  (`nb_noul_domain`), because noul never takes route terms (the
  issue-030 measured law) and the drafter's yes/no candidates are
  question vocabulary. The polarity is SELECTED on the cal front, never
  fixed. The 12 eval fixtures + their gold are byte-unchanged (the
  non-goal law, gate-pinned). LANDED 2026-09-28.
- [x] T2 — gates: G1 power statement stated in the bench record (n=12,
  Wilson [0.646, 0.985] — no accuracy claim); G2 p50 0.009 ms; the
  bit-identity repeat is green (the envelope's wall clock differs — the
  volatile fields, not the lane). The runner's loud-skip path is now
  unreachable over the registry (kept as declared-inability machinery);
  the families gate gained the noul-aware smoke + the grounding gate
  through the PRODUCTION seat path. LANDED 2026-09-28 (Bench 072).
- [-] T3 — both lanes re-run (modelless done, Bench 072; the laya row
  stands at its published 0.5000 — re-running it changes nothing at this
  n), lane-scoped publish: the REFLEX half is `.benchmarks/072_*`; the
  reflex-site half (bench.json row + the arena TL;DR's 14th row via
  `pick()`) belongs to the session holding the reflex-site checkout.
- [ ] T4 — HISTORY: the T3 law's reversal recorded with the measured
  verdict POSITIVE — landed with the reflex commit below; the issue file
  closes when T3's publish half lands.

## Non-goals

- No serve-path decision cache here (separate concern, separate gates).
- No change to `harness_cache_reuse`'s gold or fixture text (the 12 eval
  fixtures + their programmatic gold stay exactly as authored).
