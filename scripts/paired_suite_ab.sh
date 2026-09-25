#!/bin/sh
# paired_suite_ab.sh — the PAIRED PER-SUITE A/B instrument (reflex issue 020:
# "a paired per-suite A/B is the next instrument" for the every-published-cell
# bar).
#
# WHY THIS EXISTS: Bench 036 measured the publication itself unstable at the
# ±20–40% level on typed_decisions across same-day full runs, and Bench 041's
# single sequential republish (rust p50 +30.6% on typed·english) contradicts
# Bench 035's same-run table (typed·english WINNING by 20–30%). A single
# sequential run cannot adjudicate the bar: the lanes run hours apart in box
# time. This instrument alternates the two lanes SUITE BY SUITE with the
# within-suite order balanced across suites — drift that lands between two
# suites' arms lands on BOTH suites' comparisons in opposite directions, and
# the per-suite medians meet in the middle.
#
# Design (position-balanced, the arena-spot A/B's law at suite granularity):
#   even-indexed suite:  ONE run, both lanes (rust laya first, then py) —
#                        the harness's native within-suite order.
#   odd-indexed suite:   TWO runs — py-only (`--laya-python --skip-laya`)
#                        then rust-only — the order REVERSED.
# Each lane runs first for half the suites. A per-suite verdict table is
# rendered by scripts/paired_suite_ab_summary.py from the saved results.
#
# Usage:
#   scripts/paired_suite_ab.sh [suite,suite,...]
#     default: the stable-band suites from issue 020's closing table
#   OUT_DIR=/tmp/paired_ab_$(date +%s)   (default)
#   LAYA_PYTHON=.raw/laya-env/bin/python  (the torch venv; default python3)
#
# REFUSES to run without bench_preflight.sh passing unless
# PAIRED_AB_NO_PREFLIGHT=1 — and then the summary marks the run
# NOT-FOR-PUBLICATION (mechanical validation of the plumbing only; a
# latency number taken on a loaded box is not a measurement — issue 021).
#
# This script stages NOTHING itself: every invocation writes its own
# results.json under OUT_DIR/<suite>/{both,rust,py}/, and the summary is a
# separate, re-runnable step over those files.

set -eu

cd "$(dirname "$0")/.."

DEFAULT_SUITES="banking77,massive_intent_en,ag_news,sst5,emotion,xnli_en,typed_decisions"
SUITES="${1:-$DEFAULT_SUITES}"
OUT_DIR="${OUT_DIR:-/tmp/paired_ab_$(date +%s)}"

mkdir -p "$OUT_DIR"

# ---- preflight (the issue-021 law) ----------------------------------------
PROVENANCE="(no preflight — PAIRED_AB_NO_PREFLIGHT=1, NOT FOR PUBLICATION)"
if [ "${PAIRED_AB_NO_PREFLIGHT:-0}" != "1" ]; then
    if ! sh scripts/bench_preflight.sh; then
        echo "REFUSE — the box failed its preflight; no paired table from this state" >&2
        exit 1
    fi
    # the PASSED posture is the publication posture — provenance.txt must
    # carry the real PROVENANCE line, never the no-preflight default (the
    # first publication-grade run rendered its summary with the ⛔ marker
    # beside a PASSED preflight — the marker and the evidence disagreed)
    line=$(sh scripts/bench_preflight.sh 2>/dev/null | grep '^PROVENANCE:' | head -1)
    PROVENANCE="PROVENANCE (paired_suite_ab): ${line#PROVENANCE: }"
else
    echo "⚠ PAIRED_AB_NO_PREFLIGHT=1 — the run's numbers are NOT FOR PUBLICATION"
fi
printf '%s\n' "$PROVENANCE" > "$OUT_DIR/provenance.txt"
sh scripts/bench_preflight.sh > "$OUT_DIR/preflight.txt" 2>&1 || true

# ---- build -----------------------------------------------------------------
# The metal posture is the published m3 lane. A missing harness is a loud
# build, never a silent wrong-binary run.
FEATURES="laya-riir-metal"
if [ "$(uname)" != "Darwin" ]; then
    FEATURES="laya-riir"
fi
if [ ! -x "target/release/harness" ] || [ "${PAIRED_AB_REBUILD:-0}" = "1" ]; then
    echo "building harness --features $FEATURES …"
    cargo build --release --features "$FEATURES" --bin harness
fi
HARNESS="target/release/harness"

# ---- the alternation --------------------------------------------------------
echo "paired per-suite A/B → $OUT_DIR"
echo "suites: $SUITES"
i=0
for suite in $(echo "$SUITES" | tr ',' ' '); do
    mkdir -p "$OUT_DIR/$suite/both" "$OUT_DIR/$suite/py" "$OUT_DIR/$suite/rust"
    if [ $((i % 2)) -eq 0 ]; then
        echo "[$suite] even → one run, rust laya first then py"
        "$HARNESS" --suites "$suite" --laya-python --out "$OUT_DIR/$suite/both" \
            > "$OUT_DIR/$suite/both.log" 2>&1 || {
            echo "REFUSE — suite $suite (both-lane run) failed; see $OUT_DIR/$suite/both.log" >&2
            exit 1
        }
    else
        echo "[$suite] odd → order reversed: py-only, then rust-only"
        "$HARNESS" --suites "$suite" --laya-python --skip-laya --out "$OUT_DIR/$suite/py" \
            > "$OUT_DIR/$suite/py.log" 2>&1 || {
            echo "REFUSE — suite $suite (py-only run) failed; see $OUT_DIR/$suite/py.log" >&2
            exit 1
        }
        "$HARNESS" --suites "$suite" --out "$OUT_DIR/$suite/rust" \
            > "$OUT_DIR/$suite/rust.log" 2>&1 || {
            echo "REFUSE — suite $suite (rust-only run) failed; see $OUT_DIR/$suite/rust.log" >&2
            exit 1
        }
    fi
    i=$((i + 1))
done

echo "all suites done — render the verdict:"
echo "  python3 scripts/paired_suite_ab_summary.py $OUT_DIR"
