# Bench 103 — bekko-system-one-v0 (17M/68M) vs the reflex modelless lane; the lane wired (`--bekko`)

**Status: 17M — MEASURED NEGATIVE on the owner's accuracy gate · 68M — mixed (reflex +6.1 pt overall, bekko wins 4/7 suites) · OWNER DECISION 2026-10-01: wire the lane into the harness + board anyway, and distill the architecture. The `--bekko` comparison lane LANDED this bench.**

Date: 2026-10-01 · host m3 (M3 Max, CPU) · reflex `5587bda` + this bench's lane commit · box load 5–11 (siblings active — latency ORDER-OF-MAGNITUDE only, never quotable absolute)

## Question

The owner pointed at hotchpotch/bekko-system-one (17M/68M/400M encoder "System One" decision
models — the same choice/noul/score vocabulary as `decision_wire`) and asked: **bench vs
reflex**; originally "dont bother if not accurate less than reflex", revised same-day after
the 68M read: **"result better than i thought so let's add that to the arena, some technics
may worth distill judge from it size is good enough"** — i.e. keep the lane, seat it on the
board, and mine the architecture.

## Method

Bekko answered the harness's OWN suite questions through the external-oracle lane protocol.
First run through the `GLINER_LANE_SCRIPT` env override (zero code changes — the table label
read "gliner" but the model column carried the bekko id); then the lane was wired properly as
**`--bekko`** (`run_jsonl_oracle_lane`, the DRY extraction both lanes now share; canonical
run `TABLES_68m_canonical.md`) and the numbers came back **byte-identical** across all three
invocation paths — a 3-process determinism witness per model.

Mapping (each lane applies its own rendering law — the CLM parity-note law): wire state passed
AS JSON (bekko's reference law renders JSON states itself); choice criteria in insertion order,
description = the description or the key (bekko's renderer requires a JSON-string description
— `json.loads` unconditionally); score native when every level parses numeric, else
choice-over-level-labels (disclosed); noul native with generic false/true meanings (the
instruction carries the question — the seat's fixed-default-phrases posture). Model pinned at
the card's release revisions (17M `2147c3d9…`, 68M `6eb1bae2…`), CPU FP32 (the author's
reference posture). ⚠ The model card assigns **no license yet** — measurement-only use.

## ⛔ The comparator-posture trap (the run's most reusable lesson)

The harness **DEFAULT** modelless posture runs count tables OFF — and reads roughly **HALF**
the published rows (ag_news 0.4050 vs published 0.8825; emotion 0.2750 vs 0.8850; banking77
0.3940 vs 0.8420). The published site board (`reflex.gist.rs/data/bench.json`, regenerated
2026-10-01T00:33Z at `25fd257`) and the serving posture arm the cal-selected count tables:
**`--nb-select --ridge-select`**. A first gate run against the default posture would have
compared bekko against a de-tuned reflex and "won" 6/7 suites meaninglessly. Every gate run
below passes the selects, and the modelless column **byte-reproduces the published board
exactly on all 7 suites** — the posture-verification law for any future external-oracle gate:
reproduce the comparator's published row first, then compare.

## Result — the family board (same questions, same harness, reflex at its published posture)

| suite | n | reflex | bekko-17m | bekko-68m | 68M winner |
|---|---|---|---|---|---|
| ag_news | 400 | 0.8825 | 0.9000 | **0.9000** | bekko +1.75 pt |
| emotion | 400 | **0.8850** | 0.3325 | 0.4825 | reflex +40.3 pt |
| sst5 | 600 | 0.3967 | 0.3333 | **0.4050** | bekko +0.83 pt |
| prompt_injections | 116 | **0.7672** | 0.4741 | 0.5000 | reflex +26.7 pt |
| xnli_en | 300 | 0.5233 | 0.4067 | **0.6767** | bekko +15.3 pt |
| massive_intent_en | 300 | 0.7800 | 0.7933 | **0.8667** | bekko +8.7 pt |
| banking77 | 500 | **0.8420** | 0.5940 | 0.7380 | reflex +10.4 pt |
| **total** | 2616 | **0.7058** | 0.5371 | 0.6446 | reflex +6.1 pt overall |

- **17M (3.9M active params):** reflex wins 5/7; bekko's two wins are marginal (+7 and +4
  questions). The owner's original accuracy gate FIRED for the 17M.
- **68M (42M active params):** reflex still leads overall (+6.1 pt = 1846 vs 1686 of 2616) on the strength of
  emotion/prompt/banking77, but **bekko-68m wins 4 of 7 suites** — including xnli +15.3 and
  massive +8.7. A real mixed record, and the owner judged it arena-worthy.
- Determinism: every bekko accuracy is byte-identical across 3 independent processes (two
  env-override runs + the canonical `--bekko` run). Latency: reflex p50 0.07–0.46 ms
  in-process vs bekko-68m p50 65–251 ms subprocess round-trip (IPC included — the laya-python
  measurement law); bekko-17m 7–50 ms.
- Calibration side-note: bekko's ECE(maxp) is dramatically better than reflex's raw readout
  on the suites it wins (0.027–0.097 vs 0.478–0.819) — a softmax over trained candidates is
  well-calibrated out of the box where the Lz4-drafter readout is not (reflex's calibrated
  readout-ECE column remains competitive, 0.046–0.112).

## Why the lane landed anyway (the owner's call)

The 17M-only negative was real but the 68M showed the family is competitive per-suite, the
size class is practical (42M active, ~244 MB fp32, browser-class), and the architecture
(shared-prefix K/V reuse for multi-candidate scoring) is directly relevant to our own encoder
lane's serve-refusal latency class. Board policy applies as-is: every seated suite's cell is
published, wins and losses alike, with the model column carrying the truth.

400M (343M active) is UNMEASURED on this box — CPU-infeasible at ~88× the 17M compute
(~2 h); it is the natural 4090-box follow-up if the 68M's per-suite wins justify it.

## Distill (the "technics worth mining" answer)

**The shared-prefix K/V reuse architecture** — the state+instruction prefix is encoded
bidirectionally ONCE per microbatch; each candidate branch attends to the cached prefix K/V
and its own tokens only; the prefix never attends candidates. At their published wide-load
point this is a 2.04× wall win and a 92.4% backbone token-op reduction (33,547 → 2,551; their
table, their box) — and AgentJev independently disclosed the same shape (their landscape
footnote: shared-prefix 298.91 ms vs unshared 609.65 ms). Grounded against our substrate:
riir-infer-laya is LISTWISE per question (all options in one `[MASK]`-marked sequence,
`build_sequence`) and already PACKS a case's questions into one batched forward
(`system_one_packed`) — but every question-sequence repeats the full state tokens, so the
state is re-encoded N times per case and packing amortizes dispatch, not encode. Bekko's
mask asymmetry (prefix-masks-from-candidates) is precisely what makes state K/V cacheable —
and it is a TRAINING-TIME property: the shipped laya checkpoints were trained bidirectionally,
so the mask change needs the next laya training round to adopt it (accuracy measured at
train time) before the serving win can be harvested. Filed as **riir-infer `.research/006`**
with the full mapping; the consumer is the encoder lane's serve-refusal latency class
(instinct Issue 014).

## Artifacts

- `TABLES_68m_canonical.md` + `results_68m_canonical.json` — the canonical `--bekko` run
  (68M, all 7 suites, `--nb-select --ridge-select`, byte-reproduces the published modelless
  rows).
- `TABLES_17m_canonical.md` + `results_17m_canonical.json` — the same for the 17M (the
  measured-negative cell; byte-identical to the original env-override gate runs — 4th
  determinism witness).
- `bekko_lane.py` — the lane instrument, byte-identical to the committed
  `scripts/bekko_lane.py` (the `--bekko` default).
- The 17M/68M numbers first measured through the `GLINER_LANE_SCRIPT` env override before
  the lane landed; every accuracy reproduced byte-identically across all invocation paths.

Repro (68M):

```sh
uv venv --python 3.12 .raw/bekko-env
uv pip install --python .raw/bekko-env/bin/python \
  torch==2.10.0 transformers==5.17.0 sentence-transformers==6.1.0 safetensors tqdm 'huggingface-hub>=1,<2'
BEKKO_MODEL=hotchpotch/bekko-system-one-v0-68m \
BEKKO_REVISION=6eb1bae2d35066b0d634fabaf8c79beafc6fd9f1 \
BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python \
target/release/harness --bekko --skip-laya --nb-select --ridge-select \
  --suites emotion,ag_news,sst5,massive_intent_en,banking77,prompt_injections,xnli_en \
  --datasets-dir .raw/datasets_t20k --out /tmp/bekko_rerun
```

## Honest caveats

- Rendering laws are per-lane by design: bekko saw JSON states (its native shape) while the
  seat/other lanes render prose; a bekko-specific prose rendering could shift its numbers,
  but its reference law is the JSON state — the native-shape choice is the generous one.
- Unknown which of our suites sit inside bekko's 153 training subsets (dataset-family overlap
  undisclosed). Symmetric risk: if they DO, bekko's wins are home-field-inflated; if they
  DON'T, its losses are out-of-domain — the board publishes the raw cells either way.
- Score-level fallback: wire score criteria that are not numeric rubrics become choice
  questions (the level label is picked without rubric semantics) — did not arise on these
  suites' numeric levels; typed_decisions (the mixed suite) refuses on this pool (stale
  700-row typed pull, the known pool staleness) so bekko went unmeasured there.
- English-only model on English-only suites — no language confound.
- 400M unmeasured (see above).
