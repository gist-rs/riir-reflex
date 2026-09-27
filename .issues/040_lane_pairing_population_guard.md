# Issue 040 — the lane-pairing population guard: rust-vs-py "identical on 11/14" is a sample mismatch, not a port regression

**Status:** IN PROGRESS — root cause measured 2026-09-27; guard (T1–T4) landing same day; T5 (py re-run) pending a quiet box.

## Symptom

The site TL;DR (`reflex-site/assets/arena_tldr.js`) has read
`Rust vs Python laya, accuracy: identical on 11/14` since the 09-27 republish
(was 15/15 through 09-26). The 3 moved suites: `massive_intent_en`
(rust 0.6933 vs py 0.7500), `banking77` (0.4220 vs 0.4980),
`code_fixtures` (0.6667 vs 0.5833 — rust HIGHER there).

## Root cause (measured, not reasoned)

- The 09-27 republish refreshed the rust laya lanes (and modelless) to the
  Bench-052 **stratified** protocol (`b1aee72` 2026-09-26T15:34Z) — test
  sample changed from first-N prefix to label-stratified round-robin
  (052 BENCH.md: banking77's first 500 test rows spanned **13 of 77**
  labels, massive's 300 spanned **30 of 60**).
- The **python comparison lane never re-ran at 052** — the 052 run's own
  meta says `python lane: off (pass --laya-python)` — so the py rows
  survived in place on the **first-N sample** from `9dbdca5`
  (2026-09-26T12:02Z).
- On 11 suites the stratified sample is byte-equal to the old first-N
  prefix (balanced label blocks ⇒ round-robin picks the same rows), so
  rust==py still reads identical there. On the 3 suites above the case
  sets genuinely differ — the card counted **different question sets** as
  a parity failure.
- Proof the rust lane is healthy: 052 laya accuracy identical
  metal↔cuda **8/8**, G5 parity green, and the py rows are byte-identical
  across the before/after publishes (same acc, n, p50 — they never moved).

## The pairing law (the guard)

*Lanes are comparable on a suite iff their case populations are identical.
Anything else is DISCLOSED, never silently counted as a pass or a failure.*

- [x] **T1 — per-suite population identity in the runner.**
      `SuiteResult.cases_digest`: stable FNV-1a64 hex over the canonical
      serde_json of the served `suite.cases` (+ suite name). Dependency-free
      on purpose — `pub mod harness` is ungated, so harness code must
      compile at `--no-default-features` where blake3 is not in the tree;
      this is an identity tag, not a security hash. Byte-identical served
      cases ⇒ byte-identical digest, cross-host and cross-time.
- [x] **T2 — publisher stamps every lane cell.** `publish_bench.py` carries
      the source run's `cases_digest` (when the run has one) and already
      carries per-lane `lane_sources` (git_sha + date) — the pair key.
- [x] **T3 — the TL;DR pairs on identity.** rust-vs-py pairs with matching
      source identity (digest when present, else lane_sources run id) count
      toward `identical on N/N`; pairs with MISMATCHED identity render as
      their own third state ("different sample — not comparable") and are
      excluded from the count, never pooled either way.
- [x] **T4 — fail loud at publish.** `scripts/check_lane_pairing.py` in
      reflex-site reads `data/bench.json`, lists every compared pair whose
      identity mismatches, exits 1 unless `PUBLISH_ALLOW_SAMPLE_MISMATCH`
      names the suites (opt-in ack, the DOCS_GATE_PARTIAL_CLONE idiom;
      stale acks red). Wired into `republish_bench.sh`.
- [ ] **T5 — the data repair: re-run the py reference at the 052 protocol**
      (`--laya-python`, `.benchmarks/061_*`, new highwater) and republish so
      every rust↔py pair shares a sample. Expected: rust≈py within G5-class
      argmax wobble (a few borderline cases on the many-class suites), NOT
      the 5–7 pt gaps — those were sample difficulty. Quiet box required
      (MPS/GPU exclusivity law).

## Why a guard and not a one-off re-run

The publisher's live-as-primary merge refreshes lanes ONE CLASS AT A TIME by
design (the 052 republish refreshed rust+modelless; py/clm/gliner/agentjev
survived). Nothing asserted that the lanes it PAIRS still answer the same
questions. T1–T4 make population identity a checked invariant at publish
time, so the next protocol change (there will be one) trips a loud gate
instead of silently rewriting the site's parity claim.
