# Bench 067 — typed_decisions laya re-run: the Bench 062 "Rust slower" cell was box state

**Status:** DONE 2026-09-27 · republished at reflex-site (see commit below) · no code change

## Why

After the Bench 062 lane-scoped republish, reflex.gist.rs read
*"Rust faster on 13/14, slower on 1 (typed_decisions 406 vs 362 ms)"*.
Issue 020 had closed at Bench 050 with typed english **190 vs 334 ms**.

## Diagnosis (before measuring)

- Only typed_decisions moved; the other 14 suites' Rust p50 held within
  ±5 ms of Bench 052 / the prior publish. Box-wide contention alone would
  have moved every suite.
- The same stratified sample at Bench 052 read Rust english **198 ms**.
  Between 052 and 062, riir-infer landed no Metal change (c64d0b1 is CUDA;
  20ea589 is temp-path sites).
- The Python lane on typed also slowed in 062 (py/typed 350→442,
  py/multilingual 155→185), and typed_decisions is the FIRST suite the
  harness runs.
- 062's own `box_state` read `latency_quotable: false` at BOTH ends
  (load 6.31 / 6.97, "a sibling job is on the box"). The latency cells
  were published regardless.

## What ran

Sibling-level worktree at `origin/develop` 5ada17a (this checkout carried
another session's WIP), `CARGO_TARGET_DIR=/tmp/reflex_w020_target`,
`--features laya-riir,laya-riir-metal`, `LAYA_DEVICE=metal LAYA_PYTHON=python3`,
`--suites typed_decisions`, datasets `.raw/datasets_t20k`. Lane order
balanced: r1 = one run, both lanes (rust → py); r2 = py-only then rust-only.
A sibling `harness` run was active at launch (preflight REFUSED at load 6.43
just before r1). The harness's own box gate read `latency_quotable: true`
at the start and end of all three runs (loads 5.99 → 5.24, `uptime.log`).

## Result — Rust wins 6/6 paired cells, both orders

| checkpoint | Rust r1 | Rust r2 | Python r1 | Python r2 |
|---|---|---|---|---|
| english | **195** | 256 | 328 | 513 |
| typed | **286** | 319 | 352 | 470 |
| multilingual | **92** | 97 | 173 | 295 |

(p50 ms per question.) Accuracy was byte-identical across every run and
matched the published cells (typed 0.7445 · english 0.3575 · multilingual 0.3490).
The ±50% run-to-run swing (Python english 328 vs 513) is the same
instability class Bench 036 measured. Only a paired comparison can speak
to the parity bar; one sequential cell cannot.

## Aftermath

- reflex-site republished typed_decisions' laya cells from **r1** (one
  run, both lanes, same `cases_digest`), lane-scoped
  (`PUBLISH_BENCH_LANES=laya`, current `data/bench.json` as primary). The
  pairing gate stayed green (29 same-sample pairs) and the TL;DR reads
  **Rust faster on 14/14**.
- ⚠ Premise worth carrying: a republish can print a regression that is
  the box. Before publishing, check `box_state.*.latency_quotable` on the
  run you are about to publish. 062's cells were non-quotable at both ends.
