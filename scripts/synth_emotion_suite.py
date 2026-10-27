#!/usr/bin/env python3
"""synth_emotion_suite.py -- the first-party synthetic `emotion_syn` suite.

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
  .venv/bin/python scripts/synth_emotion_suite.py --split train --n 4000 \\
      --gen bonsai:<pack_dir> --judge mlxlm:<qwen_dir> --seed 1
  .venv/bin/python scripts/synth_emotion_suite.py --split test --n 1000 \\
      --gen mlxlm:<qwen_dir> --judge bonsai:<pack_dir> --seed 2
"""

import argparse
import hashlib
import json
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
    """First-token logprob argmax over the six label words. Refuses (raises)
    when two labels share a first token -- the read would be ambiguous."""

    def __init__(self, backend):
        self.b = backend
        firsts = []
        for lab in LABELS:
            ids = backend.encode(lab)
            if not ids:
                raise SystemExit("judge: empty encoding for " + lab)
            firsts.append(ids[0])
        if len(set(firsts)) != len(firsts):
            raise SystemExit("judge: label first-tokens collide: %r" % (firsts,))
        self.firsts = firsts

    def read(self, text):
        import mlx.core as mx
        logits = self.b.first_logits(JUDGE_PROMPT.format(text=text))
        sel = mx.take(logits, mx.array(self.firsts))
        mx.eval(sel)
        vals = sel.tolist()
        return LABELS[max(range(len(vals)), key=lambda i: vals[i])]


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

FEATURES = [
    {"feature_idx": 0, "name": "text", "type": {"dtype": "string", "_type": "Value"}},
    {"feature_idx": 1, "name": "label", "type": {"names": LABELS, "_type": "ClassLabel"}},
]


def page_digest(b):
    try:
        import blake3  # type: ignore
        return "blake3:" + blake3.blake3(b).hexdigest()
    except ImportError:
        return "sha256:" + hashlib.sha256(b).hexdigest()


def write_pages(out_dir, split, rows):
    digests = {}
    for p in range(0, len(rows), 100):
        chunk = rows[p:p + 100]
        body = {
            "features": FEATURES,
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


# -- main -------------------------------------------------------------------

def self_test():
    a, b = FAMILY["A"], FAMILY["B"]
    assert not set(a["topics"]) & set(b["topics"]), "topic families overlap"
    assert not set(a["registers"]) & set(b["registers"]), "register families overlap"
    assert clean_lines("1. Here are lines\n- I can't stop smiling at the letter she sent me\nok") == \
        ["I can't stop smiling at the letter she sent me"]
    assert norm("Hi, THERE!") == "hi there"
    print("self-test OK")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--split", choices=["train", "test"])
    ap.add_argument("--n", type=int, default=0, help="target kept rows (balanced over labels)")
    ap.add_argument("--gen", help="bonsai:<pack_dir> | mlxlm:<model_dir>")
    ap.add_argument("--judge", help="the OTHER model (veto)")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--k", type=int, default=8, help="texts asked per prompt")
    ap.add_argument("--temp", type=float, default=0.9)
    ap.add_argument("--out", default=".raw/datasets/emotion_syn")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return
    self_test()
    if not (a.split and a.n and a.gen and a.judge):
        ap.error("--split --n --gen --judge are required")

    import mlx.core as mx
    random.seed(a.seed)
    mx.random.seed(a.seed)
    fam_id = "A" if a.split == "train" else "B"
    fam = FAMILY[fam_id]
    tmpl = PROMPT_A if fam_id == "A" else PROMPT_B
    out_dir = Path(a.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    # Cross-split dedupe: test never repeats a train text.
    blocked = load_existing_norms(out_dir, "train") if a.split == "test" else set()

    gen = make_backend(a.gen)
    judge = Judge(make_backend(a.judge))
    quota = a.n // len(LABELS)
    kept = {lab: [] for lab in LABELS}
    stats = {lab: {"generated": 0, "agreed": 0, "dupes": 0} for lab in LABELS}
    seen = set(blocked)
    t0 = time.time()
    prompts = 0
    while any(len(kept[lab]) < quota for lab in LABELS):
        lab = min(LABELS, key=lambda x: len(kept[x]))
        prompt = tmpl.format(
            k=a.k, register=random.choice(fam["registers"]), person=random.choice(fam["persons"]),
            topic=random.choice(fam["topics"]), intensity=random.choice(INTENSITY),
            gloss=LABEL_GLOSS[lab], lexical=random.choice(LEXICAL))
        lines = clean_lines(gen.generate(prompt, max_tokens=60 * a.k, temp=a.temp))
        prompts += 1
        for line in lines:
            stats[lab]["generated"] += 1
            key = norm(line)
            if key in seen:
                stats[lab]["dupes"] += 1
                continue
            seen.add(key)
            if judge.read(line) != lab:
                continue
            stats[lab]["agreed"] += 1
            if len(kept[lab]) < quota:
                kept[lab].append(line)
        if prompts % 10 == 0:
            done = sum(len(v) for v in kept.values())
            print("[%s] prompts=%d kept=%d/%d %.0fs" % (a.split, prompts, done, quota * 6,
                                                       time.time() - t0), flush=True)

    rows = [{"text": t, "label": LABELS.index(lab)} for lab in LABELS for t in kept[lab]]
    random.Random(a.seed ^ 0x5EED).shuffle(rows)
    for old in out_dir.glob(a.split + "-*.json"):
        old.unlink()
    digests = write_pages(out_dir, a.split, rows)

    man_path = out_dir / "manifest.json"
    man = json.loads(man_path.read_text(encoding="utf-8")) if man_path.exists() else {}
    man["suite"] = "emotion_syn"
    man["labels"] = LABELS
    man["licence"] = "Apache-2.0 (first-party synthetic; generators Apache-2.0)"
    man[a.split] = {
        "rows": len(rows), "per_label": quota, "seed": a.seed, "family": fam_id,
        "generator": {"name": gen.name, "digest": gen.digest},
        "judge": {"name": judge.b.name, "digest": judge.b.digest},
        "prompts": prompts, "temp": a.temp, "k": a.k,
        "veto": {lab: dict(stats[lab], agree_rate=round(
            stats[lab]["agreed"] / max(1, stats[lab]["generated"] - stats[lab]["dupes"]), 4))
            for lab in LABELS},
        "cross_split_blocked": len(blocked),
        "pages": digests,
        "seconds": round(time.time() - t0, 1),
    }
    man_path.write_text(json.dumps(man, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print("[%s] DONE rows=%d prompts=%d %.0fs -> %s" % (a.split, len(rows), prompts,
                                                       time.time() - t0, out_dir))


if __name__ == "__main__":
    main()
