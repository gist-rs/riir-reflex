#!/usr/bin/env python3
"""Issue 020 probe: per-kernel serialized wall (LAYA_KTIME=1) vs length.
Each dispatch is committed+waited alone, so per-kernel ms include ~one
dispatch round-trip each; the `copy`/`add` rows bound that overhead.
Usage: ktime_sweep.py <bin> <rounds> <lines,...> [flash=1|0]"""
import json, os, statistics, subprocess, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "scripts"))
import laya_seq_sweep as L

bin_, rounds = sys.argv[1], int(sys.argv[2])
lens = [int(x) for x in sys.argv[3].split(",")]
flash = sys.argv[4] if len(sys.argv) > 4 else "1"
src = L.extract_fns(os.path.join(L.REPO, "src", "engine.rs"))[1]
sl = src.splitlines(keepends=True)
states = {n: "".join(sl[:n]) for n in lens}
env = dict(os.environ, LAYA_DEVICE="metal", LAYA_SPLIT="1", LAYA_KTIME="1")
if flash == "0":
    env["LAYA_METAL_FLASH"] = "0"
lane = L.Lane("rust", [bin_, "english"], env)
seq = {n: lane.call({"state": states[n], "questions": [L.Q_MODULE], "probe_seq": True})[1]["seq_len"] for n in lens}
for _ in range(2):
    for n in lens:
        lane.call({"state": states[n], "questions": [L.Q_MODULE]})
acc = {n: [] for n in lens}
for r in range(rounds):
    for n in lens:
        acc[n].append(lane.call({"state": states[n], "questions": [L.Q_MODULE]})[1])
    print(f"# round {r+1} load1={os.getloadavg()[0]:.2f}", file=sys.stderr, flush=True)
lane.close()
med = statistics.median
names = sorted({k for n in lens for resp in acc[n] for k in resp["ktime"]})
print(f"# flash={flash}")
print("| seq | enc | " + " | ".join(names) + " | flash global(10) | flash slide(18) |")
print("|---" * (len(names) + 4) + "|")
for n in lens:
    row = [med(resp["ktime"].get(k, [0, 0])[0] for resp in acc[n]) for k in names]
    g = s = "-"
    if acc[n][0].get("kflash"):
        fl = acc[n][0]["kflash"]
        nl = len(fl)
        gs = [med(resp["kflash"][i] for resp in acc[n]) for i in range(nl)]
        g = f"{sum(gs[i] for i in range(nl) if i % 3 == 0):.1f}"
        s = f"{sum(gs[i] for i in range(nl) if i % 3 != 0):.1f}"
    enc = med(resp["enc_ms"] for resp in acc[n])
    print(f"| {seq[n]} | {enc:.1f} | " + " | ".join(f"{v:.1f}" for v in row) + f" | {g} | {s} |")
calls = {k: acc[lens[-1]][0]["ktime"][k][1] for k in names}
print("# calls/forward @max len:", calls)
