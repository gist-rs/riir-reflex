# Issue 059 — re-open the six harness families: wide template-disjoint eval

Status: IN_PROGRESS — plan 009

Instinct Issue 008 T8 (recorded in instinct HISTORY.md 2026-09-30) dropped
`harness_visibility` / `harness_permissions` / `harness_tool_fit` /
`harness_routing` / `harness_sensitivity` / `harness_cache_reuse` from the
specialist covered set: n = 12–16 template-shared evals cannot distinguish
generalization from memorization, and visibility/cache_reuse sat at reflex's
near-ceiling (no certification headroom). The recorded re-open condition is a
**larger template-disjoint reflex-side eval**.

This issue is the reflex-side anchor for landing that eval:

- wide per-family eval populations (~96 cases, class-balanced),
- template-disjointness enforced by gate (unigram-overlap ceiling + 3-gram
  wall + label-token bans + BLAKE3 digest pin),
- corpus/cal untouched, A0 re-baselined and recorded (bench 104),
- training handoff to riir-train (the trainer lane extends from
  code_fixtures; this repo consumes bytes only, per the instinct law).

Re-open is complete when a specialist is certified — or honestly refuted —
on the wide read. Until then the families keep serving A0 (`a0_stands`),
which is the designed fallback, not a failure.

Refs: instinct Issue 008 T8 · reflex `.plans/009_families_wide_eval.md` ·
bench 104 (lands with the eval).
