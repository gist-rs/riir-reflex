# corpus A/B (Plan 426 T5 — the V5 gate)

- cap 48/label · synth extra-cap 128 · datasets .raw/datasets_t20k · read @ 2e8a023
- one frozen test read; two engine builds differing ONLY in corpus content; forced-pick hard accuracy; paired LB95 (synth − gold) > 0 is the V5 gate; both arms reported (C3)

## massive_intent_en — gold 0.7800 → synth 0.8133 · paired LB95 +0.0110

- arm A reproduces the published 0.78 exactly (the seat \
                 posture, selected knobs + fitted gate) — the instrument is alive
- V5 PASS — synthesis lifts the modelless row (paired LB95 > 0)
- corpus: gold 11314 doc(s) → synth 13362 doc(s) (2048 in scope of 2048 artifact rows)
- latency: gold p50 115 µs / p99 137 µs · synth p50 112 µs / p99 195 µs (V6: the lane stays sub-ms)
- flips: 11 synth-only wins, 1 gold-only wins · determinism true

| label | gained | lost | net |
|---|---|---|---|
| audio_volume_other | 3 | 0 | +3 |
| audio_volume_mute | 2 | 0 | +2 |
| audio_volume_down | 1 | 0 | +1 |
| general_greet | 1 | 0 | +1 |
| iot_hue_lighton | 1 | 0 | +1 |
| lists_createoradd | 1 | 0 | +1 |
| music_likeness | 1 | 0 | +1 |
| qa_maths | 1 | 0 | +1 |
| news_query | 0 | 1 | -1 |

synth-only wins: massive_intent_en:7, massive_intent_en:12, massive_intent_en:85, massive_intent_en:120, massive_intent_en:143, massive_intent_en:176, massive_intent_en:223, massive_intent_en:233, massive_intent_en:256, massive_intent_en:257, massive_intent_en:283

gold-only wins: massive_intent_en:136

