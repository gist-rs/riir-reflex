# Issue 049 — the seat seam refuses the synthetic harness families and code_fixtures; a no-specialist hybrid lane cannot even be measured there

**Status:** OPEN — filed 2026-09-27 (owner direction: the hybrid lane should
produce a bench result for the no-train case too). Companion: riir-instinct
Issue 010 (arena population + A0-stands posture + three-state rendering — its
T6 consumes this seam), riir-train (specialist training for dataset suites —
the trainable case).

## Why

`harness::runner::seat::prepare_seat` — the ONE public surface the instinct
hybrid lane plugs into (the one-way dep law: reflex never depends on the
consumer) — refuses every suite a no-specialist lane would need:

```rust
/// Prepare a registered DATASET suite by name (the synthetic families
/// and `code_fixtures` refuse — no specialist exists for them, and a
/// seat there would be a silent posture fork).
pub fn prepare_seat(name: &str, dir: &Path) -> Result<Seat, String> {
    ...
    if spec.synthetic.is_some() || name == "code_fixtures" {
        return Err(format!(
            "seat: {name} is not a dataset suite (synthetic/code paths have no seat)"
        ));
    }
```

So the harness families (Issue 004: visibility, permissions, tool_fit,
routing, sensitivity, cache_reuse) and `code_fixtures` are measurable by THIS
repo's own runner (`modelless_lane: true` on five of the six families) but
unmeasurable by any downstream hybrid lane — "not run" is manufactured at the
seam, not by the consumer. The refusal's stated reason ("no specialist exists
for them, and a seat there would be a silent posture fork") conflates two
things: a specialist is not required to seat A0 (the deployed modelless
posture is what the seat already serves), and the posture-fork risk is
solvable by making the synthetic posture EXPLICIT in the seat result rather
than by refusing the seat.

`harness_cache_reuse` stays out of scope here — `modelless_lane: false` is a
measured structural fact (the modelless lane has no KV cache; Issue 004 T3),
and a modelless answer there would be a fake task. Its seat refusal is
CORRECT and this issue must not widen it.

## Tasks

- [ ] T1 — Seat the five modelless harness families: a synthetic branch of
      `prepare_seat` that builds from the family's `synth_*()` SynthData
      (corpus/cal/eval splits already exist in-process) and returns the same
      `Seat` shape, carrying an explicit marker that the suite is synthetic
      (the posture-fork defence: the consumer cannot mistake it for a dataset
      seat). The refusal message stays for `harness_cache_reuse` and for any
      family seat requested without the marker plumbing — fail loud, never a
      silent fork.
- [ ] T2 — Decide `code_fixtures`' seat posture explicitly (owner-visible
      decision recorded in this issue when made): it is in-process generated
      with its own programmatic cal slice (`synthetic: None` legacy path), so
      either it joins T1's synthetic branch or its refusal gains a reason
      that names the decision. No silent default.
- [ ] T3 — The seat's own guarantees hold unchanged: byte-identical questions,
      the DEPLOYED modelless posture, one-way dep — the families' seats must
      reproduce the runner's own family rows (pin: a test seating one family
      and asserting its modelless eval accuracy equals the runner's published
      row for the same build).
- [ ] T4 — Cross-repo consumer green: riir-instinct Issue 010 T6 seats the
      families through the new branch; cite the consumer commit here when it
      lands (a cross-repo repair is not landed until it is committed in the
      sibling).

## Honest scope notes

- This issue enables MEASUREMENT; it does not promise the families' hybrid
  rows beat anything. A family's expected outcome is "A0 stands" (Issue 010's
  vocabulary) — authored fixtures with programmatic gold likely have no
  promotable hybrid arm, and that is a valid, publishable answer.
- No specialist is implied: specialist training on authored synthetic
  fixtures is a separate question nobody has argued for yet.
