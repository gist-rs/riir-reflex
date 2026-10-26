# 097 — pre-existing `cargo test --all-features` failures (stack overflows + three parity/oracle targets)
**Status:** OPEN — observed 2026-10-26, unadjudicated (no root cause read yet)

Found while verifying the `src/engine.rs` test-module move (`062ec1e`). Every
failure below reproduces **identically with the pre-move `engine.rs` restored**
(the attribution control), so none is caused by that commit — they predate it.
Box: amo (M5 Ultra, macOS 27.0.1, AC), default parallel test threads.

| target | test | shape |
|---|---|---|
| lib | `harness::runner::synth::tests::b0_judge_serves_pool_replicas_and_is_deterministic` | **stack overflow — kills the whole lib harness** (550 others pass; the count is truncated until it is skipped) |
| `tests/harness_seat_gates.rs` | `seat_engine_accuracy_equals_the_manual_build` | stack overflow |
| `tests/laya_ane_parity.rs` | `g5_ane_decision_parity_per_checkpoint` | assertion failure |
| `tests/slice_leak_oracle.rs` | `g1_rust_index_reproduces_the_probe_exactly`, two `t3_*` | assertion failure |

## Tasks
- [ ] T1 — the two stack overflows: test threads get a 2 MiB stack; check whether
  they pass under `RUST_MIN_STACK=16777216` (a large on-stack array / deep
  recursion in the synth judge path) and fix at the source (box the buffer or
  run the body on a sized `std::thread::Builder`), never by a global env var.
  The lib-harness kill is the priority: it truncates the lib pass count for
  every other test.
- [ ] T2 — `laya_ane_parity` g5 + `slice_leak_oracle` g1/t3: read whether these
  depend on an absent artifact / sibling checkout / ANE availability (a
  SKIP-vs-FAIL question) or are genuine drift; record the verdict here.
- [ ] T3 — once green, confirm `cargo test --all-features` exits 0 end to end.
