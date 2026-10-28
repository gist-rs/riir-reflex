# Issue 102 — harness runner robust_ab.rs carries a second inline f64 sigmoid body (the router.rs class, site 2)

**Status:** OPEN — detection-only filing (substrate-first Mode 2 delta audit 2026-10-28; fix deferred to a quiet window — the league gate2 driver + the riir-train 650-T5 release build held the box, zero cargo this session).

**Claimed:** amo-idle-refine-g, 1793155785, expires 2026-10-30T03:00:00Z

`src/harness/runner/robust_ab.rs:278` (the T2 label-smoothing section):

```rust
#[inline]
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
```

This is the copy-gate class the 10-15 audit noted at `harness/runner/router.rs:111` and the 10-24b discharge repaired (katgpt-rs substrate-first run log rows) — a restated sigmoid body with the substrate REACHABLE (reflex deps katgpt-core non-optional; `exact_sigmoid_f64` is the exported form router.rs consumes).

## Menu (the discharged shape)

Delegate the body to `katgpt_core::exact_sigmoid_f64` with the provenance comment (the `router.rs:51` + `router.rs:119` form), keep the local fn signature so no call site moves, and pin bit-identity with a frozen-legacy-body test (the `sigmoid_delegation_matches_frozen_legacy_body` pattern in `src/engine_tests.rs`) so the T2 label-smoothing numbers cannot silently move.

Verify at a quiet window (build contention defer): `cargo clippy --all-targets --all-features -- -D warnings` — robust_ab sits inside the harness runner's feature-gated mod set, so compile under the gate that includes it (`--all-features` covers it).

**Refs:** katgpt-rs `.agents/skills/substrate-first/SKILL.md` run log (10-15 note, 10-24b discharge); `src/harness/runner/router.rs:997` (the delegation-stability comment + pin rationale).
