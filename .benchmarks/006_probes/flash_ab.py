#!/usr/bin/env python3
"""Issue 020 T10 rung 1 A/B: base vs flash-softmax-parallel binaries, two
resident lanes, interleaved + position-balanced; paired B/A enc ratio per
(round, length), median across rounds. Then a LAYA_KTIME pass (serialized
per-dispatch wall) for the flash_attn kernel alone."""
import os, random, statistics, sys
sys.path.insert(0, "/tmp/cf020b/riir-reflex/scripts")
import laya_seq_sweep as L
rounds = int(sys.argv[1]); lens = [int(x) for x in sys.argv[2].split(",")]
src = L.extract_fns(os.path.join(L.REPO, "src", "engine.rs"))[1]
sl = src.splitlines(keepends=True); states = {n: "".join(sl[:n]) for n in lens}
base = dict(os.environ, LAYA_DEVICE="metal", LAYA_SPLIT="1")
BIN = {"A": "/tmp/cf020b/bin_base", "B": "/tmp/cf020b/bin_patched"}
med = statistics.median


def lanes(env):
    return {x: L.Lane(x, [BIN[x], "english"], env) for x in "AB"}


ln = lanes(base)
seq = {n: ln["A"].call({"state": states[n], "questions": [L.Q_MODULE], "probe_seq": True})[1]["seq_len"] for n in lens}
for _ in range(3):
    for n in lens:
        for x in "AB":
            ln[x].call({"state": states[n], "questions": [L.Q_MODULE]})
t = {(x, n): [] for x in "AB" for n in lens}; ratio = {n: [] for n in lens}
for r in range(rounds):
    order = lens[:]; random.Random(r).shuffle(order)
    for j, n in enumerate(order):
        pair = "AB" if (r + j) % 2 == 0 else "BA"; got = {}
        for x in pair:
            got[x] = ln[x].call({"state": states[n], "questions": [L.Q_MODULE]})[1]["enc_ms"]
            t[(x, n)].append(got[x])
        ratio[n].append(got["B"] / got["A"])
    print(f"# round {r+1}/{rounds} load1={os.getloadavg()[0]:.2f}", file=sys.stderr, flush=True)
for x in "AB":
    ln[x].close()
print("| seq | enc A base | enc B patched | paired B/A median | B wins |")
print("|---|---|---|---|---|")
for n in lens:
    w = sum(x < 1 for x in ratio[n])
    print(f"| {seq[n]} | {med(t[('A', n)]):.1f} | {med(t[('B', n)]):.1f} | {med(ratio[n]):.3f} | {w}/{len(ratio[n])} |")

# Per-kernel: flash_attn serialized wall per forward (sum over its dispatches).
kl = lanes(dict(base, LAYA_KTIME="1"))
kf = {(x, n): [] for x in "AB" for n in lens}
for r in range(max(3, rounds // 2)):
    for j, n in enumerate(lens):
        for x in ("AB" if (r + j) % 2 == 0 else "BA"):
            resp = kl[x].call({"state": states[n], "questions": [L.Q_MODULE]})[1]
            kf[(x, n)].append(resp["ktime"].get("flash_attn", [0.0, 0])[0])
for x in "AB":
    kl[x].close()
print()
print("| seq | flash_attn ms A | flash_attn ms B | B/A |")
print("|---|---|---|---|")
for n in lens:
    a, b = med(kf[("A", n)]), med(kf[("B", n)])
    print(f"| {seq[n]} | {a:.2f} | {b:.2f} | {b / a:.3f} |")
