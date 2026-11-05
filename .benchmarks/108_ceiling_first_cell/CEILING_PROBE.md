# Gold-support ceiling probe — sst5 (bonsai under CEILING_CONTRACT_V1)

- judge: bonsai @ 127.0.0.1:8082 (seat ladder primary; no-self-judging holds — the judge is not a scored entrant)
- validity gate: flipped catch 40/40 = 1.0000 (bar 0.8500; PASS) · supported-half catch 11/40 = 0.2750
- fate census (n = 600): correct supported 9 · correct NOT-supported 15 (shared-bias cell) · correct unclear 0 · wrong supported 76 · wrong not-supported 35 · wrong unclear 0 · abstained 349 · unknown 116
- SUPPORT CEILING ĝ = 0.6296 (Wilson 95% [0.5456, 0.7064], half-width 0.0804; Kish deff 1.1032 — n_eff = 122)
- coverage ceiling (gate serves): 0.4183 · raw lane accuracy (headline): 0.0867
- ceiling-adjusted score = per-item JOIN on the supported subset: 0.1059 over n 85 (raw stays the headline — never a division by ĝ) · correct∧not-supported = 15 (the lane agreeing with unsupported gold)

population floor: 600 sampled · 135 adjudicated · thresholds score 0.20488091/distance 0.41663086 at ρ 0.3
