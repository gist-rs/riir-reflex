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

1. **The population is PARTIAL and the pin is partial-era.** The local test fetch holds 10,300/12,032 rows (qids 70..10504, holes at both ends — the 429 wall scattered the failed pages; 12 of 14 subjects present). The run's `cases_digest` = `fnv1a64-6a2d24628ce7b916` — BYTE-IDENTICAL to the manifest §15 pin, which therefore was minted over the SAME partial population on the rethink side (Bench 067 `hashampled_reread`), not the full 12,032 the manifest's fetch table describes. Recorded in the dataset-manifest Gaps; any FROZEN claim read needs the top-up + a NEW full-population pin first.
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
