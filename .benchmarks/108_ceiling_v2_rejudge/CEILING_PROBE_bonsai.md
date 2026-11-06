# Gold-support ceiling probe — sst5 (bonsai under CEILING_CONTRACT_V2)

- judge: bonsai @ 127.0.0.1:8083 (seat ladder primary; no-self-judging holds — the judge is not a scored entrant)
- validity gate: flipped catch 40/40 = 1.0000 (bar 0.8500; PASS) · supported-half catch 23/40 = 0.5750
  - supported-half split: not_supported 7 (the judge rejecting true gold — the ceiling's own signal) · unclear 0 · unknown 10 (quote/format failures — the SEAT cost, never folded into gold quality)
- fate census (n = 600): correct supported 33 · correct NOT-supported 13 (shared-bias cell) · correct unclear 0 · wrong supported 119 · wrong not-supported 41 · wrong unclear 0 · abstained 349 · unknown 45 (of which seat-starved 45 — empty content at finish=length, the seat cost)
- SUPPORT CEILING ĝ = 0.7379 (Wilson 95% [0.6738, 0.7932], half-width 0.0597; Kish deff 1.0602 — n_eff = 194)
- coverage ceiling (gate serves): 0.4183 · raw lane accuracy (headline): 0.0867
- ceiling-adjusted score = per-item JOIN on the supported subset: 0.2171 over n 152 (raw stays the headline — never a division by ĝ) · correct∧not-supported = 13 (the lane agreeing with unsupported gold)

population floor: 600 sampled · 206 adjudicated · thresholds score 0.20488091/distance 0.41663086 at ρ 0.3
