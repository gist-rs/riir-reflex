# Research 007 — local Clef serving rigs: the M3/MLX lane and the 4090/EXL3 lane

**Status:** DISTILLED — rigs operational (M3) and specified (4090); plan 011 Phase C/D1 consume this.

**Sources (fetched + read 2026-10-03):** `Cloudflare/clef` + `Cloudflare/clef-flash` HF repos
(model cards, file lists, blob sizes, `joint_schema_model.py` full read), `mlx-community/clef-{,flash-}{4,8}bit`,
`ramgpt/clef-EXL3` (+ `VALIDATION.json`), `simonlehmann/clef-NVFP4`, `prithivMLmods/clef-FP8`,
`bartowski/Cloudflare_clef-GGUF`, exllamav3 releases/docs, vLLM platform docs, the Cloudflare Clef blog.

## TL;DR

Clef runs locally on BOTH our boxes without any Workers AI credential — the weights are public,
ungated, Apache-2.0. The M3 lane (this box) runs `mlx-community/clef-flash-4bit` through the
community MLX port's own HTTP server plus a thin translation layer; the 4090 lane's only clean route
is `ramgpt/clef-EXL3` (4.0 bpw, 18.15 GB, head preserved via bridge adapter, validated ON a 4090).
GGUF is confirmed headless (the recorded blocker, now evidence-backed): llama.cpp serves clef as a
chat VLM and never runs the joint schema head, so its outputs are not typed decisions.

## The wire contract (read from `joint_schema_model.py` + `clef_mlx.py`, not guessed)

* Request: `{"model": <str, required>, "state": <any>, "questions": {<qid>: {"type":
  "noul"|"choice"|"score", "instructions": str, "criteria": ...}}}` — questions is a **dict keyed by
  id**, the text field is **`instructions`**, and noul auto-injects true/false criteria when
  `criteria` is null. Our harness lane sends a LIST with `question` — the translation is
  list→dict + question→instructions (implemented in `.raw/clef_srv/clef_lane_server.py`).
* Response: `{"model", "answers": {qid: {type, ...}}, "usage"}` where noul carries
  `noul: P(true)`; choice carries `choice` + `confidence` + `probabilities{option: p}`; score
  carries `score` = the **expected value** Σ i·pᵢ (NOT the pick — a wrapper must derive the argmax
  `level` itself), `confidence` = max p, `probabilities` keyed `"0".."n-1"`.
* `encode_record` renders the state + a `SCHEMA FIELDS` block and tracks token spans; the head
  cross-attends question/option spans against the backbone's `last_hidden_state` with option
  embeddings taken from the **`lm_head` rows** (why every head-preserving quant keeps `lm_head`
  unquantized). Prefill-only (`use_cache=False`) — no decode loop anywhere.

## The M3 lane (operative, 2026-10-03)

`mlx-community/clef-flash-4bit` (6.2 GB, U32-packed 4-bit backbone + BF16 head) via the repo's own
`clef_mlx.py` — `mlx 0.32.3 / mlx-lm 0.32.0 / mlx-vlm 0.7.4` (the versions the port declares as
tested; pinned in `.raw/clef_srv/.venv`). The repo ships a `serve` subcommand (stdlib HTTP,
`POST /v1/systemone`); the thin lane server adds our list-shape → dict-shape translation and the
Workers-AI envelope (`result.results[0].answers[]`, argmax `level`) the strict lane maps.
Same recipe covers `clef-8bit`, `clef-4bit` (the 27B, ~16 GB — fits 64 GB unified), and
`clef-flash-8bit` by swapping the repo id.

⛔ **Determinism trap (caught pre-run):** the lane's determinism probe byte-compares two raw
replies; any wall-clock field in the envelope (`usage.server_wall_ms`) fails it falsely. The lane
envelope must be byte-deterministic — usage lives on the vendor pass-through wire only.

## The 4090 lane (specified, not yet built)

Routes measured against a 24 GB Ada card on Windows-native (no WSL):

| Route | Fits 24 GB? | Head survives? | Windows-native? | Verdict |
|---|---|---|---|---|
| **`ramgpt/clef-EXL3` 4.0 bpw** | ✅ 18.15 GB weights | ✅ bridge adapter (`clef_exl3.py`) + `VALIDATION.json` | ✅ win_amd64 wheels (exllamav3 1.5.3, cu128/torch 2.10, cp310–314) | **the route** |
| NVFP4/FP8 (vLLM pooling) | barely, and **Blackwell-only kernels** | ✅ | ❌ vLLM has no Windows build | reject on this card |
| FP8 W8A8 compressed-tensors | ❌ 35.9 GB | intended ✅ (repo head files missing at HEAD, copy from base) | transformers yes | reject on size |
| GGUF (bartowski et al.) | ✅ | ❌ **headless** — mmproj is vision-only | ✅ | reject: not typed outputs |
| BF16 transformers (reference) | ❌ ~55 GB (H200-class) | ✅ | — | the golden, not local |

Fidelity: EXL3 4.0 bpw measured (repo's own validation) mean |Δp| **0.0071**, max **0.0178** vs
BF16, discrete decisions matched — decisions-grade, not bit-exact; no 24 GB route is bit-exact for
the 27B. Build recipe: Python 3.12 venv → `torch==2.10.0+cu128` → the exllamav3 win_amd64 wheel
(pulls `triton-windows`) + `transformers>=5.10` + `huggingface_hub`; `snapshot_download("ramgpt/clef-EXL3")`;
`sys.path.insert(0, path)`; `from clef_exl3 import ClefEXL3`; wrap in the same translation server.
Latency expectation (repo/released notes): ~1,800-token record ≈ 0.7 s prefill at 4090 rates + 3–6 ms head.

## Honest caveats

1. **Provenance split.** The hosted Workers AI run (plan 011 A6, still owner-gated) remains the
   board-fidelity cell — the vendor's own serving posture. A local quantized cell (4-bit MLX or
   EXL3) is a *different* posture: published rows must carry the quant + box in the posture line,
   never mix with hosted rows in one column.
2. **The community MLX/EXL3 ports are third-party** (mlx-community, ramgpt). The M3 numbers inherit
   their conversion fidelity; the EXL3 repo at least ships a VALIDATION.json + reference answers to
   diff against.
3. **`clef` (27B) ≠ `clef-flash` (9B)** on quality — the 4090 lane's purpose is the flagship cell;
   the M3 27B cell (`clef-4bit`, ~16 GB) is the cheaper way to get the flagship locally if the
   4090 window stays busy.
4. Version pins on the MLX port are load-bearing (`_check_versions` warns off-pin); an mlx/mlx-lm
   bump needs a re-probe before any new number.

## Feeds

* Plan 011 Phase C (this repo) — the `--clef` lane's local posture, running.
* Plan 011 Phase D1 (riir-infer, deferred there) — the EXL3 route is the prefill-league workload
  spec when a 4090 window opens.
