#!/bin/sh
# Plan 617 A6 — the bekko-system-one-v0-400M board seat (Bench 103's recorded follow-up).
# Two serial invocations mirroring the two published postures exactly:
#   run 1 = the canonical 7 suites at the canonical flags (Bench 103 repro block)
#   run 2 = typed_decisions + code_fixtures at the addendum flags (--oc-select; typed arms oc scale 4.0)
set -e
cd /Users/katopz/git/riir-reflex
export BEKKO_MODEL=hotchpotch/bekko-system-one-v0-400m
export BEKKO_REVISION=4aeb85b9d4042d75d8b8adf6ff7ba9e4629510ba
export BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python
mkdir -p .benchmarks/107_bekko_v0_400m_board
echo "=== RUN 1 start $(date -u +%FT%TZ) ==="
target/release/harness --bekko --skip-laya --nb-select --ridge-select \
  --suites emotion,ag_news,sst5,massive_intent_en,banking77,prompt_injections,xnli_en \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/107_bekko_v0_400m_board
echo "=== RUN 2 start $(date -u +%FT%TZ) ==="
target/release/harness --bekko --skip-laya --nb-select --oc-select --ridge-select \
  --suites typed_decisions,code_fixtures \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/107_bekko_v0_400m_board/addendum_suites
echo "=== ALL DONE $(date -u +%FT%TZ) ==="
