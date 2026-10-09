# The memory-control decision pack (the Jev-Mem-class consumer lane)

Issue 081's funnel artifact: everything a memory system that runs its control
loop on typed decisions needs in order to adopt reflex as its decision
backend — the question-pack structure, the wire mapping, the integration
recipe, and the measured posture. Published as a PROTOCOL NOTE: the question
texts themselves belong to the integrating system (it brings its own
questions over the wire); nothing of their prose is copied here, per the
lane's licensing posture.

## What a "memory-control decision pack" is

A published memory system (Jev-Mem, arXiv:2609.23986 — named only to compare)
runs its ENTIRE memory lifecycle on a typed decision engine. Its controller
asks batches of small typed questions against shared state; every question is
one of exactly two wire shapes:

- **noul** — "probability of a defined proposition": instructions + a
  `{true, false}` criteria pair (each side a acceptance/rejection description).
- **choice** — a selection among 2+ labeled options, each with a description;
  the answer carries the pick, a full probability distribution (sums to 1
  ±1e-3, pick must be the argmax), and a confidence.

The observed pack structure (their inventory, per operation):

| operation | lifecycle stage | questions | shape |
|---|---|---|---|
| observation | write: admission + typing | 5 admission + 4 type | 9 noul |
| memory_type | write: typing (admission off) | 4 | 4 noul |
| relations | write: candidate linking | 3–4 per candidate | batched noul |
| consolidation | write: periodic merge | 4 + 1 per candidate | 4 noul + 1 choice (4 options) |
| routing | read: graph-need analysis | 6 | 6 noul |
| stopping | read: sufficiency gate | 4 | 4 noul |
| traversal | read: candidate scoring | 4 per candidate | batched noul |

~29 distinct noul propositions + 1 choice. Every one maps 1:1 onto
`decision_wire` (`noul` → Noul, `choice` → Choice with criteria) — the pack
is a ready-made decision-domain corpus for any decision engine that speaks
the wire.

## The wire mapping (reflex `/v1/systemone`)

The TypeSafe dialect is spoken natively by the reflex serve edge
(`src/systemone.rs`):

- noul criteria `{true, false}` → the decision question's two option
  descriptions; instructions become the decision context.
- choice criteria keys → options in wire order (insertion-ordered);
  descriptions join the criteria string.
- `state` → canonical JSON string (the shared evidence/candidate payload).
- Responses: noul → `{type, noul: p}`; choice → `{type, choice, confidence,
  probabilities}` with the distribution laws enforced (sum ±1e-3, argmax
  consistency) — their SDK's own pydantic validation accepts every response.
- Abstention maps honestly (the wire has no abstain): noul → 0.5, choice →
  uniform + deterministic argmax. An unfitted engine therefore reads as the
  flat ~0.5 posture — visible, not deceptive.
- `usage: {}` (the modelless engine counts no tokens), `model:
  "reflex-modelless"` (never an echo of the requested model name).

## The integration recipe (zero fork)

The consumer's own config surface does the work — three environment
variables, no patches, nothing contributed upstream:

```sh
reflex serve                          # 127.0.0.1:7331 (the modelless lane)
# consumer side:
TYPESAFE_BASE_URL=http://127.0.0.1:7331
TYPESAFE_API_KEY=reflex-local         # dummy — the route never checks it
# their config: decision_backend stays "jev" (the default); the SDK's
# system_one() call lands on reflex's /v1/systemone.
```

## The measured posture (what to expect before fitting)

Over the full LoCoMo-10 matrix (1540 QAs, categories 1-4, their published
config; [Bench 133](../../.benchmarks/133_jevmem_t2/RESULTS.md)):

- **recall parity-plus**: 0.7921 vs the deterministic-heuristic baseline's
  0.7847 pooled evidence_recall (paired 61W-41L, sign test p=0.0297) — with
  the engine UNFITTED for the domain (flat ~0.5 values on their question
  texts; the abstain class).
- **−21% retrieval work** at equal decision-call count; the write path with
  flat relation values rejects every semantic link and the retrieval still
  recalls at parity — the substrate (anchors + temporal/entity links)
  carries the load; decision differentiation concentrates where traversal
  matters.
- **The stopping surface is a binary gate** ([Bench
  134](../../.benchmarks/134_jevmem_t3_stopping/RESULTS.md)): the traversal
  collapses to depth ≤ 1 everywhere; the only live decision is
  expand-once-or-not. The fitted target is one sigmoid over anchor-evidence
  sufficiency — labels from corpus supervision (QAs whose anchors-only
  recall equals their post-expansion recall → don't expand).

## The corpus lever (the fitting lane, next)

The flat ~0.5 posture is the ceiling of an unfitted engine. The fitting data
is the integrating system's own supervision (for LoCoMo: an utterance in
some QA's gold evidence ⇒ high future-utility; never-referenced ⇒ low) —
authored as a reflex corpus for the memory-control suite, never pooled with
the evaluation read (Bench-127 determinism law applies to every re-fit).

**Executed and closed** ([Bench 135](../../.benchmarks/135_jevmem_t2b_corpus/RESULTS.md) +
[Bench 136](../../.benchmarks/136_jevmem_t2c_pairwise_head.md)): the fitted read-path corpus
dominates the mock-stop-1 floor on both axes (0.7691 @ 1074 edges held-out), and the
fitted-token head (the pairwise table margin, `nb_pair_scale`) now reads O(1)-token
signals perfectly — but the stopping lever itself has NO reachable Pareto win on this
controller: the loop is bounded by the ROUTING-derived depth limit (2-3 beam waves; the
gold lives in waves 2-3), so a perfect depth-1 stop still costs −15.6pt recall at 200
edges, and the v5 supervision labels are content-inseparable on every estimator measured
(compression path AND count tables — the class prior dominates). The head machinery stays
available for any future O(1)-token policy surface.
