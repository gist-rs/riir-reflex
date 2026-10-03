#!/bin/sh
# paw hub health probe (bench 117) — a REAL infer on a REAL cached program.
# The 2026-10-04 lesson: a 422 on an invalid body only proves the validation
# layer answers; the inference backend can still be 502-down behind it.
# Probes exactly what the harness sends at warmup (paw.rs run_suite):
#   POST /api/v1/infer {program_id, input, temperature 0.0, max_tokens 48}
# with the harness's own warmup input string.
# Exit 0 = healthy (HTTP 200 with an output field); exit 1 = still down.
set -u
CACHE="${PAW_CACHE:-.raw/paw/programs.json}"
KEY="massive_intent_en|intent|paw-ft-bs48-20260530|10b2865efdaaeff8129e625e18a36b141e07fd7f5fea6407c759487c31096f17"
PID=$(python3 -c "
import json,sys
d=json.load(open('$CACHE'))['entries']
v=d.get('$KEY')
print(v['program_id'] if isinstance(v,dict) and 'program_id' in v else v if isinstance(v,str) else '')
")
if [ -z "$PID" ]; then
  echo "probe: no cached program for $KEY — cache shape changed?"; exit 2
fi
RESP=$(curl -s --max-time 60 -X POST https://programasweights.com/api/v1/infer \
  -H "Content-Type: application/json" \
  -d "{\"program_id\":\"$PID\",\"input\":\"warmup: discarded, not a measured case\",\"temperature\":0.0,\"max_tokens\":48}")
echo "$RESP" | head -c 300; echo
echo "$RESP" | python3 -c "
import json,sys
try:
    v=json.load(sys.stdin)
except Exception:
    print('probe: UNPARSEABLE'); sys.exit(1)
if 'output' in v and isinstance(v.get('output'),str):
    print('probe: HEALTHY'); sys.exit(0)
print('probe: DOWN —', json.dumps(v)[:200]); sys.exit(1)
"
