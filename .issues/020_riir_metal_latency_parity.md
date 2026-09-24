# Issue 020 — the riir Metal lane must BEAT the python torch MPS oracle on every published cell (p50 AND p99)

**Status:** OPEN — **waves 1–2 LANDED and measured on AC**
([Bench 006](../.benchmarks/006_issue020_latency_wave1.md) Addendum 2, which
supersedes the battery-era §2/§3 deltas). Class B (the first-forward cliff)
is **closed** at −64…−68% (reproduced on AC). Class A is **NOT closed**: the
wave moved `massive_intent_en` p50 by ≈ −5% (10 paired rounds) and the GEMM by
−10% wide / −21.5% narrow, but a **same-run** head-to-head against the python
oracle still has rust **losing p50 by ~10%** (massive_intent, banking77) while
**winning p99** — and `code_fixtures` joins them (Bench 006 Addendum 3:
same-run p50 **+7.4%** median over 8 rounds, max **+43%** — T8 attributed
it to ONE long case, not a cold start). Its largest remaining lever (T5, per-case question batching)
is IDENTIFIED from the reference's own source. **T9 (Bench 006 Addendum 4):** case 3 = 512 tokens; its gap is T5 batching first, GEMM at m ≥ 256 (T7) second, and attention is about even in total (new small T10). **T10 rung 1 LANDED** (riir-infer `0ec88a9`, Bench 006 Addendum 5): flash_attn's row softmax on one simdgroup per row — encoder −6…−8% at 188–512 tokens, 10/10 paired wins, G5 green. **T10 rung 3 LANDED** (riir-infer `14af99f`, Bench 006 Addendum 6): one-pass online softmax — encoder a further −1.7…−3.5%, 38/40 paired wins, G5 green. The T7 `XWIDE_N_MIN` 2048 → 1024 pick was A/B'd through the harness and is **NOT landed** (inside noise at stable load). **T5 LANDED in code** (riir-infer `da30007` — packed multi-question encoder forward, attention dispatched per sequence at bind offsets so the kernel math stays exact; reflex consumer gate `tests/laya_batch_parity.rs` replays the frozen capture at top-1 1.000000, `f180cfc`); the publishable position-balanced A/B is PENDING a quiet box (directional read at load 25–35: typed −18…−28% p50, accuracies byte-identical). Filed 2026-09-24 from the published arena table
(`https://reflex.gist.rs/data/bench.json`, `git_sha 77c408e`, M3, release).
Owner directive in-session: *"rust slower than python in p99 and other case
… make rust faster as it should in all cost."* Two independent causes are
already measured apart; the fix waves are T1–T4 below.

## The finding — where the published table loses

Both lanes answer byte-identical cases; `english` is the riir Metal lane,
`py/english` the ORIGINAL torch MPS reference run as a subprocess oracle.
Rust-slower cells, published run:

| suite | n cases | rust p50 | py p50 | Δp50 | rust p99 | py p99 | Δp99 | tail support |
|---|---|---|---|---|---|---|---|---|
| typed_decisions (typed) | 400 | 457 | 369 | **+23.8%** | 820 | 749 | **+9.5%** | 5 |
| typed_decisions (english) | 400 | 421 | 353 | **+19.3%** | 678 | 618 | **+9.7%** | 5 |
| typed_decisions (multiling.) | 400 | 211 | 153 | **+37.9%** | 375 | 320 | **+17.2%** | 5 |
| massive_intent_en | 300 | 63 | 51 | **+23.5%** | 129 | 98 | **+31.6%** | 4 |
| code_fixtures | 14 | 147 | 125 | **+17.6%** | 350 | 235 | **+48.9%** | 1 |
| sst5 | 600 | 34 | 29 | **+17.2%** | 44 | 70 | −37.1% | 7 |
| banking77 | 500 | 90 | 84 | **+7.1%** | 116 | 162 | −28.4% | 6 |
| xnli_en | 300 | 37 | 35 | **+5.7%** | 54 | 87 | −37.9% | 4 |
| ag_news | 400 | 38 | 37 | **+2.7%** | 64 | 98 | −34.7% | 5 |
| harness_sensitivity | 15 | 37 | 45 | −17.8% | 243 | 125 | **+94.4%** | 1 |
| harness_tool_fit | 12 | 27 | 68 | −60.3% | 270 | 152 | **+77.6%** | 1 |
| harness_routing | 16 | 38 | 52 | −26.9% | 204 | 153 | **+33.3%** | 1 |
| harness_permissions | 12 | 25 | 38 | −34.2% | 162 | 129 | **+25.6%** | 1 |
| harness_cache_reuse | 12 | 36 | 56 | −35.7% | 156 | 153 | **+2.0%** | 1 |

⛔ **The two columns are TWO DIFFERENT DEFECTS and pooling them is the trap.**
Read the `tail support` column (this workspace's own
`percentile_index_audit` rule: at p99, `n ≤ 100` ⇒ the reported index lands
on `n − 1` = the **MAX**):

- **Class A — steady-state p50 on mid/long sequences.** `tail support ≥ 4`,
  so the p50 is a real median over ≥ 300 cases. rust loses 2.7–37.9%. This
  is the kernel/dispatch cost and it is the harder half.
- **Class B — the FIRST FORWARD.** Every `tail support = 1` row is a
  12–16-case suite whose "p99" IS its maximum, and the maximum is case 0.
  rust's cold call is **5–10×** its own median (270/27, 243/37, 204/38,
  186/26); the python oracle's is **2–3×**. That ratio gap is not noise —
  it is a structural asymmetry, named in T1.

## Root cause, Class B — rust pays the device residency on the timed call

`scripts/laya_python_lane.py:91` does `laya.load(shim, device=device)` and
only THEN prints `{"ready": true}` — torch moves the whole model to MPS
**before the handshake**, i.e. **outside** `run_laya_python_checkpoint`'s
per-case timer (`src/harness/runner.rs:2196`).

`RiirAgent::load` (`src/laya/riir/agent.rs:114-150`) builds the backend and
leaves every weight in a host `Vec<f32>`. The device copy happens lazily in
`Metal::weight_buf` (`src/laya/riir/metal.rs:1172-1181`) on first use — i.e.
**inside the first `system_one`**, which the harness times
(`src/harness/runner.rs:2143`). English geometry (`d=1024`, 28 encoder
layers, `I=2624`, 2 head layers) is ≈ **0.5 GB of f32 projections** uploaded
as ~120 separate `newBufferWithBytes` allocations on the critical path of
request #1.

Neither lane warms up. The asymmetry is not the harness's fairness — it is
**ours**: the served binary has the same first-request cliff, so this is a
product defect that the bench merely reveals.

Second Class-B contributor: the MSL library is compiled **from source at
every process start** (`src/laya/riir/metal.rs:1121-1126`,
`new_library_with_source` over ~830 lines incl. `<metal_simdgroup_matrix>`),
with no `.metallib` and no `build.rs` shader step.

## Root cause, Class A — per-forward waste the geometry does not need

Measured from the hot-path map at the english geometry, `seq ≈ 317`:

1. **~341 GPU dispatches per forward, each in its OWN
   `MTLComputeCommandEncoder`** (`metal.rs:1303-1334`): 341
   `new_compute_command_encoder()` + `end_encoding()` pairs. A default
   compute encoder is `MTLDispatchTypeSerial` — dispatches inside ONE
   encoder already run in order with implicit barriers — so the per-op
   encoder is pure driver overhead with identical semantics.
2. **The rope tables recompute `powf` inside the position loop**
   (`src/laya/riir/ops.rs:717-728`): `inv` depends only on `j`, yet it is
   evaluated `seq × half` times. At `seq=317, hd=64` that is **~10 144
   `powf` per table, up to 2 tables per forward**, on the critical path
   between layer 0's Wqkv and its attention.
3. **A `seq²` mask is built every forward and DISCARDED by the flash path**
   (`encoder.rs:202-214` builds `vec![f32::MIN; seq*seq]` ≈ 402 KB + an
   `O(seq·2·window)` fill; `metal.rs:1661` is literally `let _ = mask;`).
4. **Per-dispatch heap churn**: a `Vec` per `run`/`run_rows` call
   (`metal.rs:1360`, `metal.rs:1433` — ~250 of the 341 dispatches), a
   `HashMap<&str, _>` string-hash pipeline lookup plus an ObjC
   `thread_execution_width()` property fetch per dispatch
   (`metal.rs:1354-1358`, `1394-1397`, `1429-1432`, `1486-1489`).
5. **Every activation buffer is freed and re-allocated per forward** —
   `begin_pass_impl` clears the chain cache (`metal.rs:1252`) and the host
   scratch is rebuilt (`encoder.rs:193`, `head.rs:172-208`: ~12 `Vec`s ≈
   24 MB per head layer).
6. **`matmul_w` always takes the UNCOALESCED staging branch**: the weight is
   bound as `Wᵀ` via `b_rs=1, b_cs=k` (`metal.rs:1578-1585`), so consecutive
   lanes read `k`-strided (4 KB apart) — `metal.rs:345/462/599`. The
   module's own BK=48 negative (`metal.rs:110-113`) already attributes the
   wide instance's binding cost to this gather.
7. **`ln_rows` runs one simdgroup (32 threads) per row** (`metal.rs:1440`)
   — at `d=1024` that is 1/32 occupancy across **62 dispatches/forward**.
8. **The head materializes the full `seq²` scores tensor** (`head.rs:188`,
   6.4 MB × 2 layers) although `hd == 64` makes it `flash_attn`-eligible
   modulo rope.
9. **Three syncs per forward** (`head.rs:231`, `:253`, `:265`) where two
   would do — the logits and the CLS row are both device-resident before
   the first one.

## Confounder recorded, NOT yet resolved

An isolated single-suite re-run of `massive_intent_en` on this box measured
rust **p50 43.0 / p99 52.0 ms** (tail support 4) — i.e. **beating the
published python 51/98 on BOTH** — against the published rust 63/129. The
published run is one process over 15 suites with 3 checkpoints resident and
a torch subprocess per suite; the isolated run is a cold process on one
suite. Until the control run (T0) lands, the published Class-A magnitudes
are an UPPER BOUND on the real gap, not the gap. **This does not retire any
T1–T4 item** — every root cause above is a real per-forward cost readable
from the source, independent of which box state measured it.

⛔ **Every paired A/B behind the numbers below was taken on BATTERY** — this
box was unplugged at 09:56:10 (100% → 48%), before the first A/B round and
after the two AC absolute baselines. Full disclosure, and the
survives/does-not-survive split, in Bench 006 Addendum 1; the AC re-bench and
its refusal gate are Issue 021 (closed — record in `HISTORY.md` § 2026-09-24). What that
changes here: the Class-B result stands (p50 unchanged while p99 falls 64–67%
in the SAME runs is its own control), the small Class-A deltas are **not
quotable until T3/T5 of 021 re-take them on AC**, and **T0's question is now
known to be a STATE effect, not a power one** — the published cell (63/129)
and the isolated 09:45 run (43.0/52.0) are both AC.

⛔ **The lane MOVED mid-issue.** `src/laya/` was carved out to
`../riir-infer/crates/riir-infer-laya` by the Issue 008 consolidation while
waves 1–2 were being gated (riir-infer `c6716a4`), which deleted the files
this issue's fix lives in from THIS repo's working tree. The fix was
replayed onto the new home against a byte-identical base (all six files
`shasum`-equal to this repo's pre-carve `HEAD`) and re-gated there; G5
parity then reported the drift **to the digit** — english 4.016e-6, typed
9.806e-7, multilingual 3.520e-6 — proving the replay equivalent to what was
measured here. ⚑ **The replay was then verified against an independent copy.** The T4-move
session had snapshotted the uncommitted WIP before deleting it
(`.issues/020_wip_snapshot_t4move/`, taken 10:39–10:44) — discovered only
after the replay had landed. Diffed code-only (comments stripped, whitespace
normalised) against what was committed: **0 differing lines in `agent.rs`,
`backend.rs`, `encoder.rs`, `head.rs`, `ops.rs`**, and `metal.rs` differs by
**8 lines that are both non-semantic** — a `rustfmt` line-wrap of the
`PendingPass { cb, enc, encodes: 0 }` literal, and the `MAX_RUN_BUFFERS`
`assert!` placed BEFORE the array init rather than after (the replay's order,
and the stricter of the two). So the equivalence rests on two independent
witnesses — the G5 drift matching to the digit, and a byte-level diff against
a copy neither session made for that purpose. Snapshot removed per its own
README (*"commit or delete once the WIP is landed"*) and the noise-reduction
rule; recoverable from this commit's parent tree if ever needed.

**The code lives at riir-infer `6c56f04`;** this issue and
[Bench 006](../.benchmarks/006_issue020_latency_wave1.md) stay here, because
the losing table they answer is this repo's published arena.

## Results — waves 1–2 (Bench 006)

Position-balanced, paired, with the box load recorded per round. Every gate
green: G5 metal 88/88 @ agreement 1.000000 / drift ≤ 4.016e-6, G5 CPU 2/2,
`metal_ops_smoke` 7/7, `scripts/ci_feature_guard.sh` **PASSED (full)**.

- **Class B — CLOSED.** First-forward latency on the five suites whose p99 IS
  their max: **144→52, 144→47, 143→53, 145→57, 138→51 ms (−63…−67%)**, p50
  unchanged in all five. All five were LOSING to the python oracle's p99
  (+25.6…+94.4%); they now beat it by **2.3×–11×**.
- **Class A — partially.** `massive_intent_en` p50 **−8% median** over four
  valid paired rounds (−10.4 / −7.0 / −6.0 / −16.7%, NEW ahead in both
  positions). GEMM kernel **−2…−8%** on the wide/xwide instances and
  **−19…−24%** on the narrow `k = 2624` instance, bit-identical.
  ⛔ **SUPERSEDED on AC (Bench 006 Addendum 2):** −8% → **≈ −5%** p50 over 10
  rounds (the four battery rounds over-stated it); wide GEMM −2…−8% →
  **−9.5…−10%** (battery under-stated it); narrow −21.5% confirmed.
- **`code_fixtures` is unmoved and that is correct** — its 14 cases vary
  hugely in length, so its max is the longest CASE, not the cold call. Its
  published +48.9% p99 loss is Class A, not Class B.

⛔ **T0's answer is a MEASUREMENT DISCIPLINE, not a number.** Three separate
attempts to measure the published Class-A gap on this box were destroyed by
sibling load — including one 3-round run that read a confident **+24.6% /
+30.9% REGRESSION** for the arm that, paired and de-loaded, is **−8% faster**.
The box moved from load 6 to 41.7 mid-experiment. So: the published Class-A
magnitudes remain an UPPER BOUND measured under unrecorded box state, the
only defensible quantity on this workstation is a **paired delta with its
load average beside it**, and an absolute rust-vs-python cell needs a quiet
box this repo does not currently have.

## Tasks

- [x] **T0 — control run.** DONE 2026-09-24 on AC (Issue 021 T5, Bench 006
      Addendum 2): the 15-suite rust-only run took **9.4 min**, not 20+, and
      read `massive_intent_en` **56/73** against a single-suite median of
      **55.5/67** on the same code — **no accumulated-state penalty at p50**.
      ⛔ It also RETIRES this issue's cross-run comparisons: a same-run
      `--laya-python` cell measured the python oracle **16–30% faster than
      its own published column**, so "43.0/52.0 beats the published 51/98"
      (§Confounder above) compared two box states and is not evidence. Every
      rust-vs-python claim from here on is a SAME-RUN cell or it is not made.
      Original deferral text: a 15-suite single-process control is 20+
      minutes, and nothing on this box holds still that long.
- [x] **T1 — Class B: kill the first-forward cliff.**
  - [x] `Backend::warm_weight` / `warm_weight_2d` (default no-op) +
        `Encoder::warm` / `Head::warm` over exactly the slices that reach a
        weight-consuming op; called from `RiirAgent::load`.
  - [-] Precompile the MSL to a `.metallib`. DEFERRED — measured as a
        process-START cost (`Metal::new`), not a first-REQUEST cost, so it
        does not touch the p99 class this issue is about. Worth doing for
        CLI/serve startup; needs an `xcrun metal` build step and a fallback
        for boxes without the Metal toolchain.
  - [x] Gate: first-forward-vs-median ratio measured on a fresh agent —
        **4.1–5.5× before, 1.4–2.0× after** on this box (the published run's
        ratios reached 10×), i.e. inside the python oracle's own 2–3× band.
- [x] **T2 — Class A wave 1 (pure waste, bit-identical by construction).**
  - [x] `inv` hoisted out of `rope_tables`' position loop (`seq·half` →
        `half` `powf` calls; 10 144 → 32 at the english geometry).
  - [x] The `seq²` mask is skipped when the backend predicates the window
        (`Backend::needs_window_mask`, mirroring the fused dispatch's own
        gate so `LAYA_METAL_FLASH=0` and off-geometry `hd` still get one).
  - [x] `(pipeline, thread_execution_width)` resolved once into `Kern`; the
        per-dispatch operand `Vec` replaced by a stack array with an
        asserted `MAX_RUN_BUFFERS` bound.
  - [-] Cache the rope tables across forwards. DEFERRED — `seq` differs per
        question, so the hit rate is low; the `powf` hoist took the cost that
        mattered.
- [x] **T3 — Class A wave 2 (structural, semantics-preserving).**
  - [x] ONE `MTLComputeCommandEncoder` per command buffer instead of ~341
        create/`endEncoding` pairs per forward (`MTLDispatchTypeSerial`
        already orders and barriers inside an encoder).
  - [-] Pool activation buffers + persistent host scratch. DEFERRED to T6 —
        real (~60 MB of host `Vec`s and ~50 device buffers per forward) but
        it needs the scratch to outlive the forward, which is a lifetime
        change through `Encoder`/`Head`, not a local edit.
  - [x] Collapse sync #1 and #2 — **NOT NEEDED, and the map that proposed it
        was wrong.** `download_into` calls `sync()`, which drains everything;
        the second `download_into` then finds nothing pending and is already
        a bare `copy_out`. Only two of the "three syncs" are real waits.
- [x] **T4 — Class A wave 3 (kernel), partially.**
  - [x] Load-time transposed weight copy so `matmul_w` takes the coalesced
        `b_cs == 1` branch. Bit-identical; device memory unchanged (the
        untransposed form is never uploaded for a `matmul_w` operand).
  - [-] Widen `ln_rows` past one simdgroup. DEFERRED — 62 dispatches/forward
        at 1/32 occupancy is real, but a cross-simdgroup reduction CHANGES
        THE SUMMATION ORDER, so it is a drift-moving rung that needs its own
        parity read, not a free win. Sized at ~1.3–3% of the forward.
  - [-] Route the head's MHA through a rope-free flash variant (removes the
        only remaining `seq²` tensor, 6.4 MB × 2 layers). DEFERRED.
- [ ] **T5 — the largest remaining Class-A lever: BATCH a case's questions
      into ONE forward.** **LANDED IN CODE 2026-09-24** (riir-infer `da30007`, consumer gate reflex `f180cfc`) — `forward_packed`: GEMMs/elementwise over all rows in one pass, attention dispatched PER SEQUENCE at bind offsets (kernel untouched, exact per-sequence math, no padding), head per-question on device-copied slabs, one `begin_pass` per case, kill-switch `RIIR_LAYA_NO_BATCH=1` = the loop arm. Gates: lane `tests/packed_forward_equiv.rs` (CPU drift 1.4e-6 / 1e-5 gate, Metal arm), `metal_ops_smoke` 7/7, reflex G5 parity BOTH postures, `tests/laya_batch_parity.rs` top-1 1.000000 both checkpoints, clippy `-D` clean. **A/B PENDING a quiet box**: directional read at load 25–35 (typed_decisions @600-question cap: batched p50 −18…−28% — typed 195 vs 260, english 204 vs 250, ml 73 vs 101 ms; accuracies byte-identical) is NOT quotable; `bench_preflight.sh` REFUSED at load 7.78 + 1.5 GB swap on 2026-09-24 late. Publishable run = position-balanced paired harness A/B (the `harness_ab.sh` pattern in `.benchmarks/006_probes/`, arms = `RIIR_LAYA_NO_BATCH=1` vs default) once preflight passes; record as Bench 006 Addendum 7. The reference does this and says so —
      `.raw/laya/laya/agent.py:267` *"Evaluate typed questions across state
      in a single, parallel forward pass"*, collated at `:291`. Ours runs one
      forward per question (`agent.rs:224`). The published losses track
      q/case exactly: `typed_decisions` at 5 q/case loses worst
      (+19.3/+23.8/+37.9%), `code_fixtures` at 2 q/case +17.6%. It is also
      where the GPU utilisation is: at `m = seq ≈ 317` the xwide GEMM launches
      **120 threadgroups for a 40-core GPU**; at `m = 5·seq` it launches 600.
      Needs a batch dimension with per-sequence masking through encoder +
      head + attention, and a G5 re-capture — a project, not a rung.
      **Evidence, T9:** on `code_fixtures` case 3 the 1q → 2q step is
      flat at every length (1.6–1.8× total at 2q against 1.25–1.45× at 1q),
      so batching is the single largest remaining gap on that suite too.
- [ ] **T6 — per-forward allocation churn.** ~60 MB of host `Vec`s rebuilt
      per forward (`encoder.rs:193`, `head.rs:172-208`) and every activation
      device buffer freed by the per-pass chain clear (`metal.rs:1252`).
- [ ] **T7 — the GEMM itself.** Measured at **3.6–4.9 TFLOP/s**, ~¼ of this
      GPU's fp32 peak, and **~58% of a banking77 forward** (52.5 ms of ~90 ms).
      With the coalescing question now ANSWERED (it was not the binding cost
      — T4a bought only 2–8% on the wide instances, confirming the repo's own
      BK=48 negative), the untried axes are occupancy (24 960 B of threadgroup
      memory caps residency), double-buffered staging, and an f16-operand
      instance behind its own feature flag + G5 re-gate.
      **Located, T9:** the loss is **shape-specific**. Rust's encoder GEMM
      beats torch MPS below m = 256 (0.89–0.93×) and loses above it
      (1.19–1.33×). At m = 283 it drops to 2.8–3.5 TF/s, against about 4.0–4.4
      at 188 and at 512. So the first rung is the wide/xwide pick at
      256 ≤ m < ~400 (tile padding at m = 283 is 283/320 on BM 64), before any
      new kernel.
      **Candidate measured, NOT landed (Bench 006 Addendum 5):**
      `XWIDE_N_MIN` 2048 → 1024 (xwide for the n = 1024 projections too).
      Per-length probe: −5…−6% at 512 tokens (12/12), −2…−5% at 283/461,
      but +1…2% at 321–427. Through the real harness (6 paired rounds,
      order alternated, accuracy identical every round): rounds 1–3 read
      0.77–0.93 B/A wall, but the load was FALLING 14.5 → 3.9 inside them
      and `massive_intent_en` (mostly short inputs, largely untouched by
      the change) read 0.88 there too — a load-trend artifact. At stable load
      5–6 (rounds 4–6): banking77 0.976 / 1.000 / 0.950, code_fixtures
      1.000 / 0.985 / 1.030. Inside noise → not landed. Re-run only on a
      box where `bench_preflight.sh` passes; the probe env switch lives in
      `.benchmarks/006_probes/` (never committed to the substrate).

- [x] **T8 — attribute the `code_fixtures` max** — DONE `e2d2060`.
      Step 1: `src/harness/latency.rs` — every lane now stamps
      `latency_extremes {first_ms, max_ms, argmax_case}` into `results.json`
      and the TABLES p99 cell reads `(1, max@caseN)` whenever the p99 IS the
      max. Step 2, 3 rounds, AC, load 3.7–4.0, all quotable: **rust's max is
      case 3 every round at exactly 231 ms**; rust's case 0 is 108–114 ms, so
      it is **not** a residual cold cost. Python's max is its case 0 (the
      cold start, 134–277 ms) in 2 of 3 rounds; in the round where python's
      case 0 was warm, its max was also case 3, at **153 ms**. So the +43%
      max is a **length-scaled** gap on one long input (case 3 =
      `code:engine.rs:1`, the second fn span of `engine.rs` per the builder
      order — a self-referential fixture, so its length moves when that file
      is edited), ~1.5× python there against ~1.07× at p50: consistent with
      a cost that grows with sequence length (attention / GEMM at larger
      `m`), which is T5/T7's territory, not T1's. Case 3's token length and
      what it costs: T9.

- [x] **T9 — case 3's length and what it costs** — DONE (Bench 006
      Addendum 4; probes in `.benchmarks/006_probes/`). Load 6–11, so ratios
      and within-run breakdowns only. **Case 3 is 512 tokens** (the english
      cap). Its gap is **mostly not length**:
      - Rust ÷ python at 2 questions is flat at **1.6–1.8× at every length**
        from 92 to 512 tokens. That is T5 batching.
      - At 1 question, the gap grows from about 1.25× (≤ 188 tokens) to about
        1.45× (≥ 283). That growth is the only length-dependent part, and it
        is all in the encoder.
      - The encoder GEMM wins below m = 256 (0.89–0.93× torch) and loses above
        it (1.19–1.33×). That goes to T7.
      - Attention is **about even in total** (rust ≈ 17 ms, torch ≈ 15 ms at
        512). Rust's full-attention layers are 2× slower than torch SDPA, but
        its sliding layers walk only the window where the reference runs seq²
        with a mask. That becomes T10.
      - ⛔ The earlier "13 → 42 ms superlinear non-GEMM jump" was a
        two-instrument subtraction artifact. In the forward, non-GEMM growth
        from 283 to 512 tokens is about 12 ms, 10 of it flash_attn.
- [ ] **T10 — full-attention `flash_attn` throughput** (small, low priority).
      - [x] **Rung 1 — parallel row softmax — LANDED riir-infer `0ec88a9`.**
            Both softmax steps ran serially on 32 of 1024 threads; now one
            simdgroup per row (`simd_max` bit-identical, `simd_sum` changes
            only the l order). Encoder paired patched/base **0.935 / 0.938 /
            0.939 / 0.924** at 188 / 283 / 370 / 512 tokens, **10/10** wins
            each; flash_attn kernel alone 0.79 → 0.62. G5 3×/arm
            deterministic, max prob drift ≤ 5.1e-6 (gate 1e-3), top-1 1.000.
            Load 7–9, AC, powermode 2 — ratios only (Bench 006 Addendum 5).
      - [ ] Rung 2 — hoist rope-K into one pre-pass per layer. Upside
            SHRANK after rung 3 (K is now staged once per tile, not twice):
            what is left is a coalesced `[h][64][seq]` read and dropping the
            cos/sin loads. Needs a per-layer device scratch through
            `AttnScratch`.
      - [x] **Rung 3 — one-pass online softmax — LANDED riir-infer
            `14af99f`** (taken before rung 2 because it halves the staging
            rung 2 optimizes). Row max/sum in registers, accumulator
            rescaled by one diagonal 8×8 MMA per tile. Encoder paired
            patched/base **0.965 / 0.975 / 0.983 / 0.979** at 188 / 283 /
            370 / 512 tokens, 38/40 wins; flash_attn kernel 0.98 → 0.82.
            G5 deterministic, drift ≤ 6.1e-6, top-1 1.000 (Bench 006
            Addendum 6 — AC, load 6.3–6.6, GPU canary clean; ratios only).
      On the 10 full-attention layers the fused kernel runs about **1.15
      ms/layer at 512 tokens (≈ 0.9 TF/s)**, against torch SDPA's
      **0.59 ms/layer (1.8 TF/s)**. Those layers scale quadratically as
      expected, so the loss is **efficiency, not complexity**. Suspects, from
      reading the kernel: two passes per query block, and K re-staged with
      rope applied on every key tile. Hoisting rope-K into one pre-pass per
      layer (it is O(seq·d), shared by every query block) is the cheap first
      rung. Ceiling: about 5–6 ms per 512-token forward. G5 re-gate needed.

## Gates every wave must hold

- **G5 parity green at BOTH postures** (`tests/laya_riir_parity.rs`) — the
  CPU lane is bit-identical and the Metal lane is ≤ 1e-3 drift / ≥ 99.9%
  top-1. A latency rung that moves parity is REVERTED, not argued about.
- `tests/metal_ops_smoke.rs` green (the ragged-shape arms).
- **Position-balanced A/B** for every timing claim — this repo has already
  paid for the cold-GPU artifact once (`README.md`: a 95→80 ms reading that
  was −3% under position balancing). Never publish a first-arm-first number.
- **Box state recorded with every latency figure** (load average, free RAM),
  per the AGENTS.md rule.
