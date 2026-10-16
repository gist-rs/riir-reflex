# Bench 150 — Plan 627 T3 GOAT: `set_refine` measured NEGATIVE on the real board (G1 fail, monotone in γ)

**Status:** RECORD — negative landing (the Bench-109 shape: the lane stays opt-in with the measured record; promotion moot).

- Plan: `../katgpt-rs/.plans/627_set_refine_operator.md` (T3) · Issue: `.issues/090` §C2
- Posture: reflex `t2-setrefine` @ 63ccf78 (T2 seam + `--set-refine-ab`), feature `set_refine`, katgpt-core @ 0fb56b269 (T1)
- Box: 4090 (shikuwa), release, `--skip-laya`, datasets = canonical `.raw/datasets`; the LOWO chain (riir-train) was running (GPU-bound; the modelless lane is CPU-only — accuracy rows load-insensitive)
- Protocol: one frozen test read per suite (slice-integrity fnv1a64 pins in the run log), paired A/B — the refined engine rebuilt at the SAME fitted posture + knob, calibrator re-fit on the same cal pairs. The dose knobs were NOT selected cal-side (the plan's working-envelope center γ=1.0 τ=0.5 pre-registered; the γ=0.25 arm is the monotonicity probe, not a tuned pick).

## G1 — flag-on beats flag-off beyond noise: **FAIL**

γ=1.0 τ=0.5 (the T1 S1-fixture envelope center):

| suite | opts | Δ acc | lcb95 | flips | +corr/−wrong | abstain → |
|---|---|---|---|---|---|---|
| typed_decisions (control) | 29 | 0.0000 | — | **0** | 0/0 | 1.000 (unchanged) |
| prompt_injections (control) | 2 | 0.0000 | — | **0** | 0/0 | 1.000 (unchanged) |
| emotion | 6 | +0.0200 | −0.0325 | 92 | 24/16 | 0.330 (unchanged) |
| xnli_en | 3 | −0.0067 | −0.0701 | 17 | 5/7 | 0.557→0.880 |
| sst5 (score by-index) | 5 | −0.0283 | −0.0654 | 351 | 58/75 | 0.582→1.000 |
| ag_news (by-name) | 4 | **−0.1400** | −0.1943 | 162 | 24/80 | 0.393 (unchanged) |
| massive_intent_en (by-name) | 59 | **−0.1233** | −0.1897 | 85 | 5/42 | 0.347→0.747 |
| banking77 (by-name) | 77 | **−0.1340** | −0.1823 | 210 | 7/74 | 0.526→0.974 |

γ=0.25 monotonicity probe (the "too-hot knob" artifact check — damage scales with γ, no reversal):

| suite | γ=1.0 Δ | γ=0.25 Δ |
|---|---|---|
| ag_news | −0.1400 | −0.0350 |
| banking77 | −0.1340 | −0.0140 |
| massive_intent_en | −0.1233 | −0.0067 |
| emotion | +0.0200 | +0.0125 |

**The controls validate the seam, the tranche convicts the mechanism**: typed_decisions (drafter-only — the seam's issue-030 legality scope) and prompt_injections (noul-only) read exactly 0 flips, byte-identical, as designed. Every route-active suite loses, worst where option sets are widest and where route terms carry the most signal — the refinement pulls the state toward the attended option-centroid cloud, and with unit-centroid keys that pull REGRESSES THE STATE TOWARD THE CORPUS MEAN, homogenizing exactly the geometry the route terms discriminate with. The collateral: refined confidences collapse the calibrator on wide sets (abstain 0.526→0.974 banking77). The one positive point (emotion +2.0, lcb95 −0.03) is not beyond noise and shrinks toward zero with γ.

## G2 — p99 ≤ 1 ms armed: **PASS at bench scale, with a scaling caveat**

`decision_set_goat` (`set_refine armed` posture, k=2 by-name): p50 362 µs · **p99 454 µs ≤ 1000 µs** (default posture p99 92 µs). Caveat, disclosed: the cost is the kernel's identity-projection matmuls — O(n·d²) per refine (2·(k+1)·256² ≈ 393k mults at k=2 ≈ 90 µs/refine measured). Extrapolated banking77 (k=77): ~9 ms/question — over budget at real width. The identity fast-path (skip the d² copy when w_q/w_k are identity) is a clean katgpt-core kernel_opt candidate — NOT pursued: G1's failure moots any perf work (a faster wrong answer is still wrong).

## G3 — flag-off identity: **PASS**

- gamma=0 byte-identity to the default cfg (seam test `gamma_zero_is_byte_identical_to_default`)
- default-posture compile = the feature compiles to nothing (default clippy/lib/engine_gates green)
- 0-flip control suites at harness scale (typed, prompt_injections) — the whole-suite A/B twin of the G3 gate

## G4 — alloc-free hot path: **PASS**

`decision_set_goat` G4: solve_into × 200 after warmup = **0 allocations** armed (engine-held scratch + keys buffer are build-time; the kernel never allocates pre-ensured).

## Verdict

**MEASURED NEGATIVE — the lane stays opt-in (feature `set_refine`, knob default 0.0), promotion REFUSED on G1.** The S1 capability (reversal exists in constructible geometry) is real katgpt-core-side, but at this engine's option model (unit corpus centroids as keys) the set-conditioned pull destroys route-term discrimination at real suite widths, monotonically in γ. A future re-open needs a DIFFERENT key family (e.g. per-option fitted directions rather than corpus centroids, riir-train territory) — not a retune of γ/τ (the monotonicity probe closes that door).

Raw: `150_set_refine_ab_goat/results.json` (+ `gamma025/results.json`).
