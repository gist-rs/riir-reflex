# Plan 004 — Issue 038 T7(b)+T7(a): the option-conditioned scorer and the NBSVM ridge readout

**Status:** IN PROGRESS — premises probed 2026-09-27 (T7a emotion +15.5pt, T7b typed +13pt); implementation started.

## Premise probes (done, recorded before any Rust)

`scripts/issue038_t7_probe.py` + `issue038_t7_probe_k.py` (throwaway, measurement-only — the
`.issues/038` NB-POC pattern). Protocol: corpora from train rows only, label-stratified
train selection slice, test read once. Tokenization mirrors `src/embed.rs`/`suites.rs`.

**T7(b) — per-(qid, option) counts over the state text, typed_decisions:**

| posture | test acc (2000 q) | published modelless |
|---|---|---|
| mnb/word | 0.4475 | 0.32 |
| idf/word | 0.4585 | 0.32 |
| word+char5 | ≈ same | — |

Per-kind (mnb/word): choice 0.415 · noul 0.592 · score 0.364. Variant forms
(noprior / contrastive margin) read the same → the margin-sigmoid term form is
safe. Char n-grams add nothing. Data volume is fixed (train split total 1200;
800 fetched). Premise: **+13pt, GOAT-worthy; does not pass laya (0.745) — the
cascade lane (T4′) covers the rest later.**

**T7(a) — NBSVM ridge (binary presence uni+bi, NB log-count-ratio scaling,
df-top-k features, closed-form one-vs-rest ridge):**

| suite | best test | published | verdict |
|---|---|---|---|
| emotion | **0.8925** (k=2048, per_class, λ=10) | 0.7375 | **+15.5pt — the lever** |
| prompt_injections | 0.7672 (global, λ=10) | 0.500 | +27pt BUT the T1b slice declined the noul polarity there — protocol question, NOT taken on test-selected evidence |
| banking77 | 0.8384 (per_class, λ=10) | 0.826 | +1.2pt — under the 5pt bar |
| ag_news / massive | ≈ flat | 0.8825 / 0.7800 | not a lever |
| sst5 / xnli | 0.3583 / 0.4367 | 0.3967 / 0.5233 | WORSE — shipped NB/pair views win |

k-sweep (emotion): k=1024 0.6925 · k=2048 **0.8925** · k=4096 0.8875 → **k=2048**
(the Rust Cholesky is O(k³); 2048 is 4× cheaper than 4096 at equal quality).

Note recorded: prompt_injections' T1b selection slice reads ~.50 for both noul
polarities while any full-train count model reads ~.80+ on test — the selection
slice is under-powered on that suite (n=100, and the suite is noul-only so the
ladder has 2 candidates). Not actionable this plan; flagged for a later
slice-size re-measure (selection-side only, protocol-legal).

## Tasks

- [ ] **T1 — `src/option_cond.rs`: the option-conditioned count tables (T7b).**
      Frozen sorted `Vec<(u64, u32)>` keyed `fnv1a(qid ‖ 0x00 ‖ option)`;
      per-key `ContrastiveScoreTable` one-vs-the-SAME-qid's other options
      (katgpt-core `ContrastiveScoreBuilder`, the nb_scope substrate — DRY).
      Fit input: `(qid, option, doc)` events. Score-time term (mirrors
      `NbScope::blend_term`): `scale · σ((in_d − max_other)/n_tok)`, absent
      key → no term. Zero-alloc hot path (binary search + fnv, no alloc).
- [ ] **T2 — engine wiring:** `ExpertSpec::option_events` (build-time only) +
      `EngineConfig::oc_scale` (0 = off); build refuses scale > 0 with no
      events (fail-closed, the `NbNeedsCorpora` shape); hot path adds the
      oc term for any question kind (choice/score keys; noul internal
      "yes"/"no" candidates) — independent of `route_active` (typed is
      drafter_only today and must still arm).
- [ ] **T3 — harness:** `typed_option_events(rows)` (gold → option key per
      suites.rs rules: choice criteria key, score `str(int)`, noul
      true→yes/false→no; doc = the RAW stored state string, the corpus rule);
      oc-scale selection ladder on the stratified slice (the nb_lane shape,
      promotion bar + margin); test read once. G3: every other suite has no
      events → byte-identical.
- [ ] **T4 — T7a `src/nb_ridge.rs`:** df-top-k (k=2048) binary presence
      features over the SAME `hashed_tokens_into` stream; NB log-count-ratio
      scaling R (per-class form; the probe's 3/4 winner); Gram from sparse
      rows (shared, O(Σ nnz²)); `katgpt-core::linalg::ridge_solve_direct_f32`
      per class (λ=10 fixed, probe-selected); score = w_cᵀ(x ⊙ r_c) + b_c.
      `EngineConfig::ridge_scale` (0 = off) + the same selection ladder.
      Emotion is the target suite; a suite whose slice doesn't clear the bar
      stays OFF and pays nothing.
- [ ] **T5 — gates:** feature `nb_ridge` (+ the oc tables under the existing
      `nb_scope`? NO — oc is NOT contrastive-scope-dependent; gate it as
      `option_cond`, default-on-safe); G1 cal-selected/test-once per lever;
      G2 p50 sub-ms (oc lookup + ridge dot are O(tokens)); G3 unarmed suites
      byte-identical (pinned); G4 alloc-free hot path (counting-allocator
      bench arms). Fit-time budget: typed oc fit is O(events) trivial; the
      emotion ridge fit is ONE Cholesky set at k=2048 (probe-instrumented
      before promotion — if scalar solve exceeds ~60 s, k drops or the fit
      moves behind the bench gate).
- [ ] **T6 — bench + record:** `benches/` arm per lever (latency + alloc),
      `.benchmarks/057_issue038_t7_levers/` record with the probe tables
      above, tables + site republish if any published row moves, issue 038
      checkboxes updated, commit + push (docs/feat/fix prefix).
- [-] T4′ cascade lane + T5 blend genome — DEFERRED to after the two levers
      land (T5's genome should tune the blend INCLUDING the new terms).
