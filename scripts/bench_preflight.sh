#!/bin/sh
# bench_preflight.sh — REFUSE to publish a latency number the box invalidates.
#
# riir-reflex Issue 021. AGENTS.md already says "a latency number without its
# BOX STATE is not a measurement" and names free RAM, commit-vs-limit and
# concurrent jobs. On a LAPTOP that list is incomplete in a first-order way:
# Apple Silicon drops sustained GPU clocks on battery, and this box was
# measured unplugged (09:56:10, 100% -> 48%) through an entire A/B session
# whose AC-taken predecessors read 26.2 ms where the battery ones read
# 33-35 ms on the SAME binary. Power state is not a footnote; it is an arm.
#
# Deliberately a SHELL script with NO exit trap and no temp files: there is
# nothing for an aborted run to launder (AGENTS.md trap-sentinel class), and
# every refusal exits non-zero with the reason on its own line.
#
# Usage:
#   scripts/bench_preflight.sh                # refuse unless the box is fit
#   MAX_LOAD=8 scripts/bench_preflight.sh     # raise the load ceiling
#   SETTLE_MIN=10 scripts/bench_preflight.sh  # longer post-plug-in settle
#   CANARY_REF_US=150 scripts/bench_preflight.sh  # judge the canary (T2)
#
# Exit: 0 = fit to bench (the provenance line is on stdout, QUOTE IT in the
# record); 1 = refused, reason named; 2 = the instrument itself could not read
# the box (never a green zero).

set -eu

MAX_LOAD="${MAX_LOAD:-6.0}"
SETTLE_MIN="${SETTLE_MIN:-5}"
CANARY_BIN="${CANARY_BIN:-./target/release/examples/sgemm_shape_timing}"
# Pinned AC-plugged canary reference, microseconds, shape 317x1024x1024.
# EMPTY until one is taken on AC on a quiet box — an unpinned canary PRINTS
# and never judges, because a reference taken on battery would bless the
# very state this gate exists to refuse.
CANARY_REF_US="${CANARY_REF_US:-}"
CANARY_TOL_PCT="${CANARY_TOL_PCT:-15}"

fail=0
note() { printf '%s\n' "$*"; }
refuse() { note "REFUSE — $*"; fail=1; }

# ---- 1. power source ------------------------------------------------------
ps_out=$(pmset -g ps 2>/dev/null || true)
if [ -z "$ps_out" ]; then
    note "⛔ pmset unreadable — the power axis is UNKNOWN, not fine"
    exit 2
fi
case "$ps_out" in
    *"'AC Power'"*) note "ok   power        AC" ;;
    *"'Battery Power'"*)
        pct=$(printf '%s' "$ps_out" | grep -o '[0-9]\{1,3\}%' | head -1 || true)
        refuse "on BATTERY (${pct:-?}) — Apple Silicon sheds sustained GPU clock off AC; plug in and re-run"
        ;;
    *) note "⛔ power source unrecognised in: $ps_out"; exit 2 ;;
esac

# ---- 2. power mode ---------------------------------------------------------
# `pmset powermode` on a 14"/16" Apple Silicon MacBook Pro is a THREE-state
# enum, not a Low-Power-Mode boolean: 0 = Automatic, 1 = Low Power, 2 = High
# Power. The first version of this gate read "not 0" as Low Power and refused
# the one mode that is BEST for a sustained number (measured: this box's AC
# profile is powermode 2, its battery profile 0). Only 1 refuses; 0 and 2 are
# both fit but are DIFFERENT arms, so the mode is named in the provenance
# line and a canary reference is only comparable within one mode.
pmode=$(pmset -g 2>/dev/null | awk '/powermode/{print $2}' || true)
case "$pmode" in
    "") note "warn powermode    unreadable (disclosed, not assumed)"; pmode_name="?" ;;
    0)  pmode_name="auto"; note "ok   powermode    0 (Automatic)" ;;
    1)  pmode_name="low";  refuse "Low Power Mode is ON (powermode=1) — it caps clocks by design" ;;
    2)  pmode_name="high"; note "ok   powermode    2 (High Power)" ;;
    *)  pmode_name="unknown"; note "⛔ powermode=$pmode is not a value this gate knows (0/1/2)"; exit 2 ;;
esac

# ---- 3. settle window since the last power transition ---------------------
# A box plugged in one minute ago is still carrying the heat it built on
# battery; AC is necessary, not sufficient. ENFORCED, not just disclosed:
# the first version printed the transition and left the arithmetic to a
# reader, who is exactly the person in a hurry to bench.
last=$(pmset -g log 2>/dev/null | grep -E "Using (AC|Batt)" | tail -1 || true)
if [ -n "$last" ]; then
    last_ts=$(printf '%s' "$last" | cut -c1-19)
    last_kind=$(printf '%s' "$last" | grep -oE "Using (AC|Batt)" | head -1)
    note "info last power transition: $last_ts ($last_kind)"
    last_epoch=$(date -j -f "%Y-%m-%d %H:%M:%S" "$last_ts" +%s 2>/dev/null || true)
    if [ -n "$last_epoch" ] && [ "$last_kind" = "Using AC" ]; then
        on_ac_min=$(( ( $(date +%s) - last_epoch ) / 60 ))
        if [ "$on_ac_min" -lt "$SETTLE_MIN" ]; then
            refuse "on AC for only ${on_ac_min} min (< SETTLE_MIN=${SETTLE_MIN}) — still carrying battery-era heat"
        else
            note "ok   settle       on AC ${on_ac_min} min (>= ${SETTLE_MIN})"
        fi
    elif [ -z "$last_epoch" ]; then
        note "warn settle window UNVERIFIED — could not parse '$last_ts'"
    fi
else
    note "warn no power-transition history — settle window UNVERIFIED"
fi

# ---- 4. load --------------------------------------------------------------
la=$(uptime | sed 's/.*averages*: *//' | awk '{print $1}' | tr -d ',' || true)
if [ -z "$la" ]; then
    note "⛔ load average unreadable"; exit 2
fi
if awk "BEGIN{exit !($la > $MAX_LOAD)}"; then
    refuse "load average $la exceeds MAX_LOAD=$MAX_LOAD — a sibling session is on the box"
else
    note "ok   load         $la (ceiling $MAX_LOAD)"
fi

# ---- 5. memory pressure ---------------------------------------------------
swap=$(sysctl -n vm.swapusage 2>/dev/null | awk '{print $6}' | tr -d 'M' || true)
if [ -n "$swap" ]; then
    if awk "BEGIN{exit !(${swap:-0} > 1024)}"; then
        note "warn swap used   ${swap}M — paging can masquerade as a slow kernel"
    else
        note "ok   swap used   ${swap}M"
    fi
else
    note "warn swap usage unreadable"
fi

# ---- 6. GPU clock canary --------------------------------------------------
# The ONLY throttle detector available without sudo on this box: run a fixed
# kernel and read its absolute time. powermetrics needs root; there is no
# sudo-free thermal-pressure sysctl on Apple Silicon here (measured).
canary=""
if [ -x "$CANARY_BIN" ]; then
    canary=$(LAYA_DEVICE=metal "$CANARY_BIN" 2>/dev/null \
        | awk '/317x1024x1024/{gsub("gpu_p50=","");gsub("us","");print $3}' || true)
fi
if [ -z "$canary" ]; then
    note "warn canary       SKIPPED — build it first:"
    note "                  cargo build --release --features laya-riir-metal \\"
    note "                      --example sgemm_shape_timing"
elif [ -z "$CANARY_REF_US" ]; then
    note "info canary       ${canary} us (317x1024x1024) — NO AC REFERENCE PINNED."
    note "                  If this run is on AC and quiet, pin it as CANARY_REF_US."
else
    if awk "BEGIN{exit !($canary > $CANARY_REF_US * (1 + $CANARY_TOL_PCT/100.0))}"; then
        refuse "canary ${canary} us vs AC reference ${CANARY_REF_US} us (+${CANARY_TOL_PCT}% tol) — the GPU is throttled"
    else
        note "ok   canary       ${canary} us vs ref ${CANARY_REF_US} us"
    fi
fi

# ---- provenance -----------------------------------------------------------
prov="power=$(printf '%s' "$ps_out" | grep -o "'[A-Za-z ]*Power'" | head -1 | tr -d "'")"
prov="$prov load=$la swap=${swap:-?}M canary=${canary:-skipped}us powermode=${pmode:-?}(${pmode_name})"
note ""
note "PROVENANCE: $prov"

if [ "$fail" -ne 0 ]; then
    note "✗ preflight REFUSED — do not publish a latency number from this box now"
    exit 1
fi
note "✓ preflight PASSED — quote the PROVENANCE line in the bench record"
