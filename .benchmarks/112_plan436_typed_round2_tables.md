# Bench 112 — Plan 436 T3: laya round 2 (typed replay) on our board — serving cell 0.7825, the round-1 cost REVERSED

**Status:** MEASURED 2026-10-03 — typed_decisions serving cell **0.7825**
(round0 published 0.7445, round1 0.6910); record + the training-side half:
riir-train `.benchmarks/622_plan436_laya_round2_replay.md` (Plan 436 / Issue
607). NOT promoted — serving adoption stays the owner-gated step (Bench 111
law); the tables are the handoff for that call.

Run: `cargo run --release --features laya-riir-cuda --bin harness --
--suites typed_decisions --out .benchmarks/112_plan436_typed_round2_tables` at
reflex `3905b17` on `4090-windows`, `LAYA_DEVICE=cuda`,
`LAYA_WEIGHTS_DIR=E:/git/riir-train/.raw/plan435/laya_weights_r2` (typed =
round2 checkpoint), `LAYA_ALLOW_UNPINNED_WEIGHTS=1` (the Bench 620 T5a
research-lane pin escape).

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | p50 | det |
|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.3345 | 0.2456 | 0.0488 | 0.6895 | 1.2301 | 0.5429 | 0.4690 | 0.929 ms | ✓ |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2113 | 0.7684 | 1.3593 | 0.5627 | 0.4290 | 84.0 ms | ✓ |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.3228 | 0.9057 | 1.9437 | 0.5548 | 0.4040 | 45.0 ms | ✓ |
| **laya-riir · typed (round2)** | 2000 | **0.7825** | **0.7744** | 0.1477 | 0.3426 | 0.5991 | 0.0878 | **0.9180** | 83.0 ms | ✓ |

- Slice-integrity digest identical to Bench 111/082
  (`fnv1a64-287d5f73a11932cd` test / `0b6e8426558b0d1e` cal / `a1679de56e9278b1`
  pool) — same split, same posture; english/multilingual byte-exact vs 082 →
  the delta is the checkpoint.
- Round 2 = round-1 recipe + typed_decisions TRAIN replay ×12 (12.87% of the
  mix). The replay taught the breadth-round encoder the typed task: +9.15 pt
  over round1, +3.8 over the never-fine-tuned published baseline, and every
  quality column (Brier/NLL/AURC/acc@50) improved. ECE 0.1477 vs round1's
  0.1041 — the sharper checkpoint's calibration trade, disclosed (the readout
  calibration owns that axis).
- p50 83 ms — the round2 checkpoint serves at the same latency class as
  round1/base (same architecture; no serving-cost change).

Full box state + S1MB half: riir-train Bench 622.
