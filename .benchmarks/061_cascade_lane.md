# Bench 061 — Issue 038 T4′: the "Reflex · cascade" lane — landed, measured, verdict NEGATIVE at the shipped gate

**Date:** 2026-09-27 · **Host:** M3 Max (macOS 26.6.2, AC, High Power mode, load 5.7→3.8
over the run — preflight PASSED, `latency_quotable: true`) · **Protocol:** Bench-052
(`--datasets-dir .raw/datasets_t20k`) · **Features:** default (`modelless, nb_scope,
option_cond, nb_ridge`) + `laya-riir-metal`, `LAYA_DEVICE=metal` · **Commits:** lane
`337974d`, record this commit.

## What landed (T4′)

`--cascade` (opt-in, mutually exclusive with `--skip-laya`, needs `laya-riir`): the
modelless answers stand; the CALIBRATED fused gate's abstains (the shipped T1.6
arena posture) escalate to each served riir-laya checkpoint. Pure composer
(`src/harness/cascade.rs`, ungated, 6 module tests): per-question join, loud on
shape drift; un-escalated abstains (escalator bucket skip / question-cap trim)
keep the forced modelless pick and are DISCLOSED, never folded into the escalation
rate — the rate IS the latency claim (a deployed cascade pays the escalator only
on that fraction). The python reference lane never escalates (latency oracle).
TABLES.md gains the cascade block with the causal pair: the escalator's accuracy
ON the escalated set beside what the forced modelless picks would have scored
there. The escalation target is per checkpoint (`cascade/english`, …,
`cascade/typed`) — the checkpoint is part of the lane identity, same keying as the
laya rows.

## The measurement (armed posture: the selection flags on, the published posture)

| suite | mless | cascade | esc% | laya | laya@esc | mls@esc | Δ cascade−mless |
|---|---|---|---|---|---|---|---|
| typed_decisions | 0.4655 | **0.7425** (typed) | 96.7 | 0.7445 | 0.7384 | 0.4519 | **+27.7** |
| ag_news | 0.8825 | 0.9500 | 93.0 | 0.9500 | 0.9462 | 0.8737 | +6.8 |
| emotion | 0.8850 | 0.6025 | 97.2 | 0.5925 | 0.5913 | 0.8817 | **−28.3** |
| sst5 | 0.3967 | 0.3750 | 98.8 | 0.3717 | 0.3727 | 0.3946 | −2.2 |
| prompt_injections | 0.7672 | 0.7414 | 90.5 | 0.6983 | 0.7143 | 0.7429 | −2.6 |
| xnli_en | 0.5233 | 0.6400 | 32.3 | 0.8600 | **0.8660** | 0.5052 | **+11.7** |
| massive_intent_en | 0.7800 | 0.7500 | 31.0 | 0.6933 | 0.7097 | 0.8065 | −3.0 |
| banking77 | 0.8420 | 0.7020 | 32.2 | 0.4220 | 0.3478 | 0.7826 | **−14.0** |
| code_fixtures | 0.2500 | 0.2500 | 12.5 | 0.6667 | 0.3333 | 0.3333 | 0 |
| typed (english ckpt) | 0.4655 | 0.3815 | 96.7 | 0.3575 | 0.3650 | 0.4519 | −8.4 |
| typed (multilingual) | 0.4655 | 0.3730 | 96.7 | 0.3490 | 0.3563 | 0.4519 | −9.3 |

(`laya@esc` / `mls@esc` = correctness on the escalated set only — the causal pair.)

## Verdict: the T4′ gate FAILS at the shipped fused gate

Gate (issue 038): accuracy ≥ modelless on every suite, within noise of laya on
xnli/typed, escalation < 100% on the topical suites.

- **accuracy ≥ modelless** — FAILS on 5/8 dataset suites (emotion −28.3, banking77
  −14.0, massive −3.0, prompt_injections −2.6, sst5 −2.2). PASSES on typed (+27.7),
  ag_news (+6.8), xnli (+11.7).
- **within noise of laya on xnli/typed** — typed ✓ (0.7425 vs 0.7445); xnli ✗
  (0.6400 vs 0.8600 — 22 pt short, the gate escalates only 32% there).
- **escalation < 100%** — ✓ everywhere (max 98.8%, sst5).

The lane stays OPT-IN (a flag, nothing promoted — nothing to demote). The negative
is structural, and the measurement names the mechanism precisely:

**The shipped fused gate's test-side abstain rate is 90–99% on exactly the suites
where the armed modelless lane is strong** (ag_news 93.0, emotion 97.2, sst5 98.8,
typed 96.7, prompt_injections 90.5 — the same rates the site already publishes as
`calibrated_abstain`; my run reproduces the published rows byte-identically on
13/14 suites, so this is the shipped posture, not a new defect). The cal-slice fit
targets ρ = 30%, so the gate's cal→test transfer is broken AT THE ARMED POSTURES —
escalation wholesale-replaces the modelless lane, and the cascade inherits laya's
accuracy wherever laya is worse (emotion: cascade 0.6025 ≈ laya 0.5925 while
modelless alone was 0.8850 — the escalated set's modelless-would-have was 0.8817).

Where the gate DOES transfer (~30% test abstain: xnli 32.3, massive 31.0,
banking77 32.2), escalation helps exactly where laya > modelless (xnli: laya@esc
0.8660 vs mls@esc 0.5052 → cascade +11.7) and hurts exactly where laya < modelless
(banking77: 0.3478 vs 0.7826 → −14.0; massive −3.0). The abstain gate is
anti-correlated with where escalation helps: it escalates the suites where
modelless needs no help and spares the ones where the escalator is strong.

**Secondary posture (unarmed, same commit, same run protocol minus the selection
flags — drafter-only readout):** the gate transfers at ~31–39% on the dataset
suites (typed 92.0 the exception), and the cascade reads between its parents on
every suite (ag_news 0.4050/0.6050/0.9500, emotion 0.2750/0.4025/0.5925, xnli
0.3400/0.5133/0.8600) — the composition math is verified end-to-end, and the
armed postures are what break the transfer. Console record: this run's log
(pre-selection); results.json kept for the armed run only.

## G3 — the flag-off claim

Full no-cascade run at the same commit (`468ca24` tree, same command minus
`--cascade`): **1,712 non-timing fields across every suite's modelless + laya
lanes — ZERO diffs** against the cascade run. Only wall-clock fields move
(latency p50/p99/seconds — the second run rode a hotter box; ag_news laya p50
22→85 ms is the known sequential-run artifact, not a code path) plus the meta
timestamp/box_state. `results.json` carries the cascade blocks only when armed
(`skip_serializing_if` empty) — unarmed runs stay byte-identical in shape.

## Provenance

- Lane + composer + tests: `337974d` (clippy clean at default / laya-riir /
  laya-riir-metal / no-default; lib 132/132 incl. 6 cascade module tests).
- Determinism: engine `det=true` on every suite (both runs); the composer is a
  pure function of the two lanes' records (module tests pin it both directions:
  escalation beats and loses modelless).
- The laya numbers here are the same-run metal rows (typed 0.7445 / ag_news
  0.9500 / xnli 0.8600 — matching the published site rows).
- G4: no new hot-path allocation — the composer runs analysis-side (post-hoc,
  µs over 2k questions), the engine paths are untouched.

## Follow-up (filed)

`.issues/042_cascade_escalation_gate_transfer.md` — the gate-transfer lever: the
cascade is only as good as the abstain gate's cal→test transfer. Any fix must
stay protocol-legal (test never enters a fit): candidates are a selection-slice
fit for the ρ target, a distance-gate-only escalation axis, or per-suite
escalation-worthiness derived from the cal slice's OWN laya-vs-modelless delta.
Re-run this bench unchanged (one command) when the gate moves — the lane is the
instrument.
