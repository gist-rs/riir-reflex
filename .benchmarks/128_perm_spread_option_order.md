# Bench 128 — Option-permutation spread probe (issue 077): the modelless engine is NOT order-invariant by default, and the probe that proved it

**Status:** COMPLETE — T1–T4 measured; the control red found a real engine defect; the fix landed and re-verified; comparison lanes measured. Commit: see HISTORY.md row.

## What ran

Issue 077's lane-agnostic option-permutation spread probe (`--perm-probe`): every probed lane
answers the SAME choice questions under K deterministic option orderings (identity + 4
SplitMix64 shuffles, `slot_seed(case_id, qid)`; every permutation vector in the JSON) through
its OWN production decide path (the `ChoiceOracle` seam — the probe never re-renders a lane's
wire). Gate (issue text): per lane × suite, median top-pick swing ≤ 2 pt; max + flip rate
disclosed beside. Canary case (5 options, neutral strings) rides every lane in every run as
the liveness cell — an order-biased lane flips it BY DESIGN (the pure-module mock tests pin
both directions; drex's canary cell flips 27.5 pt on every suite below — the probe provably
fires on real lanes).

- K=5 orderings × ≤40 case(s)/suite (stride sampling, deterministic; laya runs at 20 to
  bound CPU wall time), all choice questions of each sampled case.
- **Control contract (revised by this bench):** the modelless control holds flips-only-on-
  exact-ties + median swing (untied slots) ≤ 0.01 pt — the measured L1-normalizer fp
  envelope — with STRICT byte-identity as a disclosed column, never the verdict: the L1
  sum (`scores.iter().sum()`, engine.rs) reorders with the presentation and moves ULPs by
  design. A control red refuses the run verdict (exit 1, record on disk).
- Sampling: the suite's choice-carrying cases in dataset order, first of every
  ceil(n/max) stride; ALL choice questions per sampled case.

## The finding (run 1, pre-fix — `.benchmarks/128_perm_spread/modelless/`)

The issue's premise — "the modelless engine is permutation-invariant by construction" — is
**FALSE** on the default posture. Measured (sha 1b8873e base + working tree; host
4090-windows; AC/high/load 0.72; the modelless cells are GPU-free):

| suite | median pt | max pt | flips | rate |
|---|---|---|---|---|
| typed_decisions | 4.70 | 18.81 | 41/60 | 0.683 |
| ag_news | 0.73 | 4.18 | 38/40 | 0.950 |
| emotion | 0.10 | 1.36 | 14/40 | 0.350 |
| xnli_en | 0.52 | 4.45 | 14/38 | 0.368 |
| massive_intent_en | 0.00 | 0.00 | 0/38 | 0.000 |
| banking77 | 0.00 | 0.00 | 0/39 | 0.000 |

**Root cause 1 (the real movement): the legacy `k == N` index alignment in
`DecisionEngine::solve_sample_into`.** When the by-name option→domain resolution fails and
the option count equals the domain count, `opt_dom[i] = i` binds the option AT POSITION i to
domain i's route term — permuting the options permutes the binding, so the ranking is
partially POSITION-driven. ag_news/emotion/xnli_en named their domains by the train docs'
integer class labels ("0".."N-1") while their option keys are words ("world"/"joy"/
"entailment"...) → by-name always failed → k == N always armed. typed_decisions: 4-option
questions against 4 workflow domains — same shape.

**Root cause 2 (the ULP class, visible on the invariant suites): the L1 normalizer**
(`scores.iter().sum()`) sums in PRESENTATION order; f32 addition is not associative, so
permuting options moves every probability by ULPs — banking77/massive read control-invariant
FALSE with 0.00 pt display swing and zero flips. Bounded by the measured envelope (< 0.005 pt
display grain); NOT fixed (a canonical sum would move the canonical-order bytes that
published tables and fixtures pin) — disclosed as a bounded artifact, candidate for a future
owner-gated change.

## The fix (landed in the same change as the probe)

**Content binding for the fixed-criteria classification suites** — canonical-order outputs
byte-identical, permuted-order outputs corrected:

- `train_row_label` maps the int class label through the suite's OPTION-KEY consts
  (`AG_NEWS_KEYS`/`EMOTION_KEYS`/`XNLI_KEYS`/`WISESIGHT_KEYS`, hoisted module-level and now
  shared by the builders AND the label rule) — one spelling for corpora, stratification,
  the slice audit, and the engine domains.
- `prepare()` labels := the option-key union for ag_news/emotion/sst5/xnli_en(_val)/
  thai_wisesight; the fetch guard is now the EXPECTED KEY SET (exact membership pin — a
  swapped key would silently rebind every corpus).
- sst5 needs no behavioral change (its option keys ARE "0".."4"); prompt_injections
  (noul-only) keeps the integer labels (the noul exemption).
- By-name resolution then arms for these suites under ANY option order: the option's own
  name picks its domain's route term. The legacy `k == N` path stays for typed_decisions
  (its binding is load-bearing for the published typed numbers — see the follow-up issue).

**Post-fix control (run 2, `.benchmarks/128_perm_spread/modelless_v2/`): ag_news, emotion,
xnli_en PASS at 0.00 pt median / 0 flips** (was 0.95/0.35/0.37 flip rates); massive/banking
PASS unchanged. **typed_decisions stays RED** (median 4.70 pt, 68.3% flips — unchanged, as
predicted: the legacy binding is its deployed behavior).

## Comparison lanes (`.benchmarks/128_perm_spread/lanes/`, `typed/`, `laya_cpu/`, `laya_typed_cpu/`)

Provenance: 4090-windows, AC power, High-performance scheme, load 0.72, swap 2914 MB,
GPU disclosed (lane servers legitimately busy: agentjev on cuda:0, drex CPU-hosted torch;
`scripts/bench_preflight.ps1` PASSED). drex = their serve.py @ `.raw/drex-model`
(safetensors snapshot, CPU forward); agentjev = AgentJev-0.6B @ checkpoint sha 68883998,
cuda:0; laya = the riir lane, CPU posture, **G5 parity GREEN pre-run** (2/2 checkpoints,
41.8 s — the publish law held). Latency columns deliberately absent (a
distribution-stability probe; the det/latency axes live in benches 126/127).

| suite | lane | n | median pt | max pt | flips | verdict |
|---|---|---|---|---|---|---|
| ag_news | modelless (control) | 40 | 0.00 | 0.00 | 0 | PASS |
| ag_news | drex | 40 | 0.38 | 13.25 | 1 | PASS |
| ag_news | agentjev | 40 | 0.00 | 0.00 | 0 | PASS |
| emotion | drex | 40 | **14.92** | 44.39 | 9 | **RED** |
| emotion | agentjev | 40 | 0.00 | 0.00 | 1 | PASS |
| xnli_en | drex | 38 | **4.41** | 19.10 | 5 | **RED** |
| xnli_en | agentjev | 38 | 0.00 | 0.00 | 0 | PASS |
| massive_intent_en | drex | 38 | **5.11** | 76.29 | 6 | **RED** |
| massive_intent_en | agentjev | 38 | 0.27 | 0.86 | 2 | PASS |
| banking77 | drex | 39 | **8.65** | 52.23 | 7 | **RED** |
| banking77 | agentjev | 39 | 0.24 | 0.77 | 2 | PASS |
| typed_decisions | modelless (control) | 60 | 4.70 | 18.81 | 41 | **RED** (the finding above) |
| typed_decisions | drex | 60 | **11.93** | 40.37 | 16 | **RED** |
| typed_decisions | agentjev | 60 | 0.00 | 0.00 | 0 | PASS |
| ag_news | laya/english | 20 | 0.68 | 16.84 | 0 | PASS |
| emotion | laya/english | 20 | 1.53 | 44.50 | 1 | PASS |
| xnli_en | laya/english | 20 | 0.39 | 39.26 | 0 | PASS |
| massive_intent_en | laya/english | 20 | 0.01 | **99.97** | 6 (30%) | PASS (median gate) |
| typed_decisions | laya/typed | 30 | **4.12** | 9.23 | 4 | **RED** |

Canary cells: drex flips it on EVERY suite (27.5 pt — order-biased BY the probe's own
liveness proof); agentjev never; laya flips it on every suite at 4.2 pt (the position
surface is live).

## Readings

1. **Drex DLM is strongly order-biased** — 4 of 6 suite cells RED with medians 4.4–14.9 pt
   and max swings to 76 pt, worst on the typed criteria set (11.93 pt median, 26.7% flips),
   and the canary flips everywhere. This is exactly the failure class LiquidAI's d1 recipe
   names "shuffling answer options" among the levers that mattered — independently measured
   on a third-party decision model, first-party instrument.
2. **AgentJev is content-bound** — 0.00–0.27 pt medians across all six suites (1–2 flips per
   suite, the recorded wobble class, disclosed; Bench 127's det ✓ post-strip is consistent).
3. **Our modelless engine, post-fix, holds the strictest cell on the board** — flips only on
   exact ties, median inside the fp envelope, on five of six suites.
4. **Laya: median-holds, tail-fragile** — the pooled-state cross-marker attention passes the
   median gate everywhere except typed, but carries max swings to 99.97 pt and a 30% flip
   rate on massive. This is riir-train Research 471 item 4's measured evidence: option-shuffle
   AUGMENTATION is worth its LENC-cache regeneration cost for the laya family specifically
   (the d1 lever applies to our own encoder lane, not just to d1).
5. **typed_decisions is order-fragile in BOTH our lanes** (modelless 4.70 pt via the legacy
   k==N binding — follow-up issue filed; laya/typed 4.12 pt) — the decision suite where
   option order matters most is the one our board's hardest suite lives on.

## Reproduce

```bash
scripts/bench_preflight.ps1
cargo run --release --bin harness -- --perm-probe \
  --out .benchmarks/128_perm_spread/<posture>          # modelless control (default suites)
cargo run --release --bin harness -- --perm-probe --drex --agentjev \
  --suites ag_news,emotion,xnli_en,banking77,massive_intent_en \
  --out .benchmarks/128_perm_spread/lanes              # + comparison lanes (servers up)
cargo test --release --features laya-riir --test laya_riir_parity   # G5 BEFORE any laya cell
cargo run --release --features laya-riir --bin harness -- --perm-probe \
  --perm-max-cases 20 --suites <suites> --out .benchmarks/128_perm_spread/laya_cpu
```

Every permutation vector, per-ordering probability vector, and the canary cells are in each
run's `perm_probe.json`.
