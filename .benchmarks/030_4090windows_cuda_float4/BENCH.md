# 030 — the float4 sgemm rung (riir-infer `.issues/006`): every suite improves, the packed zone's first win

- **Date:** 2026-09-25 · **Host:** `4090-windows` · **Reflex sha:** `3957674` (+ working-tree float4 artifacts) · **Substrate:** riir-infer working tree (the float4 rung, `.issues/006`, commit `aadbc07`)
- **Posture:** `LAYA_DEVICE=cuda`, release, feature `laya-riir-cuda`, `REFLEX_BENCH_HOST=4090-windows`
- **Box state:** AC · 9.8 GB free RAM at launch (a sibling agent's release builds running — tighter than 029's 14.1 GB, ample for the ~3 GB process) · GPU 504 MiB / 43 °C / util 20 % (GUI apps only — the exempt class) · run window ~6 min

## 1. What landed

The `.issues/004` open question ("the packed multi-wave zone has no measured
win — split-K or an occupancy-tuned instance") was reframed and answered in
the substrate (riir-infer `aadbc07`): the zone's problem was never the
straggler tail — the wide instance ran at **15-20 % of the 4090's fp32 peak**
because the inner loop issues **6 smem loads per 8 FMAs** with the four
B-fragment loads CONTIGUOUS in the staging tile. Every instance's B staging
row now pads to a 16 B multiple (65→68, 129→132 — `col0` is a multiple of 4
at every call site, so the fragment address is float4-aligned by
construction) and the four loads collapse to ONE `float4` (6→3 loads per
8 FMAs; narrow's 1×4 fragment 5→2). Result-identical by construction —
bit-identical on every probe shape, both arms; every ragged edge is
staging-side-safe (out-of-range elements stage as 0.0f, stores stay guarded).

## 2. The instrument (kernel level)

The probe's CUDA population gained the packed-zone rows (m=424 ≈ 4×106,
m=1268 ≈ 4×317 — every projection multi-wave on every instance there; the
A/B columns read flat-by-construction and the ABSOLUTE µs is the datum).
Measured (in-process experiment A/B, then landed-absolutes vs the morning
baseline): gate/up m=1268 839→727 µs (−13 %), QKV 536→445 (−17 %), down
470→406 (−14 %), m=317 QKV 158→125 (−21 %); the single-question wide rows
−5..−14 %; narrow-served tails (m=4/m=1) −13..−17 %.

Forward level (`laya_fixture_timing`, cross-binary same-box same-session):
english 15.1→12.9 ms row p50 (−14.6 %), multilingual 7.2→6.3 (−12.5 %),
typed 15.1→12.8 (−15.2 %).

## 3. This run (the published row refresh)

Every suite improves p50 −6.8..−16.7 % — the PACKED suites included, the
multi-wave zone's first measured win:

| suite | 029 p50 | 030 p50 | delta |
|---|---|---|---|
| typed_decisions (en/mul/typed) | 109/59/109 | 100/55/99 | −8.3/−6.8/−9.2 % |
| ag_news | 15 | 13 | −13.3 % |
| emotion | 12 | 10 | −16.7 % |
| sst5 | 14 | 12 | −14.3 % |
| prompt_injections | 14 | 12 | −14.3 % |
| xnli_en | 15 | 13 | −13.3 % |
| massive_intent_en | 18 | 16 | −11.1 % |
| banking77 | 28 | 25 | −10.7 % |
| code_fixtures | 31 | 28 | −9.7 % |
| harness_visibility / permissions | 14 | 12 | −14.3 % |
| harness_tool_fit | 12 | 10 | −16.7 % |
| harness_routing / sensitivity / cache_reuse | 14 | 12 | −14.3 % |

**Result identity:** 13/16 laya rows BIT-IDENTICAL accuracy. The three
typed_decisions rows wobble 1-4 cases in 2000 — that lane's pre-existing
`determinism_ok: false` variance (false in 026, 028, 029 AND 030; on record
since v1, independent of every kernel rung — the kernel-level results are
bit-identical, the wobble is the lane's own repeat variance). The G5 gate
at the cuda posture (top-1 1.000000 ×3 checkpoints) + `laya_batch_parity` +
`packed_forward_equiv` + `cuda_ops_smoke` (tile-edge + ragged arms on the
new kernels) green at the landing.

## 4. Artifacts + follow-ups

- `.benchmarks/030_4090windows_cuda_float4/` — results.json + TABLES.md.
- Site: the 4090 lane row refresh is PREPARED here but the `wrangler deploy`
  remains the standing M3-side handoff (Node tooling broken on this box too
  — `npx wrangler` dies in the version-manager check).
- Remaining rungs (riir-infer HISTORY 006): the instances now sit at
  ~20-24 TFLOP/s ≈ 25-30 % of fp32 peak — register blocking /
  double-buffered staging is the next (bigger) rung, NOT another load-pattern
  tweak. CUDA graphs closed NEGATIVE 2026-09-25 (`.issues/005` — the lane is
  GPU-bound; `LAYA_CUDA_STATS` ships as the instrument).
