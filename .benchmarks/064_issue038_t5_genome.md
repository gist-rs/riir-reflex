# Bench 064 — issue 038 T5: the joint blend-genome selection (`--genome-select`)

**Status:** RECORD — T5 steps 2+ LANDED; 2 adopted postures (banking77, sst5), 2 rejections (massive G1, emotion accuracy); RRF declined.

M3 (AC, quiet-ish — one sibling fetch + one foreign cargo test running; accuracy
work, latency cells are same-class not comparison-grade), `datasets_t20k`,
release binary, all selections on (head/nb/oc/ridge) + `--genome-select`.

## What landed

- `DecisionEngine::set_blend_scales` — scoring-time scale moves on a built
  engine, fail-closed (`EngineError::ScaleNotFitted`: a positive scale over
  tables the build never fitted refuses). The search builds ONCE per
  (α, view) with every lever FITTED (positive placeholder scales), then a
  coordinate eval is one sel-slice scoring pass — no rebuild.
- `src/harness/runner/genome_lane.rs` — genome = `{route, head,
  nb(+α,+view), oc, ridge}`; pure `descend` core (coordinate descent,
  per-candidate noise floor 0.01, ties hold; unit-tested on synthetic
  surfaces), acceptance-vs-seed bar disclosed per run
  (`--genome-accept-margin`, default 0.05 = the house arming bar).
  λ fixed (`RIDGE_LAMBDA=10.0`, build-time), noul polarity fixed
  (already cal-selected).
- Protocol: cal-slice walk only; ONE test read per run at the walk end;
  genome-off runs byte-identical (`skip_serializing_if` on the row field);
  held ⇒ `default_cfg` untouched ⇒ plain rebuild = the published posture.

## Round 1 — house bar 0.05: ALL HELD (G3 proven)

Every suite's walk end cleared the seed by less than 5 pt ⇒ every test row
reproduced the published state byte-exactly (typed .4655, ag_news .8825,
emotion .8850, sst5 .3967, prompt .7672, xnli .5233, massive .7800,
banking77 .8260 — all == published). The walks still surfaced cal-side
interaction gains of +3.0..+4.0 on four suites — all with moves the greedy
per-lever ladders could not reach (notably ridge@16, a rung the published
ridge ladder never had — the round-2 saturated-top lesson repeating).
Seed evals reproduced the lanes' own ladder tops exactly (cross-lane
consistency check).

## Round 2 — pre-registered refinement bar 0.03 (4 suites accepted)

The 0.05 bar is the house ARMING bar (lever vs off); the genome compares
two already-armed postures — a refinement comparison. 0.03 ≈ 6 questions
at n=200, pre-registered before the test read, same shape as round 2's
ladder extension. Verdicts (test read once, per suite):

| suite | genome posture | test | Δ vs published | G1 | verdict |
|---|---|---|---|---|---|
| banking77 | ridge 0→1, head 1→0 | **0.8620** | **+3.60 pt** | PASS (ece .0222 vs floor .3268) | **ADOPTED** |
| sst5 | ridge 0→8 | **0.4017** | **+0.50 pt** | PASS (ece .0033 vs floor .2062) | **ADOPTED** |
| massive_intent_en | head 0→2, ridge 0→16 | 0.8333 | +5.33 pt | **FAIL** (ece .2383 vs floor .1361) | REJECTED — G1 |
| emotion | head 0→2, nb 4→off, ridge 8→16 | 0.3500 | **−53.50 pt** | pass (moot) | REJECTED — accuracy collapse |

Held suites (typed/ag_news/prompt/xnli) byte-identical to published ✓.

## The two rejections are the finding

- **emotion (−53.5):** the walk turned the NB tables OFF — the term that
  carries emotion's signal since T1 (+31 pt) — because ridge@16 + head@2
  scored 0.800 on the 200-case cal slice. On test the posture collapsed to
  0.35 (≈ the pre-NB 045 engine +7). Mechanism (hypothesis, labeled): the
  ridge closed-form fit overfits its own training-pool distribution; the
  count tables generalize. The house 5 pt bar would have refused this walk
  end (+4.0 cal < 5.0) — the bar is the guardrail, and the 0.03 refinement
  round paid for the lesson: a refinement bar admits cal-overfit postures
  that an arming bar refuses.
- **massive (G1 fail):** +5.33 pt accuracy with readout ECE 0.2383 against
  the conformal-naive floor 0.1361 — the UQ-bearing law binding: an
  accuracy gain that breaks the calibration floor is a failed gate. The
  published posture (ECE .0796 vs floor .1209) stands. The accuracy column
  alone would have called this the round's biggest win.

## Adoption + publication state

- Adopted posture state: banking77 (ridge@1, head off) 0.8620 and sst5
  (ridge@8) 0.4017 — both G1-passing, both determinism-ok, sub-ms p50.
- Site republish DEFERRED: `publish_bench.py` publishes a whole-run lane
  state and has no G1 filter — a wholesale r3 publish would carry the
  G1-failed massive row. The next whole-run opportunity (post-041 ag_news
  full-pull re-run, which re-selects ag_news anyway) adopts the genome
  lane with the winners and republishes then.
- 4090 cross-host verification: **PASS** (LAN `192.168.1.36`, bundle-synced
  to `b1c85fe` — the 4090's outbound GitHub fetch hangs; the M3 bundle
  `5dc2593..develop` transfers the commits directly; `b1c85fe` carries zero
  `src/` deltas over this lane's `bc0a3fb`, so the run is the same code).
  All 14 suites: identical accuracies, identical G1 verdicts, identical
  genome walk decisions (held/moves) — including all four genome postures
  (emotion 0.3500, sst5 0.4017, massive 0.8333, banking77 0.8620). The
  only diffs are ulp-scale float noise in the soft metrics (ece/nll/brier,
  the documented aarch64-vs-x86_64 reduction-order class — the same fields
  the cascade lane's G3 excludes). Decision-bearing state byte-equal.

## Gates

- G1: per-suite verdicts above — adopted rows PASS; massive REJECTED on it.
- G2: all suites sub-ms p50 (max 552 µs typed, unchanged posture); the
  genome adds no hot-path cost (`set_blend_scales` is config-only).
- G3: held suites byte-identical to published; genome-off runs byte-identical
  (row field skipped when None).
- G4: no hot-path change; solve_into untouched.

## RRF (issue 038 T5 "optionally fuse with RRF") — DECLINED

riir-rag's own RRF doc names the decisive case: additive fusion is strictly
more informative where the channels share a scale — and the blend terms are
sigmoid-squashed onto [0,1] by design. The genome round shows the additive
blend's remaining wins are posture-interaction gains (extracted by the walk),
not scale-incomparability; and emotion's collapse is posture overfitting,
which rank-only fusion (magnitude-discarding) does not address. Declined on
measured rationale, not on cost.
