# Issue 042 — the cascade's escalation gate: cal→test abstain transfer breaks at the armed postures

**Status:** OPEN — lever 3 LANDED + measured (Bench 063: the gross direction
blindness is FIXED — all four suites 061 lost now disarm and read exactly
the modelless row; the two residual FAILs are the cal→test sign flip at
|probe Δ| ≤ 0.15, same amounts 061 had; a margin ≥ 0.16 would pass the gate
10/10 on that run at the price of ag_news's +6.75). Live: the margin call
(fixed constant vs per-suite cal-fit, T3 below) + levers 1–2 (the rate axis,
blocked on the sibling's T5 genome work).

## The finding (Bench 061, measured)

The fused gate's thresholds are fitted per suite at the cal-slice 30th percentile
(the T1.6 arena posture, ρ = 30%). At the ARMED postures (the promoted nb/oc/ridge
scales), the test-side abstain rate reads 90–99% on the strong suites:

| suite | cal target | test abstain (armed) | test abstain (unarmed) |
|---|---|---|---|
| ag_news | 30% | 93.0% | 39.2% |
| emotion | 30% | 97.2% | 33.0% |
| sst5 | 30% | 98.8% | 31.5% |
| typed_decisions | 30% | 96.7% | 92.0% |
| prompt_injections | 30% | 90.5% | 38.8% |
| xnli_en | 30% | 32.3% | 32.3% |
| massive_intent_en | 30% | 31.0% | 31.7% |
| banking77 | 30% | 32.2% | 32.2% |

Consequences, measured (Bench 061): the cascade (escalate abstains to laya)
inherits laya's accuracy on the escalated mass — emotion cascade 0.6025 vs the
modelless lane's own 0.8850 (−28.3); and where the gate does transfer, escalation
is uncritical about DIRECTION — banking77 loses 14.0 pt (laya-on-escalated 0.3478
vs modelless-would-have 0.7826) while xnli gains 11.7 (0.8660 vs 0.5052).

The gate is anti-correlated with where escalation helps. Any cascade promotion is
blocked on this; the lane is the instrument (re-run Bench 061's one command when
the gate moves).

## Constraints (protocol, non-negotiable)

- The test split never enters any fit (issue 038's protocol rule). The cal→test
  shift itself cannot be "measured away" with test data.
- No training: modelless paths only (this is the public repo).

## Candidate levers (each must re-clear selection on the cal/selection slice)

1. **Selection-slice ρ fit** — fit the abstain-rate target (and/or the threshold
   percentile) on the STRATIFIED selection slice instead of the train-tail cal
   slice; the selection slice is already the posture-picker's distribution and is
   drawn from the pool region. Hypothesis: the train-tail cal slice's confidence
   distribution is closer to the corpus than test rows are at the armed postures
   (the armed readout's score scale shifts with posture; the cal slice's
   quantile misplaces the test threshold).
2. **Distance-axis-only escalation** — escalate on the corpus-distance gate alone
   (no score axis). The distance axis read ~31–32% test abstain on xnli/massive/
   banking77 in every posture measured so far — it may be the transfer-stable half.
3. **Per-suite escalation-worthiness** — derive from the cal slice whether the
   escalator is expected to beat the forced modelless picks (a laya-vs-modelless
   delta on the cal slice is protocol-legal selection), and disarm the cascade
   where it reads negative (banking77/massive would disarm; xnli/typed arm).
   **LANDED 2026-09-27 (Bench 063, `--cascade-worthiness`):** the prediction
   held on the gross errors — banking77/emotion/typed-en+multi/prompt
   disarmed, xnli/typed-typed armed — but the 061 spec's own example was
   half-wrong: massive reads POSITIVE on cal (+0.150) yet loses −3.0 on
   test; sst5 the same (+0.072 → −2.2). The arm-side reliability is
   magnitude-gated: probe-positive deltas ≥ 0.288 predicted the test-side
   sign 2/2, probe-positive 0.072–0.150 went 1/3 (ag_news right, sst5 and
   massive flipped), while the DISARM side was 5/5 across −0.049…−0.350 —
   only the ARM bar needs raising, not the disarm bar. A margin anywhere
   in (0.150, 0.288] separates the measured rows perfectly (0.16 sits in
   that gap) — the margin call is T3 below.
4. ~~**Posture-conditional refit**~~ — **VOID (checked 2026-09-27, the day of
   filing)**: the fit order in `run_modelless` is already posture-correct —
   `default_cfg` absorbs the selected head/nb (2188) → oc (2228) → ridge
   (2250) scales BEFORE the fused-gate probe (2276) builds its engine at
   `default_cfg.clone()` and fits both thresholds on the cal slice at the
   FULLY ARMED posture. The ρ=30% target is realized on cal by construction;
   the 90–99% test-side abstain is a genuine cal→test confidence-geometry
   shift at the armed postures, not a stale fit. The live levers are 1–3.

## T3 — the margin call (lever 3's open half, measured Bench 063)

The probe's DISARM side is magnitude-robust: every negative probe
(−0.049…−0.350, five checkpoints) predicted a test-side loss and disarmed
— 5/5, including the smallest (prompt −0.049 → would-be −2.6). The ARM
side is where magnitude gates reliability: armed positives ≥ 0.288 went
2/2 (typed-typed +0.288 → test +0.277; xnli +0.400 → +0.117), while
probe-positive 0.072–0.150 went 1/3 (ag_news +0.132 → test +0.0675 right;
sst5 +0.072 → test −0.0217 and massive +0.150 → test −0.030 flipped).
Two candidate fixes, both protocol-legal:

- **(a) fixed margin** — ship `--cascade-worthiness-margin 0.16` as the
  recommended posture (one CLI flag, already landed; zero code). Cost:
  ag_news's +6.75 pt armed gain is surrendered (its probe reads +0.132).
- **(b) per-suite cal-fit margin** — fit the bar per suite on a cal SPLIT
  (hold out half the cal slice for the fit, probe on the other half), so
  ag_news's margin is fit from its own cal distribution. More code, keeps
  the gains; the fit-split law must be pinned (never test, and the probe
  half never overlaps the fit half).

Acceptance for either: Bench 063's command re-run → cascade ≥ modelless on
every dataset suite AND no armed suite loses its measured gain vs the
0.0-margin run (ag_news must keep ≥ +0.0675 under (b)).

## Acceptance

Bench 061's command re-run with the new gate: cascade accuracy ≥ modelless on
every dataset suite (the T4′ gate), escalation within [15%, 60%] on the topical
suites, no suite regresses beyond noise vs the modelless row. Then the T4′
promotion question re-opens with data.
