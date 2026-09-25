# Bench 035 — Issue 020 closing measurement: the same-run rust-vs-python table after waves 1–2 + T5 + T7 + T10 — the big losses FLIPPED, a stable small-sequence band remains

Three full 15-suite same-run tables (`--laya-python`, one process, both lanes
per suite) on 2026-09-25, metal lane, binary `/tmp/rhoistab/release/harness`
(reflex `902f409` + substrate `9a4e17a` — the landed band predicate, T5
packing, T10 rungs; rope hoist OFF = shipped default in all three). Raw
runs: `/tmp/issue020_closing{2,3,4}/results.json` (ephemeral; the tables
below are the record).

**Box states** (`bench_preflight.sh` / the harness's own Issue-021 stamp):
run 2 REFUSED (start load 6.14 — a sibling job landed mid-window; directional
only); runs 3 and 4 PASSED (3.46→2.91 and 2.77 start, powermode 2, canary
135.9 µs). **No single table is the record** — the three disagree on
individual cells by more than their margins while agreeing on the shape, so
per the two-box/one-hour lesson the verdict quotes the ACROSS-RUN RANGE per
cell, never one run's number.

## The across-run table (rust p50 vs same-run python p50; Δ = rust/py − 1)

| suite | Δ across 3 runs | published Δ (pre-waves) | verdict |
|---|---|---|---|
| typed_decisions · english | **−19.9 / −29.4 / +33.2%** | +19.3% | FLIPPED to win (2/3; r4's +33% is the cold-first-suite signature — see below) |
| typed_decisions · multiling. | **−21.9 / −30.1 / +2.3%** | +37.9% | FLIPPED to win |
| typed_decisions · typed | −1.1 / +25.3 / +22.7% | +23.8% | UNSTABLE — 1 win, 2 losses; not claimed either way |
| massive_intent_en | +4.8 / +36.2 / +4.4% | +23.5% | narrowed to **≈ +5%** (r3's +36% is the outlier of the three) |
| banking77 | +3.1 / +3.1 / +4.3% | +7.1% | narrowed to **≈ +3–4%** |
| code_fixtures | +3.2 / +3.2 / −4.3% | +17.6% | narrowed to **≈ ±3%** (parity band) |
| ag_news | +3.1 / +6.2 / +7.1% | +2.7% | ≈ +3–7% (unchanged-to-slightly-worse) |
| sst5 | +12.5 / +7.7 / +8.0% | +17.2% | ≈ +8–12% |
| emotion | +4.8 / +9.5 / +15.8% | — | ≈ +5–16% |
| xnli_en | +11.1 / +25.0 / +3.3% | +5.7% | noisy +3–25% |
| prompt_injections | +0.0 / −13.3 / −3.4% | — | parity-to-win |
| harness_visibility | +0.0 / +0.0 / +47.4% | −17.8% | unstable (r4's +47% is 28 vs 25 ms — 3 ms of quantized noise on a 25 ms cell) |
| harness_permissions | −24 / −24 / −36% | −34.2% | WIN (stable) |
| harness_tool_fit | −20 / −20 / −32% | −60.3% | WIN (stable) |
| harness_routing | −22 / −22 / −17% | −26.9% | WIN (stable) |
| harness_sensitivity | −28 / −20 / −31% | −17.8% | WIN (stable) |
| harness_cache_reuse | −28 / −30 / −36% | −35.7% | WIN (stable) |

**8 stable wins / 0 stable big losses** — the published table's +19…+38%
Class-A losses on the three typed lanes are GONE (typed/english and
multilingual now WIN by 20–30%; typed is unstable), and every former loss
narrowed to a +3…12% band. The issue's headline goal ("beat python on every
published cell") is **not yet met**: a stable band of small-sequence suites
remains +3–12% behind (banking77, massive_intent, ag_news, sst5, emotion,
xnli).

## The cold-first-suite signature (why r4's typed/english is discarded)

typed_decisions runs FIRST in the process, immediately after three
checkpoints load. Runs 3 and 4 (started after idle gaps) inflated the first
suite's rust cells (english 282/286 → 349) while runs separated from idle by
other GPU work read it fast — the M3 GPU clock-ramp lottery, the same
first-arm class the position-balancing rule exists for, now showing
BETWEEN-PROCESSES: a suite-first cell cannot be quoted from a single cold
process. massive_intent r3 (+36% while its neighbours read +4.4/+4.8) is the
same class mid-run. The across-run range is the honest quantity; the stable
cells (banking77/code/ag_news/sst5/emotion) agreed within ±5% in all three
runs regardless.

## The remaining band shares one mechanism hypothesis (the next lever)

Every stable loss is a SHORT-sequence suite (20–66 ms questions) — and every
one of them is a single-question-case suite, which T5 packing **excludes by
design** (`packed_eligible(n < 2)`, the Addendum-7 verdict that 1-q packed
LOST +4.5%). So these cells run the LOOP path: the full ~per-question fixed
cost (one-CB-per-case, the two real syncs, ~120+ dispatches at tiny m where
the tiny-op floor dominates). The lever class is FIXED-COST amortization at
small m — not GEMM efficiency (T7's territory, closed) and not packing
(T5's, gated by its own measured 1-q regression). Candidate rungs: a
re-pricing of packed-at-1q POST-T7 (the +4.5% 1-q regression was measured
BEFORE the band predicate landed — the band's narrow-at-small-m wins may
have flipped that verdict), and the per-case sync/dispatch count. Filed as
the issue's next lever; NOT scheduled here.
