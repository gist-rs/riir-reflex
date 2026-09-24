# Bench 033 — T10 rung 2 (rope hoist) promotion probe — **PENDING**

The instrument is archived; the verdict is not yet taken. The box has not
offered a preflight-clean window since the rung landed (load 20.5 at the
09-25 archiving against the 6.0 ceiling).

- **Rung:** riir-infer `5ef7442` — `attn_rope` derives the Q/K rope once per
  layer into a packed `[2, seq, d]` device scratch; `flash_attn`'s staging
  copies instead of re-rotating per (query block × head × key tile).
  DEFAULT-OFF behind `LAYA_METAL_ROPE_HOIST=1`; gates green at both
  postures (reflex `.issues/020_riir_metal_latency_parity.md`, rung 2 tick).
- **Arm switch:** the env flag IS the A/B — same binary, no rebuild between
  arms. OFF = `LAYA_METAL_ROPE_HOIST=0` (in-kernel rope, shipped behavior),
  ON = `LAYA_METAL_ROPE_HOIST=1` (hoist).
- **Run:** `bash .benchmarks/033_rope_hoist_ab/rope_hoist_ab.sh` — 6
  position-balanced rounds × 2 arms × 3 suites
  (`massive_intent_en,banking77,code_fixtures`; code_fixtures carries the
  512-token case where flash_attn dominates), load logged per arm-run to
  `/tmp/rhoistab/rope_ab.idx`.
- **Verdict rule:** position-balanced medians per arm across all rounds; a
  win needs round consistency — per the T5 confound lesson the per-run load
  trace is load-bearing and aliased rounds are DISCARDED, not averaged in.
  Promote to default only on a measured win + gates re-green at the
  promoted posture; otherwise the rung stays opt-in (or is reverted).
- Record the verdict HERE (or as a Bench 006 addendum) with the
  `bench_preflight.sh` PROVENANCE line.
