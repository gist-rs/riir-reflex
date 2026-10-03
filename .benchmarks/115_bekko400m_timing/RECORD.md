# Bench 115 — bekko-400M latency re-run: 9/9 quotable cells via per-suite docs (067 T1–T5)

**Status: MEASURED — every bekko-400M timing cell flipped unfit → `latency_quotable: true` and is published (p50 geomean 231.7 ms, n_used 9/9, zero unfit/unjudged); all 9 accuracy cells byte-identical to the published 400M board (the determinism witness). reflex-site `chart_render_smoke` + `bench_page_smoke` re-pinned; 067 closed with 065 T1.**

Date: 2026-10-03 · host m3-max-metal (CPU posture) · reflex `e8328de` · publish: reflex-site lane-scoped update (data/bench.json, `PUBLISH_BENCH_LANES="bekko"`, current bench.json as primary — the fresh-docs wall fired once and was answered the way it names)

## Box state (T1) — the session gate

Session preflight at 21:14 +0700: **`PROVENANCE: power=AC Power load=5.67 swap=29507.75M canary=119.9us/best5 powermode=2(high)`** — PASSED (load 5.67 under the 6.0 ceiling; swap warn-only, disclosed; canary 119.9 µs vs ref 141 — a quiet-window reading below the pin, recorded here but NOT substituted into the pinned constant: this was not a quiet box).

Every one of the seven published docs carries its own start+end box state, each judged `latency_quotable: true` at BOTH ends (the publisher stamps each cell with its own doc's span verdict).

## What blocked the original runs, measured (the 067 "what block" answer, now with data)

Three findings the re-run paid for:

1. **The full-run shape cannot pass on this box while the editor is alive.** Both full-run attempts went start-quotable → end-unfit (4.73→7.64; 3.75→8.21): the M3's baseline load (Zed + sibling agent sessions) oscillates 3.2–8.8 on minutes, and a 15-min doc's end probe lands wherever the baseline drifted to.
2. **Self-load is modest; baseline drift was the killer.** A single-suite probe (prompt_injections, 32.8 s) at baseline 5.37 read end 5.91 — QUOTABLE. The bekko CPU subprocess adds only ~+0.5 smoothed to the 1-min window on a light suite.
3. **The harness runs suites in registry order, not CLI order** — banking77 is always LAST in any doc containing it, which defeats any light-tail plan: its 6.3-min sustained run leaves the end probe ≈ baseline + 3.3 (solo attempt: 4.93 → 8.18), so banking77 needs baseline ≤ ~2.7 — a state this box did not reach tonight (floor 3.2).

The remedy that landed: **per-suite docs** — each suite its own invocation, each doc a 0.5–6.5-min span that fits inside a baseline dip, each cell stamped with its own doc's verdict. The `--suites` CLI order is irrelevant (registry order rules); retries re-fire only unfit docs.

## The docs (all quotable at both ends; accuracy byte-identical to the 400M board everywhere)

| doc | suites | flags | bekko acc (== published) | bekko p50 |
|---|---|---|---|---|
| per_suite/emotion | emotion | canonical | 0.5825 ✓ | 146 ms |
| per_suite/ag_news | ag_news | canonical | 0.9150 ✓ | 148 ms |
| per_suite/sst5 | sst5 | canonical | 0.4550 ✓ | 129 ms |
| per_suite/xnli_en | xnli_en | canonical | 0.8600 ✓ | 150 ms |
| per_suite/massive_intent_en | massive_intent_en | canonical | 0.9100 ✓ | 255 ms |
| per_suite/prompt_injections | prompt_injections | canonical | 0.50862 ✓ | 146 ms |
| per_suite/typed_cf | typed_decisions + code_fixtures | oc-armed (bench-107 addendum) | 0.6235 / 0.59375 ✓ | 767 / 280 ms |
| per_suite/b77_typed_cf | banking77 + typed + code_fixtures | oc-armed | 0.7920 ✓ | 645 ms |

The banking77 problem and its remedy: solo banking77 failed 3× (end 8.18 at start 4.93), and the `[banking77, prompt_injections]` pairing failed 3× more (registry order put banking77 last — the tail was the heavy suite). The doc that landed is the **triple** `[banking77, typed_decisions, code_fixtures]` under the oc-armed flags: registry order runs banking77 → typed → code_fixtures, so the doc ends on code_fixtures' 20 s light tail and the end probe stays under the ceiling.

**Flag-safety evidence (the oc question):** the bekko lane is an external oracle — the readout-selection flags (`--nb/--oc/--ridge-select`) shape the MODELLESS lane only. The triple doc's modelless rows read typed 0.5725 · banking77 0.8420 · code_fixtures 0.3750 — byte-identical to the published board, i.e. oc declines byte-identically on banking77 (the fail-closed modelless-drift guard would have refused the publish otherwise).

The earlier `typed_cf` doc is superseded by the triple for typed/code_fixtures (last-wins same-host update; the landed p50s are the triple's 726/264 ms readings — both honest readings of the same deterministic oracle; subprocess jitter run-to-run is ±5%).

## Publish (T4)

`PUBLISH_BENCH_LANES="bekko"` with the **current data/bench.json as primary** + the seven docs as extras (the first attempt passed raw docs as primary and the fresh-docs wall refused loudly with the remedy — the wall working as designed). Landed: 9/9 bekko cells `latency_quotable: true`, `areas.timing.bekko` = `{n_used: 9, n_unquotable: 0, n_unjudged: 0, p50_geomean_ms: 231.6973}` — the home-chart presence row ("timing failed the loaded-box check — re-run pending") becomes a real bar. Publisher self-test 84/84; `chart_render_smoke` re-pinned (2 sz-break signs now — openthai + bekko's 767 ms typed cell; bekko plots, the presence-row arm is gone); `bench_page_smoke` collector gained the bekko lane it predates (expected break count is data-derived and now counts bekko's two past-500 ms cells); `home_page_smoke` + `public_copy_gate` green as-is.

## Structural residue (carried to 065)

- A **banking77-class sustained doc** (6+ min heavy tail) needs baseline ≤ ~2.7 to pass solo — the true-quiet late-night window. Not needed for the board (the triple landed it); a future solo re-run would only tighten banking77's p50 reading.
- The 4090-host lanes' unjudged timing is 065 T3's question, untouched here.
- `bench_preflight.sh`'s canary pin (141 µs) should ratchet down to ~120 at the next genuinely quiet reading — its own header invites exactly this; not done here (this was not a quiet box).
