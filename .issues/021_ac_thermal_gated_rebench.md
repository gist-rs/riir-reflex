# Issue 021 — re-bench the Issue 020 waves on AC, plug- and thermal-gated (the power axis nothing was recording)

**Status:** OPEN — T1, T3, T4, T5, T6, T7 DONE 2026-09-24 (AC re-bench =
Bench 006 Addendum 2); **only T2 (pin the canary reference) remains**, blocked
on a genuinely quiet window (load < 2 never occurred this session). Filed
2026-09-24 on the owner flag *"beware thermal and unplug recently, rebench if
need, file issue to bench again as plug and thermal gated"*.

## The gap

AGENTS.md's rule — *"a latency number without its BOX STATE is not a
measurement"* — enumerates free RAM, commit-vs-limit and concurrent heavy
jobs. It does **not** name the power source, and on a laptop that is a
first-order axis: Apple Silicon sheds sustained GPU clock off AC. Every
instrument in this repo records load and memory; none records power.

Measured from `pmset -g log`, this box:

| window (local, +0700) | power | what ran |
|---|---|---|
| 2026-09-23 22:20:59 → **2026-09-24 09:56:10** | **AC** | the published `bench.json` (`77c408e`, `date_utc 01:52:39Z` = 08:52:39 local) · Bench 006's two absolute baselines at 09:45 |
| **09:56:10** (unplugged at 100%) → 10:55 (48%) | **BATTERY** | **every paired A/B in Bench 006** |

- ✅ The **published arena table is AC on both columns** — its numbers are not
  in question.
- ⛔ **Bench 006's §1/§2/§3 A/B rounds are all battery.** Full disclosure and
  the survives/does-not-survive split: Bench 006 Addendum 1.
- ⚑ **The published-vs-isolated gap is NOT power, and that is new.** Both the
  published `massive_intent_en` rust cell (63/129) and the 09:45 isolated
  single-suite run (43.0/52.0, beating the published python 51/98 on both) are
  **AC**. So Issue 020 T0's open question — 15-suite single-process
  accumulated state vs a cold single-suite process — is a **state** effect,
  with power now eliminated as its explanation.

## Why a canary and not a sensor

There is **no sudo-free thermal or GPU-clock readout on this box**, measured
rather than assumed: `pmset -g therm` reports *"No thermal warning level has
been recorded"*, `kern.thermalpressure` does not exist, and `powermetrics`
requires root. A non-interactive agent session cannot read the throttle state
directly.

So the detector is a **fixed kernel whose absolute time is the signal**: run
one `317×1024×1024` `matmul_w` and compare against a reference taken on AC.
That is the whole design of T1's canary, and it is why T2 (pinning the
reference) is the gate's arming step rather than a nicety.

## Tasks

- [x] **T1 — `scripts/bench_preflight.sh`, the refusal gate.** Refuses on
      battery, refuses under Low Power Mode, refuses over `MAX_LOAD`
      (default 6.0), discloses swap and the last power transition, runs the
      canary, and prints a one-line `PROVENANCE:` string a bench record must
      quote. Deliberately carries **no EXIT trap and no temp files** — there
      is nothing for an aborted run to launder (the AGENTS.md trap-sentinel
      class), and every refusal exits non-zero with its reason named. First
      run on this box refused correctly:
      `REFUSE — on BATTERY (47%)` · `PROVENANCE: power=Battery Power
      load=4.05 swap=2947.94M canary=151.2us lpm=0`.
      ⛔ **Amended 2026-09-24 11:0x — its Low-Power check was WRONG on the
      first AC run.** `pmset powermode` on this M3 Max is a THREE-state enum
      (0 Automatic, 1 Low Power, 2 High Power), and this box's AC profile is
      `powermode 2` while its battery profile is `0`. The gate read "not 0"
      as Low Power and refused **High Power** — the best mode for a sustained
      number — so on battery it looked right for the wrong reason, and on AC
      it could never pass. Now: only `1` refuses, `0`/`2` pass and are named
      in provenance (`powermode=2(high)`, replacing `lpm=`), an unknown value
      exits 2. Same amendment **enforces** the settle window (it was printed
      and left to a reader) and drops two usage lines (`--canary-only`,
      `--record`) naming flags the script never implemented. First AC run
      after the fix refused correctly on the two remaining axes:
      `on AC for only 1 min (< SETTLE_MIN=5)` and `load 9.88 > 6.0`, canary
      **860.9 us** against 151–181 us quiet — so the canary does move with
      GPU contention, which is the property T2 relies on.
- [ ] **T2 — pin the AC canary reference.** ⚠ Measured 2026-09-24 (Bench
      006 Addendum 2): 148.0 / 151.6 / 172.6 µs on AC/High Power at load
      3.8–7 — a **16% spread** against the script's 15% tolerance, so pin a
      **best-of-N** (or widen the tolerance) at the same time, or the armed
      gate will cry wolf on an ordinary busy box. Original task: On AC, ≥ `SETTLE_MIN` minutes
      after plugging in, at load < 2, take the `317×1024×1024` canary and set
      `CANARY_REF_US` in the script. ⚠ It **must not** be taken on battery —
      a battery reference blesses the state the gate exists to refuse, which
      is this workspace's own *"a pin measured under the wrong conditions reds
      on every box but the one that produced it"* shape. Until it is pinned
      the canary PRINTS and never judges, and the gate says so.
- [x] **T3 — re-run Bench 006 §2 and §3 on AC** — DONE, Bench 006
      Addendum 2: narrow k=2624 GEMM **−21.5%** (−19…−24 every round,
      CONFIRMED); wide **−9.5 / −10.0%** (LARGER than battery's −4.4/−4.6 —
      a throttled GPU compressed the delta); xwide flat; `massive_intent_en`
      **≈ −5% p50 / −9% p99** over **10** paired rounds, 8/10 NEW — the
      battery "−8%" RETIRED as an over-statement. Arms: riir-infer `c6716a4`
      vs `6c56f04`. Original task: (`bench_preflight.sh` PASSED,
      provenance quoted in the record), position-balanced, ≥ 4 paired rounds.
      The numbers to re-confirm, in descending sturdiness: narrow-instance
      GEMM **−19…−24%**, `massive_intent_en` **−8%**, wide/xwide GEMM
      **−2…−8%**. Record as Bench 006 Addendum 2, never by editing §2/§3 —
      the battery rows stay on the record as what was measured.
- [x] **T4 — re-run Bench 006 §1 on AC.** — DONE: **−64…−68%** p99, p50
      unchanged in all five, REPRODUCED. Original task: Expected to reproduce (p50 unchanged
      while p99 falls 64–67% is its own internal control, and 144→52 ms is out
      of reach of a clock effect), which makes it the cheap **confirmation**
      that the AC re-bench is measuring the same thing.
- [x] **T5 — the absolute AC cell the arena actually needs.** — DONE, and
      the answer reframes the question: a same-run `--laya-python`
      head-to-head shows the **python oracle itself 16–30% faster than its
      published column**, so no rust-vs-published-python comparison is
      evidence. Same-run: rust **wins p99** (massive −18…−23%, ag_news
      −26…−38%, banking77 ±), **loses p50 ~10%** (massive +9…+10%, banking77
      +10…+17%, ag_news ±7). Class A NOT closed. 15-suite order: massive
      56/73 ≈ single-suite 55.5/67 → **no accumulated-state penalty at p50**
      (Issue 020 T0 answered). Original task: The one claim
      this session could never make: `massive_intent_en` / `banking77` /
      `ag_news` post-change p50+p99 on AC, against the published python
      column, single-suite AND in the published 15-suite order (which also
      answers Issue 020 T0 with power eliminated). This is what turns
      "−8% paired" into "rust beats the oracle at this cell".
- [x] **T6 — name the power axis in the rule.** Landed 2026-09-24: a
      sub-bullet in katgpt-rs AGENTS.md §Feature Flag Discipline G2 (the
      GENERAL rule; the §Docs gate INSTANCE paragraph is about docs-gate CPU
      figures and names no box-state list, so it needed no twin edit) + a
      GOAT-gates bullet in this repo's AGENTS.md pointing at the gate.
      Original task text: AGENTS.md's box-state
      paragraph should enumerate power source + Low Power Mode alongside free
      RAM and concurrent jobs, and point at `bench_preflight.sh`. ⚠ The same
      paragraph exists in katgpt-rs's CLAUDE.md §Feature Flag Discipline G2 —
      the two copies are already documented as able to drift; edit with that
      pointer in hand, do not silently fix one.
- [x] **T7 — decide whether the gate becomes mandatory.** Resolved to the
      recommended middle (owner-gated item → made the call per the
      workspace's "ask Claude for verdict" rule; reversible): **advisory,
      stamped**. `src/harness/box_state.rs` captures power source /
      powermode / load / swap at run START and END into `results.json`
      `meta.box_state` with a `latency_quotable` verdict (true / false +
      reasons / null = UNJUDGED when the probes are absent, e.g. the 4090),
      and TABLES.md prints it as a header line. Never refuses — a
      correctness run is not blocked. 3 unit tests over the pure parsers +
      verdict; verified end to end (a load-8.36 run stamped NOT QUOTABLE).
      ⚠ `MAX_LOAD` is duplicated with `bench_preflight.sh` — the constant's
      doc names the twin. Original task: Owner-gated: should
      `harness` itself refuse to write a `results.json` carrying latency
      fields when the preflight refuses, or stay advisory? A refusing harness
      cannot be bypassed by forgetting; an advisory one cannot block a
      correctness run that does not care about latency. The cheap middle is to
      **stamp the provenance line into `results.json`'s `meta`** so a table can
      never again be read without its power state — recommended, and it is
      also what would have caught this on the published run.

## Re-run

```sh
# plug in, wait out the settle window, then:
cargo build --release --features laya-riir-metal --example sgemm_shape_timing
scripts/bench_preflight.sh           # must PASS; quote its PROVENANCE line
MAX_LOAD=3 scripts/bench_preflight.sh  # tighter ceiling for a publishable cell
```
