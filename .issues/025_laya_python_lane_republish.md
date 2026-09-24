# Issue 025 — put the `laya (python)` lane back on reflex.gist.rs/bench

**Status:** OPEN — T1–T4 DONE (run 17:51–18:12, 15/15 suites, python = rust accuracy on every suite). T5 (publish + deploy + live check) is in progress.

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

- [x] T1: build the detached-worktree binary (`b2fd694`, `--features laya-riir-metal`, 31.7 s). A 5-case smoke run of the python lane passed first.
- [x] T2: quiet-gate autofire of the `--laya-python` run; record the
  PROVENANCE lines.
- [x] T3: copy the run into `.benchmarks/025_m3_laya_python/` and commit it.
- [x] T4 (reflex-site `20ba115`): publisher disclosure. On a lane update, the host row keeps its
  ORIGINAL `laya_python_lane: "off"` string, so after the merge the page would
  say "off" beside python numbers. Set the row's `laya_python_lane` from the
  update doc when that doc contributes `py/*` lanes, and add a self-test case.
- [ ] T5: publish, commit reflex-site, manual deploy, and live-verify that
  the page shows python bars on the m3 row.

## Run record (2026-09-24)

- Window 17:51:44 to 18:12:47, 21 min, 15 suites, `harness: PASSED — no absences`.
- Start PROVENANCE, the second of two passes 60 s apart: `power=AC Power
  load=3.63 swap=1207.50M canary=139.5us/best5 powermode=2(high)`.
- End preflight was **rc 1**: `load=8.28 canary=147.1us/best5`. Load rose
  during the run, so latency in the later suites may carry sibling
  contention. Accuracy is deterministic and unaffected; read the latency
  columns as an upper bound for this window.
- Python accuracy equals laya (rust) accuracy on **all 15 suites and all
  three checkpoints**, which is Issue 012's parity again, now on the post-023
  engine.
- Modelless massive_intent_en scored 0.6900. The publisher's pairwise drift
  gate passed across m3 / m3-ane / 4090-windows.
- Rust answers before python in every suite, so the latency pair is
  same-run but NOT order-balanced.
