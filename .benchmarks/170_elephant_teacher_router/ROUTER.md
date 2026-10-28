# Teacher router — sigmoid-gated per-suite blend over captured columns (riir-train Issue 629 T2.5)

- date 2026-10-28T04:51:42Z · sha 4c1d21e · host amo
- inputs .raw/distill_router_elephant, .raw/distill_teacher_elephant
- temperature 0.01 (sigmoid, never softmax; g = sigmoid((acc_a-acc_b)/T))
- law: train-row captures only — the routed artifact's verdict is T4's single frozen read; nothing here selects on it

| suite | sources (acc) | gate g | routed acc | cv (loo) | mode | classes | routed blake3 |
|---|---|---|---|---|---|---|---|
| ag_news | elephant 0.8616 | — | — | — | passthrough | 4 | e49f76ea601c84cf |
| banking77 | router 0.9492 ⊕ elephant 0.9416 | 0.6815 | 0.9526 | 0.9525 | blend | 77 | ec47b155d39fa90d |
| massive_intent_en | router 0.9681 ⊕ elephant 0.9530 | 0.8192 | 0.9687 | 0.9687 | blend | 60 | 6fdbd1f9077c81c9 |
| prompt_injections | router 0.9890 ⊕ elephant 0.8187 | 1.0000 | 0.9890 | 0.9890 | blend | 2 | 4f46079dc2eb4677 |
| xnli_en | router 0.9199 ⊕ elephant 0.8853 | 0.9694 | 0.9197 | 0.9197 | blend | 3 | 311cf6691134046c |

## Skipped

- emotion: row counts differ (16000 vs 15999) — different datasets posture or survivor predicate; refusing the join
- sst5: row counts differ (8544 vs 8533) — different datasets posture or survivor predicate; refusing the join
