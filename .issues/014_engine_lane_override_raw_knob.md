# Issue 014 — the engine lane-override knob `X-Reflex-Lane: raw` (the raw-modelless baseline lane)

**Status:** OPEN — recorded unblock path for katgpt-rs Plan 607's one open
roadmap item (the 3-board arena layout); sibling of `.issues/011`.

## The ask

`/decide` gains a second explicit lane-override value: `X-Reflex-Lane: raw`
(alongside the shipped `X-Reflex-Lane: laya`, Plan 607 T8). The override
makes the serve path SKIP the game-head try and answer from the raw
modelless engine (corpus-is-the-model cosine scoring) — so a client can
ASK for the honest baseline instead of being served the head's answer.

**Why it exists:** the serve path tries the head first (the production
posture, Plan 001). That is correct for players and wrong for the
baseline board: the three-tier arena the plan ratified (T11 Q&A round 3)
is **laya-rust teacher / latent-first fitted head / raw modelless
baseline**, and the baseline cannot be shown live beside the head while
head-first is unconditional — the board would render the head's answers
and call them the baseline.

## Design constraints (inherit T8's lane pattern verbatim)

1. **Explicit client override, default posture unchanged.** No header =
   head-first try, byte-identical. `raw` is opt-in per request.
2. **Fail-closed, never a silent fallback.** `raw` runs the modelless
   engine and reports what IT decided — if that is an abstain (`noul`),
   the abstain IS the answer (it is the baseline's point). The response
   must disclose which lane served it (the per-lane-claims law: never
   answer from a lane that did not run).
3. **`/healthz` lane map gains `raw`** so the site and smokes can
   discover it the same way they discover `laya`.
4. **Unknown lane values keep failing closed** (existing behavior — do
   not widen the accepted set beyond `laya` + `raw`).
5. No new state, no new threads: it is a routing predicate at the same
   seam T8 used, not a second engine.

## What `raw` shows today vs after issue 011

- **Today:** the engine head serves Tetris only (`.issues/011`), so
  `raw` on flappy/lanes questions returns the modelless abstain — the
  honest baseline, exactly what the 3-board layout needs. Tetris under
  `raw` shows the cosine engine's own answer/abstain.
- **After `.issues/011`'s engine-side flappy/lanes serving lands:** the
  DEFAULT posture serves those heads, and `raw` remains the explicit
  escape hatch that shows the engine without them. The knob is
  independent of that TODO and unblocks the layout either way.

## Acceptance

1. Paired smoke pins BOTH directions: a flappy (and a lanes) `/decide`
   with `X-Reflex-Lane: raw` returns the modelless-lane answer (abstain
   today), while the same request WITHOUT the header returns the
   head-first answer. Never a silent fallback between the two.
2. `/healthz` lists `raw` in the lane map.
3. `X-Reflex-Lane: bogus` still refuses (fail-closed preserved).
4. The 3-board arena layout (reflex-site, Plan 607 roadmap) unblocks on
   this — the site files its layout change once `/healthz` advertises
   the lane.

## Cross-refs

- katgpt-rs `.plans/607_modelless_game_lane.md` — T11 addendum Q&A
  (the ratified three-tier arena) + the open roadmap item this closes.
- `.issues/011` — the lanes/flappy engine-side serving TODO (separate
  work; this knob does not depend on it).
