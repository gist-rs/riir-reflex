# Issue 017 — ANE lane reference design (laya-apple distill; owner hint 2026-09-24)

**Status:** OPEN — reference design distilled from tc3oliver/laya-apple v1.0; Phase-0 feasibility + owner gates pending. No code landed.

**Source pin:** `tc3oliver/laya-apple` @ `128a19302c616173617d1d8d4b152e2f667c618c` (Apache-2.0), benchmarks/v1.0.md + docs/support-matrix.md + docs/no-silent-fallback.md. Measured on **Apple M4 Max / macOS 26.6.2 / coremltools 9.0 / MLX 0.32.2** — their own routing.json says "thresholds are not validated elsewhere". Distilled 2026-09-24 (riir-clippy distill verdict: MARGINAL C corpus / LANE INTEL HIGH).

## Why an ANE lane is plausible for reflex

The owner hint: "reflex may need ane lane btw". laya-apple v1.0 is the first production-qualified ANE configuration for THIS exact model family (Laya decisions), and its shape fits the reflex serve edge precisely:

- **ANE wins the short single-question lane decisively** (M4 Max, forward P50): laya L64 8.1 ms / L128 9.9 ms; laya-multilingual L64 **3.4 ms** / L128 4.3 ms / L256 8.3 ms; typed L64 8.0 ms. At those lengths ANE beats even MLX FP16 GPU (9.4 / 5.4 ms at L64) and PyTorch MPS (21.4 / 16.7 ms). Reflex requests are single-decision, short (fixture p50 ~100 tokens) — exactly the winning class.
- **Heterogeneous GPU+ANE serving** (their closed-loop mix): 2.92–4.57× throughput vs GPU-only, short-stream P99 36–84 ms → 7–12 ms, head-of-line blocking removed (a long GPU forward no longer delays short requests). Their open-loop bursty rows are starker: short P50 195 ms GPU-only → 10.6 ms heterogeneous.
- **What an ANE lane does NOT replace**: L512+ (bucket sets top out at L128 for laya/typed, L128 for multilingual auto-routing — L256 parity passes but loses the conservative routing comparison) and multi-question requests (MLX/GPU batch wins at every measured length — never route those to ANE).

## The qualified configuration (what MUST be copied, not approximated)

1. **BC1S graphs only** — fixed batch=1, fixed length buckets (they convert per-bucket, `bc1s-masked`). The "ordinary" CoreML graph (BxLxC/SDPA, flexible/enumerated shapes) is **numerically WRONG on the ANE** — up to 85/187 hard decision mismatches, max prob error 1.0 — AND at L1024 a compile failure **silently drops to CPU** (2201.8 ms). Flexible/enumerated graphs run **100% on CPU** on this OS under every compute-unit setting. An ANE lane ships fixed-shape buckets or it ships garbage.
2. **Compute-plan verification at load**: 100% of ops on the Neural Engine + 0 device transitions, checked at build/import/load and after any change — else reject (`ComputeUnitMismatchError`), plus a RUNTIME placement probe (the static plan cannot see runtime placement — their audit finding V1).
3. **No silent fallback** (their doctrine, worth adopting verbatim): the router decides BEFORE the request runs and records the reason (`routing_reason`); a failure on a device stays on that device and fails the request; a corrupt/failed ANE artifact is dropped with a `RuntimeWarning` and its requests go to the other device WITH the reason recorded; an explicit-device request never pads up to a larger bucket (`UnsupportedShapeError` instead).
4. **Conservative auto-routing derivation** (derive_routing.py): walking buckets ascending, bucket b_i joins iff (a) BC1S parity passed at b_i AND (b) ANE P50 at b_i < MLX P50 at **b_{i-1}** (padding-cost-aware: a request just above b_{i-1} pads to b_i on ANE but costs ~MLX(b_{i-1}) on GPU); stop at the first failure. Note their own measured crossover ≠ production threshold (multilingual L256 ANE 8.3 ms is actually faster than MLX-same-length 8.6 ms, but loses to MLX at the previous bucket — the rule is deliberately conservative).
5. **Parity contract is DECISION-level, not p-drift** (load-bearing for our G5): their FP16 gate = prob error ≤ 0.02 + 0 hard mismatches + all finite + bitwise-identical repeats. A **hard mismatch** = argmax differs AND the reference's own top1–top2 calibrated margin ≥ 2× tolerance. A **near-tie flip** = decision differs inside that band — listed row by row, never hidden, never counted as a failure. Their ANE FP16 measured max prob err 0.0077–0.0128 — an ANE arm will very likely FAIL our Metal-lane G5 bar (p-drift ≤ 1e-3), so the ANE arm needs its OWN gate: top-1 agreement + near-tie-band report + a decision-level bar in the 0.02 class (calibrated on OUR goldens, not copied).

## NOT transferable

- **Absolute milliseconds** — M4 Max vs our M3 Max, their MLX FP16 GPU vs our hand-written MSL kernels. Same-box league cells own comparisons; our ANE numbers must be measured fresh on the M3.
- **The Python runtime/conversion tooling** — owner directive: no Python anywhere in the laya lane. Consistent split: coremltools conversion is a ONE-TIME OFFLINE step (like GGUF export); the shipped artifact is a compiled `.mlpackage`; the RUNTIME is native Rust. Python never runs at serving time.
- **MLX** — we do not use it; our GPU lane stays the Metal kernels.

## Reference implementation path (Rust)

- Feature `laya-riir-ane` (opt-in, macOS-only, native-only — never joins a wasm32 combo; per repo law, lands in the SAME commit as its parity `[[test]]` row).
- Deps: `objc2` (already in the Metal lane) + `objc2-core-ml` (+ `objc2-foundation`); MLModel load, fixed-shape `MLArrayBatchProvider`/`MLFeatureValue` FP16 inputs, prediction → logits → the existing calibration head unchanged.
- Artifacts: `assets/ane/` (or `LAYA_ANE_ARTIFACTS_DIR`) per model × bucket `.mlpackage`, produced by an offline `scripts/` conversion helper; digest-pinned in the manifest (the weights-hash discipline this repo already has).
- Load-time: compute-plan 100%-ANE/0-transitions verify → refuse otherwise; bucket → shape table; explicit-device request outside buckets = loud `Unsupported` (never pad up).
- Serve routing (serve.rs): single-question + padded-len ≤ max bucket + ANE healthy → ANE (reason recorded); everything else → Metal/CPU unchanged; ANE artifact/runtime failure → loud one-time warning + recorded-reason fallback (their rule 6/14), never a silent swap.
- Bench: `laya_fixture_timing --device ane` posture vs Metal vs CPU, SERIALIZED (Issue 015 containment — no parallel Metal/ANE instances in the same process until 015's root cause is diagnosed); A/B against the current Metal numbers, position-balanced rounds.

## Tasks

- [ ] Owner gate 1: is an ANE lane wanted at all (new artifact class `.mlpackage` in release archives vs download-on-demand; the no-Python boundary formally ratified for the offline-conversion step)?
- [ ] Phase 0 feasibility: offline coremltools conversion of laya + laya-multilingual (BC1S FP16, buckets {64,96,128} + {256} ml), compute-plan verify passes on the M3.
- [ ] `laya-riir-ane` feature + objc2-core-ml runtime path (same commit as the gate row).
- [ ] G5-ANE parity gate: top-1 agreement vs our frozen goldens + near-tie-band report + decision-level prob-err bar calibrated on OUR fixtures; p-drift published as OBSERVED, never gated at 1e-3.
- [ ] `laya_fixture_timing --device ane` posture + serialized A/B vs Metal on the M3; record absolute numbers + the box state.
- [ ] serve.rs routing + no-silent-fallback wiring (reasons recorded, explicit-device refusal outside buckets).
- [-] defer heterogeneous ANE+Metal two-queue serving (the 2.9–4.6× class) until Issue 015's parallel-instance root cause is diagnosed and the site's single-instance serialized posture has a plan — record as deferred, not dropped.
- [-] defer laya-typed-decisions ANE conversion until laya + multilingual are green (same ModernBERT-large geometry; no new information expected from a third model early).
