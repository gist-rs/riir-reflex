#!/usr/bin/env python3
"""Issue 108 cross-family recheck — three-category + decisive-only kappa.

Joins two ceiling-probe runs (different judge seats, same suite front and
seed => the same 600-item sample) on (case_id, gold_label) and measures
verdict agreement over the items BOTH judges adjudicated (support reading
not in {unsupported_scope, unknown}):

  - kappa (three-category: supported / not_supported / unclear)
  - kappa (decisive-only: supported / not_supported — unclear dropped)
  - raw agreement both ways, confusion table, per-seat verdict counts.

The OntoSmith discipline (issue 108): report kappa BOTH ways — raw
agreement overstates at skewed base rates; decisive-only drops the
unclear class rather than folding it.
"""
import json
import sys
from collections import Counter

CATS = ("supported", "not_supported", "unclear")
NON_ADJUDICATED = {"unsupported_scope", "unknown"}


def load_items(path):
    with open(path) as f:
        d = json.load(f)
    return d["meta"], d["items"]


def kappa(a, b, cats):
    """Cohen's kappa over paired categorical readings."""
    n = len(a)
    if n == 0:
        return float("nan")
    po = sum(1 for x, y in zip(a, b) if x == y) / n
    ca, cb = Counter(a), Counter(b)
    pe = sum(ca.get(c, 0) * cb.get(c, 0) for c in cats) / (n * n)
    if pe == 1.0:
        return float("nan")
    return (po - pe) / (1.0 - pe)


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        print("usage: ceiling_kappa.py <runA.json> <runB.json>")
        return 2
    meta_a, items_a = load_items(sys.argv[1])
    meta_b, items_b = load_items(sys.argv[2])

    idx_a = {(i["case_id"], i["gold_label"]): i["support"] for i in items_a}
    idx_b = {(i["case_id"], i["gold_label"]): i["support"] for i in items_b}
    shared_keys = sorted(set(idx_a) & set(idx_b))
    mismatched_keys = [k for k in set(idx_a) ^ set(idx_b)]
    print(f"run A judge={meta_a['judge_model']} host={meta_a['host']} "
          f"sha={meta_a['git_sha'][:12]} date={meta_a['date_utc']}")
    print(f"run B judge={meta_b['judge_model']} host={meta_b['host']} "
          f"sha={meta_b['git_sha'][:12]} date={meta_b['date_utc']}")
    print(f"sample alignment: shared (case_id, gold_label) keys = "
          f"{len(shared_keys)} / A={len(idx_a)} / B={len(idx_b)}; "
          f"unmatched = {len(mismatched_keys)}")
    if mismatched_keys[:5]:
        print("  unmatched sample:", [str(k) for k in mismatched_keys[:5]])

    pairs_all = [(idx_a[k], idx_b[k]) for k in shared_keys]
    adjudicated = [(a, b) for a, b in pairs_all
                   if a not in NON_ADJUDICATED and b not in NON_ADJUDICATED]
    print(f"both-adjudicated pairs: {len(adjudicated)} "
          f"(of {len(pairs_all)} shared; rest = single-seat "
          f"unknown/unsupported_scope)")

    a = [p[0] for p in adjudicated]
    b = [p[1] for p in adjudicated]
    agree = sum(1 for x, y in zip(a, b) if x == y)
    print(f"raw agreement: {agree}/{len(a)} = {agree / max(1, len(a)):.4f}")

    k3 = kappa(a, b, CATS)
    dec = [(x, y) for x, y in zip(a, b)
           if x != "unclear" and y != "unclear"]
    kdec = kappa([p[0] for p in dec], [p[1] for p in dec],
                 ("supported", "not_supported"))
    bar = 0.70
    print(f"kappa three-category  = {k3:.4f}  (bar {bar} "
          f"-> {'PASS' if k3 >= bar else 'FAIL'})")
    print(f"kappa decisive-only   = {kdec:.4f} over n={len(dec)}  (bar {bar} "
          f"-> {'PASS' if kdec >= bar else 'FAIL'})")

    conf = Counter((x, y) for x, y in zip(a, b))
    print("\nconfusion (rows=A, cols=B):")
    cols = sorted({y for _, y in conf} | set(CATS))
    print("  " + " | ".join(f"{c:>14}" for c in ["A \\ B"] + cols))
    for r in sorted({x for x, _ in conf} | set(CATS)):
        row = [f"{conf.get((r, c), 0):>14}" for c in cols]
        print("  " + " | ".join(f"{r:>14}" if i == 0 else r
                                for i, r in enumerate([r] + row)))

    print("\nper-seat verdict counts (both-adjudicated set):")
    print(f"  A: {dict(Counter(a))}")
    print(f"  B: {dict(Counter(b))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
