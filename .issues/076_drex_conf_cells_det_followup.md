# Issue 076 — Drex lane follow-ups: the det timing-tail fix + the per-kind confidence cells, both landed; the BF16/CUDA det re-read is the open half

**Status:** LANDED (T3 the det strip @ `cd0248b`; T2 the cells + floor companion, this
commit) + OPEN (T1, the BF16/CUDA det re-read on the 4090 rig). Filed 2026-10-08 from
Bench 125 (`.benchmarks/125_drex_systemone_lane/`), the second-session pass over issue 073.

## Found (measured, not guessed)

1. **The lane's det column read ✗ by construction** (their T5 record: 10/10 rerun pairs
   byte-differ, attributed to bf16-CUDA nondeterminism): the response body carries
   `latency_ms`, which differs on every request BY DEFINITION — the raw byte-compare measured
   their clock, never the decision (the artifact behind AgentJev's recorded det ✗ too).
   **Bench 125 is the control**: with ONLY that field stripped from the compare input, two
   FULL Q8_0/Metal runs are byte-identical on the entire answers surface and det reads ✓.
   LANDED this commit: `decide_raw` strips `latency_ms` (everything else byte-exact) + the
   `decide_raw_strips_only_the_latency_tail` test pinning it. Follow-up owed by the first
   posture: re-read the BF16/CUDA det column under the normalization — if it STILL
   byte-differs after the strip, that is the real reduction nondeterminism and is currently
   unmeasured.
2. **The per-kind confidence cells + floor companion are not on the upstream lane.** Their
   T4 landed `readout_brier` through the shared tail (uniform, pooled). Bench 125's
   instrumentation goes one axis further: per-kind ECE/Brier of THEIR confidence (choice /
   score; noul carries no wire confidence → no cell, never fabricated) + the split-half
   conformal-naive floor companion (disclosed posture), carried on a
   `LaneResult.drex_conf_readout` field (serde skip-when-none; computed through
   `harness::metrics` only). Measured value: choice 0.1460 / score 0.2495 vs pooled floor
   0.1158 — the per-kind cells localize WHERE the miscalibration lives (score, the
   distance-from-mode statistic, is the worst). The reference implementation lives in this
   box's dropped commit `c9a9250` (local reflog; ~230 lines: `DrexConfCell`/`DrexConfReadout`
   + `assemble_drex_conf_readout` + the render/detail-line/JDI/dump surfaces) — re-derive
   from this description, don't resurrect blindly.

## How

- [ ] **T1** Re-measure the BF16/CUDA posture's det column under the normalization (needs
      the 4090 rig from `.benchmarks/073_drex_lane_t5_suite_pass.md` §serving; one typed_decisions
      run, quote the rerun-pair verdict).
- [x] **T2** Land the per-kind cells + floor companion (the design above; the floor stays
      the split-half posture, disclosed in the struct; per-kind floors stay pooled — the
      windows halve below the occupancy floor at 600/800 pairs). *(LANDED — this commit:
      `map_confidences` + `DrexConfCell`/`DrexConfReadout` + `assemble_drex_conf_readout`
      (`DREX_MIN_FLOOR_PAIRS` 40, first-half-cal/second-half-test split) + the
      `LaneResult.drex_conf_readout` field + the four surfaces the upstream lane was
      missing entirely (TABLES.md row, the their-confidence detail line, JDI crosswalk
      inclusion, the meta line) + 4 assembly tests + 2 wire-mapping tests; clippy -D at
      default/`--no-default-features`/`--all-features` all-targets green, `cargo test
      --lib` 303/0, wasm32 delta zero vs the pre-existing paw/openthai seams.)*
- [x] **T3** The det timing-tail strip + test. *(LANDED — the same commit as this file.)*

## Out of scope

- Any distill/product use (CC BY-NC — the license law stands).
- The agentjev lane's det column (same artifact, same fix shape — a one-line normalization
  in its `decide_raw`; noted here so nobody re-diagnoses it, not scheduled).
