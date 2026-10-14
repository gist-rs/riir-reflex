# Bench 144 — the d3 leg-A capture readout + the T2.5 router's first real run (riir-train 629 T2/T2.5)

> **Status:** RECORD — capture COMPLETE (EXIT 0, 6/6 suites, 0 absences) + router ALL 6 ROUTED, 0 joins refused (2026-10-14, 4090 lane) · host 4090-windows (shikuwa) · the routed artifacts: `.raw/distill_router/`
> **Rule:** the captures are TRAIN-ROW teacher columns (the no-cheat law — `--distill` reads train rows only, per-row gold pins); the router is the MODELLESS-first arm (629 T2.5): the fine-tune (T3) now has to beat the ROUTER, not the raw teacher. Nothing here is a test-row read — the verdict is T4's single frozen read.

## 1. The capture (d3-v3.1.0-nf4, the bench-142/143 spot-checked posture)

- 08:13:52 → 14:42:56 +07 (6.5 h wall, 23,344 s compute; includes the 08:36–08:56 spotcheck-143 interleave on the same serve).
- 6 suites, 66,597 rows, every bin blake3-sidecared; manifest `.raw/distill_teacher_d3/distill.json` (DISTILL.md beside it).

| suite | rows | classes | d3 acc | p50 ms | incumbent (teacher, acc) | Δ |
|---|---|---|---|---|---|---|
| emotion | 16000 | 6 | **0.6246** | 247 | pplx_v2 0.6301 | −0.55 pp |
| prompt_injections | 546 | 2 | **0.9890** | 223 | pplx_v2 0.9817 | **+0.73 pp** |
| xnli_en | 20000 | 3 | **0.9166** | 272 | pplx_v2 0.9207 | −0.41 pp |
| sst5 | 8544 | 5 | **0.5813** | 252 | clef 0.5988 | −1.75 pp |
| massive_intent_en | 11514 | 60 | **0.9375** | 346 | clef 0.9685 | −3.10 pp |
| banking77 | 9993 | 77 | **0.9090** | 729 | clef 0.9492 | −4.02 pp |

- **Reading:** d3 and pplx are PEERS on the language trio (±0.7 pp, all blend-zone); clef is decisively stronger on its own trio (1.8–4.0 pp) — the 623 slot assignment (pplx→language, clef→classification-heavy) is exactly what the captures show, now with full-population train-row numbers.
- **d3's pi WIN (0.9890 > 0.9817) is the first slot-level change the router surfaces** — the routed column carries d3 there.

## 2. The router (T2.5 — `--teacher-router`, reflex `0cb63ff`; sigmoid, never softmax; T=0.01)

Inputs: `.raw/distill_teacher_d3` ⊕ `.raw/distill_teacher_pplx_v2` ⊕ `.raw/distill_teacher_clef` — every join verified by per-row gold pins across captures (0 refused).

| suite | gate g | routed acc | vs best single | note |
|---|---|---|---|---|
| banking77 | 0.0178 | 0.9492 | = clef | hard clef pick |
| emotion | 0.3659 | **0.6302** | +0.01 pp | blend edges BOTH teachers |
| massive_intent_en | 0.0431 | 0.9681 | −0.04 pp | near-hard clef pick |
| prompt_injections | 0.6754 | **0.9890** | = d3 | **d3 wins a slot the 623 lane gave pplx** |
| sst5 | 0.1488 | **0.6016** | +0.28 pp | blend beats BOTH teachers |
| xnli_en | 0.3977 | 0.9199 | −0.08 pp | blend zone |

- **Routed column mean 0.8430 vs the per-suite-oracle 0.8427** — the modelless router MATCHES the best-teacher-per-suite oracle (and edges it via the sst5/emotion ensemble gains), with ZERO training spend. T2.5's premise holds on the captured columns.
- The routed RIDT artifacts (one per suite + blake3 + `router.json`/`ROUTER.md`) are the drop-in teacher column for T3's student and T4's frozen read.
- Honest caveat (the manifest carries it): the gate inputs are TRAIN-row accuracies — the routed column's TEST-row behavior is T4's to read, once; nothing here selected on it.

## 3. The determinism witness (det legs) — PASS

`--limit 200` × 2 legs over all 6 suites (`_det1`/`_det2`, the bench-141 protocol mirror; both EXIT 0, 15:22–15:41 +07):

- **Row payloads BYTE-IDENTICAL on all 6 suites** (det1 ≡ det2 after the
  header; the only whole-file difference is the `created_utc` header field
  — 3–4 bytes per file, the expected timestamp). The d3-v3.1.0-nf4 serve is
  deterministic across legs — the same witness class as pplx v2's bench-141
  det pair.
- det-leg accs at n=200 match the full-capture column within sampling
  spread (e.g. emotion 0.625 at 200 vs 0.6246 full; massive 0.9450 at 200 vs
  0.9375 full).

## 4. The mmlupro full-pop re-read — SAME DIGEST, SAME NUMBER (the §15 pin is stable; a deeper finding filed)

The 15:45 re-read over the topped-up 12,032-row population reproduced
`cases_digest 6a2d24628ce7b916` and d3 acc **0.7270** byte-identically — the
sample does not move with the top-up. Investigation (six candidate ordering
laws replicated, one emptying-a-tail-page scratch experiment, one new unit
test) could NOT reproduce the sample under ANY replicable law — the sample is
provably tail-insensitive yet matches neither first-N nor the documented
hash-order — **filed as reflex `.issues/088`**. The Bench-143 number stands as
a real measurement; the sampling-law question is the issue's.

## Provenance

```
host: 4090-windows (shikuwa) · AC · GPU exclusive post-08:56 (interleave disclosed §1)
serve: d3_server.py @ tag v3.1.0 · d3-v3.1.0-nf4 (D3_BNB_NF4=1, 20 GiB) · port 8794 · VRAM 24,085 MiB
lane: --distill --distill-teacher d3 (reflex 46ddc75 harness) · datasets .raw/datasets_t20k
router: --teacher-router (reflex 0cb63ff harness, /tmp build) · offline (no GPU, no test row)
```

Refs: riir-train `.issues/629` T2/T2.5 · Bench 141 (the pplx v2 precedent + gates) · Bench 142/143 (the d3 spot checks under the corrected row) · the dataset-manifest Gaps (the mmlupro top-up).
