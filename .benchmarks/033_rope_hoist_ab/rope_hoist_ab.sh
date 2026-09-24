#!/bin/bash
# T10 rung 2 (rope hoist) promotion A/B — ONE binary, TWO env arms: the
# flag IS the switch, no rebuild between arms.
#   OFF = LAYA_METAL_ROPE_HOIST=0  (in-kernel rope arm — the shipped default)
#   ON  = LAYA_METAL_ROPE_HOIST=1  (attn_rope pre-pass + flash staging copy)
# Order alternates per round (position-balanced — the cold-GPU/first-arm
# lesson, reflex README). Run ONLY on a box `scripts/bench_preflight.sh`
# passes; quote its PROVENANCE line beside the numbers.
#
# Binary rebuild recipe (substrate riir-infer 5ef7442+ contains the flag):
#   cd /Users/katopz/git/riir-reflex
#   CARGO_TARGET_DIR=/tmp/rhoistab cargo build --release \
#       --features laya-riir-metal --bin harness
set -u
H=/tmp/rhoistab/release/harness
W=$HOME/.cache/riir-reflex/laya
cd /Users/katopz/git/riir-reflex
OUT=/tmp/rhoistab
IDX=$OUT/rope_ab.idx
: > $IDX
for r in 1 2 3 4 5 6; do
  if [ $((r % 2)) -eq 1 ]; then order="OFF ON"; else order="ON OFF"; fi
  for arm in $order; do
    if [ "$arm" = ON ]; then FLAG=1; else FLAG=0; fi
    load=$(uptime | sed 's/.*averages*: //' | cut -d, -f1)
    LAYA_DEVICE=metal LAYA_WEIGHTS_DIR=$W LAYA_METAL_ROPE_HOIST=$FLAG \
      $H --suites massive_intent_en,banking77,code_fixtures \
      --out $OUT/h_out/h_${r}_${arm} > $OUT/h_${r}_${arm}.log 2>&1
    echo "round $r arm $arm load_start $load load_end $(uptime | sed 's/.*averages*: //' | cut -d, -f1)" >> $IDX
  done
done
echo DONE >> $IDX
