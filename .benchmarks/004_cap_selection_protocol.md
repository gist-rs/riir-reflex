# Bench 004 — Issue 013 lever 1 remainder: the stratified cap-selection protocol (and the refusal it produced)

**Status:** COMPLETE 2026-09-24 · run commit = this landing · M3 (macOS,
release, modelless lane only `--skip-laya`) · fixtures `.raw/datasets/`
ag_news + banking77 · DETERMINISTIC: the selection run re-ran
byte-identical (both suites, every reading, both selections).

## The instrument (closes Bench 003's protocol hole)

`harness --cal-select-cap [LIST]` (default ladder 8,16,32,64,128,256,512;
the registry default always joins the candidate set): for each eligible
dataset suite, forced accuracy is measured at every candidate on a
**label-stratified selection slice** — round-robin one row per label per
round over the train rows BEYOND the cal prefix, up to `cal_cap` docs, no
RNG — the argmax cap is picked on that slice ONLY (exact ties prefer the
registry default, then the smallest candidate), and the test split is read
once at the selected cap, at the FULL registry pool. Selection corpora
exclude the selection docs by content (a selection case must never score
against its own text); the exclusion is per-candidate uniform (~5% pool
trim) and ranking-only. With the flag off, every byte of the default
posture is unchanged — verified: today's default-posture banking77 row
matches the published Bench 001 row field-for-field except latency.

**Why not the registry cal slice itself (the first attempt, same day):**
the cal slice is the FIRST `cal_cap` train rows, and the mirrors store rows
label-grouped — measured, ag_news's first 200 rows span **2 of 4** labels
and banking77's **2 of 32** (the mirror's whole train split carries only 32
of the 77 test labels). Cal accuracy on that slice reads chance-level
(ag_news .13–.22, banking77 .005–.02) and cannot rank caps: the first
selection run picked cap 8 for ag_news — the sweep's worst test reading —
and cap 16 for banking77 off a .080-vs-.060 margin on a 2-label slice. The
registry cal slice stays exactly as it was (thresholds, sigmoid
calibration, conformal floor are untouched); selection got its own
stratified slice.

## ag_news — selection CONFIRMS the registry default

| cap | 8 | 16 | 32 | **64 (default)** | 128 | 256 | 512 |
|---|---|---|---|---|---|---|---|
| sel-slice acc | .3600 | .4200 | .4450 | **.4800** | .4500 | .4550 | .4750 |

Selected **64 = the registry default** (no tie-break needed). Test row at
64 (full registry pool): acc **.5100** — byte-matches the published
default row. The Bench 003 test-split sweep (peak at 64) is confirmed by a
selection-clean measurement. No registry change.

## banking77 — the test-split peak does NOT transfer; promotion REFUSED

| cap | 8 | **16** | 32 | 40 (default) | 64 | 128 | 256 | 512 |
|---|---|---|---|---|---|---|---|---|
| sel-slice acc | .0750 | **.0800** | .0700 | .0600 | .0400 | .0350 | .0350 | .0350 |

Selected **16**. Test row at 16 (full registry pool): acc **.4100** — a
**−3.6 pp REGRESSION** against the true default baseline. The +6.6 pp peak
at 128 that Bench 003 measured on the test split reads **.0350 on the
stratified holdout — tied with saturation, near-worst.** The test-split
gain was a test-split-selected artifact: exactly the
selection-biased-gain class the protocol hole existed to catch, and the
protocol caught it.

**PROMOTION REFUSED — banking77's registry cap stays 40.** The Issue 013
lever-1 deferral is resolved as NO, on protocol-clean evidence.

## Record correction (Bench 003)

Bench 003's banking77 table headed its .4380 column "**64 (default)**" —
the registry default is **40**, and today's default-posture baseline reads
**.4460**, byte-identical to the row Bench 001 published. .4380 is the
**cap-64** reading; the column was mislabeled, and "+6.6 pp over default"
re-reads as +5.8 pp over the true default — moot under the refusal, but
the record is corrected here rather than left standing.

## Caveats (honest)

- The selection slice can only span labels the train split carries:
  banking77's mirror train covers 32/77 labels — the other 45 ride the
  self-doc fallback and are invisible to selection. A fuller train fetch
  is the unblock path; with the default confirmed-safe there is no urgency.
- banking77's selection margins are 2–4 cases of 200 — noise-fragile. The
  argmax law takes them as-is; the promotion confirm (the single test read
  at the selected cap) is the guard against noise-selection, and it fired.
- The selection slice answers are in-corpus-distribution (train-region
  questions); test stays the out-of-sample read. A cap that helped only on
  the split it was selected on is precisely what this protocol refuses to
  ship.
