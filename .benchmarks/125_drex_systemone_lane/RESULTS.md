# Bench 125 — the Drex lane at the Q8_0/Metal serving posture: independent reproduction + the determinism control

**Status:** RECORD — a SECOND session's independent pass over issue 073 (the lane landed
upstream the same day at `48206d5`/`a1bae34`, their posture = their Python `serve.py` BF16
CUDA on the 4090; this record = their llama.cpp `edlm` fork @ `cdcf65d`, Q8_0 GGUF, Metal,
M3). Two postures agreeing is the strength — the accuracy verdict and the not-calibrated
verdict are cross-posture; the determinism finding is this record's own (their det ✗
attribution is contradicted below). Raw evidence: `results.json` + `TABLES.md` (the
harness's own render, `--drex --skip-laya`); companion record:
`.benchmarks/073_drex_lane_t5_suite_pass.md` (the first posture).

> **Scope law:** the open 32K release (`nace-ai/drex-dlm` @ `6c63df2`), never hosted
> "Drex 1.5". Weights CC BY-NC 4.0 / repo code MIT — measurement only.

## Serving posture (this record)

| piece | value |
|---|---|
| speaker | their llama.cpp **`edlm` fork** @ `cdcf65d` (`tools/server/systemone.h`), built locally (Metal) |
| weights | `drex-dlm-Q8_0.gguf` (HF `nace-ai/drex-dlm-Q8_0`; sha256 `04a592249958762d…`, 8.19 GB) |
| serve cmd | `GGML_METAL_TENSOR_DISABLE=1 llama-server -m …Q8_0.gguf --embedding --pooling none -c 16384 -b 16384 -ub 16384 -np 1 --no-warmup --port 8097` (their README's Apple-Silicon posture verbatim; **the `-b/-ub` flags are load-bearing** — the first launch without them died in `GGML_ASSERT(n_outputs_max)`) |
| numeric posture | **GGUF Q8_0** — NOT their Python BF16; their own cross-runner spread is 0.0050–0.0094 per-option; the example-request spot check reproduced the published outputs within that class (team [0.9668, 0.0011, 0.0322] vs published [0.9641, 0.001, 0.0349]; refund 0.8499 vs 0.8548; urgency 0.5849 vs 0.5955; **87 input tokens EXACT**) |

## Box state (the latency law)

`scripts/bench_preflight.sh` **REFUSED** (load 13–16, sibling jobs on the box; the run's own
box-state span in TABLES.md reads ⛔ NOT QUOTABLE). Accuracy/calibration/determinism cells
are load-insensitive (one deterministic forward per decision on their server) and carry this
record; **the latency columns are PROVISIONAL** — never quote the ms figures.

## Reproduction verdicts (cross-posture)

| cell | this record (Q8_0/Metal fork) | the first record (BF16/CUDA python) |
|---|---|---|
| typed_decisions acc | **0.5855** | 0.5865 |
| per-primitive | choice 0.6167 · noul 0.5750 · score 0.5700 | choice 0.6167 · noul 0.5717 · score 0.5750 |
| their-confidence ECE | choice 0.1460 / score 0.2495 (per-kind) · pooled ≈ 0.2041 | 0.1929 (pooled readout) |
| smoke suite | ag_news **0.8950** (conf ECE 0.0588, floor-beating) | sst5 0.5900 |
| verdict | NOT CALIBRATED on typed_decisions | NOT CALIBRATED |

Agreement within 0.001 on the headline, exact on choice, and the same
not-calibrated verdict from two different numeric postures — the board finding stands:
**Drex's Decision-Index lead does not transfer to our gold-label split** (board: laya 0.7445 /
AgentJev 0.7715 / rethink 0.7550), and **the model card's "not a calibrated probability"
disclaimer wins over the homepage marketing** (on typed_decisions the confidence LOSES to the
split-half conformal-naive floor: 0.1460/0.2495 per-kind vs pooled-floor 0.1158; it beats the
floor only on near-trivial ag_news: 0.0588 vs 0.3831).

## The determinism control (this record's own finding)

Their T5 record flagged `determinism_ok = false` (10/10 rerun pairs byte-differ) and
attributed it to bf16-CUDA reduction nondeterminism. **That conclusion skipped a control**:
the response body carries `latency_ms`, which differs on every request BY DEFINITION — a raw
byte-compare measures their clock, not the decision, and reads ✗ on every posture regardless
of numeric determinism (the same artifact behind AgentJev's recorded det ✗).

This record is the control: with `decide_raw` normalizing ONLY that field away, the two FULL
suite runs at this posture are **byte-identical on the entire answers surface** (same
accuracy to 4 decimals, same per-kind cells, same token counts — 191,962 in / 164,868 out on
typed_decisions, twice) and the det column reads ✓. The fix is landed with this record (the
`decide_raw` timing-tail strip + the `decide_raw_strips_only_the_latency_tail` test); the
BF16/CUDA posture should re-read its det column under the same normalization before
"nondeterministic" is claimed again — if it still byte-differs AFTER the strip, THAT would be
the real reduction nondeterminism, and it is currently unmeasured.

## Laws carried

- **License:** CC BY-NC 4.0 — comparison lane only; never a teacher, never a product lane.
- **Metrics reuse:** the cells compute through `crate::harness::metrics`
  (`ece_of`/`brier_of`/`conformal_naive_floor`/`CalibrationPair`).
- The per-kind cells + split-half floor companion are this record's instrumentation (the
  upstream lane carries the shared `readout_brier` instead); the extension is filed as
  `.issues/076_drex_conf_cells_det_followup.md`.
- Long-state caution: their own 15,644-token first-third retrieval failure — never a
  16K-token case in this lane; over-context refuses loudly (their 400; the 1 MB body cap
  refused at the client).
