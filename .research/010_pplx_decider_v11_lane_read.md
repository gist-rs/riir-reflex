# Research 010 — pplx-decider v1.1: the JDI v3 #1 measured on OUR suites (the 082 lane read)

**Status:** RECORD — measured 2026-10-09 (bench `132_pplx_lane/mlx4bit`); the teacher decision it
feeds is recorded in riir-train Issue 623. This note is the distillation record of the READ, not a
mining candidate (no Rust corpus material; the lane-intel axis is the deliverable and it landed).

## TL;DR

Perplexity's pplx-decider-v1.1-27b — the JDI v3 board's #1 (balanced_skill 62.75 blended; public-only
#2 at 62.25 behind Torchcast 65.10) — measured on our 9 comparison suites at the clef-lane posture
class (local MLX 4-bit, community quant, byte-deterministic adapter rig): **board rank does not
transfer to our suites in either direction, at extreme amplitude.** It loses banking77 to clef-flash
by 15pt (0.8040 vs 0.9540) while winning prompt_injections by 34pt (0.9224 vs 0.5862), xnli by 14pt
(0.9567 vs 0.8133), typed by 4pt (0.7350 vs 0.6955). The JDI language-category lead (pplx 0.6748 vs
clef-27B 0.6127 skill) does NOT predict per-suite dominance — the category aggregates different
benchmarks than ours.

## The read that matters (all acc, our splits, our measuring)

| suite | pplx | clef-flash | Δ | reading |
|---|---|---|---|---|
| banking77 | 0.8040 | **0.9540** | −15.0pt | clef's home turf (retrieval-flavored) |
| massive_intent_en | 0.9300 | **0.9333** | −0.3pt | tie |
| sst5 | 0.5700 | **0.6033** | −3.3pt | clef edge |
| xnli_en | **0.9567** | 0.8133 | +14.3pt | pplx's home turf (inference-flavored) |
| emotion | **0.6400** | 0.5925 | +4.8pt | pplx edge |
| prompt_injections | **0.9224** | 0.5862 | +33.6pt | pplx dominates |
| typed_decisions | **0.7350** | 0.6955 | +4.0pt | below the Rethink head's 0.7550 record-only |
| ag_news | 0.8900 | 0.9000 | −1.0pt | tie |
| code_fixtures | **0.6875** | 0.5938 | +9.4pt | n=32, wide CI |

det 10/10 verbatim byte-compare on every suite; ECE 0.033–0.080 on 8/9 — well-calibrated out of the
box (their training's calibration objective shows up in third-party measuring).

## Distill verdict (per the `distill` skill shape)

- **Verdict line: NO (riir-refine corpus) / LEAGUE-INTEL HIGH (consumed same-day) / no product-coverage axis.**
- **Corpus axis:** a Python/transformers/MLX serving stack — no Rust rule material. Not a mining candidate.
- **Lane-intel axis (consumed):** the reflex pplx lane (T1, `ecaa427`) + the measurement (bench 132) +
  the site publication (edition 2026-10-9) + the 623 teacher decision — the full chain this note records.
- **Product-coverage axis:** n/a.

## The serving postures (what exists, what fits where)

| posture | fits | verdict |
|---|---|---|
| bf16 transformers (their reference, 52 GB) | H200-class / none of ours | the golden; the 4090 snapshot (`E:\pplx\`) is the verified local copy |
| bf16 accelerate offload (GPU+CPU) | needs 52 GB RAM+VRAM resident | **DEAD on the 4090 (32 GB RAM)** — the T0 plan's hole; disk-spill mmap ≈ 15-20 h AND pins the card |
| **MLX 4-bit community quant (15.3 GB)** | **m3 (64 GB unified)** | **THE lane posture** — 33/33 argmax vs bf16, KL 0.0014 (the port's own validation); the clef-lane class |
| GGUF (llama.cpp) | — | headless by construction (the readout + noncausal mode are transformers-custom — the clef GGUF lesson, same class) |

Rig laws that mattered (all recorded in the bench + the adapter header): byte-deterministic bodies
(the port's own server emits a uuid `id` — false-fails the verbatim det probe); one request = one
predict (deterministic batch composition); reference validation semantics (the port's server is
STRICTER than the reference on optional `instructions` — would 400 our shape).

## Honest caveats

1. Quantized posture (4-bit, decisions-grade not bit-exact) — every clef-flash cell carries the same
   class of caveat since bench 113; their bf16 hosted row is a different posture, never pooled.
2. Latency unquotable under sibling load (acc-only publish; the measured p50s live in the source doc).
3. n=32 on code_fixtures (wide CI); n=116 on prompt_injections.

## Feeds

- reflex `.issues/082` (the lane issue, T0-T5 all landed) + bench `132` (the read) + the site edition
  `2026-10-9` (the publication).
- riir-train `.issues/623` (the teacher decision: per-suite teachers — clef-flash banking77/sst5/
  massive, pplx xnli/emotion; xnli's 0.9567 clears the 0.9000 class bar as a teacher read).
- `.research/005` (the JDI record; the Update 2026-10-09 note pins the v3 edition this lane measured).
