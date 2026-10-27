# Research 012 — Tabular Inductive Biases (arXiv:2207.08815): why the modelless floor beats encoders on irregular suites, and the lane-seating screen

> **Source:** Grinsztajn, Oyallon, Varoquaux — ["Why do tree-based models still outperform deep learning on tabular data?"](https://arxiv.org/abs/2207.08815) (NeurIPS 2022 Datasets & Benchmarks). 45 OpenML datasets, ~20k compute-hours of random-search records shared raw.
> **Date:** 2026-10-27
> **Status:** DISTILLED — actionable halves filed (`riir-reflex/.issues/100_inductive_bias_lane_seating_diagnostics.md`)
> **Verdict: GAIN** — the paper does not ship a primitive we lack; it ships the first mechanism-level WHY for a pattern our serving board has been recording as data since birth, plus two cheap harness diagnostics (issue 100).

## TL;DR

On 45 medium-sized tabular datasets, tree ensembles (XGBoost/RF/GBT) beat every deep learner (MLP, Resnet, FT-Transformer, SAINT) at **every** hyperparameter-search budget, and stay ahead when the budget is counted in wall-clock hours. The paper then isolates **why** via data transformations, yielding three inductive-bias findings:

1. **Smooth-bias mismatch.** Gaussian-smoothing the target function hurts trees sharply and NNs barely → real tabular targets are *irregular*; trees learn piecewise-constant functions natively, NNs are spectrally biased toward smooth ones (Rahaman et al. 2019).
2. **Uninformative-feature fragility.** Tabular data carries many genuinely uninformative features (a GBT trained on the *removed* half scores near chance). Removing them narrows the MLP gap; adding Gaussian noise features widens it. MLP-like learners pay for them; trees do not.
3. **Orientation matters.** MLPs are rotationally invariant (Ng 2004), and rotational invariance is a *liability* here: the natural feature basis (columns: age, weight) encodes meaning, and rotations mix features with different statistical properties. Random rotations REVERSE the model ranking — NNs jump above trees. Embedding layers (even for numeric features — Gorishniy 2022) help largely *because they break rotation invariance*.

Also: the gap narrows at 50k samples; tree *depth* is a top hyperparameter (deeper = more irregular patterns fit — consistent with finding 1).

## Pinned claim (§1.5 precondition)

> An inductive-bias explanation (smooth-bias mismatch + uninformative-feature robustness + orientation preservation) for the serving board's modelless-floor-vs-encoder-lane pattern, consuming the paper's three findings mapped onto the hashed-bag kNN engine vs encoder lanes, distinguished from the board's existing empirical rows and the `echo_gates` OOD rig by a mechanism-level WHY plus a cross-lane robustness-delta diagnostic the single-arm echo gate does not provide.

## Mapping to the stack — the board rows were data; this is the theory

The modelless engine (`src/engine.rs`, `src/embed.rs`) is the **kNN family**: FNV-1a unigram+bigram hashes into 256 buckets (sign trick, L2-normalized) → `pick_domain` corpus routing → `Lz4FlexDrafter` corpus-is-the-model option scoring. That is a **piecewise-constant function of the input** — the exact learner class finding 1 says owns irregular targets. And the hashed bag **preserves the natural basis** (every token owns its hash slot; nothing mixes features) — the exact "breaks rotation invariance" posture finding 3 says wins on tabular data. Encoder lanes (laya, d1, mammoth, …) carry the deep smooth bias.

Board of record (riir-instinct AGENTS.md serving table, Bench 0059/0060 posture):

| suite | modelless | encoder | read through this paper |
|---|---|---|---|
| massive_intent_en | **0.8400** | 0.6750 | utterance→intent is lexically clustered; surface markers; irregular target → kNN wins |
| banking77 | **0.8540** (T2 uncertified — reads LB95 −0.0163 at the 0059 posture) | DEAD (Bench 039 screen) | same shape, uncertified |
| prompt_injections | **0.8534** | DEAD | strong surface markers (instruction-shaped text) |
| emotion | **0.8850** | DEAD (Benches 032+035) | strong lexical markers |
| sst5 | 0.4217 | **0.5267** | sentiment intensity is graded/semantic → smooth generalization needed |
| ag_news | 0.8975 | **0.9475** | topic generalization beyond surface vocab |
| xnli_en | 0.5233 | **0.8600** | entailment is paraphrase-level semantics; the largest gap — as predicted |
| typed_decisions | 0.6475 | **0.7550** (v2 head) | rubric-weighted decision semantics |

The split lines up with the axis the paper isolates: **suites whose label function is lexically irregular → the piecewise-constant learner wins; suites whose label function is semantic/smooth (paraphrase, entailment, graded intensity) → the encoder wins.**

### Evidence strength (verdict-round-1 caveats, recorded rather than smoothed)

Three honesty marks a reviewer pinned, kept so the theory is never read as stronger than its evidence:

1. **Only one live head-to-head win.** Of the four modelless-wins rows, three (banking77, prompt_injections, emotion) are wins against encoder lanes that failed a DEAD screen — no seated encoder was ever measured against the modelless engine there. The single live head-to-head is massive_intent_en (0.8400 vs 0.6750). "Encoders lose because of a smooth bias" is therefore thin on those three rows; issue 100's probes must cover at least one DEAD suite and one seated-encoder suite to earn the claim.
2. **Finding 1 is an analogy too, not just finding 2.** The paper measured smooth-bias on NUMERIC tabular features with irregular target functions; hashed bag-of-words text is not tabular. "Encoders carry a smooth bias over text inputs" is an extrapolated mapping and is hereby marked as a prediction — T2 (`--label-smooth-ab`) is the decisive test: if label smoothing does not close the lane gap, the finding-1 mapping fails and the note's theory section gets the negative recorded.
3. **banking77 is uncertified** (T2 LB95 −0.0163 at the published posture) — kept in the table with the mark, never quoted as certified evidence.

Finding 3 additionally explains why the hashed bag does not suffer the MLP fate on noisy suites: it never mixes the basis. The finding-2→kNN extrapolation (distance concentration under feature noise) is likewise a testable prediction (issue 100 T1), not a paper result. The board shows the modelless floor winning on noisy real suites despite hashed-bag noise — the probes settle how much of that is robustness vs clean small corpora.

## Fusion (paper × board × existing machinery)

`echo_gates` already ships the OOD corruption rig (`corrupt_state`, seeded word-dropout ladder) — built as a single-arm ECHO gate for corpus projections. Reusing it ACROSS lanes turns it into a **cross-lane robustness-delta probe**: at each corruption rung, measure modelless degradation vs encoder degradation per suite. Prediction (finding 2 extrapolation): modelless degrades less on lexically-clustered suites. A second diagnostic (corpus-neighborhood label smoothing — the finding-1 analog for a kNN engine) probes the irregularity axis directly. Together they are the measurements a **lane-seating screen** would consume: two numbers per suite predicting which lane to seat, the McElfresh-style property screen applied to our board. Filed: `riir-reflex/.issues/100`.

## What changed since 2022 (prior-art landscape, §4)

- **TabPFN** (Hollmann et al., v1 2023; **v2 Nature 2025**; v3 2026): a transformer trained on synthetic prior data beats boosted trees on small tabular via in-context learning — NOT by fixing the three biases head-on. Its per-feature embeddings break rotation invariance (consistent with finding 3); one 2026 analysis notes TabPFN "learned smoother, simpler relationships than CatBoost" on shared tasks — i.e., the winner class changed, the bias axes stayed.
- **McElfresh et al.** [arXiv:2305.02997, "When Do Neural Nets Outperform Boosted Trees on Tabular Data?" NeurIPS 2023 D&B]: dataset *properties* predict the tree-vs-NN winner. The property-screen concept is published; issue 100 applies it to lane seating on OUR suites — application, not invention.
- Internal-first grep: zero hits for `2207.08815` / `Grinsztajn` / `tabular` across all workspace `.research/`; the inductive-bias-lane-seating theory note does not exist (closest cousins — katgpt-rs Research 28/70/100 — are attention-architecture bias notes, a different axis).

## Consumer check (canonical failure #5 — the healer)

Did I ask what this does for riir-refine? The healer's drafter is the same corpus-is-the-model family over code shapes, with oracle-labeled eval and retrieval floors already owning quality. The paper validates that choice (irregular rule-shape targets favor shallow learners) but yields no actionable change: no documented healer gap maps to these findings (the healer ships no MLP-like scorer to de-bias, and its retrieval floors are measured, not theorized). Not filed — recorded here as the checked answer.

## Track routing

- **Modelless (a):** the whole note — engine validated + two probe filings.
- **Self-adaptive (b):** untouched.
- **Model-based (c):** no riir-train plan. The paper carries no optimizer/loss/schedule/RL content; the encoder checkpoints on the board are external frozen artifacts, and nothing here indicates a training recipe. Path 0.5 not applicable; the three-track adversarial panel is skipped per its trigger (first-pass classification never touched training).
- **Public/private:** this note and issue 100 are public-safe — riir-reflex is the sanctioned public decision-serving tier; no rethink-tier content (no serving arms, vessels, ESC, or economy planes).

## Verdict accounting

- Not Super-GOAT: Q1 fails — the bias axes are published; our contribution is application + diagnostics.
- Not GOAT: no new primitive ships in this session.
- **GAIN:** theory note (this file) + issue 100 (report-only probes behind harness flags, byte-identity when off). If the probes separate cleanly, the lane-seating screen graduates to a plan of its own.
