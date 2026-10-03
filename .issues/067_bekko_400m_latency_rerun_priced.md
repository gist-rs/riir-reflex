# Issue 067 — bekko 400M latency re-run: what blocked it, what a window costs (065 T1, priced)

**Status:** OPEN — scoped pricing of 065 T1; the lane's board cells are accuracy-complete (9/9 published on /bench/), only the 9 latency cells are unquotable. Filed 2026-10-03 from the owner's ask on the reflex.gist.rs home row "timing failed the loaded-box check — re-run pending, values on /bench/".

## What block (the owner's question, answered)

Not a harness failure — the **Issue-021 box-state wall, working as designed**. Bench 107
(`.benchmarks/107_bekko_v0_400m_board/RECORD.md`) measured the 400M board seat on the M3
with the launch preflight GREEN (`PROVENANCE: power=AC Power load=3.89 swap=2499.25M
canary=118.2us/best5 powermode=2(high)` — PASSED), but a sibling session compiled on the
box THROUGHOUT the run: run 1 load 5.01 → 9.05, run 2 9.05 → 7.32. The end-of-run box
state reads NOT QUOTABLE, so every bekko timing cell carries `latency_quotable: false` →
the lane-scoped publish shipped as `:acc-only` (accuracy is box-state-independent — that
is why /bench/ has the values) and the home latency chart renders the presence label
instead of a bar. Recorded context, never quoted: bekko-400M p50 134–818 ms/question
subprocess round-trip (IPC included, the laya-python measurement law).

## What a window costs (the owner's question, answered)

Measured in Bench 107 (the accuracy run doubles as the timing run — same harness pass):

- Run 1 (7 canonical suites): **15.0 min** bekko subprocess wall (banking77 443 s and
  massive 119 s dominate).
- Run 2 (typed_decisions + code_fixtures, the oc-armed addendum flags): **6.3 min**.
- **Total ≈ 21 min of lane wall** (≈19–20 min total harness wall) for all 4,648
  questions, M3 CPU posture. No 4090 needed — the record's own verdict: the box met the
  wall on the M3.

The gate is the part that failed last time: **launch preflight alone is insufficient** —
Bench 107's launch preflight passed at load 3.89 and the run still went unfit because a
sibling compile spiked the load mid-run. The re-run needs the box QUIET for the whole
~21 min, verified at BOTH ends.

## Tasks

- [ ] T1 Quiet-box gate: `scripts/bench_preflight.sh` green, no sibling compile jobs
  (`uptime`; `ps aux | grep -E "cargo|riir|clippy" | grep -v grep`), quote the
  `PROVENANCE:` line in the new record.
- [ ] T2 Re-run the two Bench-107 invocations verbatim
  (`.benchmarks/107_bekko_v0_400m_board/run.sh`) into a fresh
  `.benchmarks/<NNN>_bekko400m_timing/` out dir (allocate the bench number at run time).
  Expected: 9 cells flip unfit → quotable; the accuracy cells byte-identical to the
  published 400M board (the determinism-witness law — the second independent process
  agreeing with Bench 107 is the witness).
- [ ] T3 Check the END box state is quotable BEFORE publishing (Bench 107's exact
  failure mode). If unfit again, the window didn't take — re-run; never publish.
- [ ] T4 Publish lane-scoped WITHOUT `:acc-only` this time
  (`PUBLISH_BENCH_LANES="bekko" python3 ../reflex-site/scripts/publish_bench.py
  <results.json> ../reflex-site`), then the reflex-site smokes —
  `chart_render_smoke.cjs`'s p50-family arm pins the named-not-plotted lanes and must
  re-pin when bekko plots (065 T4's step, same commit family).
- [ ] T5 Update 065's verdict table (bekko 0-of-9 → 9-of-9) and close this issue with
  065 T1 in the same commit.

## Non-goals

- No change to the Issue-021 verdict rule, the publisher's walls, or the site's
  quotable-only plotting.
- The 68M lane stays retired; the 400M seat move stands.
- The 4090-host lanes' unjudged timing is 065 T3's question, not this one.
