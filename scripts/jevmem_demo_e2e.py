#!/usr/bin/env python3
"""Issue 081 T1: the Jev-Mem offline demo against reflex's /v1/systemone.

Mirrors `jev_mem/demo.py` with the mock DISABLED — the real `typesafe_sdk`
client inside their pipeline points at reflex serve's TypeSafe dialect
route (`TYPESAFE_BASE_URL`), so every JevClient.evaluate() call (admission,
typing, relations, consolidation, routing, stopping, traversal) is answered
by the modelless engine. Zero patches to their tree: the seam is three env
vars + a config.

Posture: this is the SEAM probe (wire correctness end-to-end, the SDK's own
response validation, byte-determinism across runs) — NOT a quality claim.
The demo corpus (ops/support) is semantically unrelated to memory control;
quality cells are T2's corpus work.

Runs UNDER Jev-Mem's venv (needs their modules + typesafe_sdk):
    .raw/jevmem-env/bin/python scripts/jevmem_demo_e2e.py \
        [--serve-url http://127.0.0.1:7331] [--jevmem-dir .raw/jev-mem]

Exit 0 = seam green (both runs completed, decision audits byte-identical).
Any refusal is loud — never a green skip.
"""

import argparse
import json
import os
import re
import shutil
import sys
import tempfile
from datetime import datetime
from pathlib import Path

# The cp874 lesson: reconfigure stdin AND stdout/stderr to UTF-8 with
# backslashreplace — a lane script never dies on a console encoding.
for stream in (sys.stdin, sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    except (AttributeError, OSError):
        pass


def fail(msg: str) -> "None":
    print(f"E2E FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def run_once(jevmem_dir: str, cache_dir: str) -> "tuple":
    # Import lazily under the caller's env (their venv + our env vars).
    # Determinism law v2: the uuid patch lives HERE (per fresh process —
    # see main) so both runs draw the same node-id sequence for the same
    # build sequence; module-init draws differ between a cold and a warm
    # interpreter, which is why each run is its own SUBPROCESS.
    import uuid as _uuid
    import unittest.mock as _mock

    def _seeded_uuid4_factory():
        seq = iter(f"{n:08d}-0000-4000-8000-{n:012d}" for n in range(100_000))

        def _uuid4():
            return _uuid.UUID(next(seq), version=4)

        return _uuid4

    with _mock.patch.object(_uuid, "uuid4", _seeded_uuid4_factory()):
        from memory.jev_mem_config import JevMemConfig
        from memory.memory_builder import MemoryBuilder
        from memory.mock_encoder import MockEncoder
        from memory.query_engine import QueryEngine
        from memory.trg_memory import TemporalResonanceGraphMemory
        from memory.vector_db import NumpyVectorDB

        encoder = MockEncoder()
        trg = TemporalResonanceGraphMemory(
            vector_db=NumpyVectorDB(encoder.dimension), encoder=encoder, llm_backend=None
        )
        config = JevMemConfig.load(
            write_enabled=True,
            read_enabled=True,
            jev_mock=False,
            decision_backend="jev",
            fallback_to_magma=False,  # a backend failure RAISES, never a silent None
            timeout_seconds=5.0,
            max_retries=0,
            audit_path=str(Path(cache_dir) / "decisions.jsonl"),
            anchor_count=1,
        )
        builder = MemoryBuilder(
            cache_dir, jev_config=config, trg_memory=trg, llm_enabled=False
        )
        observations = [
            "Thanks!",
            "Alice started the Jev-Mem project in Dallas.",
            "Alice prefers concise explanations about the Jev-Mem project.",
            "Alice presented the Jev-Mem project results on Friday.",
        ]
        lines = []
        for day, text in enumerate(observations, 1):
            node = builder.build(
                text,
                timestamp=datetime(2026, 9, day),
                metadata={"source": "reflex_e2e", "entities": ["Alice"]},
            )
            lines.append(("STORED: " if node else "REJECTED: ") + text)
        builder.save()
        engine = QueryEngine(trg, builder.node_index, jev_config=config, jev_client=builder.jev)
        context, evidence = engine.query("What does Alice prefer about Jev-Mem explanations?")
        audit = Path(cache_dir) / "decisions.jsonl"
        body = audit.read_text(encoding="utf-8") if audit.exists() else ""
        return lines, evidence, body


def normalize_run_artifact(text: str) -> str:
    """Erase THEIR per-run randomness, never ours: node/memory ids are
    uuid4 by construction (`trg_memory` imports `uuid` and calls it per
    node) and `latency_seconds` is their wall clock — the same two classes
    the agentjev lane documented (a raw byte-compare measures their clock,
    not the decision). Our decision values must survive byte-identical."""
    out = re.sub(
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
        "<uuid>",
        text,
    )
    return re.sub(r'"latency_seconds": [0-9.eE+-]+', '"latency_seconds": <t>', out)


def main() -> "None":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--serve-url", default=os.environ.get("REFLEX_SERVE_URL", "http://127.0.0.1:7331")
    )
    parser.add_argument(
        "--jevmem-dir", default=os.environ.get("JEVMEM_DIR", ".raw/jev-mem")
    )
    parser.add_argument("--single-run", default=None, help=argparse.SUPPRESS)
    parser.add_argument("--single-out", default=None, help=argparse.SUPPRESS)
    args = parser.parse_args()

    if args.single_run and args.single_out:
        # The subprocess half of the v2 determinism law (see main's comment):
        # ONE pipeline run under the seeded uuid patch, results to JSON.
        sys.path.insert(0, str(Path(args.jevmem_dir).resolve()))
        os.environ["TYPESAFE_BASE_URL"] = args.serve_url
        os.environ.setdefault("TYPESAFE_API_KEY", "reflex-local")
        lines, evidence, audit = run_once(args.jevmem_dir, args.single_run)
        Path(args.single_out).write_text(
            json.dumps({"lines": lines, "evidence": str(evidence), "audit": audit}),
            encoding="utf-8",
        )
        return

    jevmem = Path(args.jevmem_dir).resolve()
    if not jevmem.is_dir():
        fail(f"Jev-Mem checkout not found at {jevmem} (clone it at the pinned sha first)")
    sys.path.insert(0, str(jevmem))
    os.environ["TYPESAFE_BASE_URL"] = args.serve_url
    os.environ.setdefault("TYPESAFE_API_KEY", "reflex-local")

    print(f"serve: {args.serve_url}   jev-mem: {jevmem}")
    base = Path(tempfile.mkdtemp(prefix="jevmem_e2e_"))
    try:
        # Determinism law, v2 (the criteria-as-options era): decision values
        # are now sensitive to the FULL ctx bytes — their `uuid4` node ids
        # leak into the values (measured: pair_0_semantic 0.673 vs 0.689
        # across runs on identical content; a logging proxy showed the
        # request bodies differed ONLY in node ids — and the offsets differ
        # between cold and warm interpreters via module-init draws, so
        # in-process seeding cannot align them). Each run is therefore its
        # own SUBPROCESS with the same seeded uuid sequence — identical
        # bytes in, byte-identical decisions out; any remaining diff is OURS.
        import subprocess

        def one_run(tag: str) -> "tuple[list, str, str]":
            cache = base / tag
            out = base / f"{tag}.json"
            cmd = [
                sys.executable, os.path.abspath(__file__),
                "--single-run", str(cache), "--single-out", str(out),
                "--serve-url", args.serve_url, "--jevmem-dir", str(jevmem),
            ]
            subprocess.run(cmd, check=True, env={**os.environ})
            data = json.loads(out.read_text(encoding="utf-8"))
            return data["lines"], data["evidence"], data["audit"]

        lines_a, evidence_a, audit_a = one_run("a")
        lines_b, evidence_b, audit_b = one_run("b")

        for line in lines_a:
            print(line)
        print("evidence:", json.dumps(evidence_a, default=str)[:400])
        n_decisions = audit_a.count("\n")
        print(f"decision events: {n_decisions} (audit lines)")
        if n_decisions == 0:
            fail("no decision events recorded — the backend was never called")

        # The determinism strip (the Bench-127 law): two full pipeline runs
        # must answer byte-identically — their audit records every decision
        # value; any nondeterminism IN THE DECISIONS (ours or theirs) lands
        # here. Their per-run uuid4 node ids are normalized out first.
        na, nb = normalize_run_artifact(audit_a), normalize_run_artifact(audit_b)
        if na != nb:
            la, lb = na.splitlines(), nb.splitlines()
            for i, (x, y) in enumerate(zip(la, lb)):
                if x != y:
                    fail(f"determinism: audit line {i} differs\n  A: {x[:300]}\n  B: {y[:300]}")
            fail(f"determinism: audit line counts differ ({len(la)} vs {len(lb)})")
        if lines_a != lines_b:
            fail("determinism: STORED/REJECTED pattern differs between runs")
        model = ""
        for line in audit_a.splitlines():
            if '"model"' in line:
                model = json.loads(line).get("model", "")
                break
        if model != "reflex-modelless":
            fail(f"answers were served by {model!r}, not reflex-modelless")
        print("E2E PASS: seam green — SDK-validated round trip + byte-determinism strip")
    finally:
        shutil.rmtree(base, ignore_errors=True)


if __name__ == "__main__":
    main()
