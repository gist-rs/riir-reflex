# Bench 137 — the modelless lane's quiet-box timing re-read (armed watcher)

**Status:** PENDING — watcher armed 2026-10-09, waiting for a preflight-clean box.

## The debt

The site's modelless lane carries **no quotable timing** since the 2026-09-24
primary (`8028a10`, load 6.31>6): the home chart renders the presence row
"timing failed the loaded-box check — re-run pending, values on /bench/".
Every later modelless refresh published **acc-only** under the `d3aeeae`
load-wall convention — the 10-suite posture refresh (`f068ae6`, 10-08) and
the s1mb trio (`0413dcd`, 10-08) — because the m3 stayed under sibling load
(the Issue-021 wall held each time, correctly).

Armed 2026-10-09 ~15:40 +0700: the m3 carries two long sibling jobs
(`svd_lbit_eval` 914% CPU 15h+, `hyperthink_t1_delta_census` 98% 20h+), so
the re-read is mechanized — `scripts/quiet_box_timing_rerun.sh` polls
`scripts/bench_preflight.sh` and fires the run the moment the box is clean.

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

Binary: rebuilt at HEAD `27ca2d9` (the `nb_pair_scale` commit `a53ce4b` is
opt-in, default 0.0 byte-identical — verified before arming). The watcher's
provenance guard stamps `src/`+`Cargo.toml` at arm time and refuses to fire
if either moves (committed or dirty) before the box quiets — docs/bench
commits do not trip it.

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
