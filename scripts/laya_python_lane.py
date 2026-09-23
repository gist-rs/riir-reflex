#!/usr/bin/env python3
"""The laya-python bench lane — the ORIGINAL torch reference as an oracle.

Measurement-only. The PRODUCT lane is the riir backend (src/laya/riir);
the owner's "no Python anywhere" directive governs the shipped binary,
not the bench reference (the probe_orig_laya_latency.py precedent).

The harness (src/harness/runner.rs) spawns ONE process per (suite,
checkpoint) and speaks a line protocol on stdin/stdout:

  request  (one JSON line):  {"state": <value>, "questions": [{"qid", "def"}]}
  response (one JSON line):  {"answers": {"<qid>": {"p": [...], "conf": c,
                              "choice": <key>|null, "noul": <x>|null}}}

"p" is the probability vector in the question's OWN option order (criteria
key order for choice, level index for score, [1-n, n] for noul) — the
reference's own rounded-4 values from `system_one`, taken as-is. The first
line on stdout is {"ready": true, ...} once the checkpoint is loaded.

Usage: laya_python_lane.py <english|multilingual|typed> [device]

Weights resolve from the shared flat cache (the SAME files the riir lane
SHA-256/BLAKE3-verifies): LAYA_PY_WEIGHTS > LAYA_HOME >
~/.cache/riir-reflex/laya. Small files are COPIED into a throwaway
hub-shaped shim — the reference's loader rewrites tokenizer_config.json in
place, and the pinned cache must not move — while model.safetensors is
symlinked. `.raw/laya` (the pinned reference checkout) supplies the code.
"""

import json
import os
import shutil
import sys
import tempfile
from contextlib import redirect_stdout

CKPTS = {"english": "english", "multilingual": "multilingual", "typed": "typed"}

SMALL_FILES = ("rl_agent_config.json", "tokenizer.json", "tokenizer_config.json",
               "encoder_config.json")


def weights_root() -> str:
    for var in ("LAYA_PY_WEIGHTS", "LAYA_HOME"):
        d = os.environ.get(var)
        if d:
            return d
    return os.path.join(os.path.expanduser("~"), ".cache", "riir-reflex", "laya")


def build_shim(root: str, sub: str) -> str:
    """Hub-shaped view of the flat cache dir (copied configs, symlinked weights)."""
    src = os.path.join(root, sub)
    missing = [f for f in ("model.safetensors",) + SMALL_FILES
               if not os.path.isfile(os.path.join(src, f))]
    if missing:
        sys.exit(f"laya-python lane: missing in {src}: {', '.join(missing)} "
                 f"(fetch the weights first — the riir lane's ensure_checkpoint downloads them)")
    shim = tempfile.mkdtemp(prefix="laya-py-lane-")
    for name in SMALL_FILES:
        shutil.copyfile(os.path.join(src, name), os.path.join(shim, name))
    os.symlink(os.path.abspath(os.path.join(src, "model.safetensors")),
               os.path.join(shim, "model.safetensors"))
    os.makedirs(os.path.join(shim, "tokenizer"))
    os.rename(os.path.join(shim, "tokenizer.json"),
              os.path.join(shim, "tokenizer", "tokenizer.json"))
    os.rename(os.path.join(shim, "tokenizer_config.json"),
              os.path.join(shim, "tokenizer", "tokenizer_config.json"))
    os.makedirs(os.path.join(shim, "encoder"))
    shutil.copyfile(os.path.join(src, "encoder_config.json"),
                    os.path.join(shim, "encoder", "config.json"))
    return shim


def main() -> int:
    ck = sys.argv[1] if len(sys.argv) > 1 else "english"
    device = sys.argv[2] if len(sys.argv) > 2 else "mps"
    if ck not in CKPTS:
        print(f"laya-python lane: unknown checkpoint {ck!r} "
              f"(expected one of {', '.join(CKPTS)})", file=sys.stderr)
        return 2
    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    sys.path.insert(0, os.path.join(repo, ".raw", "laya"))
    import laya  # noqa: E402

    shim = build_shim(weights_root(), CKPTS[ck])
    try:
        # The loader prints fallback warnings on STDOUT — keep them off the
        # protocol channel.
        with redirect_stdout(sys.stderr):
            agent = laya.load(shim, device=device)
        print(json.dumps({"ready": True, "device": str(agent.device.type)}), flush=True)
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue
            req = json.loads(line)
            qs = {}
            for q in req["questions"]:
                d = {"type": q["def"]["type"], "instructions": q["def"]["instructions"]}
                if q["def"].get("criteria") is not None:
                    d["criteria"] = q["def"]["criteria"]
                qs[q["qid"]] = d
            out = agent.system_one(req["state"], qs)
            answers = {}
            for q in req["questions"]:
                a = out["answers"][q["qid"]]
                if a["type"] == "choice":
                    keys = list(q["def"]["criteria"].keys())
                    answers[q["qid"]] = {"p": [a["probabilities"][k] for k in keys],
                                         "conf": a["confidence"], "choice": a["choice"]}
                elif a["type"] == "score":
                    k = len(q["def"]["criteria"])
                    answers[q["qid"]] = {"p": [a["probabilities"][str(i)] for i in range(k)],
                                         "conf": a["confidence"]}
                else:
                    n = a["noul"]
                    answers[q["qid"]] = {"p": [round(1.0 - n, 4), n], "conf": a["confidence"]}
            print(json.dumps({"answers": answers}), flush=True)
        return 0
    finally:
        shutil.rmtree(shim, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
