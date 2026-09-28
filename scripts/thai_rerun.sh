#!/bin/sh
# thai_rerun.sh — the Thai-board latency re-run, ONE command (plan 003 T3.5).
#
# Bench 074's Thai board landed accuracy-only: its own start load (6.95) made
# the run's timing NOT QUOTABLE (the Issue-021 wall), so the reflex-site p50
# chart carries the thai rows with no timing. The fix is a RE-RUN on a fit
# box — never a re-judgment of 074's verdict (that run's state stands).
#
# What this does:
#   1. bench_preflight.sh pre-gate — refuses over MAX_LOAD=6.0, on battery,
#      in Low Power, or on unsettled AC. A refusal here is the wall WORKING;
#      re-run when the sibling sessions are done.
#   2. Re-stages .raw/openthai-systemone (clone @ the pinned sha + uv venv)
#      if missing — the record's research clone is removed after landing.
#   3. Boots their server on 127.0.0.1:$OPENTHAI_PORT (default 8000), waits
#      for /healthz (refuses if the port is already held).
#   4. Runs the harness: modelless + laya-multilingual + openthai on
#      thai_wisesight,thai_sib200 (datasets .raw/datasets — 074's own dir)
#      into the NEXT bench number (.highwater read + written back, the
#      numbering law; a failed run does not un-consume the allocation).
#   5. Kills the server. NEVER commits, NEVER deploys — the runner quotes
#      the PROVENANCE line into the new record, then republishes the site.
#
# The harness ALSO self-judges: its results.json carries box_state with
# latency_quotable + refusals (074's own meta shows the shape) — so a load
# spike mid-run still cannot smuggle a quotable-looking number out.
#
# Usage:
#   scripts/thai_rerun.sh                    # full gate + run
#   OPENTHAI_PORT=8010 scripts/thai_rerun.sh # port conflict escape
#   MAX_LOAD=8 scripts/thai_rerun.sh         # preflight knobs pass through
#
# Exit: 0 = run landed in its bench dir; 1 = refused (gate, stage, or
# harness), reason on its own line. Explicit exits only — no set -u/-e
# abort path for the macOS-sh EXIT-trap laundering class to bite.

root=$(cd "$(dirname "$0")/.." && pwd) || exit 1
cd "$root" || exit 1

PORT="${OPENTHAI_PORT:-8000}"
BASE="http://127.0.0.1:${PORT}"
SERVE_DIR=".raw/openthai-systemone"
PIN_SHA="5d04bcca0c58bd10e7dac2d3d369d8f760bea6cf"
SRV_PID=""

cleanup() {
    if [ -n "$SRV_PID" ]; then
        kill "$SRV_PID" 2>/dev/null
        sleep 1
        kill -9 "$SRV_PID" 2>/dev/null
    fi
    true
}
trap 'rc=$?; cleanup; exit $rc' EXIT

die() { printf '✗ thai_rerun REFUSED — %s\n' "$*"; exit 1; }

# ---- 1. the wall's pre-gate -------------------------------------------------
printf '── 1/5 preflight (the Issue-021 gate; a refusal here is the wall working)\n'
PF_LOG="/tmp/thai_rerun_preflight.$$.txt"
if ! sh scripts/bench_preflight.sh > "$PF_LOG" 2>&1; then
    cat "$PF_LOG"
    rm -f "$PF_LOG"
    die "box not fit — see the preflight lines above; re-run when quiet"
fi
cat "$PF_LOG"
PROV=$(sed -n 's/^PROVENANCE: //p' "$PF_LOG")
rm -f "$PF_LOG"

# ---- 2. stage the server env if the record's clone was removed --------------
printf '── 2/5 server env\n'
if [ ! -x "$SERVE_DIR/.venv/bin/python" ]; then
    printf '   staging %s (clone @ %s + uv sync)\n' "$SERVE_DIR" "$PIN_SHA"
    git clone --quiet https://github.com/iapp-technology/openthai-systemone "$SERVE_DIR" \
        || die "git clone of openthai-systemone failed"
    ( cd "$SERVE_DIR" && git checkout --quiet "$PIN_SHA" ) \
        || die "pin checkout $PIN_SHA failed"
    ( cd "$SERVE_DIR" && uv sync --extra server --python 3.12 > /dev/null 2>&1 ) \
        || die "uv sync failed (uv on PATH? first run may need network for wheels)"
else
    printf '   staged already (%s/.venv)\n' "$SERVE_DIR"
fi

# ---- 3. boot + healthz -------------------------------------------------------
printf '── 3/5 server boot (port %s)\n' "$PORT"
if curl -sf -m 2 "$BASE/healthz" > /dev/null 2>&1; then
    die "port $PORT already answers /healthz — something holds it; set OPENTHAI_PORT"
fi
SRV_LOG="/tmp/thai_rerun_server.$$.log"
nohup "$SERVE_DIR/.venv/bin/python" -m uvicorn openthai_systemone.server:app \
    --host 127.0.0.1 --port "$PORT" > "$SRV_LOG" 2>&1 &
SRV_PID=$!
i=0
ready=""
while [ "$i" -lt 36 ]; do
    i=$((i + 1))
    sleep 5
    if curl -sf -m 3 "$BASE/healthz" > /dev/null 2>&1; then ready=1; break; fi
    kill -0 "$SRV_PID" 2>/dev/null || break
done
if [ -z "$ready" ]; then
    tail -5 "$SRV_LOG"
    die "server never answered $BASE/healthz (log: $SRV_LOG)"
fi
printf '   healthz ok after ~%ss (pid %s; log %s)\n' "$((i * 5))" "$SRV_PID" "$SRV_LOG"

# ---- 4. the run: next bench number + harness ---------------------------------
printf '── 4/5 harness run (modelless + laya-multilingual + openthai, thai suites)\n'
n=$(tr -d '[:space:]' < .benchmarks/.highwater)
case "$n" in
    "" | *[!0-9]*) die "unreadable .benchmarks/.highwater ('$n')" ;;
esac
next=$((n + 1))
out=$(printf '.benchmarks/%03d_openthai_thai_rerun' "$next")
printf '%s\n' "$next" > .benchmarks/.highwater
printf '   allocated bench %03d (.highwater written back; a failed run does NOT\n' "$next"
printf '   un-consume it); out dir %s\n' "$out"
if [ "$PORT" != "8000" ]; then
    OPENTHAI_SERVE_URL="$BASE"
    export OPENTHAI_SERVE_URL
fi
if ! LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --bin harness -- \
    --openthai --datasets-dir .raw/datasets \
    --suites thai_wisesight,thai_sib200 --out "$out"; then
    die "harness failed (bench $next allocated — see its message above)"
fi

# ---- 5. handoff ---------------------------------------------------------------
printf '── 5/5 done (server killed)\n'
printf '   bench dir: %s\n' "$out"
printf '   PROVENANCE (quote in the record): %s\n' "$PROV"
printf '   next: write the .md record (accuracy + NOW-quotable latency, check the\n'
printf '   harness box_state latency_quotable=true), then republish the site:\n'
printf '     cd ../reflex-site && PUBLISH_BENCH_LANES=openthai ./scripts/republish_bench.sh\n'
printf '   (+ deploy) so the thai p50 bars go live.\n'
exit 0
