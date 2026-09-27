# Issue 043 — prompt_injections: the selection slice cannot see the pure-NB polarity

**Status:** OPEN — filed 2026-09-27, forwarded from issue 041's non-lever
note (041 resolved by Bench 065: the ag_news volume lever measured
NEGATIVE; this was its recorded forward).

## The datum

prompt_injections' pure-NB read is **0.8362 on test (probe)** vs the
shipped blend **0.7672** — a +6.9 pt posture that the selection machinery
cannot reach. The selection slice reads ~0.50 for BOTH polarity candidates
(the T1b slice-weakness class, re-confirmed in the issue-038 round-2
re-run — nb@1 held), so the cal side cannot distinguish the postures and
the test read is spent at the blend.

## The fix shape (protocol work, not a lever)

A bigger (and/or polarity-stratified) CAL SLICE for noul-only suites.
prompt_injections' registry cal_cap is 100 over a 546-row train split;
its noul polarity is rare in the train tail, so a 100-case slice carries
too few of the discriminating cases for the polarity candidates to
separate. Candidates:

1. Raise `cal_cap` for noul-only suites (registry row — one field).
2. Polarity-aware cal sampling (stratify the cal slice by the noul
   polarity label, the Issue-039 stratified-slice law applied to the
   polarity axis).
3. A per-suite cal-slice cap selection (`--cal-select-cap` shape, already
   shipped for the corpus-cap axis — the same protocol pattern).

## Constraints

- The cal slice is train-region selection data — any of the three stays
  protocol-legal (test never enters a fit or a selection).
- The test split is read ONCE at the selected posture (the standing law).
- The suite's test cap is 0 (all 116 rows) — a single test read at the
  wrong posture burns the whole visible split for this protocol column;
  the cal side must discriminate BEFORE the read.

## Acceptance

The selection slice reads a stable preference between the pure-NB and
blend postures for prompt_injections (non-tied across re-fits), and the
single test read at the slice-selected posture is taken once, recorded
here, and compared against both the 0.7672 shipped row and the 0.8362
probe datum.
