# Bench 105 — the families wide eval: the template-disjoint frozen read (Plan 009 REVISED-2, issue 059)

Status: COMPLETE 2026-10-02 — the five widened families' eval populations landed + gated; the
modelless frozen read recorded below; the instinct hybrid re-measure rides instinct
`.benchmarks/0051_families_wide_eval`; the reflex-site quarantined section consumes both.

## What this is

Plan 009 REVISED-2 (the owner's display-only call): the six harness decision-point
families get ~96-case class-balanced template-disjoint eval populations (the
certification purpose stays dead — the product is the quarantined reflex-site section,
`data/families.json`, our lanes only, mandatory honesty caveat). `harness_cache_reuse`
is the DOCUMENTED DIVERGENCE: it keeps its frozen T3 12-fixture record
(`CACHE_REUSE_NOTE`), and the new divergence pin
(`cache_reuse_stays_the_documented_divergence`) holds that in place.

## The authoring law (enforced by `wide_evals_meet_the_authoring_law`)

Per family: population band [88, 104] with exact class balance (max−min ≤ 1, every
class ≥ 16) · per-family label-token bans (VIS `hide/short/long/full`; PERM
`allow/ask/deny`; ROUTE `local/engine/cheap/api/frontier/background/batch`; TOOL all six
tool names + component tokens, eval-only — its cal predates the law; SENS any digit) ·
zero shared word 3-grams with corpus∪cal (the memorization-path wall — the old evals
carried 2–7 such cases each) · unigram-overlap ceilings (mean ≤ 0.78, per-case ≤ 0.92)
· eval-internal dedup · a BLAKE3 digest pin per family (any fixture edit reds the gate
until consciously re-pinned).

## Old vs wide — the overlap table (the hygiene finding)

| family | old eval | old mean_ovl | old max_ovl | old 3-gram hits | wide eval | wide mean_ovl | wide max_ovl | wide 3-gram hits |
|---|---|---|---|---|---|---|---|---|
| harness_visibility | 16 | 0.7019 | 0.9167 | 3 | 96 (24×4) | 0.4534 | 0.6875 | 0 |
| harness_permissions | 12 | 0.7406 | 1.0000 | 7 | 96 (32×3) | 0.3692 | 0.6364 | 0 |
| harness_tool_fit | 12 | 0.7220 | 1.0000 | 2 | 96 (16×6) | 0.3922 | 0.7273 | 0 |
| harness_routing | 16 | 0.7741 | 1.0000 | 6 | 96 (24×4) | 0.3473 | 0.6667 | 0 |
| harness_sensitivity | 15 | 0.6845 | 0.8889 | 3 | 100 (20×5) | 0.3894 | 0.6667 | 0 |
| harness_cache_reuse | 12 | 0.8145 | 0.8750 | 11 | 12 (unchanged) | 0.8145 | 0.8750 | 11 |

**The finding, recorded not hidden: the old evals' higher accuracy was
corpus-overlap inflation.** The old populations shared 68–78% of their vocabulary with
the corpus/cal pool (per-case maxes at 1.00 — wholesale vocabulary copies); the
corpus-conditioned lane matched that shared vocabulary trivially. The template-disjoint
wide evals remove the inflation, and the modelless lane reads near its chance tier —
which is exactly what the section's honesty caveat is for. These are engineering
fixtures, not capability claims.

## The frozen read (reflex modelless, `run()` posture)

Provenance: `PROVENANCE: power=AC Power load=3.34 swap=2603.25M canary=121.1us/best5 powermode=2(high)`
— preflight PASSED, latency QUOTABLE. Host m3; run at reflex HEAD `bfae146` (the
fixture-source commits); harness `--skip-laya --suites <six>`, release profile;
count tables OFF (nb_scale 0 — the published baseline posture); fused-gate thresholds
fitted per suite at the cal 30th percentile (cal slices UNCHANGED — single-variable
measurement).

| family | n | acc | abstain | selective acc | p50 ms | p99 ms | det |
|---|---|---|---|---|---|---|---|
| harness_visibility | 96 | 0.3125 | 0.5417 | 0.3636 | 0.013 | 0.017 | ✓ |
| harness_permissions | 96 | 0.3125 | 0.5000 | 0.2708 | 0.009 | 0.010 | ✓ |
| harness_tool_fit | 96 | 0.3021 | 0.7604 | 0.3478 | 0.010 | 0.013 | ✓ |
| harness_routing | 96 | 0.2812 | 0.6146 | 0.2703 | 0.011 | 0.013 | ✓ |
| harness_sensitivity | 100 | 0.2200 | 0.5000 | 0.2600 | 0.010 | 0.011 | ✓ |
| harness_cache_reuse | 12 | 0.5000 | 0.6667 | 0.2500 | 0.010 | 0.011 | ✓ |

Chance tiers: vis 0.25 · perm 0.33 · tool 0.17 · route 0.25 · sens 0.20 · cache 0.50.
The raw zero-shot gates (`engine_answers_families_deterministically_fast_and_above_chance`)
read byte-identical accuracies on the wide evals — the two instruments agree, and every
anti-pathology floor held (vis 0.312 ≥ 0.20 · perm 0.312 ≥ 0.26 · tool 0.302 ≥ 0.13 ·
route 0.281 ≥ 0.20; sens/cache carry no floor). G2: every request < 1 ms (p99 ≤ 0.017 ms).

## The digest pins (re-pin only in the same change as a fixture edit)

```
harness_visibility   2fb6e5ecf0a5d204b264d9a59556762ec0ddf09dc5c29bf50fef0fdd2d045945
harness_permissions  e5127a1fada65c5b42e3df5a2f11cfd9a9f2ef3872248e1c29866e01da5f50e9
harness_tool_fit     47e6d75a046abb7f483689d838553e13ea4566d433bdfe3b487cbc3d3290d7c9
harness_routing      beecd817d1e7179f1f9928b888aeff035937d5fd4b8d5f1d38541c0e229f6458
harness_sensitivity  ad778f9de86320a76e947c1a7e69b3eb0b78028351282df72f281c736e76a802
```

Authoring provenance: `.scratch/famwide/*.tsv` + checker scripts (committed; five
parallel authoring sessions under the shared constraint set) assembled by
`.scratch/famwide/assemble_wide_eval.py` into `src/harness/families_eval_wide.rs`.

## Cross-repo consequences (landed in the same arc)

- **instinct** `served_family_decisions_are_the_frozen_a0_picks` replayed Bench 049's
  picks (the n=12–16 era) against `prepare_seat`'s cases — the wide swap invalidates
  it BY CONSTRUCTION. Re-measured at instinct `.benchmarks/0051_families_wide_eval`
  and the gate re-pinned there in the same arc.
- **reflex-site** main board: UNTOUCHED (its family rows stay the Bench-049-era read
  until a full republish — the quarantined section is the wide-eval product and never
  mixes into bench.json). The instinct disclosure prose in `publish_bench.py` that
  cites "n=12–16 template-shared" is updated to cite this record.
