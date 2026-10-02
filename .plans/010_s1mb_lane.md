# Plan 010 — the S1MB lane: bench our three lanes on the System One Mosaic Benchmark, publish a site section

**Status:** IN FLIGHT — owner directive 2026-10-02 ("bench s1mb against only
Reflex, Instinct, Rethink and promote to new section after
reflex.gist.rs/bench/#timing"). Opens the riir-train Issue 607 gate for THIS
scope (our lanes, our metric) — NOT the 607 repro path (their eval code,
card-row reproduction), which stays deferred.

## What S1MB is (grounded 2026-10-02, HF card)

`hotchpotch/s1mb-dataset` — 106 active subsets, **14,009 test-only cases,
26,269 judgments**, parquet, one `test` split per config (train = 0). The
`input` column is **literally the System One request format our wire already
speaks**: `{"state_json": "...", "decisions": [{id, kind: judgment, type:
choice|score|noul, instructions_json, criteria, ...}]}` — the exact shape
`scripts/bekko_lane.py::build_input` produces FROM our wire. `targets` carry
per-decision hard-label distributions (`ids` + `probabilities`; noul ids come
in BOTH orders — map to OUR fixed [false, true]). Subsets include the
`laya__` namespace (OUR dataset families converted: typed_decisions,
ag_news, enron_spam), `open_jev__*` (the Jev control tasks), the 6
`s1mb-generalization-*` authored subsets, and the classic eval sets (arc,
banking77, massive, snli, ...).

## Protocol decisions (recorded, disclosed everywhere)

1. **Our metric, never theirs**: forced-pick accuracy per our harness
   conventions. S1MB's own scoring is baseline-adjusted skill — the numbers
   are NOT comparable to the S1MB leaderboard and must never be mixed with it.
2. **Corpus/test split (S1MB is test-only)**: the converter writes a
   deterministic 50/50 split per suite — corpus+cal half (train-000.json) /
   test half (test-000.json) — stratified by (subset, type) in
   hash/input_hash order. All three lanes see the SAME halves.
3. **Flatten per judgment**: one single-question case per (case × decision) —
   26,269 rows total, `group_id` preserved in the case id. Maximizes reuse:
   every existing lane (seat, Arm A, encoder flattening, metrics) consumes
   single-question cases unchanged.
4. **Three suites** by decision type: `s1mb_choice`, `s1mb_noul`,
   `s1mb_score` under `.raw/datasets_s1mb/<suite>/`, the harness page format
   (train-000/test-000 + splits.json). Domain-vs-generalization split rides a
   `group` field on each row (the 6 `s1mb-generalization-*` subsets) — the
   bench record and site section can split the view.
5. **named_only suites**: never a default-run member (13k test rows would tax
   every run).
6. **Three lanes**:
   - **Reflex** = the modelless lane (harness run).
   - **Instinct (hybrid)** = the arena's A1/H1/H2 over **Arm A gold-trained
     specialists** on the corpus half (no teacher dump — S1MB has none; the
     bekko-teacher variant is a follow-up, reusing Issue 608's T1 seam).
   - **Rethink (encoder)** = the **v2 per-option NLEH head** (the t608 shape
     — the only head class that speaks S1MB's variable per-question candidate
     sets) over the laya-typed backbone, trained on the corpus half, replayed
     via the arena's `--encoder-art` over the test half. Record-only posture
     on the site (the class-wide serve refusal stands, Issues 014/016).

## Tasks

- [x] T1 converter: `scripts/s1mb_fetch_convert.py` — DONE reflex `c6cc1e7`
      (all 106 configs, 0 skips; state-coharent 50/50 split + exact-dup dedup;
      /rows envelope shape; gold by ID; noul definitions folded into
      instructions; raw state_json strings). Counts: choice 4330/4330,
      noul 6174/6173, score 2574/2574 = 26,261 after dedup.
- [x] T2 reflex harness: build_s1mb_{choice,noul,score} + the label arms +
      3 named-only SuiteSpec rows + dispatch arms 10/15/842 + the scoped
      slice-guard coverage flag — DONE reflex `c6cc1e7`; 3 builder tests;
      clippy -D clean; 245 lib tests green.
- [x] T3 Reflex row (modelless lane, t20k posture, preflight
      `power=AC Power load=3.99 swap=2603.25M canary=119.3us/best5
      powermode=2(high)`): **s1mb_noul 0.7055** (p50 0.292 ms) ·
      **s1mb_score 0.4967** (p50 0.243 ms; majority-class ≈ 0.48) ·
      **s1mb_choice 0.2296** (p50 0.399 ms; 842-domain by-name posture,
      464 starved-key self-doc fallbacks disclosed; wall 547 s). Tables in
      /tmp/smokes — RE-RUN into .benchmarks/105 at record time (T7).
- [ ] T4 Instinct row: riir-train `instinct_arm_b` gold-only path
      (`--arm-a-only`, the teacher dump optional) → winner artifacts for the
      3 suites → instinct arena runs (`--datasets-dir .raw/datasets_s1mb`,
      the 3 suites are SEATABLE — prepare_seat reads the registry) → A1/H1/
      H2 best per suite. instinct .benchmarks record (next number).
- [ ] T5 Rethink row: laya-typed encodes of the corpus half → lenc cache →
      the NLEH v2 per-option head trained over the S1MB label space
      (riir-train encoder trainer; may need an S1MB adapter — coordinate
      with the Lane E owner session, instinct Issue 018, which cites this
      exact intel) → sealed artifact → arena `--encoder-art` replay over the
      test half. Record-only, `serve: ✗`.
- [ ] T6 site: `reflex-site` — publish_bench.py + the /bench/ page render a
      new S1MB section positioned after the #timing section (per-lane rows ×
      3 suites + overall; disclosure line: our-metric, not leaderboard-
      comparable; corpus-half split disclosed). Self-tests + smokes + commit;
      wrangler deploy is manual. ⚠ COORDINATION: a sibling session is active
      in reflex-site (bench.json areas v3) — read the tree first, commit only
      this section's files.
- [ ] T7 records: reflex `.benchmarks/105_s1mb_lane/` (all three lanes'
      numbers + the domain-vs-generalization split + full disclosure) +
      riir-train Issue 607 updated (the our-lanes scope executed; the repro
      path still deferred) + cross-refs.

## Gates / honesty

- No T2-style superiority gate here — this is a new external comparison
  surface, not a serving change. Nothing served changes (the encoder row is
  record-only; the hybrid row is informational).
- The domain-vs-generalization split is REQUIRED reading in the record:
  Issue 607's fact #2 (laya-typed wins generalization, loses domain breadth)
  is the hypothesis this bench measures on OUR metric.
- bekko home-field disclosure: S1MB domain subsets come from the dataset
  families bekko trained on (and the `laya__` subsets are OURS) — quote it in
  the record and the site section.

## Cross-refs

- riir-train `.issues/607_s1mb_laya_external_eval.md` — the intel issue this
  executes (owner gate opened by directive 2026-10-02, our-lanes scope only)
- reflex Bench 103 + `--bekko` lane — the sibling comparison lane; the
  bekko/17m-68m rows for context
- reflex Issue 608 T1 (`20527e6`) — the teacher seam a future
  teacher-distilled S1MB specialist would reuse
- instinct Issue 018 Lane E — the Rethink encoder lane this feeds
- The site section target: reflex.gist.rs/bench/#timing (section lands after
  it)
