# Bench 057 — Issue 038 T7 levers: the option-conditioned tables (typed) + the NBSVM ridge (emotion)

**Date:** 2026-09-27 · **Host:** M3 Max (macOS 26.6.2, AC, 16 cores) · **Protocol:** Bench-052
(`--datasets-dir .raw/datasets_t20k`, the t20k byte-verified pull; selections on the
stratified slice, test read once) · **Features:** default (`modelless, nb_scope,
option_cond, nb_ridge`) + `laya-riir` flag-off for the lane (skip-laya)

## What landed (Plan 004)

| lever | suite | posture (cal-selected) | published → new | p50 |
|---|---|---|---|---|
| T7(b) `option_cond` — per-(qid, option) contrastive tables from TRAIN gold events | typed_decisions | oc_scale 2 (slice 0.370 → 0.558, +18.8) | 0.3300 → **0.4655** (+13.5 pt) | 0.578 ms |
| T7(a) `nb_ridge` — NBSVM closed-form ridge, k=2048, λ=10, per-class ratios | emotion | ridge_scale 2 (slice 0.550 → 0.690, +14.0) | 0.7375 → **0.8475** (+11.0 pt) | 0.129 ms |

G1: both rows pass — the sigmoid-gate calibrator absorbs the saturated confidence
(typed: calibrated ECE 0.0750 vs floor 0.345's predecessor run; emotion: calibrated
ECE 0.0 vs conformal floor 0.204). G2: p99 ≤ 50 µs in-process (decision_set_goat),
all armed postures. G3: **every other suite byte-identical to the site rows** (12
suites × Δacc 0.0000, full-workspace run `/tmp/t7_full_final`). G4: `solve_into`
0 allocations with BOTH levers armed, canary live. Determinism: `det=true` on
every suite.

Ladders that DECLINED (the honest arm of the selection protocol): ridge on
typed (slice −2.2..−7.8), ag_news, sst5, prompt_injections, xnli, massive,
banking77 (the 60/77-class suites pay the O(k³)·C fit for the ladder and still
decline — recorded, ~25-60 s each); oc everywhere but typed (no per-question
gold events — loud DECLINE line).

## Premise probes (before any Rust)

`scripts/issue038_t7_probe.py` + `issue038_t7_probe_k.py` + the differential
`scripts/issue038_ridge_diff.py`:

| suite | best T7a test | published | verdict |
|---|---|---|---|
| emotion | **0.8925** (k=2048, per_class, λ=10) | 0.7375 | the lever |
| prompt_injections | 0.7672 | 0.7672 | (site already there via nb polarity) |
| banking77 | 0.8384 | 0.8260 | +1.2 pt — under the 5 pt bar |
| ag_news / massive | ≈ flat | — | not a lever |
| sst5 / xnli | 0.3583 / 0.4367 | 0.3967 / 0.5233 | WORSE — shipped views win |

typed (T7b): per-(qid,option) MNB 0.4475..0.4585 vs published 0.32; variant forms
(noprior/contrastive/IDF/char-grams) read the same or worse.

## Measured lessons this bench paid for

1. **Unsorted binary search silently misses** — the Rust feature vec was in
   df-order; `binary_search` needs bucket-ascending order. Signature: `hits=1/33`
   in the differential dump and a 0.29 pure-ridge read (numpy replica of the same
   math: 0.8925). The probe's Python dict lookup cannot have this bug class.
2. **The probe's `tot[lab]` was a Python missing-key read (0), not a total** —
   the validated ratio arithmetic has NO per-class totals; mirroring the
   *intended* formula breaks the denominator (loud non-finite refusal fired).
   Mirrored verbatim, quirk recorded at the site.
3. **`/n_tokens` is the NB convention, not the ridge's** — damped ridge margins
   are O(0.1) and do not grow with the token count; σ(margin/n_tok) ≡ 0.5 (a
   constant shift, zero picks moved — measured 0.5500 at every scale). Replaced
   with a fit-time SELF-CALIBRATED temperature (mean per-doc class spread over a
   strided 512-doc sample; the threshold-fitting law, applied to a term).
4. **Fit cost is a non-issue at the shipped k**: the full 6-class emotion fit
   (tokenize + gram + 6× Cholesky at k=2049) is **0.9 s** release on the M3 —
   the earlier "hang" was the assert-abort loop, not compute. Wide suites pay
   it ×C in the ladder only (banking77 ≈ +40-60 s per selection run).

## Fit-cost disclosure (the G2 fit-budget note)

typed oc fit: O(events) ≈ ms-class. emotion ridge fit: 0.9 s once per build;
the selection ladder builds it 4×; a full lane run with `--ridge-select` on
emotion reads ~10 s end-to-end (was 0.7 s). `ridge_scale = 0` (the default
serving posture) pays NOTHING — the fit is skipped entirely.

## Artifacts

- `/tmp/t7_full_final/results.json` + this file's copy of the moved rows
- `examples/nb_ridge_probe.rs` — the differential probe (kept; measurement-only)
- Site republish: the typed + emotion rows move; every other row identical
