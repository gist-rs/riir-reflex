# Plan 006 — Issue 047 M1: the xnli validation-slice reopen lane

**Status:** IN PROGRESS — instrument implementation. The validation read
happens ONLY after every module gate below is green. This file is the
pre-registration: the posture, grids, seeds, promotion legs and void
criteria are FIXED before any validation number exists.

## Why

Bench 069 measured the G1-constrained NLI blend NEGATIVE and the verdict
review confirmed the search was underpowered and contaminated. Issue 047
filed the binding reopen rules (R1–R6) and four candidate mechanisms.
This plan pre-registers the M1 attempt under those rules.

## Primary posture (R2 — named before any read)

**M1 — pick/confidence separation with an agreement gate**, in the issue's
own words: keep the blend's pick; take confidence from a 3-feature logistic

```
m1_conf = σ(b + w1·logit(c_eng) + w2·margin + w3·agree)
```

- pick = the additive blend pick at λ*: `argmax_k(pe_k + λ*·ph_k)`.
- `c_eng` = the lane's CALIBRATED engine readout confidence (the shipped
  `SigmoidGateCalibrator` surface) — of the ENGINE's pick, literally as
  the issue names it.
- `margin` = the LDA head margin δ_top1 − δ_top2.
- `agree` ∈ {0, 1} = engine pick == head pick (the review's strongest
  correctness signal, 56/53 decorrelation).
- 4 parameters (3 weights + intercept); intercept unpenalized; L2 ridge
  1e-3 on the weights; IRLS/Newton f64, max 100 iters, ‖Δβ‖∞ < 1e-10.

Fit-data note (disclosed reading of R2): M4 ("more cal data, cross-fit")
is a FIT-SCALE choice, not a competing readout surface — it composes with
M1 and changes no decision rule. Running "M1 on 200" AND "M1 on 20k" as
separate arms would be the second look R2 forbids. ONE arm: M1 on the
grown pool below. M2/M3 run as secondaries in the same single read and
are REPORTED WITHOUT promotion (R2's first alternative).

## Confirmation surface (R1)

`xnli_en` VALIDATION split (~2490 rows), fetched once into
`.raw/datasets_t20k/xnli_en/validation-*.json`. Read ONCE, pre-registered.
The 300-item test split is never loaded by this lane: the new suite
`xnli_en_val` reads the `validation` split from disk (`eval_split` field),
is `named_only` (never a default-run member), and `--nli-m1` refuses
loudly on any suite other than `xnli_en_val`.

A0 (the engine's forced accuracy on validation) is measured in the same
single pass — it is part of the read, not a second one. Adjudication
happens ONCE, from the first completed run's record. Determinism makes a
re-run byte-identical; it does not make a second adjudication.

## Instrument (R3–R6)

- **R3 cross-fit** — `NliLda` is fit K=10-fold over the pool; every
  pool item's head pick/margin/posterior is OUT-OF-FOLD. The DEPLOYED
  head is the full-pool fit (standard cross-fit → full-fit deployment).
  The M1 logistic and the piecewise-Platt secondary are fit on the OOF
  pairs only (honest labels); at validation they read full-head features
  (the margin-distribution shift oof→full is a disclosed caveat; the
  bootstrap + n≈2490 read is the instrument that catches it).
- **R4 bootstrap** — paired bootstrap, B = 2000, seed 20260928
  (SplitMix64), resampling item indices once per replicate and applying
  the SAME indices to both arms of each comparison:
  - accuracy delta (blend − A0): report mean + 95% CI (2.5/97.5 pct);
  - ECE delta (m1 − engine floor): report CI (disclosure; the gate legs
    decide on point estimates — binned-ECE bootstrap intervals are
    discretized-statistic intervals, not decision instruments).
- **R5 per-item log** — every validation item logs
  `(idx, gold, eng_pick, eng_conf_cal, head_pick, margin, agree,
  blend_pick, blend_conf_raw, m1_conf, platt2_conf, m2_pick, m3_pick,
  eng_ok, head_ok, blend_ok, m2_ok, m3_ok)` into
  `results.json → suites[].modelless.nli_m1.items`, so McNemar, the
  win/loss split and the bootstrap are recomputable after the run.
- **R6 sharpness beside ECE** — AUROC (confidence vs correctness,
  Mann-Whitney) reported for m1_conf, the raw blend surface, the
  piecewise-Platt surface AND the engine's calibrated readout (the
  069 hole-5 disclosure); Brier + resolution (binned, same 10-bin edges
  as `ece_of`) reported for m1_conf.
- **McNemar** — b/c counts + two-sided exact binomial (log-space), blend
  vs engine, reported beside the accuracy leg.

## Pools (all pool-side; no validation item participates in any fit)

- **Fit pool** = cal front (200, corpus-free by construction) + every
  train-rest row EXCEPT (a) the corpus-cap docs — the first
  `effective_cap` valid docs per label in rest order, the exact
  `specs_from_pool` law — and (b) any row whose premise string equals a
  corpus-cap doc's premise (the MultiNLI premise-sharing guard: shared
  premises inflate corpus scores on pool items that validation never
  shows). Pool ≈ 19.5k rows from the t20k pull.
- Disclosed residual bias: pool rows remain inside the NB count tables
  (`nb_doc_sets` takes all rest docs); at ~6.6k docs per label each
  row's own-token mass is ~1/6600 of its table — measured-negligible,
  disclosed rather than engineered away.
- Engine reads on pool items: RAW probs (blend pe) and fitted-engine
  calibrated confs (x1) from the DEPLOYED engines — one extra eval pass,
  no fold engines (the fold machinery would buy nothing: the fold only
  needs to own the HEAD, which never sees the engine).

## Selection (all pool-side, OOF)

- λ grid = {0, 0.125, 0.25, 0.5, 1, 2, 4, 8} (069's ladder). λ* = max OOF
  blend accuracy; ties → smaller λ; no-regression leg: acc(λ*) ≥ acc(0).
- M2 secondary: score_k = ln(pe_k) + λ2·(δh_k − mean(δh)); same grid,
  same tie law (the mean-centering is the 3-class generalization of the
  issue's σ(δ_e + λ·δ_h)).
- M3 secondary: switch to the head pick iff `c_eng < τ AND margin > m`;
  τ ∈ {0.5, 0.6, 0.7, 0.8, 0.9} × m ∈ {0.25, 0.5, 1, 2, 4, 8}; max OOF
  accuracy; ties → larger τ then larger m (the conservative side).
- Piecewise-Platt secondary (`CalibratedGateSet<2>`, the issue's
  substrate note): direction 0 = agree, 1 = disagree; input = the
  calibrated readout confidence; fit on OOF pairs, `refit_all`, the
  lane's calibrator window config. Reported, never promoted.

## Pool-side sanity gates (pre-registered; any failure marks the run VOID)

- S1: OOF pairs ≥ 1000 and the logistic moved (‖β_weights‖₂ > 1e-6).
- S2: OOF logistic NLL < base-rate NLL − 0.02 (the gate learned).
- S3: OOF blend acc(λ*) ≥ OOF A0 acc (selection consistency).
- S4: every fold holds ≥ 3 items of every class (no degenerate fold head).
- S5: deployed-head pool accuracy ≥ OOF head accuracy (sanity direction;
  a full-fit head cannot be worse than its OOF estimate beyond noise —
  flag at −2 pts).

A VOID run is recorded with its defect; whether any aggregate survives is
adjudicated by the Bench-069 rule (which numbers the defect provably does
not touch) — the default is that the slice is spent. This is the guard
the void run 1 paid for.

## Promotion criterion (fixed before the read)

The posture is PROMOTABLE-CANDIDATE iff ALL of:

1. `m1_ece ≤ floor_ece` — the engine-derived conformal floor (fit on the
   cal-front readout pairs, read at validation confidences — the lane's
   standing G1 construction, 069-continuity; the blend-surface-derived
   floor is reported beside it as the review's hole-4 disclosure);
2. `m1_ece < blend_raw_ece` — beats its own uncalibrated surface;
3. `blend_acc ≥ a0_acc + 0.05` — the house 5 pt arming bar;
4. paired-bootstrap 95% CI of (blend_acc − a0_acc) has lower bound > 0.

McNemar, AUROC and resolution are disclosure legs, not gates. A
candidate pass does NOT silently change the shipped lane — promotion
wiring is its own follow-up review against the bench record (the 069
law). Anything else: issue 047 closes measured, xnli stays report-only.

## Implementation surface

- `fetch_datasets.sh`: xnli_en `validation` fetch block (cap=all).
- `runner.rs`: `SuiteSpec.eval_split` + `named_only`; `xnli_en_val`
  registry entry; `prepare`/`load_suite_envelope` read the named split;
  `train_docs` learns the name; `nli_m1` module + wiring (flag
  `--nli-m1`, `RunOptions`/`ModellessInput`/`LaneResult` fields, seat +
  e0 constructors pass `false`).
- `nli_m1.rs`: the pass, pools, cross-fit, logistic, bootstrap, McNemar,
  AUROC, per-item log, sanity gates + module tests (logistic
  known-answers, fold balance, bootstrap determinism + toy CI, McNemar
  known answer, AUROC known answer, pool filter vs corpus law, an
  end-to-end synthetic-slice run).
- `metrics.rs`: `auroc_of`, `brier_resolution_of` (+ known-answer tests).

## Run (after every gate above is green, and not before)

```
cargo run --release --bin harness -- --skip-laya \
  --head-select --nb-select --ridge-select \
  --nli-m1 --suites xnli_en_val \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/073_nli_m1_validation
```

Modelless only (G5 not implicated; no latency claim → no preflight owed;
box state recorded per the AGENTS.md law). Record: `.benchmarks/073_*`.

## Numbering

`.plans/.highwater` read 3 with disk at 005 — the stale-highwater class
047's own renumber note records. Allocated 006 (max of disk), highwater
written back 6 in the same commit. Bench = 073 (`.benchmarks/.highwater`
= 072).
