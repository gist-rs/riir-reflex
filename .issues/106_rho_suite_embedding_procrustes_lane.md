# Issue 106: ρ per suite embedding grouped by label + the ρ-gated cross-lane Procrustes probe — the Issue 978 T5 consumer lane (riir-reflex)

**Status:** Open — consumer-measurement lane authorized by owner verdict 2026-11-04 (Claude AGREE, katgpt-rs Issue 978 T5 GO; filed SECOND per the verdict's ordering — the riir-refine lane is first). **The savings arm below is PRE-REGISTERED — it is pinned BEFORE any measurement runs.** T4 promotion of the upstream primitive is decided only after this lane's result AND the riir-refine lane's result are recorded under katgpt-rs Issue 978 with their commit hashes.
**Date:** 2026-11-04
**Upstream primitive:** katgpt-rs `katgpt-core::nuisance_fraction` (opt-in feature `nuisance_fraction`, landed `e572a2281` 2026-11-03) — ρ = tr(C_B^{1/2})/k over grouped multi-view embeddings + `alignment_floor` = 2k(1 − min ρ) (arXiv:2610.39302 Thm 3, Balestriero & LeCun). Concurrent art to cite in any output doc: Schnaus et al. arXiv:2610.09411 (cross-set predictability) — this lane is adoption engineering, not a novelty claim.
**Boundary:** scoped contract run 2026-11-04 GREEN (`ci_boundary_contract.sh --repo riir-reflex`, exit 0, 7 cross-repo dep edges measured). The lane adds **no new dep edge**: `katgpt-core` is already the ONE foreign code-level dep (BOUNDARY.md); the lane turns on the upstream opt-in feature via a forward in this repo's own manifest behind a measurement-only feature (never in RELEASE_FEATURES), never a new crate or sibling edge. Dual-allocation gate run at allocation time: 0 twin / 0 independent / 0 counter.

## The lane

Two halves, both REPORT-ONLY over the canonical suite corpus (`.raw/datasets`, the pinned engine baseline):

1. **ρ per suite embedding grouped by label** — embed each suite's questions with the shipped modelless hashed-feature embedding, group by gold label (the label is the shared world state z; paraphrase-class members of one label are the views), compute ρ per suite. Reads the label structure the fused gate already fits against: a low-ρ suite is one whose labels do not explain the embedding variance — evidence the harness may cite beside calibration diagnostics. Refusal law inherited: < 2 labels or a single-member label refuses loud, never reads 0.
2. **The ρ-gated cross-lane Procrustes probe** — the cross-LANE transport (modelless hashed-feature space ↔ the laya lane's embedding space, the cascade lane's two arms): compute ρ both sides FIRST, evaluate `alignment_floor` = 2k(1 − min ρ), and REFUSE to run the Procrustes solve when the floor already proves the transport doomed (best-achievable whitened-RMS² error above the useful-alignment threshold BEFORE any rotation is spent). When not refused, run the Procrustes (the shipped `procrustes_polar`-class helper in katgpt-core's `leakage_probe`) and report post-RMS² against the floor — the floor's consumer contract is "compare post-Procrustes RMS² against it, never raw cosine".

Substrate-first at implementation: re-run the vocabulary grep (`grouped_variance`, `between_group`, `icc`, `rho_space`, `scene_fraction`, `alignment floor`, `procrustes`) across this repo AND katgpt-core before writing any new module — consume, never fork.

## Pre-registered savings arm (the promotion-evidence bar — pinned BEFORE measuring)

The upstream T4 promotion bar (katgpt-rs Issue 978): "the gate is real when a consumer shows refusing a doomed transport/rotation SAVES measurable compute or prevents a wrong verdict." **A ρ ranking that only correlates with something does not count.** This lane's arm:

- **The claim under test:** the ρ floor refuses at least one cross-lane transport the probe would otherwise have run, and the refusal is CORRECT — the refused transport's post-Procrustes RMS² (computed once, out-of-band, as the adjudicating ground truth) sits at or above the floor's prediction, i.e. the gate refused a genuinely doomed alignment, not a viable one.
- **Pass:** ≥ 1 refusal, adjudicated correct out-of-band, with the ledger recording wall-clock(Procrustes solve + downstream parity work avoided) vs wall-clock(ρ read), box state beside both per the latency-claim law (`scripts/bench_preflight.sh` provenance quoted; battery / power mode / load class). **Fail:** the gate refuses nothing (every suite pair passes the floor) or refuses a transport whose out-of-band alignment turns out viable (a WRONG refusal — worse than no gate) → the primitive stays opt-in upstream, the negative is recorded under katgpt-rs Issue 978 with this issue's commit hash, and the lane closes honest.
- **Wrong-verdict half (the refusing half of the same measurement):** if the probe would have reported a cross-lane "alignment" that downstream verification later contradicts, the gate's refusal is a prevented-wrong-verdict demonstration — recorded with the same rigor.

## Discipline

- Claim-lease before implementing (`py ../katgpt-rs/scripts/claim_lease_check.py --item 106` after appending the Claimed line; 48 h expiry, the push is the arbiter).
- Numbering: allocated as 106 from `.issues/.highwater` (105 → 106, written back same commit).
- G5 law: any published laya-side number in the probe re-runs the consumer-side G5 parity gate at both postures before the table is cited.
- Results land under katgpt-rs Issue 978 (cross-repo citation WITH the repo name) + this issue's Status; T4 promotion is decided there, not here.
