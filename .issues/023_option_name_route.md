# Issue 023 — the modelless centroid signal was OFF on sampled-distractor suites (massive 0.077 → 0.690)

**Status:** OPEN — T1–T4 DONE (engine fix + A/B + record, Bench 007); T5
(republish the arena `bench.json` / README three-way rows from a full
both-lane run) is the one open row. Filed 2026-09-24 from the reflex.gist.rs
arena TL;DR ("modelless ≥ laya on 3/14; worst massive 7.7% vs 75.0%").

## Finding

`DecisionEngine::solve_into` ranks options by two signals: the LZ4 drafter
delta and the state's cosine to each option's corpus centroid. Its own
comment says the drafter delta **cannot rank options** (a 4-byte option
never moves the compressed length of a 300-byte context — Issue 004 T7), so
the centroid term is the signal. It was armed only when `k == N`
(question arity == domain count) and paired option *i* with domain *i* **by
index**. Two suites break that premise:

1. **massive_intent_en** — 1 gold + 19 sampled distractors of 59 labels, per
   question, shuffled: `k = 20 ≠ N = 59`, so the centroid term was **never
   computed** and the lane ranked by the drafter delta alone — 0.0767 over
   20 options, ~1.5× chance.
2. **banking77 / code_fixtures CAL slices** — `k == N` holds, but the
   calibration cases are built from TRAIN rows whose first-appearance label
   order differs from the domain order, so the index pairing was
   **misaligned**: banking77's cal target accuracy read **0.007** and the
   calibrated gate abstained on **100%** of questions. The test-split raw eval
   happened to be aligned (the domain order IS the test union order), which
   is why no accuracy row showed it.

Issue 013 closed with *"the lane sits at its feature-class ceiling"*. That
verdict was measured with the ranking signal disabled on massive and with a
misaligned selection slice on banking77 (Bench 004's refusal) — both are
corrected in place (Bench 004/005 addenda).

## Fix

`EngineConfig::option_name_route` (default **true**): resolve every option
to the domain whose name equals the option string (exact bytes, stack
`[usize; N]`, zero-alloc), and use THAT domain's cosine as the option's
route term. Armed only on a FULL resolution (a partial map would give some
options a centroid term and others none — a bias, not a signal); otherwise
the legacy `k == N` index rule, unchanged. `false` is the pre-023 posture.

## Tasks

- [x] T1 — engine fix + three unit pins (`src/engine.rs`: shuffled k<N subset
  ranks by named centroid; flag-off is the legacy posture; k==N in-order is
  bit-identical to legacy).
- [x] T2 — full modelless A/B, all 14 suites (Bench 007): massive
  0.0767 → **0.6900** (laya best 0.7500), G1 FAIL → PASS; 12 other suites
  byte-identical outside latency; banking77 / code_fixtures test accuracy
  identical, calibration repaired.
- [x] T3 — guard `scripts/ci_feature_guard.sh` PASSED (full): clippy ×4
  postures, tests, G2 p99 52 µs ≤ 1 ms, G4 0 allocs, G5 parity.
- [x] T4 — cap re-selection under the fix (Bench 007 §3): banking77 and
  massive both select 256; **promotion declined** (banking77 +1.0 pt for 2×
  p50; massive +1.7 pt = 5 questions of 300, inside noise). Registry caps
  unchanged; the instrument stays.
- [ ] T5 — republish the site `bench.json` + README three-way rows from a
  full both-lane run. Box-gated, not deferred by choice: the M3 GPU is
  carrying a sibling session's interleaved Metal-vs-ANE latency A/B (Plan
  002 P1) and a laya run now would contaminate both measurements. Take it
  with the next Issue 018 refresh; the modelless rows are deterministic, so
  the published values are known in advance (Bench 007 table).
  ⚠ The live 4090 Issue 018 run (binary built 11:04, before this fix) and the
  site's m3@77c408e record are BOTH pre-023, so the next publish is
  consistent and still carries massive 0.0767. T5 is therefore a
  modelless-only post-023 re-run on BOTH hosts, published together (the 018
  drift-refusal gate refuses a one-host move) — plan recorded in Issue 018
  § "Pre-023 binary".
- [x] T6 — DONE (Bench 005 Addendum: refutation STANDS, banking77 net 0,
  massive arms nothing). Re-read Bench 005's pair-head A/B under the fix
  (`--skip-laya --pair-head-ab --suites banking77,massive_intent_en`). Its
  heads are armed from CAL-slice confusion, and banking77's CAL slice was
  the misaligned one, so Bench 005's correction (which voids massive
  only) may under-state the damage: banking77's pair-head rows were
  also fitted on broken confusion. CPU-only and ~minutes, but needs a build;
  run when no sibling latency A/B is in flight. Record as a Bench 005
  addendum and update the AGENTS.md lever-3 line either way.
- [x] T7 — stale-verdict sweep: Bench 003's correction (still citing
  Bench 004's refusal as a finding) gains Correction II; AGENTS.md's
  lever-1/lever-3 comment lines no longer state the pre-023 verdicts as
  standing. Bench 001's `route_scale` "FLAT" (lever 2) stands — it was
  read on the synthetic families, byte-identical under the fix.
