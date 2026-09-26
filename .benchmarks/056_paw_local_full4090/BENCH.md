# Bench 056 — PAW Posture B: the LOCAL llama.cpp runtime cells, full-N, stratified law (Issue 033)

**Status:** RECORD — measured 2026-09-27 on the 4090 Windows box (shikuwa); one run, artifacts in `056_paw_local_full4090/`. The last open lane task of Issue 033: the SAME compiled programs the hosted cells measured, answered through their LOCAL runtime (`paw.function(program_id)` over the `programasweights` venv, llama.cpp auto-CUDA) as a Python subprocess oracle (`--paw-local`, `src/lanes/paw_local.rs` + `scripts/paw_local_lane.py`). The lane never compiles — the program ids come from the hosted lane's cache, so the posture delta (server vs local llama.cpp) is measured on identical artifacts.

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| suite (n) | paw-LOCAL acc | refusals | answered-acc | p50 (local) | hosted-ft SAME LAW (055, M3, stratified 200/suite) | modelless (same run) | det |
|---|---|---|---|---|---|---|---|
| ag_news (400) | **0.8000** | 0 (0.0%) | 0.8000 | 264.0 ms | 0.7900 | 0.4050 | ✓ |
| emotion (400) | **0.4875** | 1 (0.2%) | 0.4887 | 113.7 ms | 0.5000 | 0.2750 | ✓ |
| sst5 (600) | **0.4167** | 0 (0.0%) | 0.4167 | 118.3 ms | 0.3933 | 0.2017 | ✓ |
| banking77 (500) | **0.4120** | 177 (35.4%) | 0.6378 | 131.7 ms | 0.4200 (34.4% refused) | 0.2600 | ✓ |

sst5 score MAE (answered) 0.860 · within-1 0.817. **Determinism 4/4 ✓** — the local greedy posture repeated byte-identically on every observed-repeat probe (10/suite); the hosted posture never promised this.

## Reads

1. **The runtime posture is accuracy-neutral and determinism-positive.** Local-vs-hosted deltas at the same law (055, stratified) are −1.3 to +2.3 pt — inside run-to-run variation for these sample sizes — and banking77's refusal rate is nearly identical (35.4% local vs 34.4% hosted-stratified): the refusal behavior is a property of the PROGRAM, not of the runtime serving it. What the local runtime ADDS is the determinism the hosted tier cannot promise (4/4 byte-identical) and ~4-8× lower end-to-end latency (114-264 ms local vs 927-970 ms hosted round-trip; their network + TLS dominates the hosted number).
2. **banking77 keeps its shape on both postures**: the ft program answers in its own vocabulary ("locate my card", "top up by day", "exchange currency"…) 35% of the time; answered-acc 0.6378 local. The typed-wire moat's exhibit stands.
3. **Against our modelless lane at this run's caps, PAW-ft leads all four suites** (ag_news 0.8000 vs 0.4050 · emotion 0.4875 vs 0.2750 · sst5 0.4167 vs 0.2017 · banking77 0.4120 vs 0.2600) — the compile-a-classifier category BUYS its accuracy on our protocol; reflex's claim is the serving posture (modelless = 0.1-0.3 ms p50, zero compile, zero network, bounded ECE with a passing conformal floor), not raw-accuracy parity. ⚠ The modelless CONTROL column here is pull-limited on this box: `.raw/datasets` was fetched 09-26 through the HF 429 walls and carries fewer train rows than the M3 twins' `.raw/datasets_t20k` pull, so the count-table corpora are thinner (the modelless engine is the only lane that consumes train rows — PAW programs are frozen artifacts and the test splits are complete). Do not read this run's modelless column as a regression against 052/055's; it is a different pull.
4. ⚠ **SAMPLE-LAW DISCLOSURE (do not cross-read the tables):** this run measured under the Issue-039 T2 stratified test sample. [Bench 054](../054_paw_ft_cells_win4090.md) (hosted, this box, same programs) ran at `fd3f3d3` — BEFORE `a2353e2` (039 T2) landed — so 054's rows, including its modelless controls, are FIRST-N-law numbers and NOT case-identical to these. The same-law hosted anchor is [Bench 055](../055_paw_ft_bs48_m3/BENCH.md) (M3, stratified 200/suite): every direction agrees with the local cells above. 054's "modelless baselines match 049 byte-for-byte" is a first-N-vs-first-N match — internally consistent, a different population from this run's.

## What ran

```
uv venv .raw/paw-env && uv pip install --python .raw/paw-env/Scripts/python.exe programasweights==0.4.10
.raw/paw-env/Scripts/python.exe scripts/paw_preload.py          # base (594 MB, one-time) + 4 program bundles
PAW_LOCAL_PYTHON=.raw/paw-env/Scripts/python.exe \
  target/release/harness --paw-local --skip-laya \
  --suites ag_news,emotion,sst5,banking77 --out .benchmarks/056_paw_local_full4090
```

Runtime `programasweights 0.4.10` (llama-cpp-python 0.3.19, `n_gpu_layers` auto → CUDA on the 4090). Programs all cache hits (`PAW_COMPILER` unset → each suite's single cached program auto-selected, printed loud; a `paw-ft-bs48-20260530` row stamps provenance into every cell). Wall: ~8 min for all four suites (incl. 4 model loads), accuracy-only claim — the per-call p50 is their in-process round-trip, quoted beside hosted round-trips for posture comparison, never as an engine figure. Box state UNJUDGED (probes are macOS-shaped here; irrelevant to the accuracy claim).

## Lane engineering notes (measured, for the next port)

- **The uv-venv trampoline deadlock**: `python.exe` in a uv venv is a trampoline that spawns the real interpreter as a child while keeping its own inherited copy of the stdin pipe's write handle — closing our `ChildStdin` then `wait()`ing deadlocks the reap (trampoline waits interpreter, interpreter waits stdin EOF, EOF needs every write handle closed). `PawLocalSession::drop` kills the trampoline FIRST, then waits; the interpreter EOFs and exits on its own.
- **Cache-key posture**: the local lane reads the hosted cache keyed `(suite, compiler, BLAKE3(spec))`; `PAW_COMPILER` unset + exactly one program for the suite auto-selects it (loud); ambiguity refuses naming the tiers.
- **Protocol**: one process per suite, `{"program_id","input"}` → `{"output"}` JSON lines, their loader chatter on stderr, `{"error"}` → loud. Reuses the hosted lane's spec/drift-guard/render/mapping/tally verbatim (`src/lanes/paw.rs`) — 8 in-module tests incl. an end-to-end law pin over scripted channels.

## What remains (Issue 033)

- Arena republish via `../reflex-site/scripts/publish_bench.py` lane-update merge (deferred to the Issue-039 T5 both-hosts re-run).
- HISTORY close recording which posture the published cells used.
