# Bench 058 — PAW hosted ft-tier at the stratified law: the same-law mechanized anchor (Issue 033)

(RENAMED from 057 at allocation close — same-day dual allocation with the sibling's
`057_issue038_t7_levers` (Issue 038 T7), which was committed and pushed first and keeps
057; the recorded `--out` path below is the 057 spelling the run actually wrote.)

**Status:** RECORD — measured 2026-09-27 on the M3 Max; one run, artifacts in
`058_paw_ft_hosted_m3_stratified/` (`TABLES.md`, `results.json`). This is the
mechanized, same-law successor to Bench 055's hand-derived stratified anchor:
the harness at the Issue-039 T2 stratified protocol (`d7c0d4f`, built in an
isolated worktree `/Users/katopz/git/riir-reflex.wt033` because the checkout
carried the sibling session's WIP; the doc's own `git_sha` stamp reads
`de67f50` because the sibling committed MID-RUN — the running BINARY is
`d7c0d4f`'s, and that commit's engine changes (nb_ridge default-on) are NOT
in these cells) running the hosted `--paw` lane with
`PAW_COMPILER=paw-ft-bs48-20260530` — the ft tier that Bench 055 measured by
hand at 200/suite, here at the published table's full stratified populations
(400/400/600/500).

## Provenance

- **Box:** Apple M3 Max, macOS, AC, powermode 2 (High Power). Preflight
  immediately before the run: `PROVENANCE: power=AC Power load=5.70
  swap=2366.75M canary=137.6us/best5 powermode=2(high)` → **PASSED** (load
  5.70 < ceiling 6.0). Harness box stamp: start load 5.24 (inside bounds) →
  end load 7.96 — ⛔ **latency NOT QUOTABLE** (the sibling session's next
  compile burst landed mid-run). Per the Issue-021 law this record claims
  **accuracy only**; the client round-trip latency columns in `TABLES.md`
  are observation, and the SITE publish (`reflex-site` `dffabe3`) carries the
  paw lanes `:acc-only` — accuracy published, latency fields stripped at the
  publish boundary (the source doc keeps its measured cells).
- **Command:**
  `REFLEX_BENCH_HOST=m3 PAW_COMPILER=paw-ft-bs48-20260530
  /tmp/reflex-paw-target/release/harness --paw --skip-laya
  --suites ag_news,emotion,sst5,banking77 --datasets-dir .raw/datasets_t20k
  --out .benchmarks/058_paw_ft_hosted_m3_stratified` (run from the main
  checkout so `.raw/paw/programs.json` + the datasets resolve; ~30 min wall).
- **PAW side:** hosted API, anonymous posture (no key; programs PUBLIC on
  their hub), compiler snapshot **`paw-ft-bs48-20260530`**, all four programs
  compile CACHE HITS from the Bench-055 run (compile ~1 s each). Inference
  `temperature 0.0`, `max_tokens 48`. Specs unchanged (`scripts/paw_specs/`,
  the same BLAKE3s as 049/054/055).

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| suite (n) | PAW-ft acc | refusals | answered-acc | 055 anchor (stratified 200/suite, hand-derived) |
|---|---|---|---|---|
| ag_news (400) | **0.7900** | 0 (0.0%) | 0.7900 | 0.7900 |
| emotion (400) | **0.5000** | 0 (0.0%) | 0.5000 | 0.5000 |
| sst5 (600) | **0.3950** | 0 (0.0%) | 0.3950 | 0.3933 |
| banking77 (500) | **0.4200** | **173 (34.6%)** | 0.6416 | 0.4200 (34.4% refused) |

**Every direction agrees with the Bench-055 hand-derived anchor** — three of
four suites byte-identical, sst5 +0.0017, banking77's refusal rate 34.6% vs
34.4%. Hosted inference is not promised deterministic, so this is agreement
at the noise floor, not bit-identity; it confirms 055's hand derivation and
promotes the same-law hosted cells to machine-published provenance
(`lane_sources.paw` = `de67f50` on the site).

## Reads

1. **The hosted ft tier is now mechanized at the published table's law.** The
   arena's `paw (hosted)` row and the `paw (local)` row (Bench 056) are
   same-population comparable with the published modelless/laya lanes;
   049/054's first-N-law docs stay bench records only (never published —
   the site's lane-scoped filter dropped their by-product modelless controls
   loudly rather than blending postures).
2. The published modelless rows remain the CALIBRATED Bench-052-protocol
   posture (heads ON); this run's modelless control column is the baseline
   heads-OFF engine and is NOT published (the filter's whole reason).
3. **Deferred:** a latency-quotable hosted re-run (Bench 058 class) once the
   box holds a green preflight through a whole run — the publish is then a
   pure `publish_bench.py` lane-update (the `:acc-only` suffix comes off).
