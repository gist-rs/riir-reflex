# Bench 132 — the pplx lane's first cells: local `pplx-decider-v1.1-27b-4bit` on the M3 (082 T2, the 623 teacher read)

**Status: MEASURED — the standing local rig (`mlx-community/pplx-decider-v1.1-27b-4bit` served by
`.raw/pplx_srv/pplx_mlx_adapter.py` @ `127.0.0.1:8793`, the reference wire) answered all 9 comparison
suites. Published acc-only (the Issue-021 wall: preflight REFUSED — load 21-23, canary +15%, swap in
use — the bench-131 law under sibling load; the source doc keeps its measured timing + box_state).**

## What ran

- Lane: `--pplx --skip-laya` over all 9 comparison suites, one doc, `--out .benchmarks/132_pplx_lane/mlx4bit`.
- Model: **Perplexity pplx-decider-v1.1-27b** (`perplexity-ai/pplx-decider-v1.1-27b` @ `3b45dea`, Apache-2.0,
  ungated — the JDI v3 board's #1, balanced_skill 62.75) at the **local MLX 4-bit posture**:
  `mlx-community/pplx-decider-v1.1-27b-4bit` (community quant, affine 4-bit g64 backbone; readout + vision
  tower bf16; 15.3 GB) — the clef lane's posture class exactly (bench 113/118/119 precedent), so the
  pplx-vs-clef cells stay posture-consistent. The port's own validation vs their bf16 reference:
  **33/33 argmax agree, mean KL 0.0014, max |Δp| 0.029** (README; single-row AND batched).
- Rig: `.raw/pplx_srv/pplx_mlx_adapter.py` (gitignored operator rig, the research-007 pattern) over the
  port's `decider_mlx.py` engine: reference wire (`/health` reference shape; `POST /v1/systemone` their
  dialect), **byte-deterministic bodies** (no per-request ids/clocks — the port's own server emits a
  uuid `id` that would false-fail the verbatim det probe), **deterministic batching** (one request = one
  `predict()`; the port's cross-request Scheduler deliberately NOT used), reference validation semantics
  (instructions optional; aliases checked; score 2..=10; noul keys; images ≤ 4), single lock (529 busy).
- Env pins: the port's `requirements.txt` (mlx 0.32.3 / mlx-vlm 0.7.6 / transformers 5.19.0) in
  `.raw/pplx_srv/.venv`. `PPLX_DEVICE="m3-max-mlx-4bit-local"` rides the provenance.
- Binary: HEAD `0413dcd` (release, harness `--release`). Host m3. Box state at start:
  `bench_preflight` REFUSED — load 21.21→22.95, canary 180.5 µs vs 141 ref (+15%), swap 28-6 GB used,
  power AC/high → **latency unquotable, acc-only publish** (`PUBLISH_BENCH_LANES=pplx:acc-only`).

## The read (acc, our splits, our measuring — never board-comparable)

| suite | pplx acc | macro F1 | ECE(maxp) | clef-flash (published) | Δ acc | det |
|---|---|---|---|---|---|---|
| typed_decisions | **0.7350** | 0.7220 | 0.0800 | 0.6955 | +0.0395 | 10/10 |
| ag_news | 0.8900 | 0.8807 | 0.0560 | 0.9000 | −0.0100 | 10/10 |
| emotion | **0.6400** | 0.5365 | 0.0749 | 0.5925 | +0.0475 | 10/10 |
| sst5 | 0.5700 | 0.5138 | 0.0574 | 0.6033 | −0.0333 | 10/10 |
| prompt_injections | **0.9224** | 0.9223 | 0.0589 | 0.5862 | **+0.3362** | 10/10 |
| xnli_en | **0.9567** | 0.9568 | 0.0413 | 0.8133 | **+0.1434** | 10/10 |
| massive_intent_en | 0.9300 | 0.9268 | 0.0333 | 0.9333 | −0.0033 | 10/10 |
| banking77 | 0.8040 | 0.7898 | 0.0653 | 0.9540 | **−0.1500** | 10/10 |
| code_fixtures | 0.6875 | 0.4264 | 0.1863 | 0.5938 | +0.0937 | 10/10 |

det = the lane's observed-repeat verbatim byte-compare, first 10 cases per suite — **green everywhere**
(the adapter's determinism laws held: identical inputs → identical bytes).

- **Board rank ≠ our-suite rank, both directions, at extreme amplitude**: the JDI #1 loses banking77 to
  clef-flash by 15pt while winning prompt_injections by 34pt and xnli by 14pt. No single "best model"
  exists across our suites — the per-suite profile is the decision surface (the B5 law, now measured).
- typed_decisions context: pplx 0.7350 sits above clef-flash (0.6955) and AgentJev (0.7715 gold-label,
  det ✗) territory, below the Rethink typed head's record-only 0.7550.
- Calibration: ECE 0.033–0.080 on 8 of 9 (code_fixtures 0.186 at n=32) — a well-calibrated lane
  out of the box (their training's calibration objective shows).

## The 623 teacher read (why this bench exists)

Language-class suites (the trailing set + the NLI family):

| suite | pplx | clef-flash |
|---|---|---|
| banking77 | 0.8040 | **0.9540** |
| massive_intent_en | 0.9300 | **0.9333** |
| sst5 | 0.5700 | **0.6033** |
| xnli_en | **0.9567** | 0.8133 |
| emotion | **0.6400** | 0.5925 |

Mean: **pplx 0.7801 vs clef-flash 0.7793 — a dead heat with opposite strengths**. clef-flash owns the
retrieval-flavored suite (banking77) massively; pplx owns the inference-flavored ones (xnli +14pt,
emotion +5pt). The 623 teacher decision is therefore NOT a single-number call — see the note in
riir-train Issue 623 (the per-suite profile decides, possibly per-suite teachers).

## Honest caveats

1. **Quantized posture.** The cells measure the community 4-bit MLX quant, not their bf16 serve (the
   port's fidelity: 33/33 argmax, KL 0.0014 — decisions-grade, not bit-exact; the same class of caveat
   as every clef-flash cell since bench 113). Their bf16 hosted posture is a different row, never pooled.
2. **Unquotable latency.** p50s measured under sibling load (typed 16.9 s per 5-question request,
   singles 1.0–4.7 s) — recorded in the source doc, NOT published (acc-only). A quiet-box re-read is
   the bench-118 precedent away.
3. **The JDI-crosswalk rows array** carries these cells digest-pinned to the same populations as clef's
   (the pin asserted at publish; exclusions disclosed, never mixed).

## Provenance

- Provenance line: `autojev-qwen3.8-27b@pplx-decider-v1.1-27b-4bit` on `m3-max-mlx-4bit-local`
  (read from `/health`, never hardcoded — the gliner law).
- Site: edition 2026-10-9 (the lane-set basis bump); lane `pplx (local)`; timing methodology row 14.
- Issue: `.issues/082_pplx_decider_v11_lane.md` (T2/T3). Teacher note → riir-train Issue 623.
