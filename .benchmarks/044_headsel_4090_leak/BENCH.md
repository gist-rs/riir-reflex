# Bench 044 — Issue 032 + Issue 024: the 4090 leg re-run at HEAD with the leak scan (the label fix + the 4090 leak columns in one run)

**Status:** LANDED 2026-09-26 · modelless lane, deterministic, accuracy-only
claims · head-select posture at the FIXED ladder label · `slice_leak` scan
armed (Issue 024's 4090 half) · 14/14 accuracy bit-identical vs BOTH prior
runs (Bench 040's M3 reference and Bench 043's 4090 run — the third
cross-host confirmation of the head-select promotion numbers).
⚠ **The latency cells in this run's results.json are BOX-INVALIDATED
(UNJUDGED Windows box_state + contention-shaped p50s vs the incumbents)
and were never published** — the owner-decided landing (option 3,
`.issues/034`) keeps the live table at Bench 041 until preflight-clean
FULL-lane re-runs exist; the derived `results_publish.json` in this dir
(the latency-carry experiment) is SUPERSEDED and kept only as the dry-run
record.

## Why a re-run (Bench 043's own operating note asked for this)

Bench 043 ran at `0418d33`; the label fix `d727196` (RunMeta `head_posture`
string: `0/0.5/1/2` → the shipped `0/0.25/0.5/1`) landed AFTER it, so the
published merge would have carried two different posture strings across
hosts in one table. This run re-took the 4090 leg AT the fix, and armed
`--features slice_leak` in the same invocation — Issue 024's standing gap
("the 4090 host's modelless rows need their own slice_leak-enabled run")
closed by the same pass. One run, both issues.

## The run

- Host: `4090-windows` (REFLEX_BENCH_HOST) — uname is `shikuwa` (the
  Issue-018-T5 override law; without the env the run writes `unknown`,
  which is not a merge key the site knows).
- Reflex `d727196` — the same HEAD the M3 works from, synced via git
  bundle (`git fetch` from this box was hanging on GitHub reachability;
  bundle-over-scp is the recorded workaround, prerequisite `0418d33`
  verified on the remote before the pull).
- Command: `harness.exe --head-select --skip-laya --out
  .benchmarks/044_headsel_4090_leak`, release, `--features slice_leak`
  (build `--offline` — every dep cached from the Bench 043 build).
- Posture: `head_posture: "ON — cal-selected per suite (ladder
  0/0.25/0.5/1, …)"` — the corrected string, byte-confirmed in this
  run's meta.
- Verdict: `PASSED WITH ABSENCES — 14 suite(s), 1 absence` (the absence
  is `harness_cache_reuse`, LLM-lane-only under `--skip-laya`, expected).
- Selections (from `head_selection.selected`): sst5 → 0.5, massive → 1.0,
  banking77 → 1.0, every other suite → 0 — IDENTICAL to Bench 043's and
  Bench 040's, per candidate row.

## Accuracy: the third cross-host bit-identity

14/14 modelless accuracies equal to Bench 043's 4090 run AND Bench 040's
M3 reference (verified programmatically per suite, zero mismatches):
banking77 **0.6840** and massive_intent_en **0.7933** are now three-run,
two-host facts (arm64-macOS ×2, x86_64-Windows ×1).

## Box state (meta.box_state)

`power source unreadable — UNJUDGED` — the Issue 021 T7 stamps are macOS
probes (`pmset`/`sysctl`), absent on Windows; every field `None` by
design ("None — UNJUDGED, never a green"), recorded here as the honest
degradation. Bench 044's LATENCY cells are therefore unjudged, and they
are not the claim: the run's purpose is accuracy + the leak scan. (The
4090 box state for these runs is recorded manually per BENCH.md when it
matters: Bench 043 noted a sibling cargo test concurrent; this run's own
p50 column sits at 0.09–0.57 ms across suites, consistent with an idle
box, but the published tables keep latency from the preflight-clean
lanes only.)

## Issue 024 — the 4090 leak columns (first slice_leak-enabled run on this host)

| suite | exact | near | acc | acc_deleaked | delta |
|---|---|---|---|---|---|
| ag_news | 1 | 26 | 0.5100 | 0.5067 | −0.33 pt |
| banking77 | 0 | 17 | 0.6840 | 0.6832 | −0.08 pt |
| massive_intent_en | 4 | 17 | 0.7933 | 0.7814 | −1.20 pt |
| prompt_injections | 0 | 2 | 0.4828 | 0.4737 | −0.91 pt |
| sst5 | 1 | 0 | 0.2167 | 0.2170 | +0.04 pt |
| emotion | 0 | 0 | 0.2825 | 0.2825 | 0.00 pt |
| xnli_en | 0 | 0 | 0.3467 | 0.3467 | 0.00 pt |
| typed_decisions | — | — | 0.3190 | — | not_applicable (templated) |
| 6 harness families | — | — | — | — | out of scope (synthetic) |

- **The bound law holds**: every count at or below the Issue 024 probe's
  train-vs-test bound (ag_news 1/26 = 1/26, emotion 0/0, sst5 1/0,
  prompt 0/2, xnli 0/0, massive 4/17 ≤ 13/49, banking77 0/17 ≤ 4/61) —
  verified programmatically against the Finding table.
- **The counts are byte-identical to the M3's T3 oracle arm values**
  (same fetch manifest) — the leak scan is deterministic across hosts
  exactly like the accuracy it annotates.
- **The de-leaked deltas are small and honest**: the headline head-select
  numbers lose at most 1.2 pt (massive) to the leak; banking77's +23.8 pt
  promotion stands at 0.6832 de-leaked. This is exactly the disclosure
  shape Issue 024 designed — headline comparable, de-leaked column beside
  it. The T5 write-up still waits on the M3 + laya lanes (the M3's own
  leak-enabled run is queued behind the sibling's needle run; the write-up
  covers all lanes per its task text).

## What remains for Issue 032 (unchanged shape, one leg closer)

1. The M3 leg at the same posture + `--features slice_leak` —
   preflight-clean (queued behind the sibling's 64K needle run; one run
   now serves BOTH issues' M3 halves: latency cells for the merged
   publish + the M3 leak columns).
2. The publish (merged lane-update through reflex-site
   `publish_bench.py`, primary = the live bench.json, updates = the two
   head-select docs) + the manual CF deploy + live verify.
3. README Results rows + close-out.
