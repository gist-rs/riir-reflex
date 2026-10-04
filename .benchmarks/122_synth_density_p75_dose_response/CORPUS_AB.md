# corpus A/B (Plan 426 T5 — the V5 gate)

- cap 48/label · synth extra-cap 128 · datasets .raw/datasets_t20k · read @ 16eb594
- one frozen test read; two engine builds differing ONLY in corpus content; forced-pick hard accuracy; paired LB95 (synth − gold) > 0 is the V5 gate; both arms reported (C3)

## massive_intent_en — gold 0.7800 → synth 0.8033 · paired LB95 +0.0062

- arm A reproduces the published 0.78 exactly — the instrument is alive (the seat posture, selected knobs + fitted gate)
- V5 PASS — synthesis lifts the modelless row (paired LB95 > 0)
- corpus: gold 11314 doc(s) → synth 13294 doc(s) (1980 in scope of 1980 artifact rows)
- latency: gold p50 112 µs / p99 135 µs · synth p50 113 µs / p99 200 µs (V6: the lane stays sub-ms)
- flips: 7 synth-only wins, 0 gold-only wins · determinism true

## Issue-064 gates — abstention PASS · OOD transfer-ok

- abstention-entropy: KL(gold‖synth) 0.0000 · reverse 0.0000 · abstain 0.9367 → 0.8800
- OOD word-dropout ladder:

| p | acc gold | acc synth | Δ | paired LB95 |
|---|---|---|---|---|
| 0.10 | 0.7567 | 0.7867 | +0.0300 | +0.0107 |
| 0.20 (gate) | 0.7567 | 0.7933 | +0.0367 | +0.0117 |
| 0.30 | 0.6767 | 0.6967 | +0.0200 | +0.0041 |

- retention @ gate 1. · rule: deterministic seeded word-dropout ladder p∈{0.10,0.20,0.30}; gate rung p=0.20; a clean V5 PASS whose gate-rung paired LB95 < 0 reads ECHO (recorded NEGATIVE, the lane dies); a non-negative gate-rung LB95 reads transfer-ok; retention = corrupted Δ / clean Δ
- KL(gold‖synth) ≤ 0.05 nats over the 10-bin normalized answer-entropy histogram (Laplace-smoothed) — the issue's own wording gates the KL alone; the abstain-rate drift is a DISCLOSURE column, not a gate leg (first real reading, 2026-10-04: corpus growth raises answer confidence BY DESIGN — a 6pp rate drop at a 0.93 baseline with KL 0.0000 is the system working, not a collapse; an absolute-rate leg would mis-fire at high-abstain baselines and under-bind at low ones)
- no echo verdict fired

| label | gained | lost | net |
|---|---|---|---|
| audio_volume_mute | 2 | 0 | +2 |
| audio_volume_down | 1 | 0 | +1 |
| audio_volume_other | 1 | 0 | +1 |
| general_greet | 1 | 0 | +1 |
| iot_hue_lighton | 1 | 0 | +1 |
| music_likeness | 1 | 0 | +1 |

synth-only wins: massive_intent_en:7, massive_intent_en:12, massive_intent_en:120, massive_intent_en:176, massive_intent_en:233, massive_intent_en:256, massive_intent_en:257

