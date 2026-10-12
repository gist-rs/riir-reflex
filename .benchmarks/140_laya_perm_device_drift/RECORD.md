# Bench 140 — laya perm-pair device-drift readout: the three-posture zero (reflex Issue 086 T4+T5 close)

**Date:** 2026-10-12 · **Host:** shikuwa (4090 box, Windows) · **Reflex sha:** `1c75a9b`
**Posture:** laya CUDA (`laya-riir-cuda`, `LAYA_DEVICE=cuda`) — **G5 parity GREEN pre-run** (2/2
checkpoints, 6.96 s; the publish law held). Box state at run start: AC, high-performance
mode, load 1.2, gpu_util 9%, no preflight refusals — `latency_quotable: true` (though this
probe makes no latency claim; columns deliberately absent per the 128 discipline).

```sh
LAYA_DEVICE=cuda REFLEX_BENCH_HOST=4090-windows \
  cargo run --release --features laya-riir-cuda --bin harness -- --perm-probe \
  --suites typed_decisions --perm-max-cases 20 \
  --out .benchmarks/140_laya_perm_device_drift/laya_typed_cuda
```

## T5 — the cross-device/cross-posture readout (the G5-continuous rider)

Three perm-probe records over **identical slots** (typed_decisions; seed 125239297, K=5,
max-cases 20 → 30 choice-question slots, 150 orderings; slot ids verified equal across all
three records; the stride sampling is commit-stable across the three shas):

| leg | host | posture | record | reflex sha |
|---|---|---|---|---|
| A | 4090-windows | CPU | `128_perm_spread/laya_typed_cpu` | `1b8873e` |
| B | m3 | Metal | `131_kn_route_ab/laya_typed` | `459d344` |
| C | 4090-windows | CUDA | `140_laya_perm_device_drift/laya_typed_cuda` | `1c75a9b` |

Pairwise drift over the laya/typed lane (`p_ref` per ordering; picks; swing):

| comparison | median \|Δp\| | max \|Δp\| | picks agree | shape-cos (med/min) | swing digit-equal |
|---|---|---|---|---|---|
| CPU vs Metal | 0.0000 | 0.0000 | 150/150 | +1.0000 / +1.0000 (n=30) | 30/30 |
| CPU vs CUDA | 0.0000 | 0.0000 | 150/150 | +1.0000 / +1.0000 (n=30) | 30/30 |
| Metal vs CUDA | 0.0000 | 0.0000 | 150/150 | +1.0000 / +1.0000 (n=30) | 30/30 |

- The lane's ordering response on typed reads identically on every leg: median swing
  4.12 pt · max 9.23 pt · 4 flips (each record); the canary's five `p_ref` values are
  digit-identical across all three.
- **Verdict: ZERO device/posture drift on the presentation-perturbed surface.** The
  ordering-sensitivity (median 4.12 pt — the lane's known learnable position-sensitivity,
  disclosed RED under the ≤2 pt comparison-lane gate) lives **in the weights, not the
  device**. This extends Bench 082's cross-host bit-identity (dataset questions) to
  presentation-perturbed questions — a strictly more sensitive surface.
- **Disclosures (honesty):**
  - `p_ref` is quantized at 4 decimals in the record — the claim is "identical at record
    quantization", not bit-identity of the internal f32 path.
  - The **degenerate-pair purity floor is VACUOUS on typed**: no slot reads < 0.5 pt
    swing (the lane responds to ordering on every slot). The floor is trivially upheld by
    the identity; a suite with degenerate slots would be needed to exercise it. Vacuous,
    never folded into the verdict.
  - Three harness shas across the legs (slot ids verified identical — the sampling rule
    is commit-stable).
  - The laya/typed RED cell is the standing disclosed position-sensitivity (bench 128/131
    lineage), not a regression; the ≤2 pt gate is the comparison-lane bar.
- **Modelless control (this run, CUDA leg):** median swing 0.00 pt, PASS — 16 flips, every
  one an exact top-2 tie under the disclosed tie-only contract. Re-verifies the issue-079
  content-bound fix on this device at the current sha.

## T4 — the decision (recorded here on issue closure)

**DECLINE the opt-in projection at the embedding→routing seam. No suite shows material
ordering leakage into decisions** — the T4 conditional gate set never fires:

1. **Bench 128/131:** the deployed modelless control reads 0.00 pt median swing on every
   suite post-issue-079 (the one historical red — typed's legacy `k == N` binding — was
   fixed and acceptance-PASSed at content-bound, `4ad25df`).
2. **Bench 138:** the hashed embed carries 111–255-dim presentation subspaces (holdout
   leakage 0.005–0.58) — but the probe controls on the same slots read 0.00 pt / zero
   flips: the energy does NOT ride into the modelless scores.
3. **Bench 139:** the Thm-3 perturbation certificate PASSes at max margin on
   ag_news/emotion/banking77 (zero discrete flips at every rung through full ordering
   amplitude; Δp ≤ 6e-8); typed honestly reads `vacuous_no_response` (route-inactive
   0/31, full-amplitude Δp exactly 0.0) — the vacuity verdict replaces a fake pass.

**Re-open trigger:** any future lane whose perm-probe control reads > 0.01 pt median swing
or a non-tie flip on the deployed posture — at that point the T4 gate set (p99 ≤ 1 ms
unchanged, ECE + accuracy non-inferior paired LB95, perm-spread reduction) applies.
