# Gate-fit threshold-stability audit (issue 109)

- suites: massive_intent_en, banking77, typed_decisions, prompt_injections, sst5, ag_news, xnli_en, emotion · iters 200 · seed 18713711869961 · rho 0.30 · offsets [0.02, 0.05, 0.1]
- bars: t* sd ≤ 0.05 · vacuity floor max(10, 0.05·n) · knife edge ≥2× drop at ±0.02
- posture: calibrated fit true · datasets E:\git\riir-reflex\.raw/datasets · default harness posture (levers off, kn_route on, calibrated gate fit per Issue 056) — the calibration pass re-run verbatim; bootstrap = SplitMix64 resamples with replacement, bounded index r % n (method pinned, report instrument); held-out curve on the stratified eval front through the SAME engine + calibrator. Cells are byte-identical across runs; the meta's provenance lines are disclosure, not cells.

population floor: 8 calibrated suite(s) audited · 0 unstable axis(es)

## massive_intent_en (labels 59 · corpus rows 11314 · eval n 150)

- t*: 0.2372 · cal n 200 · stable: true
  bootstrap: mean 0.2372 · sd 0.0133 · p05 0.2221 (support 190) · p95 0.2596 (support 10) · min 0.2103 · max 0.2872 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.2372 · 0.6319 · 0.9600 (144 / 150)
    0.2172 · 0.6327 · 0.9800 (147 / 150)
    0.1872 · 0.6351 · 0.9867 (148 / 150)
    0.1372 · 0.6376 · 0.9933 (149 / 150)
    0.3372 · 0.6794 · 0.8733 (131 / 150)
    0.2872 · 0.6791 · 0.8933 (134 / 150)
    0.2572 · 0.6408 · 0.9467 (142 / 150)
- t*: 0.5259 · cal n 200 · stable: true
  bootstrap: mean 0.5307 · sd 0.0105 · p05 0.5167 (support 190) · p95 0.5531 (support 10) · min 0.4955 · max 0.5595 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.5259 · 0.6289 · 0.6467 (97 / 150)
    0.5059 · 0.6075 · 0.7133 (107 / 150)
    0.4759 · 0.6240 · 0.8333 (125 / 150)
    0.4259 · 0.6232 · 0.9200 (138 / 150)
    0.6259 · 0.6515 · 0.4400 (66 / 150)
    0.5759 · 0.6250 · 0.5333 (80 / 150)
    0.5459 · 0.6344 · 0.6200 (93 / 150)

## banking77 (labels 77 · corpus rows 9793 · eval n 167)

- t*: 0.3177 · cal n 200 · stable: true
  bootstrap: mean 0.3178 · sd 0.0043 · p05 0.3098 (support 190) · p95 0.3250 (support 10) · min 0.3059 · max 0.3302 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.3177 · 0.4341 · 0.7725 (129 / 167)
    0.2977 · 0.4189 · 0.8862 (148 / 167)
    0.2677 · 0.3976 · 0.9940 (166 / 167)
    0.2177 · 0.4012 · 1.0000 (167 / 167)
    0.4177 · 0.4561 · 0.3413 (57 / 167)
    0.3677 · 0.4524 · 0.5030 (84 / 167)
    0.3377 · 0.4220 · 0.6527 (109 / 167)
- t*: 0.5549 · cal n 200 · stable: true
  bootstrap: mean 0.5515 · sd 0.0197 · p05 0.5074 (support 190) · p95 0.5817 (support 10) · min 0.4821 · max 0.5915 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.5549 · 0.4860 · 0.6407 (107 / 167)
    0.5349 · 0.4655 · 0.6946 (116 / 167)
    0.5049 · 0.4597 · 0.7425 (124 / 167)
    0.4549 · 0.4403 · 0.8024 (134 / 167)
    0.6549 · 0.4810 · 0.4731 (79 / 167)
    0.6049 · 0.4800 · 0.5988 (100 / 167)
    0.5749 · 0.4811 · 0.6347 (106 / 167)

## typed_decisions (labels 4 · corpus rows 1100 · eval n 200)

- t*: 0.3103 · cal n 100 · stable: true
  bootstrap: mean 0.3102 · sd 0.0001 · p05 0.3101 (support 190) · p95 0.3103 (support 10) · min 0.3100 · max 0.3103 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.3103 · 0.3000 · 0.7500 (150 / 200)
    0.2903 · 0.2250 · 1.0000 (200 / 200)
    0.2603 · 0.2250 · 1.0000 (200 / 200)
    0.2103 · 0.2250 · 1.0000 (200 / 200)
    0.4103 · n/a · 0.0000 (0 / 200)
    0.3603 · n/a · 0.0000 (0 / 200)
    0.3303 · n/a · 0.0000 (0 / 200)
- t*: 0.9122 · cal n 100 · stable: true
  bootstrap: mean 0.9100 · sd 0.0128 · p05 0.8885 (support 190) · p95 0.9273 (support 10) · min 0.8779 · max 0.9297 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.9122 · 0.1862 · 0.7250 (145 / 200)
    0.8922 · 0.1879 · 0.8250 (165 / 200)
    0.8622 · 0.2186 · 0.9150 (183 / 200)
    0.8122 · 0.2296 · 0.9800 (196 / 200)
    1.0122 · n/a · 0.0000 (0 / 200)
    0.9622 · 0.4583 · 0.1200 (24 / 200)
    0.9322 · 0.1754 · 0.5700 (114 / 200)

## prompt_injections (labels 2 · corpus rows 446 · eval n 116)

- t*: 0.5100 · cal n 100 · stable: true
  bootstrap: mean 0.5100 · sd 0.0000 · p05 0.5100 (support 190) · p95 0.5100 (support 10) · min 0.5100 · max 0.5100 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.5100 · 0.4828 · 1.0000 (116 / 116)
    0.4900 · 0.4828 · 1.0000 (116 / 116)
    0.4600 · 0.4828 · 1.0000 (116 / 116)
    0.4100 · 0.4828 · 1.0000 (116 / 116)
    0.6100 · n/a · 0.0000 (0 / 116)
    0.5600 · n/a · 0.0000 (0 / 116)
    0.5300 · n/a · 0.0000 (0 / 116)
- t*: 0.4619 · cal n 100 · stable: true
  bootstrap: mean 0.4647 · sd 0.0178 · p05 0.4435 (support 190) · p95 0.4957 (support 10) · min 0.4313 · max 0.5175 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.4619 · 0.4507 · 0.6121 (71 / 116)
    0.4419 · 0.4805 · 0.6638 (77 / 116)
    0.4119 · 0.4943 · 0.7500 (87 / 116)
    0.3619 · 0.5000 · 0.8621 (100 / 116)
    0.5619 · 0.3846 · 0.3362 (39 / 116)
    0.5119 · 0.3913 · 0.3966 (46 / 116)
    0.4819 · 0.4167 · 0.5172 (60 / 116)

## sst5 (labels 5 · corpus rows 8333 · eval n 200)

- t*: 0.2049 · cal n 200 · stable: true
  bootstrap: mean 0.2042 · sd 0.0046 · p05 0.1952 (support 190) · p95 0.2103 (support 10) · min 0.1900 · max 0.2170 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.2049 · 0.2044 · 0.6850 (137 / 200)
    0.1849 · 0.2000 · 0.8000 (160 / 200)
    0.1549 · 0.2100 · 1.0000 (200 / 200)
    0.1049 · 0.2100 · 1.0000 (200 / 200)
    0.3049 · 0.1724 · 0.1450 (29 / 200)
    0.2549 · 0.2083 · 0.3600 (72 / 200)
    0.2249 · 0.2414 · 0.5800 (116 / 200)
- t*: 0.4166 · cal n 200 · stable: true
  bootstrap: mean 0.4171 · sd 0.0076 · p05 0.4048 (support 190) · p95 0.4295 (support 10) · min 0.3876 · max 0.4410 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.4166 · 0.2319 · 0.6900 (138 / 200)
    0.3966 · 0.2157 · 0.7650 (153 / 200)
    0.3666 · 0.2071 · 0.8450 (169 / 200)
    0.3166 · 0.2228 · 0.9200 (184 / 200)
    0.5166 · 0.1951 · 0.4100 (82 / 200)
    0.4666 · 0.2037 · 0.5400 (108 / 200)
    0.4366 · 0.2195 · 0.6150 (123 / 200)

## ag_news (labels 4 · corpus rows 19800 · eval n 200)

- t*: 0.4397 · cal n 200 · stable: true
  bootstrap: mean 0.4397 · sd 0.0000 · p05 0.4397 (support 190) · p95 0.4397 (support 10) · min 0.4397 · max 0.4397 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.4397 · 0.4150 · 1.0000 (200 / 200)
    0.4197 · 0.4150 · 1.0000 (200 / 200)
    0.3897 · 0.4150 · 1.0000 (200 / 200)
    0.3397 · 0.4150 · 1.0000 (200 / 200)
    0.5397 · 0.5312 · 0.1600 (32 / 200)
    0.4897 · 0.4430 · 0.3950 (79 / 200)
    0.4597 · 0.4393 · 0.5350 (107 / 200)
- t*: 0.4105 · cal n 200 · stable: true
  bootstrap: mean 0.4131 · sd 0.0118 · p05 0.3949 (support 190) · p95 0.4306 (support 10) · min 0.3536 · max 0.4387 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.4105 · 0.3866 · 0.5950 (119 / 200)
    0.3905 · 0.4245 · 0.6950 (139 / 200)
    0.3605 · 0.4395 · 0.7850 (157 / 200)
    0.3105 · 0.4309 · 0.9050 (181 / 200)
    0.5105 · 0.4638 · 0.3450 (69 / 200)
    0.4605 · 0.4421 · 0.4750 (95 / 200)
    0.4305 · 0.4000 · 0.5500 (110 / 200)

## xnli_en (labels 3 · corpus rows 19800 · eval n 150)

- t*: 0.2999 · cal n 200 · stable: true
  bootstrap: mean 0.2996 · sd 0.0019 · p05 0.2966 (support 190) · p95 0.3020 (support 10) · min 0.2966 · max 0.3072 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.2999 · 0.4242 · 0.6600 (99 / 150)
    0.2799 · 0.3933 · 1.0000 (150 / 150)
    0.2499 · 0.3933 · 1.0000 (150 / 150)
    0.1999 · 0.3933 · 1.0000 (150 / 150)
    0.3999 · 0.0000 · 0.0200 (3 / 150)
    0.3499 · 0.4091 · 0.1467 (22 / 150)
    0.3199 · 0.5926 · 0.3600 (54 / 150)
- t*: 0.5110 · cal n 200 · stable: true
  bootstrap: mean 0.5077 · sd 0.0188 · p05 0.4775 (support 190) · p95 0.5337 (support 10) · min 0.4669 · max 0.5614 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.5110 · 0.3269 · 0.6933 (104 / 150)
    0.4910 · 0.3363 · 0.7533 (113 / 150)
    0.4610 · 0.3361 · 0.8133 (122 / 150)
    0.4110 · 0.3841 · 0.9200 (138 / 150)
    0.6110 · 0.3137 · 0.3400 (51 / 150)
    0.5610 · 0.3494 · 0.5533 (83 / 150)
    0.5310 · 0.3333 · 0.6400 (96 / 150)

## emotion (labels 6 · corpus rows 15799 · eval n 200)

- t*: 0.1453 · cal n 200 · stable: true
  bootstrap: mean 0.1453 · sd 0.0000 · p05 0.1453 (support 190) · p95 0.1453 (support 10) · min 0.1453 · max 0.1453 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.1453 · 0.3050 · 1.0000 (200 / 200)
    0.1253 · 0.3050 · 1.0000 (200 / 200)
    0.0953 · 0.3050 · 1.0000 (200 / 200)
    0.0453 · 0.3050 · 1.0000 (200 / 200)
    0.2453 · n/a · 0.0000 (0 / 200)
    0.1953 · n/a · 0.0000 (0 / 200)
    0.1653 · n/a · 0.0000 (0 / 200)
- t*: 0.4265 · cal n 200 · stable: true
  bootstrap: mean 0.4244 · sd 0.0108 · p05 0.4074 (support 190) · p95 0.4403 (support 10) · min 0.4010 · max 0.4509 · none 0
  vacuous resamples: 0 · knife edge: false
  held-out curve (t · acc · pass):
    0.4265 · 0.3259 · 0.6750 (135 / 200)
    0.4065 · 0.3103 · 0.7250 (145 / 200)
    0.3765 · 0.3253 · 0.8300 (166 / 200)
    0.3265 · 0.3102 · 0.9350 (187 / 200)
    0.5265 · 0.2987 · 0.3850 (77 / 200)
    0.4765 · 0.3333 · 0.5100 (102 / 200)
    0.4465 · 0.3333 · 0.6150 (123 / 200)

