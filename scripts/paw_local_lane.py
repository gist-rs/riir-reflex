#!/usr/bin/env python3
"""The PAW Posture B lane — their LOCAL runtime as a subprocess oracle.

Measurement-only (`.issues/033`; the gliner_lane.py precedent): THEIR
package serves, our Rust harness measures. The owner's "no Python anywhere"
directive governs the shipped binary, not the bench reference.

The harness (src/lanes/paw_local.rs, `PawLocalSession`) spawns ONE process
per suite and speaks this line protocol over stdin/stdout:

  handshake (one JSON line, after startup):  {"ready": true, "lane":
      "paw-local", "runtime": "<programasweights version>", "device":
      "<paw-reported device|?>"}
  request  (one JSON line):  {"program_id": "<id>", "input": "<text>"}
  response (one JSON line):  {"output": "<free text>"}
  a runtime failure answers  {"error": "<msg>"}  — the Rust side fails
  loud, never a guessed row.

The program ids come from the hosted lane's cache
(`.raw/paw/programs.json`, keyed (suite, compiler, BLAKE3(spec))) — the
Posture B lane NEVER compiles; it downloads the already-compiled public
bundle on first `paw.function(program_id)` and reuses the validated cache
after. Same compiled artifact as the hosted cells, so a local-vs-hosted
delta isolates the runtime posture.

Greedy by construction: `paw.function` exposes no sampling knobs and the
feasibility probe measured repeat-stable outputs (5/5 byte-identical) —
the determinism claim is re-verified per run by the harness's
observed-repeat check, never assumed.

Usage: paw_local_lane.py
Env:   PAW_GPU_LAYERS (their knob; unset = auto CUDA/CPU) ·
       PAW_LOCAL_N_CTX (default 2048)
"""

import json
import os
import sys


def main() -> int:
    # UTF-8 pipes before any import side effect prints a glyph (cp874 law).
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
        except (AttributeError, ValueError):
            pass

    import programasweights as paw

    n_ctx = int(os.environ.get("PAW_LOCAL_N_CTX", "2048"))
    fns: dict[str, object] = {}

    def get_fn(program_id: str):
        # Their loader caches validated bundles on disk; the dict keeps the
        # loaded interpreter warm across requests of the same program.
        if program_id not in fns:
            print(
                f"[paw-local] loading program {program_id} "
                "(first use downloads the bundle + base unless cached)",
                file=sys.stderr,
                flush=True,
            )
            fns[program_id] = paw.function(program_id, n_ctx=n_ctx)
        return fns[program_id]

    print(json.dumps({
        "ready": True,
        "lane": "paw-local",
        "runtime": f"programasweights {getattr(paw, '__version__', '?')}",
        "device": "llama.cpp auto (CUDA/Metal/CPU per their loader)",
    }), flush=True)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        req = json.loads(line)
        try:
            fn = get_fn(req["program_id"])
            out = fn(req["input"])
            if not isinstance(out, str):
                out = str(out)
            print(json.dumps({"output": out}), flush=True)
        except Exception as exc:  # noqa: BLE001 — the protocol's error channel
            print(json.dumps({"error": f"{type(exc).__name__}: {exc}"}), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
