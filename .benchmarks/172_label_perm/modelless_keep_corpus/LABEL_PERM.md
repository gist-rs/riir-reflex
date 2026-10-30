# Label-perm probe — seeded bijective label-STRING rename (riir-train 657 R8)

- date 2026-10-30T19:15:41Z · sha 56fa2a7 · host m3
- datasets /Users/kat/git/riir-reflex/.raw/datasets · posture modelless_keep_corpus · ≤0 case(s)/suite (0 = full slice)
- seed 106364929 · per-suite base = seed ^ FNV-1a(suite) · seeds base+0/+1/+2
- laya device: not probed
- spec: riir-train .issues/657 R8 — label-STRING bijection probe (options+gold follow σ). NOT Bench 119 (formatting-flip): Bench 119's axis is formatting churn of the state text; this axis is the label identity itself.
- rename: Sattolo derangement over the suite's option-key alphabet (every label moves); option ORDER preserved, criteria keys replaced in place, descriptions intact; gold rides by position (idx unchanged, string renamed through σ); per-suite base = seed ^ FNV-1a(suite), seeds base+0/+1/+2
- corpus: keep_corpus control: questions + engine domains renamed, corpus docs and centroids UNRENAMED — renamed keys miss every by-name domain, the route terms disarm, the question runs the drafter-only scorer (structural, DISCLOSED — never a gate)

| suite | lane | model | seed# | n | acc@id | acc@σ | Δ | 95% CI | flips→wrong/→right |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 0 | 400 | 0.4050 | 0.2175 | -0.1875 | [-0.2627, -0.1123] | 162/87 |
| ag_news | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 1 | 400 | 0.4050 | 0.2000 | -0.2050 | [-0.2786, -0.1314] | 162/80 |
| ag_news | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 2 | 400 | 0.4050 | 0.2350 | -0.1700 | [-0.2467, -0.0933] | 162/94 |
| emotion | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 0 | 400 | 0.2750 | 0.1150 | -0.1600 | [-0.2192, -0.1008] | 110/46 |
| emotion | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 1 | 400 | 0.2750 | 0.2575 | -0.0175 | [-0.0891, +0.0541] | 110/103 |
| emotion | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 2 | 400 | 0.2750 | 0.2825 | +0.0075 | [-0.0658, +0.0808] | 110/113 |
| xnli_en | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 0 | 300 | 0.3400 | 0.3333 | -0.0067 | [-0.0997, +0.0863] | 102/100 |
| xnli_en | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 1 | 300 | 0.3400 | 0.3267 | -0.0133 | [-0.1059, +0.0792] | 102/98 |
| xnli_en | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 2 | 300 | 0.3400 | 0.3267 | -0.0133 | [-0.1059, +0.0792] | 102/98 |
| banking77 | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 0 | 500 | 0.3940 | 0.0080 | -0.3860 | [-0.4301, -0.3419] | 197/4 |
| banking77 | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 1 | 500 | 0.3940 | 0.0100 | -0.3840 | [-0.4284, -0.3396] | 197/5 |
| banking77 | modelless | identity corpus · renamed keys (by-name misses → drafter-only) | 2 | 500 | 0.3940 | 0.0100 | -0.3840 | [-0.4284, -0.3396] | 197/5 |

- ag_news: 400 slot(s) · seeds [240f495fe5e6eba4, 240f495fe5e6eba5, 240f495fe5e6eba6] · alphabet 4 · slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK

- emotion: 400 slot(s) · seeds [c16bcbb97eeaeb63, c16bcbb97eeaeb64, c16bcbb97eeaeb65] · alphabet 6 · slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK

- xnli_en: 300 slot(s) · seeds [1d2ac555f86cefa7, 1d2ac555f86cefa8, 1d2ac555f86cefa9] · alphabet 3 · slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK

- banking77: 500 slot(s) · seeds [7f344e07d843c26e, 7f344e07d843c26f, 7f344e07d843c270] · alphabet 77 · slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK
