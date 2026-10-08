# Issue 077 — Option-permutation spread probe (lane-agnostic)

**Status:** Open — actionable from `.research/009_LiquidAI_d1_Decision_Models.md` (2026-10-08). RUNNABLE TODAY: it needs no d1 server — it runs against the lanes that already exist. The `--d1` lane (issue 078) consumes this probe once it lands.

## Context

LiquidAI's d1 recipe names **"shuffling answer options"** among the levers that mattered
more than advanced techniques (blog 2026-10-07) — order sensitivity is a live failure class
in the decision-model category. No order-sensitivity probe ships in our harness: laya
renders options in fixed order, the modelless engine is permutation-invariant by
construction (and should ASSERT it once as the control), and the pooled-state cross-marker
attention path is the only learnable position-sensitivity surface we run. A lane reading
position, not content, will swing under permutation — the probe measures exactly that.

## Tasks

- [ ] **T1 — probe in the harness (lane-agnostic):** for each scored case, re-score under K
      deterministic option orderings; report per-lane **median + max top-pick spread**;
      gate: median swing ≤ 2 pt. Ship a canary case whose gold is order-biased that MUST
      red an order-biased lane (the probe proves it can fire).
- [ ] **T2 — run against the lanes that exist today:** the modelless engine (control —
      assert invariance once, byte-identical across orderings), `--drex` (loopback server
      posture, the closed lane's runbook), `--agentjev` (hosted, det ✗ wobble class —
      disclose), and the laya lane where renderable (pooled-state cross-marker attention is
      the surface of interest for riir-train 471 item 4).
- [ ] **T3 — record as a bench row:** posture columns (server/serving dtype, box state via
      `scripts/bench_preflight.sh`, split hash); the results feed riir-train
      `.research/471_D1_PostTraining_Recipe_vs_Typed_Head.md` item 4 (option-shuffle
      AUGMENTATION is only worth its LENC-cache regeneration if this probe shows swing).
- [ ] **T4 — dependency for issue 078:** the `--d1` lane mounts the probe like every other
      systemone lane (d1's own recipe names option shuffling — their model is the
      probe's most interesting first subject after our own lanes).
