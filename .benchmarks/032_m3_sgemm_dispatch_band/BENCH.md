# Bench 032 — the M3 sgemm dispatch BAND: the single-wave predicate (Issue 020 T7)

**Status:** LANDED (substrate predicate) — harness A/B row PENDING a quiet-box window
**Date:** 2026-09-25
**Substrate:** riir-infer (this session's `metal.rs` dispatch change — sha filled at commit)
**Issue:** riir-reflex `.issues/020_riir_metal_latency_parity.md` §T7

## What

The encoder/head sgemm dispatch previously picked the instance by two m/n
thresholds (`WIDE_M_MIN = 256`, `XWIDE_N_MIN = 2048`): narrow below m 256,
wide for m ≥ 256 with n < 2048, xwide for m ≥ 256 with n ≥ 2048. The T9
shape analysis had located the loss ("the encoder GEMM beats torch below
m = 256 and loses above it") but the pick itself had never been swept per
instance. This bench swept it: **every hot-path shape family × all three
instances × position-balanced rotated postures**.

## The landed predicate

```
xwide  iff  24 < ⌈m/64⌉·⌈n/128⌉·batch ≤ 40   AND   k ≥ 128
narrow otherwise
wide: never picked (dominated in all 60 measured cells)
```

One mechanism, three constants: xwide's fat single wave (staging intensity
42.7 MAC/staged element vs narrow's 21) wins only when its grid **mostly
fills but does not spill one wave** on the 40-core M3 Max, over **enough
k-iterations to amortize the staging**.

## Measurement

Instrument: `sgemm_pick_sweep` / `sgemm_pick_sweep2` (probe-only examples in
a detached worktree, on top of the archived `006_probes/riir-infer-gemm-pick-probe.diff`
env switches `LAYA_GEMM_FORCE=narrow|wide|xwide` — the probe half is never
committed). Pipelined 24-rep blocks, 5 samples, median; divergence check vs
the CPU triple loop ≤ 1e-3 (measured bit-identical on every instance — the
k-ascending chain is instance-independent, so the predicate is a pure
dispatch change and G5 drift is untouched).

Three shape populations, each measured with the three forced postures in
rotated order (narrow→wide→xwide / wide→xwide→narrow / xwide→narrow→wide),
pooled per cell:

1. **Encoder projections** (batch = 1; k ∈ {1024, 2624}; n ∈ {1024, 3072, 5248})
   at m ∈ {231, 257, 283, 317, 370, 400, 512} and packed scale
   m ∈ {1024, 1268, 1536, 2048}. Load 2.9–5.6.
2. **Head MHA** (`matmul_kt_heads` / `matmul_heads`, batch = 16, k = hd = 64,
   n = m / 64) at m ∈ {45..512}. Load 3.9–4.7.
3. **Small-m projections** (the head a0 tail 1×1028×256; the short suites)
   at m ∈ {1, 4, 45, 92, 106, 188}. Load 2.9–4.7.

Win consistency: 3-for-3 rounds in 24/28 band-1 cells; the four splits are
near-ties. Headline cells (probe medians, µs):

| cell | narrow | wide | xwide | old pick | new pick |
|---|---|---|---|---|---|
| o @ m=317 (n=1024) | 161.6 | 146.1 | **120.4** | wide 146.1 | xwide −17.6% |
| wo @ m=317 | 390.3 | 383.7 | **322.5** | wide 383.7 | xwide −15.9% |
| o @ m=512 | **220.2** | 270.8 | 222.4 | wide 270.8 | narrow −18.7% |
| wo @ m=512 | **559.5** | 770.2 | 627.1 | wide 770.2 | narrow −27.4% |
| wi @ m=317 (n=5248) | **621.5** | 810.8 | 737.1 | xwide 737.1 | narrow −15.7% |
| wi @ m=257 | **555.6** | 763.5 | 704.3 | xwide 704.3 | narrow −21.1% |
| qkv @ m=257 | **341.3** | 404.8 | 348.3 | xwide 348.3 | narrow −2.0% |
| packed wi @ m=2048 | **3846.0** | 5213.1 | 4312.1 | xwide 4312.1 | narrow −10.8% |
| kt_heads @ m=317 | **63.0** | 74.1 | 74.3 | wide 74.1 | narrow −15.0% |
| heads @ m=317 | **59.0** | 82.5 | 123.7 | wide 82.5 | narrow −28.5% |
| qkv @ m=317 | 387.0 | 441.4 | **361.9** | xwide 361.9 | narrow **+6.9%** (the one regression) |
| o @ m=92 (small-m) | **78.0** | 78.8 | 117.8 | narrow 78.0 | narrow (kept) |
| o @ m=188 | **104.8** | 141.7 | 118.8 | narrow 104.8 | narrow (kept) |
| qkv @ m=45 | **108.0** | 156.5 | 137.4 | narrow 108.0 | narrow (kept) |

The band constants are the measured regime boundaries, not tunings: 24
threadgroups LOSE with xwide (o@188, qkv@45), 32 WIN (o@231); and at 32
threadgroups the k = 64 head-MHA shapes LOSE with xwide (heads@92 28.3 vs
20.5 µs) where the k ≥ 1024 projections WIN — hence the floor (exclusive)
at 24 and the k ≥ 128 guard.

Per-question net at banking77's m ≈ 317 (22 layers × [qkv + o + wi + wo]):
the three wins (−25.7 − 61.2 − 115.6 µs/layer) swamp the one qkv regression
(+25.1 µs/layer) ≈ **−3.9 ms/question** of encoder GEMM wall, plus the head
MHA pair at batch = 16 (−15.0% − 28.5% of ~123 µs ≈ −26 µs/question).

## Gates

All green at the landed tip:

- `riir-infer-laya` lib 41/41 · `metal_ops_smoke` 7/7 ·
  `packed_forward_equiv` 4/4 · `packed_same_shape_gate` 1/1 (raw-bit
  equal-shape gate — passing under the new pick confirms result-identity
  end to end)
- clippy `-D warnings`: `--features laya-riir-metal --all-targets`, default
  `--all-targets`, `--no-default-features --all-targets`
- reflex G5 parity BOTH postures (CPU 27.9 s / Metal 12.0 s) ·
  `laya_batch_parity` 1/1 · reflex lib 56/56
- Dispatch-reproduction check: the probe re-run against the LANDED
  predicate (no force env) reproduces the measured-best cell values
  (o@317 120.2, wo@317 321.7, o@512 219.7, wi@257 572.6, kt/heads@317
  narrow-side, small-m narrow-side).

## The harness A/B row — NOT CLAIMED YET

Three 6-round paired harness A/B attempts (OLD = substrate @ 1e8c034, NEW =
the landed predicate; reflex @ 3ac8a8a; suites massive_intent_en,banking77,
code_fixtures):

1. First attempt: binaries pointed at a non-existent path (script defect) —
   never ran.
2. Second: load swung 1.8–9.5 with the sibling's builds; per-round ratios
   0.76–1.42 in both directions; the pooled banking77 +16.1% contradicted
   the kernel table. **DISCARDED** — the T5 first-attempt confound lesson.
3. Third (after the small-m fix): direction consistent (all three suites
   negative pooled) but the load range 3.6–20.6 aliased with arm order in
   the late rounds.
4. Fourth: load climbed 9–25 mid-run (preflight canary +15%); absolute
   p50s inflated ~50% and signs flipped. **DISCARDED.**

`scripts/bench_preflight.sh` currently REFUSES (load 14.77, canary
181.9 µs vs 141 µs AC reference, swap 1.2 GB). Per the box-state rule the
suite-p50 claim is **deferred until a run where the preflight passes**; the
same `/tmp/t7ab/harness_ab_t7.sh` (OLD/NEW binaries, alternating order,
load recorded per round) is the re-run instrument. The kernel-level table
above carries its own load class (2.9–5.6, recorded per round) and is the
landing basis.

## Where the constants live

`riir-infer/crates/riir-infer-laya/src/laya/riir/metal.rs`:
`WAVE_TG_LIMIT = 40` (the M3 Max's GPU core count — one wave),
`WAVE_TG_FLOOR = 24` (exclusive; under-fill loses),
`XWAVE_K_MIN = 128` (staging amortization over four BK-32 iterations),
with the full measured basis in the `WAVE_TG_LIMIT` doc. `WIDE_M_MIN` /
`XWIDE_N_MIN` are gone; the wide kernel stays compiled but unpicked.
