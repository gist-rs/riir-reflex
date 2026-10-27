#!/usr/bin/env python3
"""rustfmt-clean, with a pinned drain list — Issue 099, ported from katgpt-rs
Issue 946 (the canonical gate) with ONE critical walker difference.

⛔ NEVER run `cargo fmt --all` in this repo: cargo-fmt's --all formats
"all packages, and also their local path-based dependencies", and reflex's
path deps pull in ../katgpt-rs — which carries INTENTIONAL pinned drift
(katgpt-rs/scripts/fmt_pending_expected.txt). A bare --all from here would
rewrite sibling files as uncommitted working-tree changes under concurrent
sessions (measured 2026-10-27 from a detached worktree: it formats the whole
dep closure when siblings resolve, and dies at cargo-metadata when they
don't). The canonical gate can use --all because katgpt-rs is the upstream
root; this consumer port walks THIS repo's own tracked .rs files with bare
`rustfmt --edition <E> --check`, one file per invocation.

Verdict, by MEMBERSHIP, both directions (a count is green on a swap):

- **UNPINNED** — a file rustfmt reports diverging that no row names: a NEW
  drift. Run `rustfmt --edition 2024 <file>` and commit it. Never pin new
  drift; the list only shrinks.
- **STALE** — a row whose file is now clean (or gone): delete the row in the
  commit that drained it.

Refusals (exit 2, NO VERDICT — never a pass): a rustfmt spawn exiting
outside {0, 1} (the founding phantom: a spawn under load exiting non-zero on
a clean file must be a loud named refusal, not a classification), a walk
below MIN_WALKED, a missing `edition` in the root Cargo.toml, or a
reasonless pin row.

`.benchmarks/**` probe artifacts are EXCLUDED from the walk by policy —
recorded bench bytes (the bench-guard principle: a bench file's bytes are
its artifact); the excluded count is disclosed on every verdict line.

Runs as layer 11 of scripts/ci_feature_guard.sh (the cheap always-run block;
zero cargo, ~seconds).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

# A non-UTF-8 console (cp874) would kill this script on its first
# non-ASCII glyph; degrade the glyph, never the verdict (katgpt-rs
# Issue 804). A captured stream without reconfigure is left alone.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

ROOT = Path(__file__).resolve().parent.parent
PINS = ROOT / "scripts" / "fmt_pending_expected.txt"
EXCLUDE_PREFIX = ".benchmarks/"
# Measured 2026-10-27 at the Issue-099 landing: 119 tracked .rs (122 minus the
# 3 .benchmarks probe artifacts). A walk below this floor is a walk regression
# (git broken, a mass deletion), never a clean tree.
MIN_WALKED = 100


def load_pins(path: Path) -> tuple[set[str], list[str]]:
    """Rows are `<relpath> # reason`. A reasonless row is refused."""
    pins: set[str] = set()
    errors: list[str] = []
    for n, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        rel, sep, reason = line.partition("#")
        if not sep or not reason.strip():
            errors.append(f"{path.name}:{n}: row has no reason: {line!r}")
            continue
        pins.add(rel.strip())
    return pins, errors


def edition(root: Path) -> str | None:
    """The root [package] edition — rustfmt's output shape depends on it."""
    manifest = root / "Cargo.toml"
    for line in manifest.read_text(encoding="utf-8").splitlines():
        m = re.match(r'^\s*edition\s*=\s*"([^"]+)"', line)
        if m:
            return m.group(1)
    return None


def walk(root: Path) -> tuple[list[str], int]:
    """(reflex's own tracked .rs, excluded-artifact count) — git is the truth."""
    out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "*.rs"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if out.returncode != 0:
        return [], -1
    files = [f for f in out.stdout.splitlines() if f.strip()]
    kept = [f for f in files if not f.startswith(EXCLUDE_PREFIX)]
    return kept, len(files) - len(kept)


def classify(root: Path, rel: str, edition_: str) -> tuple[int, str]:
    """rustfmt --check on ONE file: (0 clean | 1 diverging, stderr).

    Children are excluded via the CONFIG KEY (`--config skip_children=true`) —
    the `--skip-children` FLAG does not exist on stable rustfmt and exits 1
    with `Unrecognized option`, colliding with the diverging exit (measured
    2026-10-27: without the exclusion, rustfmt recurses into `mod` children
    by default, so a clean root file exits 1 on a drifted CHILD and the drift
    is attributed to the wrong file — lib.rs + both mod.rs roots flagged off
    their children's diffs).
    """
    proc = subprocess.run(
        ["rustfmt", "--edition", edition_, "--check", "--config", "skip_children=true", rel],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, proc.stderr


def scan(root: Path, files: list[str], edition_: str) -> tuple[set[str], str | None]:
    """(diverging files, first refusal reason) over the whole walk."""
    diverging: set[str] = set()
    for rel in files:
        rc, stderr = classify(root, rel, edition_)
        if rc == 1:
            if "Unrecognized option" in stderr or "error:" in stderr:
                # An exit-1 tooling failure masquerading as drift — refuse.
                first = stderr.strip().splitlines()[0] if stderr.strip() else "(no stderr)"
                return set(), f"rustfmt failed on {rel}: {first} — NO VERDICT, never a classification"
            diverging.add(rel)
        elif rc != 0:
            return set(), f"rustfmt exited {rc} on {rel} — NO VERDICT, never a classification"
    return diverging, None


def verdict(diverging: set[str], pins: set[str]) -> tuple[list[str], list[str]]:
    """(unpinned, stale) — the whole decision, extracted so the arms reach it."""
    return sorted(diverging - pins), sorted(pins - diverging)


def selftest() -> list[str]:
    fails: list[str] = []
    if verdict({"a.rs"}, {"a.rs"}) != ([], []):
        fails.append("exact match must be clean")
    if verdict({"a.rs", "n.rs"}, {"a.rs"}) != (["n.rs"], []):
        fails.append("a new drift must be UNPINNED")
    if verdict(set(), {"a.rs"}) != ([], ["a.rs"]):
        fails.append("a drained row must be STALE")
    if verdict({"x.rs"}, {"y.rs"}) != (["x.rs"], ["y.rs"]):
        fails.append("a swap must red both ways (membership, not count)")
    # load_pins refusals are exercised against a temp file, not a fixture path.
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False, encoding="utf-8") as tf:
        tf.write("ok.rs # a reason\nbad.rs\n")
        tmp = Path(tf.name)
    pins, errors = load_pins(tmp)
    tmp.unlink()
    if not errors or "no reason" not in errors[0]:
        fails.append(f"a reasonless row must be refused: {errors=}")
    if pins != {"ok.rs"}:
        fails.append(f"the good row must still pin while the bad row refuses: {pins=}")
    return fails


def main() -> int:
    fails = selftest()
    if fails:
        for f in fails:
            print(f"✗ fmt gate self-test: {f}")
        return 2

    if not PINS.exists():
        print(f"✗ fmt gate NO VERDICT — pins file missing: {PINS}")
        return 2
    pins, errors = load_pins(PINS)
    if errors:
        for e in errors:
            print(f"✗ {e}")
        return 1

    edition_ = edition(ROOT)
    if edition_ is None:
        print("✗ fmt gate NO VERDICT — no `edition = \"...\"` in the root Cargo.toml")
        return 2

    files, excluded = walk(ROOT)
    if excluded < 0 or len(files) < MIN_WALKED:
        print(
            f"✗ fmt gate BLIND — walk returned {len(files)} file(s) (excluded {excluded}) "
            f"< floor {MIN_WALKED}"
        )
        return 2

    diverging, refusal = scan(ROOT, files, edition_)
    if refusal is not None:
        print(f"✗ fmt gate {refusal}")
        return 2

    unpinned, stale = verdict(diverging, pins)
    for f in unpinned:
        print(f"  ✗ UNPINNED {f} — run `rustfmt --edition {edition_} {f}` and commit it")
    for f in stale:
        print(f"  ✗ STALE {f} — now rustfmt-clean (or gone); delete its row in {PINS.name}")
    if unpinned or stale:
        print(
            f"✗ fmt gate FAILED — {len(unpinned)} unpinned, {len(stale)} stale "
            f"over {len(files)} files (excluded {excluded} {EXCLUDE_PREFIX} artifacts)"
        )
        return 1
    print(
        f"✓ fmt gate PASSED — {len(files)} files checked, {len(pins)} pinned on the drain list "
        f"(excluded {excluded} {EXCLUDE_PREFIX} artifacts)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
