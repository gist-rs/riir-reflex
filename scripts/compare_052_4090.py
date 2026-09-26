#!/usr/bin/env python3
"""Issue 039 T5 — cross-host bit-identity compare: 4090 re-run vs the M3 Bench-052 record.

Compares the deterministic axes per suite (modelless exact; laya accuracy
reported with expected cross-backend drift noted). Latency/seconds are box
properties and are excluded by design (Feature Flag Discipline G2 box-state
law).
"""
import json
import sys

M3 = ".benchmarks/052_stratified_readout/results.json"
W = ".benchmarks/052_4090/results.json"

MODELLESS_EXACT = [
    ("n_cases", "n_cases"),
    ("n_questions", "n_questions"),
    ("hard.accuracy", "modelless.hard.accuracy"),
    ("hard.macro_f1", "modelless.hard.macro_f1"),
    ("readout_ece", "modelless.readout_ece"),
    ("readout_ece_raw", "modelless.readout_ece_raw"),
    ("readout_ece_calibrated", "modelless.readout_ece_calibrated"),
    ("floor_ece", "modelless.floor_ece"),
    ("g1_pass", "modelless.g1_pass"),
    ("determinism_ok", "modelless.determinism_ok"),
    ("head_selection", "modelless.head_selection"),
    ("nb_selection", "modelless.nb_selection"),
    ("confusion", "modelless.confusion"),
    ("threshold_recommendation", "modelless.threshold_recommendation"),
]

LAYA_REPORT = [
    ("hard.accuracy", "laya.{ck}.hard.accuracy"),
    ("hard.macro_f1", "laya.{ck}.hard.macro_f1"),
    ("n_cases", "laya.{ck}.n_cases"),
    ("determinism_ok", "laya.{ck}.determinism_ok"),
]


def get(obj, dotted):
    cur = obj
    for part in dotted.split("."):
        if not isinstance(cur, dict) or part not in cur:
            return None
        cur = cur[part]
    return cur


def main(m3_path, w_path):
    m3 = json.load(open(m3_path))
    w = json.load(open(w_path))
    m3_suites = {s["name"]: s for s in m3["suites"]}
    w_suites = {s["name"]: s for s in w["suites"]}

    print("hosts:", m3["meta"]["host"], "->", w["meta"]["host"])
    print("git:", m3["meta"]["git_sha"], "->", w["meta"]["git_sha"])
    print("laya_device:", m3["meta"].get("laya_device"), "->", w["meta"].get("laya_device"))
    print()

    exact_fail = 0
    laya_diff = 0
    for name in sorted(m3_suites):
        if name not in w_suites:
            print(f"⛔ {name}: MISSING on 4090")
            exact_fail += 1
            continue
        m3s, ws = m3_suites[name], w_suites[name]
        mismatches = []
        for label, path in MODELLESS_EXACT:
            a, b = get(m3s, path), get(ws, path)
            if a != b:
                mismatches.append(f"{label}: {a!r} vs {b!r}")
        if mismatches:
            exact_fail += 1
            print(f"⛔ {name}: MODELLESS MISMATCH")
            for mm in mismatches:
                print(f"    {mm}")
        else:
            print(f"✓ {name}: modelless BIT-IDENTICAL")
        # laya per-checkpoint report
        m3l, wl = m3s.get("laya") or {}, ws.get("laya") or {}
        for ck in sorted(set(m3l) | set(wl)):
            diffs = []
            for label, tmpl in LAYA_REPORT:
                path = tmpl.format(ck=ck)
                a, b = get(m3s, path), get(ws, path)
                if a != b:
                    diffs.append(f"{label}: {a} vs {b}")
            if diffs:
                laya_diff += 1
                print(f"  ~ {name}/{ck}: laya differs (cross-backend, expected class)")
                for d in diffs:
                    print(f"      {d}")
    print()
    print(f"RESULT: modelless exact {'PASS' if exact_fail == 0 else f'FAIL ({exact_fail} suites)'}; "
          f"laya differing checkpoints: {laya_diff}")
    return 0 if exact_fail == 0 else 1


if __name__ == "__main__":
    a = sys.argv[1] if len(sys.argv) > 1 else M3
    b = sys.argv[2] if len(sys.argv) > 2 else W
    sys.exit(main(a, b))
