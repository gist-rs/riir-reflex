# Issue 104: Learnability-gate class for the synth-corpus lane (baseline-margin filter)
**Status:** Open — poc/measurement task; cheap frozen read, honest null is a live outcome; source: katgpt-rs `.research/640_RT_J_Relational_Context_Recipe.md` (arXiv:2610.17867 RT-J, NeurIPS 2026).

## Why

The corpus-synthesis lane has measured gates on two axes: **teacher consistency** (agreement VETO) and **distribution** (density gates — and Benches 120/121/122 measured UNGATED beating every gate strength, which is the controlling internal record). RT-J's corpus pipeline (THE JOIN: 639 real DBs, 6,023 LLM-generated forecasting tasks) adds a third axis we have never measured: **learnability-vs-cheap-baselines** — each generated task is scored by margin over trivial historical baselines (entity-mean, last-value persistence); the corpus keeps tasks that are learnable AND hard-but-non-trivial (so too-easy rows that a trivial solver already aces carry no signal), with easy/medium/hard difficulty tiers. For their *pretraining* corpus the filter was XGBoost-based; the modelless transport of the idea is the **trivial-baseline margin score itself**, which is closed-form.

For a decision-suite synth corpus the analog of "trivial solver" is per-question heuristics: majority-class / prior-frequency pick, longest-option, lexical-overlap-with-state pick. Questions a trivial heuristic solves at ceiling are the too-easy pole (no signal for the engine to differentiate); questions where the margin sits inside a challenge band are the hard-but-keepable tier.

This issue does NOT assume RT-J transfers. Their filter served gradient-descent pretraining; ours serves ICL retrieval. The null (learnability-gated corpus no better than the ungated sealed artifact) is a genuinely live outcome and would close the gate class with a measurement.

## Scope

1. Score every question in the existing sealed SYN v2 artifact (read-only — the artifact is the corpus of record) with 2–3 closed-form trivial-solver margins (prior-frequency, longest-option, lexical-overlap). No teacher calls, no engine changes; CPU-minutes.
2. Build the gated variant: drop questions at the too-easy pole (trivial margin ≥ threshold — sweep 2–3 thresholds, the density-gate p50/p75/p90 shape), optionally tier the rest easy/medium/hard as a corpus sidecar (report-only).
3. One frozen paired V5 read: ungated sealed artifact vs the best learnability-gated variant. Echo gates recomputed both sides; the V5 paired-LB95 gate law unchanged; grammar-not-answers binds (test rows never ingested, leak-scan before the read).
4. Record the verdict in `.benchmarks/` (new number), including the per-tier composition so a future lane can use difficulty tiers even if the gate itself loses.

## Tasks

- [ ] T1. Trivial-solver margin scorer over the sealed artifact (report first: margin histograms per suite — if no suite has a too-easy pole, the gate class is vacuous here and the issue closes with that measurement).
- [ ] T2. Gated artifact construction behind the existing `--corpus-ab` tooling (no new flags unless needed; reuse `--synth-plan`/`--corpus-ab` surfaces).
- [ ] T3. Frozen paired V5 read, echo gates both sides, verdict per the V5 gate law.
- [ ] T4. Bench record + HISTORY.md line; cross-ref research note 640; on a PASS, record whether difficulty tiering survives as a report-only sidecar (usable without gating).

## References

- Research note: `../../katgpt-rs/.research/640_RT_J_Relational_Context_Recipe.md` (M7; §1 corpus pipeline)
- Paper: arXiv:2610.17867 §3.3–3.4 + Appendix D.4–D.5 (task generation, deterministic checker, learnability filter, difficulty tiers)
- Controlling internal record: `.benchmarks/120–122` (ungated wins) — this issue measures the one gate class those benches never tested
