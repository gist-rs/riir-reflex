# Bench 133 — Jev-Mem LoCoMo decision-layer cells: mock vs reflex (Issue 081 T2)

**Status:** COMPLETE — the T2 decision-layer matrix over the full LoCoMo-10 set
(1540 QAs, categories 1-4, all 10 samples), both arms deterministic, reflex at
recall parity-plus and −21% retrieval work vs the mock baseline. The TypeSafe
Jev API and Laya-local arms are SKIPPED loud (BYO key / heavy install), never
green.

## What ran

Jev-Mem's OWN pipeline, end to end, twice — only the decision backend differs:

- **mock** — `jev_mock=True`: their inline `mock_values` heuristics (cosine
  semantic relations, keyword-informed admission). The F-law baseline.
- **reflex** — `decision_backend=jev` + `TYPESAFE_BASE_URL` → `reflex serve`
  (`/v1/systemone`, the T1 seam; corpus `demo`, modelless lane; every decision
  served `model:"reflex-modelless"` — audit-verified, zero fallback events).

Everything else identical: `config/jev_mem.json` (their published LoCoMo
posture: stopping 0.95/0.15, `maximum_jev_calls` 16, anchor 30, top-k 40/50,
`admission_enabled:false`), MiniLM local embeddings (their macOS CPU posture),
MAGMA temporal/entity links, same dataset file. LoCoMo-10 fetched from
`snap-research/LoCoMo` (`data/locomo10.json`, sha12 `79fa87e90f04`), kept local
under `.raw/` per both repos' licensing posture — never committed.

Driver: `scripts/jevmem_t2_cells.py` (run/compare/summary; per-arm audit
verification refuses a run whose decisions were served by the wrong backend or
degraded to fallback — never a quiet green).

## The cells (their `RetrievalController` metadata + their recall formula — no LLM anywhere)

| arm | n | pooled evidence_recall | cat1 multi-hop | cat2 temporal | cat3 open-domain | cat4 single-hop | mean edges examined | mean jev_calls |
|---|---|---|---|---|---|---|---|---|
| mock | 1540 | 0.7847 | 0.5445 | 0.8377 | 0.5298 | 0.8730 | 2108.6 | 3.97 |
| reflex | 1540 | **0.7921** | 0.5451 | **0.8598** | 0.5091 | **0.8801** | **1670.2** | 3.97 |

- Paired per-QA (same 1540 questions): reflex better **61** / mock better 41 /
  tie 1438 → sign test one-sided **p = 0.0297** (suggestive, not decisive).
- Per-sample: mean **+0.71pt**, range −0.89pt..+4.29pt, reflex wins **6/10**.
- Stop profile: reflex `frontier_exhausted` 1540/1540; mock hit `max_edges`
  265/1540 (17%) — reflex's traversal scoring stays inside the edge budget
  every time (−20.8% edges examined at equal-or-better recall).

## The write-path read (why parity is the interesting part)

| write-path cell | mock | reflex |
|---|---|---|
| semantic relation values | mean 0.667 (cosine) | flat 0.500 (unfitted) |
| semantic links created | 2245 | **0** (0.5 < 0.6 threshold) |
| temporal / entity links | 3752 / 4109 | 3752 / 4109 (shared substrate) |
| memory_type scores | differentiated | flat ~0.495 |

The reflex arm ran the ENTIRE benchmark with zero semantic relation links —
all rejected by its flat unfitted 0.5 — and still matched or beat the
cosine-informed baseline on recall at lower cost. On this dataset the
retrieval substrate (vector anchors + temporal/entity links + top-k recovery)
carries cat2/3/4; the decision layer's differentiation shows where traversal
matters (cat2 +2.2pt, sample-7 +4.3pt). The mock's 2245 cosine semantic links
bought nothing that the anchors didn't already recover.

## Determinism (the Bench-127 law)

- Sample 0, both arms, two fresh processes each (real serve calls in both
  reflex passes): **152/152 per-QA rows byte-identical** per arm
  (`--compare`; counts + sorted dia lists — wall clock excluded by
  construction, it is not a cell).
- The full 20-run matrix was executed twice (an out-dir layout bug destroyed
  the first pass's per-QA files after printing; the re-run under the fixed
  per-sample layout reproduced every printed mean_recall exactly — the
  graphs were cached, so this also pins graph save/load fidelity).

## Honest caveats

- The reflex engine is **unfitted** for the memory-control domain (flat ~0.5
  noul posture — the abstain class; corpus `demo`). These cells measure the
  SEAM + substrate parity, not the engine's ceiling; T2's corpus lever
  (LoCoMo-derived supervision fitting future-utility/novelty/relation
  questions) is the follow-up that moves the decision values.
- Neither arm ever stops early (`evidence_sufficient ≥ 0.95` unreachable for
  both the mock 0.9 heuristic and reflex's flat 0.5) — the stopping
  controller is inert at this config; T3's calibrated thresholds are the
  jev_calls cost lever.
- Latency is NOT claimed: box under sustained multi-tenant load (sibling
  benches at ~950% CPU); advisory only, reflex ≈ 66 ms/query vs mock ≈ 32 ms
  under that load (4 HTTP calls + encode). No `bench_preflight` stamp — no
  publishable latency number.
- One config posture (`config/jev_mem.json`); the admission-on and budget
  variants are future cells, not claims.

## Skipped arms (loud, never green)

| arm | why |
|---|---|
| typesafe_jev_api | BYO `TYPESAFE_API_KEY` — spend-gated (visitor-key posture refused here) |
| laya_local | their `laya` pip package + weights — heavy install on a loaded box; NOT the reflex `laya-riir` lane (different weights, not comparable) |

## Reproduce

```sh
# serve first (the reflex arm refuses loud without it)
cargo run --release --bin reflex -- serve   # 127.0.0.1:7331, /healthz
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm mock   --jev-config .raw/jev-mem/config/jev_mem.json --sample <N>
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm reflex --jev-config .raw/jev-mem/config/jev_mem.json --sample <N>
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --compare  <pass1.json> <pass2.json>   # determinism strip
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --summary <results...>
```

Dataset: `.raw/locomo/locomo10.json` (local only). Rig: `.raw/jev-mem` @
`7ab0c73c` + `.raw/jevmem-env` (venv; `sentence-transformers` added for the
local MiniLM encoder — their default macOS CPU posture).

## Addendum 2026-10-09 — why the reflex values were flat, and the fix (T2b interim, reflex `fa315a9`)

The flat ~0.495 posture this bench recorded was STRUCTURAL, not
unfittedness: the engine's Noul kind scores the literal bytes `yes`/`no` —
a 2-3 byte literal is a constant compression offset no corpus can move
(measured identical on two corpus shapes). The fix landed in the
translation layer: canonical `{true, false}` criteria ride a 2-option
Choice whose options ARE the descriptions (criteria absent from the ctx —
rendering them there lets each candidate match the request itself and pins
p at 0.5). With a labeled-example LoCoMo corpus (fit s0-4, held-out s5-9):
should_store EV 0.795 vs FL 0.687 — differentiated but weakly (the
supervision label, not the mechanism, is the limit). The corpus-design
law and the gate measurements live in the issue's T2b note;
`examples/jevmem_gate_probe.rs` is the gate instrument (the gate passes
real requests at conf 0.94+ — never the blocker). Determinism law v2:
subprocess-isolated runs with seeded uuids (values are ctx-byte-sensitive;
their uuid4 node ids leaked into values under the in-process law).
