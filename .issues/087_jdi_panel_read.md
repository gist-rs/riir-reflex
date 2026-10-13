# Issue 087 — the JDI panel read: reflex instrument for the SubQ Snap beat lane (rethink 032 T1)

**Status:** OPEN — filed 2026-10-13 (M3). Instrument issue: the harness gains the JDI panel's
choice-shaped public benchmarks as suites (Plan 012 Phase A/B), so riir-rethink Issue 032 T1 can
score the serving ladder on the board's own `public_skill` axis. Plan: [012](../.plans/012_jdi_panel_read.md).
Cross-repo: the record + board consumption live in rethink (`.benchmarks/067_jdi_public_skill_read/`);
reflex owns lanes/adapters/suites — never a fork of the harness for a rethink need.
**Progress 2026-10-13: Phase A + B1 + B3 + Phase C LANDED** — the floor read exists (rethink
Bench 067: **A0 4.40 / ENC 12.99 zeros-included** public_skill on the served 5/37 rows vs the
58.0 M1 bar; verdict round 1 applied: hash-ordered sampling on the two category-clustered
caps — the first-N sample was measurably easier, HellaSwag ENC 0.4195→0.3635 — and
zeros-included is the headline). **B2 (clef) UN-DEFERRED at verdict round 1: the detached
tranche pass IS IN FLIGHT** (Plan 012 B2 row for the runner/resume contract).

## Why now

riir-rethink Research 004 + Issue 032 (the SubQ Snap beat lane, verdict AGREE 2026-10-13) set the
bars on the JDI public-only aggregate. The board's `public_skill` formula is now verified exactly
from the shipped JSON (Plan 012 header — gold-weighted category skill, area-weighted; reproduced
to the board's printed figures on three rows). What's missing is OUR per-benchmark rows: the panel
is NOT our 9 suites — the only overlap is BANKING77. Tranche 1 adds the four public benchmarks
whose official splits ARE the board's row counts (1267/10042/3200/12032 all match the board's
`cases` exactly): winogrande_jdi, hellaswag_jdi, anli_jdi, mmlupro_jdi.

## Tasks

- [x] A1–A6 the four suites (fetch blocks + builders + spec rows + `load_rows` comma-list eval
      splits + manifest rows + count floors + hand-case tests) — Plan 012 Phase A for the
      per-suite contract. LANDED 2026-10-13: gates green (clippy default + all-features, cargo
      test 499/0/1), fetch complete (1267/8000/3200/12032 eval rows — the board's `cases`
      counts exactly).
- [x] B1–B2 the ladder run (A0 + laya Metal + clef trimmed) with preflight box state. B1+B3
      LANDED same day (rethink Bench 067: A0 4.40 / ENC 12.99 zeros-included). **B2 (clef)
      UN-DEFERRED at verdict round 1 and IN FLIGHT** — detached ~8 h tranche pass (Plan 012
      B2 row: runner, resume markers, env contract); fold into bench 067 when it lands.
- [x] B3–C3 hand the results to rethink for the aggregate + record (rethink
      `.benchmarks/067_jdi_public_skill_read/`). LANDED — the self-verifying aggregate script +
      raw harness artifacts live in the record; Research 004's position table + issue 032 T1
      carry the floor.

## Laws

- grammar-not-answers: these are FROZEN TEST READS of the ladder — never training rows, never
  corpus fill at any proportion; train splits fetched ONLY as the corpus/cal pools the harness
  already isolates.
- B5 (Research 005): every row ships with the crosswalk caveat — different population (caps,
  first-N), our protocol, our hardware. Never presented as a board row.
- Edition law: every number quotes edition 0.3.2 + `generated_utc` + snapshot sha256.
- The macro-F1 caveat: wino/hellaswag/mmlupro option keys are POSITIONS (texts ride the
  descriptions), so their `macro_f1` columns measure position-accuracy, not class F1 — the
  board's metric for them is accuracy anyway; ANLI/banking77 use real class labels (macro-F1
  clean, and macro-F1 IS the board's metric there).
