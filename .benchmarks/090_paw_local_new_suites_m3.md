# Bench 090 — PAW Posture B (local runtime) on the four new suites: the twins complete

**Status:** MEASURED 2026-09-29 on the M3 Max — the paw-LOCAL twin cells for
`prompt_injections` + `xnli_en` + `massive_intent_en` (088) and
`typed_decisions` (089), the last open lane task from the 088/089 handoff.
The 056 verdict (accuracy-neutral vs hosted, determinism-positive) repeats
on all four, now including the lane's first FIFTEEN-program local run.

## The rule-had-not-generalised finding (fixed at root cause, pre-cell)

The first run measured **zero cells**: every suite came back `ABSENT — no
committed spec`. Root cause: the 087 per-shape spec extension landed in the
HOSTED lane only — `paw::run_suite` resolves per-qid specs
(`{suite}.{qid}.txt` wins over suite-wide, partial coverage refused), but
`paw_local::run_suite` still called the suite-wide-only `paw::load_spec`.
The four new suites carry ONLY per-shape specs, so the local lane found
nothing. Exactly the never-generalised class this workspace keeps paying
for — repaired by extracting the shared resolver `paw::resolve_shapes`
(spec resolution + drift guard + partial-coverage refusal in ONE home,
both lanes consume it) and porting the local lane onto per-shape cache
keys (`key_for_qid` / `key` per shape) + per-question program dispatch.
New gates: `multi_shape_suite_dispatches_each_question_to_its_own_program`
(scripts per-question program ids over scripted channels, end to end) and
`multi_shape_missing_one_shape_refuses_partial_coverage`; 22/22 lane tests,
lib 219/0, clippy `-D` clean at `--lib` and `--all-targets`.

## Provenance

- **Box state: ⛔ latency NOT QUOTABLE — accuracy-only record** (the 049
  posture; the local p50s are their in-process round-trips, quoted for
  posture comparison beside the hosted server p50s, never as engine
  figures). Preflight-equivalent: harness box_state load 12.71 → 14.99
  (`latency_quotable: false` — sibling sessions on the box), AC, powermode
  high.
- **Code:** reflex at `85d452d` + the per-shape local-lane fix (this
  record's commit). `--release`, default features (modelless),
  `--skip-laya`, `REFLEX_BENCH_HOST=m3`.
- **PAW side:** their LOCAL llama.cpp runtime — `programasweights 0.4.10`
  (llama-cpp-python 0.3.19, device auto) in a fresh M3 venv
  (`.raw/paw-env`; the sccache launcher trap — sccache 0.13.0 fails the
  llama-cpp-python build with "failed to zip up compiler outputs", mask it
  from PATH — is documented in AGENTS.md). `PAW_COMPILER=paw-ft-bs48-20260530`
  (the board tier — MANDATORY here: the new suites have BOTH tiers cached,
  and the unset auto-select refuses the ambiguity by design). The local
  lane never compiles: all 24 ft program bundles pre-warmed from the hosted
  cache (first-use downloads 6.5–765s per bundle, bimodal; ~2 h total warm).
- **Runs:** TWO full runs (run 1 `.benchmarks/090_paw_local_new_suites_m3/`,
  rerun byte-merged as `results_det_rerun.json` beside it). Wall ≈ 6.5 min
  per run for all four suites (2716 questions) — vs the hosted 089's
  ~81 min for typed alone.

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| suite (n) | paw-LOCAL acc | refusals | p50 (local) | hosted-ft SAME LAW (088/089) | modelless (same run) | det ×2 |
|---|---|---|---|---|---|---|
| prompt_injections (116) | **0.6379** | 0 (0.0%) | 32.9 ms | 0.6379 | 0.4828 | ✓✓ |
| xnli_en (300) | **0.7133** | 0 (0.0%) | 50.4 ms | 0.7200 | 0.3400 | ✓✓ |
| massive_intent_en (300) | **0.5133** | 118 (39.3%) | 43.8 ms | 0.5100 (119 refused) | 0.3533 | ✓✓ |
| typed_decisions (2000) | **0.5955** | 2 (0.1%) | 79.5 ms | 0.5925 (2 refused) | 0.3345 | ✓✓ |

typed score MAE (answered) **0.6387** · within-1 **0.7963** (hosted 089:
0.6485 / 0.7937) — the score axis beats the modelless lane (0.7046 /
0.7275) on the local posture too. The 2 refusals are the same two failure
classes as 089 (`data` truncated level copy, `observe` unpresented pick).

## Reads

1. **The 056 verdict repeats on the new suites: accuracy-neutral,
   determinism-positive.** Local-vs-hosted deltas are −0.7 to +0.3 pt,
   refusal counts within 1 on every suite — and BOTH full runs are
   byte-identical on every pick/acc/refusal/det field (only the latency
   wall moves, 79.5→93.3 ms p50 on typed under sibling load). The runtime
   posture is not a second measurement of the task; it is the same
   measurement with the network removed and the determinism restored.
2. **The local runtime is also ~2 orders faster end-to-end than the hosted
   round-trip at this box** (33–80 ms local p50 vs 089's 96.6 ms hosted
   server p50 — their network/TLS on top) — and 12× faster wall for the
   full typed suite (356 s vs 4886 s). Posture comparison only, per the
   law: never an engine figure.
3. **The 087 lesson generalised in code this time** — the per-shape
   resolution now lives in ONE function (`paw::resolve_shapes`) consumed
   by BOTH lanes; the next lane to carry the PAW shape law inherits it by
   import, not by copy. The two new gates pin the dispatch law (per-question
   program ids over scripted channels) and the partial-coverage refusal.
4. **Modelless drift pins byte-identical** to the published 088/089 rows on
   all four suites (0.4828 / 0.3400 / 0.3533 / 0.3345) — same pull, same
   protocol, the control column is comparable.

## Board (posture-complete)

| suite | paw-ft hosted | paw-ft LOCAL (this bench) | record |
|---|---|---|---|
| ag_news / banking77 / emotion / sst5 | 055/054 | 056 | 054–058 |
| code_fixtures | 0.6250 (087) | — (not twinned; single-box hosted cell) | 087 |
| prompt_injections | 0.6379 (088) | **0.6379** | 088, 090 |
| xnli_en | 0.7200 (088) | **0.7133** | 088, 090 |
| massive_intent_en | 0.5100 (088) | **0.5133** | 088, 090 |
| typed_decisions | 0.5925 (089) | **0.5955** | 089, 090 |

`harness_families` stays PAW-less (board symmetry).

## Artifacts

- `090_paw_local_new_suites_m3/TABLES.md` + `results.json` (canonical) +
  `results_det_rerun.json` (the byte-compare rerun).
