# Issue 034 — `publish_bench.py` host-keyed lane containers REPLACE a host's lanes on update; the head-select publish cannot land without erasing the incumbent laya + comparison lanes

**Status:** OPEN — filed 2026-09-26. Found by the Issue 032 publish dry-run
(four merged dry-runs inspected programmatically before anything was
committed or deployed; the live bench.json was NEVER left in a wiped state —
every observation ran against a scratch publish or was restored from
`b935412` immediately). Owner decision 2026-09-26 (option 3): **keep the
live table as-is; fix the shape via full per-host re-runs when the box
quiets, not publisher surgery.** The publisher work landed in reflex-site
this session is the additive half only (`LANE_FACT_META_KEYS` +
`LANE-CARRY`, both pinned, both inert without a modelless-lane update
carrying those meta keys — which none will until this issue's shape is
decided); the replace-shaped core is untouched.

## Finding (measured, four dry-run publishes)

`merge()`'s per-host lane container is the SUITE ROW ITSELF for the
primary host and `extra_host_lanes[host]` for every other host — one
container per (suite, host), written wholesale:

```python
def host_lane_entry(suite, host):
    if host == phost:
        return suite  # the primary's lanes live on the row itself
    return suite.setdefault("extra_host_lanes", {}).setdefault(host, {})
...
entry["modelless"] = em          # ← writes the container's modelless slot
entry.setdefault("laya", {})[lk] = lv   # ← but the container itself is
                                 #    CREATED EMPTY for a new host…
```

For a host the primary does not know, `setdefault` creates a FRESH
container and the update doc's lanes are ALL it ever contains. A
modelless-only update doc for a host whose published lanes live in
`extra_host_lanes` therefore lands as a container holding ONLY
`modelless` — and the drift gate then compares the primary's modelless
against the new host container's, while the host's old container (with
its laya + clm + gliner + agentjev lanes) is REPLACED in the same
assignment chain.

Measured on the real data (live `b935412` bench.json as primary):

| | live (Bench 041) | after head-select dry-run |
|---|---|---|
| suites | 15 | 14 (`harness_cache_reuse` gone — its ONLY lanes were laya, and its host container was replaced by a modelless-only one, leaving the suite empty and dropped) |
| lanes total | 49 across 3 hosts | 14 (modelless only) |
| m3 laya + py lanes | 15 suites × 2 checkpoints | **gone** |
| 4090 laya lanes | 15 suites | **gone** |
| clm / gliner / agentjev | 15 / 15 / 14 rows | **gone** |
| m3-max-ane device-variant row | 12 laya rows | **gone** |

The same replace-shape also fired in the Bench 041 landing itself
(m3-max-ane's row facts were replaced by the 027 publish) — it just
happened to be full-run-for-full-run then, so nothing was lost.

## Why it bit now

Every prior update doc carried a host's FULL lane set (or the host was
new). Issue 032's docs are the first MODELLESS-ONLY updates against
KNOWN hosts (`--head-select --skip-laya`), so the replace-shape met a
mixed container for the first time.

## What landed anyway (reflex-site, this session — all additive, 17/17 self-test)

- `LANE_FACT_META_KEYS = ("head_posture", "latency_carried")`: an update
  contributing the modelless lane refreshes those host-row facts (the
  `laya_python_lane` lane-fact class). Inert until a doc carries the keys.
- `LANE-CARRY` (`LANE_CARRY`/`apply_lane_carry`): when the merge DOES
  replace a carried lane (modelless), the lane's five TIMING fields are
  inherited from the host's pre-merge incumbent and stamped
  `latency_provenance` — so a future head-select publish cannot silently
  replace validated latency cells with box-invalidated ones (this
  session's second finding: Bench 044's 4090 p50s were ~2× the
  incumbents, contention-shaped, `box_state` UNJUDGED on Windows).
  `merge_refusing` in the test file now mirrors `main()`'s exact shape
  (snapshot → merge → apply carry) so the post-merge laws are tested.
- Both laws are PINNED in both directions
  (`case_modelless_lane_facts_refresh_on_update`,
  `case_lane_carry_keeps_incumbent_timing` + the negative arms).

## The decision on record (owner, 2026-09-26, option 3)

**No publisher surgery. The live table stays at the Bench 041 state.**
When the box quiets, take preflight-clean FULL re-runs — one per host,
ALL lanes — and publish each as the wholesale host replacement the
merge already models:

1. M3: `cargo run --release --features slice_leak --bin harness -- \
   --head-select --out .benchmarks/<new>` (NO `--skip-laya`: modelless +
   laya-riir + laya-python; preflight-clean, PROVENANCE quoted;
   `REFLEX_BENCH_HOST` unset → `m3-max-metal` via uname? NO — uname is
   `shikuwa`; the env MUST be set or the run publishes as `unknown`).
2. 4090: same full posture with `laya-riir-cuda`
   (`REFLEX_BENCH_HOST=4090-windows`), comparison lanes when their
   servers are up (clm/gliner/agentjev — else their next window re-joins
   via a lane-scoped doc AFTER the container shape is fixed or accepted).
3. Publish: live bench.json as primary, the two full docs as updates —
   accuracy AND latency both fresh, no carry, no wiped lanes (the
   containers' new content = the docs' full content).

## Remaining tasks

- [ ] M3 full-lane preflight-clean re-run (blocked on the sibling's
      64K needle run ending; `MAX_LOAD=6` preflight gate)
- [ ] 4090 full-lane re-run (its box is idle; queued behind the M3 leg
      so the two publishes land together)
- [ ] The publish + CF deploy + live verify + README rows (Issue 032
      tasks 2–4, unchanged)
- [ ] THEN the structural fix is optional: if wholesale full-run
      replacement is the standing convention, re-pin
      `host_lane_entry`'s docstring to say so and add a loud
      `note:` when an update doc's lane set is a strict subset of the
      container's (the mixed-container case), so the next lane-scoped
      update meets a wall instead of a silent wipe
- [ ] `harness_cache_reuse`'s disappearance from the published table
      (LLM-lane-only suite; it reappears the moment a doc carrying laya
      lanes lands — verify on the full-run publish)
