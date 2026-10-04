# Bench 123 — Issue 065 T3: the four 4090 lane re-runs, self-judged by the new Windows box-state probes

**Status: MEASURED — all four @4090-win lanes (openthai, clm, agentjev, gliner) re-run with
`latency_quotable: true` at BOTH capture points — the first end-to-end proof of the T3(b)
Windows probe arm (`box_state.rs` `capture_windows`, reflex `00bedc0`) inside real harness
results. Every lane's modelless control cells bit-identical to the published board (the drift
guard's premise held on all four docs).**

Date: 2026-10-04/05 · host `4090-windows` (REFLEX_BENCH_HOST) · reflex `5a52ff9` ·
binary `target/release/harness` (`--features clm-lane`), one window, four serialized passes.
Posture: the board-canonical selection flags `--nb-select --oc-select --ridge-select`,
canonical pool `.raw/datasets` (the box's 10-01 pull == the m3's canonical pull, byte-verified
by the modelless drift gate), `--skip-laya` (the laya@4090 cells were never the backlog).

## The runs (all quotable)

| lane | suites | box state (start → end) | accuracy sanity |
|---|---|---|---|
| openthai | 11 (9 dataset + thai pair) | load 0.24 → 4.8 · AC/high · gpu 4114 MiB | every cell == the published lane (xnli 0.9000, massive 0.9200, typed 0.5360, …) |
| clm | 8 (dataset, no code_fixtures — the publisher's cross-host carve-out) | load 0.72 → 2.4 · AC/high · gpu 19888 MiB (vLLM resident) | cells == the bench-034 era (xnli 0.6200, prompt 0.5345, typed 0.3465, banking77 0.0200) |
| agentjev | 8 | load 1.92 → 0.24 · AC/high · gpu 5895 MiB | typed 0.7720 (published 0.7715 — the one-question bf16 wobble class, disclosed in bench 039); sst5 0.4383 == published |
| gliner | 8 | load 0.24 → 2.88 · AC/high · gpu 844 MiB | typed 0.5280 / sst5 0.4383 / xnli 0.4767 / emotion 0.5650 == bench-037 exactly; massive 0.7267 / banking77 0.7100 are the fresh-read cells (see below) |

PROVENANCE (per run, the Windows arm's own emission): all four carry
`power=AC Power scheme=high`, swap ≤ 1080 MB, `load_source=cpu_util_pct_x_cores/100`,
gpu fields disclosure-only. No refusals at either capture point.

## What had to be fixed to land this (the window's findings)

1. **The gliner oracle crashed on typed_decisions + prompt_injections — the cp874 stdin
   decode.** The lane scripts reconfigured stdout/stderr to UTF-8 but NOT stdin; on this
   box Python decodes the harness's UTF-8 JSON pipe with the ANSI codepage, mis-decoding
   case text (em-dashes → mojibake, some bytes → `\udcXX` lone surrogates — the crash
   capture: `UnicodeEncodeError: '\udc99' surrogates not allowed`, then the mis-decoded
   text's token/schema misalignment reads as gliner2's `IndexError` at
   `processor.py:_extract_embeddings_fast`). **Fix (landed with this bench):
   `sys.stdin.reconfigure(encoding="utf-8", errors="backslashreplace")` in
   `scripts/gliner_lane.py` + `scripts/bekko_lane.py` + `scripts/laya_python_lane.py`** —
   the JSONL-subprocess lane family's missing half of the cp874 law. Both suites then
   measured cleanly (deterministic across two runs).
   ⚠ This means every gliner/bekko cell previously measured through a cp874-stdin pipe on
   this box carried mojibake'd non-ASCII case text: the cells that match exactly
   (typed/sst5/xnli/emotion/prompt/ag_news vs bench 037) do so because their ASCII
   majority dominates the pick; **massive_intent_en (0.7267 now vs 0.8233 in bench 037)
   and banking77 (0.7100 vs 0.7060) are the suites with enough non-ASCII text that the
   corrected UTF-8 decode MOVES the cell** — the new numbers are the honest ones (the
   text the engine sees is finally the dataset's actual text); the publish updates them.
2. **The torch in a fresh `uv venv` on Windows is CPU-only** — both the agentjev and
   gliner venvs needed `--index-url https://download.pytorch.org/whl/cu124` for the CUDA
   build (the `--device cuda:0` boot otherwise dies with "Torch not compiled with CUDA
   enabled"). Recorded here for the next re-stage.
3. **The gliner2 2.0.0 + transformers pairing**: transformers 5.18.0 and 4.57.6 both work
   once the stdin codec is right (the IndexError was the codec, not the transformers
   version — measured both ways). The venv carries 4.57.6.

## The load-refusal captures the probes made honestly (the instrument working)

- The FIRST openthai attempt (23:17 +0700) measured all 11 cells but its END capture read
  load 7.44 (my own concurrent venv staging + a sibling `kv_reconstruct_gate` run at ~1.9
  cores) — `latency_quotable: false`, refused. Re-run after staging completed: quotable.
- The gliner run before the sibling finished: end load 8.16 — refused. Re-run on the
  quiet box: quotable. The accuracy cells were bit-identical across both gliner runs
  (deterministic oracle) — only the timing verdict moved.

## Staging notes (the `.raw` re-stage ledger, for the next window)

Everything re-staged from caches in ~50 min: CLM repo @ `cca045ffd` + head (HF) +
`.raw/models/Qwen3-8B` (materialized from the surviving HF cache — no 16 GB re-download);
`gliner-env` (gliner2 2.0.0 + torch 2.6.0+cu124 + transformers 4.57.6); `agentjev` repo @
`a965ca8f` + their aimeigaoshou tensors (torch-wrapped once → `final.pt`, 343 tensors) +
Qwen3-0.6B backbone + `agentjev-env`; the openthai env survived from Sept (nothing to do).
The clm docker stack re-created by `scripts/clm_serve_4090.sh` (image was still local;
`GPU_UTIL=0.72` posture, one fixed warmup request before the lane — the bench-034 laws).

Raw: `results.{openthai,clm,agentjev,gliner}.json` beside this file (the four docs the
publish consumed).
