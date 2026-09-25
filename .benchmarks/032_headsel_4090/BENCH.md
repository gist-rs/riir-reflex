# Bench 043 — Issue 032, the 4090-windows leg: modelless `--head-select` full-harness run (the accuracy half of the arena republish)

**Status:** LANDED 2026-09-26 · modelless lane, deterministic · head-select
posture · cross-host determinism gate PASS vs Bench 040's M3 reference (14/14
suites bit-identical accuracy).

## The run

- Host: 4090-windows (x86_64), LAN-dead → over Tailscale; GPU idle (20%),
  one sibling riir-train cargo test running concurrently (CPU-side, no
  interaction — the modelless lane is single-threaded deterministic CPU
  compute, and the claim is accuracy, not latency).
- Head: reflex `0418d33` (the head-select code from `69a6eae` / Bench 040;
  the two later docs commits change no engine code).
- Command: `harness.exe --head-select --skip-laya --out
  .benchmarks/032_headsel_4090` — modelless-only per the issue's task row
  (parity with the `041_t11_4090_modelless` posture; the laya lane cannot
  run on this host).
- Verdict: `PASSED WITH ABSENCES — 14 suite(s), 1 absence` — the absence is
  `harness_cache_reuse` (LLM-lane only, expected under `--skip-laya`).
- Datasets: `.raw/datasets` (8 suites, the 2026-09-24 fetch) — same
  blake3-manifested fetch as the M3.
- `head_posture: "ON — cal-selected per suite …"` in RunMeta.

## Determinism gate (the issue's pairwise-accuracy law)

4090 accuracy vs the M3 Bench 040 reference (`results_head_select.json`):

| suite | 4090 | M3 (b040) | agree |
|---|---|---|---|
| ag_news | 0.5100 | 0.5100 | ✓ |
| banking77 | **0.6840** | **0.6840** | ✓ |
| code_fixtures | 0.3333 | 0.3333 | ✓ |
| emotion | 0.2825 | 0.2825 | ✓ |
| harness_permissions | 0.4167 | 0.4167 | ✓ |
| harness_routing | 0.4375 | 0.4375 | ✓ |
| harness_sensitivity | 0.4000 | 0.4000 | ✓ |
| harness_tool_fit | 0.5000 | 0.5000 | ✓ |
| harness_visibility | 0.3750 | 0.3750 | ✓ |
| massive_intent_en | **0.7933** | **0.7933** | ✓ |
| prompt_injections | 0.4828 | 0.4828 | ✓ |
| sst5 | 0.2167 | 0.2167 | ✓ |
| typed_decisions | 0.3190 | 0.3190 | ✓ |
| xnli_en | 0.3467 | 0.3467 | ✓ |

**14/14 bit-identical across arm64-macOS and x86_64-Windows.** The
head-select promotion numbers (banking77 +23.8 pt, massive +10.3 pt at sel
scale 1; every other suite at its baseline) are cross-host facts.

## What remains for Issue 032 (this bench does NOT close it)

1. The M3 leg at the same posture — preflight-clean (the M3 is under a
   sibling's 8-core GPU run all session; the m3 leg waits for the quiet
   box).
2. The publish (one merged lane-update through reflex-site
   `publish_bench.py`, carrying the untouched laya/clm/gliner/agentjev
   cells) + the manual CF deploy + live verify.
3. README Results rows + close-out.

## Operating notes (paid for this session)

- Detached `Start-Process` children die with the SSH session on this
  box — a `-Wait` invocation inside one SSH call is the reliable shape;
  the foreground `cargo build --release --bin harness` took 2m02s.
- The 4090's `harness.exe` had been built 09-25 10:30 — before
  `--head-select` landed there — and refused the flag ("unknown flag");
  the sync + rebuild was the prerequisite, exactly the divergence the
  "all repos must be sync on both m3 and 4090" rule exists for.
- ⚠ RunMeta's `head_posture` string says "ladder 0/0.5/1/2" while the
  actual ladder (Bench 040, and this run's own per-suite logs) is
  `0/0.25/0.5/1` — a stale label in the meta string, not a posture
  difference. Worth a one-line code fix when the M3 leg lands.
