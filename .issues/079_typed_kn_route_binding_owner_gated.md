# Issue 079 — typed_decisions' legacy `k == N` route binding is position-driven (Bench 128 follow-up)

**Status:** Open — OWNER-GATED on the fix posture (it changes model behavior, so it needs a re-bench, not a silent edit). Found by Bench 128 (issue 077's option-permutation probe, run 2, 2026-10-08).

## The finding

Bench 128 falsified "the modelless engine is permutation-invariant by construction": the
legacy `k == N` index alignment in `DecisionEngine::solve_sample_into` (engine.rs) binds the
option AT POSITION i to domain i's route term whenever by-name resolution fails and the
option count equals the domain count. The classification suites were fixed content-side
(same bench: `train_row_label` now names domains by the option keys — canonical-order
byte-identical). **typed_decisions cannot take that fix mechanically**: its domain names are
the 4 workflow names while its choice option keys are criteria phrases — by-name can never
resolve — so its 4-option questions (400 of 600 choice questions) ride the position-driven
binding TODAY, and that binding is load-bearing for every published typed number (the
serving board's typed cell, the laya comparisons, the instinct H2 hybrid inputs).

Measured (Bench 128 run 2): typed control median swing 4.70 pt, max 18.81 pt, 68.3% flips
under option permutation. laya/typed reads 4.12 pt median / 13.3% flips on the same probe —
the suite is order-fragile in both our lanes.

## Why owner-gated

Any repair changes scores → moves published numbers → needs the full re-bench + GOAT
discipline (the "try implement to unblock... after check goat + proof gain" law). Options,
in the order I'd price them:

- **A. Option-key→workflow routing map** (the typed rows carry gold workflows; a
  per-question key→workflow binding could route by content the way the classification fix
  does). Risk: the binding is question-authored, not derivable — needs the typed corpus's
  own metadata; if absent, this is unpriceable without new data.
- **B. Drop the legacy alignment for typed** (drafter-only posture, the Issue-036 T2 fix
  family). Cleanest semantics; biggest expected score move (route terms vanish) → full
  typed re-bench + the board re-publish + instinct's typed hybrid seat re-read.
- **C. Accept + document** (the current state): the serve contract gains a line — typed
  questions MUST present options in the authored order (the canonical order is load-bearing
  semantics, like noul's `[false, true]`). Zero risk, zero gain; the probe's typed control
  cell stays RED forever as the standing disclosure.

## Tasks

- [ ] Owner call: A / B / C.
- [ ] If A or B: implement behind a feature flag, re-bench typed (the full suite, both
      lanes), GOAT the delta, promote/demote per the gates; the perm-probe typed control
      cell is the acceptance probe (must PASS post-fix).
- [ ] If C: pin the present-order requirement in `.docs/02_protocols/` (the wire contract's
      typed section) + the serve edge's docs; keep the Bench-128 typed row cited.
