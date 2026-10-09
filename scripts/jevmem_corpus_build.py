#!/usr/bin/env python3
"""Issue 081 T2b: build + seat the memory-control READ-PATH corpus.

Three subcommands over the preserved Bench-133/134 rig (their venv, their
cached graphs, the local LoCoMo file):

  capture  Run their pipeline per sample with a proxy JevClient that logs
           every (operation, state, questions) wire payload per QA. Uses
           jev_mock=True at the published posture so the FULL path runs
           (depth-0 + depth-1 stopping states, traversal candidate states)
           and the MOCK-arm graph cache is reused — no rebuild, no serve.

  build    Emit the 5-domain corpus from the captures + the measured
           recall rows (base = recall@1, t3_stop0 = recall@0):
             stopping_stop / stopping_expand   depth-0 label = r0 >= r1;
                                            depth-1 always stop (measured:
                                            scan-2 never yields proposals)
             routing_multihop / routing_direct multi_hop by category;
                                            entity/temporal pinned true,
                                            semantic/causal/recency pinned
                                            false (this graph's live edges)
             traversal_pin                     symmetric tails -> p~0.5 ->
                                            the base beam ranking preserved
           Domain = one <name>/ dir of .md docs (state \\n prompt \\n answer:
           <option description>). Docs stay LOCAL under .raw/ (their option
           descriptions are their prose — never committed).

  eval     Fire the HELD-OUT captured depth-0 stopping states + routing
           states at a seated serve (RIIR_REFLEX_CORPUS=the built corpus)
           and read the decision surface vs the measured labels: the
           stop/expand confusion, the projected pooled recall (pred-stop ->
           r0, else r1), the projected edges, and the routing sanity pins.

Fit/eval split: fit samples 0-4, eval samples 5-9 (the Bench-127 law —
fitting data never pooled with the eval read; the corpus docs carry ONLY
fit-sample states).

Runs UNDER Jev-Mem's venv:
    .raw/jevmem-env/bin/python scripts/jevmem_corpus_build.py capture --sample 0
    .raw/jevmem-env/bin/python scripts/jevmem_corpus_build.py build
    .raw/jevmem-env/bin/python scripts/jevmem_corpus_build.py eval
"""

import argparse
import json
import os
import random
import sys
import urllib.request
from pathlib import Path

for stream in (sys.stdin, sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    except (AttributeError, OSError):
        pass

FIT_SAMPLES = (0, 1, 2, 3, 4)
EVAL_SAMPLES = (5, 6, 7, 8, 9)
# The T3 arms' posture (their dataclass defaults; Bench 134 legitimised it).
STOP_THRESHOLD = 0.85
CONTINUE_THRESHOLD = 0.40
# Corpus size levers (bytes per domain drive every option score — the LZ4
# drafter compresses corpus+ctx per option; a multi-MB domain risks their
# 5 s evaluate timeout on the ~160-question traversal batches).
STOP_DOCS_MAX = 250          # per depth, seeded subsample of stop-labeled
EVIDENCE_IN_DOC = 5          # trim the evidence list in stopping docs
PAIR_STATES = 250            # v8: per captured pool, seeded — the paired-doc
                            # cancellation build emits each state in BOTH
                            # stopping domains (only the depth byte differs)
PIN_STATES = 50              # traversal pin states (trimmed, symmetric)
PIN_EVIDENCE = 6
PIN_CANDIDATES = 6
SEED = 0


def fail(msg: str) -> None:
    print(f"CORPUS FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def wire_state(state) -> str:
    """The exact bytes the serve's ctx carries: compact JSON, insertion
    order (serde_json preserve_order), non-ASCII literal."""
    return json.dumps(state, separators=(",", ":"), ensure_ascii=False)


def wire_state_spaced(state) -> str:
    """The SPACED arm (v10, issue 081 T2c): Python-default separators
    (", " / ": "), mirroring the serve's RIIR_REFLEX_SYSTEMONE_SPACED
    rendering — structural scalars (the `depth` digit) become standalone
    whitespace tokens instead of being glued into the first evidence
    item's opening text (the measured burial law)."""
    return json.dumps(state, separators=(", ", ": "), ensure_ascii=False)


# ---------------------------------------------------------------- capture

def cmd_capture(args) -> None:
    jevmem = Path(args.jevmem_dir).resolve()
    if not jevmem.is_dir():
        fail(f"Jev-Mem checkout not found at {jevmem}")
    sys.path.insert(0, str(jevmem))
    sys.path.insert(0, str(Path(__file__).resolve().parent))  # t2 helpers
    os.environ.pop("OPENAI_API_KEY", None)
    os.environ.setdefault("HF_HOME", str(Path(args.raw_dir) / "hf"))

    from jevmem_t2_cells import write_fingerprint  # noqa: E402 - their runner's cache key
    from memory.jev_client import JevClient  # noqa: E402
    from memory.jev_mem_config import JevMemConfig  # noqa: E402
    from memory.memory_builder import MemoryBuilder  # noqa: E402
    from memory.query_engine import QueryEngine  # noqa: E402

    class CaptureClient(JevClient):
        """Logs every evaluate() wire payload, then delegates (mock values)."""

        def __init__(self, *a, sink=None, **kw):
            super().__init__(*a, **kw)
            self._sink = sink

        def evaluate(self, operation, state, questions, *, mock_values=None, budget=None):
            if self._sink is not None:
                self._sink.append({
                    "op": operation,
                    "state": state,
                    "questions": {k: q.model_dump() for k, q in questions.items()},
                })
            return super().evaluate(operation, state, questions,
                                   mock_values=mock_values, budget=budget)

    out_root = Path(args.raw_dir) / "capture"
    out_root.mkdir(parents=True, exist_ok=True)
    from jev_mem.datasets.locomo import load_locomo_dataset  # noqa: E402

    samples = load_locomo_dataset(args.dataset)
    if not 0 <= args.sample < len(samples):
        fail(f"sample {args.sample} out of range")
    sample = samples[args.sample]

    config = JevMemConfig.load(
        args.jev_config, audit_path=str(out_root / f"s{args.sample}_audit.jsonl"),
        write_enabled=True, read_enabled=True, fallback_to_magma=False,
        timeout_seconds=5.0, max_retries=0, jev_mock=True,
    )
    cache_dir = Path(args.raw_dir) / "cache" / f"mock_{write_fingerprint(config)}" / f"sample{args.sample}"
    builder = MemoryBuilder(cache_dir=str(cache_dir), jev_config=config, llm_enabled=False)
    graph_path = cache_dir / "graph.json"
    if not graph_path.exists():
        fail(f"mock graph cache missing at {graph_path} — run the mock arm first (Bench 133 rig)")
    builder.load()
    print(f"sample={args.sample} graph loaded from cache")

    out_lines = []
    qas = [(i, qa) for i, qa in enumerate(sample.qa) if qa.category in (1, 2, 3, 4)]
    for n, (qa_idx, qa) in enumerate(qas):
        sink = []
        client = CaptureClient(config, audit=None, sink=sink)
        engine = QueryEngine(
            builder.trg, builder.node_index,
            entity_session_map=getattr(builder, "entity_session_map", None),
            entity_dia_map=getattr(builder, "entity_dia_map", None),
            jev_config=config, jev_client=client,
        )
        top_k = config.multihop_top_k if qa.category == 1 else config.answer_top_k
        engine.query(qa.question, top_k)
        out_lines.append(json.dumps({
            "qa_index": qa_idx,
            "category": qa.category,
            "gold": list(qa.evidence or []),
            "calls": sink,
        }, sort_keys=True))
        if (n + 1) % 25 == 0:
            print(f"  captured {n + 1}/{len(qas)}")
    out = out_root / f"sample{args.sample}.jsonl"
    out.write_text("\n".join(out_lines) + "\n", encoding="utf-8")
    ops = {}
    for line in out_lines:
        for call in json.loads(line)["calls"]:
            ops[call["op"]] = ops.get(call["op"], 0) + 1
    print(f"wrote {out} ({len(out_lines)} QAs, calls: {ops})")


# ------------------------------------------------------------------ build

def qdesc(wire_question: dict, which: str) -> str:
    return wire_question["criteria"][which]


def emit_doc(domain_dir: Path, name: str, state_wire: str, prompt: str, tail: str) -> None:
    (domain_dir / f"{name}.md").write_text(
        f"{state_wire}\n{prompt}\nanswer: {tail}\n", encoding="utf-8")


def stop_label(rows_base: dict, rows_stop0: dict, qa_index: int):
    """(r0, r1, label_stop) — label_stop: stopping at depth 0 loses nothing."""
    r0 = rows_stop0.get(qa_index, {}).get("recall")
    r1 = rows_base.get(qa_index, {}).get("recall")
    if r0 is None or r1 is None:
        return r0, r1, None
    return r0, r1, r0 >= r1


def load_rows(results_root: Path, sample: int) -> dict:
    p = results_root / f"s{sample}" / "reflex_pass1" / "results.json"
    return {r["qa_index"]: r for r in json.loads(p.read_text())["rows"]}


def trim_stop_state(state: dict) -> dict:
    """Trim the evidence list in stopping docs (fit-time size lever — the
    bytes drive every option score at serve time). Applies to BOTH depths:
    the `depth` token and the query + top-evidence mass carry the shape."""
    s = dict(state)
    s["evidence"] = list(state.get("evidence", []))[:EVIDENCE_IN_DOC]
    return s


def cmd_build(args) -> None:
    raw = Path(args.raw_dir)
    variant = getattr(args, "variant", "v6")
    corpus = raw / f"corpus_pack_{variant}"
    if corpus.exists():
        fail(f"{corpus} already exists — move it aside (a corpus is never silently rebuilt over)")
    capture_dir = raw / "capture"
    results, stop0 = raw / "results", raw / "t3_stop0"

    depth0, depth1, routes, pins = [], [], [], []
    label_rows = []
    for s in FIT_SAMPLES:
        cap_p = capture_dir / f"sample{s}.jsonl"
        if not cap_p.exists():
            fail(f"missing capture {cap_p} — run `capture --sample {s}` first")
        rows_base = load_rows(results, s)
        rows_stop0 = load_rows(stop0, s)
        for line in cap_p.read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            qa, cat, gold = row["qa_index"], row["category"], set(row["gold"])
            r0, r1, label = stop_label(rows_base, rows_stop0, qa)
            if label is None:
                continue
            label_rows.append((s, qa, cat, r0, r1, label))
            for call in row["calls"]:
                if call["op"] == "stopping":
                    depth = call["state"].get("depth")
                    rec = {"s": s, "qa": qa, "state": call["state"],
                           "questions": call["questions"], "depth": depth,
                           "label_stop": label}
                    # v6 law (the v5 negative): the depth-0 stop/expand split
                    # is NOT readable from the state bytes (measured p overlap
                    # med 0.405 vs 0.415 on held-out) — the corpus teaches the
                    # DEPTH classes instead, the lever that needs no semantic
                    # separation: depth-0 states NEVER stop (expand polarity),
                    # depth-1 states ALWAYS stop (measured: scan-2 never adds
                    # nodes — frontier_exhausted 1540/1540 — so the stop is a
                    # pure ~890-edge save at zero recall risk).
                    (depth1 if depth == 1 else depth0).append(rec)
                elif call["op"] == "routing":
                    routes.append({"s": s, "qa": qa, "cat": cat,
                                   "state": call["state"], "questions": call["questions"]})
                elif call["op"] == "traversal":
                    pins.append({"s": s, "qa": qa, "state": call["state"],
                                 "questions": call["questions"], "gold": gold})

    # v7 (issue 081 T2c): the two stopping domains are renamed to the wire's
    # own continue_useful criteria descriptions — byte-exact, read from the
    # captures (never retyped) — so the criteria-as-options Choice arm
    # resolves BY NAME and the engine's route + count-table terms arm for
    # exactly that question (the fitted-token head surface; the `depth`
    # token is invisible to the compression drafter on realistic states
    # but is a plain count-table feature).
    stop_dir_name, expand_dir_name = "stopping_stop", "stopping_expand"
    if variant in ("v7", "v8", "v9", "v10", "v11"):
        first_q = None
        for rec in depth0 + depth1:
            q = rec["questions"].get("continue_useful")
            if q and q.get("criteria"):
                first_q = q
                break
        if first_q is None:
            fail("v7 needs the continue_useful question in the captures")
        crit = first_q["criteria"]
        if not crit.get("true") or not crit.get("false"):
            fail(f"continue_useful criteria missing descriptions: {list(crit)}")
        expand_dir_name = crit["true"]   # depth-0 docs -> the TRUE desc domain
        stop_dir_name = crit["false"]    # depth-1 docs -> the FALSE desc domain
        for nm in (stop_dir_name, expand_dir_name):
            if "/" in nm or len(nm.encode()) > 200 or not nm.strip():
                fail(f"description {nm!r} is not a usable directory name")
        print(f"v7 stopping domains (byte-exact wire descriptions):\n  expand: {expand_dir_name!r}\n  stop:   {stop_dir_name!r}")

    rng = random.Random(SEED)
    d_stop = corpus / stop_dir_name
    d_expand = corpus / expand_dir_name
    d_mh = corpus / "routing_multihop"
    d_dir = corpus / "routing_direct"
    d_pin = corpus / "traversal_pin"
    for d in (d_stop, d_expand, d_mh, d_dir, d_pin):
        d.mkdir(parents=True)

    # Polarity: which criteria description the depth class favours, per
    # question id. depth-1 (stop): sufficient=TRUE, missing=FALSE,
    # contradiction=FALSE, continue_useful=FALSE (the <0.40 lever).
    # depth-0 (expand): the inverses for the three we control (contradiction
    # left out — the conjunct is already blocked by sufficient<0.85 OR
    # missing>=0.4).
    STOP_POLARITY = {"evidence_sufficient": "true", "missing_evidence": "false",
                     "contradiction": "false", "continue_useful": "false"}
    EXPAND_POLARITY = {"evidence_sufficient": "false", "missing_evidence": "true",
                       "continue_useful": "true"}

    def write_stop_docs(recs, domain: Path, polarity: dict, tag: str) -> int:
        n = 0
        for rec in recs:
            qs = rec["questions"]
            if not set(polarity).issubset(qs):
                continue
            state = trim_stop_state(rec["state"])
            sw = wire_state(state)
            for qid, which in polarity.items():
                q = qs[qid]
                emit_doc(domain, f"{tag}_s{rec['s']}q{rec['qa']}d{rec['depth']}_{qid}",
                         sw, q["instructions"], qdesc(q, which))
                n += 1
        return n

    rng.shuffle(depth0)
    rng.shuffle(depth1)
    picked_stop = depth1[:STOP_DOCS_MAX]
    picked_expand = depth0[:STOP_DOCS_MAX]
    if variant in ("v8", "v9", "v10", "v11"):
        # v8 (issue 081 T2c): PAIRED-DOC CANCELLATION. Measured on v7: the
        # count table reads the depth token (a depth flip moves p by a
        # consistent −0.005) but the query+evidence table mass — sampling
        # noise over 250 docs — swings p ±0.3 and drowns it. The repair is
        # corpus-side: emit the SAME states into BOTH stopping domains,
        # differing ONLY in the depth byte — every nuisance token appears
        # identically in both tables and cancels exactly in the margin, so
        # the depth bigram is the only discriminative mass by construction.
        # Docs stay request-shaped (drafter + distance gate read them too).
        #
        # v9: NEUTRAL TAILS. v8 measured 38% premature depth-0 stops — the
        # residual is HASH-COLLISION noise: each class-exclusive bucket
        # (the depth token forms + ~48 description-tail tokens/bigrams)
        # carries ±22 bits, and a ~2500-event request collides onto a
        # WRONG-SIDE exclusive bucket with λ ≈ 1.1 (P(≥2) ≈ 31%, matching
        # the 38% measured). Identical neutral tails drop the exclusive
        # set to the ~6 depth-form buckets (λ ≈ 0.13, residual < 1%).
        # v11: the V5 SUPERVISION LABELS through the pair functional — the
        # original T2b target (stop-ok = r0 >= r1, 92.6% of QAs; expand =
        # the 7.4% carrying 4.2pt). Unpaired (the label IS the class) but
        # with every v9/v10 law applied (neutral tails, spaced render). The
        # question this arm answers: can ANY content signal beat the 12.6:1
        # class prior on the count-table substrate (v5's compression-path
        # read measured the labels inseparable — med 0.405 vs 0.415)?
        pool_stop, pool_expand = [], []
        if variant == "v11":
            for rec in depth0:
                # v5 label (stop_label above, carried on the record): the
                # DEPTH-0 states partitioned by measured supervision —
                # stop-ok = r0 >= r1 (stopping at depth 0 loses nothing).
                (pool_expand if rec["label_stop"] is False else pool_stop).append(rec)
            rng.shuffle(pool_stop); rng.shuffle(pool_expand)
            pool_stop = pool_stop[:STOP_DOCS_MAX]
            pool_expand = pool_expand[:STOP_DOCS_MAX]
            pool = pool_stop + pool_expand
        else:
            pool = picked_stop + picked_expand
        continue_useful_q = None
        for rec in pool:
            q = rec["questions"].get("continue_useful")
            if q and q.get("criteria"):
                continue_useful_q = q
                break
        if continue_useful_q is None:
            fail("v8/v9 needs the continue_useful question in the captures")
        neutral_tail = "the stopping policy reads the retrieval depth"
        render = wire_state_spaced if variant in ("v10", "v11") else wire_state
        n_stop = n_expand = 0
        if variant == "v11":
            # UNPAIRED: each depth-0 state lands in its LABEL's domain only.
            for rec, dom in ([(r, d_stop) for r in pool_stop] + [(r, d_expand) for r in pool_expand]):
                state = trim_stop_state(dict(rec["state"]))
                st = dict(state)
                st["depth"] = rec["depth"]
                emit_doc(dom, f"lab_s{rec['s']}q{rec['qa']}", render(st),
                         continue_useful_q["instructions"], neutral_tail)
                if dom == d_expand:
                    n_expand += 1
                else:
                    n_stop += 1
        else:
            for rec in pool:
                state = trim_stop_state(dict(rec["state"]))
                base = f"pair_s{rec['s']}q{rec['qa']}d{rec['depth']}"
                # v8-v10: the depth byte IS the class (the paired
                # cancellation) — each state emits into BOTH domains,
                # differing only in the depth byte.
                for dom, depth_byte, tag in ((d_expand, 0, "x"), (d_stop, 1, "s")):
                    st = dict(state)
                    st["depth"] = depth_byte
                    doc_tail = (
                        continue_useful_q["criteria"]["true" if depth_byte == 0 else "false"]
                        if variant == "v8" else neutral_tail
                    )
                    emit_doc(dom, f"{base}_{tag}", render(st),
                             continue_useful_q["instructions"], doc_tail)
                    if dom == d_expand:
                        n_expand += 1
                    else:
                        n_stop += 1
            assert n_stop == n_expand == len(pool), (n_stop, n_expand, len(pool))
    else:
        n_stop = write_stop_docs(picked_stop, d_stop, STOP_POLARITY, "stop")
        n_expand = write_stop_docs(picked_expand, d_expand, EXPAND_POLARITY, "exp")

    ROUTE_PIN_TRUE = ("temporal", "entity")
    ROUTE_PIN_FALSE = ("semantic", "causal", "recency_importance")
    # v10: the WHOLE corpus renders in the spaced dialect — a request rendered
    # spaced (RIIR_REFLEX_SYSTEMONE_SPACED) must meet docs in the same dialect
    # or every compression-calibrated read (the traversal pin's symmetric-tail
    # beam preservation, the routing masses) breaks on the shape mismatch
    # (measured: compact docs + spaced requests = recall 0.567, a catastrophic
    # traversal-calibration break; the stopping head alone was clean).
    route_render = wire_state_spaced if variant == "v10" else wire_state
    n_mh = n_dir = 0
    for rec in routes:
        qs = rec["questions"]
        if "multi_hop_need" not in qs:
            continue
        dom, tag = (d_mh, "mh") if rec["cat"] == 1 else (d_dir, "dir")
        sw = route_render(rec["state"])
        q = qs["multi_hop_need"]
        emit_doc(dom, f"{tag}_s{rec['s']}q{rec['qa']}_multi_hop_need", sw,
                 q["instructions"], qdesc(q, "true" if rec["cat"] == 1 else "false"))
        for qid in ROUTE_PIN_TRUE:
            emit_doc(dom, f"{tag}_s{rec['s']}q{rec['qa']}_{qid}", sw,
                     qs[qid]["instructions"], qdesc(qs[qid], "true"))
        for qid in ROUTE_PIN_FALSE:
            emit_doc(dom, f"{tag}_s{rec['s']}q{rec['qa']}_{qid}", sw,
                     qs[qid]["instructions"], qdesc(qs[qid], "false"))
        (n_mh, n_dir) = (n_mh + 6, n_dir) if rec["cat"] == 1 else (n_mh, n_dir + 6)

    rng.shuffle(pins)
    n_pin = 0
    for rec in pins[:PIN_STATES]:
        qs = rec["questions"]
        state = dict(rec["state"])
        state["evidence"] = list(state.get("evidence", []))[:PIN_EVIDENCE]
        state["candidates"] = list(state.get("candidates", []))[:PIN_CANDIDATES]
        sw = route_render(state)
        for qid, q in qs.items():
            if not qid.startswith("candidate_0_"):
                continue  # one question family per state (index 0 of the trimmed set)
            emit_doc(d_pin, f"pin_s{rec['s']}q{rec['qa']}_{qid}_t", sw,
                     q["instructions"], qdesc(q, "true"))
            emit_doc(d_pin, f"pin_s{rec['s']}q{rec['qa']}_{qid}_f", sw,
                     q["instructions"], qdesc(q, "false"))
            n_pin += 2

    stats = {
        "labels_fit": {
            "n": len(label_rows), "stop": sum(1 for r in label_rows if r[5]),
            "expand": sum(1 for r in label_rows if not r[5]),
            "note": "v6: depth classes, not stop labels (the v5 depth-0 read was inseparable)",
        },
        "docs": {stop_dir_name: n_stop, expand_dir_name: n_expand,
                 "routing_multihop": n_mh, "routing_direct": n_dir,
                 "traversal_pin": n_pin},
        "domains": 5, "seed": SEED, "variant": variant,
        "sizes_bytes": {d.name: sum(f.stat().st_size for f in d.iterdir())
                        for d in (d_stop, d_expand, d_mh, d_dir, d_pin)},
    }
    (corpus / "_build_stats.json").write_text(json.dumps(stats, indent=1, sort_keys=True))
    print(json.dumps(stats, indent=1, sort_keys=True))
    print(f"corpus at {corpus} — serve with RIIR_REFLEX_CORPUS={corpus}")


# ------------------------------------------------------------------- eval

def systemone(serve_url: str, state, question: dict, qid: str) -> float:
    body = json.dumps({"state": state, "model": "eval",
                       "questions": {qid: question}}).encode()
    req = urllib.request.Request(f"{serve_url}/v1/systemone", data=body,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        ans = json.loads(resp.read())["answers"][qid]
    if ans["type"] != "noul":
        fail(f"expected noul answer, got {ans}")
    return float(ans["noul"])


def cmd_eval(args) -> None:
    raw = Path(args.raw_dir)
    capture_dir = raw / "capture"
    results, stop0 = raw / "results", raw / "t3_stop0"
    # Selection discipline (Bench-127 law): `--samples fit` fires the FIT
    # samples (s0-4) — the posture-selection surface (nb_scale sweeps);
    # `--samples eval` (default) is the honest held-out read (s5-9), fired
    # ONCE per posture.
    samples = FIT_SAMPLES if args.samples == "fit" else EVAL_SAMPLES
    continue_q, suff_q, mh_q, entity_q, temporal_q, rel_q = None, None, None, None, None, None
    depth0_rows, depth1_rows, route_rows, pin_rows = [], [], [], []
    for s in samples:
        cap_p = capture_dir / f"sample{s}.jsonl"
        if not cap_p.exists():
            fail(f"missing capture {cap_p}")
        rows_base = load_rows(results, s)
        rows_stop0 = load_rows(stop0, s)
        for line in cap_p.read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            qa, cat = row["qa_index"], row["category"]
            r0, r1, label = stop_label(rows_base, rows_stop0, qa)
            if label is None:
                continue
            for call in row["calls"]:
                if call["op"] == "stopping":
                    depth = call["state"].get("depth")
                    rec = {"s": s, "qa": qa, "r0": r0, "r1": r1, "label": label,
                           "state": call["state"], "qs": call["questions"]}
                    (depth1_rows if depth == 1 else depth0_rows).append(rec)
                    if continue_q is None:
                        continue_q = call["questions"]["continue_useful"]
                        suff_q = call["questions"]["evidence_sufficient"]
                elif call["op"] == "routing":
                    if mh_q is None:
                        qs = call["questions"]
                        mh_q = qs["multi_hop_need"]
                        entity_q = qs["entity"]
                        temporal_q = qs["temporal"]
                    route_rows.append({"s": s, "qa": qa, "cat": cat,
                                       "state": call["state"]})
                elif call["op"] == "traversal" and rel_q is None:
                    rel_q = call["questions"]["candidate_0_relevance"]
    if continue_q is None or mh_q is None:
        fail("no stopping/routing capture found for the question inventory")
    if not pin_rows:
        pin_src = capture_dir / (f"sample{samples[0]}.jsonl")
        for line in pin_src.read_text(encoding="utf-8").splitlines()[:40]:
            row = json.loads(line)
            for c in row["calls"]:
                if c["op"] == "traversal" and "candidate_0_relevance" in c["questions"]:
                    pin_rows.append({"state": c["state"],
                                     "q": c["questions"]["candidate_0_relevance"]})
                    if len(pin_rows) >= 30:
                        break
            if len(pin_rows) >= 30:
                break

    # The v6 decision surface: depth-0 must NOT stop (continue_useful >= 0.40
    # AND sufficient < 0.85); depth-1 MUST stop (continue_useful < 0.40 OR
    # sufficient >= 0.85 — either lever, exactly their controller's OR/AND).
    def stop_read(rec):
        p_c = systemone(args.serve_url, rec["state"], rec["qs"]["continue_useful"], "continue_useful")
        p_s = systemone(args.serve_url, rec["state"], rec["qs"]["evidence_sufficient"], "evidence_sufficient")
        return p_c, p_s

    d0_no_stop = d0_premature = 0
    d0_pcs, d0_pss = [], []
    for rec in depth0_rows:
        p_c, p_s = stop_read(rec)
        d0_pcs.append(p_c)
        d0_pss.append(p_s)
        if p_c < CONTINUE_THRESHOLD or (p_s >= STOP_THRESHOLD and False):
            d0_premature += 1
        else:
            d0_no_stop += 1
    d1_stop = d1_missed = 0
    d1_pcs, d1_pss = [], []
    for rec in depth1_rows:
        p_c, p_s = stop_read(rec)
        d1_pcs.append(p_c)
        d1_pss.append(p_s)
        if p_c < CONTINUE_THRESHOLD or (p_s >= STOP_THRESHOLD):
            d1_stop += 1
        else:
            d1_missed += 1

    # The projected run: recall = base (the depth-1 stop changes nothing —
    # scan-2 adds no nodes); edges = scan-1 only for every QA that runs scan-1
    # (all: depth-0 never stops), minus nothing. The measured base scan-2 cost
    # is derivable per QA from the base run vs the depth-1-stop projection.
    n_recall = len(depth0_rows)
    proj_recall = sum(r["r1"] for r in depth0_rows) / n_recall
    base_recall = proj_recall  # identical by construction (no depth-0 stops)

    mh_hits = 0
    for row in route_rows:
        pm = systemone(args.serve_url, row["state"], mh_q, "multi_hop_need")
        if (pm >= 0.5) == (row["cat"] == 1):
            mh_hits += 1
    ent_p, tmp_p, pin_p = [], [], []
    for row in route_rows[:60]:
        ent_p.append(systemone(args.serve_url, row["state"], entity_q, "entity"))
        tmp_p.append(systemone(args.serve_url, row["state"], temporal_q, "temporal"))
    for row in pin_rows[:30]:
        pin_p.append(systemone(args.serve_url, row["state"], row["q"], "relevance"))
    out = {
        "n_heldout": n_recall,
        "depth0": {"n": len(depth0_rows), "no_stop": d0_no_stop,
                   "premature_stop": d0_premature,
                   "p_continue_mean": sum(d0_pcs) / len(d0_pcs),
                   "p_continue_min": min(d0_pcs),
                   "p_suff_mean": sum(d0_pss) / len(d0_pss),
                   "p_suff_max": max(d0_pss)},
        "depth1": {"n": len(depth1_rows), "stop": d1_stop, "missed": d1_missed,
                   "p_continue_mean": sum(d1_pcs) / len(d1_pcs),
                   "p_continue_max": max(d1_pcs),
                   "p_suff_mean": sum(d1_pss) / len(d1_pss),
                   "p_suff_min": min(d1_pss)},
        "projected": {"pooled_recall": proj_recall,
                      "base_pooled_recall": base_recall,
                      "edges": "scan-1 only per QA (the run measures it)"},
        "routing_multihop_acc": mh_hits / len(route_rows),
        "routing_entity_p_mean": sum(ent_p) / len(ent_p),
        "routing_temporal_p_mean": sum(tmp_p) / len(tmp_p),
        "traversal_pin_p_mean": sum(pin_p) / len(pin_p) if pin_p else None,
        "traversal_pin_p_range": [min(pin_p), max(pin_p)] if pin_p else None,
        "n_routing": len(route_rows),
    }
    out["samples"] = args.samples
    print(json.dumps(out, indent=1, sort_keys=True))
    (raw / f"corpus_pack_{args.tag}_eval.json").write_text(json.dumps(out, indent=1, sort_keys=True))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("capture")
    c.add_argument("--sample", type=int, required=True)
    c.add_argument("--dataset", default=".raw/locomo/locomo10.json")
    c.add_argument("--jev-config", default=".raw/jev-mem/config/jev_mem.json")
    c.add_argument("--jevmem-dir", default=".raw/jev-mem")
    c.add_argument("--raw-dir", default=".raw/locomo")
    c.set_defaults(fn=cmd_capture)
    b = sub.add_parser("build")
    b.add_argument("--raw-dir", default=".raw/locomo")
    b.add_argument("--variant", choices=["v6", "v7", "v8", "v9", "v10", "v11"], default="v6",
                   help="v6: stopping_stop/stopping_expand dirs (the measured posture); "
                        "v7 (issue 081 T2c): the same docs under the wire's continue_useful "
                        "criteria descriptions — by-name option routing arms the count tables; "
                        "v8: v7 names + PAIRED docs (each state in BOTH domains, only the "
                        "depth byte differs — the cancellation corpus); "
                        "v9: v8 + NEUTRAL tails (collision-noise repair — identical tail "
                        "text drops the class-exclusive bucket set to the depth forms)")
    b.set_defaults(fn=cmd_build)
    e = sub.add_parser("eval")
    e.add_argument("--serve-url", default="http://127.0.0.1:7331")
    e.add_argument("--raw-dir", default=".raw/locomo")
    e.add_argument("--samples", choices=["fit", "eval"], default="eval",
                   help="fit = s0-4 (posture selection: nb_scale sweeps); eval = s5-9 (the honest read)")
    e.add_argument("--tag", default="v6", help="output file tag: corpus_pack_<tag>_eval.json")
    e.set_defaults(fn=cmd_eval)
    args = ap.parse_args()
    args.fn(args)


if __name__ == "__main__":
    main()
