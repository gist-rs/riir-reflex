# Issue 093 — CorpusDistanceGate drift vs corpus growth (gate-drift probe)

- date: 2026-10-17T14:58:20Z · host: m3 · sha: 86aa735
- family: ["ag_news", "emotion", "sst5", "xnli_en"] · off sources: ["banking77", "massive_intent_en"] · cap/label 64 · GATE_MID 0.35 · GATE_SCALE 8
- datasets: /Users/katopz/git/riir-reflex/.raw/datasets · budgets: off 200/source · in 100/suite
- box start: BoxState { power: Some("AC Power"), power_mode: Some("high"), load_1m: Some(28.96), swap_used_mb: Some(7486.62), load_source: Some("loadavg1m"), gpu_util_pct: None, gpu_mem_mib: None, latency_quotable: Some(false), refusals: ["load 28.96 > 6 — a sibling job is on the box"] }
- box end:   BoxState { power: Some("AC Power"), power_mode: Some("high"), load_1m: Some(28.96), swap_used_mb: Some(7486.62), load_source: Some("loadavg1m"), gpu_util_pct: None, gpu_mem_mib: None, latency_quotable: Some(false), refusals: ["load 28.96 > 6 — a sibling job is on the box"] }

## Off-corpus probe construction

test rows of banking77 + massive_intent_en (stride-sampled, text-field rule per suite) — sources held out from EVERY family suite at EVERY size, so the off-corpus and in-corpus populations never mix (issue T1a-i). Empty/whitespace texts dropped and counted, never measured.

## Postures

fixed = the birth boundary (max_sim < GATE_MID, exactly gate_conf < 0.5); refit = the rho=30 percentile distance threshold re-fitted on THIS size's pooled cal slice (absorbs drift by construction — never pooled with fixed); held1x = the 1x fitted threshold applied unchanged at later sizes (the aging-deployment posture, a disclosed addition).

## 1x_ag_news — domains 4 · corpus rows 256 · null √(2·ln M/256) = 0.2081

Per-domain rows: [64, 64, 64, 64] · refit threshold: 0.4105 (cal n = 200)

| population · surface | n | min | p05 | p50 | p90 | p99 | max | fixed abstain | refit abstain | held1x abstain |
|---|---|---|---|---|---|---|---|---|---|---|
| off · whole-corpus | 392 | 0.1081 | 0.1750 (sup 373) | 0.2673 (sup 196) | 0.3634 (sup 40) | 0.4490 (sup 4) | 0.4813 | — | — | — |
| off · routed gate | 392 | 0.0828 | 0.1528 (sup 373) | 0.2452 (sup 196) | 0.3464 (sup 40) | 0.4247 (sup 4) | 0.4813 | 90.8% (356) | 51.3% (0.411) | 51.3% (0.411) |
| in · whole-corpus | 100 | 0.1522 | 0.2066 (sup 95) | 0.3275 (sup 50) | 0.4472 (sup 10) | 0.8688 (sup 1) | 0.8688 | — | — | — |
| in · routed gate | 100 | 0.1262 | 0.1913 (sup 95) | 0.3032 (sup 50) | 0.4103 (sup 10) | 0.8688 (sup 1) | 0.8688 | 68.0% (68) | 19.0% (0.411) | 19.0% (0.411) |
| in · ENGINE (real requests, ctx embed) | 100 | 0.1659 | 0.2332 (sup 95) | 0.3197 (sup 50) | 0.4274 (sup 10) | 0.5085 (sup 1) | 0.5085 | 66.0% (66) | 9.0% (0.411) | 9.0% (0.411) |

Desert (routed reads): off p99 → mid clearance -0.0747 · in p05 above mid -0.1587 · refit off-corpus false-accept 48.7% · fixed in-corpus false-abstain 68.0%

Off counts per source: [("banking77", 193), ("massive_intent_en", 199)] · in counts per suite: [("ag_news", 100)]

## 2x_+emotion — domains 10 · corpus rows 640 · null √(2·ln M/256) = 0.2247

Per-domain rows: [64, 64, 64, 64, 64, 64, 64, 64, 64, 64] · refit threshold: 0.4160 (cal n = 400)

| population · surface | n | min | p05 | p50 | p90 | p99 | max | fixed abstain | refit abstain | held1x abstain |
|---|---|---|---|---|---|---|---|---|---|---|
| off · whole-corpus | 392 | 0.1605 | 0.2285 (sup 373) | 0.3489 (sup 196) | 0.4560 (sup 40) | 0.5566 (sup 4) | 0.6673 | — | — | — |
| off · routed gate | 392 | 0.0828 | 0.1741 (sup 373) | 0.2995 (sup 196) | 0.4062 (sup 40) | 0.4861 (sup 4) | 0.6673 | 73.7% (289) | 24.0% (0.416) | 23.5% (0.411) |
| in · whole-corpus | 200 | 0.1908 | 0.2604 (sup 190) | 0.4103 (sup 100) | 0.5272 (sup 20) | 0.7368 (sup 2) | 0.8688 | — | — | — |
| in · routed gate | 200 | 0.1761 | 0.2221 (sup 190) | 0.3655 (sup 100) | 0.4931 (sup 20) | 0.5657 (sup 2) | 0.8688 | 43.5% (87) | 10.5% (0.416) | 10.0% (0.411) |
| in · ENGINE (real requests, ctx embed) | 200 | 0.2003 | 0.2445 (sup 190) | 0.3264 (sup 100) | 0.4158 (sup 20) | 0.5085 (sup 2) | 0.5137 | 64.5% (129) | 6.0% (0.416) | 6.0% (0.411) |

Desert (routed reads): off p99 → mid clearance -0.1361 · in p05 above mid -0.1279 · refit off-corpus false-accept 76.0% · fixed in-corpus false-abstain 43.5%

Off counts per source: [("banking77", 193), ("massive_intent_en", 199)] · in counts per suite: [("ag_news", 100), ("emotion", 100)]

## 4x_+sst5+xnli — domains 18 · corpus rows 1152 · null √(2·ln M/256) = 0.2347

Per-domain rows: [64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64] · refit threshold: 0.4257 (cal n = 800)

| population · surface | n | min | p05 | p50 | p90 | p99 | max | fixed abstain | refit abstain | held1x abstain |
|---|---|---|---|---|---|---|---|---|---|---|
| off · whole-corpus | 392 | 0.1748 | 0.2594 (sup 373) | 0.3693 (sup 196) | 0.4670 (sup 40) | 0.5687 (sup 4) | 0.6673 | — | — | — |
| off · routed gate | 392 | 0.1060 | 0.1871 (sup 373) | 0.3040 (sup 196) | 0.4082 (sup 40) | 0.4936 (sup 4) | 0.6673 | 71.9% (282) | 23.0% (0.426) | 21.4% (0.411) |
| in · whole-corpus | 400 | 0.2094 | 0.2730 (sup 380) | 0.4003 (sup 200) | 0.5272 (sup 40) | 0.6237 (sup 4) | 0.8688 | — | — | — |
| in · routed gate | 400 | 0.1503 | 0.2134 (sup 380) | 0.3444 (sup 200) | 0.4716 (sup 40) | 0.5550 (sup 4) | 0.8688 | 51.5% (206) | 15.2% (0.426) | 14.0% (0.411) |
| in · ENGINE (real requests, ctx embed) | 400 | 0.1914 | 0.2409 (sup 380) | 0.3425 (sup 200) | 0.4573 (sup 40) | 0.5243 (sup 4) | 0.5501 | 54.8% (219) | 8.2% (0.426) | 7.0% (0.411) |

Desert (routed reads): off p99 → mid clearance -0.1436 · in p05 above mid -0.1366 · refit off-corpus false-accept 77.0% · fixed in-corpus false-abstain 51.5%

Off counts per source: [("banking77", 193), ("massive_intent_en", 199)] · in counts per suite: [("ag_news", 100), ("emotion", 100), ("sst5", 100), ("xnli_en", 100)]

## T2 close-condition inputs (the 4x_+sst5+xnli cell)

- off-corpus p99 clear of mid (whole-corpus read, fixed boundary): false
- off-corpus MAX clear of mid (whole-corpus read, fixed boundary): false
- off-corpus p99 clear of mid (routed read, fixed boundary): false
- refit false-accept rate on off-corpus: 77.0%

T2 closes ONLY if the tail stays clear in BOTH postures; cite the support columns beside every quantile (the p99 support shrinks with n).
