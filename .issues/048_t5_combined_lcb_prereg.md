# Issue 048 — T5 closure preregistration: the combined-posture LCB leg + the Bench-070 second repro

**Status:** OPEN — preregistration committed BEFORE any run (the issue-046
pattern). This issue closes the two deferred legs recorded in
`.benchmarks/070_cascade_probe_lcb.md` §4 and the AGENTS.md cascade section
("combined LCB + second repro deferred (046 T5)"). Measurement-only: no
library default, no recommendation change is in scope at this issue — any
recommendation change stays an owner call under the 042 promotion-trigger
discipline.

## What runs (two lanes, one session, byte-compared against committed baselines)

### Run A — combined posture + the LCB leg (the new cell)

The Bench-066 `both/` recommended posture + `--cascade-worthiness-lcb 0.05`:

```sh
LAYA_DEVICE=metal CARGO_TARGET_DIR=/tmp/reflex_lcb cargo run --release \
  --features laya-riir-metal --bin harness -- \
  --datasets-dir .raw/datasets_t20k --cascade --cascade-worthiness \
  --cascade-worthiness-margin 0.16 --cascade-worthiness-lcb 0.05 \
  --gate-fit-selection --gate-distance-only \
  --nb-select --oc-select --ridge-select \
  --suites ag_news,emotion,sst5,typed_decisions,prompt_injections,xnli_en,massive_intent_en,banking77 \
  --out .benchmarks/071_combined_lcb/combined_lcb
```

### Run B — the exact Bench-070 fused+LCB command (the second independent repro)

The `.benchmarks/070_cascade_probe_lcb.md` §"What ran" command verbatim,
fresh out dir:

```sh
  ... --out .benchmarks/071_combined_lcb/fused_lcb_repro
```

## Preregistered predictions (from the committed 066 `both/` probe table + the LCB formula — written before the run)

- **P1 (headline): Run A is a no-op.** All 8 dataset suites read
  byte-identical to `.benchmarks/066_gate_rate_axis_levers/both/results.json`
  (accuracy, escalation %, n_escalated, probe n, probe delta). The three
  066-combined armed rows arm via the MARGIN leg (probe Δ +0.2750 typed /
  +0.2075 ag_news / +0.3514 xnli — all ≥ 0.16), and every disarmed row's
  probe Δ (sst5 +0.1341, massive +0.1489, emotion −0.3333, prompt −0.2609,
  banking77 −0.3571) sits too low for its LCB to reach 0.05: the LCB leg
  rescues exactly the fused-posture ag_news cell (Bench 070) and nothing on
  the recommended posture. The run still records the combined probe sets'
  LCB column — new data, closing the floor family's reach boundary.
- **P2 (numeric sub-prediction):** measured LCBs in Run A — massive ≈
  +0.1489 − 1.645·SE ≈ −0.001 (probe n ≈ 60); sst5 ∈ [−0.02, +0.01]
  (n ≈ 60–82, the twice-measured flip class); emotion/prompt/banking77
  strongly negative (< −0.15).
- **P3: Run B reproduces Bench 070 exactly.** All ten rows byte-identical to
  `.benchmarks/070_cascade_probe_lcb/results.json` (accuracy, esc%,
  n_escalated, probe Δ, probe LCB to 4dp); the live LCBs again match the
  pinned unit-test vectors (ag_news +0.0854, massive +0.0064, sst5 −0.0059).

## Kill criteria (written before the run)

- **K1 (determinism):** any Run B row differing in ANY compared field → a
  nondeterminism finding; STOP, no repro claim, diagnose G5 before any
  further cascade-lane measurement anywhere.
- **K2 (floor transferability):** if a combined disarmed row ARMS via the
  LCB leg (P1 violated), the story changes from "no-op" to "the leg adds on
  the recommended posture too" — acceptable ONLY if that row's test-side
  delta is positive AND neither sst5 nor massive (the twice-measured
  arm-side flip class) arms with a negative test delta. If a flip-class
  suite arms and regresses on test → the 0.05 floor has a false-arm on the
  combined probe family → record NEGATIVE for floor transferability across
  probe families, lever stays opt-in, recommendation unchanged.

## Acceptance

Run A rows == 066 `both/` (P1) AND Run B rows == 070 (P3) → T5 CLOSED with
the recorded answer: the LCB leg's reach is exactly the fused-posture
ag_news cell; the recommended combined posture needs no LCB leg; Bench 070
stands reproduced. Bench record: `.benchmarks/071_combined_lcb.md` (next
number — highwater 070), issue closed + removed at landing per the
noise-reduction rule.
