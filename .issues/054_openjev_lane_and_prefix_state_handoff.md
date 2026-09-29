# Issue 054 — openjev lane candidate (Open-Jev-27B-v1.1, hardware-gated) + the prefix-state handoff serving lead

**Status:** OPEN — lane candidate BLOCKED on GPU memory (re-arm recorded); prefix-state handoff lead UNASSIGNED (riir-infer-laya owning session's call)

Filed from the open-jev-fast distill (riir-clippy Research 221 / Plan 185 / queue Batch 189, 2026-09-29). Source pins: `lyuyiqi/open-jev-fast @ c52b8bb9` (MIT) + upstream `Zefan-Cai/Open-Jev @ 3308a15` (MIT, wire schema verified this session).

## Part 1 — the `openjev` comparison-lane candidate (BLOCKED)

**What:** serve Open-Jev-27B-v1.1 (the 27B member of the typed-decision model family our `agentjev` lane samples — but a DIFFERENT model: Open-Jev-27B on Qwen3.8-27B vs AgentJev-0.6B on Qwen3-0.6B) as a new lane on our `typed_decisions` suite. Open question the cell answers: does the 27B Jev model beat AgentJev-0.6B **0.7715** / laya-typed **0.7445** on OUR gold labels (bench 039 posture)?

**Wire: PINNED compatible.** Open-Jev's server takes `POST /v1/{inference,systemone,api/jev}` with `{"state": ..., "questions": [{"id", "type": "choice"|"score"|"noul", "question", ...}]}` — the exact `decision_wire` vocabulary. Deltas from the agentjev lane's `build_question`: noul sends `type:"noul"` (agentjev: `"boolean"`); choice options are a descriptions LIST keyed positionally (agentjev: an OBJECT); score = 2–10 descriptive levels (same as agentjev's `levels`). Answers return per-question `{type, choice|noul|score, probabilities, confidence}` — `map_answers` maps directly. A lane is a thin translation shim over `lanes/agentjev.rs`'s shape (the openthai-lane pattern).

**Serve backend options (either works; open-jev-fast is 6.1× faster on JevBench mean latency):** the original `jev.server` (PyTorch+FLA) or `open-jev-fast` (fused CUDA + prefix tree + CUDA Graphs, drop-in same request format). bf16 only — the source measured FP8 (14.6→7.8 ms GEMM) and REJECTED it on numerics; no accepted quantized path exists upstream.

⛔ **BLOCKED on hardware — the MODEL, not the backend:** ~98 GiB GPU in bf16 (weights + merged/split copies + graph pool; `nvidia-smi` 100,332 MiB after a JevBench run). Our 4090 = 24 GB. M3 = no CUDA. Re-arm triggers (either):
1. any ≥98 GB GPU window (cloud B300/H200-class session — owner-gated, GPU-hours),
2. an owner-sanctioned quantized serve (would deviate from every published number; the cell would need a quant-posture disclosure like the openthai DTYPE pin).

**Non-goals:** no change to the agentjev lane (their 0.6B stack serves a different model; open-jev-fast cannot serve it); no claim on JevBench accuracy here — our typed_decisions split is the measuring stick (the source's 197/231 is their benchmark, different pool).

## Part 2 — the prefix-state handoff serving lead (UNBLOCKED, latency, accuracy-neutral)

reflex's `agentjev` lane already batches every question of a case into ONE request precisely because the case state is shared (their several-decisions-at-once pattern). OUR laya lane re-encodes the shared case state once PER QUESTION on `typed_decisions` (5-question cases → ~5× redundant prefix encoding over the lane's HTTP round trips).

open-jev-fast's `fla_mode="state"` path (fastmodel.py / server.py `EntryState`) computes the shared prefix once and hands its **final GDN recurrent state** to each candidate — EXACT, not approximate (the linear-attention recurrence is sequential; unlike the suffix-replay class, no reconstruction error). laya is GDN-family (riir-infer `deltanet` substrate), so the same handoff shape applies to `riir-infer-laya` serving: encode the case state prefix once, feed its final state + per-question suffixes.

**Value:** typed_decisions lane latency (comparison-cell cost, not accuracy — G5 parity and the published accuracy rows are unaffected). NOT urgent: lanes are measurement surfaces, not production serving. Owner of the work: the riir-infer-laya substrate session (reflex consumes). Reference: `open-jev-fast @ c52b8bb9` `src/fastmodel.py::forward_tree` + `src/server.py::EntryState`.

## References

- riir-clippy `.research/221_open_jev_fast_cuda_gdn_serving_distill.md` — full distill verdict
- reflex bench 039 (the agentjev gold-label row this lane would join), bench 014 (typed zip semantics)
- League context: the same Qwen3.8-27B GDN+FA architecture is our perf-league lane; the GDN scheduling lead is filed detection-only in the riir-clippy queue (riir-ai/riir-infer owning sessions)
