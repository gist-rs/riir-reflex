#!/usr/bin/env python3
"""docs_shape_gate.py — the .docs/ numbered-book shape gate.

Walks .docs/ and asserts the fleet's numbered-folder convention (the
katgpt-rs / riir-ai shape):

  1. a root README.md exists (the top-level index);
  2. every subdirectory name matches ^[0-9]{2}_[a-z0-9_]+$ (numbered
     folders only — no unnumbered or stray dirs);
  3. every numbered dir carries its own README.md (the folder index);
  4. every .md file in a numbered dir EXCEPT that dir's README.md is
     mentioned by filename in the README's text (the index is complete —
     an undocumented doc is invisible to the reader the index serves);
  5. floors: >= 4 numbered dirs and >= 8 .md files total. A ceiling is
     green over whatever the instrument can SEE, so the finding count
     needs the population that produced it — a walk regression must fail
     loud, never report a green zero.

Findings are named lines; exit 1. Green prints a counts line; exit 0.

--self-test: temp-dir fixtures asserting BOTH directions — a valid tree
passes; planted violations (a missing index entry, a non-numbered dir, a
missing folder README) each fail. The self-test runs on demand only (the
guard invokes the plain check; its own arms are exercised in dev).

Usage:
    python3 scripts/docs_shape_gate.py
    python3 scripts/docs_shape_gate.py --self-test
"""

import re
import shutil
import sys
import tempfile
from pathlib import Path

# Console-safe streams (the console_encoding discipline — the
# publish_bench.py pattern): this script's refusal messages carry non-ASCII
# glyphs and must die with a verdict, not a codec traceback, on a
# cp874-class console. Backslashreplace keeps every byte readable.
for _s in (sys.stdout, sys.stderr):
    _s.reconfigure(encoding="utf-8", errors="backslashreplace")

DOCS_DIR = Path(__file__).resolve().parent.parent / ".docs"
NUMBERED_RE = re.compile(r"^[0-9]{2}_[a-z0-9_]+$")
MIN_NUMBERED_DIRS = 4
MIN_MD_FILES = 8
OK = "✓"
FAIL = "✗"


def scan(docs_dir: Path):
    """Return (findings, counts). findings is a list of named strings;
    counts is the dict the green line prints."""
    findings = []
    numbered_dirs = []
    md_total = 0

    if not docs_dir.is_dir():
        findings.append(f"{FAIL} .docs/ is MISSING entirely — nothing to gate "
                        f"(walk regression)")
        return findings, {"dirs": 0, "md_files": 0, "indexed": 0}

    root_readme = docs_dir / "README.md"
    if not root_readme.is_file():
        findings.append(f"{FAIL} .docs/README.md missing — the root index "
                        f"is required")

    for entry in sorted(docs_dir.iterdir()):
        if not entry.is_dir():
            continue
        if not NUMBERED_RE.match(entry.name):
            findings.append(
                f"{FAIL} .docs/{entry.name}/ — directory name does not match "
                f"^[0-9]{{2}}_[a-z0-9_]+$ (numbered folders only)")
            continue
        numbered_dirs.append(entry)
        dir_readme = entry / "README.md"
        if not dir_readme.is_file():
            findings.append(f"{FAIL} .docs/{entry.name}/README.md missing — "
                            f"every numbered folder carries an index")
            continue
        readme_text = dir_readme.read_text(encoding="utf-8")
        for md in sorted(entry.glob("*.md")):
            md_total += 1
            if md.name == "README.md":
                continue
            if md.name not in readme_text:
                findings.append(
                    f"{FAIL} .docs/{entry.name}/{md.name} — not mentioned in "
                    f".docs/{entry.name}/README.md (add one index-table line)")
            else:
                # counted as indexed for the green line
                pass

    if root_readme.is_file():
        md_total += 1

    if len(numbered_dirs) < MIN_NUMBERED_DIRS:
        findings.append(
            f"{FAIL} only {len(numbered_dirs)} numbered dirs found, floor is "
            f"{MIN_NUMBERED_DIRS} — a walk regression must fail loud, never "
            f"report a green zero")
    if md_total < MIN_MD_FILES:
        findings.append(
            f"{FAIL} only {md_total} .md files found, floor is "
            f"{MIN_MD_FILES} — a walk regression must fail loud, never "
            f"report a green zero")

    counts = {
        "dirs": len(numbered_dirs),
        "md_files": md_total,
        "findings": len(findings),
    }
    return findings, counts


def main() -> int:
    findings, counts = scan(DOCS_DIR)
    if findings:
        for line in findings:
            print(line)
        print(f"{FAIL} docs_shape_gate FAILED — {counts['findings']} finding(s) "
              f"(numbered dirs: {counts['dirs']}, .md files: {counts['md_files']})")
        return 1
    print(f"{OK} docs_shape_gate PASSED — .docs/ book shape holds "
          f"(numbered dirs: {counts['dirs']}, .md files: {counts['md_files']}, "
          f"floors: {MIN_NUMBERED_DIRS} dirs / {MIN_MD_FILES} md)")
    return 0


def _write_tree(root: Path, files: dict):
    for rel, text in files.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")


VALID_TREE = {
    "README.md": "# index\n| [`01_a_/`](01_a_/) | x |\n| [`02_b_/`](02_b_/) | x |\n"
                 "| [`03_c_/`](03_c_/) | x |\n| [`04_d_/`](04_d_/) | x |\n",
    "01_a_/README.md": "# a\n| `doc1.md` | x |\n| `doc2.md` | x |\n",
    "01_a_/doc1.md": "x",
    "01_a_/doc2.md": "x",
    "02_b_/README.md": "# b\n| `doc3.md` | x |\n| `doc4.md` | x |\n",
    "02_b_/doc3.md": "x",
    "02_b_/doc4.md": "x",
    "03_c_/README.md": "# c\n| `doc5.md` | x |\n| `doc6.md` | x |\n",
    "03_c_/doc5.md": "x",
    "03_c_/doc6.md": "x",
    "04_d_/README.md": "# d\n| `doc7.md` | x |\n| `doc8.md` | x |\n",
    "04_d_/doc7.md": "x",
    "04_d_/doc8.md": "x",
}


def selftest() -> int:
    """Assert BOTH directions on temp-dir fixtures: the valid tree passes;
    each planted violation fails with the RIGHT finding."""
    failures = []

    # Arm 1: the valid tree passes.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        _write_tree(root, VALID_TREE)
        findings, counts = scan(root)
        if findings:
            failures.append(f"valid tree reported findings: {findings}")
        elif counts["dirs"] != 4 or counts["md_files"] != 13:
            failures.append(f"valid tree counts wrong: {counts}")

    # Arm 2: a missing index entry fails (the doc exists, the README omits it).
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        tree = dict(VALID_TREE)
        tree["02_b_/README.md"] = "# b\n| `doc3.md` | x |\n"  # doc4 dropped
        _write_tree(root, tree)
        findings, _ = scan(root)
        if not any("doc4.md" in f and "02_b_" in f for f in findings):
            failures.append(f"missing-index-entry arm did NOT fire on doc4: {findings}")

    # Arm 3: a non-numbered dir fails the name regex.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        tree = dict(VALID_TREE)
        tree["notes_extra/README.md"] = "# stray\n"
        _write_tree(root, tree)
        findings, _ = scan(root)
        if not any("notes_extra" in f for f in findings):
            failures.append(f"non-numbered-dir arm did NOT fire: {findings}")

    # Arm 4: a missing folder README fails.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        tree = dict(VALID_TREE)
        del tree["03_c_/README.md"]
        _write_tree(root, tree)
        findings, _ = scan(root)
        if not any("03_c_/README.md" in f for f in findings):
            failures.append(f"missing-folder-README arm did NOT fire: {findings}")

    # Arm 5: a walk regression (dirs deleted below the floor) fails loud.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        tree = {k: v for k, v in VALID_TREE.items()
                if not k.startswith(("03_c_", "04_d_"))}
        _write_tree(root, tree)
        findings, _ = scan(root)
        if not any("numbered dirs" in f for f in findings):
            failures.append(f"dir-floor arm did NOT fire: {findings}")
        if not any(".md files" in f for f in findings):
            failures.append(f"md-floor arm did NOT fire: {findings}")

    if failures:
        for f in failures:
            print(f"{FAIL} selftest: {f}")
        print(f"{FAIL} docs_shape_gate SELF-TEST FAILED — {len(failures)} arm(s)")
        return 1
    print(f"{OK} docs_shape_gate selftest PASSED — 5/5 arms "
          f"(valid passes; missing-index-entry, non-numbered-dir, "
          f"missing-folder-README, walk-regression each fail)")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(selftest())
    sys.exit(main())
