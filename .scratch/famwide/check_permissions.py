#!/usr/bin/env python3
"""Self-check for .scratch/famwide/harness_permissions.tsv (harness_permissions eval authoring).

Verifies the authored TSV against src/harness/families.rs PERM corpus+cal:
  - 96 lines, exactly 32 per class, grouped gold 0 -> 1 -> 2
  - `gold<TAB>text`, exactly one tab per line
  - 8..25 words, single sentence (one terminal period, none interior), capitalized, period-terminated
  - banned label tokens allow/ask/deny at SUBSTRING level (strictest reading)
  - trigram wall: no 3-word window (case-folded, punctuation handled two ways,
    apostrophes removed) appears in any corpus or cal text
  - unigram freshness: >=3 tokens absent from the corpus+cal vocabulary (warn <4)
  - informational: trigrams shared by >=3 of our own cases (near-duplicate signal)

Exit 0 only when zero failures.
"""
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC = (ROOT / "src/harness/families.rs").read_text(encoding="utf-8")
TSV = ROOT / ".scratch/famwide/harness_permissions.tsv"
BANNED = ["allow", "ask", "deny"]


def perm_region() -> str:
    start = SRC.index("static PERM")
    end = SRC.index("\n};", start)
    return SRC[start:end]


def array_texts(region: str, name: str) -> list[str]:
    i = region.index(f"{name}: &[")
    j = region.index("\n    ],", i)
    return re.findall(r'"([^"\n]*)"', region[i:j])


def toks_a(t: str) -> list[str]:
    # punctuation stripped WITHIN whitespace-separated words, apostrophes removed
    t = t.lower().replace("\u2019", "").replace("'", "")
    out = []
    for w in t.split():
        w = re.sub(r"[^a-z0-9]", "", w)
        if w:
            out.append(w)
    return out


def toks_b(t: str) -> list[str]:
    # punctuation RUNS act as separators
    t = t.lower().replace("\u2019", "").replace("'", "")
    return [w for w in re.split(r"[^a-z0-9]+", t) if w]


def trigrams(toks: list[str]) -> list[tuple[str, str, str]]:
    return list(zip(toks, toks[1:], toks[2:]))


region = perm_region()
corpus = array_texts(region, "corpus")
cal = array_texts(region, "cal")

wall_a: set = set()
wall_b: set = set()
vocab: set = set()
for t in corpus + cal:
    wall_a.update(trigrams(toks_a(t)))
    wall_b.update(trigrams(toks_b(t)))
    vocab.update(toks_a(t))
    vocab.update(toks_b(t))

lines = TSV.read_text(encoding="utf-8").rstrip("\n").split("\n")
errors: list[str] = []
warnings: list[str] = []
golds: list[str] = []
own: defaultdict = defaultdict(set)

for idx, line in enumerate(lines, 1):
    parts = line.split("\t")
    if len(parts) != 2:
        errors.append(f"L{idx}: expected exactly one tab, got {len(parts) - 1}")
        continue
    gold, text = parts
    if gold not in {"0", "1", "2"}:
        errors.append(f"L{idx}: bad gold {gold!r}")
    golds.append(gold)
    low = text.lower()
    for b in BANNED:
        if b in low:
            errors.append(f"L{idx}: banned token {b!r} present (substring)")
    words = text.split()
    if not (8 <= len(words) <= 25):
        errors.append(f"L{idx}: word count {len(words)} outside 8..25")
    if text and not text[0].isupper():
        errors.append(f"L{idx}: first letter not capitalized")
    if not text.endswith("."):
        errors.append(f"L{idx}: does not end with a period")
    if text.count(".") != 1:
        errors.append(f"L{idx}: {text.count('.')} periods (want exactly one terminal)")
    ta = toks_a(text)
    for gram in trigrams(ta):
        if gram in wall_a:
            errors.append(f"L{idx}: trigram-A collision {gram}")
    for gram in trigrams(toks_b(text)):
        if gram in wall_b:
            errors.append(f"L{idx}: trigram-B collision {gram}")
    fresh = [w for w in ta if w not in vocab]
    if len(fresh) < 3:
        errors.append(f"L{idx}: only {len(fresh)} fresh words")
    elif len(fresh) < 4:
        warnings.append(f"L{idx}: only {len(fresh)} fresh words: {text[:50]}")
    for gram in trigrams(ta):
        own[gram].add(idx)

expected = ["0"] * 32 + ["1"] * 32 + ["2"] * 32
if golds != expected:
    errors.append(f"grouping wrong: {dict(Counter(golds))} vs 32/32/32 grouped 0,1,2")

for gram, ls in sorted(own.items(), key=lambda kv: -len(kv[1])):
    if len(ls) >= 3:
        warnings.append(f"own trigram {gram} shared by {len(ls)} cases: {ls}")

print(f"corpus texts: {len(corpus)}   cal texts: {len(cal)}")
print(f"trigram wall: {len(wall_a)} (A) / {len(wall_b)} (B)   vocab: {len(vocab)}")
print(f"lines: {len(lines)}   per-class: 0={golds.count('0')} 1={golds.count('1')} 2={golds.count('2')}")
for w in warnings:
    print(f"WARN  {w}")
for e in errors:
    print(f"FAIL  {e}")
print("RESULT: PASS" if not errors else "RESULT: FAIL")
sys.exit(1 if errors else 0)
