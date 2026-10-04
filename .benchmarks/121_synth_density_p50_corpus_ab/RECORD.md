# Bench 121 — the Issue-064 p50 density-gated artifact: every gate PASSES, at a 7× weaker lift than ungated

**Status:** COMPLETE — measured 2026-10-04 (m3, host `m3`, datasets `.raw/datasets_t20k`,
read @ `69cec9e`). The teacher-gated p50 RUN (2.83 h wall, openthai teacher, 2,521 forwards)
+ the V5 corpus-ab + both echo gates over the sealed artifact
`.raw/corpus_synth_dens50/massive_intent_en_synth.jsonl` (2,06,914 B, blake3
`aede3e556dd418d5124dbd5a114d2fb4ef431b4aaedff62a792def259047ca50`).
Comparator: the UNGATED seated artifact (Bench 0029 lineage, 2,048 rows) re-read the same
morning (`.raw/corpus_synth/corpus_ab.json`, 06:26, same binary era) — the dose-response
pair the issue pre-registered as "p75 as the A/B arm" completes with the p75 run (launched
immediately after this record; its record follows as its own bench).

## The question (Issue 064 task 2, the p50 arm)

Does gating the synth corpus by the minimal-deviation vMF density rule at the label's own
control-p50 |Δ| (Bench 120's instrument, DRY — one density home) still produce a corpus
that (a) lifts the modelless row (V5: paired LB95 > 0), (b) holds BOTH echo gates
(abstention-entropy KL ≤ 0.05; OOD word-dropout ladder, gate rung p=0.20), and (c) how
does the lift compare to the UNGATED artifact at the same budget?

## The run (density-first, the pre-registered order)

| suite | teacher | cands | accepted | vetoed | forwards | p50 ms | s | blake3 |
|---|---|---|---|---|---|---|---|---|
| massive_intent_en | openthai/openthai:openthai-systemone | 212,912 | **1,859** | 662 | 2,521 | 3,940 | 10,177 | `aede3e55…` |

- Budget ≤ 2,048 / 128 per label; the walk accepted **1,859** — allocated was 2,048, the
  forwardable stream ran short under the gate (all 59 labels non-empty, "0 absence(s)").
- **Density-first economics, measured in production**: `density_rejected = 6,982`
  pre-forward rejects at ZERO teacher cost. Under the old post-veto order those would
  each have been a teacher forward ≈ 6,982 × 3.94 s ≈ **7.6 h of teacher time not spent**
  — the reorder's claim (~2.5× at the p50 pass rate) validated on the real run.
- Per-block accept split (the log's every-200 lines — block 1 climbed from 41.5% as the
  walk left the head, mid-run blocks ~71–84.5%, blend 1,859/2,521 = 73.7%): the walk's
  forwardable population is DENSER than the pilot's global candidate set (38.3% at the
  same quantile over all 212,912 cands) — different populations, both honest, the record
  carries both.
- Teacher veto rejected 662 of the 2,521 forwards (26.3%).

## The readings

### V5 (the frozen single test read; both arms reported)

| arm | acc | latency p50/p99 | note |
|---|---|---|---|
| gold (11,314 docs) | 0.7800 | 113/158 µs | reproduces the published 0.78 exactly — instrument alive |
| +synth p50 (13,173 docs) | **0.8000** | 142/217 µs | paired LB95 **+0.0016** → **V5 PASS** (marginal) |

Flips: 7 synth-only wins, 1 gold-only (`massive_intent_en:155`). Determinism true.

### Echo gates — BOTH PASS

- **Abstention-entropy**: KL(gold‖synth) **0.0000**, reverse 0.0000 (≤ 0.05) — PASS.
  Abstain 0.9367 → 0.8833 (disclosure only; the first-reading law — corpus growth raises
  answer confidence by design).
- **OOD word-dropout ladder** (gate rung p=0.20):

| p | acc gold | acc synth | Δ | paired LB95 |
|---|---|---|---|---|
| 0.10 | 0.7567 | 0.7800 | +0.0233 | +0.0062 |
| **0.20 (gate)** | 0.7567 | 0.7833 | +0.0267 | **+0.0042** |
| 0.30 | 0.6767 | 0.6967 | +0.0200 | +0.0016 |

  Gate rung LB95 ≥ 0 → **transfer-ok**, retention @ gate **1.0**, no echo verdict fired.

## The dose-response pair (the honest headline)

| metric | UNGATED (2,048 rows, 06:26 re-read) | **p50-gated (1,859 rows)** |
|---|---|---|
| synth acc | 0.8133 | **0.8000** |
| paired LB95 | +0.0110 | **+0.0016** (7× weaker) |
| V5 | PASS | PASS |
| OOD gate-rung LB95 | +0.0130 | +0.0042 |
| abstain (synth) | 0.8767 | 0.8833 |
| flips ± | 11/1 | 7/1 |

**Reading:** the p50 gate PASSES every gate — the transfer is genuine (retention 1.0 at
the corrupted rung; not an echo) — but the lift is **~7× weaker than the ungated
artifact's** on this suite. Two confounds separate only with the p75 arm: fewer rows
(1,859 vs 2,048 — the gate starved the walk) and different selection (more
central/minimal-deviation rows). If p75 (closer to budget, wider band) recovers the
ungated lift, the gate's cost is mostly QUANTITY; if p75 also sits near +0.002, the
minimal-deviation selection itself does not add lift for massive_intent_en — a recorded
negative for the fusion at this suite, with every gate green on both sides.

## Verdict

- Kill gate (Bench 120): already PASS; this run adds the production-economics proof.
- **p50 artifact: V5 PASS (marginal) · abstention PASS · OOD transfer-ok — the lane's
  gates hold under the density gate.** The lift-vs-ungated question stays OPEN pending
  the p75 A/B arm (in flight; `.raw/corpus_synth_dens75/`, log `/tmp/synth_dens75.log`).
- The SEATED production corpus is UNCHANGED by this record (the ungated artifact stays
  seated; a seat change would require beating it, which p50 does not).

## Provenance

- Run: `/tmp/reflex-echo-gates/release/harness --synth-corpus --synth-teacher openthai
  --synth-density-gate p50 --synth-out .raw/corpus_synth_dens50 --suites
  massive_intent_en --datasets-dir .raw/datasets_t20k` (06:31–09:21 +07, pid 63012;
  teacher pid 62976, openthai fp32).
- Corpus-ab: the same prebuilt binary, `--corpus-ab .raw/corpus_synth_dens50/
  massive_intent_en_synth.jsonl --datasets-dir .raw/datasets_t20k --synth-out
  .raw/corpus_synth_dens50` (⚠ `--synth-out` REQUIRED — outputs default to the UNGATED
  artifact's dir; corrected in the issue's pickup protocol this session).
- Box state: M3 Max, AC, idle apart from the run itself; 82 GiB free.
