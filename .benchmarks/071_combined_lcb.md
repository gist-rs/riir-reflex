# Bench 071 — T5 closure: the combined-posture LCB leg is a provable no-op + the Bench-070 repro is byte-exact (issue 048)

**Status:** COMPLETE — both preregistered legs PASS, zero kill criteria
fired. Run A (combined + LCB) is byte-identical to Bench 066 `both/` on all
ten dataset rows — **the LCB leg changes nothing on the recommended
posture**. Run B (the exact Bench-070 fused+LCB command) is byte-identical
to `.benchmarks/070_cascade_probe_lcb/results.json` on all ten rows
INCLUDING the `probe_lcb` floats — the 042-style second-independent-repro
bar is met. T5 CLOSED; no default change, no recommendation change.

## What ran

Preregistered in `.issues/048` (predictions P1–P3, kill criteria K1–K2)
BEFORE any run, committed `eedfe24` — the issue-046 pattern.

Run A — the Bench-066 `both/` recommended posture + the LCB leg:

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

Run B — the Bench-070 "What ran" command verbatim, fresh out dir
(`--out .benchmarks/071_combined_lcb/fused_lcb_repro`, no
`--gate-*` flags — the fused posture).

## PROVENANCE (box state)

`scripts/bench_preflight.sh` **REFUSED at launch** — `load 6.92 >
MAX_LOAD 6.0` (three sibling sessions on the box; power=AC,
powermode=2(high), settle 3005 min on AC, canary 116.7 µs best-of-5). No
latency number is claimed anywhere in this bench: every measured quantity
is a pick count or pick-count arithmetic (the LCB over probe picks),
load-immune by G5 determinism — and the byte-identity results themselves
are the strongest load-immunity witness. Both runs at meta `git_sha
eedfe24`, code-identical to the lever commit `a6bfec3` (docs-only between).

## Run A — the acceptance table (vs Bench 066 `both/`)

| row | modelless | cascade | Δ | esc% | verdict | probe n | probe Δ | LCB |
|---|---|---|---|---|---|---|---|---|
| typed_decisions·english | 0.4655 | 0.4655 | 0 | 0% | DISARMED | 160 | −0.2188 | −0.3081 |
| typed_decisions·multilingual | 0.4655 | 0.4655 | 0 | 0% | DISARMED | 160 | −0.2562 | −0.3443 |
| typed_decisions·typed | 0.4655 | 0.6340 | +0.1685 | 49.6% | armed (margin) | 160 | +0.2750 | +0.1946 |
| ag_news | 0.8825 | 0.9125 | +0.0300 | 37.5% | armed (margin) | 53 | +0.2075 | +0.1081 |
| emotion | 0.8850 | 0.8850 | 0 | 0% | DISARMED | 57 | −0.3333 | −0.4659 |
| sst5 | 0.3967 | 0.3967 | 0 | 0% | DISARMED | 82 | +0.1341 | **+0.0152** |
| prompt_injections | 0.7672 | 0.7672 | 0 | 0% | DISARMED | 23 | −0.2609 | −0.4749 |
| xnli_en | 0.5233 | 0.5833 | +0.0600 | 15.7% | armed (margin) | 37 | +0.3514 | +0.2052 |
| massive_intent_en | 0.7800 | 0.7800 | 0 | 0% | DISARMED | 47 | +0.1489 | −0.0086 |
| banking77 | 0.8420 | 0.8420 | 0 | 0% | DISARMED | 70 | −0.3571 | −0.4844 |

All ten rows byte-identical to `.benchmarks/066_gate_rate_axis_levers/both/`
(accuracy, esc%, n_escalated, worthiness.disarmed/probe_n/probe accs/delta;
`probe_lcb` excluded from the comparison — 066's pre-lever rows do not
carry the key; its values here are the new data). **P1 confirmed.**

The mechanism, measured: the three 066-combined armed rows all clear the
margin on their own (probe Δ +0.2750 / +0.2075 / +0.3514 ≥ 0.16 — their
LCBs are also ≥ 0.05 but the margin leg already arms them), and every
disarmed row's LCB sits below the floor. The combined probe sets are
SMALLER than the fused family's (the selection-slice fit raises thresholds
→ fewer cal abstains → smaller probes: ag_news 53 vs 190, xnli 37 vs 60,
massive 47 vs 60), so the LCBs come out weaker where it matters: massive's
fused +0.0064 becomes −0.0086, sst5's −0.0059 becomes +0.0152.

**P2 honesty:** two numeric sub-predictions missed their preregistered
bands while the decision predictions held — sst5 measured **+0.0152** vs
the band [−0.02, +0.01] (probe n 82 with the actual probe accs → a smaller
SE than the max-variance guess), massive −0.0086 vs predicted ≈ −0.001
(n 47, not the guessed 60). Neither is a kill criterion (K2 fires only on
an LCB ≥ 0.05 arming a disarmed row). Recorded as-is.

## Run B — the Bench-070 second repro

All ten rows **byte-identical** to `.benchmarks/070_cascade_probe_lcb/`
(modelless, cascade, escalation rate, n_escalated, probe n, probe accs,
probe delta, probe LCB — exact float equality). The pinned unit-test
vectors reproduce a second time through the real lane: ag_news +0.0854,
massive +0.0064, sst5 −0.0059. **P3 confirmed** — the lever-4 record now
carries the same second-independent-repro standing the margin-0.16
promotion rode (the 042 precedent), with the promotion question itself
untouched (below).

## Verdict

1. **T5 CLOSED.** The floor family's reach is now measured on BOTH
   probe-set families and it rescues exactly one cell: the fused posture's
   ag_news (sub-margin point estimate, statistically firm support). On the
   recommended combined posture it is provably inert — every armed row
   clears the margin, every disarmed row's LCB < 0.05.
2. **The recommended lane command needs no LCB flag.** The fused posture
   keeps `--cascade-worthiness-lcb 0.05` as the recorded opt-in for the
   063-class probe family. Nothing here triggers the 042 promotion
   discipline — the lever's value proposition was always the fused family;
   on the combined family the run is a no-op by construction (measured,
   not argued).
3. **Watch item (recorded, not armed):** sst5's combined-posture LCB
   +0.0152 is the closest any flip-class suite has come to the 0.05 floor —
   a future probe-family shift lifting its Δ by ~0.035 would arm it. The
   066 third-flip tripwire (an sst5-specific probe-size floor, not a margin
   change) remains the recorded answer if that ever fires.
4. **Scope:** the escalation question only — issue 047's fresh-slice rule
   and its A1–A4 ag_news modelless-mechanism work are untouched (the 070
   wording stands).

## Artifacts

- `results.json` + `TABLES.md` per run dir; meta `git_sha eedfe24` both.
- Comparison script (scratch, gitignored): `scripts/out/compare_071.py` —
  final verdict `A diffs=0  B diffs=0`.
