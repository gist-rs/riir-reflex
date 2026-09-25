# Issue 034 — `publish_bench.py` lane WIPE: measured, wall landed, and the mechanism CORRECTED (the wipe lives on the fresh-docs path; the update path preserves)

**Status:** OPEN — filed 2026-09-26; mechanism CORRECTED + wall landed same
day (reflex-site `1bc4bab`). Found by the Issue 032 publish dry-run (four
merged dry-runs inspected programmatically before anything was committed or
deployed; the live bench.json was NEVER left in a wiped state). Owner
decision 2026-09-26 (option 3): keep the live table as-is until full per-host
re-runs exist. **The correction below does NOT reopen that decision** — the
full re-runs remain the plan (fresh preflight-clean latency + the head-select
modelless promotion + the Issue 024 leak columns); what changed is the
publish SHAPE and the urgency around comparison-lane servers.

## The correction (measured 2026-09-26, both paths, against the live table)

The issue as filed attributed the wipe to `host_lane_entry`'s container
("CREATED EMPTY for a new host") and required full-lane re-runs so the docs
carry every lane. **That mechanism was wrong.** `host_lane_entry`'s
`setdefault` returns the EXISTING container for any host the primary already
knows — an update writes its lane slots in place and every undeclared lane
SURVIVES. Re-measured with the real artifacts:

| publish shape | suites | lane cells | hosts | verdict |
|---|---|---|---|---|
| fresh-docs (`044 results.json` as primary) over a live-seeded site | 14 | 14 | 1 | **the wipe** — 123 published slots dropped (all m3/ane/4090 laya + clm/gliner/agentjev + `harness_cache_reuse`) |
| live `bench.json` as PRIMARY + lane-scoped docs as updates | 15 | 85 | 3 | **preserved** — every container in place, modelless refreshed in place, `head_posture` meta law fired |

The dry-run numbers the issue was filed from (15→14 suites, 49→14 lanes)
match the FRESH-DOCS path — `publish_bench.py <doc1> <doc2> <site>`, which is
also the docstring's historical "regenerate" shape. That path replaces the
table wholesale: the output carries only what the docs carry.

## The SECOND real blocker the re-measurement surfaced (drift gate)

The first update-path attempt REFUSED — correctly: `m3-max-metal=0.69 vs
4090-win=0.7933` on `massive_intent_en`. The published m3 modelless is the
PRE-head-select run; the fresh docs carry the head-select promotion. The
Issue 018 T7 law (hosts must move TOGETHER — verified 14/14 bit-identical in
Benches 040/043/044) binds on the publish itself. **This, not lane
preservation, is why both full docs must land in one publish.**

## What landed (reflex-site `1bc4bab`)

- `guard_wholesale_replace` + `lane_inventory` in `publish_bench.py`: a
  fresh-docs publish whose output would DROP any published lane slot
  (suite, host, lane-class triples; `laya` expanded per checkpoint; host
  rows included) REFUSES exit 1 naming the dropped slots. The real-data
  refusal reads: *"would silently DROP 123 published lane slots … Publish
  with the CURRENT data/bench.json as the PRIMARY …"*
- `PUBLISH_BENCH_FULL_REPLACE=1` acknowledges a deliberate wholesale
  replacement with a loud disclosure line.
- The UPDATE path (primary IS the destination file, resolved-path compare)
  never reaches the wall — merge() cannot drop lanes there by construction.
- `host_lane_entry`'s docstring re-pinned to the measured behavior.
- Self-test 17 → **21/21** (refusal arm, env-acknowledge arm, update-path
  bypass arm, laya-checkpoint inventory arm).

## The publish plan of record (supersedes the old task 3)

1. M3 full-lane run (blocked on the sibling's 64K needle run;
   `MAX_LOAD=6` preflight) — `--head-select`, NO `--skip-laya`, `slice_leak`.
2. 4090 full-lane run (blocked 2026-09-26 ~03:25 by the riir-train
   `plan410_stage0_train` job holding ~17.5 GiB VRAM; GPU exclusivity law)
   — `--head-select`, NO `--skip-laya`, `--features slice_leak,laya-riir-cuda`,
   `REFLEX_BENCH_HOST=4090-windows`.
   **The comparison servers are NOT needed for this publish**: under the
   update path the published clm/gliner/agentjev cells ride along untouched;
   their refresh stays a future lane-scoped doc (also safe under the update
   path) when their servers are up.
3. Publish: `publish_bench.py <live bench.json> <m3 full doc> <4090 full doc>
   <site>` — live-as-primary, both docs as updates, no env needed, no wipe.
4. CF deploy + live verify + README rows (Issue 032 tasks 2–4) → close
   032/034.

## Remaining tasks

- [ ] M3 full-lane preflight-clean re-run (blocked on the sibling's 64K
      needle run ending; `MAX_LOAD=6` preflight gate)
- [ ] 4090 full-lane re-run (blocked on the plan410 training job ending —
      GPU exclusivity, not queueing)
- [ ] The publish (live-as-primary + both docs) + CF deploy + live verify +
      README rows (Issue 032 tasks 2–4)
- [ ] `harness_cache_reuse` reappears via the M3 doc's laya lanes — verify
      on the publish
- [x] The structural guard (fresh-docs wall + env opt-out) — landed
      `1bc4bab`, pinned 21/21
- [x] Mechanism correction recorded (this rewrite)
