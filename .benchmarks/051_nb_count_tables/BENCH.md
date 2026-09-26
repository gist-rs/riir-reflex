# Bench 051 — Issue 038 T1/T3: the count-table lane (`nb_scope`) — honest A/B vs the published modelless posture, + the transductive column

**Status:** MEASURED 2026-09-26 (M3, 4000-row canonical train pull). The T2 corpus-size leg (20k-row pull) is appended below when its fetch lands.

## What ran

Two harness runs, **same box, back to back**, same datasets (`.raw/datasets`,
the manifest-digested 4000-row train pull), `--skip-laya`, 8 dataset suites:

- **baseline:** `--head-select` (the published arena protocol, Bench 045)
- **nb:** `--head-select --nb-select`, built with `--features nb_scope`

Box state (Issue 021): AC power, `powermode` high, load 3.7–4.1, swap
~2.5 GB, sibling sessions active. Latency QUOTABLE per the preflight, but
the two runs are **sequential, not interleaved**, so p50 deltas across runs
are box drift, not a timing claim. The timing claim is the G2/G4 gate
below.

Tree: the nb run was built from the working tree later committed as
`0d8eaa0` (its stamp reads `d66ef21`, the parent, because the pair view was
still uncommitted at build time; the diff between them is the pair view
plus noul polarity, both exercised in this run).

## Headline (honest protocol)

Corpora and count tables come from **train rows only**. The count-table
posture (scale × α × view × noul polarity) is selected on the **stratified
selection slice**: train rows, content-excluded from the corpora, with the
heads' promotion bar (+5 pt over off; nothing clears → off,
byte-identical). **Test is read once.** laya numbers are Bench 045's
published lanes on the same test split.

| suite | n | baseline (head-select) | + nb-select | Δ | selected (scale · α · view) | laya best (Bench 045) | vs laya | G1 | p50 base → nb |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | 2000 | 0.3190 | **0.3190** | +0.0 | off | 0.7445 | −42.6 | PASS | 0.553 → 0.569 ms |
| ag_news | 400 | 0.5100 | **0.8750** | +36.5 | 16 · observed-laplace · bag | 0.9500 | −7.5 | PASS | 0.144 → 0.159 ms |
| emotion | 400 | 0.2825 | **0.5950** | +31.2 | 4 · observed-laplace · bag | 0.5930 | **at/above** | PASS | 0.111 → 0.120 ms |
| sst5 | 600 | 0.2167 | **0.2167** | +0.0 | off | 0.3720 | −15.5 | PASS | 0.096 → 0.102 ms |
| prompt_injections | 116 | 0.4828 | **0.4828** | +0.0 | off | 0.6980 | −21.5 | PASS | 0.056 → 0.058 ms |
| xnli_en | 300 | 0.3467 | **0.5200** | +17.3 | 4 · fixed-1 · pair | 0.8600 | −34.0 | PASS | 0.088 → 0.090 ms |
| massive_intent_en | 300 | 0.7933 | **0.9267** | +13.3 | 1 · observed-laplace · bag | 0.7500 | **at/above** | PASS | 0.088 → 0.097 ms |
| banking77 | 500 | 0.6840 | **0.8700** | +18.6 | 1 · observed-laplace · bag | 0.4980 | **at/above** | PASS | 0.316 → 0.375 ms |

On these 8 dataset suites, modelless at or above laya goes from **2/8 → 3/8**:
emotion joins massive and banking77. ag_news closes from −44.0 to −7.5 and
xnli from −51.3 to −34.0.

**GOAT gates:**
- **G1** (readout calibration beats the conformal-naive floor): PASS on every suite.
- **G2/G4** (`cargo bench --features nb_scope --bench decision_set_goat`,
  new armed posture `32aee61`): p99 43 µs ≤ 1 ms, and `solve_into` does
  **0 allocations** after warmup with the tables armed (choice + noul
  polarity).
- **G3** (no regression where the lever is not armed): typed_decisions,
  sst5, prompt_injections and xnli-at-scale-0 are **byte-identical** to
  baseline (`hard`, `confusion`, and calibrated readout ECE compared
  field-for-field).

## Why the declines are real, not wiring bugs

- **sst5:** the best candidate read +2.5 pt on the selection slice (0.255 →
  0.280), under the +5 bar. Off by protocol. NBSVM / ridge are the next
  levers (Issue 038).
- **prompt_injections:** on its train-derived selection slice (100 cases,
  pool 346 docs) both polarities read ~0.50. This was **reproduced outside
  the harness** at the identical split (49 and 51 of 100), while a different
  train hold-out (rows 400–546) reads 0.64 for "yes → injection". So the
  slice genuinely does not reward the tables (the deepset set mixes sources,
  including German). The protocol declines it. Tuning the split until NB
  wins would be selection on test by another name.
- **typed_decisions:** the tables never arm. Its options are state-field
  values, not label corpora, so route terms (and the tables' legality guard)
  never activate. It needs an option-conditioned scorer, which is a separate
  lever in Issue 038.

## The transductive column (NOT the headline)

The owner's concern (2026-09-26): a hybrid rules/count engine needs "full
vocab and grammar". Measured, rather than argued: the fetched train rows
already carry **91–95% of the test tokens** (Issue 038 POC). The column
answers the rest directly. Test **text** (never labels) joins the count
tables, pseudo-labelled by the honest engine's own forced picks, with a
2-fold cross-fit so no case scores against its own text:

| suite | honest acc | transductive acc | Δ | pseudo docs |
|---|---|---|---|---|
| ag_news | 0.8750 | 0.8750 | +0.0 | 400 |
| emotion | 0.5950 | 0.5550 | −4.0 | 400 |
| xnli_en | 0.5200 | 0.4933 | −2.7 | 300 |
| massive_intent_en | 0.9267 | 0.9300 | +0.3 | 300 |
| banking77 | 0.8700 | 0.8640 | −0.6 | 500 |

**Verdict:** the train split already supplies the vocabulary. Adding the
test text self-labelled does nothing on the topical suites and **hurts**
where the honest engine is weakest (emotion, xnli): its own errors become
training signal, which is the self-confirmation failure mode of transductive
self-training. The column stays published beside the headline as the
standing answer; it is never folded into `acc`.

## Disclosures (read before quoting)

1. **The Issue 038 POC probes read the test split** (throwaway script,
   `.issues/038` §POC). Three design choices were informed by them:
   (a) count tables as the lever at all; (b) the observed-vocabulary
   Laplace α (noticed when a fixed α at 2^17 width collapsed emotion);
   (c) the 2^17 width. Mitigation: (b) is not hard-wired. The harness
   selects between it and fixed α = 1 on the selection slice, and xnli
   picked fixed-1. (a) and (c) are structural, fixed a priori here, and
   not tuned per suite. Treat the headline as honest-protocol numbers from
   a design that was not test-blind at inception.
2. The **pair view** (T3) was designed and measured on a **train-only**
   hold-out (xnli 3000 fit / 1000 score: bag .386 → pair .505) before any
   harness run.
3. The engine knob `nb_scale` stays **0 by default** (byte-identical serving
   posture). The gain is the arena protocol `--nb-select`, the Issue 030
   heads precedent.

## Reproduce

```bash
cargo build --release --features nb_scope --bin harness
S=ag_news,emotion,sst5,prompt_injections,xnli_en,massive_intent_en,banking77,typed_decisions
target/release/harness --skip-laya --head-select --suites $S --out /tmp/base
target/release/harness --skip-laya --head-select --nb-select --suites $S --out /tmp/nb
cargo bench --features nb_scope --bench decision_set_goat
```

Artifacts: `TABLES_base.md` / `results_base.json`, `TABLES_nb.md` /
`results_nb.json` (the nb run, including the per-candidate selection tables
and the transductive rows).
