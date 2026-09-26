# Bench 045 — Issue 032/034's M3 full-lane leg: modelless + laya (rust×3 + py×3) + leak scan at the fold-promoted default, preflight-clean, 15 suites no absences

**Status:** LANDED 2026-09-26 · the M3 half of the both-hosts-together
publish (Issue 034's corrected plan: live-as-primary + both docs as
updates). Taken at reflex `6939420` + riir-infer `53334f9` (the fold
epilogues PROMOTED default-on the same hour — this doc's rust cells ARE the
shipped lane; an earlier same-day fold-OFF pass was superseded and never
landed). `--head-select --laya-python`, `--features
slice_leak,laya-riir,laya-riir-metal`, `REFLEX_BENCH_HOST=m3`, 15/15
suites, no absences.

**PROVENANCE:** start `power=AC Power mode=high load=3.21 swap=1054M` · end
`power=AC Power mode=high load=2.21` — latency QUOTABLE (results.json
meta.box_state, both ends).

## Accuracy: the head-select promotion holds on the M3 at the promotion commit

Modelless head selections: sst5 → 0.5, massive_intent_en → 1.0,
banking77 → 1.0, every other suite → 0 — **IDENTICAL to Benches 040, 043
and 044** (four-run, three-of-them-cross-host stability). The headline
suites: banking77 **0.6840**, massive_intent_en **0.7933** — bit-identical
to all three prior runs.

## The metal lane moved (the fold promotion is visible in the suite cells)

rust laya p50 vs the published Bench 041 cells (same host, same protocol):

| suite | this run | published 041 | Δ |
|---|---|---|---|
| typed_decisions/english | 309 ms | 367 ms | −15.8% |
| ag_news/english | 28 ms | 31 ms | −9.7% |
| massive_intent_en/english | 40 ms | 46 ms | −13.0% |
| banking77/english | 61 ms | 69 ms | −11.6% |

(one sequential run — read with Bench 036's instability caveat; the paired
adjudication is **Bench 046**, which is the record that prices these cells
against the python oracle.)

## Issue 024 T5 — the M3 leak columns (the scan is a TWO-HOST fact)

7/7 in-scope suites carry leak blocks; the counts are **byte-identical to
the 4090's Bench 044 run** — exact/near: ag_news 1/26 · banking77 0/17 ·
massive_intent_en 4/17 · prompt_injections 0/2 · sst5 1/0 · emotion 0/0 ·
xnli_en 0/0. The scan is a dataset+registry-cap property, not a host
property — now confirmed across both hosts independently.

## Lanes landed

- modelless: 15 suites (head-select posture, the fixed ladder label)
- laya rust: typed_decisions ×3 checkpoints, other 14 ×english — the
  shipped fold-on lane
- laya python: same shape (the `py/` prefix) — the torch reference, IPC
  latency, mps device
- leak blocks: 7 in-scope suites

## What this unblocks

The publish (Issue 032 tasks 2–4, Issue 034's plan step 3) once the 4090
full-lane doc exists — the drift gate requires both hosts' modelless to
move in ONE publish (m3's published 0.69-class cells are pre-head-select;
both docs carry the promoted values). 4090 status: blocked behind the
riir-train `plan410_stage0_train` GPU job (GPU exclusivity law; checked
twice this session).

## Addendum 2026-09-26 — the laya-lane de-leaked columns (issue 024 T5 close-out)

T5's tail. The `acc_deleaked` plumbing reached the laya lanes at T3 but
the columns had never been READ — the runner threads the leak flags
through `assemble_laya_lane_result` (served_flags remapped through the
ANE bucket skips, `&f[..cases.len()]` on the python lane), and this run's
results carry them on every checkpoint row of every leak-scope suite.
Per-suite (headline hard accuracy → acc_deleaked; riir and py agree to
all printed digits per suite — same cases, same mask, the determinism
cross-check):

| suite | exact/near | modelless | laya·english | py/english |
|---|---|---|---|---|
| ag_news | 1/26 | 0.5100 → 0.5067 | 0.9500 → 0.9491 | 0.9500 → 0.9491 |
| emotion | 0/0 | 0.2825 → 0.2825 | 0.5925 → 0.5925 | 0.5925 → 0.5925 |
| sst5 | 1/0 | 0.2167 → 0.2170 | 0.3717 → 0.3723 | 0.3717 → 0.3723 |
| prompt_injections | 0/2 | 0.4828 → 0.4737 | 0.6983 → 0.6930 | 0.6983 → 0.6930 |
| xnli_en | 0/0 | 0.3467 → 0.3467 | 0.8600 → 0.8600 | 0.8600 → 0.8600 |
| massive_intent_en | 4/17 | 0.7933 → 0.7814 | 0.7500 → **0.7634** | 0.7500 → **0.7634** |
| banking77 | 0/17 | 0.6840 → 0.6832 | 0.4980 → 0.4948 | 0.4980 → 0.4948 |

The honest reads: (1) the laya lanes DROP with the leaks removed on the
near-leak-heavy suites (banking77 −0.32 pt, ag_news −0.09 pt,
prompt_injections −0.53 pt) — the shared leak inflates them too, the
disclosure is not a modelless-only concern; (2) massive_intent_en moves
UP (+1.34 pt) — its 21 flagged rows were net UNLUCKY for the lane, so
the de-leaked read is not mechanically lower; (3) emotion / xnli_en /
sst5 are flat (0 / 0 / 1 exact-only flags). typed_decisions stays
`not_applicable` (templated rows). The laya clippy postures T3 owed are
discharged in the same window (`--no-default-features --features
laya-riir --all-targets` needed `laya-riir = [..., "dep:blake3"]` — the
parity files' capture hashing predates the posture's last run; both
parity gates re-run green at that posture).
