# Issue 073 — Drex DLM comparison lane (`--drex`, `/v1/systemone`)

**Status:** OPEN — filed from `.research/008_Drex_DLM_SystemOne_Lane.md` (2026-10-07).

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
- product/serving lane: **NO**.

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

- [ ] **T1** `src/lanes/drex.rs` — `DrexLane` over `http_mini` (`DREX_SERVE_URL`,
      default `http://127.0.0.1:8000`); request builder from the TypeSafe shape
      (`{state, questions: {qid: {type, instructions, criteria}}}`, `noul` criteria
      `true`/`false` keys), response mapper for their `answers` + `usage` +
      `latency_ms` tail; stub-HTTP wire-shape test (the clm T2(b) pattern).
- [ ] **T2** `src/lanes/mod.rs` registration + `src/bin/harness.rs` `--drex` flag,
      `opts.drex`, `run_drex_lane` (agentjev laws: loud refusal without the server,
      fixed-throwaway warmup request, determinism rerun probe, the layla trim cap so
      capped rows never compare full-N vs capped-N).
- [ ] **T3** `SuiteResult.drex: Option<LaneResult>` + `RunMeta.drex_lane` posture
      string (serde skip-when-absent — never a fabricated row).
- [ ] **T4** Calibration readout REUSING `src/harness/metrics.rs` (substrate-first —
      no parallel metrics code): per-primitive `hard_metrics` (ECE/Brier/coverage)
      over their per-option probabilities + `ece_of`/`brier_of` on (their
      `confidence`, correctness) pairs for the confidence-vs-correctness cells
      (choice + score formulas pinned from their `api.py` at the pinned sha).
      The `conformal_naive_floor` companion stays the G1 floor law.
- [ ] **T5** Suite pass on `typed_decisions` (+ one short-suite smoke) with
      `scripts/bench_preflight.sh` PROVENANCE quoted; table lands in
      `.benchmarks/` with serving posture + box state; the "this is the open 32K
      release, not hosted Drex 1.5" scope line in every table.
- [ ] **T6** Long-state caution carried from their own validation: do NOT add
      16K-token states to the lane's cases (their own cross-runner file shows a
      first-third-marker retrieval failure); if a long-state probe is ever wanted it
      is its own bench with the fragility disclosed.

## Out of scope

- riir-infer eDLM/diffusion arch support (owner priority: bonsai/ternary first;
  Research 008 records the pattern intel — block-causal mask, row-form parity,
  prefix-cache reuse — for whenever a causal decision lane lands).
- The Research-002 shared-prefix-KB gap stays deferred (second reference impl noted).
- Any distill/product use (license law above).
