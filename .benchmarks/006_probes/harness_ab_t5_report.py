"""Paired-report for harness_ab_t5.sh (Bench 006 Addendum 7).

Per-suite × per-checkpoint: accuracy-equality across arms, median p50 per
arm, paired B/A ratios per round, wall-clock medians + win count.

Usage: python3 harness_ab_t5_report.py <logs-dir>
"""
import re, sys, statistics, glob

LOGS = sys.argv[1] if len(sys.argv) > 1 else "."
rows = {}
for f in glob.glob(f"{LOGS}/h_*_*.log"):
    m0 = re.search(r"h_(\d+)_([AB])\.log", f)
    if not m0:
        continue
    r, arm = int(m0.group(1)), m0.group(2)
    suite = None
    for line in open(f, encoding="utf-8"):
        m = re.search(r"═+ suite (\S+) ═+", line)
        if m:
            suite = m.group(1)
        m = re.search(r"laya\[(\w+)\]: acc ([\d.]+) .* p50 ([\d.]+) ms · ([\d.]+) s", line)
        if m and suite:
            rows[(r, arm, suite, m.group(1))] = (float(m.group(2)), float(m.group(3)), float(m.group(4)))

idx = open(f"{LOGS}/harness_ab.idx").read()
print(idx)

keys = sorted(rows)
suites = sorted({k[2] for k in keys})
cks = sorted({k[3] for k in keys})
rounds = sorted({k[0] for k in keys})
print("| suite | ck | acc A=B? | A p50 med | B p50 med | paired B/A p50 (per round) | B/A wall med | B wins wall |")
print("|---|---|---|---|---|---|---|---|")
for s in suites:
    for ck in cks:
        pr = [r for r in rounds if (r, "A", s, ck) in rows and (r, "B", s, ck) in rows]
        if not pr:
            continue
        acc = all(rows[(r, "A", s, ck)][0] == rows[(r, "B", s, ck)][0] for r in pr)
        rp = [rows[(r, "B", s, ck)][1] / rows[(r, "A", s, ck)][1] for r in pr]
        rw = [rows[(r, "B", s, ck)][2] / rows[(r, "A", s, ck)][2] for r in pr]
        med = statistics.median
        accs = "=" .join(f"{rows[(r, a, s, ck)][0]:.4f}" for r in pr[:1] for a in "AB")
        detail = "/".join(f"{x:.3f}" for x in rp)
        print(
            f"| {s} | {ck} | {acc} ({accs}) | {med(rows[(r, 'A', s, ck)][1] for r in pr):.1f} "
            f"| {med(rows[(r, 'B', s, ck)][1] for r in pr):.1f} | {med(rp):.3f} ({detail}) "
            f"| {med(rw):.3f} | {sum(x < 1 for x in rw)}/{len(rw)} |"
        )
