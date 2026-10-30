#!/usr/bin/env python3
"""Issue 104 — pair two corpus-ab reads synth-arm-vs-synth-arm.

The learnability-gate class question is whether the gated corpus lifts the
modelless row MORE than the ungated one on the SAME frozen rows. Both reads
share an identical gold arm (arm A) and eval cases; the paired statistic is
computed over per-case synth-arm (arm B) correctness, disclosed as
``synth_ok_cases`` in each read's corpus_ab.json (additive, Issue 104).

Paired LB95 formula = the harness's own ``paired_lb95`` (the one home):
    d_i = ok_x(i) - ok_y(i) over n cases
    LB95 = mean(d) - 1.96 * sqrt(var(d, ddof=1) / n)
Concordant cases (both correct / both wrong) contribute d=0; only the
differing sets (x-only wins / y-only wins) carry signal. n is derived from
each read's acc_b and correct count and must agree across reads.

Usage:
    python3 scripts/corpus_ab_pair.py <x.json> <y.json>
      x = the candidate arm (e.g. gated read), y = the baseline (ungated).
Prints flips both directions + the paired LB95; exit 1 on inconsistent
inputs (different case counts, different gold-arm accuracy).
"""

import json
import math
import sys

# A non-UTF-8 console (cp874) would kill this script on its first
# non-ASCII glyph; degrade the glyph, never the verdict (katgpt-rs
# Issue 804). A captured stream without reconfigure is left alone.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

def load(path):
    with open(path, encoding="utf-8") as fh:
        doc = json.load(fh)
    suites = doc.get("suites")
    if not suites or len(suites) != 1:
        raise SystemExit(f"{path}: expected exactly one suite row, got {suites!r}")
    s = suites[0]
    ok = set(s.get("synth_ok_cases", []))
    acc_b = s["acc_synth"]
    acc_a = s["acc_gold"]
    if not s.get("b_only_cases") or not s.get("a_only_cases"):
        # Older reads carry the flip lists always; presence is a sanity pin.
        raise SystemExit(f"{path}: missing b_only/a_only flip lists — re-read needed")
    return {
        "path": path,
        "artifact": s.get("synth_artifact", "?"),
        "ok": ok,
        "acc_b": acc_b,
        "acc_a": acc_a,
        "verdict": s.get("verdict", "?"),
    }


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    x = load(sys.argv[1])
    y = load(sys.argv[2])

    # n from acc_b = |ok| / n — must agree across the two reads (the same
    # frozen case set), else the pairing is void.
    ns = []
    for r in (x, y):
        if r["acc_b"] <= 0:
            raise SystemExit(f"{r['path']}: acc_b = 0 — n underivable")
        n = round(len(r["ok"]) / r["acc_b"])
        ns.append(n)
    if ns[0] != ns[1]:
        raise SystemExit(
            f"case counts disagree: {x['path']} says n={ns[0]}, {y['path']} says n={ns[1]}"
            " — not the same frozen read; refuse"
        )
    n = ns[0]
    if n < len(x["ok"]) or n < len(y["ok"]):
        raise SystemExit("derived n smaller than a correct-set — inconsistent inputs")

    # The gold arm must be byte-identical across the two reads (same
    # posture, same corpus) — the reads' only difference is arm B's corpus.
    if abs(x["acc_a"] - y["acc_a"]) > 1e-12:
        raise SystemExit(
            f"gold arms differ: {x['acc_a']} vs {y['acc_a']} — the reads are not comparable"
        )

    x_only = sorted(x["ok"] - y["ok"])   # gated-correct, ungated-wrong
    y_only = sorted(y["ok"] - x["ok"])   # ungated-correct, gated-wrong
    d_wins = len(x_only)
    d_losses = len(y_only)
    m = (d_wins - d_losses) / n
    # var over the paired diffs: winners (1-m)^2, losers (-1-m)^2, the
    # n - differing concordant cases contribute m^2 each.
    differing = d_wins + d_losses
    if n >= 2:
        var = (d_wins * (1 - m) ** 2 + d_losses * (-1 - m) ** 2
               + (n - differing) * m * m) / (n - 1)
        lb95 = m - 1.96 * math.sqrt(var / n)
    else:
        var, lb95 = 0.0, 0.0

    print(f"x = {x['artifact']} ({x['path']})")
    print(f"y = {y['artifact']} ({y['path']})")
    print(f"n = {n} cases · acc_b x={x['acc_b']:.4f} y={y['acc_b']:.4f} · "
          f"delta={x['acc_b'] - y['acc_b']:+.4f}")
    print(f"x-only wins ({d_wins}): {x_only}")
    print(f"y-only wins ({d_losses}): {y_only}")
    print(f"paired mean d = {m:+.4f} · var = {var:.6f} · paired LB95 = {lb95:+.4f}")
    verdict = "PASS (gated lifts more than ungated)" if lb95 > 0 else \
              "NEGATIVE (gated does not beat ungated)"
    print(f"class verdict: {verdict}")


if __name__ == "__main__":
    main()
