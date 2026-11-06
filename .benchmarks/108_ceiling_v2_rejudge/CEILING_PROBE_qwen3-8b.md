# Gold-support ceiling probe — sst5 (qwen3-8b under CEILING_CONTRACT_V2)

- judge: qwen3-8b @ 127.0.0.1:8082 (seat ladder primary; no-self-judging holds — the judge is not a scored entrant)
- validity gate: flipped catch 39/40 = 0.9750 (bar 0.8500; PASS) · supported-half catch 27/40 = 0.6750
  - supported-half split: not_supported 12 (the judge rejecting true gold — the ceiling's own signal) · unclear 0 · unknown 1 (quote/format failures — the SEAT cost, never folded into gold quality)
- fate census (n = 600): correct supported 34 · correct NOT-supported 16 (shared-bias cell) · correct unclear 0 · wrong supported 136 · wrong not-supported 59 · wrong unclear 1 · abstained 349 · unknown 5 (of which seat-starved 0 — empty content at finish=length, the seat cost)
- SUPPORT CEILING ĝ = 0.6911 (Wilson 95% [0.6307, 0.7455], half-width 0.0574; Kish deff 1.0458 — n_eff = 235)
- coverage ceiling (gate serves): 0.4183 · raw lane accuracy (headline): 0.0867
- ceiling-adjusted score = per-item JOIN on the supported subset: 0.2000 over n 170 (raw stays the headline — never a division by ĝ) · correct∧not-supported = 16 (the lane agreeing with unsupported gold)

population floor: 600 sampled · 246 adjudicated · thresholds score 0.20488091/distance 0.41663086 at ρ 0.3
