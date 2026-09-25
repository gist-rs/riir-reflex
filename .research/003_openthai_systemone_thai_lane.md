# OpenThai-SystemOne — the Thai+English System-One lane (competitor distill + Thai boundary verdict)

**Status:** DISTILLED — pending owner decision; plan `.plans/003_thai_lane_research.md` filed, implementation NOT started (research-sake posture per owner ask: "we not focus in Thai but just for research sake").

> External source: `iapp-technology/openthai-systemone` @ `5d04bcca0c58bd10e7dac2d3d369d8f760bea6cf` (2026-09-22, Apache-2.0). Weights: HF `iapp/OpenThai-SystemOne` (Qwen3.5-0.8B text tower + 256-slot head, Apache-2.0). Cloned into `.raw/openthai-systemone` for the verdict; removed after commit. Sponsor: Siam AI (H100s).

## TL;DR

The **fifth named entry** in the System-One typed-decision family (TypeSafe Jev → Laya → AgentJev → Bespoke Nimble → OpenThai), and the first **Thai+English specialist**: a Qwen3.5-0.8B text tower whose LM head is replaced by a 256-way slot head, Thai-continued-pretrained (5B Thai-heavy tokens), answering `choice`/`score`/`noul` questions about an arbitrary state in ONE forward pass, over a wire that mirrors TypeSafe's `POST /v1/systemone` — nearly reflex's native `decision_wire` vocabulary, closer than AgentJev's dialect.

The owner's actual question — *"plan to handle Thai without contaminating EN code, e.g. around tokenizer and other"* — has a measured three-part answer, and **none of it is tokenizer surgery**:

1. **Our lanes already degrade safely on Thai** (pinned, not accidental): the laya script-detector router (ported `lang.rs`/`router.rs`) collapses the EN checkpoints on non-Latin script (frozen G5 row `ml-thai-collapse`, conf 0.0002 → abstain), and the modelless embedder hashes Thai deterministically (`embed.rs` passes non-ASCII bytes through) — but its **posture is UNPINNED** (expected: clause-unit hash bag → distance-gate abstain; a confident answer would be a finding). Plan 003 Phase 1 pins it.
2. **The laya multilingual checkpoint already answers Thai** (frozen G5 row `ml-thai`).
3. **Real Thai capability = an external specialist consumed as a measurement lane** — their FastAPI server serves, our Rust measures (the agentjev/clm lane family law). Zero deps, zero EN-path edits.

**Boundary verdict** (the suggest the owner asked for): lane = self-contained module + runtime `--openthai` flag, **no feature gate — by the agentjev law, which is imports-based, not zero-dep-based**: `agentjev` is ungated because it imports only `crate::harness::suites` (surface the default build already activates); `clm` is gated *despite being zero-new-packages* because it imports `katgpt_core::decision_wire`, which `--no-default-features` does not activate (`clm-lane = ["katgpt-core/decision_wire"]`). OpenThai is agentjev-shaped (builds from `SuiteCase`) → ungated. **The trigger, recorded**: if any openthai module or test ever imports `katgpt_core::decision_wire`, the lane takes `openthai-lane = ["katgpt-core/decision_wire"]` (the clm-lane shape) — G-ISO-4 then yields to it rather than forcing the wrong answer. Any FUTURE Thai *engine* code = feature `thai`, default-off, one module tree, byte-identity-gated — **deferred** with an explicit reopen trigger; a **new crate only if** that arm ever needs the vocab/tokenizers tree (`crates/riir-infer-thai`, the laya-substrate precedent) — not today.

## What their stack is

- **Backbone**: Qwen3.5-0.8B text tower (vision encoder dropped, LM head removed). Qwen3.5 is a **gated-delta-net hybrid** (`use_reference_kernels()` unwraps `chunk_gated_delta_rule`/causal-conv Triton paths for CPU/MPS) — the *second* GDN-based decision model in the family, reinforcing the GDN-decision-head pattern our GDN lanes (riir-ai/riir-infer) already ride.
- **Head**: `SlotHead` = one `Linear(H→256)` read at every `<|ts_answer|>` hidden state. Slot *i* = "the option introduced by `<|ts_opt_i|>`"; **slot 255 = abstain** (native in-head abstention — our family value). Mask slots ≥ k; softmax over the k options (+abstain).
- **Formatting** (`formatting.py`, 298 LOC — the transferable substrate): one causal sequence
  `<|ts_state|> {state} <|ts_q|><|ts_choice|> {instructions} <||ts_opt_0|> name: desc … <|ts_answer|>` — **all questions answered from ONE forward pass** (vs AgentJev's per-candidate causal paths). 6+256 special tokens added to the STOCK Qwen3.5 tokenizer; `sanitize()` zero-width-space-injects against control-token smuggling; state JSON-dumped `ensure_ascii=False`.
- **NO Thai tokenizer code anywhere.** Stock Qwen3.5 byte-level BPE; Thai-ness lives in 5B Thai-heavy CPT tokens + vocab coverage. (The tokenizer question dissolves: byte-level BPE encodes any UTF-8 — quality rides training data, not code.)
- **Calibration**: stage-3 frozen backbone, head + per-question-type learned log-temperatures (CE + Brier 1.0): choice 1.062 / noul 1.047 / score 1.008.
- **Order-invariance**: `order_invariant` averages the answer over ≤32 **cyclic option shifts** (auto: 8 perms when choice ≥ 11 options, ~2× latency) — the inference-time answer to position bias, where AgentJev trained a permutation-equivariant head. Cheap construction, runtime-cost tradeoff.
- **Wire**: `{state: str|dict|list, questions: {qid: {type: choice|score|noul, instructions, criteria}}}` → `{answers: {qid: {…, probabilities, confidence, abstain?}}, usage}`. Choice ≤ 255 options (slot limit), score 2..10 levels (same bounds as our harness), noul = p(yes). Response `probabilities` are **renormalized over the k options** (abstain reported separately as an OpenThai extension).

## Numbers (their model card, zero-shot, cited not re-scored)

Public benchmark (Bespoke Nimble's 13-subset `PUBLIC_BENCHMARKS` protocol): **OpenThai-0.8B 74.3 macro** vs Nimble-9B 74.8 vs Jev 1.13.0 76.0. A 0.8B model at 9B parity. Subsets we ALREADY run: `massive-en-US` (them 88.3) and `xnli` (them 89.0) — free EN cross-check lanes.

Thai held-out (never in training): MASSIVE-th intent 60-way **90.0** · Prachathai67k topics **98.1**/94.2 · XNLI-th **77.1**/84.3 · SIB-200 Thai 77.9 · wongnai stars MAE 0.44 · **wisesight 4-class 51.6 (weakest, ECE 0.353 — their own honest miss)**. All public HF datasets — fetchable by our existing `scripts/fetch_datasets.sh` datasets-server mechanism.

Latency: 44 ms batch-1 255-option (H100, shared); 154 ms M3 Max MPS (3-question Thai ticket) — a laptop-reachable specialist.

## The Thai posture of OUR lanes (measured / pinned state)

| lane | Thai input behavior | evidence |
|---|---|---|
| laya english / typed-decisions | script-detected → collapse conf 0.0002 → abstain | frozen G5 row `ml-thai-collapse` |
| laya multilingual | **answers** | frozen G5 row `ml-thai` |
| modelless | deterministic clause-unit hashed bag (ASCII-whitespace split → whole Thai clauses as tokens) → **expected** distance-gate abstain — UNPINNED | `src/embed.rs`; plan 003 Ph.1 pins it |
| openthai lane (new) | their specialist answers | plan 003 |

Thai between spaces = one giant "word" to `embed.rs`; FNV-hashed deterministically; no crash, no NaN (zero-vector law). The risk worth pinning: a long Thai text fills many hash buckets and *could* land near a corpus centroid with false confidence. Phase 1 measures and records the honest verdict either way.

## Signal-diff vs what we already ship (§3.6 discipline)

| their component | our cousin | diff | verdict |
|---|---|---|---|
| typed wire choice/score/noul + instructions + criteria | `decision_wire` + harness `SuiteQuestion` | near-native vocabulary match (noul shares our spelling, not TypeSafe's boolean) — the smallest lane adapter yet | covered (wire) |
| abstain slot 255 | first-class abstention + `distance_abstain` | same family value, different mechanism (in-head slot vs fused gate) | covered |
| one-forward all-questions slot readout | laya batched system_one (issue 020 T5) | both single-pass; slot/control-token layout is their recipe shape | covered (lane-side) |
| stock BPE, Thai via CPT | laya pinned BPE 0.22 + multilingual ckpt | **no tokenizer work exists to port** — Thai capability is weights, not code | nothing to do |
| cyclic-permutation order invariance | agentjev's trained equivariant head (recorded); our modelless scores options independently (invariant by construction) | third answer to position bias in the family | note only |
| Thai CPT + decision SFT recipe | riir-train backlog (002's deferral row) | 5B-token CPT on 1×H100 ≈ 1 day + 0.5d SFT + hours calib — recipe recorded below | deferred (riir-train) |

## Boundary verdict — "handle Thai without contaminating EN code"

**Contamination surface is three code seams, each with its own answer:**

| seam | boundary | why |
|---|---|---|
| comparison lane (measurement) | self-contained `src/lanes/openthai.rs` + runtime `--openthai` flag, **no feature gate** | the agentjev law (imports-based): builds from `crate::harness::suites` only — surface the default build already activates; harness-only, never in the release set; EN lanes' files untouched (isolation chosen OVER DRY-extracting a shared `lanes/http.rs` — the third HTTP client's extraction is a deferred task, deliberately). NOT a zero-dep claim: clm is zero-new-packages yet gated on its `decision_wire` import |
| posture/safety pins | tests only (`tests/`), zero runtime code | pins current safe degradation as contract |
| engine Thai arm (segmentation etc.) | feature `thai`, default-off, one module tree, byte-identity gate when off — **DEFERRED, `- [-]`** | no product focus today; reopen trigger = owner call. Substrate candidates recorded: (a) unigram Viterbi over a Thai-covering vocab (in-stack precedent: SentencePiece unigram Viterbi, riir-ai Issue 400; gemma-2 vocab segments `'สวัสดี'`, katgpt-rs Research 565 Row D), (b) newmm/longest-match dictionary (needs an open-licensed wordlist asset in a now-PUBLIC repo), (c) Thai char-n-gram hashing (no segmentation). **New crate `crates/riir-infer-thai` only if the arm needs the vocab/tokenizers tree** (the laya-substrate precedent) — otherwise reflex-local feature module |

**Non-contamination gates** (plan 003's contract, enforced as tests): G-ISO-1 serve-path byte-identity (lane code unreachable from the `reflex` bin — pinned); G-ISO-2 default harness suite set unchanged (thai suites opt-in via `--suites`); G-ISO-3 G5 parity fixtures untouched (`ml-thai`/`ml-thai-collapse` stay frozen); G-ISO-4 `Cargo.toml` unchanged **while the lane stays agentjev-shaped** (zero new packages AND no non-default katgpt-core imports) — the decision_wire trigger above overrides it if ever fired.

## Fusion

- The arena gains its **first Thai board** (wisesight + SIB-200 Thai + MASSIVE-th probes) with EN boards byte-identical — honest-comparison moat, the reflex lane.
- The `--openthai` lane also measures **EN** on suites we already fetch (`massive_intent_en`, `xnli_en`) → a fifth column on existing boards, free.
- GDN-decision-head pattern: second external datum (after AgentJev's Qwen3-0.6B… which was dense; OpenThai's Qwen3.5 GDN hybrid is the first GDN one) → feeds the riir-train deferred row.

## riir-train deferral (justified, recipe recorded)

Own Thai decision-head training deferred (extends 002's deferral): Apache-2.0 weights are consumable directly; the modelless lane + abstention-first remains our differentiator. Recipe on record if ever reopened: surgery (drop vision+LM head, add 256 tokens, digit-mean init) → CPT 5B Thai-heavy tokens 1×H100 ~1d → decision SFT (synthetic 600k via OpenAI-compatible endpoint) ~0.5d → calib (head + per-type temps, CE+Brier) hours. Reopen trigger: same as 002 (a quantized edge decision head becomes a product requirement) AND a Thai product surface exists.

## Prior art (§4 sweep)

- **Bespoke Nimble-9B** (2026-09-10, Apache-2.0): LoRA on Qwen3.5-9B, 2,676 contrastive synthetic examples, 2-day train, ~100 ms — the 9B generalist row in OpenThai's table.
- **TypeSafe Jev 1.13.0** — category namesake; the production generalist (76.0 macro on the same bench).
- Thai word segmentation families for the deferred engine arm: newmm, longest-match, deepcut, tcc/etcc (pythainlp bundles) — plus in-stack precedent: unigram Viterbi (riir-ai Issue 400 / katgpt-rs Research 565 Row D + its `Bm25Index::with_tokenizer` extension point) and Darts double-array trie (katgpt-rs Research 137, citing the Thai linux.thai.net reference).
- No Thai-specific typed-decision model prior art beyond OpenThai itself (first mover, 2026-09-22; searched 2026-09-25).

No PoC required: no quality-parity claim is made by us — their numbers cited as published, ours to be measured by the lane when it runs (plan 003); the lane IS the measurement instrument.
