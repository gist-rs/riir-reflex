# Issue 054 — openjev lane candidate (Open-Jev-27B-v1.1, hardware-gated) + the prefix-state handoff serving lead

**Status:** OPEN — Part 1 lane candidate BLOCKED on GPU memory (re-arm recorded; owner call 2026-10-02: do NOT add to the arena — the model is too huge for the fleet); Part 2 prefix-state handoff lead REFUTED 2026-09-30 for laya (measured — the laya encoder is ModernBERT, bidirectional, NOT GDN; verdict + probe below); Part 3 TECHNIQUE FOLLOW-THROUGH LANDED 2026-10-02 (`.research/006` — the four open-jev-fast serving techniques mapped; the transferable one IMPLEMENTED in the modelless engine, bit-identical; GDN candidate 1 verdict = absorbed by the v3 chunked prefill, residual named as the measured-GOAT re-arm)

Filed from the open-jev-fast distill (riir-clippy Research 221 / Plan 185 / queue Batch 189, 2026-09-29). Source pins: `lyuyiqi/open-jev-fast @ c52b8bb9` (MIT) + upstream `Zefan-Cai/Open-Jev @ 3308a15` (MIT, wire schema verified this session).

## Part 1 — the `openjev` comparison-lane candidate (BLOCKED)

**What:** serve Open-Jev-27B-v1.1 (the 27B member of the typed-decision model family our `agentjev` lane samples — but a DIFFERENT model: Open-Jev-27B on Qwen3.8-27B vs AgentJev-0.6B on Qwen3-0.6B) as a new lane on our `typed_decisions` suite. Open question the cell answers: does the 27B Jev model beat AgentJev-0.6B **0.7715** / laya-typed **0.7445** on OUR gold labels (bench 039 posture)?

**Wire: PINNED compatible.** Open-Jev's server takes `POST /v1/{inference,systemone,api/jev}` with `{"state": ..., "questions": [{"id", "type": "choice"|"score"|"noul", "question", ...}]}` — the exact `decision_wire` vocabulary. Deltas from the agentjev lane's `build_question`: noul sends `type:"noul"` (agentjev: `"boolean"`); choice options are a descriptions LIST keyed positionally (agentjev: an OBJECT); score = 2–10 descriptive levels (same as agentjev's `levels`). Answers return per-question `{type, choice|noul|score, probabilities, confidence}` — `map_answers` maps directly. A lane is a thin translation shim over `lanes/agentjev.rs`'s shape (the openthai-lane pattern).

**Serve backend options (either works; open-jev-fast is 6.1× faster on JevBench mean latency):** the original `jev.server` (PyTorch+FLA) or `open-jev-fast` (fused CUDA + prefix tree + CUDA Graphs, drop-in same request format). bf16 only — the source measured FP8 (14.6→7.8 ms GEMM) and REJECTED it on numerics; no accepted quantized path exists upstream.

⛔ **BLOCKED on hardware — the MODEL, not the backend:** ~98 GiB GPU in bf16 (weights + merged/split copies + graph pool; `nvidia-smi` 100,332 MiB after a JevBench run). Our 4090 = 24 GB. M3 = no CUDA. Re-arm triggers (either):
1. any ≥98 GB GPU window (cloud B300/H200-class session — owner-gated, GPU-hours),
2. an owner-sanctioned quantized serve (would deviate from every published number; the cell would need a quant-posture disclosure like the openthai DTYPE pin).

**Non-goals:** no change to the agentjev lane (their 0.6B stack serves a different model; open-jev-fast cannot serve it); no claim on JevBench accuracy here — our typed_decisions split is the measuring stick (the source's 197/231 is their benchmark, different pool).

## Part 2 — the prefix-state handoff serving lead (REFUTED 2026-09-30, measured)

**Verdict: the open-jev-fast `fla_mode="state"` handoff does not transfer to the laya lane.** The as-filed premise ("laya is GDN-family, riir-infer `deltanet` substrate") was wrong about the architecture: the laya checkpoints are **ModernBERT-large / mmBERT-base** (`encoder_config.json`) — a bidirectional encoder. The op stream has **no causal mask anywhere** (`attention_forward_default`; the only mask is the symmetric sliding-window band `|q − k| ≤ window`, and the config's own doc names the reference's "bidirectional overlay"), and the `deltanet` substrate serves the ternary Bonsai/GDN lane, unrelated to laya. GDN's causal delta-rule recurrence is exactly what makes the upstream handoff lossless — a final prefix state IS the continuation input; ModernBERT has no per-position state a suffix could resume from. Three structural grounds:

1. **Bidirectional coupling.** A shared state span's hidden rows are functions of the per-question head span; encoding the state once and reusing it across a case's questions changes the math at the first full-attention layer (layers 0, 3, 6, … — `global_attn_every_n_layers = 3`).
2. **Per-question positions.** `build_sequence` renders `[CLS] {t} question: {ins} [SEP] [MASK] opt… [SEP] {state} [SEP]` — the state sits AFTER the per-question head span, at a different RoPE offset in every question's sequence (head lengths differ). A single shared encode cannot reproduce the positional geometry at all.
3. **Per-question truncation.** The state is truncated to `room = max_len − ids.len() − 1`, which varies per question — the "shared prefix" is not guaranteed identical tokens across questions.

**Measured** (`riir-infer` `crates/riir-infer-laya/tests/prefix_state_coupling.rs`, commit `ff5de37`, CPU posture, release + debug agree, 2026-09-30): two distinct choice questions against ONE shared case state on the real **typed** checkpoint — the shared state span (**283 of 317/318 tokens: the state is ~89% of each sequence, so the as-filed ~5× prize was real**) drifts **5.1e2** between the two forwards — five-plus orders above the **1e-5** CPU GEMM reduction-order budget the packed-equivalence gate prices. Synthetic-geometry arm: **2.7e0** over a 16-row shared prefix, with a bit-identical determinism control (the drift is attention coupling, not numerics). The probe is **two-sided**: a reading at or below the budget FAILS the test, so an architecture change that ever makes the span independent re-opens this record mechanically instead of silently.

**Consequence:** no exact prefix-state win exists for the laya lane; the packed per-question pass (reflex issue 020 T5) remains the exact floor for case serving. The lead's SHAPE stays valid where the serving model is causal (GDN/KV-cache decoders — the league lane); a re-filing should target that surface, not laya. Not urgent stands: lanes are measurement surfaces, not production serving.

## Part 3 — the serving-technique follow-through (LANDED 2026-10-02, owner call: techniques + impl only, no model load, no bench, no arena lane)

Owner decision on this issue: **do not add the openjev lane to the arena — the model is too huge**; distill the serving techniques and implement what transfers instead. Done, recorded in [`.research/006_open_jev_fast_serving_techniques_mapping.md`](../.research/006_open_jev_fast_serving_techniques_mapping.md):

1. **Prefix-tree root sharing → IMPLEMENTED in the modelless engine** (`src/engine.rs`, `solve_sample_into`): every state-alone quantity the pipeline re-derived once per QUESTION (the state embed, the cosine route terms, the per-domain head blends, the count-table token view + in-scores shared by the route-nb / noul / option-conditioned arms, the ridge token view + scores) is now derived once per CASE. Bit-identical by construction (pure functions of the same bytes — the bag-semantics case of Part 2's law: no attention coupling in a hashed bag). Regression gate: `engine::tests::per_question_slots_are_independent_of_their_neighbors` (mixed 4-question case == four single-question solves, bit for bit). No model, no bench — the latency reading rides the next natural seat run.
2. **GDN chunked prefill (candidate 1) → ABSORBED**: `riir-infer-gpu`'s v3 design already hoists the ENTIRE state-independent set for ALL chunks into parallel pre-pass kernels (`decay`/`gram`/`tinv`) before the serial kernel — a stronger form of the source's intra-kernel two-chunk overlap. Residual (the cross-chunk state-dot recurrence) = the measured-GOAT re-arm on the 4090 prefill row, unchanged from Research 221's detection-only filing.
3. **Tree attention bit block-skip (candidate 2) → NOT TRANSFERABLE** (league is single-stream; our tree verify is linear-attention forward substitution, not softmax attention). **bf16 activation LUT (candidate 3) → N/A** (engine is f32 scalar sigmoids; the GDN lane's activations are already fused — the source's own anti-data says the LUT loses where folding removed the op). **Shape-bucketed graph-cache LRU (candidate 4) → ALREADY ADJUDICATED** (riir-infer Issue 967 capture-ladder status-quo; our capture-once-at-`capture_p()` posture is deliberate).

## References

- riir-clippy `.research/221_open_jev_fast_cuda_gdn_serving_distill.md` — full distill verdict
- reflex bench 039 (the agentjev gold-label row this lane would join), bench 014 (typed zip semantics)
- League context: the same Qwen3.8-27B GDN+FA architecture is our perf-league lane; the GDN scheduling lead is filed detection-only in the riir-clippy queue (riir-ai/riir-infer owning sessions)
