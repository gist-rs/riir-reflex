# Bench 170 — the elephant teacher joins the T2.5 routed fold (riir-train Issue 629 T2.5 addendum)

**Status:** RECORD — router executed 2026-10-28T04:49Z (amo, release `4c1d21e`-tree harness, zero GPU,
train rows only) · inputs `.raw/distill_router_elephant` (the 4090's 6-suite routed fold, scp'd) ⊕
`.raw/distill_teacher_elephant` (the Bench-164 recapture, 7 suites) · out
`.raw/distill_router_elephant_plus/` (ROUTER.md + router.json committed here) · **5 suite(s) folded:
4 routed + 1 passthrough; 2 refuses (posture-mismatched, below)**. Nothing here is a test-row read —
the routed artifact's verdict stays T4's single frozen read (the 629 law).

## The fold (per-suite)

| suite | sources (train acc) | gate g | routed acc | cv (loo) | mode |
|---|---|---|---|---|---|
| ag_news | elephant 0.8616 | — | — | — | passthrough (router col has no ag_news) |
| banking77 | router 0.9492 ⊕ elephant 0.9416 | 0.6815 | **0.9526** | 0.9525 | blend |
| massive_intent_en | router 0.9681 ⊕ elephant 0.9530 | 0.8192 | **0.9687** | 0.9687 | blend |
| prompt_injections | router 0.9890 ⊕ elephant 0.8187 | 1.0000 | 0.9890 | 0.9890 | blend (gate saturated to the router col) |
| xnli_en | router 0.9199 ⊕ elephant 0.8853 | 0.9694 | **0.9197** | 0.9197 | blend |

Elephant routed lift is REAL but marginal on the gated suites: banking77 +0.34 pt, massive +0.06 pt,
xnli −0.02 pt (the gate correctly keeps the router column dominant), prompt_injections unchanged at
gate 1.0000. The elephant's own train-row accuracies ride Bench 164's capture record.

## The two refuses (the finding of this run)

`emotion` (16000 vs 15999) and `sst5` (8544 vs 8533) refuse the join on **row-count mismatch — and the
mismatch is a DATASETS-POSTURE split, not a bug**:

- The 4090 captures (`d3`/`pplx_v2`/`clef`, Bench 144's lane) read `.raw/datasets_t20k` **as pulled
  2026-09-26**: emotion 16000 / sst5 8544 rows (the pre-dedupe recorded aggregates, dataset manifest
  "Full train pull" table).
- The elephant capture (Bench 164 recapture, amo) read amo's **canonical** `.raw/datasets` tree, which
  carries the **Issue-058 fetch-plane dedupe**: emotion 15999 / sst5 8533 (1 + 11 exact dupes dropped).
- Verified locally: amo canonical emotion train rows = 15999, sst5 = 8533; the t20k `sst5/train-*.json`
  pages amo still carries are byte-identical to canonical (the amo t20k sst5 re-derives 8533 — it was
  deduped in place), so the 8544 count lives ONLY in the 4090's t20k tree.
- Consequence: **the 4090 and amo teacher captures are not row-alignable on emotion/sst5 across
  boxes** — the gold-pin join would refuse on the extra rows even if the counts were faked. The
  router's refusal is the designed loud-skip doing its job (never guess a join).

## Consequences (recorded, nothing armed)

1. The routed fold stays **9→7→5-suite-shaped per posture**: the JDI-public 5-suite column has no
   committed capture (anli_jdi / hellaswag_jdi / mmlupro_jdi / typed_decisions / wanli_en) — the
   elephant's Bench-164 capture covered the reflex-comparison 7 suites (ag_news, banking77, emotion,
   massive, prompt_injections, sst5, xnli_en), of which emotion + sst5 refuse the cross-box join.
2. **No re-router shrink of the 6-suite routed artifact is armed**: the shipped `.raw/distill_router/`
   (4090, Bench 144) remains T4's teacher column for the d3/pplx/clef lane. The elephant fold lives in
   `.raw/distill_router_elephant_plus/` as an ADDITIVE artifact; which fold T4 reads stays the
   pre-registered 623/629 posture decision (owner-gated lane).
3. **Posture law for future captures**: any new teacher capture intended for a cross-box router join
   must name its datasets posture (pre/post-dedupe) in the RIDT header — the elephant + 4090 splits are
   the measured precedent. A same-posture re-capture of d3/pplx/clef on the amo canonical tree would
   re-open emotion/sst5 to the fold; until then the 5-suite fold is the MAXIMUM honest cross-box fold.
4. Elephant's marginal routed lift (+0.34 pt worst-case on one suite) does not re-order the teacher
   board; the mammoth/elephant comparison rows stay Bench 162/164's (elephant adds ZERO routed lift at
   the 4-lane fold — that verdict was about the RELEASE fold's routed row and stands).

## Provenance

- Router: reflex release harness at the working tree `4c1d21e`-era (the bench-144 router code path,
  unchanged); `--teacher-router .raw/distill_router_elephant,.raw/distill_teacher_elephant
  --router-out .raw/distill_router_elephant_plus`; sigmoid T=0.01, never softmax; the leave-one-out
  CV column is train-row (no test touch).
- Inputs: `.raw/distill_router_elephant/` = byte-copy of the 4090's `.raw/distill_router/` (Bench 144
  artifacts, scp 2026-10-28); `.raw/distill_teacher_elephant/` = the Bench-164 §6 recapture (7/7
  suites END, per-suite serve hygiene).
- Box: amo (M5 Ultra), CPU-only; the 4090 box was ssh'd read-only for the byte-copy (no GPU consumer
  started; the gitsync quirk unaffected).
