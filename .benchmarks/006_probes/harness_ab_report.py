import re, statistics, glob
rows = {}
for f in glob.glob("/tmp/cf020b/h_*_*.log"):
    r, arm = re.search(r"h_(\d+)_([AB])\.log", f).groups()
    suite = None
    for line in open(f, encoding="utf-8"):
        m = re.search(r"═══ suite (\S+) ═══", line)
        if m: suite = m.group(1)
        m = re.search(r"laya\[english\]: acc ([\d.]+) .* p50 ([\d.]+) ms · ([\d.]+) s", line)
        if m and suite:
            rows[(int(r), arm, suite)] = tuple(map(float, m.groups()))
idx = open("/tmp/cf020b/harness_ab.idx").read()
print(idx)
suites = sorted({k[2] for k in rows}); rounds = sorted({k[0] for k in rows})
print("| suite | acc A=B? | A p50 (med) | B p50 (med) | paired B/A p50 | A wall s | B wall s | paired B/A wall | B wins wall |")
print("|---|---|---|---|---|---|---|---|---|")
for s in suites:
    pr = [r for r in rounds if (r, "A", s) in rows and (r, "B", s) in rows]
    acc = all(rows[(r, "A", s)][0] == rows[(r, "B", s)][0] for r in pr)
    rp = [rows[(r, "B", s)][1] / rows[(r, "A", s)][1] for r in pr]
    rw = [rows[(r, "B", s)][2] / rows[(r, "A", s)][2] for r in pr]
    med = statistics.median
    print(f"| {s} | {acc} | {med(rows[(r,'A',s)][1] for r in pr):.1f} | {med(rows[(r,'B',s)][1] for r in pr):.1f} | {med(rp):.3f} | "
          f"{med(rows[(r,'A',s)][2] for r in pr):.1f} | {med(rows[(r,'B',s)][2] for r in pr):.1f} | {med(rw):.3f} | {sum(x<1 for x in rw)}/{len(rw)} |")
