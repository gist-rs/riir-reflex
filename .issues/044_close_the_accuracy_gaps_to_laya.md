# Issue 044 — close Reflex's accuracy gaps to laya on the five suites it trails

**Status:** EXECUTED 2026-09-27 (Bench 068) — every task measured; verdicts
below. code_fixtures resolved-by-pin (the gap was a population artifact:
reflex 0.3750 vs laya 0.4062 on the frozen population, inside the 95% CI);
routing + sensitivity resolved as NON-gaps (laya inside reflex's Wilson CI —
3 questions each); ag_news stands at −6.8 pt with no shipped lever moving it
(measured twice: 064/065 + this bench); xnli stands at −28.2 pt with a REAL
+5.67 pt modelless pair-feature harvest found — refused promotion on G1
(blend readout ECE 0.1596 > conformal floor 0.1351), reopen path recorded.
Kept open for the two real gaps' follow-ups (the next mechanisms), not for
any unanswered question this issue asked.

## Why

Reflex already wins latency (sub-ms vs laya's 100+ ms p50) and bundle size
on every suite. Where it trails laya on accuracy, that is a gap to close,
not a reason to pick laya. reflex-site now states it that way (reflex-site
`e7f4638`, arena TL;DR "Reflex vs laya, accuracy": "gap to win the
other 5").

## Targets (published data/bench.json, english checkpoint, 2026-09-27)

| suite | Reflex | laya (rust) | gap to win | T2 verdict (95% Wilson) |
|---|---|---|---|---|
| code_fixtures | 0.2500 | 0.6667 | +41.7 pt | significant — **but the population was the defect** (T4): on the frozen population 0.3750 vs 0.4062 @ n=32, NOT significant |
| xnli_en | 0.5233 | 0.8600 | +33.7 pt | significant — real gap; +5.67 pt harvestable via the T3 head (G1-refused as published posture) |
| harness_sensitivity | 0.4000 | 0.6000 | +20.0 pt | NOT significant (3 questions; laya in CI [0.198, 0.643]) |
| harness_routing | 0.4375 | 0.6250 | +18.8 pt | NOT significant (3 questions; laya in CI [0.231, 0.668]) |
| ag_news | 0.8825 | 0.9500 | +6.8 pt | significant — real gap; no shipped lever moves it (T1) |

## Tasks

- [x] T1 — ag_news: **no shipped lever moves it.** The published row IS the
  lever surface (head/nb/oc/ridge selected in it). Volume: Bench 065
  measured NEGATIVE (+0.25 pt at 120k rows; cap axis exactly nothing).
  Joint blend-genome: HELD twice (Bench 064 round 1 at cap 64; Bench 065
  round 3 at cap 1000). This bench re-confirmed the posture byte-exactly
  (0.8825). The residual is encoder-owned word-order (065's fallback) —
  closing it needs a NEW modelless mechanism, and the T3 head is the
  demonstration that new mechanisms gate on G1.
- [x] T2 — the Wilson screen: routing and sensitivity's gaps are INSIDE
  reflex's 95% CI (3 questions each) — no accuracy work owed at this n; the
  protocol lever there is MORE AUTHORED QUESTIONS (families.rs is
  synthetic), not a mechanism. code_fixtures / xnli / ag_news are
  significant.
- [x] T3 — xnli_en: the pair-feature head is MEASURED and the verdict is
  two-sided (Bench 068): lexical pair features carry real decorrelated
  signal — head alone 0.5333 (> engine 0.5233), additive blend at
  cal-selected λ=0.5 **0.5800 (+5.67 pt)**, oracle 56/53 wins/losses — but
  the blended readout FAILS G1 (ECE 0.1596 > floor 0.1351), so the posture
  stays report-only. NOT out of reach — but not promotable as measured.
  Reopen: pre-registered cal-side G1-constrained λ, or a calibrated blend
  readout; either lands as its own bench.
- [x] T4 — code_fixtures: population PINNED (`fe7c123`): the
  BLAKE3-pinned committed fixture `src/harness/code_fixtures_frozen.json`
  (digest `034774df…b44362`, 8 healthy modules, full slices — the old live
  harvest had rotted to 4 of 8 labels without corpus docs and moved 0.2500
  → 0.2917 with the tree); builders read the frozen bytes; regeneration is
  `cargo run --features modelless --example gen_code_frozen` (deliberate,
  never a build side effect). The +41.7 pt gap was mostly the artifact:
  frozen-population read is reflex 0.3750 vs laya 0.4062 @ n=32 — inside
  the CI.
- [x] T5 — republish: code_fixtures republished lane-scoped (both lanes,
  one run, one cases_digest, box_state quotable both ends — the Issue-021
  wall satisfied, nothing waived). No other suite's published row changed
  (postures byte-identical; the xnli/ag_news gaps stand). reflex-site
  commit: see the lane_sources rows for code_fixtures.

Follow-ups (new work, not this issue's questions): ag_news new-mechanism
hunting; the xnli G1-constrained λ promotion bench.
