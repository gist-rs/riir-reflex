# Bench 151 — the Issue-092 2-feature readout-gate candidate (T1+T2): MEASURED NEGATIVE

**Status:** RECORD — the candidate leg LANDED (T1, report-only, 5 unit tests) and the
bench ran at the exact 147 posture (T2). **Verdict: NEGATIVE — the 2-feature logistic
gate does NOT dominate the conformal floor; the lane CLOSES (T3 not fired, the 047
acceptance stands).**

**Provenance:** host shikuwa (4090 box), 2026-10-16T12:4x +07 · git `f767c64`+ (the
candidate leg is THIS commit) · release build · datasets `.raw/datasets` · CPU-only (the
modelless decide path; sibling LOWO finetune holds the GPU — no latency claims) ·
artifacts `.benchmarks/151_readout_floor_candidate/` (`null_probe.json` +
`NULL_PROBE.md`). Exit 2 = the designed control-red refusal (3 standing floor reds —
the record is the point, never a quiet pass).

## Reproducibility (the instrument's own gate)

The floor cells are **BIT-IDENTICAL to the 147 n-firmed addendum** on all six suites
(e.g. ag 0.5588/0.5846, emotion 0.4745/0.5208, xnli 0.4767/0.4791 tie-zone, banking
0.6322/0.6321) and the A2 cross-regime counts match (banking 130, massive 117) — the
FloorObs/ArmReadout refactor preserved the floor leg exactly; the candidate leg is
additive. The standing 10× picture is therefore RED 3/6 (ag_news −0.026, emotion
−0.046, xnli tie-zone Δ 0.0024), not the first read's 4/6.

## The candidate (Issue 092 T1)

`conf₂ = σ(a·maxp + b·(1−H/lnK) + c)` — both features through the SHIPPED readout
functionals (`crate::readout::confidence_with`), fit on the same cal slice by the
`label_heads` sequential-SGD law at D=2 (deterministic; unit-pinned), evaluated on the
same test slots, judged by the floor's own `dominates` rule (weak both axes + strict
one). REPORT-ONLY — never gates, never arms.

| suite | floor AUROC/ECE | shipped gate | candidate | cand verdict |
|---|---|---|---|---|
| typed | 0.5716 / 0.4350 | 0.5726 / 0.1679 PASS | σ(−2.013·maxp +0.016·ent −1.271) → 0.4296 / 0.0066 | **RED — regresses a green suite** |
| ag_news | 0.5846 / 0.2047 | 0.5588 / 0.0813 RED | σ(+3.826·maxp +0.269·ent −1.050) → 0.5845 / 0.0797 | **RED by 1e-4** (tie-zone) |
| emotion | 0.5208 / 0.2980 | 0.4745 / 0.1297 RED | σ(−0.191·maxp −0.008·ent −1.742) → 0.4791 / 0.1300 | **RED** |
| xnli | 0.4791 / 0.3201 | 0.4767 / 0.0335 RED (tie-zone) | σ(+0.875·maxp +0.083·ent −1.055) → 0.4801 / 0.0196 | **PASS** |
| massive | 0.7153 / 0.1249 | 0.7156 / 0.0614 PASS | σ(+8.444·maxp +2.485·ent −0.809) → 0.7117 / 0.1729 | **RED — regresses a green suite** |
| banking77 | 0.6321 / 0.1942 | 0.6322 / 0.0278 PASS | σ(+1.116·maxp +0.467·ent −0.445) → 0.6171 / 0.0010 | **RED — regresses a green suite** |

## The verdict and its mechanism

**NEGATIVE — 1/6 pass (xnli, the tie-zone-noise suite), 1/6 within 1e-4 (ag_news), 4/6
clear fails including REGRESSIONS on all three green suites.** The lane closes by its
own pre-registered rule (issue 092 T3: "If T2 does not dominate: record the negative
and CLOSE — the 047 acceptance stands").

The mechanism is the Bench-052 lesson repeating one level up, now measured on the FIT
itself rather than the selection:

1. **The cal slice is not a ranking-training signal.** typed's fit went NEGATIVE on
   maxp (−2.013: the cal slice says lower-maxp → more-correct) and the test read flips
   hard (AUROC 0.4296 vs the floor's 0.5716). Fitting joint (a,b,c) on ~150–200 cal
   rows minimizes NLL, and NLL is not AUROC — the optimizer happily buys ECE (typed
   cand ECE 0.0066!) with ranking destruction.
2. **Where the cal signal is real, the candidate only TIES the floor.** ag_news's fit
   recovered the maxp direction (+3.826) and landed at AUROC 0.5845 vs the floor's
   0.5846 — a 1e-4 tie-zone loss under the strict lose-no-axis rule, with a huge ECE
   win (0.0797 vs 0.2047). This is the same tie-zone shape the 147 addendum flags on
   xnli ("a noise band on the AUROC axis would be an instrument refinement") — it is
   NOT dominance, and per-suite reading of tie-zone deltas was declined there already.
3. **Where the ranking gap is largest (emotion −0.046), the fit finds NOTHING**
   (weights ≈ 0: −0.191, −0.008) — the cal slice's maxp→correct signal on emotion is
   too weak/conflicted to train even a 3-parameter model toward the direction the test
   read shows the raw maxp carries. More fitting power (epochs, standardization) would
   sharpen the typed-class overfit, not find emotion's signal — the 052 arithmetic
   unchanged.

## Disposition

- **Issue 092 CLOSED negative** (T1 landed as instrumentation, T2 measured, T3 not
  fired). The 047 acceptance and the Bench-052 demotion STAND; the adjudicated posture
  is undisturbed.
- The candidate leg STAYS in the null-probe instrument (report-only) — any future
  readout evidence base now gets the 2-feature reading for free, and the next
  re-open needs a mechanism whose CAL-SLICE FIT TRANSFERS TO TEST RANKING, which
  this record prices as the binding constraint (not ECE, not the functional family).
- The honest open residue is exactly the 147 addendum's own: ag_news −0.026 /
  emotion −0.046 ranking gaps vs raw max-prob, and the instrument refinement
  candidate (an AUROC noise band for tie-zone deltas) — both recorded there, neither
  actionable modellessly today.

Refs: Issue 092 · Bench 147 + its 10× addendum · Bench 052 (the demotion) · Issue 047
(the standing acceptance) · Plan 340 (the floor shape) · `src/readout.rs` (the
Bench-817 dispatch law, untouched).
