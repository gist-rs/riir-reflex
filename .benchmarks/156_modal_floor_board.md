# Bench 156 — the modal-floor board read (issue 094 T1, first full-suite census)

**Status:** RECORD — T1 validation; report-only (issue 094 T5: no exit-code coupling).
**Date:** 2026-10-22 · **Host:** shikuwa (4090 box) · **SHA:** `ee1c2a4` · **Profile:** release
**Datasets:** `.raw/datasets` (canonical) · **Posture:** `--skip-laya` (modelless lane only,
registry defaults) · **Command:** `cargo run --release --bin harness -- --skip-laya --out
.benchmarks/156_modal_floor_board`
**Box state:** AC power; the GPU carried a sibling session's StartLux capture (llama-server
resident) for the whole window — IRRELEVANT to these numbers: the modelless lane is
CPU-only (p50 0.02–0.29 ms/question) and NO latency figure is quoted here. Accuracy rows
are load-independent (deterministic engine, byte-identical across two same-sha runs).

## What ran

The issue-094 T1 input-keyed modal-answer floor (`src/harness/modal_floor.rs`, landed
`b884210`, redesigned `ee1c2a4`) over all nine dataset suites. The floor is the
arXiv:2610.17495 anti-mimicry control: modal gold over a reference population keyed by
(first-4 normalized input tokens × qid × option-count), pooled by option IDENTITY,
mapped through each case's own option order; ties → global prior → case order. Census
side = the CAL slice (disjoint by slice-guard); LOO only without a cal slice (none
here — every suite rode `cal`).

## The board

| suite | lane acc | modal floor | ref-modal share | keys | fallbacks | verdict |
|---|---|---|---|---|---|---|
| typed_decisions | 0.2940 | **0.4640** | 0.2280 | 80 | 40 | ⛔ FLAG (lane ≤ floor) |
| ag_news | 0.4050 | 0.2625 | 0.2500 | 195 | 398 | above |
| emotion | 0.2750 | **0.3125** | 0.1700 | 162 | 267 | ⛔ FLAG |
| sst5 | 0.2017 | 0.1383 | 0.2000 | 188 | 578 | above |
| prompt_injections | 0.4828 | **0.5345** | 0.5000 | 84 | 107 | ⛔ FLAG |
| xnli_en | 0.3400 | 0.3333 | 0.3350 | 198 | 288 | above |
| massive_intent_en | 0.6100 | 0.1100 | 0.0200 | 175 | 265 | above |
| banking77 | 0.3940 | 0.0460 | 0.0150 | 148 | 311 | above |
| semantic_defects | 0.1863 | 0.1667 | 0.2778 | 18 | 102 | above |

(Lane numbers = the modelless lane's forced accuracy at registry defaults — the same
rows the run's TABLES.md prints; the floor block now rides every future harness table.)

## Findings

1. **Three genuine flags.** typed_decisions (0.2940 vs 0.4640), emotion (0.2750 vs
   0.3125), prompt_injections (0.4828 vs 0.5345): the modelless engine underperforms a
   pure input-keyed lookup on these suites — the anti-mimicry control doing its job.
   (Context, not contradiction: the instinct board's seated rows read different
   postures — winner bridges, tuned caps, fused gates. This board is the BARE harness
   engine; the floor now sharpens every posture read that follows.)
2. **xnli floor ≈ chance (0.3333 ≈ share 0.3350)** — the honest read: premise-prefix
   buckets carry no NLI signal; the floor degenerates to the majority prior and SAYS SO
   via `global_modal_share`.
3. **massive/banking77 floors are low but valid** (0.1100 / 0.0460): sampled per-case
   distractors + 59/77-way label spreads → sparse cal pools (265/311 fallbacks) → the
   census mostly serves the cal global modal. The fallback counts make the sparsity
   visible rather than hidden.
4. **The first read caught the instrument's own defects** (pre-`ee1c2a4`, recorded for
   the class): (a) position pooling was invalid on per-case-shuffled suites (massive,
   typed) — a gold INDEX is not an identity; (b) LOO over eval is ANTI-correlated in
   small pools — balanced buckets read floors of EXACTLY 0.0000 (xnli, semantic_defects).
   The exact-zero signature is the tell: a census systematically wrong is a census
   leaking its own construction, not a weak baseline. Both fixed at root cause;
   `reference` + fallback counts now disclose the census side and its sparsity.
5. **Determinism:** two same-sha runs byte-identical (integer counts, fixed tie rule —
   the Issue-940 law holds at the table level, not just in unit fixtures).

## Records

- `results.json` / `TABLES.md` in this directory carry the full meta (modal-floor
  definition line included) and the per-suite floor blocks with the ⛔ flags.
- Module gates: 9 unit tests (identity mapping, global-prior tie, LOO disclosure,
  case-order determinism, fallback accounting); clippy `-D` ×3 postures clean; 452 lib
  tests + integration targets green.
