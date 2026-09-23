# Issue 015 — parallel-Metal-instance smoke divergence (3 observations, root cause unproven)

**Status:** OPEN — reproduction + containment recorded, root cause NOT diagnosed.

## The class

`tests/metal_ops_smoke.rs` fails under the default multi-threaded test
runner with a HUGE divergence (max abs 8–81 against a 1e-3 tolerance),
in a DIFFERENT op each run, and passes every time with
`-- --test-threads=1`. The failing op is arbitrary — it is not the
kernel under test.

## Observations

1. 2026-09-23 (previous session, recorded as "the rare
   parallel-Metal-instance smoke flake", never filed): one divergence,
   details in that session's summary.
2. 2026-09-24 ~03:15Z (this session, during the a51ea42 kernel work):
   `glu_gelu_gate` DIVERGED max 2.0972e1 — a kernel the same change did
   not touch; the run finished in 0.22 s.
3. 2026-09-24 ~03:20Z, immediate re-run: `matmul_w 1x257x129` DIVERGED
   max 8.1324e1; the run finished in 0.05 s — impossibly fast for the
   GEMM set it contains, suggesting GPU work did not actually complete
   correctly.

Serialized `-- --test-threads=1` after both failures: all 7 tests green,
bit-identical. The G5 parity gate (single Metal instance) has never
flagged it.

## What is NOT established

- The mechanism: candidates are (a) multiple `Metal::new()` instances in
  one process contending for the GPU with cross-instance buffer hazards,
  (b) an unrelated process (sibling agent session) issuing Metal work
  concurrently, (c) a hazard-tracking gap that only shows with several
  command queues in flight. Nothing has been isolated; do not quote a
  cause as fact.
- Whether CI/self-hosted runners can even reproduce it (single-lane).

## Containment + the ask

- Lane discipline until fixed: run the smoke serialized
  (`-- --test-threads=1`); it is the repo's own documented posture for
  flaky-v-by-construction suites. A green parallel run means nothing and
  a red one means nothing until the contention is understood.
- The ask: reproduce deliberately (spawn N Metal instances in one test
  and hammer ops concurrently), pin the failing seam, and either fix the
  instance isolation or make the smoke file self-serialize
  (`std::sync::Mutex` around a shared instance, or a
  `serial_test`-style lock) so the flake dies instead of being dodged.

Session: m3, 2026-09-24, commits `a51ea42`/`9ed1211` (the observations
were collected during that work; the kernel changes themselves are
exonerated — the diverging ops were untouched by them and everything
passes serialized).
