# Bench 073 — Drex DLM comparison lane, T5 suite pass (typed_decisions + sst5 smoke)

**Date:** 2026-10-08 (shikuwa/4090) · **Lane:** `--drex` (Issue 073, T1–T4 landed `48206d5`)
**Verdict:** **T5 PASS — the lane measures end-to-end; the owned calibration cell reads
NOT-CALIBRATED (ECE 0.193 on typed / 0.283 on sst5).** Drex's card disclaims calibration;
its homepage markets it; the empirical read sides with the card.

## Scope lines (every table carries these)

- This is the **open 32K release** (`nace-ai/drex-dlm` @ `6c63df2`, Efficient-DLM-8B
  backbone), **not hosted Drex 1.5** (128K, homepage edition 0.2.1) — different artifact,
  never a comparable cell.
- Weights **CC BY-NC 4.0**: measurement only. Comparison lane yes; distill teacher NO;
  product serving NO.
- **Serving posture: their Python `serve.py` on CUDA** (bf16, this 4090) — the reply's
  own `model` field ("drex-dlm") + `DREX_SERVE_URL=http://127.0.0.1:8000` disclose it.
  Their llama.cpp `edlm` fork (8097) is the alternative numeric posture (GGUF-vs-BF16
  spread is THEIR measured 0.0050 sample / 0.0094 255-option — a second posture would
  carry the same disclosure).

## Box state (the provenance law)

`bench_preflight.ps1` at run start: **`PROVENANCE: power=AC scheme=high load=0.72
swap=4242MB gpu=5 %, 15633 MiB canary=skipped`** — preflight PASSED (the swap warning is
the standing 4.2 GB resident set, disclosed). In-run end-capture: power=AC, mode=high,
load_1m 0.24, latency_quotable=true. GPU carried the resident 15.5–15.9 GB model (a lane
server is legitimately busy while serving timed requests — disclosure, not contention).

## Serving setup (the turnkey record)

- Venv: `.raw/drex-env` (uv, py3.11): torch 2.6.0+cu124 (CUDA live), transformers
  5.19.0, pydantic 2.13.x, hf_hub 0.36.x — their `requirements.txt` pins satisfied
  (transformers >=5.17,<6 is LOAD-BEARING: their `from_pretrained(..., dtype=)` spelling
  is 5.x).
- Weights: `nace-ai/drex-dlm` snapshot (~16 GB BF16) → `.raw/drex-model` (hf_hub
  snapshot_download, 29 files, 8m39s, unauthenticated).
- Boot: `.raw/drex_serve.py` (chdir + argv shim over their `serve.py`) → weights load
  399/399 tensors (~96 s), **15.5 GB VRAM resident**, `/health` →
  `{"status": "ok", "model": "drex-dlm"}`.
- Warm sanity: their `examples/request.json` → team billing 0.9686 (published 0.9641),
  refund 0.8522 (published 0.8548), urgency argmax URGENT exact, **87 input tokens
  EXACT** (their published count; also riir-infer 1005 T6's count) — argmaxes exact on
  all three, deltas ≤ 0.012 (inside their own 0.0050–0.0094 cross-runner class).

## typed_decisions (400 cases / 2000 questions)

| metric | value |
|---|---|
| **accuracy** | **0.5865** |
| ece(maxp) over their per-option probabilities | 0.1082 |
| **readout_ece** (their `confidence` fields vs correctness) | **0.1929** |
| **readout_brier** (same pairs — the Issue-073-T4 axis) | **0.2663** |
| hard brier / nll | in `results.json` (`hard`) |
| latency p50 / p99 (support) | 98.0 / 157.0 ms (5) |
| determinism (observed-repeat, first 10) | **✗ flagged** (see below) |
| server latency sum / tokens | 39,672 ms · in 191,962 · out 164,821 |

Per-primitive (`by_question_type`): **choice 0.6167** (ece 0.1205) · **noul 0.5717**
(ece 0.1874) · **score 0.5750** (ece 0.0618).

Board context (the frozen split's published rows): laya-riir·typed 0.7445, rethink
typed 0.7550, AgentJev 0.7715 (det ✗), modelless 0.6475 (served H2). Drex 0.5865 sits
between the modelless lane and the trained lanes on THIS suite — the new-open-weights
Decision-Index-top does not transfer to the typed board's bar.

## sst5 (smoke; 120 cases / 600 questions)

| metric | value |
|---|---|
| **accuracy** | **0.5900** |
| ece(maxp) | 0.0785 |
| readout_ece / readout_brier | 0.2827 / 0.3143 |
| latency p50 / p99 | 35.0 / 45.0 ms |
| determinism | ✗ flagged (first 10) |
| server latency sum / tokens | 20,979 ms · in 32,559 · out 66,108 |

⚠ **Board consequence (disclosed, not seated):** sst5 0.5900 is ABOVE every published
sst5 bar — gliner 0.4383 (the named bar), reflex A1 0.4217 (the served arm), even the
encoder class 0.5267 (the Issue-014-closed specialist). Drex is a comparison LANE (CC
BY-NC: never a product arm, never a teacher); the lane's number re-prices the board's
"best published lane" row for sst5 — the amended Issue-008 bar ("beat the best
published lane on every suite it sells") now names Drex there. Seating/acting on it is
owner-gated (the license law bars the direct answer).

## Determinism flag (the honest read)

`determinism_ok = false` on both suites (10/10 rerun pairs byte-differ). Their forward
is a bf16 CUDA computation with no determinism guarantee (their server makes none;
their validation file separately documents load-wobble). The probabilities round to 4
decimals AFTER a non-bit-deterministic reduction — identical requests legitimately
produce last-digit deltas. The lane law holds: the det column is FLAGGED, never
assumed, and the accuracy/ECE cells read the first pass only.

## Latency note

p50 98 ms (typed, mean ~96 q/request ≈ 1 ms/question server-side) and 35 ms (sst5,
~5 q/case ≈ 7 ms/question) are CLIENT round-trips including their Python `ThreadingHTTPServer`
hop. First-hit cold was 6.1 s (kernel autotune); the warm numbers above are the
comparable class. Not directly comparable to the in-process lanes' µs rows — the
cross-lane latency comparison law (the laya-python IPC disclosure) applies.

## Files

- `.benchmarks/073_drex_t5/results.json` + `TABLES.md` (typed_decisions)
- `.benchmarks/073_drex_t5_smoke/results.json` + `TABLES.md` (sst5)
- Lane: `src/lanes/drex.rs` (T1–T4 @ `48206d5`); issue: `.issues/073_drex_systemone_lane.md`
