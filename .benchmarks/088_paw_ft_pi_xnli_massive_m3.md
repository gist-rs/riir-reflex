# Bench 088 — PAW on `prompt_injections` + `xnli_en` + `massive_intent_en`: the single-shape board completion, both compiler tiers

**Status:** MEASURED 2026-09-29 on the M3 Max — the PAW comparison lane's
remaining single-shape dataset suites, finishing the dataset board (all 8
suites + code_fixtures now carry committed specs and cells; only the
`harness_families` synthetic probes stay PAW-less, deliberately — board
symmetry: no comparison lane runs them).

## The tier incident (documented, both runs kept)

The first run **forgot `PAW_COMPILER`**, and the anonymous default served
**`paw-4b-qwen3-0.6b-20260407`** — the BASE tier, 049's posture, not the
board tier. 054's law is that the comparable-to-board tier is the finetune
compiler via the async endpoint. The run was re-executed under the board
tier and **both runs are kept**: ft = the board cell (canonical,
`088_paw_ft_pi_xnli_massive_m3/`), base = the second tier beside it
(`088_paw_pi_xnli_massive_m3/`) — the 054 "both compiler tiers" pattern
(its banking77 finding, now on three more suites). The programs cache by
`(suite, qid, compiler, BLAKE3(spec))`, so the two tiers never collide.

## Provenance

- **Box state: ⛔ latency NOT QUOTABLE — accuracy-only record** (the 049
  posture: hosted-PAW latency is network-dominated and is never engine
  latency). Preflight REFUSED (load 17.84 > 6 — sibling sessions); harness
  box_state load 12.56 → 13.30, `latency_quotable: false`, AC, powermode
  high. Cite `server p50` for their stack's own latency; client round-trip
  columns are observation only.
- **Code:** reflex at `9dc33da` (the commit that froze all 18 specs BEFORE
  any cell — the 049 discipline, this time in a prior commit). `--release`,
  default features (modelless), `--skip-laya`. `REFLEX_BENCH_HOST=m3`.
- **PAW side:** hosted API, anonymous posture (no key; programs PUBLIC on
  their hub), temperature 0, max_tokens 48. Board tier: compiler
  **`paw-ft-bs48-20260530`**, async compile (`PAW_COMPILE_ASYNC=1`),
  compiles 77.6 + 121.0 + 88.6 = **287.2 s**; programs pi
  `d9cfba100af1e46f97cd`, xnli `1949b801cf9746028d56`, massive
  `1b5d4b654f60b508602c`. Base tier: **`paw-4b-qwen3-0.6b-20260407`**
  compiles 4.2–4.7 s each, programs `d60abfa3d58b0ceeace3` /
  `ff4314018c08b8378fc0` / `b1ffa7cee92660869ff9`.
- **Determinism ✓ every cell, both tiers** (observed-repeat, first 10
  questions) — the ft tier stays deterministic (058/087's reading,
  reproduced on three more suites).
- **Modelless drift pins byte-identical to the published 084/086 rows:**
  prompt_injections 0.4828, xnli_en 0.3400, massive_intent_en 0.3533.

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| suite | n | paw-ft-bs48 | ft refusals | paw-4b (base) | base refusals | modelless | best other lane (record) |
|---|---|---|---|---|---|---|---|
| prompt_injections | 116 | 0.6379 | 0/116 | **0.6983** | 0/116 | 0.4828 | laya 0.6983 (037/039) |
| xnli_en | 300 | **0.7200** | 0/300 | 0.5833 | 0/300 | 0.3400 | openthai 0.9000 (084) |
| massive_intent_en | 300 | **0.5100** | 119/300 | 0.0967 | 238/300 | 0.3533 | openthai 0.9200 (084) |

(server p50: ft 76.6 / 96.0 / 91.6 ms; base tier comparable. The ft tier
wins xnli by +13.7 pts and massive by +41.3 pts over base; base wins
prompt_injections by +6.0 pts — n=116, one suite, noted not explained.
⚠ The 037/039 comparison-lane cells were measured on the OLD t20k pool
(their same-table modelless pi row reads 0.4397 vs today's canonical-pool
0.4828) — cross-pool cells are cited for board context, never diffed
numerically.)

## The massive structural note (read before quoting the refusal column)

`massive_intent_en` presents **20 options sampled per row** from the 59-intent
frozen universe; a PAW program is a fixed-output-set function and answers over
the FULL universe (the spec names all 59 verbatim — the guard requires it).
Scoring is exact-gold against the row's presented 20, so accuracy is fair
(correct iff the pick is the gold intent); but a WRONG pick that lands outside
that row's presented 20 is an honest REFUSAL under the never-guess law — the
refusal column over-counts by construction. Measured exactly as predicted:
ft 119/300 (39.7%), base 238/300 (79.3%). Refusal samples show both classes:
`qa_factoid` (a real intent, just unpresented in that row) and
`iot_hue_sound` / `screen_query` (plausible-but-nonexistent tokens — honest
refusals). The handicap vs option-aware lanes is presentation, not
classification; openthai's 0.9200 reads the presented options directly.

## Board placements (every comparison-lane cell on record, cited)

- **prompt_injections: PAW lands mid-board** — laya 0.6983 ≈ paw-BASE
  0.6983 (an exact tie on n=116) > gliner 0.6810 > **paw-ft 0.6379** >
  openthai 0.6293 > modelless 0.4828 (037/039/084 cells). The finding is
  the TIER FLIP, not the lead: the base compiler wins this suite by +6.0
  pts over the board tier — the only suite of the nine where base > ft.
- **xnli_en: ft third** — openthai 0.9000 > laya 0.8600 (037) >
  **paw-ft 0.7200** > paw-base 0.5833 > gliner 0.4767 > modelless 0.3400.
  The 3-way NLI shape transfers to the ft compiler cleanly (zero
  refusals: the label set is tiny and the spec's Label-meanings block did
  its job).
- **massive_intent_en: ft fourth** — openthai 0.9200 > gliner 0.8233 >
  laya 0.7500 (037) > **paw-ft 0.5100** > modelless 0.3533 > paw-base
  0.0967. The sampled-presentation handicap above + the 59-way long tail
  (29 of 59 intents have NO train docs even in the modelless lane's
  corpus) — the option-aware lanes read the presented 20 directly, the
  PAW program must know the answer outright.

## What remains

- Nothing on this bench's scope. The dataset board is complete:
  ag_news / banking77 / emotion / sst5 (049/054–058), code_fixtures (087),
  prompt_injections / xnli_en / massive_intent_en (088), typed_decisions
  (089). The `harness_families` probes stay PAW-less by board symmetry.
- paw-local (Posture B) for the new suites: the programasweights venv lives
  on the 4090; the local-vs-hosted twin ride belongs there (the 056
  posture), not on this box.

## Artifacts

- `088_paw_ft_pi_xnli_massive_m3/TABLES.md` + `results.json` (ft, canonical)
- `088_paw_pi_xnli_massive_m3/TABLES.md` + `results.json` (base tier)
