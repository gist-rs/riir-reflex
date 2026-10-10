# Issue 085 — Drex v1.5 revisit: the open 9B release, the llama.cpp fork posture, and the missing site rows

**Status:** OPEN — staged (CUDA build + sha256-verified GGUF ready on C:); T0 owner gate blocks the measurement, T1 site publication is unblocked.

## Trigger

`nace-ai/drex-v1.5` opened 2026-10-09/10: the hosted "Drex 1.5" that Benches 073/125
deliberately scoped OUT ("different artifact, never a comparable cell") is now an open
artifact — a 9B decision model (Qwen3_5ForCausalLM backbone, 32 layers, hybrid 3:1
linear:full attention, plus a pointer head scoring every option from hidden states; one
forward pass per question, no generation; `choice`/`noul`/`score` — our decision_wire
shape). Decision Index 0.3.1 public 58.08 (#1 under 10B; Jev 1.13.0 = 57.96 — AgentJev
is our Jev lane). License CHANGED from the DLM's CC BY-NC 4.0 to **Nace AI Open RAIL-M
(modified) v1.0**. The `nace-ai/llama.cpp` branch `drex-v1.5`
(@ `e8f79610fa5aa460660439ca45aca45ea9d23df0`) serves BOTH archs (`edlm` DLM +
`qwen35`+pointer) over `POST /v1/systemone`. A prebuilt GGUF exists:
`nace-ai/drex-v1.5-Q8_0` (9,531,694,816 bytes, arch `qwen35`).

User ask 2026-10-10: "why do I not see it in reflex.gist.rs? btw github just release so
revisit needed."

## Why nothing on reflex.gist.rs (measured 2026-10-10)

The site's `/bench/` tables render from `reflex-site/data/bench.json`; `drex` appears
ZERO times in both `bench.json` and the publisher
(`reflex-site/scripts/publish_bench.py` — no `LANE_DISPLAY` entry, no filter/References
entry). Comparison lanes DO publish — `clm`/`gliner`/`agentjev`/`bekko` ship as
`(reference)` display rows — so Drex was simply never registered. The measured cells
exist in-repo only: Bench 073 (their Python serve.py BF16 CUDA on this 4090; DLM
typed_decisions 0.5865, sst5 0.5900, NOT-CALIBRATED), Bench 125 (M3, their edlm fork
Q8_0 Metal — independent reproduction), Bench 126 (det ✓ re-read). Registering the
lane on the site = the pplx work pattern (reflex-site `f1d9953`, "pplx joins the lane
filter + every chart"). NOTE: reflex-site is a registered contract repo that works on
`main` (it is not one of this session's working directories) — the edit lands from
whichever session owns site edits.

## T0 — OWNER GATE (blocks T2/T3): the RAIL-M §2(c) competition clause

This license is NOT "CC-BY-NC with a new name". Attachment A §2(a)/(b) (revenue and
funding thresholds) carry an "except where Your Use is limited to personal use or
research purposes" exception — **§2(c) has NONE**: use by any party that "provides or
otherwise makes available any product or service that competes with any product or
service offered or made available by Licensor or its affiliates, **including decision
models**" is unlicensed per §4, and §6 defines Use to include "running … evaluating".
Our stack IS decision-model serving, so whether §2(c) reaches even MEASUREMENT of
drex-v1.5 is a call about the owner's business — owner-gated, never asserted by an
agent. Options: (a) written permission / commercial license (§4 names
nischay@nace.ai); (b) owner rules we are not a §2(c) competitor; (c) skip v1.5
measurement. The same gate covers the distill-teacher law (§6 includes "distilling").
Disclosure: the Q8_0 GGUF was already downloaded to `C:\models` as run-staging before
the clause was read — license "reproduce" is a Use — so T0 also decides whether it
stays or is deleted. The OLD DLM stays CC BY-NC: measurement-only, never teacher,
never product (unchanged).

## Scope law (carried from Benches 073/125, new form)

A v1.5 cell is a NEW lane row (`drex-v1.5`, Q8_0 GGUF, `qwen35`+pointer arch, posture
disclosed) — NEVER a continuation of the Drex-DLM rows (0.5865 typed / 0.5900 sst5).
Board context on our frozen typed split: laya 0.7445 / rethink 0.7550 / AgentJev
0.7715 / modelless 0.6475. Numeric posture: Q8_0 vs their Python BF16 — their measured
cross-runner spread ≤0.034 per-option (A10G) / ≤0.019 (M5 Pro Metal), answers
identical; disclose beside the cell (the Bench-125 pattern). The lane reads the
artifact from the response's own `model` field (will read `drex-v1.5`); the lane's
meta string that hardcodes the DLM description needs a v1.5 variant.

## Tasks

- [ ] T0 owner gate: the RAIL-M §2(c) ruling (written permission via nischay@nace.ai,
      or the owner's non-competitor ruling, or skip v1.5) + decide the downloaded
      GGUF's fate (`C:\models\drex-v1.5-Q8_0.gguf`, sha256
      `7ff3285686e7bef7b477a37e6f260837381222c3e463cc802f61f617de2a4ed5`, matches HF LFS).
- [ ] T1 (NOT blocked by T0): publish the EXISTING Drex-DLM cells (Benches 073/125/126)
      to reflex.gist.rs — register the `drex` lane in `LANE_DISPLAY` + filter/charts +
      the CC BY-NC measurement-only disclosure, following the pplx pattern
      (reflex-site `f1d9953`). This alone answers the user's question permanently.
- [-] T2 (BLOCKED by T0, not by the GPU): measure drex-v1.5 on this box at the staged
      posture — serve: `C:\builds\nace-llama\bin\Release\llama-server.exe` (fork @
      `e8f79610…`, GGML_CUDA=ON, sm_89, VS2022 + CUDA 13.3) with
      `C:\models\drex-v1.5-Q8_0.gguf`:
      `llama-server -m C:\models\drex-v1.5-Q8_0.gguf --port 8097 -np 2 -c 32768 -b 16384 -ub 2048 --embedding --pooling none -ngl 99`
      (the `-b/-ub` flags are LOAD-BEARING — Bench 125's `GGML_ASSERT(n_outputs_max)`
      lesson), then
      `DREX_SERVE_URL=http://127.0.0.1:8097 cargo run --release --bin harness -- --drex --suites typed_decisions,sst5 --skip-laya --out .benchmarks/138_drex_v15_4090`
      (bench highwater read 137 at filing — re-check at write). Box note: the GPU is
      held by the sibling training run (19.6/24.5 GB, 2026-10-10) — the GPU freeing
      alone does NOT start this task; T0 does.
- [ ] T3 (after T2): publish the v1.5 rows via the same T1 registration + republish.

## References

- Benches 073 / 125 / 126 (the in-repo DLM cells) ·
  `.research/008_Drex_DLM_SystemOne_Lane.md` (2026-10-10 delta)
- `nace-ai/drex-v1.5` (model card, RAIL-M LICENSE) · `nace-ai/drex-v1.5-Q8_0` (GGUF)
- `nace-ai/llama.cpp` branch `drex-v1.5` @ `e8f79610fa5aa460660439ca45aca45ea9d23df0`
  — unique surface `tools/server/systemone.{h,cpp}` + `src/models/edlm.cpp` + pointer
  tensors; `ggml/src` untouched by the fork keywords; distill verdict NO-corpus
  (riir-refine `.distill/001_mining_queue_snapshot.md`, 2026-10-10 row)
- reflex-site `f1d9953` (pplx lane-registration precedent) · `publish_bench.py` LANE_DISPLAY
