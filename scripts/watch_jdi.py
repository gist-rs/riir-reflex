#!/usr/bin/env python3
"""watch_jdi.py -- standing re-check of the Jev Decision Index board (reflex
Issue 082 T5; the perf-rematch watch_repo pattern applied to a Space data
file instead of a repo).

The lesson that filed this: pplx-decider v1.1 published 2026-10-05 and took
#1 on the 2026-10-07 board -- and nothing on our side noticed for two days,
because our pinned reference (.research/005 / the site board_reference) was
a snapshot with no standing re-check. This script is that re-check: fetch
the space's data/index.json, sha256 it, compare against the pin. A board
move prints old vs new + the new #1 and our tracked entrants, and exits 1 --
file the issue, don't sit on it.

Usage:
  python3 scripts/watch_jdi.py            # check: 0 unchanged, 1 CHANGED, 2 fetch-failure
  python3 scripts/watch_jdi.py --update   # re-pin after an intentional refresh (T4-style)

Exit codes:
  0  unchanged (pin sha == fetched sha)
  1  CHANGED (prints old/new sha, generated_utc, edition, the new #1, and
     our tracked entrants' rows; a change is a FILE-AN-IVENT signal, never
     an auto-repin -- the --update flag is the only writer)
  2  fetch/parse failure (offline boxes skip loud, never a CI hazard: a
     watch that hard-fails offline gets unplugged, and an unplugged watch
     is the two-day-late #1 again)

The pin file scripts/jdi_board.sha is FOUR lines, in order:
  line 1: sha256 hex of the index.json bytes
  line 2: generated_utc: <the board's own generation stamp>
  line 3: edition: <panel_id / edition label>
  line 4: fetched: <the date we pinned it>

Stdlib only, ASCII-only output (no console-safe glyphs needed).
"""

import hashlib
import json
import re
import sys
import urllib.request

URL = ("https://huggingface.co/spaces/multimodalart/"
       "jev-decision-index/resolve/main/data/index.json")
PIN_PATH = "scripts/jdi_board.sha"
# tracked entrants (substring match, case-insensitive) -- the rows whose
# movement matters to our lanes; extend freely, the print stays bounded
TRACKED = [
    "Perplexity Decider",
    "JEV-27B",
    "clef",
    "Torchcast",
    "laya",
    "GLiNER",
    "CLM",
]


def fetch(url, timeout=30):
    req = urllib.request.Request(url, headers={"User-Agent": "reflex-watch-jdi/1"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read()


def read_pin(path):
    try:
        with open(path, "r", encoding="utf-8") as f:
            lines = [ln.rstrip("\n") for ln in f.readlines()]
    except FileNotFoundError:
        return None
    if len(lines) < 4 or not re.fullmatch(r"[0-9a-f]{64}", lines[0].strip()):
        return None
    pin = {"sha": lines[0].strip(), "lines": lines[:4]}
    for ln in lines[1:4]:
        m = re.match(r"(\w+):\s*(.+)", ln)
        if m:
            pin[m.group(1)] = m.group(2).strip()
    return pin


def write_pin(path, sha, meta_lines):
    with open(path, "w", encoding="utf-8") as f:
        f.write(sha + "\n")
        for ln in meta_lines:
            f.write(ln + "\n")


def summarize(blob):
    """generated_utc, panel edition, the #1 blended, and tracked rows -- or
    None when the JSON is not the shape we pinned (that IS a change worth
    exit 1; the caller prints the sha move and a parse note)."""
    try:
        d = json.loads(blob)
        gen = d.get("generated_utc", "?")
        suite = d.get("suite", {})
        edition = suite.get("panel_id") or d.get("edition") or "?"
        top = tracks = ""
        models = d.get("models", [])
        by_rank = sorted(
            models,
            key=lambda m: m.get("v03", {}).get("rank_v", 10**9),
        )
        if by_rank:
            m0 = by_rank[0]
            top = "{} (balanced_skill {}, rank_v {})".format(
                m0.get("name", "?"),
                m0.get("scores", {}).get("balanced_skill", "?"),
                m0.get("v03", {}).get("rank_v", "?"),
            )
        rows = []
        for m in models:
            name = m.get("name", "")
            if any(t.lower() in name.lower() for t in TRACKED):
                rows.append("{}: skill {} (rank_v {}, public {})".format(
                    name,
                    m.get("scores", {}).get("balanced_skill", "?"),
                    m.get("v03", {}).get("rank_v", "?"),
                    m.get("scores", {}).get("public_skill", "?"),
                ))
        tracks = " | ".join(rows) if rows else "(no tracked entrants found)"
        return "generated_utc {} edition {}\n  new #1: {}\n  tracked: {}".format(
            gen, edition, top, tracks)
    except (ValueError, KeyError, TypeError):
        return None


def main():
    update = "--update" in sys.argv[1:]
    try:
        blob = fetch(URL)
    except Exception as e:  # noqa: BLE001 -- a watch reports, never crashes
        print("WATCH-JDI: fetch failed ({}) -- offline skip, board not checked".format(e))
        return 2
    sha = hashlib.sha256(blob).hexdigest()
    pin = read_pin(PIN_PATH)

    if update:
        meta = summarize(blob)
        lines = []
        try:
            d = json.loads(blob)
            lines.append("generated_utc: " + str(d.get("generated_utc", "?")))
            suite = d.get("suite", {})
            lines.append("edition: " + str(suite.get("panel_id") or d.get("edition") or "?"))
        except ValueError:
            print("WATCH-JDI: --update refused -- fetched bytes are not JSON")
            return 2
        import datetime
        lines.append("fetched: " + datetime.date.today().isoformat())
        write_pin(PIN_PATH, sha, lines)
        print("WATCH-JDI: pin updated -> {} ({} lines)".format(sha[:16] + "...", len(lines)))
        if meta:
            print(meta)
        return 0

    if pin is None:
        print("WATCH-JDI: no valid pin at {} -- run with --update to seed it".format(PIN_PATH))
        return 1
    if pin["sha"] == sha:
        print("WATCH-JDI: unchanged ({}...) {} / {}".format(
            sha[:12], pin.get("generated_utc", "?"), pin.get("edition", "?")))
        return 0

    print("WATCH-JDI: CHANGED")
    print("  old sha {} ... {} / {}".format(
        pin["sha"][:16], pin.get("generated_utc", "?"), pin.get("edition", "?")))
    print("  new sha {} ...".format(sha[:16]))
    meta = summarize(blob)
    if meta:
        print(meta)
    else:
        print("  (new bytes did not parse as the pinned shape -- inspect manually)")
    print("  -> file the lane/refresh issue; re-pin only via --update after the read")
    return 1


if __name__ == "__main__":
    sys.exit(main())
