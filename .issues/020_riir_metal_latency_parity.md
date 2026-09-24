# Issue 020 — the riir Metal lane must BEAT the python torch MPS oracle on every published cell (p50 AND p99)

**Status:** OPEN — **waves 1–2 LANDED and measured on AC; T5 batch-vs-loop A/B measured on a quiet box (Bench 006 Addendum 7: 5-q/case −8…−9% p50, 1-q gate landed)**; **T6 CLOSED NEGATIVE** (host/GPU split probe: the whole encoder host side incl. all allocation churn is 1.0–1.6% of forward wall — pooling cannot move case wall); **T7 CLOSED** — rung 1 (the dispatch band) LANDED **and its suite-p50 row PASSED 2026-09-25 (Bench 032/SUITE_AB: NEW/OLD −29…−36% p50 median on all three suites, 6/6 rounds, both load classes)**, the occupancy axis REFUTED at kernel level (bk32/bn32 both lose; code reverted), and the follow-up axes are now ALL measured-closed: **the MMA-roofline probe (`riir-infer-laya/examples/sgemm_roofline.rs`) measured the narrow instance staging-bound — shipped 3.1–4.9 TF/s vs MMA-only 5.1–10.2 TF/s (+63…+134% headroom) — and its f16-B arm then REFUTED the byte-halving lever (flat within ±2% on every cell; B fits L2, so re-reads were never DRAM traffic — the binding cost is the TG-issue path) — five axes now refuted at kernel level (occupancy, BK48, coalescing ×2, f16-B); narrow's shape is the measured local optimum, the lossy backend rung is dead before being built, and the GEMM axis reopens only on a Metal/toolchain change or an L2-oversized working set (n > ~4096)**; **T10 rung 2 (rope hoist) MEASURED 2026-09-25 and NOT PROMOTED (Bench 033: clean-round medians banking77 p50 −4.4% 2/3, banking77 p99 −14.3%, code_fixtures p99 −3.3%, massive flat — thinner than the rung-1/3 promotion bands; stays opt-in `LAYA_METAL_ROPE_HOIST=1`, not reverted — the attention-heavy p99 profile and a post-f16 re-price remain open doors)**
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
- [x] **T5 — the largest remaining Class-A lever: BATCH a case's questions
      into ONE forward.** **LANDED + MEASURED 2026-09-24** (riir-infer `da30007`, consumer gate reflex `f180cfc`, **[Bench 006 Addendum 7](../.benchmarks/006_issue020_latency_wave1.md)**) — `forward_packed`: GEMMs/elementwise over all rows in one pass, attention dispatched PER SEQUENCE at bind offsets (kernel untouched, exact per-sequence math, no padding), head per-question on device-copied slabs, one `begin_pass` per case, kill-switch `RIIR_LAYA_NO_BATCH=1` = the loop arm. Gates: lane `tests/packed_forward_equiv.rs` (CPU drift 1.4e-6 / 1e-5 gate, Metal arm), `metal_ops_smoke` 7/7, reflex G5 parity BOTH postures, `tests/laya_batch_parity.rs` top-1 1.000000 both checkpoints, clippy `-D` clean. **The position-balanced A/B ran on a quiet box (Addendum 7): 5-q/case −8…−9% median p50, 4/4 rounds per checkpoint; 2-q parity; 1-q control +4.5% median → single-question cases EXCLUDED from the packed pass the same day (`packed_eligible(n < 2)` — the gated posture dominates: multi-q wins measured, 1-q parity by construction; all gates re-green after). The first A/B attempt is DISCARDED as the confound lesson: a ~4-min load oscillation aliased with the alternating arm order and inverted the pooled sign — the per-run load trace is load-bearing (Addendum 7 §confounded). The pre-landing directional read (−18…−28% at load 25–35) is superseded: −8…−9% clean is the real effect.** The reference does this and says so —
      `.raw/laya/laya/agent.py:267` *"Evaluate typed questions across state
      in a single, parallel forward pass"*, collated at `:291`. Ours packs instead of padding — per-question answers are bit-identical
      to the loop, not "modulo padding". Residual multi-q headroom (the head still runs per-question on copied slabs) is the v2 packed head (strided `matmul_kt_heads`/`matmul_heads`/`merge_heads`) — contingent, not scheduled.
      **Evidence, T9:** on `code_fixtures` case 3 the 1q → 2q step is
      flat at every length (1.6–1.8× total at 2q against 1.25–1.45× at 1q),
      so batching is the single largest remaining gap on that suite too.
- [x] **T6 — per-forward allocation churn — CLOSED NEGATIVE 2026-09-25, measured
      before building (the head scratch-pool rung's discipline).** The sizing
      probe `riir-infer-laya/tests/metal_host_gpu_split.rs` (#[ignore]d,
      required-features row, run with `--ignored --nocapture`) splits one
      encoder forward at the real english geometry into `enq` (the wall of
      `forward_packed` alone — it contains no sync, so that is the WHOLE host
      side: dispatch encoding + the chain uploads/slots + the ~60 MB host
      `Vec` churn) and `sync` (`download_into`'s drain — the GPU side).
      Measured 2026-09-25, AC, load 11–12 falling, 88% RAM free, one sibling
      CPU bench ~6 cores, no GPU consumers: **seq188 enq p50 0.80 ms / sync
      p50 48.7 ms (min 36.1) — host share 1.6% · seq512 enq 1.45 ms / sync
      141.6 ms (min 132.6) — 1.0% · packed2x256 enq 1.37 ms / sync 135.2 ms
      (min 130.7) — 1.0%.** The host reading is load-INFLATED (CPU contention
      inflates the host, never the GPU), so a quiet box only shrinks it — the
      verdict is robust in the recorded direction. Allocation pooling can
      shrink only part of a 1.0–1.6% host share (dispatch encoding stays),
      which cannot move case wall; the per-pass chain clear stays. The 09-25
      follow-up's head-side scratch-pool rung (built, measured, moved
      nothing, reverted) was the same verdict at the head's scale — T6's
      premise is dead at the encoder's too.
- [x] **T7 — the GEMM itself — Rung 1 LANDED 2026-09-25: the dispatch BAND
      predicate** (substrate `metal.rs`, [Bench 032](../.benchmarks/032_m3_sgemm_dispatch_band/BENCH.md)).
      The pick was never swept per instance — this rung swept it: every
      hot-path family (encoder projections at m 231–512 AND packed scale
      1024–2048, head MHA at batch = 16, small-m 1–188) × all three
      instances × 3 position-rotated postures, load 2.9–5.6, win
      consistency 3-for-3 on 24/28 band cells. The old thresholds
      (`WIDE_M_MIN`/`XWIDE_N_MIN`) were measured backwards on most of the
      hot path — wide won ZERO cells — and the landed predicate is one
      mechanism: **xwide iff 24 < ⌈m/64⌉·⌈n/128⌉·batch ≤ 40 AND k ≥ 128**
      (xwide's fat single wave only pays when it mostly fills but does not
      spill one wave, over enough k to amortize the staging); **narrow
      otherwise; wide never picked** (kernel stays compiled, unpicked).
      Headline: o/wo @ m 231–317 −14…−18% (xwide band), wi −10…−22% and
      o/wo @ m ≥ 370 + packed scale −5…−27% (narrow), head MHA @ m ≥ 283
      −13…−29% (narrow; wide lost its whole upper range), small-m kept
      narrow. One known regression: qkv@317 +6.9% — swamped ≈ −3.9
      ms/question net at banking77's m ≈ 317. The instances are
      result-identical (instance-independent k-ascending chain — the probe's
      divergence check is bit-identical), so G5 drift is untouched; gates:
      substrate lib 41/41 + smoke 7/7 + packed gates + G5 BOTH postures +
      batch parity + clippy −D ×3 postures. **Suite-p50 row PENDING**: three
      clippy `-D` ×3 postures. **Suite-p50 row PASSED 2026-09-25
      (`.benchmarks/032_m3_sgemm_dispatch_band/SUITE_AB.md`)**: the frozen
      A/B on a preflight-clean window — NEW (band) / OLD (1e8c034
      m/n-threshold pick) medians −34.3% massive_intent_en, −29.1%
      banking77, −35.7% code_fixtures on clean rounds 4/5/6, direction-
      consistent 6/6 rounds in BOTH load classes (rounds 1–2 discarded on
      the sibling ppl bench restarting; isolation verified:
      `1e8c034..HEAD` substrate delta is exactly the band + the DEFAULT-OFF
      rope hoist). The suite sum lands ABOVE the kernel rows' range: the
      old pick mis-dispatched several GEMM families per question at once.
      Original rung text: the GEMM measured at 3.6–4.9 TFLOP/s, ~¼ of this
      GPU's fp32 peak, ~58% of a banking77 forward; the loss shape-specific
      (beats torch below m = 256 at 0.89–0.93×, loses above at 1.19–1.33×;
      at m = 283 2.8–3.5 TF/s vs 4.0–4.4 at 188/512 — the BM-64 padding
      valley). The earlier XWIDE_N_MIN candidate (Bench 006 Addendum 5,
      inside noise, not landed) is SUPERSEDED: its n-threshold question is
      answered by the band — xwide at n ≥ 2048 loses everywhere except the
      qkv@317 cell.
      **Occupancy axis REFUTED 2026-09-25 (kernel-level, both arms,
      bit-identical results, code reverted — only this record carries the
      negative, the BK=48 precedent):** narrow's 24 960 B staging caps
      residency at one threadgroup/core, and the hypothesis was that
      smaller staging → 2–3 co-resident threadgroups → latency hiding
      during the staging loads. Two challengers were built behind
      `LAYA_METAL_SGEMM_VAR` (unset = byte-identical), both keeping the
      k-ascending per-element chain (every probe row bit-identical):
      **bk32** (BK 32 at the same 32×64 tile, 12 544 B → 2 TGs/core) and
      **bn32** (32×32 tile, 8 448 B → 3 TGs/core). Instrument:
      `sgemm_shape_timing` (extended with four standing narrow-zone rows:
      m 188/512 @ 1024×1024, packed 1024 @ 3072/5248), 3 position-paired
      base/bk32 rounds + 1 bn32 run, load 10–11.5 recorded. **bk32 LOST
      17–33% on every resolvable cell, growing with k exactly as the
      barrier model predicts** (BK 32 doubles the k-loop's two barriers:
      512×1024×1024 +24…+32%, 1024×1024×3072 +21…23%, 317×1024×3072 +33%,
      317×1024×5248 +30–31%); small m-106 cells sat in the ±4% noise band.
      **bn32 lost harder** (m-106 cells +33…+45%, packed rows +18–32% —
      same barrier doubling plus halved B reuse). The 2–3× co-residency
      gain is real but strictly smaller than the barrier cost — so the
      occupancy axis is dead at BK < 64; at BK 64 a 2-TG fit would need to
      break the bank-conflict padding (stride 65) or the 8×8 block
      structure (BN ≤ 24 idles 4 of 16 simdgroups), and wide (BM 64, the
      other 1-TG shape) already lost zero-for-zero in the band rung.
      Narrow's shape is the local optimum among the tried geometries.
      Remaining untried axes for a later rung: double-buffered staging,
      f16 operands behind a flag + G5 re-gate + the Issue-750-T3 lossy
      law. **⚠ BOTH SUPERSEDED 2026-09-25 by the roofline probe's arms
      (riir-infer `be181ef` + `5872947`):** the MMA-only twin measured the
      kernel staging-bound (+63…+134% headroom) — which first pointed at
      f16-B as the lever — and the f16-B arm then measured FLAT (±2% on
      every cell incl. deep-k): B fits L2, re-reads were never DRAM
      traffic, the binding cost is the TG-issue path. Double-buffering is
      dead with it (it relieves staging latency, not issue-path cost).
      Five axes refuted at kernel level; reopen triggers recorded in
      riir-infer HISTORY.md (Metal/toolchain staged-layout change, or
      L2-oversized B at n > ~4096 — re-probe on the packed path first).

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
      - [x] **Rung 2 — the rope hoist pre-pass — IMPLEMENTED riir-infer
            `5ef7442`, DEFAULT-OFF behind `LAYA_METAL_ROPE_HOIST=1` —
            **MEASURED 2026-09-25, NOT PROMOTED (Bench 033)**.**
            `attn_rope` derives the Q/K rope ONCE per layer into a packed
            `[2, seq, d]` device scratch (grow-only in seq, the
            chain-cache-external weight-buffer lifetime class, write-first
            within the dispatch pair); `flash_attn`'s staging then COPIES
            the rotated K instead of re-rotating it per (query block ×
            head × key tile). Flag-off stays the in-kernel rope arm —
            bit-identical by construction (same expressions, same order).
            Gates green at BOTH postures: `metal_ops_smoke` 7/7 ×2,
            `packed_forward_equiv` 4/4 ×2, `packed_same_shape_gate` 1/1
            raw-bit, substrate lib 41/41, reflex G5 metal top-1 1.000000
            ×3 checkpoints drift ≤ 6.1e-6 ×2, `laya_batch_parity` ×2,
            clippy `-D` × 3 feature postures. One defect caught mid-landing
            and fixed in the same commit: the widened buffer list moved the
            flash staging to `[[threadgroup(11)]]` while the host still
            bound its length at index 9 — the unbound-pointer class; small
            smoke shapes passed on allocation luck, full-model G5 failed
            degenerate (uniform probs, agreement 12/26). **Promotion probe
            EXECUTED 2026-09-25 on a preflight-clean window** (6
            position-balanced rounds × 2 arms × 3 suites, one binary, the
            env flag as the switch; rounds 1/2/5 discarded on load spikes
            6.5–6.7 per the T5 lesson): clean-round medians ON/OFF —
            banking77 p50 **0.956** (wins r4/r6; r3 +1.5% is inside the
            1-ms quantization at 67–68 ms), banking77 p99 **0.857**,
            code_fixtures p99 **0.967**, massive p50 1.000, code_fixtures
            p50 1.000. Verdict per the BENCH.md rule: the win is real but
            concentrated in the p99s and one round — **thinner than the
            bands that promoted rungs 1 (10/10 at −6…−8%) and 3 (38/40 at
            −1.7…−3.5%)** → NOT promoted, NOT reverted; stays opt-in. Full
            table + rationale: `.benchmarks/033_rope_hoist_ab/BENCH.md`.
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

## Follow-up 2026-09-25 — the packed path's aliasing hazard, found and fixed

Chasing the T5 residual (the v2 packed head sizing) surfaced a LIVE
CORRECTNESS BUG in the packed path the A/B could not see: two same-shape
questions in one case could corrupt the second question's answers — a
shared CLS row and a stale `act_in`, act_probabilities 0.52/0.48 where the
loop reads 1.0/0.0, at the raw-f32-bit level. The T5 A/B's "accuracy
byte-identical" held because the rounded 4-decimal envelope and the
fixture corpus (distinct question lengths) both masked it.

Two stacked causes, both fixed substrate-side (riir-infer `2ea0197`):

1. `download_into` resolved a host slice to a device buffer by
   (base ptr, len ≥ src.len(), newest epoch) — and TIED keys fell through
   to HashMap iteration order, randomized per process. The encoder's
   per-forward scratch frees its host Vecs mid-case while their keys stay
   in the map; the packed slabs reuse those addresses; the CLS download
   then matched a dead key holding the raw token-embedding row (~1 run in
   3, flaky by RandomState).
2. Per-question host buffers malloc-reused addresses within the case's
   single chain epoch, aliasing the previous question's device state
   (the `act_in` upload path).

Fixes: deterministic download resolution (newest epoch, then the TIGHTEST
container — never iteration order); per-question slabs + a caller-owned
`HeadScratch` allocated up front and alive for the whole case; packed
answers' raw f32 bits now ride an always-on capture seam
(`PACKED_ACT_BITS`). New gate: riir-infer-laya
`tests/packed_same_shape_gate.rs` — real checkpoint, equal-shape pair,
packed-vs-loop at the raw-bit level; 12 consecutive greens across fresh
processes under a sweep the old code failed ~1 in 3.

**T5's published numbers are unaffected**: the fix changes buffer
lifetime, not the op stream — allocation volume per case is unchanged and
the op order is untouched, so the Addendum 7 A/B (same-build comparison)
stands. What the episode DOES retire: the "per-question answers are
bit-identical to the loop" phrasing in the T5 docs — the packed encoder
runs different-shaped GEMMs (different m → different sgemm instances →
different reduction order), so per-row outputs agree to the
`packed_forward_equiv` drift budget (1e-5/1e-4), not bit-for-bit; the
invariant that actually holds is rounded-envelope equality plus the
gate's raw-bit equality on the HEAD pipeline (which IS shape-identical
per question).

The v2 packed head (strided `matmul_kt_heads`/`matmul_heads`/
`merge_heads`) re-sizing note: the split probe measured the head at
~10% of a case's wall (packed: heads 43.1 ms of 447 ms at 2048 rows,
2q) with the loop-path head at 2.5–4.5 ms/question across seq 46–205 —
sized against dispatch count, not allocation churn (a scratch-pool rung
built and measured 2026-09-25 moved nothing: head wall is dispatch+GPU
bound; reverted). Batching the head across questions would remove the
per-question dispatch cost — worth at most a few percent of multi-q case
wall, contingent as before.
