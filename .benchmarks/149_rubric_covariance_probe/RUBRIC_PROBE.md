# Rubric-covariance + route-agreement probe (Issue 090 C1)

2026-10-15T22:57:16Z host shikuwa · c6fedbb · datasets `E:\git\riir-reflex\.raw/datasets` · ≤40 case(s)/suite

| suite | slots | I1 scale (raw/norm/wire) | I2 shift | I3 argmax Δ | Σq | verdict |
|---|---|---|---|---|---|---|
| ag_news | 25 | 1.1e-13/1.1e-13/1.1e-13 | 1.4e-14/7.1e-15/1.2e-14 | 0/0/0 | 1.0e-7/1.1e-16/1.0e-6 | PASS |
| emotion | 24 | 1.7e-13/1.1e-13/1.7e-13 | 1.4e-14/1.4e-14/1.4e-14 | 0/0/0 | 1.2e-7/1.1e-16/2.0e-6 | PASS |
| sst5 | 25 | 5.7e-14/1.7e-13/1.7e-13 | 0.0e0/7.1e-15/6.2e-15 | 0/0/0 | 3.0e-8/0.0e0/1.0e-6 | PASS |
| xnli_en | 14 | 1.1e-13/1.1e-13/1.1e-13 | 7.1e-15/1.4e-14/7.1e-15 | 0/0/0 | 6.0e-8/0.0e0/1.0e-6 | PASS |
| massive_intent_en | 20 | 2.8e-13/1.7e-13/1.7e-13 | 1.4e-14/3.0e-14/2.0e-14 | 0/0/0 | 1.5e-7/2.2e-16/3.0e-6 | PASS |
| banking77 | 21 | 3.4e-13/4.0e-13/3.4e-13 | 1.4e-14/4.3e-14/3.2e-14 | 0/0/0 | 2.5e-7/4.4e-16/6.0e-6 | PASS |

Route agreement (characterization, context = state+prompt+criteria route): context==state-alone 59.7% · option-route==context 4.7% · name==semantic 43.4% of named options.

- skip [ag_news/modelless]: 15 abstained slot(s) skipped (no distribution)
- skip [emotion/modelless]: 16 abstained slot(s) skipped (no distribution)
- skip [sst5/modelless]: 15 abstained slot(s) skipped (no distribution)
- skip [xnli_en/modelless]: 24 abstained slot(s) skipped (no distribution)
- skip [massive_intent_en/modelless]: 18 abstained slot(s) skipped (no distribution)
- skip [banking77/modelless]: 18 abstained slot(s) skipped (no distribution)
