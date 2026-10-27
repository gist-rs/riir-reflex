# Bench 168 — emotion_syn baseline (the ESC re-source emotion lane, plan 013 T5)

**Status:** RECORD — the suite's modelless-floor cell; the rethink-side ESC
re-fit + GOAT is plan 013 T6 (riir-train re-fit hand-off, routing-restricted
to its own lane).

## What ran

First harness measurement of the `emotion_syn` suite (plan 013's first-party
synthetic emotion suite: generator A Ternary-Bonsai-2-27B mlx-2bit over
prompt family A for train, generator B Qwen3.5-4B-4bit over disjoint family
B for test, every row dual-model confirmed by the OTHER model's zero-shot
first-token logprob read). Modelless lane ONLY (`--skip-laya`): no
specialist exists yet — the bag/encoder re-fit + ESC GOAT are T6's product.

```
cargo run --release --bin harness -- --skip-laya --suites emotion_syn \
  --out .benchmarks/168_emotion_syn_baseline
```

Posture: registry defaults — test 400 (stratified round-robin over the whole
1,200-row split), cal 200, corpus cap 64/label, pool floor 3,680 (measured
pool 4,600 = 4,800 train − 200 cal).

## T4 provenance (plan 013's record requirement — digests + veto rates)

Generation verified 2026-10-28 before this bench: all 60 page digests match
`manifest.json` exactly (sha256, re-computed); train 4,800 rows at exactly
800/label, test 1,200 at 200/label; 0 within-split dupes, 0 cross-split
overlap (manifest `cross_split_blocked: 4800`), 0 rows outside the 5–45
word band, no partial pages.

Judge veto (agree_rate, train — Bonsai→Qwen judge): sadness 0.8372 · joy
0.9571 · love 0.6001 · anger 0.7061 · fear 0.7920 · surprise 0.5513. Test
(Qwen→Bonsai judge): 0.8712 / 0.9071 / 0.8306 / 0.7605 / 0.9760 / 0.7380 —
the veto is load-bearing in both directions (the pilot's love 0.47 /
surprise 0.36 rates were the motivation). Split digests: test 400
`fnv1a64-f7bbe4d97652f51b` · cal 200 `fnv1a64-b3ac6e5adbbfac0d` · pool
4,600 `fnv1a64-5f1546c854d4b0d3` · corpus `fnv1a64-0b13e56b64652578` ·
cases `fnv1a64-0f685b0ecd299ae7`.

## Cells (results.json)

| cell | value |
|---|---|
| **A0 modelless acc** | **0.3125** (400 cases) — 1.88× the 6-way chance 0.1667 (jdi_skill 0.1742 over chance 0.1675) |
| macro F1 | 0.2619 |
| ECE(maxp) | 0.1381 (raw readout ECE 0.3120 → calibrated 0.0258) |
| Brier / NLL | 0.8287 / 1.7783 |
| acc@50 coverage | 0.385 |
| **G1 (readout ECE vs conformal-naive floor)** | **PASS** — calibrated 0.0258 < floor 0.3671, and < raw 0.3120 |
| abstain raw / calibrated | 1.00 (sel n/a @ 0) / 0.5275 (sel-acc 0.3016 @ 189) |
| gate-fit (ρ=.30) | score 0.2555 / distance 0.3687 (n 200) |
| p50 / p99 latency | 0.079 / 0.089 ms (tail support 5) |
| determinism | ✓ (10-case re-run bit-identity) |
| modal floor (issue-094) | 0.1775 — A0 is 1.76× it, real input-keyed signal |
| **A1 / H2** | **did not run** — no emotion_syn specialist/fusion posture exists; that cell is T6's ESC re-fit in riir-rethink |

## Honest caveats

- **joy absorbs errors.** The top confusion pairs all land on
  gold-sadness→joy (35, 12.7% of errors), gold-surprise→joy (29),
  gold-anger→joy (21) and joy→love (11): the corpus's joy docs are the
  attractor. This is the honest modelless floor for the GOAT, not a defect —
  exactly the shape the A3 holdout design (disjoint family B + a different
  generator for test) is there to expose if it were generator style rather
  than corpus geometry; the family-B test rows still read 0.3125, so the
  floor is not a family-A artifact.
- The distances: pool 4,600 vs floor 3,680 — 80% slice_guard satisfied with
  margin; slice-integrity audit clean (pool∩test hard check, zero
  violations).
- sst5_syn (plan 013 T1b's suite) generated on the same wave (51 files on
  disk); its licence row + A0 follow this suite's T5 per the T1b note — not
  part of this record.

## Provenance

`PROVENANCE: power=AC Power load=2.70 swap=15.94M canary=skipped
powermode=0(auto)` — preflight PASSED, `latency_quotable: true` both
starts and ends (box_state load 2.73 flat; the sibling league measurement
is GPU-bound on the same box and does not touch this CPU-only lane). Run at
reflex `ed0ffd6`, 2026-10-27T20:03Z, host amo, release profile.
