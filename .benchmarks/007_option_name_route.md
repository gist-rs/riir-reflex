# Bench 007 — Issue 023: option → domain by NAME (the centroid signal the sampled-distractor suites never saw)

**Status:** COMPLETE 2026-09-24 · base `d9c8ab9`, fix = the Issue 023 landing
commit · M3 (macOS 26.6.2, release, AC power, loadavg 6.5–11.2 — sibling
agent sessions active; latency figures are paired ratios on that box, never
absolutes) · modelless lane only (`--skip-laya`) · fixtures `.raw/datasets/`
· DETERMINISTIC: every accuracy/calibration field below is a pure function
of (binary, fixtures).

## 1. Accuracy — full 14-suite run, base vs fix

| suite | base acc | fix acc | G1 base → fix | everything else (non-latency) |
|---|---|---|---|---|
| **massive_intent_en** | **0.0767** | **0.6900** | fail → **pass** | macro-F1 0.075 → 0.691; readout ECE 0.034 vs floor 0.489 → 0.075 vs floor 0.085 |
| banking77 | 0.4460 | 0.4460 | pass → pass | test eval identical; **calibration repaired** (below) |
| code_fixtures | 0.2917 | 0.2917 | pass → pass | test eval identical; cal-slice readout ECE 0.125 → 0.111 |
| typed_decisions, ag_news, emotion, sst5, prompt_injections, xnli_en, harness_visibility / permissions / tool_fit / routing / sensitivity | — | — | unchanged | **byte-identical** results.json record outside latency |

laya best on massive is 0.7500 — the modelless gap there goes from 67.3 pt
to **6.0 pt**. harness_cache_reuse is LLM-lane only (absent both runs).

**banking77 calibration.** The cal cases are built from train rows whose
label order is first-appearance, not the domain order, so the legacy index
pairing mis-assigned centroids on the CAL slice only: cal target accuracy
**0.007**, calibrated gate abstain rate **1.000** (selective_n 0). Fixed:
target 0.243, abstain **0.23**, selective accuracy **0.532** over 385
questions; readout ECE 0.364 → 0.181 (floor 0.441). The raw gate still
abstains at 100% on the test split — the distance axis, untouched here.

## 2. Latency — paired, alternating, 3 rounds (base, fix, base, fix, …)

| suite | base p50 ms (3 runs) | fix p50 ms | median paired ratio |
|---|---|---|---|
| typed_decisions | 0.729 · 0.698 · 0.525 | 0.690 · 0.637 · 0.549 | 0.947 |
| massive_intent_en | 0.106 · 0.104 · 0.089 | 0.118 · 0.093 · 0.091 | 1.022 |
| banking77 | 0.416 · 0.440 · 0.363 | 0.421 · 0.361 · 0.444 | 1.012 |
| ag_news | 0.199 · 0.195 · 0.167 | 0.202 · 0.171 · 0.149 | 0.892 |

Parity within the box's run-to-run spread (≥ ±20% per arm). The name map is
≤ k·N byte compares on the stack; the guard's G2 (p99 52 µs ≤ 1 ms per
decision set) and G4 (0 allocs post-warmup, canary live) both PASS.

## 3. Cap re-selection under the fix (`--cal-select-cap`, Bench 004's instrument)

| suite | sel-slice acc, base (cap 8…512) | fix | selected | test at selected |
|---|---|---|---|---|
| ag_news | 0.36 … 0.48 | identical | 64 (= registry) | 0.5100 (= default) |
| banking77 | **0.035 – 0.080** | 0.45 – 0.515 | 256 (registry 40) | 0.4560 (default 0.4460) |
| massive_intent_en | **0.040 – 0.050** | 0.495 – 0.560 | 256 (registry 48) | 0.7067 (default 0.6900) |

The base selection slices read near chance: Bench 004's banking77 refusal
was adjudicated on a misaligned slice (addendum there). Under the fix the
selection transfers, but **promotion is declined**: banking77 buys +1.0 pt
for 2× p50 (0.40 → 0.81 ms), massive +1.7 pt = 5 questions of 300, inside
noise. Registry caps unchanged.

## Reproduce

```bash
cargo run --release --bin harness -- --skip-laya --out /tmp/fix        # section 1
cargo run --release --bin harness -- --skip-laya --suites banking77,massive_intent_en,ag_news --cal-select-cap
# flag-off posture: EngineConfig { option_name_route: false, .. } (unit-pinned)
```
