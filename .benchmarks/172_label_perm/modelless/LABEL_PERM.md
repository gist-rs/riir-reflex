# Label-perm probe — seeded bijective label-STRING rename (riir-train 657 R8)

- date 2026-10-30T19:14:03Z · sha 56fa2a7 · host m3
- datasets /Users/kat/git/riir-reflex/.raw/datasets · posture modelless · ≤0 case(s)/suite (0 = full slice)
- seed 106364929 · per-suite base = seed ^ FNV-1a(suite) · seeds base+0/+1/+2
- laya device: not probed
- spec: riir-train .issues/657 R8 — label-STRING bijection probe (options+gold follow σ). NOT Bench 119 (formatting-flip): Bench 119's axis is formatting churn of the state text; this axis is the label identity itself.
- rename: Sattolo derangement over the suite's option-key alphabet (every label moves); option ORDER preserved, criteria keys replaced in place, descriptions intact; gold rides by position (idx unchanged, string renamed through σ); per-suite base = seed ^ FNV-1a(suite), seeds base+0/+1/+2
- corpus: full-consistent relabel: train docs + engine domains + question keys through the SAME σ (the invariance arm; expected ≈ 0 delta — a large delta IS the finding)

| suite | lane | model | seed# | n | acc@id | acc@σ | Δ | 95% CI | flips→wrong/→right |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | modelless | full-consistent σ (corpus+domains+keys) | 0 | 400 | 0.4050 | 0.3300 | -0.0750 | [-0.1330, -0.0170] | 86/56 |
| ag_news | modelless | full-consistent σ (corpus+domains+keys) | 1 | 400 | 0.4050 | 0.3500 | -0.0550 | [-0.1076, -0.0024] | 69/47 |
| ag_news | modelless | full-consistent σ (corpus+domains+keys) | 2 | 400 | 0.4050 | 0.4050 | +0.0000 | [-0.0524, +0.0524] | 57/57 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 0 | 400 | 0.2750 | 0.1550 | -0.1200 | [-0.1781, -0.0619] | 97/49 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 1 | 400 | 0.2750 | 0.2625 | -0.0125 | [-0.0831, +0.0581] | 106/101 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 2 | 400 | 0.2750 | 0.2850 | +0.0100 | [-0.0594, +0.0794] | 98/102 |
| xnli_en | modelless | full-consistent σ (corpus+domains+keys) | 0 | 300 | 0.3400 | 0.3167 | -0.0233 | [-0.0999, +0.0532] | 72/65 |
| xnli_en | modelless | full-consistent σ (corpus+domains+keys) | 1 | 300 | 0.3400 | 0.3333 | -0.0067 | [-0.0863, +0.0729] | 75/73 |
| xnli_en | modelless | full-consistent σ (corpus+domains+keys) | 2 | 300 | 0.3400 | 0.3333 | -0.0067 | [-0.0863, +0.0729] | 75/73 |
| banking77 | modelless | full-consistent σ (corpus+domains+keys) | 0 | 500 | 0.3940 | 0.2320 | -0.1620 | [-0.2140, -0.1100] | 135/54 |
| banking77 | modelless | full-consistent σ (corpus+domains+keys) | 1 | 500 | 0.3940 | 0.2560 | -0.1380 | [-0.1847, -0.0913] | 110/41 |
| banking77 | modelless | full-consistent σ (corpus+domains+keys) | 2 | 500 | 0.3940 | 0.2140 | -0.1800 | [-0.2305, -0.1295] | 136/46 |

- ag_news: 400 slot(s) · seeds [240f495fe5e6eba4, 240f495fe5e6eba5, 240f495fe5e6eba6] · alphabet 4 · slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK

- emotion: 400 slot(s) · seeds [c16bcbb97eeaeb63, c16bcbb97eeaeb64, c16bcbb97eeaeb65] · alphabet 6 · slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK

- xnli_en: 300 slot(s) · seeds [1d2ac555f86cefa7, 1d2ac555f86cefa8, 1d2ac555f86cefa9] · alphabet 3 · slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK

- banking77: 500 slot(s) · seeds [7f344e07d843c26e, 7f344e07d843c26f, 7f344e07d843c270] · alphabet 77 · slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK
