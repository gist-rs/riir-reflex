# Bench 086 — the M3 lane fill: openthai on the remaining 13 suites (the M3 mirror of 084)

**Status:** MEASURED 2026-09-28 — the 13 openthai suites that had no
`@m3-max-metal` cells, one run, box_state `latency_quotable: true` BOTH
spans. Together with 085 (the 4 prior suites), the **openthai lane now has
all 17 suites measured and quotable on BOTH hosts** (m3-max-metal + 4090-win).

## PROVENANCE (the Issue-021 box-state law)

- Preflight gate passed immediately before launch (AC 5-min settle complete,
  load 3.39 < 6.0, canary 127.3 µs vs ref 141 µs — same gate state as 085;
  the 085 record's PROVENANCE block carries the full recipe).
- The harness's own `meta.box_state`: start load **2.52**, end load **0.34**,
  `latency_quotable: true` on BOTH spans, refusals `[]`.
- Server: same posture as 085 — `.raw/openthai-systemone` @ `5d04bcc`, MPS
  fp32 (no dtype env; upstream default), `OPENTHAI_SERVE_URL=127.0.0.1:8001`,
  `permutations=1` pin green ×13. Server + harness launched as ONE detached
  `caffeinate -is` command (the 085 lesson: a detached server does not
  survive terminal-session teardown); the server was killed by the same
  command at completion.
- Clean worktree build at `3a7be30` (re-created after the 085 cleanup —
  the dirty main tree still carries sibling Plan-426 WIP in the runner).
  git_sha in results.json = `00bddd1` (the 085 record commit; the harness
  binary is byte-identical — built from the same tree).
- Datasets `.raw/datasets` (the published pool). Suite digests match 084's
  4090 run (same questions, cross-host comparable).
- Wall: 22:52–23:27 (~35 min), banking77 the long pole (1650 s — the
  77-option prompt at ~3.1 s/question).

## Results (openthai · openthai-systemone · MPS fp32 · det ✓ ×13)

| suite | n_q | acc | M3 p50 | 4090 p50 (084) | ratio | acc Δ vs 4090 |
|---|---|---|---|---|---|---|
| typed_decisions | 2000 | 0.5345 | 497 ms | 79 ms | 6.3× | −0.2 pt |
| ag_news | 500 | 0.8900 | 113 ms | 56 ms | 2.0× | exact ✓ |
| emotion | 400 | 0.5900 | 78 ms | 50 ms | 1.6× | −0.5 pt |
| sst5 | 300 | 0.4300 | 203 ms | 59 ms | 3.4× | −0.3 pt |
| prompt_injections | 116 | 0.6293 | 101 ms | 50 ms | 2.0× | exact ✓ |
| banking77 | 500 | 0.6560 | **3056 ms** | 178 ms | **17.2×** | +0.2 pt |
| code_fixtures | — | 0.5938 | 171 ms | 86 ms | 2.0× | exact ✓ |
| harness_visibility | 16 | 0.4375 | 105 ms | 62 ms | 1.7× | exact ✓ |
| harness_permissions | 12 | 0.5833 | 105 ms | 55 ms | 1.9× | exact ✓ |
| harness_tool_fit | 12 | 0.9167 | 63 ms | 54 ms | 1.2× | exact ✓ |
| harness_routing | 16 | 0.4375 | 107 ms | 57 ms | 1.9× | exact ✓ |
| harness_sensitivity | 15 | 0.7333 | 110 ms | 52 ms | 2.1× | exact ✓ |
| harness_cache_reuse | 12 | 0.4167 | 107 ms | 53 ms | 2.0× | exact ✓ |

- **Accuracy: the 4090 board reproduces on the M3** — 9 of 13 suites
  byte-exact, the other 4 within ±0.5 pt (tiny-n suites + sentiment
  borderline rows). The lane measures their stack faithfully across boxes,
  again.
- **Latency: the option-count scaling law measured as a curve.** With 085's
  four cells the M3 ratio-to-4090 now spans the full option range:
  1.2–2.1× at 3–7 options (xnli/wisesight/sib200/harness rows) → 3.4× at
  sst5's 5 → 6.3× at typed's ~5–9 (2000 q of them) → 18× at massive's 59 →
  **17.2× at banking77's 77**. Cost scales with the inlined-option prompt,
  exactly as the bench-page note (reflex-site, same day) describes: their
  template reads every option as prompt text
  (`<|ts_opt_i|> option: description`, one forward over all 256 slots), so
  per-question cost ≈ f(options) on a compute-bound device. banking77 at
  3.06 s/question is the 77-way instance of massive's 1.71 s — same law,
  more slots.

## Reading

- The M3 host rows for openthai are now COMPLETE (17/17 quotable). The
  site's per-suite tables carry both hosts for every openthai cell; the
  option-scaling note explains the 1.2–18× spread a reader sees between
  `@m3-max-metal` and `@4090-win` rows.
- No promotion claim — comparison lane, report-only (the Bench 074 law).

## Determinism

`permutations=1` pin green on all 13 suite-runs (`determinism_ok: true`).
Modelless by-product cells reproduce the published M3 rows (the harness's
deterministic sampling law).

## Artifacts

- `086_openthai_m3_lane_fill/TABLES.md` + `results.json`

## Publish — recipe

`PUBLISH_BENCH_LANES=openthai ../reflex-site/scripts/republish_bench.sh
../reflex-site/data/bench.json
.benchmarks/086_openthai_m3_lane_fill/results.json` — the current
bench.json as PRIMARY (the Issue-034 lane-scoped-update shape; the 084/085
recipe). Cells land under `m3-max-metal`, joining 085's four.
