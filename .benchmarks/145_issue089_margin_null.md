# Bench 145 — Issue 089 T1: the margin-null read — perm-disagreement carries NO signal on the modelless lane (0/345 untied slots) and its residual laya signal closes the issue as a comparison note

**Status:** COMPLETE — T1 executed 2026-10-15 (M3, this box); issue 089 CLOSED per its
pre-registered scope statement. The probe record gained three additive fields
(`gold`, `margin_top2`, `tie` — `CaseRow`/`CaseSpread`, the T1 join); the analyzer is
`scripts/issue089_margin_null.py`; records under `145_issue089_margin_null/`.

## What ran

The Issue-077 permutation probe (`--perm-probe`, K=5 = identity + 4 SplitMix64 shuffles,
`slot_seed(case_id, qid)`) extended ADDITIVELY to carry per slot: the gold option key, the
identity ordering's top-2 margin `p(1) − p(2)`, and the exact-tie flag — everything the
margin-null test needs, in one artifact (the issue's stated alternative: extend the probe
rather than join offline). Three records:

| record | lanes | sampling |
|---|---|---|
| `modelless/` | modelless (control, plain deployed posture) | 6 choice suites, default (40–60 cases/suite) |
| `laya_cpu/` | modelless + laya/english | ag_news, emotion, xnli_en, massive_intent_en, banking77 @ 20 cases |
| `laya_typed_cpu/` | modelless + laya/typed | typed_decisions @ 30 cases |

Provenance: host m3, sha `abd778d` (the schema extension itself is the next commit — the
records were produced by the working tree carrying it), G5 parity GREEN pre-run
(2/2 checkpoints, 28.1 s, the publish law held), laya CPU posture, serde default suites,
deterministic stride sampling identical to Bench 128's. Box state in each record's
`meta.box_start/end`. **Not measured: drex / agentjev / d1** — their servers are
4090-hosted and the 4090 was unreachable today (LAN + Tailscale both timed out); named
as absent postures, never folded into any verdict.

## Pre-registration (pinned before any result was read)

Floors verbatim from issue 089: **PRIMARY** ΔAUROC ≥ 0.02 over the margin-only baseline on
at least one lane; **SECONDARY** partial correlation ≥ 0.05 (disagreement ↔ error,
controlling margin); both read → each judged at α/2 (Bonferroni); the typed_decisions
defect lane cannot satisfy the "at least one lane" bar; floors tighten-only.

Mechanical definitions (pinned in the analyzer header before execution): untied = the
codebase's exact-top-2-f64 tie class (`!tie`); disagreement = `flipped` (any non-identity
ordering's top pick ≠ identity top pick); error = `ref_pick ≠ gold`; margin =
`margin_top2`; baseline score = −margin; AUROC = Mann–Whitney with midranks; lane level =
suites pooled per lane; the pre-registered POINT bars judge, a 2000-resample bootstrap CI
rides each pooled ΔAUROC as disclosure.

## Results

Lane-level (pooled over suites; `analysis.txt` carries every per-suite cell):

| lane | n | err | dis | dis rate | AUROC(margin) | AUROC(dis) | ΔAUROC | partial r |
|---|---|---|---|---|---|---|---|---|
| **modelless** | 345 | 218 | **0** | 0.000 | 0.512 | 0.500 | −0.012 (degenerate) | — |
| **laya/english** | 100 | 31 | 21 | 0.210 | 0.682 | 0.745 | **+0.063** [−0.073, +0.196] | **+0.470** |
| laya/typed | 44 | 14 | 7 | 0.159 | 0.774 | 0.645 | −0.129 [−0.285, +0.023] | +0.261 |

- **The modelless lane's disagreement signal does not exist**: zero untied slots (0/345,
  across all six suites and all three samplings) moved their top pick under any of the
  four shuffles. The predictor is constant; ΔAUROC is a degenerate 0.5 − AUROC(margin)
  artifact. **Primary MISS.** Bench 128's control implication is thereby confirmed as a
  measurement, not an inference: on the deterministic engine, "K orderings disagree" and
  "the top-2 is exactly tied" are the same statement, and one decide call already exposes
  the tie through its margin.
- **laya/english**: primary PASSES at the point bar (+0.063 ≥ 0.02) and the secondary
  passes decisively (partial r +0.470 against a 0.05 floor; at n=100 a null r would sit
  within ±0.2 at 2σ — 0.47 is far outside). The signal is concentrated exactly where
  Bench 128 located laya's fragility: `massive_intent_en` (Δ +0.094, dis rate 0.30 — the
  same 30% flip cell) and xnli_en (+0.095); banking77 disagrees often (0.70) but
  non-selectively (Δ −0.133). The ΔAUROC bootstrap CI straddles zero — the PASS is
  point-bar-honest, support-thin, and disclosed as such.
- **laya/typed**: mixed (primary miss, secondary pass) and defect-lane-excluded regardless.
- **typed_decisions modelless is no longer the Bench-128 red**: this run reads 0.00 pt
  median with all 31 flips in the exact-tie class (route binding
  "by-name + index-anchored … issue 079") — the legacy `k == N` positional red did not
  reproduce at the current default posture. The defect-lane exclusion was kept anyway
  (conservative direction).

## Verdict — issue 089 CLOSED per its own scope statement

The issue pre-registered its closure classes. The deterministic-engine delta is **gone**
(measured null, not inferred); the residual error-prediction power lives only on a
trained/order-sensitive lane (laya/english). Scope statement: *"If yes ONLY on
trained/order-sensitive lanes, this issue closes as a comparison note inside the published
PriDe/self-consistency class; a trained-lane serving mode, if ever wanted, is a NEW issue
filed at that point — never a scope extension of this one."* No serving mode is wanted on
this evidence (thin ΔAUROC support, single lane, K× decide cost for a lane that is not the
product), so no new issue is filed; the comparison note IS this record.

## Disclosures

- **Error convention**: `error = ref_pick ≠ gold` counts every untied slot, abstentions
  included (the probe records no abstain bit at question level). This is deliberately NOT
  the board's scoring convention (answered-only) — the per-cell error rates here
  (pooled 0.63 modelless) are therefore much higher than board A0 accuracies; the T1
  reads are rank-based within the pinned convention and unaffected.
- **Gold join**: `gold.idx` is consumed against the criteria-map iteration order — the
  same index space every harness consumer uses (`option_display_labels`, cascade compose,
  confusion rows, pair-head); consistent by construction, no dataset-text join performed.
- The pooled `modelless` row mixes the three records' samplings (40/60-case default and
  the 20/30-case laya companions) — irrelevant to a 0/345 null, disclosed for the laya
  rows' n.
- Old (Bench-128-era) records lack the three new fields; they remain valid — the fields
  are additive and nothing reads them programmatically except the T1 analyzer.

## Reproduce

```bash
cargo test --release --features laya-riir --test laya_riir_parity   # G5 BEFORE any laya cell
cargo run --release --bin harness -- --perm-probe \
  --out .benchmarks/145_issue089_margin_null/modelless
cargo run --release --features laya-riir --bin harness -- --perm-probe --perm-max-cases 20 \
  --suites ag_news,emotion,xnli_en,massive_intent_en,banking77 \
  --out .benchmarks/145_issue089_margin_null/laya_cpu
cargo run --release --features laya-riir --bin harness -- --perm-probe --perm-max-cases 30 \
  --suites typed_decisions --out .benchmarks/145_issue089_margin_null/laya_typed_cpu
python3 scripts/issue089_margin_null.py .benchmarks/145_issue089_margin_null/*/perm_probe.json
```
