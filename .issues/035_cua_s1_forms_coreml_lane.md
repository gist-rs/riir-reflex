# Issue 035: cua-s1-forms CoreML arena arm — the CoreML/ANE serving posture of the System-One family

**Status:** OPEN — T1 DONE 2026-09-26: lineage HOLDS by contract (Cua-trained jev-like System-One option scorer, MIT; FluidInference converted only) — exit clause does NOT fire; T2–T5 remaining.
**Date:** 2026-09-26
**Filed from:** riir-infer [`.research/001_FluidInference_OnDevice_Audio_Stack.md`](../../riir-infer/.research/001_FluidInference_OnDevice_Audio_Stack.md) (F4)
**Extends:** `.research/002_agentjev_system1_landscape.md` (which already flags
"CoreML/ONNX ports" as ecosystem arms) + `.issues/025` (the AgentJev lane pattern)
**Sources:** FluidAudio `@ 762baf6733ca0f0dbeb1ce335363fc75066bd2c9` —
`CuaS1FormsManager` (Documentation/API.md §Decision Scoring) ·
HF `FluidInference/cua-s1-forms-coreml` · mobius
`models/computer-use/cua-s1-forms` (confirmed present at
`5beb34007656d16349fec757c379a9beae45c4d5`)

## Problem

Research 002's landscape table has three serving postures measured (AgentJev on CUDA
via their HTTP service, laya-typed via our port, modelless in-process) — but the
ecosystem's **CoreML/ANE arm** is unmeasured. FluidInference converted a CUA-S1-forms
decision model to CoreML (2–32 options, 224-byte context truncation, stable softmax,
"scores do not authorize an action" posture) and serves it on Apple silicon. That is
the fourth posture: **their stack serves (macOS/ANE), our Rust harness measures** —
the exact AgentJev lane shape, macOS posture like the metal lane.

Lineage (VERIFIED 2026-09-26, T1): cua-s1-forms is **Cua's** (trycua) independent
"jev-like System One" option scorer, converted to CoreML by FluidInference — not a
FluidInference-trained model, and **not** a TypeSafe/Jev checkpoint either (the original
hypothesis named the wrong trainer; the family tie is by I/O CONTRACT, not by weights).

## Scope honesty

Their model selects form-filling ACTIONS, not typed_decisions primitives — the suites
may not map 1:1. Start from the **choice-primitive subset** (options=2–32 maps onto
choice questions) or their published forms fixtures; do not force the full 15-suite
lane. If the task mapping is too lossy, the honest outcome is a landscape-table
footnote ("CoreML arm exists, not comparable on our fixtures") — that is still worth
one measurement session.

## Plan

- [x] **T1 — lineage check (a real STOP, not a formality):** HF card for
      `cua-s1-forms-coreml` — who trained it, what benchmark it publishes, license.
      **Exit clause:** if the card does NOT tie it to the S1/Jev System-One family,
      this issue CLOSES as a landscape footnote ("a CUA decision model exists in
      CoreML serving; not comparable on our fixtures"), the `Extends:` framing above
      is dropped from any record that survives, and no lane is filed. The Extends
      framing is only valid while the lineage holds.
      **DONE 2026-09-26 — the lineage HOLDS (by contract), exit clause does NOT fire.**
      Read both cards: HF `FluidInference/cua-s1-forms-coreml` (MIT, `base_model:
      cua-ai/cua-s1-forms`, `base_model_relation: quantized`, "No training was
      performed") and upstream HF `cua-ai/cua-s1-forms` (MIT; tags `jev`,
      `system-one`; card @ `f54adbf447f4ca6ec259f529ee3f2e3e09f8cc71`).
      - **Who trained it:** Cua (trycua, `libs/cua-s1`), NOT TypeSafe — the upstream
        card self-describes as "a small, jev-like ('System One') one-pass option
        scorer … the same input/output contract as TypeSafe's Jev" and states "Not
        calibrated with TypeSafe's RLCD method — an independent research checkpoint,
        not a reproduction of Jev." So the System-One tie is the CONTRACT (one
        forward pass → one probability per supplied option), which is exactly the
        axis the arena measures; the `Extends:` framing stands with that wording.
      - **Architecture:** 706,048 params; byte-level embedding + 2-layer Transformer
        (width 128, 4 heads) over context and each option; jevlike `AttentionHead`
        readout; softmax over live options. Portable CoreML package 1.51 MB, FP16.
        Tensors: `context_ids` i32[1,224], `option_ids` i32[1,32,96], `option_mask`
        i32[1,32] → `logits`/`probabilities` f32[1,32] (UTF-8 bytes + 1, zero pad,
        byte truncation; 2–32 options, overflow rejected).
      - **What it publishes:** synthetic form-disjoint test 99.95% top-1 (upstream
        ~15k rows; FluidInference re-ran the released 24,370-row `test.jsonl` @
        `8273f34778b99ac2e12d9f6e7d57dad99ae20845` → 24,359/24,370 = 99.9549% on
        PyTorch CPU AND both CoreML exports, same 11 errors, all `fill`-for-`skip`);
        196-decision real demo 100%; shuffled-context control 37%; vs hosted
        `jev-latest` zero-shot 99.7% vs 83.6% (vendor numbers — reference only, the
        Research 002 protocol-footnote grade). Latency (vendor, M5 Pro, incl. Python
        call overhead): CoreML CPU+ANE p50 ≈ 0.90–1.00 ms, p95 ≈ 0.94–1.13 ms.
      - ⚠ **Strict numerical conversion parity FAILS on the full split** (their own
        report: 11 rows > 0.005 abs prob error, max 0.0205; argmax parity 24,370/24,370
        holds). Consequence for T4: publish ACCURACY/argmax and latency; never quote
        their probabilities as calibrated, and any ECE column carries that footnote.
      - **Scope consequence (sharpens §Scope honesty):** the model is a
        form-filling specialist trained ONLY on `TASK/FORM/ELEMENT` contexts with
        `fill <entity>` / `check` / `click` / `skip` options — running it on our 15
        NLU suites is out-of-distribution by construction and would publish noise.
        The comparable fixture is THEIRS: the published `test.jsonl` (and the 196-row
        demo if redistributable), with OUR lanes (modelless + laya typed) answering
        the same supplied-option choice questions zero-shot. That is the honest cell:
        a 1.5 MB trained specialist on ANE vs our zero-shot engines on its home task.
- [ ] **T2 — serving path:** their `CuaS1FormsManager` is Swift — reflex is Rust.
      Options: (a) tiny Swift CLI subprocess (their `fluidaudiocli` shape), (b) score
      via `coreml-native` once riir-infer `.issues/015` T2 answers external-bundle
      loading, (c) their `fluid-server` HTTP (WIP). Latency = round-trip per the
      laya-python measurement law.
- [ ] **T3 — fixture mapping:** choice-subset or forms fixtures; pin the mapping in
      the lane module so the comparability claim is explicit.
- [ ] **T4 — measure:** accuracy + p50/p95 on the M3 (ANE posture), added to the
      Research 002 landscape table as a new row with the serving posture labeled.
- [ ] **T5 — lane hygiene:** macOS-only flag posture (metal-lane pattern), honest
      degradation when the model/CLI is absent, never in the default run.

## Gates

- G1: fixture mapping pinned + disclosed in every published table row.
- G2: latency provenance line (box state) beside every number.
- G3: absent-model posture = loud skip, never a fabricated row.

## References

- `.research/002_agentjev_system1_landscape.md` — the landscape + protocol notes
  (their accuracy is teacher-argmax agreement; footnote-grade protocol differences).
- riir-infer Research 001 §F4 — the distill context.
