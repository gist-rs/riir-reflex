# 152 — Plan 619 T3.2: clef-text-0.6b LOCAL ANE arm on the 8+1-suite board (full-N frozen splits)

**Status:** MEASURED 2026-10-17 — 9 suites, full-N frozen splits, zero absences, determinism ✓ on every suite (`clef-local` arm, plan 619 T3.1; run `CLEF_LOCAL=1 ./target/release/harness --clef-local --skip-laya --out .benchmarks/152_clef_local_board/tables`, release build, log `/tmp/clef_board_152.log`).

## The rows (clef · clef-text-0.6b, LOCAL in-process ANE, M3 Max, artifacts `3c61a609…`)

| suite | n | clef acc | clef ECE | macro F1 | p50 | p99 | det | modelless acc (the floor) |
|---|---|---|---|---|---|---|---|---|
| typed_decisions | 2000 | **0.4545** | 0.1224 | 0.4475 | 244 ms | 749 ms | ✓ | 0.2940 |
| ag_news | 400 | **0.8975** | 0.0361 | 0.8887 | 36 ms | 90 ms | ✓ | 0.4050 |
| emotion | 400 | **0.6375** | 0.0972 | 0.5267 | 36 ms | 41 ms | ✓ | 0.2750 |
| sst5 | 600 | **0.4800** | 0.0403 | 0.4203 | 35 ms | 37 ms | ✓ | 0.2017 |
| prompt_injections | 116 | **0.4828** | 0.4093 | 0.3256 | 35 ms | 90 ms | ✓ | 0.4828 |
| xnli_en | 300 | **0.7400** | 0.0920 | 0.7381 | 35 ms | 36 ms | ✓ | 0.3400 |
| massive_intent_en | 300 | **0.7700** | 0.0858 | 0.7629 | 87 ms | 245 ms | ✓ | 0.6100 |
| banking77 | 500 | **0.8540** | 0.0706 | 0.8481 | 749 ms | 755 ms | ✓ | 0.3940 |
| semantic_defects | 102 | **0.5000** | 0.1475 | 0.5070 | 87 ms | 95 ms | ✓ | 0.1863 |

Honesty caveats carried verbatim from the plan (T3.2): **student ≠ teacher** — this is the 0.6b student of clef-flash (9B), not the teacher's quality; the ARC-C-class disclosure stands (the model's broad capability claims are the publisher's, our rows are OUR splits only). Comparison-lane law: never a product row; board membership requires the owner's full frozen suite protocol.

## Beside the board (instinct AGENTS board, different posture where noted)

| suite | clef-local (this bench) | board served | board best-known (any posture) |
|---|---|---|---|
| ag_news | 0.8975 | H2 0.8975 | Rethink ENC 0.9475 |
| banking77 | 0.8540 | H2 0.8540 | H2 0.8540 (T2 uncertified) |
| massive_intent_en | 0.7700 | H2 0.8400 | H2 0.8400 |
| typed_decisions | 0.4545 | H2 0.6475 | Rethink v2 head 0.7550 |
| xnli_en | 0.7400 | A0 0.5233 | Rethink ENC 0.8600 |
| emotion | 0.6375 | A0 0.8850 | A0 0.8850 |
| sst5 | 0.4800 | A1 0.4217 | Rethink ENC 0.5267 |
| prompt_injections | 0.4828 | A1 0.8534 | A1 0.8534 |

- The clef student **beats the modelless floor on 8 of 9 suites** (ties prompt_injections exactly — a 116-case suite where both land 0.4828) and posts **xnli 0.7400 / emotion 0.6375 / sst5 0.4800 / banking77 0.8540** — several above the board's SERVED rows (xnli +21.7pt over A0, emotion below, banking77 equal, sst5 +5.8pt over A1). It does NOT beat the board's best hybrid/encoder rows on typed_decisions, massive, ag_news(=), prompt_injections.
- ⚠ The EXACT 4-dp matches with the board's served rows (ag_news 0.8975, banking77 0.8540) are SURPRISING — different lanes, different n, computed independently. Quoted as measured; flagged for a second read before any T4.1 seating decision (the arena seat law requires the T2 strict-superiority gate on the frozen read anyway).
- Latency: short-record suites sit at **35-36 ms p50** (L256), massive 87 ms (L512), banking77 749 ms (77-way choice schemas → L1024/L2048 records — the honest per-bucket decode cost, bench 884's table), typed 244 ms p50 (multi-question packs). Determinism ✓ everywhere (the bit-exact parity makes this expected).
- Box state: quiet window (the run's own latencies are the disclosure; the T2.3 latency authority is bench 884, NOT this record — accuracy is this record's payload).

## Gate fit

G1 (the lane's own calibration claim): the clef lane publishes raw per-option probabilities — no calibration claim is made or gated here (the modelless G1 row in the tables carries its own PASS). The plan-619 GOAT pair remains bench 884 (G2) + riir-infer T2.2 (G1 parity).
