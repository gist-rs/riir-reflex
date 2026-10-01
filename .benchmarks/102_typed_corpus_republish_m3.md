# Bench 102 — the Issue-057 corpus republish (m3): the primary typed timing cell refreshes

**Status:** RECORD 2026-10-01 — Issue 057 CLOSED (both halves). The
`typed_decisions` m3 primary latency cell on reflex.gist.rs now measures the
pool it sits beside: p50 **0.782 ms** (was 0.517, carried from the pre-lift
800-row corpus since Bench 078's publish), `corpus_digest
fnv1a64-2fe9ec3132dbc924` stamped, `source_run 19cb043` (post-landing HEAD),
`latency_provenance` null — the timing is its OWN, the LANE_CARRY note is
gone. Accuracy bit-identical to the incumbent (0.5725); slice digests
byte-equal Bench 100's (test `287d5f73…` / cal `0b6e8426…` / pool
`a1679de5…` — the pairing gate's own check). Published in reflex-site
`2b699f0` via `PUBLISH_BENCH_CORPUS_RESET=typed_decisions`; deployed
(CF version `30402050`), live-verified by curl.

## The run (the 099 command mirrored, this box)

```
cargo run --release --bin harness -- \
  --skip-laya --nb-select --oc-select --ridge-select \
  --suites typed_decisions --out .benchmarks/102_typed_corpus_republish_m3
```

Engine `19cb043` (develop HEAD). Posture = the published baseline verbatim
(canonical pool `.raw/datasets`, heads OFF, `--gate-fit-calibrated` promoted
default). Single-suite scope is deliberate: the publish is a lane-scoped
update declaring exactly the lane it refreshes. The repo's own target dir
(warm, built at HEAD minutes before — no sibling session in the checkout).

## Box state (the Issue-021 law)

`scripts/bench_preflight.sh` PASSED, PROVENANCE quoted:

```
PROVENANCE: power=AC Power load=3.90 swap=3831.00M canary=129.2us/best5 powermode=2(high)
```

Settle 6 min on AC (the preflight's own SETTLE_MIN=5 gate — the first
attempt at 0 min on AC was refused, correctly). Swap 3831M carried the
preflight's warn (disclosed, judged ok — canary 129.2 µs best-of-5 against
the 141 µs reference). Harness-recorded box state: AC Power / high /
`latency_quotable: true` at both ends (start load 4.08 → end 3.91), zero
refusals.

**Results:** typed acc 0.5725 · readout ECE 0.0993 · p50 0.782 ms · p99
1.69 ms · tail support 5. Sits beside the issue's quotable M3 reads on the
1200-row pool (0.764 Bench 096 / 0.795 Bench 100 / 0.815 Bench 078) and the
4090's 0.917 — per-host cells, no cross-host timing claim.

## The defect the close-out caught — and the repair that unblocked it

The 4090 half (Bench 099) assumed "its incumbent is the digest-less primary,
so the stale-equal arm cannot block it". **Falsified by the Bench-100 board
restore** (2026-10-01): that publish carried the stale 0.517 onto a cell that
already carried Bench 100's new-pool digest — `_carry_into` re-attaches ONLY
the timing fields, so the merged cell read digest(new pool) + timing(old
pool), internally inconsistent. On that state:

- the ack's stale-equal arm refused (update digest == incumbent digest), and
- without the ack the carry re-attached 0.517 —

i.e. **the m3 primary was permanently unrefreshable by the landed mechanism**.

Repair (reflex-site `8d33e3f`, self-test 67/67): the stale-equal adjudication
keys on whether the incumbent's TIMING is its own. A carried incumbent
(`latency_provenance` note present — the `_carry_into` shape) proves its
timing measured an older corpus by construction; its equal digest stamps the
accuracy merge, not the timing. Only a cell whose timing is its OWN can make
the ack stale — preserving the ack's one-time property (the new arm's second
assertion re-refuses on the refreshed cell). Validated end-to-end against a
/tmp copy of the real `data/bench.json` with Bench 100's doc BEFORE the live
publish; the live publish printed both disclosure notes verbatim.

## Publish gates (all green)

publish self-test 67/67 · chart render smoke · the real publish (both
CORPUS_RESET notes: the carried-timing disclosure + the exemption) · pairing
gate · docs-mirror parity · bench-page smoke in headless chromium (4090
scope renders its 65 cells; M3 scope keeps the 5 ANE rows) · live curl
(p50 0.782 / src 19cb043 / digest stamped / prov null).

## Issue 057 — closed

Both named remainders executed: the m3 primary cell refresh (this record) and
the CF deploy from the credentialed context (`npx wrangler deploy`, version
`30402050`). The harness digest stamping (`7e03117`), the publisher
corpus-mismatch exemption + ack (reflex-site `6e06264`), the 4090 half
(Bench 099, reflex-site `e55a9ab`), and the carried-timing repair
(`8d33e3f`) together close the lane-carry corpus-axis gap: a corpus change
can now refresh published timing on every host slot.
