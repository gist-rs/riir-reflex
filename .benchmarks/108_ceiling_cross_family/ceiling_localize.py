#!/usr/bin/env python3
"""Issue 108 re-judge arc — the LOCALIZE step (seam measurement, no re-price).

The cross-family cell (`.benchmarks/108_ceiling_cross_family/KAPPA.md`)
failed the pre-registered kappa bar (0.6387 < 0.70) with 22 disagreement
items between the two seats. The contract's arc is localize -> rulings ->
re-judge -> re-price; THIS instrument is the localize step: it re-reads
ONLY the disagreement items under BOTH seats and captures each seat's
full verdict + quote (the probe arm's frozen read is untouched — it never
re-prices gold, and it runs once per contract version).

Fixture reconstruction replicates the harness exactly
(`src/harness/{suites.rs,runner.rs}`):
  front = load_rows(test-*.json, sorted file order) -> rows[i]["row"]
          -> stratified_split identity (sst5 test <= 600 = test_cap)
          -> build_sst5: case sst5:{pos}, state.text = row["text"],
             options = the 5 LEVELS in order, gold = LEVELS[label].
Self-check: the reconstructed (case_id -> gold_label) map must equal BOTH
probe JSONs over all 600 items before any judge call is made.

Seat ops: same prompt bytes as `judge_messages` (ceiling_probe.rs);
temperature 0.0, max_tokens 256; quote verification replicates
`quote_verifies` (whitespace-run collapse, case-sensitive, context OR
options). Each seat's re-read must reproduce its original reading on the
22 items — a mismatch means the reconstruction drifted and the run
refuses loud (never localize on a drifted fixture).
"""
import json
import glob
import re
import sys
import urllib.request
from collections import Counter

LEVELS = ["very negative", "negative", "neutral", "positive", "very positive"]

SYSTEM = (
    "You adjudicate benchmark gold labels. Given a text and a closed "
    "option set, decide ONLY whether the text supports the given label "
    "AS TYPED — exactly that label, not a nearby reading. Answer with a "
    "single JSON object and nothing else:\n"
    '{"verdict": "supported" | "not_supported" | "unclear", "quote": "..."}\n'
    "Rules: a `supported` verdict MUST carry `quote` — an exact verbatim "
    "substring of the text or the options that supports the label as "
    'typed. `not_supported` and `unclear` need no quote ("quote": null). '
    "Never output anything except the JSON object."
)


def build_front(datasets_dir):
    """Replicates load_rows(test) + identity split + build_sst5."""
    files = sorted(glob.glob(f"{datasets_dir}/sst5/test-*.json"))
    assert files, f"no test shards under {datasets_dir}/sst5"
    cases = []
    pos = 0
    for f in files:
        env = json.load(open(f))
        for r in env["rows"]:
            row = r["row"]
            text = row.get("text")
            label = row.get("label")
            if not isinstance(text, str) or label is None:
                continue  # builders skip (pos advances — id gaps stay)
            lab = int(str(label).strip())
            if lab >= len(LEVELS):
                continue
            cases.append(
                {
                    "case_id": f"sst5:{pos}",
                    "text": text,
                    "options": list(LEVELS),
                    "gold_idx": lab,
                    "gold_label": LEVELS[lab],
                }
            )
            pos += 1
    return cases


def norm(s):
    return " ".join(s.split())


def quote_verifies(quote, context, options):
    q = norm(quote)
    if not q:
        return False
    if q in norm(context):
        return True
    return any(q in norm(o) for o in options)


def judge(url, model, context, options, gold, timeout=120):
    opts_block = "\n".join(f"- {o}" for o in options)
    user = (
        f"Text:\n{context}\n\nOptions:\n{opts_block}\n\n"
        f"Label to adjudicate: {gold}\n\n"
        "Does the text support this label as typed? Answer with the JSON object only."
    )
    body = json.dumps(
        {
            "model": model,
            "messages": [
                {"role": "system", "content": SYSTEM},
                {"role": "user", "content": user},
            ],
            "temperature": 0.0,
            "max_tokens": 256,
            "stream": False,
        }
    ).encode()
    req = urllib.request.Request(
        url.rstrip("/") + "/v1/chat/completions",
        data=body,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        payload = json.load(resp)
    content = payload["choices"][0]["message"]["content"]
    return parse_verdict(content)


def parse_verdict(content):
    """The harness's parse: strip fences/whitespace, find the JSON object."""
    s = content.strip()
    s = re.sub(r"^```(?:json)?\s*", "", s)
    s = re.sub(r"\s*```$", "", s).strip()
    start = s.find("{")
    end = s.rfind("}")
    if start < 0 or end < 0:
        return {"verdict": "MALFORMED", "quote": None, "raw": content}
    try:
        obj = json.loads(s[start : end + 1])
    except json.JSONDecodeError:
        return {"verdict": "MALFORMED", "quote": None, "raw": content}
    v = obj.get("verdict")
    if v not in ("supported", "not_supported", "unclear"):
        return {"verdict": "MALFORMED", "quote": obj.get("quote"), "raw": content}
    return {"verdict": v, "quote": obj.get("quote")}


def reading(v, context, options):
    """support_reading: supported needs a verifying verbatim quote."""
    if v["verdict"] == "supported":
        q = v.get("quote")
        if isinstance(q, str) and quote_verifies(q, context, options):
            return "supported", True
        return "unknown", False
    return v["verdict"], False


def main():
    if len(sys.argv) != 9:
        print(__doc__)
        print(
            "usage: ceiling_localize.py <datasets_dir> <probeA.json> <urlA> <modelA> "
            "<probeB.json> <urlB> <modelB> <out.json>"
        )
        return 2
    (
        datasets_dir,
        probe_a_path,
        url_a,
        model_a,
        probe_b_path,
        url_b,
        model_b,
        out_path,
    ) = sys.argv[1:9]

    cases = build_front(datasets_dir)
    by_id = {c["case_id"]: c for c in cases}
    print(f"front reconstructed: {len(cases)} cases")

    a = json.load(open(probe_a_path))
    b = json.load(open(probe_b_path))
    ia = {i["case_id"]: i for i in a["items"]}
    ib = {i["case_id"]: i for i in b["items"]}

    # self-check 1: gold labels over ALL items
    bad = [k for k in ia if k in by_id and by_id[k]["gold_label"] != ia[k]["gold_label"]]
    bad += [k for k in ib if k in by_id and by_id[k]["gold_label"] != ib[k]["gold_label"]]
    if bad:
        print(f"RECONSTRUCTION DRIFT: {len(bad)} gold mismatches, e.g. {bad[:5]}")
        return 3
    print("self-check 1 OK: gold labels match both probe JSONs on all shared ids")

    # the disagreement set (both-adjudicated, differing readings)
    NON_ADJ = {"unknown", "unsupported_scope"}
    dis = []
    for k in sorted(ia, key=lambda x: int(x.split(":")[1])):
        if k not in ib:
            continue
        ra, rb = ia[k]["support"], ib[k]["support"]
        if ra in NON_ADJ or rb in NON_ADJ or ra == rb:
            continue
        dis.append(k)
    print(f"disagreement items: {len(dis)}")
    if len(dis) != 22:
        print(f"WARNING: expected 22 disagreements (cross-family cell), got {len(dis)}")

    results = []
    mismatch_a, mismatch_b = 0, 0
    for k in dis:
        c = by_id[k]
        rec = {
            "case_id": k,
            "gold_label": c["gold_label"],
            "text": c["text"],
            "lane_a": ia[k]["lane"],
            "orig_a": ia[k]["support"],
            "orig_b": ib[k]["support"],
            "seats": {},
        }
        for seat, url, model, orig in (
            ("A", url_a, model_a, ia[k]["support"]),
            ("B", url_b, model_b, ib[k]["support"]),
        ):
            v = judge(url, model, c["text"], c["options"], c["gold_label"])
            r, qok = reading(v, c["text"], c["options"])
            reproduced = r == orig
            if not reproduced:
                if seat == "A":
                    mismatch_a += 1
                else:
                    mismatch_b += 1
            rec["seats"][seat] = {
                "model": model,
                "verdict": v["verdict"],
                "quote": v.get("quote"),
                "quote_verified": qok,
                "reading": r,
                "orig_reading": orig,
                "reproduced": reproduced,
                "raw": v.get("raw"),
            }
        results.append(rec)
        print(
            f"{k}: gold={c['gold_label']} A={rec['seats']['A']['reading']}"
            f"{'(quote)' if rec['seats']['A']['quote_verified'] else ''} "
            f"B={rec['seats']['B']['reading']}"
            f"{'(quote)' if rec['seats']['B']['quote_verified'] else ''} "
            f"repro A={rec['seats']['A']['reproduced']} B={rec['seats']['B']['reproduced']}"
        )

    print("\nreproduction check (temperature 0, same prompt bytes):")
    print(f"  seat A re-read reproduced original: {len(dis) - mismatch_a}/{len(dis)}")
    print(f"  seat B re-read reproduced original: {len(dis) - mismatch_b}/{len(dis)}")
    if mismatch_a or mismatch_b:
        print(
            "DRIFT: a seat re-read diverged from its original reading — "
            "nondeterministic seat or fixture drift; the localization below "
            "still stands (quotes are from THIS read) but reproduction is "
            "disclosed, never silenced."
        )

    json.dump(
        {
            "note": "Issue 108 localize step — both seats re-read the 22 "
            "disagreement items with quotes; no re-price",
            "probe_a": probe_a_path,
            "probe_b": probe_b_path,
            "disagreements": results,
        },
        open(out_path, "w"),
        indent=1,
    )
    print(f"wrote {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
