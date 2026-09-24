# Issue 025 — put the `laya (python)` lane back on reflex.gist.rs/bench

**Status:** OPEN — filed 2026-09-24. The M3 run is IN FLIGHT as a detached quiet-gate autofire (session riir-reflex-025). Do NOT start a second M3 harness run; read `target/pyl025/autofire.log` first.

## Finding

The bench page shows `laya (python) — not run` on every suite. No open issue
owned it. The lane itself shipped and closed as Issue 012 (`67470be`,
`scripts/laya_python_lane.py`, harness `--laya-python`). It is opt-in, and
every run published since then left it off: `001_phase1_tables_metal`,
`018_4090windows_run`, and both `023_t5_*` modelless re-runs. All three host
rows in `data/bench.json` carry `laya_python_lane: "off"`. The last run with
it on is `001_phase1_tables` at `aa37823`, which is older than the Issue 023
engine fix and not what the site serves.

## Scope

- **M3 only.** The oracle is torch-MPS, and the 4090 row already discloses the
  lane as honestly absent (reflex-site `7508b7a`). m3-ane keeps its own row
  unchanged.
- **One run carries all three M3 lanes.** `--laya-python` also re-runs the
  laya (rust) Metal lane and the modelless lane. The publisher's ordered
  lane-updates (reflex-site `1619d32`) then replace m3's declared lanes and
  add the `py/*` lanes. The modelless drift gate checks the result against
  the 4090 and m3-ane rows.
- The rust and python lanes answer the same questions in one process, so
  their latency comparison is a same-run head-to-head. Rust always answers
  first, so the order is NOT balanced (the Bench 006 Addendum 3 disclosure).

## Recipe

- Build: a detached worktree at committed HEAD (`/tmp/pyl025/riir-reflex`,
  with `katgpt-rs` and `riir-infer` symlinked because both are clean at
  their origin tips, and `.raw` symlinked). Command:
  `cargo build --release --features laya-riir-metal --bin harness`. The
  shared worktree carries sibling WIP in `src/serve.rs`, and `code_fixtures`
  harvests the repo's own sources at runtime, so the binary is built from,
  and run inside, the clean tree.
- Run: `REFLEX_BENCH_HOST=m3 harness --laya-python --out
  .benchmarks/025_m3_laya_python`, from the worktree, only after
  `scripts/bench_preflight.sh` passes twice 60 s apart (the Issue 023 T5
  autofire pattern). Estimated ~20 min: ~9 rust, ~7–10 python, ~0.5
  modelless.
- Publish: `publish_bench.py <site>/data/bench.json <run>/results.json
  <site>`. The drift gate must pass, then commit the site and deploy it by
  hand (the free-tier CI limit).

## Tasks

- [ ] T1: build the detached-worktree binary.
- [ ] T2: quiet-gate autofire of the `--laya-python` run; record the
  PROVENANCE lines.
- [ ] T3: copy the run into `.benchmarks/025_m3_laya_python/` and commit it.
- [ ] T4: publisher disclosure. On a lane update, the host row keeps its
  ORIGINAL `laya_python_lane: "off"` string, so after the merge the page would
  say "off" beside python numbers. Set the row's `laya_python_lane` from the
  update doc when that doc contributes `py/*` lanes, and add a self-test case.
- [ ] T5: publish, commit reflex-site, manual deploy, and live-verify that
  the page shows python bars on the m3 row.
