#!/usr/bin/env python3
"""Paraphrase the TEST shards of massive_intent_en + banking77 (rethink Issue 057,
the ESC contamination shadow read).

- Paraphraser: gemma-3-4b-it Q4_K_M via llama-server (127.0.0.1:8097), greedy
  (temperature 0). Independent lineage from the mammoth-1 teacher (a Qwen3.5
  finetune). Residual: gemma-3 may itself have seen the public benchmarks;
  disclosed, unquantifiable. The trigram-overlap gate controls the
  surface-persistence channel.
- Only row.text inside test-*.json changes; row order, labels, train shards
  and splits.json are byte-preserved, so the harness's label-stratified test
  selection picks the SAME cases with paraphrased surfaces, and the seat's
  cal slice (train-derived) + corpus stay at the production posture.
- Per-item gates: non-empty; not identical (case-insensitive) to the source;
  char-length ratio in [0.4, 2.5]; word-trigram overlap <= 0.65. Up to three
  attempts (greedy base prompt, greedy stronger prompt, temp 0.7); a case
  that still fails keeps the ORIGINAL text and is flagged unchanged.
"""
import concurrent.futures as cf
import json
import re
import shutil
import sys
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SRC = REPO / ".raw" / "datasets_t20k"
DST = REPO / ".raw" / "datasets_t20k_para"
MANIFEST_DIR = REPO / ".raw" / "para-model"
SERVER = "http://127.0.0.1:8097"
SUITES = ["massive_intent_en", "banking77"]
SYSTEM = (
    "You paraphrase single utterances. Rewrite with different wording and "
    "sentence structure while preserving the exact same meaning and intent. "
    "Reply with ONLY the rewritten utterance - no quotes, no preamble, no "
    "explanation."
)
MAX_ATTEMPTS = 3
MAX_TOKENS = 120


def chat(text, stronger, temp):
    user = "Paraphrase this utterance: " + text
    if stronger:
        user = ("Paraphrase this utterance using completely different words "
                "where possible: " + text)
    body = json.dumps({
        "model": "gemma-3-4b-it",
        "temperature": temp,
        "max_tokens": MAX_TOKENS,
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": user},
        ],
    }).encode()
    req = urllib.request.Request(
        SERVER + "/v1/chat/completions",
        data=body,
        headers={"Content-Type": "application/json"},
    )
    last = None
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                out = json.loads(r.read().decode("utf-8"))["choices"][0]["message"]["content"]
            return out.strip().strip('"').strip()
        except Exception as e:
            last = e
            time.sleep(2 * (attempt + 1))
    raise RuntimeError("chat failed after retries: %s" % last)


def tokens(s):
    return re.findall(r"[a-z0-9']+", s.lower())


def trigram_overlap(a, b):
    ta, tb = tokens(a), tokens(b)
    if len(ta) < 3 or len(tb) < 3:
        return 1.0 if ta == tb else 0.0
    ga = set(tuple(ta[i:i + 3]) for i in range(len(ta) - 2))
    gb = set(tuple(tb[i:i + 3]) for i in range(len(tb) - 2))
    if not ga or not gb:
        return 0.0
    return len(ga & gb) / min(len(ga), len(gb))


def paraphrase_one(orig):
    flags, attempts = [], 0
    base = orig.lower()

    def ratio_ok(p):
        return 0.4 <= len(p) / max(len(orig), 1) <= 2.5

    for attempt in range(MAX_ATTEMPTS):
        attempts += 1
        stronger = attempt == 1
        temp = 0.0 if attempt < 2 else 0.7
        para = chat(orig, stronger, temp)
        if not para:
            continue
        if para.lower() == base:
            continue
        if not ratio_ok(para):
            flags.append("len_ratio")
            continue
        ov = trigram_overlap(orig, para)
        if ov > 0.65:
            flags.append("high_overlap:%.2f" % ov)
            if attempt == MAX_ATTEMPTS - 1:
                return para, flags, attempts
            continue
        return para, flags, attempts
    return orig, flags + ["unchanged"], attempts


def main():
    t0 = time.time()
    for suite in SUITES:
        src_dir, dst_dir = SRC / suite, DST / suite
        dst_dir.mkdir(parents=True, exist_ok=True)
        test_files = sorted(src_dir.glob("test-*.json"))
        pages = []
        for f in test_files:
            pages.append((f, json.loads(f.read_text(encoding="utf-8"))))
        n_rows = sum(len(p["rows"]) for _, p in pages)
        print("[%s] %d test file(s), %d row(s)" % (suite, len(test_files), n_rows), flush=True)

        flat = []
        for fi, (f, page) in enumerate(pages):
            for ri, wrapper in enumerate(page["rows"]):
                flat.append((fi, ri, wrapper["row"]["text"]))

        done = 0
        results = {}
        with cf.ThreadPoolExecutor(max_workers=8) as ex:
            futs = {
                ex.submit(paraphrase_one, t): (fi, ri)
                for fi, ri, t in flat
            }
            for fut in cf.as_completed(futs):
                fi, ri = futs[fut]
                results[(fi, ri)] = fut.result()
                done += 1
                if done % 250 == 0:
                    print("[%s] %d/%d (%.0fs)" % (suite, done, n_rows, time.time() - t0), flush=True)

        mpath = MANIFEST_DIR / ("para_manifest_%s.jsonl" % suite)
        unchanged = high_overlap = flagged_len = 0
        with mpath.open("w", encoding="utf-8") as mf:
            for fi, (f, page) in enumerate(pages):
                for ri, wrapper in enumerate(page["rows"]):
                    orig = wrapper["row"]["text"]
                    para, flags, attempts = results[(fi, ri)]
                    if "unchanged" in flags:
                        unchanged += 1
                    if any(x.startswith("high_overlap") for x in flags):
                        high_overlap += 1
                    if "len_ratio" in flags:
                        flagged_len += 1
                    if para != orig:
                        wrapper["row"]["text"] = para
                    mf.write(json.dumps({
                        "file": f.name,
                        "row_idx": wrapper["row_idx"],
                        "orig": orig,
                        "para": para,
                        "flags": flags,
                        "attempts": attempts,
                    }, ensure_ascii=False) + "\n")
                (dst_dir / f.name).write_text(
                    json.dumps(page, ensure_ascii=False), encoding="utf-8"
                )
            mf.write(json.dumps({
                "summary": suite,
                "rows": n_rows,
                "unchanged_kept_original": unchanged,
                "high_overlap_accepted": high_overlap,
                "len_ratio_flagged": flagged_len,
            }) + "\n")
        print(
            "[%s] written: unchanged=%d high_overlap=%d len_ratio=%d -> %s"
            % (suite, unchanged, high_overlap, flagged_len, dst_dir),
            flush=True,
        )

    for suite in SUITES:
        src_dir, dst_dir = SRC / suite, DST / suite
        for f in sorted(src_dir.iterdir()):
            if f.name.startswith("test-"):
                continue
            target = dst_dir / f.name
            if not target.exists():
                shutil.copy2(f, target)
        print("[%s] non-test files copied verbatim" % suite, flush=True)
    print("ALL DONE in %.0fs" % (time.time() - t0), flush=True)


if __name__ == "__main__":
    sys.exit(main())
