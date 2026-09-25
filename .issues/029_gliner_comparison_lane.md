# Issue 029 — the GLiNER comparison lane (fastino/GLiNER2.5-Decide)

**Status:** OPEN — the lane LANDED 2026-09-25 (`scripts/gliner_lane.py` +
`run_gliner_lane` + `--gliner` + the site publisher/charts/filter, all
landed on `develop`); the FIRST CELLS ran same-day on the 4090 window
(host `4090-windows`, all 15 suites, no absences, gliner determinism ✓ on
every suite) and are published to the site repo's `data/bench.json`
(lane-update merge: modelless + laya + gliner refreshed for that host, the
bench-034 clm cells carried over untouched). The site DEPLOY rides the M3
(the standing `.issues/027` posture — this box has no CF creds). Verdict
vs laya (same run, same box): **gliner beats the laya BASE checkpoints on
9/15 suites** (banking77 0.706 vs 0.498 · typed_decisions base 0.528 vs
0.3575 · massive_intent 0.823 vs 0.750), **loses on classic NLU** (ag_news
0.70 vs 0.95 · xnli 0.48 vs 0.86 · emotion 0.57 vs 0.59) and **does not
beat the laya `typed` specialist on the headline suite** (0.528 vs
0.7445). Latency: gliner p50 22–32 ms (subprocess IPC included, fp32
CUDA); on long-context typed_decisions it is 3.5× faster than the
in-process laya-riir cuda lane (30 vs 107 ms), on short suites ~2× slower
(22 vs 10–12 ms). The fast-decisions LANDSCAPE ROW landed 2026-09-25
(their published table quoted as published — verified against both their
cards — beside our measured 15-suite verdict; renderer `## Landscape`
block + README Results/attribution, the `.issues/025` pattern).

## Why

fastino/GLiNER2.5-Decide (Apache-2.0, 340M DeBERTa-v3-large encoder)
publishes a zero-shot classification benchmark claiming it beats "Laya
Router" 60.2% vs 46.6% on their `fast-decisions` suite. The arena's
charter (the `.issues/019` clm-lane precedent, the `.issues/025`
positioning row): external "System One" challengers are MEASURED on OUR
harness under OUR protocol and published beside the incumbent — vendor
numbers are never transcribed. The wire maps cleanly (their schema-driven
`classify` ⇄ our choice/score/noul questions; labels-with-descriptions is
their native `{label: description}` form), and their `ClassificationScores`
exposes per-label probabilities (softmax over labels for single-label
heads) — everything the metrics tail needs.

## What landed

- `scripts/gliner_lane.py` — THEIR package as a JSONL subprocess oracle
  (the `laya_python_lane.py` protocol): one schema per case (a task per
  qid — their several-decisions-at-once pattern, one forward for the whole
  case), state rendered as `key: value` prose (OUR choice — disclosed),
  choice → single-label task over `LabelSpec(key, description)`, score →
  ordinal task, noul → `["no","yes"]` (the wire's `[1-n, n]` convention).
  Their structural-token refusal (labels/instructions may not carry
  `(`/`)`/`[L]`/…) is handled by a DISCLOSED strip — the alternative is
  dropping every paren-carrying case, a coverage loss, not a purer
  measurement. The loader's stdout banner is redirected off the protocol
  channel (the laya-python lane's law). `conf` = top-label probability
  (their probability readout; they expose no separate confidence scalar on
  the scoring path).
- `run_gliner_lane` (runner.rs) — spawns the oracle, stamps the
  HANDSHAKE-advertised model id + device into the row (never hardcoded),
  the trim law (same `--laya-max-questions` cap as the laya lanes), the GPU
  pre-ramp warmup (Issue 020's law, `LAYA_HARNESS_NO_WARMUP=1` restores
  the cold posture), the observed-repeat determinism check (first 10
  cases), and the SAME metrics tail via `assemble_laya_lane_result` +
  `parse_python_answer` (the answer mapping is the laya-python lane's).
  `SuiteResult.gliner` + `RunMeta.gliner_lane` + `RunOptions.gliner`
  (`--gliner`; NO feature gate — zero new deps: std::process + the
  in-tree serde_json).
- The site ride (gist-rs/reflex-site): the publisher carries + renames the
  lane ("gliner (reference)", the clm carry law verbatim), the bench page
  renders its rows (primary + extra-host), the charts gained clm + gliner
  palette slots with extra-host hero fallback (host-tagged tooltips), and
  the **lane filter checkbox bar** — the owner ask — governs EVERY section
  (hero, per-suite bars, tables) with localStorage persistence.

## Non-goals (decided)

- No riir-infer port of DeBERTa-v3 (disentangled attention) — the lane is
  a MEASUREMENT comparison, never a product lane; the 025 lane-3 deferral
  posture stands.
- No fast-decisions suite adoption — our 15 suites are the protocol;
  their 17-domain suite is their protocol (the landscape row pattern).
- No fp16/quant variants — their fp32 default is the measured posture.

## Remains

- [ ] The site deploy (`npx wrangler deploy` from the M3 — the 027 law).
- [ ] (publisher-owner, cosmetic) `bench.json`'s per-host header row keeps
      the ORIGINAL run's facts by law (Issue 023 T5) — the 4090-win header
      sha (afacc3a/09-24) is stale beside its d69f0c7/09-25 lanes; the
      per-lane truth lives in `lane_sources`. If the header should track
      the latest contributing run, that is a publish_bench.py owner call.
- [x] (optional, next 4090 window) the fast-decisions suite as a SEPARATE
      landscape row — vendor-published numbers quoted as published beside
      our measured row, the `.issues/025` pattern. DONE 2026-09-25 — the
      second static `## Landscape` block in the renderer (`render_markdown`)
      + the README Results paragraph + the `--gliner` quick-start bullet +
      the attribution bullet. Numbers verified against BOTH their cards
      (model + dataset) at fetch time: their full 7-row table quoted; the
      scored split is PRIVATE (public repo = 100/domain dev split with an
      explicit do-not-score note — quoted, never re-runnable here); their
      two cards disagree on the 1B row's name (Decide-1B vs "GLiNER2 XL
      (1B)", same 59.6%) — quoted with both. Committed TABLES.md artifacts
      are run snapshots (never hand-edited); the section appears at the
      next harness run.
