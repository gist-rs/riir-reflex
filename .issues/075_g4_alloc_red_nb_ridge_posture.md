# Issue 075 — G4 alloc gate red at the nb_ridge posture (decision_set_goat, 800 allocs)

**Status:** OPEN — filed 2026-10-08 (shikuwa/4090) during the Issue-073 T5 session's
final `ci_feature_guard.sh` run.

## What

`scripts/ci_feature_guard.sh` aborts at the `nb_ridge`-armed G4 cell:

```
thread 'main' panicked at benches\decision_set_goat.rs:195:5:
assertion `left == right` failed: G4 FAIL (nb_ridge armed): the zero-alloc core
allocated 800 time(s) after warmup
  left: 800
 right: 0
error: bench failed
```

**Attributed PRE-EXISTING (not the 073 lane landing):** reproduced byte-identical at
`0a3c79e` (the base before the 073 commits) in an isolated worktree — the 073 diff
(drex lane, harness flag, `readout_brier` report field) touches no decide-path code.

## Likely shape (unverified)

The `nb_ridge` ridge-readout arm allocates on the decide path (the arm was landed as
`--ridge-select` calibration-side; the G4 gate arms it IN THE SERVE PATH where the
zero-alloc law binds). Either the arm grew an allocation when katgpt-core moved
(0.4.1 — `contrastive_scope`/calibration pairs), or the G4 counting allocator's
arm-coverage widened and exposed an old allocation. 800 = 200 solves × 4
`solve_into` calls in the bench loop.

## Fix direction

`cargo bench --bench decision_set_goat --features nb_ridge` on a quiet box, then either
(a) make the ridge readout zero-alloc on the serve path (pre-allocated scratch /
returned-slice pattern like the other readout arms), or (b) if the allocation is in the
CALIBRATION-only path, gate the G4 arm to the serve posture it actually claims.

## Validation

`scripts/ci_feature_guard.sh` full green after the fix (the guard aborts at this cell
today — layers after the bench are unreached; check nothing else moved while red).
