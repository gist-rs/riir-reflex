# Bench 107 — bekko-system-one-v0-**400M** vs the reflex modelless lane; the family's 400M takes the board seat (Plan 617 A6)

**Status: MEASURED — the 400M wins 6/9 EN dataset suites vs the published modelless rows and beats the 68M on ALL 9; the bekko lane's board seat moves 68M → 400M. The Bench-103 recorded follow-up ("400M unmeasured — the natural follow-up if the 68M's per-suite wins justify it") is discharged.**

Date: 2026-10-02 · host m3 (M3 Max, CPU — the author's reference posture) · reflex `8ca8770` · box load 5.0→9.1 across the two runs (a sibling session active on the box)

**Plan home:** [riir-infer Plan 617 task A6](../../riir-infer/.plans/617_exl3_openthai_convert_and_infer.md) — the reflex-side, EXL3-independent board seat recorded there as "the natural 4090-box follow-up"; run here on the M3 under the plan's own "(or any box that meets the wall)" clause — the box met the wall (19 min total for 4,648 questions, see Latency).

## Comparator posture (the Bench-103 law, applied first)

Every modelless row below **byte-reproduces the published board** (`reflex-site data/bench.json`, the rows Bench 103's canonical run pinned) on all 9 suites — the posture-verification law before any external-oracle comparison. Two postures, exactly as published:

- the 7 original suites at `--nb-select --ridge-select` (the Bench-103 canonical flags);
- `typed_decisions` + `code_fixtures` at the Bench-103 addendum flags (`--nb-select --oc-select --ridge-select`; typed arms the oc tables at selected_scale 4.0).

| suite | n | published modelless (reproduced) | bekko-68M (published) | **bekko-400M** | winner |
|---|---|---|---|---|---|
| ag_news | 400 | 0.8825 ✓ | 0.9000 | **0.9150** | bekko +3.25 |
| emotion | 400 | 0.8850 ✓ | 0.4825 | 0.5825 | reflex +30.3 |
| sst5 | 600 | 0.3967 ✓ | 0.4050 | **0.4550** | bekko +5.8 |
| prompt_injections | 116 | 0.7672 ✓ | 0.5000 | 0.5086 | reflex +25.9 |
| xnli_en | 300 | 0.5233 ✓ | 0.6767 | **0.8600** | bekko +33.7 |
| massive_intent_en | 300 | 0.7800 ✓ | 0.8667 | **0.9100** | bekko +13.0 |
| banking77 | 500 | 0.8420 ✓ | 0.7380 | 0.7920 | reflex +5.0 |
| typed_decisions | 2000 q | 0.5725 ✓ | 0.4840 | **0.6235** | bekko +5.1 |
| code_fixtures | 32 q | 0.3750 ✓ | 0.40625 | **0.5938** | bekko +21.9 |
| **total** | 4648 q | **0.6461** (3003) | 0.5738 (2667) | **0.6721** (3124) | **bekko-400M +2.6 pt overall** |

- **The family sign FLIPS**: the 68M lost the overall row by −6.1 pt (Bench 103); the 400M **wins it by +2.6 pt**. Per-suite it beats the 68M on **all 9 suites** — the seat move is strictly an upgrade, no cell got worse.
- **vs the other board lanes (same 9 suites, board cells):** openthai 0.6205 (2884) — bekko-400M wins 5, loses 3 (emotion ~tie, prompt_injections, xnli), ties 1 (code_fixtures, 19/32 both); the Rethink hybrid row 0.6270 (2914) — bekko-400M wins 6/9. **The 400M is the strongest lane on the board's EN dataset suites overall** — read with the family-familiarity caveat below, never as a clean generalization claim.
- typed_decisions by question type (bekko-400M): choice 0.6467 · noul 0.6583 · score 0.5800 (modelless: 0.5483 / 0.7300 / 0.4725) — the 400M flips noul to modelless but takes choice + score decisively.
- Calibration side-note (the Bench-103 pattern): bekko's ECE(maxp) is strong on the suites it wins (0.035–0.19 everywhere except prompt_injections 0.31), where the modelless raw readout is not (0.13–0.82); the modelless **calibrated readout-ECE** column stays competitive (0.046–0.11).

## Determinism

- **code_fixtures bekko acc 0.5938 byte-identical across two independent processes** (the pre-run smoke in `/tmp` + run 2) — the smoke's `results.json` is committed as `determinism_witness_smoke_results.json` (the Bench-103 multi-process witness law, this record's witness).
- The harness det column ✓ for both lanes on every suite (the observed-repeat check).
- All 9 modelless rows byte-reproduce the published board — the comparator-posture witness.

## Box state + latency (the Issue-021 wall, working as designed)

Preflight at launch: `PROVENANCE: power=AC Power load=3.89 swap=2499.25M canary=118.2us/best5 powermode=2(high)` — PASSED.

Run 1 box state: start load 5.01 → end 9.05; run 2: 9.05 → 7.32 — **⛔ latency NOT QUOTABLE (a sibling session compiling on the box throughout)**. Latency is recorded here as order-of-magnitude context, never quoted: bekko-400M p50 134–818 ms per question subprocess round-trip (IPC included — the laya-python measurement law); the longest suites dominate (banking77 443 s, typed_decisions 358 s for 400 five-question cases via the shared-prefix path). **Total wall ≈ 19 min for all 4,648 questions** — the Bench-103 "~2 h CPU-infeasible" estimate did not survive contact with the actual M3 (that estimate extrapolated from the 17M's compute ratio; the real ratio at shared-prefix batched forwards is far kinder). The 4090-box alternative is thereby unnecessary for this cell — the box met the wall on the M3.

## The board seat (the publish)

The bekko lane's board cell moves **68M → 400M** (lane-scoped update, `PUBLISH_BENCH_LANES="bekko:acc-only"` — this run's own box state judged latency unquotable, so the cells publish accuracy-only; the Issue-021 wall as in Bench 103). The 68M cells live in Bench 103's record; the model column carries the truth. 26fddb9 is the 68M publish this one mirrors.

## Honest caveats (carried forward from Bench 103 + the card)

- **Family familiarity (author-flagged)**: bekko v0's training data shares dataset families with eval suites — board wins are home-field-inflated; never a broad-generalization claim. Symmetric risk per Bench 103: where they DON'T overlap the losses are out-of-domain; the board publishes raw cells either way.
- English-only model on English-only suites; the two thai suites stay openthai/encoder-only.
- Rendering laws are per-lane (JSON states for bekko — its native shape); a bekko prose rendering could shift numbers but the native-shape choice is the generous one.
- Score-level fallback (non-numeric rubrics → choice-over-labels) did not arise on these suites' numeric levels.
- License: MIT (verified 2026-10-02, Bench 103 addendum) — measurement + distill/teacher use license-clear with attribution.
- **This is a comparison-lane refresh only** — no serving/product posture changes; the lane stays harness-side, never in a release set. The EXL3 lane work (riir-infer Plan 617 Phase A/B) is unaffected: bekko remains an EXL3 structural NO (encoder class).

## Artifacts + repro

- `results.json` + `TABLES.md` — run 1 (the 7 canonical suites, canonical flags).
- `addendum_suites/results.json` + `TABLES.md` — run 2 (typed_decisions + code_fixtures, the oc-armed addendum flags).
- `determinism_witness_smoke_results.json` — the pre-run smoke (code_fixtures; the second process for the witness).
- `run.sh` — the exact two-invocation script.

```sh
# run 1 (canonical 7; canonical flags)
BEKKO_MODEL=hotchpotch/bekko-system-one-v0-400m \
BEKKO_REVISION=4aeb85b9d4042d75d8b8adf6ff7ba9e4629510ba \
BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python \
target/release/harness --bekko --skip-laya --nb-select --ridge-select \
  --suites emotion,ag_news,sst5,massive_intent_en,banking77,prompt_injections,xnli_en \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/107_bekko_v0_400m_board

# run 2 (typed + code_fixtures; the addendum oc-armed flags)
BEKKO_MODEL=hotchpotch/bekko-system-one-v0-400m \
BEKKO_REVISION=4aeb85b9d4042d75d8b8adf6ff7ba9e4629510ba \
BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python \
target/release/harness --bekko --skip-laya --nb-select --oc-select --ridge-select \
  --suites typed_decisions,code_fixtures \
  --datasets-dir .raw/datasets_t20k --out .benchmarks/107_bekko_v0_400m_board/addendum_suites
```

Revision `4aeb85b9d4042d75d8b8adf6ff7ba9e4629510ba` = the card's short pin `4aeb85b`, resolved via the HF API this session (the plan-A6 full-hash law). Model pre-downloaded (1.5 GB, `onnx_browser/` excluded — the lane never reads it).
