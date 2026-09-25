# Bench 036 — Issue 029: the GLiNER comparison lane's first cells — beats the laya BASE checkpoints on 9/15, does not touch the `typed` specialist

One full 15-suite run on 2026-09-25, host `4090-windows` (REFLEX_BENCH_HOST),
reflex `d69f0c7` + substrate `1afd4f8` (the SAME sha the published 4090 laya
lanes were measured at — the lanes are comparable by construction), binary
`/tmp/gliner-lane-target/release/harness` (`--features modelless,laya-riir-cuda`,
`--gliner`). Lanes in this run: modelless · laya-riir (cuda, fresh pass — the
bench-032 lanes' substrate + the new GPU pre-ramp) · gliner (cuda, fp32, their
gliner2 package via `scripts/gliner_lane.py`, latency = subprocess round-trip
IPC included). The bench-034 clm cells are NOT in this run (their server was
down); they carried over in the publish merge untouched.

**Box state** (the standing disclosure): GPU 518 MiB used / 24 564 MiB at run
start (desktop only, no sibling compute — the serialized-window law), gliner
oracle resident ~2.3 GiB fp32 beside the laya lane's allocation (total well
under the card; no contention signature — no stall on any suite). No
power-source axis on this box (AC desktop); run stamped in results.json
(`meta.box_state`).

## The verdict table (accuracy; gliner vs the same run's laya lanes)

| suite | modelless | laya·en | laya·typed* | gliner | verdict |
|---|---|---|---|---|---|
| typed_decisions | 0.3190 | 0.3575 | **0.7445** | 0.5280 | +17pt over base; **−21.7pt under the specialist** |
| ag_news | 0.5100 | **0.9500** | — | 0.7025 | laya win (−24.8pt) |
| emotion | 0.2825 | **0.5925** | — | 0.5650 | laya win (−2.8pt, near) |
| sst5 | 0.2167 | 0.3717 | — | **0.4383** | gliner win (+6.7pt) |
| prompt_injections | 0.4397 | **0.6983** | — | 0.6810 | laya win (−1.7pt, near) |
| xnli_en | 0.3467 | **0.8600** | — | 0.4767 | laya win (−38.3pt) |
| massive_intent_en | 0.6900 | 0.7500 | — | **0.8233** | gliner win (+7.3pt) |
| banking77 | 0.4460 | 0.4980 | — | **0.7060** | gliner win (**+20.8pt**) |
| code_fixtures | 0.2500 | 0.5833 | — | **0.6667** | gliner win (+8.3pt) |
| harness_visibility | 0.3750 | 0.3125 | — | **0.5000** | gliner win |
| harness_permissions | 0.4167 | 0.4167 | — | **0.5833** | gliner win |
| harness_tool_fit | 0.5000 | 0.5000 | — | **0.7500** | gliner win |
| harness_routing | 0.4375 | 0.6250 | — | 0.6250 | tie |
| harness_sensitivity | 0.4000 | 0.6000 | — | **0.8000** | gliner win |
| harness_cache_reuse | — (LLM-only) | 0.5000 | — | 0.5000 | tie |

\* `typed` + `multilingual` checkpoints run only on typed_decisions
(the registry law); typed 0.7445 / multilingual 0.3490.

**9 gliner wins / 4 laya wins / 2 ties** against the BASE checkpoint —
consistent with fastino's own fast-decisions claim (60.2% vs "Laya Router"
46.6%) in DIRECTION on the decision-style suites, and honestly REFUTED on
classic NLU (ag_news −24.8pt, xnli −38.3pt: their card's "not a
general-purpose model" is the right frame). The typed_decisions headline
stays laya's (the `typed` specialist at 0.7445 is the accuracy bar; gliner
0.528; our measured AgentJev landscape row 0.7925 published).

## Latency (p50 ms; same run, same box)

| shape | laya-riir cuda (in-process) | gliner (subprocess IPC in) |
|---|---|---|
| typed_decisions (long ctx, 3 ckpts) | 107/59/108 | **30** |
| short suites (ag_news…sensitivity) | 10–15 | 22–23 |
| banking77 / code_fixtures | 24 / 27 | 24 / 32 |

gliner is 3.5× FASTER on the long-context headline, ~2× slower on short
inputs — and its number includes the subprocess IPC (the laya-python
measurement law; the honest cross-lane comparison for IPC-inclusive numbers
is the python oracle, not the in-process rust lane).

## Integrity

- gliner determinism (observed-repeat, first 10 cases/suite): ✓ TRUE on all
  15 suites (their fp32 scorer repeats byte-identically on this box).
- Modelless cross-host drift gate: 13/13 suite accuracies bit-identical to
  the published m3 cells; `code_fixtures` the designed exclusion
  (commit-relative population — the publisher's own carve-out).
- The laya det=False cells on typed_decisions (english/typed) are the
  pre-existing variance class (bench 029's record: "wobbles 1 case within
  its pre-existing det variance"), not new.
- Publish: lane-update merge (Issue 023 T5 law) — 4090-windows's
  modelless + laya + gliner refreshed, bench-034 clm carried over,
  `lane_sources` disclose the split.

Raw: `results.json` beside this file (the tables above quote it).
