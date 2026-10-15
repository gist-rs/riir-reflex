# Bench 148 — Jeva-4B measured cells on our split (reflex Issue 090 B1/B2 + riir-train 632 T2 lane)

**Status:** COMPLETE 2026-10-16 — the first decision-native PAPER model measured on our
split: gold-label accuracy + ECE over the full 9-suite board, the perm-probe (B2's
0-flip floor), and the null-probe family A1/A2/A3 at the `JEVA_ALLOW_NULL=1` posture.
Posture: `LIQIIIII/Jeva-4B` @ snapshot `490ba499b4c0cc26d2d0ba55429d12093f52225e`, their
code `LiQiiiii/Jeva` @ `f231c9ac31ba72f2ca4ca8aac662b279f3438fdb` (Apache-2.0), OUR
stdlib reference server (`scripts/jeva_server.py`), bf16 (the checkpoint config dtype,
never assumed — /health discloses), MPS device on the M3 Max, `cache=hybrid`,
`lora_merged=true`, allow_null=false (comparison cells) / true (A1 probe arm). Weights
license UNDECLARED (train 632 T0) — measurement posture, adoption fail-closed.

## Provenance (Issue 021)

- `PROVENANCE: power=AC Power load=4.13 swap=5585.00M canary=124.3us/best5 powermode=2(high)` (bench_preflight, session start; High Power mode = the performance posture)
- Machine: M3 Max 16-core, 64 GB, macOS 26.6.2. MPS device (their CUDA-only default
  declined — `JEVA_DEVICE=mps`). The qwen3_5 hybrid backbone (3 GDN linear-attention + 1
  full attention per 4 layers, 32 layers) runs transformers' reference PyTorch
  fallback for `causal_conv1d` + `chunk_gated_delta_rule` (fla/triton unavailable on
  macOS — "correct but much slower", their own loader's disclosure). Accuracy cells are
  contention-insensitive (deterministic per smoke probe: identical probabilities across
  repeated rounds); latency cells are shared-GPU MPS numbers, never a CUDA-parity claim.
- Datasets: `.raw/datasets` (canonical, seeded slices — slice-integrity fnv1a64 pins in
  results.json).

## The board (gold-label accuracy, harness `--jeva --skip-laya`)

| suite | jeva | modelless (raw, same run) | jeva ECE(maxp) | p50 ms/case | their published |
|---|---|---|---|---|---|
| typed_decisions | **0.7680** | 0.2940 | 0.1711 | 4108 | 78.30 (their split/protocol — cross-protocol, never same-aggregate) |
| massive_intent_en | **0.9200** | 0.6100 | 0.0382 | 173 | — |
| banking77 | **0.9160** | 0.3940 | 0.0347 | 484 | — |
| semantic_defects | **0.9118** | 0.1863 | 0.1060 | 174 | — |
| xnli_en | **0.8800** | 0.3400 | 0.0850 | 303 | — |
| ag_news | **0.7475** | 0.4050 | 0.0453 | 314 | — |
| prompt_injections | **0.6897** | 0.4828 | 0.1487 | 183 | — |
| emotion | **0.5225** | 0.2750 | 0.0560 | 177 | — |
| sst5 | **0.4667** | 0.2017 | 0.0671 | 180 | — |

Artifacts: `.benchmarks/148_jeva_measured/typed/` (typed first cell) ·
`board/` (7 suites) · `board_barelabel/` (the 3 bare-label suites after the sequence-arm
fix below). Server latency sum (typed): 1622.7 s over 2000 questions; `output_tokens`
is their ALWAYS-ZERO answer-slot law made visible.

### Posture comparison (CROSS-PROTOCOL, the B5 law — different splits/protocols; context only)

Our published served postures (instinct board) vs jeva on the same suite NAMES:
massive 0.9200 vs H2 0.8400/ENC — · banking77 0.9160 vs H2 0.8540 · xnli 0.8800 vs ENC
0.8600 · typed 0.7680 vs encoder-v2 0.7550/AgentJev 0.7715 · sst5 0.4667 vs ENC 0.5267
· emotion 0.5225 vs A0 0.8850 (**Jeva's largest show-loss — our modelless floor beats
it outright there**) · prompt_injections 0.6897 vs A1 0.8534 · ag_news 0.7475 vs ENC
0.9475. Shape: Jeva leads on massive/banking77/xnli, trails on ag_news/emotion/
prompt_injections, is mid on typed/sst5. Their confidence beats the conformal-naive
floor on ECE on typed (choice 0.1735 / score 0.1995 vs floor 0.2082) — the readout is
honestly calibrated there.

## The probes

### perm-probe (B2's 0-flip floor) — `.benchmarks/148_jeva_measured/perm/`

**jeva: 0 flips across all 6 suites (255 slots, K=5 orderings) — 0 RED cells.** Their
paper's 0.00% order-flip row (mean TV 3.17e-8) reproduces on our split exactly: the
hybrid option cache + shared-denominator softmax is order-invariant in practice. The
modelless control held (median 0.00 pt, untied flips 0; strict byte-identity BROKEN on
exact-tie slots = the disclosed L1-reorder ULP column, never the verdict).

### null-probe A1/A2/A3 (the `JEVA_ALLOW_NULL=1` server posture) — `.benchmarks/148_jeva_measured/null/`

- **A2 (decoy set-preference, Prop 1): jeva 0 flips / 255 slots across all six suites**
  — the independent-scorer invariance their paper proves holds on their own model to
  the probe's resolution.
- **A3 (noul conditional, their A3 readout): `noul_bit_identical: true` on every
  suite** — the paired noul question is bit-identical across decoy arms (also proving
  no cross-request state through their hybrid cache); confidence is **joint-shaped**
  on every suite (median |Δconf| up to 19.5 pt on ag_news at the top arm) — the
  readout reads the diluting joint, exactly the shape their design predicts.
- **A1 (abstention drift, Prop 2's ⚠): their max-based coverage gate DOES drift with
  K, milder than ours**: abstain-rate drift K=0→256: typed +11.7 pt (85.0→96.7%) ·
  ag_news +12.5 pt (non-monotone −5.0/+10.0) · xnli +10.5 pt · emotion +2.5 pt ·
  massive +2.6 pt · banking77 **0.0 pt flat**. Their gate removes the log-K term (Prop
  2) but not the max-of-K drift — now measured on their own model. Baseline abstention
  is HIGH on typed (85% at K=0 — their coverage gate is conservative there).
- Floor leg: computed for the modelless rows only (RED 4/6 = the standing Bench-147
  finding, re-confirmed); **the harness does not yet wire the Report-the-Floor leg for
  comparison lanes** — a recorded gap, not a pass.

## Two wire findings fixed this session (the lane's first live exercise)

1. **Bare-label suites 400'd on their mapping arm.** Their `_criteria` requires
   nonempty mapping descriptions; emotion/banking77/semantic_defects carry bare labels.
   Fix (lane-side, `src/lanes/jeva.rs`): null/empty/value==key criteria ride their
   SEQUENCE arm (texts = keys verbatim) — including the null-probe's augmented arms
   (bare originals + decoys whose value == their own key), keeping presentation
   consistent across K arms (the A2 comparability law). A genuinely mixed real suite
   keeps the mapping arm and their loud 400 (the wire has no faithful mixed shape).
   `jeva_server.py` mirrors the same two-shape law. 22 lane tests; clippy `-D warnings`.
2. **`--suites list` clobbered the sibling's uncommitted `001_phase1_tables` WIP** (the
   harness's default out dir). Regenerated at the same commit `83031d2` with the same
   binary — deterministic tables (typed modelless 0.2940 identical across both my
   runs; seeded slices), only the run-header/box-state lines carry this session.
   Disclosed here as the incident record; the standing fix is personal: NEVER invoke
   the harness without `--out`.

## What rides forward

- **riir-train 632 T2 capture**: `--distill --distill-teacher jeva --distill-out
  .raw/distill_teacher_jeva` LAUNCHED on this session's server (allow_null=0 = joint ==
  conditional, no ρ dilution in teacher targets; RIDT header carries the UNDECLARED
  license note). ~6 T3 suites × full train split ≈ 15 h on MPS (ag_news teacher acc
  0.684 at 2000/20000 rows in the first attempt — consistent with its 0.7475 test cell).
- reflex Issue 090: A1–A3 + B1 + B2 now MEASURED; remaining C1 (low) + C2
  (katgpt-core `set_refine`, do-not-wire-blind — its own set-awareness suite + GOAT
  first).
- research 007 beat-lane: the measured row for the teacher candidate is THIS record
  (0.7680 typed same-wire on our split — the number the distill lane chases).
