# Issue 014 — engine.rs local sigmoid: delegate to `katgpt_core::exact_sigmoid`
# (substrate-first Mode 2 audit finding)

**Status:** OPEN — filed by the riir-clippy idle-loop substrate-first audit
(2026-09-24 ~02:5x +07), **detection-only** per the skill's discipline (no fix
in the filing commit). The engine lane is sibling-active (X-Reflex-Lane) —
fix at the lane owner's convenience. No behavior claim is made: the delegation
is bit-identical on the reachable input domain.

## The finding

`src/engine.rs:624`:

```rust
#[inline]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
```

Module-local, no in-source rationale comment, no divergence pin. Call sites:
`:437` `sigmoid(dot * self.cfg.route_scale)` (route-term gate, issue 004 T7
blend) and `:458` `sigmoid(s / self.cfg.score_temperature)` (per-option score
normalization).

Meanwhile the SAME file consumes katgpt-core substrate wholesale — imports at
`:47-54`: `Lz4FlexDrafter`, `decision_wire::*`, `CorpusDistanceGate`,
`SigmoidGateCalibrator`, `pick_domain` — and katgpt-core is this crate's ONE
unconditional dep (the repo's own boundary test). The one primitive
re-implemented locally is the sigmoid.

## Classification: DRY violation candidate (the ndb-611 / chain-156 class)

- **Vocabulary translation** (the skill's defense): "confidence gate / score
  normalization" → the substrate ships as `katgpt_core::exact_sigmoid` (the
  bit-stable two-branch reference) + `katgpt_core::sigmoid` (the fast Cephes
  approx). This repo is the decision-engine consumer that substrate was
  extracted FOR (Proposal 014 / Plan 603) — `exact_sigmoid` is ungated and
  always available.
- **Precedents**: riir-neuron-db Issue 611 (4 production sigmoid copies → all
  delegate), riir-chain Issue 156 (delegate + permanent bit-identity pins vs
  frozen legacy bodies), katgpt-rs Issue 870 (`distance_abstain` →
  `exact_sigmoid` + envelope pin).
- **Copy-gate convention (unmet)**: a justified copy needs in-source rationale
  + a divergence-failing test. Neither exists here.
- **Honest history**: engine.rs shipped at repo birth (`31207ec`, 09-22 00:18)
  carrying the local fn; the 09-22→09-23 substrate-first wave read the fresh
  reflex set CLEAN — this filing is the re-read with the sharper question
  (why does an import block consuming five substrate items keep one local
  math fn).

## Why delegation is sound

`katgpt_core::exact_sigmoid`
(`katgpt-rs/crates/katgpt-types/src/simd/activations.rs:336`) is the
two-branch libm form — `x >= 0: 1/(1+exp(-x))`, else `ex/(1+ex)`. That is
**bit-identical to the local body on x ≥ 0**, and the two forms are
bit-identical everywhere above the one-branch form's single-overflow
threshold (x > −88.7, where `exp(-x)` saturates to inf and the local body
returns 0.0 via `1/inf`). The reachable domain is far inside that:
`dot·route_scale` over L2-normalized direction vectors (route_scale default
8 → |x| ≤ 8-class) and `s/score_temperature` over compressed-length deltas.
Only the unreachable deep tail (x < −88.7) diverges: local returns exactly
0.0 via inf; the two-branch returns a representable tiny. Document it, don't
fear it.

## Proposed fix (NOT this commit — detection only)

1. `use katgpt_core::exact_sigmoid;` and delete the local fn (call sites
   unchanged).
2. Add the chain-156-class pin:
   `sigmoid_delegation_matches_frozen_legacy_body` — bit-identity over the
   reachable domain + an envelope assert (not equality) documenting the
   x < −88.7 tail class.
3. Re-run G2 (`decision_set_goat`) + G4 — expected neutral (same libm
   `exp` call, one extra branch, zero alloc delta).
4. No golden/output change expected: the harness re-fits calibration
   thresholds at runtime, and on the reachable domain the delegation moves
   zero bits.

## References

- Substrate: `katgpt-core::exact_sigmoid` (lib.rs:54) →
  `katgpt_types::simd::activations.rs:336`.
- Skill: `katgpt-rs/.agents/skills/substrate-first/SKILL.md` (Mode 2 filing
  discipline: issue BEFORE fix, vocabulary translation included).
- Sibling lineage: ndb 611 → chain 156 → katgpt-rs 870 → this.
