# Research 009 — LiquidAI d1 (d1-3B + d1-omni-600M): the foundation-vendor entrant, open weights, at our encoder scale

**Status:** RECORD — external-release distill (owner ask, 2026-10-08). Lane actionable:
`.issues/078_d1_comparison_lane.md` (`--d1` lane) + `.issues/077_option_permutation_spread_probe.md`
(the probe — lane-agnostic, runnable today against the existing lanes).
Training-recipe half: `../riir-train/.research/471_D1_PostTraining_Recipe_vs_Typed_Head.md`.

> Release sources (blog published 2026-10-07; d1-3B was scored with the official
> Decision Index scorer per their card — their claim, not a board submission).

## TL;DR

LiquidAI open-weighted the **d1 decision-model family** — the category our arena benchmarks
(Research 002 AgentJev / 005 Clef / 008 Drex) now has its foundation-vendor entrant, and the
fourth vendor to converge on the TypeSafe-Jev wire: d1 speaks `decision_wire` natively
(`POST /decisions/v1/systemone`, `state` + `questions{noul|choice|score}`, **zero output
tokens**, multiple named questions over one state in one pass). d1-3B (3.1B, LFM2.5-VL-3B
decoder backbone + SigLIP2 NaFlex 400M vision) reads **48.57** on Decision Index v0.2.1;
**d1-omni-600M (587M, LFM2.5-Encoder-350M bidirectional encoder + 94M vision + 112M audio)
reads 15.95 index / 78.4 mean on 7 public text benchmarks — and it is the FIRST entrant in
the laya scale class (350–600M bidirectional encoder)**; their own vision ablation (45.1
text-only vs 48.57 for the 3B) argues the class is decision-sufficient. Nothing in the
mechanism is new to us — the wire is `decision_wire`'s, the packed one-pass shape is laya's
`system_one`, the per-option distribution readout is what our modelless engine does
invariantly. Actionables: (1) the `--d1` comparison lane (issue 078) — early third-party
read, the arena fusion continuing; (2) an **option-permutation spread probe** (issue 077) —
their recipe names option shuffling as a top lever, so order sensitivity is a MEASURED axis
every lane should carry; (3) the training half — the first vendor recipe published at OUR
encoder scale (riir-train 471). Their board table is a **subset**: it omits Jev 57.91 and
Drex 1.5 58.28 (both above d1-3B on the 0.2.1 board we snapshot in 005/008) — "best
decision model under 10B" is a subset-table claim until a shared-protocol read exists (the
lane).

## The sources (pinned)

| Source | Pin | License | Read |
|---|---|---|---|
| Liquid AI blog "Open d1" | `liquid.ai/blog/d1-open`, fetched 2026-10-08 (published 2026-10-07) | prose | full |
| `LiquidAI/d1-3B` (HF API) | sha `051bcc464b01b9f92942b364d9586b0ef5912432`, created 2026-10-05, modified 2026-10-07 | **`license: other`, `license_name: lfm1.0`** (LFM Open License v1.0 — NOT Apache/MIT; terms not yet read by us) | metadata + card |
| `LiquidAI/d1-omni-600M` (HF card) | web read 2026-10-08 | same family (not separately verified) | full card |
| Liquid docs "Decision Models" | `docs.liquid.ai/lfm/models/decision-models`, fetched 2026-10-08 | — | full (wire + hosted `d1:free` tier) |

Model metadata (HF API): arch `Lfm2VlForConditionalGeneration`, custom code
`modeling_d1.D1Model` via `auto_map`, 3,123,483,888 params BF16, vocab 128,000, context
32,768, vision encoder SigLIP2 NaFlex 400M, base `LiquidAI/LFM2.5-VL-3B`
(`base_model:finetune`), serving code ships IN the repo (`api.py`, `runner.py`, `prompt.py`,
`modeling_d1.py`, `hybrid.py`, `lfm2_vl.py`) — `trust_remote_code=True` required. Tags:
`system-one`, `decision-model`, `calibration`. Cites arXiv:2511.23404 (LFM2 technical
report). No d1-specific arXiv paper exists.

## What d1 is

- **Wire** (theirs, official): `POST /decisions/v1/systemone`,
  `{model, state, questions: {qid: {type: noul|choice|score, instructions, criteria}}}`.
  `noul` → P(yes) (optional `{true, false}` criteria); `choice` → named options
  (`name: description`), answer = pick + **full `probabilities` + `confidence`**; `score` →
  2–10 ordered levels, answer = probability-weighted expected level + `probabilities` +
  `legend`. `system_one(state, questions)` evaluates all named questions over ONE state in
  one pass (state + images encoded once for all questions); `system_one_batch` packs many
  requests with no padding. Response carries `usage.output_tokens: 0` always. Hosted tier
  `d1:free` exists. This is the TypeSafe-Jev wire — the shape of `decision_wire` (Research
  005 §"What Clef is", 008 §"What Drex DLM is").
- **d1-3B**: 3.1B decoder-only (LFM2.5-VL-3B); multimodal text+image; 32K context.
- **d1-omni-600M**: 587M = **381M shared trunk + decision head** (LFM2.5-Encoder-350M,
  bidirectional) + 94M vision (SigLIP2 tower from LFM2.5-VL-450M) + 112M audio (17-layer
  FastConformer); 16K context (text cut to 896 with images); every modality runs the same
  trunk; text+image OR text+audio per request (never both). **The first open decision model
  at our laya lane's scale class.**
- **Latency (their tables, their protocol — warm, median of 20, 1 request at a time):**
  4090 **8 ms**/question with CUDA graphs (`model.compile(mode="reduce-overhead")`; **16 ms
  without**), 21 ms/3 questions, 102 ms/3.4K-token state, 17 ms/384px image, **475 states/s
  packed** (1106/s MI325X); M5 Pro 30 ms; Jetson AGX Thor 16 ms; Jetson Orin Nano 50 ms.
  Day-one llama.cpp support claimed (full NVIDIA stack + Apple/AMD/Qualcomm, NVFP4).
- **Scores (their card, public benchmarks):** Decision Index v0.2.1 — d1-3B **48.57**,
  d1-omni **15.95**. "Benchmarks as decisions" (their internal eval of public datasets,
  ≤1000 rows): d1-3B SQuAD2.0 85.3 / Civil Comments 93.0 / MASSIVE intent 87.3 / PubMedQA
  66.0 / BoolQ 86.7 / XNLI 85.0 / PAWS-X 76.9 / HelpSteer2 36.7 / **mean 82.9** (Decider 4B
  81.1, Decider 2B 77.1); d1-omni mean 78.4. d1-3B 71.8 DecisionBench (eng v1, 23,900 rows)
  and 69.3 "Fast Decisions (dev split)" — **Fast Decisions is unidentified; never mapped to
  our suites**. d1-omni's card EXCLUDES HelpSteer2 ("may overlap with training data") —
  their own contamination honesty. Vision: d1-3B 74.1 on 11 public image benchmarks vs
  backbone 73.9 (retention, not decision gain); **with images removed the same questions
  score 45.1** — a vendor-published blindness/leak probe.
- **Calibration + numeric posture (d1-omni card):** text answers calibrated with
  **per-type temperatures stored in `config.json`** (post-hoc artifact, fitted after
  training); **image and audio answers are the model's softmax as trained** (uncalibrated —
  their own disclosure). fp16 serves **same top answer as fp32 on every checked row** (243
  text / 214 image / 416 audio); **bf16 flips 0.8% of text and 1.7% of audio rows** — the
  vendor publishing the dtype-flip-rate table we demand from lanes (our posture law, now
  with a reference implementation of the disclosure).
- **Training recipe (their blog, quoted in full in riir-train 471):** base-model weight
  averaging (LFM2.5-2.6B + LFM2.5-VL-3B text backbone) → multi-seed/multi-mixture
  fine-tunes → merged again; *"Training on long inputs, shuffling answer options, and fixing
  shortcuts in the data made a bigger difference than more advanced techniques."* d1-omni:
  staged modality additions (audio encoder + adapter with frozen text backbone; vision
  adapter + LoRA **active only when the input included images**, vision encoder frozen) →
  full FT → merge LoRA → **average weights with the previous checkpoint to regularize**
  (WiSE-FT form). Per-type post-hoc temperature at the end.

## Board position — the subset-table law

d1's leaderboard table ("Decision Index 0.2.1", official-scorer run) is **the JDI
balanced_skill column we snapshot in Research 005** — Winnow-12B 50.02 matches our 005
board row exactly. But their table **starts at Winnow-12B**: the entrants ABOVE it on the
0.2.1 board we recorded are absent — **Jev 1.13.0 hosted 57.91** (Research 005 board) and
**Drex 1.5 hosted 58.28 / Rune 26B-A4B v3 57.44** (Research 008 homepage reads, same
edition). "Ahead of every model under 10B" is a subset-table claim: Drex 1.5 reads 58.28
and is "under 10B" per its vendor. Same discipline as Research 008's edition law — every
cross-vendor number stays "their table" until our own lane measures the release on our own
splits. Their latency table is likewise their protocol (warm, CUDA graphs on the 4090 row);
the JDI board's latency block is a different protocol — never side-by-side without the
posture columns.

Scale-class signal worth keeping: d1-omni (587M) scores 45.1 text-only against d1-3B's
48.57 index — ~93% of the 3B's index from ~1/5 the parameters, published by the vendor.
That is direct evidence the 350–600M bidirectional-encoder class (our laya lane) is
decision-sufficient, and it makes d1 the first category recipe at OUR scale (Clef 27B and
Drex 8B were both scale-stretches).

## Signal-diff vs what we ship (§3.6 discipline)

| their component | our cousin | diff | verdict |
|---|---|---|---|
| TypeSafe wire `state` + `noul/choice/score` | `decision_wire` (choice/score/noul, per-option probabilities, **abstain first-class**) | d1 has confidence but NO abstention primitive (their guidance: threshold `confidence` at 0.5 and flag) — abstention + the fused `CorpusDistanceGate` remain ours alone; their text calibration is a shipped per-type artifact (ours: `SigmoidGateCalibrator` + laya per-(kind,count) refit, Research 576) | covered; calibration claim is the lane's G1 test |
| packed multi-question one pass (`system_one`, batch, 64-state packing) | laya `system_one`/`system_one_packed`; modelless engine scores options independently (invariant by construction) | same serving shape; their packed-throughput axis (475/s @ 4090) is a bench FORM we can mirror in `decision_set_goat` | covered; packed axis = lane bench row |
| per-type temperature in `config.json` | laya per-(kind, option-count) refit (Research 576); engine global calibrator | per-TYPE (noul/choice/score) is a coarser partition than laya's per-(kind,count) — nothing to adopt; the transferable bit is calibration-shipped-as-versioned-artifact | covered; note only |
| fp16=0 flips / bf16=0.8% flips disclosure | our serving-posture + bench-preflight laws; openthai lane pins `OPENTHAI_SYSTEMONE_DTYPE` (fp32 vs their bf16 default) | vendor published the flip-rate TABLE we ask lanes for — the d1 lane records the SERVED dtype beside every cell (fp16/fp32 preferred per their own data) | covered; posture column in issue 078 |
| vision-removal blindness probe (45.1 vs 74.1) | our near-duplicate slice-leak probe (text leaks); family-quarantine control cases | modality-removal analog; we have no multimodal lane — the PROBE SHAPE (ablate the claimed input channel, assert the delta) generalizes to any lane | note; probe shape cited in issue 077 |
| option shuffling as a training lever | **no order-sensitivity probe ships in the harness** — laya renders options in fixed order; the modelless engine is permutation-invariant by construction; laya's pooled-state cross-marker attention is the only learnable position-sensitivity surface (riir-train 471 item 3) | the INFERENCE-TIME extraction is the probe: K orderings per case, max spread reported — a lane reading position, not content, will swing | **GAP → actionable (issue 077)** |
| SQuAD2.0-as-noul (85.3) | no SQuAD2.0 suite; abstention axis lives on our own suites + `prompt_injections` | an external abstention-anchored suite (unanswerable questions as noul) | deferred option (issue 078 T6) |
| "Fast Decisions (dev split)" 69.3/76.9 | — | unidentified internal benchmark — never mapped to our suites | n/a |
| hosted `d1:free` HTTPS | lanes are std-only plaintext HTTP/1.1 (`http_mini`; no TLS crate — the clm-lane law) | identical transport finding to Research 005's Clef lane: loopback TLS-terminating forwarder (operator-run) or owner-gated `ureq`; DEFAULT lane posture = their in-repo Python serve on loopback (plaintext), the Drex-lane shape | covered by issue 078 posture |

## Verdict (per track)

- **Modelless/inference track: GAIN** — no new primitive (every mechanism covered or
  note-only above); two actionable arena additions: the `--d1` comparison lane (issue 078
  T1–T3, T5) and the option-permutation spread probe (issue 077, lane-agnostic — it will
  run against every systemone lane, ours included). Owned cells: per-primitive gold-label
  accuracy on typed_decisions + massive_intent_en + xnli_en (d1 PUBLISHED their own numbers
  on MASSIVE 87.3 / XNLI 85.0 — a direct same-benchmark third-party check), ECE/Brier of
  probabilities AND confidence (their per-type-temperature calibration claim = the G1 test,
  Research-005 risk-6 posture), round-trip latency + serving posture + dtype column.
- **Training track: GAIN** — the first vendor recipe at our encoder scale; distilled with
  pre-registered gates in `../riir-train/.research/471_D1_PostTraining_Recipe_vs_Typed_Head.md`
  (per-type temperature at artifact mint, seed/mixture greedy soup, WiSE-FT interpolation,
  option-shuffle augmentation — all CPU-class under Research 464 §3's gate shape; encoder
  soup / encoder FT are the GPU-class tail, trigger-gated). No runs this session; owner
  gate before any retrain window opens.
- **Tiers:** not Super-GOAT (no new mechanism, no new capability class), not GOAT (no
  provable gain over anything we run), **Gain** (arena + intel + recipe). Same verdict
  shape as Research 008.
- **MOAT gate:** `riir-reflex` decision-serving row — "honest comparison arenas" is
  literally the moat; the lane + probe land inside it. `riir-train` active-moat row — the
  recipe note feeds the typed-head lane's next retrain window. No katgpt-rs primitive is
  consumed, so the §1.7 cherry-pick audit does not apply.
- **Public/private tier check:** reflex note + issue = public repo, content is an external
  public model + our public harness + board crosswalk — public-safe. Nothing rethink-class
  (no serving arm, no vessel posture, no ESC). The riir-train note is private-repo and
  references private trainer surfaces — correctly housed.

## Path 0 inventory (advocate-merged)

No-GD advocate returned 17 findings; merged below (coverage judgment = coordinator's, per
§3.6). Model-based advocate's 10 findings → riir-train 471 verbatim-structured.

| # | finding | analog/extracted/deferred |
|---|---|---|
| 1 | `--d1` lane | **extracted** → issue 078 |
| 2 | per-type temperature as engine readout variant | partial (laya per-(kind,count) refit is finer; engine calibrator global) — lane carries the calibration CELLS instead |
| 3 | modality-asymmetric calibration posture row | note (no multimodal lane) |
| 4 | fp16/bf16 flip-rate posture | partial (posture law ships; openthai dtype pin) — posture column in 078 |
| 5 | modality-removal blindness probe | note (probe shape cited; no multimodal input to ablate) |
| 6 | option-permutation spread probe | **GAP → extracted** → issue 077 |
| 7 | packed-throughput bench axis | partial (decision_set_goat; laya packed example) — bench row noted |
| 8 | state-length cost curve + rows-vs-packed parity | partial (typed_case_split; Drex parity pattern) — note |
| 9 | score expected-level readout | covered (wire score semantics identical) |
| 10 | SQuAD2.0-as-noul abstention-anchor suite | deferred → 077 T6 |
| 11 | seven-benchmark panel as suites | partial (massive_intent_en + xnli_en already run) — extra suites deferred |
| 12 | Decision Index edition/split/hardware discipline | covered (Research 008 law; restated in 077 T5) |
| 13 | one-pass-per-frame decision loop | note (demand evidence; our arena heads model it) |
| 14 | calibration-as-committed-artifact | partial (laya frozen fixtures; engine calibrator runtime-fit) — note |
| 15 | edge-latency ladder as feasibility evidence | note (modelless floor strictly cheaper per decision) |
| 16 | uncalibrated-confidence third-party read duty | covered (G1 on BOTH fields in 077 T3) |
| 17 | perf-league watch row | declined — d1 is prefill-only 3B serving, not a decode-league row |

## Lane design (issue 078, summary)

Mirror of `--drex`/`--agentjev` (same TypeSafe wire): `src/lanes/d1.rs` + `--d1` harness
flag + posture in `RunMeta`; std-only `http_mini` transport; `D1_SERVE_URL` default
loopback; reference server = their in-repo Python (`modeling_d1.D1Model`,
`trust_remote_code=True`) behind a plaintext loopback listener — the Drex-lane shape.
Hosted `d1:free` = HTTPS → loopback TLS-terminating forwarder or owner-gated `ureq` (the
Research-005 transport finding verbatim; both owner-gated). Laws carried: loud refusal
without the server, fixed-throwaway warmup, determinism rerun, the laya trim cap,
serving-posture + **dtype column** (their fp16=0/bf16=0.8% table makes fp16/fp32 the
preferred posture), license law (**lfm1.0 "other" — measurement-only posture; read
`LICENSE` before any local weight redistribution; the blog's "without restrictions" is
marketing, the license tag is `other`**), bench_preflight posture beside every latency
number. Board context cells to land beside: laya-typed 0.7445 / AgentJev 0.7715 det✗ /
rethink typed 0.7550; the drex lane's closed calibration verdicts (det ✓ three-posture,
not-calibrated — Benches 125/126) are the template for d1's.

## Prior art (§4 sweep)

- Searched "LiquidAI d1-3B d1-omni decision model benchmark third-party" (2026-10-08):
  NO independent third-party coverage found — results were unrelated benchmarks/marketing
  pages. d1 is one day old; our lane would be an early third-party read (the Research-008
  Drex posture).
- In-workspace: grepped every `.research`/`.plans`/`.docs` for
  `liquidai|liquid ai|lfm2|d1-3b|decision index|decider` — the only hits are this note's
  cousins (005, 008, 003, 007 and the Clef/JDI plan). No prior d1 distillation. Research
  002/005/008 are the controlling category notes.

## Fusion

Research 002 × 005 × 008 × this note: one wire, one split, N lanes — d1 is the fourth
vendor on the TypeSafe-Jev wire and the first whose base is our encoder's own architecture
family (LFM2.5-Encoder). Two structural reads the fusion adds: (1) the category is now
validated by a foundation-model vendor shipping decision models as a FIRST-CLASS product
line with a hosted tier (`d1:free`) — the modelless floor's differentiation (abstention
first-class, sub-ms in-process latency, zero weights) needs the arena's honest cells more
than ever; (2) their published dtype-flip table and blindness ablation are the category
adopting OUR measurement laws — the arena's discipline is becoming the industry's eval
shape, which is the moat compounding.

## Risks / honest caveats

1. **License: `lfm1.0` ("other")** — the blog says "open-weight without restrictions"; the
   HF license tag is `other`/`lfm1.0`. Measurement-only posture is safe regardless; LOCAL
   weight redistribution or product serving is gated on actually reading `LICENSE` (the
   Drex CC BY-NC lesson generalized: read the license, not the blog).
2. **Board claims are vendor-shaped** — subset table (Jev/Drex/Rune rows absent), their
   own scorer run, their latency protocol. Every number above is cited "their table".
3. **Hosted `d1:free`** is HTTPS + network posture + provider ToS — the std-only transport
   law keeps it out of the default lane; loopback forwarder/`ureq` are owner-gated.
4. **"Fast Decisions"** is unidentified — never mapped to our suites; HelpSteer2 excluded
   by the vendor for contamination-possible (their honesty, noted).
5. **bf16 flips 0.8% of text rows (vendor-disclosed)** — the lane records the served dtype;
   a bf16-served cell is a different numeric posture (the Drex GGUF-vs-BF16 class).
6. **The training half (471) is advocate-grounded, not run** — the rethink-side trainer
   pins were read by the advocate sub-agent; the coordinator did not re-open the trainer.
   All gates are pre-registered (464 §3 shape) BEFORE any run; the retrain window is
   owner-gated.
