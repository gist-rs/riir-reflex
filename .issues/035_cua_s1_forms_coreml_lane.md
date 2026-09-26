# Issue 035: cua-s1-forms CoreML arena arm — the CoreML/ANE serving posture of the System-One family

**Status:** OPEN
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

Lineage hypothesis (UNVERIFIED, check the HF card first): cua-s1-forms is the
TypeSafe-Jev/agent-jev ecosystem's computer-use S1 model, converted by FluidInference
— not a FluidInference-trained model.

## Scope honesty

Their model selects form-filling ACTIONS, not typed_decisions primitives — the suites
may not map 1:1. Start from the **choice-primitive subset** (options=2–32 maps onto
choice questions) or their published forms fixtures; do not force the full 15-suite
lane. If the task mapping is too lossy, the honest outcome is a landscape-table
footnote ("CoreML arm exists, not comparable on our fixtures") — that is still worth
one measurement session.

## Plan

- [ ] **T1 — lineage check (a real STOP, not a formality):** HF card for
      `cua-s1-forms-coreml` — who trained it, what benchmark it publishes, license.
      **Exit clause:** if the card does NOT tie it to the S1/Jev System-One family,
      this issue CLOSES as a landscape footnote ("a CUA decision model exists in
      CoreML serving; not comparable on our fixtures"), the `Extends:` framing above
      is dropped from any record that survives, and no lane is filed. The Extends
      framing is only valid while the lineage holds.
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
