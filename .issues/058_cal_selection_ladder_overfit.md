# Issue 058 — the cal-selection ladder overfits the cal slice; dataset-suite accuracy collapsed at Bench 076/077 and no arm owns a floor

**Status:** (d) ROOT-CAUSED + FIXED 2026-10-01 — the canonical pool is REBUILT (clean full pulls, deduped, provenance-manifested), the board is RESTORED at HEAD (Bench 100: every cell back to its pre-076 value, banking77 +1.6 real gain), the fetcher root cause is CLOSED (TRAIN_CAP default 20000 + per-suite pool floors in the gate). Remaining: 4090 re-run (deferred — box busy) then the reflex-site republish (the cross-host drift gate refuses without it); (a)/(b)/(c) DEFERRED with triggers recorded below.

## THE RESTORATION (Bench 100, m3, 2026-10-01 — `.benchmarks/100_pool_fix_m3`)

Verdict round 2 (claude reviewer, AGREE): rebuild the canonical dir as clean full pools, re-run, republish. Box state (Issue 021 preflight): `PROVENANCE: power=AC Power load=3.91 swap=294.50M canary=116.3us/best5 powermode=2(high)`.

**The pool** (`.raw/datasets/POOL_MANIFEST.json` has the full provenance): t20k full pages copied for ag_news/banking77/massive/xnli/sst5/emotion, then the fetcher's own dedupe law applied in place (first occurrence kept, order preserved): sst5 −11 rows (8533), emotion −1 (15999); prompt_injections kept canonical (byte-identical to t20k); typed_decisions kept canonical (the fuller 1200-row pull — the 078/099 corpus-cap lift stays); thai kept canonical. Test pages untouched everywhere (byte-identical across all three pool generations — verified per suite).

| suite | 4k pools (076–096 board, wrong) | t20k-era published (the selection basis) | **restored (Bench 100, clean full pool)** | Δ vs pre-076 |
|---|---|---|---|---|
| ag_news | 0.8625 | 0.8825 | **0.8825** | 0.0 |
| emotion | 0.7700 | 0.8850 | **0.8850** | 0.0 |
| sst5 | 0.2017 | 0.3967 | **0.3967** | 0.0 |
| prompt_injections | 0.7672 | 0.7672 | **0.7672** | 0.0 |
| xnli_en | 0.5033 | 0.5233 | **0.5233** | 0.0 |
| massive_intent_en | 0.4067 | 0.7800 | **0.7800** | 0.0 |
| banking77 | 0.4020 | 0.8260 | **0.8420** | **+1.6** (real engine gains since 052 — nb_ridge/count-table work — now actually serving) |
| typed_decisions | 0.5725 | 0.4655 (800-row pull era) | **0.5725** | +10.7 vs the t20k column = the 078 corpus-cap lift, retained (the t20k typed pool was the stale 800-row pull; canonical is the fuller 1200) |

(The t20k-era column cites the 052-era published records at old code; the at-HEAD exact-reproduction proof for the t20k bytes is the 3-suite A/B earlier in this issue — emotion/massive byte-exact, banking77 +1.6.) The canonical-deduped column reproduces the t20k-era numbers EXACTLY on 6 of 7 (the deduped rows are sub-measurable, as the earlier sst5/emotion A/Bs established) and gains on banking77 from real code drift. Nothing here is a regression.

All eight dataset suites' slice lines read `OK` at full-pool scale (e.g. ag_news pool 19800, emotion 15799, sst5 8333, massive 11314, banking77 9793, xnli 19800) — zero KNOWN-DIRTY acks on the default board.

## The fetcher root cause is CLOSED (verdict condition 1)

- `scripts/fetch_datasets.sh`: `TRAIN_CAP` default 4000 → **20000** (the t20k re-baseline scale). The 4000 default writing into the canonical dir was the original sin; a fresh fetch no longer rebuilds 4k pools under the same name.
- `slice_guard::MIN_POOL_ROWS`: per-suite pool floors (~80% of full-pull scale) — a shrunken pull is a hard refusal naming `TRAIN_CAP=20000 scripts/fetch_datasets.sh`. Digests tell you the pool changed; the floor tells you it shrank. 16 gate unit tests green.

## DEFERRALS, with triggers (verdict round 2 — B/C/D deferred on purpose)

- **(a) baseline-arm floor** — DEFERRED. Trigger to reopen: if the publisher ever publishes a modelless cell below its pre-076 value without a loud disclosure row. The slice digests + floors have closed the silent channel the floor was belt-and-braces against.
- **(b) LCB-based selection** — DEFERRED. Trigger: a future re-selection whose ladder top-2 are cal-tied (massive 0.715/0.72/0.72 was the recorded shape) AND the pick flips the served posture. Running it now would re-pick winners again — a second re-baseline right after this one, for robustness the restored board does not need.
- **(c) k-fold cal-front probe** — DEFERRED. Trigger: any future selection whose cal→test gap exceeds ~15 pt at the selected posture (the massive-076 shape). Measurement-only whenever it runs.

## PUBLISHED (2026-10-01) — the board is live at the restored basis

- The 4090 re-ran the same command at `ad94345` after a full data sync
  (**1039 dataset files SHA256-verified identical across hosts BEFORE the
  run** — the verdict's pre-run hash condition): **15/15 accuracy
  bit-identity vs the m3 doc, zero drift**. Records:
  `.benchmarks/100_pool_fix_m3` + `.benchmarks/101_pool_fix_4090`.
- Published through the Issue-057 CORPUS_RESET ack scoped to the six
  suites whose pool actually changed (typed/prompt kept their pools —
  the ack correctly REFUSED them as stale on the first attempt, the
  stale-ack discipline working as designed); the fresh quotable m3
  timing + the unjudged 4090 timing publish per the standing 4090
  posture. Pairing gate PASS, self-test 66/66, chart smoke PASS.
- reflex-site `d750175` + deployed (live-verified: the restored numbers
  + corpus digests serve on reflex.gist.rs; the dated correction note
  renders on the bench page). The wrapper's env-clean also gained
  `-u PUBLISH_BENCH_ALLOW_UNQUOTABLE` — my first publish attempt leaked
  the ack into the self-test's own stale-ack case and made it
  pass-by-refusal (the 099 leak class, one env over).
- En-route: the 4090's `git pull` over its GitHub path hangs (worked
  around with a local `git bundle --all` + pull-from-bundle), and
  detached (Start-Process) harness runs on that box die silently —
  the foreground ssh run is the reliable posture (two silent detached
  deaths, then 15/15 foreground).

Issue 058's own surface is now CLOSED: root cause fixed (fetcher default
+ floors), data rebuilt + provenance-manifested, board restored on both
hosts, published with a dated disclosure. The deferred (a)/(b)/(c) stay
on their recorded triggers; the instinct arena re-point stays documented
in instinct `.issues/013` (ready-to-flip).

## THE REMAINING PUBLISH PATH (the one open task)

The reflex-site republish is BLOCKED on the 4090 re-run — the publisher's cross-host drift gate (Issue 018 T7, mechanized) refuses merged-state modelless drift, and the 4090's modelless cells carry the 4k-pool numbers. The 4090 was busy (2026-10-01, owner call) — the run is deferred, not skipped. When free: sync `.raw/datasets` (bytes + POOL_MANIFEST) to `E:\git\riir-reflex`, hash-verify BOTH sides BEFORE running (SHA256 per file, the 077 procedure), git pull + rebuild there, run the same command at `REFLEX_BENCH_HOST=4090-windows`, then `scripts/republish_bench.sh data/bench.json 100_pool_fix_m3/results.json <4090>/results.json` + the dated disclosure note (condition 7: the 09-28..10-01 modelless cells came from 4k pools and are superseded — swapping numbers without the note would hide the mistake).

## VERDICT on (d) — root cause found, 2026-09-30 (A/B-confirmed, same binary, same flags, only `--datasets-dir` changed)

The 076-era "the ladder now selects postures that calibrated better on the cal slice" mechanism
story is **wrong-in-mechanism**: the primary driver is a silent **datasets-dir move**.

- **Every run that produced the board's "before" cells read `.raw/datasets_t20k`** (the 09-26
  full-pull re-baseline: `meta.datasets_dir` in 052/057/058/061/062/063/064/069/070/073/092–095).
- **Bench 076/077 — and 096/097 after them — read the DEFAULT `.raw/datasets`** (the Sep 22 pull,
  `TRAIN_CAP=4000` → 40 pages ≈ 4,000 train rows/suite): the first runs in the whole
  038→076 lineage to drop the t20k dir. 074/075 (the plan003 thai lane) started the drift; the
  076 republish made it the published board.
- **Same binary, same published flags, three same-selection suites** (run this session):

| suite | t20k pools | default pools | 076 published | old board |
|---|---|---|---|---|
| emotion (ridge@8) | **0.8850** | 0.7700 | 0.77 ✓ | 0.8850 ✓ exact |
| massive_intent_en (nb@4) | **0.7800** | 0.4067 | 0.4067 ✓ | 0.7800 ✓ exact |
| banking77 (nb@1) | **0.8420** | 0.4020 | 0.4020 ✓ | 0.8260 (−1.6 pt = code drift since 052, disclosed) |

  The old numbers reproduce EXACTLY on t20k at HEAD; the new numbers reproduce EXACTLY on the
  default dir at HEAD. Test pages are byte-identical between the two dirs (verified per suite,
  blake3) — only the corpus side moved: **a 3–5× smaller train pool rebuilt every count table
  and ridge head under the same nominal rung**, and the ladder re-rolled its cal selection on
  the new cal front. No head-semantics change; nothing to revert in code.
- **Why nobody saw it**: `meta.datasets_dir` records the path, but no run-time or publish-side
  check compared pool identity across runs (the 076 record's own words about its 4090 twin:
  "the eval-side digest checks never caught it"). The cal-overfit story in the 076 disclosure
  describes a real but SECONDARY effect (the ladder genuinely re-picks on the smaller cal front);
  the halving is corpus starvation.
- **Canonicalization is owner-gated** (the instinct precedent: "the frozen Bench-005
  re-baseline pool (datasets_t20k) — re-pointing it is a re-baseline decision, never a
  cleanup"). Options for the owner: (i) re-run the board on t20k (the selection-history pool),
  or (ii) re-baseline the published posture on the default dir (a full re-selection read).
  Either way the posture now CARRIES its pool: the slice-integrity digests below make the next
  silent dir move impossible to publish through.

## Slice assertions, landed (the owner's "do assert the slices — we get it wrong 3rd times around
slices already")

Three incidents priced this: 023 (misaligned cal index), 039 T2 (positional pool cut), 076 (this
issue — the silent dir move). The gate (`src/harness/slice_guard.rs`, ungated + pure, 12 unit
tests) runs in `prepare()` for every dataset suite and:

1. **Refuses the suite** on: cal∩test / pool∩test / pool∩cal exact-row overlap, and a test
   sample that misses an engine label while the budget could have covered it (the Issue-039
   non-representative-sample class).
2. **Discloses** (never fails): cal/pool label starvation (the fallback guard's territory).
3. **Records raw slice-identity digests + counts** (test/cal/pool) into results.json and the
   TABLES.md header + the run log — the pool digest is the RAW (uncapped) identity, the axis the
   per-lane consumed-corpus digest (057) is deliberately not (at a small cap a truncated pool
   and a full one consume near-identical docs — exactly why the 076 publish sailed through).
   A future dir move is now visible at RUN time ("pool 3800" vs "pool 15,800") and mechanical
   at publish time (digest compare).

**First-run yield — the gate fired on real data**: sst5's train pull carries exact duplicate
rows — one cross-split (train idx 1139 = test idx 180, the row `"no. ."`) and two train-internal
(`"too bad ."`, `"see it ."` — copies straddling the cal/pool split, i.e. cal cases scoring
against pool docs with their own text). 14/15 suites clean; sst5 refused until fixed.

## Data fixes landed with the gate

- `.raw/datasets/sst5` train pages surgically de-duplicated (3 train-side rows dropped; the test
  population untouched, n_test stays 600). sst5 re-ran green: pool 3797, acc 0.2017 (== the 076
  cell — the 3 docs moved no accuracy; the digests moved as designed). The fetcher now enforces
  the same law (`scripts/fetch_datasets.sh dedupe_train <suite>`, jq-path, idempotent,
  key-order-independent — planted-fixture proven incl. a flipped-key twin; skips LOUD without
  jq, the 4090 syncs data by copy from this box).
- ⛔ **`.raw/datasets_t20k` carries the same sst5 dups and is LEFT FROZEN by verdict (2026-09-30, claude reviewer round 1 — REVISE)**: its aggregate blake3 (`ebfb0317…`, 8544 rows) is a published pin in `.docs/02_protocols/dataset_manifest.md`, and every published t20k-era row (reflex Benches 052+; the instinct arena's frozen re-baseline rows) was measured on those exact bytes — changing bytes under the name IS a re-baseline, not a cleanup. Instead the gate accepts the exact dirty state under a membership pin: `SliceFacts::KNOWN_DIRTY` pins the (test, cal, pool) triple (`67fe9666…`/`d210e7c6…`/`cf933108…`, pool 8344); only that triple proceeds, loudly (`⛔ KNOWN-DIRTY SLICE` line + `acknowledged: true` in results.json), and any repair/re-pull moves the triple → the refusal returns until the pin is deleted in the same change. Verified: t20k sst5 proceeds under the ack (acc 0.2017 == the fixed canonical pool's reading — **the 3 duplicate rows are sub-measurable at the published posture**); the canonical dir runs clean with no ack; **riir-instinct serve_gates 17/17 green again** (my first gate commit had red'd their `enc_row_boots…` test through the sst5 seat on the dirty t20k pool — caught before their next run). The re-baseline decision (dedupe t20k + re-verify the published rows, or a deduped sibling pool) rides **instinct `.issues/013_owner_gate_pickup.md`**.
  The 4090's `.raw/datasets/sst5` needs the same 3-row sync before the next cross-host run (the 077 copy
  procedure) — its copy predates the fix.
- **t20k-wide dup sweep (2026-09-30, throwaway canon probe + the gate itself):** sst5 carries 1 cross-split + **10**
  train-internal dups (the bigger pull has more mirror dups — the KNOWN_DIRTY pin covers that exact state, verified
  proceeding under the ack); **emotion carries 1 train-internal dup that does NOT straddle cal/pool** (harness on
  t20k: `pool 15800 · OK` — proven, not inferred); every other t20k suite reads 0 cross-split / 0 internal. The
  instinct/Rethink arena (frozen t20k, all lanes selected + measured + served on it) is internally consistent and
  needed no numeric fix — the cross-lane break is on the SITE: modelless rows (post-076 canonical pools) vs
  hybrid/encoder rows (t20k) are different data bases, which is instinct 013's E2 observation with its cause now
  named. The owner canonicalization call (one pool everywhere, likely t20k — the A/B shows the old modelless
  numbers reproduce exactly there) restores cross-lane comparability and retires the arena's `--skip-pin-a0` posture.

## TL;DR

The public board's Reflex-vs-laya verdict reads **3/8** (wins: typed, emotion, prompt_injections) with
gaps xnli +35.7, massive +28.7, sst5 +17.0, ag_news +8.7 (banking77 +2.0 within noise at n=500).
The laya bar did **not** move — laya (english) m3 accuracy cells are byte-identical in every board
version since 09-27 (`fe7c123`). Reflex's own modelless cells dropped at ONE commit:
`reflex-site b08aef7` 09-28 10:05 (reflex Benches 076+077, run `c464a8a`), and the board has carried
the low numbers since. The drop is **disclosed** in the 076 record but **unowned** — no open issue
tracks the repair. This issue owns it.

## The board diff at b08aef7 (extracted from reflex-site `data/bench.json` history)

| suite | before (armed posture, 038-era levers) | after (run c464a8a) | Δ pt | selection before → after |
|---|---|---|---|---|
| banking77 | 0.8260 | 0.4020 | −42.4 | nb@1 → nb@1 (**same**) |
| massive_intent_en | 0.7800 | 0.4067 | −37.3 | nb@4 → nb@4 (**same**) |
| sst5 | 0.3967 | 0.2017 | −19.5 | nb@16 → nb@0 (off) |
| emotion | 0.8850 | 0.7700 | −11.5 | ridge@8 → ridge@8 (**same**) |
| xnli_en | 0.5233 | 0.5033 | −2.0 | nb@16 → nb@1 |
| ag_news | 0.8825 | 0.8625 | −2.0 | nb@4 → nb@16 |
| typed_decisions | 0.4655 | 0.4655 | 0 | oc@2 → oc@2 (later **rose** to 0.5725 via the Issue-052 corpus-cap lift) |

Sharpening beyond the 076 record's own phrasing: on **three suites the nominal selection did not
change** (massive nb@4, banking77 nb@1, emotion ridge@8) yet accuracy halved — so the 072-era
ladder rework changed what a given selected posture *does* (head application semantics / cal-front
construction), not merely which winner gets picked. The record's "the ladder now selects postures
that calibrated better on the cal slice and generalize worse on test" covers the re-picked suites;
the same-selection suites are a stronger claim: **same posture, cal ≈ unchanged, test −12 to −42**.

## Evidence chain (all verified this session, 09-30)

1. **Laya static**: `reflex-site` bench.json at `1999b38`/`a2f1f9e`/`0ce46a8`/`e55a9ab` — laya english
   m3 acc identical (xnli 0.86, massive 0.6933, sst5 0.3717, ag_news 0.95), lane source `fe7c123` 09-27.
2. **The drop commit**: `.benchmarks/076_modelless_quotable_full.md` discloses it verbatim:
   > ⚠ The accuracy deltas are an ENGINE MOVE, not a re-measurement claim … the ladder now selects
   > postures that calibrated better on the cal slice and generalize worse on test … the accuracy
   > movement is disclosed, never hidden.
3. **Not cross-host flake**: the same run re-proved m3↔4090 bit-identity 15/15, and caught+fixed two
   real dataset-sync defects (xnli train pages missing on the 4090; a truncated massive train pool).
4. **Families moved UP in the same run**: visibility +18.8, permissions +25.0, tool_fit +41.7,
   routing +31.3, sensitivity +40.0 — the count-table lever arms there now. The dataset suites paid
   for it.
5. **Cal→test gap is the mechanism**: massive cal_acc at the selected posture reads ~0.72 on the cal
   slice while test reads 0.4067 — a ~31-pt generalization gap the raw-cal-argmax selection cannot see.
6. **Downstream visibility**: the site verdict line is computed from the board, so it flipped
   automatically; the reflex-site FAQ sentence "took several suites to or past the laya lanes
   (Bench 051)" is now STALE relative to the board it sits beside.

## Why this matters (the honest framing)

- The 09-28 drop moved the published Reflex-vs-laya verdict from "reflex wins most dataset suites"
  to 3/8 **without any owner-visible decision** — it rode a timing-fix republish (site Issue 003 T1).
  HEAD-truth publishing is correct and stays; but a posture whose selection is cal-overfit should not
  silently become the serving posture for two days with no floor and no owner call.
- The Instinct hybrid lane currently papers over part of the gap (certified H2 arms above reflex on
  several suites) — that is composition, not a reflex repair.

## Repair directions (owner decides; not mutually exclusive)

- (a) **Baseline-arm law (the floor)**: pre-register the pre-072 winners (massive nb@4, banking77
  nb@1, sst5 nb@16, emotion ridge@8, xnli nb@16, ag_news nb@4) as a ALWAYS-RUN floor arm. The
  published posture may never be an arm that scores below the floor arm on the frozen test read
  without a loud disclosure row (ratchet-style, the `carry_beats_incumbent` precedent from site
  Issue 003 T2, pointed at selection instead of timing).
- (b) **Generalization-aware selection**: replace raw cal-argmax in the ladder with a
  lower-confidence-bound pick (Beta-LCB over cal, or a paired bound) — the fleet already ships the
  primitive (`katgpt_core` best_belief / the riir-dao Beta-LCB pattern). Suites whose ladder
  candidates are cal-tied (massive 0.715/0.72/0.72) should not be decided by noise.
- (c) **Grow or k-fold the cal front**: the cal slice is ~200 questions and sits INSIDE the train
  tail; a k-fold or enlarged cal front directly prices the cal→test gap that (b) bounds.
- (d) **Root-cause the same-selection deltas first**: git-blame the 072-era count-table-ladder
  commits (Bench 072 / Issue 045 T1+T2 window, 09-27..09-28) for what changed in head APPLICATION
  under a fixed `selected_scale` — massive/banking77/emotion losing half their accuracy at the same
  nominal posture says the head semantics moved, and that may be a plain defect with a plain revert.

## Tasks

- [-] (d first) Pin the 072-era commit(s) that changed same-posture head behavior; classify revert-vs-intended — **RESOLVED 2026-09-30: no code to revert; a datasets-dir move (t20k → default 4k pools), A/B-confirmed both directions; verdict + table above**
- [ ] (a) Land the baseline-arm floor in the harness + publisher (loud disclosure on any below-floor publish)
- [ ] (b) LCB-based selection behind a flag; A/B on the 8 dataset suites, both hosts, byte-identity gates
- [ ] (c) k-fold cal-front probe (measurement-only) to price the cal→test gap per suite
- [ ] Re-run the full 15-suite matrix at the repaired posture — **now carries the slice digests; the pool question (t20k vs default) is the owner-gated half**
- [x] reflex-site: refresh the stale FAQ sentence ("took several suites to or past the laya lanes") to describe the current board honestly — done at reflex-site `1d32a33` (the sentence now scopes the claim to the 051-era posture and names this issue for the pool split)
- [x] Slice-integrity gate (overlaps + test coverage hard-refuse; slice digests in results/TABLES/run log) + the sst5 data fix + the fetcher dedupe law (2026-09-30)
- [ ] Owner: canonicalize the board's datasets dir (t20k re-run vs default re-baseline) + decide the t20k sst5 dedupe (instinct pins that pool)

## Non-goals

- No hiding: the board keeps publishing HEAD truth; the repair changes selection, never disclosure.
- No gate-posture entanglement: Issue 056's `--gate-fit-calibrated` promotion is orthogonal (gate
  abstention, forced accuracy unchanged there) and is not to be reverted as part of this.

## Constraints

- Published posture flags remain `--skip-laya --nb-select --oc-select --ridge-select` (066's lane
  posture) unless the owner re-baselines.
- Any repaired posture re-proves cross-host bit-identity (the Issue-018 T7 law) before publish.
