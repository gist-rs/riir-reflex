# Bench 041 — the table republished after Issue 020 T11 (split-K + measured split rule + ln_rows_wide): Rust Metal faster than torch MPS on 14 of 17 p99 cells, 8 of 17 p50 cells

**Status:** COMPLETE — published to reflex.gist.rs/bench (reflex-site, see commit). Issue 020's every-cell bar is **not** met, so the issue stays OPEN.

## What ran

| host | binary | lanes | box state |
|---|---|---|---|
| m3 (`@m3-max-metal`) | riir-reflex `0b6a7c4` + riir-infer `9b0e55c`, `--features laya-riir laya-riir-metal`, `--laya-python` | modelless, laya rust ×3 checkpoints, laya python ×3 | start `load=2.92 canary=120.6us powermode=2 AC`; end `load=9.20 canary=132.8us` (`preflight_*.txt`) — the load rose during the run |
| 4090-windows (`@4090-win`) | riir-reflex `0b6a7c4`, default features (modelless only) | modelless | idle box (22% GPU before launch); synced by git bundle, since GitHub SSH hangs from that box |

Both hosts' modelless lanes moved TOGETHER, and the publisher's cross-host
drift gate passed (modelless accuracy identical on all 14 shared suites;
`harness_cache_reuse` is LLM-lane only and is absent from a modelless-only
build by design). The published modelless accuracy moved on two suites:
prompt_injections 0.440 → 0.483 (the `36e4e0a` noul route fix, now on the
page) and code_fixtures 0.250 → 0.333 (a commit-relative population).

## m3: Rust Metal vs torch MPS, same run

| suite | ckpt | rust p50 | py p50 | Δp50 | rust p99 | py p99 | Δp99 | tail | rust p50 before |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | english | 367 | 281 | +30.6% | 683 | 572 | +19.4% | 5 | 421.0 |
| typed_decisions | multilingual | 143 | 142 | +0.7% | 258 | 285 | -9.5% | 5 | 143.0 |
| typed_decisions | typed | 288 | 363 | -20.7% | 543 | 691 | -21.4% | 5 | 431.0 |
| ag_news | english | 31 | 32 | -3.1% | 55 | 75 | -26.7% | 5 | 31.0 |
| emotion | english | 18 | 21 | -14.3% | 28 | 49 | -42.9% | 5 | 22.0 |
| sst5 | english | 28 | 26 | +7.7% | 36 | 56 | -35.7% | 7 | 29.0 |
| prompt_injections | english | 27 | 30 | -10.0% | 71 | 69 | +2.9% | 2 | 26.0 |
| xnli_en | english | 31 | 29 | +6.9% | 39 | 57 | -31.6% | 4 | 33.0 |
| massive_intent_en | english | 46 | 49 | -6.1% | 59 | 104 | -43.3% | 4 | 47.0 |
| banking77 | english | 69 | 83 | -16.9% | 95 | 162 | -41.4% | 6 | 72.0 |
| code_fixtures | english | 88 | 63 | +39.7% | 272 | 208 | +30.8% | 1 | 65.0 |
| harness_visibility | english | 22 | 20 | +10.0% | 28 | 37 | -24.3% | 1 | 24.0 |
| harness_permissions | english | 21 | 20 | +5.0% | 27 | 37 | -27.0% | 1 | 24.0 |
| harness_tool_fit | english | 14 | 18 | -22.2% | 17 | 208 | -91.8% | 1 | 20.0 |
| harness_routing | english | 30 | 28 | +7.1% | 45 | 55 | -18.2% | 1 | 30.0 |
| harness_sensitivity | english | 31 | 31 | +0.0% | 39 | 57 | -31.6% | 1 | 29.0 |
| harness_cache_reuse | english | 29 | 46 | -37.0% | 36 | 49 | -26.5% | 1 | 27.0 |

p50: rust faster in 8 of 17 cells · p99: rust faster in 14 of 17 cells

("rust p50 before" = the previously published cell, riir-infer `d69f0c7`.)

## Reading

- ⚠ **One sequential run, not a paired measurement.** Bench 036 measured
  single-run cells moving ±20–40% on typed_decisions and the python lane
  across processes. The load also climbed 2.9 → 9.2 during this run. Read
  small deltas as noise.
- **p99 is where the T11 work shows**: Rust is faster on 14/17, by 25–45%
  on most short suites.
- **p50 losses fall into three groups.** (1) The short suites (+5–10%)
  are 1–2 ms on 20–30 ms cells, where the harness's whole-ms quantization
  is the resolution. (2) `code_fixtures` (+39.7%, tail support 1) is one
  512-token case, the Class-A item T9 attributed. (3) `typed_decisions ·
  english` (+30.6%) is the suite Bench 036 found least stable. The same
  suite's `typed` checkpoint reads −20.7%, and its rust p50 fell 421 → 367
  against the previous table.
- The split-K rule only engages at ≤ 3 row tiles (seq ≤ 96) or ≤ 64 narrow
  TGs, so the long typed_decisions sequences run the unchanged kernels.
  Their gain here is `ln_rows_wide` (−1.5…−3% at seq 188–317 in the paired
  A/B). The short-sentence gain is measured PAIRED on the arena (Rust/Python
  0.993–0.998, 372 sentences).
- Next per Issue 020: a paired per-suite A/B (the harness has no paired mode),
  and the remaining small-m lever (fusing the split reduce into the residual
  add).
