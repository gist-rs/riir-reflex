# 029 — the CUDA sgemm tile ladder (riir-infer `.issues/004`): the narrow instance's single-question win

- **Date:** 2026-09-25 · **Host:** `4090-windows` · **Reflex sha:** `2a61a6d` (+ working-tree ladder artifacts) · **Substrate:** riir-infer working tree (the ladder, `.issues/004`)
- **Posture:** `LAYA_DEVICE=cuda`, release, feature `laya-riir-cuda`, `REFLEX_BENCH_HOST=4090-windows`
- **Box state:** AC · 14.1 GB free RAM at launch · GPU 503 MiB / 44 °C / util 20 % (GUI apps only — the exempt class) · run window ~7 min

## 1. What landed

The `.issues/002` v1 backend ran ONE sgemm instance (64×64×32, 512 threads) for
every GEMM shape. This run carries the THREE-instance ladder (riir-infer
`.issues/004`): `sgemm_narrow` (32×64×64), `sgemm_wide` (64×64×32 — the v1
kernel), `sgemm_xwide` (64×128×32), picked per call by **BLOCK-FIT on the SM
count** (a grid over 128 blocks pays the straggler tail — measured as an exact
cliff: at m=106 narrow wins −15.7 % at n=2048 = 128 blocks and LOSES +46 % at
n=2560 = 160 blocks; static block scheduling strands 32 SMs at 2× while 96
idle). The M3 Metal lane's m<256 floor does NOT transfer — it is subsumed by
the block-fit arithmetic on this population. xwide keeps Metal's m≥256 ∧
n≥2048 floors PLUS the block-fit cap (the multi-wave zone reverts to the
proven wide instance — measured inside the instrument's noise band).

Kill-switch: `LAYA_CUDA_LADDER=0` (wide everywhere — the A/B posture).

A launch-defect fixed in passing: the `.issues/002` form passed the staging
footprint as DYNAMIC shared memory on top of the kernels' STATIC `__shared__`
arrays — harmless at wide's 2×16 768 B but narrow/xwide's 2×24 960 B crosses
the 48 KB default and the launch dies `CUDA_ERROR_INVALID_VALUE`. All
instances now launch with dynamic smem 0.

## 2. The instrument (reflex `sgemm_shape_timing`, new CUDA arm)

Same-process two-backend A/B (the kill-switch is read at `Cuda::new()`, so
one binary holds both postures), rounds alternating which backend goes
first. `--control` (both backends baseline) measured the two-context
artifact band: **±8-10 %** on the bigger shapes — per-shape deltas inside
that band are NOT resolvable by that instrument; every floor decision below
sits OUTSIDE it (or was confirmed by the single-backend-per-process forward
A/B).

Per-shape verdicts (position-balanced, outside the artifact band):
- narrow wins −13.7..−19.6 % across n=1024/1536/2048 at m≤106, the m=4/m=1
  head tail GEMMs −12..−17.5 %
- the cliff: n=2560 +46.4 % (160 blocks) → the block-fit cap
- xwide at QKV (m=317, n=3072, 120 blocks): −6.8/−8.5/−7.5 % across three
  runs (consistent)
- xwide at gate/up (205 blocks): +1.8/−6.5/−0.2 % (inside noise) → reverted
  to wide by the cap (conservative)

## 3. Forward-level A/B (`laya_fixture_timing`, single backend per process, ABAB)

| checkpoint | base p50 | ladder p50 | delta |
|---|---|---|---|
| english | 16.6 / 16.6 ms | 14.8 / 15.3 ms | **−10.4 %** |
| multilingual | 8.5 / 8.3 ms | 7.3 / 7.1 ms | **−14.1 %** |
| typed | 16.3 / 16.4 ms | 14.9 / 15.2 ms | **−8.4 %** |

(the fixture rows are single-question — the narrow zone; pairs reproduce
across both positions)

## 4. This run (the published row refresh)

Single-question suites (the narrow zone) improve p50 −6..−14 %: emotion
14→12, tool_fit 14→12, routing 16→14, sensitivity 16→14, cache_reuse 16→14,
prompt_injections 15→14, sst5 15→14, permissions 15→14, massive 20→18,
xnli 16→15, ag_news 16→15 ms. Packed suites flat by design (the block-fit
floor keeps wide on the multi-wave grids): typed_decisions 108→109,
multiling 59→59, banking77 27→28, code_fixtures 30→31 ms.

**Result identity:** 13/16 suite-lane rows BIT-IDENTICAL accuracy + macro-F1
vs 028 (every single-question suite). The three typed_decisions rows wobble
by exactly 1 case in 2000 each — that suite's laya lane carries
`determinism_ok: false` in 026, 028 AND this run (pre-existing, on record
since v1; its within-run repeat check fails independent of the ladder). The
G5 gate (top-1 1.000000 ×3 checkpoints, drift ≤5.1e-5) + `laya_batch_parity`
+ `packed_forward_equiv` + the op-level `cuda_ops_smoke` (boundary + ragged
arms at every tile edge) are green at the final floors.

## 5. Artifacts + follow-ups

- `.benchmarks/029_4090windows_cuda_ladder/` — results.json + TABLES.md.
- Site: reflex-site `1a59939` (the 4090 lane refresh, ordered merge — m3 +
  m3-ane lanes untouched). The M3-side `wrangler deploy` handoff stands.
- Remaining rungs (riir-infer): CUDA graphs for per-op dispatch overhead;
  the packed-suite zone (multi-wave grids) has no measured win yet — a
  split-K or occupancy-tuned instance is the open question, not another
  tile size.
