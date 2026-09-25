# Bench 047 — Issue 032/034's 4090 full-lane leg: modelless + laya-cuda (×3 on typed, english elsewhere) + leak scan, cross-host bit-identity with Bench 045

**Status:** LANDED 2026-09-26 · the 4090 half of the both-hosts publish.
`--head-select --out .benchmarks/047_full_4090` at reflex `d727196` (the
harness source is byte-identical to the M3 doc's `6939420` — every commit
between them is docs/scripts; the fold promotion `53334f9` is Metal-only,
the CUDA lane untouched by construction),
`--features slice_leak,laya-riir-cuda`, `REFLEX_BENCH_HOST=4090-windows`,
15/15 suites, no absences.

**Box state:** `power source unreadable — UNJUDGED` (the Issue 021 stamps
are macOS probes; Windows carries them as None — the honest degradation).
The box WAS idle for the run (the riir-train `plan410_stage0_train` job that
occupied the GPU all night ended ~06:00; 518 MiB desktop residual before and
after; the launch was gated on `nvidia-smi` compute-process absence). Read
the latency cells with the standing UNJUDGED caveat; the accuracy +
selections + leak counts are the deterministic claims.

**Launch note (the recorded trap):** two `Start-Process` attempts died with
the ssh session (Windows OpenSSH kills the session's process tree; both
deaths were silent — 0-byte then 2-line logs). The working pattern is a
`schtasks` one-shot task running a `.cmd` batch (`schtasks /Run /TN …`),
which survives the session; task deleted after the run.

## The deterministic claims

- **Cross-host modelless: ALL BIT-IDENTICAL vs Bench 045's M3 doc**
  (programmatic per-suite compare, zero mismatches) — the drift gate's
  both-hosts shape satisfied; the publish landed on it.
- Head selections: sst5 → 0.5, massive_intent_en → 1.0, banking77 → 1.0,
  rest 0 — identical to Benches 040/043/044/045 (five-run stability).
- **Leak counts byte-identical to 045 and 044** (ag_news 1/26, banking77
  0/17, massive 4/17, prompt_injections 0/2, sst5 1/0, emotion 0/0,
  xnli_en 0/0) — the third independent confirmation that the scan is a
  dataset+registry-cap property, not a host property.

## Headline cells (UNJUDGED-box caveat, indicative only)

typed_decisions CUDA p50s: english 105 ms · typed 74 ms · multilingual —
ag_news 12 ms · banking77 24 ms (the published 4090 cuda cells refreshed
from these; the previous published 4090 laya cells were the single-ckpt
bench-035-era rows).
