# Bench 046 — Issue 020's paired per-suite A/B, first publication-grade run: the stable band FLIPPED to wins after the fold promotion; the bar's remaining cells are the typed_decisions trio

**Status:** LANDED 2026-09-26 · paired per-suite instrument (`paired_suite_ab.sh`,
position-balanced across suites: even suites rust-lane-first in one run, odd
suites py-then-rust reversed), preflight PASSED and quoted. This is the run
the fold-epilogue promotion (riir-infer `53334f9`, default-ON) was priced
with — taken AFTER the promotion, so the rust lane is the shipped posture.

**PROVENANCE:** `power=AC Power load=1.76 swap=1054.44M canary=120.7us/best5
powermode=2(high)` (preflight.txt; start load 1.76, the harness full-lane run
had just ended). The summary's own single-sample caveat applies — banking77
was re-run as a second paired sample (`rerun_banking77/`): **byte-identical
wins** (55/58 p50, 66/76 p99, −5.2%/−13.2% both runs, acc 0.498/0.498), and
the run reproduced on a load-3.12 box.

## The verdict (SUMMARY.txt, 9 lane cells, 7 suites)

| suite | ckpt | rust p50 | py p50 | Δp50 | Δp99 | verdict |
|---|---|---|---|---|---|---|
| ag_news | english | 24.0 | 25.0 | **−4.0%** | −21.4% | p50 WIN p99 WIN |
| banking77 | english | 55.0 | 58.0 | **−5.2%** | −13.2% | p50 WIN p99 WIN |
| emotion | english | 14.0 | 17.0 | **−17.6%** | −50.0% | p50 WIN p99 WIN |
| massive_intent_en | english | 37.0 | 36.0 | +2.8% | −14.8% | p50 LOSS (within-tol, sup 4) p99 WIN |
| sst5 | english | 18.0 | 20.0 | **−10.0%** | −41.5% | p50 WIN p99 WIN |
| typed_decisions | english | 313.0 | 256.0 | +22.3% | +21.3% | p50 LOSS p99 LOSS |
| typed_decisions | multilingual | 130.0 | 122.0 | +6.6% | −4.1% | p50 LOSS p99 WIN |
| typed_decisions | typed | 276.0 | 261.0 | +5.7% | −6.5% | p50 LOSS p99 WIN |
| xnli_en | english | 19.0 | 23.0 | **−17.4%** | −45.7% | p50 WIN p99 WIN |

cells 9: p50 wins 5 · losses 4 — **p99 wins 8 · losses 1** · ties 0. Every
cell's accuracy pair matched (rust/py identical to 3 decimals) — lane
integrity green.

## What this closes and what it opens

- **The stable band (Bench 035's +3–12% losers) is GONE**: ag_news,
  banking77, emotion, sst5, xnli_en all read p50 WINS, most by double
  digits. The fold promotion's −1.5…−5.9% encoder medians (seq 10–128) +
  T10's softmax rungs + split-K compounded into the suite cells.
- **The every-cell bar's remaining deficit is ONE suite**: typed_decisions
  (english +22.3% p50 AND +21.3% p99 — the only double loss; multilingual
  +6.6%, typed +5.7% p50) plus massive_intent_en's +2.8% (within-tol, tail
  support 4 — the summary's own "rerun before acting" band; the banking77
  rerun suggests rerunning it before calling it).
- typed·english stays the least-stable cell in the workspace record (Bench
  036) AND the widest loss here — its next lever is still the named one
  (T5 packed-at-1q re-price post-T7 / the v2 packed head), not yet scheduled.

## Instrument repairs landed with this record (both found BY this run)

- `paired_suite_ab.sh` mkdir'd `OUT_DIR` only — the first per-suite log
  redirect died on the missing suite dir (rc=1 before any measurement).
  All three per-suite dirs are pre-created now.
- `provenance.txt` carried the `(no preflight … NOT FOR PUBLICATION)`
  default EVEN WHEN the preflight passed — the variable was set before the
  if and never updated, so the summary rendered its ⛔ marker beside a
  PASSED preflight (marker and evidence disagreeing). The PASSED posture
  now writes the real PROVENANCE line.

## Issue 024 T5 (the leak axis rides this run too)

The binary the instrument found already carried `slice_leak` (built for
Bench 045's run; the script only builds when missing), so these A/B runs
compute leak stats too — harmless to the lanes (a per-case dataset
property). The leak COLUMNS for the M3 host are recorded from **Bench
045's** run (`.benchmarks/045_full_m3/`, the same binary, leak blocks on
the 7 in-scope suites); this record is the paired A/B verdict.
