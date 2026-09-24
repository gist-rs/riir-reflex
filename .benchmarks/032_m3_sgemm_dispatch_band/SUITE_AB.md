# T7 suite-p50 A/B — the dispatch band predicate at suite level — **GOAT-PASS, measured 2026-09-25**

**PROVENANCE** (per-run `bench_preflight.sh`): the run rode a box the preflight
had passed minutes before (power=AC, powermode=2, settle 449 min, load 3.20,
swap 1206M, canary 135.5 µs/best5); the sibling `rlf_b615672` ppl bench
**restarted mid-run** — the per-round load trace (below) is load-bearing and
rounds 1–2 are DISCARDED per the T5 confound lesson (reflex issue 020), not
averaged in. Rounds 4–6 are clean (start loads 3.7 / 2.1 / 2.0, all under the
6.0 ceiling); round 3 started at the ceiling edge (6.46) and is reported only
as the incl-r3 row.

## What ran

`/tmp/t7ab/harness_ab_t7.sh` — the frozen A/B from the T7 rung landing:
6 position-balanced rounds × 2 arms × 3 suites
(`massive_intent_en,banking77,code_fixtures`).

- **OLD** = `/tmp/t7ab/release/harness` — substrate worktree @ riir-infer
  `1e8c034` (the m/n-threshold xwide pick), harness @ reflex `3ac8a8a`.
- **NEW** = `/tmp/t7ab-new/release/harness` — the LANDED substrate (the
  single-wave band predicate), same harness code.
- **Isolation check (this run):** `git diff 1e8c034..HEAD` over the laya
  substrate is exactly TWO commits — `1afd4f8` (the band predicate) and
  `5ef7442` (the rope hoist, DEFAULT-OFF ⇒ inert at the A/B's default env);
  `encoder.rs`/`head.rs` are byte-identical between the arms, so T5 packing
  and the T10 softmax rungs are in BOTH. The A/B isolates the band. ✓

## Verdict — clean rounds 4/5/6, per-suite NEW/OLD p50 median

| suite | NEW/OLD p50 | per-round (OLD→NEW) | consistency |
|---|---|---|---|
| massive_intent_en | **0.657** (−34.3%) | 70→48 · 73→48 · 71→44 | 6/6 rounds incl. loud |
| banking77 | **0.709** (−29.1%) | 104→81 · 103→73 · 106→72 | 6/6 rounds |
| code_fixtures | **0.643** (−35.7%) | 112→72 · 112→68 · 120→84 | 6/6 rounds |

Every round, BOTH load classes, same direction — the win survives the loud
rounds (where both arms inflate together), which is the robustness signature
the T5 confound taught us to check. Including round 3 moves the medians to
−32.8 / −25.6 / −33.3% — same verdict.

The band predicate was LANDED on kernel-level rows (Bench 032: o/wo −14…−18%,
wi −10…−22%, head MHA −13…−29%); this row is the suite-level confirmation the
rung's landing required, and it lands ABOVE the kernel rows' range — the old
pick mis-dispatched several GEMM families at once per question (xwide outside
its band on the n ≥ 2048 projections AND wide on the head MHA), so the suite
sums several kernel-level wins.

## Loads per round (start / end, 1-min avg)

```
r1 OLD 11.52→10.66  r1 NEW 10.66→8.93   ← DISCARDED (sibling ppl bench restarted)
r2 NEW  8.93→7.07   r2 OLD 7.07→6.46    ← DISCARDED
r3 OLD  6.46→5.21   r3 NEW 5.21→3.72    ← borderline (start at ceiling edge)
r4 NEW  3.72→3.11   r4 OLD 3.11→2.10
r5 OLD  2.10→2.21   r5 NEW 2.21→2.02
r6 NEW  2.02→1.80   r6 OLD 1.80→1.81
```

Raw per-round results: `/tmp/t7ab/h_out/h_{r}_{arm}/results.json` (ephemeral);
the medians above are the record.
