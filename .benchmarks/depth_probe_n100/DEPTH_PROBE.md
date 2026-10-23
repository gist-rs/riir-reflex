# Depth probe — residual stabilization vs correctness (Issue 094 T2)

checkpoint `english` · device `Cpu` · band cos ≥ 0.995 / l2 ≤ 25 · trough window 2 · ≤100 case(s)/suite

| suite | q | acc | trough p10/p50/p90 | none | AUROC(trough→correct) | ECE(trough) | modal floor |
|---|---|---|---|---|---|---|---|
| typed_decisions | 500 | 0.3260 | 3/3/4 | 321 | 0.5393 | 0.4888 | 0.5175 |
| ag_news | 100 | 0.9600 | 3/3/3 | 0 | 0.6510 | 0.0600 | 0.3275 |
| emotion | 100 | 0.5400 | 2/2/3 | 0 | 0.5370 | 0.2000 | 0.3275 |
| sst5 | 100 | 0.3200 | 2/2/3 | 0 | 0.4026 | 0.4200 | 0.2750 |
| prompt_injections | 58 | 0.7241 | 2/3/3 | 0 | 0.3899 | 0.3103 | 0.4741 |
| xnli_en | 100 | 0.8700 | 2/2/3 | 0 | 0.5225 | 0.1667 | 0.0000 |
| massive_intent_en | 100 | 0.7200 | 2/2/2 | 0 | 0.5288 | 0.1000 | 0.0467 |
| banking77 | 100 | 0.4500 | 4/4/10 | 79 | 0.5754 | 0.2524 | 0.0420 |
| semantic_defects | 51 | 0.3137 | 3/3/3 | 0 | 0.4464 | 0.6863 | 0.0000 |
