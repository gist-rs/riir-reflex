#!/bin/bash
# Issue 020 T5 position-balanced A/B: the packed multi-question forward
# (default) vs the per-question loop (RIIR_LAYA_NO_BATCH=1).
# A = loop (incumbent), B = packed (challenger); order alternates per round.
# Adapted from harness_ab.sh (the xwide pattern). Record: Bench 006 Addendum 7.
#
# Usage: bash harness_ab_t5.sh <harness-binary> <repo-cwd> <logs-dir> [rounds]
# The harness MUST be prebuilt from the tree under test (release +
# laya-riir-metal). Runs `scripts/bench_preflight.sh` from the repo cwd
# first and REFUSES when it refuses — Addendum 7's confounded first run is
# the lesson: a paired A/B without a per-run load trace (written to
# <logs>/harness_ab.idx) would have published the wrong direction.
set -eu
H="${1:?harness binary}"
W="${2:?repo cwd (datasets resolve relative to it)}"
LOGS="${3:?logs dir}"
ROUNDS="${4:-4}"
mkdir -p "$LOGS"
cd "$W"
bash scripts/bench_preflight.sh || { echo "REFUSED: preflight failed — not an A/B box"; exit 1; }
: > "$LOGS/harness_ab.idx"
r=1
while [ "$r" -le "$ROUNDS" ]; do
  if [ $((r % 2)) -eq 1 ]; then order="A B"; else order="B A"; fi
  for arm in $order; do
    load_start=$(sysctl -n vm.loadavg | awk '{print $2}')
    if [ "$arm" = A ]; then RIIR_LAYA_NO_BATCH=1; else RIIR_LAYA_NO_BATCH=; fi
    export RIIR_LAYA_NO_BATCH
    LAYA_DEVICE=metal LAYA_WEIGHTS_DIR="${LAYA_WEIGHTS_DIR:-$HOME/.cache/riir-reflex/laya}" \
      "$H" --suites typed_decisions,code_fixtures,massive_intent_en --laya-max-questions 600 \
      --out "$LOGS/h_${r}_${arm}" > "$LOGS/h_${r}_${arm}.log" 2>&1
    echo "round $r arm $arm rc $? load_start $load_start load_end $(sysctl -n vm.loadavg | awk '{print $2}')" >> "$LOGS/harness_ab.idx"
  done
  r=$((r + 1))
done
echo DONE >> "$LOGS/harness_ab.idx"
cat "$LOGS/harness_ab.idx"
