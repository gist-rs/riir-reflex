# Research 016: XERJ Query-1 — pointer-program serving distill (questions compiled to executor-checked programs over typed facts)

> **Source:** XERJ Query-1 announcement — <https://xerj.org/blog/xerj-query-ai-model> (published 2026-11-05; fetched 2026-11-06 via webReader) + <https://github.com/xerj-org/xerj> (Apache-2.0 engine; release `66fbc1d85423b7ef9c401fc25fc546dff9abf7f3` from the GitHub page metadata; the model bundles and the train/serve crates for the model are PROPRIETARY per their §08)
> **Date:** 2026-11-06
> **Status:** Active — **Gain** verdict; two adoption issues filed (this repo `.issues/110`, `../riir-infer/.issues/1182`)
> **Related Research:** 004 (Program-as-Weights serving landscape), 005 (Clef/Jev decision index), 013 (JevAct decision gate)
> **Classification:** Public — public mechanism + our public contract ideas only; no rethink-class content

---

## TL;DR

XERJ Query-1 is a small trained reader whose **output vocabulary is program tokens over numbered pointers** into a typed-fact store (`LOOKUP #10; LOOKUP #7; LT; RETURN`). The engine executes the program over the records, checks types/units per step, logs a trace naming every record and source span, and a **frozen rule-set verifier — never the gold answer** — returns one of three statuses: ANSWER (with evidence), UNKNOWN (a pointer grounds to nothing), CONFLICT (two stored records disagree). Served int8 on 4 CPU cores under 1 GiB, $0.042 per 1,000 questions, no embeddings, offline. The training story is thin and proprietary; the **serving design is the distill**: constrained output space → verifiability, refusal as a *typed* contract (three causes, not one), evidence pointers per answer, and an int8 mode guarded by exact-path fallback. Three concrete consumables filed as issues; the rest maps to existing substrate.

**Distilled for the decision-serving family (modelless):** the program/trace/verifier decomposition is a serving-CONTRACT pattern, independent of their model. Any scorer whose evidence is a finite, addressable candidate set (corpus rows, rule candidates, option scores) can disclose *why it refused* as a typed cause and *what it answered from* as named evidence. Our engine already has the cause-typing internally and throws it away at the wire boundary.

---

## 1. What it does (mechanism + their measured claims)

Mechanism (blog §06): typed records (subject, relation, typed value + unit, source span) are written to the engine's store. Per question the engine lists the namespace's candidate records as a numbered listing; the model reads question + listing and emits 2–6 pointer steps; the executor runs them (types/units checked; UNKNOWN on an ungrounded pointer; CONFLICT when two records disagree and the program touched both); a frozen rule set reads the trace + decline codes and decides serve/decline. Counts additionally require a **second, independently trained model to agree on the program**. Every answer carries the record ids and doc spans it was computed from; a question can be asked **as of an earlier revision** (bitemporal store). Op-set (served counts on their 2,387-item eval): `LOOKUP #n` 2,972 · `CHAIN` 378 · LT/GT/EQ/NOT 583+190 · `PICK_GT/PICK_LT` 59 · `RETURN` 2,378; set programs `SET/FIND/FILTER/AGG COUNT` 836 (529 served after verification); `REFUSE` 998; `ABSTAIN` trained but used 0.

Their measured claims (all self-reported; we re-ran nothing — see §7):

| Axis | XERJ Query-1 | Their baselines (same 160 questions, same rules) |
|---|---|---|
| Wrong answers | **1 / 160** | RAG+Opus 5.5: 9–15 · GraphRAG: 9 · whole-store Opus (40 q/call): 6 · oracle Opus: 5 |
| Declined | 35 / 160 | no LM method declined more than 24 |
| Cost / 1k questions | $0.042 (live CPU run) | RAG $13.41–$20.95 · GraphRAG $65.39 · whole-store 1 q/call $694.30 |
| Serving | 120 ms median (int8-guard, 4 cores, quiet x86); 1,039 ms on the shared aarch64 head-to-head box; 674 MiB resident | — |
| Indexing | 10 GB Wikipedia in 17 min / 0.7 CPU-hours | bge-small embedding the same text: 62–97 CPU-hours + 8.5 GB vectors |

Standard evaluation (1,600 questions in own collections): 1,208 right / 9 wrong / 383 declined, score 90.29 where the reference (Opus 5.5 reading the same listing) defines 100. **Their own caveats, kept:** the standard eval is the one the model was developed against; `bench_select`/`d1_select` were design data for the decline rules; n=160 makes most score gaps non-significant; it **over-declines** (383/1600 vs reference 216; most declines suppressed answers the executor had already computed correctly); counting tallies members without binding the question's noun (a disclosed defect: a member count can be served as a "verified" answer for an unsummable quantity); fresh-wording robustness is unmeasured for the shipped bundle.

Method-discipline items worth recording (they mirror ours): scoring rules + Net formula + seed set (11/29/47) + promotion bars **pre-registered before scoring**; their first score (100 capped / 130.7 unclipped) rewarded declining items the reference was penalised for, was **withdrawn by them**, and v2 restarted from 49.9; the int8 configuration was accepted on **400/400 answer agreement with FP16**, not on a benchmark score; the engine executor is bit-exact-parity-gated against the research implementation on all 2,387 items ("program, raw output and evidence identical on every item").

## 2. Verdict

**Gain** (external product distillation; smallest-artifact discipline: one note + two issues, no plan).

- **Tier reasoning.** Not Super-GOAT/GOAT: we make **no novelty claims** — the mechanisms are published prior art (below) and the actionable items are adoptions into our serving contract, sized as `.issues/` poc/proof tasks. Not Pass: three components have verified in-tree gaps with runnable proof paths (the ungoat rule — no GOAT filing without a proof path, and these are issues, not GOAT filings).
- **Prior-art posture (§4 of the skill, hard gate).** Pinned claims, searched, all landing as ADOPTION not novelty:
  1. "CONFLICT as a first-class typed answer status, consuming top-option disagreement, additive to the decision_wire v1 contract, distinguished from the shipped fused-abstain (single decline channel, cause not wire-visible) by typing *evidence disagreement* separately from *no grounding*." → published landscape: **ConfQA** (Nachshoni et al. 2025, "Consensus or Conflict? Fine-Grained Evaluation of Conflicting Answers in MAQA", aclanthology 2025.uncertainlp-main.13), **Conflict-Aware RAG** (ACM 2026, 10.1145/3774904.3792289), **ConflictingQA** (2026), **ConflictBank** (2024/25), **"RAG with Conflicting Evidence"** (Wang et al., ~121 citations). The distinction is established; the delta is ours to adopt.
  2. "Per-decision corpus-evidence pointers in the served response, opt-in + capped, additive to the v1 response field set." → disclosure discipline, not a novel mechanism (XERJ's own "the trace is the product" is the citation).
  3. "Online per-query drift guard with exact-path fallback for lossy serving arms." → published/shipped practice: speculative-decode verification (in-family: `katgpt-speculative`) + shadow/canary cross-checking (a TensorRT builder-time precision-constraint citation was considered and dropped — per-layer build choice, not an online guard); the delta is applying the guard to our lossy serving arms, which today gate **offline only** (per-family retention gates + bench-side FNV/argmax tripwires + load-time Q8 sidecar BLAKE3 — verified greps this session, §3 row 3).
- **MOAT gate.** Primary: `riir-reflex` decision-serving moat (typed refusal + evidence disclosure on the public serving contract — exactly this repo's MOAT row). Secondary: `riir-infer` public-substrate moat (serving posture for lossy arms). No game-runtime, chain, or shard claim. No rethink-class content anywhere (public destinations only; §6).
- **Fusion-priority ladder check (two-sided).** Decision-serving loop: CONFLICT + evidence pointers are direct serving-contract improvements — priority #1 family. Rulewright: covered — the fixer's `fix_verify` compile gate is a *stronger* verifier than XERJ's frozen trace rules (ours consults the real oracle, theirs a rule proxy), and per-edit provenance/corpus citation already ships; the rescue lane's agreement VETO is the second-model-agreement genus. Signal-diffs in §4.

## 3. Component inventory (Path 0) + routing

| # | Component | In-tree coverage (verified this session) | Modelless-extractable? | Routing |
|---|---|---|---|---|
| 1 | **CONFLICT typed refusal** — "two independent readings disagree" as a distinct status | **No analog.** Fused abstain is cause-typed internally only: `AbstainCause { Answered, ScoreGate, DistanceGate, GrammarInvalid, DensityGate }` (Issue 060's closed taxonomy, `src/engine.rs` L840–847); the wire discloses every abstain as `outcome: null` (`to_response`, `src/engine.rs` L2015–2016). No arm means "readings disagree"; grep "conflict" over `src/` hits only the slice-leak counter | Yes — computable from two INDEPENDENT witnesses (drafter vs fitted-head argmax, `src/engine.rs` L1887 / L1838–1839; or top-k reference-label split) that survive the existing gates, no training | **`.issues/110`** (this repo) |
| 2 | **Per-answer evidence pointers** — the records/rules behind a pick | **Partial.** `to_response` ships `answers[{question_id, outcome, probabilities, confidence}]` + `routing{lane, reason}` + calibration; `routing_reason` discloses per-domain hit **counts** as a string (`src/engine.rs` L2218–2222) but never names the corpus entries behind a pick | Yes — the engine already holds the routed domain's per-entry state at solve time | **`.issues/110`** |
| 3 | **int8-guard exact-path fallback** — online per-query drift guard for lossy arms | **No online guard.** Greps `exact-path fallback` / `drift guard` / `guard.*fallback` over `riir-infer/**/*.rs`: no serving hits (only an unrelated "staging-chain fallback is exact" comment and a temperature-table fallback). Lossy arms gate offline: per-family retention at promotion, FNV/argmax tripwires (league bench), Q8 sidecar BLAKE3 (load-time integrity) | Yes — sampled exact-path cross-check + sigmoid threshold, no training | **`../riir-infer/.issues/1182`** |
| 4 | Frozen-rule trace verifier (decline codes, never gold) | **Covered, stronger.** `fix_verify` = the compile oracle itself (baseline → apply → re-check → revert), not a rule proxy over a trace | — | audited discard: our verifier class is strictly stronger; no gap |
| 5 | Second-model agreement for counts | **Covered.** Rescue-lane agreement VETO is the acceptance authority over tallies (`self_evolve` stack, refine AGENTS.md); same genus: dual-path agreement before serve | — | audited discard: genus covered; no decision-content delta named by their post (they describe the gate, not a law we lack) |
| 6 | Pointer-program op-set / compiler-in-the-engine | **No consumer.** No typed-fact QA surface ships anywhere in the family; the decision engine scores options, it does not compile programs | The op-set *design* (pointers + typed compare + refuse) is training-free, but there is no surface to hang it on | recorded only — no file. Re-open trigger: any typed-fact/structured-QA lane in the family |
| 7 | Bitemporal ask-as-of-revision | **Partial.** Corpus lease epochs (refine lease/diff client-side), editor snapshots, warm-tier anchors — versioned consumption exists per surface | — | recorded only — no live consumer gap |
| 8 | Training recipe (FP16, rented GPUs, synthetic worlds, 3 seeds, pre-registered promotion bars) | Recipe is disclosed at summary level only; trainer is proprietary | — | audited discard: nothing concrete to extract that our own pre-registration/seed discipline does not already do; Path 0.5 not triggered (no applicable recipe rows survive the summary-level disclosure) |

## 4. Cousins + signal-diffs (§3.6 discipline)

- **Fused abstain (`CorpusDistanceGate` + `AbstainCause`)** — signal consumed: embedding distance to the routed corpus + calibrated confidence = *"no evidence grounds this answer"* (relevance). XERJ CONFLICT consumes *"two stored evidences disagree"* (agreement). Different signal, no arm carries it — this is the gap issue 110 fills, and the one place a name-level cousin (`DistanceGate`) does NOT cover the mechanism.
- **`ScoreGate` (low calibrated confidence)** — adjacent but NOT the witness: "top-2 options within epsilon with disagreeing labels" is TAUTOLOGICAL (the top two options of one scored question always carry different labels — that predicate is the `ScoreGate` subset itself and would "confirm" CONFLICT for the wrong reason). The witness is two independent readers disagreeing while both clear the gates: drafter argmax vs fitted-head argmax (`src/engine.rs` L1887 / L1838–1839), or top-k nearest-reference label split with `DistanceGate` passing. Issue 110 T1 measures the residual band — the share NOT already ScoreGate/DistanceGate/DensityGate — and closes the issue honestly if it is negligible.
- **Seat rerank contract (instinct, public grammar: pick-or-abstain, options echoed exactly)** — a typed conflict decline code would be additive there too; recorded as issue 110 T5's second consumer, instinct-side follow-up only if T1 proves the band real. Not filed separately (smallest-artifact rule).
- **`fix_verify` / rustc_errors playbooks** — signal-diff run: XERJ verifier = frozen rules over a trace (a proxy); ours = the compile oracle (ground truth). Their "verifier never reads gold" discipline is already our P104/verify-refusal posture. No gap.
- **Corpus lease epochs** — versioned-corpus consumption exists; bitemporal query semantics (ask-as-of-epoch) is a client-side projection today, no filed gap.
- **Speculative decoding (`katgpt-speculative`, katgpt-rs)** — the in-family genus of inventory row 3 (1182): a lossy draft verified against the exact path with fallback on disagreement. Mechanism-level diff: spec-decode verifies EVERY step (lossless by construction); the guard samples a bounded fraction and demotes the breached family (lossy retained, overhead bounded). Recorded in 1182's Why; a TensorRT builder-time precision-constraint citation was considered and dropped (per-layer build choice, not an online guard).

## 5. Reframes (step 4)

- **Game-context:** the CONFLICT signal is the belief-side counterpart of the two-brain spatial model — two same-zone witnesses disagreeing should lower belief confidence and can drive a *re-observe* drive rather than action, exactly as stale `SpatialBelief`s fade by sigmoid decay. Recorded as the game lens; no riir-ai file (no live consumer asked for it).
- **Consumer-context (rulewright):** `heal_with_pool` rank-0 ordering over contradicting candidate rules is the same shape — a conflict-typed decline handed to the seat would be more informative than a bare abstain. Covered by the T5 line in issue 110.
- **Stationary-surface (step 4a):** CONFLICT is a per-query LEVEL classification over independent witnesses (drafter vs fitted head; reference-label split — a pure function of inputs), a typed readout, not a dynamics signal; no mutated state required. The int8 guard in 1182, by contrast, mutates runtime state (the fallback counter + family demotion latch) — it names its state in the issue.

## 6. Public/private tier check + proof paths

- Every filed artifact is public-safe: this note (public mechanism + public contract ideas), `.issues/110` (public serving contract), `riir-infer/.issues/1182` (public substrate posture). No rethink-class content (no serving arms, no ESC enforcement, no economy planes) in any destination.
- Proof paths live in the issues: 110 T1 (measure the disagreement band on harness suites — the falsification arm; a negligible band closes the issue), 110 T4 (wire-contract tests + flag-off byte-identity), 1182 T2 (injected-drift PoC: fires on drift, silent on clean) + T5 (the GOAT gate with pinned bars).
- **Weakest point, named by us:** XERJ's numbers are self-reported on their own developed-against evaluation; and our CONFLICT premise ("the disagreement band matters on OUR suites") is borrowed from their data, not measured here. Issue 110 T1 is therefore the load-bearing gate — it can kill the adoption honestly.

## 7. What we did NOT verify

- Every accuracy/cost/latency figure in §1 is **their** claim, cited as such; we re-ran nothing and did not audit their harness.
- The engine source was **not cloned** (§0.5 of the skill not triggered — the distillation is over the product description + README; no byte-quotes from their source are cited). A later mining pass over the executor/verifier design would clone at the pinned release sha `66fbc1d85423b7ef9c401fc25fc546dff9abf7f3` and re-verify.
