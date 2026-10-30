# Label-perm probe — seeded bijective label-STRING rename (riir-train 657 R8)

- date 2026-10-30T19:19:55Z · sha 56fa2a7 · host m3
- datasets /Users/kat/git/riir-reflex/.raw/datasets · posture modelless · ≤200 case(s)/suite (0 = full slice)
- seed 106364929 · per-suite base = seed ^ FNV-1a(suite) · seeds base+0/+1/+2
- laya device: Metal
- spec: riir-train .issues/657 R8 — label-STRING bijection probe (options+gold follow σ). NOT Bench 119 (formatting-flip): Bench 119's axis is formatting churn of the state text; this axis is the label identity itself.
- rename: Sattolo derangement over the suite's option-key alphabet (every label moves); option ORDER preserved, criteria keys replaced in place, descriptions intact; gold rides by position (idx unchanged, string renamed through σ); per-suite base = seed ^ FNV-1a(suite), seeds base+0/+1/+2
- corpus: full-consistent relabel: train docs + engine domains + question keys through the SAME σ (the invariance arm; expected ≈ 0 delta — a large delta IS the finding)

| suite | lane | model | seed# | n | acc@id | acc@σ | Δ | 95% CI | flips→wrong/→right |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | modelless | full-consistent σ (corpus+domains+keys) | 0 | 200 | 0.4150 | 0.3350 | -0.0800 | [-0.1626, +0.0026] | 44/28 |
| ag_news | modelless | full-consistent σ (corpus+domains+keys) | 1 | 200 | 0.4150 | 0.3450 | -0.0700 | [-0.1415, +0.0015] | 34/20 |
| ag_news | modelless | full-consistent σ (corpus+domains+keys) | 2 | 200 | 0.4150 | 0.4650 | +0.0500 | [-0.0258, +0.1258] | 25/35 |
| ag_news | laya/english | laya/english (keys renamed through σ; descriptions intact) | 0 | 200 | 0.9650 | 0.0350 | -0.9300 | [-0.9681, -0.8919] | 187/1 |
| ag_news | laya/english | laya/english (keys renamed through σ; descriptions intact) | 1 | 200 | 0.9650 | 0.0650 | -0.9000 | [-0.9501, -0.8499] | 184/4 |
| ag_news | laya/english | laya/english (keys renamed through σ; descriptions intact) | 2 | 200 | 0.9650 | 0.0300 | -0.9350 | [-0.9693, -0.9007] | 187/0 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 0 | 200 | 0.3050 | 0.1650 | -0.1400 | [-0.2246, -0.0554] | 53/25 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 1 | 200 | 0.3050 | 0.2600 | -0.0450 | [-0.1492, +0.0592] | 61/52 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 2 | 200 | 0.3050 | 0.2850 | -0.0200 | [-0.1239, +0.0839] | 58/54 |
| emotion | laya/english | laya/english (keys renamed through σ; descriptions intact) | 0 | 200 | 0.5700 | 0.0600 | -0.5100 | [-0.5945, -0.4255] | 114/12 |
| emotion | laya/english | laya/english (keys renamed through σ; descriptions intact) | 1 | 200 | 0.5700 | 0.0950 | -0.4750 | [-0.5660, -0.3840] | 113/18 |
| emotion | laya/english | laya/english (keys renamed through σ; descriptions intact) | 2 | 200 | 0.5700 | 0.1100 | -0.4600 | [-0.5509, -0.3691] | 110/18 |
| xnli_en | modelless | full-consistent σ (corpus+domains+keys) | 0 | 150 | 0.3933 | 0.4067 | +0.0133 | [-0.1039, +0.1306] | 39/41 |
| xnli_en | modelless | full-consistent σ (corpus+domains+keys) | 1 | 150 | 0.3933 | 0.3200 | -0.0733 | [-0.1847, +0.0381] | 42/31 |
| xnli_en | modelless | full-consistent σ (corpus+domains+keys) | 2 | 150 | 0.3933 | 0.3200 | -0.0733 | [-0.1847, +0.0381] | 42/31 |
| xnli_en | laya/english | laya/english (keys renamed through σ; descriptions intact) | 0 | 150 | 0.8400 | 0.0533 | -0.7867 | [-0.8708, -0.7025] | 126/8 |
| xnli_en | laya/english | laya/english (keys renamed through σ; descriptions intact) | 1 | 150 | 0.8400 | 0.1067 | -0.7333 | [-0.8360, -0.6307] | 126/16 |
| xnli_en | laya/english | laya/english (keys renamed through σ; descriptions intact) | 2 | 150 | 0.8400 | 0.1067 | -0.7333 | [-0.8360, -0.6307] | 126/16 |
| banking77 | modelless | full-consistent σ (corpus+domains+keys) | 0 | 167 | 0.4012 | 0.2335 | -0.1677 | [-0.2552, -0.0801] | 44/16 |
| banking77 | modelless | full-consistent σ (corpus+domains+keys) | 1 | 167 | 0.4012 | 0.2934 | -0.1078 | [-0.1927, -0.0228] | 36/18 |
| banking77 | modelless | full-consistent σ (corpus+domains+keys) | 2 | 167 | 0.4012 | 0.2096 | -0.1916 | [-0.2764, -0.1068] | 45/13 |
| banking77 | laya/english | laya/english (keys renamed through σ; descriptions intact) | 0 | 167 | 0.4251 | 0.0060 | -0.4192 | [-0.4960, -0.3423] | 71/1 |
| banking77 | laya/english | laya/english (keys renamed through σ; descriptions intact) | 1 | 167 | 0.4251 | 0.0060 | -0.4192 | [-0.4942, -0.3441] | 70/0 |
| banking77 | laya/english | laya/english (keys renamed through σ; descriptions intact) | 2 | 167 | 0.4251 | 0.0120 | -0.4132 | [-0.4881, -0.3383] | 69/0 |

- ag_news: 200 slot(s) · seeds [240f495fe5e6eba4, 240f495fe5e6eba5, 240f495fe5e6eba6] · alphabet 4 · slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK

- emotion: 200 slot(s) · seeds [c16bcbb97eeaeb63, c16bcbb97eeaeb64, c16bcbb97eeaeb65] · alphabet 6 · slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK

- xnli_en: 150 slot(s) · seeds [1d2ac555f86cefa7, 1d2ac555f86cefa8, 1d2ac555f86cefa9] · alphabet 3 · slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK

- banking77: 167 slot(s) · seeds [7f344e07d843c26e, 7f344e07d843c26f, 7f344e07d843c270] · alphabet 77 · slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK
