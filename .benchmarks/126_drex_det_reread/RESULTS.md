# Bench 126 — Drex BF16/CUDA det re-read under the latency-tail strip (issue 076 T1) + the per-kind cells at the third posture

**Status:** RECORD — the T1 re-measure from `.issues/076` (removed-upstream 073's follow-up
issue, still open here): the BF16/CUDA posture's determinism column re-read under the
`decide_raw` timing-tail strip landed at `cd0248b`, and the FIRST end-to-end exercise of the
T2 per-kind cells (`7162840`) at this posture. Raw evidence: `results.json` + `TABLES.md`
(harness `--drex --skip-laya --suites typed_decisions`, host label `4090-windows`).
**A sibling session's INDEPENDENT second execution of the same T1** (same box, same
posture, host label `shikuwa`; their commit `e7111bc`, unpushed) produced byte-agreeing
cells — preserved as `results.sibling_run.json`; two executions, one verdict. The two
sessions' records collided on the number 126; this directory is the canonical holder
(their dir `126_drex_det_reread_bf16_cuda/` was folded in, their 073 addendum salvaged
verbatim).

> **Scope law:** the open 32K release (`nace-ai/drex-dlm` @ `6c63df2`), never hosted
> "Drex 1.5". Weights CC BY-NC 4.0 / repo code MIT — measurement only.

## Verdict (the T1 answer)

**`determinism_ok = true` (10/10 rerun pairs byte-identical) at BF16/CUDA under the strip.**
Their T5 record (`073`, removed upstream — closure narrative in `HISTORY.md`) flagged
`determinism_ok = false` and attributed it to bf16-CUDA reduction nondeterminism; Bench 125
showed the raw byte-compare measured their clock (`latency_ms` differs per request by
definition), not the decision. This run closes it: the SAME numeric posture that read ✗
reads ✓ with only the timing tail stripped — **there is no measured reduction
nondeterminism on this posture.** The accuracy agrees EXACTLY with their T5 (0.5865,
ece(maxp) 0.1082, readout_ece 0.1929 / readout_brier 0.2663, tokens 191,962 in / 164,821
out) — same slice, same answers surface, only the det column flipped.

## Box state (the provenance law)

`scripts/bench_preflight.ps1` at run start: **`PROVENANCE: power=AC scheme=high load=0
swap=3600MB gpu=1 %, 15556 MiB canary=skipped`** — preflight PASSED (the 15.5 GB GPU
resident set is the lane server itself, disclosure not contention). The GPU-exclusivity
rule held: 604 MiB used before the model loaded, no compute consumer besides the lane.

## Serving posture (the turnkey record)

| piece | value |
|---|---|
| speaker | their Python `serve.py` (the `.raw/drex_serve.py` chdir+argv shim), venv `.raw/drex-env` (torch 2.6.0+cu124, transformers 5.19.0) |
| weights | `nace-ai/drex-dlm` snapshot (~16 GB BF16) at `.raw/drex-model` — 399/399 tensors, **15,556 MiB VRAM resident**, `/health` → `{"status": "ok", "model": "drex-dlm"}` |
| harness | worktree at `7162840` (clean — the box's main checkout carried the 075 engine.rs WIP, since landed upstream as the `284cc98` fix), `cargo build --release --bin harness`, 1m05s |
| datasets | the box's `.raw/datasets_t20k/typed_decisions` was a STALE 8-shard pull (pool 700 < floor 960 — the slice-integrity refusal fired, never a shrunken-corpus row); refreshed byte-exact from the M3's 12-shard slice → pool 1100, digests match Bench 125's test slice (`fnv1a64-287d5f73a11932cd`) |
| env | `REFLEX_BENCH_HOST=4090-windows` (the phantom-host sentinel refused the first attempt — the gate works), `DREX_SERVE_URL=http://127.0.0.1:8000` |

## The three-posture picture (typed_decisions, 400 cases / 2000 questions)

**The sibling run (e7111bc) agrees with this record's run to every quoted digit** — det ✓
10/10, acc 0.5865, choice ece 0.14713 / score 0.247749 / floor 0.121991 in BOTH
results files. A concurrent-session collision (the Issue-825 class) that resolved into
independent reproduction: neither session knew of the other mid-run (their first
preflight REFUSED at load 7.2 — this session's footprint; this session's preflight read
load 0 between our runs), and the agreement is the strongest form of the verdict.

| cell | T5 (BF16/CUDA, pre-strip) | **this record (BF16/CUDA, strip)** | Bench 125 (Q8_0/Metal, strip) |
|---|---|---|---|
| accuracy | 0.5865 | **0.5865** | 0.5855 |
| ece(maxp) | 0.1082 | **0.1082** | — (in results.json) |
| readout_ece / readout_brier (pooled) | 0.1929 / 0.2663 | **0.1929 / 0.2663** | — |
| choice conf cell (n / ece / brier) | — | **600 / 0.1471 / 0.2389** | 0.1460 |
| score conf cell (n / ece / brier) | — | **800 / 0.2477 / 0.2877** | 0.2495 |
| split-half floor (cal/test) | — | **0.1220** (700/700) | 0.1158 |
| determinism | ✗ flagged | **✓ (10/10)** | ✓ |
| server tokens | 191,962 in · 164,821 out | **191,962 · 164,821** | 191,962 · 164,868 |

The per-kind cells agree with Bench 125's Q8_0/Metal posture within 0.0011 (choice) /
0.0018 (score) — the **not-calibrated verdict is now three-posture**: their confidence
LOSES to the split-half conformal-naive floor on typed_decisions at every measured posture
(choice 0.1471 / score 0.2477 vs floor 0.1220 here). The card's disclaimer stands; the
homepage marketing does not.

## Laws carried

- **License:** CC BY-NC 4.0 — comparison lane only; never a teacher, never a product lane.
- **Latency columns quotable at the provenance line above** (preflight PASSED this time —
  unlike Bench 125's PROVISIONAL rows); p50 106.0 ms / p99 167.0 ms (support 5) are
  client round-trips on the same box (loopback), still the cross-lane-latency-disclosure
  class, never compared to in-process µs rows.
- The agentjev lane carries the same det artifact (its `decide_raw` lacks the strip) —
  noted in issue 076, unscheduled.
- The 4090's stale datasets pull is the second instance of the silent-shrink class the
  slice-integrity floor exists for — the refusal fired before any row was minted, exactly
  as designed.

## Files

- `.benchmarks/126_drex_det_reread/results.json` + `TABLES.md`
- Issue: `.issues/076_drex_conf_cells_det_followup.md` (T1 closed by this record; T2
  landed `7162840`)
- Companion records: `.benchmarks/073_drex_lane_t5_suite_pass.md` (the pre-strip first
  posture) + `.benchmarks/125_drex_systemone_lane/` (the second posture)
