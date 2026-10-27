# Bench 169 — sst5_syn baseline (the ESC re-source sentiment lane, plan 013 T1b follow-up)

**Status:** RECORD — the suite's modelless-floor cell; the specialist re-fit
+ ESC GOAT follows the emotion_syn pattern (plan 013 T6's shape, its own
lane).

## What ran

First harness measurement of the `sst5_syn` suite (plan 013 T1b: the
generator generalized to a SPECS table — `sst5_syn` = the five SST-5 levels
over DISJOINT review-domain prompt families A/B, judge answers digits 1..5,
every row dual-model confirmed by the other model's zero-shot read; no SST
text is copied — the five-level scale vocabulary is the only carried
concept). Modelless lane ONLY (`--skip-laya`): no specialist exists yet.

```
cargo run --release --bin harness -- --skip-laya --suites sst5_syn \
  --out .benchmarks/169_sst5_syn_baseline
```

Posture: registry defaults — test 600 (the suite's test_cap; stratified
round-robin over the whole 1,000-row split), cal 200, corpus cap 64/label,
pool floor 3,040 (measured pool 3,800 = 4,000 train − 200 cal).

## Generation provenance (verified 2026-10-28 before this bench)

All 50 page digests (40 train + 10 test) match `manifest.json` exactly
(sha256, re-computed); train 4,000 rows at exactly 800/level, test 1,000 at
200/level; 0 within-split dupes, 0 cross-split overlap, no partial pages.
Train: Bonsai → Qwen judge (seed 1, family A) — veto agree rates: very
negative 0.0711 · negative 0.4797 · neutral 0.4374 · positive 0.5066 ·
very positive 0.8469 (the very-negative train veto is EXTREME — the Qwen
judge rejects nearly all of Bonsai's very-negative generations; the 800
landed rows are the heavily-vetted survivor set, ~11k generated). Test:
Qwen → Bonsai judge (seed 2, family B): 0.9854 / 0.3675 / 0.6871 / 0.3992
/ 0.9349. The A3 holdout property holds: family-B test agree rates are
generally HIGH (different generator+judge orientation), and the A0 below
still reads the family-B rows at the floor level — no family-A style leak
in the floor cell. Split digests: test 600 `fnv1a64-2586b1ddf640f1ba` ·
cal 200 `fnv1a64-9e74e758cf98831d` · pool 3,800 `fnv1a64-e500251313a4357d`.

## Cells (results.json)

| cell | value |
|---|---|
| **A0 modelless acc** | **0.3117** (600 cases) — 5-way chance 0.20; 1.29× the modal floor 0.2417 |
| macro F1 | 0.3070 |
| ECE(maxp) | 0.0984 (raw readout ECE 0.3098 → calibrated 0.0862) |
| Brier / NLL | 0.7914 / 1.5872 |
| acc@50 coverage | 0.340 |
| **G1 (readout ECE vs conformal-naive floor)** | **PASS** — calibrated 0.0862 < floor 0.2453, and < raw 0.3098 |
| abstain raw / calibrated | 1.00 (sel n/a @ 0) / 0.6517 (sel-acc 0.3828 @ 209) |
| p50 / p99 latency | 0.057 / 0.065 ms (tail support 7) |
| determinism | ✓ (10-case re-run bit-identity) |
| modal floor (issue-094) | 0.2417 — A0 is 1.29× it (thinner margin than emotion_syn's 1.76× — disclosed) |
| **A1 / H2** | **did not run** — no sst5_syn specialist/fusion posture exists; that cell follows the T6 pattern in its own lane |

## Honest caveats

- **The graded-scale pull is toward `positive`.** The top confusion pairs
  are very-positive→positive (47), very-negative→positive (46),
  negative→positive (44): the modelless corpus scorer collapses the
  ends of the scale toward the middle-positive band. The extreme ends
  carry the thinnest lexical overlap with the 64-doc/label corpus — the
  very-negative train rows survived an ~0.07 judge agreement, so that
  label's corpus is the most idiosyncratic survivor set. This is the
  honest floor; the specialist GOAT decides against it.
- sst5_syn's licence row + this bench close the T1b follow-ups; T6's
  re-fit hand-off pattern applies per-suite (its own lane).
- The synthetic suites are NOT members of the gate_drift 4-suite family
  (T1b law — a synthetic suite is not a drift-family member).

## Provenance

`PROVENANCE: power=AC Power load=1.79 swap=15.94M canary=skipped
powermode=0(auto)` — preflight PASSED, `latency_quotable: true`. Run at
reflex `1e67222`, 2026-10-27T20:1xZ, host amo, release profile.
