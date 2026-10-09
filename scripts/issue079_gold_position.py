#!/usr/bin/env python3
"""Issue 079 leakage decomposition (throwaway, measurement-only — never a product path).

Two questions the bench-128 finding left open, answered from data the probe
already produced plus the dataset itself:

  (a) GOLD POSITION — typed_decisions' choice questions are answered under
      the legacy `k == N` route binding (incumbent). If the gold labels sit
      non-uniformly by POSITION in the criteria order, the position-bound
      route term is a gold-position leak: part of the published hard accuracy
      is "the gold tends to sit at index i and index i scores highest", not
      content. This script replicates the suite builder's parse
      (`build_typed_decisions`) and the perm probe's deterministic stride
      sampling (`stride_indices`, max_cases = DEFAULT_PERM_MAX_CASES = 40 for
      the modelless control — bench 128's "60" was choice SLOTS, 40 sampled
      cases) and prints the gold-position histogram over the sampled slots.

  (b) PERMUTATION-AVERAGED BASELINE — the honest incumbent baseline under
      option permutation: mean over the K orderings' accuracies (per slot,
      pick under ordering o vs the gold label). The identity ordering (o=0)
      is the published posture; the perm-averaged figure is what the
      incumbent would score if presentations were order-randomized. Read
      from BOTH perm runs' `perm_probe.json` (modelless control lane).

Full-vector per-ordering Brier is NOT derivable: the probe record carries
only `p_ref` (probability mass on the identity ordering's pick). Disclosed
where a Brier would have gone; never imputed.

Output: a markdown table on stdout, `gold_position.json` and
`perm_summary.json` written beside the bench artifacts. Results land in the
bench record, never in code.
"""

from __future__ import annotations

import glob
import json
import math
import os
from collections import Counter
import sys

# Keep this instrument's verdict printable on a non-UTF-8 console
# (katgpt-rs Issue 804 / the 928 drift census): it prints non-ASCII glyphs,
# and print() raises UnicodeEncodeError on e.g. cp874 — the process then dies
# with NO verdict. backslashreplace degrades the glyph visibly and keeps
# ASCII exact, so a verdict line stays greppable. Best-effort: a detached or
# captured stream is left alone rather than made fatal at import.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, ".raw", "datasets", "typed_decisions")
OUT = os.path.join(ROOT, ".benchmarks", "131_kn_route_ab")

# perm_probe.rs: DEFAULT_PERM_MAX_CASES = 40 (the modelless control's cap;
# the laya lane is invoked at 20 to bound CPU wall time — different lane,
# same stride rule).
DEFAULT_PERM_MAX_CASES = 40


def py_str(v):
    """suites.rs `py_str` — Python str() for the JSON values gold labels
    arrive as (str identity; bool -> true/false; number -> JSON repr;
    null -> "None")."""
    if isinstance(v, bool):
        return "true" if v else "false"
    if v is None:
        return "None"
    if isinstance(v, str):
        return v
    return json.dumps(v)


def load_rows(data_dir):
    """runner.rs `load_rows`: every `{split}-*.json` in sorted filename
    order, `.rows[].row` concatenated into one list."""
    rows = []
    for path in sorted(glob.glob(os.path.join(data_dir, "test-*.json"))):
        with open(path, encoding="utf-8") as f:
            env = json.load(f)
        for r in env.get("rows", []):
            if isinstance(r, dict) and "row" in r:
                rows.append(r["row"])
    return rows


def parse_json_string_or_value(v):
    """suites.rs `parse_json_string_or_value`: a JSON string column is
    parsed; malformed -> None (row/question skipped downstream)."""
    if isinstance(v, str):
        try:
            return json.loads(v)
        except json.JSONDecodeError:
            return None
    return v


def value_as_i64(v):
    """suites.rs `value_as_i64` (only needed to mirror Score's gold read —
    choice questions never hit it)."""
    if isinstance(v, bool):
        return None
    if isinstance(v, (int, float)):
        return int(v)
    if isinstance(v, str):
        try:
            return int(v.strip())
        except ValueError:
            return None
    return None


def gold_answer_choice(qdef, g):
    """suites.rs `gold_answer` for QKind::Choice: keys = criteria order
    (dict insertion order, or list mapped through py_str); gold position =
    index of str(gold.label) in that order; None = the question is skipped
    at build time (so it never becomes a probe slot)."""
    crit = qdef.get("criteria")
    if isinstance(crit, dict):
        keys = list(crit.keys())
    elif isinstance(crit, list):
        keys = [py_str(v) for v in crit]
    else:
        return None
    if "label" not in g:
        return None
    label = py_str(g["label"])
    if label not in keys:
        return None
    return keys.index(label), keys


def build_cases(rows):
    """suites.rs `build_typed_decisions` (test_cap = 0: ALL rows) — only the
    fields the leakage question needs: case_id, and per-question (qid,
    n_options, gold position, gold label). Question skips replicated
    exactly: no workflow / malformed questions/gold / no gold entry /
    missing instructions / unresolvable gold mapping each drop the question;
    an empty case never becomes a case."""
    cases = []
    for pos, row in enumerate(rows):
        workflow = row.get("workflow")
        if not isinstance(workflow, str):
            continue
        questions_v = parse_json_string_or_value(row.get("questions"))
        gold_v = parse_json_string_or_value(row.get("gold"))
        if not isinstance(questions_v, dict) or not isinstance(gold_v, dict):
            continue
        questions = []
        for qid, qdef in questions_v.items():
            if qid not in gold_v:
                continue
            if not isinstance(qdef, dict):
                continue
            kind = qdef.get("type")
            if kind not in ("choice", "score", "noul"):
                continue
            instructions = qdef.get("instructions")
            if not isinstance(instructions, str):
                continue
            g = gold_v[qid]
            if not isinstance(g, dict):
                continue
            if kind == "choice":
                answered = gold_answer_choice(qdef, g)
                if answered is None:
                    continue
                idx, keys = answered
                questions.append(
                    {"qid": qid, "kind": "choice", "n": len(keys),
                     "gold_idx": idx, "gold": keys[idx]}
                )
            elif kind == "score":
                crit = qdef.get("criteria")
                n = len(crit) if isinstance(crit, list) else None
                if n is None or "label" not in g or value_as_i64(g["label"]) is None:
                    continue
                label = value_as_i64(g["label"])
                if label < 0 or label >= n:
                    continue
                questions.append(
                    {"qid": qid, "kind": "score", "n": n,
                     "gold_idx": label, "gold": str(label)}
                )
            else:  # noul — always resolvable (label true/false)
                if "label" not in g:
                    continue
                is_true = py_str(g["label"]).lower() == "true"
                questions.append(
                    {"qid": qid, "kind": "noul", "n": 2,
                     "gold_idx": int(is_true),
                     "gold": "true" if is_true else "false"}
                )
        if not questions:
            continue
        cases.append({"id": f"{workflow}:{pos}", "questions": questions})
    return cases


def is_choice_case(case):
    return any(q["kind"] == "choice" for q in case["questions"])


def stride_indices(n_total, max_cases):
    """perm_probe.rs `stride_indices`: all when they fit, else the first of
    every ceil(n/max) stride."""
    if n_total == 0:
        return []
    if n_total <= max_cases:
        return list(range(n_total))
    stride = math.ceil(n_total / max_cases)
    return list(range(0, n_total, stride))[:max_cases]


def gold_position_table(cases, sampled_case_ids):
    """The histogram over the probe's slots: sampled cases × their CHOICE
    questions (perm_suite's slot filter — the probe is choice-only; score's
    level list and noul's [false, true] pair are NOT slots), keyed by option
    count."""
    hist = {}  # n -> Counter(position)
    for c in cases:
        if c["id"] not in sampled_case_ids:
            continue
        for q in c["questions"]:
            if q["kind"] != "choice":
                continue
            hist.setdefault(q["n"], Counter())[q["gold_idx"]] += 1
    lines = ["| n_options | slots | " + " | ".join(f"pos {i}" for i in range(5)) +
             " | chance |", "|---|---|" + "---|" * 6]
    total_slots = 0
    weighted_chance = 0.0
    for n in sorted(hist):
        cnt = hist[n]
        slots = sum(cnt.values())
        total_slots += slots
        weighted_chance += slots / n
        cells = []
        for i in range(5):
            c = cnt.get(i, 0)
            cells.append(f"{c} ({c / slots:.1%})" if slots else "0")
        lines.append(f"| {n} | {slots} | " + " | ".join(cells) +
                     f" | {1.0 / n:.4f} |")
    lines.append(f"| all | {total_slots} | | | | | {weighted_chance / total_slots:.4f} |"
                 if total_slots else "| all | 0 | — |")
    return "\n".join(lines), hist, total_slots


def load_perm_lane(path):
    with open(path, encoding="utf-8") as f:
        d = json.load(f)
    for suite in d.get("suites", []):
        for lane in suite.get("lanes", []):
            if lane.get("lane") == "modelless":
                return lane, d.get("meta", {})
    return None, d.get("meta", {})


def perm_summary(lane, gold_by_slot):
    """Permutation-averaged accuracy over the K orderings + the tie census.
    Slots missing from the gold map (canary, or a parse divergence) are
    COUNTED and never folded into the accuracy population."""
    per_order_acc = []
    per_order_pref = []
    unmatched = 0
    n = 0
    correct = [0] * len(lane["cases"][0]["pick_labels"]) if lane["cases"] else []
    ties = 0  # p_ref == 1/n_options across every ordering (an all-way tie)
    for c in lane["cases"]:
        key = (c["case_id"], c["qid"])
        if key not in gold_by_slot:
            unmatched += 1
            continue
        gold = gold_by_slot[key]
        n += 1
        for o, pick in enumerate(c["pick_labels"]):
            if pick == gold:
                correct[o] += 1
        per_order_pref.append(sum(c["p_ref"]) / len(c["p_ref"]))
        if all(abs(p - 1.0 / c["n_options"]) < 1e-9 for p in c["p_ref"]):
            ties += 1
    k = len(correct)
    per_order_acc = [c / n for c in correct] if n else []
    return {
        "n_slots_scored": n,
        "n_slots_unmatched": unmatched,
        "n_orderings": k,
        "identity_accuracy": per_order_acc[0] if per_order_acc else None,
        "per_ordering_accuracy": [round(a, 6) for a in per_order_acc],
        "permutation_averaged_accuracy": (
            round(sum(per_order_acc) / k, 6) if per_order_acc else None
        ),
        "mean_p_ref_identity_ordering": (
            round(sum(c["p_ref"][0] for c in lane["cases"]) / len(lane["cases"]), 6)
            if lane["cases"] else None
        ),
        "all_way_tie_slots": ties,
        "flips": lane.get("flips"),
        "flip_rate": lane.get("flip_rate"),
        "median_swing_pt": lane.get("median_swing_pt"),
        "max_swing_pt": lane.get("max_swing_pt"),
        "verdict": lane.get("verdict"),
        "control_invariant": lane.get("control_invariant"),
        "brier": "NOT DERIVABLE — the probe record carries p_ref (mass on the "
                 "identity pick) only, never the full per-ordering vector",
    }


def main():
    rows = load_rows(DATA)
    cases = build_cases(rows)
    # perm_suite's filter: cases carrying at least one CHOICE question (the
    # probe is choice-only), in dataset order.
    choice_cases = [c for c in cases if is_choice_case(c)]

    sampled = stride_indices(len(choice_cases), DEFAULT_PERM_MAX_CASES)
    sampled_ids = {choice_cases[i]["id"] for i in sampled}
    n_options_seen = Counter(
        q["n"] for c in choice_cases for q in c["questions"]
        if q["kind"] == "choice"
    )

    table, hist, total_slots = gold_position_table(cases, sampled_ids)

    print("## Gold position — typed_decisions choice slots (the probe's population)")
    print()
    print(f"- dataset rows: {len(rows)} → cases: {len(cases)} → choice-carrying "
          f"cases: {len(choice_cases)}")
    print(f"- sampling: stride_indices({len(choice_cases)}, "
          f"{DEFAULT_PERM_MAX_CASES}) → {len(sampled)} case(s) "
          f"(stride = {math.ceil(len(choice_cases) / DEFAULT_PERM_MAX_CASES)}) — "
          f"the modelless control's cap; bench 128's '60' was choice SLOTS")
    print(f"- choice questions by option count over ALL choice-carrying cases: "
          f"{dict(sorted(n_options_seen.items()))}")
    print(table)
    print()

    # full-suite histogram as context (every CHOICE question, not just sampled)
    full_hist = {}
    for c in cases:
        for q in c["questions"]:
            if q["kind"] != "choice":
                continue
            full_hist.setdefault(q["n"], Counter())[q["gold_idx"]] += 1

    gold_by_slot = {}
    for c in cases:
        if c["id"] not in sampled_ids:
            continue
        for q in c["questions"]:
            gold_by_slot[(c["id"], q["qid"])] = q["gold"]

    summary = {
        "sampling": {
            "dataset_rows": len(rows),
            "cases": len(cases),
            "choice_carrying_cases": len(choice_cases),
            "max_cases": DEFAULT_PERM_MAX_CASES,
            "sampled_cases": len(sampled),
            "stride": math.ceil(len(choice_cases) / DEFAULT_PERM_MAX_CASES)
            if choice_cases else 0,
            "slots": total_slots,
        },
        "gold_position_sampled": {
            str(n): {str(p): c.get(p, 0) for p in range(5)}
            for n, c in sorted(hist.items())
        },
        "gold_position_full_suite": {
            str(n): {str(p): c.get(p, 0) for p in range(5)}
            for n, c in sorted(full_hist.items())
        },
    }

    perm = {}
    for tag in ("incumbent", "content", "content_per_byte"):
        path = os.path.join(OUT, f"perm_{tag}", "perm_probe.json")
        if not os.path.exists(path):
            print(f"- perm[{tag}]: perm_probe.json ABSENT — skipped loud")
            continue
        lane, meta = load_perm_lane(path)
        if lane is None:
            print(f"- perm[{tag}]: no modelless lane in the record — skipped loud")
            continue
        perm[tag] = {
            "route_binding": meta.get("route_binding"),
            "git_sha": meta.get("git_sha"),
            "k": meta.get("k"),
            "max_cases": meta.get("max_cases"),
            **perm_summary(lane, gold_by_slot),
        }
        p = perm[tag]
        print(f"## Perm-averaged baseline — {tag} ({p['route_binding']})")
        print()
        print(f"- sha {p['git_sha']} · K {p['k']} orderings · slots scored "
              f"{p['n_slots_scored']} (unmatched {p['n_slots_unmatched']})")
        print(f"- identity-ordering accuracy: **{p['identity_accuracy']:.4f}**")
        print(f"- per-ordering: {p['per_ordering_accuracy']}")
        print(f"- **permutation-averaged: {p['permutation_averaged_accuracy']:.4f}**")
        print(f"- all-way-tie slots (p_ref == 1/k at every ordering): "
              f"{p['all_way_tie_slots']}/{p['n_slots_scored']}")
        print(f"- flips {p['flips']} (rate {p['flip_rate']}) · median "
              f"{p['median_swing_pt']} pt · verdict {p['verdict']} · "
              f"byte-identity {p['control_invariant']}")
        print(f"- brier: {p['brier']}")
        print()

    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, "gold_position.json"), "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=1)
    with open(os.path.join(OUT, "perm_summary.json"), "w", encoding="utf-8") as f:
        json.dump(perm, f, indent=1)
    print(f"wrote {os.path.join(OUT, 'gold_position.json')}")
    print(f"wrote {os.path.join(OUT, 'perm_summary.json')}")


if __name__ == "__main__":
    main()
