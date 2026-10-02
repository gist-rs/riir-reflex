#!/usr/bin/env python3
"""Self-check for .scratch/famwide/harness_visibility.tsv.

Validates every hard constraint from the authoring brief:
  1. exactly 96 lines, 24 per class, grouped gold 0 -> 1 -> 2 -> 3
  2. format gold<TAB>text, gold in 0..3, no second tab
  3. 12-30 words, 1-3 sentences, declarative, capitalized first letter, ends with '.'
  4. no banned token (hide/short/long/full) anywhere — whole-text substring check,
     word-level check, and hyphen-part check (strictest wins)
  5. trigram wall — no 3-word window (case-insensitive, punctuation stripped,
     apostrophes removed) appears in any corpus or cal text of
     src/harness/families.rs; checked under BOTH hyphen interpretations
     (hyphen->space and hyphen->removed) so neither gate spelling can slip
  6. fresh vocabulary — each text carries >= 3 words absent from corpus+cal
  7. pairwise near-duplicate screen (Jaccard on word sets)
"""
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SRC = REPO / "src" / "harness" / "families.rs"
TSV = Path(__file__).resolve().parent / "harness_visibility.tsv"
BANNED = ["hide", "short", "long", "full"]


def block(src: str, marker: str) -> str:
    i = src.index(marker)
    j = src.index("],", i)
    return src[i:j]


def variants(text: str):
    t = text.lower().replace("'", "")
    a = re.sub(r"[^a-z0-9]+", " ", t).split()          # hyphen -> space
    b = re.sub(r"[-]", "", t)
    b = re.sub(r"[^a-z0-9]+", " ", b).split()          # hyphen -> removed
    return a, b


def trigrams(words):
    return {" ".join(words[i : i + 3]) for i in range(len(words) - 2)}


src = SRC.read_text(encoding="utf-8")
corpus_block = block(src, "corpus: &[")
cal_block = block(src, "cal: &[")

corpus = [(int(g), t) for g, t in re.findall(r'(\d+),\s*\n\s*"([^"]*)"', corpus_block)]
cal = [(int(g), t) for t, g in re.findall(r'text:\s*"([^"]*)",\s*\n\s*gold:\s*(\d+)', cal_block)]

assert len(corpus) == 12, f"corpus parse got {len(corpus)}"
assert len(cal) == 20, f"cal parse got {len(cal)}"

wall = set()
ref_words = set()
for _, t in corpus + cal:
    for words in variants(t):
        wall |= trigrams(words)
        ref_words |= set(words)

lines = TSV.read_text(encoding="utf-8").splitlines()
errors = []
warns = []

if len(lines) != 96:
    errors.append(f"line count {len(lines)} != 96")

golds = []
texts = []
for n, line in enumerate(lines, 1):
    if "\t" not in line:
        errors.append(f"L{n}: no tab")
        continue
    parts = line.split("\t")
    if len(parts) != 2:
        errors.append(f"L{n}: {len(parts) - 1} tabs, expected exactly 1")
        continue
    gold, text = parts
    if gold not in "0123":
        errors.append(f"L{n}: bad gold {gold!r}")
        continue
    golds.append(int(gold))
    texts.append(text)

    if text != text.strip() or text != text.rstrip():
        errors.append(f"L{n}: leading/trailing whitespace")
    if not text.endswith("."):
        errors.append(f"L{n}: does not end with period")
    if not text[0:1].isupper():
        errors.append(f"L{n}: not capitalized")
    if "!" in text or "?" in text:
        errors.append(f"L{n}: not declarative")
    sents = text.count(".")
    if not (1 <= sents <= 3):
        errors.append(f"L{n}: {sents} sentence-ending periods")

    toks = text.split()
    if not (12 <= len(toks) <= 30):
        errors.append(f"L{n}: {len(toks)} words outside 12-30")

    low = text.lower()
    for b in BANNED:
        if b in low:
            errors.append(f"L{n}: banned substring {b!r} in text")
    for w in toks:
        wl = re.sub(r"[^a-z0-9-]", "", w.lower())
        parts_w = [wl] + wl.split("-")
        for p in parts_w:
            for b in BANNED:
                if b in p:
                    errors.append(f"L{n}: word {w!r} contains banned {b!r}")

    mygrams = set()
    for words in variants(text):
        mygrams |= trigrams(words)
        hit = mygrams & wall
        if hit:
            errors.append(f"L{n}: trigram wall hit {sorted(hit)[:4]}")
        mygrams.clear()
        mygrams |= trigrams(words)
    # (recompute cleanly to avoid partial-set confusion)
    mygrams = set()
    for words in variants(text):
        mygrams |= trigrams(words)
    hit = mygrams & wall
    if hit:
        errors.append(f"L{n}: trigram wall hit {sorted(hit)[:6]}")

    fresh = [w for w in variants(text)[0] if w not in ref_words]
    if len(fresh) < 3:
        errors.append(f"L{n}: only {len(fresh)} fresh words ({fresh})")

if golds != sorted(golds) or golds != [g for g in range(4) for _ in range(24)]:
    bad = True
    errors.append("classes not 24x each grouped ascending 0,1,2,3")

# per-class counts
from collections import Counter

counts = Counter(golds)
for g in range(4):
    print(f"class {g}: {counts.get(g, 0)} lines")

# near-duplicate screen
def jac(a, b):
    sa, sb = set(variants(a)[0]), set(variants(b)[0])
    return len(sa & sb) / len(sa | sb)

worst = []
for i in range(len(texts)):
    for j in range(i + 1, len(texts)):
        v = jac(texts[i], texts[j])
        if v > 0.55:
            worst.append((round(v, 2), i + 1, j + 1))
for v, i, j in sorted(worst, reverse=True)[:10]:
    warns.append(f"near-dup {v}: L{i} vs L{j}")

if errors:
    print(f"\nFAIL — {len(errors)} error(s):")
    for e in errors:
        print("  " + e)
    sys.exit(1)
print("\nbanned-word check: PASS (whole-text + word + hyphen-part, hide/short/long/full)")
print(f"trigram wall check: PASS (corpus {len(corpus)} + cal {len(cal)} texts, "
      f"{len(wall)} blocked trigrams, both hyphen interpretations)")
print(f"fresh-vocabulary check: PASS (>= 3 novel words per text)")
for w in warns:
    print("WARN " + w)
if not warns:
    print("near-duplicate screen: clean (no pair above 0.55 Jaccard)")
print("ALL CHECKS PASSED")
