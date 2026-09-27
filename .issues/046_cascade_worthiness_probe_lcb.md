# Issue 046 — cascade worthiness: the probe-LCB arm leg (support-aware arming where magnitude cannot separate)

**Status:** PREREGISTERED 2026-09-27 — the rule, floor, and predictions below are
fixed from the RECORDED Bench-063 probe table before any run. Implementation +
the verification lane (Bench 070) land in the same batch; any run deviation
from these predictions is recorded honestly in the bench doc.

## Why

Issue 042 T3 fixed the escalation arm bar at margin 0.16 because MAGNITUDE was
"the one measured discriminator" — and it recorded ag_news's +6.75 as the
price: in the fused probe family (Bench 063 @ 0.16), massive_intent_en's probe
delta (+0.1500) OUTRANKS ag_news's (+0.1316) while massive flips −3.0 on test.
No point-estimate margin can arm ag_news without arming massive. The T3 record
pre-registered the remedy family: "a third flip anywhere files a per-suite
probe-size floor, not a margin change."

The recorded probe table carries a second axis magnitude ignores — SUPPORT:

| suite | probe Δ | probe n | LCB₉₅(Δ) |
|---|---|---|---|
| ag_news | +0.1316 | **190** | **+0.0854** |
| massive_intent_en | +0.1500 | **60** | +0.0064 |
| sst5 | +0.0718 | 195 | −0.0059 |

LCB₉₅ = Δ − 1.645·√(p₁(1−p₁)/n + p₂(1−p₂)/n) (two-proportion, one-sided 95%).
This SE is CONSERVATIVE on paired data: it is ≥ the paired McNemar SE whenever
the two per-question reads are positively correlated (same questions), so a
probe clearing this bound clears the exact test harder. The flip-prone pair
(massive, sst5) has LCB ≤ +0.0064; ag_news has +0.0854 at 3.2× the support.
Support separates what magnitude provably cannot.

## The rule (lever 4, `--cascade-worthiness-lcb <F64>`)

The worthiness gate gains a second arm leg, additive and default-off:

```text
armed = (delta >= min_delta)                       // lever-3 margin leg, unchanged
     || (probe_lcb >= min_probe_lcb)               // NEW: only when the flag is set
```

- `probe_lcb` computed per the formula above from the probe accs + n the
  verdict already carries; serialized only when the leg is on
  (`skip_serializing_if`) — flag-off JSON stays byte-identical with the
  Bench-063 lane.
- `None` (flag absent) = the leg does not exist: byte-identical with the
  landed lever-3 lane, library default unchanged.
- Unprobed rows (missing cal records / thin support) never arm via the LCB —
  the pre-lever behavior (armed + named reason) holds exactly as before.
- **Honest limitation (stated, not hidden):** the LCB leg is a
  support-confidence requirement, not a cal→test-shift guarantee. The arm-side
  flip class is a cal→test shift, which NO cal-side gate can rule out (the T3
  wording, verbatim). The leg's claim is the measured one: the flip-prone
  probes had LCB ≈ 0 while the reliable arms clear it — support-aware arming
  prices the risk the point estimate ignores.

## Predictions (deterministic given G5 — probe accs are byte-identical to 063)

Floor preregistered at **0.05** — mid-gap between massive's +0.0064 and
ag_news's +0.0854; any floor in (0.0064, 0.0854) separates on the recorded
table, 0.05 sits centrally. Margin stays the promoted 0.16.

Run = the exact Bench-063@0.16 command + `--cascade-worthiness-lcb 0.05`.

| row | prediction |
|---|---|
| typed_decisions·english | armed (margin leg, Δ +0.288 ≥ 0.16) → **+0.2770 @ 96.7%**, byte-equal to 063's margin-0 row |
| ag_news | armed via the **LCB leg** (+0.0854 ≥ 0.05; margin leg fails, +0.132 < 0.16) → **+0.0675 @ 93.0%**, byte-equal to 063's margin-0 row |
| xnli_en | armed (margin leg, Δ +0.400) → **+0.1167 @ 32.3%**, byte-equal to 063's margin-0 row |
| sst5 | disarmed (margin +0.072 < 0.16 AND LCB −0.006 < 0.05) → modelless EXACTLY |
| massive_intent_en | disarmed (margin +0.150 < 0.16 AND LCB +0.006 < 0.05) → modelless EXACTLY |
| emotion, prompt_injections, banking77, typed·typed, typed·multi | disarmed (margin leg, unchanged) → modelless EXACTLY, byte-equal to 063@0.16 |
| worthiness JSON | ag_news carries `probe_lcb ≈ +0.0854`; massive `≈ +0.0064`; sst5 `≈ −0.0059`; flag-off rows carry no `probe_lcb` key |

The three LCB vectors are pinned as unit tests in the same batch — the
preregistration becomes compile-checked arithmetic.

**Escalation-window note (owner-visible):** ag_news escalates ~93% under the
armed fused lane. The [15%, 60%] window is the COMBINED posture's T4′
acceptance axis; the fused lane's shipped record already carries
typed_decisions·english at 96.7% escalation. The fused lane's gate is "cascade
≥ modelless everywhere" with the rate DISCLOSED (the lane's published latency
claim) — this lever changes no gate. At 93% the ag_news row reads laya's
accuracy at near-laya serving cost; that trade is the point of the disclosed
rate, not a hidden one.

## Acceptance (Bench 070)

1. All ten checkpoint rows: cascade ≥ modelless.
2. ag_news armed with a positive test delta (+0.0675 expected).
3. sst5 + massive read modelless EXACTLY.
4. Every other row byte-identical to its Bench-063@0.16 counterpart.
5. Probe deltas byte-identical to 063 (G5 determinism end to end).

## Kill criteria

Any acceptance miss beyond float formatting — above all massive (or sst5)
arming and regressing on test — is a NEGATIVE bench record: the
probe-stability direction is REFUTED (this WAS the preregistered
probe-size-floor family, the T3-recorded remedy), ag_news's fused +6.75 is
priced unreachable, and the margin-0.16 posture stands as final. Either way
this issue closes with a measurement, and 044's ag_news note gains the
recorded verdict.

## Tasks

- [ ] T1 — land lever 4: `WorthinessInput.min_probe_lcb` + `WorthinessVerdict.probe_lcb`
      + the additive leg in `probe()`, CLI flag, TABLES column; flag-off
      byte-shape pinned.
- [ ] T2 — pin the three preregistered LCB vectors + the arm/disarm/disabled
      arms as module tests.
- [ ] T3 — run the verification lane (Bench 070) at the preregistered
      command; preflight + box-state disclosure (accuracy pick-counts are
      load-immune by G5; latency provisional under sibling load, as in 063).
- [ ] T4 — bench record + verdict vs the predictions; close this issue
      (remove per the noise-reduction rule, record in HISTORY.md) or file
      the NEGATIVE.
- [-] T5 — combined-posture LCB leg + second repro run for any DEFAULT
      recommendation: deferred — the lever ships opt-in; one run is one run
      (the 042 promotion trigger discipline applies to any posture change).
