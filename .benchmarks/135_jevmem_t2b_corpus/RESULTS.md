# Bench 135 — Jev-Mem T2b: the fitted read-path corpus (the stopping lever, measured)

**Status:** COMPLETE — the corpus-fitting lane executed end to end on the held-out
slice (LoCoMo s5-9, 778 QAs, their pipeline, graph cache reused — only the serve
corpus differs). The honest verdict is SPLIT: the fitted arm **dominates the
mock-stop-1 floor on both axes** (0.7691 recall @ 1074 edges vs 0.7637 @ 1496)
but misses the T3 gate's recall hold (−0.96pt vs the held-out base) — and the
session's mechanism findings say WHERE the modelless substrate separates and
where it cannot. Two corpus shapes were built and measured (v5 label-split,
v6 depth-split); v6 is the executed arm.

## The corpus (5 domains, `scripts/jevmem_corpus_build.py`)

Fit samples s0-4 only (the Bench-127 law — fitting data never pooled with the
eval read). Docs are request-shaped: `<state JSON>\n<prompt>\nanswer: <option
description>`, per the T2b interim's measured corpus-design law (max-asymmetric
tails). Everything local under `.raw/` (their option descriptions are their
prose — never committed).

| domain | docs | bytes | what it teaches |
|---|---|---|---|
| stopping_stop | 1000 | 1.7 MB | depth-1 states → stop polarity (all 4 stopping questions) |
| stopping_expand | 750 | 1.3 MB | depth-0 states → expand polarity (3 controlled questions) |
| routing_multihop | 852 | 173 KB | cat1 queries: multi_hop→true; entity/temporal→true, semantic/causal/recency→false |
| routing_direct | 3708 | 777 KB | non-cat1: multi_hop→false; the same pins |
| traversal_pin | 400 | 2.0 MB | symmetric tails → p≈0.5 (preserve the base beam ranking) |

Labels: routing from the QA category (cat1 = multi-hop); stopping from the
MEASURED per-QA recall rows (base = recall@1, t3_stop0 = recall@0). The
label distribution that motivated v6: **92.6% of QAs lose nothing from
stopping at depth 0** (1423/1536; expansion helps only 113 QAs carrying
~4.2pt pooled) — the oracle-stop ceiling is recall-equal at ~66 mean edges,
which is why the depth-0 lever was worth trying.

## The measured cells (s5-9, 778 QAs, `config/jev_mem.json`, graph cache hit)

| arm | pooled recall | mean edges | jev_calls | stop reasons |
|---|---|---|---|---|
| base reflex (0.95/0.15, demo corpus) | 0.7787 | 1700 | 3.94 | frontier_exhausted 778 |
| **fitted reflex (0.85/0.40, corpus v6)** | **0.7691** | **1074** | 3.29 | frontier_exhausted 405 / further_retrieval_unhelpful 373 |
| mock stop-1 (0.85/0.40) — the floor | 0.7637 | 1496 | 3.94 | (Bench 134 arm) |
| stop-0 reflex (the anchor floor) | 0.7291 | 0 | 1.97 | (Bench 134 arm) |

- Fitted vs base: **−36.8% edges, −0.96pt recall** (paired 15W/30L/731T).
- Fitted vs the mock-stop-1 floor: **+0.54pt recall AND −28.2% edges** —
  dominated on both axes.
- Determinism (Bench-127 law): s8 re-run in a fresh process, real serve calls
  both passes — **156/156 per-QA rows byte-identical**.
- jev_calls 3.29: the depth-1 stops skip the traversal call for stopped QAs.

## The mechanism findings (the durable part)

1. **The route IS the classifier, and it only reads textual class mass.**
   Routing-class decisions separate: multi_hop_need accuracy **0.817** on
   held-out queries (the ctx is `{"query": …}` + prompt — short, the class
   signal is a large share of the bytes). Big-JSON-state decisions do not:
   the depth-0 stop/expand label split (v5) read p_continue med **0.405 vs
   0.415** (fully overlapping — no threshold pays: 0.4 → −2.1pt, 0.5 →
   −4.4pt); the depth-0/depth-1 split (v6) read **0.420 vs 0.417** — the
   `depth:1` token is ~1 token of mass in a ~7 KB state, invisible to the
   hashed-bag centroid against evidence noise.
2. **The option-delta surface is the domain's last ~64 KB** (LZ4 window +
   single-slot hash table): corpus mass beyond that neither helps nor harms
   the option scores — only the domain CENTROID (routing) uses the full
   corpus. Corpus size is a latency knob, not a fidelity knob.
3. **The pin-steal**: a domain whose docs share the request's STATE shape
   captures the route of every question over that state family (measured:
   100% of stopping ctxs routed to `traversal_pin`). Per-operation domains
   over near-identical state shapes do not layer cleanly — the prompt is
   ~2% of the mass.
4. **The traversal batch is self-symmetric**: every candidate question's ctx
   is the same state (only the `candidates[i]` index digit differs), so
   per-candidate p is near-constant whatever the corpus — the beam ranking
   stays cosine-dominated (base behavior) with or without a pin. Fitting
   per-candidate relevance through this wire is structurally vacuous.
5. **The write path was frozen on purpose**: the fitted run reuses the base
   graph (cache fingerprint is config-only) — the comparison isolates the
   read-path decisions. On this data the flat-reject relation posture
   (0 semantic links) already BEAT mock's 2245 cosine links (Bench 133) —
   fitting relations toward link-creation is measured-negative value.

## Honest caveats

- The T3 gate letter (issue 081: "0.7921 @ ~1200 edges") is NOT met: the
  cost target is beaten (1074) but the recall hold missed by 0.96pt — the
  depth-0 premature stops (the route coin-flip) cost exactly what the v5
  read predicted. The gate's SPIRIT (beat the stop-1 floor Pareto) is met.
- The depth-1 stop saves the scan-2 edges (~626/QA here) at zero semantic
  risk (frontier_exhausted 1540/1540 in Bench 134); the recall cost comes
  entirely from the ~depth-0 stops that should not have fired.
- One corpus posture (v6), one config posture (0.85/0.40 — their dataclass
  defaults, the Bench-134-legitimised arm); latency not claimed (loaded
  box, no preflight stamp).
- The capture states for the depth-1 docs came from the MOCK trajectory
  (mock beam ≠ fitted beam); the depth-0 states are decision-independent
  (anchors precede any decision) — exact. The run's measured outcome
  absorbs the approximation.

## Reproduce

```sh
# serve with the fitted corpus (build it first — see below)
RIIR_REFLEX_CORPUS=.raw/locomo/corpus_pack_v6 ./target/release/reflex &
.raw/jevmem-env/bin/python scripts/jevmem_corpus_build.py capture --sample <0-9>   # states (mock cache)
.raw/jevmem-env/bin/python scripts/jevmem_corpus_build.py build                    # -> corpus_pack_v6
.raw/jevmem-env/bin/python scripts/jevmem_corpus_build.py eval                     # held-out decision surface
.raw/jevmem-env/bin/python scripts/jevmem_t2_cells.py --arm reflex --jev-config \
  .raw/jev-mem/config/jev_mem.json --sample <5-9> --stop-threshold 0.85 \
  --continue-threshold 0.40 --out-dir .raw/locomo/t2b_v6/s<N>
```

Rig: `.raw/jev-mem` @ `7ab0c73c` + `.raw/jevmem-env` + `.raw/locomo/`
(dataset, captures, corpora, all run dirs). The v5 corpus + its eval JSON
are preserved beside v6 (`.raw/locomo/corpus_pack_v5*`) — the negative's
evidence.

## What this closes / opens

- **Closed**: the T2b corpus lever, executed — the fitted read-path corpus
  beats the floor on both axes but cannot hold base recall; the depth-0
  stop decision is measured inseparable on this substrate (the honest
  negative, with the mechanism laws above).
- **The named re-open**: a fitted-token head (nb/ridge/option-cond over
  state tokens — the `depth` token is exactly the feature a count table
  latches) would need a serve-side head-loading path (the harness lane's
  fitting machinery, not the user-corpus lane). That is new engine surface,
  owner-gated by effort, not by this bench.
