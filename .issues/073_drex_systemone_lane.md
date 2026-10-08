# Issue 073 — Drex DLM comparison lane (`--drex`, `/v1/systemone`)

**Status:** OPEN — filed from `.research/008_Drex_DLM_SystemOne_Lane.md` (2026-10-07).
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
- [ ] **T5** Suite pass on `typed_decisions` (+ one short-suite smoke) with
      `scripts/bench_preflight.sh` PROVENANCE quoted; table lands in
      `.benchmarks/` with serving posture + box state; the "this is the open 32K
      release, not hosted Drex 1.5" scope line in every table.
      *(STAGED 2026-10-08, shikuwa/4090: the serving posture = their Python
      `serve.py` on CUDA — `.raw/drex-env` built (torch 2.6.0+cu124 live,
      transformers 5.19.0, pydantic, hf_hub; their requirements satisfied),
      `nace-ai/drex-dlm` BF16 (~16 GB) downloading into `.raw/drex-model`;
      `nvidia-smi` headroom fine for 8B bf16 on the 4090. The Q8_0-GGUF/
      llama.cpp-fork posture (their edlm fork @ cdcf65d, nvcc 13.3 present) is
      the alternative — the reply's `model` field discloses whichever serves.
      The run itself is the next unit once the weights land.)*
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
