# Bench 117 — paw hosted tail, re-attempt #2: their hub's inference backend still down; the health-probe methodology corrected

**Status: MEASURED (blocked externally) — the 065-T2 hosted tail re-attempt executed in a clean quiet
window (preflight PASS, load 3.89, canary 119.8 µs) and the hub STILL refused every inference: warmup
`POST /api/v1/infer` → HTTP 502 `inference_failed` through all 6 retries on the first suite. The outage
that started ~00:10 +0700 (bench 116) was still live at ~01:28 +0700 (bench 117's attempt). No hosted
cell landed; the one doc this attempt produced (massive_intent_en) carries quotable box state at both
ends and a paw ABSENCE — the failure is 100% third-party.**

Date: 2026-10-04 · host m3-max-metal · reflex `652ddae`

## The probe lesson (this record's durable finding)

The session's first health check was WRONG and would have wasted a second suite run had the harness
not retried on its own:

- `POST /api/v1/infer` with an **invalid body** (`{}`) returned **422 validation_error** — the routing
  and validation layers were UP, which read as "healthy" and launched the run.
- The **real** request (cached program, the harness's own warmup input, `temperature 0.0`,
  `max_tokens 48`) returned **502 `inference_failed`** — the inference backend behind the validation
  layer was down the whole time.

A validation-layer answer is not an inference-backend answer. The faithful probe is now a script —
**`scripts/paw_hub_probe.sh`** — which sends the harness's exact warmup request against the cached
massive_intent_en program and exits 0 only on a 200 with an `output` field. The re-run protocol for
whichever session catches the hub healthy is now: **probe first, launch only on exit 0.**

## The attempt (chronological)

| step | reading |
|---|---|
| box watch | sibling `cargo-refine --fix` on katgpt-rs held load 6.9–7.5 for ~10 min; waited for 2 consecutive reads < 5.5 |
| preflight (launch) | `PROVENANCE: power=AC Power load=3.89 swap=29451.75M canary=119.8us/best5 powermode=2(high)` — **PASS** |
| doc box state | start load 4.04 → end load 3.04, both `latency_quotable: true` — the window was clean end to end |
| compile | cache-hit (`compile 88.6s, CACHED — not recompiled`) — no compile traffic, inference-only attempt |
| warmup infer | 502 `inference_failed` ×6 retries (2s→30s backoff) → the harness recorded the absence and closed the doc |
| direct probe after | real-request probe: 502 `inference_failed` (the validation 422 alongside it — layers split) |

## Docs

- `per_suite/massive_intent_en/` — the failed-attempt doc (modelless rows complete; `paw` absent —
  the bench-115 pattern: failed attempts sit beside the good ones). Retained as the outage's
  second-window witness.

## What remains (unchanged from bench 116)

The 6 hosted suites (emotion, ag_news, sst5, massive_intent_en, banking77, typed_decisions) at
`PAW_COMPILER=paw-ft-bs48-20260530 PAW_COMPILE_ASYNC=1`, `--skip-laya --paw --nb-select --oc-select
--ridge-select`, per-suite docs under this directory — **gated on `scripts/paw_hub_probe.sh` exit 0**,
then the T4 publish (`PUBLISH_BENCH_LANES="paw"` update path) + smoke re-pin. typed_decisions is the
long doc (~35–40 min, 2000 q × ~1.1 s wall; its 15 per-qid programs are all cached — bench 089's
81 min included compiles).
