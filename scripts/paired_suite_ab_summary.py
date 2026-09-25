#!/usr/bin/env python3
"""Render the paired per-suite A/B verdict from paired_suite_ab.sh's output.

Reads OUT_DIR's per-suite results.json files — the alternation driver saves
one directory per posture:
  <suite>/both/results.json   (even suites: rust laya + py in one run)
  <suite>/rust/results.json   (odd suites: rust-only run)
  <suite>/py/results.json     (odd suites: py-only run)

For every suite × checkpoint the summary pairs the rust lane (`laya.<ckpt>`)
against the python reference (`laya["py/<ckpt>"]`) and reports:

  Δ = (rust − py) / py, per cell (p50 and p99)

The every-published-cell bar (issue 020) is Δ ≤ 0 on p50 AND p99 for every
cell — a WIN here, a LOSS there, with TIE the |Δ| < --tol band (default 2%:
ms-quantization on the short suites makes sub-2% unadjudicable; Bench 036's
own noise study is the reason the threshold is a flag, not a law).

Tail support prints beside every p99 (the workspace percentile rule: a p99
over < 100 cases is the max wearing a percentile's name).

Usage:
  python3 scripts/paired_suite_ab_summary.py OUT_DIR [--tol 0.02]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_suite(d: Path) -> dict:
    """One suite's lane readings keyed checkpoint → (p50, p99, support, acc),
    rust from `laya`, python from `laya["py/<ckpt>"]`."""
    out: dict[str, dict[str, tuple]] = {}
    for posture_dir, lane_kind in (("both", None), ("rust", "rust"), ("py", "py")):
        path = d / posture_dir / "results.json"
        if not path.is_file():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        for suite in data.get("suites", []):
            laya = suite.get("laya") or {}
            for key, res in laya.items():
                if key.startswith("py/"):
                    ck = key[3:]
                    slot = out.setdefault(ck, {})
                    slot["py"] = _reading(res)
                else:
                    slot = out.setdefault(key, {})
                    slot["rust"] = _reading(res)
    return out


def _reading(res: dict) -> tuple:
    return (
        res.get("latency_p50_ms"),
        res.get("latency_p99_ms"),
        res.get("latency_tail_support"),
        (res.get("hard") or {}).get("accuracy"),
    )


def fmt_delta(rust: float, py: float) -> str:
    if py == 0:
        return "n/a"
    return f"{(rust - py) / py * 100:+6.1f}%"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("out_dir", type=Path)
    ap.add_argument("--tol", type=float, default=0.02, help="tie band (fraction)")
    args = ap.parse_args()

    prov = args.out_dir / "provenance.txt"
    if prov.is_file():
        note = prov.read_text(encoding="utf-8").strip()
        if "NOT FOR PUBLICATION" in note:
            print(f"⛔ {note}")
    if (args.out_dir / "preflight.txt").is_file():
        for line in (args.out_dir / "preflight.txt").read_text(encoding="utf-8").splitlines():
            if line.startswith("PROVENANCE:"):
                print(line)
                break

    suites = sorted(p for p in args.out_dir.iterdir() if p.is_dir())
    rows: list[tuple] = []
    for sd in suites:
        lanes = load_suite(sd)
        for ck, slot in sorted(lanes.items()):
            if "rust" not in slot or "py" not in slot:
                missing = "rust" if "rust" not in slot else "py"
                print(f"⚠ {sd.name}/{ck}: no {missing} reading — skipping (loud, never fabricated)")
                continue
            rows.append((sd.name, ck, slot["rust"], slot["py"]))

    if not rows:
        print(f"REFUSE — no paired readings under {args.out_dir}: nothing to adjudicate")
        return 1

    print()
    print(f"{'suite':<22} {'ckpt':<14} {'rust p50':>9} {'py p50':>9} {'Δp50':>8} "
          f"{'rust p99':>9} {'py p99':>9} {'Δp99':>8} {'sup':>4}  verdict")
    p50_wins = p50_losses = p99_wins = p99_losses = ties = 0
    cell_ok = True
    for suite, ck, (r50, r99, sup, racc), (p50, p99, _psup, pacc) in rows:
        d50 = (r50 - p50) / p50 if p50 else float("inf")
        d99 = (r99 - p99) / p99 if p99 else float("inf")
        v50 = "WIN" if d50 <= -args.tol else ("LOSS" if d50 >= args.tol else "tie")
        v99 = "WIN" if d99 <= -args.tol else ("LOSS" if d99 >= args.tol else "tie")
        if v50 == "WIN":
            p50_wins += 1
        elif v50 == "LOSS":
            p50_losses += 1
        if v99 == "WIN":
            p99_wins += 1
        elif v99 == "LOSS":
            p99_losses += 1
        if "tie" in (v50, v99):
            ties += 1
        if d50 > 0 or d99 > 0:
            cell_ok = False
        acc = f"acc {racc:.3f}/{pacc:.3f}" if racc is not None and pacc is not None else ""
        print(f"{suite:<22} {ck:<14} {r50:>8.1f} {p50:>9.1f} {fmt_delta(r50, p50):>8} "
              f"{r99:>9.1f} {p99:>9.1f} {fmt_delta(r99, p99):>8} {sup or 0:>4}  "
              f"p50 {v50:<4} p99 {v99:<4} {acc}")

    n = len(rows)
    print()
    print(f"cells {n}: p50 wins {p50_wins} losses {p50_losses} · p99 wins {p99_wins} "
          f"losses {p99_losses} · ties(±{args.tol:.0%}) {ties}")
    if cell_ok and p50_losses == 0 and p99_losses == 0:
        print("EVERY-CELL BAR: MET on this paired sample (no cell behind, no tie band losses)")
    else:
        losers = [f"{s}/{c}" for s, c, r, p in rows
                  if (r[0] - p[0]) / p[0] > 0 or (r[1] - p[1]) / p[1] > 0]
        print(f"EVERY-CELL BAR: NOT met — behind at: {', '.join(losers)}")
    print("(paired per-suite sample — quote WITH the preflight line; a single")
    print(" sample is one sample: rerun before acting on any within-tol cell)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
