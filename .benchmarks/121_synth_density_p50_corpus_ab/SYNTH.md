# synth corpus report (Plan 426 T5)

- budget: ≤ 2048 accepted / 128 per label · span ≤ 4 · datasets .raw/datasets_t20k
- host m3 @ 69cec9e (2026-10-04T02:20:51Z)

| suite | teacher | cands | accepted | vetoed | forwards | p50 ms | s | blake3 |
|---|---|---|---|---|---|---|---|---|
| massive_intent_en | openthai/openthai:openthai-systemone | 212912 | 1859 | 662 | 2521 | 3940 | 10177 | aede3e556dd418d5… |

| suite | weighting | dedup pool/cal/dup/out |
|---|---|---|
| massive_intent_en | per-intent E0 rumor fraction (bag view, n<4, own-tables; unmeasured labels take the mean) | 65/27/4598/0 |

Per-label allocation (top 20 by accepted):

| label | weight | cands | alloc | accepted | vetoed |
|---|---|---|---|---|---|
| massive_intent_en/audio_volume_down | 1.000 | 1121 | 128 | 128 | 23 |
| massive_intent_en/iot_cleaning | 1.000 | 2203 | 128 | 128 | 10 |
| massive_intent_en/music_settings | 1.000 | 1177 | 128 | 128 | 22 |
| massive_intent_en/audio_volume_mute | 0.750 | 2368 | 103 | 103 | 38 |
| massive_intent_en/email_querycontact | 0.667 | 2130 | 92 | 92 | 52 |
| massive_intent_en/iot_hue_lighton | 0.667 | 368 | 92 | 92 | 29 |
| massive_intent_en/qa_maths | 0.667 | 1193 | 92 | 92 | 2 |
| massive_intent_en/recommendation_events | 0.667 | 3959 | 91 | 91 | 16 |
| massive_intent_en/music_query | 0.500 | 3671 | 69 | 69 | 16 |
| massive_intent_en/calendar_set | 0.333 | 8158 | 46 | 46 | 19 |
| massive_intent_en/cooking_recipe | 0.333 | 3617 | 46 | 46 | 0 |
| massive_intent_en/email_query | 0.333 | 8230 | 46 | 46 | 0 |
| massive_intent_en/general_joke | 0.333 | 1399 | 46 | 46 | 1 |
| massive_intent_en/lists_createoradd | 0.333 | 4150 | 46 | 46 | 6 |
| massive_intent_en/lists_remove | 0.333 | 3621 | 46 | 46 | 79 |
| massive_intent_en/play_audiobook | 0.333 | 2123 | 46 | 46 | 41 |
| massive_intent_en/play_game | 0.333 | 1933 | 46 | 46 | 16 |
| massive_intent_en/play_podcasts | 0.333 | 4614 | 46 | 46 | 3 |
| massive_intent_en/play_radio | 0.333 | 4956 | 46 | 46 | 27 |
| massive_intent_en/qa_definition | 0.333 | 4502 | 46 | 46 | 9 |
