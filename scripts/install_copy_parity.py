#!/usr/bin/env python3
"""Install-copy parity — the site MUST name the same install commands as the
dist repo README (the `install_copy_parity` precedent, Plan 606 T3.4).

Usage:
    python3 install_copy_parity.py <dist-repo-README.md> <site-index.html>

Fails (exit 1) when a canonical command present in one surface is missing
from the other. Canonical commands (exact substrings):
  - curl -fsSL https://raw.githubusercontent.com/gist-rs/reflex/main/install.sh | sh
  - iwr -useb https://raw.githubusercontent.com/gist-rs/reflex/main/install.ps1 | iex
  - brew tap gist-rs/tap && brew install riir-reflex
  - scoop bucket add gist-rs https://github.com/gist-rs/scoop-bucket && scoop install riir-reflex
"""

import sys
from pathlib import Path

CANONICAL = (
    "https://raw.githubusercontent.com/gist-rs/reflex/main/install.sh",
    "https://raw.githubusercontent.com/gist-rs/reflex/main/install.ps1",
    "brew tap gist-rs/tap && brew install riir-reflex",
    "scoop bucket add gist-rs https://github.com/gist-rs/scoop-bucket && scoop install riir-reflex",
)


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    dist = Path(sys.argv[1]).read_text(encoding="utf-8")
    site = Path(sys.argv[2]).read_text(encoding="utf-8")
    fails = []
    for cmd in CANONICAL:
        in_dist, in_site = cmd in dist, cmd in site
        if in_dist != in_site:
            fails.append(f"  MISMATCH {cmd!r}: dist={in_dist} site={in_site}")
        elif not in_dist:
            fails.append(f"  MISSING EVERYWHERE {cmd!r}")
    if fails:
        print("INSTALL-COPY-PARITY: FAIL")
        print("\n".join(fails))
        return 1
    print(f"INSTALL-COPY-PARITY: PASS ({len(CANONICAL)} canonical commands agree on both surfaces)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
