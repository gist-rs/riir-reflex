#!/usr/bin/env python
"""The elephant-1 reference server (riir-train Issue 640 T2(a)/T3 — the
SELF-HOSTED capture lane; rethink Issue 032 T10.2's local-posture row).

Serves Base Labs' `baselabs/elephant-1` (9B, Qwen3.5-9B full FT — JDI 0.4.1
board #4 public_skill 72.24) through THEIR OWN runtime — OneJev `qev`
@81ce62f + the repo's `serve.patch` (the checkpoint is trained + served in
this shape: `modeling_overrides` block-bidirectional full attention, slot_lm
readout) — behind a stdlib HTTP listener on loopback. The d1/jeva lane's
reference-server shape: their engine, our listener, zero framework deps.

LICENSE (riir-train 640 T0, verified 2026-10-24): weights MIT, base models
Apache-2.0 (Qwen3.5), runtime Apache-2.0 (OneJev) — PERMISSIVE, no NC
posture, no RESTRICTED lineage. The 640 T0 caveat: no LICENSE FILE ships in
either repo; the assertion is the HF tag + README front-matter (re-read at
any adoption act). Capture-for-distill + comparison both clear.

Wire (the pplx/d3/mammoth family's SystemOne dialect — the lane reuses the
pplx parser verbatim):

  GET  /health  -> {"ok": true, "model": ..., "dtype": ..., "device": ...,
                    "attention": "noncausal", "readout": "slot_lm",
                    "fork_mode": ..., "license": "MIT ..."}
  POST /decide  <- {"model": str (ignored, echoed), "state": any,
                    "questions": {qid: {"type": "choice"|"noul"|"score",
                    "instructions": str, "criteria": {...}|[...]}}}
                -> {"model": ..., "answers": {qid: {type, choice|noul|score,
                    probabilities, confidence}}, "usage": {...},
                    "latency_ms": ...}

`probabilities` are keyed by OPTION LABEL (choice) / level index string
(score) — qev/answers.py `make_choice`/`make_score`; rounded 6 decimals
(their `_round`). Choice questions over `MAX_SLOTS` (256) options are
chunked by their engine natively (the NONE_OPTION reweighting — banking77's
77 options fit).

Env pins:
  ELEPHANT_MODEL   default <repo>/elephant-1 (a LOCAL dir; the engine
                   refuses non-local loads without network — point it at
                   the pinned-sha download)
  ELEPHANT_DEVICE  default mps (the M3 probe posture; cuda:0 on the 4090)
  ELEPHANT_DTYPE   default bfloat16 (the qev engine default — OVERRIDES the
                   config's top-level float32; /health reports the RESOLVED
                   engine info dtype — the d1 law)
  ELEPHANT_BIND    default 127.0.0.1
  ELEPHANT_PORT    default 8096 (fresh in the lane family: 8000
                   drex/openthai, 8078 d1, 8090 startlux, 8095 jeva, 8793
                   pplx, 8794 d3, 8149 agentjev)

Run with the python that has `qev` importable (OneJev installed -e +
serve.patch applied — riir-train 640 T2(a) records the exact venv):
  ELEPHANT_PYTHON=../riir-train/.raw/probe-venv/bin/python
  $ELEPHANT_PYTHON scripts/elephant_server.py
"""

from __future__ import annotations

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

PORT = int(os.environ.get("ELEPHANT_PORT", "8096"))
BIND = os.environ.get("ELEPHANT_BIND", "127.0.0.1")
MODEL_PATH = os.environ.get(
    "ELEPHANT_MODEL", "../riir-train/.raw/elephant-1"
)
DEVICE = os.environ.get("ELEPHANT_DEVICE", "mps")
DTYPE = os.environ.get("ELEPHANT_DTYPE", "bfloat16")

LICENSE_NOTE = (
    "MIT (baselabs/elephant-1; base Qwen3.5 Apache-2.0; runtime OneJev "
    "Apache-2.0) — T0-verified 2026-10-24 per riir-train 640; no LICENSE "
    "file ships (tag+front-matter assertion, re-read at adoption)"
)

ENGINE = None
INFO = {}


def load_model():
    global ENGINE, INFO
    from qev.engine import DecisionEngine

    print(
        f"[elephant-server] loading {MODEL_PATH} device={DEVICE} dtype={DTYPE}",
        flush=True,
    )
    t0 = time.perf_counter()
    ENGINE = DecisionEngine(MODEL_PATH, device=DEVICE, dtype=DTYPE)
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
    print(
        f"[elephant-server] up in {time.perf_counter() - t0:.1f}s: "
        f"{json.dumps(INFO, sort_keys=True)} license={LICENSE_NOTE}",
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
        "model": f"elephant-1@{os.path.basename(os.path.normpath(MODEL_PATH))}",
        "answers": answers,
        "usage": resp.usage.model_dump(),
        "latency_ms": latency_ms,
    }


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):  # quiet: ASCII one-liners only
        sys.stderr.write("[elephant-server] %s\n" % (fmt % args))

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
                "model": "baselabs/elephant-1",
                **INFO,
                "license": LICENSE_NOTE,
            },
        )

    def do_POST(self):
        if self.path != "/decide":
            self._send(404, {"error": f"no route {self.path}"})
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            body = json.loads(self.rfile.read(length).decode("utf-8"))
            payload = handle_decide(body)
            self._send(200, payload)
        except ValueError as exc:
            self._send(400, {"error": str(exc)})
        except Exception:  # noqa: BLE001 - one loud 500 with the traceback tail
            tail = traceback.format_exc().strip().splitlines()[-1]
            self._send(500, {"error": tail})


def main():
    load_model()
    server = ThreadingHTTPServer((BIND, PORT), Handler)
    print(f"[elephant-server] listening on http://{BIND}:{PORT}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
