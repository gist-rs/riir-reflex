# Issue 018 — the reflex bench on the 4090 (windows lane): harness run + the bench-site refresh procedure

**Status:** OPEN — filed 2026-09-24 (owner directive: "add issue to reflex bench on 4090 and update reflex.gist.rs/bench"). The 4090 tasking shape per the global rule (long tasks route to the 4090 via issue). Pairs with the ANE bench plan (`.plans/002_ane_lane_bench.md`, M3-owned) — that plan adds the DEVICE axis on Apple silicon; this issue adds the PLATFORM axis on Windows/CUDA.

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

- [ ] T1 Sync: `riir-reflex` on the 4090 to origin/develop (bundle path if
      GitHub auth hangs from that box — the carve-era lesson; ask the M3
      for a bundle rather than debugging auth mid-task).
- [ ] T2 Build: `cargo build --release --bin harness --features laya-riir`
      (CPU laya posture; NO metal — the feature is macOS-gated by
      construction). Record `--version` stamp + rustc + box state (RAM,
      load, power) beside the run.
- [ ] T3 Datasets: `scripts/fetch_datasets.sh` (once; the blake3-digested
      manifest in `.docs/dataset_manifest.md` verifies the fetch).
- [ ] T4 The run: `cargo run --release --features laya-riir --bin harness`
      — full 15-suite set, uncapped (the M3 full-run precedent; ~the same
      wall budget). The laya-python oracle lane SKIPS loudly on windows (no
      python lane on that box per the owner directive) — its absence is
      recorded, never fabricated.
- [ ] T5 Publish the SECOND-HOST rows: the results.json meta gains
      `host: "4090-windows"`; `publish_bench.py` needs a small extension to
      MERGE per-host meta rows instead of overwriting (the healqual
      fleet-join precedent) — that sanitizer change lands in THIS repo,
      the merged bench.json lands in the site repo.
- [ ] T6 Site update: commit the refreshed `data/bench.json` in
      `../reflex-site` + `npx wrangler deploy` (manual deploy per the
      free-tier rule) — reflex.gist.rs/bench then renders both hosts with
      the provenance row naming each.
- [ ] T7 Cross-host sanity: the modelless lane's ACCURACY columns must be
      bit-identical to the M3 run per suite (the determinism claim); the
      LATENCY columns are expected to differ and are the point. Any
      accuracy divergence is a STOP-and-file, not a publish.

## Discipline

- Serialized runs; no other GPU/CPU-heavy sibling job on the box during
  the latency rows (the box-state disclosure rule — record load beside the
  numbers or the number is not a measurement).
- No Python anywhere in the lane (the owner directive) — the oracle
  column's absence is the honest posture on windows.
- The site repo (`../reflex-site`) is the ONLY place bench.json is written;
  this repo carries only results.json + the sanitizer change.

## Out of scope

- The ANE device rows (Plan 002, M3-only, sequenced first by the owner).
- Any serve/arena deployment change — this is bench data + site refresh
  only.
