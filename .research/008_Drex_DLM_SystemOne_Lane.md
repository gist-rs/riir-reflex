# Research 008 — Drex DLM (Nace.AI): the diffusion-LM decision model, new top open entry on the Decision Index

**Status:** RECORD — external-repo distill (owner ask, 2026-10-07). Lane CLOSED: issue 073
landed `48206d5`/`a1bae34` + `cd0248b`, closed and removed upstream (closure narrative in
`HISTORY.md`); its follow-up issue 076 (det re-read + the per-kind confidence cells)
ALSO closed — det ✓ 10/10 under the timing-tail strip at BOTH serving postures, the
not-calibrated verdict three-posture (Benches 125 + 126).

> **Update 2026-10-08:** the next entrant after this note is LiquidAI's open-weight d1
> (same wire, d1-3B 48.57 / d1-omni-600M 15.95 on 0.2.1 — and d1-omni is the first
> laya-scale-class entrant) — `.research/009_LiquidAI_d1_Decision_Models.md` + issues 077+078.
> d1's board table omits the Drex-1.5/Jev rows above d1-3B (subset-table law, again).

> External source: `nace-ai/drex-dlm` @ `6c63df209db96c32160e1b4dfc769efb56b21f89`
> (cloned 2026-10-07; repo code MIT, model weights **CC BY-NC 4.0**). HF:
> `nace-ai/drex-dlm` (BF16 safetensors, ~16 GB, 8B) + `nace-ai/drex-dlm-Q8_0` (pointer
> head stays F16). Cloned into `riir-reflex/.raw/drex-dlm` for the verdict; removed
> after commit. Backbone: `nvidia/Efficient-DLM-8B` (arXiv:2512.14067 — AR→diffusion
> conversion; block tensor names match Qwen3).

## TL;DR

Nace.AI open-weights-released **Drex DLM**, an 8B diffusion-LM decision model that
scores TypeSafe-Jev-wire typed questions (`choice`/`noul`/`score` over `state`) in one
forward pass — the same product category as our laya lane and the six existing
comparison lanes, and the **new top open entry** on the Decision Index (their README
table, edition 0.2: Drex 52.31 > Gemma-4-31B decider 51.93 > Jev 51.67; their homepage,
edition 0.2.1: hosted "Drex 1.5" 58.28 > Jev 1.13.0 **57.91** — the same 57.91 our
Research-005 board snapshot recorded for Jev). Nothing in the mechanism is new to us —
the wire is `decision_wire`'s, the packed one-pass shape is the laya lane's, the
per-option scoring is what our modelless engine already does invariantly — but the
category's open-weights bar moved again, and the arena answer is a `--drex` comparison
lane (issue 073). License law: **weights are CC BY-NC — comparison lane yes, distill
teacher NO, product serving NO** (the bekko-MIT teacher posture does not transfer).

⚠ **Board-number law: edition 0.2 and edition 0.2.1 are NOT comparable — never put a
0.2 cell beside a 0.2.1 cell.** Jev reads 51.67 on one and 57.91 on the other; that
spread is an edition change, not a regression. Every number below is cited as "their
table" and stays there until our own lane measures this release on our own splits.

## What Drex DLM is (from the code, pinned sha above)

- **Wire**: `POST /v1/systemone`, `{state, model, questions: {qid: {type:
  noul|choice|score, instructions, criteria}}}` — pydantic `SystemOneRequest` whose
  docstring says "TypeSafe-compatible" outright. `noul` → 2 options `[false, true]`,
  answer = p(true); `choice` → named options (`name: desc`); `score` → ordered levels,
  probability-weighted expected level + legend. Response adds `usage` (input tokens +
  serialized-answer "output" tokens — billing-style, no generation happens) and
  `latency_ms`. One decision per request is the release contract (multi-decision
  `requests` wrapper refused with 400).
- **Packing** (`kev/model.py::encode`): one sequence `[state][q1 branch][q2 branch]…`;
  per question `[<q> instr][<opt> o1 </opt> <opt> o2 </opt>…][<decide>]`. Segment ids
  (0 = state, k = question k) drive a **block-causal attention mask**
  (`branch_mask_batch`): `attend(i,j) iff j≤i and (seg[j]==0 or seg[j]==seg[i])`, plus
  `state_bidir` for the diffusion backbone (state attends to state both directions —
  state still never sees branches, so the state prefix cache stays exact).
- **Row form** (`rows_of`/`forward_rows_batch`): hybrid (GDN/linear-attention)
  backbones cannot honour the packed mask, and over-long packed sequences grow an L×L
  mask without bound — so every question can instead run as a self-contained causal row
  `state + branch`, parity-tested equal to packed (`test_rows_match_packed`). Serving
  picks per request (`rows_form`).
- **Pointer head** (`PointerHead`, 256-dim): query = Linear(h@`<decide>`), keys =
  Linear(h@each `</opt>`), scaled dot product, per-question softmax. A single
  temperature is fitted on dev rows at checkpoint time (argmax unchanged by
  construction; training always sees T=1).
- **State-prefix reuse** (`probs_and_prefix` / `probs_with_prefix`): the state is
  encoded once (KV cache + hidden kept, cache cropped back to state after each use);
  question branches run as continuation rows attending to the cached state. "Exact by
  construction: branch tokens never attend across questions and the state never sees
  the branches." `prefix_min_tokens` = 384 for attention-only backbones (below that a
  packed pass beats per-op overhead on MPS), always for hybrid (Kev-0.8B bf16 on MPS, 5
  questions: 1011 → 413 ms).
- **option_isolation** (a capacity knob): every option span becomes its own sub-branch
  (sees state + instruction + itself only), option spans share position ids, `<decide>`
  sits at one fixed position after the longest span — per-option representations and
  the decide readout become **permutation-invariant by construction** (options never
  see each other; contrast AgentJev's permutation-equivariant set head, which adds
  cross-candidate attention).
- **Unforgeable option boundaries** (`user_tokens`): caller text is tokenized with
  `<|name|>`-style special tokens rewritten to a broken form first, so user data can
  never mint delimiters/control tokens.
- **`date_facts`** (opt-in `KEV_DATE_FACTS=1`): deterministic preprocessing that
  appends every pairwise day-count between absolute dates found in the state ("August
  3, 2026 is 12 days after July 22, 2026.") because "the model cannot subtract dates
  reliably (issue #8); it can use a stated day count" — a modelless patch over a model
  weakness, our house philosophy in one function.
- **Confidence formulas** (`api.py`): choice = `(max(p) − 1/K) / (1 − 1/K)` (winning
  probability rescaled above the uniform baseline); score = `1 − E|level − mode| /
  (L−1)`. The model card says plainly: "neither is a calibrated probability of
  correctness." (Their homepage marketing says "calibrated probability" — the card
  disclaims what the blog claims; our G1 axis can settle that empirically.)
- **Serving**: plain `ThreadingHTTPServer` behind a lock; 1 MB body cap; non-finite
  JSON numbers rejected; strict encoding (over-context = 400 error, never silent crop).
  Runners: their Python (port 8000), a llama.cpp fork branch `edlm` with
  `POST /v1/systemone` (port 8097), an Ollama fork. 32,768-token model capacity,
  16,384 default everywhere. On Apple Silicon they force `GGML_METAL_TENSOR_DISABLE=1`
  ("the Metal tensor matmul path produced incorrect long-input results in validation" —
  the same llama.cpp Metal seam our perf league tracks).

## Their own validation file (honest, quoted from `validation/RESULTS.md`, 2026-10-06)

"**Result: not signed off.**" — a 15,644-token state with the marker in the FIRST third
returned the wrong option on both GGUF runners (Python SIGSEGV'd entirely; marker in
the final third answered correctly): a **retrieval-correctness failure, not a capacity
rejection**. BF16-vs-GGUF max per-option probability delta 0.0050 (sample) / 0.0094
(255 options). An earlier concurrent native+Ollama run produced a spurious 0.5/0.5
distribution (`kIOGPUCommandBufferCallbackErrorOutOfMemory`) — do not run 16K GGUF
contexts concurrently on a 48 GiB machine. Lane expectations from this: det can wobble
under load (the AgentJev bf16-HTTP-wobble class), long-state cases are fragile, and a
GGUF-served lane is a different numeric posture than BF16 (disclose which served).

## Landscape position

- Sep 24 2026 blog: "Introducing Drex, a small model that decides instead of writing" —
  original Drex, **under 6B**, topped the public Decision Index.
- This release (the one we can serve): 8B Efficient-DLM backbone, 32K window, open
  weights CC BY-NC + MIT code.
- ~Oct 6 2026: **Drex 1.5** on the homepage — "under 10B", **128K** tokens per
  request, Decision Index **0.2.1** score **58.28** vs Jev 1.13.0 **57.91** vs Rune
  26B-A4B v3 57.44. 128K ≠ this release's 32K, so the hosted 1.5 is a separate/newer
  artifact; a local lane measures THIS release only (say so in every table).
- Nace.AI: $21.5M raise (May 2026); a hypernetwork knowledge-injection research line
  (Jul 2026, "hypernetworks beat LoRA and full FT on OOD generalization" — adjacent
  riir-train intel, not consumed here); "Kev-9B" (their earlier family name — the
  `KEV_*` env vars and `code/kev/` survive it) already appeared in the Clef blog table
  our Research 005 recorded (BANKING77 84.83).

## Signal-diff vs what we ship (§3.6 discipline)

| their component | our cousin | diff | verdict |
|---|---|---|---|
| TypeSafe wire `state`+`choice/noul/score` | `decision_wire` (choice/score/noul, per-option probabilities, **abstain first-class**) | abstention + calibrated confidence are ours alone; their confidence is an explicit uncalibrated rescaling | covered |
| packed multi-question one-forward-pass | laya lane `system_one` / `system_one_packed` (one bidirectional sequence; `typed_case_split` example measures exactly this split) | same serving shape on a bidirectional encoder; theirs is the block-causal causal-backbone analog | covered |
| pointer head (marker-query × option-key dot product) | modelless engine scores each option independently (LZ4 delta + cosine-to-centroid) — invariant by construction; AgentJev's set head was Research-002-covered the same way | their head adds NO cross-option context (option_isolation makes even the backbone path permutation-invariant) — simpler than AgentJev's, not new capability | covered modellessly |
| state-prefix KV reuse (exact by construction) | uniform-content cache (decode-side, riir-ai); **Research 002's open GAP**: "candidate-branch prefix reuse on a CAUSAL backbone for scoring workloads — not shipped anywhere in the stack" | Drex `probs_and_prefix`/`probs_with_prefix` is a SECOND reference implementation (with AgentJev `prefix.py`); still only actionable if a causal decision lane lands in riir-infer | gap stays open, deferred |
| `date_facts` deterministic preprocessing | modelless-patch philosophy; no date-heavy suite on our board | nothing to ship (we do not serve their model); pattern noted | note only |
| unforgeable delimiters (special-token rewrite) | decision_wire is JSON-typed (no token surface in the modelless engine); laya renders text programmatically | a prompt-injection consideration for ANY text-rendering lane, ours included | note only |
| strict over-context refusal (400, never crop) | modelless lane has no token budget; laya lane crops per reference behavior | refuse-vs-crop is a wire-semantics choice per lane | note only |
| per-primitive temperature at checkpoint | `Calibration{method, temperature}`; laya per-(kind, option-count) refit (Research 576) | same class | covered |

## Verdict (per track)

- **Modelless/inference track: GAIN** — no primitive to implement (every mechanism is
  covered or note-only), one actionable arena addition: the `--drex` comparison lane
  (issue 073) with the calibration readout as its owned data point (their card disclaims
  calibration; their marketing claims it; G1-style ECE on our splits is the honest
  third-party read — exactly Research 005's Clef risk-#6 posture).
- **Training track: nothing filed** — no recipe ships in the repo (only
  `trainable_parameters` and a calibration script name); the AR→dLM conversion is
  NVIDIA's Efficient-DLM work, not consumable training method. No Path-0.5 plan.
- **Tiers**: not Super-GOAT (no new mechanism, no new capability class — Q1/Q2 fail),
  not GOAT (no provable gain over anything we run), **Gain** (arena + intel). MOAT
  gate: `riir-reflex` decision-serving row — "honest comparison arenas" is literally
  the moat; this lands inside it. Public-safe: external public model + our public
  harness; nothing private-tier.

## Lane design (issue 073, summary)

Mirror of `--agentjev`/`--clm` (same TypeSafe wire): `src/lanes/drex.rs` +
`--drex` harness flag + `SuiteResult.drex` + `RunMeta.drex_lane` posture; std-only
`http_mini` transport; `DREX_SERVE_URL` (default `http://127.0.0.1:8000`, their Python
server; the llama.cpp `edlm` port 8097 equally valid — disclose which served);
agentjev laws throughout (loud refusal without the server, fixed-throwaway warmup,
determinism rerun, the laya trim cap). Owned cells: gold-label accuracy per primitive
on typed_decisions (board context: laya-riir 0.7445 / AgentJev 0.7715 det✗ / rethink
typed 0.7550), **ECE/Brier of their per-option probabilities AND their `confidence`
fields**, round-trip latency with serving posture + box state quoted. License law in
the issue header: CC BY-NC ⇒ measurement only.

## Prior art (§4 sweep)

- Searched "nace-ai Drex DLM decision model" + "NVIDIA Efficient-DLM-8B" (2026-10-07):
  the HF card, the two Nace blogs, and arXiv:2512.14067 (Efficient-DLM: AR→dLM
  conversion; 8B = +5.4% acc / 4.5× throughput vs Dream 7B). No independent bench
  coverage of Drex beyond their own tables yet — our lane would be an early
  third-party read.
- In-workspace: `drex` greps zero prior distill; `Kev-9B` appears once (Research 005,
  the Clef blog table). Research 002 (AgentJev landscape) + 005 (Clef/JDI) are the
  cousin notes; 003 (OpenThai systemone lane) is the lane-pattern precedent.

## Fusion

Research 002 × Research 005 × this note: the arena is the fusion — one wire, one
split, N lanes, with the calibration axis (G1/ECE + Report-the-Floor) applied to EVERY
entrant including ourselves. Drex is also the second data point (with AgentJev) that
the category is converging on marker-anchored per-option readouts over a shared state
— evidence the reflex modelless stance (independent per-option scoring, no weights) is
the right floor to sit under all of them.
