# Bench 113 — the Clef lane's first cells: local `clef-flash-4bit` on the M3 (plan 011 Phase C, local posture)

**Status:** LANDED — the lane's first real cells (banking77 + typed_decisions, full-N); hosted posture still A6-blocked.

## What ran

The `--clef` comparison lane (plan 011 Phase A) against **`mlx-community/clef-flash-4bit`**
(Cloudflare/clef-flash, 9B, quantized 4-bit by the community MLX port) served **locally on this
M3** by `.raw/clef_srv/clef_lane_server.py` (gitignored operator rig over the repo-shipped
`clef_mlx.py`; `mlx 0.32.3 / mlx-lm 0.32.0 / mlx-vlm 0.7.4`, the port's tested pins). No Workers AI
credential involved — the open weights, run like every other external lane. The lane's wire
translation (our list shape → the vendor's systemone dict shape; the reply wrapped in the
Workers-AI envelope the strict lane maps, with the argmax `level` derived — systemone's `score`
field is the expected value) is the rig's `.raw/clef_srv/clef_lane_server.py`; wire contract +
rig specs: `.research/007`.

- Suites: `banking77` (500 cases / 500 q) + `typed_decisions` (400 cases / 2000 q), full-N,
  `CLEF_SMOKE_MAX_CASES=1000` — the documented ceiling raise, disclosed: **local serving has zero
  marginal spend** (the ceiling exists for undisclosed HOSTED pricing).
- Tables: `full_banking77/` + `full_typed/` (harness-written TABLES.md + results.json). Smoke
  runs at 50-case caps: `smoke_banking77/` (acc 0.9800) + `smoke_typed/` (acc 0.6000).
- Determinism: **det ✓ on both suites** — the lane's raw-reply byte-compare passed (the local
  server is byte-deterministic; the envelope deliberately carries no wall-clock field — a
  caught-before-run trap).
- Envelope model provenance: `clef-flash-4bit` read from `result.model` (never the CLEF_MODEL stamp).

## Rows (the lane's own tables carry the full metric tail)

| suite | n (cases/q) | acc | macro F1 | ECE(maxp) | p50 | det |
|---|---|---|---|---|---|---|
| banking77 | 500 / 500 | **0.9540** | 0.9536 | 0.0349 | 3901 ms | ✓ |
| typed_decisions | 400 / 2000 | **0.6955** | 0.6742 | 0.0215 | 2377 ms | ✓ |

Slice integrity (harness-stamped): banking77 test 500 `fnv1a64-cb0b5cbcb0cdfcce` · typed test 400
`fnv1a64-287d5f73a11932cd` — the registry slices; the C2 case-identity pin formalizes the
comparison against other lanes' rows on exactly these.

## Provenance

- Commits: banking77 run at HEAD `2290415`, typed run at HEAD `50d340d` (a sibling committed
  between the two starts; the harness BINARY was unchanged — built once before the smoke).
- Box state (the harness's own Issue-021 stamp, quoted verbatim): start load **11.03** → end
  **18.10**, swap 6.1→18.5 GB, AC, High Power — **⛔ latency NOT QUOTABLE**. `bench_preflight`
  REFUSED before the run (load 6.36 > 6.0). **Accuracy and the det probe are load-invariant**
  (deterministic MLX forwards); every latency figure in the tables is PROVISIONAL — rerun on a
  quiet box (preflight-clean) before quoting any ms figure anywhere.
- Model: `clef-flash-4bit` — a 4-BIT QUANTIZED 9B (community conversion). This is the **local
  quantized posture**, never pooled with hosted rows: the hosted 27B cell (plan 011 C4's deciding
  cell) stays owner-gated (A6 — the provided CF tokens lack Workers AI scope, probed 2026-10-03).

## Honest reading (C4 inputs, not the verdict)

Against the strongest published cells on the same suites (their postures differ — cross-posture
comparisons are indicative, not certified):

- **banking77: 0.9540 is the strongest cell measured on this board** — above the served hybrid H2
  0.8540 (nbsvm-v2 seat), GLiNER 0.706, laya-base 0.498. The 4-bit 9B beats our best served arm by
  ~10 points on this suite.
- **typed_decisions: 0.6955 sits between PAW-ft (0.5925) and the typed specialists** — agentjev
  0.7715 (det ✗), the Rethink record head 0.7550 (record-only), laya-typed 0.7445. The deciding
  C4 cell remains the **hosted 27B** (or a local 27B quant) — this 9B row is a floor, not the
  verdict.
- ECE at the raw readout: 0.0349 / 0.0215 — notably calibrated for a quantized model (Clef's
  calibration training shows through).

## What stays open

- Hosted C1 run (A6): the AI-scoped token; the A2.5 hosted wire fixture (this run's fixture is the
  LOCAL wire — the lane-expected envelope was verified against it, but the hosted shape claim is
  still unverified).
- C2: the case-identity pin + instinct-side lane-doc columns (this run's slice hashes are the
  pin's input).
- C3: reflex-site crosswalk publication; C4: the headline verdict after the hosted/27B cell.
- The remaining 13 suites (the 15-suite lane) as budget allows — local serving makes that a time
  question, not a money question.
- Latency re-read on a quiet box for every provisional ms figure above.
