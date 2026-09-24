#!/usr/bin/env python3
"""Issue 020 length-scaling probe: rust Metal vs the python torch MPS
oracle on ONE input truncated to several lengths.

Measurement-only. The input is `code_fixtures` case 3 (`code:engine.rs:1`
— the SECOND top-level fn span of src/engine.rs, extracted with the
harness's own `extract_fns` rule), cut to its first N lines; the questions
are that suite's two (module choice, is_pub noul). Both lanes speak
`scripts/laya_python_lane.py`'s line protocol and are timed IDENTICALLY
here, as a request round-trip, so neither lane's timer is privileged.

Per round: lengths in a fresh shuffled order, lane order alternating per
(round, length) — position-balanced. Two request shapes are timed:
  1q  — one question per request (forward-for-forward: T5 cannot help
        python here, the reference batches only WITHIN a request)
  2q  — the suite's real shape (python batches both into one forward,
        rust runs two).

Usage: laya_seq_sweep.py <rust-binary> [rounds] [lines,lines,...]
"""

import json
import os
import random
import statistics
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LABELS = ["embed", "engine", "readout", "serve", "harness::metrics",
          "harness::suites", "laya::agent", "laya::router"]
Q_MODULE = {"qid": "module", "def": {
    "type": "choice",
    "instructions": "Which module of the riir-reflex crate is this Rust function from?",
    "criteria": {k: None for k in LABELS}}}
Q_PUB = {"qid": "is_pub", "def": {
    "type": "noul", "instructions": "Is this Rust function declared `pub`?"}}


def extract_fns(path):
    """runner.rs `extract_fns`, transcribed: fn decl → body up to a col-0 `}`
    or an 80-line cap."""
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    fns, i = [], 0
    while i < len(lines):
        t = lines[i].lstrip()
        if t.startswith("pub fn ") or t.startswith("fn "):
            start, body = i, []
            while i < len(lines) and i - start < 80:
                body.append(lines[i])
                if i > start and lines[i].startswith("}"):
                    break
                i += 1
            fns.append("\n".join(body) + "\n")
        i += 1
    return fns


class Lane:
    def __init__(self, name, argv, env=None):
        self.name = name
        self.p = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  text=True, encoding="utf-8", env=env, cwd=REPO)
        ready = json.loads(self.p.stdout.readline())
        assert ready.get("ready"), ready
        self.device = ready.get("device")

    def call(self, req):
        t0 = time.perf_counter()
        self.p.stdin.write(json.dumps(req) + "\n")
        self.p.stdin.flush()
        resp = json.loads(self.p.stdout.readline())
        return (time.perf_counter() - t0) * 1000.0, resp

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=60)


def main():
    rust_bin = sys.argv[1]
    rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    lens = ([int(x) for x in sys.argv[3].split(",")] if len(sys.argv) > 3
            else [5, 10, 20, 40, 80])
    src = extract_fns(os.path.join(REPO, "src", "engine.rs"))[1]
    src_lines = src.splitlines(keepends=True)
    states = {n: "".join(src_lines[:n]) for n in lens if n <= len(src_lines)}
    if len(src_lines) not in states:
        states[len(src_lines)] = src  # the exact harness case
    lens = sorted(states)

    env = dict(os.environ, LAYA_DEVICE="metal", LAYA_SPLIT="1")
    rust = Lane("rust", [rust_bin, "english"], env)
    py = Lane("py", [sys.executable, os.path.join(REPO, "scripts", "laya_python_lane.py"),
                     "english", "mps"])
    print(f"# lanes: rust={rust.device} py={py.device}; case-3 span = {len(src_lines)} lines",
          flush=True)

    seq = {n: rust.call({"state": states[n], "questions": [Q_MODULE], "probe_seq": True})[1]
           ["seq_len"] for n in lens}
    shapes = {"1q": [Q_MODULE], "2q": [Q_MODULE, Q_PUB]}
    # Warmup: every (lane, length, shape) twice, untimed.
    for _ in range(2):
        for n in lens:
            for qs in shapes.values():
                for lane in (rust, py):
                    lane.call({"state": states[n], "questions": qs})

    t = {(ln, sh, n): [] for ln in ("rust", "py") for sh in shapes for n in lens}
    split = {n: [] for n in lens}
    for r in range(rounds):
        load = os.getloadavg()[0]
        order = lens[:]
        random.Random(r).shuffle(order)
        for j, n in enumerate(order):
            for sh, qs in shapes.items():
                lanes = (rust, py) if (r + j) % 2 == 0 else (py, rust)
                for lane in lanes:
                    ms, resp = lane.call({"state": states[n], "questions": qs})
                    t[(lane.name, sh, n)].append(ms)
                    if lane is rust and sh == "1q":
                        split[n].append((resp["enc_ms"], resp["head_ms"]))
        print(f"# round {r + 1}/{rounds} load1={load:.2f}", flush=True)
    rust.close()
    py.close()

    med = statistics.median
    print("\n| lines | seq | rust 1q | py 1q | rust/py 1q | rust 2q | py 2q | rust/py 2q "
          "| rust enc | rust head | head share |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for n in lens:
        r1, p1 = med(t[("rust", "1q", n)]), med(t[("py", "1q", n)])
        r2, p2 = med(t[("rust", "2q", n)]), med(t[("py", "2q", n)])
        enc = med(e for e, _ in split[n])
        head = med(h for _, h in split[n])
        print(f"| {n} | {seq[n]} | {r1:.1f} | {p1:.1f} | {r1 / p1:.3f} | {r2:.1f} | {p2:.1f} "
              f"| {r2 / p2:.3f} | {enc:.1f} | {head:.1f} | {head / (enc + head):.0%} |")
    print(json.dumps({"seq": seq, "t": {f"{a}|{b}|{c}": v for (a, b, c), v in t.items()},
                      "split": split}), file=sys.stderr)


if __name__ == "__main__":
    main()
