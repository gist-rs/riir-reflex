#!/usr/bin/env python3
"""Differential for the Rust NbRidge: replicate the RUST fit math exactly
(presence gram + bias, λ on all diagonal entries incl. bias, per-class R
scaling, one-vs-rest solve) in numpy over the emotion pool, then compare
pure-argmax accuracy against the probe's 0.8925.

If this reads ~0.89 → the Rust implementation is buggy (find the site).
If it reads ~0.29 → the Rust MATH diverges from the probe's (diff the
transcription below against scripts/issue038_t7_probe.py).
"""

import glob, json, math, sys
import numpy as np

sys.path.insert(0, "/Users/katopz/git/riir-reflex/scripts")
from issue038_t7_probe import load, docs_of, stratified_front, tokens, Counts, V

suite = "emotion"
train = docs_of(suite, load(suite, "train"))
test = docs_of(suite, load(suite, "test"))
labels = sorted({lab for lab, _ in train})
# The seat's pool = train minus the STRATIFIED cal front (100 cases for
# emotion; approximate with the probe's 8/label front — close enough for a
# 0.89-vs-0.29 differential).
front = stratified_front(train, labels, 8)
front_texts = {t for _, t in front}
pool = [(l, t) for (l, t) in train if t not in front_texts]

cnt = Counts()
toks_all = [tokens(txt) for _, txt in pool]
for (lab, _), t in zip(pool, toks_all):
    cnt.add(lab, t)
feats = sorted(cnt.df.items(), key=lambda kv: (-kv[1], kv[0]))[:2048]
fidx = {w: i for i, (w, _) in enumerate(feats)}
k = len(fidx)
n = len(pool)
Xb = np.zeros((n, k + 1), dtype=np.float64)
for i, t in enumerate(toks_all):
    for w in {w for w in t if w in fidx}:
        Xb[i, fidx[w]] = 1.0
    Xb[i, k] = 1.0
y_by = {lab: j for j, lab in enumerate(labels)}
Y = np.zeros((n, len(labels)))
for i, (lab, _) in enumerate(pool):
    Y[i, y_by[lab]] = 1.0

alpha, AV = 1.0, float(V)
nin = AV                      # the probe's `tot[lab]` missing-key read
nout = n + AV                 # ndocs + αV
R = np.zeros((k, len(labels)))
for j, lab in enumerate(labels):
    for w, i in fidx.items():
        cin = cnt.per[lab][w] + alpha
        cout = (cnt.tot[w] - cnt.per[lab][w]) + alpha
        R[i, j] = math.log(cin / nin) - math.log(cout / nout)

lam = 10.0
accs = []
for j, lab in enumerate(labels):
    Xj = Xb * np.append(R[:, j], 1.0)
    A = Xj.T @ Xj + lam * np.eye(k + 1)
    w = np.linalg.solve(A, Xj.T @ Y[:, j])
    accs.append(w)
W = np.stack(accs, axis=1)

def predict(txt):
    t = set(tokens(txt))
    x0 = np.zeros(k + 1)
    for w in t:
        i = fidx.get(w)
        if i is not None:
            x0[i] = 1.0
    x0[-1] = 1.0
    s = [float(x0 @ (W[:, j] * np.append(R[:, j], 1.0))) for j in range(len(labels))]
    return labels[int(np.argmax(s))]

c = sum(predict(t) == lab for lab, t in test)
print(f"numpy replica of the RUST math: {c}/{len(test)} = {c/len(test):.4f}")
