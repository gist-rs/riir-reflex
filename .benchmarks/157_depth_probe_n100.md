# Bench 157 — Issue 094 T2: residual-stability depth probe (the n=100 read)

**Status:** COMPLETE — report-only; the depth-confidence candidate reads WEAK-TO-ABSENT on the English checkpoint (no gate passes; nothing armed)

**Date:** 2026-10-23 · **Box:** m3 (M3 Max) · **Posture:** CPU laya lane, band cos ≥ 0.995 / l2 ≤ 25 (the measured scale — see the calibration note), trough window 2, ≤100 cases/suite · 1209 question rows, 0 skips · run: `cargo run --release --features twt_profile --bin harness -- --depth-probe --depth-max-cases 100 --out .benchmarks/depth_probe_n100`

## What was measured

Per (case, question): one `system_one` forward for the serving pick (correctness = the serving path's own read), one `encoder_forward_capture` (riir-infer `b4b9205`'s consumer seam) streaming the residual stream through the substrate's `StabilityTracker` (Issue 1106) — the LAST-POSITION row per stage. Readouts:

- **suffix depth** (the substrate's law: everything after the depth stays sub-band) — **structurally None on this checkpoint**: the residual stream RE-ACCELERATES in the tail (per-case l2 series: mid-depth trough ≈ 4–20 at transitions 3–14, final layers ≈ 90–130; all 28 final transitions break every sane band). This is itself the measured finding — the English encoder's residual stream has a stability TROUGH, not a suffix.
- **windowed-trough depth** (the consumer-side candidate): the earliest stage whose next `window` (2) transitions are both sub-band. The paper's "decision depth" analog for a trough-shaped stream.

## The board

| suite | q | acc | trough p10/p50/p90 | none | AUROC(trough→correct) | ECE(trough) | modal floor |
|---|---|---|---|---|---|---|---|
| typed_decisions | 500 | 0.3260 | 3/3/4 | 321 | 0.5393 | 0.4888 | 0.5175 |
| ag_news | 100 | 0.9600 | 3/3/3 | 0 | 0.6510 | 0.0600 | 0.3275 |
| emotion | 100 | 0.5400 | 2/2/3 | 0 | 0.5370 | 0.2000 | 0.3275 |
| sst5 | 100 | 0.3200 | 2/2/3 | 0 | 0.4026 | 0.4200 | 0.2750 |
| prompt_injections | 58 | 0.7241 | 2/3/3 | 0 | 0.3899 | 0.3103 | 0.4741 |
| xnli_en | 100 | 0.8700 | 2/2/3 | 0 | 0.5225 | 0.1667 | 0.0000 |
| massive_intent_en | 100 | 0.7200 | 2/2/2 | 0 | 0.5288 | 0.1000 | 0.0467 |
| banking77 | 100 | 0.4500 | 4/4/10 | 79 | 0.5754 | 0.2524 | 0.0420 |
| semantic_defects | 51 | 0.3137 | 3/3/3 | 0 | 0.4464 | 0.6863 | 0.0000 |

## Verdict — the G1 discipline fires

**The depth-confidence candidate does not beat its floors.** G1 requires every confidence candidate to beat BOTH (a) the conformal floor and (b) the T1 input-keyed modal floor, or be recorded NEGATIVE with the miss named:

- **AUROC ≈ chance or inverted** on 6 of 9 suites (0.39–0.58); the single >0.6 read (ag_news 0.6510) sits on a suite whose accuracy is already 0.96 — no headroom to monetize.
- **ECE(trough) is nowhere near accuracy** on the hard suites (typed 0.489 vs acc 0.326; semantic_defects 0.686 vs 0.314; sst5 0.420 vs 0.320) — a depth-derived confidence would be MISCALIBRATED exactly where confidence matters.
- **The modal floor beats the depth signal on typed_decisions** (floor 0.5175 vs acc 0.326 — the modal lookup is a HIGHER accuracy than the lane itself, so depth-as-confidence is dominated before calibration even starts).

**Recorded NEGATIVE for confidence use on this checkpoint; the lane ships report-only, arms nothing, changes no serving path.** The trough DISTRIBUTION itself is a usable characterization artifact (per-suite settle depth: ag_news 3, massive 2, banking77 4-or-never at 79%) — that part is a keeper as a checkpoint fingerprint.

## Calibration notes (the consumer-band story)

1. The first live run pinned the band at the substrate example's scale (cos 0.9999 / l2 1e-3) → structurally-None everywhere: raw hidden-state l2 runs 4–130. Re-pinned to the MEASURED scale (0.995 / 25.0), disclosed in the option docs.
2. The suffix law cannot pass on a trough-shaped stream BY CONSTRUCTION (the final transition is the largest, ≈ 100% of scale) — the windowed trough is the honest adaptation; the substrate's own contract is untouched.
3. banking77 (79/100 no-trough): its sub-band transitions are ISOLATED (single transitions at 2 and 4, never consecutive) — a genuinely different stream shape, not a bug.

## Instrument defects caught by the first runs (kept for the record)

- The AUROC tie-group accumulator credited each MIXED tie group's full size to the correctness of its FIRST row — inflating AUROC to 1.31 (>1.0, impossible). Fixed to per-row credit; all-tie AUROC reads exactly 0.5.
- The capture sink learned the sequence length from the Embedding stage's own sweep — which fires on every row (the count grows with the sweep), mixing all positions into the tracker. Fixed to the substrate smoke's pattern: tokenize first, filter rows to the true last index. Double-tokenize is calibration-cost.
