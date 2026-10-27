# Bench 166 — Plan 012 R2 adjudication: the OPTION_SORT fresh read is NEUTRAL on banking77 (FLAT, H-A refuted for banking77), emotion sorted read NEGATIVE-WEAK — B0+B4 fire (record + the calibration lever, doc-only); B1/B2/B3/B5/B6 do not

**Status:** RECORD — executed 2026-10-27 (amo, M5 Ultra), commit `<this-commit>`. Consumes ONLY the R1 artifacts (`.benchmarks/opendecider_r1_optionsort/`, run sha `8297345`, 2026-10-27T04:50–04:52Z); zero new reads, zero GPU. Adjudication of [rethink Plan 012](../../riir-rethink/.plans/012_r2_inverted_family_split.md) — pre-registered 2026-10-26 BEFORE the read landed (gates-fixed-before-data).

## Provenance

- R1 fresh read: `OPENDECIDER_OPTION_SORT=1`, opendecider 0.9.0 / torch 2.14.1 / transformers 5.20.2 (the rebuilt, version-pinned venv — NOT the byte-identical seal1 venv; 034's standing disclosure), nano on MPS, localhost :8000, 9 suite legs + the perm-probe, all rc=0, per-suite det ✓ 10/10.
- ⛔ First watcher fire (04:23Z) DIED rc=127 on every leg — launchd does not source `~/.cargo/bin`; `run_r1_read.sh` printed "RUN COMPLETE" over nine failed legs (rc unchecked). FIXED in the rig same session (PATH guard + fail-fast leg accounting); the successful re-fire is the only read. The R1 harness build stamp: `8297345`.
- Box: power=AC; load 14.8 → 10.7 (sibling jobs on the box — 645/469 train arms). Every number below is an ACCURACY; no latency is quoted (the tables' own NOT-QUOTABLE marker stands).

## I1 — the R1 sorted cells vs bench 163 (unsorted incumbent, frozen read)

| suite | n | bench163 | R1 SORTED | Δ pt | band | card anchor |
|---|---:|---:|---:|---:|---|---:|
| typed_decisions (canary) | 2000 | 0.7850 | **0.7850** | **0.00** | drift canary SETTLED — |Δ| ≤ 0.5, no posture drift | — |
| ag_news | 400 | 0.8100 | 0.8025 | −0.75 | NEGATIVE-WEAK (reported) | 0.73 |
| **banking77** | 500 | 0.6340 | **0.6340** | **0.00** | **FLAT** | 0.645 (ours −0.011 under) |
| **emotion** | 400 | 0.5700 | **0.5625** | **−0.75** | **NEGATIVE-WEAK** | — |
| massive_intent_en | 300 | 0.7933 | 0.7933 | 0.00 | reported (coarse n) | — |
| prompt_injections | 116 | 0.5172 | 0.5172 | 0.00 | reported (coarse n) | — |
| sst5 | 600 | 0.5550 | 0.5550 | 0.00 | reported | — |
| winogrande_jdi | 1267 | 0.5067 | 0.5067 | 0.00 | reported | — |
| xnli_en | 300 | 0.8300 | 0.8200 | −1.00 | NEGATIVE-WEAK (reported) | — |

Typed Δ = 0.0000 EXACTLY (pooled column keeps R1's own ±0.5 paired-posture expectation) — the rebuilt venv reconstructs the incumbent posture faithfully; branches adjudicate on clean ground.

## I2 — H-C concentration (banking77, 77-class): NOT CONFIRMED

Top-10 error classes carry **57/183 = 31.1%** of error mass — below the pinned 40% bar (~3× the uniform 13% expectation). Errors are SPREAD across the label space, not concentrated in jargon-heavy classes; top confusion pairs max out at 5 occurrences (18→76, 50→22, 39→22, 44→39). H-C NOT confirmed → **B3 does not fire** (plan law: reported descriptively, no branch keys on it). Emotion's 6-class confusion is descriptive only: the mass concentrates on label-0 pulls (3→0 ×26, 1→0 ×24, 4→0 ×20 — the neutral-class gravity), no bar applied.

## I3 — flip-vs-deficit: order-sensitivity does NOT explain the deficit

banking77 carries the board's worst flip rate (12.96% at 163) and the widest max TV (64.87) — and its sorted cell moved 0.00. Emotion carries a baseline-rate flip (4.27%) — and its sorted cell moved −0.75. Across the two branch-bearing suites the flip column does NOT predict the deficit column (the correlation answer is NO); the deficit is distribution-side, not presentation-side.

## I4 — per-family ECE under sort (H-D)

| suite | 163 ECE | R1 SORTED ECE | > 0.15 persisting? |
|---|---:|---:|---|
| typed_decisions | — (unmeasured) | **0.1590** | fold-family, above bar in the sorted posture |
| ag_news | 0.037 | 0.0313 | no |
| banking77 | 0.092 | 0.0923 | no |
| emotion | 0.103 | 0.1057 | no |
| massive_intent_en | 0.076 | 0.0847 | no |
| prompt_injections | 0.347 | 0.3467 | reported — not a fold family, not branch-bearing |
| sst5 | 0.044 | 0.0439 | no |
| winogrande_jdi | 0.153 | 0.1527 | reported — never branch-bearing (chance-adjacent) |
| xnli_en | 0.198 | **0.1976** | fold-family, persists (sort did NOT bring it under) |

The plan's "any family" near-sure-fire reading is avoided per the pinned scoping: the B4 test keys on the 640-T4 fold families (sst5 / xnli / typed) + branch-bearing families. **xnli persists (0.198 → 0.1976) and typed reads above the bar (0.1590)** → B4's trigger is met.

## The perm-probe arm (R1 leg 2 — condition-(c) input)

5 orderings × ≤40 cases/suite, seed 125239297, label-space-invariant comparison: **opendecider-nano 0 flips / 0.00 pt median swing on ALL 6 probed suites — PASS on every cell**; the modelless control PASS (its typed 31 flips are exact-tie artifacts, contract-allowed). Note the probe's 40-case sample vs 163's full-suite 12.96% banking77 flip rate — both facts stand: order-sensitivity exists at full-suite scale and is undetectable at probe scale; condition (c) is resolved by the probe PASS as pinned.

## The decision tree — verdicts

| branch | trigger | verdict |
|---|---|---|
| B0 (default) | no branch-bearing lever arms | **FIRES** — record + hold; the A2 spend gate stays shut |
| B1 | banking77 MATERIAL (> +1.5) | no — Δ 0.00 FLAT; H-A (presentation-side) refuted FOR banking77 |
| B2 | banking77 FLAT **AND** emotion FLAT | **no** — banking77 FLAT ✓ but emotion −0.75 NEGATIVE-WEAK ✗ (the conjunct fails) |
| B3 | H-C ≥ 40% | no — 31.1% |
| B4 | fold/branch family ECE > 0.15 persisting under sort | **FIRES (doc-only)** — calibration lever priced into the T2 loss + per-family temperature notes on the 640-T4 fold, scoped to usable-accuracy suites (typed/ag/banking/massive/xnli; never winogrande) |
| B5 | emotion WEAK-or-better POSITIVE | no — NEGATIVE-WEAK is not positive; recorded |
| B6 | any branch-bearing REGRESSION (< −1.5) | no — emotion −0.75 sits in NEGATIVE-WEAK, not REGRESSION |

**Net: B0 + B4.** The finding: option canonicalization is NEUTRAL on the board (typed/ag/massive/prompt/sst5/winogrande all within a flip of zero; ag/xnli NEGATIVE-WEAK; emotion NEGATIVE-WEAK) — the banking77/emotion deficit is distribution-side (H-B stands, B2's flat-flat conjunct fails only on emotion's small negative), NOT presentation-side, and NOT class-concentrated. Nothing arms but the doc-only calibration lever.

## The Fusion-3 router re-read (carried regardless of branch — feeds 640-T4)

As pre-registered: 013's "BANKING77-strong, AG News-weak" complementarity thesis is REFUTED as filed (the card's own honest-limits read banking 0.645 < AG 0.73 — "banking-strong" was only ever a RELATIVE claim vs Laya-td 0.492). The R1 sorted read does not change the complementarity map. The nano column for the 640-T4 router fold stays NARROW: **sst5** (0.5550 — the genuine cell: +13.33 vs served A1 0.4217, +2.83 vs the ENC record column 0.5267, the only cell winning BOTH comparisons) and **xnli** (deployed-arm-only: 0.8200 vs A0 0.5233 — BUT the ENC record 0.8600 beats nano by 4.0 pt at the encoder rung nano would actually join; never quoted without the ENC caveat), never a board-wide column, never pooled with order-invariant lanes without the bench-163 §5(c) disclosure. The B4 temperature notes ride this same fold.

## Artifacts

- `.benchmarks/opendecider_r1_optionsort/<suite>/` — results.json + TABLES.md + items/<suite>.jsonl (the `--dump-items` arm; banking77 1000 / emotion 800 / typed 4000 item lines)
- `.benchmarks/opendecider_r1_optionsort/perm_probe/` — PERM_PROBE.md + json
- `.benchmarks/opendecider_r1_optionsort/r2_items_analysis.txt` — the I2/I4 instrument output (staged analyzer, fixture-tested pre-data per plan 012 T1)
- Rig fix this session: `run_r1_read.sh` PATH guard + leg-rc fail-fast (gitignored rig; the 04:23Z rc=127 incident recorded above)
