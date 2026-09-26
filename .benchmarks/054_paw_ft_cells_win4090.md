# Bench 054 — PAW `paw-ft-bs48` finetune-compiler cells, hosted anonymous posture, 4-suite subset (Issue 033)

**Status:** RECORD — measured 2026-09-26/27 on the 4090 Windows box (shikuwa); one run,
artifacts in `054_paw_ft_cells_win4090/`. ⚑ **TWIN-RECORD NOTE (the Issue-825
discipline, repair-beats-delete):** an independent session measured the same
lane posture in parallel — [Bench 053](053_paw_ft_bs48/BENCH.md) (M3,
stratified sample, re-measured default rows beside). Every direction AGREES
across the two boxes: ft strictly beats the mapper tier on all 4 suites; the
win concentrates at banking77 (refusals collapse 74.8%→34.4% theirs /
73%→42.2% mine; strict +32.4pt theirs / +25.4pt mine); sst5 +6.2/+6.5;
ag_news +0.8/+1.0; emotion +2.8/+2.8. This record keeps its own cells because
the sample differs (full test pull vs their stratified 200/suite) — the two
tables are independent confirmations, not one measurement twice. (`TABLES.md`, `results.json`,
`paw_programs.json` = the program-id cache that produced the cells). Continues
[Bench 049](049_paw_lane_hosted_m3.md) with the finetune tier — the same 4
suites, the same specs (BLAKE3-identical), the same mapping law, so every cell
is comparable to 049's default-compiler row.

## Provenance

- **Box:** Windows 11, i7-13700K, RTX 4090 (idle: 549 MiB / 24 GB, no compute
  apps) — **accuracy only**, as in 049: PAW latency is dominated by their
  network + a fresh TLS handshake per curl spawn, never an engine figure.
  Box state UNJUDGED (the harness probes are macOS-shaped on this box) and
  irrelevant to the claim.
- **Code:** reflex `fd3f3d3` (HEAD, the Bench 051 tree), harness `--release`
  rebuilt from it, default features, `--skip-laya`. Command:
  `PAW_COMPILER=paw-ft-bs48 PAW_COMPILE_ASYNC=1 PAW_PROGRAM_CACHE=<repo>/.raw/paw/programs.json harness --suites banking77,ag_news,emotion,sst5 --skip-laya --paw`.
- **PAW side:** hosted API, **anonymous posture** (no key; programs PUBLIC on
  their hub — the 049 finding). Compiler snapshot **`paw-ft-bs48-20260530`**
  (their finetune tier; 049's default mapper was
  `paw-4b-qwen3-0.6b-20260407`). Inference `temperature 0.0`,
  `max_tokens 48`. Health at launch: `paw-ft-bs48` 1 worker, queue 0.
- **Datasets:** fetched fresh on this box (`.raw/datasets/`, the
  `fetch_datasets.sh` protocol; HF datasets-server 429 walls during the fetch
  were ridden out with cooldown + re-run — the banking77 mirror test split is
  3,076 rows / 31 pages, not the 10.4k the `cap=all` name suggests; the
  harness served its own 500-case eval cap). The modelless baselines below
  match 049 byte-for-byte on all four suites — same pull, same caps, so the
  PAW columns are apples-to-apples.

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| suite (n) | PAW-ft acc | refusals | answered-acc | PAW default (049) | Δ | modelless (same run) | det |
|---|---|---|---|---|---|---|---|
| ag_news (400) | **0.7925** | 0 (0.0%) | 0.7925 | 0.7825 | +1.0pt | 0.5100 | ✓ |
| emotion (400) | **0.5025** | 0 (0.0%) | 0.5025 | 0.4750 | +2.8pt | 0.2825 | ✓ |
| sst5 (600) | **0.3933** | 0 (0.0%) | 0.3933 | 0.3283 | +6.5pt | 0.2167 | ✓ |
| banking77 (500) | **0.3940** | **211 (42.2%)** | 0.6817 | 0.1400 (73.0% refused) | +25.4pt | 0.4460 | ✓ |

sst5 score MAE (answered) 0.885, within-1 0.818. Compile was **~12 s per
suite** (async endpoint answered near-instantly; the 049 issue's "2–5 min"
estimate did not materialize on this tier either). Server-reported inference
p50 60–96 ms (their `latency_ms`; observation only).

## Findings

1. **The finetune tier is strictly better than the mapper tier, everywhere —
   and the win concentrates exactly where the mapper tier failed.** banking77
   refusals 73% → 42.2% (the ft program still answers in its own vocabulary —
   `"locate my card"`, `"card waiting"` — but less often), answered-acc
   0.5185 → 0.6817, and strict acc ×5 (0.1400 → 0.3940). Small label sets
   gained +1 to +6.5pt.
2. **banking77 stays the typed-wire moat's exhibit.** Even the ft tier sits
   below the modelless lane's 0.4460 strict (and far below the
   heads-selected 0.684) — a free-text product paying a 42% refusal tax on a
   77-way label set is the finding, now measured on BOTH compiler tiers.
3. **Hosted determinism: 4/4 ✓ this run** (049 measured sst5 ✗ on the mapper
   tier, same box class). One repeat-stable reading is not a determinism
   claim — recorded per run-date per the lane's disclosure.
4. **Cost honesty:** anonymous tier, 4 compiles + 1,900 inferences, no key.
   The finetune-compile latency the issue priced at 2–5 min measured ~12 s —
   the tier's "much higher accuracy" cost is compute on THEIR side, invisible
   to the caller.

## What remains (Issue 033)

- Posture B lane wiring (local deterministic runtime, the gliner
  subprocess-oracle precedent) + a full-N local run.
- Arena republish via `../reflex-site/scripts/publish_bench.py` (both 049 and
  this record's PAW rows — needs the reflex-site checkout's owner pass).
- HISTORY close recording which posture the published cells used.
