# Issue 043 — prompt_injections: the selection slice cannot see the pure-NB polarity

**Status:** RESOLVED 2026-09-27 (measured) — the premise split in half under
measurement: the stratified selection lanes ALREADY discriminate the polarity
candidates (that half of the issue was stale — it described the old positional
registry slice), but the selected posture's single test read is byte-identical
to the shipped row, so the expected gain does not materialize through the
shipped term. Full verdict at the bottom; record in HISTORY.md.

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

## The measurement (2026-09-27, HEAD `767c577`, isolated worktree — the
sibling's in-flight manifest edits blocked the main checkout)

**Probe first** (`scripts/issue043_slice_probe.py`, plain-MNB mirror): the
train mirror is NOT label-grouped — injections appear from index 4; the
positional prefixes carry 15/37/54/98 injection cases at 100/200/300/400.
Positional prefixes FLIP the preferred polarity by region: prefix100 →
nb1, but prefix200/300/400 → **nb0 — the WRONG posture** (mirror test
0.32–0.34). **Candidate 1 is refuted as dangerous, not merely weak.**
Label-stratified round-robins discriminate with the right sign at every
budget (100/200/300 all select nb1). The mirror also validates its scorer
against the 038-POC datum: full-train MNB reads 0.8017 ≈ the recorded
0.802.

**Real engine** (`--nb-select`, stratified selection slice 100, corpora
pool 446 → 346 docs at cap 64/label; run at `/tmp/reflex_issue043_run`):

| candidate | sel-slice acc |
|---|---|
| scale 0 (off) | 0.5000 |
| noul-yes Some(0), observed-laplace (all scales) | 0.3900 |
| noul-yes Some(0), fixed-1 | 0.49–0.50 |
| **noul-yes Some(1), observed-laplace (all scales)** | **0.6100** |
| noul-yes Some(1), fixed-1 | 0.5100 |

Selection: `scale 1 α observed-laplace noul-yes Some(1)` — clears the
house margin (0.61 vs 0.50 = 0.11 > 0.05), STABLE (identical 0.61 at
every scale — the noul pick is the margin's SIGN, scale-invariant; and
observed-laplace beats fixed-1 at every scale). **Single test read at the
selected posture: acc 0.7672 — byte-identical to the shipped row.** The
cal preference does not transfer: the polarity term's margin-sign picks
coincide with the shipped engine's picks on test.

Corpus-cap axis (`--cal-select-cap --nb-select`, `/tmp/reflex_issue043_run2`):
FLAT — every cap (8..512) reads 0.5000 on the slice, cap stays 64, test
again 0.7672. The corpus cap is irrelevant to the polarity question.

## Verdict

1. Candidate 2 (stratified sampling) was the real fix and ALREADY SHIPS —
   the `--nb-select`/`--cal-select-cap` lanes select on
   `stratified_selection_slices` (label round-robin over the pool), and
   on that slice the polarity candidates separate cleanly (0.61 / 0.50 /
   0.39). The issue's premise was true only of the OLD positional
   registry slice (the T1b era, pre-selection-lane).
2. The gain is NOT reachable through the shipped term: the selected
   posture's test read is 0.7672 == shipped. The 0.8362 probe datum came
   from a DIFFERENT scorer shape — plain MNB argmax with priors over the
   full train corpus — while the engine's noul polarity is
   `σ(margin/n_tokens)` of one-vs-rest in-scope log2-odds with NO prior
   term, and its pick is the margin's sign. At the engine's reachable
   corpus (≤64/label), the mirror's own MNB reads only 0.63–0.65 on test
   — BELOW the shipped 0.7672 — so even the different-shape lever's
   premise is doubtful at this corpus size.
3. Reopen path (a NEW lever, not protocol work): a `noul_full_posterior`
   polarity term (full multinomial argmax incl. priors) — only worth
   building if the corpus cap for this suite also rises, since the
   mirror's full-corpus advantage (0.80) needs documents the 64/label cap
   throws away; and the cap axis measured FLAT on the drafter corpora,
   so the rise would be an NB-table-only change. Expected gain vs
   shipped: unproven, mirrored evidence says small and possibly negative.

Acceptance disposition: the stable cal preference EXISTS (Some(1),
0.61 > 0.50, stable across re-fits) and the single test read was taken
once and recorded (0.7672, == shipped, vs the 0.8362 probe datum) — the
acceptance is satisfied with a NEGATIVE gain verdict; the issue closes
with the slice-hypothesis refuted in its strong form and the term-shape
hypothesis recorded as the reopen path.
