#!/usr/bin/env python
"""The Jeva reference server (reflex Issue 090 B1, feeding riir-train 632 T2).

Serves `LIQIIIII/Jeva-4B` (arXiv:2610.18773 "Jeva: System One Models from
First Principles", Li & Wang, NUS; code Apache-2.0 at LiQiiiii/Jeva) behind
a stdlib HTTP listener on loopback — the d1 lane's reference-server shape:
their class, our listener, zero framework deps.

LICENSE POSTURE (riir-train Issue 632 T0, verified 2026-10-15): the HF
weights are `gated: "auto"` (accept-conditions auto-approval) with an
UNDECLARED license (no license field/tag/file on the repo or card).
Undeclared is neither open nor restricted-declared — CAPTURE-FOR-
MEASUREMENT is allowed (the card's own research intent); ADOPTION of the
weights into any product is FAIL-CLOSED until the author declares. The
/health endpoint and the RIDT header both carry this posture.

Wire (the harness `--jeva` lane, `src/lanes/jeva.rs`):

  GET  /health  -> {"ok": true, "model": ..., "dtype": ..., "device": ...,
                    "cache": ..., "allow_null": bool, "lora_merged": bool,
                    "license": "UNDECLARED ..."}
  POST /decide  <- {"state": str|any, "questions": {qid: {
                      "type": "choice"|"noul"|"score",
                      "instructions": str,
                      "criteria": {...choice} | [...score] | omitted(noul),
                      "values": [float] (optional, score)}}}
                 -> {"model": ..., "answers": {qid: {kind, probabilities,
                      conditional_probabilities, null_probability, choice,
                      noul, score, confidence, candidate_probabilities,
                      usage, latency_ms}}, "usage": {...}, "latency_ms": ...}

The answer fields are their `jeva.types.Answer` verbatim (asdict):
`probabilities` are JOINT ((1-rho)*q), `conditional_probabilities` are the
conditional q (the paper's A3 readout), `null_probability` is the coverage
gate rho, `confidence` = max(rho, max(joint)).

Env pins:
  JEVA_MODEL      default LIQIIIII/Jeva-4B
  JEVA_DEVICE     default cuda
  JEVA_DTYPE      default "" = the checkpoint config's dtype (never assumed;
                  /health reports the RESOLVED served dtype — the d1 law)
  JEVA_CACHE      default hybrid (none|set|hybrid)
  JEVA_ALLOW_NULL default "0" — the forced-pick comparison posture all
                  reflex lanes speak; "1" arms their coverage gate (the A1
                  probe posture; an abstain reads choice=null on the wire)
  JEVA_BIND       default 127.0.0.1
  JEVA_PORT       default 8095 (fresh in the lane family: 8000 drex/openthai,
                  8078 d1, 8149 agentjev, 8793 pplx, 8794 d3)

Run with the python that has `jeva` importable (their repo installed):
  JEVA_PYTHON=.raw/jeva-env/Scripts/python.exe
  %JEVA_PYTHON% scripts/jeva_server.py
"""

from __future__ import annotations

import json
import os
import sys
import traceback
from dataclasses import asdict
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = int(os.environ.get("JEVA_PORT", "8095"))
BIND = os.environ.get("JEVA_BIND", "127.0.0.1")
MODEL_ID = os.environ.get("JEVA_MODEL", "LIQIIIII/Jeva-4B")
DEVICE = os.environ.get("JEVA_DEVICE", "cuda")
DTYPE_ARG = os.environ.get("JEVA_DTYPE", "").strip()
CACHE = os.environ.get("JEVA_CACHE", "hybrid")
ALLOW_NULL = os.environ.get("JEVA_ALLOW_NULL", "0").strip() not in ("", "0", "false", "False")

LICENSE_NOTE = (
    "UNDECLARED (LIQIIIII/Jeva-4B weights; code Apache-2.0 LiQiiiii/Jeva) — "
    "capture-for-measurement posture per riir-train 632 T0; adoption fail-closed"
)

MODEL = None
SERVED_DTYPE = "unknown"
LORA_MERGED = False


def load_model():
    global MODEL, SERVED_DTYPE, LORA_MERGED
    from jeva import Jeva

    dtype = None
    if DTYPE_ARG:
        dtype = DTYPE_ARG
    print(f"[jeva-server] loading {MODEL_ID} device={DEVICE} dtype={dtype or 'config'} cache={CACHE}", flush=True)
    MODEL = Jeva.from_pretrained(
        MODEL_ID,
        device=DEVICE,
        merge_lora=True,
        cache=CACHE,
        dtype=dtype,
    )
    LORA_MERGED = bool(getattr(MODEL, "lora_merged", False))
    try:
        SERVED_DTYPE = str(next(MODEL.model.parameters()).dtype)
    except StopIteration:  # pragma: no cover - defensive
        SERVED_DTYPE = "unknown"
    print(
        f"[jeva-server] up: {MODEL_ID} dtype={SERVED_DTYPE} lora_merged={LORA_MERGED} "
        f"allow_null={ALLOW_NULL} license={LICENSE_NOTE}",
        flush=True,
    )


def build_question(qid, spec):
    from jeva import Choice, Noul, Score

    qtype = spec.get("type")
    instructions = spec.get("instructions", "")
    if qtype == "noul":
        return Noul(instructions=instructions)
    criteria = spec.get("criteria")
    if qtype == "choice":
        # Their `_criteria` accepts BOTH shapes: a mapping (label ->
        # description, all nonempty) and a sequence (bare labels; texts =
        # the labels themselves). Mirror that law exactly — the bare-label
        # suites (emotion/banking77/semantic_defects) ride the sequence arm.
        is_map = isinstance(criteria, dict) and bool(criteria)
        is_seq = (
            isinstance(criteria, list)
            and len(criteria) >= 1
            and all(isinstance(k, str) and k.strip() for k in criteria)
        )
        if not is_map and not is_seq:
            raise ValueError(
                f"qid {qid}: choice criteria must be a non-empty object "
                "or a non-empty array of nonempty string labels"
            )
        return Choice(instructions=instructions, criteria=criteria, allow_null=ALLOW_NULL)
    if qtype == "score":
        if not isinstance(criteria, list) or len(criteria) < 2:
            raise ValueError(f"qid {qid}: score criteria must be an array of >=2 levels")
        values = spec.get("values")
        if values is not None:
            if not isinstance(values, list) or len(values) != len(criteria):
                raise ValueError(f"qid {qid}: score values must match criteria count")
            return Score(instructions=instructions, criteria=criteria, values=[float(v) for v in values])
        return Score(instructions=instructions, criteria=criteria)
    raise ValueError(f"qid {qid}: unknown question type {qtype!r}")


def handle_decide(body):
    state = body.get("state")
    questions = body.get("questions")
    if not isinstance(questions, dict) or not questions:
        raise ValueError("questions must be a non-empty object keyed by qid")
    built = {str(qid): build_question(qid, spec) for qid, spec in questions.items()}
    response = MODEL.system_one(state=state, questions=built)
    answers = {}
    for qid, answer in response.answers.items():
        d = asdict(answer)
        # candidate_probabilities renders as a list under asdict already
        answers[qid] = d
    return {
        "model": MODEL_ID,
        "answers": answers,
        "usage": asdict(response.usage),
        "latency_ms": response.latency_ms,
    }


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):  # quiet: ASCII one-liners only
        sys.stderr.write("[jeva-server] %s\n" % (fmt % args))

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
                "model": MODEL_ID,
                "dtype": SERVED_DTYPE,
                "device": DEVICE,
                "cache": CACHE,
                "allow_null": ALLOW_NULL,
                "lora_merged": LORA_MERGED,
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
    print(f"[jeva-server] listening on http://{BIND}:{PORT}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
