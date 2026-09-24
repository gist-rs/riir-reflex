#!/bin/bash
# Paired harness A/B: A = default pick, B = LAYA_XWIDE_N_MIN=1024. Order alternates per round.
set -u
H=/tmp/cf020b/target/release/harness
cd /tmp/cf020b/riir-reflex
for r in 1 2 3 4 5 6; do
  if [ $((r % 2)) -eq 1 ]; then order="A B"; else order="B A"; fi
  for arm in $order; do
    load=$(sysctl -n vm.loadavg | awk '{print $2}')
    if [ "$arm" = B ]; then export LAYA_XWIDE_N_MIN=1024; else unset LAYA_XWIDE_N_MIN; fi
    LAYA_DEVICE=metal LAYA_WEIGHTS_DIR=$HOME/.cache/riir-reflex/laya $H --suites massive_intent_en,banking77,code_fixtures --out /tmp/cf020b/h_${r}_${arm} > /tmp/cf020b/h_${r}_${arm}.log 2>&1
    echo "round $r arm $arm load_start $load load_end $(sysctl -n vm.loadavg | awk '{print $2}')" >> /tmp/cf020b/harness_ab.idx
  done
done
echo DONE >> /tmp/cf020b/harness_ab.idx
