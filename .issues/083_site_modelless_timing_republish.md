# Issue 083 — site modelless timing re-publish (pending the quiet-box run)

**Status:** OPEN — bench 137's watcher armed (2026-10-09); publish blocked on a preflight-clean m3.

## What the user saw

https://reflex.gist.rs/ home chart renders, for "Reflex · modelless":

> timing failed the loaded-box check — re-run pending, values on /bench/

That row is the honest disclosure that the modelless lane's published cells
carry **no quotable timing**: every m3 run since the 2026-09-24 primary
(`8028a10`) started or ended on a loaded box (sibling agent jobs), so each
refresh published acc-only (the `d3aeeae` load-wall convention; Issue 021's
measured 12×-swing wall held each time). The values sit on /bench/ unverified;
the chart refuses to plot them.

## The plan

- [x] Verify the site posture reproduces digit-for-digit on this box (sst5
      0.3967, s1mb_score 0.4982497082847141 + corpus digest — bench 137 RECORD)
- [x] Rebuild the harness at HEAD (`27ca2d9`; `a53ce4b`'s `nb_pair_scale` is
      opt-in default-0 = byte-identical)
- [x] Arm `scripts/quiet_box_timing_rerun.sh` (bench 137) — fires the 13-suite
      modelless run the moment `scripts/bench_preflight.sh` passes; stamps
      `READY_TO_PUBLISH.md` when the doc's box_state is quotable both ends
- [ ] Quiet-box run completes (watcher; m3 was carrying 15–20h sibling jobs at
      arming — `svd_lbit_eval`, `hyperthink_t1_delta_census`)
- [ ] Publish: `../reflex-site/scripts/republish_bench.sh
      ../reflex-site/data/bench.json
      .benchmarks/137_modelless_timing_quiet/results.json` (accuracy
      digit-match is the determinism witness; drift = posture moved, STOP)
- [ ] Add the `data/changes.json` row (the clef 2026-10-04 graduation
      precedent), commit + push reflex-site, `npx wrangler deploy`
- [ ] Verify the home chart plots the modelless p50s (presence row retires)
      and /bench/ cells read "quotable"

## Kill switches / guards

- The watcher never publishes; it leaves the doc + a READY note.
- `STOP` file in the bench dir cancels the watcher cleanly.
- Provenance guard: refuses to fire if reflex HEAD moves or `src/`/`Cargo.toml`
  goes dirty after arming (a stale binary stamping a newer sha is a
  provenance lie) — re-arm after rebuilding + re-verifying digit-match.
