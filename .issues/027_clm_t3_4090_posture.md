# Issue 027 — the CLM T3 4090 bench posture (+ the AgentJev measure-only row riding the same window)

**Status:** THE WINDOW EXECUTED 2026-09-25 (bench 034 post-renumber, reflex `967415c`,
site `ae79a87`) — the serving + measuring halves are DONE: the
determinism pin GREEN (8/8 byte-identical + head mtime unchanged, after
the warmup law below), the CLM cells landed on ALL 15 suites, the leak
columns rode the publish (.issues/024 T4), the README row + site table
carry the lane. REMAINS: the optional AgentJev gold-label row (a
DIFFERENT server — their jev_service; the next 4090 window can run it;
025's "do NOT auto-start" stands) and the site DEPLOY (M3-gated — this
box has no CF creds; `npx wrangler deploy` from the M3 discharges it).
The Rust-side halves LANDED on `develop`:
`src/harness/runner.rs` grows the `clm` column (`--clm`, feature-gated
`clm-lane`, the parity law in-module: THEIR `to_text` state prose +
their candidates-as-descriptions law; the same metrics tail as every
lane + the observed-repeat determinism check),
`tests/clm_determinism_pin.rs` (amendment 1's pin: 8-repeat
byte-identity + head-mtime-unchanged, loud UNSEEN without
`CLM_SERVE_URL`), `scripts/clm_serve_4090.sh` (the boot script: one
docker container — vLLM `--runner pooling` at their exact
`serve_qwen3_8b.sh` flags + `clm-serve` over the mounted head,
`--no-download` provenance posture). The serving window waits on the GPU
(no-concurrency rule). Filed 2026-09-24 from the 019 Gate-0 ratification
(commit `87b4779`; verdict `#Verdict: AGREE`); **RENUMBERED 026→027 same
day** — upstream dual-allocated 026 while this was in flight (the 4090
CUDA-lane bench issue, closed + removed at `6d6cd8c`); numbers are never
reused, the renumber consumes 027 exactly like a fresh allocation
(`/.highwater` = 27). This is the 4090-WINDOW issue: nothing here runs on
the M3. The lane that picks this up will not have the filing
conversation — everything it needs is in-file.

## ⛔ First lines — the no-concurrency rule (binds before anything else)

**This window NEVER runs alongside the perf-league re-pin cycles on the 4090
box.** vLLM at `--gpu-memory-utilization 0.35` contends for VRAM, and VRAM
contention reads exactly like a config-dependent failure (the Bench-649
divergence class). Check `nvidia-smi` + the perf-league state first; if a
league cycle is live, wait or book a different window. Serialized posture
only — one GPU job at a time, box state recorded beside every figure (power
source, free RAM, commit-vs-limit, concurrent jobs — the standing rule).

## Scope (from `.issues/019` T3, with the Gate-0 amendments baked in)

- vLLM Qwen3-8B pooling server (one-time boot, `--gpu-memory-utilization 0.35`)
  + `clm-serve` heads boot script under `scripts/` — their stack serves, our
  Rust measures (the ANE-lane split posture; no Python in OUR tree).
- The determinism pin executes HERE (N-repeat byte-identity, head mtime
  unchanged) — it could not run on the M3 (amendment 1).
- **Same-box latency law (amendment 2):** a CLM-on-4090 cell vs
  laya-on-M3-Metal (1312 ms p50/case) is NOT a comparison. Either laya gets a
  4090 cell (cuda posture; the lane exists — `laya-riir-cuda` since
  riir-infer Issue 002) or the table runs one latency column per box.
  Decide before the first cell lands; accuracy columns are box-independent
  and may mix.
- The "9× faster" figure (CLM vs Jev, their in-repo T-Rex cell) is measured
  against Jev's REMOTE endpoint, network included — it NEVER carries into our
  tables (amendment 2).
- First same-box cells: modelless vs laya vs clm on the 9 dataset suites +
  6 harness families (vs Jev BYO-key where the visitor lane exists).
- T4 (the optional T-Rex re-run under OUR protocol) is gated on this issue.
- T5 (`/bench` publish + README row: comparison lane, Apache-2.0 attribution,
  not affiliated) is gated on this issue.

## The riding row — AgentJev gold-label re-score (from `.issues/025`, amendment 4)

Optional, same window, same no-concurrency rule. The missing data point is
AgentJev's **gold-label** accuracy on our 400-case split (their published
79.25% is teacher-argmax agreement — a different protocol from our measured
74.45%). Their Apache-2.0 Python service runs on the 4090; our Rust harness
measures over HTTP. **Wire condition RESOLVED at the contract level
2026-09-24** (read at the pinned sha `a965ca8f`, clone removed): their
service is `POST /api/evaluate` (`jev_service/contract.py`), NOT the TypeSafe
systemone wire — the mapping is direct and client-side only:

| our wire | their `/api/evaluate` |
|---|---|
| noul question | `type: boolean` + `criteria` (or the default `TRUE`/`FALSE` candidates) |
| choice question | `type: choice` + `options` array |
| score question | `type: score` + `levels` (2..10 ordered descriptions) |

Response `{results: [{id, answers: […probabilities]}], usage: {wall_ms, …}}`
maps into `DecisionResponse` with no server shim. Latency column: record BOTH
client round-trip and their `usage.wall_ms` (server-side measure); prefer
client round-trip for cross-lane consistency. The serving lane-3 port itself
stays DEFERRED in `.issues/025` (checkable reopen triggers there) — this row
is measurement only.

## Order of work

1. Book a quiet 4090 window (no perf-league cycle; `nvidia-smi` clean).

> **EXECUTED-WINDOW NOTES (2026-09-25, bench 034 post-renumber).**
> * **The serialization law bit exactly as written**: the first
>   combined attempt ran the laya CUDA lane BESIDE the resident vLLM
>   (17.7 GB reserved at util 0.72) — ~13 min stuck on
>   `laya[typed]` (3.8-thread CPU-average — kernel-compile/CPU-gemm
>   posture, the contention signature) — and was KILLED; its cells were
>   discarded. The published run is TWO SERIALIZED PASSES: clm with vLLM
>   resident, laya untouched (the 4090 laya lanes carry over from bench
>   032 at the NEWER substrate `1afd4f8`; re-running laya at this box's
>   older checkout would regress the table — lane_sources disclose the
>   split).
> * **util 0.35 does not fit a 24 GB card**: the bf16 weights alone are
>   14.11 GiB; 0.35×24.5 GB = 8.6 GB dies with "No available memory for
>   the cache blocks". The dedicated-window posture is 0.72 (their
>   co-existence posture assumes a bigger card).
> * **The first-request usage quirk (the pin's cold-start finding)**:
>   the very FIRST request after a clm-serve boot answers correctly but
>   reports `usage.input_tokens = 0` — the answers are byte-identical,
>   the accounting is not. The lane + the pin send one fixed warmup
>   request first (never a case's — no cache pollution of measured
>   latencies).
2. Boot vLLM Qwen3-8B pooling + `clm-serve`; run the determinism pin first
   (a lane that cannot repeat byte-identically produces no cells).
3. CLM cells (the 9 suites + 6 families), then the same-box laya-4090 cell
   (or the per-box column decision, whichever was chosen above).
4. Optional: the AgentJev gold-label row (mapping table above).
5. Publish: T4/T5 of 019 + the 025 measure row → `/bench` + README, with the
   per-figure box provenance lines.
