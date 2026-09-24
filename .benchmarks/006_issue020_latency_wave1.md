# Bench 006 — Issue 020 waves 1–2: the first-forward cliff, per-forward waste, coalesced weight staging

**Status:** COMPLETE 2026-09-24 + Addendum 1 (battery disclosure) + **Addendum 2 (AC re-bench — supersedes §2/§3's small deltas: GEMM wide −9.5/−10%, narrow −21.5%, massive_intent ≈ −5% p50 over 10 rounds; same-run vs python: p99 wins, p50 still loses ~10%)** + **Addendum 3 (same-run `code_fixtures`: rust loses p50 +7.4% median over 8 rounds, max +43% — unattributed, Issue 020 T8)** · baseline = `HEAD` in a DETACHED worktree
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
