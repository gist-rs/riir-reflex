# Clef conf probe — last-position entropy vs the two floors (Issue 094 T3)

clef-text-0.6b (FluidInference student of Cloudflare clef, Apache-2.0) · local in-process clef-text-0.6b @ 3c61a60954478ef385aa42d67d5764642dcc9135 — buckets [256, 512, 1024, 2048] (max_questions 16, max_options 96) compute_units CpuAndNe — parity: bit-exact vs FluidUse@0230049c (riir-infer bench 884 / plan 619 T2.2) · vocab 151936 · ≤100 case(s)/suite · entropy→conf map `(1 − H/ln V).clamp(0,1)` · case-level signal (one entropy per case, repeated across its questions)

| suite | q | acc | AUROC(−H→correct) | ECE(H) | ECE(conf) | conformal floor | modal acc | modal ECE | verdict |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | 500 | 0.4220 | 0.4340 | 0.4599 | 0.1578 | 0.3319 | 0.4640 | 0.1308 | NEGATIVE(conformal 0.4599 ≥ floor 0.3319; modal 0.4599 ≥ floor 0.1308; lane acc 0.4220 ≤ modal acc 0.464 (mimicry)) |
| ag_news | 100 | 0.8700 | 0.6198 | 0.0406 | 0.0671 | 0.4733 | 0.2625 | 0.0088 | NEGATIVE(modal 0.0406 ≥ floor 0.0088) |
| emotion | 100 | 0.6500 | 0.5741 | 0.2874 | 0.1352 | 0.1745 | 0.3125 | 0.2123 | NEGATIVE(conformal 0.2874 ≥ floor 0.1745; modal 0.2874 ≥ floor 0.2123) |
| sst5 | 100 | 0.4700 | 0.5183 | 0.4859 | 0.1214 | 0.2263 | 0.1383 | 0.0790 | NEGATIVE(conformal 0.4859 ≥ floor 0.2263; modal 0.4859 ≥ floor 0.0790) |
| prompt_injections | 58 | 0.4828 | 0.5298 | 0.4558 | 0.3994 | 0.3494 | 0.5345 | 0.0043 | NEGATIVE(conformal 0.4558 ≥ floor 0.3494; modal 0.4558 ≥ floor 0.0043; lane acc 0.4828 ≤ modal acc 0.5344827586206896 (mimicry)) |
| xnli_en | 100 | 0.7700 | 0.5935 | 0.1659 | 0.1411 | 0.3584 | 0.3333 | 0.0283 | NEGATIVE(modal 0.1659 ≥ floor 0.0283) |
| massive_intent_en | 100 | 0.8100 | 0.4659 | 0.1126 | 0.1329 | 0.2878 | 0.1100 | 0.0440 | NEGATIVE(modal 0.1126 ≥ floor 0.0440) |
| banking77 | 100 | 0.8300 | 0.3650 | 0.1460 | 0.0807 | 0.3200 | 0.0460 | 0.1771 | PASS |
| semantic_defects | 51 | 0.4510 | 0.5202 | 0.4706 | 0.2245 | 0.2426 | 0.1667 | 0.1111 | NEGATIVE(conformal 0.4706 ≥ floor 0.2426; modal 0.4706 ≥ floor 0.1111) |

G1: the candidate must beat BOTH floors strictly; `beats conformal but not modal` = mimicry, `beats modal but not conformal` = miscalibration, a lane at/below the modal accuracy carries no input-independent signal. Lens depth is DEFERRED (the ANE artifact ships one output — a CPU forward port is the unblock; recorded in the issue).
