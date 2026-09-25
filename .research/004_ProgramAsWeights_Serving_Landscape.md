# Research 004: ProgramAsWeights — Serving-Lane Landscape for riir-reflex

> **Source:** [programasweights/programasweights-python](https://github.com/programasweights/programasweights-python) — Python SDK for PAW (ProgramAsWeights): NL spec → compiled tiny neural function, run locally. Cloned to `.raw/programasweights-python`, pinned **@ `74919f6958b127f10776689277f5a74321857b40`** (shallow, MIT). Companion paper: arXiv:2609.04199 "Compile by Training: Turning Natural-Language Specifications into Local Neural Functions" (Deng et al., Sep 2026). Author: Yuntian Deng (hub slugs `yuntian-deng/...`; public hub ~3,800+ programs per his CV).
> **Date:** 2026-09-25, distilled 2026-09-25
> **Status:** Done — verdict **Gain (reflex serving lane)**; actionable item filed as Issue 033 (PAW comparison lane). Engine-level pieces: covered-or-inapplicable, each with a signal-diff below. §5 verdict ping-pong: **AGREE** — ran via the sub-agent reviewer path (`spawn_agent`; the `claude_code` verdict backend was rate-limited this session), reviewer independently re-verified the pinned-sha claims against the clone with zero fabrications found; its two non-blocking clarifications (engine-path network law wording, vessel repo pointer) are applied in this file.
> **Related Research:** katgpt-rs 229 (PAW spec→compile verdict — the SYMBOLIC axis: we compile specs into symbolic constraints, they into weights; F1 SpecAsPruner GOAT), riir-train 117 (PAW ternary spec adapter — the MODEL-BASED response axis: ternary deltas 4400× smaller). This note is the third in the series: the **serving/competitive lens for riir-reflex**.
> **Related Issues:** 033 (this note's filed task — PAW comparison lane) · 029 (GLiNER lane — the subprocess-oracle precedent) · 027 (AgentJev lane — the HTTP-oracle precedent) · 019 (CLM lane)
> **Classification:** Public note; no substrate claim. PAW is consumed (if ever) ONLY as a measured oracle — never a product dependency.

## TL;DR

PAW is the **closest commercial product to reflex's serving story**: compile a natural-language task spec into a small neural program (Q4_0 LoRA adapter over GPT-2 124M / Qwen3-0.6B), run it locally (llama.cpp / browser WASM) or hosted (~150 ms), with program IDs, a hub, offline validation, and an agent-distribution pattern. It validates that the "local, tiny, task-specific neural function" market is real. Reflex is structurally different on every load-bearing axis — **typed decisions over `decision_wire` with first-class abstention, modelless primary lane, ed25519+BLAKE3 vessel authenticity** — and PAW's engineering pieces are either already covered (stronger) in reflex or inapplicable (KV-prefix persistence is a generation-side trick; reflex lanes have no decode). **The one actionable item: add PAW as the third external classifier comparison lane (Issue 033)**, following the established gliner/agentjev oracle pattern, so the arena table measures reflex against the direct commercial competitor.

## 1. What PAW is (at the pinned sha)

**Architecture — the compiler is server-side (closed); the SDK is client + local runtime:**

- **Compile:** NL spec → (server) synthetic data → LoRA finetune → program. Sync compile allows a 2,400 s read timeout; `compile_async` (explicit compiler, e.g. `paw-ft-bs48`) returns submit/status/cancel handles. `paw-4b-qwen3-0.6b` (Standard, ~22 MB programs) vs `paw-4b-gpt2` (Compact, ~5 MB, browser-compatible). Compile response carries `compiler_snapshot`, `pseudo_program_strategy`, `runtime_id`, version fields — programs are versioned server-side.
- **Program artifact (`.paw` format v2, `paw_format.py`):** `b"PAW\x02"` magic + version + JSON metadata + safetensors tensors. Contents: KV-cache prefix ("continuous program"), **discrete pseudo-program text**, prompt token IDs, LoRA adapter weights, generation config, hub metadata.
- **Local runtime (`runtime_llamacpp.py`):** base GGUF (Q6_K Qwen3-0.6B 594 MB / Q8_0 GPT-2 134 MB) loaded once; per-program Q4_0 LoRA adapter via `llama_adapter_lora_init`; prompt template split into prefix/suffix. **The serving trick: the prefix KV state is saved to disk after the first eval (`llama_state_seq_save_file`, atomic `os.replace` after mkstemp) and reloaded on later runs with token-array validation (`llama_state_seq_load_file`, counts must match) — eliminating a 2–3 s cold-start prefix evaluation.** Default `temperature=0.0` (greedy). Caller-supplied `logits_processor` hook (0.4.6) for token-level constraints; processor failures propagate (never silently unconstrained).
- **Browser:** GPT-2 tier runs in WebAssembly (`@programasweights/web`), assets auto-download from HF, then client-side inference.
- **Distribution:** immutable program IDs win over slugs; `resolve/{slug}` endpoint; strict cache (0.4.4): streamed downloads bounded + validated before atomic staged install, versioned runtime manifests, canonical size/SHA-256 for known GGUF runtimes; `offline=True`/`PAW_OFFLINE=1` fails loudly on missing assets (zero network calls); asset endpoints return `202 Retry-After` while a program is still generating (no 404 races).
- **Remote inference:** hosted `POST /api/v1/infer` `{program_id, input}` → `{output}` (~150 ms), `X-API-Key` auth; `remote=True` on the same `paw.function` surface; rejected when combined with offline mode.
- **Agent distribution:** `programasweights.com/AGENTS.md` at a stable public URL + "paste this into your agent's chat" copy — agents read the integration doc automatically.

**Companion paper (arXiv:2609.04199, "compile by training"):** two-stage compile — teacher models synthesize task-specific input/output examples from the spec, then the fast-compiler-initialized adapter is finetuned on them. Model-based track throughout; the training-method response axis is owned by riir-train (R117's ternary spec adapter is the filed analog). Reflex-relevant only as competitive intel.

## 2. Serving-lane signal-diffs — PAW piece vs reflex status (§3.6 discipline)

| PAW piece | Signal it consumes | Reflex counterpart | Verdict |
|---|---|---|---|
| Prefix KV persistence (disk-cached seq state, token-validated) | token IDs of the program's prompt prefix; consumer = llama.cpp decode state | Reflex lanes have **no decode prefix to cache**: laya is a bidirectional encoder (no KV), the modelless lane scores options over a corpus, game heads are boot-fitted from BLAKE3-pinned frozen fixtures (`assets/game_heads/`) — the same ROLE (eliminate per-boot recompute with a validated persisted artifact) ships today | Covered by analog; no KV surface exists to cache. No gap |
| `logits_processor` constrained decoding | per-step token logits; caller supplies masks | `decision_wire` constrains the **question space** at the wire (choice options / score levels / boolean); answers are argmax + calibrated confidence + first-class abstain. PAW's own docs admit callers "must validate the returned result" — free text can fail to parse. Reflex cannot emit an invalid option **by construction** | Covered (stronger). Moat confirmed |
| Offline cache validation (SHA-256, atomic staged install, `offline_ready`) | canonical byte size + SHA-256 of downloaded assets; trust anchored in the server + HF | `reflexer-vessel` (the sibling public engine repo `riir-reflexer`'s vessel crate): **ed25519 signature over `header ‖ payload`** + BLAKE3 digest, two-class artifact header, key-id rotation with fail-closed pins, monotonic apply, single-read bounded open. PAW validates integrity-to-hash; **authenticity** is PAW's gap | Covered (the stack is stricter). Moat confirmed |
| Program hub + slug→ID resolve + 202-while-generating | server-side registry of 3,800+ programs | Reflex is a single engine — no hub; game heads are pinned by BLAKE3 digest in tests, not discoverable artifacts | N/A (product observation, not an engine gap) |
| Hosted remote inference (~150 ms, same program ID as local) | server trust + API key | Reflex's **engine path** is zero-network by law; the harness's external-oracle HTTP measurement lanes (AgentJev 027, CLM 019) are the established exception — their network latency is disclosed inside the hosted cell, never quoted as engine latency | N/A (product observation; the engine-path local-only law stands) |
| Greedy `temperature=0.0` default | sampling distribution | Reflex decision readouts are argmax/sigmoid projections — no sampling step exists | Covered (by absence of the axis) |
| Compile-by-training (arXiv:2609.04199) | teacher-synthesized examples + gradient descent | Model-based track; reflex's model-based lane is the laya pinned-checkpoint forward. Training-method responses → riir-train (R117 owns the spec-adapter axis) | Routed; not a reflex work item |

## 3. Moat comparison (what PAW validates vs where reflex differs)

**Validated by PAW's traction:** (a) "compile an English spec into a local tiny function" is a real, marketed need (classification, extraction, format repair, log triage, intent routing — reflex's exact decision-workload neighborhood); (b) the agent-distribution channel works — a stable AGENTS.md URL that coding agents read is how the product onboards; (c) browser-WASM tiny-model inference is a shippable tier.

**Where reflex structurally differs (and wins on the decision workload):**
- **Typed decisions vs free text.** Every PAW call returns a string the caller must validate; every reflex answer is typed (option key / score level / boolean) or an explicit abstain, with calibrated confidence. The failure mode PAW pushes into caller code is unrepresentable in reflex output.
- **Abstention is a first-class answer.** PAW always answers; reflex's fused abstain (score gate + corpus distance) can say "not in my competence" — the safety axis PAW's product shape lacks.
- **Modelless primary lane.** PAW's every capability is a finetuned artifact (compile = training, server-side, per task). Reflex answers from corpus + fitted heads with no per-task training; the model-based laya lane serves pinned checkpoints with G5 parity gates before any published number.
- **Measurement discipline.** Reflex publishes per-suite tables, box-state provenance (`scripts/bench_preflight.sh` PROVENANCE lines), host-pairwise determinism gates, and external-oracle comparison lanes. PAW's public accuracy claim is a two-word tier table ("Higher"/"Lower"). The comparison lane (Issue 033) is how the gap gets measured instead of asserted.
- **Artifact authenticity.** Vessels are signed; PAW bundles are hash-checked only.

## 4. Actionable items

1. **PAW comparison lane → Issue 033 (filed).** The third external classifier oracle after GLiNER (`--gliner`, subprocess, 2026-09-25 cells) and AgentJev (`--agentjev`, HTTP). PAW is the *commercial* analog: compile-once-per-suite (server-side finetune, API key, cached program ID) + per-question infer (hosted REST or local Python runtime subprocess). Mapping law per the lane precedents: free-form text output → strip → exact-match against the option set; unparseable = recorded refusal, never guessed. 77-way banking77 is the stress cell; spec-authoring quality is a disclosed confound (one NL spec per suite, committed with the lane).
2. **Agent-skill stable-URL pattern → reflex-site recommendation (recorded, not filed).** Reflex already mirrors `reflex-integration` SKILL.md to the site (`sync_mirror.py` law, guard layer 9). PAW's lesson is distribution: advertise one stable, discoverable URL for agents to read, and put the "paste this into your agent" copy in the README. Site-repo work; no engine change.
3. **arXiv:2609.04199 → riir-train competitive intel (cross-ref only).** The reflex stack's model-based response to "compile by training" already exists in skeleton: R117's ternary spec adapter (4400× smaller claim, lossy) and the laya lane's pinned checkpoints. No new plan filed from this note — a teacher-synthesis data pipeline is riir-train scope, and reflex consumes trained artifacts only through the G5-gated lane.

## 5. What NOT to do

- **No hub copy.** Reflex serves one engine with pinned heads; a program marketplace is a different product.
- **No free-form-text lane in the product binary.** The wire law (typed options, abstention) is the moat; PAW's surface is the counterexample, not a feature request.
- **No remote-hosted decision path.** The engine path stays zero-network (measurement-integrity law); external oracles measure over HTTP at the harness seam only.
- **No llama.cpp dependency.** The laya lane owns its forward (`riir-infer-laya`); PAW's runtime is their stack, at most a subprocess oracle behind the lane seam.

## 6. Provenance & cleanup

- Clone: `riir-reflex/.raw/programasweights-python` @ `74919f6958b127f10776689277f5a74321857b40`, MIT, shallow (no history mining needed). Quotes above re-checked against the pinned tree (`paw_format.py`, `runtime_llamacpp.py`, `_remote.py`, `client.py`, `CHANGELOG.md`, `AGENTS.md`, `README.md`).
- Server-side compiler internals are NOT in this repo — everything above is the client-visible contract. Accuracy/latency figures are PAW's own marketing numbers, unverified; measuring them is Issue 033's job.
- Cleanup: `.raw/programasweights-python` removed at commit time (this note is the record).
