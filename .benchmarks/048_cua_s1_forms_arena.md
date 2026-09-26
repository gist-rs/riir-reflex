# Bench 048 — cua-s1-forms arena arm: the CoreML/ANE posture on ITS OWN fixture (Issue 035 T4)

**Status:** RECORD — measured 2026-09-26 on the M3 Max; three runs, artifacts in
`048_cua_s1_forms_arena/`.

## Provenance

- **Box:** Apple M3 Max, macOS 26.6.2, AC, powermode 2 (High Power).
  Preflight before run 1: `PROVENANCE: power=AC Power load=4.38 swap=1443.62M canary=116.4us/best5 powermode=2(high)`.
  Before run 2: `PROVENANCE: power=AC Power load=5.21 swap=1427.62M canary=119.2us/best5 powermode=2(high)`.
  Harness box stamp run 1: start load 4.19 → end 5.06, **QUOTABLE**; run 2 CPU_AND_NE
  5.19 → 5.50 QUOTABLE; run 2 CPU_ONLY 5.50 → **6.02 NOT QUOTABLE** (sibling sessions on
  the box — that row is observation only).
- **Code:** reflex `a3c69f3` + the Issue 035 lane (`examples/cua_s1_forms_arena.rs`,
  `scripts/cua_s1_lane.py`), `--release`. Laya lane built against riir-infer `4e89963`
  (a clean detached worktree — the shared checkout carried a sibling's uncommitted
  `DeviceKind::Cubecl` WIP that does not compile against reflex); katgpt-rs `fdf8a6305`.
  **G5 re-run green at the metal posture on that build** (`laya_riir_parity`, english
  26/26 top-1, prob drift 3.5e-6; typed-decisions green) before any laya number here.
- **Fixture:** HF dataset `cua-ai/cua-s1-forms` @ `8273f34778b99ac2e12d9f6e7d57dad99ae20845`,
  `test.jsonl`, 24,370 rows, SHA-256 `d63a7e0d…4475e7c` (matches their card), BLAKE3
  `78223e7a…167ae14b` (pinned in the example; it refuses any other bytes). Train corpus
  source `train.jsonl` BLAKE3 `19206c9d…e0c5191f`.
- **Model:** HF `FluidInference/cua-s1-forms-coreml` @ `8e18ee41083f251b2fc3641ebf2671eab19a1650`
  (all 64 files verified against their `checksums.json`), `cua_s1_forms_fp16_options32.mlpackage`,
  coremltools 9.0, numpy 1.26.4, Python 3.12 (uv venv under gitignored `.raw/`), their
  `preprocessing.py` imported verbatim. Upstream model `cua-ai/cua-s1-forms` @ `f54adbf4`.

## Mapping (the comparability claim — G1)

One test row → one choice question: `context` = the wire state (string, verbatim);
`options` in order (7–32; `fill <entity>: <value>` … then `check`/`click`/`skip`);
`label` = gold index. No row dropped, reordered, or truncated (max context 217 B < 224,
max option 92 B < 96). **Lane knowledge is asymmetric and that is the cell:**

- `coreml` — Cua's 706K-param model TRAINED on this generator's train split
  (form-signature-disjoint test per their card). Their softmax taken verbatim;
  accuracy + latency only — never calibration (their strict-parity report fails).
- `modelless` — our engine, no gradients, house corpus protocol: 4 domains = gold
  action (`meta.action`), 64 FIRST-N train docs per domain (pre-declared, never
  selected on test), doc = `{context}\n{PROMPT}{gold option}`. Route terms are
  structurally inactive on this option space (k ≠ 4, options never name a domain).
- `laya-riir·typed` — ZERO-SHOT: criteria = the options (null descriptions),
  instructions = the example's fixed `PROMPT`. Even-stride subsample N = 2437
  (row `⌊i·24370/2437⌋` = every 10th row).

## Results (run 1 — all lanes, one invocation)

| lane | serving posture | N | top-1 | per gold action (fill · check · click · skip) | picked (fill · check · click · skip) | p50 / p95 / p99 |
|---|---|---:|---:|---|---|---|
| cua-s1-forms · coreml | **their stack serves** — coremltools subprocess, CPU_AND_NE; round-trip | 24,370 | **24,359 = 99.9549%** | 9802/9802 · 816/816 · 1040/1040 · 12701/12712 | 9813 · 816 · 1040 · 12701 | 2.199 / 2.596 / 2.718 ms rt (tail 1219/244) · model-only 2.115 / 2.490 / 2.596 ms |
| reflex · modelless | in-process Rust, CPU | 24,370 | 1,063 = 4.36% | 247/9802 · 816/816 · 0/1040 · 0/12712 | 1052 · 23318 · 0 · 0 | 0.129 / 0.207 / 0.231 ms (tail 1219/244) |
| reflex · laya-riir·typed | in-process Rust, metal | 2,437 | 730 = 29.95% | 563/1014 · 35/59 · 79/114 · 53/1250 | 1584 · 348 · 449 · 56 | 50.97 / 60.00 / 64.06 ms (tail 122/25) |

Floors — full split: uniform chance 5.57%, constant-`skip` **52.16%**. On the paired
2,437-row subsample (chance 5.56%, constant-skip 51.29%): coreml 2435/2437 = 99.92%,
modelless 90/2437 = 3.69%, laya-typed 730/2437 = 29.95%.

**Sanity gate PASSED:** the CoreML lane reproduces FluidInference's published full-split
result exactly — 24,359 / 24,370, the same 11 errors, all `fill`-for-`skip` (rows 137,
3216, 6631, 6748, 7280, 10703, 12255, 13480, 15214, 21022, 23671). The encoding path is
correct. Accuracy is deterministic across all three runs (identical counts).

## Posture observation (run 2 — coreml alone, sequential, NOT an A/B)

| compute units | top-1 | round-trip p50 / p95 / p99 | model-only p50 / p95 / p99 | box |
|---|---:|---|---|---|
| CPU_AND_NE | 24,359 | 2.621 / 2.831 / 3.837 ms | 2.507 / 2.677 / 3.689 ms | quotable (5.19→5.50) |
| CPU_ONLY | 24,359 | 2.230 / 2.458 / 2.589 ms | 2.117 / 2.331 / 2.452 ms | NOT quotable (end 6.02) |

On this M3 Max the ANE posture shows **no** latency advantage over CoreML-on-CPU for
this 706K model: the two sequential CPU_AND_NE readings (2.115 and 2.507 ms model-only
p50) bracket the CPU_ONLY reading (2.117). Sequential runs on a shared box — no speedup
or slowdown claim either way; an interleaved A/B would be needed. Their card's
0.90–1.00 ms (M5 Pro, CPU_AND_NE) is a different chip generation and is not reproduced
here (~2.1–2.5 ms).

## Verdicts

1. **The CoreML arm is measured and reproducible**: 99.95% on its home task, ~2.2 ms
   p50 round-trip on the M3 through a Python subprocess. It is a trained
   in-distribution specialist; that is what 99.95% means.
2. **Neither of our lanes clears the constant-`skip` floor** on this task. Zero-shot
   laya-typed reaches 29.95% (≫ chance, ≪ floor — it almost never picks `skip`,
   56/2437, where skip is gold 51%). The honest landscape reading: a 1.5 MB
   task-trained specialist beats our zero-shot laya-typed checkpoint (807 MB on disk) by ~70 pt on its own
   task, at ~23× lower latency (2.2 vs 51 ms p50).
3. **The modelless lane is DEGENERATE here (4.36% < 5.57% chance)** — its picks
   collapse onto `check` (23,318 / 24,370). Mechanism: with route terms structurally
   inactive, the option score is the LZ4 compressed-length delta alone, which carries a
   literal-cost prior a row-unique `fill` value rarely overcomes (247 of 9,802 fill rows). Filed as
   **Issue 036** (a real engine finding for any request-time option space); no fix is
   tuned on this test split.
4. This is a landscape footnote, not an arena suite: the fixture is theirs, the
   out-of-distribution direction (their model on our 15 suites) was ruled out in T1.

## Reproduce

See the `examples/cua_s1_forms_arena.rs` module doc (downloads, venv, commands).
Run 1: `LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --example
cua_s1_forms_arena -- --lanes coreml,modelless,laya --n-laya 2437 --out run1.json`.
Run 2: `CUA_S1_COMPUTE_UNITS=<units> … -- --lanes coreml`.
