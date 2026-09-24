# Bench 006 — Issue 020 waves 1–2: the first-forward cliff, per-forward waste, coalesced weight staging

**Status:** COMPLETE 2026-09-24 · baseline = `HEAD` in a DETACHED worktree
(`/tmp/reflex_base`, its own `CARGO_TARGET_DIR`) · arm = this working tree ·
M3, `--release`, `--features laya-riir-metal`, `LAYA_DEVICE=metal`,
english checkpoint.

⛔ **BOX STATE, because it is part of every number here.** This workstation
carried sibling agent sessions throughout: load average **3.4–14** during the
rounds that count, **25.4 / 41.7** during two rounds that are VOIDED below,
and ~3 GB of swap in use. Every latency claim is therefore a **paired,
position-balanced delta**, never an absolute. The one absolute reading taken
on a quieter box is labelled as such and is NOT used for a verdict.

⛔ **The code moved mid-bench.** `src/laya/` was carved to
`../riir-infer/crates/riir-infer-laya` by the Issue 008 consolidation
between the A/B runs and the write-up. Every number below was measured in
THIS repo against its pre-carve tree; the change then landed at riir-infer
**`6c56f04`** after a replay onto a byte-identical base, re-gated there with
G5 reporting the same drift to the digit. The baseline arm named below is
this repo's pre-carve `HEAD`.

## What changed

| # | change | where | numerics |
|---|---|---|---|
| T1 | weights placed on the device at LOAD (`Backend::warm_weight` / `warm_weight_2d`, `Encoder::warm`, `Head::warm`, called from `RiirAgent::load`) | `agent.rs`, `backend.rs`, `encoder.rs`, `head.rs`, `metal.rs` | none |
| T2a | `rope_tables` hoists `inv` out of the position loop | `ops.rs` | bit-identical (same `powf`, same cast, `half` evaluations instead of `seq·half`) |
| T2b | the `seq²` sliding-window mask is not built when the backend predicates the window in-kernel (`Backend::needs_window_mask`) | `encoder.rs`, `metal.rs` | none (the fused path already discarded it) |
| T2c | `thread_execution_width` resolved once into `Kern`; the per-dispatch operand `Vec` replaced by a stack array | `metal.rs` | none |
| T3a | ONE `MTLComputeCommandEncoder` per command buffer instead of one per dispatch | `metal.rs` | none (`MTLDispatchTypeSerial` already orders + barriers inside an encoder) |
| T4a | `matmul_w` binds `Wᵀ` row-major `[k, n]` from a load-time transposed cache, taking the kernel's coalesced `b_cs == 1` staging branch | `metal.rs` | bit-identical (same staged values, same k-accumulation order) |

## 1. The first forward — the p99 class

Every `tail support = 1` suite is 12–16 cases, so its reported **p99 is the
MAX, and the max is case 0**. Two runs per arm, position-balanced.

| suite | BASE p99 (ms) | NEW p99 (ms) | Δ | BASE p50 | NEW p50 | published `py/english` p99 |
|---|---|---|---|---|---|---|
| harness_visibility | 144, 140 | **52, 51** | **−64%** | 32, 31 | 32, 31 | 585 |
| harness_permissions | 144, 141 | **47, 53** | **−67%** | 35, 34 | 33, 34 | 129 |
| harness_routing | 143, 142 | **53, 52** | **−63%** | 33, 32 | 32, 32 | 153 |
| harness_sensitivity | 145, 138 | **57, 51** | **−63%** | 32, 33 | 32, 33 | 125 |
| harness_tool_fit | 138, 130 | **51, 41** | **−66%** | 25, 22 | 25, 21 | 152 |
| code_fixtures | 288, 278 | 285, 291 | **+3% (flat)** | 101, 101 | 100, 100 | 235 |

- **p50 is unchanged in all six** — this rung moves the cold call and nothing
  else, which is what a residency fix should do.
- ⛔ **`code_fixtures` is flat and that is the honest reading, not a miss.**
  Its 14 cases vary enormously in length, so its max is the **longest case**,
  not the cold one. Issue 020's Class-B fix cannot reach it; its +48.9%
  published p99 loss belongs to Class A.
- The four harness rows were all LOSING to the python oracle on p99 in the
  published table (+25.6% … +94.4%). At 41–57 ms they now beat it by
  **2.3×–11×**.

## 2. The GEMM kernel — `sgemm_shape_timing`, 4 position-balanced rounds

The forward's real `matmul_w` geometries. Median of 4 paired rounds, and the
per-round paired deltas beside it because round 1 carries this repo's
recorded cold-GPU first-position artifact.

| shape (m, k, n) | instance | BASE med µs | NEW med µs | Δ med | paired Δ% r1..r4 |
|---|---|---|---|---|---|
| 317×1024×1024 (O proj) | wide | 167.4 | 159.9 | −4.4% | −28, +12, −5, −8 |
| 317×2624×1024 (down proj) | wide | 460.4 | 439.1 | −4.6% | +4, −1, −6, −14 |
| 317×1024×3072 (QKV) | xwide | 431.7 | 430.4 | −0.3% | +24, +1, −2, −6 |
| 317×1024×5248 (gate/up) | xwide | 907.5 | 894.9 | −1.4% | +17, −2, −5, −5 |
| 106×1024×1024 (ag_news O) | narrow | 95.9 | 92.1 | −4.0% | +14, −4, −10, −10 |
| **106×2624×1024 (ag_news down)** | narrow | 274.9 | **222.3** | **−19.1%** | −8, **−20, −24, −24** |

**Read the narrow row as the finding.** The `k = 2624` narrow instance stages
a `[64][64]` B tile; at `b_cs = k` consecutive lanes were 10.5 KB apart, so
its staging had the least cache relief and gains the most from the flip
(−20/−24/−24% in the three non-cold rounds). The wide/xwide instances gain
**2–8%** — real but small, which **CONFIRMS this repo's own BK=48 negative**
(`README.md`: *"what binds is either the uncoalesced gather work itself or the
MMA"*): the gather was not the wide instances' binding cost.

**Bit-identity verified, not asserted:** the probe's CPU-vs-GPU relative
divergence is byte-identical on both arms for all six shapes —
`0e0, 0e0, 0e0, 0e0` and `2.2426077e-5` twice, the same values before and
after the rebinding.

## 3. End-to-end — `massive_intent_en`, alternating paired rounds

| round | order | BASE p50 | NEW p50 | Δ | box load |
|---|---|---|---|---|---|
| 1 | BASE→NEW | 77 | **69** | **−10.4%** | 9.9 |
| 2 | NEW→BASE | 71 | **66** | **−7.0%** | 12.7 |
| 3 | BASE→NEW | 67 | **63** | **−6.0%** | 12.9 |
| 4 | NEW→BASE | 78 | **65** | **−16.7%** | 9.0 |
| 5 | BASE→NEW | 99 | 162 | ⛔ VOID | **25.4** |
| 6 | NEW→BASE | — | 222 | ⛔ VOID | **41.7** |

**Median of the four valid paired rounds: −8%. NEW wins in every one of
them, in both positions.** p99 moves with it (130/107/87/103 → 84/107/78/75).

⛔ **Rounds 5–6 are VOIDED by box state, and the reason is recorded rather
than averaged in.** A sibling session's build storm took the load from 6 to
41.7 mid-experiment; NEW happened to hold both of those slots. An earlier,
coarser 3-round run of the same pair — taken across the same storm — read
`+24.6%` for NEW on this suite and `+30.9%` on banking77, i.e. a **confident
regression that does not exist**. That reading is recorded here precisely so
it is not rediscovered as a finding.

## 4. Where the forward's time actually is (sizing, not an A/B)

Summing the probe's per-shape medians at the english geometry, `seq ≈ 317`:
Wqkv 394 + Wo 160 + Wi 895 + mlp_wo 439 ≈ **1.89 ms of GEMM per encoder
layer × 28 layers ≈ 52.5 ms**, against a ~90 ms banking77 forward — so
**~58% of the forward is `matmul_w`**, running at a measured **3.6–4.9
TFLOP/s**. That is roughly a quarter of this GPU's fp32 peak and it is where
the remaining Class-A gap lives.

## 5. The structural finding this bench did NOT fix

The reference's own `system_one` docstring (`.raw/laya/laya/agent.py:267`)
reads *"Evaluate typed questions across state in a **single, parallel forward
pass**"*, and `:291` collates every question of a case into ONE padded batch.
`RiirAgent::system_one` (`agent.rs:224`) runs **one forward per question**.

That is exactly the shape of the published losses: `typed_decisions` is
**5 questions per case** and loses worst (+19.3% / +23.8% / +37.9% across the
three checkpoints); `code_fixtures` is 2 q/case (+17.6%); the 1 q/case suites
sit between −5.9% and +23.5%. Batching is Issue 020's largest remaining
Class-A lever and is filed as T5 there — it is a batch dimension through
encoder + head + attention with per-sequence masking, not a tuning rung.

## Gates

| gate | verdict |
|---|---|
| G5 parity, metal posture | **PASS** — english/typed/multilingual, 88/88 forwards, top-1 agreement **1.000000**, prob drift ≤ **4.016e-6** (bar 1e-3) |
| G5 parity, CPU posture | **PASS** — 2/2, 36.9 s |
| `metal_ops_smoke` | **7/7** green (all ragged/fused arms) |
| `cargo clippy --release --features laya-riir-metal --all-targets` | clean (also removed a pre-existing shadowed `let c = Cpu;` in the smoke test — the one warning the lane carried) |

## Re-run

```sh
git worktree add --detach /tmp/reflex_base <baseline-sha>
ln -s "$PWD/.raw" /tmp/reflex_base/.raw          # datasets are gitignored
( cd /tmp/reflex_base && CARGO_TARGET_DIR=/tmp/reflex_base_target \
    cargo build --release --features laya-riir-metal --bin harness \
    --example sgemm_shape_timing )

# kernel A/B (position-balanced; never compare across positions)
LAYA_DEVICE=metal ./target/release/examples/sgemm_shape_timing

# end-to-end paired rounds — ALTERNATE the order every round and record
# the load average beside every row; discard any round taken above ~15
LAYA_DEVICE=metal LAYA_WEIGHTS_DIR=~/.cache/riir-reflex/laya \
  ./target/release/harness --suites massive_intent_en --out /tmp/mi.json
```

---

## Addendum 1 (same day, owner flag "beware thermal and unplug") — ⛔ EVERY A/B ABOVE WAS TAKEN ON BATTERY

The `BOX STATE` block at the top of this record names load average, free RAM
and concurrent jobs — the three axes AGENTS.md's rule names. On a LAPTOP that
list is incomplete in a first-order way, and this record fell through the gap.

**Measured from `pmset -g log`:**

| window (local, +0700) | power | what ran in it |
|---|---|---|
| 2026-09-23 22:20:59 → 2026-09-24 **09:56:10** | **AC** | the PUBLISHED `bench.json` run (`77c408e`, `date_utc 01:52:39Z` = **08:52:39 local**) · this record's two absolute baselines at 09:45 |
| **09:56:10** (unplugged at 100%) → now (48%) | **BATTERY** | **every A/B in §1, §2 and §3 above** |

So the power axis splits this record in two, and not the way that would have
been convenient:

- ✅ **The published table is an AC measurement — BOTH columns.** Nothing about
  the arena's own numbers is in question here.
- ✅ **The two absolute baselines at 09:45 are AC**, and they are the only
  AC-to-AC comparison this session produced: pre-change `massive_intent_en`
  read **p50 43.0 / p99 52.0 ms** (load 4.71, tail support 4) against the
  published `py/english` **51 / 98**. ⛔ Read what that implies: the published
  rust cell for that suite is **63 / 129**, and both it and the 43/52 are AC —
  so the **published-vs-isolated gap is NOT power**, it is the 15-suite
  single-process accumulated state against a cold single-suite process. That
  is Issue 020 T0, still open, and it is now known to be a state effect rather
  than a power one.
- ⛔ **Every paired A/B — §1's cold-start rows, §2's sgemm rounds, §3's
  massive_intent rounds — was taken unplugged**, on a battery draining 100% →
  48% underneath them.

### What survives that, and what does not

**Survives — §1, the cold-start result.** Not because battery does not matter,
but because the run carries its own control: **p50 is unchanged in all five
suites while p99 falls 64–67% in the SAME runs.** A clock effect moves both.
An effect that moves only the first forward and leaves the median alone is not
a clock effect, and 144→52 ms is not a margin an uncontrolled power axis
reaches.

**Survives — bit-identity.** §2's byte-identical CPU-vs-GPU divergence and the
G5 drift matching to the digit are value claims, not timing claims. Power
cannot touch them.

**Does NOT survive as firm — the small deltas.** §3's **−8%** median on
`massive_intent_en` and §2's **−2…−8%** on the wide/xwide instances are small
enough that an uncontrolled power/thermal axis could plausibly account for part
of them. Alternating order cancels a *monotone* drift, and battery discharge is
monotone, which is the argument for keeping them — but it is an argument, not a
control. **They need an AC re-run before they are quoted anywhere.** The
narrow-instance **−19…−24%** is large and consistent across three rounds, so it
is the sturdiest of the Class-A numbers, and it too should be re-confirmed.

**Never valid, before or after this addendum:** comparing any absolute number in
§2/§3 against the published python column. Those are battery-vs-AC.

### The instrument that should have caught it

There is no sudo-free thermal-pressure or GPU-clock readout on this box
(measured: `pmset -g therm` records nothing, `kern.thermalpressure` does not
exist, `powermetrics` needs root). So the check has to be a **refusal plus a
measured canary**, and it now exists: `scripts/bench_preflight.sh` (Issue 021)
refuses on battery, refuses under Low Power Mode, refuses over a load ceiling,
discloses swap and the last power transition, and runs a fixed
`317×1024×1024` sgemm whose absolute time is the only throttle detector
available without root. Its first run on this box printed exactly the refusal
this addendum is about:

```
REFUSE — on BATTERY (47%) — Apple Silicon sheds sustained GPU clock off AC
PROVENANCE: power=Battery Power load=4.05 swap=2947.94M canary=151.2us lpm=0
```

⚠ The canary has **no AC reference pinned yet** — it prints and never judges,
because a reference taken on battery would bless the state the gate exists to
refuse. Pinning it is Issue 021 T2 and is the first thing the AC re-bench owes.
