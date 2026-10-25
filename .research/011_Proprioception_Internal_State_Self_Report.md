# Research 011: Proprioception — Verifiable Internal-State Self-Report for Serving (Modelless)

> **Source:** "A Sense of Proprioception in Language Models" — Jonas Geiping (ELLIS Tübingen / MPI-IS), arXiv:2610.17495, 2026-10-12. Code: `seal-rg/proprioception` @ `1b43bd3eae30195be42062d5c6812f86188b9a88` (shallow clone to `.raw/`, provenance pinned, removed after distill; two-stage training pipeline + ~25 eval suites incl. `sad`/`perez`/`turpin`/`introspection`/`injection`/`control`/`ablation`).
> **Date:** 2026-10-21
> **Status:** RECORD — modelless track = **Gain** (reflex Issue 094 + riir-infer Issue 1106 filed); self-adaptive track = discipline recorded (label-vintage law); trained track = **conditional issue** (riir-train Issue 639, owner-gated). No promotion claims. §5 ping-pong history: round 1 ABNORMAL — the reviewer executed the commit+push itself instead of issuing a verdict (deviation recorded; content verified byte-intact against the drafts); verdict obtained post-commit, round 2 = **REVISE** with four content defects, all repaired in the immediate follow-up commit.
> **Related Research:** katgpt-rs 388 (jacobian-lens single-layer concept readout — REFUTED as a prefilter on latency 10–70×; **the cost lesson binds every lens-sweep signal in this note**), katgpt-rs 244 (Self-Evolver FaithfulnessProbe), katgpt-rs 609 (FlyBy state_probe — counterfactual value classification), riir-infer 011 (Kolmogorov MDL description length)
> **Related Plans:** none (Gain tier files issues, not plans)
> **Related Issues:** riir-reflex 094 (confidence probes + input-keyed modal floor) · riir-infer 1106 (laya per-layer stability readout) · riir-train 639 (conditional proprioception SFT lane)
> **Classification:** Public

---

## TL;DR

The paper trains open LLMs (LFM2.5-8B-A1B MoE, OLMoE-1B-7B, Qwen3.6-35B-A3B, GLM-4.7-Flash) to **verbally report their own internal states** — MoE routing per layer, next-token top-k/probability/entropy, logit-lens readouts after layer ℓ, and "decision depth" (earliest layer from which the final top prediction stays unchanged). Its decisive contribution for this stack is **not the training — it is the measurement discipline**, and every piece of that discipline ships modelless:

1. **The proprioception ≠ introspection distinction** — a *verifiable* internal-state channel: the reported state is evidentially real (checkable against full internal access), non-transparent (not decodable from output text), and natural (state changes move the report with inputs unchanged).
2. **The live-label law** — labels are read from the SAME forward pass, never stored; stale labels teach *exteroception* (guessing from surface context) instead of internal access.
3. **The input-keyed modal-baseline control** — a running lookup of the modal label keyed by (token, question type, params); any self-report that cannot beat this census baseline is mimicry, not sensing.
4. **Decision depth** — a cheap internal-trajectory scalar (earliest-layer top-1 stabilization) with confidence-signal potential.

**Verdict: Gain.** Two modelless issues + one conditional trained issue. **Not Super-GOAT**: the readout primitives themselves are published prior art (logit lens, 2020; layer-wise entropy confidence for early exit — SimLens; LogitLens4LLMs toolkit), and the paper's own novelty is the *trained verbalization*, which this stack does not need (we own the runtime everywhere we serve).

## The paper in one section

Three domains, 12 question types (Table 2 of the paper), each asked in-conversation about a quoted token position:

| Domain | Types |
|---|---|
| **Routing** (MoE router at layer ℓ) | top expert · k-th rank · share · partner token (nearest token sharing the top expert) |
| **Logits** (final next-token distribution) | top-k token · strongest non-followed alternative · P(actual next) · entropy |
| **Lens** (logit lens after layer ℓ) | lens top-k · lens P · lens entropy · **decision depth** |

Training: **Stage 1 SFT** with a *live* objective — the label is read off the same forward pass being trained on (single-pass when the answer is one token, 2-pass placeholder-refill otherwise); the answer sentence is down-weighted (λ=0.1); a **context lookup table** (modal label per (token, type, params)) down-weighs loss ×0.1 whenever the live label equals the modal answer — the anti-mimicry term; KL(β=2.0) on the first assistant turn anchors capabilities; LR 3e-5 AdamW, bf16. **Stage 2 on-policy SFT** (DAgger): model generates 4-turn conversations, labels refilled same-pass. Results: routing 54–71% conversational accuracy (chance ≈ 3% at 32 experts), decision depth ≈ 41%, entropy ≈ 51%; GSM8K/MATH/IFEval preserved on the flagship; generalizes across phrasings, turn distances, and recursive queries. Two honest limits: models **cannot distinguish float-level tie flips** (batched vs single-sequence routing) — "which expert" is ill-posed inside the tie band; and routing-share questions did not learn. Behavioral readout: fewer "human attribute" claims post-training; linear-probe legibility unchanged.

The paper's own framing cuts our way: the trained lane exists to give *uninstrumented* models a self-report channel. Our stack never serves an uninstrumented model — the runtime can compute every readout in the table directly.

## Why not Super-GOAT (the four gates, honestly)

1. **No prior art? NO.** Layer-wise prediction-stability and entropy-as-confidence are published: logit lens (nostalgebraist 2020; LogitLens4LLMs, arXiv:2503.11667), early-exit confidence from per-layer entropy (SimLens, OpenReview), verbalized-confidence calibration (ConfTuner, Lin et al. 2022 lineage). The paper is the class founder for *trained* proprioception (2026-10-12, three prior art searches found no competitor), but our extractions are **applications of published readouts** to serving lanes. Q1 fails for Super-GOAT.
2. **New behavior class? Partial.** A stability-depth confidence signal would be a new *signal class* (internal-trajectory) beside the corpus-distance/margin signals — a feature addition to the calibration lane, not a new capability.
3. **Product selling point? Weak for the primary engine.** The modelless decision engine has no layers — no internal trajectory exists to read. The signal class applies only to headful/headless encoder lanes (laya, clef), which are comparison/auxiliary surfaces.
4. **Force multiplier? Yes** — connects the harness calibration axis + Report-the-Floor law + the riir-infer capture seam + (private-tier) escalation-adjacent consumers.

**Tier: GOAT-tier fusion, filed as Gain.** The two issued items are cheap, measurable, and fuse with shipped substrate; none is a fundamental new primitive.

## Pinned novelty claim (pre-condition for the prior-art check)

> *Runtime-instrumented internal-trajectory signals (headless layer-stabilization depth on the laya encoder; headful entropy/lens depth on the clef lane) evaluated as decision-serving confidence candidates against the conformal-naive floor and an input-keyed modal-baseline mimicry control — distinguished from SimLens/early-exit (which buys compute, not calibration) and from the paper itself (which trains verbalization; we instrument).*

## Path 0 merged inventory (adversarial panel, both tracks)

Panel: No-GD advocate + model-based advocate, run in one parallel round; coordinator merged. Discards carry auditable reasons.

| # | Component | Modelless analog | Verdict |
|---|---|---|---|
| 1 | Routing readouts (top/rank/share/partner) | argmax/sort over live router gate | **DISCARDED for this stack** — no served MoE checkpoint (league models dense/ternary; the only MoE vehicle is a 0.4B test arch, riir-train Issue 639's lane) |
| 2 | Top-k / P(actual) / alternative | partial-sort, log-softmax lookup, masked argmax over live logits | folded into Issue 094 as secondary columns (headful lanes only) |
| 3 | Entropy (final + lens) | textbook closed form over live softmax | folded into Issue 094 (clef lane) |
| 4 | Logit-lens top-k/P/entropy at layer ℓ | re-apply head to captured residual | **headless adaptation required** — laya ships no LM head → Issue 1106's stability-depth variant |
| 5 | **Decision depth** | suffix-stability ordering law over per-layer top-1 | **Issue 1106** (headless residual-stability variant; headful variant iff a checkpoint ships a head) |
| 6 | Question registry (12 typed probes × chance floors) | data + label functions in the harness | folded into Issue 094's baseline column |
| 7 | **Live-label law** | readout recomputed at query time by construction — modelless is *stronger* (parity-by-construction, the G5 gate's law one layer up) | validated, no new work; recorded as the design law for Issue 094/1106 |
| 8 | **Input-keyed modal baseline** | offline census (hash map, no model) + floor gate | **Issue 094** (harness baseline column) |
| 9 | KL(2.0) capability anchor | the G3 disarmed-byte-identity convention, with the paper's 2.0 as the armed-KL pin if ever needed | validated existing gate; no new work |
| 10 | Tie-indistinguishability test | determinism characterization: disclose `tie: true`, first-index rule (`float_order` convention), never silently resolve | pinned in Issue 1106's tie rule |
| 11 | Chance/majority floors | harness floor conventions | folded into Issue 094 |
| 12 | On-policy eval distribution / recursive queries / paraphrase axes | harness suite families; game twin ships (runetrace per-tick pods + replay) | DEFERRED — no trained policy in the serving lane to be on-policy about; re-opens with Issue 639 |
| 13 | Ridge-probe legibility audit (closed-form, no GD) | katgpt-core linalg over captured states | DEFERRED — becomes relevant only after arming any readout |
| 14 | Adaptive compute via depth (early exit) | runtime policy over depth | **DISCARDED with the 388 lesson cited**: lens-sweep cost structurally unfavorable (10–70× refutation shape); early exit is a different product axis with dense prior art; never gates raw sync |
| 15 | Mechanized introspection answerer | serving layer computes + serves the readout, abstain on ties | covered by Issue 094's report-only probes; a served answerer route is a consumer decision after the probes read |
| 16 | **Trained proprioception SFT** (live labels, DAgger refill, ×0.1 down-weight, λ=0.1, KL anchor) | — | **riir-train Issue 639, CONDITIONAL** — see below |
| 17 | Label-vintage discipline (self-adaptive track) | dataset meta pins blake3(weights)+quant-variant; refuse on mismatch | recorded in Issue 639 (matters only when labels are stored; the modelless stack computes at use-time) |
| 18 | Sense-direction alignment from introspection QA (latent-side) | loss_sense contrastive over answer embeddings | conditional rider on Issue 639 (circular without the trained lane producing pairs) |

**The one capability modelless cannot ship** (named honestly, per the panel): *uninstrumented self-narration* — the model answering internal-state questions from its own weights with the probe removed, under paraphrase pressure. That is the paper's actual contribution and it requires training. Everything else ships without a gradient step, with **stronger** correctness guarantees (parity-by-construction vs the trained lane's 41–71% approximation of its own labels, graded against a baseline an offline census computes for free).

## Fusion

- **paper × katgpt-rs Research 388 (cost lesson) × `forward_capture` × Report-the-Floor.** The laya encoder already ships a per-stage capture seam (`encoder.rs::forward_capture`, `twt_profile`, "bit-identical to a plain forward by construction", "calibration only, never a serving path") — the exact hook a stability-depth readout needs, with the cost class already labeled calibration-only. 388's refutation (lens-sweep prefilter 10–70× slower than the probe) sets the standing law: **monitor a layer subset, price G2 before any serve-time adoption**, and treat "quality-pass + cost-fail" as the recorded trigger for the trained lane (Issue 639).
- **paper × conformal floor.** The paper's lookup baseline and our conformal-naive floor are two different honesty axes: the lookup floor catches *mimicry* (signal ≈ input-keyed census), the conformal floor catches *miscalibration*. Issue 094 requires both — a confidence candidate that beats one but not the other is recorded with the miss named.
- **Game twin (fusion priority #1) — validation, no new work.** `runetrace` + `MonsterCognitionDebugPod` in riir-games-mmorpg ARE the live-label law + introspection pods in game clothing: per-tick internal-state pods read from the live tick, replay lane answering questions about past ticks (the recursive-query axis). The two-brain model maps cleanly: info-brain/raw cognition pods = proprioception (verifiable), think-brain beliefs = introspection-class (subjective, unverifiable by construction). The monster-ai digest triple-check (editor/vessel/viewer) is the stale-trace detector. Nothing to build; the note records the mapping.
- **Healer twin (fusion priority #2) — validation, no new work.** The healer's per-lint corpus proposer IS the modal-baseline class, and score-bench already compares candidates against it; EvidenceTier guards evidence quality. The paper's law ("any internal-access claim must beat the modal baseline") is the standing score-bench comparison restated.

## Track verdicts

- **Modelless (primary):** Gain — reflex Issue 094 + riir-infer Issue 1106.
- **Self-adaptive:** discipline recorded (label-vintage law, row 17) — no new mechanism.
- **Model-based (secondary):** conditional issue riir-train 639, owner-gated, trigger = probe quality-pass + serve-cost-fail. Honest class caveat in the issue: the paper's 8B–35B MoE setting is out of this workspace's class; no served model is pretrained here.

## Public/private split

This note and reflex Issue 094 + riir-infer Issue 1106 are public-safe (published paper + public repo surfaces). The trained-lane recipe detail (riir-train Issue 639) is private-tier. Escalation-adjacent consumers of the confidence signal are referenced only as "private tier, owner-gated" — no product posture here.

## Prior art searched (2026-10-21, three searches)

1. Headline class: "proprioception LLM self-report internal states verifiable" — the paper is the class founder; no competing proprioception-for-LLM paper found.
2. Component: "logit lens layer-wise prediction stability confidence early exit" — **prior art found** (SimLens early-exit entropy confidence; LogitLens4LLMs; lens lineage) → kills readout-primitive novelty, hence Gain not Super-GOAT.
3. Component: "verbalized confidence exteroception privileged access" — dense calibration field (ConfTuner, verbalized-confidence lineages); the exteroception critique (Song et al. 2025b) is cited by the paper itself and is answered by the modal-baseline control, which we adopt as a floor gate.

Internal-first grep: `2610.17495|proprioception` across all 16 workspace `.research/` folders → one incidental hit (katgpt-rs 553, fruit-fly locomotion, biological sense — unrelated domain).

## Priorities

- **P0:** reflex Issue 094's modal-baseline floor column (cheapest, sharpens every existing table).
- **P1:** riir-infer Issue 1106 stability readout + reflex depth probes (measurement-first; Report-the-Floor governs).
- **P2:** clef headful entropy/lens candidates (owner-gated hosted-spend posture respected).
- **P3:** riir-train Issue 639 (conditional, owner-gated, trigger-gated).

## Executed state (2026-10-25 — reflex Issue 094 CLOSED at T4/T5 adjudication)

All three probe lanes LANDED and read out:

- **T1 modal-baseline floor** — LANDED (`b884210` + `ee1c2a4`, Bench 156): `src/harness/modal_floor.rs` (ungated + pure), census side = cal slice, pooled by option IDENTITY; board read caught 3 genuine flags. The floor dominated every challenger that followed.
- **T2 laya stability-depth probe** — LANDED (`b4b9205` substrate + `d3ceb00` probe, Bench 157): the encoder's residual stream has a stability TROUGH, not a suffix; **RECORDED NEGATIVE** (AUROC ≈ chance or inverted on 6/9, the modal floor dominates the lane on typed). Report-only, arms nothing.
- **T3 clef headful entropy** — LANDED (substrate `0f876b8` + probe, Bench 158): one-pass entropy via the tied LM head; **RECORDED NEGATIVE** — fails both floors on 6/9; the letter-PASS disqualified by inverted AUROC (0.365). Lens depth DEFERRED on the named unblock (ANE ships one output; a CPU forward port is the lane that would unblock). Report-only, arms nothing.

**T4 (cost law) + T5 (report-only posture) adjudicated as STANDING LAWS, not tasks** — there is no per-layer sweep to monitor (T2/T3 arm nothing), and both shipped probe lanes carry the REPORT-ONLY module contract (no serving-path edit, no exit-code coupling beyond loud refusals, CPU-capped). The laws live on in the module docs (`src/harness/runner/depth_probe.rs`, `clef_conf_probe.rs`) and in this note; the issue file is removed per the noise rule with this section as the record. The paper's measurement discipline transferred exactly as the note hoped: the input-keyed modal floor is the anti-mimicry control that dominated every internal-signal candidate tried — three benches, three negatives, each with the miss named.

The conditional trained-lane trigger (riir-train 639) stays OPEN and owner-gated: its trigger (quality-pass + serve-cost-fail) has not fired — no candidate passed quality.
