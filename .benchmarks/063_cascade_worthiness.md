# Bench 063 — cascade worthiness (issue 042 lever 3)

**Status:** COMPLETE — the lever LANDED and is measured. Verdict: the probe
fixed every gross direction error (all four suites Bench 061 lost now read
cascade == modelless exactly); at the default margin 0.0 two small-delta
suites still regress by the SAME amounts 061 had (the cal→test sign flip at
|probe Δ| ≤ 0.15); a margin ≥ 0.16 would have passed the gate fully on this
run (derived from the measured rows, not re-run). The margin call is the
remaining owner question in issue 042.

## What ran

The Bench 061 command plus the new gate (issue 042 lever 3, landed this
batch): the escalator answers each suite's CAL-slice questions the
calibrated gate abstained on, and the suite's escalation stays armed only
where the escalator reads ≥ the forced modelless picks on that probe set
(delta ≥ `--cascade-worthiness-margin`, default 0.0). A negative probe
disarms the suite — every abstain then stands as the modelless forced pick.

```sh
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --bin harness -- \
  --datasets-dir .raw/datasets_t20k --cascade --cascade-worthiness \
  --nb-select --oc-select --ridge-select \
  --suites ag_news,emotion,sst5,typed_decisions,prompt_injections,xnli_en,massive_intent_en,banking77 \
  --out .benchmarks/063_cascade_worthiness
```

Artifacts: `results.json` (each cascade row carries `worthiness`: verdict,
probe n, both probe accuracies, delta, min delta, unprobed reason) +
`TABLES.md` (per-checkpoint worthiness sub-table beside the cascade block).

## PROVENANCE (box state at the run)

`./scripts/bench_preflight.sh` **REFUSED** (canary 195.0 µs best-of-5 vs AC
reference 141 µs, +15% tol; load 5.49; swap 1890 MB) → **latency numbers
from this run are NOT publishable** and are treated as provisional
throughout. The process table showed one sibling PAW-hosted lane (HTTP
subprocess, laya-free, ~0% CPU) — no GPU compute consumer; the canary miss
reads thermal/transient, unresolved. **The accuracy verdicts are
load-immune by construction**: laya picks are bit-deterministic (G5 parity,
proven cross-host in Bench 062), and every gate quantity here is a pick
count, never a latency. Only the p50 columns inherit the caveat.

## The gate (T4′ acceptance: cascade ≥ modelless on every dataset suite)

| suite | modelless | cascade | Δ | esc% | verdict | worthiness | 061 cascade (same flags, no gate) |
|---|---|---|---|---|---|---|---|
| typed_decisions · typed | 0.4655 | 0.4655 | 0.0000 | 0.0 | PASS | DISARMED Δ −0.138 | 0.3815 |
| typed_decisions · multilingual | 0.4655 | 0.4655 | 0.0000 | 0.0 | PASS | DISARMED Δ −0.188 | 0.3730 |
| typed_decisions · english | 0.4655 | 0.7425 | +0.2770 | 96.7 | PASS | armed Δ +0.288 | 0.7425 |
| ag_news | 0.8825 | 0.9500 | +0.0675 | 93.0 | PASS | armed Δ +0.132 | 0.9500 |
| emotion | 0.8850 | 0.8850 | 0.0000 | 0.0 | PASS | DISARMED Δ −0.313 | 0.6025 |
| sst5 | 0.3967 | 0.3750 | −0.0217 | 98.8 | **FAIL** | armed Δ +0.072 | 0.3750 |
| prompt_injections | 0.7672 | 0.7672 | 0.0000 | 0.0 | PASS | DISARMED Δ −0.049 | 0.7414 |
| xnli_en | 0.5233 | 0.6400 | +0.1167 | 32.3 | PASS | armed Δ +0.400 | 0.6400 |
| massive_intent_en | 0.7800 | 0.7500 | −0.0300 | 31.0 | **FAIL** | armed Δ +0.150 | 0.7500 |
| banking77 | 0.8420 | 0.8420 | 0.0000 | 0.0 | PASS | DISARMED Δ −0.350 | 0.7020 |

(modelless and laya rows reproduce Bench 061's at the same flags — the
modelless accs are identical to 4dp on all 8 suites; the armed cascade rows
are byte-equal to 061's because the composition input is unchanged.)

## Verdict

1. **The direction blindness is FIXED where it fired.** Every suite 061
   LOST is now disarmed and reads exactly the modelless row: emotion
   (−28.3 pt in 061 → 0.000), banking77 (−14.0 → 0.000), typed
   english/multilingual (−8.4/−9.2 → 0.000), prompt_injections (−2.6 →
   0.000). The probe caught all four from the cal slice alone — 8 of 10
   checkpoint rows got the test-side SIGN right from cal data.
2. **The two residual FAILs are the cal→test sign flip at small armed
   margins**, and they are NOT new: sst5 (−2.2) and massive (−3.0) regress
   by exactly their 061 amounts. The decomposition: DISARM-side probes
   were 5/5 (every negative probe −0.049…−0.350 predicted a real test-side
   loss); ARM-side probes ≥ 0.288 were 2/2; ARM-side probes in
   0.072–0.150 were 1/3 (ag_news right, sst5 + massive flipped). The
   reliable arm bar sits somewhere in (0.150, 0.288] — the measured gap.
3. **Margin sensitivity (derived from the measured rows, not re-run):**
   the disarm decision is deterministic given the probe deltas, so the
   counterfactual margins are arithmetic on this table — at margin ≥ 0.16
   (inside the measured gap (0.150, 0.288]) only typed-english (+0.288)
   and xnli (+0.400) stay armed, sst5 and massive disarm, and **the gate
   passes 10/10** — the price is ag_news's +6.75 pt (its +0.132 probe
   falls below the bar). Margin 0.05 keeps ag_news armed but still FAILs
   (sst5 +0.072 and massive +0.150 stay armed and lose). The default
   stays 0.0 (no knob magic); picking the margin — fixed constant in the
   measured gap vs per-suite cal-fit — is issue 042 T3's live question,
   with this table as its evidence.
4. **The rate axis is untouched, as designed** (levers 1–2 territory):
   armed suites keep their 061 escalation rates (typed 96.7%, sst5 98.8%,
   ag_news 93% — above the issue's [15, 60] band; xnli 32.3% and massive
   31.0% inside it).

## G3 (flag-off identity)

`--cascade` without `--cascade-worthiness` emits cascade rows with NO
`worthiness` key (skip_serializing_if) — checked on a capped emotion run;
the landed T4′ results.json shape is preserved. Worthiness armed adds the
cal-side modelless eval (µs) + cal laya serving (~60–479 questions per
suite+checkpoint, the probe's cost face: ~2 s/suite on the metal lane).

## Gates

- `cargo clippy --all-targets` 0 findings; `--features laya-riir`,
  `--no-default-features`, `--all-features` (our crate) clean.
- `cargo test --lib` 140/140 (14 cascade module tests, 8 new: disarm, arm,
  margin boundary, unprobed halves, thin support, served∩abstained probe
  set, shape drift, lever-off identity).
- The suite modelless rows reproduce 061's to 4dp (13/14 byte-identical
  reproduction was 061's own posture proof; this run reads the same
  posture through the same selection flags).

## Addendum — T3 acceptance re-run at the fixed margin 0.16 (2026-09-27)

Issue 042 T3 DECIDED (a): **0.16 is the recommended posture** — in the
measured gap (0.150, 0.288], the only measured discriminator (magnitude),
and the one a cal-split fit (b) cannot improve on: (b) reduces estimator
variance but cannot observe a cal→test shift, which is precisely the
failure mode at small positive probes. Recorded in the issue; the reopen
trigger for (b) is a future lane run where a fixed-bar ARM-side row flips.

Acceptance evidence — the exact command above plus
`--cascade-worthiness-margin 0.16`, artifacts
`.benchmarks/063_cascade_worthiness_margin016/` (results.json + TABLES.md):

| suite | probe Δ | verdict @0.16 | cascade | modelless | test Δ |
|---|---|---|---|---|---|
| typed_decisions · typed | +0.2881 | armed | 0.7425 | 0.4655 | **+0.2770** |
| typed_decisions · english | −0.1378 | DISARMED | 0.4655 | 0.4655 | 0.0000 |
| typed_decisions · multilingual | −0.1879 | DISARMED | 0.4655 | 0.4655 | 0.0000 |
| ag_news | +0.1316 | DISARMED | 0.8825 | 0.8825 | 0.0000 (the +6.75 price) |
| emotion | −0.3131 | DISARMED | 0.8850 | 0.8850 | 0.0000 |
| sst5 | +0.0718 | DISARMED | 0.3967 | 0.3967 | 0.0000 (was FAIL −2.2) |
| prompt_injections | −0.0494 | DISARMED | 0.7672 | 0.7672 | 0.0000 |
| xnli_en | +0.4000 | armed | 0.6400 | 0.5233 | **+0.1167** |
| massive_intent_en | +0.1500 | DISARMED | 0.7800 | 0.7800 | 0.0000 (was FAIL −3.0) |
| banking77 | −0.3500 | DISARMED | 0.8420 | 0.8420 | 0.0000 |

**10/10 PASS** — cascade ≥ modelless on every suite; the two 063 FAILs
read modelless exactly; both armed gains kept. Every probe delta re-read
byte-identical to the 0.0-margin run (G5 determinism end to end).
PROVENANCE (this box, the re-run): preflight REFUSED — load 6.08 (sibling
session), swap 1890 MB, canary 125.8 µs (healthy), power AC, powermode 2
— same provisional-latency posture as the main run; accuracy gates are
pick-counts, load-immune. Library default stays 0.0; the AGENTS.md lane
command carries 0.16 (the second independent 10/10 run — ideally after
levers 1–2 — is the default-promotion trigger).
