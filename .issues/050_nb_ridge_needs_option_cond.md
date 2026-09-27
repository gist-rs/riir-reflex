# Issue 050 — `nb_ridge` without `option_cond` does not compile (the feature's dep list is missing the lane it reads)

**Status:** OPEN — filed 2026-09-27 by the riir-instinct session (Issue 008
T1's re-baseline), which enabled `nb_ridge` on the consumer dep and hit it.

## The defect

`ridge_lane::build_ridge_selection`'s signature names
`crate::option_cond::OcEvent` **unconditionally**:

```rust
events_for_pool: impl Fn(&[TrainDoc]) -> Vec<crate::option_cond::OcEvent>,
```

and `runner.rs` calls `oc_events_for(inp.pool_rows, pool)` into it
unguarded. But the feature table declares:

```toml
nb_ridge = ["nb_scope"]
```

so `--no-default-features --features modelless,nb_scope,nb_ridge` — the
exact posture a selective consumer builds — fails:

```
error[E0433]: cannot find `option_cond` in `crate`
  --> src/harness/runner/ridge_lane.rs:53:57
error[E0425]: cannot find function `oc_events_for` in this scope
  --> src/harness/runner.rs:2328:20
```

Reflex's own defaults carry `option_cond` and `nb_ridge` together, so
every in-repo posture compiles and the gap is invisible upstream — it
only bites a consumer selecting the ridge lane alone.

## The fix (one line)

Either of:

- `nb_ridge = ["option_cond"]` — the honest declaration (the ridge lane
  reads oc events for the pool; the measured reality is the two features
  ship together), or
- cfg-gate the `OcEvent` plumbing (`Vec<()>`/a no-op `events_for_pool`
  when `option_cond` is off) — preserves the narrower feature, at the
  cost of an `#[cfg]` seam through the selection signature.

The first is recommended: the lane already cannot arm without the oc
events' builder, and reflex's defaults unify the set anyway.

## Consumer note

riir-instinct works around it today by enabling
`["modelless", "nb_scope", "nb_ridge", "option_cond"]` on its dep (the
tested combo), with the oc lane OFF (`oc_select: false` — typed's
oc-armed posture is not an instinct suite). When this issue closes, the
instinct dep can drop `option_cond` if the narrower feature is preserved.
