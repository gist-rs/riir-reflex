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

- [ ] T1 converter: `scripts/s1mb_fetch_convert.py` (stdlib-only python:
      urllib + json; the fetch_datasets.sh politeness/resume laws — 0.3s
      sleep, skip-if-complete, loud per-page failures; utf-8 everywhere, the
      locale_io lesson) — `/splits` enumerates configs, `/rows` pages each
      config, flatten judgments → our row shape, deterministic 50/50 split,
      write `.raw/datasets_s1mb/<suite>/` + a fetch manifest with blake3
      digests + row counts (the dataset_manifest law).
- [ ] T2 reflex harness: the generic `build_s1mb` builder (rows carry the
      pre-parsed decision payload verbatim: qid/kind/instructions/criteria/
      keys/gold_idx/gold_score/soft) + 3 SuiteSpec rows (named_only, counts
      asserted at build time). `--suites s1mb_choice --limit`-style smoke.
- [ ] T3 Reflex row: the modelless lane over the 3 suites (t20k-pool
      preflight posture; box-state PROVENANCE quoted).
- [ ] T4 Instinct row: riir-train `instinct_arm_b` gains a gold-only path
      (`--arm-a-only`, the teacher dump optional) → winner artifacts for the
      3 suites → instinct arena runs (`--datasets-dir .raw/datasets_s1mb`)
      → A1/H1/H2 best per suite. instinct .benchmarks record (next number).
- [ ] T5 Rethink row: laya-typed encodes of the corpus half → lenc cache →
      the NLEH v2 per-option head trained over the S1MB label space
      (riir-train encoder trainer; may need an S1MB adapter — coordinate with
      the Lane E owner session, instinct Issue 018, which cites this exact
      intel) → sealed artifact → arena `--encoder-art` replay over the test
      half. Record-only, `serve: ✗`.
- [ ] T6 site: `reflex-site` — publish_bench.py + the /bench/ page render a
      new S1MB section positioned after the #timing section (per-lane rows ×
      3 suites + overall; disclosure line: our-metric, not leaderboard-
      comparable; corpus-half split disclosed). Self-tests + smokes + commit;
      wrangler deploy is manual. ⚠ COORDINATION: a sibling session is active
      in reflex-site (bench.json areas v3) — read the tree first, commit only
      this section's files.
- [ ] T7 records: reflex `.benchmarks/105_s1mb_lane/` (the lane record + all
      three lanes' numbers + the domain-vs-generalization split + full
      disclosure) + riir-train Issue 607 updated (the our-lanes scope
      executed; the repro path still deferred) + cross-refs.

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
