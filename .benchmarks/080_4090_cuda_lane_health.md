# Bench 080 — 4090 CUDA lane health: ops smoke + repeat probes GREEN at HEAD (post-Bench-079 sweep)

**Verdict: PASS — the 4090 GPU lane is healthy at `46b8deb`.** Driven by
the reflex Bench-079 session's GPU sweep pass over the box (the same
ssh lane; riir-infer is reflex's laya substrate, boundary-legal read
side: reflex consumes it as upstream substrate):

| probe | result | wall |
|---|---|---|
| `cuda_ops_smoke` (fused attention vs CPU op-by-op, sliding-window mask, packed offsets) | ok | 1.83 s |
| `cuda_agent_repeat_probe` (`cuda_agent_system_one_repeats_stable`) | ok | 15.66 s |
| `cuda_repeat_probe` (`cuda_ops_repeat_bit_stable_at_banking77_shapes`) | ok | 13.04 s |

GPU state during the run: RTX 4090, ~21% util baseline, 721 MiB
resident (desktop), no competing compute tenants. All three binaries
ran from the existing release cache (`target/release/deps/`, no
rebuild) — the box was already at `46b8deb` (its local HEAD; that repo
is a train-side lane, untouched by this session — record-only).

## Provenance

- The reflex/instinct sync to this box rode git **bundles**
  (`052.bundle` → `riir-reflex` ff to `2cbbce7`): the box's direct
  `git fetch origin` hangs (GitHub egress/credential stall, observed
  twice) — bundles are the working sync path.
- The detached-launch quirk (Start-Process dies silently after the
  harness banner) is real on this box; foreground ssh runs are the
  working shape.

## Why this row exists here

The typed cross-host verify (Bench 079) ran on this box; its GPU was
idle during that run (modelless lane). This row records the GPU lane's
health at the same visit so the reflex laya CUDA posture (`laya-riir-cuda`,
the serving matrix's cuda cell) has a current green to cite —
the full consumer-side G5 + typed specialist timing at `LAYA_DEVICE=
cuda` stays open (needs the laya weights synced to the box; not
present in `E:/git/riir-train/data/`).
