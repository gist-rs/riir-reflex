# 031 — the register-blocking sgemm rung (riir-infer `.issues/007`): every suite ≤ 030, the packed trio −5.5..−9.0 %

- **Date:** 2026-09-25 · **Host:** `4090-windows` · **Reflex sha:** `3668b6b` (+ working-tree reg4 artifacts) · **Substrate:** riir-infer working tree (the register-blocking rung, `.issues/007`, the commit that closes it)
- **Posture:** `LAYA_DEVICE=cuda`, release, feature `laya-riir-cuda`, `REFLEX_BENCH_HOST=4090-windows`
- **Box state:** AC · GPU 503 MiB / 45 °C / util ~18-20 % (GUI apps only — the exempt class; a sibling agent session's CPU builds running throughout) · run window ~5 min

## 1. What landed

The `.issues/006` follow-up rung in the substrate: `sgemm_wide_reg4` +
`sgemm_xwide_reg4` — the SAME tiles and grids as the 2-acc wide/xwide at
HALF the threads (256 / 512), warp tile 32×16, thread fragment 4 rows ×
4 cols = 16 accumulators: 4 LDS.32 + 1 LDS.128 per 16 FMAs = **2 B of smem
reads per FMA (down from 3)** — the shared-bandwidth roofline the lane was
measured against (25-30 % of fp32 peak at a 33 % ceiling) moves to 50 %.
Result-identical by construction (per-output accumulation k-ascending in
ONE thread on every instance); kill-switch `LAYA_CUDA_REG4=0` holds the
2-acc posture.

## 2. The instrument (kernel level)

The probe's A/B axis moved to `LAYA_CUDA_REG4` (base = the shipped 2-acc
posture, challenger = reg4; two full runs + `--control`). Every
wide/xwide-served row negative in run 2: banking77 zone −9.4..−16.1 %,
the packed multi-wave zone −9.4..−21.4 % (O m=1268 175→151 µs, down
436→345, QKV 477→387, gate/up 735→637), m=106/45 n≥2560 rows −10..−22 %.
The `--control` arm (SAME-kernel pairs) measured the instrument band at
−14.9..+10.1 % this hour (the sibling session's builds — wider than the
recorded ±8 %), which is why run-1's lone +3.5 % row (424 QKV) was read as
noise — it flipped to −9.4 % in run 2. Narrow-served rows flat on both
runs (both postures route narrow). Live-row median ≈ −12..−13 %.

Forward level (paired same-binary env-flip, 3 alternating pairs): english
13.1→12.4 ms row p50 (−5.3 %), typed 12.9→12.4 (−3.9 %), multilingual flat
— the dilution is structural: at fixture seqs only the n≥2560 projections
route wide-class (the rest narrow, unchanged).

## 3. This run (the published row refresh)

Every suite ≤ 030's p50 — median −6.2 %, no regressions:

| suite | 030 p50 | 031 p50 | delta |
|---|---|---|---|
| typed_decisions (en/mul/typed) | 100/55/99 | 91/52/92 | −9.0/−5.5/−7.1 % |
| ag_news | 13 | 12 | −7.7 % |
| emotion | 10 | 10 | 0.0 % |
| sst5 | 12 | 11 | −8.3 % |
| prompt_injections | 12 | 11 | −8.3 % |
| xnli_en | 13 | 12 | −7.7 % |
| massive_intent_en | 16 | 15 | −6.2 % |
| banking77 | 25 | 24 | −4.0 % |
| code_fixtures | 28 | 26 | −7.1 % |
| harness_visibility | 12 | 11 | −8.3 % |
| harness_permissions / tool_fit / routing / sensitivity / cache_reuse | 12/10/12/12/12 | 12/10/12/12/12 | flat (fixed-overhead + narrow-served) |

**Result identity:** 14/17 laya rows BIT-IDENTICAL accuracy. The three
typed_decisions rows wobble 3-7 cases in 2000 — that lane's pre-existing
`determinism_ok: false` variance (false in 026, 028, 029, 030 AND 031; on
record since v1, independent of every kernel rung). The G5 gate at the
cuda posture (top-1 1.000000 ×3, prob drift ≤ 3.3e-6) + `laya_batch_parity`
+ `packed_forward_equiv` + `cuda_ops_smoke` (boundary + ragged arms through
the reg4 kernels at the default posture) green at the landing.

## 4. Artifacts + follow-ups

- `.benchmarks/031_4090windows_cuda_reg4/` — results.json + TABLES.md.
- Site: the 4090 lane row refresh carries here too — `wrangler deploy`
  remains the standing M3-side handoff (Node tooling broken on this box).
- Next rung (riir-infer HISTORY 007): the NARROW instance (1×4 fragment,
  ~20 % ceiling, the m<256 single-wave zone) is the relative laggard — a
  narrow reg4 arm is the recorded follow-up, measured before built.
