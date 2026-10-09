#!/usr/bin/env python3
"""Issue 038 T7(c)+T7(d) premise probe (throwaway, measurement-only — never a
product path). Same discipline as `issue038_t7_probe.py`: a cheap Python check
of whether a lever's premise holds BEFORE any Rust build.

  T7(c) BM25-kNN vote — score the query against TRAIN rows with BM25 (Okapi:
        k1/b saturation), take top-k neighbours, vote weighted by score^gamma.
        Premise: intent/sentiment one-liners share phrasings with their test
        rows, so neighbourhood vote may beat the generative count model on
        suites where NB plateaus (banking77 / massive / sst5 / emotion).
  T7(d) Hebbian bilinear xnli map — per-class premise-token x hypothesis-token
        co-occurrence counts accumulated by outer product over TRAIN (Hebbian:
        pure co-occurrence accumulation, no gradient descent, modelless).
        Score(class) = sum over premise x hypothesis pairs of a contrastive
        log-odds weight; optionally added to the existing Pair-view MNB term
        (the actual lever shape: a new interaction term beside the bag).

Protocol (same law as the harness, `src/harness/suites.rs`): corpora and
tables from TRAIN rows only; the cal slice is a label-stratified TRAIN front
whose own docs are excluded from the corpus; posture selected on cal; test
read ONCE at the selected posture. Tokenization is imported verbatim from
`issue038_t7_probe` (the src/embed.rs mirror) so the numbers are comparable.

Box state 2026-09-27: M3, AC power, accuracy-only probe (no latency claims —
bench_preflight not owed). Results land in the issue/plan record, never code.
"""

from __future__ import annotations

import heapq
import math
import os
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import issue038_t7_probe as base  # noqa: E402  (tokens/docs_of/load/PUBLISHED)
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

ROOT = base.ROOT
TRIM = base.TRIM
fnv1a = base.fnv1a

BEST_KNOWN = {  # current lane best (Bench 052 protocol + genome/arms), echo only
    "banking77": ("genome H2 0.8620", 0.8620),
    "emotion": ("A1 ridge 0.8550", 0.8550),
    "sst5": ("genome 0.4017", 0.4017),
    "massive_intent_en": ("H2 0.8267", 0.8267),
    "ag_news": ("H2 0.8975", 0.8975),
    "xnli_en": ("Pair view 0.520", 0.520),
}


def ns_tokens(text: str, salt: bytes) -> list[int]:
    """embed.rs mirror with a namespace salt (unigrams + bigrams, salted)."""
    out: list[int] = []
    prev: int | None = None
    for raw in text.encode("utf-8").split():
        t = raw.strip(bytes(TRIM))
        if not t:
            continue
        h = fnv1a(salt + t) % base.V
        out.append(h)
        if prev is not None:
            pair = prev.to_bytes(8, "little") + h.to_bytes(8, "little")
            out.append(fnv1a(salt + pair) % base.V)
        prev = h
    return out


def unigrams(text: str, salt: bytes) -> list[int]:
    return [fnv1a(salt + t) % base.V
            for raw in text.encode("utf-8").split()
            if (t := raw.strip(bytes(TRIM)))]


# ── the shipped Pair view (src/nb_scope.rs pair_tokens_into), mirrored ────
# Shape-only mirror (probe-local salts): hypothesis words + bigrams, novel
# (premise-absent) hypothesis words, coverage/length/negation meta tokens.

NEG = {"not", "no", "never", "nobody", "nothing", "none", "neither", "nor",
       "cannot", "without", "nowhere", "hardly", "isnt", "dont"}


def rust_words(span: str):
    for w in span.split():
        t = w.strip("_.-!\"'()[]{}<>,;:=+*/\\|~`@#$%^&?")
        if t:
            yield t.encode("utf-8")


def shipped_pair_tokens(p: str, h: str) -> list[int]:
    if not p or not h:
        return base.tokens(p + "\n" + h)
    out: list[int] = []
    ctx = set()
    ctx_neg = False
    for w in rust_words(p):
        ctx.add(fnv1a(b"N" + w) % base.V)
        ctx_neg |= w.lower() in NEG or (len(w) > 3 and w[-3:].lower() in ("n't", 'n"t'))
    (n_words, covered, hyp_neg) = (0, 0, False)
    prev = None
    for w in rust_words(h):
        n_words += 1
        wl = w.lower()
        hyp_neg |= wl in NEG or (len(w) > 3 and wl[-3:] in ("n't", 'n"t'))
        hw = fnv1a(b"H" + w) % base.V
        out.append(hw)
        if prev is not None:
            out.append(fnv1a(b"B" + prev.to_bytes(8, "little")
                             + hw.to_bytes(8, "little")) % base.V)
        prev = hw
        nh = fnv1a(b"N" + w) % base.V
        if nh in ctx:
            covered += 1
        else:
            out.append(nh)
    cov = (covered * 5) // n_words if n_words else 0
    for m in (b"cov" + str(cov).encode(),
              b"len" + str(min(n_words // 4, 5)).encode(),
              b"neg" + str((2 if ctx_neg else 0) + (1 if hyp_neg else 0)).encode()):
        out.append(fnv1a(b"M" + m) % base.V)
    return out


# ── T7(c): BM25-kNN ────────────────────────────────────────────────────────


class BM25Knn:
    def __init__(self, docs_toks, k1: float, b: float):
        self.docs = docs_toks
        self.N = len(docs_toks)
        self.avgdl = sum(len(d) for d in docs_toks) / max(self.N, 1)
        self.k1, self.b = k1, b
        df = Counter()
        for d in docs_toks:
            df.update(set(d))
        self.idf = {t: math.log(1.0 + (self.N - n + 0.5) / (n + 0.5))
                    for t, n in df.items()}
        self.post: dict[int, list[tuple[int, int]]] = defaultdict(list)
        for i, d in enumerate(docs_toks):
            for t, f in Counter(d).items():
                self.post[t].append((i, f))

    def scores(self, q_toks) -> dict[int, float]:
        out: dict[int, float] = defaultdict(float)
        for t in set(q_toks):
            idf = self.idf.get(t)
            if idf is None:
                continue
            for i, f in self.post[t]:
                denom = f + self.k1 * (1 - self.b + self.b * len(self.docs[i]) / self.avgdl)
                out[i] += idf * f * (self.k1 + 1) / denom
        return out

    def vote(self, q_toks, labels_of, k: int, gamma: float):
        sc = self.scores(q_toks)
        if not sc:
            return 0, 0.0
        top = heapq.nlargest(k, sc.items(), key=lambda kv: kv[1])
        w: dict[str, float] = defaultdict(float)
        for i, s in top:  # inserted best-neighbour first; max() keeps first max
            w[labels_of[i]] += (s ** gamma if gamma > 0 else 1.0)
        best = max(w.items(), key=lambda kv: kv[1])[0]
        margin = max(w.values()) / (sum(w.values()) or 1.0)
        return best, margin


def stratified_front(docs, labels, per_label):
    by = defaultdict(list)
    for d in docs:
        by[d[0]].append(d)
    front = []
    for lab in labels:
        front += by[lab][:per_label]
    return front


def run_knn(suite, ks=(1, 3, 5, 9, 15), k1s=(1.2, 1.5), bs=(0.3, 0.75),
            gammas=(0.0, 1.0), front_per_label=8):
    train = base.docs_of(suite, base.load(suite, "train"))
    test = base.docs_of(suite, base.load(suite, "test"))
    labels = sorted({lab for lab, _ in train})
    cal = stratified_front(train, labels, front_per_label)
    cal_texts = {t for _, t in cal}
    pool = [(l, t) for (l, t) in train if t not in cal_texts]
    toks = [base.tokens(t) for _, t in pool]
    labels_of = [l for l, _ in pool]

    # MNB reference (shipped shape) on the same cal/test
    cnt = base.Counts()
    for (l, t), tk in zip(pool, toks):
        cnt.add(l, tk)
    mnb_cal = base.mnb_eval(cnt, cal, labels)

    results = {}
    for k1 in k1s:
        for b in bs:
            idx = BM25Knn(toks, k1, b)
            for k in ks:
                for g in gammas:
                    c = sum(idx.vote(base.tokens(t), labels_of, k, g)[0] == l
                            for l, t in cal)
                    results[(k1, b, k, g)] = c / len(cal)
    posture = max(results, key=results.get)
    k1, b, k, g = posture
    idx = BM25Knn(toks, k1, b)
    acc_test = sum(idx.vote(base.tokens(t), labels_of, k, g)[0] == l
                   for l, t in test) / len(test)
    mnb_test = base.mnb_eval(cnt, test, labels)
    best_name, best_val = BEST_KNOWN[suite]
    print(f"== {suite}: pool={len(pool)} cal={len(cal)} test={len(test)} "
          f"labels={len(labels)}  (MNB cal={mnb_cal:.4f} test={mnb_test:.4f})")
    for c in sorted(results, key=results.get, reverse=True)[:4]:
        print(f"   kNN k1={c[0]} b={c[1]} k={c[2]:2d} g={c[3]} -> cal={results[c]:.4f}")
    print(f"   SELECTED k1={k1} b={b} k={k} g={g} cal={results[posture]:.4f}")
    print(f"   kNN test={acc_test:.4f}   [best-known {best_name} / MNB {mnb_test:.4f}]")
    return {"suite": suite, "knn_test": acc_test, "posture": posture,
            "knn_cal": results[posture], "mnb_test": mnb_test}


# ── T7(d): Hebbian bilinear xnli ───────────────────────────────────────────


def pair_docs(rows):
    out = []
    for r in rows:
        try:
            lab = str(int(r["label"]))
        except Exception:
            continue
        p, h = r.get("premise"), r.get("hypothesis")
        if p is None or h is None:
            continue
        out.append((lab, p, h))
    return out


class HebbianBilinear:
    """Per-class premise x hypothesis unigram co-occurrence, contrastive
    log-odds weights, restricted to a top-k df vocabulary. Weights are
    computed on the fly per QUERY pair (never materialized: the table is
    millions of entries, a question touches only |premise|x|hypothesis|)."""

    def __init__(self, pool_facts, vocab_k: int, alpha: float = 1.0):
        df = Counter()
        for _, _, tp, th in pool_facts:
            df.update(tp)
            df.update(th)
        self.vocab = {w for w, _ in sorted(df.items(),
                                           key=lambda kv: (-kv[1], kv[0]))[:vocab_k]}
        self.cc: dict[str, Counter] = defaultdict(Counter)
        self.cnt: Counter = Counter()
        for lab, _, tp, th in pool_facts:
            for i in tp:
                if i not in self.vocab:
                    continue
                for j in th:
                    if j not in self.vocab:
                        continue
                    self.cc[lab][(i, j)] += 1
                    self.cnt[(i, j)] += 1
        self.n_by_lab = Counter(lab for lab, _, _, _ in pool_facts)
        self.n = len(pool_facts)
        self.alpha = alpha

    def score(self, tp, th, lab: str) -> float:
        ps = {t for t in tp if t in self.vocab}
        hs = {t for t in th if t in self.vocab}
        n_c = self.n_by_lab[lab]
        n_o = self.n - n_c
        a = self.alpha
        ccl = self.cc[lab]
        cnt = self.cnt
        pin, pout = a * n_c / self.n, a * n_o / self.n
        din, dout = math.log(n_c + a), math.log(n_o + a)
        s = 0.0
        for i in ps:
            for j in hs:
                pair = (i, j)
                tot = cnt.get(pair)
                if tot is None:
                    continue
                cin = ccl.get(pair, 0)
                cout = tot - cin
                s += math.log(cin + pin) - din - (math.log(cout + pout) - dout)
        return s


def run_hebb(vocab_ks=(2048, 8192), priors=(0.0, 1.0),
             lambdas=(0.0, 0.25, 0.5, 1.0, 2.0), front_per_label=100,
             full_pair=False):
    """Combined lever shape: MNB + lambda * bilinear log-odds. lambda=0 is
    the reference row. full_pair=False uses the bare namespaced bag (the
    T3 events MINUS the cross features); True mirrors the SHIPPED pair view
    (novel marks + coverage/length/negation meta) — the decisive premise
    test: does the interaction term add on top of everything shipped?"""
    bag_of = shipped_pair_tokens if full_pair else \
        (lambda p, h: ns_tokens(p, b"P:") + ns_tokens(h, b"H:"))
    rows_tr = base.load("xnli_en", "train")
    rows_te = base.load("xnli_en", "test")
    train, test = pair_docs(rows_tr), pair_docs(rows_te)
    labels = sorted({l for l, _, _ in train})
    cal = stratified_front([(l, p + "\n" + h) for l, p, h in train],
                           labels, front_per_label)
    cal_keys = {t for _, t in cal}
    pool = [(l, p, h) for (l, p, h) in train
            if p + "\n" + h not in cal_keys]

    def row_facts(lab, p, h):
        return (lab, bag_of(p, h),
                set(unigrams(p, b"P:")), set(unigrams(h, b"H:")))

    pool_f = [row_facts(*r) for r in pool]
    cal_f = [row_facts(l, *t.split("\n")) for l, t in cal]
    test_f = [row_facts(*r) for r in test]

    # Reference: MNB over the bag (bare or shipped-shape)
    cnt = base.Counts()
    for lab, bag, _, _ in pool_f:
        cnt.add(lab, bag)
    mnb_cal = sum(max(labels, key=lambda l2: mnb_log(cnt, bag, l2)) == lab
                  for lab, bag, _, _ in cal_f) / len(cal_f)
    mnb_test = sum(max(labels, key=lambda l2: mnb_log(cnt, bag, l2)) == lab
                   for lab, bag, _, _ in test_f) / len(test_f)

    def fused(m, lab, bag, tp, th, lam, prior):
        best, best_s = None, None
        for l2 in labels:
            s = mnb_log(cnt, bag, l2)
            if lam > 0:
                s += lam * m.score(tp, th, l2)
            if prior:
                s += prior * math.log(m.n_by_lab[l2] / m.n)
            if best_s is None or s > best_s:
                best, best_s = l2, s
        return best

    results = {}
    models = {}
    for vk in vocab_ks:
        m = HebbianBilinear(pool_f, vk)
        models[vk] = m
        for prior in priors:
            for lam in lambdas:
                c = sum(fused(m, lab, bag, tp, th, lam, prior) == lab
                        for lab, bag, tp, th in cal_f)
                results[(vk, prior, lam)] = c / len(cal_f)
    posture = max(results, key=results.get)
    vk, prior, lam = posture
    m = models[vk]
    acc_test = sum(fused(m, lab, bag, tp, th, lam, prior) == lab
                   for lab, bag, tp, th in test_f) / len(test_f)
    print(f"== xnli_en[{('shipped-pair' if full_pair else 'bare-pair')}]: "
          f"pool={len(pool_f)} cal={len(cal_f)} test={len(test_f)} "
          f"(MNB cal={mnb_cal:.4f} test={mnb_test:.4f})")
    for cpos in sorted(results, key=results.get, reverse=True)[:6]:
        print(f"   hebb vk={cpos[0]} prior={cpos[1]} lam={cpos[2]} "
              f"-> cal={results[cpos]:.4f}")
    print(f"   SELECTED vk={vk} prior={prior} lam={lam} cal={results[posture]:.4f}")
    print(f"   hebb test={acc_test:.4f}   [best-known Pair 0.520 / MNB {mnb_test:.4f}]")
    return {"hebb_test": acc_test, "posture": posture,
            "mnb_pair_test": mnb_test}


def mnb_log(cnt, toks, lab):
    alpha = 1.0
    cc = cnt.per.get(lab)
    tot = cnt.tot[lab]
    denom = tot + alpha * base.V
    s = math.log((tot + alpha) / (cnt.ndocs + alpha))
    for w in toks:
        s += math.log(((cc[w] if cc else 0) + alpha) / denom)
    return s


def main() -> int:
    which = sys.argv[1] if len(sys.argv) > 1 else "all"
    if which in ("all", "knn"):
        for suite in ("banking77", "emotion", "sst5", "massive_intent_en", "ag_news"):
            try:
                run_knn(suite)
            except Exception as e:
                print(f"== {suite}: FAILED {type(e).__name__}: {e}")
    if which in ("all", "hebb"):
        for full in (False, True):
            try:
                run_hebb(full_pair=full)
            except Exception:
                import traceback
                traceback.print_exc()
                print(f"== xnli_en hebb(full={full}): FAILED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
