# Issue 018 — the reflex bench on the 4090 (windows lane): harness run + the bench-site refresh procedure

**Status:** OPEN — **T1–T7 LANDED 2026-09-24 (~14:2x, session `katgpt-rs-4090-b`)**; the run is published to the site repo (`7508b7a` — bench.json carries BOTH hosts); the ONE remaining step is `npx wrangler deploy` from a Node-≥22 box with CF creds (NOT this box — see the T6 handoff note). Filed 2026-09-24 (owner directive: "add issue to reflex bench on 4090 and update reflex.gist.rs/bench"). The 4090 tasking shape per the global rule (long tasks route to the 4090 via issue). Pairs with the ANE bench plan (`.plans/002_ane_lane_bench.md`, M3-owned) — that plan adds the DEVICE axis on Apple silicon; this issue adds the PLATFORM axis on Windows/CUDA.

## Why the 4090 lane matters for the bench page

The bench page's provenance row is single-host today (`host: m3` in
`data/bench.json` meta). The modelless engine is deterministic
cross-platform (the v0.2.3 4090 windows smoke proved byte-identical head
answers), so a windows run's value is NOT accuracy cross-checking — it is:

1. **Latency on different silicon**: the G2 latency column (p50/p99 per
   decision) measured on the 4090's 16-core-class CPU vs the M3's — the
   page's throughput claims gain a second box instead of one machine's
   word.
2. **The laya lane's CPU posture on windows**: `laya-riir` builds on
   windows (the metal lane is macOS-gated; the CPU backend is portable) —
   its numbers widen the same table the ANE lane will join from the other
   side.
3. **A standing second host for the page**: the fleet-join seam the
   healqual card already models (`machine` field) — the bench meta gains a
   second host row instead of overwriting.

## The run procedure (4090 box)

- [x] T1 Sync: `riir-reflex` on the 4090 to origin/develop (bundle path if
      GitHub auth hangs from that box — the carve-era lesson; ask the M3
      for a bundle rather than debugging auth mid-task). *(synced clean at
      `b60bba9`; the run completed against a worktree at `afacc3a` — see the
      pre-023 note below.)*
- [x] T2 Build: `cargo build --release --bin harness --features laya-riir`
      (CPU laya posture; NO metal — the feature is macOS-gated by
      construction). Record `--version` stamp + rustc + box state (RAM,
      load, power) beside the run. *(built 11:04, 55.7 s, laya-riir cpu;
      box state recorded in the launch block below.)*
- [x] T3 Datasets: `scripts/fetch_datasets.sh` (once; the blake3-digested
      manifest in `.docs/dataset_manifest.md` verifies the fetch). *(fetch
      completed via the 429 retry loop; 243 manifest-digested files
      blake3-verified clean; the windows python resolution rider landed as
      `f7c22c9` — the Store-stub `python3` trap.)*
- [x] T4 The run: full 15-suite set, uncapped. *(sibling session's live run
      11:16:35→14:11 wall ≈ 2h57m, ~590 CPU-min at ~3.8 cores; the python
      oracle lane honestly absent; artifacts committed at
      `.benchmarks/018_4090windows_run/`.)*
- [x] T5 Publish the SECOND-HOST rows. *(their publisher `3d86b84` used;
      results.json meta carries `host: 4090-windows` via `REFLEX_BENCH_HOST`;
      merged bench.json = m3@77c408e + 4090-windows@afacc3a; site commit
      `7508b7a`.)*
- [-] T6 Site update: commit the refreshed `data/bench.json` + `npx
      wrangler deploy` — **the commit+push half is DONE (`7508b7a`); the
      DEPLOY half is BLOCKED on this box** (wrangler wants Node ≥22, this
      box has v20.20.0; no CF token exists here). **Handoff: run `npx
      wrangler deploy` from the site repo (`E:\git\reflex-site` or any
      checkout at `main` ≥ `7508b7a`) on a Node-≥22 box with CF creds — the
      M3.** reflex.gist.rs/bench then renders both hosts. *(A population-
      guard disclosure patch also landed: `715e8a5` — excluded/absent
      suites render under the provenance row.)*
- [x] T7 Cross-host sanity: modelless ACCURACY bit-identical on **all 14
      comparable suites** (the drift gate passed; `code_fixtures`
      population-excluded 24-vs-28; `harness_cache_reuse` modelless-absent
      on both hosts by design). Secondary metrics (brier/nll/ece/mean_conf)
      show 1e-9..1e-11 RELATIVE drifts on 4 suites — cross-arch f64
      aggregation-order effects with identical answers, invisible at the
      page's render precision; recorded here so the next reader does not
      misread them as answer drift. Accuracy columns: bit-identical, as
      claimed.

## Discipline

- Serialized runs; no other GPU/CPU-heavy sibling job on the box during
  the latency rows (the box-state disclosure rule — record load beside the
  numbers or the number is not a measurement).
- No Python anywhere in the lane (the owner directive) — the oracle
  column's absence is the honest posture on windows.
- The site repo (`../reflex-site`) is the ONLY place bench.json is written;
  this repo carries only results.json + the sanitizer change.

## Session coordination (2026-09-24 ~11:30, two agents on this box — read before doing anything)

A full 4090 run is LIVE (PID-visible, started 11:16:35,
`--out .benchmarks/018_4090windows_run`, binary built at HEAD post-laya-move
WITH the host_label change). Status of the supporting work:

- **Datasets: FETCHED CLEAN** (rc=0, ~356 files; the 429-wall retry loop
  finished xnli ~11:20 — the runner loads suites lazily so the 11:16 launch
  is not compromised). Verify with `.raw/verify_datasets.py` (blake3 vs
  the manifest).
- **`REFLEX_BENCH_HOST=4090-windows` is REQUIRED on the run** (landed
  `ba13bcf`): the bench-site merge keys rows by host and the site's
  provenance row must read `4090-windows`, not this box's uname. ⚠ The live
  run was launched WITHOUT the env visible in its command line — if its
  results.json lands with `host: "shikuwa"`, that run's meta is
  mislabeled for the merge; re-run with the env rather than hand-editing
  results.json (a hand-edited provenance field is a defect by definition).
- **T5 assets are READY in `.raw/`** (this box, shared):
  `publish_bench_new.py` (multi-host merge + modelless-drift REFUSAL gate +
  population guard) and `bench_index_new.html` (renders `meta.hosts` +
  per-suite `extra_host_lanes` rows). Primary input = the site's current
  `data/bench.json` (the m3@77c408e record — the raw 77c408e results.json
  was never committed; the sanitizer's transforms are idempotent on it).
- **T7 assets**: `.raw/compare_hosts.py` (modelless bit-identity, skips
  population-mismatched suites) + the `77c408e` worktree at
  `../riir-reflex.w018` (datasets copied in; modelless-only build ready).
  MEASURED already: 77c408e vs HEAD are modelless-identical on the five
  comparable family suites on this box; **`code_fixtures` is
  repo-tree-relative at runtime** (harvests fn spans from this repo's own
  sources — 28 questions @77c408e vs 24 post-laya-move) and is EXCLUDED
  from cross-host merge by the population guard, never compared.
- **Do not run anything CPU-heavy while the live run's laya rows are in
  flight** (the serialization discipline cuts both ways). The worktree
  modelless run (~3 min) waits for the live run to exit.

### Collision disclosure (2026-09-24 ~12:1x, session `katgpt-rs-4090-b` — appended by the second agent, facts only)

A DUPLICATE full harness run existed on this box for a ~10–40 min window,
  overlapping the live run — disclosed here so the live run's owners can
  judge its latency columns:

- The live run (PID 105348, started 11:16:35 via `bench_018.bat`, which DOES
  `set REFLEX_BENCH_HOST=4090-windows` — the stale env warning above is
  resolved by that fact) was joined at ~11:27 by a second harness run this
  session launched before reading this coordination section (the collision
  the section was written to prevent — the second agent's fault, recorded
  here for the latency record). The duplicate was last confirmed alive at
  ~11:36 and was gone by ~12:05 with no panic output in its redirected
  stderr and no results (kill source unobserved; OOM under two concurrent
  laya worksets is equally consistent with the evidence). Its artifacts
  (`.benchmarks/018_4090_windows_run/`) are REMOVED — nothing of it survives.
- **Exposure window on the live run's numbers:** the duplicate's CPU overlap
  ran ~11:27 → no later than ~12:05 — inside the live run's FIRST suite
  (typed_decisions laya forwards; the live run's typed modelless rows ran
  11:16–11:2x, clean of the duplicate). Both processes held ~1.5–1.7 GB RSS
  with 8–16 worker threads each on 24 logical CPUs — moderate contention,
  not saturation; the duplicate's own contended typed modelless reading was
  +2% vs the M3 cell (0.481 vs 0.472 ms), which bounds the likely inflation
  class. **Accuracies are unaffected** (deterministic; the duplicate's typed
  modelless accuracy was bit-identical to the M3 cell, 0.319). If the live
  run's typed_decisions LAYA p50s look anomalous at completion against the
  other suites' CPU-posture scaling, the honest remedies are a disclosed
  publish or a full re-run — never a hand-trimmed number.
- This session's other 018 work, for the record: the windows python
  resolution rider on `scripts/fetch_datasets.sh` (committed `24acf56`,
  pushed — the Store-stub `python3` trap, `py -3` fallback, no-op when jq
  exists); duplicate T5/T7 assets (a competing publisher/page design and a
  dataset verifier) were built and then DISCARDED in favor of the landed
  `3d86b84` site state; the deploy wall was independently confirmed
  (wrangler wants Node ≥22, this box has v20.20.0, and no CF token exists
  here — `.raw/wrangler_probe.txt` agrees).
- **Handoff:** the completion lane (verify `meta.host`, publish via the
  site's `publish_bench.py` with the site's current bench.json as primary,
  commit reflex artifacts, record the deploy handoff) is taken by this
  session; the live run is NOT to be re-launched by anyone while it lives.

### Pre-023 binary (2026-09-24 ~13:15, session `katgpt-rs-59` — facts + one recommendation)

The live run's `harness.exe` was built **11:04:08**; the modelless ranking fix
(Issue 023, `d02f3a8`) landed **13:03**. So this run's modelless rows are the
**pre-023** engine: `massive_intent_en` will read **0.0767** (post-023:
**0.6900**) and banking77's calibrated gate will abstain on **100%** of
questions (post-023: 23%). The laya lane does not go through
`DecisionEngine::solve_into` and is unaffected. The 12 other suites are
byte-identical across the fix outside latency (Bench 007 §1).

- **T7 still works as written** — the site's `m3@77c408e` record is pre-023
  too, so the two hosts are compared at the same engine. Do not "correct" the
  massive row by hand; a spliced number is a defect by definition.
- ⚠ **The drift-refusal gate will fire the moment one host moves past 023
  and the other does not** — correctly. Both hosts' modelless rows have to
  move together.
- **Recommendation:** publish this run as-is with the provenance disclosing
  the pre-023 engine on both hosts, then take Issue 023 T5 as a
  **modelless-only** re-run at post-023 HEAD on BOTH hosts (~3 min CPU each,
  after this run exits and while nothing heavy runs), re-publish, and let T7
  confirm the two post-023 modelless records are bit-identical. The expected
  values are known in advance (Bench 007 table).

## Out of scope

- The ANE device rows (Plan 002, M3-only, sequenced first by the owner).
- Any serve/arena deployment change — this is bench data + site refresh
  only.
