#!/usr/bin/env python3
"""Train/test near-duplicate leak probe over the fetched dataset slices.

Issue 007 P2's decision trigger ("a near-duplicate leak measured in the
wild") and the known-answer oracle for Issue 024's Rust port: the in-harness
`slice_leak` report must reproduce these per-suite counts on the same
`.raw/datasets/` checkout.

Method (modelless, zero deps): normalise (lowercase, collapse whitespace,
strip non-alphanumerics), char 4-gram shingles, inverted index over TRAIN
skipping shingles held by >= RARE_CAP rows, top-5 candidates by shared
shingle count, Jaccard against each. A test row is EXACT when its normalised
text is in train, NEAR when best Jaccard >= NEAR_J. Only the first
TEST_CAP test rows are scanned for NEAR (the exact check scans all).

Caveat measured on the first run: `typed_decisions` rows are one fixed JSON
schema, so shingle Jaccard is dominated by the schema keys, and a NEAR hit
there is a TEMPLATE artifact, not a paraphrase. Read that row as
not-applicable, never as a leak rate.

    python3 scripts/slice_leak_probe.py            # all suites
    python3 scripts/slice_leak_probe.py banking77  # one suite
"""

import collections
import glob
import json
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".raw", "datasets")
K = 4
NEAR_J = 0.8
RARE_CAP = 200
TEST_CAP = 2000


def row_text(row):
    if "text" in row:
        return row["text"]
    if "state" in row:  # typed_decisions: the raw state is the routed text
        return json.dumps(row["state"], sort_keys=True)
    if "premise" in row:  # xnli: the pair is the unit
        return row["premise"] + " || " + row.get("hypothesis", "")
    return next((v for v in row.values() if isinstance(v, str)), "")


def row_label(row):
    for key in ("label_text", "label", "workflow"):
        if key in row:
            return row[key]
    return None


def load(suite, split):
    out = []
    for path in sorted(glob.glob(os.path.join(ROOT, suite, f"{split}-*.json"))):
        with open(path, encoding="utf-8") as f:
            for r in json.load(f).get("rows", []):
                out.append((row_text(r["row"]), row_label(r["row"])))
    return out


def norm(s):
    return re.sub(r"[^a-z0-9 ]", "", re.sub(r"\s+", " ", s.lower())).strip()


def shingles(s):
    s = norm(s)
    return {s[i : i + K] for i in range(max(1, len(s) - K + 1))}


def probe(suite):
    train, test = load(suite, "train"), load(suite, "test")
    if not train or not test:
        return f"{suite:20s} UNSEEN train={len(train)} test={len(test)}"
    train_norm = {}
    for t, lab in train:
        train_norm.setdefault(norm(t), lab)
    exact = [(t, lab) for t, lab in test if norm(t) in train_norm]
    conflict = sum(1 for t, lab in exact if train_norm[norm(t)] != lab)

    train_sh = [shingles(t) for t, _ in train]
    index = collections.defaultdict(list)
    for i, s in enumerate(train_sh):
        for g in s:
            index[g].append(i)

    near = near_same = scanned = 0
    for t, lab in test[:TEST_CAP]:
        if norm(t) in train_norm:
            continue
        scanned += 1
        s = shingles(t)
        counts = collections.Counter()
        for g in s:
            hits = index.get(g)
            if hits and len(hits) < RARE_CAP:
                counts.update(hits)
        best, best_i = 0.0, None
        for i, _ in counts.most_common(5):
            union = len(s | train_sh[i])
            j = len(s & train_sh[i]) / union if union else 0.0
            if j > best:
                best, best_i = j, i
        if best >= NEAR_J:
            near += 1
            near_same += train[best_i][1] == lab
    return (
        f"{suite:20s} train={len(train):6d} test={len(test):5d} "
        f"exact={len(exact):4d} ({100 * len(exact) / len(test):.2f}%) "
        f"exact_label_conflict={conflict} "
        f"near>={NEAR_J}={near}/{scanned} ({100 * near / max(1, scanned):.2f}%) "
        f"near_same_label={near_same}"
    )


def main():
    suites = sys.argv[1:] or sorted(
        d for d in os.listdir(ROOT) if not d.startswith("_") and os.path.isdir(os.path.join(ROOT, d))
    )
    for suite in suites:
        print(probe(suite))


if __name__ == "__main__":
    main()
