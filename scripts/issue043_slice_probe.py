#!/usr/bin/env python3
"""Issue 043 premise probe (throwaway, measurement-only).

Datum to reproduce: pure-NB prompt_injections reads 0.8362 on test (probe,
041) vs the shipped blend 0.7672; the shipped cal slice (first 100 train
rows, 85/15 benign/injection — NOT label-grouped) reads ~0.50 for BOTH
noul polarity candidates in the real engine, so selection cannot reach the
polarity posture.

This probe: (1) calibrate a plain-MNB mirror of the polarity scorer against
the 0.8362 datum; (2) read candidate cal slices (bigger prefixes,
label-stratified round-robins) and ask whether the mirror's polarity
postures SEPARATE on cal alone — with the selection rule cal-only, test
numbers printed for analysis only.
"""

from __future__ import annotations

import glob
import json
import math
import os
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import issue038_t7_probe as base  # noqa: E402
# Keep this instrument's verdict printable on a non-UTF-8 console
# (katgpt-rs Issue 804 / the 928 drift census): it prints non-ASCII glyphs,
# and print() raises UnicodeEncodeError on e.g. cp874 — the process then dies
# with NO verdict. backslashreplace degrades the glyph visibly and keeps
# ASCII exact, so a verdict line stays greppable. Best-effort: a detached or
# captured stream is left alone rather than made fatal at import.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

DATA = base.DATA
V = base.V


def load_pi(split):
    rows = []
    for f in sorted(glob.glob(os.path.join(DATA, "prompt_injections",
                                           f"{split}-*.json"))):
        d = json.load(open(f, encoding="utf-8"))
        rows += [r["row"] for r in d["rows"]]
    return [(str(int(r["label"])), r["text"]) for r in rows
            if r.get("text") is not None and r.get("label") is not None]


def corpus_from(pool, cap_per_label):
    """Per-label first-cap in file order (the builder's cap rule)."""
    cnt = Counter()
    out = []
    for lab, txt in pool:
        if cnt[lab] >= cap_per_label:
            continue
        cnt[lab] += 1
        out.append((lab, txt))
    return out


class Mnb:
    def __init__(self, docs, alpha=1.0):
        self.per = defaultdict(Counter)
        self.tot = Counter()
        self.n = len(docs)
        self.alpha = alpha
        for lab, txt in docs:
            tk = base.tokens(txt)
            self.per[lab].update(tk)
            self.tot[lab] += len(tk)

    def score(self, txt, lab):
        tk = base.tokens(txt)
        denom = self.tot[lab] + self.alpha * V
        s = math.log((self.tot[lab] + self.alpha) / (self.n + self.alpha))
        cc = self.per.get(lab)
        for w in tk:
            s += math.log(((cc[w] if cc else 0) + self.alpha) / denom)
        return s

    def pred(self, txt, labels):
        return max(labels, key=lambda l2: self.score(txt, l2))

    def acc(self, rows, labels):
        return sum(self.pred(t, labels) == l for l, t in rows) / len(rows)


def round_robin(rows, labels, budget):
    by = defaultdict(list)
    for lab, txt in rows:
        by[lab].append((lab, txt))
    picks = []
    cursors = {l2: 0 for l2 in labels}
    while len(picks) < budget:
        took = False
        for l2 in labels:
            if cursors[l2] < len(by[l2]):
                picks.append(by[l2][cursors[l2]])
                cursors[l2] += 1
                took = True
                if len(picks) >= budget:
                    break
        if not took:
            break
    return picks


def main():
    train = load_pi("train")
    test = load_pi("test")
    labels = ["0", "1"]
    print(f"train={len(train)} test={len(test)} "
          f"train mix={Counter(l for l, _ in train)} test mix={Counter(l for l, _ in test)}")

    # (1) calibrate the scorer against the 0.8362 datum
    for cap in (64, 10**9):
        m = Mnb(corpus_from(train, cap))
        print(f"[calib] corpus cap/label={cap if cap < 10**9 else 'all'} "
              f"n={m.n}  test={m.acc(test, labels):.4f}")

    # (2) candidate cal slices; corpus = pool minus cal rows, cap 64/label
    #     (the registry posture); postures: off / nb1(argmax) / nb0(inverse)
    slices = {}
    for n in (100, 200, 300, 400):
        slices[f"prefix{n}"] = train[:n]
        pool = train[n:]
        slices[f"prefix{n}+pool"] = (train[:n], pool)
    for n in (100, 200, 300):
        front = round_robin(train, labels, n)
        front_keys = {t for _, t in front}
        pool = [(l, t) for l, t in train if t not in front_keys]
        slices[f"strat{n}"] = (front, pool)

    for name, val in slices.items():
        if isinstance(val, tuple):
            cal, pool = val
        else:
            cal, pool = val, train[len(val):]
        m = Mnb(corpus_from(pool, 64))
        a_off = max(set(l for l, _ in corpus_from(pool, 64)),
                    key=[l for l, _ in corpus_from(pool, 64)].count)
        acc = {p: sum((m.pred(t, labels) == l) if p == "nb1"
                      else (m.pred(t, labels) != l) if p == "nb0"
                      else (a_off == l)
                      for l, t in cal) / len(cal)
               for p in ("off", "nb1", "nb0")}
        sel = max(acc, key=acc.get)
        # test numbers ANALYSIS ONLY — the selection rule is cal-only
        test_acc = {p: sum((m.pred(t, labels) == l) if p == "nb1"
                           else (m.pred(t, labels) != l) if p == "nb0"
                           else (a_off == l)
                           for l, t in test) / len(test)
                    for p in ("off", "nb1", "nb0")}
        print(f"== {name}: cal={len(cal)} pool={len(pool)} "
              f"cal_mix={Counter(l for l, _ in cal)}")
        print(f"   cal  {acc}  -> SELECTED {sel}")
        print(f"   test {test_acc}  (analysis only)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
