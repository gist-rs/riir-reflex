# Plan 009 — the six harness families: wide eval feeding a quarantined our-lanes web section

Status: REVISED-2 2026-10-02 (owner call): UN-WITHDRAWN with a new product — the wide
eval feeds a NEW reflex-site section showing ONLY our lanes (Reflex modelless · Rethink
hybrid · Rethink encoder) behind a mandatory honesty caveat. Display-only; the
specialist-certification purpose stays dead.

## Arc (kept for the record)

As filed (wide-eval re-open anchor, `72c1238`) → withdrawn before execution by the owner
verdict that authored data cannot become a benchmark (`2367a38`, REVISED-1) →
UN-WITHDRAWN same day with the scope this file now carries. The REVISED-1 verdict's
reasoning is NOT reversed — it is ENCODED: the section exists BECAUSE these numbers must
never wear benchmark clothes. The caveat is the claim's boundary, rendered verbatim on
the page. Anchor issue: `.issues/059_harness_families_web_quarantine_bench.md` (revived,
redefined).

## Product (the web section)

- A NEW section on reflex-site — separate page/block from the main bench board.
- **Our lanes only** (owner call "bench only us"): Reflex (modelless) · Rethink (hybrid)
  · Rethink (encoder). No external lanes ever render here.
- **The caveat, rendered verbatim, mandatory** (copy lives in issue 059; the section
  must not ship without it):

> **Honest caveat — self-authored fixtures.** These six families are our own synthetic
> decision-point fixtures, not standardized external benchmarks. Gates prevent
> train/eval token leakage, but the question styles, distractors and class balance are
> ours. These numbers measure how our lanes behave on our own fixture distribution —
> engineering signal, not capability claims. Not comparable to the dataset-suite board
> or to the Jev Decision Index.

- Absent lanes render `not run` (pending-not-zero law) — never a zero-fill, never a
  fabricated cell.
- Data isolation: the section rides its OWN data file (e.g. `data/families.json`) —
  publish_bench's areas/index math must never ingest it, and the main board's
  cc/index/lanes machinery never reads it.

## Design (carried from the original, re-purposed)

- **Wide populations** (~96/family, class-balanced, 88–104 accepted) replace each
  family's 12–16-case eval — one eval per family; the runner, seat, and any lane read
  consume the SAME frozen population so cells stay coherent.
- **Template-disjointness gates double as permanent fixture hygiene** (unigram-overlap
  ceilings, shared-3-gram wall, label-token bans, BLAKE3 digest pin): they prevent
  future drift where a new case leaks corpus vocabulary and silently makes the fixture
  test the memorization path. Ceilings measured against the old eval's overlap stats.
- **corpus/cal UNCHANGED** — the trainable substrate and calibration front stay put;
  single-variable measurement.
- **One frozen read** per publish: every lane cell (reflex/hybrid/encoder) comes off the
  same frozen eval bytes; no per-lane re-baselines.
- **`cache_reuse_eval()` keeps the frozen T3 12-fixture record** (documented divergence,
  as originally planned — its gate pin stays a history pin).

## Lane cells

- **Reflex (modelless)** — the engine's run() posture + abstain rates on the wide eval;
  this repo lands the numbers and the bench record.
- **Rethink (hybrid)** — instinct's seat/arena seam over the frozen read (the Bench-011
  precedent). Display-only: instinct's specialist covered set stays law-excluded; no
  certification/T2-style gate runs on these cells — the caveat is the boundary.
- **Rethink (encoder)** — requires riir-train heads trained on the family corpora.
  OWNER CALL pending: spend the trainer lane for display cells, or leave `not run`.
  Absence is honest, not failure; the section ships complete with two lanes if the
  encoder stays out.

## Tasks

- [x] Restore + refine `.issues/059` (revived, redefined — this commit)
- [ ] Author wide eval populations: 6 families × ~96 cases, class-balanced,
      label-token-free, template-disjoint (per-family subagents; assembled + reviewed
      here)
- [ ] `src/harness/families_eval_wide.rs`; rewire all six `FamilyDef.eval`; gates
      (ceilings from measured old-vs-new overlap stats, 3-gram wall, label bans, BLAKE3
      pin); update CACHE_REUSE_NOTE + the T3 gate comment (documented divergence)
- [ ] One frozen read: reflex modelless rows (run() posture + abstain rates) →
      `.benchmarks/104_families_wide_eval.md` (box state + overlap stats + provenance)
- [ ] Hand the frozen eval to instinct: Rethink (hybrid) cells via the seat/arena seam;
      Rethink (encoder) = owner call on riir-train heads (absent → `not run`)
- [ ] reflex-site: build the NEW section (own page/block, own data file), our lanes
      only, caveat rendered verbatim, pending-not-zero, never touching bench.json's
      areas/index
- [ ] Full lane green: `cargo test` (families gates + lib) + `cargo clippy
      --all-targets -- -D warnings` at default and all-features postures
- [ ] Records: bench file, HISTORY.md entry, commit + push (develop); cross-ref issue
      059 + the reflex-site section landing hash

## Non-goals

- No external lanes on this section (gliner / agentjev / clm / paw / openthai) — our
  lanes only, per the owner call.
- No certification claims, no T2-style registration gates, no specialist-covered-set
  change — instinct's law-exclusion of the families STANDS; the section is display-only.
- No mixing with the main bench board, its areas/cc/index math, or the Jev-index
  standardization track (reflex-site Plan 001) — quarantined by data file and by page.
- No corpus/cal changes; no engine changes; no deletion of the families' existing
  fixture roles (G2/G4 workloads, regression pins, cache_reuse grounded gate).
- No bench claims anywhere without the caveat rendered beside them.
