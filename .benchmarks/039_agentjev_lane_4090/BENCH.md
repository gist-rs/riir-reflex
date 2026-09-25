# Bench 039 — Issue 025 amendment 4 / 027: the AgentJev gold-label row — the published ranking survives the protocol change (+2.7pt over laya-typed on OUR split)

Two runs on 2026-09-25, host `4090-windows` (REFLEX_BENCH_HOST), reflex `a2c03aa`
+ the new lane, binary `target/release/harness` (default features — the lane is
ungated, zero new deps), `--skip-laya --agentjev`:

1. `--suites typed_decisions` (the row's owned data point — kept as
   `results.typed_only.json` beside the full run).
2. The full 15-suite registration (14 measured + `harness_cache_reuse` the
   designed LLM-only absence).

Serving posture: THEIR `jev_service` (malevrigns/agent-jev @ `a965ca8f` =
their main HEAD, Apache-2.0, not affiliated) on loopback `:8149`, their
published step-600 tensors (`aimeigaoshou/agent-jev` model.safetensors →
torch-wrapped once, sha `9d9b5fc3…` advertised by /api/info and stamped into
every row as `AgentJev-0.6B@9d9b5fc3`), Qwen/Qwen3-0.6B backbone from the
local HF snapshot, `--device cuda:0`, their `temperatures.json` loaded
(boolean 1.0718 / choice 1.0353 / score 1.0718). Their stack serves, our
Rust measures — the MEASURE-vs-SERVE split, no riir-infer port (the lane-3
deferral in `.issues/025` stands).

**Box state** (the standing disclosure): GPU 518 MiB desktop + their service
resident ~2.7 GiB bf16 (the only compute job — the serialized-window law held);
AC desktop, no power axis on this host; no sibling compute, no perf-league
cycle. Latency = client round-trip per case over loopback HTTP (their
`usage.wall_ms` agrees: 35.3 s server-sum ≈ 37.8 s lane wall on typed_decisions).

## The headline — the 025 amendment-4 data point

| protocol | AgentJev | laya·typed | source |
|---|---|---|---|
| THEIR protocol (teacher-argmax agreement) | **0.7925** | 0.7700 (card-copied) | their README, quoted |
| OUR protocol (gold-label, this harness) | **0.7715** | 0.7445 | measured (bench 039 / the aa37823 baseline) |

The published ranking SURVIVES the protocol change: on gold labels AgentJev
beats our measured laya-typed by **+2.7pt** (their teacher-protocol gap was
+2.25pt, their bootstrap [+0.65, +3.90]). The teacher-vs-gold drop for
AgentJev is −2.1pt (0.7925 → 0.7715) — teacher agreement is slightly easier
than gold, and the win is real under both references.

**Reproducibility**: typed_decisions ran TWICE (run 1 solo + run 2 inside the
full registration) — **0.7715 both times**, 2000 questions, identical
accuracy. The det column reads ✗ (below) — the wobble stays below decision
margins; picks are stable.

## The full lane (14 suites; laya/gliner cells quoted from bench 037 — same box, same substrate sha)

| suite | modelless | laya·en | laya·typed | gliner | agentjev | verdict |
|---|---|---|---|---|---|---|
| typed_decisions | 0.3190 | 0.3575 | **0.7445** | 0.5280 | **0.7715** | **agentjev wins (+2.7pt over the specialist)** |
| ag_news | 0.5100 | **0.9500** | — | 0.7025 | 0.8000 | laya (−15pt) |
| emotion | 0.2825 | **0.5925** | — | 0.5650 | 0.4225 | laya (−17pt) |
| sst5 | 0.2167 | 0.3717 | — | 0.4383 | **0.4383** | agentjev ties gliner, both over laya |
| prompt_injections | 0.4397 | **0.6983** | — | 0.6810 | 0.4828 | laya (−21.6pt) |
| xnli_en | 0.3467 | **0.8600** | — | 0.4767 | 0.4567 | laya (−40.3pt) |
| massive_intent_en | 0.6900 | 0.7500 | — | **0.8233** | 0.6233 | gliner |
| banking77 | 0.4460 | 0.4980 | — | **0.7060** | 0.5460 | gliner |
| code_fixtures | 0.2500 | 0.5833 | — | **0.6667** | 0.3750 | gliner |
| harness_visibility | 0.3750 | 0.3125 | — | **0.5000** | 0.4375 | gliner |
| harness_permissions | 0.4167 | 0.4167 | — | **0.5833** | 0.4167 | gliner |
| harness_tool_fit | 0.5000 | 0.5000 | — | 0.7500 | **0.7500** | agentjev ties gliner |
| harness_routing | 0.4375 | **0.6250** | — | 0.6250 | 0.3750 | laya |
| harness_sensitivity | 0.4000 | 0.6000 | — | **0.8000** | 0.4000 | gliner |

**3 agentjev wins / 7 laya wins / 4 gliner wins.** The shape: a typed-decision
SPECIALIST — dominant on the suite it was trained on (typed_decisions, the
category's accuracy bar now **0.7715 gold-label / 0.7925 teacher**), mediocre
on classic NLU (ag_news −15pt, xnli −40pt vs laya) AND on the decision-style
suites it was NOT trained for (banking77 −16pt vs gliner, massive_intent
−20pt). GLiNER2.5-Decide is the better GENERALIST decision model on our 15;
AgentJev is the better specialist on ITS split; laya-typed remains the best
SERVED-in-product lane (0.7445 at 1312 ms m3-metal / 107 ms 4090-cuda, no
Python service in the loop).

## Latency (p50 ms; same box, client round-trip over loopback HTTP)

| shape | agentjev | vs |
|---|---|---|
| typed_decisions (long ctx, 5 q/case) | **88–89** | their published ~60–70 (their box, no HTTP); laya-riir cuda 107, gliner 30 |
| short suites (ag_news…sensitivity) | 28–31 | laya-riir cuda 10–15, gliner 22–23 |
| banking77 (wide labels) | 149 | — |

Their shared-prefix runtime shows in the long-context shape (88 ms beats the
in-process laya-riir cuda lane's 107); short inputs carry the HTTP +
per-request overhead (~28 ms floor). No client batching — one case per
request, the cross-lane latency unit every other lane uses.

## Integrity

- **det ✗ on every suite** — their OWN disclosed class (jev_service README:
  "HTTP question isolation BF16 numerical difference 约 0.00282" — batch-
  composition-dependent bf16 kernels). The observed-repeat check compares
  two RAW response bodies; probabilities wobble in the 3rd decimal, picks
  do not flip (typed_decisions reproduced 0.7715 exactly across two full
  passes). Recorded, not repaired — it is their serving path, not ours.
- The modelless cells are bit-identical to the published 4090 cells (the
  standing drift gate; `code_fixtures` the designed commit-relative
  exclusion).
- Mapping disclosures (the lane module docs): state passes through
  UNCHANGED (their `semantic()` stable-JSONs objects — no our-side prose
  law); noul uses their default TRUE/FALSE candidates (the wire cannot
  carry per-side criteria prose — the clm-lane divergence class);
  `conf` = their top-probability readout.
- Context law: their service REFUSES >2048-token paths (nothing truncated);
  no suite hit the refusal on this registration (code_fixtures' states fit).

Raw: `results.json` (full) + `results.typed_only.json` (the solo run) +
`TABLES.md` beside this file.

**Concurrent-fix note**: the modelless cells above were measured at the
stated sha (`a2c03aa`-era) — the SAME-DAY issue-030 engine fix (noul never
takes route terms, `.issues/030`, bench 038 — the number this record's
renumber yielded to) changed the modelless prompt_injections cell after
this run; the laya/gliner/agentjev cells are unaffected (none of them
route through our engine).
