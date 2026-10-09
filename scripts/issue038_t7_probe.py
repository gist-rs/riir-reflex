#!/usr/bin/env python3
"""Issue 038 T7 premise probe (throwaway, measurement-only — never a product path).

Mirrors the NB POC recorded in `.issues/038`: a cheap Python check of whether a
lever's premise holds BEFORE any Rust build. Two premises, one run:

  T7(a) NBSVM/ridge readout — binary presence features scaled by NB log-count
        ratios, closed-form one-vs-rest ridge, features restricted to the
        top-k by document frequency. The standard sst5/emotion winner shape.
  T7(b) option-conditioned scorer for typed_decisions — per-(qid, option)
        multinomial counts over the state text, because the workflow-level
        tables never arm there (k != N on every question shape).

Protocol (same law as the harness): corpora from TRAIN rows only, selection on
a label-stratified train slice (its own docs excluded from the corpus), test
read once at the selected posture. Tokenization mirrors `src/embed.rs`
(hashed_tokens_into: whitespace split, non-alnum edge trim, no lowercasing,
unigram + bigram hashes) and `src/harness/suites.rs` (label/text field rules).

Output is a table per suite; published modelless numbers are echoed beside for
reference. Results land in the plan/issue record, never in code.
"""

from __future__ import annotations

import glob
import json
import math
import os
import sys
from collections import Counter, defaultdict

import numpy as np
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

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, ".raw", "datasets_t20k")
V = 1 << 17  # NB_VOCAB

PUBLISHED = {  # modelless (Bench-052 protocol) / laya best, echo only
    "ag_news": (0.8825, 0.950),
    "emotion": (0.7375, 0.593),
    "sst5": (0.3967, 0.372),
    "banking77": (0.8260, 0.498),
    "massive_intent_en": (0.7800, 0.750),
    "xnli_en": (0.5233, 0.860),
    "prompt_injections": (0.500, 0.698),
    "typed_decisions": (0.320, 0.745),
}

TRIM = set(b"_.-!\"'()[]{}<>,;:=+*/\\|~`@#$%^&?")


def fnv1a(b: bytes) -> int:
    h = 0xCBF29CE484222325
    for x in b:
        h ^= x
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def tokens(text: str) -> list[int]:
    out: list[int] = []
    prev: int | None = None
    for raw in text.encode("utf-8").split():
        t = raw.strip(bytes(TRIM))
        if not t:
            continue
        h = fnv1a(t) % V
        out.append(h)
        if prev is not None:
            pair = prev.to_bytes(8, "little") + h.to_bytes(8, "little")
            out.append(fnv1a(pair) % V)
        prev = h
    return out


def tokens_char(text: str, n: int = 5) -> list[int]:
    """Word tokens PLUS char n-grams over the raw bytes (alnum runs kept
    intact; punctuation positions are skipped). Salted apart from words."""
    out = tokens(text)
    b = text.encode("utf-8")
    for i in range(len(b) - n + 1):
        g = b[i:i + n]
        if all((48 <= c <= 57) or (65 <= c <= 90) or (97 <= c <= 122) for c in g):
            out.append((fnv1a(bytes([0xE1]) + g)) % V)
    return out


def load(suite: str, split: str) -> list[dict]:
    rows = []
    for f in sorted(glob.glob(os.path.join(DATA, suite, f"{split}-*.json"))):
        d = json.load(open(f, encoding="utf-8"))
        rows += [r["row"] for r in d["rows"]]
    return rows


def docs_of(suite: str, rows: list[dict]) -> list[tuple[str, str]]:
    out = []
    for r in rows:
        if suite in ("ag_news", "emotion", "sst5", "banking77", "prompt_injections"):
            try:
                lab = str(int(r["label"]))
            except Exception:
                continue
            txt = r.get("text")
        elif suite == "massive_intent_en":
            lab, txt = r.get("label_text"), r.get("text")
        elif suite == "xnli_en":
            try:
                lab = str(int(r["label"]))
            except Exception:
                continue
            p, h = r.get("premise"), r.get("hypothesis")
            txt = f"{p}\n{h}" if p is not None and h is not None else None
        else:
            continue
        if lab is None or txt is None:
            continue
        out.append((lab, txt))
    return out


class Counts:
    def __init__(self):
        self.per: dict[str, Counter] = defaultdict(Counter)
        self.tot: Counter = Counter()
        self.df: Counter = Counter()
        self.ndocs = 0

    def add(self, label: str, toks: list[int]) -> None:
        self.per[label].update(toks)
        self.tot.update(toks)
        for w in set(toks):
            self.df[w] += 1
        self.ndocs += 1


def mnb_eval(cnt: Counts, rows, labels, alpha=1.0) -> float:
    c = 0
    for lab, txt in rows:
        t = tokens(txt)
        best, best_s = 0, None
        for i, l in enumerate(labels):
            cc = cnt.per.get(l)
            tot = cnt.tot[l]
            denom = tot + alpha * V
            s = math.log((tot + alpha) / (cnt.ndocs + alpha))
            for w in t:
                s += math.log(((cc[w] if cc else 0) + alpha) / denom)
            if best_s is None or s > best_s:
                best_s, best = s, i
        if labels[best] == lab:
            c += 1
    return c / max(len(rows), 1)


# ── NBSVM-ridge (T7a) ─────────────────────────────────────────────────────


class Nbsvm:
    """Binary presence uni+bi features over the top-k df vocabulary, NB
    log-count-ratio scaling, closed-form one-vs-rest ridge (pure solve)."""

    def __init__(self, docs, labels, topk, scaling, alpha=1.0):
        cnt = Counts()
        toks_all = [tokens(txt) for _, txt in docs]
        for (lab, _), t in zip(docs, toks_all):
            cnt.add(lab, t)
        feats = sorted(cnt.df.items(), key=lambda kv: (-kv[1], kv[0]))[:topk]
        self.fidx = {w: i for i, (w, _) in enumerate(feats)}
        k = len(self.fidx)
        self.k = k
        self.labels = labels
        self.scaling = scaling
        n = len(docs)
        Xb = np.zeros((n, k + 1), dtype=np.float32)
        for i, t in enumerate(toks_all):
            for w in {w for w in t if w in self.fidx}:
                Xb[i, self.fidx[w]] = 1.0
        Xb[:, k] = 1.0
        y_by_lab = {lab: i for i, lab in enumerate(labels)}
        Y = np.zeros((n, len(labels)), dtype=np.float32)
        for i, (lab, _) in enumerate(docs):
            Y[i, y_by_lab[lab]] = 1.0
        tot = cnt.tot
        R = np.zeros((k, len(labels)), dtype=np.float32)
        for j, lab in enumerate(labels):
            nin = tot[lab] + alpha * V
            nout = (cnt.ndocs - tot[lab]) + alpha * V
            for w, i in self.fidx.items():
                cin = cnt.per[lab][w] + alpha
                cout = (tot[w] - cnt.per[lab][w]) + alpha
                R[i, j] = math.log(cin / nin) - math.log(cout / nout)
        if scaling == "global":
            for w, i in self.fidx.items():
                r = math.log((cnt.df[w] + alpha) / (cnt.ndocs - cnt.df[w] + alpha))
                R[i, :] = r
        self.R = R
        self.Xb = Xb
        self.Y = Y

    def solve(self, lam):
        k, n_lab = self.k, len(self.labels)
        W = np.zeros((k + 1, n_lab), dtype=np.float32)
        if self.scaling == "global":
            Xj = self.Xb * np.append(self.R[:, 0], 1.0).astype(np.float32)
            A = Xj.T @ Xj + lam * np.eye(k + 1, dtype=np.float32)
            W[:, :] = np.linalg.solve(A, Xj.T @ self.Y)
        else:
            for j in range(n_lab):
                Xj = self.Xb * np.append(self.R[:, j], 1.0).astype(np.float32)
                A = Xj.T @ Xj + lam * np.eye(k + 1, dtype=np.float32)
                W[:, j] = np.linalg.solve(A, Xj.T @ self.Y[:, j])
        self.W = W
        return self

    def score_row(self, toks):
        x0 = np.zeros(self.k + 1, dtype=np.float32)
        for w in set(toks):
            i = self.fidx.get(w)
            if i is not None:
                x0[i] = 1.0
        x0[-1] = 1.0
        s = np.zeros(len(self.labels), dtype=np.float32)
        if self.scaling == "global":
            s[:] = x0 @ (self.W * np.append(self.R[:, 0], np.float32(1.0))[:, None])
        else:
            for j in range(len(self.labels)):
                s[j] = float(x0 @ (self.W[:, j] * np.append(self.R[:, j], np.float32(1.0))))
        return s

    def acc(self, rows) -> float:
        c = 0
        for lab, txt in rows:
            if self.labels[int(self.score_row(tokens(txt)).argmax())] == lab:
                c += 1
        return c / max(len(rows), 1)


def stratified_front(docs, labels, per_label):
    by = defaultdict(list)
    for d in docs:
        by[d[0]].append(d)
    front = []
    for lab in labels:
        front += by[lab][:per_label]
    return front


def run_suite(suite, topks=(4096,), lams=(1.0, 10.0), front_per_label=8):
    train = docs_of(suite, load(suite, "train"))
    test = docs_of(suite, load(suite, "test"))
    labels = sorted({lab for lab, _ in train})
    front = stratified_front(train, labels, front_per_label)
    front_texts = {t for _, t in front}
    pool = [(l, t) for (l, t) in train if t not in front_texts]
    sel = front

    cnt = Counts()
    for lab, txt in pool:
        cnt.add(lab, tokens(txt))
    mnb_sel = mnb_eval(cnt, sel, labels)
    mnb_test = mnb_eval(cnt, test, labels)

    cands = {}
    models = {}
    for scaling in ("per_class", "global"):
        for topk in topks:
            m = Nbsvm(pool, labels, topk, scaling)
            models[(scaling, topk)] = m
            for lam in lams:
                m.solve(lam)
                cands[(scaling, topk, lam)] = m.acc(sel)
    scaling, topk, lam = max(cands, key=lambda c: cands[c])
    m = models[(scaling, topk)].solve(lam)
    acc_test = m.acc(test)

    pub, laya = PUBLISHED[suite]
    print(f"== {suite}: n_train={len(train)} n_test={len(test)} labels={len(labels)}")
    print(f"   MNB sel={mnb_sel:.4f} test={mnb_test:.4f}")
    for c in sorted(cands, key=cands.get, reverse=True)[:4]:
        print(f"   NBSVM sel {c[0]:9s} k={c[1]:5d} lam={c[2]:4.1f} -> sel={cands[c]:.4f}")
    print(f"   SELECTED scaling={scaling} k={topk} lam={lam}")
    print(f"   NBSVM test={acc_test:.4f}   [published modelless {pub} / laya {laya}]")
    return {"suite": suite, "mnb_test": mnb_test, "nbsvm_test": acc_test,
            "sel_posture": (scaling, topk, lam), "nbsvm_sel": cands[(scaling, topk, lam)],
            "mnb_sel": mnb_sel}


# ── typed per-(qid, option) scorer (T7b) ──────────────────────────────────


def typed_questions(r: dict):
    """Mirror suites.rs build_typed_decisions skip rules (per question)."""
    try:
        state = r["state"]
        qs = r["questions"]
        gd = r["gold"]
        qs = json.loads(qs) if isinstance(qs, str) else qs
        gd = json.loads(gd) if isinstance(gd, str) else gd
        if not isinstance(qs, dict) or not isinstance(gd, dict):
            return None, None
    except Exception:
        return None, None
    out = []
    for qid, qdef in qs.items():
        g = gd.get(qid)
        if g is None:
            continue
        typ = qdef.get("type")
        instr = qdef.get("instructions")
        if not isinstance(instr, str):
            continue
        if typ == "choice":
            crit = qdef.get("criteria")
            if crit is None:
                continue
            if isinstance(crit, str):
                try:
                    crit = json.loads(crit)
                except Exception:
                    continue
            if isinstance(crit, list):
                crit = {str(v): None for v in crit}
            if not isinstance(crit, dict):
                continue
            keys = list(crit.keys())
            lab = g.get("label")
            if not isinstance(lab, str) or lab not in keys:
                continue
            out.append((qid, "choice", instr, keys, keys.index(lab)))
        elif typ == "noul":
            lab = g.get("label")
            if not isinstance(lab, str):
                continue
            out.append((qid, "noul", instr, ["no", "yes"], int(lab.lower() == "true")))
        elif typ == "score":
            crit = qdef.get("criteria")
            if isinstance(crit, str):
                try:
                    crit = json.loads(crit)
                except Exception:
                    continue
            if not isinstance(crit, list):
                continue
            try:
                idx = int(g.get("label"))
            except Exception:
                continue
            levels = [str(v) for v in crit]
            if idx < 0 or idx >= len(levels):
                continue
            out.append((qid, "score", instr, levels, idx))
    return state, out


def run_typed(train_rows, test_rows, alpha=1.0, mode="mnb", tok=None):
    tok = tok or tokens
    """Per-(qid, gold-option) counts over the state text. Premise check only —
    a cal-selected ladder comes later if the premise holds.

    mode: mnb        — argmax log p(opt) + sum log p(t|opt)  (plain NB)
          noprior    — argmax sum log p(t|opt)
          contrastive— argmax sum [log p(t|opt) - max_other log p(t|other)]
          idf        — noprior with idf-weighted tokens
    """
    counts: dict[tuple[str, str], Counter] = defaultdict(Counter)
    tot: dict[tuple[str, str], int] = defaultdict(int)
    prior: dict[tuple[str, str], int] = defaultdict(int)
    df: Counter = Counter()
    ndocs = 0
    for r in train_rows:
        state, qs = typed_questions(r)
        if state is None:
            continue
        tk = set(tokens(state))
        ndocs += 1
        df.update(tk)
        for qid, kind, instr, keys, idx in qs:
            opt = keys[idx]
            counts[(qid, opt)].update(tk)
            tot[(qid, opt)] += len(tk)
            prior[(qid, opt)] += 1
    n_q = correct = 0
    per_kind = defaultdict(lambda: [0, 0])
    for r in test_rows:
        state, qs = typed_questions(r)
        if state is None:
            continue
        tk = tok(state)
        for qid, kind, instr, keys, idx in qs:
            opt = keys[idx]
            n_q += 1
            per_kind[kind][1] += 1
            best, best_s = 0, None
            scores = []
            for oi, opt in enumerate(keys):
                c = counts.get((qid, opt))
                if not c:
                    s = -1e9
                else:
                    denom = tot[(qid, opt)] + alpha * V
                    if mode == "mnb":
                        s = math.log(prior[(qid, opt)])
                    else:
                        s = 0.0
                    for w in tk:
                        lw = math.log((c[w] + alpha) / denom)
                        if mode == "idf":
                            lw *= math.log((ndocs + 1) / (df[w] + 1))
                        s += lw
                scores.append(s)
            if mode == "contrastive":
                arr = scores
                mx = max(arr)
                adj = [s - mx for s in arr]
                scores = [a - max(v for o, v in enumerate(adj) if o != oi) if False else a for a in arr]
                # margin over the best OTHER option
                scores = []
                for oi in range(len(arr)):
                    other = max(v for o, v in enumerate(arr) if o != oi)
                    scores.append(arr[oi] - other)
            best = max(range(len(scores)), key=lambda i: scores[i])
            if best == idx:
                correct += 1
                per_kind[kind][0] += 1
    acc = correct / max(n_q, 1)
    kd = {k: f"{v[0]}/{v[1]}={v[0] / max(v[1], 1):.3f}" for k, v in sorted(per_kind.items())}
    return acc, n_q, kd


def main() -> int:
    which = sys.argv[1] if len(sys.argv) > 1 else "all"
    if which in ("all", "typed"):
        tr = load("typed_decisions", "train")
        te = load("typed_decisions", "test")
        for mode in ("mnb", "idf"):
            for tokname, tok in (("word", tokens), ("word+char5", lambda s: tokens_char(s, 5))):
                acc, n, kd = run_typed(tr, te, mode=mode, tok=tok)
                pub, laya = PUBLISHED["typed_decisions"]
                print(f"== typed_decisions[{mode}/{tokname}]: n_train={len(tr)} n_test_rows={len(te)} n_q={n}")
                print(f"   per-(qid,option) acc={acc:.4f}  kinds {kd}")
                print(f"   [published modelless {pub} / laya {laya}]")
    if which in ("all", "datasets"):
        for suite in ("emotion", "sst5", "ag_news", "prompt_injections", "banking77",
                      "massive_intent_en", "xnli_en"):
            try:
                run_suite(suite)
            except Exception as e:  # keep going; the probe records the failure
                print(f"== {suite}: FAILED {type(e).__name__}: {e}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
