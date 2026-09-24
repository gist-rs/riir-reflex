# Bench 006 — Issue 020 waves 1–2: the first-forward cliff, per-forward waste, coalesced weight staging

**Status:** COMPLETE 2026-09-24 + Addendum 1 (battery disclosure) + **Addendum 2 (AC re-bench — supersedes §2/§3's small deltas: GEMM wide −9.5/−10%, narrow −21.5%, massive_intent ≈ −5% p50 over 10 rounds; same-run vs python: p99 wins, p50 still loses ~10%)** + **Addendum 3 (same-run `code_fixtures`: rust loses p50 +7.4% median over 8 rounds, max +43% — attributed by Issue 020 T8 to one long case (case 3, 231 ms every round), NOT a cold start)** + **Addendum 4 (Issue 020 T9: case 3 = 512 tokens; its gap is mostly T5 batching (1q→2q step flat at every length) + GEMM at m≥256 (T7); attention ≈ even in total — the earlier "13→42 ms non-GEMM jump" is retracted as a two-instrument subtraction artifact; load 6–11, ratios only)** + **Addendum 5 (T10 rung 1 LANDED riir-infer `0ec88a9`: encoder −6…−8%, 10/10 paired wins, G5 green; T7 `XWIDE_N_MIN=1024` harness A/B inside noise at stable load → NOT landed)** + **Addendum 6 (T10 rung 3 LANDED riir-infer `14af99f`: one-pass online softmax — encoder −1.7…−3.5%, 38/40 paired wins, flash_attn −2…−21% growing with length, G5 green; a first contended run discarded)** · baseline = `HEAD` in a DETACHED worktree
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

---

## Addendum 2 (2026-09-24 11:12–11:41, Issue 021 T3/T4/T5) — the AC re-bench

**Box state, per run** (every row logged with its own load and power source,
`/tmp/abws021/results/log.tsv`, and each run preceded by a wait for 1-min load
< 4): **AC Power** on every row · `powermode 2` (High Power) · plugged in
11:03:52, first run 11:12:41 (8.8 min settle) · load **3.06–4.39** · swap
~2.8 GB. Preflight at start and end, quoted verbatim:

```
PROVENANCE: power=AC Power load=6.21 swap=2787.94M canary=151.6us powermode=2(high)
PROVENANCE: power=AC Power load=3.78 swap=2779.94M canary=172.6us powermode=2(high)
```

Both REFUSED at `MAX_LOAD=3` — this workstation never went below ~3 with
sibling sessions active, so every number below is a **paired or same-run
comparison**, never a publishable absolute, and the canary reference (T2)
stays unpinned (see the end of this addendum).

**Arms.** BASE = riir-infer **`c6716a4`** (the laya crate as it landed,
pre-Issue-020) · NEW = riir-infer **`6c56f04`** · both consumed by the same
riir-reflex HEAD, BASE from sibling worktrees in `/tmp/abws021/`
(`CARGO_TARGET_DIR=/tmp/abws021/target_base`; dep-info verified to compile
`/private/tmp/abws021/riir-infer/crates/riir-infer-laya/src/*`). This is a
cleaner A/B than §1–§3, whose arms differed by a working-tree edit.

### T3a — GEMM kernel, 4 position-balanced rounds (AC)

| shape (m, k, n) | instance | BASE med µs | NEW med µs | paired Δ% r1..r4 | paired med | battery §2 |
|---|---|---|---|---|---|---|
| 317×1024×1024 | wide | 167.6 | 151.8 | +8, −10, −11, −9 | **−9.5%** | −4.4% |
| 317×2624×1024 | wide | 483.7 | 437.9 | +10, −8, −13, −12 | **−10.0%** | −4.6% |
| 317×1024×3072 | xwide | 447.1 | 444.0 | +17, +1, 0, −2 | +0.5% | −0.3% |
| 317×1024×5248 | xwide | 955.9 | 941.2 | +7, −4, −4, −2 | −3.0% | −1.4% |
| 106×1024×1024 | narrow | 103.7 | 97.1 | +1, −17, −1, −12 | −6.5% | −4.0% |
| **106×2624×1024** | narrow | 301.3 | **232.3** | **−23, −24, −19, −20** | **−21.5%** | −19.1% |

- ✅ **Narrow `k = 2624` CONFIRMED on AC: −19…−24% in all four rounds**, both
  positions — the one Class-A kernel number that is firm by any standard.
- ⚑ **The wide instances gained MORE on AC than on battery** (−9.5 / −10.0%
  against −4.4 / −4.6%) — three of four rounds each, round 1 (BASE first) the
  recorded cold-GPU first-position artifact. The battery reading under-stated
  the win, which is the direction a clock-shedding confound predicts: a
  throttled GPU compresses every kernel-level delta toward zero.
- xwide stays **flat** (±3%, within round-1 artifact noise) — the BK=48
  negative's reading holds: the gather is not the xwide instances' binding
  cost.
- Bit-identity re-confirmed on AC: `rel 0e0` on the four exact shapes and
  `2.2426077e-5` on the two k=2624 shapes, identical on both arms, all rounds.

### T3b — end-to-end `massive_intent_en`, 10 alternating paired rounds (AC)

| round | order | BASE p50/p99 | NEW p50/p99 | Δp50 | Δp99 |
|---|---|---|---|---|---|
| 1 | B→N | 63/75 | 55/66 | −12.7% | −12.0% |
| 2 | N→B | 59/73 | 57/67 | −3.4% | −8.2% |
| 3 | B→N | 61/70 | 62/84 | +1.6% | +20.0% |
| 4 | N→B | 57/77 | 58/77 | +1.8% | 0.0% |
| 5 | B→N | 70/83 | 56/69 | −20.0% | −16.9% |
| 6 | N→B | 58/74 | 56/71 | −3.4% | −4.1% |
| 7 | B→N | 58/72 | 54/65 | −6.9% | −9.7% |
| 8 | N→B | 57/79 | 54/67 | −5.3% | −15.2% |
| 9 | B→N | 48/63 | 45/54 | −6.2% | −14.3% |
| 10 | N→B | 49/59 | 47/59 | −4.1% | 0.0% |

**Median paired Δ: p50 −4.7%, p99 −9.0%. NEW faster at p50 in 8 of 10**
(sign test one-sided p ≈ 0.055), in both positions (BASE-first median −6.9%,
NEW-first −3.4% — a ~1.7-pt position bias each way, so the position-cancelled
effect is ≈ −5%). Accuracy identical (0.7500) in all 20 runs.

- ⛔ **The battery §3 "−8%" was an OVER-statement and is retired.** The AC
  figure is **≈ −5% p50**, and it is the one that matches the arithmetic: a
  ~10% GEMM win on the wide instances × ~58% of the forward in `matmul_w`
  (§4) predicts ≈ −5.5%. Four rounds (§3) could not resolve a 5% effect on a
  box whose absolute p50 moved **45 → 70 ms** within twenty minutes at the
  same load class; ten can, barely.
- ⚠ Read the rounds-3/4 pair: two consecutive rounds read NEW ≥ BASE. A
  four-round run that happened to start there would have reported "no
  effect". n is part of this claim.

### T4 — first-forward p99, one process per suite, 2 alternating rounds (AC)

| suite | BASE p99 | NEW p99 | Δ | BASE p50 | NEW p50 |
|---|---|---|---|---|---|
| harness_visibility | 163, 160 | **50, 58** | −66% | 32, 29 | 30, 30 |
| harness_permissions | 154, 183 | **47, 62** | −68% | 33, 38 | 32, 39 |
| harness_routing | 179, 190 | **65, 63** | −66% | 36, 36 | 36, 37 |
| harness_sensitivity | 183, 179 | **67, 64** | −64% | 37, 36 | 37, 38 |
| harness_tool_fit | 193, 185 | **62, 63** | −67% | 30, 29 | 27, 28 |

✅ **REPRODUCED on AC: −64…−68%, p50 unchanged in all five.** Addendum 1's
argument that §1 carried its own control holds, now with the control run too.
(BASE p99 reads 154–193 here against 138–145 in §1 — one cold process per
suite rather than one per invocation of the §1 run; the delta, not the level,
is the claim.)

### T5 — the absolute cells, and why the published python column cannot be the comparator

**Same-run head-to-head** — `harness --laya-python`, the rust lane and the
torch-MPS oracle answering byte-identical questions in ONE process, NEW arm,
2 rounds (accuracy identical per suite, rust = python):

| suite | rust p50/p99 r1, r2 | py p50/p99 r1, r2 | Δp50 | Δp99 | published py |
|---|---|---|---|---|---|
| massive_intent_en | 43/55, 47/63 | 39/71, 43/77 | **+10.3%, +9.3%** | −22.5%, −18.2% | 51/98 |
| banking77 | 69/87, 75/97 | 59/85, 68/105 | **+16.9%, +10.3%** | +2.4%, −7.6% | 84/162 |
| ag_news | 30/49, 29/46 | 28/66, 31/74 | +7.1%, −6.5% | −25.8%, −37.8% | 37/98 |

⛔ **The python oracle is itself 16–30% faster on this box now than its
published column** (39–43 vs 51 on massive_intent, 59–68 vs 84 on banking77).
So every rust-vs-published-python comparison in this record and in Issue 020 —
including Addendum 1's "43.0/52.0 beats the published 51/98" — compared two
different box states and is **not evidence either way**. The only valid
comparator is a same-run cell, and on that:

- ✅ **p99: rust wins** on massive_intent (−18…−23%) and ag_news (−26…−38%);
  banking77 is a coin flip (+2.4 / −7.6%).
- ⛔ **p50: rust still LOSES by ~10%** on massive_intent and **10–17%** on
  banking77; ag_news is a wash. **Class A is NOT closed**, and wave 1 did not
  change that sign — it narrowed it by ~5 points (T3b). Issue 020 T5
  (question-batching) and T7 (the GEMM at ~¼ of peak) remain the levers.

**15-suite published-order run** (rust only, NEW, AC, load 3.06→3.49, 9.4
min): every rust cell is at or below its published-rust value except
banking77 p99 (185 vs 116, tail support 6 — a max-class reading), and the
Issue 020 T0 question comes out the other way from how it was posed:
`massive_intent_en` reads **56/73** in 15-suite order against a single-suite
NEW median of **55.5/67** — **no measurable accumulated-state penalty** at
p50. The published **p50 63** is within this box's run-to-run spread of the
pre-change code (BASE single-suite p50 48–70, median 58); the published
**p99 129 is reproduced by NONE of ten BASE runs** (59–83, median 73.5), and
its cause is unmeasured — the published run's box state was never recorded,
which is the gap Issue 021 exists for. There is no 15-suite effect left to
chase at p50. Notable in the same run: `code_fixtures`
p50 **79** against the published 147 (and python's 125 from the published
run — ⛔ a cross-run comparison, per the rule above; the same-run cell is
owed before it is quoted), and every harness suite now below the published
python column on BOTH p50 and p99.

### The canary, and T2

Canary readings (317×1024×1024, NEW binary): **148.0 µs** (11:09, load ~7),
**151.6** (11:12, load 6.2), **172.6** (11:41, load 3.8); **860.9** at 11:04
(load 9.9, one minute after plug-in). So it moves by ~16% between two
AC/High-Power readings at similar load — the tolerance of 15% in the script
is too tight for this box as it is used, or the quantity needs best-of-N.
**Not pinned**: T2's condition (load < 2) was not met at any point this
session, and a reference taken at load 4–6 would be read as "throttled" on a
quiet box. Left for a genuinely quiet window.

### Re-run

```sh
mkdir -p /tmp/abws021
git -C ../riir-reflex worktree add --detach /tmp/abws021/riir-reflex HEAD
git -C ../riir-infer  worktree add --detach /tmp/abws021/riir-infer  c6716a4
ln -s "$PWD/../katgpt-rs" /tmp/abws021/katgpt-rs
ln -s "$PWD/.raw" /tmp/abws021/riir-reflex/.raw
( cd /tmp/abws021/riir-reflex && CARGO_TARGET_DIR=/tmp/abws021/target_base \
    cargo build --release --features laya-riir-metal --bin harness --example sgemm_shape_timing )
cargo build --release --features laya-riir-metal --bin harness --example sgemm_shape_timing
scripts/bench_preflight.sh     # quote PROVENANCE
# paired rounds: alternate BASE/NEW order per round, wait for load < 4 before
# EVERY run, log load + power per row; same-run h2h: --laya-python
```

## Addendum 3 (2026-09-24 11:55–12:05) — the owed same-run `code_fixtures` cell

Addendum 2 quoted `code_fixtures` p50 **79** (15-suite run) against the
published **147** rust / **125** python and marked the python comparison
*owed*. Paid here. Build: DETACHED worktrees at riir-reflex `122276b` +
riir-infer `0121a3b` (committed HEADs — a sibling's uncommitted riir-infer
`agent.rs` edit is excluded by construction), own `CARGO_TARGET_DIR`,
`--release --features laya-riir-metal`, `harness --suites code_fixtures
--laya-python`, AC / powermode 2. Box state is the harness's own T7 stamp
per run: rounds 1–3 **NOT quotable** (load 6.7–8.7, a sibling build), round 4
start-refused / end-quotable, rounds 5–8 **quotable** (load 5.1–5.5). Rust
answers before python in every round — the harness cannot alternate lane
order, so this is position-UNbalanced and disclosed as such.

| round | load | rust p50 / max | python p50 / max | Δp50 |
|---|---|---|---|---|
| 1 | 8.68 | 81 / 231 | 75 / 161 | +8.0% |
| 2 | 7.86 | 79 / 229 | 82 / 161 | −3.7% |
| 3 | 6.70 | 80 / 230 | 77 / 163 | +3.9% |
| 4 | 6.13 | 77 / 223 | 70 / 152 | +10.0% |
| 5 | 5.51 | 77 / 224 | 67 / 152 | +14.9% |
| 6 | 5.16 | 78 / 223 | 71 / 166 | +9.9% |
| 7 | 5.45 | 77 / 224 | 75 / 151 | +2.7% |
| 8 | 5.13 | 78 / 224 | 73 / 165 | +6.8% |

(n = 24 questions per lane, so the harness's "p99" has tail support **1** —
it is the MAXIMUM, printed here as max. Accuracy identical, 0.5417 both.)

- ⛔ **Rust LOSES p50 on `code_fixtures`: median paired +7.4%, 7 of 8
  rounds** (quotable rounds 5–8 alone: +2.7…+14.9%, same sign). Rust is
  rock-steady at 77–81 ms; python moves 67–82. The published 147→79 rust
  improvement is real; the claim *"every harness suite now below the
  published python column"* is **retracted for `code_fixtures`** — that column
  (125) was a different box state, and same-run python is ~74.
- ⛔ **The max is worse by a median +43%** (rust 223–231 vs python 151–166),
  every round, well outside either lane's spread. **Unattributed**:
  `results.json` does not persist per-question latencies, so whether the max
  is the first question (a residual cold cost the Issue 020 T1 fix did not
  reach on this suite) or a specific long input cannot be read off this run.
  Filed as Issue 020 T8 rather than guessed at.
- ⚑ This is the third suite (after `massive_intent_en`, `banking77`) where
  the same-run p50 sign is rust-loses by 5–15%. Class A's shape is consistent;
  T5 (question batching — `code_fixtures` is 2 q/case) remains the lever.
- ✅ **Attributed (Issue 020 T8, `e2d2060`, 3 more rounds, AC, load 3.7–4.0,
  all quotable):** with `latency_extremes` stamped, rust's max is **case 3
  at 231 ms in every round** while its case 0 reads 108–114 ms, so it is
  **not** a residual cold cost. Python's max is its cold case 0 (134–277 ms)
  in 2 of 3 rounds, and its case 3 reads **153 ms** when visible. The +43%
  is therefore one long input (`code:engine.rs:1`) on which rust is ~1.5×
  python, against ~1.07× at p50 — a sequence-length-scaled cost.

## Addendum 4 (2026-09-24 12:20–12:46, Issue 020 T9): what makes case 3 slow, and it is mostly not length

**Box state, recorded because it limits what can be quoted:** AC, `powermode
2` (High Power), 88% memory free, **load 6–11 throughout**. The riir-reflex
`bench_preflight.sh` gate **refused** on load. So every figure below is either
a **paired, interleaved ratio** or a **within-run breakdown**. None is an
absolute cell, and none replaces a published number. Probes are built from
committed code only (detached worktrees at riir-reflex `fba613e` and riir-infer
`0121a3b`, their own target dir); sources are archived in
[`006_probes/`](006_probes/README.md).

**Case 3 is 512 tokens**, the english checkpoint's cap: a 48-line fn span, so
it is the longest input the suite can produce.

### A. Rust vs python on the SAME input cut to different lengths (10 rounds, interleaved, lane order alternated)

Ratio is rust ÷ python, round-trip timed identically for both lanes by one driver.

| tokens | 1 question | 2 questions (the suite's real shape) | rust head share |
|---|---|---|---|
| 92 | 1.29× | 1.77× | 11% |
| 188 | 1.21× | 1.61× | 10% |
| 283 | 1.41× | 1.76× | 9% |
| 370 | 1.45× | 1.82× | 9% |
| 512 | 1.45× | 1.64× | 8% |

- **2 questions:** flat at 1.6–1.8× at **every** length. That is **batching**
  (python runs both questions in one forward, rust runs two). It is **T5**, not
  length.
- **1 question:** grows from about 1.25× (≤ 188 tokens) to about 1.45×
  (≥ 283 tokens). That growth is the only length-dependent part, and it is all
  in the encoder (the head stays at 8–11% of rust's time).
- ⚠ At load 7–9 these are worse than Addendum 3's load-4 cells (about 1.07×
  p50, 1.5× case 3). Rust's case-3 time rose 21% under the load against
  python's 11%. That suggests rust is more load-sensitive, but it is
  **unconfirmed**.

### B. Encoder GEMM, rust vs torch MPS (4 projection shapes × 28 layers)

| tokens | rust | torch | rust ÷ torch |
|---|---|---|---|
| 92 | 19.4 ms | 21.8 ms | 0.89× |
| 188 | 34.5 ms | 37.1 ms | 0.93× |
| 283 | 65.0 ms | 48.7 ms | 1.33× |
| 512 | 93.1 ms | 78.1 ms | 1.19× |

Rust **wins below m = 256 and loses above it**. At 283 it drops to 2.8–3.5
TF/s, against 4.0–4.4 at 188 and at 512. That is the wide/xwide geometry that
switches in at m ≥ 256 (in-forward dispatch counts at 512 tokens: 64
`sgemm_wide` + 60 `sgemm_xwide`, 4 narrow). This goes to **T7**, now with a
located shape.

### C. Attention, per kernel, in the forward (`LAYA_KTIME=1`, 7 rounds)

Each dispatch is committed and waited alone, so every row also pays about
0.1–0.2 ms per call (60 `add` calls read 13–15 ms, nearly all overhead). Read
the **deltas** and the split, not the absolutes.

| tokens | flash_attn total | 10 full-attention layers | 18 sliding layers | sgemm (all) |
|---|---|---|---|---|
| 188 | 10.6 | 3.5 | 5.4 | 58.3 |
| 283 | 13.9 | 5.5 | 7.0 | 81.9 |
| 370 | 16.2 | 7.7 | 7.5 | 95.5 |
| 461 | 22.7 | 12.1 | 8.8 | 112.0 |
| 512 | 24.2 | 13.5 | 9.2 | 114.8 |

- **Full-attention layers (0, 3, …, 27) scale quadratically.** From 283 to 512
  tokens, after taking out about 0.2 ms × 10 dispatches: 3.5 → 11.5 ms =
  **3.3×** against (512/283)² = 3.27×. **Sliding layers scale linearly**
  (the fused kernel walks only the ±64 window).
- **torch SDPA at the same shape** (`[1, 16, S, 64]`, load 11, so a range and
  not a figure): full attention at 512 tokens is **0.59 ms/layer (1.8 TF/s)**
  against rust's roughly **1.15 ms/layer (≈ 0.9 TF/s)**. That is 2× slower per
  full-attention layer. But the reference runs **all 28 layers** at seq² (HF
  ModernBERT's `sdpa` path applies the sliding window as a mask), where rust
  walks only the window on 18 of them. Model totals at 512 tokens: rust ≈
  11.5 + ~5.6 ≈ **17 ms**, torch ≈ **15 ms**. **Attention is about even. It is
  not what makes case 3 slow.**
- ⛔ **Retracted: the previous session's "non-GEMM encoder time jumps
  13 → 42 ms from 283 to 512 tokens, superlinear".** That residual was
  (unserialized encoder wall) minus (a **standalone** GEMM timing), which is
  two instruments subtracted. The standalone GEMM at 283 read slow, which is
  §B's anomaly, and that pushed the residual down. Measured **in the
  forward**, non-GEMM growth from 283 to 512 is about **12 ms, 10 of it
  flash_attn** (8 of that in the full-attention layers). The other kernels are
  flat within noise.

### Verdict

Case 3's 1.5–1.8× gap, in order of size:
1. **Question batching (T5).** It accounts for the whole 1q → 2q step
   (~1.25–1.45× → ~1.6–1.8×) at every length. By far the largest lever.
2. **GEMM at m ≥ 256 (T7).** Rust's GEMM goes from winning (0.89–0.93×) to
   losing (1.19–1.33×) exactly where the wide/xwide geometry takes over.
3. **Full-attention flash_attn efficiency (new T10, small).** It runs at about
   half of torch's per-layer throughput, but it is only 10 layers, and the
   window walk on the other 18 more than offsets it. Worth about 5–6 ms at
   512 tokens. Low priority.

Re-quote §A/§B/§C as absolutes only after `bench_preflight.sh` passes.

## Addendum 5 (2026-09-24 12:58–13:25, Issue 020 T7 candidate + T10 rung 1)

**Box state:** AC (battery 100%, charged), `powermode 2` (High Power),
~9 GB free + ~24 GB inactive of 64 GB, 5+ concurrent agent sessions (a
sibling ANE/Metal timing session and cargo builds among them). **Load 3.9–14.5**
across the session. `bench_preflight.sh` would refuse on load, so every figure
below is a **paired, interleaved ratio**, never an absolute cell.

### A. T7 candidate — `XWIDE_N_MIN` 2048 → 1024, through the real harness

Same binary, the probe env switch `LAYA_XWIDE_N_MIN` (probe-only, never
committed), suites `massive_intent_en,banking77,code_fixtures`, 6 rounds,
arm order alternated per round. Accuracy was **identical in both arms every
round** (0.750 / 0.498 / 0.5417).

| suite | wall B/A per round (1 … 6) | median | p50 B/A per round | median |
|---|---|---|---|---|
| banking77 | 0.773 0.847 0.795 0.976 1.000 0.950 | 0.898 | 0.795 0.796 0.743 0.966 0.966 0.921 | 0.859 |
| code_fixtures | 0.815 0.929 0.829 1.000 0.985 1.030 | 0.957 | 0.772 0.862 0.729 1.052 1.020 0.902 | 0.882 |
| massive_intent_en | 0.880 1.056 1.000 1.023 1.018 0.996 | 1.009 | 0.833 1.019 1.018 1.017 1.000 1.000 | 1.009 |

Load per run: 9.7→14.5, 14.5→12.2 | 12.2→8.3, 8.3→5.5 | 5.5→4.4, 4.4→3.9 |
3.9→5.7, 5.7→5.8 | 5.8→6.0, 6.0→5.3 | 5.3→5.9, 5.9→7.0.

- ⛔ **The medians are not the finding.** Rounds 1–3 ran while the load FELL
  from 14.5 to 3.9, so the second arm of each pair ran on a quieter box. The
  control shows it: `massive_intent_en`, whose inputs are mostly below the
  m ≥ 256 threshold the change touches, read **0.88 in round 1**.
- At **stable load 5–6 (rounds 4–6)** the change is inside noise:
  banking77 0.95–1.00, code_fixtures 0.985–1.03.
- **Verdict: NOT landed.** It stays a measured candidate for a quiet-box
  re-run. The per-length probe's −5…−6% at 512 tokens is real but is offset
  by +1…2% at 321–427, and the suites mix lengths.

### B. T10 rung 1 — flash_attn row softmax on one simdgroup per row (LANDED riir-infer `0ec88a9`)

Two resident lanes (base binary vs patched binary, same probe tree,
`english` checkpoint), 3 warm-up passes, 10 rounds, length order shuffled
and pair order alternated per round; load 8.2–8.7.

| tokens | encoder base | encoder patched | paired B/A median | B wins | flash_attn A → B (`LAYA_KTIME`, serialized) | B/A |
|---|---|---|---|---|---|---|
| 188 | 39.4 | 37.0 | **0.935** | 10/10 | 8.97 → 7.12 | 0.794 |
| 283 | 65.0 | 60.7 | **0.938** | 10/10 | 12.31 → 8.76 | 0.712 |
| 370 | 82.0 | 76.5 | **0.939** | 10/10 | 15.26 → 10.77 | 0.706 |
| 512 | 106.3 | 98.4 | **0.924** | 10/10 | 22.95 → 14.25 | 0.621 |

(ms columns are load-8 medians — read the ratios.) The encoder saving at
512 (≈ 8 ms) matches the kernel's own saving, so the win is the kernel, not
a second effect. It is larger than Addendum 4 §C's "5–6 ms ceiling" for the
whole of T10, because that ceiling was sized against torch's per-layer
throughput on the 10 full-attention layers. This rung also speeds up the 18
sliding-window layers, which run the same serial softmax.

**G5** (`tests/laya_riir_parity.rs`, `LAYA_DEVICE=metal`, 3 runs per arm,
alternated; riir-infer at `0121a3b` for BOTH arms because riir-reflex
`origin/develop` already needs its `DeviceKind::Ane` — `metal.rs` is
byte-identical between `0121a3b` and the commit's parent `b70fee8`):
deterministic across runs within each arm.

| checkpoint | top-1 (both arms) | prob drift base | prob drift patched | gate |
|---|---|---|---|---|
| english | 26/26 | 4.02e-6 | 1.90e-6 | ≤ 1e-3 |
| typed-decisions | 26/26 | 9.81e-7 | 2.65e-6 | ≤ 1e-3 |
| multilingual | 36/36 | 3.52e-6 | 5.10e-6 | ≤ 1e-3 |

The new summation order moves drift by a few 1e-6 in either direction, still about 200× under the gate.
`metal_ops_smoke` 7/7; clippy `-p riir-infer-laya --features laya-riir-metal
--all-targets` clean. Probe driver: `006_probes/flash_ab.py`.

## Addendum 6 (2026-09-24 14:45–14:52, Issue 020 T10 rung 3 — LANDED riir-infer `14af99f`)

**One-pass online softmax in `flash_attn`.** The two-pass form's max-only
pass re-staged every K tile (rope included) and re-ran every score MMA; now
each tile is staged and scored ONCE, the row max / sum ride in registers
(simdgroup `sg` owns row `sg`), and the accumulator is rescaled by
`exp(m_old − m_new)` through ONE diagonal 8×8 MMA per tile (`dg`, +1 KB
threadgroup staging → 30 592 B of 32 768). Rung 3 was taken before rung 2
(the rope-K pre-pass) on purpose: it halves the K staging rung 2 would
have optimized, so rung 2's remaining upside is only the coalesced read +
the dropped cos/sin loads on ONE staging per tile.

Same probe tree and driver as Addendum 5 §B (`006_probes/flash_ab.py`,
detached worktrees of riir-infer `883ddd6` + riir-reflex `8028a10` (plus the sibling's uncommitted `runner.rs` `laya-riir-ane` cfg guard, without which `8028a10` does not build `--features laya-riir-metal`),
`riir-infer-ktime-split.diff` on both arms; line counts `10,20,30,45` →
188/283/370/512 tokens), 3 warm-ups, 10 rounds, order shuffled + pair order
alternated. **Box: AC, powermode 2, load 6.3–6.6, GPU canary 147.9 µs vs
141 µs reference; `bench_preflight.sh` REFUSED on the 6.0 load ceiling only
— ratios, not absolutes.**

| tokens | encoder base | encoder patched | paired B/A median | B wins | flash_attn A → B (`LAYA_KTIME`, serialized) | B/A |
|---|---|---|---|---|---|---|
| 188 | 43.4 | 42.1 | **0.965** | 8/10 | 7.62 → 7.49 | 0.983 |
| 283 | 71.4 | 69.7 | **0.975** | 10/10 | 9.36 → 8.88 | 0.948 |
| 370 | 88.4 | 86.8 | **0.983** | 10/10 | 11.42 → 9.05 | 0.793 |
| 512 | 114.8 | 112.4 | **0.979** | 10/10 | 15.04 → 12.38 | 0.824 |

Smaller than the kernel's two-pass cost suggested (the retired pass was
~half the tile work): the rung-1 row softmax had already made the max pass
cheap, and on the 18 sliding-window layers each query block walks only ~5
key tiles, so the fixed per-block cost (Q staging, drain) dominates there.
The saving grows with length because it lives in the 10 full-attention
layers.

⛔ **A first 10-round run (14:31–14:34) is DISCARDED** — the GPU canary read
340 µs (2.4× its reference) because another session's
`/tmp/ane_p1/release/harness --out .benchmarks/001_phase1_tables_metal`
started at 14:32 inside the window. Its encoder columns were 2.2–2.5×
Addendum 5's, and its ratios (0.970–1.003, 4–8/10 wins) are a contended
measurement. The overlap cuts BOTH ways: that harness's Metal numbers from
14:32–14:34 shared the GPU with this A/B.

**G5** (`tests/laya_riir_parity.rs`, `LAYA_DEVICE=metal`), run on the probe
tree and twice on the real trees — deterministic, identical digits:

| checkpoint | top-1 | prob drift (rung 1, Addendum 5) | prob drift (rung 3) | gate |
|---|---|---|---|---|
| english | 26/26 | 1.90e-6 | 3.21e-6 | ≤ 1e-3 |
| typed-decisions | 26/26 | 2.65e-6 | 1.09e-6 | ≤ 1e-3 |
| multilingual | 36/36 | 5.10e-6 | 6.08e-6 | ≤ 1e-3 |

`metal_ops_smoke` 7/7; clippy `-p riir-infer-laya --features laya-riir-metal
--all-targets` clean. Output: `006_probes/flash_ab_r3.out`.
