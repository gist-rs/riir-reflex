# Bench 137 — the modelless lane's quiet-box timing re-read

**Status:** COMPLETE — quotable doc published 2026-10-10 (reflex-site `data/bench.json`, source_run `197e855`); the site's loaded-box presence row retires.

## The run (the QUIET window, first since the 09-24 primary)

- fired 2026-10-10 13:52 +0700 after `bench_preflight.sh` PASSED
  (PROVENANCE: power=AC Power load=2.96 swap=2404.75M canary=140.0us/best5
  powermode=2(high)); ~11.5 min, 13/13 suites, start load 4.0 / end load 2.5 —
  **latency QUOTABLE both ends**, zero refusals.
- p50s: typed 0.762 · banking77 0.357 · code_fixtures 0.215 · s1mb_choice
  0.373 · s1mb_noul 0.291 · s1mb_score 0.245 · ag_news 0.143 ·
  massive 0.111 · emotion 0.110 · xnli 0.084 · sst5 0.090 · prompt_inj 0.076 ·
  semantic_defects 0.024 (ms).
- **Determinism witness: 13/13 suites digit-match** the published cells'
  accuracy AND corpus_digest — a pure timing attach, no posture drift
  (binary rebuilt at `197e855` after the sibling's `src/harness/metrics.rs`
  commit moved the armed stamp; sst5 re-smoked 0.39666… digit-exact first).
- The armed watcher's two earlier attempts (09-40Z, 12:00Z) ended unquotable
  when siblings returned mid-run (start 5.76 → end 11.48) — the Issue-021
  wall held both times; the watcher then GAVE UP on a sibling's transient
  `src/` dirt, all three of its behaviors proving themselves live.

## The publisher defect the re-read surfaced (fixed same day)

An **acc-only-stripped incumbent** (no timing fields, verdict ABSENT) could
VACUOUSLY carry: `apply_lane_carry` fired on the bench-137 update, donated
nothing (the incumbent owned no timing), popped the update's
`latency_quotable: true`, and stamped a false `latency_provenance` note.
The Issue-003 T2 suppression couldn't see it (it checks verdict `is False`,
and the stripped incumbent's verdict is absent). Fix: the vacuous-carry
guard (`not any(k in src_lane for k in LANE_LATENCY_FIELDS)` → skip) in
reflex-site `scripts/publish_bench.py`; self-test
`case_stripped_incumbent_cannot_vacuously_carry` (93/93 green; prove-fires
with the guard removed).

## Publish record

`republish_bench.sh` full pass (self-test 93/93 · chart smoke · publish ·
pairing 18 pairs · mirror parity — after syncing the sibling's dev_flow
mirror drift · bench-page chromium smoke PASS). All 13 modelless cells now
carry `latency_quotable: true`, clean provenance, `source_run 197e855`.
`data/changes.json` row added (the clef 2026-10-04 graduation precedent).

## The debt (historical context)

The site's modelless lane carried **no quotable timing** from the 2026-09-24
primary (`8028a10`, load 6.31>6) until this bench. Every later modelless
refresh published **acc-only** under the `d3aeeae` load-wall convention —
the 10-suite posture refresh (`f068ae6`, 10-08) and the s1mb trio
(`0413dcd`, 10-08) — because the m3 stayed under sibling load (the
Issue-021 wall held each time, correctly), leaving the home chart's
presence row standing.

## The posture (digit-verified on this box, correctness is load-insensitive)

Command (the site posture — matches the `f068ae6`/`70f2b2c` runs exactly):

```sh
REFLEX_BENCH_HOST=m3 ./target/release/harness \
  --skip-laya --nb-select --oc-select --ridge-select --drafter-fix per_byte \
  --suites typed_decisions,ag_news,emotion,sst5,prompt_injections,xnli_en,massive_intent_en,banking77,code_fixtures,semantic_defects,s1mb_choice,s1mb_noul,s1mb_score \
  --out .benchmarks/137_modelless_timing_quiet
```

Smoke evidence (under load, `/tmp/137_smoke_*` — accuracy only, latency ignored):

- `sst5` acc **0.3967** == published cell digit-for-digit
- `s1mb_score` acc **0.4982497082847141** == published cell; corpus_digest
  `fnv1a64-f5d9…` == published

Binary provenance: first rebuilt at `27ca2d9` (`a53ce4b`'s `nb_pair_scale`
is opt-in default-0 byte-identical), then REBUILT at `197e855` before the
final run — the sibling's `src/harness/metrics.rs` commit moved the armed
src stamp, and the watcher's provenance guard had (correctly) refused the
stale binary. The guard stamps `src/`+`Cargo.toml` at arm time and refuses
to fire if either moves (committed or dirty) — docs/bench commits do not
trip it.

## After the run

`READY_TO_PUBLISH.md` appears here when the doc's START+END box_state both
judge latency quotable. Then (review, then):

```sh
../reflex-site/scripts/republish_bench.sh ../reflex-site/data/bench.json \
  .benchmarks/137_modelless_timing_quiet/results.json
```

The doc carries the modelless lane only (a `--skip-laya` run), so the update
is lane-shaped by construction. Accuracies must digit-match the published
cells (determinism) — drift means the posture moved: STOP and re-verify (the
cross-host bit-identity gate refuses on drift anyway). Remaining steps after
publish: commit reflex-site `data/bench.json` + a `data/changes.json` row,
push, `npx wrangler deploy` (manual by design).

Companion issue: `.issues/083_site_modelless_timing_republish.md`.
