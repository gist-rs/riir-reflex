# Issue 044 — close Reflex's accuracy gaps to laya on the five suites it trails

**Status:** OPEN — filed 2026-09-27. Targets measured from the published reflex-site data; no lever work in this issue yet.

## Why

Reflex already wins latency (sub-ms vs laya's 100+ ms p50) and bundle size
on every suite. Where it trails laya on accuracy, that is a gap to close,
not a reason to pick laya. reflex-site now states it that way (reflex-site
`e7f4638`, arena TL;DR "Reflex vs laya, accuracy": "gap to win the
other 5").

## Targets (published data/bench.json, english checkpoint, 2026-09-27)

| suite | Reflex | laya (rust) | gap to win |
|---|---|---|---|
| code_fixtures | 0.2500 | 0.6667 | +41.7 pt |
| xnli_en | 0.5233 | 0.8600 | +33.7 pt |
| harness_sensitivity | 0.4000 | 0.6000 | +20.0 pt |
| harness_routing | 0.4375 | 0.6250 | +18.8 pt |
| ag_news | 0.8825 | 0.9500 | +6.8 pt |

Reflex is at or above laya on the other 9 of 14.

## Tasks

- [ ] T1 — ag_news first: the smallest gap (+6.8 pt) and the largest
  question count. Check whether the Plan 004/005 levers (Issue 038 T5/T7:
  cal-selected heads, blend genome) already move it, before adding anything.
- [ ] T2 — the harness_* suites carry 8–24 questions each. Confirm the gap
  exceeds the Wilson interval before spending effort (one question is
  4–12 pt there).
- [ ] T3 — xnli_en: NLI needs pair reasoning, which a corpus-bound lane is
  structurally weak at. Measure a modelless pair-feature head before
  declaring it out of reach; record the verdict either way.
- [ ] T4 — code_fixtures: the population is commit-relative (harvested
  from this repo's own fn spans). Pin a frozen population first, or the
  gap moves with the tree.
- [ ] T5 — each closed gap re-publishes lane-scoped through
  reflex-site `publish_bench.py`, with latency from a run that passes
  `scripts/bench_preflight.sh` (the Issue-021 publish wall refuses
  otherwise).
