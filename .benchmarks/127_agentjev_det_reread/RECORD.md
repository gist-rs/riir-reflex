# Bench 127 — the agentjev det re-read: the det ✗ was the timing tail, measured

**Status: MEASURED — post-strip det ✓ 10/10 at their real CUDA/BF16 serving posture; bench
039's det ✗ is REVERSED at this posture (the strip removed the last volatile field).**

Date: 2026-10-08 · host `4090-windows` (REFLEX_BENCH_HOST) · reflex `a81813b` (the
`usage.wall_ms` strip in `AgentJevLane::decide_raw` — the Issue-075 Drex law mirrored,
landed this window) · binary `target/release/harness` (release, default features — the
lane is ungated), one window, their service the only compute job.

Posture: THEIR `jev_service` (malevrigns/agent-jev @ `a965ca8f`) on loopback `:8149`,
`--checkpoint .raw/agent-jev-model/final.pt --model-path .raw/models/Qwen3-0.6B --device
cuda:0 --temperatures .raw/agent-jev-model/temperatures.json` (the bench-123 re-stage,
turnkey `boot_agentjev.cmd`) — served model advertised `AgentJev-0.6B@68883998`.
`--agentjev --skip-laya --nb-select --oc-select --ridge-select --suites typed_decisions`
(board-canonical selection flags). Slice integrity OK: test 400
(fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100
(fnv1a64-a1679de56e9278b1) — the same digests bench 123 measured.

## The cells

| lane · model | n | acc | ECE(maxp) | p50 | det |
|---|---|---|---|---|---|
| agentjev · AgentJev-0.6B@68883998 | 2000 | **0.7720** | 0.1087 | 119.0 ms | **✓ (n=10, byte-identical)** |
| modelless (control) | 2000 | 0.5725 | 0.0993 | 0.819 ms | ✓ |

- **det ✓ 10/10** — the first-10-cases observed-repeat pairs (20 requests) are
  byte-identical AFTER the strip: their disclosed bf16 wobble ("HTTP question isolation
  BF16 numerical difference 约 0.00282", their README) did NOT fire in any pair at this
  posture. Bench 039's det ✗ was over-determined: the `usage.wall_ms` timing tail alone
  forced it BY CONSTRUCTION; with it removed, the column reads clean.
- **acc 0.7720 == bench 123's cell exactly** (same weights `@68883998`, same split
  digests) — the det re-read changed no decision surface, only the compare input. ECE
  0.1087 and p50 119 ms are fresh reads of the same posture.
- Modelless control bit-identical to the published board (the standing drift gate).

## Provenance (the standing disclosure)

- START: `PROVENANCE: power=AC scheme=high load=2.4 swap=3569MB gpu=0 %, 638 MiB
  canary=skipped` — preflight PASSED (`preflight_start.log`).
- END: `PROVENANCE: power=AC scheme=high load=4.8 swap=3564MB gpu=1 %, 5706 MiB
  canary=skipped` — preflight PASSED (`preflight_end.log`); the in-process
  `box_state` stamps both capture points `latency_quotable: true`, refusals none.
- GPU exclusivity held: their service the only compute consumer (serialized-window law).

## What this re-verdicts — and what it does not

- REVERSED (this posture): bench 039's "det ✗ on every suite" — the observed-repeat
  artifact was their wall clock, not their forward. Their typed_decisions number is now
  QUOTABLE WITH ITS DET COLUMN CLEAN at the current serving posture.
- NOT claimed: bench 039's original weights (`@9d9b5fc3`) are gone (the 2026-10-05
  re-stage torch-wrapped fresh → `@68883998`); bench 123 established the two serve the
  same effective model (0.7715 vs 0.7720 = the one-question bf16 wobble class). The
  re-verdict therefore attaches to the CURRENT posture and the bench-039 addendum says
  exactly that.
- The bf16 wobble class is NOT refuted in general — it is UNFIRED in 20 observed
  requests here; their README's disclosure stands as their own measurement.
- One posture, one suite: typed_decisions (the row's owned data point, bench 039's own
  scope for the headline). Other suites inherit the code fix; their det columns re-read
  on the next full lane run — no separate claim made.

Raw: `results.json` + `TABLES.md` beside this file.
