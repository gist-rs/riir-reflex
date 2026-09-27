# Thai posture — how every lane treats a Thai input (pinned)

> **Purpose:** the repo-visible form of the research-003 posture table
> (`.research/003_openthai_systemone_thai_lane.md`): what each lane does
> when the state or question is Thai, and which behavior is CONTRACT
> (test-pinned) versus measured-once. Plan 003 Phase 1 landed the pins;
> this page records their verdicts.

| lane | Thai input behavior | standing |
|---|---|---|
| laya english / typed-decisions | script-detected → collapse conf 0.0002 → abstain | frozen G5 row `ml-thai-collapse` (`tests/fixtures/laya_parity_v1.jsonl`, G-ISO-3 — never edited) |
| laya multilingual | **answers** (the same Thai ticket the EN checkpoints collapse on) | frozen G5 row `ml-thai` |
| **modelless** | **zero-vector embed → distance-gate abstain** | **PINNED** — `tests/thai_posture_pins.rs` (Plan 003 T1.1/T1.2) |
| openthai comparison lane | their specialist answers | Plan 003 Phase 2 (not yet landed) |

## The modelless mechanism (measured at the pin's landing)

The research note guessed "clause-unit hash bag → distance-gate abstain".
The pin measured the real shape, and it is simpler:

1. `src/embed.rs` splits on **ASCII whitespace**, then `token()` trims
   **non-alphanumeric-ASCII edges**. A pure-Thai clause is non-ASCII end to
   end, so every clause is emptied **before** hashing — zero tokens,
   zero hash buckets.
2. The state embeds to the **zero vector** — the designed no-signal shape
   (finite, never NaN; a zero vector passes through un-normalized per the
   `distance_abstain::unit` law). "Non-ASCII bytes pass through" is true of
   `fnv1a_word` but moot: `token()` empties the token first.
3. The engine abstains **by the zero vector, by construction**:
   `max_similarity(0) = 0` → gate confidence `sigmoid(8·(0−0.35)) ≈ 0.057`
   < the 0.5 distance threshold. The routing argmax ties to the lowest
   domain deterministically; the answer is byte-identical across repeat
   and fresh-engine decides.

So the safe degradation is the **empty-bag law**, not a clause-hash
distance read: any input whose tokens are all non-ASCII-alphanumeric
(Thai, Chinese, Japanese, Korean, Arabic — any script outside ASCII) gets
the same deterministic abstain. That is broader than Thai and weaker than
a real language gate — a **mixed** EN+Thai state tokenizes its ASCII words
normally (the English half drives the bag), and the engine's Thai
capability is genuinely absent, not collapsed. Both facts are the pinned
posture; changing either is a Plan-003-T1.1 FINDING (file it, never tune
the gate to keep the pin green).

## The pins (tests only, zero runtime code — the Phase-1 contract)

`tests/thai_posture_pins.rs` (target `thai_posture_pins`, required-features
`modelless`, file-level `#![cfg]` — the repo-birth pair):

- `modelless_thai_posture_is_deterministic_and_abstains` — the fixture row
  read verbatim from the frozen G5 fixture; zero-vector assertion;
  deterministic byte-identical answers; the abstain.
- `serve_edge_thai_degrades_safely` — `/decide` over a real loopback
  socket (the serve_cors pattern): 200, a wire-valid `DecisionResponse`,
  finite confidences, no 5xx, abstain-forward.

Non-contamination gates G-ISO-1..4 hold: no runtime code changed, no suite
list change, fixtures untouched, no new packages.

## The probe suites (Plan 003 T3.1/T3.2 — opt-in dataset lanes)

Two Thai probe suites joined the harness as **named-only** registry members
(G-ISO-2: they run ONLY under `--suites thai_*`, never a default-run
member; no serve-path or runtime-code surface — pure harness builders):

| suite | dataset | shape | options |
|---|---|---|---|
| `thai_wisesight` | `pythainlp/wisesight_sentiment` (config `wisesight_sentiment`) | 4-class sentiment choice; test cap 400 / train 4000 | fixed ClassLabel order `[pos, neu, neg, q]` |
| `thai_sib200` | `Davlan/sib200` (config `tha_Thai`) | 7-way topic choice; test + train whole-set (204 / 701) | sorted unique `category` strings (banking77 law) |

Data provenance, digests, and label distributions:
`.docs/02_protocols/dataset_manifest.md` §9–§10 (fetched 2026-09-28 via
`SUITES=thai_wisesight,thai_sib200 scripts/fetch_datasets.sh`). wisesight
is the OpenThai board's WEAKEST published set (51.6 / ECE 0.353) — chosen
for honesty, not cherry-picking.

**First modelless readings on the new suites (smoke, 2026-09-28, M3 · AC ·
release; NOT the T3.3 board — the openthai and laya-multilingual columns
need their servers/weights in a board run):**
`thai_wisesight` acc 0.1225 (chance 0.25) · `thai_sib200` acc 0.1422
(chance ≈ 0.143). Both consistent with the empty-bag law above: a pure-Thai
state embeds to the zero vector → the distance gate abstains → forced
accuracy sits at/below chance. The lane answers at parity-with-chance
exactly as the Phase-1 pin predicts; real Thai capability on the modelless
side stays behind Plan 003 T4.2 (segment → space-join → the EXISTING
embedder), deferred with its trigger. Determinism: two runs byte-identical
ex-meta (only the volatile wall-clock latency fields differ).

Board status: T2.6 (EN cross-check) and T3.3 (the full Thai board:
openthai vs laya-multilingual vs modelless) wait on THEIR service live
(`uvicorn openthai_systemone.server:app` + the HF weights env) — the code
lane is landed and stub-verified (T2.1–T2.5).
