# Teacher router — sigmoid-gated per-suite blend over captured columns (riir-train Issue 629 T2.5)

- date 2026-10-24T02:21:27Z · sha 5ec1d63 · host m3
- inputs .raw/distill_teacher, .raw/distill_teacher_jeva
- temperature 0.01 (sigmoid, never softmax; g = sigmoid((acc_a-acc_b)/T))
- law: train-row captures only — the routed artifact's verdict is T4's single frozen read; nothing here selects on it

| suite | sources (acc) | gate g | routed acc | cv (loo) | mode | classes | routed blake3 |
|---|---|---|---|---|---|---|---|
| ag_news | unknown 0.9769 ⊕ jeva 0.7331 | 1.0000 | 0.9769 | 0.9769 | blend | 4 | 9580f37322d149e6 |
| banking77 | unknown 0.4312 ⊕ jeva 0.9274 | 0.0000 | 0.9274 | 0.9274 | blend | 77 | 7871da02574e0763 |
| massive_intent_en | unknown 0.7341 ⊕ jeva 0.9480 | 0.0000 | 0.9480 | 0.9480 | blend | 60 | 87ed93982312ecea |
| prompt_injections | jeva 0.8443 | — | — | — | passthrough | 2 | 3c94f3848c1ba7b2 |
| xnli_en | unknown 0.9654 ⊕ jeva 0.8508 | 1.0000 | 0.9654 | 0.9654 | blend | 3 | ff2aae7523067817 |

## Skipped

- emotion: row counts differ (16000 vs 15999) — different datasets posture or survivor predicate; refusing the join
- sst5: row counts differ (8544 vs 8533) — different datasets posture or survivor predicate; refusing the join
