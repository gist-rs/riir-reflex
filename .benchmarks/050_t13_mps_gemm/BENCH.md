# Bench 050 — Issue 020 T13/T13b: the MPS GEMM arm closes the every-cell bar (9/9 p50 AND p99 wins vs the torch MPS oracle)

**Status:** LANDED 2026-09-26 · riir-infer `b0de034` (T13, MPS GEMM arm,
default ON) + `5e18da4` (T13b, the MPS-posture split rule) · paired
per-suite instrument (`scripts/paired_suite_ab.sh`), preflight PASSED and
quoted · plus the full-lane M3 run the site republish is taken from
(`full_m3/`).

**PROVENANCE (paired):** `power=AC Power load=4.46 swap=2702.75M
canary=119.6us/best5 powermode=2(high)` at launch; summary-time
`load=4.28 canary=117.1us/best5` (`paired/preflight.txt`,
`paired/provenance.txt`).

## What changed in the engine

The Bench 046 residual was the typed_decisions trio (english +22.3% p50,
multilingual +6.6%, typed +5.7%) — the encoder GEMM at big m, where the
oracle's MPS kernel out-ran our narrow sgemm. T7 + the roofline probe had
closed that axis among OUR instances (five kernel-level refutations); the
issue named the reopen as a Metal-stack change. T13 takes it: unsplit
batch-1 dense GEMMs dispatch Apple's `MPSMatrixMultiplication`.

- **Priced before building** (riir-infer
  `examples/sgemm_mps_probe.rs`, 24 encoder projection cells, paired,
  CPU-anchored): MPS/narrow geo-mean **0.88 at m 106 → 0.59 at m 1700**,
  output **bit-identical** to the narrow instance on every cell.
- **T13 whole-forward paired A/B** (riir-infer `tests/metal_mps_gemm.rs`
  `t13`, 24 rounds/shape): loop 106 **0.741** · 140 0.682 · 188 0.657 ·
  317 0.639 · 512 **0.596**; packed [24,200] 0.717 · [200,220] 0.609 ·
  typed 5×179 **0.575** — 24/24 wins each; the no-MPS controls (54, 80,
  [30,61]) flat 0.999–1.001.
- **T13b** — the first suite A/B (T13 alone) left sst5 +7…+11% across three
  samples in both lane orders; a rust-only MPS on/off interleave cleared T13
  of it (on 27/23/24 ms vs off 27/28/27). The lever was the split rule:
  with MPS live, DEFAULT still sent every GEMM at m ≤ 96 to split-K, which
  MPS beats. `SplitRule::WITH_MPS` splits only m ≤ 32: shipped-rule A/B
  (`t13b`) seq 10/24/32 **0.997/1.010/1.004** (same plan — control),
  seq 33/40/46/54/64/80/96 **0.801/0.779/0.763/0.814/0.813/0.734/0.733**,
  24/24 each.
- **Numerics.** T13 is bit-identical (raw-bit gate, loop 10…512 + packed
  incl. the mixed plan; reach asserted both ways). T13b IS a reduction-order
  change at m 33–96 (split slices → one k chain) and was gated on the drift
  budget: reflex G5 metal top-1 **1.000000 ×3**, prob drift
  **1.744e-6 / 1.285e-6 / 3.729e-6** — DOWN from 3.473e-6 / 1.524e-6 /
  5.219e-6 against the CPU reference; batch parity, packed_forward_equiv,
  packed_same_shape_gate, metal_fold_bits, metal_ops_smoke green; clippy
  `-D` at three feature postures.
- Kill-switches: `LAYA_METAL_MPS=0` (the arm), `LAYA_METAL_MPS_SPLIT=0`
  (back to the DEFAULT split rule under MPS).

## The verdict (`paired/SUMMARY.txt`, 9 lane cells, 7 suites)

| suite | ckpt | rust p50 | py p50 | Δp50 | rust p99 | py p99 | Δp99 | sup | verdict |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | english | 34.0 | 52.0 | **−34.6%** | 48.0 | 122.0 | −60.7% | 5 | WIN / WIN |
| banking77 | english | 39.0 | 60.0 | **−35.0%** | 46.0 | 95.0 | −51.6% | 6 | WIN / WIN |
| emotion | english | 15.0 | 23.0 | **−34.8%** | 23.0 | 62.0 | −62.9% | 5 | WIN / WIN |
| massive_intent_en | english | 28.0 | 38.0 | **−26.3%** | 32.0 | 70.0 | −54.3% | 4 | WIN / WIN |
| sst5 | english | 17.0 | 26.0 | **−34.6%** | 21.0 | 56.0 | −62.5% | 7 | WIN / WIN |
| typed_decisions | english | 214.0 | 352.0 | **−39.2%** | 372.0 | 563.0 | −33.9% | 5 | WIN / WIN |
| typed_decisions | multilingual | 96.0 | 153.0 | **−37.3%** | 172.0 | 304.0 | −43.4% | 5 | WIN / WIN |
| typed_decisions | typed | 225.0 | 400.0 | **−43.8%** | 427.0 | 706.0 | −39.5% | 5 | WIN / WIN |
| xnli_en | english | 21.0 | 34.0 | **−38.2%** | 29.0 | 66.0 | −56.1% | 4 | WIN / WIN |

cells 9: **p50 wins 9 · p99 wins 9** · losses 0 · ties 0 —
**EVERY-CELL BAR MET.** Accuracy pairs identical on every cell (lane
integrity green).

## Honest caveats

- **One paired sample.** The smallest margin is −26.3% (massive p50) — far
  outside the ±2% tie band and the per-suite drift the pairing cancels, but
  it is still one sample.
- **The python lane read HIGH against Bench 046** (ag_news 52 vs 25, typed
  400 vs 261) — the Bench 036 publication-noise class, on a box carrying
  Zed + the fskitd/UVFS I/O load. Read the ABSOLUTE rust column as the
  corroboration that does not lean on the oracle: typed·english rust p50
  **313 → 214 ms**, banking77 **55 → 39**, massive **37 → 28** against
  Bench 046 — the rust lane got faster in absolute terms on a box that was
  no quieter.
- The two intermediate samples (T13 only; `/tmp`, not kept) are recorded in
  the issue close-out: 7/9 p50 with emotion (noise, flipped on rerun) and
  sst5 (the T13b lever) behind.

## Full-lane M3 run (the site republish input, `full_m3/`)

`REFLEX_BENCH_HOST=m3 cargo run --release --features
slice_leak,laya-riir,laya-riir-metal --bin harness -- --head-select
--laya-python`, gated on a DIRECT preflight (no pipe between —
`full_m3_preflight.txt`: `power=AC load=3.80 canary=129.4us/best5
powermode=2(high)`). 15 suites, harness PASSED, no absences. One
sequential run — the paired table above is the adjudicator; this is the
publication sample. Same-run Rust Metal vs torch MPS, with Bench 045's
rust p50 beside it:

| suite | ckpt | rust p50 | py p50 | Δp50 | rust p99 | py p99 | Δp99 | sup | 045 rust p50 |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | english | 20 | 33 | -39.4% | 34 | 80 | -57.5% | 5 | 28.0 |
| banking77 | english | 47 | 73 | -35.6% | 62 | 104 | -40.4% | 6 | 61.0 |
| code_fixtures | english | 41 | 79 | -48.1% | 135 | 230 | -41.3% | 1 | 62.0 |
| emotion | english | 15 | 23 | -34.8% | 23 | 61 | -62.3% | 5 | 16.0 |
| harness_cache_reuse | english | 16 | 48 | -66.7% | 19 | 55 | -65.5% | 1 | 19.0 |
| harness_permissions | english | 15 | 26 | -42.3% | 16 | 55 | -70.9% | 1 | 18.0 |
| harness_routing | english | 17 | 28 | -39.3% | 22 | 57 | -61.4% | 1 | 22.0 |
| harness_sensitivity | english | 17 | 29 | -41.4% | 21 | 61 | -65.6% | 1 | 19.0 |
| harness_tool_fit | english | 12 | 19 | -36.8% | 15 | 45 | -66.7% | 1 | 13.0 |
| harness_visibility | english | 15 | 25 | -40.0% | 17 | 50 | -66.0% | 1 | 18.0 |
| massive_intent_en | english | 29 | 48 | -39.6% | 34 | 81 | -58.0% | 4 | 40.0 |
| prompt_injections | english | 17 | 30 | -43.3% | 35 | 71 | -50.7% | 2 | 20.0 |
| sst5 | english | 18 | 27 | -33.3% | 24 | 65 | -63.1% | 7 | 21.0 |
| typed_decisions | english | 190 | 334 | -43.1% | 334 | 540 | -38.1% | 5 | 309.0 |
| typed_decisions | multilingual | 88 | 155 | -43.2% | 153 | 309 | -50.5% | 5 | 132.0 |
| typed_decisions | typed | 194 | 350 | -44.6% | 379 | 706 | -46.3% | 5 | 283.0 |
| xnli_en | english | 19 | 31 | -38.7% | 25 | 62 | -59.7% | 4 | 22.0 |

cells 17: p50 wins 17 · p99 wins 17

Rust is faster than the oracle on **every published cell, p50 AND p99**
(Bench 041's published run: 8/17 p50, 14/17 p99). Rust p50 fell against
Bench 045 on every cell. The python column read high again (typed·english
334 vs 045's 256) — the caveat above applies; the site shows this run's
same-run pair, as the protocol requires.

Published: reflex-site `6074b2b` (`republish_bench.sh` over the live
`data/bench.json` + this doc; self-test 21/21, chart smoke, mirror parity,
bench-page smoke PASS; hosts m3-max-metal / m3-max-ane / 4090-win
preserved, extra-host lanes byte-identical, modelless accuracy unchanged),
deployed by `wrangler deploy` (version `1fba86ca`) and verified live
(typed·english 190 vs 334 ms; plumbing paths 404).
