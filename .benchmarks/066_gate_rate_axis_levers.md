# Bench 066 — Issue 042 levers 1–2, the gate rate axis: lever 2 transfers, the combined posture passes the T4′ acceptance

**Date:** 2026-09-27 · **Commit:** the `feat(042)` levers commit (`842f89d`) ·
**Box:** M3 Max (macOS 26.6.2), AC, `powermode=2(high)`; **preflight REFUSED**
(load 7.93 — a sibling session active) → all p50/p99 latency figures
PROVISIONAL. Accuracy gates are pick-count/load-immune (the 063/065
precedent); the modelless forced rows matched the published A0/armed-posture
numbers exactly (ag_news 0.8825, xnli 0.5233, emotion ridge@8 0.8850,
cross-host-verified state), which is the no-perturbation check.

## The runs

One command, three gate postures (all else = the 063 lane posture:
`--cascade --cascade-worthiness --cascade-worthiness-margin 0.16
--nb-select --oc-select --ridge-select`, t20k datasets, LAYA_DEVICE=metal):

| run dir | flags |
|---|---|
| `lever2_only/` | `--gate-distance-only` |
| `lever1_only/` | `--gate-fit-selection` |
| `both/` | both flags |

15/15 suites PASSED in every run; det ✓ throughout.

## The acceptance table (cascade − modelless, dataset suites)

| suite (n) | shipped 063@0.16 | lever 1 | lever 2 | **BOTH** |
|---|---|---|---|---|
| typed·typed (2000) | +0.2770 @ 96.7% | +0.2775 @ 97.0% | +0.1655 @ 47.5% | **+0.1685 @ 49.6%** |
| ag_news (400) | 0 (disarmed) | 0 (disarmed) | +0.0325 @ 39.2% | **+0.0300 @ 37.5%** |
| emotion (400) | 0 | 0 | 0 | **0** |
| sst5 (600) | 0 | 0 | **−0.0133** | **0 — disarmed** |
| prompt_injections (116) | 0 | 0 | 0 | **0** |
| xnli_en (300) | +0.1167 @ 32.3% | +0.0600 @ 15.7% | +0.1167 @ 32.3% | **+0.0600 @ 15.7%** |
| massive_intent_en (300) | 0 | 0 | 0 | **0** |
| banking77 (500) | 0 | 0 | 0 | **0** |

(Δ column = cascade accuracy − modelless forced accuracy; `@ x%` = the
escalation rate — the lane's latency claim. "0" = disarmed by worthiness,
reads modelless EXACTLY.)

## Verdicts

1. **Lever 2 (distance-only) — the transfer finding CONFIRMED, with one
   new flip.** The corpus-distance axis's test-side abstain lands at
   31.5–49.6% on every armed topical suite (the fused gate read 90–99% —
   the 061 pathology is a SCORE-axis property; the distance axis transfers).
   xnli's +11.67 recovers at a sane 32.3%; ag_news ARMS on the new probe
   set (+0.20) and gains +3.25 at 39.2%. **But sst5 flips again**: its
   distance-probe reads +0.2000 — exactly equal to ag_news's +0.2000 (12/60
   on both) — so NO margin separates them, and the armed sst5 loses −1.33
   on test (laya 0.3704 vs modelless 0.4127 on the escalated 189). The
   arm-side flip class is now measured on BOTH probe-set families (063:
   fused +0.072 → −2.2; 066: distance +0.20 → −1.33). sst5's cal→test
   geometry is the instability, not any one gate.
2. **Lever 1 (selection-slice fit) alone — NULL on the fused gate.** Every
   armed/disarmed verdict repeats the shipped posture (typed +0.2775 ≈
   +0.2770; ag_news same disarm at +0.1316); xnli's gain HALVES (+0.0600 at
   15.7% — the selection-slice fit raises the distance threshold, fewer
   escalations). The cal-slice fit was never the fused gate's problem. Its
   value is only in combination (below).
3. **BOTH — the first posture that passes the issue's full T4′ acceptance.**
   - cascade ≥ modelless on EVERY dataset suite ✓ (typed +0.1685, ag_news
     +0.0300, xnli +0.0600; all others read modelless exactly);
   - escalation within [15%, 60%] on the armed topical suites ✓
     (49.6 / 37.5 / 15.7% — the shipped posture's typed 96.7% FAILS this
     window, which is the latency half of the acceptance);
   - no suite regresses beyond noise ✓ — the lever-2 sst5 loss is GONE:
     the selection-slice fit shifts sst5's distance threshold, its probe
     set reads +0.1341 < 0.16, and the suite disarms to modelless-exact;
   - **the fixed 0.16 margin holds 8/8 on the shifted probe sets** (armed
     right: typed +0.2750 → test +0.1685, ag_news +0.2075 → +0.0300, xnli
     +0.3514 → +0.0600; disarmed right: sst5 +0.1341, massive +0.1489 —
     the known 063 massive test-side flip avoided — prompt −0.2609,
     emotion −0.3333, banking77 −0.3571). This is additional, independent
     evidence for the T3 margin call — a second probe-set family where
     (0.150, 0.288] continues to separate.
   - The tiny synthetic/code families carry the known thin-support-armed
     noise (probe n < 16 → pre-lever behavior), unchanged from the shipped
     posture; one 16-question family flips −0.1250 there (2 questions) —
     pre-existing, not lever-caused.
4. **Recommended posture updated:** the cascade lane command gains
   `--gate-fit-selection --gate-distance-only` (keeping margin 0.16). The
   library defaults stay OFF (byte-identical); the fused-posture command
   remains valid and keeps its 063@0.16 record (higher headline gain
   +0.394 total, but typed at 96.7% escalation pays the escalator on
   almost everything — cascade 0.7430 vs laya-typed alone 0.7445 — the
   rate window exists for exactly this).

## What this re-opens

The T4′ promotion question now has data: at the BOTH posture every armed
suite sits at a defensible latency/accuracy point (ag_news +3.0 pt at 37.5%
escalator cost; xnli +6.0 at 15.7%; typed +16.9 at 49.6% — half-cost laya
vs laya-alone 0.7445). Promotion to any DEFAULT surface remains a separate
owner call (the lane stays opt-in; library defaults untouched).

**(b)-reopen trigger update (issue 042):** sst5 is now a TWICE-measured
arm-side flip across two probe families. If it flips a third time (any
posture), the fix is not a margin — it is an sst5-specific mechanism
(e.g. a per-suite probe-size floor: its flip happened at probe n=60/82,
the smallest dataset cal slice in play) — file it there first.

## Provenance

- `results.json` + `TABLES.md` per run (the exact harness bytes).
- Commands: the 063 lane command + the posture flags per the table above;
  `--datasets-dir .raw/datasets_t20k`; `LAYA_DEVICE=metal`.
- Preflight REFUSED (load 7.93, sibling session): latency PROVISIONAL,
  accuracy pick-count/load-immune; forced rows byte-match the published
  state (ag 0.8825 / xnli 0.5233 / emotion@ridge8 0.8850).
