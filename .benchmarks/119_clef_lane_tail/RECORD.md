# Bench 119 — the Clef lane's dataset-suite tail: 7 suites measured, all quotable (the lane's index reaches the 9-suite comparison population)

**Status: MEASURED — the standing local rig (`clef-flash-4bit` @ `127.0.0.1:8793`, `/v1/clef-lane`) answered
every suite; per-suite docs on a preflight-clean quiet box (launch load 2.91, canary 154.5 µs; every doc
quotable at BOTH ends, loads 1.1–2.7 throughout). The clef lane now carries 9 of 9 cells on the comparison-lane
population (the bekko/openthai/paw set: 8 dataset suites + code_fixtures) with latency everywhere.**

Date: 2026-10-04 · host m3-max-metal · reflex `fec0d7c`

## Why

Plan 011 C1 says "run the dataset suites with `--clef`" — benches 113+118 landed banking77 + typed_decisions
(the crosswalk's pin suites); the remaining 7 suites were the time-not-money tail. The handoff's "13" counted
board suites; the honest population per lane-consistency (clm/gliner/agentjev run the 8 dataset suites;
openthai/bekko/paw add code_fixtures; NO comparison lane runs thai/s1mb) is the 9 — so 7 remained.

## The cells (per-suite docs, quick→slow)

| suite | n | acc | ECE(maxp) | p50 | wall | det |
|---|---|---|---|---|---|---|
| code_fixtures | 32 | 0.5938 | 0.1730 | 673.0 ms | 26.1 s | ✓ |
| prompt_injections | 116 | 0.5862 | 0.3628 | 330.0 ms | 46.2 s | ✓ |
| xnli_en | 300 | 0.8133 | 0.0909 | 351.0 ms | 115.9 s | ✓ |
| massive_intent_en | 300 | **0.9333** | 0.0363 | 903.0 ms | 300.7 s | ✓ |
| emotion | 400 | 0.5925 | 0.2275 | 416.0 ms | 180.8 s | ✓ |
| ag_news | 400 | **0.9000** | 0.0399 | 456.0 ms | 199.3 s | ✓ |
| sst5 | 600 | 0.6033 | 0.0885 | 409.0 ms | 254.6 s | ✓ |

Modelless controls == the published board on all seven (the drift-guard premise). Posture identical to
113/118: `CLEF_SERVE_URL=http://127.0.0.1:8793 CLEF_RUN_PATH=/v1/clef-lane CLEF_SMOKE_MAX_CASES=1000`
(the local no-spend ceiling), `--skip-laya --clef --nb-select --oc-select --ridge-select`,
`--datasets-dir .raw/datasets_t20k`. Total lane wall ≈ 19 min.

## Readings worth the record

- **massive_intent_en 0.9333** — clef's strongest cell vs our lanes (Instinct H2 0.8400 serves; modelless
  0.7800): +9.3 pt over the served best. The 20-of-59 sampled-option shape suits the routing head.
- **ag_news 0.9000** — edges Instinct H2's 0.8975 (+0.25 pt); both over modelless 0.8825.
- **xnli_en 0.8133** vs modelless 0.5233 / Instinct A0 — clef's NLI strength (the Qwen base).
- **emotion 0.5925 / sst5 0.6033** — clef sits BETWEEN modelless (0.8850/0.3967) and chance on emotion
  (a weak cell: modelless beats it by 29 pt) while BEATING modelless on sst5 by 20.7 pt. prompt_injections
  0.5862 — modelless leads (0.7672). code_fixtures 0.5938 — near Instinct A1's 0.5625/paw's class.
- **Latency** — the choice-question suites sit at 330–456 ms p50 (vs b77's 3368/typed's 1690: those two carry
  per-case multi-question criteria scoring, the heavy path). All local-4-bit-on-M3 loopback HTTP — never
  pooled with hosted rows (the lane's TIMING_METHOD string).

## What this completes

- Plan 011 C1's dataset-suite wording: the clef lane index = the 9-suite comparison population, all with
  quotable latency (`areas.timing.clef` goes 2/0/12-rest → 9/0/0 over its index).
- The crosswalk's TL;DR card + table stay b77+typed (the digest-pin requirement — the crosswalk is the
  C2-pin population, not every clef cell).
