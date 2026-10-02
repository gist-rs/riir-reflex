# Bench 104 — the bekko distill closes (riir-train Issue 608): the teacher arm lands (T1), the RIDT dumps land (T2), the T2 specialist gates MISS (instinct Bench 0050), and the bekko-veto synth corpus lifts but does not beat the incumbent (Path 2)

**Status: T1 LANDED (`20527e6` — `--distill-teacher bekko`, the `JsonlOracle`
shared wire, the RIDT license field) · T2 LANDED (massive 11,514 rows /
teacher(train) 0.9244; xnli 20,000 / 0.6668) · T3 NEGATIVE (both T2 gates miss —
massive wash at the serving arm, xnli 12.3 pt below; instinct Bench 0050,
`3478bbd`) · PATH 2 NEGATIVE (the bekko-veto corpus lifts 0.7800 → 0.8067,
LB95 +0.0062 — V5 PASS but BELOW the incumbent openthai corpus's 0.8133 /
+0.0110; no adoption). Nothing served changes anywhere.**

## Question (pre-registered, riir-train Issue 608)

Bench 103 measured bekko-68m strong on exactly the two suites with distill
headroom (massive +8.7 pt, xnli +15.3 pt over the modelless lane) and the owner
asked whether the teacher is worth distilling from. The lane: teacher pluggability
→ RIDT dumps over the train rows → the student retrain + the T2 gates (Path 1),
and the synth-corpus bekko-veto (Path 2 — the axis that actually moved the
serving cell once already: the openthai veto's corpus lifted massive A0
0.7800 → 0.8133).

## Path 1 — the specialist artifact (the T3 gate; full record: instinct Bench 0050)

1. **Teacher dumps** (`--distill --distill-teacher bekko`, CPU FP32, preflight
   `power=AC Power load=3.66 swap=2611.25M canary=129.4us/best5 powermode=2(high)`):
   massive 11,514 rows / 60 classes / teacher(train) **0.9244** (p50 63 ms,
   739 s) · xnli 20,000 / 3 / **0.6668** (p50 42 ms, 851 s). RIDT headers carry
   `license: MIT (hotchpotch/bekko-system-one — Copyright (c) 2026 Yuichi
   Tatsumi; verified 2026-10-02)` — the attribution law riding the artifact.
   Seals `7b2a9ad88bc047bf…` / `bb29de6f3aa7f36d…`.
2. **Student retrain** (riir-train `instinct_arm_b`): both winners are
   B(gold_mix=0) — the pure-distill arm BEAT gold train-side (massive 0.6850 vs
   A 0.6600; xnli 0.3700 vs 0.3650). The distillation works; every winner beat
   its lr=0 control. Artifacts: `data/instinct_specialists_bekko68m/` (massive
   blake3 `a0310b4bb44747b1…`, xnli `6aff5eb78ba8f173…`) — never the serving dir.
3. **The single test reads** (instinct Bench 0050, the 0029 protocol,
   `--winners-dir` the only change): **massive — bekko-H2 0.8400 ties the served
   cell, but the paired verdict vs the incumbent-H2 picks is mean +0.0000
   (10 wins / 10 losses / 280 ties): a WASH at the serving arm** (the +1.0 pt
   A1 edge, 0.8267 vs 0.8167, is uncertified and absorbed by the fusion — the
   modelless engine dominates the H2 ordering); **xnli — bekko-A1 0.4000 vs the
   served A0 0.5233, the instrument registered A0 outright** (A0 pin held:
   arena == reflex run() == 0.5233, site ✓). Both T2 gates MISS — negatives
   recorded, the manifest byte-untouched, the "On PASS" task (arsenal row +
   serve parity + deploy.yaml) not exercised.

## Path 2 — the synth-corpus bekko-veto (this repo's lane)

The synth lane's veto teacher is pluggable by the SAME seam T1 landed
(`--synth-teacher bekko` — `construct_teacher` shared, zero extra code). Same
generation law as the incumbent corpus (same 212,912-candidate pool, same
per-intent E0 weighting, same caps ≤2048/128, span ≤4) — ONLY the veto teacher
differs:

| corpus | teacher | accepted / vetoed / forwards | teacher p50 | wall | digest |
|---|---|---|---|---|---|
| incumbent | openthai/openthai-systemone | 2048 / 1475 / 3523 | 4070 ms | 14,278 s (4090) | `8fe10f8fabde6471…` |
| this run | bekko/bekko-system-one-v0-68m | 1975 / 2314 / 4289 | **101 ms** | **455 s (M3)** | `63a459be5fc96cc7…` |

(The M3+bekko veto is ~30× faster than the openthai-HTTP route — the cost
shape of a re-veto is now trivial; the DECISION cost is the test read, not the
compute.)

**The V5 gate** (`--corpus-ab`, one frozen massive test read, the 0029/091
protocol):

| corpus | arm A (gold) | arm B (+synth) | paired LB95 | flips |
|---|---|---|---|---|
| incumbent openthai (bench 091) | 0.7800 | **0.8133** | +0.0110 | — |
| **bekko (this read)** | 0.7800 ✓ anchor | **0.8067** | **+0.0062** | 9 synth-only / 1 gold-only |

- Arm A reproduces the published 0.78 EXACTLY — the instrument is alive.
- V5 PASS: the bekko corpus genuinely lifts the modelless row (LB95 > 0, +12
  net questions).
- **The adoption gate FAILS: 0.8067 < 0.8133** — the incumbent openthai corpus
  stays seated; the bekko corpus is not adopted. No seat/H2 read follows (the
  corpus loses on its own gate; the serving arm is engine-dominated per Path
  1's wash, so a corpus swap needs to beat 0.8133 outright to move the cell).

Per-label flips: gains on audio_volume_other +3, audio_volume_mute +2,
audio_volume_down +1, iot_hue_lighton +1, music_likeness +1, qa_maths +1;
loss news_query −1 — the bekko veto's accepted set skews audio/qa where the
openthai set presumably covered differently.

## The adjudication

1. **The bekko distill lane is CLOSED with two measured negatives** (specialist
   wash/below; corpus lifts-but-loses) and **two landed assets**: the teacher
   pluggability seam (`20527e6` — bekko joins laya/openthai in `--distill-teacher`
   AND `--synth-teacher`), and the license/attribution machinery (the RIDT
   header field). Bench 103's per-suite wins do not transfer through either
   distill axis on these suites.
2. **What the pair of negatives MEANS together:** bekko's edge is real but
   thin-on-this-board — as a SPECIALIST teacher it lands within the H2 fusion's
   noise floor (engine-dominated), and as a VETO teacher its acceptance profile
   produces a slightly weaker corpus than the 0.92-qualified openthai. The 60-class
   massive board is where the wash lives: the serving arm never needed a better
   specialist.
3. **xnli's +15.3 pt gap stays OPEN** — the largest board gap remains unclaimed
   (the teacher measures 0.6767 on it; the bag student reaches 0.4000). The
   recorded lever for that gap is now ONLY Issue 607's data-breadth laya round
   (owner-gated) — or a fundamentally different student class; the bag-class
   student is measured-out for xnli (the 579 law: this negative is on record).
4. **Repro (the dumps + the corpus, all CPU):**

```
BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python \
target/release/harness --distill --distill-teacher bekko \
  --suites massive_intent_en,xnli_en --datasets-dir .raw/datasets_t20k \
  --distill-out .raw/distill_teacher_bekko68m
BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python \
target/release/harness --synth-corpus --synth-teacher bekko \
  --suites massive_intent_en --datasets-dir .raw/datasets_t20k \
  --synth-out .raw/corpus_synth_bekko
target/release/harness --corpus-ab .raw/corpus_synth_bekko/massive_intent_en_synth.jsonl \
  --datasets-dir .raw/datasets_t20k --synth-out .raw/corpus_synth_bekko
```

(The student retrain: `cargo run --release -p riir-train-engine --example
instinct_arm_b -- --suites massive_intent_en,xnli_en --teacher-dir
../riir-reflex/.raw/distill_teacher_bekko68m --out
data/instinct_specialists_bekko68m` from ../riir-train.)

## Box state

`PROVENANCE: power=AC Power load=3.66 swap=2611.25M canary=129.4us/best5
powermode=2(high)` (the dumps' preflight; the corpus-ab run carried the same
box state). The paired stats are integer arithmetic on frozen picks —
box-independent.

## Artifacts

- reflex `20527e6` — T1 (the seam; includes the harness CLI, `JsonlOracle`,
  the RIDT license field, the `bekko_lane.py` license-note repair)
- instinct Bench 0050 (`3478bbd`) — the T2 reads + paired verdicts (Path 1's
  record of the massives/xnli test reads)
- `.raw/distill_teacher_bekko68m/` — the RIDT dumps + DISTILL.md (DATA on disk,
  gitignored by law)
- `.raw/corpus_synth_bekko/` — the bekko-veto corpus + SYNTH.md + CORPUS_AB.md
  (the V5 read's raw record; this file is the dated bench record)
- riir-train Issue 608 — the lane's issue, updated with all four task verdicts

## Cross-refs

- Bench 103 — the teacher's measured cells (the why-now)
- instinct Bench 0050 — Path 1's test reads
- Bench 091 / the V5 anchor — the corpus_ab instrument (arm-A 0.78 anchor)
- riir-train Issue 607 — the xnli gap's remaining lever (owner-gated)
