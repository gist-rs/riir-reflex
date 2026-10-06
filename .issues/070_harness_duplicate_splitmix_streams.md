# Issue 070 — the harness carries TWO identical inline SplitMix64 streams (suites.rs `pub(crate)` + echo_gates.rs private); the substrate exports no reusable form

**Status:** OPEN (detection-only — substrate-first Mode 2 audit 2026-10-06, the 10-03 16:00→10-06 wave; no fix in this commit per the skill law)

## What

`src/harness/` now carries **two structurally-identical SplitMix64 implementations in one
crate**:

1. `src/harness/suites.rs:475` — `pub(crate) struct SplitMix64(pub u64)` with
   `next_u64()` (golden-ratio add `0x9E37_79B9_7F4A_7C15`, finalizer multiplies
   `0xBF58_476D_1CE4_E5B9` / `0x94D0_49BB_1331_11EB`, `z ^ (z >> 31)`), `below()`
   (Lemire multiply-shift), and a Fisher–Yates `shuffle` — the dataset-suite shuffler.
2. `src/harness/runner/echo_gates.rs:56` — `struct SplitMix64(u64)` with the SAME
   `next_u64()` body, plus `next_f64()` (53 mantissa bits as a VALUE, never
   `from_bits` — the frozen-read law) — the corpus-ab corruption stream.

echo_gates.rs landed LATER (`0b4a351`, feat(064) echo gates) and re-implemented the
stream instead of consuming the `pub(crate)` type one module over
(`crate::harness::suites::SplitMix64` is already visible to it). Two copies of a
determinism-load-bearing primitive in one crate can drift independently — e.g. a
mantissa-policy change in one `next_f64`-alike and not the other silently forks the
harness's byte-reproducibility claims (corpus synthesis artifacts + echo-gate seeds
are both frozen-read commitments).

## Vocabulary translation (substrate-first skill)

Searched: `splitmix`, `0x9E3779B97F4A7C15`, `BF58476D1CE4E5B9`, `C2B2AE3D27D4EB4F`,
"finalizer", "deterministic rng". What exists:

- `katgpt_core::types` → `katgpt_types::rng::Rng` (re-exported at katgpt-core
  `lib.rs:599` `pub use katgpt_types as types;` — zero new deps for this crate) —
  but `Rng` is **XorShift64 with splitmix used ONLY as one-shot seed mixing** in
  `Rng::new` (Issue 296); the output stream is XorShift, NOT the splitmix stream.
  Not a drop-in for either harness site.
- The finalizer body ships INLINE in `Rng::new`'s doc-commented code and in ~45
  substrate-side module-locals across katgpt-core/dec/forward — all bit-identical,
  none exported as a reusable `fn`/struct.
- No `pub fn splitmix*` / exportable `pub struct SplitMix64` in a consumer-facing
  substrate path (`katgpt-dec::birth_death::SplitMix64` is a DEC-specific home;
  `katgpt_core::mi::test_support::SplitMix64` is test-support-scoped).

## Workspace copy-gate context (note-level, not the primary finding)

The standing splitmix64 copy-gate ("consolidate at the 4th consumer-side arrival")
counts the full-finalizer family in PRODUCTION consumer code: seal-remake
`seal-view/viewer.rs` (finalizer over UUID halves, documented in-source), reflex
harness ×2 (this finding), plus established home-module/bridge classes in riir-ai
(swarm/luck.rs single-home + cgsp bridges), chain `rtdc_bridge`, dao `sim/rng.rs`,
shader `pattern/rng.rs`, train homes/mirrors. The odd-multiplier-only forms (chain
`mcp/supervisor.rs:1466` placeholder hash, train `sil_replay.rs:133` Knuth stride
finalize) are a LOOSER family — noted, not counted. The arrival arithmetic on the
strict family: this wave pushes it to/past the gate.

## Fix (proposed, not executed here)

- **Minimal (intra-crate):** hoist ONE `SplitMix64` to a shared home
  (`src/harness/rng.rs` or keep `suites::SplitMix64`) and have echo_gates consume
  it; keep `next_f64` as a thin method on the one type. Both call sites keep their
  exact stream bytes — a bit-identity assertion on one known seed pair pins it.
- **Substrate (the copy-gate resolution path):** export
  `katgpt_types::rng::{splitmix64_finalize, SplitMix64}` (the natural home —
  `Rng::new` already documents the algorithm) and delegate the reflex pair +
  seal-view's viewer finalizer at next touch. Zero behavior change; the frozen-read
  law is preserved because every existing body is already the standard
  Steele/Lea/Leiserson finalizer.

## Classification

- **DRY violation (intra-crate, primary)** — the later file duplicated a
  `pub(crate)` type in the same subsystem.
- **Copy-gate arrival (secondary, note-level)** — workspace family census above;
  substrate export is the recorded consolidation path.

Refs: substrate-first SKILL.md run-log row 2026-10-06 (katgpt-rs) · katgpt-types
`src/rng.rs` (the finalizer's documented home) · reflex commit `0b4a351` (the
duplicating arrival).
