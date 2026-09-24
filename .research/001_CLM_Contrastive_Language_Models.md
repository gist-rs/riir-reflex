# Research 001 — CLM: Contrastive Language Models (the first open contrastive System One; a new reflex lane + the flywheel's cheaper recipe)

> **Source:** blog <https://contrastive-lm.notion.site> ("Contrastive Language Models: A System One Model for Fast and Generalizable Decision-Making", Kwok, Kang, Suresh, Saad-Falcon, Pavone, Ré, Mirhoseini — posted 2026-09-23) · github.com/Contrastive-LM/CLM @ `cca045ffdb07b3ebcfe6938537cdeac5e14899c9` (Apache-2.0, full-tree read in `.raw/`, clone deleted after this pin) · HF org `Contrastive-LM` (models `CLM-v0.1-8B` + `deepswe-clm-heads-8k`; datasets `CLM-v0.1-Pretrain-Nemotron` 62.5M rows + `deepswe-clm-train-embeddings-8k` 406k rows — **precomputed embeddings shipped**), all Apache-2.0 per README.
> **Date:** 2026-09-24
> **Status:** DISTILLED — pending owner decision (files `.issues/019_clm_lane_reference_design.md`, the actionable half)
> **Lineage:** Research 562 (TypeSafe Jev, katgpt-rs) → 573 (CUA-S1 open specialist recipe; choice_scorer PoC REFUTED at 101 contexts, reopen = 10⁴–10⁵) → 576 (Laya open generalist, RLCD recipe, pinned at `.docs/laya_reference_pin.md`) → **CLM** (the first open *contrastive* System One; Jev-compatible wire; beats Jev-as-verifier on long-horizon agentic benchmarks at 4–9× lower latency).
> **Classification:** Public sources; distillation internal.

## TL;DR

CLM is the fourth entry in the Jev lineage and the first one whose mechanism is a **CLIP-style contrastive dual-encoder over a frozen LLM backbone**: a state head and an action head (each a ~20M-param MLP, 4096→512, L2-normalised) trained with bidirectional InfoNCE so that `scale · cos(state_head(s), action_head(a))` ranks the taken action first. The two heads then serve as a **zero-shot action classifier**: embed the state once, embed every candidate once, dot products answer. The repo serves it behind the **TypeSafe `POST /v1/systemone` wire verbatim** (`noul`/`choice`/`score`, state + questions → distributions — `schema.py`'s own docstring), which makes it the **cheapest comparison lane reflex can ever add**: no new wire, no new harness, a thin HTTP adapter beside the laya lane. Beyond the lane, three distillables matter more than the model: (1) the **state/action disaggregation cache** (`VectorArena` — one vLLM-KV-style pre-claimed device slab, per-width pools, LRU, head@generation namespacing, hot-reload-safe; measured 2.8× on revisited states, and the whole T-Rex run at p50 16.5 ms vs Jev's 149.8 ms — 9×); (2) the **head-only training recipe** (frozen backbone, precomputed embeddings, bidirectional InfoNCE → hard-negative mid-train → agentic post-train with 40/60 replay; a full 60M-pair pretrain ≈ 1 h on ONE RTX 4090 — hardware this box has); (3) the **scaling-law priors** (test InfoNCE loss is a power law in C/D/N/N_enc; optimal head size N\* ∝ D^1.02 at ~310 tokens/param; encoder size scales strongest). **Verdict: YES (A−) as a reflex lane distill** — file `.issues/019` for the `clm` comparison lane + record the recipe/caching intel for the corpus flywheel (katgpt-rs Proposal 014's specialist-retrain path just got a recipe that is ~2 orders of magnitude cheaper than the RLCD plan at our corpus scale, on hardware we own).

---

## 1. What it is (measured facts, their numbers unless noted)

| Fact | Value |
|---|---|
| Contract | **TypeSafe `POST /v1/systemone` shape verbatim** (`schema.py`: "The wire format is the TypeSafe POST /v1/systemone shape"): `state` (string/object/array) + `questions {id: {type: noul\|choice\|score, …}}` → per-question distributions + confidence |
| Scoring | `softmax(scale · cos(z_s, z_a) / temperature)` per question; `confidence = top_prob − mean(rest)` ("TypeSafe-style confidence", in code); `score` = expected level index |
| Heads | 2× MLP 4096→width→…→512 (≈20M params total), L2-normalised output, `logit_scale` clamped ≤100, GELU, optional LayerNorm/residual; hot-reload on mtime, `generation` counter stamps caches |
| Encoder | **frozen Qwen3-8B, last-token pooling**, served by vLLM `--runner pooling` at `:8090` as OpenAI `/v1/embeddings` (prefix caching on); heads are the ONLY trained part |
| Caching | `VectorArena`: one flat device slab claimed at startup (2% of VRAM default), per-width pools (512-d projections + 4096-d raw), LRU, namespaced `head@generation/state\|action`; a hit skips encoder + H2D copy + head forward |
| Latency (their RTX 4090, server p50) | new state: 28.6→28.0 ms (3 actions) / 28.8→28.1 ms (50); **revisited states: 1.7→0.6 ms (2.8×)**; `~1k candidates: 13× faster than Jev` (blog) |
| T-Rex (in-repo JSON, 5 seeds, 60 s) | **CLM p50 16.5 ms (model 2.6) vs Jev-1.13.0 p50 149.8 ms (model 131.9)**; 3342 vs 1119 decisions; agreement-with-planner 0.658 vs 0.987; both 5/5 survive, score 697 (capped) |
| Benchmarks | Jev-parity on computer-use/gaming/tool-calling at up to **9× lower latency**; as verifier (best-of-N picker, Opus 5/Fable 5 candidates): **DeepSWE 81.6% / Terminal-Bench 2.1 87.6% SOTA**, 4.1–5.7× faster; **Jev scores below pass@1 as a verifier on these long-horizon tasks** |
| Training | bidirectional InfoNCE (CLIP loss); stage 1: 60M Nemotron DQA pairs; stage 2: +30M Gemini-2.5-Flash-Lite synthetic hard negatives; stage 3: 1M agentic trajectories (ADP + Endless-Terminals + LiteCoder-Terminal-SFT) with **40% DQA replay** (replay keeps DQA top-1 68.5% vs 56.2% without) |
| Recipe laws | two-stage beats hard-negatives-from-scratch **69.2% vs 62.4%** at fixed budget ("hard negatives are a refinement, not a substitute"); pre-train alone 52.1% → mid-train lifts to 69.2%; scaling L(X) ≈ (X_c/X)^α_X over {C, D, head-N, encoder-N}; **N\* ∝ D^1.02 (~310 tokens/param)**; encoder size strongest |
| Cost | full 60M-pair pretrain ≈ **1 h on a single RTX 4090** (embeddings precomputed once, heads-only training); fine-tune = `train/finetune.py --task clm\|choice` over precomputed-embedding datasets (shipped on HF) |
| `clm-raw` ablation | cosine in the encoder's own 4096-d space, no head, RAW_SCALE=100 — shipped as a *served model lane* in their own engine (the honest zero-training control) |
| License | code + weights Apache-2.0 |
| Honest limits | Qwen3-8B-class encoder needed for the reference head (heads are encoder+pooling-specific); softmax-over-cosine (not sigmoid); **no abstention anywhere** (the recorded katgpt-rs Research-562 flaw, inherited); Reddit-style Q&A ≠ all decision domains; research repo is separate (scaling/paper figures not public yet); "Jev" baseline is their measurement, not ours |

## 2. Why it lands on riir-reflex (the lane mapping)

1. **The wire is already ours.** `decision_wire` (katgpt-rs Plan 603 T1.2) was built on the Jev/laya vocabulary; CLM speaks the same `systemone` shape (`noul`/`choice`/`score`). A `clm` lane = an adapter like the laya lane: POST state+questions, map answers, publish in `/bench` tables beside modelless + laya. No harness change; byte-identical questions by construction.
2. **It is the first lane that beats the Jev comparison at the Jev game.** Our arena's standing problem: Jev BYO-key lane is the incumbent generalist. CLM claims Jev-parity zero-shot at 9× lower latency and SOTA as a verifier where Jev is below pass@1. That is a *publishable table row* — and the T-Rex harness ships in-repo (Apache-2.0 freedom to re-run under our protocol; their JSONs are reference, never our published numbers).
3. **The disaggregation cache is the game-head pattern, generalised.** Reflex's game heads (Tetris/flappy/lanes) have FIXED option sets; CLM's "4 action embeddings precomputed, 1 forward per step" is the same shape with a principled arena (pre-claimed slab, generation-stamped namespaces, hot-reload). Steal the *pattern* (budget semantics, namespace law, LRU, `/health` hit-rate disclosure), not the Python.
4. **The flywheel's specialist recipe just got cheaper.** katgpt-rs Proposal 014's flywheel: at 10⁴–10⁵ labeled contexts (riir-clippy Issue 125's reopen trigger) the specialist retrains — planned as RLCD (katgpt-rs Research 576 §3). CLM's recipe at that scale: precompute frozen-encoder embeddings ONCE, train a 20M head with InfoNCE (+ hard negatives + 40% replay) — **~1 h on the 4090 this session runs on**. Same shape already validated in-house: `rule_embed_poc` (riir-clippy Issue 126/Bench 099) is a sigmoid-NCE two-tower over a hashed bag with a BLAKE3-sealed frozen artifact — the CLM recipe is that pattern at decision-engine scale, with published sizing laws (N\* ∝ D^1.02).
5. **The verifier framing extends the arena's story.** CLM-as-verifier (best-of-N picking for agentic coding) is a second product axis the site can later expose (`/v1/rank` exists in their API) — the same primitive reflex's `choice` already is.

## 3. Distillables (ranked)

| # | Candidate | Grade | Notes |
|---|---|---|---|
| 1 | **`clm` comparison lane** (adapter + bench row + G-parity harness posture) | **A− (GOAT candidate)** | Benchable on reflex's own harness (byte-identical questions); exact semantics = their published numbers re-measured under OUR protocol on the 4090; magnitude story = the 9×/13× latency claims + T-Rex 16.5 vs 149.8 ms; deterministic serving (cached embeddings, temperature=1). The lane doubles as the flywheel's future consumer. |
| 2 | **VectorArena caching pattern** → the serve edge + game heads | B+ | Pre-claimed arena (never grows → no OOM drift), per-width pools, `head@generation` namespace (hot-reload-safe), LRU, budget as fraction-or-absolute-or-0, `/health` occupancy+hit-rate. Rust shape: a fixed `Vec<f32>` slab + slot map; ~200 LOC, zero deps. Applies to laya lane option embeddings too. |
| 3 | **Head-only InfoNCE recipe + sizing law** (flywheel specialist track) | B | Frozen-backbone + precomputed-embedding training; two-stage hard-negative law (69.2 vs 62.4 — matches our corpus-hardening instinct); 40/60 replay ratio; N\* ∝ D^1.02 (~310 tok/param) sizes the head from the corpus budget. Recorded for the Issue-125 reopen path; NOT a reflex dependency (training stays offline/4090, artifact ships BLAKE3-pinned — the `rule_embed_frozen_v1.bin` law). |
| 4 | **`confidence = top − mean(rest)`** (TypeSafe formula, now in open code) | B− | Cross-check our `SigmoidGateCalibrator` readout against it in the lane's table (we inherit katgpt-rs Bench 817's dispatch; publish both columns; never replace ours). |
| 5 | `clm-raw` lane (cosine in raw encoder space) | C+ | Their own ablation, free to mirror — a zero-training baseline row that isolates the heads' contribution in OUR tables. |
| 6 | `to_text` prose-rendering law (objects → `key: value`, never JSON) | C | The heads-are-trained-on-prose note; applies to any future reflex-trained head. |
| — | DROP: softmax-over-cosine | — | Diverges from the house law (sigmoid, never softmax — per-option independent evidence; decision_wire normalises sigmoid-then-L1). CLM's own temperature semantics (0,100] differ. Record as divergence, do not adopt. |
| — | DROP: vLLM-pooling-server encoder | — | The lane runs THEIR stack for comparison fidelity; OUR modelless/laya lanes stay native. Never a runtime dep (boundary: default build = one code-level dep, katgpt-core). |

**Yield estimate:** 1 lane (issue 019) + 2 pattern adoptions (arena-cache; recipe law recorded) + 2 divergences documented. Realistic landing: the lane + the cache pattern; the recipe rides the flywheel issue in riir-clippy/katgpt-rs territory.

## 4. Dedup evidence (both vocabularies grepped, 2026-09-24)

- `InfoNCE|two_tower|dual.encoder` over workspace `*.rs`: hits are **katgpt-band `conditional_dependence_infonce`** (an InfoNCE CMI *test* — modelless CI estimator, not a trained scorer; distinct) and **katgpt-attn `DualEncoderIndexer`** (katgpt-rs Plan 337 trained sparse-attention indexer; attention stack, not decisions; distinct). Zero decision-lane dual-encoders.
- `rule_embed|span_embed|sigmoid_nce` (riir-clippy): `src/draft/rule_embed.rs` — **the closest cousin** (two-tower span↔rule, sigmoid-NCE, BLAKE3-sealed `rule_embed_frozen_v1.bin`, `LatentMatcher::with_rule_embed`). Same architecture *class*, different domain (healer rule selection) and artifact scale (4096-bucket hashed bag vs 4096-d LLM embedding). CLM is not a duplicate; it is the cousin's validation at 8B scale + the decision-domain instance of it.
- `choice_scorer_poc` (riir-clippy Issue 125): REFUTED at 101 gold contexts — the reopen trigger is corpus scale; CLM's recipe is the scale-up path, not a contradiction of it.
- reflex lanes: modelless (LZ4 drafter + centroid blend), laya (options-at-[MASK] softmax), game heads (fitted decoded heads). No contrastive lane exists. **No collision.**

## 5. Caveats (honest)

- **Research-grade freshness:** blog + repo are ~24 h old (org HF models updated "1 hour ago" at fetch time); the scaling-experiments repo ("main branch") is NOT public; paper is cite-as-notion. Treat every number as vendor-published until our harness re-measures it (the arena's standing rule; T-Rex JSONs are reference cells, not our published numbers).
- **"Jev" comparisons are their measurement** (their endpoint, their protocol, H100 for verifier latency). Our lane publishes OUR cells only.
- **Heads are encoder-bound** ("a head only makes sense with the encoder and pooling it was trained against") — Qwen3-8B last-token pooling. Retraining against OUR frozen laya embeddings (or riir-infer Qwen3.8) is possible (recipe ships) but is a flywheel-scale task, not a lane task.
- **No abstention** in CLM (inherited Jev flaw); our wire keeps abstain first-class — the lane maps their distributions straight, and our gates read the difference.
- 8B-encoder GPU footprint at serve time (vLLM ~0.35 GPU-mem util on their quickstart) — the lane is a *comparison* lane on the 4090, not a default-on product lane.
- Prose-trained heads: state/object rendering law matters for parity (their `to_text`, context-first-question-last layout — copy verbatim in the adapter or scores drift silently).

## 6. Bottom line

**Mine as one lane (issue 019) + two pattern records.** The lane: `clm` comparison lane on the existing harness (adapter + `/bench` row + parity protocol), the first lane that enters the arena already claiming to beat the Jev incumbent at 9× lower latency — and the T-Rex harness to re-run under our protocol ships in the same repo. The patterns: the VectorArena cache law (reflex serve edge + game heads) and the head-only InfoNCE recipe with its sizing law (the corpus flywheel's specialist path at 10⁴–10⁵ contexts — ~1 h on this box's 4090). Divergences held: sigmoid-not-softmax; abstention stays ours; no runtime dep beyond the adapter's HTTP client.

**Fusion idea (novelty TBD, filed in the issue's tail):** CLM-cached-action-embeddings × reflex game heads × the corpus flywheel = a trained-head lane whose action side is BLAKE3-pinned per fixed option set (the game-head digest law) and whose state side is the only live forward — the "System One at 20 Hz" posture the arena story already claims, now with a published recipe.
