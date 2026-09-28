# Issue 052 — typed_decisions train-split fetch cap: 400 of 1200 train rows never fetched (A0 0.4655 → 0.5725 measured on the full pool)

**Status:** DISTILLED — pending owner decision — filed 2026-09-28 by the
riir-instinct 581 lane (cross-repo evidence; no reflex code touched).

## The finding (measured)

`scripts/fetch_datasets.sh` caps the typed_decisions TRAIN split at 800
rows of a **1200-row train split** (`fetch_suite typed_decisions
LocalLLaMA%2Ftyped-decisions all train 800`) while fetching ALL 400 test
rows (`cap=all`). The train split's workflow blocks are ordered
agent_trace(300) · customer_service(300) · invoice_processing(300) ·
security_incidents(300) — so **the entire security_incidents train block
(offsets 900–1199) plus invoice rows 200–299 sit behind the cap**, and
the modelless lane's corpus guard self-docs security_incidents at every
run ("1 option label(s) with NO train docs in the corpus pool →
self-doc fallback"). The published A0 0.4655 is a pool-scoped number,
not a lane limit.

## The measured upside (riir-instinct Bench 015, A0′ arm)

Re-running THIS repo's own published-posture lane (`run()` through the
instinct arena seat, identical knobs: head/nb/ridge select + oc armed,
registry caps, genome off) on an extended datasets dir — reflex's
canonical envelope byte-copied + the four unfetched train pages appended
— reads:

- **A0′ 0.5725 vs the published 0.4655: +10.7 pt** from data the
  dataset already declares (train total 1200, `num_rows_total` printed
  on every page).
- Per-workflow (500 q each): security_incidents 0.242 → 0.620 ·
  invoice_processing 0.528 → 0.614 · customer_service 0.660 → 0.626 ·
  agent_trace 0.432 → 0.430. (Two small regressions within the
  re-balanced evidence mix; the pool is +10.7 pooled.)
- Snapshot integrity: pages 000–007 + test re-fetched fresh and
  byte-compared — 12/12 identical to the on-disk envelope; dataset repo
  sha `f7a2487edd7a043a5441a5e9ccc7fe5ddbd9ebe8`. Train∩test
  disjointness: 0 collisions over three hashes (state-key, full-row,
  ids).

## The fix (reflex-owned; not landed here)

- `fetch_datasets.sh`: the typed_decisions train row's cap 800 → 1200
  (or `all` — the API reports 1200) + the manifest's typed section
  updated with the four new page digests
  (`d67bf234…` / `10ffa982…` / `6e588068…` / `f5186ace…`).
- Downstream pins move with the corpus: the typed_decisions modelless
  rows in the harness tables/site (a re-published bench), the digest
  floors that name the train envelope, and any corpus-digest pin keyed
  on the old 800-row bytes. The cal front is SAFE — the registry cal
  slice is the FIRST cal_cap train rows and the new pages append behind
  row 800, so cal bytes are unchanged.
- Consumers of the published number: the reflex-site typed row and the
  instinct A0 pins (instinct Bench 015 already records both postures;
  their lane re-pins when the site republishes).

## Honest scope

- The number is ONE measured read on a loaded box (accuracy
  deterministic; the arena's box line read load 8.06, quotable false —
  the accuracy claim stands, latency rows don't).
- The specialist lane (instinct) benefits independently: its Bench 614
  artifact trained on the full pool. The two fixes are independent —
  this one is pure modelless upside for reflex's published lane.
