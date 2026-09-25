# Issue 031 — tetris v4 Phase-2 hand-off: fixture_pins four-hash + the parse-precision note (fidelity only)

**Status:** OPEN — hand-off from katgpt-rs Plan 609 Phase 1 / Bench 890 (G1 FAIL → the crossed-head integration is CLOSED; this issue is the fidelity surface only)

Upstream: katgpt-rs `.plans/609_tetris_v4_preview_phase1.md` (Phase 1 complete), `.benchmarks/890_tetris_v4_preview_phase1.md`, fixture `tests/fixtures/tetris_oracle_laya_en_v4.jsonl` (blake3 `18e6b260…`, sha256 `caee3293…`).

## What changed upshot-side (2026-09-25)

Plan 609 Phase 1 measured the next-piece preview lane: flip fraction 75/120
(62.5% — the preview IS a real oracle input under the two-line envelope), but
the crossed head **failed G1** (argmax 332/840 vs the spot-only comparator's
383/840, board-grouped holdout) → **no crossed-head promotion**. The
envelope control also quantified the board-blind premise: the state line
alone moves the oracle to 24/120 argmax agreement with the v3 option-only
labels.

Consequence for this repo: the previously-planned crossed-head serving
(config knob, within-board gate, wasm head regen for a crossed head) is
CLOSED. **Do not serve two-line tetris payloads to the existing served head**
— the served spot head is calibrated on option-only inputs; a two-line
payload would unpin it.

## Tasks

- [ ] `fixture_pins()` four-hash: add the v4 fixture digest (blake3
      `18e6b2604a2f01433a5f7d860b7c98009fa1f41ad75be5c35ee251717ac52903`,
      sha256 `caee3293674365be76977c512108ffe0c4fc5d805d96300ef1553f98f4bc664c`)
      beside the v2/v3/flappy-v3 pins — never length-only (the 885 lesson).
      NOTE: coordinates with the in-flight `fixture_pins()` hardening WIP
      (same file) — land together with or after that lane, never over it.
- [ ] Parser note for any future fit from these fixtures: katgpt-rs's
      anchors are computed under `serde_json/float_roundtrip` (exact parse).
      The tetris structured head digest under the DEFAULT parser differs
      (`65409c14…` vs `b3c91ee0…` — Bench 890 §G3 finding); a fit here that
      must land on the katgpt-rs-side head bytes needs the same feature.
- [ ] Two-line tetris serving + any arena-wide re-oracle under the v4
      envelope is a NEW PROTOCOL PROPOSAL (it changes every lane's inputs,
      not just tetris) — file it in katgpt-rs if ever wanted; not a task
      here.
