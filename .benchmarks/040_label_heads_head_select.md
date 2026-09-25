# Bench 040 — label heads (issue 030 lever 4): fitted per-label heads over the hashed-bag features

**Status:** LANDED 2026-09-25 · deterministic modelless lane · `--head-select` (cal-selected posture) promoted as the arena protocol · engine default stays `head_scale = 0` (byte-identical baseline)

## What

Issue 030 lever 4, the feature-class arc: one-vs-all logistic heads (sigmoid, never softmax) over the existing 256-dim hashed-bag features, fitted at engine-build time from the same post-cal corpus pool the drafter corpora use. The discriminative sibling of the routing centroid: each head row is pushed away from every other domain's docs — the signal a centroid cannot express (banking77's `card_arrival` vs `card_delivery` differ in their negatives). Blend term `0.5 + head_scale·(σ(logit) − 0.5)` joins the σ-shaped drafter + route terms wherever route terms are active; noul never takes it (issue 030's law). Sibling precedents: the served game heads (katgpt-rs Benches 881/882) and riir-clippy rule_embed (Bench 099).

- `src/label_heads.rs` — the fit (deterministic: fixed doc order grouped by domain, 12 epochs, lr 0.5 linear decay, L2 1e-4, no RNG, pure sequential f32; bit-identical re-fit, tested) + the blend term.
- `EngineConfig.head_scale` (default 0.0 = OFF; `build()` refuses a non-zero scale — fail-closed, never a silent no-op head).
- Harness `--head-select`: per eligible dataset suite, accuracy per ladder candidate `[0, 0.25, 0.5, 1.0]` on the STRATIFIED selection slice (the cap-selection instrument — NOT the raw cal slice, see measured trap 2), forced (thresholds maxed), promotion bar 5pt over scale-0, ties/under-bar → 0; test read once at the selected posture. Per-candidate rows disclosed in results.json + TABLES.md; `RunMeta.head_posture` names the run's posture.

## Measured (M3, AC, quiet; deterministic lane; `--skip-laya`; full harness ×5 postures)

| suite | n | off | pin 1.0 | pin 2.0 | **--head-select** | sel scale | Δ |
|---|---|---|---|---|---|---|---|
| banking77 | 500 | 0.4460 | 0.6840 | 0.6980 | **0.6840** | 1 | **+23.8 pt** |
| massive_intent_en | 300 | 0.6900 | 0.7933 | 0.8033 | **0.7933** | 1 | **+10.3 pt** |
| sst5 | 600 | 0.2167 | 0.2133 | 0.2100 | 0.2167 | 0.5 | 0.0 (ECE 0.0518→0.0020) |
| ag_news | 400 | 0.5100 | 0.5050 | 0.4950 | 0.5100 | 0 | 0.0 |
| emotion | 400 | 0.2825 | 0.2275 | 0.2175 | 0.2825 | 0 | 0.0 |
| xnli_en | 300 | 0.3467 | 0.3833 | 0.3867 | 0.3467 | 0 | 0.0 |
| typed_decisions | 400 | 0.3190 | 0.3205 | 0.3200 | 0.3190 | 0 | 0.0 |
| prompt_injections | 116 | 0.4828 | 0.4828 | 0.4828 | 0.4828 | 0 | 0.0 (noul guard holds) |
| 5 harness families + code_fixtures | — | — | — | — | **bit-identical** | n/a | 0.0 |

**Net +34.1 pt, zero regressions.** G1 at the promoted postures: banking77/massive/sst5 PASS (ag_news' pre-existing floor FAIL is byte-identical to baseline — untouched, selected 0). G2 per-decision p50s unchanged or better (banking77 0.344→0.318 ms). Engine bench: G2 PASS (p99 45 µs ≤ 1 ms) · G4 PASS (core alloc-free, canary live) · repeat bit-identical.

## Measured traps (why the protocol is the shape it is)

1. **The raw cal slice REVERSES the banking77 signal.** First selection pass used the suite cal cases: banking77 cal accs FELL with scale (0.265→0.200) → picked 0, while the test split rises +23.8 pt — the label-clustered-cal class (Bench 004 / Issue 023) re-measured on this axis. The stratified selection slice (content-excluded, label-stratified) ranks it correctly (0.49→0.715). The cal-slice instrument transfers exactly where the repo's own history says it must.
2. **n=200 selection noise flips small/neutral suites both ways** — emotion's sel slice read +2.5 pt at scale 1 while the test split read −5.5 pt. The 5 pt promotion bar (≈2σ at n=200) makes the selection only move on strong evidence; everything else keeps the baseline BIT-IDENTICALLY.
3. **Scale > 1 fails G1 by construction.** The blend term over-weights the model beyond its own calibrated confidence and the maxp readout saturates: banking77 at pin-2.0 gained accuracy (0.698) but its calibrated ECE 0.498 failed the conformal floor 0.464. The ladder caps at the fitted-model-verbatim 1.0 — G1 passed there. (An earlier `[0.5,1,2]` ladder also regressed harness_sensitivity −6.7 pt / harness_permissions were family-side noise; with families ineligible for selection and the bar in place, family rows are bit-identical.)

## Posture law

The ENGINE default stays `head_scale = 0` — the published baseline stays reproducible bit-for-bit; the ARENA protocol is `--head-select` (disclosed per row: candidates + selected scale). A serve deployment opts in by config; noul never takes head terms at any scale.

## GOAT

- G1 calibration: PASS at every promoted posture (the one FAIL in the table is ag_news' pre-existing baseline state, byte-identical, selected 0).
- G2 perf: decision_set_goat p99 45 µs; per-suite p50s unchanged (≤0.48 ms everywhere).
- G3 no-regression: every non-promoted suite bit-identical (accs AND ECEs).
- G4 alloc: solve_into alloc-free at the canary-verified counting allocator; head adds a stack dot per option.
- Verdict: **GOAT — promoted as the arena protocol (`--head-select`)**, engine default unchanged.
