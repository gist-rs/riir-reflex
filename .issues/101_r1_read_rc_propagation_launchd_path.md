# Issue 101 — run_r1_read.sh lies success on dead legs (rc=127 class) + the launchd PATH gap that caused it

**Status:** OPEN — unclaimed, filed 2026-10-27 ~12:2x +07 by the fmt-hold lane after the sub-lane's collision-guard report; incident already recorded in Bench 166 (`8671409`) and `HISTORY.md`; this issue owns the FIX.
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
- [ ] PATH hardening in BOTH places: `export PATH="$HOME/.cargo/bin:$PATH"` at the top of `run_r1_read.sh` (defense at the victim), and `EnvironmentVariables` (or a `ProgramArguments` wrapper through the user's login shell) on `com.riir.r1-after-window3.plist` (defense at the source).
- [ ] Verify the fix two-sided: (a) a poisoned-PATH invocation of `run_r1_read.sh` with the guards unsatisfiable must REFUSE (rc=2) not succeed; (b) with legs failing mid-read, the script must exit non-zero so `KeepAlive SuccessfulExit=false` actually relaunches.
- [ ] Re-check `r1_after_window3_v3.sh`'s "R1 failed rc=$rc after $attempt attempt(s)" message — it printed `rc=0` in the failed case above (cosmetic, but it misleads the log reader exactly when the log matters).

## Boundary

`.raw/opendecider-local/` + `~/Library/LaunchAgents/` are rig/box-level, not repo surface — no tracked code changes; this issue is the record + recipe. Cross-ref: Bench 166 (the adjudication that survived the incident), `HISTORY.md` (Issue 099 arc, same box-day), the `riir-refine` snapshot rows for the fmt-hold lane.
