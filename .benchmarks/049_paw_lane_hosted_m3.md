# Bench 049 — PAW (ProgramAsWeights) comparison lane, first cells: hosted anonymous posture, 4-suite subset (Issue 033)

**Status:** RECORD — measured 2026-09-26 on the M3 Max; one run, artifacts in
`049_paw_lane_hosted_m3/` (`TABLES.md`, `results.json`, `paw_programs.json` = the
program-id cache that produced the cells).

## Provenance

- **Box:** Apple M3 Max, macOS, AC, powermode 2 (High Power). Preflight before the run:
  `PROVENANCE: power=AC Power load=23.36 swap=1451.62M canary=skipped powermode=2(high)`
  → **REFUSED** (sibling sessions on the box). Harness box stamp: start load 18.71 → end
  38.11 — ⛔ **latency NOT QUOTABLE**. This record claims **accuracy only**; the latency
  columns are observation (and for PAW they are dominated by the network anyway).
- **Code:** reflex `20c949f` (the Issue 033 lane commit, pre-rebase sha — same lane tree as
  the rebased commit on `develop`), `--release`, default features, `--skip-laya`;
  katgpt-rs `7512a0d99`. Command:
  `PAW_PROGRAM_CACHE=<repo>/.raw/paw/programs.json harness --suites banking77,ag_news,emotion,sst5 --skip-laya --paw`.
- **PAW side:** hosted API `https://programasweights.com` (health: all compilers healthy,
  queue 0), **anonymous posture — no `PAW_API_KEY`** (their free tier; anonymous programs
  must be public, so these four programs are listed on their hub). Compiler = server
  default, snapshot **`paw-4b-qwen3-0.6b-20260407`** (mapper_lora, Qwen3-0.6B Q6_K
  runtime `qwen3-0.6b-q6_k`). Inference `temperature 0.0`, `max_tokens 48`. SDK contract
  read at `programasweights-python` @ `74919f69`.
- **Specs** (`scripts/paw_specs/`, zero-shot: task description + the option set verbatim,
  one per line; BLAKE3):

  | suite | spec BLAKE3 | program | compile (client wall) |
  |---|---|---|---|
  | ag_news | `c5d5b475…1d543063` | `06f283857d30b77a18fc` | 4.6 s |
  | emotion | `ab8a01c6…a794baf4` | `25f883f21290e923a627` | 4.1 s |
  | sst5 | `03ec0079…aa643493fa71f` | `0e0968ebe6a700f0f4ad` | 0.8 s (server-cached: same spec as the feasibility probe) |
  | banking77 | `387c823c…48ff2d43` | `f2c6df69e759026ade01` | 4.3 s |

  Compile is seconds, not minutes, for the default mapper compiler (a pseudo-program is
  generated, then a LoRA is mapped — no finetune). The `paw-ft-bs48` finetune compiler
  ("~2–5 min, much higher accuracy") is NOT measured here — see "What remains".

## The mapping law (declared before the first measured cell)

Trim → strip ONE matched surrounding quote pair (`"…"`/`'…'`/`` `…` ``) → trim → exact,
case-sensitive match against the option KEYS, else a unique exact match against the option
DESCRIPTIONS; anything else is a **REFUSAL**, counted, scored wrong, never guessed. The
quote strip was added after a 2-row feasibility probe on **train** rows showed the compiled
pseudo-programs emit `"neutral"` with literal quotes — it is counted per suite
(`quote-stripped`) so the normalization is visible. No confidence/ECE columns: PAW returns
free text, not a distribution (disclosed divergence).

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| suite (n) | PAW acc | refusals | answered-acc | quote-stripped | modelless (same run, heads OFF) | reference rows (other runs) |
|---|---|---|---|---|---|---|
| ag_news (400) | **0.7825** | 0 (0.0%) | 0.7825 | 0 | 0.5100 | laya english 0.95 (047) · gliner 0.7025 (037) |
| emotion (400) | **0.4750** | 2 (0.5%) | 0.4774 | 393 | 0.2825 | laya english 0.5925 (047) · gliner 0.565 (037) |
| sst5 (600) | **0.3283** | 0 (0.0%) | 0.3283 | 581 | 0.2167 | laya english 0.3717 (047) · gliner 0.4383 (037) |
| banking77 (500) | **0.1400** | **365 (73.0%)** | 0.5185 | 0 | 0.4460 | modelless heads-selected 0.684 · laya english 0.498 (047) · gliner 0.706 (037) |

sst5 score MAE (answered) 0.845, within-1 0.852. Server-reported inference p50 95–113 ms
(their `latency_ms`); client round-trip p50 ~1.0 s (network from this box + a fresh TLS
handshake per `curl` spawn) — observation only.

**Determinism (observed-repeat, first 10 questions):** ✓ ag_news / emotion / banking77,
**✗ sst5** — hosted inference repeated a question with a different output. Their docs
promise determinism for the LOCAL runtime, not the hosted one; this row is the measured
answer for hosted.

## Findings

1. **The refusal column IS the banking77 finding.** At 77 ways the program answers in its
   own vocabulary — `"card status"`, `"track new card"`, `"new card not received"`,
   `"still waiting on that card"` for the gold `card arrival` family — so 73% of answers
   are not an option at all. Of the 27% that did parse, 51.9% were right. The typed-wire
   moat (reflex cannot emit a non-option by construction) measured, not asserted.
2. **Small label sets parse cleanly** (0–0.5% refusals). There PAW's zero-shot program
   beats the modelless lane at the heads-OFF posture (ag_news +0.27, emotion +0.19, sst5
   +0.11) and sits below laya english on all three; vs gliner it wins ag_news (+0.08) and
   loses emotion (−0.09) and sst5 (−0.11).
3. **The quote habit is systematic** (393/400 emotion, 581/600 sst5 answers quoted; 0 on
   ag_news/banking77) — a strict strip-only integrator would score emotion/sst5 at ~0.
   The law's single quote strip is what makes those rows comparable, and it is disclosed.
4. **Spec confound:** these are zero-shot specs (no worked examples). PAW's own guidance is
   to iterate specs against test cases; tuning against OUR test split would be overfitting,
   so the specs were authored once and frozen. A train-split example-tuned spec is a
   legitimate second cell (below).

## Posture B (local runtime) — keyless, measured feasibility

`uv`-installed `programasweights==0.4.10` (+ `llama-cpp-python`) into a throwaway venv,
**no key**: `paw.function("0e0968ebe6a700f0f4ad")` downloaded the program and the 594 MB
Qwen3-0.6B Q6_K base (1002 s on this link) and ran locally. On the first 5 sst5 test rows:
5/5 byte-identical on repeat (local greedy is deterministic), and 5/5 **decision-level**
agreement with hosted — one surface difference (local `neutral` vs hosted `"neutral"`),
which the quote-strip law maps to the same label. Posture B is therefore a viable
determinism-comparable cell with no key; it is NOT wired as a lane here (a Python
subprocess oracle, the gliner precedent) — open task in Issue 033.

## What remains (Issue 033)

- `paw-ft-bs48` finetune-compiler cells (async compile path is implemented; anonymous
  tier allows 20 compiles/h, 1 concurrent — no key required).
- Posture B lane wiring (local, deterministic) + a full-N local run.
- Arena republish via `../reflex-site/scripts/publish_bench.py`.
- A quotable-box re-run if PAW latency is ever to be cited (it should be cited as
  server `latency_ms` beside client round-trip, never as engine latency).
