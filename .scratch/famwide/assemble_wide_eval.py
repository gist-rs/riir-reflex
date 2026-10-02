#!/usr/bin/env python3
"""Assemble src/harness/families_eval_wide.rs from the .scratch TSVs.

One static [FamilyText; N] per widened family (VIS/PERM/TOOL/ROUTE/SENS —
cache_reuse keeps its frozen 12-fixture record per Plan 009 REVISED-2).
Re-run only when re-authoring; the committed .rs is the source of truth
afterward (the gates pin its BLAKE3 digest).
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRATCH = ROOT / ".scratch" / "famwide"
OUT = ROOT / "src" / "harness" / "families_eval_wide.rs"

FAMILIES = [
    ("harness_visibility", "VIS_WIDE_EVAL"),
    ("harness_permissions", "PERM_WIDE_EVAL"),
    ("harness_tool_fit", "TOOL_WIDE_EVAL"),
    ("harness_routing", "ROUTE_WIDE_EVAL"),
    ("harness_sensitivity", "SENS_WIDE_EVAL"),
]


def esc(text: str) -> str:
    return text.replace("\\", "\\\\").replace('"', '\\"')


def rust_str_lit(text: str) -> str:
    # Wrap long texts across lines like the rest of the file does.
    return f'"{esc(text)}"'


def main() -> None:
    blocks = []
    for tsv_name, static_name in FAMILIES:
        rows = []
        for line in (SCRATCH / f"{tsv_name}.tsv").read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            gold, text = line.split("\t", 1)
            rows.append((int(gold), text))
        n = len(rows)
        entries = ",\n".join(
            f"    FamilyText {{\n        text: {rust_str_lit(text)},\n        gold: {gold},\n    }}"
            for gold, text in rows
        )
        blocks.append(
            f"/// `{tsv_name}` — the wide eval population ({n} cases, "
            f"class-balanced), authored under Plan 009 REVISED-2.\n"
            f"pub static {static_name}: [FamilyText; {n}] = [\n{entries},\n];\n"
        )

    header = """//! The five widened family eval populations (Plan 009 REVISED-2 —
//! `.issues/059`): ~96 authored cases per family, class-balanced, replacing
//! each family's 12–16-case eval slice. `harness_cache_reuse` deliberately
//! keeps its frozen T3 12-fixture record (the documented divergence — see
//! `families.rs`'s `CACHE_REUSE_NOTE`).
//!
//! Authoring law (the gates in `tests/harness_families_gates.rs` enforce
//! every clause): no label tokens (per-family ban lists — TOOL bans the six
//! tool names, SENS bans every digit), no shared word 3-gram with the
//! family's corpus or cal texts (the memorization-path wall), per-case
//! unigram overlap ceilings, exact class balance, and a BLAKE3 digest pin
//! per family so any fixture edit reds the gate until consciously re-pinned.
//!
//! Authoring provenance: the `.scratch/famwide/*.tsv` sources + their
//! checker scripts (committed); assembled by
//! `.scratch/famwide/assemble_wide_eval.py`. The corpus and cal slices are
//! UNCHANGED — single-variable measurement.

// noqa-style note: this file is generated data; keep it data-only.

use super::families::FamilyText;

"""
    OUT.write_text(header + "\n".join(blocks), encoding="utf-8")
    print(f"wrote {OUT} ({sum(1 for _ in OUT.read_text().splitlines())} lines)")


if __name__ == "__main__":
    main()
