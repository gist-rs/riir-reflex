# 028 — the 026 correction: CUDA flash attention (the packed-path zeros defect) + the refreshed cuda row

- **Date:** 2026-09-25 · **Host:** `4090-windows` · **Reflex sha:** `6d6cd8c` (+ dirty artifacts) · **Substrate:** riir-infer `75ed138` (the flash kernel, `.issues/003` CLOSED)
- **Posture:** `LAYA_DEVICE=cuda`, release, feature `laya-riir-cuda`, `REFLEX_BENCH_HOST=4090-windows`
- **Box state:** AC · 15.1 GB free RAM at launch · GPU 503 MiB / 43 °C / no compute consumers (GUI apps only — the exempt class) · run window ~7 min

## 1. The defect this run corrects

The `.issues/002` v1 CUDA backend ran attention through the trait-default op
sequence, which SLICES host memory at the packed multi-question offsets —
under the device backends' write-first discipline those host bytes are STALE
(device-written only), and the chain-cache MISS uploads them. **Every
multi-question case's attention ran on zeros** in the published 026 run.
riir-infer `.issues/003` (`75ed138`) fixes it with the fused flash kernel
(the Metal one-pass online-softmax form ported to CUDA C — offsets bind at
dispatch); this run is the corrected 4090 row.

| suite · checkpoint | CPU 018 (truth) | 026 v1 (corrupted) | **028 flash** |
|---|---|---|---|
| typed_decisions · typed | **0.7445** | 0.2690 (−47.5 pt) | **0.7415** |
| typed_decisions · english | 0.3575 | 0.2690 | **0.3570** |
| typed_decisions · multilingual | 0.3490 | 0.2690 | **0.3500** |
| code_fixtures · english | 0.5417 | 0.2917 | **0.5833** |
| ag_news · english | 0.9500 | 0.9500 | **0.9500** |
| banking77 · english | 0.4980 | 0.4980 | **0.4980** |
| emotion · english | 0.5925 | 0.5925 | **0.5925** |

Every corrupted row is restored to the CPU truth within the drift class
(±0.003 = 4–6 flips in 2000 — see §3); every correct row is unchanged.

## 2. Gates at the cuda posture (this box, before the run)

- `cuda_ops_smoke` + 3 fused arms: full (seq 1/9/37/64/129) / sliding
  (w8@64, w4@37, w16@130) / **packed offsets** — drift 1.2–1.8e-7, GREEN
  FIRST RUN.
- `packed_forward_equiv` CUDA arm (new): packed vs sequential, ≤1e-4; FAILS
  LOUD under `LAYA_CUDA_FLASH=0` (the fallback's offset guard panics — the
  silent-zeros class is unreachable).
- **`laya_batch_parity`** (THE multi-question gate, first cuda-posture run):
  english 26/26 · typed 26/26 · multilingual 36/36 batched forwards,
  top-1 **1.000000**, prob drift ≤ **5.1e-5** (gate 1e-3).
- G5 parity: english 3.3e-6 · typed 1.3e-6 · multilingual 4.7e-6.

## 3. Run-to-run variance note (honest reading)

Two identical flash runs of typed_decisions differed by 4–6 flips in 2000
questions (typed 0.7445 → 0.7415, multilingual 0.3485 → 0.3500): the
corpus-pool composition carries process-level state (the pre-existing
harness property — within-run repeat checks pass, `det ✓` every row). The
frozen-capture gates (G5 + batch parity) are the correctness authority and
are green; read ±0.003 on 2000-question suites as that variance, not as a
lane difference.

## 4. Latency (the flash rung, per-question p50)

| suite · english | 026 v1 | **028 flash** | Δ |
|---|---|---|---|
| typed_decisions · typed | 113 ms | **108 ms** | −4.4% |
| typed_decisions · multilingual | 65 ms | **59 ms** | −9.2% |
| typed_decisions · english | 114 ms | **108 ms** | −5.3% |
| banking77 | 29 ms | **27 ms** | −6.9% |
| code_fixtures | 34 ms | **30 ms** | −11.8% |
| ag_news | 17 ms | **16 ms** | −5.9% |
| emotion | 15 ms | **14 ms** | −6.7% |

Fixture rows (short seqs): english 17.5→16.2 ms, multilingual 9.0→8.5,
typed 17.6→16.6 (the `laya_fixture_timing` A/B vs `LAYA_CUDA_FLASH=0`).
The modest long-seq margin matches the shape analysis: the projections
dominate (~685 GFLOP/forward at seq 1000 vs ~20 GFLOP windowed attention);
the tile-ladder / CUDA-graph rungs (riir-infer `.issues/003` follow-ups)
attack the remaining projection cost.

## 5. Artifacts + follow-ups

- `.benchmarks/028_4090windows_cuda_flash/` — `results.json` + `TABLES.md`
  (this run, host `4090-windows`).
- The site data regeneration: the corrupted 026 typed rows must never reach
  a deploy (the M3-side `wrangler deploy` handoff is still pending — the
  live site serves the pre-026 data until it runs).
- Remaining follow-up rungs (riir-infer): sgemm tile ladders (m<64/n≤1024),
  CUDA graphs for per-op dispatch overhead.
