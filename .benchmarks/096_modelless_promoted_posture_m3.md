# Bench 096+097 — the promoted-posture modelless re-publish (m3 + 4090-windows): `--gate-fit-calibrated` reaches the site's modelless lane

**Status:** RECORD 2026-09-30 — the Issue-056 close-out's owed reflex-site
republish (the site's modelless rows still carried the pre-promotion gate
posture). Two coordinated runs, one engine state `d4051c8`: 096 = m3
(fresh QUOTABLE timing + the promoted posture's gate cells), 097 =
4090-windows (cross-host bit-identity re-proven at the promoted posture).
The publish (`reflex-site`, lane-scoped update over `data/bench.json` as
primary) is the 076+077 flow verbatim, replayed for the 056 promotion.

## Why both hosts had to run

The modelless lane's cross-host bit-identity claim (Issue 018 T7) gates on
the FINAL merged state — publishing fresh m3 gate cells beside stale 4090
cells risks the drift gate and, per the README law, the modelless lane
moves TOGETHER or not at all. Same sha, same flags, same datasets on both
hosts (the 076+077 precedent, executed the same way).

## The exact posture (the published one — unchanged from 076)

```
cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --out <dir>
```

canonical pool (`.raw/datasets` — `DEFAULT_DATASETS_DIR`), heads OFF
(the published baseline posture), genome off, cascade absent. The only
difference from 076's run is the engine: `d4051c8` carries the katgpt-rs
LLW solver substrate (Issues 909/910/911) and **`--gate-fit-calibrated`
promoted to the default** (Issue 056's close).

## What moved — and what did not

**Hard accuracy: bit-identical to every incumbent cell, both hosts, all
15 suites.** The promotion is a gate-fit change (boot-time, cal-slice);
forced picks and the cal-selected ladders are untouched — measured, not
assumed: the fresh m3 run reproduces every published `hard.accuracy`
digit-for-digit (ag_news 0.8625, banking77 0.402, typed 0.5725 post-cap,
families all equal), and the 4090 run reproduces the m3 run.

**The gate cells move to the promoted posture** — the point of the
publish. On every dataset suite the fresh `calibrated_abstain` EQUALS the
incumbent `raw_abstain` (the fitted percentile target the raw gate chose
in the pre-promotion runs): ag_news 0.525, emotion 0.4675, sst5 0.5817,
xnli_en 0.5767, prompt_injections 0.6810, typed 0.7170, banking77 1.0 →
1.0. The fresh `raw_abstain` reads 1.0 everywhere by construction (the
promoted posture refits BOTH lanes' thresholds on the calibrated scale;
the raw lane's identity-calibrated confs sit below it — the documented
Bench-095 lever artifact; the raw-lane baseline to read is the
pre-promotion runs' raw gate, which is what the incumbent cells hold).

**A stale latency cell fixed en route (typed_decisions, m3):** the
incumbent typed p50 0.517 ms was LANE_CARRIED from Bench 076 — measured on
the PRE-cap-lift 800-row corpus, while the accuracy cell (0.5725) is the
1200-row corpus's. The carried value understated the current corpus by
~35% since 078 (whose own quotable read was 0.815). This publish's fresh
QUOTABLE read (0.764 ms) replaces it via the `carry_beats_incumbent` path
— the Issue-003 T2 lane-carry law working as designed, in the direction
it exists for.

## PROVENANCE (the Issue-021 box-state law)

- 096 (m3): preflight PASSED at the run window — load 3.92→4.16 across
  the run, `latency_quotable: true` BOTH spans.
  `PROVENANCE: power=AC Power load=4.09 swap=388.06M canary=114.7us/best5
  powermode=2(high)`. A first attempt (start-span load 8.65, the sibling
  mining window) was discarded per the 076 scout-run lesson and re-run on
  the quiet box.
- 097 (4090-windows): the host has no box probes — `box state UNJUDGED`,
  disclosed per lane (the standing 4090 posture). Its timing publishes as
  unjudged; the accuracy half is the load-bearing one here.

## Results — the gate-posture deltas that matter (m3 cells, site→fresh)

| suite | hard (site→fresh) | calibrated_abstain (site→fresh) | = old raw target |
|---|---|---|---|
| typed_decisions | 0.5725 → 0.5725 | 0.9415 → 0.7170 | ✓ |
| ag_news | 0.8625 → 0.8625 | 0.9000 → 0.5250 | ✓ |
| emotion | 0.7700 → 0.7700 | 0.9875 → 0.4675 | ✓ |
| sst5 | 0.2017 → 0.2017 | 0.3150 → 0.5817 | ✓ |
| prompt_injections | 0.7672 → 0.7672 | 0.9052 → 0.6810 | ✓ |
| xnli_en | 0.5033 → 0.5033 | 0.3233 → 0.5767 | ✓ |
| massive_intent_en | 0.4067 → 0.4067 | 0.3900 → 0.6500 | ✓ |
| banking77 | 0.4020 → 0.4020 | 0.3500 → 1.0000 | ✓ |
| families (6) | all equal | all equal | — |

The pre-promotion `calibrated_abstain` values (0.94 typed, 0.90 ag_news,
0.99 emotion) were the DISTORTED operating points — the Issue-056 defect
(thresholds fit on the raw scale, applied on the calibrated scale). The
fresh values land exactly on the fitted percentile targets.

## Results — 4090-windows (097): 15/15 cross-host bit-identity at the promoted posture

Every suite's modelless `hard.accuracy` AND `calibrated_abstain` (including
the selective sub-dicts) are byte-identical between 096 (m3) and 097
(4090-windows) — ag_news 0.8625 @ abstain 0.525 / selacc 0.9421, typed
0.5725 @ 0.7170 / 0.6537, banking77 0.402 @ 1.0, all six families equal.
The Issue 018 T7 claim re-proven on the promoted engine. Sibling tree
synced both boxes first (reflex d4051c8, katgpt-rs 04e0d18fe, riir-infer
befcaa2, riir-reflexer 60408d2; dataset pools MD5-verified across boxes:
xnli 000/039, typed 011, massive 039 all identical — the 077-era repairs
hold).

## Republish (reflex-site, lane-scoped)

`scripts/republish_bench.sh data/bench.json <096>/results.json <097>/results.json`
— current published doc as PRIMARY (the wholesale-drop guard), the two
fresh docs as lane-scoped updates. All gates green: publish self-test
52/52, chart render smoke, the cross-host drift gate (both hosts at HEAD,
15/15 identical), pairing gate, docs-mirror parity, bench-page smoke
(incl. the rig-scope + instinct-card checks).

**Timing cells CARRIED, not replaced (the Issue-032 law, working as
-called):** every modelless latency cell keeps the host incumbent's
quotable timing (m3: Bench 076's; 4090: Bench 077's) with the LANE_CARRY
provenance note — `apply_lane_carry` carries unconditionally for a
refreshed carried-class lane unless the incumbent is UNFIT (Issue 003
T2's narrow refusal). Both sides quotable → incumbent wins; timing moves
only through dedicated timing publishes. The gate cells, accuracy
columns, and source stamps all moved to `d4051c8` — the promotion is
live on the page.

⚠ **A pre-existing gap surfaced, filed as Issue 057**: the typed m3
p50 0.517 ms was carried from Bench 076 — measured on the pre-lift
800-row corpus, while the accuracy cells beside it measure the 1200-row
corpus (every quotable read on the new pool: 0.764–0.815 ms). The
corpus axis has no LANE_CARRY escape (the population reset covers only
the question axis), so no lane-scoped publish can refresh it while the
incumbent stays quotable. Disclosed on the cell; owner-gated amendment
proposed in the issue.

## Reproduce

```sh
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- \
  --skip-laya --nb-select --oc-select --ridge-select \
  --out .benchmarks/096_modelless_promoted_posture_m3
# 4090-windows: same flags, REFLEX_BENCH_HOST=4090-windows, the synced
# sibling tree (katgpt-rs 04e0d18fe, riir-infer befcaa2, riir-reflexer 60408d2)
```
