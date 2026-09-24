# Issue 015 — parallel-Metal-instance smoke divergence (7 observations, root cause unproven)

**Status:** RESOLVED 2026-09-24 — root cause FOUND and fixed (`a3215da`): the smoke violated the chain-cache epoch contract (never called `begin_pass`), not GPU contention. The obs 1–7 "flake" history and the containment's cross-process reading were both this. Post-fix 45/45 green (30 parallel + 15 serialized).

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
4. 2026-09-24 ~04:0x (the v0.2.3 release pre-flight, serialized posture
   `-- --test-threads=1`): one failure (6/7, test name not captured — the
   run was not tee'd), immediately followed by 3× serialized 7/7 greens,
   the FULL laya-riir-metal suite serialized 154/0, and a parallel
   full-suite pass 9 suites 0 FAILED. Timing: minutes after the sibling
   Metal session's own parity runs ceased — consistent with candidate (b)
   (cross-process GPU contention decaying), NOT with a code regression
   (nothing changed between the red and the greens; the version-bump-only
   tree is otherwise identical to a51ea42).
5. 2026-09-24 ~04:4x (the flash_attn session, HEAD `a386119` + the fused
   kernel in the tree): 4/6 PARALLEL full-suite runs red on
   `metal_ops_match_cpu_op_by_op` (diverging arm not re-printed per run),
   while a single-test run of that test alone passed every arm. Read as
   load at first — see observation 6, which supersedes the amplification
   reading.
6. 2026-09-24 ~04:5x, the attribution run that matters: the BASE tree
   (`git worktree` at `a386119`, NO flash kernel) flaked **serialized**
   at 1/8 — same class, no new kernel in the library. So the class is
   PRE-EXISTING and not amplified by the flash_attn work; the 4/6 rate
   in observation 5 was concurrent-compile load, not the kernel. Also
   the flash session's own first full-suite parallel run red on
   `matmul_heads seq300` (max 2.8e1), which then passed unchanged in
   every later run — same decay shape as observation 4.
7. 2026-09-24 ~06:5x (the WBK=48 probe session), the SHARPEST
   host-level evidence yet, and it KILLS the cold-shader-compile
   hypothesis: an interleaved 12× loop alternating the BASE and NEW
   binaries flaked BOTH SIDES SIMULTANEOUSLY on round 11 — base red on
   `glu_gelu_gate` (not an sgemm op at all), new red on
   `matmul_w 1x257x129` (a narrow shape the WBK edit does not touch).
   Two different processes, two different ops, one instant. Shader
   caches were warm by round 11 (10 prior green rounds each), so
   "first-run MSL compile" cannot explain it; simultaneous cross-process
   failure on UNRELATED kernels points at a HOST/DRIVER-level transient
   (GPU scheduler or memory-state corruption shared across processes),
   not a kernel defect and not a compile artifact. Both trees finished
   ~10–11/12 green with rates in the same class.

Serialized `-- --test-threads=1` after both failures: all 7 tests green,
bit-identical. The G5 parity gate (single Metal instance) has never
flagged it. (Observation 4 shows the flake class can surface EVEN
serialized right after heavy sibling GPU work — the one red there was
followed by stable greens with no code change, so the run-to-run decay
pattern is the load-bearing signal, not the test-thread count.)

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

## The containment + what it measured (2026-09-24, same day)

The smoke file now SELF-SERIALIZES: every test body holds a file-wide
`gpu_lock()` mutex, keeping per-test `Metal::new()` instances (commit
ref: this commit — `tests/metal_ops_smoke.rs`). Results, in order:

1. **A shared single instance is NOT viable — new measured datum.** The
   first cut implemented the ask's literal sketch (`Mutex` around ONE
   shared instance) and diverged **3/7 SERIALIZED** (max 8.2e-2;
   merge_heads, ops_match, sliding_chain). Cause: the backend's `weights`
   cache keys device buffers by slice `(ptr, len)` under the
   agent-lifetime stable-address contract — tests drop scratch Vecs, the
   allocator recycles addresses, and the shared instance serves a
   previous test's stale weight buffer for a same-shaped one (the
   probe-birth chain-cache trap class, one layer up; `chain` has the
   `gen` defense, `weights` deliberately does not). Per-test instances
   are therefore LOAD-BEARING, and the lock must wrap test BODIES, not
   the instance.
2. **In-process contention is eliminated by construction** — at most one
   live instance + GPU stream at any time. Serialized: 7/7 (the
   historical invariant).
3. **The residual reds are now diagnostic: 1/18 parallel rounds failed
   with the lock in place** (7/8 first batch, then 10/10; the failing
   round's op was not captured). Per this issue's own decision rule, a
   parallel red AFTER in-process isolation is evidence the class is
   CROSS-PROCESS: `MTLCompilerService` + the riir-shader sibling's
   browser/GPU captures were active on the box during the batch,
   matching observations 4 and 7 (cross-process simultaneity on
   unrelated kernels).

**Updated ask:** the remaining experiment is a QUIET-GPU window — run
the parallel suite while NO other GPU process is active (shader
sibling idle, MTLCompilerService quiet). All-green there would pin the
class to cross-process GPU scheduling/host transients beyond this
repo's reach (an OS/driver report, not a code fix); a red there would
re-open in-process hunting with the instance-contention hypothesis
dead. Until such a window: serialized stays the honest posture, and
the lock makes parallel runs safe-but-unproven rather than hazardous.

## RESOLUTION (2026-09-24 ~08:0x +07, the quiet-window session) — the experiment ran and REFUTED the cross-process reading

The window opened (MTLCompilerService idle, no shader-sibling captures,
only ambient WindowServer) and the suite ran from a fresh worktree
build at `77c408e` (post-containment code, worktree-isolated target;
commit `77c408e`'s library is bit-identical to `8f138f3`'s — the
intermediate commit touched only harness files):

- **16/20 parallel red + 5/5 serialized red on a QUIET GPU.** The
  opposite of the expected all-green. Cross-process GPU contention is
  REFUTED as the primary class.
- The reds' signature differed from the flake prose in one load-bearing
  way: DETERMINISTIC values per batch (`matmul_heads 9.8192e0`,
  `attn slide 37x4 w4 6.7278e-1`, identical across fresh processes),
  always the same small set of arms per batch — a race does not do
  that; a reproducible wrong computation does.
- G5 parity stayed GREEN (2/2, 10.2 s) on the same binary throughout —
  the full 88-forward model parity passes while the smoke's tiny-shape
  arms diverge. G5 begins a pass per layer; the smoke did not.
- Alone-vs-suite: each failing test PASSES alone in a fresh process and
  REDS in-suite — the divergence needs the process context, not the
  test.
- **The trace smoking gun** (`LAYA_METAL_TRACE=1`): in a red round,
  `matmul_kt` uploaded ONE of its two inputs (the `k` upload MISSING —
  a silent chain-cache HIT); the green round uploaded both. A silent
  hit serves the STALE device buffer keyed by the colliding `(ptr,
  len, epoch)` — a DST slot from an earlier arm holds the previous
  arm's device OUTPUT, so the op reads garbage at full scale
  (max 2.66e1 at scale 2.57e1).

**Root cause:** `chain_buf`/`chain_slot_for` key device slots by
`(ptr, len, epoch)`; the epoch moves ONLY inside `begin_pass`
(`metal.rs:1251`). The backend's own doc states the contract — host
buffers are rebuilt per forward "often at recycled heap addresses with
fresh contents — so last pass's keys must never hit" — and the forward
honors it (`agent.rs:190`). The smoke called ops STANDALONE, never
beginning a pass: epoch 0 forever, the key set grew across a test's
~50 arm allocations, and any arm whose input Vec recycled an earlier
arm's freed DST address silently read the wrong buffer. Whether a
collision fires is a PER-PROCESS HEAP-LAYOUT LOTTERY — which alone
reproduces every one of obs 1–7: a different op each run, deterministic
values within a session (stable layout per binary+env),
alone-passes/in-suite-reds, serialized reds, the 1/18-then-10/10 post-
lock parallel record, the 07:33 7/7 green (that binary+env's layouts
dodged the collisions), and obs 7's simultaneous two-binary red (both
layouts collided that round). **It was never GPU contention, never
shader-compile, never the driver — the quiet-window "confounder" WAS
the finding.**

**Fix (`a3215da`):** `m.begin_pass()` at each arm boundary in the
smoke (fresh-fixture blocks + fused shape iterations); the two
composed chains begin ONE pass at their start and compose
device-side within it, exactly like a forward. Post-fix: 30/30
parallel + 15/15 serialized green in the same window, epochs 1–13+
visible in the trace. The containment section's decision rule ("a
parallel red AFTER in-process isolation is evidence the class is
cross-process") is RETIRED — a red after that lock is evidence of an
epoch-contract violation in the caller; check the trace for the
missing-upload signature first. The `weights` cache keeps its own
permanent `(ptr, len)` map with NO epoch defense — deliberate for
agent-lifetime weights, and the smoke's weight lens are distinct per
arm today, but any future op-level consumer that passes recycled
same-len slices as weights re-opens this class one layer over; the
contract note in the module doc covers it.

Session: m3, 2026-09-24, commits `a51ea42`/`9ed1211` (the observations
were collected during that work; the kernel changes themselves are
exonerated — the diverging ops were untouched by them and everything
passes serialized). Observations 5–6: m3, 2026-09-24, the flash_attn
session — observation 6's base-tree worktree run (`a386119`, no flash
kernel) is the one that exonerates the new kernel explicitly.
Observation 7: m3, 2026-09-24, the WBK=48 probe session (uncommitted at
write time) — the simultaneous cross-binary flake. Containment + the
shared-instance finding + the post-lock 1/18 reading: m3, 2026-09-24,
the site-polish remains session (this commit).
