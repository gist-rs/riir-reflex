# Issue 073 — Drex DLM comparison lane (`--drex`, `/v1/systemone`)

**Status:** CLOSED 2026-10-08 — ALL TASKS LANDED (T1–T4+T6 @ `48206d5`, T5 the same day):
T5 PASS on typed_decisions (acc 0.5865, readout_ece 0.1929 / readout_brier 0.2663 — the
owned calibration cell reads NOT-CALIBRATED, the card's disclaimer vindicated over the
homepage claim) + the sst5 smoke (0.5900 — ABOVE every published sst5 bar, disclosed as
a board re-pricing, never a seat: CC BY-NC). Full record:
`.benchmarks/073_drex_lane_t5_suite_pass.md` (serving posture + box state + the
turnkey serving setup + the determinism-flag read). The issue file is removed per the
noise-reduction rule; the lane doc (`src/lanes/drex.rs` module doc) + the bench record
are the standing documentation.

Filed from `.research/008_Drex_DLM_SystemOne_Lane.md` (2026-10-07).
**T1–T4 + T6 LANDED 2026-10-08 (`48206d5`, shikuwa/4090):** the lane + harness wiring + the
T4 readout + the T6 caution; T5 (the suite pass) staged — serving setup recorded under T5
below. Wire pinned from their `api.py`/`serve.py`/`inference.py` @ `6c63df2` (re-cloned
under `.raw/drex-dlm` for the pin).

## What

Add Drex DLM (Nace.AI, open-weights 8B diffusion-LM decision model, the new top open
entry on the Decision Index) as a harness comparison lane, exactly the `--agentjev` /
`--clm` shape: same TypeSafe wire, different endpoint. Their stack serves, our Rust
measures — the MEASURE-vs-SERVE split.

## License law (load-bearing)

Weights are **CC BY-NC 4.0** (repo code MIT). This lane is **measurement only**:

- comparison/bench lane: YES (we publish numbers ABOUT the model);
- distill teacher: **NO** (NC contaminates — the bekko-MIT teacher posture does not
  transfer; never wire `TeacherForward::Drex`);
- product/serving lane: **NO** — and per the family ladder, a trained-encoder
  SERVING arm is `riir-rethink`'s charter (private), never reflex's; this lane
  stays measurement. For Drex that rung is license-blocked anyway (CC BY-NC).

## Why (owned data points)

1. Gold-label accuracy on `typed_decisions` per primitive — board context:
   laya-riir·typed 0.7445 / AgentJev 0.7715 (det ✗) / rethink typed 0.7550.
2. **Calibration readout** — their model card says `confidence` is "not a calibrated
   probability of correctness" while their homepage markets "calibrated probability";
   ECE/Brier of (a) their full per-option distributions and (b) their `confidence`
   fields vs correctness is the honest third-party read (the Clef risk-#6 posture).
3. No abstention surface — coverage/ECE contrast with our fused-abstain row.
4. Latency with serving posture disclosed (8B BF16 ~16 GB; Python-torch vs
   llama.cpp-`edlm` GGUF are different numeric postures — record which served; their
   own cross-runner delta is 0.0050–0.0094 per-option).

## How (mirror the agentjev lane)

- [x] **T1** `src/lanes/drex.rs` — `DrexLane` over `http_mini` (`DREX_SERVE_URL`,
      default `http://127.0.0.1:8000`); request builder from the TypeSafe shape
      (`{state, questions: {qid: {type, instructions, criteria}}}`, `noul` criteria
      `true`/`false` keys), response mapper for their `answers` + `usage` +
      `latency_ms` tail; stub-HTTP wire-shape test (the clm T2(b) pattern).
      *(LANDED `48206d5` — 8 tests; the measured wire divergences pinned: questions
      are an OBJECT keyed by qid (the openthai dialect), noul answer is
      `{type, noul: p_true}` with NO fallback (their to_answers has one shape,
      unlike openthai's two), choice/score probabilities read in OUR option order,
      their `confidence` read as-is (T4), their E[level] `score` float never the
      pick, wire bound 1..=255 (the suites' 2..=10 checked upstream), scorer
      `latency_ms` at TOP level, billing-style token counts under usage,
      liveness = GET /health.)*
- [x] **T2** `src/lanes/mod.rs` registration + `src/bin/harness.rs` `--drex` flag,
      `opts.drex`, `run_drex_lane` (agentjev laws: loud refusal without the server,
      fixed-throwaway warmup request, determinism rerun probe, the layla trim cap so
      capped rows never compare full-N vs capped-N).
      *(LANDED `48206d5` — the openthai-shape lane runner: /health probe, warmup
      captures the model id from the response's own `model` field, the trim cap,
      observed-repeat determinism over the first 10 cases.)*
- [x] **T3** `SuiteResult.drex: Option<LaneResult>` + `RunMeta.drex_lane` posture
      string (serde skip-when-absent — never a fabricated row).
      *(LANDED `48206d5`.)*
- [x] **T4** Calibration readout REUSING `src/harness/metrics.rs` (substrate-first —
      no parallel metrics code): per-primitive `hard_metrics` (ECE/Brier/coverage)
      over their per-option probabilities + `ece_of`/`brier_of` on (their
      `confidence`, correctness) pairs for the confidence-vs-correctness cells
      (choice + score formulas pinned from their `api.py` at the pinned sha).
      The `conformal_naive_floor` companion stays the G1 floor law.
      *(LANDED `48206d5` — `readout_brier: Option<f64>` added to LaneResult through
      the SHARED `assemble_laya_lane_result` tail: `brier_of` over the same pairs
      `readout_ece` reads — every comparison lane gains the axis uniformly, the
      modelless lane computes its calibrated-pairs twin; per-primitive cells ride
      the tail's `by_question_type` on typed_decisions. The formulas are pinned in
      the lane doc and the wire delivers their computed values — the lane never
      re-derives.)*
- [x] **T5** Suite pass on `typed_decisions` (+ one short-suite smoke) with
      `scripts/bench_preflight.sh` PROVENANCE quoted; table lands in
      `.benchmarks/` with serving posture + box state; the "this is the open 32K
      release, not hosted Drex 1.5" scope line in every table.
      *(PASS 2026-10-08 — `.benchmarks/073_drex_lane_t5_suite_pass.md`: their Python
      serve.py on CUDA (bf16, 15.5 GB resident on the 4090; `.raw/drex-env` torch
      2.6.0+cu124 + transformers 5.19; weights snapshot 16 GB → `.raw/drex-model`);
      typed_decisions acc 0.5865 / ece(maxp) 0.1082 / readout_ece 0.1929 /
      readout_brier 0.2663 / p50 98 ms, per-primitive choice 0.6167 · noul 0.5717 ·
      score 0.5750; sst5 smoke 0.5900 (above every published sst5 bar — board
      re-pricing disclosed, never a seat); determinism flagged ✗ (their bf16-CUDA
      forward is not bit-deterministic — the observed-repeat check doing its job);
      the warm sanity read matched their published example (87 tokens EXACT,
      argmaxes exact, deltas ≤ 0.012).)*
- [x] **T6** Long-state caution carried from their own validation: do NOT add
      16K-token states to the lane's cases (their own cross-runner file shows a
      first-third-marker retrieval failure); if a long-state probe is ever wanted it
      is its own bench with the fragility disclosed.
      *(LANDED `48206d5` — the caution lives in the lane module doc with the
      measured citation.)*

## Out of scope

- riir-infer eDLM/diffusion arch support: **FILED as riir-infer `.issues/1005_edlm_drex_inference_lane.md`** (owner opt-in 2026-10-07; Research 008 records the pattern intel — block-causal segment mask, row-form parity, prefix-cache reuse). Priority ordering stays owner-gated (bonsai/ternary first).
- The Research-002 shared-prefix-KB gap stays deferred (second reference impl noted).
- Any distill/product use (license law above).
