# Plan 012 — the JDI panel read: score the serving ladder on the Jev Decision Index's own axis

**Status:** COMPLETE 2026-10-14 — filed 2026-10-13 (M3). Landed: reflex `8d1a9f7`; B2 clef
fold landed 2026-10-14 (this commit; rethink bench 067 carries the three-lane record). Feeds
riir-rethink Issue 032 T1 (the SubQ Snap beat lane's floor read, DONE at Bench 067) and T3.5's
router (the clef score column over the tranche-1 rows). Grounding: `.research/005` (the JDI
structure study), the LIVE board
snapshot 0.3.2 @ `2026-10-12T13:25:35Z` (sha256 `5d405036…ebe49`, fetched direct-curl per the B1
law), riir-rethink `.research/004`. Phases D/E stay QUEUED (`- [-]`, each its own session slice).

## The board formula — verified exactly (the T0 finding this plan is built on)

`public_skill = 100 × Σ_area area_weights[area] × category_skill[area]` where, per panel
benchmark, `skill = clip01((score − chance)/(1 − chance))` on the BOARD's pinned `chance_levels`,
the category mean is **weighted 1.2 for the 13 `suite.gold` benchmarks / 1.0 for the rest**, and
the area weights are the shipped `suite.area_weights`
(knowledge .258494 · language .258494 · retrieval .200229 · tools .182783 · arts .1; `area_sizing`
sqrt). Verified to the board's own printed 2-decimal figures on three rows: jev 57.9692 vs 57.96,
`decision-27b-ckpt-nothink` 59.0517 vs 59.06, `rsi-jev-v6.1-vl-27b-auto` 65.6529 vs 65.64.

- Panel = the 37 `in_suite` benchmarks minus the 5 `not_in_index` (RouterBench, SGD, MMLU,
  ARC-Easy, ARC-Challenge — run by the board but excluded from the index).
- The board's per-benchmark metric varies: accuracy (GPQA, WinoGrande, HellaSwag, MMLU-Pro,
  BBH, MuSR, HoVer, PhishNChips) vs macro-F1 (BANKING77, CLINC150, ANLI, NLI4CT, ContractNLI,
  FinEntity, VAST) vs exotic (BFCL case-exact, RAGTruth class-F1, iSarcasm track-F1, ACOS
  per-review F1). Our aggregate picks the board's metric per benchmark.
- `balanced_skill` (the headline) = the BLENDED formula `0.20*public + 0.50*private_same +
  0.30*private_new_domains` — NOT computable from public data and never quoted for a self-served
  read (the aggregate law, rethink Research 004).

## Phase A — tranche-1 suites (the choice-shaped public panel subset)

The JDI panel is NOT our 9 harness suites (GPQA, GSM8K, BFCL, chess, cartoons…). Overlap audit:
BANKING77 is the only benchmark we already serve. Tranche 1 = the four public benchmarks whose
official splits ARE the board's rows (split sizes match the board's `cases` exactly), fetchable
parquet-backed from the datasets-server, and castable to the choice wire with a FIXED label
universe (positional keys + option texts as descriptions — the xnli pattern, so the engine label
universes stay small):

- [x] A1 `winogrande_jdi` — JDI id 28, area knowledge, metric accuracy, chance 0.5, board cases
      1267 = `allenai/winogrande` config `winogrande_debiased` split `validation` (1267, the
      labeled eval split — HF `test` is unlabeled). 2-way: keys `option1`/`option2`, descs =
      the option texts; gold = `answer` ("1"/"2") − 1. Train split fetched for the corpus pool.
      **LANDED 2026-10-13** — validation 1267 + train 4000 fetched; smoke A0 0.4830 (debug,
      not quotable — Phase B re-measures).
- [x] A2 `hellaswag_jdi` — id 29, language, accuracy, chance 0.25, board 10042 =
      `Rowan/hellaswag` split `validation` (10042). 4-way: keys `1`..`4`, descs = the endings;
      gold = `label` int. `test_cap` 2000 (B5-disclosed cap; laya-full would be ~10 min but the
      clef lane prices the population). Train fetched at TRAIN_CAP.
      **LANDED 2026-10-13** — validation 8000 (eval universe; the cap lives in the harness) +
      train 8000 fetched; smoke A0 0.2495 (debug, not quotable).
- [x] A3 `anli_jdi` — id 12, language, **macro-F1**, chance 0.3324, board 3200 =
      `facebook/anli` splits `test_r1`(1000)+`test_r2`(1000)+`test_r3`(1200) concatenated in
      that order. Label ClassLabel order [entailment, neutral, contradiction] == the XNLI_KEYS
      order — the xnli criteria/keys reused verbatim (index identity holds, macro-F1 clean).
      `eval_split` gained comma-list support in `load_rows` (one function, all callers pass
      through) so one suite reads the three-way union. Train rounds fetched at cap.
      **LANDED 2026-10-13** — r1+r2+r3 = 3200 + train_r1 4000 (as `train`) fetched; the
      ClassLabel order verified on the fetched features; smoke A0 0.3281 (debug, not quotable).
- [x] A4 `mmlupro_jdi` — id 57, knowledge, accuracy, chance 0.1109, board 12032 =
      `TIGER-Lab/MMLU-Pro` split `test` (12032). Variable option count (4–10): keys `A`..`J`
      (the board's own letter convention), descs = the option texts; gold = `answer_index`.
      `test_cap` 2000. No train split exists (validation 70 = the few-shot prompt pool) —
      fetch validation as the corpus/cal pool and disclose the starved corpus (the A0 floor on
      MMLU-Pro reads corpus-thin BY CONSTRUCTION).
      **LANDED 2026-10-13** — test 12032 + validation 70 (as `train`) fetched; full-test
      option-key union = the ten letters (dispatch 10 confirmed live). DEVIATION: registry
      `cal_cap` 32, not 200 — a 200-row cal front identity-splits the 70-row pool to empty and
      the corpus guard refuses (the pool-after-cal law); disclosed in the registry + manifest.
      Smoke A0 0.1150 (debug, not quotable).
- [x] A5 fetch blocks (`probe_license` + `fetch_suite` per split) + `dataset_manifest.md` rows
      + licence probes landing as `license.json` per suite. **LANDED 2026-10-13** — probes:
      winogrande + hellaswag carry NO hub licence tag (⚠ rows); anli `cc-by-nc-4.0` (⛔
      benchmark-only); mmlupro `mit`. `fetch_suite` gained an optional 6th `as-split` arg (the
      stored filename when it differs from the fetched split — all existing callers unchanged).
- [x] A6 count floors + hand-case build tests in `tests/harness_units.rs`; the full `cargo test`
      green; clippy `-D warnings` default + `--all-features`. **LANDED 2026-10-13** — 5 new
      tests (4 hand cases + the fetched-count floors 1267/8000/3200/12032 + the ten-letter
      union); gates: clippy default + all-features green, `cargo test` 499 passed / 0 failed /
      1 ignored; end-to-end smoke over all four suites PASSED (slice-integrity OK ×4, dispatch
      2/4/3/10 domains). All four suites are `named_only: true` (the wanli/thai precedent —
      Phase B addresses them via `--suites`; the default board population stays stable).

Banking77 already has its JDI-protocol read (the existing crosswalk cell, population
`fnv1a64-dd8ab35333abb82a`, 500-cap) — carried into the record as-is, never re-run at a
different cap (the digest-pin law).

## Phase B — the ladder run

- [x] B1 A0 modelless + laya (ENC) over the four suites, `LAYA_DEVICE=metal`, `--release`,
      box state per `scripts/bench_preflight.sh` quoted beside every number. LANDED 2026-10-13:
      preflight PASSED (`load=5.11 powermode=2(high)`), 4 suites + the banking77 re-read, both
      lanes, ~7 min total; rows in rethink Bench 067.
- [x] B2 clef reference column (the local `clef-flash-4bit` rig) — UN-DEFERRED at verdict
      round 1 (benchmarks are not deferred): **RUN COMPLETE + FOLDED 2026-10-14.** The detached
      runner executed all four tranche suites 2026-10-13 04:45–06:20Z (instrument `9061f98`,
      local rig port 8792, det ✓ ×4, ZERO case absences — each harness run PASSED clean).
      Cells: wino 0.9708 · hellaswag 0.9875 · anli mF1 0.5550 · mmlupro 0.6280; banking77
      CARRIED from bench 113 per the carry law. Folded into rethink bench 067 (three lanes:
      **clef 71.00 subset / 36.71 zeros-included**); raw artifacts at
      `rethink .benchmarks/067_jdi_public_skill_read/clef_read/`. Disclosures in the record:
      the digest delta on wino/anli (pre-commit WIP build vs the committed 8d1a9f7
      instrument — same population, the B2 run's modelless lane reproduced the tranche A0
      rows bit-identically, the witness), latency NOT QUOTABLE (sibling load), and the
      runner's DONE-marker false negative (its `absence` grep trips on the deliberate
      laya-absent row; data was never at risk). The T3.5 router feed: the clef score column
      over the tranche-1 rows is now in the record next to A0/ENC.
- [x] B3 `results.json` + `tables.md` into rethink `.benchmarks/067_jdi_public_skill_read/`
      (the scoring-run record lives in rethink per the decision-index-beat records table;
      reflex owns the instrument, rethink the board consumption). LANDED — the record dir
      carries the raw harness outputs + the self-verifying aggregate script.

## Phase C — the aggregate + the record

- [x] C1 aggregation script (rethink side, committed beside the record): per-benchmark score
      (the board's metric per benchmark) → board chance → gold-weight → category mean →
      area-weighted `public_skill` on the COVERED subset; the zeros-included full-panel bound
      (unserved benchmarks score 0 per the JDI's own unanswered-wrong law); both quoted, never
      blended. Board chance levels + gold weights + area weights pinned from the snapshot
      (edition + hash beside). PLUS the T0 discovery the plan predicted: the board's category
      mean is gold-WEIGHTED (1.2 for the 13 `suite.gold` members) — the fold is exact on the
      clean anchors (jev 57.9692 vs 57.96); below-chance rows fold NEGATIVE (no clip — the
      gemma4-31b anchor); the board's private fold still carries a residual on partial-coverage
      entrants (instrument uncertainty, disclosed in the record).
- [x] C2 the record states the coverage honestly: the T1 read covers 5/37 panel benchmarks
      (banking77 carried + the four new suites — all five gold-key members; knowledge 2/11,
      language 2/9, retrieval 1/6, tools 0/5, arts 0/6) — a SUBSET read, never presented as a
      board-rank claim.
- [x] C3 rethink Research 004 position table + Issue 032 T1 checkbox updated; the floor number
      named: **A0 6.10 / ENC 19.41 subset (4.38 / 13.92 zeros-included) vs the 58.0 M1 bar** —
      the gap is coverage + the missing teacher, exactly the beat lane's anticipated shape.

## Phase D — tranche 2 (queued, each its own session slice)

- [-] GPQA Diamond (id 25, gold-key, 198 cases): gated download on HF; needs a verified mirror
      or the owner's dataset grant — provenance disclosed either way.
- [-] CLINC150+OOS (id 5, gold-key, 5500): mirror availability probe pending.
- [-] MuSR (id 32), ContractNLI (id 11), iSarcasmEval (id 40), HoVer (id 61), NLI4CT (id 42),
      VAST (id 41), FinEntity (id 39), BBH (id 58, 23 subtasks), HellaSwag full, RAGTruth
      (id 59, span-shaped — needs a case-level cast first).
- [-] The non-choice panel (tools/function-calling: BFCL, ToolRet, API-Bank, When2Call, Home
      appliance; arts: BPoMP, Humicroedit, POP909, cfcolor, Habermas, New Yorker; retrieval:
      BRIGHT, ESCI; knowledge: GSM8K, ChessBench, SATA, CRUXEval, CLadder, HLE, PhishNChips) —
      NOT servable on the decision wire today; standing coverage loss, named in the record,
      not argued away.

## Phase E — composition reads over the JDI suites (queued)

- [-] The incumbent H2 (instinct arena) + the rethink ESC lane consume the reflex seats — run
      both over the tranche-1 suites once seated; same digest pins.
- [-] The pplx lane (Bench 132's rig) capture over the JDI rows — feeds rethink 032 T3.5's
      modelless router (pplx + clef score columns captured in the SAME window).

## Validation

- [x] `cargo test` green; clippy `-D warnings` (default + all-features); count floors pin the
      four new suites. **SATISFIED at A6 landing (2026-10-13, commit `8d1a9f7`): 5 new tests,
      clippy default + all-features green, `cargo test` 499 passed / 0 failed / 1 ignored. The
      B2 fold (2026-10-14) changed no code — data + docs only; that green state stands.**
- [x] The aggregate script reproduces the board's own printed figures from the shipped
      per-benchmark results (the T0 verification, re-run in the record) BEFORE our rows are
      aggregated through it — the instrument proves itself on known answers first.
      **SATISFIED: STEP 1 runs on every invocation (aggregate.py main); re-proven in both the
      2026-10-13 fold and the 2026-10-14 B2 fold (`aggregated.md` STEP 1: clean anchors 1/1
      OK, gold-weight pin re-verified).**
- [x] Every quoted number carries: edition `0.3.2`, `generated_utc`, snapshot sha256, box
      state (preflight) for latency rows. **SATISFIED: bench 067 RESULTS.md carries the
      edition/generated_utc/sha256 header + the preflight line; the B2 fold adds the clef
      provenance block (instrument commit, run window, rig posture, box-state refusals,
      latency-not-quotable).**
