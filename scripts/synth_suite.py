#!/usr/bin/env python3
"""synth_suite.py -- first-party synthetic classification suites (`emotion_syn`, `sst5_syn`).

riir-rethink Research 002 §Execution step 1 (owner call 2026-10-27: synthetic
emotion FIRST). Replaces the licence-barred dair-ai/emotion niche with rows we
generate ourselves from Apache-2.0 open-weights models -- the typed_decisions
precedent ("latent factors + our teacher labels, no scraped content").

Design (pinned before any fit -- amendment A3 of the ESC re-source GOAT):
  * Wire: the SAME six labels and ClassLabel order as `build_emotion`
    (sadness, joy, love, anger, fear, surprise), so the harness reuses the
    suite builder unchanged.
  * TRAIN split: generator A (Ternary-Bonsai-2-27B, Apache-2.0) over prompt
    family "A" (its own topic + register lists).
  * TEST split: generator B (Qwen3.5-4B, Apache-2.0) over prompt family "B"
    (DISJOINT topic + register lists, different instruction shape) -- the
    outside-teacher / outside-prompt-family holdout: a model that memorises
    generator A's style or family A's topics gains nothing on test.
  * Labels: intended label from the latent factors, KEPT only when the OTHER
    model's zero-shot read agrees (first-token logprob argmax over the six
    label words) -- every kept row is dual-model confirmed.
  * Exact + normalised dedupe within and ACROSS splits (test rows that
    collide with train are dropped from test).
  * Output: `.raw/datasets/emotion_syn/<split>-NNN.json`, the HF
    datasets-server page shape the harness's `rows_of` reads, plus
    `manifest.json` (generator digests, factor lists, per-label veto rates,
    blake3 of every page when the `blake3` module is importable, else sha256).

Run with an MLX venv (Apple Silicon), e.g.:
  .venv/bin/python scripts/synth_suite.py --suite emotion_syn --split train --n 4800 \\
      --gen bonsai:<pack_dir> --judge mlxlm:<qwen_dir> --seed 1
  .venv/bin/python scripts/synth_suite.py --suite emotion_syn --split test --n 1200 \\
      --gen mlxlm:<qwen_dir> --judge bonsai:<pack_dir> --seed 2

`sst5_syn` (Research 002 step 2) is the same lane over the five SST-5 levels
(label = the 0..4 score index, `build_sst5`'s wire); its judge answers with a
digit 1..5 because the level NAMES share first tokens ("very ...").
"""

import argparse
import hashlib
import json
import os
import random
import re
import sys
import time
from pathlib import Path

LABELS = ["sadness", "joy", "love", "anger", "fear", "surprise"]
LABEL_GLOSS = {
    "sadness": "sadness (grief, loss, loneliness, disappointment, feeling down)",
    "joy": "joy (happiness, delight, pride, relief, contentment)",
    "love": "love (affection, tenderness, caring, romantic or familial warmth toward someone)",
    "anger": "anger (irritation, frustration, outrage, resentment)",
    "fear": "fear (worry, anxiety, dread, nervousness about what might happen)",
    "surprise": "surprise (astonishment, being caught off guard, disbelief at something unexpected)",
}
INTENSITY = ["mild", "moderate", "strong"]

# Family A (train) and family B (test) are DISJOINT by construction -- the
# self-test below asserts it.
FAMILY = {
    "A": {
        "topics": [
            "a job or workplace", "a sibling", "a school exam", "a pet",
            "cooking dinner", "a long commute", "a doctor's appointment",
            "a phone call from a parent", "moving to a new apartment",
            "a sports match", "a birthday", "a broken appliance",
            "a neighbour", "a group project", "an old photo",
            "a rainy weekend", "a garden", "a concert",
        ],
        "registers": [
            "short social media posts", "diary entries", "text messages to a friend",
            "forum comments",
        ],
        "persons": ["first person", "first person", "third person"],
    },
    "B": {
        "topics": [
            "a flight delay", "a wedding", "a hospital stay", "a promotion",
            "a lost wallet", "a grandparent", "a first date", "a job interview",
            "a car repair", "a holiday trip", "a news headline", "a child's school play",
            "a landlord", "a reunion with an old friend", "a medical result",
            "a power outage", "a new baby", "a game night",
        ],
        "registers": [
            "spoken remarks to a coworker", "personal blog sentences",
            "voice-note transcripts", "comments under a video",
        ],
        "persons": ["first person"],
    },
}

PROMPT_A = (
    "Write {k} different {register} in {person}. In each, the writer is dealing with "
    "{topic} and feels {intensity} {gloss}. {lexical} Vary sentence structure, "
    "vocabulary and length (6 to 35 words). Output exactly {k} lines, one text per "
    "line, with no numbering, no quotes and no commentary."
)
PROMPT_B = (
    "Imagine many different people, each reacting to {topic}. Give {k} separate "
    "{register} (each 6 to 35 words, in first person) where the dominant emotion is "
    "{gloss}, at {intensity} strength. {lexical} Every line must be a different "
    "person with a different detail. One per line. Do not number them or add anything else."
)
LEXICAL = [
    "Do not name the emotion directly; show it through what happens and how they react.",
    "They may describe their feelings in their own words.",
]
JUDGE_PROMPT = (
    "Read the text and decide which single emotion it most strongly expresses.\n"
    "Options: sadness, joy, love, anger, fear, surprise.\n\n"
    "Text: {text}\n\nAnswer with exactly one word from the options."
)

SST5_LABELS = ["very negative", "negative", "neutral", "positive", "very positive"]
SST5_GLOSS = {
    "very negative": "very negative (scathing, furious, thoroughly let down)",
    "negative": "negative (disappointed, unimpressed, mildly annoyed)",
    "neutral": "neutral (balanced, factual, neither pleased nor displeased)",
    "positive": "positive (pleased, satisfied, would recommend)",
    "very positive": "very positive (delighted, enthusiastic, the best of its kind)",
}
SST5_FAMILY = {
    "A": {
        "topics": [
            "a film", "a novel", "a restaurant meal", "a TV series", "a music album",
            "a stage play", "a video game", "a cafe", "a documentary", "a comic book",
            "a bakery", "a podcast",
        ],
        "registers": ["one-sentence review snippets", "short critic blurbs", "forum review lines"],
        "persons": ["first person", "third person"],
    },
    "B": {
        "topics": [
            "a hotel stay", "a phone app", "a pair of headphones", "a gym", "a museum exhibit",
            "a car rental", "a theme park", "a hair salon", "a delivery service",
            "a laptop", "a yoga class", "a furniture store",
        ],
        "registers": ["spoken opinions to a friend", "replies in a comment thread",
                      "quick survey answers"],
        "persons": ["first person"],
    },
}
SST5_PROMPT_A = (
    "Write {k} different {register} in {person} about {topic}. Each expresses a "
    "{gloss} overall opinion, at {intensity} strength. {lexical} Vary structure, "
    "vocabulary and length (6 to 35 words). Output exactly {k} lines, one per line, "
    "with no numbering, no quotes, no star ratings and no commentary."
)
SST5_PROMPT_B = (
    "Imagine many different customers giving {register} about {topic}. Give {k} of "
    "them (each 6 to 35 words, first person) whose overall opinion is {gloss}, at "
    "{intensity} strength. {lexical} Every line a different person and detail. One "
    "per line, no numbering, no ratings, nothing else."
)
SST5_LEXICAL = [
    "Do not state a rating or score; let the opinion come through the details.",
    "They may say plainly how they feel about it.",
]
SST5_JUDGE_PROMPT = (
    "Rate the overall sentiment of the text on a 5-point scale: 1 = very negative, "
    "2 = negative, 3 = neutral, 4 = positive, 5 = very positive.\n\n"
    "Text: {text}\n\nAnswer with a single digit from 1 to 5."
)

# One spec per suite. emotion_syn's values are the module constants above
# UNCHANGED (its first run pinned them); new suites add rows, never edit one.
SPECS = {
    "emotion_syn": {
        "labels": LABELS, "gloss": LABEL_GLOSS, "family": FAMILY,
        "prompt": {"A": PROMPT_A, "B": PROMPT_B}, "lexical": LEXICAL,
        "intensity": INTENSITY, "judge_prompt": JUDGE_PROMPT, "judge_answers": LABELS,
    },
    "sst5_syn": {
        "labels": SST5_LABELS, "gloss": SST5_GLOSS, "family": SST5_FAMILY,
        "prompt": {"A": SST5_PROMPT_A, "B": SST5_PROMPT_B}, "lexical": SST5_LEXICAL,
        "intensity": INTENSITY, "judge_prompt": SST5_JUDGE_PROMPT,
        "judge_answers": ["1", "2", "3", "4", "5"],
    },
}

META_RE = re.compile(r"^(here (are|is)|sure|okay|certainly|note:|\*\*)", re.I)
LEAD_RE = re.compile(r"^\s*(?:[-*•]|\d+[.)]|\(?\d+\))\s*")


# -- backends ---------------------------------------------------------------

class Backend(object):
    """generate(prompt, max_tokens, temp) -> str; first_logits(prompt) -> mx array."""

    name = "?"
    digest = "?"


class MlxLmBackend(Backend):
    def __init__(self, path):
        from mlx_lm import load
        self.model, self.tok = load(path)
        self.name = "mlxlm:" + Path(path).parent.parent.name
        self.digest = _dir_digest(path)

    def render(self, prompt):
        return self.tok.apply_chat_template(
            [{"role": "user", "content": prompt}], add_generation_prompt=True,
            tokenize=False, enable_thinking=False)

    def encode(self, text):
        return self.tok.encode(text, add_special_tokens=False)

    def generate(self, prompt, max_tokens, temp):
        from mlx_lm import generate
        from mlx_lm.sample_utils import make_sampler
        return generate(self.model, self.tok, self.render(prompt), max_tokens=max_tokens,
                        sampler=make_sampler(temp=temp, top_p=0.95))

    def first_logits(self, prompt):
        import mlx.core as mx
        ids = mx.array([self.encode(self.render(prompt))])
        return self.model(ids)[0, -1, :]


class BonsaiBackend(Backend):
    def __init__(self, pack):
        sys.path.insert(0, str(Path(pack) / "runtime"))
        from tokenizers import Tokenizer
        from jinja2.sandbox import ImmutableSandboxedEnvironment
        self.pack = Path(pack)
        cfg = json.loads((self.pack / "config.json").read_text(encoding="utf-8"))
        if cfg.get("components", {}).get("vision"):
            # schema v2 packs carry the vision tower: the pack's mlx-vlm loader,
            # text generation through its language model only.
            from vision_artifact import load_vl_model
            vl, _, _ = load_vl_model(self.pack, load_processor=False)
            self.lm = vl.language_model
        else:
            from artifact import load_model  # the pack's text loader (PACK-RUNTIME.md)
            m, _ = load_model(self.pack)
            self.lm = _TextLm(m)
        self.tok = Tokenizer.from_file(str(self.pack / "tokenizer.json"))
        self.tmpl = ImmutableSandboxedEnvironment(trim_blocks=True, lstrip_blocks=True) \
            .from_string((self.pack / "chat_template.jinja").read_text(encoding="utf-8"))
        gen = json.loads((self.pack / "generation_config.json").read_text(encoding="utf-8"))
        eos = gen.get("eos_token_id")
        stop = set(eos if isinstance(eos, list) else [] if eos is None else [eos])
        for t in ("<|im_end|>", "<|endoftext|>"):
            i = self.tok.token_to_id(t)
            if i is not None:
                stop.add(i)
        self.stop = stop
        self.name = "bonsai:" + self.pack.parent.parent.name
        self.digest = _dir_digest(pack)

    def render(self, prompt):
        return self.tmpl.render(messages=[{"role": "user", "content": prompt}],
                                add_generation_prompt=True, enable_thinking=False)

    def encode(self, text):
        return self.tok.encode(text, add_special_tokens=False).ids

    def _forward(self, x, cache):
        out = self.lm(x, cache=cache)
        logits = getattr(out, "logits", out)
        return logits[:, -1, :]

    def _cache(self):
        return self.lm.make_cache()

    def generate(self, prompt, max_tokens, temp):
        import mlx.core as mx
        from mlx_lm.sample_utils import make_sampler
        sample = make_sampler(temp=temp, top_p=0.95)
        x = mx.array([self.encode(self.render(prompt))])
        cache = self._cache()
        out = []
        for _ in range(max_tokens):
            x = sample(self._forward(x, cache))[:, None]
            mx.eval(x)
            t = int(x.item())
            if t in self.stop:
                break
            out.append(t)
        return self.tok.decode(out)

    def first_logits(self, prompt):
        import mlx.core as mx
        x = mx.array([self.encode(self.render(prompt))])
        return self._forward(x, self._cache())[0]


class _TextLm(object):
    """Adapter: the text-only pack (artifact.load_model) to the lm(x, cache) shape."""

    def __init__(self, model):
        self.m = model

    def __call__(self, x, cache=None):
        return self.m.lm_head(self.m.model(x, cache=cache))

    def make_cache(self):
        return self.m.make_cache()


def _dir_digest(path):
    """sha256 over the weight files' (name, size, first+last MiB) -- cheap and
    stable; the full weights are pinned by the HF snapshot revision in the name."""
    h = hashlib.sha256()
    for f in sorted(Path(path).glob("*.safetensors")):
        st = f.stat()
        h.update(f.name.encode("utf-8"))
        h.update(str(st.st_size).encode("utf-8"))
        with open(f, "rb") as fh:
            h.update(fh.read(1 << 20))
            fh.seek(max(0, st.st_size - (1 << 20)))
            h.update(fh.read(1 << 20))
    return h.hexdigest()[:16]


def make_backend(spec):
    kind, _, path = spec.partition(":")
    if kind == "bonsai":
        return BonsaiBackend(path)
    if kind == "mlxlm":
        return MlxLmBackend(path)
    raise SystemExit("unknown backend kind: " + kind)


# -- judge ------------------------------------------------------------------

class Judge(object):
    """First-token logprob argmax over the spec's judge answers (index-aligned
    with its labels). Refuses (raises) when two answers share a first token --
    the read would be ambiguous."""

    def __init__(self, backend, spec):
        self.b = backend
        self.labels = spec["labels"]
        self.prompt = spec["judge_prompt"]
        firsts = []
        for lab in spec["judge_answers"]:
            ids = backend.encode(lab)
            if not ids:
                raise SystemExit("judge: empty encoding for " + lab)
            firsts.append(ids[0])
        if len(set(firsts)) != len(firsts):
            raise SystemExit("judge: label first-tokens collide: %r" % (firsts,))
        self.firsts = firsts

    def read(self, text):
        import mlx.core as mx
        logits = self.b.first_logits(self.prompt.format(text=text))
        sel = mx.take(logits, mx.array(self.firsts))
        mx.eval(sel)
        vals = sel.tolist()
        return self.labels[max(range(len(vals)), key=lambda i: vals[i])]


# -- text hygiene -----------------------------------------------------------

def norm(text):
    return re.sub(r"[^a-z0-9 ]+", "", text.lower()).strip()


def clean_lines(raw):
    out = []
    for line in raw.splitlines():
        line = LEAD_RE.sub("", line).strip().strip('"').strip("“”").strip()
        if not line or META_RE.match(line):
            continue
        words = len(line.split())
        if words < 5 or words > 45:
            continue
        out.append(line)
    return out


# -- pages ------------------------------------------------------------------

def features(labels):
    return [
        {"feature_idx": 0, "name": "text", "type": {"dtype": "string", "_type": "Value"}},
        {"feature_idx": 1, "name": "label", "type": {"names": labels, "_type": "ClassLabel"}},
    ]


def page_digest(b):
    try:
        import blake3  # type: ignore
        return "blake3:" + blake3.blake3(b).hexdigest()
    except ImportError:
        return "sha256:" + hashlib.sha256(b).hexdigest()


def write_pages(out_dir, split, rows, labels):
    digests = {}
    for p in range(0, len(rows), 100):
        chunk = rows[p:p + 100]
        body = {
            "features": features(labels),
            "rows": [{"row_idx": p + i, "row": r, "truncated_cells": []}
                     for i, r in enumerate(chunk)],
            "num_rows_total": len(rows),
            "num_rows_per_page": 100,
            "partial": False,
        }
        name = "%s-%03d.json" % (split, p // 100)
        data = json.dumps(body, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        (out_dir / name).write_bytes(data)
        digests[name] = page_digest(data)
    return digests


def load_existing_norms(out_dir, split):
    seen = set()
    for f in sorted(out_dir.glob(split + "-*.json")):
        body = json.loads(f.read_text(encoding="utf-8"))
        for r in body.get("rows", []):
            seen.add(norm(r["row"]["text"]))
    return seen


# -- checkpoint -------------------------------------------------------------

class Checkpoint(object):
    """Append-only per-prompt journal: `<out>/.partial-<split>.jsonl`.

    A run killed from outside (the measured 2026-10-27 loss: 4,000/4,800 rows,
    50 min, no traceback) resumes from the journal instead of from zero. Line 1
    is the run identity; a resume under a different identity REFUSES (mixing
    two generators' rows would silently corrupt the A3 holdout). Each later
    line is one prompt: [label, [[norm_key, outcome, text_or_null], ...]],
    outcome in "dupe" | "veto" | "kept" | "surplus". Flushed + fsynced per
    prompt (~1/s), removed only after the pages are written.
    """

    def __init__(self, path, identity):
        self.path, self.identity = Path(path), identity
        self.fh = None

    def replay(self, labels):
        kept = {lab: [] for lab in labels}
        stats = {lab: {"generated": 0, "agreed": 0, "dupes": 0} for lab in labels}
        seen, prompts = set(), 0
        if not self.path.exists():
            return kept, stats, seen, prompts
        lines = self.path.read_text(encoding="utf-8").splitlines()
        if not lines:
            return kept, stats, seen, prompts
        head = json.loads(lines[0])
        if head != self.identity:
            raise SystemExit("REFUSE resume: journal %s identity %r != this run %r "
                             "(delete it to start fresh)" % (self.path, head, self.identity))
        for raw in lines[1:]:
            try:
                lab, outcomes = json.loads(raw)
            except ValueError:
                break  # a torn final line from the kill; everything before it is whole
            prompts += 1
            for key, outcome, text in outcomes:
                stats[lab]["generated"] += 1
                if outcome == "dupe":
                    stats[lab]["dupes"] += 1
                    continue
                seen.add(key)
                if outcome in ("kept", "surplus"):
                    stats[lab]["agreed"] += 1
                if outcome == "kept":
                    kept[lab].append(text)
        return kept, stats, seen, prompts

    def open(self):
        fresh = not self.path.exists() or self.path.stat().st_size == 0
        self.fh = open(self.path, "a", encoding="utf-8")
        if fresh:
            self._write(self.identity)

    def record(self, lab, outcomes):
        self._write([lab, outcomes])

    def _write(self, obj):
        self.fh.write(json.dumps(obj, ensure_ascii=False) + "\n")
        self.fh.flush()
        os.fsync(self.fh.fileno())

    def finish(self):
        if self.fh:
            self.fh.close()
        self.path.unlink()


# -- main -------------------------------------------------------------------

def self_test():
    for name, spec in sorted(SPECS.items()):
        a, b = spec["family"]["A"], spec["family"]["B"]
        assert not set(a["topics"]) & set(b["topics"]), name + ": topic families overlap"
        assert not set(a["registers"]) & set(b["registers"]), name + ": register families overlap"
        assert len(spec["judge_answers"]) == len(spec["labels"]), name + ": judge/label arity"
        assert set(spec["gloss"]) == set(spec["labels"]), name + ": gloss keys"
    assert clean_lines("1. Here are lines\n- I can't stop smiling at the letter she sent me\nok") == \
        ["I can't stop smiling at the letter she sent me"]
    assert norm("Hi, THERE!") == "hi there"
    _self_test_checkpoint()
    print("self-test OK")


def _self_test_checkpoint():
    import tempfile
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / ".partial-train.jsonl"
        ident = {"suite": "t", "split": "train", "n": 4}
        ck = Checkpoint(path, ident)
        ck.open()
        ck.record("a", [["x", "kept", "X"], ["y", "veto", None], ["x", "dupe", None]])
        ck.record("b", [["z", "kept", "Z"], ["w", "surplus", None]])
        ck.fh.write('["a", [["q", "ke')  # the torn tail a kill leaves
        ck.fh.close()
        kept, stats, seen, prompts = Checkpoint(path, ident).replay(["a", "b"])
        assert kept == {"a": ["X"], "b": ["Z"]}, kept
        assert prompts == 2 and seen == {"x", "y", "z", "w"}, (prompts, seen)
        assert stats["a"] == {"generated": 3, "agreed": 1, "dupes": 1}, stats["a"]
        assert stats["b"]["agreed"] == 2, stats["b"]
        try:
            Checkpoint(path, dict(ident, n=5)).replay(["a", "b"])
            raise AssertionError("identity mismatch must refuse")
        except SystemExit:
            pass


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", choices=sorted(SPECS), default="emotion_syn")
    ap.add_argument("--split", choices=["train", "test"])
    ap.add_argument("--n", type=int, default=0, help="target kept rows (balanced over labels)")
    ap.add_argument("--gen", help="bonsai:<pack_dir> | mlxlm:<model_dir>")
    ap.add_argument("--judge", help="the OTHER model (veto)")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--k", type=int, default=8, help="texts asked per prompt")
    ap.add_argument("--temp", type=float, default=0.9)
    ap.add_argument("--out", default="", help="default .raw/datasets/<suite>")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return
    self_test()
    if not (a.split and a.n and a.gen and a.judge):
        ap.error("--split --n --gen --judge are required")

    import mlx.core as mx
    spec = SPECS[a.suite]
    labels = spec["labels"]
    random.seed(a.seed)
    mx.random.seed(a.seed)
    fam_id = "A" if a.split == "train" else "B"
    fam = spec["family"][fam_id]
    tmpl = spec["prompt"][fam_id]
    out_dir = Path(a.out or (".raw/datasets/" + a.suite))
    out_dir.mkdir(parents=True, exist_ok=True)
    # Cross-split dedupe: test never repeats a train text.
    blocked = load_existing_norms(out_dir, "train") if a.split == "test" else set()

    gen = make_backend(a.gen)
    judge = Judge(make_backend(a.judge), spec)
    quota = a.n // len(labels)
    identity = {"suite": a.suite, "split": a.split, "n": a.n, "seed": a.seed, "k": a.k,
                "temp": a.temp, "gen": gen.name, "gen_digest": gen.digest,
                "judge": judge.b.name, "judge_digest": judge.b.digest}
    ck = Checkpoint(out_dir / (".partial-%s.jsonl" % a.split), identity)
    kept, stats, seen, prompts = ck.replay(labels)
    resumed_prompts = prompts
    if prompts:
        # A fresh stream per resume point: re-running seed's prefix would
        # re-ask the prompts already journaled (all dupes).
        random.seed(a.seed * 1000003 + prompts)
        mx.random.seed(a.seed * 1000003 + prompts)
        print("[%s %s] RESUME prompts=%d kept=%d" % (
            a.suite, a.split, prompts, sum(len(v) for v in kept.values())), flush=True)
    seen |= blocked
    ck.open()
    t0 = time.time()
    while any(len(kept[lab]) < quota for lab in labels):
        lab = min(labels, key=lambda x: len(kept[x]))
        prompt = tmpl.format(
            k=a.k, register=random.choice(fam["registers"]), person=random.choice(fam["persons"]),
            topic=random.choice(fam["topics"]), intensity=random.choice(spec["intensity"]),
            gloss=spec["gloss"][lab], lexical=random.choice(spec["lexical"]))
        lines = clean_lines(gen.generate(prompt, max_tokens=60 * a.k, temp=a.temp))
        prompts += 1
        outcomes = []
        for line in lines:
            stats[lab]["generated"] += 1
            key = norm(line)
            if key in seen:
                stats[lab]["dupes"] += 1
                outcomes.append([key, "dupe", None])
                continue
            seen.add(key)
            if judge.read(line) != lab:
                outcomes.append([key, "veto", None])
                continue
            stats[lab]["agreed"] += 1
            if len(kept[lab]) < quota:
                kept[lab].append(line)
                outcomes.append([key, "kept", line])
            else:
                outcomes.append([key, "surplus", None])
        ck.record(lab, outcomes)
        if prompts % 10 == 0:
            done = sum(len(v) for v in kept.values())
            print("[%s %s] prompts=%d kept=%d/%d %.0fs" % (
                a.suite, a.split, prompts, done, quota * len(labels), time.time() - t0), flush=True)

    rows = [{"text": t, "label": labels.index(lab)} for lab in labels for t in kept[lab]]
    random.Random(a.seed ^ 0x5EED).shuffle(rows)
    for old in out_dir.glob(a.split + "-*.json"):
        old.unlink()
    digests = write_pages(out_dir, a.split, rows, labels)

    man_path = out_dir / "manifest.json"
    man = json.loads(man_path.read_text(encoding="utf-8")) if man_path.exists() else {}
    man["suite"] = a.suite
    man["labels"] = labels
    man["licence"] = "Apache-2.0 (first-party synthetic; generators Apache-2.0)"
    man[a.split] = {
        "rows": len(rows), "per_label": quota, "seed": a.seed, "family": fam_id,
        "generator": {"name": gen.name, "digest": gen.digest},
        "judge": {"name": judge.b.name, "digest": judge.b.digest},
        "prompts": prompts, "temp": a.temp, "k": a.k,
        "veto": {lab: dict(stats[lab], agree_rate=round(
            stats[lab]["agreed"] / max(1, stats[lab]["generated"] - stats[lab]["dupes"]), 4))
            for lab in labels},
        "cross_split_blocked": len(blocked),
        "resumed_at_prompt": resumed_prompts,
        "pages": digests,
        "seconds": round(time.time() - t0, 1),
    }
    man_path.write_text(json.dumps(man, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    ck.finish()
    print("[%s %s] DONE rows=%d prompts=%d %.0fs -> %s" % (
        a.suite, a.split, len(rows), prompts, time.time() - t0, out_dir))


if __name__ == "__main__":
    main()
