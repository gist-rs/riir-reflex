# Bench 114 — Plan 010 T7: the S1MB lane record — the three serving lanes on the System One Mosaic Benchmark, our metric

**Status:** COMPLETE 2026-10-03 — plan 010 executed end to end (owner directive
2026-10-02). Reflex **0.2296 / 0.7055 / 0.4967** (choice / noul / score) ·
Instinct serves **A0 on all three** (no specialist arm cleared its gate) ·
Rethink encoder head **EARN-NO** (three recipes; probes read-reserved) · laya
(round-1, zero-shot english) **0.6964 / 0.2590 / absent** (the 842-option
choice suite exceeds the encoder head budget — disclosed). Site section:
reflex.gist.rs/bench/#s1mb (after #timing). Full disclosure below.

## Protocol (plan 010, recorded and disclosed everywhere)

- **Our metric, never theirs**: forced-pick accuracy per the harness
  conventions on the converter's deterministic 50/50 corpus/test halves
  (`scripts/s1mb_fetch_convert.py`, reflex `c6cc1e7` — choice 4330/4330,
  noul 6174/6173, score 2574/2574 after dedup). S1MB's own leaderboard
  score is baseline-adjusted skill — a DIFFERENT scale, never mixed here
  and not published by the section.
- All lanes read the SAME halves. One judgment = one single-question case
  (26,261 rows), `group` field rides every row (domain vs the six
  `s1mb-generalization-*` subsets).
- **bekko home-field disclosure**: the S1MB domain subsets come from the
  dataset families the bekko teacher trained on, and the `laya__` subsets
  are our own converted families — the domain-vs-generalization split
  below is the required reading for exactly this reason.

## The Reflex row (modelless lane, this record's tables)

Run: `./target/release/harness --skip-laya --datasets-dir .raw/datasets_s1mb
--suites s1mb_choice,s1mb_noul,s1mb_score --out
.benchmarks/114_s1mb_lane_tables` at reflex `d009604`, m3-max-metal,
release. **Accuracy reproduced the plan-T3 read EXACTLY** (0.2296 / 0.7055 /
0.4967 — the modelless lane is deterministic; the A0 pin in the instinct
record pins the same numbers through reflex's own seat).

| suite | n | acc | p50 | det |
|---|---|---|---|---|
| s1mb_choice | 4329 | **0.2296** | 0.414 ms | ✓ |
| s1mb_noul | 6173 | **0.7055** | 0.314 ms | ✓ |
| s1mb_score | 2571 | **0.4967** | 0.266 ms | ✓ |

Box state (Issue 021): power AC · mode high · load 6.23–6.5 (> 6 — a sibling
job on the box) → **latency NOT QUOTABLE**; the cells publish accuracy-only
(the site strip). The plan-T3 preflight (`power=AC load=3.99 canary` clean)
remains the quotable-latency record for these same accuracy numbers: noul
0.292 ms · score 0.243 ms · choice 0.399 ms p50. The 464 starved-key
self-doc fallbacks on choice (842 domains vs cap-16 pool breadth) are the
choice suite's disclosed posture, unchanged from T3.

G1 (harness readout ECE): noul **PASS** (raw 0.1098 · calibrated 0.0061 ·
floor 0.0614); choice/score FAIL (disclosed, no serving claim rides them).

## The Instinct row (hybrid lane — instinct `.benchmarks/0057_s1mb_hybrid/`)

The arena over the three suites (`--datasets-dir ../riir-reflex/.raw/
datasets_s1mb`, winners `data/demo_specialists/s1mb_{choice,noul}_winner_v1`
— the plan-T4 gold-only `instinct_arm_b --arm-a-only` artifacts; NO score
winner exists → a0-only there). Re-run with the record's case-id freeze
(see the split below) — **every arm's test read reproduced byte-exactly**:
choice A0 0.2296 · A1 0.3024 · H1 0.2885; noul A0 0.7055 · A1 0.7024; score
A0 0.4967.

Served verdicts: **a0_stands on all three** — noul's instrument pick A1
refused by the superiority gate (paired LB95 −0.0169); choice's instrument
registered A0 outright (A0 wins the CAL front 0.1550 vs 0.1250 even though
A1's test read is +7.3 pt — the gate reads the frozen test pair only when
the instrument picks a specialist; disclosed, not hidden); score has no
specialist. The full gate tables (G1/G2/G4/G0/G6) ride the instinct record.

## The Rethink row (encoder lane — EARN-NO, riir-train Issue 607)

Three v2 per-option-head recipes over the laya-typed backbone trained on
the corpus half (`.raw/s1mb/` — cases via `riir-train
scripts/plan010_make_s1mb_cases.py`, LENC caches + probe artifacts
blake3-sidecarred): **default, h256×30-epoch, per-suite — all failed the
earn bar** against the backbone's own zero-shot reference logits (disclosed
diagnostics: noul 0.44 vs ref 0.58 · score 0.21 vs 0.29 · choice 0.11 vs
0.39; the h256 retry 0.4387 vs 0.58). Per the read-reserved law the probes
are NOT seated: no artifact, no arena replay, no site cell. The lane's
site row shows the serving tier's fallback answer (marked, per the
fallback law) with the gap note — never a padded number.

**The encoder family's our-metric reading exists via the laya (round-1)
zero-shot lane** — reflex Bench 111 (`.benchmarks/111_plan435_s1mb_round1_tables/`,
4090-win): noul 0.6964 · score 0.2590 · choice ABSENT (the encoder head
budget refuses the 842-option unions — `options exceed the head budget:
151 rendered, 126 markers survived`). That lane's rows ride the site
section as "laya (rust)". Bench 622's round-2 numbers (task avg 55.82, gen
38.42) are the S1MB AUTHORS' evaluator at the skill metric — a different
scale, quoted in riir-train Bench 622 only, never on this board.

## Domain vs generalization (the required split; Issue 607 fact #2 on our metric)

`group_split.json` — computed by joining the instinct record's frozen
`case_ids` (this landing adds the id freeze to the arena's predictions)
to the converter rows' `group` field; every id joined (0 unjoined);
weighted blend == the overall accuraries exactly. A0 = the served row:

| suite | A0 domain | A0 generalization |
|---|---|---|
| s1mb_choice | 0.2305 (n=4221) | 0.1944 (n=108) |
| s1mb_noul | 0.7091 (n=6085) | 0.4545 (n=88) |
| s1mb_score | 0.5081 (n=2474) | 0.2062 (n=97) |

Reading: the modelless lane ALSO drops hard on the six generalization
subsets (−0.04 … −0.30 per suite; 293 of 13,073 questions) — the
"generalization is the hard axis" posture is not encoder-specific. On the
encoder side, Bench 622 (their metric) reads gen 38.42 vs domain-heavy
task avg 55.82 — the same direction. The laya__-subset home-field caveat
applies to the domain half of every lane's numbers.

## Site section (T6)

reflex.gist.rs/bench/#s1mb — a derived `d["s1mb"]` block in data/bench.json
(publish_bench.py `compute_s1mb`, rendered after #timing): one row per lane
× 3 suites + unweighted avg + per-lane disclosure; missing cells render as
gaps (the laya choice gap and the Rethink fallback carry their reasons).
Published from this record + the instinct doc (build_hybrid_doc) + the two
Bench-111 laya docs, acc-only for the loaded-box lanes. The publisher's
`stamp_cell` now refuses to stamp a latency verdict onto a timing-stripped
cell (the verdict describes timing that is not there).

## What changed for this record

- reflex: this record + tables; plan 010 checkboxes; nothing served changes
  (the encoder row is record-only; the hybrid row informational — plan 010
  has no T2-style superiority gate by design).
- instinct: the arena freezes `case_ids` for the s1mb lanes (predictions
  schema extension; the frozen suites stay Null — the Bench-001 trim law).
- riir-train: the s1mb Arm-A path (`--arm-a-only`, presented-union class
  space, presented-constrained holdout read) + `scripts/plan010_make_s1mb_cases.py`
  tracked; Issue 607 carries the our-lanes verdict.
- reflex-site: publish_bench.py `compute_s1mb` + the #s1mb section +
  the acc-only verdict fix; the bench.json s1mb rows + block.

Cross-refs: plan 010 · instinct Bench 0057 · riir-train Issue 607 (the
intel issue; the repro scope stays deferred) + Bench 622 · reflex Bench 111
(the laya our-metric s1mb rows) · reflex-site publish_bench.py.
