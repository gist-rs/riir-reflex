#!/bin/sh
# quiet_box_timing_rerun.sh — arm the Issue-021 quiet-box timing re-read.
#
# The site's bench tables refuse latency from a loaded box (the measured
# 12x-swing law, reflex Issue 021), so a timing debt left by an acc-only
# publish can only be retired by a run that starts AND ends preflight-clean.
# Sibling agent jobs can hold the box for hours; this watcher polls until
# the box is quiet, runs the harness, and stamps the bench dir READY when
# the doc is quotable. It NEVER publishes — the publish is review-gated by
# design (republish_bench.sh prints the human steps).
#
# Usage (detached):
#   nohup scripts/quiet_box_timing_rerun.sh <out-dir> -- <harness args...> \
#       >> <out-dir>/watcher.log 2>&1 &
#
# Controls:
#   <out-dir>/STOP   touch it to make the watcher exit at the next poll.
#   <out-dir>/.armed-stamp
#                     written at arm time: the git tree stamp of src/ + the
#                     blob stamp of Cargo.toml the target/release/harness
#                     binary was built against. Each poll REFUSES to fire if
#                     the stamps moved (committed or dirty — either way the
#                     binary no longer describes the tree, and the harness
#                     stamps the RUNTIME HEAD into results.json: a stale
#                     binary stamping a newer sha is a provenance lie).
#                     Docs/bench/issue commits do NOT trip it — only a
#                     src/ or Cargo.toml change does. Wipe the stamp (or
#                     pass --rearm) to re-arm after a deliberate rebuild.
#   MAX_WAIT_H       hours to wait before giving up loud (default 48).
#   POLL_S           poll interval (default 60).
#   MAX_ATTEMPTS     quotable-run attempts before giving up (default 3) —
#                    a sibling returning mid-run fails the END box_state
#                    and the attempt is honestly unquotable; retry later.
#
# Exit codes: 0 = quotable doc written + READY_TO_PUBLISH.md stamped;
# 1 = gave up (MAX_WAIT / attempts / provenance guard / STOP); 2 = usage.
set -eu

if [ "$#" -lt 3 ] || [ "$2" != "--" ]; then
    echo "usage: $0 <out-dir> -- <harness args...>" >&2
    exit 2
fi

rearm=0
for a in "$@"; do
    if [ "$a" = "--rearm" ]; then rearm=1; fi
done

OUT_DIR=$1
shift 2
REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
POLL_S=${POLL_S:-60}
MAX_WAIT_H=${MAX_WAIT_H:-48}
MAX_ATTEMPTS=${MAX_ATTEMPTS:-3}

mkdir -p "$OUT_DIR"
OUT_ABS=$(CDPATH= cd -- "$OUT_DIR" && pwd)
PID_FILE=$OUT_ABS/watcher.pid
STAMP_FILE=$OUT_ABS/.armed-stamp

# The binary-describes-tree stamp: src/ tree hash + Cargo.toml blob hash at
# HEAD, plus a dirty-worktree marker. Recorded once at arm time; every poll
# re-derives it and refuses on any change.
src_stamps() {
    dirty=x
    if git -C "$REPO_ROOT" diff --quiet -- src Cargo.toml; then dirty=; fi
    printf '%s %s%s\n' "$(git -C "$REPO_ROOT" rev-parse HEAD:src)" \
        "$(git -C "$REPO_ROOT" rev-parse HEAD:Cargo.toml)" "$dirty"
}

if [ "$rearm" = 1 ] || [ ! -f "$STAMP_FILE" ]; then
    src_stamps > "$STAMP_FILE"
    rm -f "$OUT_ABS/GAVE_UP"
fi
ARMED_STAMP=$(cat "$STAMP_FILE")

if [ -e "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
    echo "refusing: a watcher is already running (pid $(cat "$PID_FILE"))" >&2
    exit 1
fi
echo $$ > "$PID_FILE"
trap 'rc=$?; rm -f "$PID_FILE"; exit $rc' EXIT

say() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

give_up() {
    say "GAVE UP: $*"
    echo "$*" > "$OUT_ABS/GAVE_UP"
    exit 1
}

deadline=$(( $(date +%s) + MAX_WAIT_H * 3600 ))
attempt=1
say "armed: out=$OUT_ABS stamp=$ARMED_STAMP poll=${POLL_S}s max_wait=${MAX_WAIT_H}h attempts=$MAX_ATTEMPTS"
say "harness args: $*"

while [ "$(date +%s)" -lt "$deadline" ]; do
    if [ -e "$OUT_ABS/STOP" ]; then
        say "STOP file present — exiting cleanly (remove READY expectations; nothing ran past this point)"
        exit 1
    fi

    # provenance guard: the binary must match the tree it will be stamped from
    now_stamp=$(src_stamps)
    if [ "$now_stamp" != "$ARMED_STAMP" ]; then
        give_up "src/ or Cargo.toml moved since arming ($now_stamp != $ARMED_STAMP) — re-arm the watcher at the new posture (rebuild, re-verify digit-match, then re-run with --rearm)"
    fi

    if ! (cd "$REPO_ROOT" && bash scripts/bench_preflight.sh > "$OUT_ABS/preflight.last" 2>&1); then
        say "preflight refused (attempt $attempt) — $(grep -m1 '^REFUSE\|load average' "$OUT_ABS/preflight.last" || echo see preflight.last)"
        sleep "$POLL_S"
        continue
    fi
    prov=$(grep -m1 '^PROVENANCE:' "$OUT_ABS/preflight.last" || true)
    say "preflight PASSED — $prov"

    say "firing harness (attempt $attempt): $*"
    (cd "$REPO_ROOT" && REFLEX_BENCH_HOST=${REFLEX_BENCH_HOST:-m3} ./target/release/harness "$@") \
        || { say "harness exited non-zero — see log above"; sleep "$POLL_S"; attempt=$((attempt+1)); [ "$attempt" -le "$MAX_ATTEMPTS" ] || give_up "harness failed $MAX_ATTEMPTS times"; continue; }

    quotable=$(python3 - "$OUT_ABS" <<'EOF'
import json, sys, glob
docs = sorted(glob.glob(sys.argv[1] + "/results*.json"))
latest = docs[-1] if docs else None
try:
    d = json.load(open(latest))
    print("true" if (d["meta"]["box_state"].get("end") or {}).get("latency_quotable") else "false")
except Exception:
    print("unreadable")
EOF
)
    case "$quotable" in
        true)
            {
                echo "# READY TO PUBLISH — quiet-box timing doc (bench $(basename "$OUT_ABS"))"
                echo
                echo "- stamped: $(date -u +%Y-%m-%dT%H:%M:%SZ) · attempt $attempt · $prov"
                echo "- publish (review first, then):"
                echo "    ../reflex-site/scripts/republish_bench.sh ../reflex-site/data/bench.json $OUT_ABS/results.json"
                echo "- the doc carries the modelless lane only (a --skip-laya run); accuracies must"
                echo "  digit-match the published cells (determinism) — if they drifted, the posture"
                echo "  moved: STOP, the cross-host gate will refuse, re-verify at the new posture."
            } > "$OUT_ABS/READY_TO_PUBLISH.md"
            say "QUOTABLE — READY_TO_PUBLISH.md stamped"
            exit 0
            ;;
        false)
            say "run completed but END box_state judged latency NOT QUOTABLE (a sibling returned mid-run) — attempt $attempt consumed"
            ;;
        *)
            say "results.json unreadable — attempt $attempt consumed"
            ;;
    esac
    attempt=$((attempt + 1))
    [ "$attempt" -le "$MAX_ATTEMPTS" ] || give_up "$((attempt - 1)) attempts without a quotable doc"
    sleep "$POLL_S"
done

give_up "MAX_WAIT (${MAX_WAIT_H}h) elapsed without a preflight-clean window"
