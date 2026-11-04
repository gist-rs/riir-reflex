# Issue 107 — reflex routing-state probe + pinned-path replay

- date: 2026-11-04T04:38:40Z · host: amo · sha: b8ec0eb
- datasets: /Users/katopz/git/riir-reflex/.raw/datasets
- posture: deployed engine build (fit_posture_inner + build_engine_with, selection levers pinned off, cal-slice observe calibration); routing-state features re-derived at the routing site (dot(embed, direction), the pick_domain fold) via the read-only routing_directions accessor; ridge OVR closed-form (katgpt-core ridge_solve_direct_f32, bias column, λ grid, gates at the pre-nominated λ = 10.0); pins replay through the Scratch::forced_domain seam (report-only; production paths disarm it)
- gates: λ = 10 (pre-nominated; the grid is disclosure) · tail floor 10 · non-inferiority LB95 > −0.01

## T1 — the supervised probe (score-vector arm MANDATORY; one-hot = MI consistency)

| suite | fit | eval | N | MI bits | H(label) | routed acc | probe acc (score) | Δ | 2σ | macro AUROC (score) | incl/excl | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ag_news | 200 | 200 | 4 | 0.1506 | 1.9686 | 0.3650 | 0.3600 | -0.0050 | 0.0681 | 0.5880 | 4/0 | AGREE-WITH-MI |
| emotion | 200 | 200 | 6 | 0.1197 | 2.2993 | 0.1450 | 0.1950 | +0.0500 | 0.0498 | 0.4955 | 5/1 | AGREE-WITH-MI |
| sst5 | 200 | 300 | 5 | 0.0677 | 2.2606 | 0.2133 | 0.2333 | +0.0200 | 0.0473 | 0.5187 | 5/0 | AGREE-WITH-MI |
| banking77 | 200 | 250 | 77 | 4.8947 | 6.2547 | 0.5120 | 0.1920 | -0.3200 | 0.0632 | — | 0/77 | UNDERREAD |

### Arms at the gate λ

| suite | arm | acc | macro AUROC | included | excluded | degenerate |
|---|---|---|---|---|---|---|
| ag_news | score_vector | 0.3600 | 0.5880 | 4 | 0 | no |
| ag_news | one_hot | 0.3650 | 0.5947 | 4 | 0 | no |
| emotion | score_vector | 0.1950 | 0.4955 | 5 | 1 | no |
| emotion | one_hot | 0.1650 | 0.5304 | 5 | 1 | no |
| sst5 | score_vector | 0.2333 | 0.5187 | 5 | 0 | no |
| sst5 | one_hot | 0.2267 | 0.4999 | 5 | 0 | no |
| banking77 | score_vector | 0.1920 | — | 0 | 77 | no |
| banking77 | one_hot | 0.3800 | — | 0 | 77 | no |

### λ grid (score-vector arm — disclosure; gates never read the argmax-λ row)

| suite | λ | acc | macro AUROC | included |
|---|---|---|---|---|
| ag_news | 0.001 | 0.4550 | 0.6899 | 4 |
| ag_news | 0.01 | 0.4500 | 0.6891 | 4 |
| ag_news | 0.1 | 0.4600 | 0.6858 | 4 |
| ag_news | 1 | 0.4000 | 0.6549 | 4 |
| ag_news | 10 | 0.3600 | 0.5880 | 4 |
| ag_news | 100 | 0.3300 | 0.4995 | 4 |
| emotion | 0.001 | 0.1900 | 0.5257 | 5 |
| emotion | 0.01 | 0.1900 | 0.5316 | 5 |
| emotion | 0.1 | 0.1950 | 0.5477 | 5 |
| emotion | 1 | 0.2200 | 0.5484 | 5 |
| emotion | 10 | 0.1950 | 0.4955 | 5 |
| emotion | 100 | 0.1350 | 0.5026 | 5 |
| sst5 | 0.001 | 0.2267 | 0.5267 | 5 |
| sst5 | 0.01 | 0.2300 | 0.5267 | 5 |
| sst5 | 0.1 | 0.2367 | 0.5254 | 5 |
| sst5 | 1 | 0.2367 | 0.5225 | 5 |
| sst5 | 10 | 0.2333 | 0.5187 | 5 |
| sst5 | 100 | 0.2267 | 0.4993 | 5 |
| banking77 | 0.001 | 0.3160 | — | 0 |
| banking77 | 0.01 | 0.3480 | — | 0 |
| banking77 | 0.1 | 0.3400 | — | 0 |
| banking77 | 1 | 0.2480 | — | 0 |
| banking77 | 10 | 0.1920 | — | 0 |
| banking77 | 100 | 0.1000 | — | 0 |

### T1.4 verdict notes

- **ag_news**: LOW-MI (0.1506 bits): macro AUROC 0.5880 ≤ 0.70 — no hidden signal; the MI picture holds
- **emotion**: LOW-MI (0.1197 bits): macro AUROC 0.4955 ≤ 0.70 — no hidden signal; the MI picture holds
- **sst5**: LOW-MI (0.0677 bits): macro AUROC 0.5187 ≤ 0.70 — no hidden signal; the MI picture holds
- **banking77**: HIGH-MI (4.8947 bits): probe acc -0.3200 BELOW the 2σ band (±0.0632) — the linear probe underreads the lookup at this fit size (fit-slice capacity, not absent signal)

## T2 — mode-path pins (report-only; the forced_domain seam stays production-disarmed)

| suite | pin | paired | live | pinned | Δ | LB95 | verdict | abst Δ | fallback |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | suite_constant | 100 | 0.4000 | 0.4000 | +0.0000 | +0.0000 | NON-INFERIOR | +0.055 | 0 |
| ag_news | predicted_class | 109 | 0.4037 | 0.4037 | +0.0000 | +0.0000 | NON-INFERIOR | +0.010 | 0 |
| ag_news | gold_oracle | 104 | 0.4327 | 0.4327 | +0.0000 | +0.0000 | NON-INFERIOR | +0.040 | 0 |
| emotion | suite_constant | 132 | 0.3258 | 0.3182 | -0.0076 | -0.0224 | INFERIOR | -0.020 | 0 |
| emotion | predicted_class | 132 | 0.3258 | 0.3182 | -0.0076 | -0.0224 | INFERIOR | -0.020 | 0 |
| emotion | gold_oracle | 132 | 0.3258 | 0.3182 | -0.0076 | -0.0224 | INFERIOR | -0.020 | 0 |
| sst5 | suite_constant | 176 | 0.1591 | 0.1591 | +0.0000 | +0.0000 | NON-INFERIOR | +0.013 | 0 |
| sst5 | predicted_class | 175 | 0.1543 | 0.1543 | +0.0000 | +0.0000 | NON-INFERIOR | +0.020 | 1 |
| sst5 | gold_oracle | 176 | 0.1591 | 0.1591 | +0.0000 | +0.0000 | NON-INFERIOR | +0.013 | 0 |
| banking77 | suite_constant | 6 | 0.3333 | 0.3333 | +0.0000 | +0.0000 | NON-INFERIOR | +0.440 | 0 |
| banking77 | predicted_class | 58 | 0.5862 | 0.6379 | +0.0517 | -0.0058 | NON-INFERIOR | +0.216 | 0 |
| banking77 | gold_oracle | 66 | 0.6212 | 0.6364 | +0.0152 | -0.0145 | INFERIOR | +0.176 | 0 |

### Pin ceilings

- **ag_news / suite_constant**: class-blind constant pin — every case forced to one domain (a named baseline shape, not a result)
- **ag_news / predicted_class**: ceiling = T1 probe accuracy (0.3600) — the pin replays what the probe predicts, never beyond it
- **ag_news / gold_oracle**: ORACLE CEILING ONLY — the key IS the gold label (leakage-bound); bounds what a perfect classifier-keyed pin could achieve, never a non-inferiority result
- **emotion / suite_constant**: class-blind constant pin — every case forced to one domain (a named baseline shape, not a result)
- **emotion / predicted_class**: ceiling = T1 probe accuracy (0.1950) — the pin replays what the probe predicts, never beyond it
- **emotion / gold_oracle**: ORACLE CEILING ONLY — the key IS the gold label (leakage-bound); bounds what a perfect classifier-keyed pin could achieve, never a non-inferiority result
- **sst5 / suite_constant**: class-blind constant pin — every case forced to one domain (a named baseline shape, not a result)
- **sst5 / predicted_class**: ceiling = T1 probe accuracy (0.2333) — the pin replays what the probe predicts, never beyond it
- **sst5 / gold_oracle**: ORACLE CEILING ONLY — the key IS the gold label (leakage-bound); bounds what a perfect classifier-keyed pin could achieve, never a non-inferiority result
- **banking77 / suite_constant**: class-blind constant pin — every case forced to one domain (a named baseline shape, not a result)
- **banking77 / predicted_class**: ceiling = T1 probe accuracy (0.1920) — the pin replays what the probe predicts, never beyond it
- **banking77 / gold_oracle**: ORACLE CEILING ONLY — the key IS the gold label (leakage-bound); bounds what a perfect classifier-keyed pin could achieve, never a non-inferiority result

### Witnesses

- **ag_news**: capture determinism GREEN · extraction digest GREEN (`deaab4c27e9c387fd999ee3522a6dad923f60c861a4a637cbd0808b03709fc7b`) · suite modal domain 2
- **emotion**: capture determinism GREEN · extraction digest GREEN (`597e2138723877c7f7c4149ddbf4c2754dc4be256939c4010d406b162d8f4de4`) · suite modal domain 2
- **sst5**: capture determinism GREEN · extraction digest GREEN (`d1aa9adb84c5ccb6f6ebe8bf4d816b02f191e2c17e28679065aa650bb838ce5f`) · suite modal domain 3
- **banking77**: capture determinism GREEN · extraction digest GREEN (`28baaed38e0da277741eea6c65cbfd211d76c36c805675c3b0bf86bb47fb4922`) · suite modal domain 52

**Lane verdict: LIVE** — the probe lane is live when any suite's score-vector arm is non-degenerate with green witnesses.

Scope notes: the score vector is re-derived with `pick_domain`'s exact fold from `Scratch.q` + `routing_directions()` and per-slot asserted against the engine's routed domain. The one-hot arm MUST agree with the MI picture (a linear readout over one-hot inputs is the per-domain lookup) — agreement is the deliverable. Bench 157's depth-as-confidence NEGATIVE is a different feature source (residual stream, not routing state) and is not re-run here. NO promotion: the seam stays disarmed on production paths exactly as Bench 167 left it (T2.3).
