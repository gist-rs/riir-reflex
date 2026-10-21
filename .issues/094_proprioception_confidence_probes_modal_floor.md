# Issue 094: Proprioception-derived confidence probes + input-keyed modal-baseline floor

**Status:** OPEN — from `.research/011` (arXiv:2610.17495 distill, 2026-10-21); modelless track, report-only probes, no serving-path change.
**Claimed:** 4090-modal-floor, 1792613492, expires 2026-10-23T20:11:33Z

## Motivation

The paper's measurement discipline transfers modelless: (a) any internal-signal claim must beat an **input-keyed modal lookup** (the anti-mimicry census the paper uses as its training control — here a floor gate), and (b) internal-trajectory scalars (decision depth, entropy) are cheap confidence *candidates* for the encoder lanes — provided they beat the conformal-naive floor (Report-the-Floor law) and price their per-layer sweep cost (katgpt-rs Research 388's 10–70× refutation lesson).

## Tasks

- [ ] **Modal-baseline floor column**: harness tables gain an input-keyed modal-answer lookup baseline (key = suite-appropriate input identity — e.g. normalized-input bucket × option-count class; census computed offline, no model). Every lane's accuracy and confidence readout is quoted against it; a lane at/below the floor carries no internal signal and the table says so.
- [ ] **laya stability-depth probe**: per-layer last-position residual stability (cosine + L2 between consecutive stages) via the `twt_profile` capture seam; stabilization depth per case; report depth-distribution per suite + depth-vs-correctness AUROC. Substrate half: riir-infer Issue 1106.
- [ ] **clef headful candidates**: last-position entropy + lens depth (iff the checkpoint ships an LM head) as cal-select candidates; G1 — every confidence candidate MUST beat BOTH floors or is recorded NEGATIVE with the miss named: (a) the **conformal floor** — the shipped split-half conformal-naive floor ECE adaptation (`src/lanes/drex.rs` `DrexConfReadout::floor_ece`, first half calibrates the conformal CDF, second half audited through it, occupancy floor `DREX_MIN_FLOOR_PAIRS`) so the gate measures the same way as the lanes it is compared against; (b) the **input-keyed modal floor** — beats conformal but fails modal = mimicry; beats modal but fails conformal = miscalibrated.
- [ ] **Cost law**: any per-layer sweep monitors a layer SUBSET; price G2 against the incumbent readout before any serve-time adoption. A quality-pass + cost-fail outcome is the recorded trigger for the trained lane (riir-train Issue 639).
- [ ] **Report-only posture**: no exit-code coupling, no serving-path edit; if any depth signal is ever armed, G3 disarmed byte-identity + never gating raw-sync correctness.

## Context

- Existing baselines: `harness/mc_ab.rs` carries a suite-level `acc_majority_pick`; the input-keyed census is the finer, per-case control from the paper.
- The modelless decision engine has no internal trajectory — these candidates are encoder-lane-only (laya headless, clef headful) and stay comparison-lane citizens.
- Paper provenance: arXiv:2610.17495; note `.research/011`; code `seal-rg/proprioception` @ `1b43bd3` (clone removed).
