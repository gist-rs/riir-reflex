# Bench 052 — Issue 039 T2/T3/T4: the stratified split (representative test sample + stratified cal + complement pool + fallback guard), and the readout-selection lever's measured negative

**Status:** MEASURED 2026-09-26 (M3, metal). T2 + T3 LANDED; T4 adjudicated NEGATIVE and demoted to a report-only table. This is the re-run Issue 039 T2 demanded: the sampling law changed every suite's cases, so every lane re-ran (modelless + laya metal, all 15 suites, one run).

**Box state (Issue 021 preflight, quoted):** `PROVENANCE: power=AC Power load=5.16 swap=2502.75M canary=131.1us/best5 powermode=2(high)` — settle 1653 min on AC; swap warning noted (paging can masquerade as slow kernels; accuracy is unaffected). Latency rows are single-pass readings under sibling load — read them as magnitude, not record.

## What changed (the protocol, `suites.rs::stratified_split` + runner rewiring)

- **Test sample** = label-stratified round-robin over the whole test split (budget = registry cap; first-appearance label order, dataset order within each label; deterministic, no RNG). The first-N law was a label PREFIX on label-sorted mirrors: banking77's 500 first test rows spanned **13 of 77** labels, massive's 300 spanned **30 of 60**. Budget-0 suites (typed_decisions, prompt_injections) are identity — byte-identical to before.
- **Cal slice** = the same law over the train rows (the old first-200 cal was label-clustered: ~2/77 banking77 labels).
- **Corpus pool** = the train rows MINUS the cal front (the split's `rest` envelope — excluded by CONSTRUCTION, not position; the positional cut was orphaning whole label blocks: banking77's first label had ZERO pool docs at the full pull).
- **Fallback guard (T3)**: labels with no pool docs are disclosed loud (stderr + results + ⛔ markdown), never silently self-doc'd again.
- **Readout lever (T4)**: `EngineConfig::readout` + candidates {dispatch, max_prob, inv_entropy} ranked by in-sample calibrated ECE on cal. **Measured NEGATIVE** (below) → demoted to report-only.

## Dataset suites (modelless vs laya best, the published posture: `--head-select --nb-select`, t20k full pull, dispatch readout everywhere)

| suite | n | modelless | laya best | Δ | G1 (cal ece vs floor) | readout report (best_on_cal) |
|---|---|---|---|---|---|---|
| typed_decisions | 400 | 0.3300 | 0.7445 | −41.5 | PASS (0.0805 vs 0.3446) | max_prob (not armed) |
| ag_news | 400 | 0.8825 | 0.9500 | −6.8 | PASS (0.0025 vs 0.2482) | max_prob (not armed) |
| emotion | 400 | 0.7375 | 0.5925 | **+14.5** | **FAIL (0.2274 vs 0.1371)** ¹ | max_prob (not armed) |
| sst5 | 600 | 0.3967 | 0.3717 | **+2.5** | PASS (0.0050 vs 0.2220) | max_prob (not armed) |
| prompt_injections | 116 | 0.7672 | 0.6983 | **+6.9** | PASS (0.0348 vs 0.3622) | max_prob (not armed) |
| xnli_en | 300 | 0.5233 | 0.8600 | −33.7 | PASS (0.0067 vs 0.1351) | max_prob (not armed) |
| massive_intent_en | 300 | 0.7800 | 0.6933 | **+8.7** | PASS (0.0796 vs 0.1209) | dispatch |
| banking77 | 500 | 0.8260 | 0.4220 | **+40.4** | PASS (0.1740 vs 0.2887) | dispatch |

**Modelless ≥ laya on 5/8** (was 4/8 at Bench 051's fair posture): emotion, sst5, prompt_injections, massive, banking77. p50 stays sub-ms (banking77 0.394 ms — the count tables re-rank, they do not re-read the pool).

¹ **The honest regression.** emotion's G1 PASSED at 051 (ece below floor on the first-400 sample) and FAILS here (raw ece 0.7252 → calibrated 0.2274, floor 0.1371). The 051 pass was a sampling artifact: the first-400 emotion test rows were a label subset; at the representative sample the calibration genuinely does not reach the conformal floor. The demoted maxp readout would read 0.1506 — closer, still failing. The temperature candidate (T4') is the untested arm; deferred.

## T4 verdict — measured NEGATIVE, lever demoted to report-only

The lever armed per-suite readout modes selected on in-sample calibrated ECE over the cal slice (margin 0.005, min 32 pairs). One full armed run measured before the demotion (accuracy byte-identical to the dispatch run on every suite — the readout never moves picks; only confidence moves):

- **On the wide suites the lever exists for** (banking77 77-way, massive 60-way): `max_prob` IS the dispatch wide arm — the candidates' cal ECE coincide exactly (banking77 0.7860 = 0.7860; massive 0.4618 = 0.4618), and `inv_entropy` measured WORSE (0.8070 / 0.5519). **No functional beats the shipped law where the gap was.**
- **Where a candidate won on cal** (narrow suites, entropy→maxp switch), arming it overfit the cal slice: emotion's test G1 read 0.1506 (fail vs floor 0.1363) under the armed maxp vs 0.2274 under dispatch — closer, still failing; the flip was cal-noise dressed as signal.
- **The wide-label G1 gap closed anyway — via T2's stratified cal slice**: massive/banking77/ag_news all PASS at the shipped dispatch readout (the 051 failures were fit on a ~2-label cal slice).
- GOAT adjudication per the house law (a lever that regresses a suite is demoted): the arming is OFF. The candidate table stays in results (`readout_report`) + the `EngineConfig::readout` knob stays opt-in. G2/G4 re-pinned green with the plumbing in (`decision_set_goat`: p99 171 µs ≤ 1 ms, 0 post-warmup allocs, canary live).

## T3 first-run findings (the guard's yield)

- typed_decisions: **1 starved label** — a workflow whose rows all sit in the cal front (disclosed by name; the pool genuinely lacks it).
- code_fixtures: **4** (embed, readout, laya::agent, laya::router) — by construction (modules with <10 fns self-doc; the suite's own doc says so). The guard made the silent class visible.
- All classification suites: clean at the full pull — T1's fix holds under the guard.

## What ran

```
cargo build --release --features laya-riir,laya-riir-metal --bin harness
LAYA_DEVICE=metal ./target/release/harness --head-select --nb-select \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/052_stratified_readout
cargo bench --bench decision_set_goat     # G2/G4 re-pin
```

Tree: working tree at `b1aee72` + this change set (uncommitted at run time). 15 suites, no absences; laya metal determinism `ok` on every suite. Artifacts: `results.json`, `TABLES.md` (this dir).

## 4090 re-run — Issue 039 T5 DONE (2026-09-27)

Same protocol on the 4090 (Windows, RTX 4090, `LAYA_DEVICE=cuda`,
`REFLEX_BENCH_HOST=4090-windows`) at reflex `634093f`, harness built
`--features laya-riir,laya-riir-cuda` in an ISOLATED WORKTREE on that box
(the checkout's dirty sibling-WIP excluded by construction). 15/15 suites,
no absences, exit 0. Artifacts: `results_4090.json` / `TABLES_4090.md`
(this dir); comparator `scripts/compare_052_4090.py` (decision-level exact
axes; latency/seconds excluded by design — box properties).

- **Datasets**: the queued premise "`datasets_t20k` bytes already copied
  there" was STALE — the box had only the old `.raw/datasets` pull and its
  own re-fetch had died on curl 429s. Copied this session (tarball over
  scp), then byte-identity VERIFIED before the run: 977/977 files, sorted
  SHA256-manifest diff EMPTY vs the M3.
- **Modelless: bit-identical at the decision level, 14/14 suites** —
  accuracy, macro-F1, the full confusion (pick sets), head + nb
  selections, gate-fit thresholds, and G1 verdicts all exact. 13 suites at
  the 052 snapshot; `code_fixtures` (whose cases are this repo's OWN source
  spans — `code_fn_slices()` include-derived, so the row moves when src/
  moves) verified at the SAME snapshot via an M3 modelless-only re-run at
  `634093f`: byte-identical to the 4090 (acc 0.2917, identical confusion,
  thresholds, and the same 4-label corpus-fallback set). The 052-record
  mismatch on that suite was input drift (its tree predates `b092869`'s
  new fns), not cross-host divergence. `harness_cache_reuse` is LLM-lane
  only — skipped-identical on both.
- **One raw statistic differs at ulp level, 5/14 suites** (`readout_ece_raw`,
  absolute Δ ≤ 4.5e-10: ag_news 4.5e-10, sst5 9.9e-11, typed_decisions
  6.0e-11, massive 2.5e-11, banking77 2.2e-11) — the NEON↔AVX SIMD
  reduction-order class: f32 score paths drift at ulp, NO pick or decision
  moves anywhere, and the calibrated `readout_ece` is exact on every suite.
  The cross-host identity claim stays at the decision level; the continuous
  statistic is disclosed rather than pooled into the claim.
- **laya: accuracy identical cross-backend (metal ↔ cuda) on 8/8 dataset
  suites** — every published 052 laya cell reproduces on cuda. ONE
  repeat-check flag: banking77 cuda `determinism_ok = FALSE` (the first-10
  double-answer render differs on ≥1 repeat; metal True, every other cuda
  suite True). Reported-not-claimed per `meta.determinism_scoping`; the
  measured picks still match metal exactly. Shape-scoped (banking77 = the
  long-sequence ~317-token, 77-way suite) — filed to riir-infer (the lane
  owner) as riir-infer Issue 021. **RESOLVED 2026-09-27** (riir-infer
  `c64d0b1`): the cuda chain cache kept a dead buffer's slot entry at a
  recycled host address (the fused-GLU temp vs the hidden that reused its
  address), so `download_into`'s prefix match had two same-epoch candidates
  and HashMap iteration order picked — ~50/50 — feeding the act head a
  garbage CLS row (act_probability saturated, picks never moved); fixed by
  same-ptr different-len eviction on bind + the download's trailing sync
  (cudarc's `memcpy_dtoh` is async). `determinism_ok = true` ×4 on the
  re-run; record: riir-infer HISTORY (Issue 021).
- Latency indicative only (UNJUDGED box state — the Issue-021 probes are
  macOS; the Bench-047 precedent): banking77 cuda p50 23 ms vs the M3
  metal 47 ms; whole run 8m33s wall. Launcher: the Bench-047 schtasks
  one-shot pattern (ssh-launched processes die with the session); worktree
  + task deleted after the run.

## Cross-checks

- Accuracy identical between the armed and dispatch runs on every suite (readout independence, measured).
- typed_decisions n=400/2000-questions matches 051's results.json (the BENCH.md "2000" there was the question count — comparability holds).
- Uncapped suites byte-identical (identity split): prompt_injections laya 0.6983 = 051's row exactly.
