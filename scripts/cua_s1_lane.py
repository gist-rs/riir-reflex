#!/usr/bin/env python3
"""The cua-s1-forms CoreML comparison lane — THEIR stack serves, our Rust measures.

Measurement-only (`.issues/035`; the `.issues/029` gliner lane and
`laya_python_lane.py` precedents). The owner's "no Python anywhere" directive
governs the shipped binary, not the bench reference: this script is spawned
ONLY by `examples/cua_s1_forms_arena.rs`, never by the product.

The model: FluidInference/cua-s1-forms-coreml (MIT, FP16 CoreML conversion of
Cua's cua-ai/cua-s1-forms — a 706,048-param byte-level option scorer). The
ENCODING LAW is theirs verbatim: this script imports the `preprocessing.py`
shipped in the model directory (UTF-8 bytes truncated to 224/96, byte + 1,
zero pad, 2-32 options, overflow REJECTED) — never a re-implementation.

Line protocol (the laya-python shape — one JSON object per line):

  handshake (stdout, once):  {"ready": true, "model": ..., "package": ...,
                              "compute_units": ..., "coremltools": ...,
                              "numpy": ..., "load_ms": ...}
  request   (stdin):         {"context": "<str>", "options": ["<str>", ...]}
  response  (stdout):        {"p": [...], "pick": i, "predict_us": n}
                             or {"error": "<message>"}

"p" is THEIR package's `probabilities` output restricted to the live options,
taken VERBATIM (their softmax — the house sigmoid law binds OUR engines, not
their published head). Their own conversion report shows strict numerical
parity FAILS on the full split (11 rows > 0.005 abs error), so a consumer
publishes argmax accuracy and latency only — never calibration.
"predict_us" times the synchronous `model.predict` call alone (encoding
excluded — their card's timer scope); the Rust side additionally times the
full subprocess round-trip (the laya-python measurement law).

Usage: cua_s1_lane.py <model_dir>
Env:   CUA_S1_PACKAGE        (default cua_s1_forms_fp16_options32.mlpackage —
                              their shipped default variant)
       CUA_S1_COMPUTE_UNITS  (default CPU_AND_NE — their card's ANE posture;
                              ALL / CPU_ONLY / CPU_AND_GPU accepted)
Venv:  uv venv --python 3.12 .raw/cua-s1-forms/venv &&
       VIRTUAL_ENV=.raw/cua-s1-forms/venv uv pip install coremltools==9.0 numpy==1.26.4
"""

import json
import os
import sys
import time


def emit(obj) -> None:
    sys.stdout.write(json.dumps(obj, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: cua_s1_lane.py <model_dir>", file=sys.stderr)
        return 2
    model_dir = os.path.abspath(sys.argv[1])
    package = os.environ.get("CUA_S1_PACKAGE", "cua_s1_forms_fp16_options32.mlpackage")
    units_name = os.environ.get("CUA_S1_COMPUTE_UNITS", "CPU_AND_NE")

    # THEIR encoding law — imported from the model repo, never copied.
    sys.path.insert(0, model_dir)
    import coremltools as ct
    import numpy as np
    from preprocessing import InputLimits, prepare_inputs

    units = getattr(ct.ComputeUnit, units_name, None)
    if units is None:
        print(f"unknown CUA_S1_COMPUTE_UNITS={units_name}", file=sys.stderr)
        return 2
    t0 = time.perf_counter()
    model = ct.models.MLModel(os.path.join(model_dir, package), compute_units=units)
    load_ms = (time.perf_counter() - t0) * 1000.0
    limits = InputLimits()

    emit(
        {
            "ready": True,
            "model": "FluidInference/cua-s1-forms-coreml",
            "package": package,
            "compute_units": units_name,
            "coremltools": ct.__version__,
            "numpy": np.__version__,
            "load_ms": round(load_ms, 1),
        }
    )

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
            options = req["options"]
            inputs = prepare_inputs(req["context"], options, limits)
            t = time.perf_counter()
            out = model.predict(inputs)
            predict_us = (time.perf_counter() - t) * 1e6
            probs = out["probabilities"][0, : len(options)]
            emit(
                {
                    "p": [float(x) for x in probs],
                    "pick": int(np.argmax(probs)),
                    "predict_us": round(predict_us, 1),
                }
            )
        except Exception as exc:  # the Rust side counts + reports, never guesses
            emit({"error": f"{type(exc).__name__}: {exc}"})
    return 0


if __name__ == "__main__":
    sys.exit(main())
