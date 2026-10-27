# Plan 013 — `emotion_syn`: the first-party synthetic emotion suite (the ESC re-source, step 1)

**Status:** IN EXECUTION — filed 2026-10-27. T1–T3 landed; T4 (generation) running on amo (M5 Ultra, MLX/Metal); T5–T7 follow the generated rows.

Upstream decision: riir-rethink Research 002 §Execution (owner call 2026-10-27: **synthetic emotion FIRST**, ahead of the elephant panel + the 640-T3 capture). The dair-ai/emotion suite stays ⛔ benchmark-only (licence ledger); this suite is the licence-clean re-entry for the niche. Precedent: typed_decisions ("latent factors + our teacher labels, no scraped content").

## Design (pinned BEFORE any fit — the Research 002 amendment A3)

- **Wire:** the same six keys and ClassLabel order as `emotion` (`EMOTION_KEYS`: sadness, joy, love, anger, fear, surprise) — `build_emotion_syn` is `build_emotion` under its own suite name, so every downstream head shape carries over.
- **Train:** generator A = Ternary-Bonsai-2-27B (Apache-2.0, prism-ml mlx 2-bit pack, the pack's own loader) over prompt family A.
- **Test (the A3 holdout):** generator B = Qwen3.5-4B (Apache-2.0, mlx-community 4-bit) over prompt family B — DISJOINT topic and register lists and a different instruction shape (asserted by the script's self-test). A model that learns generator A's style or family A's topics gains nothing on test.
- **Labels:** the intended latent label, KEPT only when the OTHER model's zero-shot read agrees (first-token logprob argmax over the six label words; the judge refuses on a first-token collision). Every row is dual-model confirmed; per-label agreement rates land in `manifest.json`.
- **Hygiene:** exact + normalised dedupe within each split and ACROSS splits (test never repeats a train text); 5–45 words; meta lines dropped.
- **Sizes:** train 4,800 (800/label), test 1,200 (200/label); the arena reads a 400-case stratified test slice (emotion's cap).
- **Floor the GOAT must beat:** the modelless A0 on this suite (the beat-the-modelless-arm floor), measured at T5 BEFORE any specialist fit.

## Tasks

- [x] T1 Generator `scripts/synth_suite.py` (3.9-grammar clean, ASCII console output, `--self-test`); pilot 2026-10-27: 24 rows / 9 prompts / 35 s; judge agreement sadness 0.88 · joy 1.00 · fear 0.86 · love 0.47 · anger 0.45 · surprise 0.36 — the veto is load-bearing, not a rubber stamp.
- [x] T2 Harness wiring: `build_emotion_syn`, the `emotion_syn` SuiteSpec (`named_only`), train_row_label / train_docs / eval_case_text / option-key + expected-key arms, slice_guard floor 3,680 (80% of the 4,600 pool after cal), slice-leak oracle arm; clippy `-D warnings` clean, 39 targeted tests green.
- [x] T3 Licence ledger row (`.docs/02_protocols/dataset_manifest.md` §Licences): Apache-2.0 (ours), generator grants verified on the hub 2026-10-27, paid lane ✓.
- [ ] T4 Generate train (Bonsai → Qwen judge, seed 1) then test (Qwen → Bonsai judge, seed 2) into `.raw/datasets/emotion_syn/`; record the manifest digests + veto rates in the T5 bench.
- [ ] T5 Baseline bench: A0 modelless on the 400-case slice (determinism, G1 calibration, slice-integrity) — the wanli_en Bench 130 shape. This A0 is the floor every specialist must beat.
- [ ] T6 Hand-off: riir-train re-fit (bag + encoder classes) → the suite's own ESC GOAT (Bench 0059 G1–G5, G1 = paired LB95 ≥ 0 AND ≥ A0) in riir-rethink; the escalate table arms per-suite on a passed GOAT, never a flag.
- [x] T1b Generator generalized to a SPECS table (`--suite emotion_syn|sst5_syn`; emotion's constants unchanged, so the running generation is unaffected): `sst5_syn` = the five SST-5 levels, review-domain families A/B disjoint, judge answers digits 1..5 (the level names share first tokens). Its harness wiring (wider: report/distill/gate_drift arms) lands as Research 002 step 2 AFTER this suite's T5.
- [ ] T7 Commit the generated pages to the release lane only after T5 (the `.raw/` tree is gitignored; the manifest + digests are the record, the bytes regenerate deterministically from the seeds only up to MLX sampling determinism — the digests, not the seeds, are the pin).
