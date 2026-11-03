# Issue 105: pair-free probe example — laya ↔ modelless hashed-bag transport (riir-train Issue 663 T2)

**Status:** Open — the code lane for riir-train [Issue 663 T2](../../riir-train/.issues/663_good_view_recipe_applicability_gate.md)
**Date:** 2026-11-03
**Source:** arXiv:2610.39302 (JEPA-PRH) via katgpt-rs Research 646
**Consumes:** `katgpt-core/leakage_probe` (Issue 736 — the pair-free Sinkhorn↔Procrustes transport chain, Bench 495 posture) + `riir_reflex::embed` (the modelless hashed-bag embedder) + the laya lane (`encode_question` hidden states, Metal).

## What

`examples/pair_free_probe.rs` (feature `pair_free_probe`, Metal lane): per suite
(banking77 / massive_intent_en / ag_news — T1's measured q table at 0.72 / 0.46 / 0.09),
embed N train texts into BOTH spaces and measure whether the modelless pair-free
chain aligns them:

- **Space A (frozen laya):** `RiirAgent::encode_question` final hidden states,
  mean-pooled over the sequence (checkpoint `english`, `LAYA_DEVICE=metal`).
- **Space B (modelless):** `embed::Embedder` hashed unigram+bigram bags, 256-d.
- **Pair-free chain:** `katgpt_core::leakage_probe::probe` — per-side suite class
  labels, chance-normalized transfer lift + alignment cos + neighborhood hit +
  the honest `InsufficientAlignment` verdict tier.
- **Paired arm (the positive control):** top-64 PCA per side (f64
  `symmetric_eig`), orthogonal Procrustes fitted on the first 64 anchors, then
  post-Procrustes residual + **FOSCTTM** (mean rank fraction of the true
  counterpart) on the held-out rest — paired vs raw.

## Why this repo

The only checkout that path-deps BOTH embedding lanes (`riir-infer-laya` +
`riir_reflex::embed`); katgpt-core rides the existing path dep. The RECORD and
adjudication live in riir-train (Bench 691, Issue 663 T2 tick) — this issue is
the code lane's home, per the cross-repo citation law.

## Expected reading (pre-registered, from Research 646 Laws 1–3)

laya is next-token-trained ⇒ GL(k)-identified, nuisance-carrying (Law 2/3) —
the PRH paper's own pair-free result on NTP encoders is FOSCTTM .287 ≈ chance.
So: pair-free verdict `InsufficientAlignment`/`Low` + paired arm strictly
better than raw = the recorded mechanism (rotation+pairing works, pair-free
fails on this space pair). If instead the pair-free chain reads usable lift,
that revises the Law-2 reading for THIS space pair — either way the numbers,
not the prior, decide. ρ rider (both sides) lands when katgpt-rs Issue 978's
primitive ships; not a blocker.
