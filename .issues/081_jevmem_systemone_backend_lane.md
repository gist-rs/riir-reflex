# Issue 081 — Jev-Mem System-One backend lane (reflex as a memory-control decision backend)

**Status:** OPEN — measurement lane (Research 310, ndb; arXiv:2609.23986). Decision-layer cells are free + local; LoCoMo end-to-end needs an answer model (owner-gated spend, BYO-key posture like the Proposal-014 lane-3 shape).

Jev-Mem (`libingzheren/Jev-Mem`, MIT, @ `7ab0c73c`) is the first published system that runs an ENTIRE memory lifecycle on a typed decision engine — and its System-One backend is **pluggable**: TypeSafe Jev API is the default, local Laya was added 2026-09-27 in one commit (`memory/laya_backend.py`, `config/laya_mem.json`). The backend interface is small: `client.evaluate(operation, state, questions{Noul|Choice}, mock_values, budget)` — every question maps 1:1 onto `decision_wire` (`noul` = Noul, `choice` = Choice with criteria). Their question inventory (admission/typing/relations/consolidation/routing/stopping/traversal, ~25 noul + 2 choice) is a ready-made **memory-control decision pack**.

Why it matters for reflex: the arena thesis extends from datasets to APPLICATIONS — a live third arena where the modelless engine + calibration surface competes for the controller slot of a published memory system. Their paper's honest caveat ("model-reported values are not assumed to be calibrated probabilities") is exactly our `SigmoidGateCalibrator` + conformal-floor differentiator.

## Tasks

- [ ] T1 Backend adapter: `AgentJevLane`-style HTTP seam → a `JevMemBackend` mapping `Noul/Choice → decision_wire noul/choice` (batched per operation; `generated_tokens: 0` posture holds — we never generate). Serve against their offline demo first (`python -m jev_mem.demo`, no keys, deterministic mock baseline = the F lane must beat the mock).
- [ ] T2 Decision-layer cells (free, local): routing / stopping / traversal decision quality vs (a) their deterministic `mock_values` baselines, (b) TypeSafe Jev API (BYO `TYPESAFE_API_KEY`, visitor-key posture — we never pay), (c) Laya local (`config/laya_mem.json`). Determinism strip per the Bench-127 law (byte-identical re-runs).
- [ ] T3 Calibrated stopping: fit the sufficiency/continue thresholds per-suite (sigmoid-calibrated) vs their raw 0.95/0.15 — measure stop-round count + downstream evidence quality at equal accuracy; Report-the-Floor posture on the stopping decision.
- [ ] T4 (owner-gated) LoCoMo end-to-end with an OpenAI-compatible answer model; publish cells in the landscape table with the answer-model cost + protocol footnotes (their scorer is reference-aware at best-of-n>1 — run `--best-of-n 1`).
- [ ] T5 If the lane holds: publish the "memory-control decision pack" as reflex question templates (the pack IS the funnel artifact for Jev-Mem-class consumers).

## Provenance / licensing notes

- Their repo is MIT (clean to read/clone for the verdict; cloned at the pinned sha, removed after — Research 310 records the pin).
- We contribute NOTHING upstream; a backend that speaks their interface over loopback HTTP keeps license surfaces clean either way.
- Prose/games/benchmark text never copied; `Jev`/`Jev-Mem` named only to compare (the standing trademark caveat).
