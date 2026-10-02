#!/usr/bin/env python3
"""Authoring gate for .scratch/famwide/harness_routing.tsv.

Verifies, per the task constraints:
  1. exactly 96 lines, exactly 24 per class, grouped gold 0 -> 1 -> 2 -> 3
  2. format `gold<TAB>text`, exactly one tab per line
  3. one sentence (no internal period), 10..26 words, capitalized, ends '.'
  4. banned label vocabulary + near-forms absent (case-insensitive substring)
  5. trigram wall: no 3-word window (case-insensitive, punctuation stripped,
     apostrophes removed) of any text appears in any corpus or cal text
  6. >= 4 fresh words (tokens absent from every corpus/cal text) per line
  8. no internal near-duplicates (Jaccard / shared 4-grams reported as warnings)
"""
import re
import sys
from pathlib import Path

CORPUS_CAL = [
    # -- corpus (families.rs 677-726) --
    "The local engine takes microsecond deterministic decisions at volume: fixed label sets, in-process, no network. Tick-rate routing and per-command verdicts live here.",
    "The fix lane scores spans in-process on the local engine — microsecond tier, bit-identical repeats.",
    "Thousands of classifications per hour with a fixed option universe belong on the local engine.",
    "The cheap API takes simple prose work where cost matters: summarizing short changelog entries, tagging sentiment.",
    "Structured extraction from support email at volume runs on the cheap API.",
    "Backlogs of brief reviews get labeled on the cheap API — good enough, and nearly free.",
    "The frontier API takes hard novel reasoning: the multi-region schema migration design, the cross-service heisenbug.",
    "The security review of a new authentication flow needs the frontier model's depth.",
    "When the bug reproduces only under load and the cause is unknown, the frontier API gets it.",
    "Deferred bulk work runs as a background batch: regenerate the docs site overnight, re-embed the corpus after a swap.",
    "Scheduled maintenance — compaction, archival, weekly digests — is background batch work.",
    "The monthly usage report assembles itself as a background batch job.",
    # -- cal (families.rs 727-792) --
    "Classify this ticket into one of eight queues; ten thousand arrive per hour.",
    "Decide, in microseconds, whether this command may run.",
    "Score these candidate spans for the fix lane, in-process, deterministically.",
    "Route each incoming request's intent against the fixed option set, per tick.",
    "Summarize the day's release notes; the prose is simple and the budget is tight.",
    "Extract the invoice fields from a thousand emails this week.",
    "Label the support threads by topic, at minimal cost per item.",
    "Rewrite these changelog blurbs in plainer language.",
    "Design the concurrency model for the new executor from scratch.",
    "Root-cause the parity gate that fails only under load.",
    "Write the threat model for the new credential flow.",
    "Work out which algebraic identity the optimizer might be exploiting.",
    "Rebuild the search index tonight, after the traffic window.",
    "Generate the evaluation tables for every checkpoint overnight.",
    "Archive the old runs and compact the stores on a schedule.",
    "Assemble the monthly usage report from the ledgers.",
]

BANNED = ["local", "engine", "cheap", "api", "frontier", "batch", "background"]


def toks(text: str) -> list[str]:
    t = text.replace("'", "").lower()
    return [w for w in re.split(r"[^a-z0-9]+", t) if w]


def trigrams(tokens: list[str]) -> set[tuple[str, ...]]:
    return {tuple(tokens[i : i + 3]) for i in range(len(tokens) - 2)}


def fourgrams(tokens: list[str]) -> set[tuple[str, ...]]:
    return {tuple(tokens[i : i + 4]) for i in range(len(tokens) - 3)}


wall: set[tuple[str, str, str]] = set()
vocab: set[str] = set()
for t in CORPUS_CAL:
    tk = toks(t)
    wall |= trigrams(tk)
    vocab |= set(tk)

path = Path(sys.argv[1])
raw = path.read_text(encoding="utf-8")
lines = raw.splitlines()
errors: list[str] = []
warnings: list[str] = []

if raw and not raw.endswith("\n"):
    errors.append("file does not end with a newline")
if len(lines) != 96:
    errors.append(f"line count {len(lines)} != 96")

per_class: dict[int, list[str]] = {0: [], 1: [], 2: [], 3: []}
parsed: list[tuple[int, int, str]] = []
for i, line in enumerate(lines, 1):
    if line.count("\t") != 1:
        errors.append(f"L{i}: tab count {line.count(chr(9))} != 1")
        continue
    g, _, text = line.partition("\t")
    if len(g) != 1 or g not in "0123":
        errors.append(f"L{i}: bad gold field {g!r}")
        continue
    g = int(g)
    per_class[g].append(text)
    parsed.append((i, g, text))

    words = text.split()
    if not 10 <= len(words) <= 26:
        errors.append(f"L{i}: word count {len(words)} outside 10..26")
    if not text[:1].isupper():
        errors.append(f"L{i}: first char not uppercase")
    if not text.endswith("."):
        errors.append(f"L{i}: does not end with a period")
    if "." in text[:-1]:
        errors.append(f"L{i}: internal period (more than one sentence?)")
    low = text.lower()
    for b in BANNED:
        if b in low:
            errors.append(f"L{i}: banned word {b!r} present")
    tk = toks(text)
    hits = trigrams(tk) & wall
    if hits:
        errors.append(
            f"L{i}: TRIGRAM COLLISIONS: " + "; ".join(" ".join(h) for h in sorted(hits))
        )
    fresh = [w for w in tk if w not in vocab]
    if len(fresh) < 4:
        errors.append(f"L{i}: only {len(fresh)} fresh words (<4): {fresh}")

for g in range(4):
    if len(per_class[g]) != 24:
        errors.append(f"class {g}: {len(per_class[g])} lines != 24")
order = [g for _, g, _ in parsed]
if order != [0] * 24 + [1] * 24 + [2] * 24 + [3] * 24:
    errors.append("classes not grouped ascending 0,1,2,3 with 24 lines each")

# internal near-duplicate probe: token Jaccard >= 0.5 or any shared 4-gram
toksets: dict[int, set[str]] = {}
fours: dict[int, set[tuple[str, ...]]] = {}
for i, _g, text in parsed:
    tk = toks(text)
    toksets[i] = set(tk)
    fours[i] = fourgrams(tk)
items = sorted(toksets)
for a in range(len(items)):
    for b in range(a + 1, len(items)):
        la, lb = items[a], items[b]
        j = len(toksets[la] & toksets[lb]) / len(toksets[la] | toksets[lb])
        shared = fours[la] & fours[lb]
        if j >= 0.5 or shared:
            warnings.append(
                f"L{la}/L{lb}: jaccard {j:.2f} shared4={sorted(' '.join(x) for x in shared)}"
            )

print(
    f"lines={len(lines)} per_class="
    + ",".join(f"{g}:{len(per_class[g])}" for g in range(4))
)
for w in warnings:
    print("WARN", w)
if errors:
    print(f"FAIL ({len(errors)} errors)")
    for e in errors:
        print("  ", e)
    sys.exit(1)
print(
    "PASS: 96 lines, 24x4 grouped, format/one-sentence/length/caps/period ok, "
    "banned words absent, trigram wall clean, >=4 fresh words per line"
)
