# Bench 122 — the Issue-064 p75 arm completes the dose-response pair: the gate's cost is QUANTITY, and ungated stays best

**Status:** COMPLETE — measured 2026-10-04 (m3, host `m3`, datasets `.raw/datasets_t20k`,
read @ `16eb594`). The p75 RUN (19:23–22:29 +07, 3.1 h wall, openthai teacher, 2,797
forwards) + the V5 corpus-ab + both echo gates over the sealed artifact
`.raw/corpus_synth_dens75/massive_intent_en_synth.jsonl` (blake3
`0f0890e2853011d218e6b1d913083913c465abc9cfb8332803dd085a7925e8f5`).
Closes the dose-response pair with [Bench 121](../121_synth_density_p50_corpus_ab/RECORD.md)
(p50) and the ungated comparator (same-morning re-read, `.raw/corpus_synth/corpus_ab.json`).

## The question (Bench 121's open confound)

p50 passed every gate at a ~7× weaker lift than ungated. Two candidate causes: row
starvation (1,859 vs 2,048 — the gate starves the walk) vs selection quality (the
minimal-deviation rows are worse for accuracy). p75 (wider band → 1,980 rows, 97% of
ungated's count) separates them.

## The run

| suite | teacher | cands | accepted | vetoed | forwards | p50 ms | s | blake3 |
|---|---|---|---|---|---|---|---|---|
| massive_intent_en | openthai/openthai:openthai-systemone | 212,912 | **1,980** | 817 | 2,797 | 3,918 | 11,167 | `0f0890e2…` |

- `density_rejected = 2,965` pre-forward (vs p50's 6,982 — the wider band rejects less;
  ≈ 3.2 h of teacher calls still saved by density-first).
- **Determinism witness**: the rerun (the morning attempt was interrupted at ~505/2048
  for an owner silent-rig call) reproduced the interrupted run's every-block counters
  EXACTLY (111/200, 254/400, 335/600, 505/800) before diverging into new territory —
  the sealed-artifact discipline's restart is trajectory-identical.
- Per-block accept: early 55–71%, mid 76–85%, tail blocks dipped to ~43% then recovered
  (cap dynamics), blend 1,980/2,797 = 70.8%.

## The readings

### V5 (frozen single test read)

| arm | acc | latency p50/p99 | note |
|---|---|---|---|
| gold (11,314 docs) | 0.7800 | 112/135 µs | published 0.78 reproduced — instrument alive |
| +synth p75 (13,294 docs) | **0.8033** | 113/200 µs | paired LB95 **+0.0062** → **V5 PASS** |

Flips: **7 synth-only wins, 0 gold-only** — the p75 corpus never breaks a case the gold
arm got right (strictly monotone; p50 was 7/1, ungated 11/1). Determinism true.

### Echo gates — BOTH PASS

- **Abstention-entropy**: KL(gold‖synth) **0.0000**, reverse 0.0000 — PASS. Abstain
  0.9367 → 0.8800 (disclosure).
- **OOD word-dropout ladder** (gate rung p=0.20):

| p | acc gold | acc synth | Δ | paired LB95 |
|---|---|---|---|---|
| 0.10 | 0.7567 | 0.7867 | +0.0300 | +0.0107 |
| **0.20 (gate)** | 0.7567 | 0.7933 | +0.0367 | **+0.0117** |
| 0.30 | 0.6767 | 0.6967 | +0.0200 | +0.0041 |

  Gate rung ≥ 0 → **transfer-ok**, retention @ gate **1.0**, no echo.

## The dose-response triplet (the answer)

| metric | UNGATED (2,048 rows) | p50 (1,859) | **p75 (1,980)** |
|---|---|---|---|
| synth acc | **0.8133** | 0.8000 | 0.8033 |
| paired LB95 | **+0.0110** | +0.0016 | +0.0062 |
| OOD gate-rung LB95 | +0.0130 | +0.0042 | **+0.0117** |
| abstain (synth) | 0.8767 | 0.8833 | 0.8800 |
| flips ± | 11/1 | 7/1 | 7/0 |
| V5 / echo | PASS / transfer-ok | PASS / transfer-ok | PASS / transfer-ok |

**Reading — the confound separates:**

1. **The lift tracks row count** (1,859 → +0.0016; 1,980 → +0.0062; 2,048 → +0.0110):
   the p50 deficit was mostly QUANTITY (walk starvation under the stricter band), not
   selection quality. p75 at 97% of ungated's rows recovers 56% of its LB95 and
   essentially all of its OOD gate-rung transfer (+0.0117 vs +0.0130).
2. **The gate does not beat ungated at ANY strength** — ungated wins every accuracy
   cell. The fusion's premise (density-gated selection improves corpus quality for the
   frozen consumer) measures **NO accuracy gain on massive_intent_en**. The gate costs
   rows and buys nothing the echo gates can see: corpus health (KL, retention) is
   IDENTICAL at every strength — the gate neither helps nor hurts health, it only
   shrinks the corpus.
3. **Honest scope**: one suite (the pre-registered V5 board suite), one engine, one
   teacher. A corpus-health-improving gate might still pay on a suite where UNGATED
   synthesis degrades health (an echo-class failure) — none exists today (the ungated
   artifact reads transfer-ok everywhere); the gate is a remedy without a disease.

## Verdict

- p75 artifact: V5 PASS · abstention PASS · OOD transfer-ok — all gates green.
- **The SEATED production corpus stays the UNGATED artifact** (best measured at every
  cell; the gate arms are recorded as the dose-response evidence, not candidates).
- **Issue 064 task 2: COMPLETE** — the lane's question is answered: the minimal-deviation
  density gate works exactly as designed (Bench 120 signature, deterministic, ~free at
  pre-forward) but does not improve downstream accuracy where ungated synthesis already
  transfers cleanly. Recorded as the honest negative-for-the-fusion; the code stays
  (`--synth-density-gate`, the DensityGate instrument, the echo-gate rig — all reused
  by any future gate experiment at zero cost).
- Teacher killed after this record (the pair's verdict makes p90 unnecessary — it would
  only re-approximate ungated at full cost).

## Provenance

- Run: `/tmp/reflex-echo-gates/release/harness --synth-corpus --synth-teacher openthai
  --synth-density-gate p75 --synth-out .raw/corpus_synth_dens75 --suites
  massive_intent_en --datasets-dir .raw/datasets_t20k` (19:23–22:29 +07, pid 34182;
  teacher pid 34156 — rebooted per the recorded resume protocol after the silent-rig
  interruption; the trajectory reproduced the interrupted attempt block-for-block).
- Corpus-ab: same binary, `--corpus-ab …dens75/massive_intent_en_synth.jsonl
  --datasets-dir .raw/datasets_t20k --synth-out .raw/corpus_synth_dens75`.
- Box state: M3 Max, AC, evening (Zed + Chrome only), 49 GiB free.
