# Issue 012 — the laya-python reference lane on the bench tables

**Status:** CLOSED at `67470be` — landed with the harness lane + the site
regeneration; the reference answers byte-identically to the riir lane on
every suite and checkpoint (accuracy + macro-F1; ECE/Brier differ only at
the reference's own 4-dp rounding). Durable record: `HISTORY.md`.

## Ask

The bench page (reflex.gist.rs/bench/) shows `laya-riir` only. The owner
asked for (1) the ORIGINAL python/torch reference lane alongside it, (2)
display spellings `laya (rust)` / `laya (python)`, (3) `modelless · none`,
(4) the table fitting without the last column cut off.

## The lane

`scripts/laya_python_lane.py` — the pinned reference checkout
(`.raw/laya`) driven as a JSONL subprocess oracle: the harness spawns ONE
process per (suite, checkpoint), sends the SAME cases it sends
`RiirAgent`, and reads back the reference's own rounded-4 probabilities.
Weights come from the SAME flat cache the riir lane pins
(small files COPIED into a throwaway hub-shaped shim — the reference's
loader rewrites `tokenizer_config.json` in place; the pinned cache must
not move — `model.safetensors` symlinked).

Measurement-only, opt-in (`--laya-python`): the owner's "no Python
anywhere" directive governs the SHIPPED binary (the product lane stays
the riir backend), not the bench reference — the
`probe_orig_laya_latency.py` precedent.

## Contract guards

- `assemble_laya_lane_result` — ONE metrics tail for both backends; the
  lanes differ only in how answers are produced, never in how they are
  scored.
- `parse_python_answer` — the answer mapping mirrors the riir lane's
  (choice pick follows the reference's `choice` KEY, not a rounded-probs
  argmax; noul thresholds on p[1]); 4 known-answer unit tests in
  `tests/harness_units.rs`.
- Determinism: the same observed repeat check as the riir lane (first 10
  cases re-sent; raw response lines compared).
- Meta disclosures: `laya_device` (the riir lane's posture) +
  `laya_python_lane` (on/off + caveats: rounded-4 probs, round-trip
  latency includes IPC).

## First measurement (smoke, prompt_injections, m3, metal posture)

| lane | acc | ECE(maxp) | p50 |
|---|---|---|---|
| laya-riir (metal) | 0.6983 | 0.2620 | 75.0 ms |
| laya-python (mps) | 0.6983 | 0.2620 | 28.0 ms |

IDENTICAL accuracy/ECE/F1 — the port is parity-true outside the fixture
corpus too. Latency: torch MPS row-batching leads our per-op dispatch
~2.7× at this suite; the recorded optimization ladder (Bench 001
addenda) measures against that.
