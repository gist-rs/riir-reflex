# Bench 078 — typed_decisions TRAIN pull un-capped (Issue 052): the modelless row reads 0.4655 → 0.5725

**Verdict: POSITIVE — pure modelless upside.** The birth cap in
`scripts/fetch_datasets.sh` fetched 800 of the typed_decisions train
split's 1200 rows; the entire security_incidents train block (offsets
900–1199) plus invoice rows 200–299 sat behind it, and the corpus guard
self-doc'd security_incidents at every published run (the ⛔ line in every
table since Bench 001). With the four missing pages fetched, the same
harness at the same published posture (registry defaults; nb/oc/ridge
cal-selected; heads off; genome off) reads:

| lane · posture | n | acc | macro F1 | ECE(maxp) | readout-ECE | G1 |
|---|---|---|---|---|---|---|
| modelless — capped pool (Bench 077, 4090) | 2000 | 0.4655 | 0.4455 | 0.0750 | 0.0055 | PASS |
| modelless — extended pool (Bench 078, m3) | 2000 | **0.5725** | 0.5429 | 0.0993 | 0.0114 | PASS |

**+10.7 pt**, byte-cross-checked with the instinct arena seat's A0′ read
(their Bench 015: 0.5725 — the same number, same corpus bytes, the seat
consumes this repo's `run()`).

## The fix (reflex-owned, landed)

1. `scripts/fetch_datasets.sh`: typed_decisions train cap 800 → 1200.
   Re-running `SUITES=typed_decisions` fetched exactly pages
   train-008…train-011 (100 rows each; the eight existing pages skipped by
   the resume law) — dataset total 1200, 12 files.
2. Corpus byte-identity: pages 000–007 + test-000…003 + splits.json are
   byte-identical (`cmp`) to their pre-extension selves AND to the
   instinct lane's verified copy. The **test split is untouched** — the
   published claim stays on the same 400 cases / 2000 questions.
3. The four new page digests + byte sizes joined the typed section of
   `.docs/02_protocols/dataset_manifest.md`; the full-pull aggregate
   table gained the typed row (12 pages / 1200 rows); the stale
   "typed caps already cover their train split" note was rewritten.

## Honest notes

- **The cal slice re-derives.** Issue 052's draft text claimed the cal
  front is positional and therefore unchanged — wrong for this harness:
  since Issue 039 T2 the cal slice is the STRATIFIED round-robin front
  over the WHOLE train split, so the wider pool re-balances it (e.g. the
  nb cal acc moved 0.37 → 0.36, and oc cal-selected **oc@4** where the
  capped pool selected oc@2). The number is published only at its own
  re-measured posture; nothing else was touched.
- **OC lever re-selection, not re-tuning**: the selection law is
  unchanged; the wider evidence moved the cal argmax from scale 2 to
  scale 4. Both are cal-selected per the promotion bar.
- Per question kind (n=600/600/800): choice 0.4383 → **0.5483**, noul
  0.6200 → **0.7300**, score 0.3700 → **0.4725**. Every kind moved up.
- G1 (readout ECE): calibrated 0.0114 vs the conformal-naive floor
  0.1818 → PASS (beats the uncalibrated 0.4837 and the floor).
- Corpus fallback guard: the security_incidents self-doc line is GONE
  (0 fallback labels — the guard's first silence for this suite).
- Latency p50 0.815 ms (m3, AC/High-Power mode, load ~3.6-4.8) —
  QUOTABLE per the box line; slower than the 077 4090 row's column but
  that row was a different host; the m3 column is the README's lane.
- The **laya·typed lane is unaffected by construction**: it is a frozen
  trained checkpoint scored on the (byte-identical) test split; the
  harness train corpus feeds only the modelless drafter + cal selects.
  0.7445 stays the published laya-typed number.

## Downstream

- README results table: typed row 0.4655 ³ → **0.5725 ³⁵** (+ footnote
  ³⁵ carrying this record); the Issue-038 lane's against-fair-baseline
  sentence gains the post-lift clause.
- The reflex-site typed row re-publishes from the updated tables (the
  site session's lane — `data/bench.json` regeneration +
  `scripts/republish_bench.sh`); instinct's site-parity re-pin rides the
  same republish (their Bench 015 already records both postures).
- Consumers of the old 800-row bytes: none found (no test pins the
  corpus digests; the manifest is the record).
