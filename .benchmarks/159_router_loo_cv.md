# Bench 159 — teacher-router LOO cross-validation: instrument + first real read (2026-10-24)

**Status:** RECORD — instrument landed in this commit (the CV column), first read over the
box-resident columns. Feeds riir-train [Issue 640](../../riir-train/.issues/640_4090_mammoth_teacher_capture_lane.md)
T4's CROSS-VALIDATION gate (verdict round 1) and rethink
[Issue 032](../../riir-rethink/.issues/032_subq_snap_beat_lane.md) T10.4/T3.5.

## What landed

`run_teacher_router` (src/harness/runner/router.rs) now emits a leave-one-row-out routed
accuracy (`routed_accuracy_cv`) beside every blend suite's in-sample number — the 640-T4
gate's answer to "the 067 routed-oracle row is in-sample by construction". Each row's pick
uses `g_r` computed with that row dropped from BOTH accuracy sums
(`((hits_a−hit_r)−(hits_b−hit_r))/(n−1)` at T=0.01): exact, O(1) per row, no RNG,
bit-reproducible, the pick never sees its own row. The emitted artifact keeps the full-data
gate (the deployable form); the CV column rides the report, the markdown, and the routed
bin's `router` header (`routed_accuracy_cv` + `cv_law`). Tests: complement fixture
(in-sample 0.5 vs CV 0.0 — the optimism class at its smallest shape), dominance
(CV == in-sample where one column wins everywhere), n<2 refusal.

## The first read — laya-english ⊕ jeva-4B, the four row-aligned suites

Inputs: `.raw/distill_teacher` (laya english, metal, t20k) × `.raw/distill_teacher_jeva`
(jeva-4B, `.raw/datasets`), command:
`cargo run --release --bin harness -- --teacher-router .raw/distill_teacher,.raw/distill_teacher_jeva --router-out .benchmarks/159_router_loo_cv`
(ran at the pre-landing tree; `sha 5ec1d63` in ROUTER.md is HEAD at run time). Gold pins
verified row-by-row at join time on all four; emotion (16000 vs 15999) and sst5 (8544 vs
8533) correctly REFUSED — different survivor predicates, never force-joined.

| suite | laya acc | jeva acc | gate g | routed acc (in-sample) | cv (loo) | Δ |
|---|---|---|---|---|---|---|
| ag_news (n=20000) | 0.9769 | 0.7331 | 1.0000 | 0.9769 | 0.9769 | 0 |
| banking77 (n=9993) | 0.4312 | 0.9274 | 0.0000 | 0.9274 | 0.9274 | 0 |
| massive_intent_en (n=11514) | 0.7341 | 0.9480 | 0.0000 | 0.9480 | 0.9480 | 0 |
| xnli_en (n=20000) | 0.9654 | 0.8508 | 1.0000 | 0.9654 | 0.9654 | 0 |

## Verdict

**CV == in-sample on every joined suite — zero in-sample optimism on the resident columns.**
The per-suite gaps (12–50 pp) saturate the sigmoid gates to f64-exact 0.0/1.0, so dropping
one row out of 10–20k cannot move a gate; the routed numbers ARE the dominant teacher's
accuracies, honestly. The 640-T4 gate now has a machinery + a baseline: when the
mammoth-1/elephant-1 columns land (their board areas differ by 3–7 pts per family — gates
will stay UNSATURATED), the CV column is the number that moves things, and this read is the
control showing what saturation-null looks like.

## Caveats

- The `teacher` field reads "unknown" for the laya-english column — its bins predate the
  header field; display-only, the checkpoint string still carries provenance.
- The 067 routed-oracle row (clef ⊕ pplx, 40.81 zeros-included) is NOT re-read here — those
  capture dirs are not on this box. The instrument now CAN re-read it wherever the dirs
  exist; the 067 disclosure stands until that re-run.
- Routed artifacts emitted under `.benchmarks/159_router_loo_cv/` are TRAIN-row artifacts
  (the no-cheat law) — their verdict remains T4's single frozen read; nothing selects on them.
