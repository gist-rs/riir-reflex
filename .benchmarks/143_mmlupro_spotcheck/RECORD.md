# Bench 143 — the d3 NF4 mmlupro spot check (riir-train 629 T2, the 4th family)

> **Status:** RECORD — PASS (2026-10-14 08:36–08:56 +07, 4090 lane, INTERLEAVED with the running leg-A capture) · host 4090-windows (shikuwa) · lane: riir-train Issue 629 T2 · completes the bench-142 family gate under its CORRECTED board row
> **Rule:** the same teacher-quality gate as Bench 142 (the lossy-surface law): the quantized teacher's deployed-path behavior must hold per family vs the board's own row for the same entrant — d3-v310 bench-57 (MMLU-Pro) accuracy **0.7304**, full 12,032-case population, extracted from the Bench-067 pinned snapshot (the CORRECT row — see Bench 142's correction section; the reference JSON is `riir-train/scripts/plan629_d3_spotcheck_reference.json`).

## The read

| suite (population) | d3 NF4 ours | d3-v310 board row | Δ | ece(maxp) | p50 |
|---|---|---|---|---|---|
| mmlupro_jdi (2000 of 10,300-row local pop, hash-ordered) | **0.7270** acc | **0.7304** acc (full 12,032 pop) | **−0.34 pp** | 0.0297 | 565 ms ⚠ |

- **Verdict: PASS** — the 4-family gate completes at −0.34..−0.87 pp vs the board's own rows (winogrande −0.87, hellaswag −0.61, anli −0.47 acc-vs-macroF1, mmlupro −0.34); every family inside subsample noise, no collapse, no family beyond 1 pp. The NF4 capture posture holds the teacher's deployed-path behavior.
- The modelless A0 context row reads 0.1080 (chance ≈ 0.1 at 4–10 options) — sanity holds.

## Disclosures (load-bearing)

1. **The population and the pin (CORRECTED post-read).** The run read 2,000 real
   mmlupro questions; at run time the local fetch held 10,300/12,032 test rows
   (topped to full 12,032/12,032 the same morning). The full-population re-read
   (15:45, §below) reproduced `cases_digest 6a2d24628ce7b916` and acc 0.7270
   BYTE-IDENTICALLY — the sample is **stable across the top-up**; an earlier
   draft here called the pin "partial-era" (retracted — see the manifest Gaps
   entry). The DEEPER finding: the sample is provably insensitive to tail-page
   content AND matches neither the documented hash-order law nor first-N
   under independent replication — **filed as reflex `.issues/088`** (the
   sampling law needs instrumentation; the numbers here are real measurements
   of the served questions).
2. **INTERLEAVED with the leg-A capture** (both clients serialized on the one serve, 08:36–08:56): the p50 565 ms is contention-contaminated — accuracy unaffected (deterministic per-request scoring), latency NOT quotable.
3. The −Δ direction is consistent with Bench 142's corrected table (small systematic −Δ, the T4 comparability caveat).

## Provenance

```
host: 4090-windows (shikuwa) · AC · GPU 88% under dual-client load (capture + spotcheck)
serve: d3_server.py @ tag v3.1.0 · d3-v3.1.0-nf4 · port 8794 · VRAM 24,085 MiB
lane: --d3 (reflex 0cb63ff-era harness @ 46ddc75 lane code) · 1165.7 s for 2000 rows
datasets: .raw/datasets/mmlupro_jdi (103 test pages + train-000 via /first-rows — the Gaps entry)
```

Refs: Bench 142 (the 3-family gate + the correction) · riir-train `.issues/629` T2 · reflex Plan 012 A4 (the suite) · the dataset-manifest Gaps entry (the partial-population pin finding).
