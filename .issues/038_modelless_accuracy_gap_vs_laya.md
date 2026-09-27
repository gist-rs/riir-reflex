# Issue 038 — modelless accuracy gap vs laya: the scorer is the ceiling, not the corpus

**Status:** OPEN — T1/T1b/T2/T3/T6 DONE (Bench 051; `d66ef21` `0d8eaa0` `32aee61` `1fa8823` `754eca0`; published reflex-site `9b8223d`). At the fair full pull, modelless ≥ laya on 4/8 dataset suites (emotion, sst5, massive, banking77). T4 REVISED: the model-based/hybrid lane goes to private `riir-instinct` (riir-ai Proposal 047, riir-train Issue 576); T4′ cascade becomes that repo's hybrid. T7(a)+(b) LANDED 2026-09-27 (Plan 004: option-conditioned tables + NBSVM ridge, both promoted default-on — typed +13.5 pt, emotion +11 pt). T5 step 1 LANDED `6199e5e` (selection ladders extended upward; round-2 re-selection lifted emotion to **0.885** ridge@8). **T5 steps 2+ LANDED 2026-09-27** (Plan 005 / Bench 064: the joint blend-genome lane `--genome-select` — build-once-fitted + `set_blend_scales`, coordinate descent over `{route, head, nb(+α,+view), oc, ridge}` on the cal slice, acceptance-vs-seed bar `--genome-accept-margin` default the house 0.05). Verdict: **banking77 0.8260 → 0.8620** (ridge@1 + head off, +3.60, G1 pass) and **sst5 0.3967 → 0.4017** (ridge@8, +0.50, G1 pass) ADOPTED; massive +5.33 REJECTED (G1 FAIL — ECE .2383 vs floor .1361, the accuracy gain broke calibration); emotion REJECTED (0.35 — the walk's nb-off posture collapsed on test; the house bar would have refused it); RRF DECLINED (the blend terms share a σ scale — additive wins there by riir-rag's own RRF doc; emotion's failure is posture overfitting, not scale incomparability). Site republish deferred to the post-041 whole re-run (`publish_bench` has no G1 filter — a wholesale publish would carry the rejected massive row). With banking77 0.862 the modelless lane extends its lead; the scoreboard reads modelless ≥ laya best on 5/9 dataset suites with banking77's margin now +36.2. Open here: T7(c) BM25 kNN, T7(d) Hebbian xnli (T7(d) reopens NOW that T5 landed); the ag_news volume lever is `.issues/041`. T4′ cascade lane LANDED 2026-09-27 (Bench 061, `337974d`): the lane + measurement are done; the gate verdict at the shipped fused gate is NEGATIVE (cascade < modelless on 5/8 dataset suites — the fused gate's test-side abstain is 90–99% at the armed postures vs the ρ=30% cal target; follow-up lever filed as issue 042; the lane stays opt-in as the instrument).

## Finding (Bench 045, M3, fold-promoted default)

Modelless is at or above the best laya lane on **5/14** suites; on the rest it trails:

| suite | n | modelless | laya best | gap |
|---|---|---|---|---|
| xnli_en | 300 | 0.347 | 0.860 | **−51.3** |
| ag_news | 400 | 0.510 | 0.950 | **−44.0** |
| typed_decisions | 2000 | 0.319 | 0.745 (typed ckpt) | **−42.6** |
| emotion | 400 | 0.282 | 0.593 | −31.0 |
| code_fixtures | 24 | 0.333 | 0.583 | −25.0 |
| prompt_injections | 116 | 0.483 | 0.698 | −21.6 |
| harness_sensitivity | 15 | 0.400 | 0.600 | −20.0 |
| harness_routing | 16 | 0.438 | 0.625 | −18.8 |
| sst5 | 600 | 0.217 | 0.372 | −15.5 |
| banking77 | 500 | **0.684** | 0.498 | +18.6 |
| massive_intent_en | 300 | **0.793** | 0.750 | +4.3 |

The Tetris win (katgpt-rs Bench 891/892, reflexer 60/60 seeds, 92–299× points) does
**not** transfer as a mechanism. It came from an exact simulator plus depth-3
lookahead over about 900 continuations, scored on hand-designed board features.
Choosing one label from N is a single decision with no next state to search.
What does transfer is the **scaffolding**: rules as data, a genome, and a gated
hill-`climb` against held-out seeds (T5).

## POC measurement (2026-09-26, throwaway analysis script, not the product path)

Plain multinomial naive Bayes: add-1 smoothing, unigram + bigram, one fixed
configuration with **no hyperparameter selection**. Trained on the already-fetched
train rows only and scored on the same test split the harness uses:

| suite | train rows | test tokens seen in train | NB @64/label | NB @all fetched train | modelless (045) | laya best |
|---|---|---|---|---|---|---|
| ag_news | 4000 | 94.2% | 0.665 | **0.868** | 0.510 | 0.950 |
| emotion | 4000 | 94.8% | 0.268 | **0.570** | 0.282 | 0.593 |
| sst5 | 4000 | 91.3% | 0.263 | **0.387** | 0.217 | 0.372 |
| prompt_injections | 546 | 77.8% | 0.595 | **0.802** | 0.483 | 0.698 |
| xnli_en | 4000 | 92.2% | 0.363 | 0.387 | 0.347 | 0.860 |

Reads: (1) data volume matters a lot for a count-based scorer. The same scorer goes
0.665 → 0.868 on ag_news just from 64/label to all 4000 rows, and only 4000 of 120k
were fetched. (2) Train already covers about 92–95% of the test tokens, so the test
rows add almost nothing a train-only corpus lacks. (3) NB alone passes laya on sst5
and prompt_injections and comes within 2 pt on emotion. (4) xnli stays at the
bag-of-words floor, as predicted (T3/T4). Not a GOAT number yet: T1 must reproduce
it in Rust with selection on the cal slice.

## Diagnosis — three ceilings, all in the scorer

1. **The scorer gets worse with more data.** Bench 003: ag_news peaks at cap 64/label
   (0.51) and falls to 0.46 at 512, with about 950/label available. A count-based
   estimator improves steadily with n, so the byte-level `Lz4FlexDrafter` delta plus
   centroid cosine is the ceiling here. katgpt-rs Bench 285 and negative_results §23
   say the same about LZ4: it needs long, redundant text, and these rows are one-liners.
2. **The feature space collides.** `EMBED_DIM = 256` hashed buckets (`src/embed.rs:18`)
   for a vocabulary of tens of thousands of words, and the Issue 030 label heads sit on
   those same 256 buckets. That is plausibly why head-select picks scale 0 on ag_news.
3. **xnli is structurally blind.** Premise and hypothesis are hashed into ONE bag
   (`suites.rs:624`, `:978`). Which sentence a token came from is lost, and there are no
   cross-sentence features (overlap, negation mismatch). NLI labels are not topical, so
   the three centroids coincide: Bench 005 reports 94.9% of errors landing on `neutral`.

Also noted: `scripts/fetch_datasets.sh:333` fetches only 4,000 train rows per suite.

## Levers the audit found untried

Checked against every lever Issues 004/013/023/030/036 recorded: **no** naive Bayes,
TF-IDF/BM25, kNN vote, un-hashed vocabulary, or pair-structured features has ever been
tried.

## Plan (ranked by expected gap closed per unit of work)

- [x] **T1 — per-label naive-Bayes log-odds scorer** — LANDED as `nb_scope`
      (`d66ef21`), promoted default-on (Bench 051): ag_news .510→.875,
      emotion .283→.595 (past laya .593), massive .793→.927, banking77
      .684→.870; G1–G4 PASS; unarmed suites bit-identical. Original spec
      below kept for the record.
      **(spec)** One
      `katgpt_core::contrastive_scope::ContrastiveScoreBuilder` per label, one-vs-rest
      (label docs in, all other labels out). Vocabulary: 2^18 (shipped at 2^17, `NB_VOCAB`) hashed unigrams + bigrams,
      NOT the 256-dim embed. Score = `scope_score` → sigmoid; it is either a new option
      term beside route/head, or the replacement for the drafter delta (measure both).
      The new dependency is one forwarded feature, `katgpt-core/contrastive_scope`
      (default-on there, GOAT-passed at Bench 669: 6.9 µs/doc, alloc-free). It is still
      modelless: frozen counts from the train split, BLAKE3 `freeze`/`thaw`. Targets:
      ag_news, emotion, sst5, prompt_injections. Expected (not measured): topic NB on
      ag_news is typically 85–90%.
- [x] **T2 — corpus-size sweep under T1** — DONE (Bench 051 full-pull section): full train pull (`TRAIN_CAP=20000`, `--datasets-dir`) lifts emotion .595→.7375 and sst5 off→.3917 (both past laya); ag_news .875→.8825. It also EXPOSED Issue 039 (label-truncated 4k pull). Published both hosts at reflex-site `9b8223d`. Original spec: Raise the train fetch (ag_news full
      120k) and sweep the NB pool size on the cal slice. NB table build cost is O(tokens)
      once and scoring is O(query tokens), independent of pool size. So the Bench 003
      perf/sec refusal (drafter cost ∝ corpus) does not apply to this term, but measure
      p50/p99 anyway.
- [x] **T3 — pair-structured xnli features** — LANDED (`0d8eaa0`) as
      `NbView::Pair`, cal-selected: xnli .347→.520 (train-only hold-out
      before building: .386→.505). Spec below kept for the record.
      **(spec)** Namespace the hash by sentence
      (`p:` / `h:`), and add cross features: hypothesis-token coverage by the premise,
      a negation-mismatch flag, a number-mismatch flag, and length ratio. Feed them into
      T1's table. Honest ceiling: lexical NLI tops out around 55–65%, so this closes part
      of the gap, not 86%.
- [-] **T4 — REVISED 2026-09-26 (owner pushback, accepted):** the model-based lane IS
      wanted; katgpt-rs Proposal 014 always planned three lanes (model-based /
      modelless / hybrid). It moves to the PRIVATE `riir-instinct` repo (riir-ai
      Proposal 047; trainer = riir-train Issue 576), because hybrid wiring and
      weights are product moat (Research 003:74), and this repo is public. This
      repo keeps the modelless lane (the drafter/pruner the hybrid consumes).
      The earlier "not built" reasoning, which applied only to putting an
      encoder INSIDE the modelless lane:
      Why: (a) laya's encoder + our head is laya-with-a-worse-head at laya's
      latency, so it can't honestly "beat laya"; (b) Bonsai/Gemma embedders are
      2B+ params, bigger and slower than laya's 421M (Bonsai ~50 s/chunk on CPU,
      Bench 583), giving up the latency axis modelless wins on; (c) both put
      neural weights inside the lane the site sells as "no neural weights".
- [x] **T4′ — cascade lane "Reflex · cascade" (opt-in, its own named lane):**
      LANDED as the harness measurement lane (Bench 061, `337974d`): `--cascade`
      composes the modelless answers with each served riir-laya checkpoint over
      the calibrated fused gate's abstains; publishes accuracy AND the
      escalation rate per suite (the rate is the latency claim). Verdict at the
      shipped gate: **NEGATIVE** — accuracy ≥ modelless fails on 5/8 dataset
      suites (emotion −28.3, banking77 −14.0); the mechanism is the fused gate's
      cal→test abstain transfer (90–99% test-side at the armed postures vs the
      ρ=30% cal target), filed as issue 042. G3 proven (1,712 non-timing fields
      byte-identical with the flag off). The lane stays opt-in — it is the
      instrument the gate fix re-measures with. Original spec, kept for the
      record: modelless answers first; questions where the EXISTING fused abstain
      gate fires escalate to the laya lane. No training, no new model. Publish
      accuracy AND escalation rate per suite (the rate is the latency claim).
      Gate: accuracy ≥ modelless on every suite and within noise of laya on
      xnli/typed, with escalation < 100% on the topical suites. Runs after T7.
      Original T4 spec, kept for the record:
      `riir_infer_laya::LayaEncoder::forward` (already in the dep tree) or
      `BonsaiEmbedder::embed_full` hidden states → mean-pool → per-label centroid /
      diagonal LDA, with `[u, v, |u−v|, u·v]` for xnli. It is modelless under
      freeze/thaw, but it is no longer the "0.5 ms, no model" lane. Publish it as a
      **separate named lane** (e.g. `reflex · frozen-encoder`), never under `modelless`.
      A laya-encoder variant would be "laya's encoder + our head", so label it as such.
- [x] **T5 — reflexer-style self-evolve over the score blend (the part of Tetris that
      transfers).** LANDED (Plan 005 / Bench 064, 2026-09-27): the joint genome lane
      `--genome-select` — `{route, head, nb(+α,+view), oc, ridge}` as one genome line,
      coordinate descent on the cal slice (build-once-fitted + `set_blend_scales`, a
      coordinate eval is one scoring pass), acceptance-vs-seed bar
      `--genome-accept-margin` (default the house 0.05), test read once at the walk
      end. banking77 +3.60 and sst5 +0.50 ADOPTED (both G1-passing); massive +5.33
      REJECTED on G1; emotion REJECTED on accuracy; RRF DECLINED (see Bench 064).
      Original spec, kept for the record: treat `{drafter, route, head, nb}` weights
      plus the smoothing α as a genome line, and hill-`climb` on the **cal/validation
      slice only**, accepting only above a fixed margin. Optionally fuse the rankers
      with RRF (`riir-rag/src/rrf.rs` shape). Read test once, at the end.
- [x] **T6 — GOAT gate** — run for T1/T3 at Bench 051: G1 PASS all suites, G2 p99 43 µs, G3 unarmed suites byte-identical, G4 0 allocs armed → PROMOTED (`nb_scope` default-on). Re-run per new lever. Promote a lever to default only if (G1) accuracy was selected
      on cal and read on test once and beats the current default by ≥ 5 pt on a suite
      without regressing any other suite beyond noise; (G2) p50 stays sub-ms class
      (current 0.46 ms); (G3) nothing changes on suites where the lever is not armed;
      (G4) the hot path stays alloc-free (pre-built tables). A loser is demoted.

- [x] **T1b — noul polarity** (`0d8eaa0`): `nb_noul_domain`, one
      candidate per domain on noul suites, selected on the cal slice.
      prompt_injections still selects OFF: its train-derived slice reads
      ~.50 for both polarities, reproduced outside the harness at the same
      split. That is a slice property, and the protocol declines it.
- [x] **Transductive column** (owner ask, 2026-09-26): published beside
      the headline, never in it. Measured +0.3 to −4.0 pt (Bench 051): train
      already supplies the vocabulary; self-labelling feeds back errors.
- [x] **T7 — next levers (from the 2026-09-26 GOAT hunt):** (a)+(b) LANDED
      2026-09-27 (Plan 004, `.benchmarks/057_issue038_t7_levers.md`): (b) the
      option-conditioned per-(qid, option) count tables (`option_cond`, promoted
      default-on) — typed_decisions 0.33 → **0.4655** (+13.5 pt, cal-selected
      oc@2; per-kind choice .415/noul .592/score .364 at the probe); (a) the
      NBSVM closed-form ridge (`nb_ridge`, promoted default-on) — emotion
      0.7375 → **0.8475** (+11.0 pt, cal-selected ridge@2, calibrated ECE 0.0
      vs floor 0.204; the banking77/ag_news/sst5/xnli/massive ladders all
      decline — G3 byte-identical). Probed first (scripts/issue038_t7_probe*.py):
      banking77 +1.2 (under the 5 pt bar), sst5/xnli WORSE (shipped NB/pair
      views win), ag_news/massive flat — none armed. (c) BM25 kNN vote and
      (d) the Hebbian bilinear xnli map remain UNTRIED — the T5 genome
      (Bench 064) landed 2026-09-27, so (d)'s reopen condition is MET
      (the genome walk held xnli at every coordinate, so the suspected
      residual is representational, not blend-weight — (d) is the probe
      for that); (c) rides the same probe-first discipline. The suspected "T-pass / looped transformer /
      shallow reasoning" items were checked and all need a model (LT2
      `forward_looped` is a throughput GOAT only; "shallow reasoning" is a
      positioning phrase whose shipped form is a kNN + operator tokenizer).
      None applies to the modelless lane.

## Protocol rule — test data NEVER enters a corpus, grammar, or vocabulary

Asked 2026-09-26: "run all data … test data should include for build grammar only
purpose too?" **No.** Building the corpus IS this engine's training step: the corpus
is the model. Putting test rows into it, even unlabeled and even "for grammar/vocab
only", is test-set contamination, and it voids the laya comparison, because laya never
saw those rows. This repo already refuses the milder form: selecting a cap on test
(Bench 003:62-64), a pair on test (Bench 005:32-34), and cal-in-corpus (AGENTS.md:330-333).
The `slice_leak` / `acc_deleaked` column exists to catch exactly this.

- Allowed: **all train + validation rows** for corpora and tables. Select on cal or
  validation, read test once.
- If a transductive number is ever wanted (unlabeled test text for IDF or vocab
  only), it goes in its own column labelled `transductive`, never in the headline
  `acc`.

## References

- `.benchmarks/045_full_m3/` (the gap table source), `003_corpus_cap_lever.md`,
  `005_pair_head_lever3.md`, `040_label_heads_head_select.md`
- katgpt-rs `crates/katgpt-core/src/contrastive_scope.rs`, `.benchmarks/669_contrastive_scope_poc_goat.md`
- katgpt-rs `.benchmarks/891_tetris_lookahead_poc.md`, `892_laya_h2h.md` (why Tetris won)
- `.issues/036` (a sibling degenerate-drafter finding; T1 here may help its T2 too)
