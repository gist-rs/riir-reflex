#!/usr/bin/env python3
"""Preload the PAW local-runtime assets for the Posture B lane (Issue 033).

Downloads (keyless, public programs) the .paw bundles for the cached
hosted-compiled program ids + the shared base model, then runs ONE throwaway
inference per program so the first lane request is cache-warm. Run detached:

    nohup .raw/paw-env/Scripts/python.exe scripts/paw_preload.py \
        > .raw/paw_preload.log 2>&1 &

UTF-8 pipes everywhere (the cp874 console law) — every print names its suite.
"""

import json
import os
import sys
from pathlib import Path

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

REPO = Path(__file__).resolve().parent.parent
CACHE = REPO / ".raw" / "paw" / "programs.json"


def main() -> int:
    import programasweights as paw

    entries = json.loads(CACHE.read_text(encoding="utf-8"))["entries"]
    # key = "suite|compiler|specblake3" — keep one program per suite (the ft
    # tier is the one this box's hosted cells used).
    per_suite: dict[str, str] = {}
    for key, row in entries.items():
        suite = key.split("|", 1)[0]
        per_suite[suite] = row["program_id"]

    print(f"preloading {len(per_suite)} program(s): {per_suite}", flush=True)
    for suite, pid in sorted(per_suite.items()):
        print(f"[{suite}] loading {pid} …", flush=True)
        fn = paw.function(pid)
        out = fn("warmup: discarded, not a measured case")
        print(f"[{suite}] warm output: {out!r}", flush=True)
    print("PRELOAD DONE", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
