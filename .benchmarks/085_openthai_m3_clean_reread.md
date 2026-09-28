# Bench 085 — the M3 clean re-read: massive 1.71 s REPRODUCES, the contamination hypothesis REFUTED

**Status:** MEASURED 2026-09-28 — confirmatory re-read, all four prior M3 openthai
suites, box_state `latency_quotable: true` BOTH spans. Verdict: **074's
massive_intent_en 1.63 s p50 was never contaminated** — the clean run reads
**1710 ms** (Δ +4.9% vs 074), accuracy byte-identical on every suite, and the
massive:xnli latency ratio reproduces at **15.4×** (1710/111) exactly as 074's
1630/106. The thermal/load suspicion raised in the 084 review thread is
REFUTED by measurement.

## Why this run exists

The 084 review thread flagged `massive_intent_en 1.63 s` as the lone outlier
across both boards (every other openthai cell 50–180 ms) with two suspicion
lines: (a) 074's board run carried a start-load 6.95 sibling and the record
stamped its latencies order-of-magnitude only; (b) the massive:xnli ratio is
15.4× on the M3 but 1.8× on the 4090 (95/52), which looked like no single
cost model could reconcile (106, 1630) from one run. Both lines are now
answered by a preflight-clean re-read.

## PROVENANCE (the Issue-021 box-state law)

- `scripts/bench_preflight.sh` immediately before the run:
  **`PROVENANCE: power=AC Power load=3.39 swap=404.06M canary=127.3us/best5 powermode=2(high)`**
  — PASSED (AC, 5-min settle completed, load 3.39 < ceiling 6.0, canary
  127.3 µs vs ref 141 µs — the GPU running COOL, best-of-5 below reference).
- The harness's own `meta.box_state`: start load **3.25**, end load **2.91**,
  `latency_quotable: true` on BOTH spans, refusals `[]`. All timing cells
  below are QUOTABLE.
- **Exclusive GPU:** the concurrent Plan-426 session's openthai processes (an
  `8000` uvicorn + a full-train-split `--distill-teacher openthai` harness
  run over massive_intent_en, detached since 19:52) were killed as
  owner-confirmed orphans BEFORE the preflight; no other compute consumer
  during the run (the run's own end-load 2.91 agrees).
- **Server:** `openthai_systemone.server:app` (FastAPI/uvicorn) on
  127.0.0.1:**8001** from `.raw/openthai-systemone` @ `5d04bcc` (Bench 074's
  pin; the 084 dtype patch does NOT exist in this clean clone — MPS defaults
  fp32, the board's numerics, no env set). `permutations=1` pin green ×4
  (`determinism_ok: true`). The lane was pointed at 8001 via
  `OPENTHAI_SERVE_URL` — the sibling's 8000 post-run was gone.
- **Clean tree:** the working tree carried sibling Plan-426 WIP INSIDE the
  runner (`src/bin/harness.rs`, `src/harness/runner.rs` + submodules
  modified), so the harness was built from a detached worktree at
  `3a7be30` (= origin/develop, the 084-publish commit):
  `git worktree add --detach /Users/katopz/git/reflex-wt-085 3a7be30`, binary
  `reflex-wt-085/target/release/harness`. git_sha in results.json = `3a7be30`.
- **Datasets:** `.raw/datasets` (the 074/075 pool, NOT `datasets_t20k`) —
  slice digests match the published cells exactly (massive
  `fnv1a64-49390c8b80475849`, xnli `fnv1a64-b136750e5af1d412`), so every
  cell below is same-questions comparable with 074/075/084.
- **Suite order:** the harness sorts alphabetically — massive ran LAST, not
  the cold-first order requested. Immaterial: the number reproduced anyway
  (see Results), which is itself evidence against the thermal-ramp theory.

## Results — latency reproduces; accuracy byte-identical

| suite | p50 (this run) | p50 (prior M3) | acc | prior acc |
|---|---|---|---|---|
| massive_intent_en | **1710 ms** (p99 2384, support 4) | 1630 ms (074) | **0.9200** | 0.9200 ✓ |
| xnli_en | **111 ms** | 106 ms (074) | **0.8967** | 0.8967 ✓ |
| thai_wisesight | **107 ms** | 111 ms (075) | **0.4750** | 0.4750 ✓ |
| thai_sib200 | **112 ms** | 115 ms (075) | **0.8382** | 0.8382 ✓ |

- Accuracy byte-identical to 074/075 on all four suites — the lane measures
  faithfully across box states, again.
- Latency: every prior M3 cell reproduces within ±5% — including the one
  under suspicion. 1.63 s and 1.71 s are the same measurement; the workload
  is GPU-compute-bound, which is also why a CPU-load sibling (074's start
  6.95) barely moved it while our clean box reads within 5%.

## The mechanism note (inference, clearly labeled) — why 15× here and 1.8× on the 4090

The ratio divergence that motivated this run is REAL and is now explained,
not anomalous. Their design scores a question's options as one padded
forward (59 options ≈ 12k-token batch vs xnli's 3 ≈ 1k tokens, per the run's
own `usage.input_tokens`: massive 60907/300 q ≈ 203 tok/q). That batch is
**fp32-compute-bound on MPS** — M3-class fp32 throughput puts a 59×203-token
0.8B forward at ~1–2 s, measured **1.71 s** — while the same batch sits well
inside a 4090's stride at **95 ms**. So the massive:xnli ratio is 15.4× on
the M3 (compute scaling) and 1.8× on the 4090 (fixed overhead dominates) —
the 074-vs-084 comparison was never inconsistency; it is the fp32
compute-throughput gap on a big-batch workload. "Same 59-option slot head,
~17× faster on the 4090" (084) stands, now measured cleanly.

## The one correction this record makes

Nothing about the published numbers changes: 074's massive cell was right,
the site's 1.63 s is right. What changes is the STATUS of the M3 cells —
they were published order-of-magnitude / quotable-by-adjacent-run; this run
makes all four **QUOTABLE from their own clean run** (box_state both spans
true) and upgrades the family's provenance. The optional site republish
(`PUBLISH_BENCH_LANES=openthai`) refreshes the four cells to these values
(equal within noise, now self-certified).

## Determinism

`permutations=1` pin green on all four suite-runs (`determinism_ok: true`).
Modelless bit-identity green. The modelless by-product cells reproduce
074/075 exactly (0.3533 / 0.3400 / 0.1225 / 0.1422).

## Artifacts

- `085_openthai_m3_clean_reread/TABLES.md` + `results.json`

## Verdicts

- **The contamination hypothesis: REFUTED.** Clean box, canary-cool GPU,
  exclusive GPU, 5-min AC settle — and massive reads 1710 ms. The 084
  review thread's suspicion lines are both answered: (a) 074's load stamp
  was irrelevant to a GPU-compute-bound cell; (b) the cross-device ratio
  divergence is the workload's compute scaling (mechanism note above).
- **Plan 003 T3.5 scope extended:** all four M3 openthai latency cells are
  now quotable from their own run (was: thai ×2 via 075, EN ×2
  order-of-mag).
- No promotion claim — comparison lane, report-only (the Bench 074 law).

## Publish — recipe (site updates only if republished)

`PUBLISH_BENCH_LANES=openthai ../reflex-site/scripts/republish_bench.sh
.benchmarks/085_openthai_m3_clean_reread/results.json` from this repo's
root — the wrapper runs the self-test, chart smoke, the Issue-021 wall
(this run's own box_state passes it), the pairing gate, mirror parity and
the page smoke, then prints the manual commit + `npx wrangler deploy`
steps. Cells land under the `m3-max-metal` host container (`host: m3`),
replacing the 074/075-era values (equal within noise).
