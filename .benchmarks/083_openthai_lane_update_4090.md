# Bench 083 — the OpenThai lane-update: all 17 suites filled on the 4090 (the Issue-025 lane-update shape)

**Status:** MEASURED 2026-09-28 — complete on this box (15 default suites + the 2 Thai
probe suites, det green ×17). **NOT yet published to reflex-site** — the publish is the
next subtask (steps recorded in §Publish, deliberately not executed this session:
concurrent-agent handoff, the reflex-site `data/bench.json` write is another lane's
collision surface).

## Why this run exists

reflex.gist.rs/bench showed `openthai — not run` on every suite except the four Bench
074 carried (thai_wisesight, thai_sib200, xnli_en, massive_intent_en — M3 host row).
Cause: the lane is opt-in (`--openthai`, default off) and every published run since
Bench 074 left it off — the exact Issue-025 class (`laya (python) — not run`; no issue
owned that one either). Fix shape is Issue-025's: one run with the lane on, published
as an ordered lane-update.

## PROVENANCE (box state — the Issue-021 law)

- **Box:** 4090 Windows workstation (`shikuwa`), RTX 4090, driver 610.62, torch
  2.6.0+cu124, transformers 5.17.0, python 3.10 venv at
  `.raw/openthai-systemone/.venv`, weights HF `iapp/OpenThai-SystemOne` cached under
  `.raw/hf` (~2.7 GB VRAM resident).
- **Server:** `iapp-technology/openthai-systemone` @ `5d04bcca0c58bd10e7dac2d3d369d8f760bea6cf`
  (Bench 074's pin; the repo advertises no branch refs — clone defaults to the model-card
  HEAD `11ffd38`, docs-only delta vs the pin), FastAPI `uvicorn
  openthai_systemone.server:app` on 127.0.0.1:8000, `permutations=1` pinned.
- **⛔ Device/dtype DIVERGENCE from Bench 074, pinned deliberately:** their client
  defaults CUDA→**bfloat16**; Bench 074 measured MPS **fp32**. A 4-line disclosed patch
  in `.raw/openthai-systemone/openthai_systemone/client.py` (marked with the
  `katgpt-rs workspace patch` comment) adds `OPENTHAI_SYSTEMONE_DTYPE=float32` — this
  run is **CUDA fp32**, the board's numerics. Unset env = upstream behavior.
- **Cross-check vs the Bench 074 M3 board (fp32→fp32, different device):**
  massive_intent_en **0.9200 == 0.9200 exact**, thai_sib200 **0.8382 == 0.8382 exact**,
  xnli_en 0.9000 vs 0.8967 (+0.3 pt), thai_wisesight 0.4675 vs 0.4750 (−0.8 pt).
  The dtype pin held; residual delta is box-level, sub-pt, disclosed.
- **GPU exclusivity:** only GUI processes (C+G: zed, Docker Desktop frontend, tray apps)
  during the run — no compute consumer. Latency cells are quotable at this state; the
  Windows harness has no preflight probes, so the publisher stamps these cells
  UNJUDGED (its loud-note law) — the record here is the box-state disclosure.
- **Host label:** `REFLEX_BENCH_HOST` was NOT set; the runner recorded the raw box name
  `shikuwa`. The publisher's `HOST_DISPLAY` aliases `shikuwa → 4090-win` (the recorded
  Issue-033 PAW precedent — same physical box the fleet publishes as 4090-win). Next
  runner on this box should set `REFLEX_BENCH_HOST=4090-windows` (the AGENTS.md law).
- Harness binary at `bee92d3` (stale target/ build): `src/lanes/openthai.rs` and
  `src/harness/runner.rs` are byte-identical to develop `b5cf4b0` in that range
  (verified `git log bee92d3..HEAD` on both paths = empty), so the lane wire and the
  determinism check are current-code.

## Results (openthai · openthai-systemone · CUDA fp32 · permutations=1 · det ✓ all rows)

| suite | n | acc | macro F1 | ECE(maxp) | p50 | M3 (074) |
|---|---|---|---|---|---|---|
| typed_decisions | 2000 | 0.5360 | 0.5392 | 0.2275 | 79 ms | — |
| ag_news | 500 | 0.8900 | 0.8826 | 0.0474 | 56 ms | — |
| emotion | 400 | 0.5950 | 0.4877 | 0.2124 | 50 ms | — |
| sst5 | 300 | 0.4333 | 0.4144 | 0.1738 | 59 ms | — |
| prompt_injections | 116 | 0.6293 | 0.5821 | 0.2118 | 50 ms | — |
| banking77 | 500 | 0.6540 | 0.6331 | 0.0895 | 178 ms | — |
| code_fixtures | — | 0.5938 | 0.3146 | 0.1933 | 86 ms | — |
| xnli_en | 300 | 0.9000 | 0.9000 | 0.0453 | 52 ms | 0.8967 |
| massive_intent_en | 300 | **0.9200** | 0.9142 | 0.0444 | 95 ms | **0.9200** |
| thai_wisesight | 400 | 0.4675 | — | 0.3682 | 57 ms | 0.4750 |
| thai_sib200 | 204 | **0.8382** | — | 0.0651 | 60 ms | **0.8382** |
| harness_visibility | 16 | 0.4375 | 0.4249 | 0.2194 | 62 ms | — |
| harness_permissions | 12 | 0.5833 | 0.5524 | 0.3147 | 55 ms | — |
| harness_tool_fit | 12 | 0.9167 | 0.9111 | 0.1963 | 54 ms | — |
| harness_routing | 16 | 0.4375 | 0.3843 | 0.4883 | 57 ms | — |
| harness_sensitivity | 15 | 0.7333 | 0.6548 | 0.1926 | 52 ms | — |
| harness_cache_reuse | 12 | 0.5000 | 0.3333 | 0.4058 | 53 ms | — |

Reading, honestly scoped: openthai leads the EN topical pair (ag_news 0.890 /
massive 0.920) and xnli (0.900); mid on banking77 (0.654 — the 77-way slot head under
a 500-question stratified slice; the M3 board never carried this cell) and typed
(0.536 vs the served Instinct A1 0.630/0.6475 — not comparable postures, the lane is
report-only); weak on sst5/emotion fine-grained sentiment. Family rows are tiny-n
sanity cells, never a leaderboard. **No promotion claim — comparison lane
(report-only), the Bench 074 law.** The Instinct/Plan-426 relevance: the 0.9200
massive teacher datum reproduces on a second box+device (fp32 pin), which is the V2
teacher-qualification bar in riir-train Plan 426.

## Determinism

The lane pin (`permutations=1`, `decide_raw` ×2 byte-compare) green on all 17
suite-runs (`determinism_ok: true` in both results.json). The modelless by-product
control in these docs is a DIFFERENT posture than published (stale runner knobs) —
moot: the publish filters it (next section).

## Artifacts

- `083_openthai_lane_update_4090/TABLES.md` + `results.json` — 15 default suites
  (main run ~14 min wall, ended 20:59 +0700)
- `083_openthai_lane_update_4090/thai/TABLES.md` + `results.json` — the 2 Thai probe
  suites (39 s, ended 21:02 +0700)

## Publish — the next subtask (NOT executed; recorded for the handoff)

reflex-site is 1 commit ahead of this box's last known state and its
`data/bench.json` is a concurrent-agent collision surface — this session stops before
that write. The steps, in order:

1. `git -C ../reflex-site pull` (sync first — the repo moves).
2. `py -3.10 ../reflex-site/scripts/publish_bench.py ../reflex-site/data/bench.json
   .benchmarks/083_openthai_lane_update_4090/results.json
   .benchmarks/083_openthai_lane_update_4090/thai/results.json ../reflex-site`
   with `PUBLISH_BENCH_LANES="openthai"` (extras only — drops our stale modelless
   control cells loudly, the Issue-033 law; primary is the existing bench.json, the
   Issue-034 safe default) — verify the shikuwa→4090-win alias landed the rows under
   the existing host container.
3. `py -3.10 ../reflex-site/scripts/test_publish_bench.py` (52/52 at this writing) +
   the bench-page smoke.
4. Data commit + push (deploy is the manual/M3-side wrangler handoff — no CF creds on
   this box, the 018 T6 precedent).

Server lifecycle: the uvicorn process was STOPPED after the runs (unattended GPU
processes are the exclusivity hazard for sibling gate runs). Restart for any
re-verification: weights cached, ~1 min — see §PROVENANCE for the exact env
(`OPENTHAI_SYSTEMONE_DTYPE=float32` is REQUIRED to reproduce these numbers).
