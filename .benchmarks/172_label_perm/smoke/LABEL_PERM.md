# Label-perm probe — seeded bijective label-STRING rename (riir-train 657 R8)

- date 2026-10-30T19:13:34Z · sha 56fa2a7 · host m3
- datasets /Users/kat/git/riir-reflex/.raw/datasets · posture modelless · ≤8 case(s)/suite (0 = full slice)
- seed 106364929 · per-suite base = seed ^ FNV-1a(suite) · seeds base+0/+1/+2
- laya device: not probed
- spec: riir-train .issues/657 R8 — label-STRING bijection probe (options+gold follow σ). NOT Bench 119 (formatting-flip): Bench 119's axis is formatting churn of the state text; this axis is the label identity itself.
- rename: Sattolo derangement over the suite's option-key alphabet (every label moves); option ORDER preserved, criteria keys replaced in place, descriptions intact; gold rides by position (idx unchanged, string renamed through σ); per-suite base = seed ^ FNV-1a(suite), seeds base+0/+1/+2
- corpus: full-consistent relabel: train docs + engine domains + question keys through the SAME σ (the invariance arm; expected ≈ 0 delta — a large delta IS the finding)

| suite | lane | model | seed# | n | acc@id | acc@σ | Δ | 95% CI | flips→wrong/→right |
|---|---|---|---|---|---|---|---|---|---|
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 0 | 8 | 0.2500 | 0.1250 | -0.1250 | [-0.5691, +0.3191] | 2/1 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 1 | 8 | 0.2500 | 0.6250 | +0.3750 | [-0.2598, +1.0098] | 2/5 |
| emotion | modelless | full-consistent σ (corpus+domains+keys) | 2 | 8 | 0.2500 | 0.6250 | +0.3750 | [-0.2598, +1.0098] | 2/5 |

- emotion: 8 slot(s) · seeds [c16bcbb97eeaeb63, c16bcbb97eeaeb64, c16bcbb97eeaeb65] · alphabet 6 · slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK
