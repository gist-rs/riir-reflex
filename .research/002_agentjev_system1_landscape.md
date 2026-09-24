# AgentJev / System-One typed-decision landscape — competitor distill

**Status:** RECORD — external-repo distill, landed as reflex `.issues/025` + riir-ai `.issues/1005`.

> External source: `malevrigns/agent-jev` @ `a965ca8ff06ccabc0c796dca5447b55cc2069cee`
> (2026-09-23, Apache-2.0). Weights: HF `aimeigaoshou/agent-jev` (bf16
> safetensors, step-600 run, `temperatures.json`). Cloned into
> `katgpt-rs/.raw/agent-jev` for the verdict; removed after commit.

## TL;DR

A new open "System One" typed-decision model (Qwen3-0.6B backbone, LM head
removed, permutation-equivariant candidate-set head, zero decoded tokens)
publishes **79.25%** on the Typed Decisions official test split — the exact
400-case / 2000-question split reflex's harness already runs — against the
laya-typed checkpoint we ported (their table: 77.00% card-copied; our measured
laya-riir·typed: **74.45%**). The category now has three named players
(TypeSafe's Jev = namesake, Laya, AgentJev) plus an ecosystem (CoreML/ONNX
ports, OpenAI-compatible bridges, awesome lists). Reflex is the fourth entry
with a different axis: modelless core, abstention-first, conformal-floor
calibration. Nothing here invalidates our positioning — but the specialist
accuracy bar moved to ~79% and our README/bench docs cite only laya.

## Hard numbers (same split shape: 400 cases / 2000 questions)

| lane | source | acc | per-primitive | latency |
|---|---|---|---|---|
| AgentJev-0.6B | their table (published) | **79.25%** | bool 88.83 / choice 75.33 / score 75.00 | ~60–70 ms p50/case, cuda:0 |
| Laya (published ckpt) | their table, card-copied, not re-scored | 77.00% | — | — |
| TypeSafe Jev 1.13.0 | their table, zero-shot generalist | 72.7% | — | — |
| laya-riir·typed | OUR harness (`aa37823`, m3, metal, 2026-09-23) | **74.45%** | choice 73.33 / noul 78.50 / score 72.25 | 1312 ms p50/case |
| laya-python·typed | OUR harness (torch oracle) | 74.45% (identical — G5 holding) | same | 279 ms p50/case |
| modelless | OUR harness | 31.90% | choice 18.67 / noul 53.17 / score 25.87 | **0.472 ms** p50/case |

Protocol differences (footnote-grade, not excuses): their accuracy is
agreement with the public teacher argmax and their run held out 120 dev + 120
cal cases with checkpoint selected on dev soft-CE before opening test; our
harness scores gold labels with no per-benchmark refit beyond the standard
gate fit. Their Laya row is copied from the dataset card.

Wide-load measurement (their box): 64 choice + 1 boolean, 66 paths, 33,547
path tokens — unshared 609.65 ms vs **shared-prefix 298.91 ms**, max prob
delta 0.000508, backbone token-ops 33,547 → 2,551 (**92.4% reduction**).
Shared path engages at ≥8 candidates and prefix ≥128 tokens.

## What their stack is

- Backbone Qwen3-0.6B, no LM head; each candidate = causal path
  `[state][question][candidate]`, hidden read at candidate's last token.
- Head: Linear 1024→256 → 2-layer transformer d=256 **with no positional
  embeddings over the candidate axis** (permutation-equivariant by
  construction) → Linear 256→1024 residual → ScalarScorer
  (RMSNorm→1024→256→SiLU→256→1). Per-question softmax.
- Training: soft CE + 0.1·Brier; a permutation KL regularizer (re-score a
  permuted candidate order, align back — guards bf16 numerics); LLRD hooks;
  per-primitive temperature fit on 120 calibration cases.
- Serving: loopback HTTP, batch ≤32 states / 128 questions / 1024 candidate
  paths; **2048-token context, over-length REFUSED** (no silent crop);
  distinct-candidate requirement; `generated_tokens: 0`.
- Consumer surface: stdlib python client + a Claude Code `PreToolUse` hook
  (Bash/Write/Edit; blocks only on score level 3 AND boolean-unsafe;
  fail-open when the server is down).
- `routing.py`: 6-route action vocabulary (read/search/edit/test/finish/
  delegate) + `compact_state` — per-field token caps VALIDATION 26 /
  PREVIOUS_ACTION 32 / PROGRESS 18 / LATEST_OBSERVATION 94 / TASK 48,
  tail-caps for recency fields, head-caps for stable ones, shrink-longest-
  field-by-4 under budget, recent observations preserved before task text.

## Signal-diff vs what we already ship (§3.6 discipline)

| their component | our cousin | diff | verdict |
|---|---|---|---|
| typed wire bool/choice/score | `decision_wire` choice/score/noul, full per-option probabilities, abstain first-class | margin (top−second) is derivable from our full distribution — no wire change; abstention is ours alone | covered |
| per-primitive temperature | `Calibration{method, temperature}`; laya per-(kind, option-count) refit (Research 576) | same class | covered |
| permutation-equivariant set head | modelless engine scores each option independently (LZ4 delta + cosine-to-centroid blend) | independent per-option scoring = invariant by construction; their cross-candidate attention context is model-based-only | covered modellessly |
| shared-prefix KV branching | uniform-content cache (decode-side, riir-ai); laya = one bidirectional sequence | candidate-branch prefix reuse on a CAUSAL backbone for scoring workloads — not shipped anywhere in the stack | **GAP** — actionable only if a causal decision lane lands (issue 025 stretch arm) |
| bounded state serialization | riir-agents task/shared_context (no token-budgeted field schema) | per-field caps + tail-preserve + never-drop-a-field | **GAP** → riir-ai issue 1005 |
| PreToolUse hook | reflex `/decide` edge; healer guard gates | a hook CLIENT — adoption funnel, not engine tech | product item → issue 025 |
| over-length refusal | modelless lane has no token budget (N/A); laya lane crops per reference behavior | refuse-vs-crop is a wire-semantics choice | note only |

## Fusion

`Lane::Hybrid` in decision_wire already sketches "specialist proposes,
modelless gates/abstains". The category answer: ANY open decision model
(laya today, AgentJev-class tomorrow) can sit behind `decision_wire` as a
lane; our moat is the abstention + conformal-floor calibration + modelless
latency floor + game heads, not the specialist's raw accuracy.

## Routing (files created)

- reflex `.issues/025_agentjev_system1_positioning.md` — bench-doc row +
  README landscape + (owner-gated stretch) lane-3 feasibility: serve
  `aimeigaoshou/agent-jev` Apache-2.0 weights via riir-infer (Qwen3-0.6B
  backbone port + set head + shared-prefix KV; their `prefix.py` is the
  reference implementation).
- riir-ai `.issues/1005_riir_agents_typed_decision_gates.md` — decision_wire
  typed gates at agent-loop route points + `compact_state`-style bounded
  state serialization.

## riir-train deferral (justified)

Own ternary ~0.6B decision-head training (Bonsai lineage) deferred: the
Apache-2.0 weights are consumable directly, the modelless lane is the
differentiator, and training only pays for ternary/edge ownership of a
decision head. Reopen trigger: lane-3 lands AND a quantized edge decision
head becomes a product requirement.

## Prior art (§4 sweep)

- TypeSafe "Jev" — the category namesake; their 1.13.0 generalist row is in
  AgentJev's own table. Ecosystem: awesomejev.com, jevai.dev, CoreML/ONNX
  laya ports, OpenAI-compatible bridges (searched 2026-09-24).
- Permutation-equivariant set heads: Set Transformer (Lee et al., 2019).
- Shared-prefix KV reuse: vLLM prefix caching / SGLang RadixAttention class,
  applied here to candidate scoring.
- No-decode classification heads on LM trunks: the reward-model / verifier
  lineage. AgentJev's contribution is the combination aimed at agent gates.

No PoC required: no quality-parity claim is made by us — their numbers are
cited as published, ours as measured, protocol differences flagged.
