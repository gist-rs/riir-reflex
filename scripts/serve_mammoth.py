#!/usr/bin/env python
"""The mammoth-1 reference server (rethink Issue 032 T10.2's LOCAL-posture
lane; riir-train Issue 640's self-hosted capture serve) — the mechanical
twin of `scripts/elephant_server.py`, tracked so the rig survives a box
loss (the amo-only `.raw/mammoth-local/serve_mammoth.py` died with its
host; this file is the durable home).

Serves Base Labs' `baselabs/mammoth-1` (27B, JDI board #1 public_skill
75.34 @ the 2026-10-29 snapshot, MIT) through THEIR OWN runtime — OneJev
`qev` @81ce62f + the checkpoint repo's `serve.patch` (the model is trained
+ served in this shape: `modeling_overrides` block-bidirectional full
attention + causal linear attention, slot_lm readout, head_dim 256,
n_slots 256) — behind a stdlib HTTP listener on loopback.

LICENSE (rethink 032 T10 / riir-train 640 T0, verified 2026-10-24 and
re-read at the 2026-11-01 local-lane re-open): weights MIT (HF tag; no
LICENSE file ships — tag+front-matter assertion), base Qwen3.5 Apache-2.0,
runtime OneJev Apache-2.0. Capture-for-distill + comparison both clear.

Wire — the pplx/d3/mammoth family's SystemOne dialect, and the ONE PATH
the reflex mammoth lane ever posts (`src/lanes/mammoth.rs::post` is
hard-wired to `/v1/systemone` on every posture, hosted or local):

  GET  /health    -> {"ok": true, "model": ..., "dtype": ..., "device": ...,
                      "attention": ..., "readout": "slot_lm", "fork_mode": ...,
                      "license": "MIT ..."}
  POST /v1/systemone
       <- {"model": str (tolerated, echoed — ignored: the served checkpoint
           is the one this process loaded), "state": any,
           "questions": {qid: {"type": "choice"|"noul"|"score", ...}}}
       -> {"model": ..., "answers": {qid: {type, choice|noul|score,
           probabilities, ...}}, "usage": {...}, "latency_ms": ...}

(`/decide` is aliased to the same handler for the elephant-server's probe
shape, so the family's smoke checks work against either rig.)

Env pins (deliberate name: `MAMMOTH_WEIGHTS`, never `MAMMOTH_MODEL` — that
name is the HARNESS lane's catalog-id env and would collide):

  MAMMOTH_WEIGHTS   default ../riir-train/.raw/mammoth-1 (a LOCAL dir —
                    the pinned-sha snapshot_download; the engine refuses
                    non-local loads without network)
  MAMMOTH_DEVICE    default mps (the M5/M3 posture; cuda:0 on the 4090)
  MAMMOTH_DTYPE     default bfloat16 (the qev engine default)
  MAMMOTH_FORK_MODE default sequential (MEASURED 2026-10-25, amo M5 Ultra:
                    the engine's auto-selected `batched` fork runs the
                    qwen3_5 GDN reference fallback pathologically slow;
                    sequential carried the whole amo panel at p50
                    1.9 s/case on the 27B — the family's consistent
                    posture; treat fork mode as part of the box-state
                    disclosure)
  MAMMOTH_BIND      default 127.0.0.1
  MAMMOTH_PORT      default 8793 (the lane's SERVE_URL default — the rig
                    must answer where the lane already looks)

ATTENTION BOOTCHECK (the 635 −26 pts class): the checkpoint is a
block-bidirectional model; a CAUSAL serve reads as the wrong model and
loses ~26 pts silently. The loader logs the engine's own `attention`
field LOUDLY at boot — expect `block_bidirectional` (the overrides key)
or the engine's noncausal spelling; a `causal` value means the serve.patch
did not apply. This script does not hard-refuse on the string (qev's
exact vocabulary is engine-version-dependent) — it names the value so the
operator's bootcheck cannot miss it.

Run with the python that has `qev` importable (OneJev -e + serve.patch —
riir-train 640 T2(a) records the venv; the 2026-11-01 bmo rebuild recipe
lives in rethink Issue 032's local-lane block):
  MAMMOTH_PYTHON=../riir-train/.raw/mammoth-venv/bin/python
  $MAMMOTH_PYTHON scripts/serve_mammoth.py
"""

from __future__ import annotations

import gc
import json
import os
import sys
import time
import traceback
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

# A non-UTF-8 console (cp874) would kill this script on its first
# non-ASCII glyph; degrade the glyph, never the verdict (katgpt-rs
# Issue 804). A captured stream without reconfigure is left alone.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

PORT = int(os.environ.get("MAMMOTH_PORT", "8793"))
BIND = os.environ.get("MAMMOTH_BIND", "127.0.0.1")
MODEL_PATH = os.environ.get(
    "MAMMOTH_WEIGHTS", "../riir-train/.raw/mammoth-1"
)
DEVICE = os.environ.get("MAMMOTH_DEVICE", "mps")
DTYPE = os.environ.get("MAMMOTH_DTYPE", "bfloat16")
FORK_MODE = os.environ.get("MAMMOTH_FORK_MODE", "sequential")

LICENSE_NOTE = (
    "MIT (baselabs/mammoth-1; base Qwen3.5 Apache-2.0; runtime OneJev "
    "Apache-2.0) — T0-verified 2026-10-24 per riir-train 640; no LICENSE "
    "file ships (tag+front-matter assertion, re-read at adoption)"
)

ENGINE = None
INFO = {}


def load_model():
    global ENGINE, INFO
    from qev.engine import DecisionEngine

    print(
        f"[mammoth-server] loading {MODEL_PATH} device={DEVICE} dtype={DTYPE}",
        flush=True,
    )
    t0 = time.perf_counter()
    ENGINE = DecisionEngine(
        MODEL_PATH,
        device=DEVICE,
        dtype=DTYPE,
        fork_mode=FORK_MODE,
    )
    info = ENGINE.info
    INFO = {
        "model_type": info.model_type,
        "dtype": info.dtype,
        "device": str(info.device),
        "fork_mode": info.fork_mode,
        "attention": info.extra.get("attention"),
        "readout": info.extra.get("readout"),
        "calibration": info.extra.get("calibration"),
    }
    attention = INFO.get("attention")
    print(
        f"[mammoth-server] ATTENTION BOOTCHECK: attention={attention!r} "
        "readout={readout!r} — expect a block-bidirectional/noncausal "
        "value; a causal value means serve.patch did NOT apply (the "
        "635 −26 pts wrong-model class).".format(readout=INFO.get("readout")),
        flush=True,
    )
    print(
        f"[mammoth-server] up in {time.perf_counter() - t0:.1f}s: "
        f"{json.dumps(INFO, sort_keys=True)} fork_mode={FORK_MODE} "
        f"license={LICENSE_NOTE}",
        flush=True,
    )


def handle_decide(body):
    from qev.schema import SystemOneRequest

    state = body.get("state")
    questions = body.get("questions")
    if not isinstance(questions, dict) or not questions:
        raise ValueError("questions must be a non-empty object keyed by qid")
    # The "model" field the pplx-family body carries is tolerated (qev's
    # request model is lenient) and IGNORED — the served checkpoint is the
    # one this process loaded, never a per-request pick.
    req = SystemOneRequest(state=state, questions=questions)
    t0 = time.perf_counter()
    resp, _meta = ENGINE.decide(req)
    latency_ms = (time.perf_counter() - t0) * 1000.0
    answers = {}
    for qid, a in resp.answers.items():
        answers[qid] = a.model_dump()
    return {
        "model": f"mammoth-1@{os.path.basename(os.path.normpath(MODEL_PATH))}",
        "answers": answers,
        "usage": resp.usage.model_dump(),
        "latency_ms": latency_ms,
    }


def mps_cache_hygiene():
    """MPS CACHE HYGIENE (the mammoth launcher's measured fix, 2026-10-24):
    the qev engine's empty_cache guard is CUDA-only, and forked branches
    deep-copy the prefix cache — on MPS the cached allocator never returns
    memory and RSS ballooned 44 → 70 GB with 23.7 GB of swap inside one
    600-case suite. gc + torch.mps.empty_cache() after EVERY decide.
    Answers are byte-identical with and without the hygiene (the mammoth
    determinism probes predate it); only the allocator's footprint moves.
    """
    gc.collect()
    try:
        import torch

        if torch.backends.mps.is_available():
            torch.mps.empty_cache()
    except ImportError:  # non-torch posture (e.g. a CPU stub build)
        pass


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):  # quiet: ASCII one-liners only
        sys.stderr.write("[mammoth-server] %s\n" % (fmt % args))

    def _send(self, code, payload):
        raw = json.dumps(payload, ensure_ascii=True).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_GET(self):
        if self.path != "/health":
            self._send(404, {"error": f"no route {self.path}"})
            return
        self._send(
            200,
            {
                "ok": True,
                "model": "baselabs/mammoth-1",
                **INFO,
                "license": LICENSE_NOTE,
            },
        )

    def do_POST(self):
        # The lane's one path is /v1/systemone (hosted and local alike);
        # /decide is the elephant-server's probe shape, aliased here.
        if self.path not in ("/v1/systemone", "/decide"):
            self._send(404, {"error": f"no route {self.path}"})
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            body = json.loads(self.rfile.read(length).decode("utf-8"))
            payload = handle_decide(body)
            # After handle_decide returns — its locals (the pydantic resp
            # tree) are gone, so the gc pass can actually reclaim the
            # transient activation tensors before the next request lands.
            mps_cache_hygiene()
            self._send(200, payload)
        except ValueError as exc:
            self._send(400, {"error": str(exc)})
        except Exception:  # noqa: BLE001 - one loud 500 with the traceback tail
            tail = traceback.format_exc().strip().splitlines()[-1]
            self._send(500, {"error": tail})


def main():
    load_model()
    server = ThreadingHTTPServer((BIND, PORT), Handler)
    print(f"[mammoth-server] listening on http://{BIND}:{PORT}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
