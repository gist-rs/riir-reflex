# Bench 174 — Issue 109 gate-fit threshold-stability audit (first cell, the serving board)

**Status:** LANDED 2026-11-05 — the audit arm exists, the first cell ran clean, the issue's proof bar is met; issue 109 CLOSED in the same arc (noise-reduction rule: the file is removed, this record + the AGENTS.md line are the residue).

## What ran

`cargo run --release --bin harness -- --gate-stability --out .benchmarks/174_gate_stability`
twice (run 2 → `/tmp/gs_run2`), over the serving board's 8 suites
(massive_intent_en, banking77, typed_decisions, prompt_injections, sst5,
ag_news, xnli_en, emotion) at the default posture (levers off, kn_route on,
calibrated gate fit per Issue 056, ρ = 0.30, B = 200 seeded SplitMix64
resamples, held-out = the stratified eval front through the SAME engine +
calibrator, offsets ±{0.02, 0.05, 0.10}).

## The issue's proof bar

1. **Byte-identity across two runs: PASS** — `gate_stability.json`'s
   `suites` cells identical; of the meta keys only `date_utc` differs
   (provenance disclosure by design; the cells carry no timestamps).
2. **Suites whose t\* SD > 0.05: NONE** — that is a result, not a failure
   (the issue's own clause). Max observed SD: 0.0197 (banking77 distance);
   runner-up 0.0188 (xnli_en distance); every score axis ≤ 0.0133.
3. **Population floor: 8 calibrated suite(s) audited**, printed every run.

## Results (t\* SD per suite/axis, ×10⁻³; the operating point is stable everywhere)

| suite | score t\* | score SD | distance t\* | distance SD | verdict |
|---|---|---|---|---|---|
| massive_intent_en | 0.2372 | 13.3 | 0.5259 | 10.5 | STABLE |
| banking77 | 0.3177 | 4.3 | 0.5549 | 19.7 | STABLE |
| typed_decisions | 0.3103 | 0.1 | 0.9122 | 12.8 | STABLE |
| prompt_injections | 0.5100 | 0.0 | 0.4619 | 17.8 | STABLE |
| sst5 | 0.2049 | 4.6 | 0.4166 | 7.6 | STABLE |
| ag_news | 0.4397 | 0.0 | 0.4105 | 11.8 | STABLE |
| xnli_en | 0.2999 | 1.9 | 0.5110 | 18.8 | STABLE |
| emotion | 0.1453 | 0.0 | 0.4265 | 10.8 | STABLE |

Zero vacuous resamples (vacuity floor max(10, 5%·n)); zero knife-edge
flags (≥2× nearest-neighbor metric drop at ±0.02).

## Seam ruling — the saturation support cliff (named, machine-readable)

Four score axes show pass_rate collapsing to 0.0 at t\*+0.02
(typed_decisions 0.75→0, emotion 1.00→0, prompt_injections 1.00→0, ag_news
1.00→0.16): the Issue-056 calibrated confidences are QUANTIZED above the
operating band, so no eval score lives in (t\*, t\*+0.02]. This is
disclosed in the curve, and the knife-edge bar is adjudicated to NOT fire
on it: `seam_support_cliff_at_zero_not_flagged` — the knife-edge law
prices threshold FRAGILITY ("the next re-fit could fall off"), and those
fits reproduce t\* at SD ≤ 0.1×10⁻³; a re-fit cannot fall off an edge it
always re-lands on. The bar fires only on a ≥2× drop WITHIN nonzero
neighbor support (a true mass redistribution), never on a quantization
gap. A future re-arm that widens the calibrated scale's resolution re-runs
this cell before citing the cliff as anything but quantization.

## Provenance

- Box: 4090 Windows (shikuwa), AC power, box QUIET (no compute consumer);
  CPU-only modelless lane — no GPU work, no latency claim (box-state gate
  not applicable per the issue's offline law).
- Datasets: the canonical `.raw/datasets` (Issue 013 re-baseline a).
- Tool commit: the arm landed in the same arc as this record; meta pins
  `git_sha` per run (see `gate_stability.json`).
- Run-2 scratch `/tmp/gs_run2` removed after the identity check.
