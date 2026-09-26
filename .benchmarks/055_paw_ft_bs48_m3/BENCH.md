# Bench 055 — Issue 033: the `paw-ft-bs48` finetune-compiler cells, at the Bench-052 stratified sample

**Status:** MEASURED 2026-09-26 (M3, hosted-anonymous posture, compiler `paw-ft-bs48-20260530`). The default-compiler rows were RE-MEASURED beside them (cached programs, zero recompiles) because Bench 052's stratified split changed every suite's cases — the Bench-049 cells (first-N sample) and these are not the same case sets, and both postures needed an apples-to-apples table. ⚠ **RENUMBERED 053→055 (2026-09-27):** 053 was dual-allocated the same day — this record's commit (`5cf57b1`) and [`053_e0_evidence_density`](../053_e0_evidence_density/BENCH.md) (commit `299a3af`, the EARLIER allocation — it keeps 053, and AGENTS.md pins its path). Content untouched under the new number; a renumber consumes 055 exactly like a fresh allocation. Cross-box twin: [Bench 054](../054_paw_ft_cells_win4090.md).

**Box state:** accuracy-only rows (the hosted lane's p50 ≈ 1 s/call includes their network; sibling sessions active). Determinism: none promised — hosted inference is not repeat-stable (the 049 finding, unchanged).

## Cells (acc with refusal = wrong; refusals in parens)

| suite | default (`paw-4b-qwen3-0.6b-20260407`) | `paw-ft-bs48` | Δ | modelless (Bench 052) | laya best |
|---|---|---|---|---|---|
| ag_news | 0.7825 (0) | **0.7900** (0) | +0.8 | **0.8825** | 0.9500 |
| emotion | 0.4725 (2) | **0.5000** (0) | +2.8 | **0.7375** | 0.5925 |
| sst5 | 0.3317 (0) | **0.3933** (0) | +6.2 | **0.3967** | 0.3717 |
| banking77 | 0.0960 (374/500 = 74.8%) | **0.4200** (172/500 = 34.4%) | **+32.4** | **0.8260** | 0.4220 |

## Reads

- **The "much higher accuracy" tier claim measures TRUE on every suite.** The marketing tier is honest: ft-bs48 beats their default compiler +0.8 to +32.4 pt at our protocol.
- **banking77 is the tier's whole story**: the default program refuses 74.8% (answers in its own vocabulary — the 049 finding, unchanged at the stratified sample); the ft program's refusal rate collapses to 34.4% and its answered-accuracy roughly holds — nearly all the gain is refusals converted to correct answers.
- **Against our lanes at the same sample**: modelless leads every suite (banking77 0.8260 vs ft 0.4200 — 2×); sst5 is the one near-tie (modelless 0.3967 vs ft 0.3933, both above laya 0.3717). PAW's compile-a-classifier category is measured, not quoted: a finetuned 0.6B-class specialist does not reach the modelless count-table lane on these suites.
- Latency p50 ≈ 0.95–1.05 s/call both compilers (their hosted network dominates; not a compiler axis).

## What ran

```
./target/release/harness --paw --skip-laya --suites ag_news,emotion,sst5,banking77 \
  --datasets-dir .raw/datasets_t20k --out /tmp/paw_default        # default compiler (all 4 programs CACHED)
PAW_COMPILER=paw-ft-bs48 PAW_COMPILE_ASYNC=1 ./target/release/harness --paw --skip-laya \
  --suites ag_news,emotion,sst5,banking77 --datasets-dir .raw/datasets_t20k --out /tmp/paw_ft
```

Compiles (anonymous tier, public programs, serial 1-concurrent): ag_news 132.0 s · emotion 263.8 s · sst5 87.9 s · banking77 109.7 s. Program ids recorded in `.raw/paw/programs.json` keyed (suite, compiler, spec-BLAKE3) — re-runs never recompile. Artifacts: `/tmp/paw_default`, `/tmp/paw_ft` (machine-local; this file is the committed record, `results.json` rows quoted in Issue 033).

## Open (Issue 033)

- Posture B (local runtime subprocess oracle, full-N deterministic cells) — the one remaining lane task.
- Site republish — deferred to the Issue-039 T5 both-hosts re-run (the site should move once, at a consistent protocol, not per-suite).
