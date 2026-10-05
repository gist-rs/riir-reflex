# Issue 070 — the modelless eval path's allocation surface: `eval_seat` internals are ~69 of the bag serve path's 83 allocs/decision

**Status:** OPEN (lead intake — measured; no heal applied here)

Filed from riir-instinct Issue 021's close-out (2026-10-05): instinct's
serve-path G4 (`tests/serve_g4_alloc.rs`, born this issue) measured the
bag server's `decide` at **107 allocs/decision** baseline → **83** after
instinct's ownership/scratch refactor. The decomposition attributes the
remaining ~69 to THIS repo's eval path — the largest class on the serve
path, and the same path the arena benches run over every suite.

## The measured attribution (sst5, 5 options, 1 question/case)

- `decide()` prelude template clones (instinct, borrow-bound): ~8
- instinct's bridge pass + case-into + receipt tier: ~6 (post-refactor)
- **`eval_seat` internals (this repo): ~69** — `eval_engine`'s per-case
  result machinery (probs/picks/confs/abstained Vecs, the request
  path, the gate calibration) PLUS the `SeatEval` copy layer
  (`eval_seat` re-collects every probs Vec into `QuestionOut`s and
  clones `durs_us` — the raw eval result is dropped unread).

Two consumer faces pay it:

1. **The serve path** (instinct `decide_multi`, the ESC cheap tier +
   gate `flags()` — the gate reads ONLY `abstained` and still pays the
   full `SeatEval` copy).
2. **The arena/harness** (every suite eval in the benches — the same
   `eval_engine`).

## Leads (this repo's classes)

1. **The `SeatEval` copy layer**: `eval_seat` clones every per-question
   probs Vec + the durations into a convenience view. A leaner entry
   (`eval_engine` is `pub(crate)` — widen, or add a flags-only /
   into-scratch variant) would let the gate leg read `abstained`
   without the copy, and let serve consumers reuse buffers
   (`eval_seat_into`).
2. **`eval_engine`'s per-case result Vecs**: rebuilt per call on the
   serve path (one case per request). The scratch-buffer shape the
   encoder lane already uses (`y` scratch) is the in-repo precedent.
3. Measurement discipline: instinct's counting-allocator pattern
   (`tests/serve_g4_alloc.rs` + Rethink's `tests/esc_g4_alloc.rs`,
   pinned 83 / 161) is the gate — any eval-path change must keep both
   pins green (downward is the intended direction; upward reds).

**Anti-goal**: no behavior change — the A0 byte-for-byte parity law
(the frozen picks replays) and the arena's eval semantics are the
contract; the refactor is allocation-shape only.
