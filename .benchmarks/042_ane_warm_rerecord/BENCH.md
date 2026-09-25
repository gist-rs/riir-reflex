# Bench 042 — the `@m3-max-ane` row re-recorded: the 650–900 ms p99 was a cold Core ML load inside the timed window, not the Neural Engine

**Status:** IN PROGRESS — diagnosis + fix LANDED (riir-infer `01281a3`, the harness warmup in this commit); the quiet-box after-preload run is armed (`after_preload/`, fires at sustained load < 4.5 + preflight) and the site republish follows it.

## The report

reflex.gist.rs/bench showed the ANE row as e.g. `harness_visibility` p50 **23 ms**, p99 **701 ms**, with the same ~650–700 ms p99 on every short
suite and `code_fixtures` at p50 **695 ms**. The owner's read, which held: any laya cell over 100 ms is suspicious.

## What it was — two cold-load paths, one measurement

The harness records where each lane's slowest sample sits (`latency_extremes`, Issue 020 T8). In the published ANE row
(`afacc3a`, 2026-09-24T07:16Z), **every short suite had `argmax_case = 0` and `first_ms ≈ max_ms` (644–891 ms)**, and the harness
suites carry 12–16 questions, so the p99 index lands on the max (tail support 1). The p99 was ONE cold call.

1. **Stale posture.** That row predates the one-case GPU pre-ramp (`d69f0c7`, 2026-09-25), so its first timed case paid the load.
   Re-running the current harness (`before_preload/`, `ae80abc` binary, preflight
   `PROVENANCE: power=AC Power load=2.77 swap=1078.44M canary=117.0us/best5 powermode=2(high)`) cleared 8 of 12 suites.
2. **Lazy per-bucket load.** `AneEncoder` loaded each length bucket (L64, L128) on the FIRST forward that reached it:
   verify (blake3 over the artifact) → compile cache → `MLModel` load → plan gate, measured at **926.8 ms** by a temporary trace in
   `forward` (reverted). The warmup case warms only its own bucket, so the first case in the OTHER bucket paid it mid-run: emotion
   case 7 (635 ms), sst5 case 1 (641 ms), harness_visibility case 3 (653 ms). `code_fixtures` serves only **4** of 12 cases in-bucket;
   2 of those 4 were each first to reach a bucket, so with n = 4 the p50 index lands on a cold call (986 ms in a code-only run).
   Warm, a code question is 40–58 ms on ANE (a throwaway probe over the cal cases, 3 reps).
3. **A warmup that warmed nothing.** The pre-ramp tried only `cases[0]` and silently swallowed an ANE bucket refusal, so a suite whose
   case 0 is over the bucket (prompt_injections:0, code:embed.rs:0) had no warmup at all.

## The fix

- riir-infer `01281a3`: `AneEncoder::preload()` loads every bucket and runs one pad-only forward through each; `RiirAgent::load_ane`
  calls it. This is a serve fix, not only a bench one: `serve.rs` builds the agent through `load_ane`, so a real first request per
  bucket no longer pays ~0.9 s either (startup pays it instead).
- `src/harness/runner.rs`: the pre-ramp walks to the first SERVABLE case instead of giving up on a bucket refusal.

Check run (code_fixtures + prompt_injections + emotion, LOADED box, a sibling's 577%-CPU MiniCPM run at load 16–17; NOT quotable):
max per suite **97 / 144 / 54 ms** where the before-preload run read **696 / 679 / 635 ms**. The cold spikes are gone. The warm
p50s read HIGHER than before (emotion 35 vs 19 ms, prompt_injections 60 vs 23 ms), which at load 17 is the box, and that is why the
publish waits for the quiet-box run.

## Verdict on the other > 100 ms cells (the Metal rows)

They are Metal and they are real workload, not a CPU fallback:

- `meta.laya_device` = `metal (LAYA_DEVICE or the build's macOS default)` on the `@m3-max-metal` row. The constructor refuses a
  silent CPU fallback loud. A CPU-posture engine reads ~700 ms on a single arena spot (reflex-site `68f056d`), an order of
  magnitude off these.
- `typed_decisions` (143–367 ms p50) runs 5 questions per case at seq 188–317. The torch MPS reference on the same box reads the
  same magnitude (142–363 ms), and the 4090 CUDA row reads 59–108 ms. `code_fixtures` (88 ms p50, 272 ms p99) includes 512-token
  cases (torch MPS 63 / 208 ms).
- Python-lane p99s over 100 ms (massive 104, banking77 162, tool_fit 208) are the subprocess IPC round-trip tail of the reference
  oracle, disclosed as such in `meta.laya_python_lane`.

## Remaining

- [ ] quiet-box after-preload run (`after_preload/`) → republish the `@m3-max-ane` row on reflex.gist.rs
