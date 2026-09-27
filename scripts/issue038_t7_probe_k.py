#!/usr/bin/env python3
"""Focused T7(a) k-sweep on emotion (the one suite the premise cleared)."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from issue038_t7_probe import load, docs_of, stratified_front, Nbsvm, Counts, mnb_eval

suite = "emotion"
train = docs_of(suite, load(suite, "train"))
test = docs_of(suite, load(suite, "test"))
labels = sorted({lab for lab, _ in train})
front = stratified_front(train, labels, 8)
front_texts = {t for _, t in front}
pool = [(l, t) for (l, t) in train if t not in front_texts]
for k in (1024, 2048, 4096):
    m = Nbsvm(pool, labels, k, "per_class").solve(10.0)
    print(f"k={k}: sel={m.acc(front):.4f} test={m.acc(test):.4f}")
