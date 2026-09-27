# Bench 057 — CLM T-Rex re-run under OUR protocol (Issue 019 T4)

The T-Rex harness (their `examples/t_rex/run.py` @ `cca045ff`, Apache-2.0)
re-run with the CLM arm under the 027 window laws — the vendor reference cells
in `results/clm_realtime.json` are THEIRS and stay reference-only; every number
below is OURS. Their Jev arm needs a TypeSafe API key this box does not hold;
the Jev column stays their frozen reference, never re-measured here.

## Posture

- Box: `4090-windows` (RTX 4090 24 GB, WDDM) · 2026-09-27 08:46–09:17 UTC-local
- Their stack, our docker (`scripts/clm_serve_4090.sh`): vLLM
  `0.29.1rc1.dev397+ga8d1aa9c9` (image `vllm/vllm-openai:nightly`, sha
  `4cbfd34a…` — the SAME image as the Bench-034 window) over `Qwen/Qwen3-8B`
  (HF snapshot downloaded fresh 2026-09-27, 15.26 GiB, 5/5 shards verified) +
  `clm-serve` over `CLM_v0.1-8B.pt` (75557149 bytes, sha256
  `b2b4a8c9c2d39263eff78a351eb909a342ce9b3bf21a3f07c1d1bf15f1c4eda5`) from
  their repo @ `cca045ff`.
- Window laws held (027): SERIALIZED passes (nothing else touched the GPU —
  the only other compute process seen during preflight was a transient cargo
  test binary that exited before boot); `--gpu-memory-utilization 0.72`; ONE
  fixed warmup request first (54 input tokens, absorbed the
  `usage.input_tokens = 0` first-request class); determinism pin GREEN (8/8
  byte-identical repeats, re-verified again at the resume invocation);
  same-box latency law (no CLM-4090 vs laya-M3 comparison published); the
  "9× vs Jev" figure stays out of our tables.
- Harness: `python examples/t_rex/run.py --model clm` — 5 seeds × 60 s,
  real time, shield on (their table posture), inflight 6, original course,
  `labeled` prompt. Python 3.12 venv (`.raw/clm-env`, requests + httpx only).
- Box state: start `mem_load 86–87% · phys_free 3.9–4.4 GB · commit
  76.8 GB limit / 11.2–13.2 GB avail` (the box carried unrelated host RAM
  pressure through boot — disclosed; the realtime loop kept
  `host_stall_seconds_dropped = 0.0` on all 10 runs, so run quality held);
  VRAM under load 19904/24564 MiB; GPU 45–48 °C. Desktop on mains (AC), no
  battery axis.

## OUR cells — shield-on (their table posture)

Raw: `clm_realtime_4090.json` (per-seed rows verbatim from their harness).

| | CLM v0.1 (OURS, 4090) | CLM v0.1 (their reference @ cca045ff) |
|---|---|---|
| survived 60 s (5 seeds) | **5/5** | 5/5 |
| deaths | 0 | 0 |
| best score | 697.0 ×5 (capped) | 697.0 ×5 (capped) |
| decisions per 60 s (mean) | 2441.6 | 3341.8 |
| agreement with planner (mean) | 0.749 | 0.658 |
| answer-to-effect latency p50 (median of seeds) | 48.9 ms | 16.5 ms |
| model-side p50 (median of seeds) | 34.5 ms | 2.6 ms |
| failed calls | 0 | 0 |
| shield interventions (shield+arrival+emergency) | 3050 | 4883 |
| host_stall_seconds_dropped | 0.0 (all seeds) | 0.0 (reported 0.0) |

**The cold→warm cache trajectory is the story behind the latency column.**
Their headline p50 (16.5 ms) is the WARM steady state of `clm-serve`'s
VectorArena; our invocation boots cold and pays the encoder for every distinct
situation once:

| seed | decisions | agreement | latency p50 | model p50 | arrival_saves |
|---|---|---|---|---|---|
| 0 | 1491 | 0.778 | 66.6 ms | 55.5 ms | 277 |
| 1 | 2126 | 0.798 | 49.8 ms | 38.5 ms | 397 |
| 2 | 2543 | 0.778 | 48.9 ms | 34.5 ms | 498 |
| 3 | 2673 | 0.720 | 33.4 ms | 23.4 ms | 638 |
| 4 | 3375 | 0.669 | 16.7 ms | 3.4 ms | 926 |

Seed 4 — the cache warm (the arena's 512-pool holds every distinct situation
of the five-seed run) — reproduces their headline almost exactly: 16.7 vs
16.5 ms p50, 3.4 vs 2.6 ms model-side, 3375 vs ~3500 decisions/seed. `/health`
post-run: `hit_rate 0.9749` (120632 hits / 3100 misses, 2951 of 220088 pool
slots used). Read the vendor latency row as warm-state; a cold boot pays
~4× on the first minute.

## OUR cells — no-shield (secondary; the model on its own)

Raw: `clm_realtime_noshield_4090.json`. Ran second, so the cache was warm —
this is the steady-state posture (p50 16.7 ms / model 4.0 ms from seed 0).

| | OURS no-shield | OURS shield-on | their reference (shield-on) |
|---|---|---|---|
| survived | **0/5** (23 deaths) | 5/5 (0 deaths) | 5/5 (0 deaths) |
| mean best score | 201.6 | 697.0 | 697.0 |
| decisions (mean) | 3123.4 | 2441.6 | 3341.8 |
| agreement (mean) | 0.885 | 0.749 | 0.658 |
| latency p50 (median) | 16.7 ms | 48.9 ms | 16.5 ms |
| model p50 (median) | 4.0 ms | 34.5 ms | 2.6 ms |
| errors | 0 | 0 | 0 |

The survival number measures the COMBINED system (their README's own
convention): the shield — an unsafe-labelled answer replaced by the model's
most probable safe action + the emergency check — is load-bearing, exactly as
their table implies and their write-up quantifies (their CLM run: 4883
interventions over five minutes). Ours: 3050 interventions, and without the
shield the model dies 23 times in five minutes. The agreement row measures the
model. Their reported CLM-vs-Jev disagreement profile (CLM prefers an
action-sounding option over the one marked *Best*) reproduces qualitatively —
agreement never reaches the Jev column's 0.987 in any of our runs.

## Determinism

8 identical `POST /v1/systemone` requests → 8 byte-identical responses
(`{'jump': 0.9750385981619527, 'duck': 0.01248070091902369, 'run':
0.01248070091902369}`), verified TWICE (initial boot + the resume
invocation). Reproduces the Bench-034 pin on a fresh boot.

## Protocol deltas vs their table (disclosed, none hidden)

1.util 0.72, not their default 0.35 (the 24 GB card law — 027).
2. Their run was on their Linux box (2026-09-23); ours is this Windows box —
   fp16 embedding drift across driver/hardware can move probabilities by ulps,
   which is why every number here is OURS, not a confirmation of theirs.
3. Our invocation boots the arena cold within the measured window; theirs
   likely ran against a warm server (their per-seed p50 is flat at 16.5).
   We publish the trajectory (above) instead of hiding it.
4. PYTHONIOENCODING=utf-8 was required to run their harness on this cp874
   console (their `log()` prints a U+00B7 detail string — the katgpt-rs
   console-encoding class, live; no source changes to their tree).

## What T4 closes

Issue 019's tasks T0–T6 are ALL done (T3 rode `.issues/027`): the lane landed
(`clm-lane`, goldens + stub-HTTP pins), the 15-suite harness column ran on the
4090 (Bench 034), the site carries the `clm (reference)` rows (T5), and THIS
record is the optional T-Rex row. The T-Rex cells are bench-record-only by
design — they are not `/bench` lane columns (the site's tables are generated
from OUR harness output; this is their instrument under our protocol). Issue
019 is closed and removed with this landing; its record is here + HISTORY.md.
