# Teacher-ensemble gate (Plan 426 T2, GATE V3) — both teachers over the frozen test slice

- date 2026-09-28T12:47:01Z · sha ccb371a · teachers openthai vs laya
- fusion: logit-mean (pinned, V3) (primary) · rank-fusion (recorded alternate) (alternate)
- datasets .raw/datasets · rows: the frozen TEST slice, read once for the pre-registered V3 arm; teachers answer through the T1 seam (one forward per family)

| suite | n | acc A | acc B | logit-mean | rank | logit−best LB95 | rank−best LB95 | both wrong / A-only / B-only | V3 |
|---|---|---|---|---|---|---|---|---|---|
| massive_intent_en | 300 | 0.9200 | 0.6933 | 0.8200 | 0.8267 | -0.1409 | -0.1333 | 16 / 8 / 76 | FAIL |

- **massive_intent_en**: V3 FAIL — neither fusion beats the best member (openthai) at LB95 > 0; the distill falls back to single-teacher (the stronger member)
