# Bench 089 — PAW on `typed_decisions`: the 15-program suite, board complete

**Status:** MEASURED 2026-09-29 on the M3 Max — the PAW comparison lane's
last missing dataset suite, and the lane's first FIFTEEN-program run: one
program per (qid) shape across the 5 workflows (4 Choice / 5 Score / 6
Noul), all compiled on the board tier.

## Provenance

- **Box state: ⛔ latency NOT QUOTABLE — accuracy-only record** (the 049
  posture). Preflight REFUSED earlier in the session (load 17.84); harness
  box_state load 12.56 → 13.30, AC, powermode high. Server p50 is their
  side's figure.
- **Code:** reflex at `9dc33da` (specs frozen before any cell).
  `--release`, default features (modelless), `--skip-laya`,
  `REFLEX_BENCH_HOST=m3`.
- **PAW side:** hosted-anonymous, compiler **`paw-ft-bs48-20260530`** via
  `PAW_COMPILE_ASYNC=1`, temperature 0, max_tokens 48. **15 programs**,
  compile wall **2645.5 s** total (126–264 s each; the full list is in
  `089_paw_ft_typed_m3/TABLES.md`'s PAW lane line). Programs are PUBLIC on
  their hub (anonymous tier). Run wall 4886 s (~81 min) for 2000 questions.
- **Determinism ✓** (observed-repeat).
- **Modelless drift pin:** 0.3345 byte-identical to the published 084/086
  rows (2000 q).

## Cells

| lane · model | n | acc | refusals | det | score MAE (answered) | within-1 | server p50 |
|---|---|---|---|---|---|---|---|
| paw · paw-ft-bs48-20260530 | 2000 | **0.5925** | **2/2000 (0.1%)** | ✓ | **0.6485** | **0.7937** | 96.6 ms |
| modelless | 2000 | 0.3345 | — (abstain lane) | ✓ | 0.7046 | 0.7275 | 0.759 ms (086) |
| openthai · openthai-systemone | 2000 | 0.5345 (086) / 0.5360 (084) | — | ✓ | — | — | 497 / 79 ms |

**Board placement: second among the comparison lanes, behind agentjev.**
agentjev 0.7715 (038, 4090) > laya·typed specialist 0.7445 (037/081, the
specialist) > **paw-ft 0.5925** > openthai 0.5345 > gliner 0.528 (038,
4090) > laya·en base 0.3575 > modelless 0.3345.

- **The verbatim-copy score levels held.** The 5 Score qids require the
  program to emit the exact level line (e.g. `High: access to production,
  secrets or customer data.`) — the parser's exact-match law with zero
  fuzzy matching. 800 score questions produced essentially clean answers:
  score MAE 0.6485 and within-1 0.7937 BOTH beat the modelless lane's
  0.7046 / 0.7275. The 2 refusals are exactly the two failure classes:
  `data` (a truncated level copy — honest refusal, not fuzzy-matched into
  a level) and `observe` (a real `action` token outside that row's
  presented 4 — the same unpresented-pick class as massive 088, on a
  suite whose `action` union is 9 tokens across two workflows).
- **No probability surface** (disclosed divergence — no ECE/abstention
  columns for PAW).
- **No per-shape breakdown in the lane row** (the modelless lane's
  choice/noul/score extras have no PAW counterpart yet — instrumentation
  follow-up if the split is ever needed; the aggregate and score MAE
  stand without it).

## The board, complete

| suite | paw-ft | best other lane (record) | paw record |
|---|---|---|---|
| ag_news | 0.7900 | laya·en 0.9500 (055) | 054–058 |
| banking77 | 0.4200 (answered-acc 0.6416; 34.6% refusals) | openthai 0.6540 (084) | 058 |
| emotion | 0.5000 | laya·en 0.7375 (055) | 054–058 |
| sst5 | 0.3950 | openthai 0.4333 (084) | 058 |
| prompt_injections | 0.6379 | laya 0.6983 (037/039) | 088 |
| xnli_en | 0.7200 | openthai 0.9000 (084; laya 0.8600) | 088 |
| massive_intent_en | 0.5100 | openthai 0.9200 (084; gliner 0.8233) | 088 |
| typed_decisions | 0.5925 | agentjev 0.7715 (039) | 089 |
| code_fixtures | 0.6250 | gliner 0.6667 (037) | 087 |
| harness_families | — deliberately not run (board symmetry) | — | — |

(banking77 is the ft tier's one weak board row — the 77-way label set
with the never-guess law refusing a third of the questions; the 054
finding survives in the ANSWERED accuracy, 0.6416 vs openthai's 0.6540,
but the board cell is the hard number and openthai leads it.)

## What remains

- paw-local (Posture B) twins for the new suites — 4090 (the venv lives
  there), the 056 posture.
- The per-shape breakdown instrumentation (above) — only if ever needed.

## Artifacts

- `089_paw_ft_typed_m3/TABLES.md` + `results.json`
