# Bench 083 — the openthai ⊕ laya teacher-ensemble gate (Plan 426 T2, GATE V3)

**Status: MEASURED — V3 FAIL. The ensemble does not beat the best member; T4's
distill falls back to openthai-single-teacher, exactly as the plan's
pre-registered clause reads.**

- date 2026-09-28 · sha dd11eab+ (this landing) · host m3 (loaded — the
  Tetris R4 trainer + sibling agents; an ACCURACY pass, no latency claim;
  the teachers are read once each over the frozen slice)
- teachers: openthai (openthai:openthai-systemone @ the pinned sha, the
  Bench-074 lane) vs laya (english checkpoint, Metal, G5-proven) — both
  through the T1 `TeacherForward` seam (one forward implementation per
  family; the fusion can never see teachers answered by different paths)
- slice: massive_intent_en's frozen test split (300 rows, the
  stratified-cap law) — **openthai reads 0.9200, byte-consistent with its
  published Bench-074 board number** (the slice construction is validated
  by reproduction)

| member/fusion | acc |
|---|---|
| openthai | **0.9200** |
| laya-english | 0.6933 |
| logit-mean (V3's pinned primary) | 0.8200 |
| rank-fusion (recorded alternate) | 0.8267 |

Paired LB95 (fusion − openthai): logit-mean **−0.1409**, rank **−0.1333**
— both fail the > 0 bar. Error overlap: **both-wrong 16 · openthai-only 8
· laya-only 76** — laya's 93 errors are almost a superset of openthai's
24: the teachers are heavily CORRELATED on this suite (openthai is a
fine-tuned superset of laya's signal there). The plan's V3 hypothesis
("heterogeneous teachers may have decorrelated errors — measured, never
assumed") is measured NEGATIVE; no ensemble rung, no label source better
than openthai alone.

Consequences: T4 (the fused-label student) proceeds under the FALLBACK
posture — openthai-single-teacher distillation through T1's
`--distill-teacher openthai` seam (V2-qualified: 0.9200 ≥ 0.90). The
stretch bar (0.9200) is the TEACHER's ceiling now; the student's win
condition is unchanged (V4: beat the served 0.8267, T2 paired LB95 > 0)
at ms-class latency (V6).

Instruments: `harness --ensemble-gate --ensemble-out <dir>` (ungated, the
agentjev family law — harness-only); per-row key alignment refuses wire
drift loud; the gate module's 4 unit tests (alignment, both fusion rules,
the paired LB95 hand-computation, the key extraction).

Files: `ensemble_gate.json` (the machine record) · `ENSEMBLE_GATE.md` (the
render) · `run.log` (the run's stderr).
