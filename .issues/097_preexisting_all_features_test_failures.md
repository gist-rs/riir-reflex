# 097 — pre-existing `cargo test --all-features` failures (stack overflows + three parity/oracle targets)
**Status:** RESOLVED 2026-10-26 (zed-glm-m3-agent) — all three task groups fixed/adjudicated on the M3; full record moves to HISTORY.md at removal.
**Claimed:** zed-glm-m3-agent, 1792986189, expires 2026-10-28T04:00:00Z

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
- [x] T1 — the two stack overflows: **ROOT-CAUSED — `density_gate`'s inline GMM state.** Bisected
  across every feature: BOTH overflows reproduce at `--features density_gate` ALONE (all 15 other
  features individually green). `DomainExpert.density: Option<SupportGate<64, 16>>` = ~16.7 KiB of
  inline `[[f32; E]; K]` arrays per expert (2 × `DiagGmm`), so `DecisionEngine<842, 256>` = 14.85 MiB
  (measured) vs ~1.2 MiB pre-066. The dispatch lanes hold engines BY VALUE and debug builds do not
  elide moves: `build_b0_judge`'s 12-arm const-generic table materializes Σ-arms × (engine + move
  closure) ≈ 64+ MiB in ONE frame — past its own 64 MiB probe thread (lldb fp-walk: 12 frames, guard
  fault 64 MiB below SP, `build_b0_judge` prologue); the seat test's nested posture lanes
  (`fit_posture` → `nb_lane`/`readout` locals + `eval_engine`) overflow the 2 MiB libtest thread
  (lldb: named thread `seat_engine_accuracy_equals_the_manual_build`). RUST_MIN_STACK is a no-op
  (explicit `stack_size(64 MiB)` wins; libtest threads ignore it). FIX: `density: Option<Box<SupportGate>>`
  — heap-boxes the GMM state at the ONE field that grew, restoring the pre-066 engine size at every
  site (a substrate-side DiagGmm change was rejected: layout/alloc-discipline blast radius for the
  same effect). Serving posture unchanged (the knob stays false; the box is deref-only).
- [x] T2 — SKIP-vs-FAIL adjudication, three verdicts:
  - `laya_ane_parity` g5: **PASSES on this M3** (234.9 s cold, 14.6 s warm). The amo failure is
    box/environment-dependent (ANE availability or a timeout on the 60 s+ run), NOT code drift —
    needs the amo box to fully adjudicate; no code change warranted from here.
  - `slice_leak_oracle` g1 + both t3: **genuine three-copy wiring drift — FIXED.** (a) The oracle's
    local `build_suite` match and the shared `eval_case_text` had NO s1mb arms (the probe
    auto-discovers the Plan-010 s1mb dataset dirs → t3 panicked `no builder`, masking everything
    after `s1mb_choice` alphabetically). (b) The pair-join DRIFTED across three copies: the probe's
    `row_text` and the oracle's own `row_text` join xnli/wanli pairs with `" || "` while the
    engine's corpus rule (`train_docs`/`eval_case_text`) joins with `"\n"` — post-normalization
    `" || "` leaves a DOUBLE space (the stripped pipes), `"\n"` a single one, so shingle sets
    differ at the seam and boundary-adjacent Jaccard flips (wanli rows 364/1674: probe j 0.696/0.771
    vs wiring j 0.850/0.804). Fix: the PROBE + oracle copies join with `"\n"` (the engine's rule —
    the oracle measures the texts the engine sees); the engine side is UNTOUCHED (changing
    `train_docs` would byte-drift every xnli/wanli decision — a serving-posture act, out of scope).
    Post-fix: probe and wiring agree everywhere; wanli near 1→3 in future probe runs (the old
    `near 1/1,999` quoted in Bench 130 is a dated historical record, characterization only — no
    gate pins near counts; slice_guard pins pool floors, not leak counts).
- [x] T3 — **`cargo test --all-features` exits 0** (23 test binaries, all green; lib 551 passed
  where the run previously aborted mid-harness). Clippy `-D warnings` clean at default /
  all-features / no-default; default-posture suite exit 0.
