# Bench 155 — StartLux-Decision v1.1 board pass (`--startlux`, reflex Issue 095 / riir-train 635 T3)

**Status:** RECORD — one frozen read, 2026-10-21T16:41Z, harness `3a9e8f5` (the lane's landing commit), host `shikuwa` (4090, Windows).

## Posture (the lossy-surface law — pinned, quoted beside every number)

- Model: `startlux-models/StartLux-Decision-v1.1-27B-Instant` — **Q4_K_M GGUF** (the 635-T2 arm), llama.cpp **b11549 exact-tag build** (`aa94f2086147`, MSVC), **`-ngl 99`** full offload, `-c 8192 --parallel 1 -b 8192 -ub 8192 --threads 12`, llama-server on :8081 (18.85 GB VRAM resident).
- Engine: THEIR Apache-2.0 `startlux-decide` v1.1.0, `STARTLUX_LLAMACPP_BIDIR=server`, :8090 — health verified **kind instant · prompt_attention bidirectional · engine.bidir_mode server** (the lane's posture gate; a causal/off serve is a different model than the one trained).
- License: weights **CC BY-NC 4.0** → DECLARED-NC RESTRICTED (635 T0: measurement + capture allowed, adoption fail-closed). NC-lineage rides every artifact.
- Scoring: gold-label, same cases as every lane, TRAIN-pool corpora law untouched; the harness's frozen stratified splits (slice-integrity OK ×9).
- Latency **NOT QUOTABLE** (box-state law): run start load 13.68 > 6 (the sibling riir-refine batch job was building on the box); accuracy unaffected — the GPU was exclusively ours (298 MiB baseline → the stack; no other compute consumer). p50 values in the tables are disclosed-but-unquotable.

## The board (startlux column vs the modelless control, n = questions)

| suite | n | startlux acc | startlux ECE | modelless acc | det |
|---|---|---|---|---|---|
| typed_decisions | 2000 | **0.7945** | 0.1581 | 0.2940 | ✓ ×10 |
| ag_news | 400 | 0.9175 | 0.0385 | 0.4050 | ✓ |
| emotion | 400 | 0.7975 | 0.0416 | 0.2750 | ✓ |
| sst5 | 600 | **0.6150** | 0.0556 | 0.2017 | ✓ |
| prompt_injections | 116 | **0.9224** | 0.0524 | 0.4828 | ✓ |
| xnli_en | 300 | **0.9367** | 0.0320 | 0.3400 | ✓ |
| massive_intent_en | 300 | 0.9133 | 0.0260 | 0.6100 | ✓ |
| banking77 | 500 | 0.9140 | 0.0359 | 0.3940 | ✓ |
| semantic_defects | 102 | 0.9510 | 0.0387 | 0.1863 | ✓ |

Determinism: verbatim observed-repeat, first 10 cases per suite, ALL 9 ✓ (their body carries `timing_ms` only with `return_details`, never set — the no-clock law; the det column measures compute determinism).

## Paired read vs per-suite incumbents (the T3 deliverable)

Incumbents from 635 T3's spec (instinct board rows / lane reads of record):

| suite | startlux | incumbent | delta | verdict |
|---|---|---|---|---|
| typed_decisions | 0.7945 | agentjev 0.7720 | **+2.25 pt** | **BEATS** |
| sst5 | 0.6150 | clef 0.6033 | **+1.17 pt** | **BEATS** |
| prompt_injections | 0.9224 | A1 0.8534 | **+6.90 pt** | **BEATS** |
| xnli_en | 0.9367 | openthai 0.9000 | **+3.67 pt** | **BEATS** |
| massive_intent_en | 0.9133 | clef 0.9333 | −2.00 pt | loses (beats the served H2 0.8400) |
| ag_news | 0.9175 | laya 0.9500 | −3.25 pt | loses (beats the served H2 0.8975) |
| emotion | 0.7975 | A0 0.8850 | −8.75 pt | loses (the board's highest bar) |
| banking77 | 0.9140 | clef 0.9540 | −4.00 pt | loses |

**4 of 8 incumbent suites beaten** → the T5 distill question is EARNED on typed_decisions, sst5, prompt_injections, xnli_en (the retrain itself stays owner-gated per 635 T5; the captures do not — T0). semantic_defects is reflex-specific (no external incumbent; the modelless floor is 0.1863 there).

Board #1 confirmation: the 72.88 public_skill is a selection-on-public number (635's warning) — on OUR suites the column is strong but NOT dominant: it holds 4 incumbent bars, loses 4. The B5 law's both-directions cut, measured.

## Findings

1. **typed_decisions overconfidence persists at the serve posture** (ECE 0.1581 vs its acc 0.7945 — the T2b probe's 0.369 was on the 60-row TRAIN probe slice, different population; the direction replicates: the fine-tuned-confidence shape concentrates on the row-shaped multi-criteria suite). The captured-but-suspect flag (T2b) rides the capture manifest; the T5 acceptance gates stay dual-shuffle-robust.
2. **banking77 knockout cost**: 77-way choice rows run their engine's knockout rounds — p50 1018 ms vs ~210-280 ms single-pass rows (load-adjacent, unquotable; the RATIO within one suite is the honest signal ≈ 4-5× per-row cost). Full-pool capture wall-clock is knockout-dominated there.
3. **The lane's posture gate works** — it refused nothing here because the boot was the documented one; the stub tests pin every refusal arm (wrong kind / causal bidir / keys-only public shape / foreign status).
4. The `none_probability` field rode every choice answer (serve default `none_check` on) and is parser-ignored on the comparison path; the capture keeps it reachable via `decide_raw` (the T4 router-column arm reads it from the raw bodies or a follow-up parse arm).

## Artifacts

- `results.json` + `TABLES.md`: `.benchmarks/155_startlux_board/` (this dir).
- Teacher capture (RIDT, train rows only, license token in the header):
  **COMPLETE** — `.raw/startlux_teacher/`, 6/6 T3 suites sealed + `.blake3`
  sidecars + the binding `MANIFEST.md` (NC-lineage + captured-but-suspect flag
  + per-family train-vs-test gaps: massive +3.8 pt, banking77 +0.7, ag_news
  −1.1, xnli −2.9, emotion −2.6, sst5 −1.9 — train inflation concentrates on
  massive; the T2b deferred degeneracy comparator reads these bins when the
  synthesis pipeline window opens). Incident disclosed in the manifest: the
  backend died silently at ~4h11m/~217k read-outs mid-capture (emotion lost
  mid-suite, re-sealed in window 2; per-suite atomic seals kept every bin
  complete or absent, never partial). Stack torn down at window close (VRAM
  331 MiB baseline).
- Stack boot line (the documented one, ~3.5 min cold reload): llama-server b11549 `-ngl 99 ... --port 8081` + `STARTLUX_LLAMACPP_BIDIR=server startlux-decide serve --model models/startlux-decision-v1.1-27b-instant --backend llamacpp --port 8090`. Torn down at window end (the 10-19 resident-stack lesson).
