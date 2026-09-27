# Bench 070 — the cascade worthiness probe-LCB arm leg (issue 046)

**Status:** COMPLETE — verdict **PASS, exactly as preregistered.** All five
acceptance legs hold: ag_news armed via the LCB leg (+0.0675 recovered at
93.0% escalation), sst5 + massive stay modelless-EXACT, every other row
byte-identical to Bench 063@0.16, and every probe delta byte-identical (G5).
The live LCB values are the preregistered vectors to 4dp — the same numbers
pinned as unit tests in the landing commit (`a6bfec3`). Issue 046 closed
(removed; this doc + the HISTORY row are the record).

## What ran

The preregistered command (the exact Bench-063@0.16 lane + the new leg;
margin explicit, floor 0.05):

```sh
LAYA_DEVICE=metal CARGO_TARGET_DIR=/tmp/reflex_lcb cargo run --release \
  --features laya-riir-metal --bin harness -- \
  --datasets-dir .raw/datasets_t20k --cascade --cascade-worthiness \
  --cascade-worthiness-margin 0.16 --cascade-worthiness-lcb 0.05 \
  --nb-select --oc-select --ridge-select \
  --suites ag_news,emotion,sst5,typed_decisions,prompt_injections,xnli_en,massive_intent_en,banking77 \
  --out .benchmarks/070_cascade_probe_lcb
```

The rule under test (issue 046 lever 4, additive + default-off):
`armed = delta ≥ min_delta || probe_LCB95 ≥ floor`, where
`probe_LCB95 = Δ − 1.645·√(p₁(1−p₁)/n + p₂(1−p₂)/n)` — two-proportion,
CONSERVATIVE on paired data (its SE is ≥ the paired McNemar SE whenever the
two reads correlate). The preregistration was committed BEFORE the run:
`.issues/046` (rule, floor, predictions, kill criteria) + the three LCB
vectors pinned as unit tests, all in `a6bfec3`.

## PROVENANCE (box state)

`scripts/bench_preflight.sh` **PASSED at launch** — `power=AC Power
load=5.68 swap=1882.19M canary=129.4us/best5 powermode=2(high)`. The run's
own end-state banner then read `load 7.25 > 6 — a sibling job is on the box`
→ **⛔ latency NOT QUOTABLE** despite the passing preflight (mid-run load
from a sibling session; preflight is a launch-time check, the banner is the
run-time one — both disclosed here). Every accuracy quantity in this bench
is a PICK COUNT, load-immune by G5 determinism: all ten rows carry
`det ✓`, and the probe deltas match Bench 063 byte-for-byte, which is the
strongest possible load-immunity witness for this run.

## The gate (issue 046 acceptance vs the recorded 063@0.16 run)

| row | modelless | cascade | Δ | esc% | verdict | probe n | probe Δ | LCB | vs 063@0.16 |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions·english | 0.4655 | 0.7425 | +0.2770 | 96.7% | armed (margin) | 479 | +0.2881 | +0.2397 | row == 063 |
| typed_decisions·multilingual | 0.4655 | 0.4655 | 0.0000 | 0.0% | DISARMED | 479 | −0.1879 | −0.2392 | row == 063 |
| typed_decisions·typed | 0.4655 | 0.4655 | 0.0000 | 0.0% | DISARMED | 479 | −0.1378 | −0.1900 | row == 063 |
| **ag_news** | 0.8825 | **0.9500** | **+0.0675** | **93.0%** | **armed (LCB)** | 190 | +0.1316 | **+0.0854** | **VERDICT-CHANGED** (063 disarmed) |
| emotion | 0.8850 | 0.8850 | 0.0000 | 0.0% | DISARMED | 198 | −0.3131 | −0.3893 | row == 063 |
| sst5 | 0.3967 | 0.3967 | 0.0000 | 0.0% | DISARMED | 195 | +0.0718 | −0.0059 | row == 063 |
| prompt_injections | 0.7672 | 0.7672 | 0.0000 | 0.0% | DISARMED | 81 | −0.0494 | −0.1567 | row == 063 |
| xnli_en | 0.5233 | 0.6400 | +0.1167 | 32.3% | armed (margin) | 60 | +0.4000 | +0.2816 | row == 063 |
| massive_intent_en | 0.7800 | 0.7800 | 0.0000 | 0.0% | **DISARMED** | 60 | +0.1500 | +0.0064 | row == 063 |
| banking77 | 0.8420 | 0.8420 | 0.0000 | 0.0% | DISARMED | 60 | −0.3500 | −0.4888 | row == 063 |

Acceptance legs (issue 046):

1. **cascade ≥ modelless on all ten rows — PASS** (three armed rows all
   positive; seven disarmed rows exact).
2. **ag_news armed with a positive test delta — PASS**: +0.0675 exactly as
   predicted, cascade 0.9500 == laya's own 0.9500.
3. **sst5 + massive read modelless EXACTLY — PASS** (both flip-prone suites
   stay dead: margins fail AND LCBs are −0.0059 / +0.0064 < 0.05).
4. **Every other row byte-identical to 063@0.16 — PASS** (the comparison
   script checked accuracy + escalation rate + n_escalated + probe fields
   per row: 8/8 `row==063`, one `VERDICT-CHANGED` — the intended one).
5. **Probe deltas byte-identical to 063 — PASS** (`probe drifts: NONE`;
   G5 determinism end to end).

And the preregistration itself held to the digit: live LCBs ag_news
+0.0854 / massive +0.0064 / sst5 −0.0059 — the pinned unit-test vectors
(`lcb_vectors_match_the_preregistered_063_table`), reproduced by the real
lane.

## Verdict

1. **The recorded price is recovered, with the flip class still priced
   out.** The fused probe family's ag_news/massive tie was a SUPPORT
   problem, not a magnitude problem — massive (+0.1500, n 60) outranks
   ag_news (+0.1316, n 190) so no margin separates them, but 3.2× the probe
   support does. The LCB leg arms exactly the suite whose evidence is
   statistically firm and nothing else. The fused lane's armed total moves
   +0.394 → **+0.4612** with the same flip-safety.
2. **The escalation trade is disclosed, not hidden**: ag_news escalates
   93.0% — the row reads laya's accuracy at near-laya serving cost. The
   fused lane's contract is "cascade ≥ modelless everywhere + the rate as
   the published latency claim" (its shipped record already carries
   typed_decisions at 96.7%); the [15%, 60%] window remains the COMBINED
   posture's acceptance axis, untouched.
3. **Scope boundary (issue 047's rules honored):** this bench reads the
   standard lane slices for the ESCALATION question only — the same reads
   every cascade lane run (061/063/066) makes. No blend-mechanism or
   xnli-reopen question is decided from it; issue 047's R1 fresh-slice rule
   is untouched, and its A1–A4 ag_news modelless-mechanism work is a
   different surface (count-table features, not escalation arming).
4. **Posture:** the lever ships OPT-IN (`--cascade-worthiness-lcb`);
   library defaults unchanged (margin 0.16, LCB off); the combined lane's
   T4′ record and the recommended lane command are UNCHANGED. A second
   independent repro + the combined-posture LCB leg stay deferred (issue
   046 T5) under the 042 promotion-trigger discipline — one run is one
   run. If the 063-class probe family ever shows a THIRD arm-side flip,
   this bench is the recorded answer for what the floor family buys.
5. **The honest limitation stands as written in the issue:** the LCB leg
   is support-confidence, NOT a cal→test-shift guarantee — no cal-side
   gate can be (the T3 wording). What it prices is the measured flip
   class: thin probes flipping while firm probes hold.

## Artifacts

- `results.json` — meta pins `git_sha a6bfec3` (the lever + preregistration
  commit), every cascade row carries `worthiness.probe_lcb`.
- `TABLES.md` — the worthiness sub-tables gained the `probe LCB` column
  (issue-046 wording in the table intro).
- Comparison script (scratch, gitignored): `scripts/out/compare_070.py`.
