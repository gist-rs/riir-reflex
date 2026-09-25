# Bench 036 — the publication-noise study: cross-process single-run tables are ±20-40% on typed_decisions and the python lane, stable ±5-8% on the short suites; the arena table republished anyway (stale by a build generation is worse)

Five same-binary-or-equivalent full/partial harness runs on 2026-09-25 (m3,
metal, `--laya-python` where present), chasing the Issue-020 handoff item
"regenerate the published arena table from a clean-window run":

| run | binary | typed·english rust p50 | note |
|---|---|---|---|
| closing2 | rhoistab (902f409 + 9a4e17a) | 282 | preflight-REFUSED (load 6.14) — directional |
| closing3 | rhoistab | 286 | preflight PASS |
| closing4 | rhoistab | 349 | preflight PASS, cold-first-suite signature |
| arena_run | issue020_arena (d69f0c7 + 06ee254, warmup commit) | 421 | preflight PASS, 15 suites, the run PUBLISHED |
| probe | issue020_arena | 488 | typed_decisions only, after 15 min GPU-idle cooldown |
| old-bin re-run | rhoistab | 312 | back-to-back with the new bin, same minutes |
| new-bin re-run | issue020_arena | 330 | the back-to-back pair |

## Findings

1. **The substrate is NOT slower.** `laya_fixture_timing` at every
   checkpoint reads its recorded speed right now: english 28.6 (recorded
   28.3), typed 26.9 (28.3), multilingual 12.3 (12.2) — burst AND sustained
   (60 reps = 1500 forwards: 27.1, faster than the burst). The GPU lane,
   sgemm instances, and band predicate are all healthy. Recorded-speed
   probes do not predict harness-section latency.

2. **typed_decisions rust cells wander ±20-40% across processes with no
   box-state signature.** The five readings (282→488) do not correlate with
   preflight load (all four preflight-passing runs span 286-488), idle
   cooldown (the 488 came AFTER cooldown), binary identity (the old binary
   re-run reads 312 where it read 282-349 this morning), or sustained-load
   thermal (the sustained fixture probe shows none). The back-to-back
   old/new pair (312/330) brackets zero: the binaries are equivalent, the
   measurement is the noise. The morning's 282/286 agreement was two
   samples of a wide distribution, not a stable regime — the same
   consistency-is-not-reproducibility trap as katgpt-rs arm_reach's five
   capped runs.

3. **The python oracle lane is noisier than the rust lane and INDEPENDENTLY
   so.** Absolute py cells across the four complete runs: py/permissions
   33/33/39/22, py/tool_fit 25/25/31/16, py/typed 277/249/264/326,
   py/english-typed 352/405/262/287. The Δ-sign flips on the small cells
   are python-side variance — the rust side reads 24/25/20/22/28 across the
   same runs (±8%). Bench 035's per-cell Δ ranges mix a stable numerator
   with a wandering denominator; the durable statement is absolute-side
   stability, not the Δ.

4. **The short-suite rust cells ARE stable** (±5-8% across all runs):
   ag_news 30-34, banking77 66-72, code_fixtures 64-67, the harness_* rust
   cells 24-30. The cross-process noise is a typed_decisions + python-lane
   phenomenon.

5. **Sibling compile activity is the standing suspect for the wander and is
   NOT excluded** — a second agent session committed to this worktree
   mid-study (game_heads fixture pins, a different lane) and sibling cargo
   builds share memory bandwidth with the harness's long-seq sections. No
   experiment here separates that from GPU clock residency; per the
   three-mechanism rule, no mechanism is claimed.

## The publication decision

The published arena table (bench.json, 2026-09-24T07:42Z, sha 8028a10)
predates the entire Issue-020 wave: its typed·english rust cell reads
**1124 ms** against today's 282-488 distribution — stale by a build
generation, wrong by 2-4× on the exact cells the wave fixed. Every sample
measured today, including the worst, strictly improves it. The site renders
single-sample tables by design; waiting for a "stable" single run is
waiting for noise to stop, which this study shows it does not do on this
box.

**PUBLISHED: the arena_run table** (d69f0c7, 15 suites, both lanes,
preflight-passing window, box state stamped in TABLES.md). The noise is
disclosed here and in the run's own header; the across-run honest quantity
remains Bench 035's ranges for any claim that needs one.

## Protocol conclusion for future published tables

A single clean-window run is publishable as a freshness update (the stale
table is the alternative), never as claim evidence — claims quote Bench-035
-style across-run ranges or same-process A/Bs. If the site ever needs
honest per-cell uncertainty, that is a site-format change (publish the
ranges), not a measurement polish.
