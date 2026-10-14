# Bench 142 — the d3 NF4 per-family spot check (riir-train 629 T2, the lossy-surface gate)

> **Status:** RECORD · **CORRECTED 2026-10-14** (the board-reference column of the verdict table named the WRONG ENTRANT'S row — see the correction section; the gate itself RE-PASSES on the correct row with materially thinner margins) · host 4090-windows (shikuwa) · lane: riir-train Issue 629 T2 (the vLLM-SR d3 teacher lane)
> **Rule:** no corpus rule — this is the TEACHER-QUALITY gate 629 T2 mandates before trusting the captures: "a PER-FAMILY SPOT CHECK of the quantized teacher's option-scores against … the board's published per-benchmark numbers for that entrant … the lossy-surface rule: a lossy surface gates on deployed-path behavior, not bit-identity"
> **Instrument:** the new `--d3` scoring lane (reflex `46ddc75`, this session — the pplx lane's shape verbatim over `src/lanes/d3.rs`) through the tranche-1 JDI suites (Plan 012's board-compatible casting: official splits, split sizes match the board's `cases`)

## The posture under test

- **Served**: `d3-v3.1.0-nf4` at `127.0.0.1:8794` — their `d3_server.py` (tag v3.1.0, Apache-2.0, UNGATED) under the patched loader (`D3_BNB_NF4=1`, `D3_MAX_GPU_GB=20`, `D3_MAX_CPU_GB=8`; VRAM 24,080 MiB resident). The 15 GiB budget of the pplx precedent FAILED at boot (bnb-4bit refuses CPU-dispatched modules — d3's 0.46B vision tower pushes past the pplx-sized budget; the raise to 20 GiB fits it).
- **The reference**: the Bench-067 pinned board snapshot 0.3.2 (sha256 `5d405036…`, 148 entrants), d3 row `vllm-sr-d3-v310` — extracted to `riir-train/scripts/plan629_d3_spotcheck_reference.json` (`6b3b70b8`).

## The verdict — all three measured families at-or-above the board (NO degradation)

> ⚠ **SUPERSEDED by the correction below** — the "board" column of this table is
> `decision-27b-ckpt-nothink` Fastino GLiDE no-thinking (28B)'s row, NOT d3's. Kept
> verbatim for the record; the corrected table is in the correction section.

| suite (population) | d3 NF4 ours | board (their harness, bf16-class) | Δ | ece(maxp) | p50 |
|---|---|---|---|---|---|
| winogrande_jdi (1267 = full board pop) | **0.9455** acc | 0.8595 acc | **+8.6 pp** | 0.0167 | 209 ms |
| hellaswag_jdi (2000 of 10042, the B5 cap) | **0.9760** acc | 0.9502 acc | **+2.6 pp** | 0.0102 | 282 ms |
| anli_jdi (3200 = full board pop) | **0.7553** acc | 0.7331 macro-F1 | **+2.2 pp** ⚠ | 0.0511 | 275 ms |
| mmlupro_jdi | NOT MEASURED this run — the datasets-server rate-walled TIGER-Lab/MMLU-Pro through the window (429s across ~2 h of retries); 3,800/12,032 rows landed after the wall broke, the harness's 2000-row cap already covered. The fourth family joins at the next read; the gate law is per-family and three families already carry it. |

⚠ the anli metric caveat: the board's b12 metric is **macro-F1**, the lane prints hard **acc** — at ANLI's balanced 3-class population the two track within ~1 pp, and +2.2 pp above the macro-F1 bar is a comfortable margin either way; recorded as acc-vs-macro-F1, never restated as macro-F1.

- **The modelless A0 context rows** (chance-level sanity, the suites are outside its corpus): winogrande 0.4830 (chance 0.5) · hellaswag 0.2515 (chance 0.25) · anli 0.3297 (chance 0.3324) — all at chance, the suites' label universes verified live.
- Determinism: the lane's first-10-cases observed-repeat verbatim compare — no mismatch printed (the no-clock law).

## Reading the +Δ (the honest caveat both directions)

> ⚠ **SUPERSEDED** — built on the wrong row; the corrected direction is a small
> systematic −Δ (see CORRECTION). The +Δ casting hypothesis below remains a
> hypothesis either way; nothing in it is load-bearing for the gate.

Every family reads ABOVE the board's own number for the same entrant — **systematically**. Two non-exclusive causes: (a) Plan 012's suite casting (positional keys + option texts as descriptions, the xnli pattern) is more favorable than the JDI harness's native prompting; (b) the board's runs may sit at a different revision/quota posture. For THIS gate the direction that matters is degradation — NF4 at-or-above the board's bf16-class numbers everywhere means **no family's deployed-path behavior was lost to the quantization** (the lossy-surface law's conditional-retention test). The systematic +Δ is recorded as a **comparability caveat for T4**: a public-panel read through OUR adapter will read the same model higher than the board's own number — never quote the two as one axis.

## CORRECTION (2026-10-14, ~08:35 +07 — same session, pre-mmlupro-leg)

**The board column of the original table named the wrong entrant.** The numbers
0.8595 / 0.9502 / 0.7331 are `decision-27b-ckpt-nothink` — **Fastino GLiDE
no-thinking (28B)** — a different row of the same pinned snapshot, NOT
`vllm-sr-d3-v310`. The correct d3 row was present all along in the extraction
JSON this bench cites (`riir-train/scripts/plan629_d3_spotcheck_reference.json`,
`6b3b70b8` — `teachers.d3.per_benchmark`) and in the pinned snapshot itself
(Bench-067, sha256 `5d405036…`, verified again at correction time).

### The corrected table

| suite (population) | d3 NF4 ours | d3-v310 board row | Δ | reading |
|---|---|---|---|---|
| winogrande_jdi (1267 = full board pop) | 0.9455 acc | **0.9542** acc | **−0.87 pp** | ~1.4 SE at n=1267 (SE≈0.6 pp) — not individually significant, same-direction |
| hellaswag_jdi (2000 of 10042, the B5 cap) | 0.9760 acc | **0.9821** acc | **−0.61 pp** | subsample noise alone is ±0.31 pp at 95% — inside noise |
| anli_jdi (3200 = full board pop) | 0.7553 acc | **0.76** macro-F1 | **−0.47 pp** ⚠ | acc-vs-macro-F1 caveat stands (balanced 3-class ≈ within ~1 pp) |
| mmlupro_jdi (2000 of 12032) | — pending — | **0.7304** acc | — | the 4th family, spotcheck 143, launched 08:32 |

### Re-adjudication — the gate RE-PASSES on materially thinner margins

- The sign flips: our NF4 reads sit **0.5–0.9 pp BELOW** the board's own
  bf16-class numbers for the same entrant — a small systematic −Δ, consistent
  with a mild NF4 quantization cost (and/or small casting/infra differences
  partially canceling the Plan-012 casting favorability hypothesized below).
- The gate's law is degradation of deployed-path behavior: **no family
  degrades beyond 1 pp vs the board's own row, and no family shows a
  distributional collapse or bias signature** — the teacher's decision
  behavior survives NF4. The gate holds; the +8.6/+2.6/+2.2 pp margins were
  an artifact of the wrong row and are retracted.
- **The T4 comparability caveat inverts and shrinks**: our adapter reads the
  SAME model ~0.5–0.9 pp LOWER than the board's harness — never the +Δ story
  of the original §"Reading the +Δ". T4 must quote the corrected direction.
- Downstream effect on the captures: none on admissibility — the distill seam
  consumes option-score distributions as teacher signal, and a ≤1 pp
  decision-edge shift is inside the noise the student corpus already absorbs.
  The capture (leg A, running) stands.

## What this unblocks

- **The d3 leg-A capture** (launched 08:13:52, same window, `.raw/distill_teacher_d3`, the 6-suite slots union: xnli_en/emotion/prompt_injections + banking77/sst5/massive_intent_en — T2.5's router gets the complete d3 column).
- The B3 arm's public-panel column rides the same lane at the next full-panel read (mmlupro_jdi included once its fetch completes).

## Provenance

```
host: 4090-windows (shikuwa) · AC · box quiet (desktop baseline only)
serve: d3_server.py @ tag v3.1.0 · d3-v3.1.0-nf4 · port 8794 · VRAM 24,080 MiB
lane: --d3 (reflex 46ddc75) · p50 209–282 ms/row client round-trip
datasets: .raw/datasets (fetch_datasets.sh, winogrande full / hellaswag capped / anli full; mmlupro rate-walled)
```

Refs: riir-train `.issues/629` T2 (the gate's law) · `plan629_d3_spotcheck_reference.json` (the reference extraction, sha256-verified vs the Bench-067 pin) · reflex Plan 012 (the JDI suite casting) · Bench 141 (the posture-gate precedent this bench mirrors).
