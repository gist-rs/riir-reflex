# Issue 064 — Learner-density ascent leg for the corpus-synthesis lane (projection-sampling accept rule)

**Status:** OPEN — fusion idea from [katgpt-rs Research 603](../../katgpt-rs/.research/603_Projection_Sampling_MCMC_SFT_Data_Shaping.md) (arXiv:2610.02140, "Finetuning with Sampling"); pairs with riir-train Plan 438 Phase 2 (the training-side RIDT v2 consumer). Novelty of the *fusion* verified: no shipped code gates rewrites by a learner-density score (workspace grep 2026-10-03); the technique itself is the paper's prior art — we distill, not claim.

## The gap

`src/harness/runner/synth.rs` already has two of the three legs of arXiv:2610.02140's projection-sampling operator:

- **proposer**: cross-frame span transplantation (L437–479) — rewrites a frame's span while preserving the gold label (= C);
- **hard constraint**: `veto_accept` (L302) teacher-argmax == gold + shape/dedup/cal-exclusion + per-label caps.

**Missing leg**: acceptance never consults a **learner-density** score. The paper's mechanism — accept a candidate iff the frozen consumer's own density does not degrade (I-projection `p_C ∝ p·1_C`, greedy-ascent form) — is absent; the lane is a single-pass filter, not ascent. The paper's evidence says the density leg is what turns "constraint-satisfying" data into "learnable" data (SFT on projected traces rivals RL; pass@k shows capability import, not sharpening).

## Proposal

Add an I-projection ascent leg **alongside** (AND-ed with, never replacing) the teacher veto:

1. Score every candidate (and the frame it would join) with the frozen encoder the serving engine actually consumes (laya log-lik or the seat-corpus density proxy).
2. Accept a synth candidate only if its density sits within ε of the source frame's density (the **minimal-deviation signature** — the gateable fingerprint of projection rather than arbitrary filtering) or improves it (ascent, optional arm).
3. Keep the shipped hard constraints untouched (provenance discipline: acceptance can only reject, never inject).
4. Monotone ledger assert over the accepted set + accept-rate/Δ-density telemetry (pilot kill gate: accept-rate < 5% = no headroom, loud stop).

## Gates (before any default-posture change)

- Cohort-level `--corpus-ab` V5 as the outer gate (gold-only vs +synth, paired LB95 > 0, frozen single read) — unchanged.
- **OOD rig mandatory**: a projected-corpus win that fails to transfer OOD is echo, not learning → recorded NEGATIVE, lane dies.
- Abstention-band preservation: abstention-entropy histogram KL ≤ ε vs the pre-synth corpus (collapsing the abstain distribution breaks the calibrated fused gate downstream).
- Sealed-artifact discipline: any new corpus class gets a new magic/version + blake3 sidecar (SYNT v2, never an in-place mutation).

## Tasks

- [ ] Pilot: density-scored transplant candidates on one suite (massive — the published 0.7800 anchor) measuring accept-rate + Δdensity + the minimal-deviation signature; kill gate on < 5% accept-rate.
- [ ] If pilot passes: wire the ascent leg behind a flag; run corpus-ab V5 + OOD rig + abstention-entropy gate.
- [ ] Coordinate with riir-train Plan 438 Phase 2 so the training-side RIDT v2 and this lane share the operator (extract the scorer/accept core into shared substrate only when this second consumer lands — DRY, no parallel substrate).
