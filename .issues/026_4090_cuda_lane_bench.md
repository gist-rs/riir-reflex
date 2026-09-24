# Issue 026 — the 4090 CUDA lane: `laya-riir-cuda` bench refresh

**Status:** OPEN — backend landed substrate-side (riir-infer `.issues/002`);
this is the consumer-side record: G5 green at the cuda posture + the
harness re-run that replaces the 4090-windows row's CPU latencies.

## Why

The published bench's `4090-windows` host ran the laya lane on CPU
(`"laya_device": "cpu"`) — 11–20× the M3 Metal row (typed 8127 ms vs
479 ms; banking77 1745 ms vs 87 ms). The Metal backend is macOS-only, so
the 4090 GPU sat idle.

## What landed

- riir-infer `.issues/002`: the CUDA backend (cudarc 0.19 + nvrtc, sm_89,
  feature `laya-riir-cuda`, target-scoped non-macOS). Non-macOS builds
  that compile it DEFAULT to the cuda posture (`LAYA_DEVICE=cpu` opts
  out); `LAYA_DEVICE=cuda` honored verbatim; fail-loud otherwise.
- reflex: the `laya-riir-cuda` forwarding feature + the harness
  `DeviceKind::Cuda` posture label.
- G5 parity at the cuda posture (this box, 2026-09-24): english 26/26
  top-1, drift 1.863e-6; typed 26/26, 2.471e-6; multilingual 36/36,
  2.894e-6 — the same drift class as the Metal lane, ~500× under the
  1e-3 gate.
- Op-level gate `cuda_ops_smoke` (in the lane crate) green: every backend
  op vs the CPU free fns (bit-exact for the data-movement ops; 1e-7…1e-4
  class for the reductions).
- Fixture timing (this box, same-session A/B): english 210.4 → 17.5 ms
  (12.0×), multilingual 95.0 → 9.0 ms (10.6×), typed 208.4 → 17.4 ms
  (12.0×) — every checkpoint BELOW the M3 Metal row (28.3 / 12.2 /
  28.3 ms) at the v1 rung (default attention path; the flash kernel is a
  follow-up rung).

## Tasks

- [x] The CUDA backend (riir-infer `.issues/002`)
- [x] reflex feature forward + harness posture label
- [x] G5 at the cuda posture (green, numbers above)
- [x] `cuda_ops_smoke` green
- [ ] The full 15-suite harness re-run at the cuda posture (this issue's
      bench record, `.benchmarks/026_4090windows_cuda/`)
- [ ] bench record + HISTORY entry + the site-row handoff (publish from a
      box with the site repo + wrangler creds — the M3 side merges the
      results.json as the refreshed 4090-windows host row)
