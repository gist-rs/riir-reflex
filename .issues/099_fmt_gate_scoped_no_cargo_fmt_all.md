# Issue 099 — fmt gate for reflex (ported from katgpt-rs Issue 946), scoped NEVER `cargo fmt --all`

**Status:** OPEN — claimed zed-amo-idle, 2026-10-27 ~03:2x +07, expires 2026-10-29 ~03:2x +07

## The gap

Reflex has ZERO fmt enforcement (`grep cargo fmt|rustfmt scripts/ .github/workflows/` → no hits),
and the drift keeps landing: the 2026-10-26 census found 6 files / 21 sites; by 2026-10-27 the
same surface measured 6 files again but a DIFFERENT set — the mammoth/elephant/opendecider lane
landings added fresh drift while earlier rows were drained. Recurrence is proven; nothing gates it.

## The reflex-specific hazard (why this is NOT a verbatim katgpt-rs port)

`cargo fmt --all` formats "all packages, **and also their local path-based dependencies**".
katgpt-rs (the canonical Issue-946 gate) is the upstream root — its `--all` reaches only its own
workspace, so the canonical gate can invoke it. Reflex is a CONSUMER: its path deps pull in
`../katgpt-rs`, `../riir-infer`, `../riir-reflexer`, and **katgpt-rs carries INTENTIONAL pinned
drift** (`katgpt-rs/scripts/fmt_pending_expected.txt`, the Issue-946 drain list). A bare
`cargo fmt --all` from reflex would rewrite those sibling files as uncommitted working-tree
changes in katgpt-rs — silent cross-repo mutation under concurrent sessions. Measured today from
a detached worktree: `cargo fmt --all` dies at cargo-metadata when siblings don't resolve, and
formats the whole dep closure when they do. **The reflex gate therefore walks reflex's OWN
tracked `.rs` files with bare `rustfmt --edition <E> --check`** (the recipe proven in the same
session), never cargo-fmt's `--all`.

## The gate (scripts/fmt_gate.py)

Ported from `katgpt-rs/scripts/fmt_gate.py` (153 lines, Issue 946 option 1), with the walker
swapped and the same laws kept:

- **Membership verdict, both directions** — UNPINNED (drift with no row: format + commit, never
  pin) and STALE (row whose file went clean: delete the row in the drain commit). A count is
  green on a swap; the list only shrinks.
- **Per-file rustfmt invocation with strict exit classification** — exit 0 = clean, 1 = diverging
  (recorded), anything else = **NO VERDICT (exit 2) with the file named**. Two tooling traps are
  baked in as design constraints, both measured at the founding run:
  - **Module recursion misattribution** — rustfmt recurses into `mod` children by default, so a
    clean root file (`src/lib.rs`, both `mod.rs` roots) exits 1 on a drifted CHILD and the drift
    is attributed to the wrong file. Children are excluded via the CONFIG KEY
    `--config skip_children=true`.
  - **The `--skip-children` flag does not exist on stable rustfmt** (1.9.0: `Unrecognized
    option`, and it exits **1** — colliding with the diverging exit; the first flagged run read
    109 of 116 files as diverging off that one rejected flag). The gate inspects stderr on rc=1:
    `Unrecognized option` / `error:` is a NO-VERDICT refusal, never a classification.
- **Walk floor** — `MIN_WALKED` pins the `git ls-files '*.rs'` population (measured 119 at
  landing); a walk regression refuses, never greens. The `.benchmarks/` probe artifacts are
  EXCLUDED from the walk by policy (recorded bench bytes — the bench-guard principle: a bench
  file's bytes are its artifact) with the excluded count disclosed on the pass line.
- **Edition read from the root Cargo.toml, refusal on absence** — a wrong edition silently
  changes rustfmt's output shape; missing `edition = "..."` is NO VERDICT, never a default.
- **Self-test arms** (ported + extended): parse/verdict membership arms + the phantom-class arm
  (a non-0/1 rustfmt exit must refuse, never classify).

## Wiring

Layer 11 of `scripts/ci_feature_guard.sh` — the cheap always-run block (zero cargo; ~119 rustfmt
spawns, seconds). The full-guard run on a quiet box is OWED at landing (this unit landed under a
loaded box — 92% CPU, sibling GPU arms hold it, Plan-337 posture — so only the new layer + the
self-test were executed, and `bash -n` syntax-checked the guard edit).

## The drain list

`scripts/fmt_pending_expected.txt` lands with 6 rows — the measured drift at `ca2eb39`
(src/bin/harness.rs, src/harness/modal_floor.rs, src/harness/slice_leak.rs, src/lanes/elephant.rs,
src/lanes/mammoth.rs, src/lanes/opendecider.rs). The one-time format commit is **staged, not yet
pushed**: worktree commit `49458fb` (detached worktree /tmp/reflex-fmt) carries the format PLUS
the deletion of these 6 rows in the same commit, and lands on origin only after the elephant
window-3 recapture + the r1_after_window3 chain end — a src/ change landing mid-window would
force the next per-suite harness rebuild under live Metal forwards (the Issue-1124 poison class;
any sibling Protocol H pull would carry it). The gate lands FIRST (scripts/ only — no rebuild
surface), green under the pins; `49458fb` lands second, draining the list to empty.

## Acceptance

- [x] Gate port lands with self-test green (executed this session).
- [x] Layer 11 executes standalone green under the 6 pins (executed this session: `✓ 116 files
  checked, 6 pinned, excluded 3 .benchmarks/ artifacts`).
- [x] UNPINNED arm proven live: pins emptied → exactly the 6 red UNPINNED.
- [x] STALE arm proven live: an appended row for the clean `src/lib.rs` → reds STALE.
- [x] Unrecognized-option refusal observed live (the founding 109-phantom run) and now guarded:
  that shape refuses NO-VERDICT instead of classifying.
- [ ] `49458fb` lands post-capture (fmt + the 6 row deletions); the drain list reaches empty.
- [ ] Full `ci_feature_guard.sh` run on a quiet box (all 11 layers).
