# Issue 011 — the lanes + flappy game-head unblock paths (the two abstaining arena boards)

**Status:** OPEN — v1 of the game-head serving lane (Plan 001) serves
Tetris only; this records why the other two boards abstain and exactly
what unblocks each.

The arena (reflex.gist.rs/arena/) plays Tetris + Flappy + three-lanes side
by side. The modelless board now plays Tetris out of the box (the decoded
Bench-881 head served over HTTP, 44/120 class). The other two boards still
abstain into a labelled random fallback. Both abstains are MEASURED
honest, not gaps:

## Lanes — the head reads the OTHER lanes

`lanes_decoded_features` (katgpt-rs `examples/common/grammar_tables.rs`)
columns 6–7 count the OTHER lanes' blocked/free status. The published
84/100 head is fitted on all 8 columns, so a serving path that sees only
ONE lane sentence cannot reproduce it, and a sentence→corpus-row lookup is
AMBIGUOUS: "The left lane is clear ahead." appears in corpus states whose
neighbors differ, and the head reads those neighbors.

**Unblock paths (pick one, measure, then serve):**
1. **Joined-state protocol (site change).** The lanes board sends ONE
   `/decide` per turn with `state` = the three option sentences joined in
   pinned lane order (the wire carries multiple questions already). The
   engine splits, decodes all three → `LanesDecoded` → the published head
   per lane → three answers. Preserves the published anchor exactly — but
   must apply to the MODELLESS lane only (the laya lane's measured
   protocol forwards each option sentence ALONE as the state; changing the
   laya request shape would un-measure its lanes play).
2. **Single-lane refit (engine-only).** Refit on columns 0–5 (per-lane
   local features) with the same recipe and MEASURE it (LOO agreement vs
   constant-pick 41/100, distinct picks ≥ 2 — the G1 floor). A new head
   design, publishable only with its own bench row.

## Flappy — the render is the bottleneck (Issue 876's residue, site-side)

The site renders frozen v2 sentences (position band only). The v2-decoded
arm is measured DEGENERATE (Bench 881: 77/100, constant-pick, distinct
picks 1 — G1 FAIL). The v3 render (Bench 882) is measured Δ0 = the
structured arm at 96/100 with a full-digest anchor
(`c93d36dc79c0490334c20353ce5d6479eaee3448b686ac057f8a4ad4b98ae3c5`) —
the engine can port the v3 grammar + the structured-units reconstruction
and serve at the published anchor the day the site renders v3.

**Unblock path (coordinated site + engine):**
1. Site: `assets/games/flappy.js` renders the v3 option sentence (band +
   quantized offset + neutral post-motion); regenerate
   `arena/demo_oracle.json` from `flappy_oracle_laya_en_v3.jsonl` (the
   demo reel is sentence-keyed — v2 keys would miss); the golden test
   binds the v3 fixture.
2. Engine: port the `laya-flappy-v3` option + state tables and the
   `flappy_v3_decoded_features` reconstruction; the option sentence alone
   carries post_rel/post_v, but the head row ALSO needs the state's
   pre-rel/v/h (features 3–4) — so the flappy serving needs the SAME
   joined-state treatment as lanes (the state sentence rides the request)
   OR the fit re-scoped to the option-only columns with its own
   measurement.

Note the asymmetry with Tetris: its spot features are fully self-contained
(one sentence → the whole row), which is why it served first.

## Acceptance

A board leaves the abstain list only when its serving path reproduces a
PUBLISHED anchor (digest or agreement) in `tests/game_heads_serve.rs`.
No serving from an unmeasured fit.
