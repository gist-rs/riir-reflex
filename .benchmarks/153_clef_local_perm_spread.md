# Bench 153 — clef-local on the perm-probe: option-ordering spread (plan 619 T3.3)

**Status:** COMPLETE 2026-10-17 — verdict PASS, 0 red cells. The first Clef artifact ever measured on the option-permutation axis.

- **Lane:** `clef-local` — the seated in-process ANE arm (clef-text-0.6b @ HF `3c61a609…`, compute units `CpuAndNe`, feature `clef-ane`; the bench-152 board lane, plan 619 T3.1). Wired into the perm-probe by `37115e9` (`--clef-local`; the HTTP arm deliberately NOT wired — an uncapped probe is hosted spend; the flag without the feature refuses loud).
- **Run:** `CLEF_LOCAL=1 ./target/release/harness --perm-probe --clef-local --out .benchmarks/153_clef_perm_spread` (release, sha `37115e9`, host m3, 2026-10-16T22:02:27Z; raw record `153_clef_perm_spread/perm_probe.json` + `PERM_PROBE.md`). Two independent runs (pre- and post-wiring-commit) produced IDENTICAL lane cells — the probe is seeded (`PROBE_SEED 125239297`) and the lane is deterministic.
- **Population:** the perm-probe's 6 default choice suites (typed_decisions 60 slots / ag_news 40 / emotion 40 / xnli_en 38 / massive_intent_en 38 / banking77 39 = **255 slots** × K=5 orderings = 1,275 clef forwards, plus the canary per suite). **Zero skips, zero absences** — every sampled case's every choice question probed.

## Verdict

| suite | clef-local median | max | flips | verdict | modelless control (median / max / flips) |
|---|---|---|---|---|---|
| typed_decisions | 0.00 pt | 0.00 pt | 0/60 | PASS | 0.00 / 3.0e-06 pt / 31 (ties-only) |
| ag_news | 0.00 pt | 0.00 pt | 0/40 | PASS | 0.00 / 6.0e-06 pt / 0 |
| emotion | 0.00 pt | 0.00 pt | 0/40 | PASS | 1.5e-06 / 3.0e-06 pt / 0 |
| xnli_en | 0.00 pt | 0.00 pt | 0/38 | PASS | 0.00 / 6.0e-06 pt / 0 |
| massive_intent_en | 0.00 pt | 0.00 pt | 0/38 | PASS | 7.5e-07 / 2.2e-06 pt / 0 |
| banking77 | 0.00 pt | 0.00 pt | 0/39 | PASS | 3.7e-07 / 9.3e-07 pt / 0 |

**Gate (Bench 128 law): median ≤ 2 pt — HELD on every suite, with the strongest possible reading: median AND max are EXACTLY 0.0 and flips are 0 everywhere.** Every one of the 1,275 ordering answers returned bit-identical probabilities in label space. RED cells: 0.

**Why exactly 0 is the honest expected result (the construction note, not a vacuity pass):** clef canonicalizes the option set BEFORE scoring — the lane builds each question's criteria as a `BTreeMap` (sorted keys) and the head scores in that sorted-id order; the reflex lane re-indexes the answer into presented order through the returned `option_ids` (the `map_answers` law). Presentation order therefore provably cannot reach the forward: every permutation of the same question produces the IDENTICAL record. The probe measures what ships — and what ships is order-invariant by construction. This is a property of the artifact (a real, measured fact worth recording: an ordering-robustness class the Drex/d1 comparison lanes do NOT share), disclosed here so nobody re-runs the axis expecting variance.

**The control holds its own stricter contract** (median ≤ 0.01 pt fp envelope, flips ONLY on exact top-2 ties): max swing ~1e-6 pt everywhere (the L1-ULP class — the disclosed `control byte-identity` column reads false by design, the normalizer's sum reorders ULPs with presentation); flips = 31/40 on typed_decisions only, ALL exact-tie flips (`untied flips: 0` on every suite). **Liveness proven in-run:** the control flipped the ag_news canary (alpha→delta→gamma across orderings — the tie-break class); the run refuses on a control red and did not.

## Box state (disclosure — this is a distribution-stability probe, no latency claims)

Harness-captured: AC power, `power_mode=high`, load 8.04 at start / 5.76 at end (sibling sessions; `latency_quotable=false` at start — irrelevant here, the probe has no latency columns by design, but recorded per the box-state law). Artifacts warm (`~/.cache/riir-infer/clef-text-0.6b-coreml`, all 4 buckets, ~6 s load).

## Relation to the board (bench 152)

Bench 152 seated the lane on accuracy; this bench closes the axis bench 152 did not measure: the lane's answers cannot be manipulated by option ordering — relevant wherever a consumer composes option lists dynamically (a presentation-order attack surface the modelless control already holds and the hosted comparison lanes do not).
