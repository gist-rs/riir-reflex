#!/usr/bin/env python3
"""The GLiNER comparison lane — fastino/GLiNER2.5-Decide as an external oracle.

Measurement-only (the `.issues/029` lane; the `.issues/019` clm-lane and
`laya_python_lane.py` precedents): THEIR package serves, our Rust harness
measures. The owner's "no Python anywhere" directive governs the shipped
binary, not the bench reference.

The harness (src/harness/runner.rs, `run_gliner_lane`) spawns ONE process
per suite and speaks the SAME line protocol as the laya-python oracle:

  request  (one JSON line):  {"state": <value>, "questions": [{"qid", "def"}]}
  response (one JSON line):  {"answers": {"<qid>": {"p": [...], "conf": c,
                              "choice": <key>|null}}}

"p" is the probability vector in the question's OWN option order (criteria
key order for choice, level order for score, [p_no, p_yes] for noul) —
their `ClassificationScores.probability` (softmax over labels for
single-label tasks, temperature 1, activation auto), taken as-is. "conf" is
the TOP label's probability (their own probability readout; they expose no
separate confidence scalar on the scoring path). "choice" is the argmax key
for choice questions.

Mapping (the PARITY LAW — what reaches THEIR scorer):
  - state: string → as-is; object → `key: value` lines (nested objects
    indented one level, arrays as `- item` lines); OUR rendering choice —
    GLiNER has no reference law for our wire states, and the card's own
    examples pass plain text.
  - choice: criteria {key: description|null} → ONE single-label task,
    labels = LabelSpec(key, description) in criteria order, instruction =
    the question's instructions.
  - score: criteria [level, ...] → ONE ordinal task (their ordered mode),
    labels = the level strings, instruction = the question's instructions.
  - noul: ONE single-label task, labels ["no", "yes"] (p = [p_no, p_yes] —
    the [1-n, n] convention every lane's noul row reports), instruction =
    the question's instructions.
  - ALL of a case's questions become tasks of ONE schema — one forward
    pass for the whole case (their several-decisions-at-once pattern).
    Task names are the wire qids (unique per case).

Usage: gliner_lane.py [device]      (device default: cuda, then cpu)
Env:    GLINER_MODEL (fastino/GLiNER2.5-Decide) · GLINER_PY_DEVICE · HF_HOME

The venv needs gliner2 + torch (CUDA build) + transformers + peft +
accelerate — gliner2 declares none of them (lazy imports). On the 4090 box:
`.raw/gliner-env/Scripts/python.exe` (uv venv, torch==2.14.0+cu126).
"""

import json
import os
import sys

DEFAULT_MODEL = "fastino/GLiNER2.5-Decide"

# gliner2 refuses these in label names, descriptions, and instructions (they
# are structural prompt tokens — a paren corrupts logit-to-label alignment).
# Wire criteria CAN carry them (dataset prose), so the lane strips them — a
# DISCLOSED transformation: the alternative is dropping every paren-carrying
# case, which is a coverage loss, not a purer measurement.
_RESERVED = ("[P]", "[L]", "[C]", "[E]", "[R]",
             "[DESCRIPTION]", "[EXAMPLE]", "[OUTPUT]", "(", ")")


def clean(value: str) -> str:
    for token in _RESERVED:
        value = value.replace(token, "")
    return value.strip()


def resolve_device(arg: str | None) -> str:
    if arg:
        return arg
    d = os.environ.get("GLINER_PY_DEVICE")
    if d:
        return d
    try:
        import torch

        return "cuda" if torch.cuda.is_available() else "cpu"
    except ImportError:
        return "cpu"


def state_text(value, depth: int = 0) -> str:
    """Render one wire state as plain prose (context first, key: value)."""
    pad = "  " * depth
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        lines = []
        for k, v in value.items():
            if isinstance(v, (dict, list)):
                lines.append(f"{pad}{k}:")
                lines.append(state_text(v, depth + 1))
            else:
                lines.append(f"{pad}{k}: {scalar(v)}")
        return "\n".join(lines)
    if isinstance(value, list):
        lines = []
        for item in value:
            if isinstance(item, (dict, list)):
                lines.append(f"{pad}- {state_text(item, depth + 1).lstrip()}")
            else:
                lines.append(f"{pad}- {scalar(item)}")
        return "\n".join(lines)
    return scalar(value)


def scalar(v) -> str:
    if isinstance(v, bool):
        return "true" if v else "false"
    if v is None:
        return "null"
    if isinstance(v, float) and v == int(v) and abs(v) < 1e16:
        return f"{int(v)}.0"
    return str(v)


def build_schema(questions):
    """Wire questions → ONE ClassificationSchema (a task per qid).

    Task names are the CLEANED qids (gliner2's structural-token check applies
    to names too); the caller maps answers back to the ORIGINAL qids.
    """
    from gliner2.classification import ClassificationSchema
    from gliner2.classification.schema import LabelSpec

    schema = ClassificationSchema()
    for q in questions:
        d = q["def"]
        kind = d["type"]
        task = clean(q["qid"]) or "_"
        instructions = clean(d.get("instructions") or "")
        criteria = d.get("criteria")
        if kind == "choice":
            if not isinstance(criteria, dict) or not criteria:
                raise ValueError(f"choice question {q['qid']}: criteria must be a non-empty object")
            labels = [
                LabelSpec(clean(key) or "_",
                          clean(desc) if isinstance(desc, str) and desc.strip() else None)
                for key, desc in criteria.items()
            ]
            schema.single(task, labels, instruction=instructions or None)
        elif kind == "score":
            if not isinstance(criteria, list) or not criteria:
                raise ValueError(f"score question {q['qid']}: criteria must be a non-empty array")
            levels = [clean(x if isinstance(x, str) else str(x)) or "_" for x in criteria]
            schema.ordinal(task, levels, instruction=instructions or None)
        elif kind == "noul":
            # [p_no, p_yes] — the wire's [1-n, n] convention.
            schema.single(task, ["no", "yes"], instruction=instructions or None)
        else:
            raise ValueError(f"question {q['qid']}: unknown kind {kind!r}")
    return schema


def main() -> int:
    device = resolve_device(sys.argv[1] if len(sys.argv) > 1 else None)
    model_id = os.environ.get("GLINER_MODEL", DEFAULT_MODEL)

    # torch needs UTF-8 pipes on Windows-class consoles before any import
    # side effect prints a glyph (the cp874 class).
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
        except (AttributeError, ValueError):
            pass

    from gliner2.classification.engine import Classifier

    # The loader prints its model banner to STDOUT — keep it OFF the protocol
    # channel (the laya-python lane's redirect law).
    import contextlib

    with contextlib.redirect_stdout(sys.stderr):
        clf = Classifier.from_pretrained(model_id)
        clf.to(device=device).eval()

    # The handshake the runner expects; the advertised model + device ride
    # the ready line (the runner stamps them into the lane's posture).
    print(json.dumps({"ready": True, "lane": "gliner", "model": model_id,
                      "device": str(clf.device)}), flush=True)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        req = json.loads(line)
        questions = req["questions"]
        text = state_text(req["state"])
        schema = build_schema(questions)
        scores = clf.score(text, schema)
        answers = {}
        for q in questions:
            qid = q["qid"]
            task = clean(qid) or "_"
            kind = q["def"]["type"]
            if kind == "choice":
                keys = [clean(k) or "_" for k in q["def"]["criteria"].keys()]
                p = [scores.probability(task, k) for k in keys]
                top = max(range(len(keys)), key=lambda i: p[i])
                answers[qid] = {"p": p, "conf": p[top], "choice": keys[top]}
            elif kind == "score":
                levels = [clean(x if isinstance(x, str) else str(x)) or "_"
                          for x in q["def"]["criteria"]]
                p = [scores.probability(task, lvl) for lvl in levels]
                answers[qid] = {"p": p, "conf": max(p)}
            else:  # noul: [p_no, p_yes]
                p_no = scores.probability(task, "no")
                p_yes = scores.probability(task, "yes")
                answers[qid] = {"p": [p_no, p_yes], "conf": max(p_no, p_yes)}
        print(json.dumps({"answers": answers}), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
