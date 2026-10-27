# Issue 101 — run_r1_read.sh lies success on dead legs (rc=127 class) + the launchd PATH gap that caused it

**Status:** PARTIAL — the script-side fixes landed 2026-10-27 11:47 (fmt-hold
lane, rig files, not commits: leg-rc abort-loud + the fail-fast PATH guard in
`run_r1_read.sh`, and the wrapper's attempt-aware retry); the plist
source-side hardening + verify (a) landed 2026-10-27 ~13:2x (zed-amo-blockfix,
this cycle, rig file + box plist). OPEN: verify (b) + the real-chain fire
(waits for the panel-done marker + bench record — see the incident note).
**Claimed:** amo-rig-fix-lane, 20261027052600, expires 2026-10-29T05:26:00Z

## The defect chain (measured 2026-10-27 ~04:23Z, /tmp/r1_after_window3.log)

1. **Root cause — the launchd PATH gap.** `~/Library/LaunchAgents/com.riir.r1-after-window3.plist` launches `r1_after_window3_v3.sh` with the launchd default env, which has NO `~/.cargo/bin` on PATH. The wrapper's handoff ran `run_r1_read.sh`, whose 10 cargo legs (`cargo run --release --bin harness`, 9 suites + perm-probe) ALL died `rc=127` (`cargo: command not found`) in 16 s (04:23:25 → 04:23:41).
2. **The lie — rc-propagation.** `run_r1_read.sh` does not aggregate leg failures: dead legs left the script exit **0**, so the wrapper's `[ "$rc" -eq 0 ] && break` took the SUCCESS branch on attempt 1. The sentinel was withheld only because the SECOND layer (`analyze_r2_items.py` over empty/partial `.benchmarks/opendecider_r1_optionsort/`) failed — the two-layer design held by luck, not by the script's own verdict.
3. **The recovery was manual.** The wrapper exited 0 → launchd `KeepAlive SuccessfulExit=false` never relaunches on success → the chain sat dead until a session diagnosed it (~04:47Z), repaired PATH, and relaunched; the fresh watcher (pid 93329) re-ran the read for real (outputs verified on disk; Bench 166 adjudicated from them).

## Why this recurs

The rig re-runs for every future window/read (window 4, the next distill capture, any post-reboot chain). A reboot or launchd re-load drops the interactive PATH; nothing in the repo or the plist pins `~/.cargo/bin`. And because the script lies success, EVERY failure mode inside the legs (not just PATH — a build break, a port conflict, a venv loss) lands as a silent no-op read with a withheld sentinel instead of a loud relaunch.

## Fix (recipe — the scripts are untracked `.raw/` rig files + a box-level plist, so this is the owning lane's / owner's act)

- [ ] `run_r1_read.sh`: aggregate leg rc — any leg `rc≠0` (or any missing output dir) → remember a FAIL flag; at end, exit non-zero when FAIL (per-suite crash isolation stays: continue the remaining legs, fail the script).
- [ ] Same treatment for the perm-probe leg (currently its rc is echoed, not propagated).
- [x] PATH hardening in BOTH places: `export PATH="$HOME/.cargo/bin:$PATH"` at the top of `run_r1_read.sh` (defense at the victim — landed 11:47 with the leg-rc fix), and `EnvironmentVariables` on `com.riir.r1-after-window3.plist` (defense at the source — landed ~13:2x: `PATH=/Users/katopz/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`, backup `plist.bak-20261027`, `plutil -lint` OK, bootout+bootstrap reloaded, `launchctl print` shows the job env carrying the hardened PATH).
- [x] Verify (a): poisoned-PATH invocation (`env PATH=/usr/bin:/bin sh run_r1_read.sh`) — the script REFUSED at the fail-fast cargo guard **rc=2** before any serve boot (measured ~13:25Z). NOTE: the 11:47 victim-side guard works; the plist env was verified independently via `launchctl print` (the hardened PATH is in the job env).
- [ ] Verify (b): legs failing mid-read must exit non-zero so `KeepAlive SuccessfulExit=false` relaunches — the abort-loud paths exit 2 by construction; the LIVE proof rides the next real-chain fire.
- [ ] Re-check `r1_after_window3_v3.sh`'s "R1 failed rc=$rc after $attempt attempt(s)" message — it printed `rc=0` in the failed case above (cosmetic, but it misleads the log reader exactly when the log matters).

## Boundary

`.raw/opendecider-local/` + `~/Library/LaunchAgents/` are rig/box-level, not repo surface — no tracked code changes; this issue is the record + recipe. Cross-ref: Bench 166 (the adjudication that survived the incident), `HISTORY.md` (Issue 099 arc, same box-day), the `riir-refine` snapshot rows for the fmt-hold lane.

## Incident note (zed-amo-blockfix cycle, ~13:2x +07)

While verifying the reload, a poisoned-PATH verify was ACCIDENTALLY run without
the poison (the invocation inherited the full env): the script PASSED its
guards (recapture marker present, no watchdog, no serve, :8000 free) and
started a REAL R1 read; the tool-call timeout then killed the process group
mid-`ag_news` (serve dead, no orphan). Consequences + disposition:

- `.benchmarks/opendecider_r1_optionsort/typed_decisions/` was OVERWRITTEN by
  a same-posture re-run (rc=0, 4000 items, 2 lanes, no absences — complete);
  `ag_news/` was PARTIAL and has been REMOVED. The other suites keep the
  04:47Z recovery-read vintage (the Bench 166 adjudication's numbers live in
  the bench record, which is the citation of record).
- The dir is MIXED-VINTAGE until the real R1 read re-runs; do NOT adjudicate
  from it before that.
- The real R1 read's precondition is NOT met yet: `/tmp/elephant_panel_done.marker`
  is ABSENT (the header requires it + the panel bench record). No re-run was
  attempted — a marker-less read would be a lying artifact by the script's own
  header law.
- Cleanup verified: no orphan serve, `:8000` free, sentinel untouched
  (11:52, predates).
