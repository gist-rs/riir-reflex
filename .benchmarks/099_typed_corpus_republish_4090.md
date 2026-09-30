# Bench 099 — the Issue-057 corpus republish (4090-windows): the typed timing cell finally refreshes

**Status:** RECORD 2026-09-30 — Issue 057 task 7 EXECUTED. The typed_decisions
4090-win latency cell on reflex.gist.rs now measures the pool it sits beside:
p50 **0.917 ms** (was 0.632, carried from the pre-lift 800-row corpus),
`corpus_digest fnv1a64-2fe9ec3132dbc924` stamped, `source_run 021e1ae`
(post-landing — the `7e03117` digest stamp is ON the published cell). Accuracy
bit-identical to the incumbent (0.5725), gate cells identical to the published
posture (calibrated abstain 0.717 / selacc 0.6537 / n 566). Published in
reflex-site `e55a9ab` via `PUBLISH_BENCH_CORPUS_RESET=typed_decisions`.

## The run

```
REFLEX_BENCH_HOST=4090-windows CARGO_TARGET_DIR=/tmp/plan099_typed \
  cargo run --release --bin harness -- \
  --skip-laya --nb-select --oc-select --ridge-select \
  --suites typed_decisions --out .benchmarks/099_typed_corpus_republish_4090
```

Engine `021e1ae` (develop HEAD; the only code delta vs the published `d4051c8`
is `7e03117`'s `LaneResult.corpus_digest` field — measurement-inert, and the
bit-identical accuracy is the proof). Posture = the published baseline
(Bench 096/097 verbatim: canonical pool `.raw/datasets`, heads OFF, genome off,
cascade absent, `--gate-fit-calibrated` promoted default). Single-suite scope
is deliberate: the publish is a lane-scoped update and declares exactly the
lane it refreshes.

**Results:** typed acc 0.5725 · readout ECE 0.0993 · p50 0.917 ms · p99
2.732 ms · calibrated abstain 0.717. The 0.917 reading sits beside the issue's
quotable-range reads on the 1200-row pool (0.764–0.815 on the M3) — this box
reads slightly higher and its verdict is UNJUDGED (below), so no
cross-host timing claim is made; the cell is per-host by construction.

## Box state (the Issue-021 law — this box's harness has no probes, so the
provenance is recorded here, manual)

2026-09-30T13:04Z · AC power (desktop, plugged) · CPU load 12.7%
(Get-CimInstance LoadPercentage) · RAM 10.1/31.8 GiB used at session start ·
GPU 660 MiB / 13% util (the sibling dq run had just ended — no compute consumer
during the run; the modelless lane is CPU-only anyway) · release profile ·
`latency_quotable: null` on the published cell is the honest unjudged stamp
(the wall's loud note named it: `4090-win@021e1ae`).

## What the publish changed — and the two defects its first live use caught

The stale-cell problem was on BOTH hosts (both cells carried the pre-lift
timing from `d4051c8` with no digest). This run refreshes the 4090-win cell.
The **m3 primary cell (0.517 ms) is intentionally untouched** — it refreshes
only from an m3-host run with the same ack; that is the remaining half (below).

Both defects are reflex-site `e55a9ab`, both found by the first live run of
the landed flow (the 057 landing's own tests were green — 56/56 — because
every arm exercised the same-host update shape):

1. **The corpus ack adjudicated the PRIMARY cell for an extra-host update.**
   `apply_lane_carry` read the update cell as `s.get("modelless")` off the
   MERGED row — for a non-primary host that slot is still the primary host's
   (digest-less) cell; the update lands in
   `extra_host_lanes['4090-win'].modelless`. Result: a fresh post-landing run
   refused as "carries no corpus_digest". Fix: the ack resolves the UPDATING
   doc's host slot (the same slot vocabulary the carry loop already used),
   per-host incumbent comparison for the stale-equal arm. New self-test arm
   `case_corpus_reset_ack_extra_host_shape` pins both directions — **57/57**
   (52 pre-existing + 4 from the 057 landing + 1 new, all pre-existing arms
   unmodified).
2. **The republish wrapper's env-clean predates the ack.**
   `republish_bench.sh` step 1 strips `PUBLISH_BENCH_LANES` /
   `PUBLISH_ALLOW_SAMPLE_MISMATCH` from the merge-law self-test env but not
   the new `PUBLISH_BENCH_CORPUS_RESET` — the ack leaked into the self-test,
   whose fixtures publish no such suite, so the stale-ack refusal killed step
   1 before the real publish. One `-u` joined the env-clean line (the
   documented reason for that line, extended).

## Publish gates (all green)

publish self-test 57/57 · chart render smoke · the real publish wall (unjudged
note, no refusal) · pairing gate · docs-mirror parity · bench-page smoke in
headless chromium (incl. "4090 scope renders exactly the 59 4090-win cells").

## What remains

- [-] **m3 primary cell refresh** — the same run + ack from an M3 session
  (post-landing reflex HEAD); the incumbent there is the digest-less primary,
  so the ack's stale-equal arm cannot block it. Until then the m3 cell keeps
  its LANE_CARRY provenance note — disclosed, not silent.
- [-] **CF deploy** — reflex-site `e55a9ab` is pushed; this box has no
  Cloudflare credentials (wrangler not logged in; Node 20 vs wrangler 4's
  Node-22 floor is moot at wrangler 3.114, which runs). Deploy from a
  credentialed context (the M3): `cd reflex-site && npx wrangler deploy`.
