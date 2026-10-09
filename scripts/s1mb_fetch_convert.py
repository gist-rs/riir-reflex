#!/usr/bin/env python3
"""S1MB fetch+convert — hotchpotch/s1mb-dataset into the harness suite format.

Plan 010 T1 (the S1MB lane, our-lanes scope — riir-train Issue 607's gate
opened by owner directive 2026-10-02). Fetches every active config's test rows
from the HF datasets-server /rows API (the fetch_datasets.sh pattern: 100-row
pages, offset paging, politeness sleep, resume — a cached valid page is never
refetched), flattens each case x decision into ONE single-question row (the
harness's universal shape; group_id rides the case id), and writes three
suites under .raw/datasets_s1mb/ — s1mb_choice / s1mb_noul / s1mb_score —
each with a deterministic 50/50 train/test split (S1MB is test-only; the train
half is the corpus+cal half every lane shares).

Row shape (what the generic build_s1mb builder consumes):
  {"subset": str, "group": "domain"|"generalization",
   "state": <parsed state_json>,
   "question": {"qid", "kind": "choice"|"score"|"noul",
                "instructions", "criteria": obj|arr|null,
                "keys": [presented order], "gold_idx", "gold_score", "soft"}}

Mapping laws (all measured on the live API, 2026-10-02):
  - gold maps BY ID: the target's argmax id is looked up in the PRESENTED
    criteria order (the two orders are independent — arc showed B,C,A vs
    B,C,A,D). An id absent from the presented keys is a loud skip, never a
    guess.
  - soft probabilities are re-aligned to the presented keys order.
  - score gold_score = the criteria `value` at the gold position.
  - noul keeps type=noul with criteria=null (our fixed [false,true]) and the
    AUTHORED definitions fold into the instructions text (the bekko lane's
    "instruction carries the question" precedent — the definitions carry task
    signal, dropping them would lose it).
  - choice criteria -> {id: parsed description or null} preserving array
    order; score criteria -> the numeric values array.

Protocol disclosures (record + site must repeat):
  - our forced-pick accuracy is NOT the S1MB leaderboard metric (baseline-
    adjusted skill); never mix.
  - the 50/50 corpus split is OUR lanes' protocol, not S1MB's.
  - bekko home-field on the domain subsets; the laya__ subsets are ours.

Usage: python3 scripts/s1mb_fetch_convert.py [--sleep 0.4] [--refetch]
Writes .raw/s1mb_raw/<config>/page-NNN.json (the raw /rows cache) and
.raw/datasets_s1mb/<suite>/{train,test}-000.json + splits.json + the fetch
manifest (blake3 digests, row counts — the dataset_manifest law).
"""

import argparse
import hashlib
import json
import sys
import time
import urllib.request
import urllib.error
from pathlib import Path
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

BASE = "https://datasets-server.huggingface.co"
DATASET = "hotchpotch/s1mb-dataset"
ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / ".raw" / "s1mb_raw"
OUT = ROOT / ".raw" / "datasets_s1mb"
PAGE = 100

GENERALIZATION_PREFIX = "s1mb-generalization-"


def fetch_json(url: str, retries: int = 5) -> dict:
    for attempt in range(retries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "riir-reflex-s1mb-lane"})
            with urllib.request.urlopen(req, timeout=60) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            # The 429 wall (the fetch_datasets.sh law): back off LONG — the
            # limiter cools on minutes, not seconds; resume makes re-runs
            # free for already-cached pages.
            if e.code == 429 and attempt < retries - 1:
                wait = 45.0 * (attempt + 1)
                print(f"  429 — backing off {wait}s (attempt {attempt + 1})", file=sys.stderr)
                time.sleep(wait)
                continue
            if attempt == retries - 1:
                raise
            wait = 5.0 * (attempt + 1)
            print(f"  retry {attempt + 1} after {wait}s: {e}", file=sys.stderr)
            time.sleep(wait)
        except (urllib.error.URLError, json.JSONDecodeError) as e:
            if attempt == retries - 1:
                raise
            wait = 5.0 * (attempt + 1)
            print(f"  retry {attempt + 1} after {wait}s: {e}", file=sys.stderr)
            time.sleep(wait)
    raise RuntimeError("unreachable")


def parse_json_str(v):
    """S1MB carries description_json/instructions_json as JSON-encoded strings."""
    if not isinstance(v, str):
        return None
    try:
        return json.loads(v)
    except json.JSONDecodeError:
        return v


def convert_decision(subset: str, group: str, state, dec, target) -> dict | None:
    """One (case, decision, target) -> one harness row; None = loud skip."""
    kind = dec.get("type")
    criteria = dec.get("criteria")
    instructions = parse_json_str(dec.get("instructions_json")) or ""

    ids = [str(x) for x in target.get("ids", [])]
    probs = target.get("probabilities", [])
    if not ids or len(ids) != len(probs) or not criteria:
        return None
    gold_pos = max(range(len(probs)), key=lambda i: probs[i])
    gold_id = ids[gold_pos]

    if kind == "noul":
        # Authored definitions fold into the instructions (signal, not loss).
        defs = {}
        for c in criteria:
            d = parse_json_str(c.get("description_json"))
            if isinstance(d, str) and d.strip():
                defs[str(c.get("id"))] = d
        folded = instructions
        extra = " / ".join(f"{k}: {defs[k]}" for k in ("false", "true") if k in defs)
        if extra and extra not in folded:
            folded = f"{folded}\nfalse/true definitions: {extra}".strip()
        keys = ["false", "true"]
        if gold_id not in keys:
            return None
        return {
            "qid": str(dec.get("id", "decision")),
            "kind": "noul",
            "instructions": folded,
            "criteria": None,
            "keys": keys,
            "gold_idx": keys.index(gold_id),
            "gold_score": None,
            "soft": [float(probs[ids.index(k)]) if k in ids else 0.0 for k in keys],
        }

    if kind == "choice":
        keys, key_desc = [], {}
        for c in criteria:
            k = str(c.get("id"))
            d = parse_json_str(c.get("description_json"))
            keys.append(k)
            key_desc[k] = d if isinstance(d, str) and d.strip() else None
        if gold_id not in keys:
            return None
        crit_obj = {k: key_desc[k] for k in keys}
        return {
            "qid": str(dec.get("id", "decision")),
            "kind": "choice",
            "instructions": instructions,
            "criteria": crit_obj,
            "keys": keys,
            "gold_idx": keys.index(gold_id),
            "gold_score": None,
            "soft": [float(probs[ids.index(k)]) if k in ids else 0.0 for k in keys],
        }

    if kind == "score":
        keys, values = [], []
        for c in criteria:
            keys.append(str(c.get("id")))
            v = c.get("value")
            values.append(float(v) if isinstance(v, (int, float)) else None)
        if gold_id not in keys:
            return None
        gi = keys.index(gold_id)
        return {
            "qid": str(dec.get("id", "decision")),
            "kind": "score",
            "instructions": instructions,
            "criteria": values,
            "keys": keys,
            "gold_idx": gi,
            "gold_score": values[gi],
            "soft": [float(probs[ids.index(k)]) if k in ids else 0.0 for k in keys],
        }

    return None


def fetch_config(config: str, sleep: float) -> int:
    """Page one config's test split into the raw cache; returns row count."""
    cfg_dir = RAW / config
    cfg_dir.mkdir(parents=True, exist_ok=True)
    offset, total = 0, None
    while True:
        page_path = cfg_dir / f"page-{offset:05d}.json"
        if page_path.is_file() and page_path.stat().st_size > 2:
            data = json.loads(page_path.read_text(encoding="utf-8"))
        else:
            url = (f"{BASE}/rows?dataset={urllib.parse.quote(DATASET, safe='')}"
                   f"&config={urllib.parse.quote(config, safe='')}&split=test"
                   f"&offset={offset}&length={PAGE}")
            data = fetch_json(url)
            if "rows" not in data:
                raise RuntimeError(f"{config}: /rows error: {json.dumps(data)[:200]}")
            page_path.write_text(json.dumps(data), encoding="utf-8")
            time.sleep(sleep)
        rows = data.get("rows", [])
        if total is None:
            total = data.get("num_rows_total", 0)
        offset += len(rows)
        if not rows or offset >= total:
            break
    return offset


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--sleep", type=float, default=0.4)
    ap.add_argument("--refetch", action="store_true")
    args = ap.parse_args()
    if args.refetch:
        import shutil
        shutil.rmtree(RAW, ignore_errors=True)

    print(f"s1mb: splits probe {DATASET}")
    splits = fetch_json(f"{BASE}/splits?dataset={urllib.parse.quote(DATASET, safe='')}")
    configs = sorted({s["config"] for s in splits.get("splits", [])})
    print(f"s1mb: {len(configs)} active configs")

    suites = {"choice": [], "noul": [], "score": []}
    skipped = 0
    for i, config in enumerate(configs):
        n = fetch_config(config, args.sleep)
        group = ("generalization" if config.startswith(GENERALIZATION_PREFIX) else "domain")
        cfg_dir = RAW / config
        n_dec = 0
        for page_path in sorted(cfg_dir.glob("page-*.json")):
            data = json.loads(page_path.read_text(encoding="utf-8"))
            for entry in data.get("rows", []):
                row = entry["row"]
                inp = row["input"]
                state = json.loads(inp["state_json"]) if isinstance(inp["state_json"], str) else inp["state_json"]
                targets = {t["decision_id"]: t for t in row.get("targets", [])}
                input_hash = row.get("input_hash") or hashlib.sha256(
                    json.dumps(inp, sort_keys=True).encode()).hexdigest()
                for dec in inp.get("decisions", []):
                    target = targets.get(dec.get("id"))
                    if target is None:
                        skipped += 1
                        continue
                    n_dec += 1
                    converted = convert_decision(config, group, state, dec, target)
                    if converted is None:
                        skipped += 1
                        continue
                    suite = converted["kind"]
                    suites[suite].append({
                        "subset": config,
                        "group": group,
                        "case_id": f"{config}:{input_hash[:12]}",
                        # The RAW state_json string (the typed_decisions row
                        # shape): the builder puts it in the case verbatim, so
                        # serialize_state returns it as-is and the corpus
                        # text == query state text with zero re-serialization
                        # drift.
                        "state": inp["state_json"],
                        "question": converted,
                    })
        print(f"  [{i + 1}/{len(configs)}] {config}: {n} rows -> {n_dec} decision(s)")

    # Deterministic 50/50 split per suite — STATE-COHARENT (the harness
    # slice-integrity audit, Issue 058, refuses any exact train∩test row):
    # all rows sharing the same state_json land in the SAME half (S1MB
    # carries multiple decisions per case AND byte-identical states across
    # different cases — civil_comments' 7 judgments, snli dups). Exact
    # duplicate rows (same state + qid + gold) dedup first, counted.
    manifest = {"dataset": DATASET, "fetched_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "configs": len(configs), "suites": {}, "skipped_no_target": skipped,
                "deduped_exact": {}}
    for suite, rows in suites.items():
        seen = set()
        deduped = []
        for r in rows:
            q = r["question"]
            key = (r["state"], q["qid"], q["gold_idx"])
            if key in seen:
                continue
            seen.add(key)
            deduped.append(r)
        manifest["deduped_exact"][f"s1mb_{suite}"] = len(rows) - len(deduped)
        rows = deduped
        by_state = {}
        for r in rows:
            by_state.setdefault(hashlib.sha256(r["state"].encode()).hexdigest(), []).append(r)
        train, test = [], []
        for state_hash in sorted(by_state):
            group = sorted(by_state[state_hash], key=lambda r: (r["case_id"], r["question"]["qid"]))
            bucket = train if len(train) <= len(test) else test
            bucket.extend(group)
        # Deterministic + balanced: sort the FINAL halves by (case_id, qid)
        # so page order is stable across runs.
        train.sort(key=lambda r: (r["case_id"], r["question"]["qid"]))
        test.sort(key=lambda r: (r["case_id"], r["question"]["qid"]))
        d = OUT / f"s1mb_{suite}"
        d.mkdir(parents=True, exist_ok=True)
        for split, split_rows in (("train", train), ("test", test)):
            p = d / f"{split}-000.json"
            # The /rows ENVELOPE shape load_rows unwraps (.rows[].row) — the
            # same shape every harness page carries (the fetch_datasets.sh
            # files are raw /rows API responses).
            envelope = {"rows": [{"row": r, "row_idx": i} for i, r in enumerate(split_rows)]}
            p.write_text(json.dumps(envelope, ensure_ascii=False), encoding="utf-8")
            digest = hashlib.blake2b(p.read_bytes(), digest_size=32).hexdigest()
            manifest["suites"][f"s1mb_{suite}"] = manifest["suites"].get(f"s1mb_{suite}", {})
            manifest["suites"][f"s1mb_{suite}"][split] = {
                "rows": len(split_rows), "blake3_sha256_prefix": digest[:16]}
        print(f"s1mb_{suite}: train {len(train)} / test {len(test)}")

    (OUT / "fetch_manifest.json").write_text(
        json.dumps(manifest, indent=1), encoding="utf-8")
    print(f"s1mb: manifest written ({skipped} skipped: no target / unpresented gold)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
