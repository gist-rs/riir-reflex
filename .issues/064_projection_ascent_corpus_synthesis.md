# Issue 064 — Learner-density ascent leg for the corpus-synthesis lane (projection-sampling accept rule)

**Status:** OPEN — task 1 (pilot) DONE 2026-10-04, kill gate PASS ([Bench 120](../.benchmarks/120_synth_density_pilot/RECORD.md)): massive_intent_en 38.3% / banking77 41.5% accept at ε_nat — the minimal-deviation signature holds (median |Δ| at 0.66×/0.30× the natural-neighbour scale), 7.7–8.3× the 5% bar, both suites src-misses 0. Task 2 (flag + corpus-ab V5 + OOD rig) is the live task. Fusion idea from [katgpt-rs Research 603](../../katgpt-rs/.research/603_Projection_Sampling_MCMC_SFT_Data_Shaping.md) (arXiv:2610.02140, "Finetuning with Sampling"); pairs with riir-train Plan 438 Phase 2 (the training-side RIDT v2 consumer). Novelty of the *fusion* verified: no shipped code gates rewrites by a learner-density score (workspace grep 2026-10-03); the technique itself is the paper's prior art — we distill, not claim.

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

- [x] Pilot: density-scored transplant candidates on one suite (massive — the published 0.7800 anchor) measuring accept-rate + Δdensity + the minimal-deviation signature; kill gate on < 5% accept-rate. **DONE 2026-10-04 — Bench 120, kill gate PASS with wide margin** (38.3% accept@ε_nat on massive + 41.5% on the banking77 control; instrument = the new `harness --synth-density-pilot` report-only mode — vMF kernel over the engine's own hashed embedding, τ data-derived per label, LOO pool scoring, NN-pair control; two runs bit-identical). En-route find: `SynthCand.src` is LABEL-LOCAL, not a global pool index — the doc comment said "pool row index" and the first pilot run mapped it globally (205,604 src-misses, all numbers discarded); doc clarified, mapping fixed, src-misses 0.
- [ ] If pilot passes: wire the ascent leg behind a flag; run corpus-ab V5 + OOD rig + abstention-entropy gate. **CODE HALF LANDED 2026-10-04** (`--synth-density-gate <p50|p75|p90>`, reflex `1973b3d9`-lineage): the gate is armed in `synth_one` AFTER the teacher veto (can only reject a veto-passing row, never rescue one; a rejected row does not consume the accept budget), per-label ε at the control-quantile |Δ| via `density_pilot::DensityGate` (the Bench-120 instrument reused — DRY, one density home), `density_rejected` per-label counter + `density_rule` disclosure in the run report AND the artifact header (additive `serde(default)`, version stays 2 — rows' semantics unchanged, both-direction compatible; verified: clippy clean, lib 272/272, ci_feature_guard 9/9). **REMAINING for this task**: the teacher-gated RUNS — generate a density-gated artifact (needs the openthai teacher up: `scripts/thai_rerun.sh`'s staging pattern, `.raw/openthai-systemone` + uv sync), then corpus-ab V5 + the OOD rig + the abstention-entropy gate. The ε rung choice (p50 vs p75) is the A/B the run answers.
- [ ] Coordinate with riir-train Plan 438 Phase 2 so the training-side RIDT v2 and this lane share the operator (extract the scorer/accept core into shared substrate only when this second consumer lands — DRY, no parallel substrate).
