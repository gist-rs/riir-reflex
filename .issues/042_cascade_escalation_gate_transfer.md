# Issue 042 — the cascade's escalation gate: cal→test abstain transfer breaks at the armed postures

**Status:** OPEN — filed from Bench 061 (T4′). The cascade lane itself is LANDED
(opt-in `--cascade`, `337974d`); this issue owns the lever its negative verdict
points at.

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
4. **Posture-conditional refit** — refit the gate thresholds AFTER the posture is
   armed on the SAME cal slice, verifying the realized cal abstain is ρ (if the
   current fit already does this, the lever is void — check the fit order in
   `run_modelless` first).

## Acceptance

Bench 061's command re-run with the new gate: cascade accuracy ≥ modelless on
every dataset suite (the T4′ gate), escalation within [15%, 60%] on the topical
suites, no suite regresses beyond noise vs the modelless row. Then the T4′
promotion question re-opens with data.
