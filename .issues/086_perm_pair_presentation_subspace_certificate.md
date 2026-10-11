# Issue 086: Perm-pair presentation-subspace certificate (upgrade the option-ordering probe)

**Status:** Open — P1. Source: `../katgpt-rs/.research/618_SplitJEPA_Invariant_Variant_Block_Identifiability.md` (SplitJEPA, arXiv:2610.12349); primitive lands via katgpt-rs Plan 625 (`basis_split`). This is the decision-serving fusion.

## Problem

The permutation probe answers robustness with a black-box aggregate (K deterministic orderings per question → gate median swing ≤ 2 pt; the control red once found a real engine order-sensitivity defect, so the probe earns its keep). It cannot say WHERE ordering energy lives or HOW MUCH reaches the decision score. The perm pairs are already the exact matched-pair structure SplitJEPA's head consumes (content-identical, presentation-differing): routing them through the hashed-feature embed gives the **presentation-variant basis**, and the leakage metric quantifies how much ordering-flip energy rides into decision scores. Rank-gate honesty: if the embed is already order-insensitive, differences ≈ 0 and the gate correctly refuses (`Unidentified`) — nothing to fix, and the certificate says so.

## Tasks

- [ ] T1 Wait for katgpt-rs Plan 625 Phase 1, then add a `--basis-split` report mode to the harness permutation runner (report-only): rank-gated presentation basis + per-suite leakage readout + agreement residual, on the existing perm-pair generators. No decide-path change.
- [ ] T2 Record per-suite numbers (typed_decisions, banking77, ag_news minimum) under `.benchmarks/` numbering discipline with preflight provenance; include the `Unidentified` postures honestly (they are results, not failures). **Holdout law:** a different SUITE is a cross-CONTENT holdout (content changes, the variation kind does not — both suites vary by option ordering); it is labeled as such, never "cross-family". The true cross-FAMILY arm is a DIFFERENT presentation change: renamed-option-label pairs and whitespace/casing permutations through the same embed — fit the basis on ordering pairs, report the residual on the label/casing pairs.
- [ ] T3 Perturbation certificate on the scorer: for each presentation-basis direction v_k, score(embedding + ε·v_k) vs score(embedding) at ε inside the fp envelope — discrete outputs must be bit-identical, scored outputs within tolerance (the Thm-3 harness; consumes the existing audit rigs).
- [ ] T4 Decide from T1–T3: if a suite shows material ordering leakage, opt-in projection at the embedding→routing seam (one matvec; off-flag bit-identical). Gate: fused-abstain p99 ≤ 1 ms unchanged, ECE + accuracy non-inferior (paired LB95, the instinct standard), perm-spread reduction on the affected suite.
- [ ] T5 Cross-lane rider (cheap): the same perm pairs through the laya comparison lane turn the G5 top-1 gate into a continuous device-drift-direction readout (degenerate-pair purity floor) — only if T1 machinery makes it free.

## Acceptance

Report-only first; any armed seam change carries the T4 gate set; the permutation probe's existing gate and control stay untouched.
