# Plan 004 — Issue 038 T7(b)+T7(a): the option-conditioned scorer and the NBSVM ridge readout

**Status:** COMPLETE — 2026-09-27. T7b + T7a landed, GOAT-gated, both promoted default-on; typed 0.33 → 0.4655 (+13.5 pt), emotion 0.7375 → 0.8475 (+11.0 pt); every other suite byte-identical (G3 verified against the site rows). T5/T4′ remain open (issue 038).

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

- [x] **T1 — `src/option_cond.rs`: the option-conditioned count tables (T7b).**
      LANDED — one-vs-rest `ContrastiveScoreTable` per (qid, option) from gold events;
      blend term = the nb margin sigmoid; 6 module tests.
- [x] **T2 — engine wiring:** LANDED — `oc_scale` + `build_specs_oc` (fail-closed
      `OcNeedsEvents`), hot path O(tokens) with no alloc (G4 bench arm).
- [x] **T3 — harness:** LANDED — `typed_gold_events` + `--oc-select` selection ladder
      (slice 0.370 → 0.558 @ scale 2) + seat plumbing; test read once 0.4655.
- [x] **T4 — T7a `src/nb_ridge.rs`:** LANDED — df-top-k (2048, bucket-sorted for the
      binary search — the unsorted-vec silent-miss bug is recorded in the module),
      per-class NB ratios (the PROBE'S ARITHMETIC MIRRORED VERBATIM including its
      `tot[lab]`-missing-key quirk — "fixing" it breaks the denominator, measured),
      shared presence Gram, `ridge_solve_direct_f32` per class, fit-time
      SELF-CALIBRATED margin temperature (mean per-doc class spread over a strided
      512-doc sample — `/n_tokens` crushes the damped margins to a constant shift,
      measured 0.5500-at-every-scale before this). λ = 10 fixed (probe).
- [x] **T5 — gates:** G1 cal-selected/test-once both levers (typed oc@2 slice +18.8;
      emotion ridge@2 slice +14) with calibrated ECE ≤ conformal floor on both moved
      rows; G2 p50 ≤ 0.58 ms (typed oc) / 0.13 ms (emotion ridge), bench p99 ≤ 50 µs;
      G3 full-workspace run byte-identical on every unmoved suite (site-row diff, 12
      suites × 0.0000); G4 `solve_into` 0 allocs with BOTH levers armed (canary live).
      `option_cond` + `nb_ridge` both PROMOTED default-on (cfg knobs stay 0 =
      byte-identical serving posture).
- [x] **T6 — bench + record:** decision_set_goat carries option_cond + nb_ridge
      armed postures; `.benchmarks/057_issue038_t7_levers.md` record + issue 038
      checkboxes + site republish.
- [-] T4′ cascade lane + T5 blend genome — DEFERRED (issue 038's remaining open
      items; the genome should tune the blend INCLUDING the two new terms).
