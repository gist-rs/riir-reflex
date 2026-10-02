#!/usr/bin/env python3
"""The Bekko comparison lane — hotchpotch/bekko-system-one-v0 as an external oracle.

Measurement-only (the gliner_lane.py / laya_python_lane.py pattern; wired as the harness --bekko lane, Bench 103): THEIR
model serves, our Rust harness measures. Speaks the SAME line protocol the
harness spawns via GLINER_LANE_SCRIPT:

  request  (one JSON line):  {"state": <value>, "questions": [{"qid", "def"}]}
  response (one JSON line):  {"answers": {"<qid>": {"p": [...], "conf": c,
                              "choice": <key>|null}}}

"p" is the probability vector in the question's OWN option order (criteria
key order for choice, level order for score, [p_no, p_yes] for noul).
Bekko's softmax over candidates is taken as-is; "conf" is the top
probability; "choice" is the argmax key for choice questions.

Mapping (each lane applies its OWN rendering law — the CLM parity-note law):
  - state: the wire state is passed AS JSON (`json.dumps`) — bekko's
    reference law renders JSON states itself (the model card's quickstart
    shape); no prose re-rendering is invented.
  - choice: criteria {key: desc|null} → ONE choice decision, criteria in
    insertion order, description_json = the description when a non-empty
    string, else None (the label key alone reaches the model — the same
    information gliner's LabelSpec(key, None) gets).
  - score: criteria [level, ...] → native score decision when EVERY level
    parses as a number (value = the number, id = the level string); else a
    choice decision over the level strings (DISCLOSED divergence: bekko
    picks the level label without rubric semantics — its score type
    requires >= 2 distinct numeric values).
  - noul: criteria null → native noul, ids false/true with generic
    meanings ("The statement is false." / "The statement is true.") — the
    instruction carries the question, the same default-phrases posture as
    the seat's [false, true] rendering. p = [p_false, p_true].
  - ALL of a case's questions become decisions of ONE input object (they
    share the state — bekko's shared-prefix path encodes the prefix once).

Env: BEKKO_MODEL (hotchpotch/bekko-system-one-v0-17m) ·
     BEKKO_REVISION (2147c3d9d00559bf972616c2589eee967e266f3b — the card's
     pinned 17M release commit) · device = argv[1] (default cpu; the
     author's reference posture is CPU FP32 — CUDA uses BF16 autocast).

The venv needs torch==2.10 transformers==5.17 sentence-transformers==6.1
(the card's runtime pins). License: MIT — verified 2026-10-02
(https://raw.githubusercontent.com/hotchpotch/bekko-system-one/refs/heads/main/LICENSE,
Copyright (c) 2026 Yuichi Tatsumi) — measurement AND distill/teacher use
are license-clear with attribution (riir-train Issue 608).
"""

import json
import os
import sys

DEFAULT_MODEL = "hotchpotch/bekko-system-one-v0-17m"
DEFAULT_REVISION = "2147c3d9d00559bf972616c2589eee967e266f3b"


def resolve_device(arg):
    if arg:
        return arg
    d = os.environ.get("BEKKO_DEVICE")
    if d:
        return d
    # CPU is the default EVERYWHERE (the author's reference posture, FP32);
    # a GPU posture is an explicit opt-in (BEKKO_DEVICE=cuda|mps) because it
    # changes numerics (CUDA = BF16 autocast) — never a silent default.
    return "cpu"


def build_input(state, questions):
    """Wire request -> ONE bekko input object (all decisions share the state)."""
    decisions = []
    for q in questions:
        d = q["def"]
        kind = d["type"]
        qid = q["qid"]
        instructions = d.get("instructions") or ""
        criteria = d.get("criteria")
        if kind == "choice":
            if not isinstance(criteria, dict) or not criteria:
                raise ValueError(f"choice question {qid}: criteria must be a non-empty object")
            # description_json MUST be a JSON string (the renderer calls
            # json.loads unconditionally) — a null description carries the
            # key itself, the same information the seat/gliner lanes get.
            crit = [
                {
                    "id": key,
                    "description_json": json.dumps(desc) if isinstance(desc, str) and desc.strip() else json.dumps(key),
                    "value": None,
                }
                for key, desc in criteria.items()
            ]
            btype = "choice"
        elif kind == "score":
            if not isinstance(criteria, list) or not criteria:
                raise ValueError(f"score question {qid}: criteria must be a non-empty array")
            values = []
            for entry in criteria:
                try:
                    values.append(float(entry))
                except (TypeError, ValueError):
                    values.append(None)
            if any(v is None for v in values) or max(values) <= min(values):
                # Not a numeric rubric -> pick the level label (disclosed).
                crit = [
                    {"id": str(entry), "description_json": json.dumps(str(entry)), "value": None}
                    for entry in criteria
                ]
                btype = "choice"
            else:
                crit = [
                    {"id": str(entry), "description_json": json.dumps(str(entry)), "value": value}
                    for entry, value in zip(criteria, values)
                ]
                btype = "score"
        elif kind == "noul":
            crit = [
                {"id": "false", "description_json": json.dumps("The statement is false."), "value": None},
                {"id": "true", "description_json": json.dumps("The statement is true."), "value": None},
            ]
            btype = "noul"
        else:
            raise ValueError(f"question {qid}: unknown kind {kind!r}")
        decisions.append({
            "id": qid,
            "kind": "judgment",
            "type": btype,
            "instructions_json": json.dumps(instructions),
            "system_prompt": "",
            "criteria": crit,
            "documents": [],
            "scoring": None,
        })
    return {"state_json": json.dumps(state), "decisions": decisions}


def answer_for(q, result):
    qid = q["qid"]
    d = q["def"]
    kind = d["type"]
    criteria = d.get("criteria")
    probs = result[qid]["probabilities"]
    if kind == "choice":
        keys = list(criteria.keys())
        p = [float(probs[k]) for k in keys]
        top = max(range(len(keys)), key=lambda i: p[i])
        return {"p": p, "conf": p[top], "choice": keys[top]}
    if kind == "score":
        levels = [str(entry) for entry in criteria]
        p = [float(probs[lvl]) for lvl in levels]
        return {"p": p, "conf": max(p), "choice": None}
    # noul: [p_no, p_yes] — the wire's [1-n, n] convention.
    p_no = float(probs["false"])
    p_yes = float(probs["true"])
    return {"p": [p_no, p_yes], "conf": max(p_no, p_yes), "choice": None}


def main() -> int:
    device = resolve_device(sys.argv[1] if len(sys.argv) > 1 else None)
    model_id = os.environ.get("BEKKO_MODEL", DEFAULT_MODEL)
    revision = os.environ.get("BEKKO_REVISION", DEFAULT_REVISION)

    # UTF-8 pipes before any import side effect prints a glyph (cp874 class).
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
        except (AttributeError, ValueError):
            pass

    from transformers.dynamic_module_utils import get_class_from_dynamic_module

    # The loader prints HF banners — keep them OFF the protocol channel.
    import contextlib

    with contextlib.redirect_stdout(sys.stderr):
        model_class = get_class_from_dynamic_module(
            "inference_v0.BekkoSentenceTransformer",
            model_id,
            revision=revision,
            code_revision=revision,
        )
        model = model_class(
            model_id,
            revision=revision,
            trust_remote_code=True,
            device=device,
        ).eval()

    dev = str(getattr(model[0], "device", device)) if isinstance(model, (list, tuple)) else str(device)
    print(json.dumps({"ready": True, "lane": "bekko", "model": model_id,
                      "revision": revision, "device": dev}), flush=True)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        req = json.loads(line)
        inp = build_input(req["state"], req["questions"])
        result = model.predict(inp, show_progress_bar=False)
        answers = {}
        for q in req["questions"]:
            answers[q["qid"]] = answer_for(q, result)
        print(json.dumps({"answers": answers}), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
