#!/usr/bin/env python3
"""Issue 081 T2: LoCoMo decision-layer cells — mock vs reflex arms.

Runs Jev-Mem's OWN pipeline (write path + Jev retrieval controller) over a
LoCoMo sample, once per decision backend arm, and measures the decision-layer
cells from THEIR metadata substrate (`RetrievalController` emits nodes_visited,
edges_examined, jev_calls, retrieval_depth, stop_reason, retrieved_dia_ids;
`TestHarness.retrieval_diagnostics` is the recall formula — exact dia_id set
intersection, no LLM anywhere):

  evidence_recall | nodes_visited | edges_examined | jev_calls | stop rounds

Arms:
  mock   jev_mock=True — their inline `mock_values` heuristics (the F-law
         baseline every real backend must beat).
  reflex decision_backend=jev + TYPESAFE_BASE_URL -> reflex serve's
         /v1/systemone route (the T1 seam). Requires the serve up; refuses
         loud on /healthz failure. The TypeSafe-Jev-API and Laya-local arms
         are deliberately NOT run here (BYO key / heavy install) — the
         summary records them as SKIPPED, never as green.

Determinism (the Bench-127 law): re-run the same arm in a FRESH process (the
graph cache is reused via the config fingerprint; the JevClient LRU cache is
per-process) and `--compare` the per-QA rows byte-identically (counts + sorted
dia lists; latency fields are excluded from rows — wall clock is not a cell).

Licensing: LoCoMo is third-party data kept LOCAL under .raw/ (never committed);
result JSONs carry question INDEXES + hashes only, never corpus text; the
committed bench record carries aggregates only.

Runs UNDER Jev-Mem's venv:
    .raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm mock \
        --dataset .raw/locomo/locomo10.json --sample 0
    .raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm reflex ... (serve must be up)
    .raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --compare A.json B.json
    .raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --summary A.json [B.json ...]

Exit 0 = the arm ran AND its audit proves the intended backend served every
decision (source counts + zero fallback events). Anything else is loud.
"""

import argparse
import hashlib
import json
import os
import re
import sys
import time
import urllib.request
from pathlib import Path

for stream in (sys.stdin, sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    except (AttributeError, OSError):
        pass


def fail(msg: str) -> None:
    print(f"T2 FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def config_fingerprint(cfg) -> str:
    """Their runner's suffix law, minus audit_path (per-invocation, never a
    construction setting — otherwise each invocation would rebuild)."""
    payload = {k: v for k, v in cfg.to_dict().items() if k != "audit_path"}
    return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()[:12]


WRITE_SIDE_FIELDS = (
    # Their validate_reuse_memory law: ONLY these fields shape the constructed
    # graph — retrieval-side settings (thresholds, budgets, top-k) are tuning
    # on an EXISTING graph and must reuse it, never rebuild.
    "write_enabled", "admission_enabled", "jev_mock", "jev_model",
    "decision_schema_version", "decision_backend", "laya_model", "laya_subfolder",
    "relation_threshold", "candidate_top_k", "consolidation_interval", "consolidation_threshold",
)


def write_fingerprint(cfg) -> str:
    """Cache-key fingerprint over the write-side fields only (their reuse law)."""
    d = cfg.to_dict()
    payload = {k: d[k] for k in WRITE_SIDE_FIELDS}
    if d.get("admission_enabled"):
        payload["admission_threshold"] = d["admission_threshold"]
        payload["admission_weights"] = list(d["admission_weights"])
    return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()[:12]


def healthz(serve_url: str) -> dict:
    with urllib.request.urlopen(f"{serve_url}/healthz", timeout=5) as resp:
        return json.loads(resp.read().decode("utf-8"))


def run_arm(args) -> None:
    jevmem = Path(args.jevmem_dir).resolve()
    if not jevmem.is_dir():
        fail(f"Jev-Mem checkout not found at {jevmem}")
    sys.path.insert(0, str(jevmem))
    os.environ.pop("OPENAI_API_KEY", None)  # no LLM anywhere in this lane
    os.environ.setdefault("HF_HOME", str(Path(args.raw_dir) / "hf"))
    if args.arm == "reflex":
        os.environ["TYPESAFE_BASE_URL"] = args.serve_url
        os.environ.setdefault("TYPESAFE_API_KEY", "reflex-local")
        try:
            health = healthz(args.serve_url)
        except Exception as exc:  # noqa: BLE001 - loud refusal, any failure class
            fail(f"reflex arm requires the serve up at {args.serve_url}: {exc}")
        print(f"serve /healthz: {json.dumps(health)[:160]}")

    from memory.jev_mem_config import JevMemConfig
    from memory.memory_builder import MemoryBuilder
    from memory.query_engine import QueryEngine

    overrides = dict(
        write_enabled=True,
        read_enabled=True,
        fallback_to_magma=False,  # a backend failure RAISES, never a silent None
        timeout_seconds=5.0,
        max_retries=0,
        jev_mock=(args.arm == "mock"),
    )
    if args.arm == "reflex":
        overrides["decision_backend"] = "jev"
    if args.max_latency:
        overrides["max_latency_seconds"] = float(args.max_latency)
    if args.stop_threshold is not None:
        overrides["evidence_sufficient_threshold"] = float(args.stop_threshold)
    if args.continue_threshold is not None:
        overrides["continue_threshold"] = float(args.continue_threshold)

    out_dir = Path(args.out_dir)
    pass_dir = out_dir / f"{args.arm}_pass{args.pass_n}"
    pass_dir.mkdir(parents=True, exist_ok=True)
    config = JevMemConfig.load(
        args.jev_config, audit_path=str(pass_dir / "decisions.jsonl"), **overrides
    )
    if args.arm == "mock" and not config.jev_mock:
        fail("mock arm configured but jev_mock is False — check --jev-config")
    fp = config_fingerprint(config)
    cache_dir = Path(args.raw_dir) / "cache" / f"{args.arm}_{write_fingerprint(config)}" / f"sample{args.sample}"
    print(f"arm={args.arm} fingerprint={fp} cache={cache_dir}")

    from jev_mem.datasets.locomo import load_locomo_dataset

    samples = load_locomo_dataset(args.dataset)
    if not 0 <= args.sample < len(samples):
        fail(f"sample {args.sample} out of range (0-{len(samples) - 1})")
    sample = samples[args.sample]
    categories = {int(c) for c in args.categories.split(",")}
    qas = [(i, qa) for i, qa in enumerate(sample.qa) if qa.category in categories]
    if args.max_questions is not None:
        qas = qas[: args.max_questions]
    print(f"sample={args.sample} QAs to run: {len(qas)} (categories {sorted(categories)})")

    builder = MemoryBuilder(
        cache_dir=str(cache_dir), jev_config=config, llm_enabled=False
    )
    graph_path = cache_dir / "graph.json"
    t0 = time.monotonic()
    if graph_path.exists():
        builder.load()
        build_stats = {"cache_hit": True, "build_seconds": time.monotonic() - t0}
        print(f"graph loaded from cache ({build_stats['build_seconds']:.1f}s)")
    else:
        stats = builder.build_memory(sample)
        builder.save()
        build_stats = {"cache_hit": False, "build_seconds": time.monotonic() - t0, **stats}
        print(f"graph built ({build_stats['build_seconds']:.1f}s): events={stats.get('events_created')}")

    engine = QueryEngine(
        builder.trg,
        builder.node_index,
        entity_session_map=getattr(builder, "entity_session_map", None),
        entity_dia_map=getattr(builder, "entity_dia_map", None),
        jev_config=config,
        jev_client=builder.jev,
    )

    rows = []
    t1 = time.monotonic()
    for n, (qa_idx, qa) in enumerate(qas):
        top_k = (
            config.multihop_top_k if qa.category == 1 else config.answer_top_k
        ) if config.read_enabled else 15
        context, _answer_ctx = engine.query(qa.question, top_k)
        meta = context.metadata
        retrieved = sorted({d for d in meta.get("retrieved_dia_ids", []) if d})
        gold = set(qa.evidence or [])
        recall = (len(gold & set(retrieved)) / len(gold)) if gold else None
        rows.append(
            {
                "qa_index": qa_idx,
                "category": qa.category,
                "question_sha16": hashlib.sha256(qa.question.encode()).hexdigest()[:16],
                "gold_n": len(gold),
                "recall": recall,
                "retrieved": retrieved,
                "nodes_visited": meta.get("nodes_visited"),
                "edges_examined": meta.get("edges_examined"),
                "jev_calls": meta.get("jev_calls"),
                "jev_cache_hits": meta.get("jev_cache_hits"),
                "retrieval_depth": meta.get("retrieval_depth"),
                "stop_reason": meta.get("stopping_decision"),
                "fallback_events": len(meta.get("fallback_events", [])),
                "top_k": top_k,
            }
        )
        if (n + 1) % 25 == 0:
            print(f"  queries {n + 1}/{len(qas)}")
    query_seconds = time.monotonic() - t1

    # Arm verification from the audit: every decision must carry this arm's
    # source, and zero fallback events may exist (never a quiet green).
    audit_path = pass_dir / "decisions.jsonl"
    sources: dict = {}
    fallback_events = 0
    models = set()
    audit_lines = 0
    if audit_path.exists():
        for line in audit_path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            audit_lines += 1
            rec = json.loads(line)
            if rec.get("event") == "jev_decision":
                sources[rec.get("source", "?")] = sources.get(rec.get("source", "?"), 0) + 1
                models.add(rec.get("model", ""))
            elif rec.get("event") in ("jev_fallback", "write_fallback"):
                fallback_events += 1
    expected_source = "mock" if config.jev_mock else config.decision_backend
    if audit_lines == 0:
        fail("audit is empty — the decision backend was never called")
    bad_sources = {s: c for s, c in sources.items() if s not in (expected_source, "cache")}
    if bad_sources:
        fail(f"decisions served by the wrong backend: {bad_sources} (expected {expected_source!r})")
    if fallback_events:
        fail(f"{fallback_events} fallback events — the backend failed mid-run (never a quiet green)")
    if args.arm == "reflex" and models and models != {"reflex-modelless"}:
        fail(f"reflex arm served by unexpected models: {models}")

    recalls = [r["recall"] for r in rows if r["recall"] is not None]
    result = {
        "arm": args.arm,
        "pass_n": args.pass_n,
        "sample": args.sample,
        "dataset_sha12": hashlib.sha256(Path(args.dataset).read_bytes()).hexdigest()[:12],
        "config": {k: v for k, v in config.to_dict().items() if k != "audit_path"},
        "config_fingerprint": fp,
        "build": build_stats,
        "query_seconds": query_seconds,
        "n_queries": len(rows),
        "audit": {"lines": audit_lines, "sources": sources, "models": sorted(models)},
        "rows": rows,
        "skipped_arms": {
            "typesafe_jev_api": "BYO TYPESAFE_API_KEY — not run (spend-gated)",
            "laya_local": "their laya pip package + weights — not run (heavy install; box loaded)",
        },
        "aggregate": {
            "mean_recall": (sum(recalls) / len(recalls)) if recalls else None,
            "by_category": {
                str(cat): {
                    "n": len([r for r in rows if r["category"] == cat]),
                    "mean_recall": (
                        sum(r["recall"] for r in rows if r["category"] == cat and r["recall"] is not None)
                        / max(1, len([r for r in rows if r["category"] == cat and r["recall"] is not None]))
                    ),
                }
                for cat in sorted(categories & {r["category"] for r in rows})
            },
            "mean_nodes_visited": sum(r["nodes_visited"] or 0 for r in rows) / len(rows),
            "mean_edges_examined": sum(r["edges_examined"] or 0 for r in rows) / len(rows),
            "mean_jev_calls": sum(r["jev_calls"] or 0 for r in rows) / len(rows),
            "stop_reasons": {
                reason: len([r for r in rows if r["stop_reason"] == reason])
                for reason in sorted({r["stop_reason"] for r in rows})
            },
        },
    }
    out_file = pass_dir / "results.json"
    out_file.write_text(json.dumps(result, indent=1, sort_keys=True), encoding="utf-8")
    print(f"mean_recall={result['aggregate']['mean_recall']!r} rows={len(rows)}")
    print(f"stop_reasons={result['aggregate']['stop_reasons']}")
    print(f"audit sources={sources}")
    print(f"wrote {out_file}")


def strip_volatile(row: dict) -> dict:
    out = dict(row)
    out["retrieved"] = sorted(row.get("retrieved", []))
    return out


def compare(args) -> None:
    a = json.loads(Path(args.results_a).read_text(encoding="utf-8"))
    b = json.loads(Path(args.results_b).read_text(encoding="utf-8"))
    if a["arm"] != b["arm"]:
        fail(f"compare is per-arm: {a['arm']} vs {b['arm']}")
    ra, rb = [strip_volatile(r) for r in a["rows"]], [strip_volatile(r) for r in b["rows"]]
    if len(ra) != len(rb):
        fail(f"row counts differ: {len(ra)} vs {len(rb)}")
    for i, (x, y) in enumerate(zip(ra, rb)):
        if x != y:
            fail(f"determinism: row {i} differs\n  A: {json.dumps(x, sort_keys=True)[:400]}\n  B: {json.dumps(y, sort_keys=True)[:400]}")
    print(f"DETERMINISM PASS: {len(ra)} per-QA rows byte-identical ({a['arm']} arm, passes {a['pass_n']} vs {b['pass_n']})")


def summary(args) -> None:
    files = [Path(p) for p in args.results_files]
    for p in files:
        if not p.exists():
            fail(f"missing results file: {p}")
    print("| arm | pass | n | mean_recall | cat1 | cat2 | cat3 | cat4 | nodes | edges | jev_calls | stop_reasons |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for p in files:
        r = json.loads(p.read_text(encoding="utf-8"))
        agg = r["aggregate"]
        cats = agg["by_category"]
        cat_cells = " | ".join(
            f"{cats[str(c)]['mean_recall']:.3f}" if cats.get(str(c)) and cats[str(c)]["mean_recall"] is not None else "-"
            for c in (1, 2, 3, 4)
        )
        stops = ", ".join(f"{k}:{v}" for k, v in agg["stop_reasons"].items())
        print(
            f"| {r['arm']} | {r['pass_n']} | {r['n_queries']} | {agg['mean_recall']:.4f} | {cat_cells} "
            f"| {agg['mean_nodes_visited']:.1f} | {agg['mean_edges_examined']:.1f} | {agg['mean_jev_calls']:.1f} | {stops} |"
        )
        sk = r.get("skipped_arms", {})
        for name, why in sk.items():
            print(f"| {name} | SKIP | - | - | - | - | - | - | - | - | - | {why} |")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--arm", choices=["mock", "reflex"])
    parser.add_argument("--pass-n", type=int, default=1)
    parser.add_argument("--dataset", default=".raw/locomo/locomo10.json")
    parser.add_argument("--sample", type=int, default=0)
    parser.add_argument("--categories", default="1,2,3,4")
    parser.add_argument("--max-questions", type=int, default=None)
    parser.add_argument("--serve-url", default=os.environ.get("REFLEX_SERVE_URL", "http://127.0.0.1:7331"))
    parser.add_argument("--jevmem-dir", default=os.environ.get("JEVMEM_DIR", ".raw/jev-mem"))
    parser.add_argument("--jev-config", default=None)
    parser.add_argument("--raw-dir", default=".raw/locomo")
    parser.add_argument("--out-dir", default=".raw/locomo/results")
    parser.add_argument("--max-latency", type=float, default=None,
                        help="override max_latency_seconds for BOTH arms (load-shielding; disclosed)")
    parser.add_argument("--stop-threshold", type=float, default=None,
                        help="override evidence_sufficient_threshold (T3 stopping sweep; graph reused, never rebuilt)")
    parser.add_argument("--continue-threshold", type=float, default=None,
                        help="override continue_threshold (T3 stopping sweep; graph reused, never rebuilt)")
    parser.add_argument("--compare", nargs=2, metavar=("A.json", "B.json"), dest="compare_pair")
    parser.add_argument("--summary", nargs="+", metavar="results.json", dest="results_files")
    args = parser.parse_args()

    if args.compare_pair:
        compare(argparse.Namespace(results_a=args.compare_pair[0], results_b=args.compare_pair[1]))
        return
    if args.results_files:
        summary(argparse.Namespace(results_files=args.results_files))
        return
    if not args.arm:
        fail("choose --arm, or --compare / --summary")
    run_arm(args)


if __name__ == "__main__":
    main()
