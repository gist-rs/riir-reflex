# Bench 065 — ag_news full-pull volume lever (issue 041)

**Status:** COMPLETE — the lever is PULLED and measured **NEGATIVE**. The
120k full pull moves ag_news modelless accuracy **+0.25 pt** (0.8825 →
0.8850) against the issue's expected 0.90–0.92, the corpus-cap axis moves
**exactly nothing** (0.8850 at cap 64 == cap 1000), and the T5 joint
blend-genome walk over the full-volume posture ends **HELD** (no volume ×
posture interaction). The −6.5 pt gap to laya (0.9500) is NOT
corpus-volume-bound on any measured axis; the issue's fallback hypothesis
(the gap is word-order/disambiguation the encoder owns) is now the only
live one. Issue 041 is resolved by this record (file removed per the
noise-reduction rule; entry in HISTORY.md).

## What ran

1. **Fetch** — `scripts/fetch_datasets.sh` gained a `SUITES` filter (the
   issue's own "scope it to ag_news if the script allows" invitation; a
   120k TRAIN_CAP without the filter would have pointlessly pulled
   ~120k xnli train rows of a 392k-row source). Then:

   ```sh
   SUITES=ag_news OUT=$PWD/.raw/datasets_agnews_full TRAIN_CAP=120000 SLEEP=1 \
     sh scripts/fetch_datasets.sh
   ```

   The HF datasets-server limiter walls after ~300 pages per burst even at
   SLEEP=2.5 (the header's ~130-rapid figure is the no-sleep case; the
   sustained window is the real quota). Four burst→cooldown(6 min)→resume
   cycles (the skip logic resumes exactly); final burst clean:
   **120,000 train rows in 1200 pages + the 400-row test split, 0 failed
   requests**, 38 MB in `.raw/datasets_agnews_full/` (gitignored — the
   canonical dirs are untouched, the issue-038-T2 OUT-override law).

2. **Byte-identity law** — every page this dir shares with the canonical
   set is byte-identical: test-000..003 (their BLAKE3s match the manifest's
   canonical table) and train-000..199, verified 204/204 mechanically
   (b3sum dir-vs-dir loop). The 1000 new train pages (0200–1199) are
   committed as per-file BLAKE3 rows in
   `ag_news_full_digests.b3.txt` beside this record; the whole-dir fold is

   ```
   b3sum (sorted per-file digests of all 1204 pages) =
   59617a3ec0335ec6abc9ae5d554b2c64374fe2ea75c3df518fce522ca7d9d702
   ```

3. **Two lane runs** (modelless only, `--skip-laya`; the same selection
   flags as the current published column — the Bench 063 run is the
   byte-comparable t20k control at the same flags):

   ```sh
   # run 1 — registry posture (corpus cap 64/label)
   cargo run --release --bin harness -- --skip-laya \
     --datasets-dir .raw/datasets_agnews_full --suites ag_news \
     --nb-select --oc-select --ridge-select \
     --out .benchmarks/065_agnews_full_volume
   # run 2 — the corpus axis pulled too (Issue 013 lever-1 instrument)
   ... --corpus-cap 1000 \
     --out .benchmarks/065_agnews_full_volume_cap1000
   # run 3 — run 2 + the T5 joint blend-genome walk (--genome-select, the
   # sibling's landed lane): the interaction question, one command
   ... --corpus-cap 1000 --genome-select \
     --out .benchmarks/065_agnews_full_volume_genome
   ```

## PROVENANCE (box state at the runs)

Both runs on a LOADED box (a sibling session throughout): run-1 start
load 6.14, run-2 within the same session — `bench_preflight.sh` had
REFUSED earlier in this session (load 6.08–6.14 > 6). **Latency numbers
are NOT quotable from this record.** The accuracy verdicts are
pick-counts, load-immune by construction (G5 determinism is the modelless
lane's own claim; the det column reads ✓ in both runs).

## Results

| posture | corpus volume (count tables) | corpus cap/label | genome walk | test acc | Δ vs control | raw ECE | cal ECE (G1) |
|---|---|---|---|---|---|---|---|
| t20k control (Bench 063, same flags) | 19,600 train rows | 64 (registry) | — | 0.8825 | — | — | — |
| full pull, run 1 | 119,600 train rows | 64 (registry) | — | **0.8850** | +0.25 pt | 0.7406 | 0.0100 PASS (floor 0.2848) |
| full pull, run 2 | 119,600 train rows | 1000 (override) | — | **0.8850** | +0.00 vs run 1 | 0.7693 | 0.0125 PASS (floor 0.2866) |
| full pull, run 3 | 119,600 train rows | 1000 (override) | HELD (cal 0.7600, 1 pass) | **0.8850** | +0.00 vs run 2 | 0.7693 | 0.0125 PASS (floor 0.2866) |

Selection lanes at full volume: nb scale **32** selected (observed-laplace,
bag view) in both runs — same scale as the t20k posture; cal-slice acc
plateaus 0.7550 → 0.7600 (nb) across the volume (the slice already reads
saturated). Ridge ladders flat (0.7550 at every scale in run 1; run 2's
ridge@8 dips to 0.7450 — within slice noise, not selected either way).
oc-select ran in both; count-table selection unchanged.

## Verdict

1. **The count-table volume axis is ~saturated at the t20k pull.** The
   uncapped count tables (NB + NBSVM-ridge + option-conditioned, "from
   TRAIN rows only, uncapped" — the T7 law) already consumed 19,600 rows;
   6× more rows buy +0.25 pt. Classic unigram saturation: ~1000 docs/label
   of news text already covers the countable signal.
2. **The corpus-cap axis (the drafter corpus) is accuracy-irrelevant for
   ag_news.** cap 64 → 1000: identical accuracy, and the raw confidence
   geometry shifts (ECE 0.3765 → 0.4034) — so the equality is set-level
   (354/400 both), not pick-level, but nothing improves. The p50 grew
   ~17× (0.166 → 2.902 ms — PROVISIONAL, loaded box, direction only):
   the bigger corpus costs serving latency for zero accuracy. The
   registry cap 64 stands.
3. **The T2-POC extrapolation did not transfer because the engine
   changed.** The 0.665 → 0.868 volume lift was measured on the
   pre-count-table engine (corpus+drafter-driven picks); T7's count
   tables now dominate the pick and were already at the t20k volume. The
   volume law was real for the engine it was measured on.
4. **The T5 joint-genome interaction axis is EMPTY at full volume too**
   (run 3, cross-checked with the sibling's freshly landed
   `--genome-select` lane): the coordinate-descent walk over
   `{route, head, nb(+α,+view), oc, ridge}` ends HELD at cal 0.7600 —
   the composed posture already sits at the slice's plateau, and the
   test row is byte-identical to run 2. No volume × posture interaction
   to harvest.
5. **The remaining −6.5 pt gap (0.8850 vs laya 0.9500) is not
   volume-reachable.** The issue's own fallback — word-order/
   disambiguation the encoder owns — is the only live hypothesis left on
   this suite. The 041 non-lever note (prompt_injections' selection-slice
   weakness, pure-NB 0.8362 vs shipped 0.7672 unreachable via the
   ~.50-reading slice) is filed forward as its own issue — it is protocol
   work (a bigger cal slice for noul-only suites), not a volume lever.

## Publication posture

**No new protocol column.** The canonical basis (`.raw/datasets_t20k`,
the Bench 052 stratified protocol) is unchanged — this was a measurement
sidecar (the issue-038-T2 OUT-override law doing its job). No republish,
no cross-host run, no site drift-gate involvement: a negative result on a
sidecar dir owes no column. The published ag_news row stays 0.8825 @ t20k.

Artifacts: `results.json` + `TABLES.md` in the three run dirs (this
record's siblings) + `ag_news_full_digests.b3.txt` (1204 per-file BLAKE3
rows).
