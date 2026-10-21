#!/usr/bin/env python3
"""Issue 089 T1 — the margin-null signal sizing (report; reads perm_probe.json).

THE PRE-REGISTRATION (pinned in .issues/089 before any result was read):

Floors (verbatim from the issue, tighten-only):
  PRIMARY   : ΔAUROC ≥ 0.02 over the margin-only baseline, on at least one lane.
  SECONDARY : partial correlation ≥ 0.05 (disagreement ↔ error, controlling margin).
  If both are read, each is judged at α/2 (Bonferroni split).
  The typed_decisions DEFECT lane cannot satisfy the "at least one lane" bar.

Mechanical definitions (pinned HERE, before execution; issue-089 vocabulary):
  untied      : `tie == false` — the codebase's pinned exact-top-2-f64-equality
                tie class (permutation.rs `case_spread`), no fp-floor discretion.
  disagreement: `flipped == true` — any non-identity ordering's top pick
                differs from the identity top pick (the recorded field).
  error       : `ref_pick != gold`.
  margin      : `margin_top2` — the identity ordering's top-2 margin p1 − p2.
  baseline    : score = −margin (larger = more error-likely).
  signal      : score = flipped ∈ {0, 1}.
  AUROC       : Mann–Whitney U with midranks for ties (margin ties exist).
  ΔAUROC      : AUROC(signal) − AUROC(baseline), same population.
  partial r   : point-biserial partial correlation of (flipped, error) given
                margin: (r_fe − r_fm·r_em)/sqrt((1−r_fm²)(1−r_em²)).
  lane level  : the suite cells POOLED per lane (the issue's "per lane");
                per-suite rows are disclosed beside, never substituted.
  judging     : the pre-registered point bars judge; a 2000-resample bootstrap
                CI over the pooled population rides each ΔAUROC as DISCLOSURE
                (n is small — 40/suite — so low-support passes must be visible,
                never silently green).

Population rules:
  - only rows with gold != null and margin_top2 != null and !tie;
  - the modelless control's typed_decisions cell is the DEFECT lane —
    disclosed separately, excluded from the "at least one lane" bar;
  - lanes not measured on this box (drex / agentjev / d1 — their servers are
    4090-hosted, box unreachable today) are named in the record as
    not-measured postures, never folded into any verdict.

Exit: always 0 when the records parse (a report, not a gate); 2 on a
missing/unparsable record (loud, never a quiet pass).
"""

from __future__ import annotations

import json
import math
import os
import random
import sys

# A non-UTF-8 console (cp874) would kill this script on its first
# non-ASCII glyph; degrade the glyph, never the verdict (katgpt-rs
# Issue 804). A captured stream without reconfigure is left alone.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# lane-name prefix -> is the typed defect lane excluded from the ≥1-lane bar
DEFECT_LANE = "modelless/typed-decisions"


def rankdata_mid(scores):
    """1-based midranks (ties average), ascending."""
    order = sorted(range(len(scores)), key=lambda i: scores[i])
    ranks = [0.0] * len(scores)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and scores[order[j + 1]] == scores[order[i]]:
            j += 1
        mid = (i + j) / 2.0 + 1.0
        for k in range(i, j + 1):
            ranks[order[k]] = mid
        i = j + 1
    return ranks


def auroc(scores, labels):
    """Mann–Whitney AUROC: P(score_pos > score_neg) + 0.5·P(equal).
    labels: 1 = positive (error), 0 = negative (correct)."""
    pos = [s for s, l in zip(scores, labels) if l == 1]
    neg = [s for s, l in zip(scores, labels) if l == 0]
    if not pos or not neg:
        return None
    ranks = rankdata_mid(scores)
    r_pos = sum(r for r, l in zip(ranks, labels) if l == 1)
    u = r_pos - len(pos) * (len(pos) + 1) / 2.0
    return u / (len(pos) * len(neg))


def pearson(xs, ys):
    n = len(xs)
    if n < 3:
        return None
    mx = sum(xs) / n
    my = sum(ys) / n
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    sx = math.sqrt(sum((x - mx) ** 2 for x in xs))
    sy = math.sqrt(sum((y - my) ** 2 for y in ys))
    if sx == 0.0 or sy == 0.0:
        return None
    return sxy / (sx * sy)


def partial_point_biserial(flipped, errors, margins):
    """partial r of (flipped, error) controlling margin (pre-registration)."""
    r_fe = pearson(flipped, errors)
    r_fm = pearson(flipped, margins)
    r_em = pearson(errors, margins)
    if None in (r_fe, r_fm, r_em):
        return None
    den = (1 - r_fm**2) * (1 - r_em**2)
    if den <= 0:
        return None
    return (r_fe - r_fm * r_em) / math.sqrt(den)


def bootstrap_delta(errors, margins, flipped, n_boot=2000, seed=89):
    """Percentile CI over the pooled population (disclosure only)."""
    n = len(errors)
    if n == 0:
        return None
    rng = random.Random(seed)
    deltas = []
    for _ in range(n_boot):
        e, m, f = [], [], []
        for i in (rng.randrange(n) for _ in range(n)):
            e.append(errors[i])
            m.append(margins[i])
            f.append(flipped[i])
        a_sig = auroc(f, e)
        a_base = auroc([-x for x in m], e)
        if a_sig is not None and a_base is not None:
            deltas.append(a_sig - a_base)
    if not deltas:
        return None
    deltas.sort()
    return (deltas[int(0.025 * len(deltas))], deltas[int(0.975 * len(deltas))])


def cell_rows(lane_report):
    """The analysis population of one lane cell: untied, gold, margin."""
    rows = []
    for c in lane_report.get("cases", []):
        gold = c.get("gold")
        m = c.get("margin_top2")
        if gold is None or m is None or c.get("tie"):
            continue
        rows.append(
            {
                "case": f"{c['case_id']}#{c['qid']}",
                "flipped": 1 if c["flipped"] else 0,
                "error": 1 if c["ref_pick"] != gold else 0,
                "margin": float(m),
            }
        )
    return rows


def analyze_cell(name, rows):
    n = len(rows)
    flipped = [r["flipped"] for r in rows]
    errors = [r["error"] for r in rows]
    margins = [r["margin"] for r in rows]
    n_err = sum(errors)
    n_dis = sum(flipped)
    out = {
        "cell": name,
        "n": n,
        "errors": n_err,
        "disagreements": n_dis,
        "untied_disagreement_rate": (n_dis / n) if n else None,
        "error_rate": (n_err / n) if n else None,
    }
    if n == 0 or n_err in (0, n):
        out.update(
            {
                "auroc_baseline": None,
                "auroc_signal": None,
                "delta_auroc": None,
                "partial_r": None,
                "note": "degenerate (no untied rows, or all-one-class outcome)",
            }
        )
        return out
    a_sig = auroc(flipped, errors)
    a_base = auroc([-m for m in margins], errors)
    out.update(
        {
            "auroc_baseline": a_base,
            "auroc_signal": a_sig,
            "delta_auroc": (a_sig - a_base) if (a_sig is not None and a_base is not None) else None,
            "partial_r": partial_point_biserial(flipped, errors, margins),
        }
    )
    return out


def load(path):
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)


def main(argv):
    paths = argv[1:]
    if not paths:
        print("usage: issue089_margin_null.py <perm_probe.json> [...]", file=sys.stderr)
        return 2
    meta_first = load(paths[0]).get("meta", {})
    print(
        f"issue089 T1 margin-null — records: {len(paths)} · K={meta_first.get('k')} "
        f"· host={meta_first.get('host')} · sha={meta_first.get('git_sha')}"
    )
    verdict_lines = []
    for p in paths:
        if not os.path.exists(p):
            print(f"MISSING RECORD: {p}", file=sys.stderr)
            return 2
        try:
            doc = load(p)
        except json.JSONDecodeError as e:
            print(f"UNPARSABLE RECORD: {p}: {e}", file=sys.stderr)
            return 2
        for suite_rep in doc.get("suites", []):
            suite = suite_rep["suite"]
            for lane_rep in suite_rep.get("lanes", []):
                lane = lane_rep["lane"]
                rows = cell_rows(lane_rep)
                per = analyze_cell(f"{lane} × {suite}", rows)
                print_cell(per)
    # Lane-level pooled read: re-walk collecting per lane.
    pooled = {}
    for p in paths:
        doc = load(p)
        for suite_rep in doc.get("suites", []):
            for lane_rep in suite_rep.get("lanes", []):
                pooled.setdefault(lane_rep["lane"], []).extend(cell_rows(lane_rep))
    print("\n── lane-level (pooled over suites) ──")
    verdicts = []
    for lane in sorted(pooled):
        per = analyze_cell(lane, pooled[lane])
        print_cell(per)
        if per["delta_auroc"] is not None:
            ci = bootstrap_delta(
                [r["error"] for r in pooled[lane]],
                [r["margin"] for r in pooled[lane]],
                [r["flipped"] for r in pooled[lane]],
            )
            per["delta_ci95"] = ci
            print(f"    ΔAUROC 95% bootstrap CI: [{ci[0]:+.4f}, {ci[1]:+.4f}]" if ci else "")
        is_defect = "typed" in lane
        bar = per["delta_auroc"] is not None and per["delta_auroc"] >= 0.02
        bar2 = per["partial_r"] is not None and per["partial_r"] >= 0.05
        tag = " [DEFECT LANE — excluded from the ≥1-lane bar]" if is_defect else ""
        verdicts.append(
            f"  {lane}: primary {'PASS' if bar else 'MISS'}{tag}"
            + (f" · secondary {'PASS' if bar2 else 'MISS'}" if per["partial_r"] is not None else "")
        )
    print("\n── verdicts (pre-registered floors: ΔAUROC ≥ 0.02 primary on ≥1 lane; "
          "partial r ≥ 0.05 secondary; Bonferroni α/2; typed defect lane excluded) ──")
    for line in verdicts:
        print(line)
    return 0


def print_cell(per):
    da = per["delta_auroc"]
    pr = per["partial_r"]
    print(
        f"  {per['cell']:<34} n={per['n']:<4} err={per['errors']:<3} dis={per['disagreements']:<3} "
        f"dis_rate={fmt(per['untied_disagreement_rate'])} err_rate={fmt(per['error_rate'])} "
        f"AUROC(margin)={fmt(per['auroc_baseline'])} AUROC(dis)={fmt(per['auroc_signal'])} "
        f"Δ={fmt(da)} partial_r={fmt(pr)}"
    )


def fmt(x):
    return "—" if x is None else f"{x:+.4f}" if isinstance(x, float) else str(x)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
