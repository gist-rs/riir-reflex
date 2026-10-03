# Issue 062 — the game-head `routing.reason` carried internal record ids onto the public wire

**Status:** OPEN — source fix LANDED (this commit); waiting on the next release cut, after which reflex-site re-captures its wire examples and drops its allow row.

## Evidence

reflex-site Issue 006 records the engine's HTTP wire verbatim from the release binary
(`scripts/capture_wire.mjs` → `data/wire.json`) and renders it on `/` and `/docs/api/`.
The only `/decide` case the stock binary ANSWERS is a Tetris spot (the game head), and its
response from reflex 0.2.3 reads:

```json
"routing": {"lane": "modelless", "reason": "game-head/tetris (corpus-fitted, Bench 881 decoded arm)"}
```

`Bench 881` is an internal record id. reflex-site's `public_copy_gate.cjs` bans those in
rendered copy, and it went red on both pages. The lanes head (`Bench 880`) and flappy head
(`Bench 882`) carry the same shape. No other serve-path string literal carries an id
(grep of `src/` string literals: the rest are harness report headers, never served).

## Fix

- [x] T1 the three `head_response` reason strings drop the id:
  `game-head/{tetris,lanes,flappy} (corpus-fitted, … decoded arm)`. Every consumer
  (`tests/serve_lanes.rs`, `tests/game_heads_serve.rs`) matches on the `game-head/…`
  prefix only — unchanged.
- [x] T2 `tests/game_heads_serve.rs::assert_public_reason` — no `Bench|Plan|Issue|Proposal N`
  pair in any served head reason, asserted on all three heads. Canary: restoring
  `Bench 881` reds `a_fixture_question_is_answered_from_the_head` with
  `internal record id Bench 881 in public routing reason …`.
- [ ] T3 ships in the next release. Then, in reflex-site: `node scripts/capture_wire.mjs &&
  node scripts/render_wire.mjs`, and delete the `Bench 881` row from
  `WIRE_ID_ALLOW` in `scripts/public_copy_gate.cjs` (a stale row reds that gate, so the
  recapture forces it).
