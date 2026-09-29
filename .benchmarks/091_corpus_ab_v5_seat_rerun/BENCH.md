# Bench 091 — the V5 corpus A/B re-run at the seat posture: **V5 PASS** — arm A reproduces the 0.7800 anchor exactly, synthesis lifts the modelless row 0.7800 → 0.8133 (paired LB95 +0.0110)

**Status:** MEASURED 2026-09-29, reflex @ `2e8a023` (clean HEAD, `git status` clean
except an untracked leftover script), harness built fresh in an isolated target
dir (`CARGO_TARGET_DIR=/tmp/rfx_v5`, release, `--features nb_scope`) — the
clean-build rule bench 089's void note demanded. This closes the single
recorded follow-up of bench 089 + riir-train 584: the instrument is repaired,
the anchor is alive, and the V5 question that the void left unqueried now has
its measured answer.

## Box state (quoted per the G2 law)

M3 Max, AC power, load avg 7.48 / 15.33 / 23.19 (LOADED — sibling sessions
active). The read is a deterministic modelless accuracy + a sub-ms latency
check: accuracy is not load-sensitive and the determinism gate reported TRUE;
the latency figures are quoted as V6's sub-ms class, not as a perf record.

## Run

```
/tmp/rfx_v5/release/harness --corpus-ab .raw/corpus_synth/massive_intent_en_synth.jsonl \
  --datasets-dir .raw/datasets_t20k --synth-extra-cap 128 --synth-out .raw/corpus_synth --nb-select
```

Pre-check (before the A/B read anything): the same binary at the plain lane —
`harness --skip-laya --suites massive_intent_en --datasets-dir .raw/datasets_t20k
--nb-select` — reproduced the published posture line (nb-select ladder picks
**scale 4 α observed-laplace**, sel-slice 0.6050) and the modelless hard row
**0.7800** exactly (TABLES.md row), matching the morning arena print
(`cap 48 · head 0.00 · nb 4.00 · ridge 0.00`). Only then was the A/B read.

## Result

| quantity | value |
|---|---|
| arm A (gold-only, seat posture) | **0.7800 — reproduces the published anchor exactly (≤1e-6); instrument ALIVE** |
| arm B (+2048 synth docs, pool AND count tables) | **0.8133** (234 → 244 of 300) |
| paired LB95 (synth − gold) | **+0.0110 > 0 — V5 PASS** |
| flips | 11 synth-only wins vs 1 gold-only loss |
| V6 latency | gold p50 115 µs / p99 137 µs · synth p50 112 µs / p99 195 µs — sub-ms holds |
| determinism | true |
| corpus | gold 11314 docs → 13362 (2048 synth in scope; extra-cap 128/label) |

Full report: [CORPUS_AB.md](CORPUS_AB.md) + [corpus_ab.json](corpus_ab.json)
(copied verbatim from `.raw/corpus_synth/` — the lane's own artifacts; the
`.raw/` copies are gitignored).

## What this means

- **The instrument repair is validated**: the `8f425d1` seat-posture fix makes
  arm A the DEPLOYED build (the `fit_posture` prologue: cal-selected
  nb_scale 4.0 + the fitted fused-gate thresholds), not `EngineConfig::default()`
  (the 0.6100 posture that voided bench 089). The aliveness law fired green.
- **V5 is answered POSITIVE**: the openthai-vetoed template×slot synthesis
  (54.2% teacher acceptance, E0-directed allocation — bench 089's own record)
  lifts the modelless lane at the published posture by **+3.33 pt on
  massive_intent_en** — the first measured lift of the modelless lane itself
  on this suite. 0.7800 → 0.8133 closes ~24% of the distance to the served
  Instinct H2 arm (0.8267) and ~7% of the distance to the openthai bar
  (0.9200) with ZERO trained weights and zero latency cost.
- **The lever generalizes by construction**: synth rows enter both the drafter
  pool AND the count tables at the seat posture (`nb_sets_b`), which is exactly
  why the wash at the default posture (bench 089's void datum) was evidence
  about that point only.

## Honest caveats

- **Thin support, quantified**: +10 net cases on n=300. Sensitivity (the
  gate's own statistic, `paired_lb95`): 11/1 reads +0.0110; 10/1 still
  +0.0086 and 9/1 +0.0062; the LB95 floor reaches ZERO only near **6 wins /
  1 loss** (−0.0005) or **11 wins / 4 losses** (−0.0019) — roughly five fewer
  wins or three more losses. An exact McNemar on 11/1 gives p ≈ 0.006
  two-sided, so the sign does not rest on the normal approximation. The V5
  gate's own law (LB95 > 0) holds; no stronger claim is made here.
- **Leakage**: the synthesis consumed TRAIN rows only — template×slot mining,
  the per-intent E0 rumor fractions (each label's OWN count tables over the
  train pool), and the openthai veto (a teacher forward, never test-derived);
  `--synth-extra-cap 128` is the default and was never swept. The frozen test
  split was read twice in this session — the pre-check (`run()`, instrument
  validation) and the A/B — both at the ONE frozen seat posture; no selection
  ever touched test.
- **089 double allocation (disclosed, not repaired here)**: `.benchmarks/`
  carries BOTH `089_paw_ft_typed_m3/` (`cc0838e`, 09:23) AND
  `089_corpus_ab_v5_anchor_void.md` (`85d452d`, 10:04) — the stale highwater
  (89) is how the second landed. "Bench 089" in plan 426 and riir-train 584
  means the ANCHOR-VOID doc. Renumbering inside this commit would break those
  citations; the repair is a separate owner-called move.
- **One suite, one read.** The synthesis lane is measured on massive only (the
  artifact in scope). Extension to the other suites is new veto passes (teacher
  forwards), not a new instrument.
- **Not yet seated**: the +synth corpus is a lane result, not the deployed
  posture. Seating it (the gold+synth pool at the registry cap) is a separate
  promotion decision with its own GOAT pass — and the arena/serve corpus path
  must consume the same artifact bytes (BLAKE3 sidecar) before any published
  row moves.
- The A/B reports git sha at RUN time (`2e8a023`); cleanliness is established
  by the fresh isolated-target build at a clean worktree, not by that field.

## Provenance chain

bench 089 (void + default-posture datum) → fix `8f425d1` (the seat-posture
repair) → this re-run (V5 PASS). riir-train 584's landing record carries the
same chain; plan 426 T5's V5 row closes with this record.
