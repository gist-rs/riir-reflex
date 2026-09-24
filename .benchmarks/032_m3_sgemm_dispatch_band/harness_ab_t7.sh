#!/bin/bash
# T7 dispatch-predicate A/B: OLD = worktree substrate @ 1e8c034 (m/n
# threshold pick), NEW = live substrate (single-wave pick). Same harness
# code both arms (reflex @ 3ac8a8a); order alternates per round.
set -u
OLD=/tmp/t7ab/release/harness
NEW=/tmp/t7ab-new/release/harness
W=$HOME/.cache/riir-reflex/laya
cd /Users/katopz/git/riir-reflex
IDX=/tmp/t7ab/harness_ab.idx
: > $IDX
for r in 1 2 3 4 5 6; do
  if [ $((r % 2)) -eq 1 ]; then order="OLD NEW"; else order="NEW OLD"; fi
  for arm in $order; do
    load=$(uptime | sed 's/.*averages*: //' | cut -d, -f1)
    if [ "$arm" = OLD ]; then H=$OLD; else H=$NEW; fi
    LAYA_DEVICE=metal LAYA_WEIGHTS_DIR=$W $H --suites massive_intent_en,banking77,code_fixtures --out /tmp/t7ab/h_out/h_${r}_${arm} > /tmp/t7ab/h_${r}_${arm}.log 2>&1
    echo "round $r arm $arm load_start $load load_end $(uptime | sed 's/.*averages*: //' | cut -d, -f1)" >> $IDX
  done
done
echo DONE >> $IDX
