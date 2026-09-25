# Bench 038 — the noul route anti-alignment fix (issue 030): prompt_injections 0.4397 → 0.4828 (the T7 −4.3 pt regression's root cause), every other suite bit-identical

**Verdict: GOAT — the changed suite improves on every axis (acc, macro F1,
ECE 0.1070 → 0.0228, acc@50 0.3966 → 0.6552, G1 PASS held), no other suite
moves a bit, and the fix is a guard (fewer terms evaluated), not a cost.**

## The mechanism (issue 030)

The T7 option-rank blend reached noul questions through the legacy
`k == N` index-alignment path (the by-name path already excluded noul).
`prompt_injections` arms exactly N = 2 domains — `["0" = benign,
"1" = injection]` — and every question is noul with internal option order
`[yes, no]` (yes = "this IS an injection"). Index alignment mapped
option 0 → domain 0, scoring "yes, injection" against the **benign**
centroid: an anti-signal by construction, and the measured below-chance
0.4397 (chance 0.50; drafter-only 0.4828) was its exact signature.

Fix: `route_active` requires a Choice/Score question — noul never takes
route terms through either path. Blast radius: only engines where
`N == k == 2` on a noul question; `prompt_injections` is the only dataset
suite in that shape. The serve edge is a 7-domain engine — unaffected.
Regression pin: `engine::tests::noul_never_takes_route_terms_even_when_k_equals_n`
(no route scale may move a noul distribution; the same engine still blends
a by-name choice).

## The A/B (modelless-only, deterministic lane — accuracy is the claim)

Same box, same build recipe, back-to-back runs 2026-09-25T02:57Z (before,
`a2c03aa`) / 02:59Z (after, the fix). Box state stamped in the tables:
AC Power, mode=high, load 4.33 → 4.07, "latency QUOTABLE" — but latency is
NOT the claim here; the modelless lane is bit-deterministic and accuracy
is box-independent.

| suite | before acc | after acc | Δ |
|---|---|---|---|
| prompt_injections | 0.4397 | **0.4828** | **+4.3 pt** (drafter posture restored; ECE 0.1070 → 0.0228; acc@50 0.3966 → 0.6552; G1 PASS before and after) |
| every other suite | — | — | **bit-identical** (typed 0.3190, ag_news 0.5100, emotion 0.2825, sst5 0.2167, xnli 0.3467, massive 0.6900, banking77 0.4460, all harness_* families unchanged) |

Full after-tables: [`TABLES.md`](TABLES.md) · raw: [`results.json`](results.json).

## The code_fixtures caveat (disclosed, expected)

code_fixtures metrics wiggled (ECE 0.0806 → 0.0809, abst 0.33 → 0.25,
acc 0.2500 unchanged): that suite's corpus is GENERATED FROM THIS REPO'S
OWN fn spans, so editing `src/engine.rs` legitimately changed its fixture
set between the two runs. Not an engine regression — a self-referential
suite tracking its own source. Its G1 was FAIL before (calibrated 0.2777 >
raw 0.2464) and FAIL after (0.2639 > 0.2464) — unchanged verdict.

## Gates

- G1: prompt_injections PASS held (raw 0.4827 / calibrated 0.3663 / floor
  0.5073). All other suites' G1 verdicts unchanged (code_fixtures FAIL
  before and after).
- G2/G4: untouched — the fix removes work on the affected path (noul
  skips the state re-embed + N dot products it was incorrectly paying);
  zero-alloc law holds by construction (branch-only guard). The
  `decision_set_goat` bench surface is unchanged.
- Determinism: `det ✓` on every suite, both runs.
- `cargo clippy --all-targets -- -D warnings` clean; `cargo test` green
  (57 lib + integration suites, 0 failures).

## Publication state

The canonical `001_phase1_tables/TABLES.md` and the site `bench.json` are
stale the moment this lands (the publisher refuses hosts whose modelless
accuracy drifts). The republish (full clean-window run, both laya lanes,
preflight-clean) is issue 030's tracked follow-up. README's headline row
is updated to 0.4828 with the issue-030 footnote.
