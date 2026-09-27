# Issue 041 — ag_news full-pull volume lever (the one cheap untried gap-closer)

**Status:** OPEN — filed 2026-09-27 at the post-T7 posture (Bench 057/round-2: emotion
0.885 ridge@8, typed 0.4655 oc@2).

## The gap

ag_news: modelless **0.8825** vs laya **0.9500** (−6.8). Every count-model lever
tried so far reads the same number: NB@4 blend 0.8825, NBSVM-ridge 0.885, pure NB
0.8825 — the lane is AT its count-model plateau **at the fetched volume**.

## The lever (proven mechanism, never pulled)

The corpus is **20k of ag_news's 120k train rows** (the T20K pull's per-suite cap;
`.raw/datasets_t20k/ag_news/`). Issue 038's own POC proved the volume law for
count scorers: the SAME scorer went **0.665 → 0.868** just from 64/label to the
4k pull, and the T2 full-pull round lifted emotion/sst5 the same way. NB table
scoring is O(query tokens) — corpus volume costs only build time (the nb_scope
law), so the lane's G2 is untouched.

Expected: 0.90–0.92 at the full 120k (extrapolating the volume curve; NOT
guaranteed to close laya's last 3–5 pt, which are likely word-order/disambiguation
the encoder owns).

## Recipe

1. Fetch: `TRAIN_CAP=120000 python3 scripts/fetch_datasets.sh` — scope it to
   ag_news if the script allows a suite filter (else fetch all; other suites are
   already at their source totals — emotion 16k, prompt 546 — so a bigger cap is
   a no-op for them). BLAKE3-digest the new bytes into
   `.docs/02_protocols/dataset_manifest.md` (the existing law).
2. This CHANGES THE SPLIT BASIS — the cross-host + cross-bench comparability law
   applies: the new bytes are a NEW PROTOCOL COLUMN (the Bench 051→052 precedent),
   not an in-place update. Re-run BOTH hosts (the site's drift gate will refuse
   one-host), republish; keep or drop the old column per the owner's preference.
3. Selection lanes re-run as usual (`--nb-select --ridge-select --head-select`);
   read test once. The nb/ridge ladders now reach 64/8 (the 6199e5e extension) —
   at 6× the corpus the winning scale may move.

## Cost

~20 min fetch (1200 × 100-row pages), one two-host lane re-run (~15 min each),
republish. The dataset dir grows ~6× for ag_news (a few hundred MB).

## Non-lever (recorded)

- prompt_injections' pure-NB read (**0.8362** on test, probe) vs the shipped
  blend 0.7672 stays UNREACHABLE via the selection slice: the slice reads ~.50
  for both polarities (the T1b slice-weakness class, re-confirmed in the round-2
  re-run — nb@1 held). A bigger selection slice for noul-only suites is the
  fix-shaped idea if anyone picks this up; it is protocol work, not a lever.
- sst5/xnli nb ladders extended to 32/64 in 6199e5e: the slice HELD the round-1
  postures (16) — no test read was spent, nothing moved.
