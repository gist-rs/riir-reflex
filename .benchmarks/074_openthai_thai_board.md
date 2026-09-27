# Bench 074 — the OpenThai board: T2.6 EN cross-check + T3.3 Thai probe lanes (Plan 003)

**Status:** COMPLETE 2026-09-28 — T2.6 + T3.3 measured; T3.4 (site republish)
belongs to the reflex-site session. Plan 003's record filename guessed `041`;
the number was taken — this is `074` (numbering discipline: never recycle).

**Lane (the agentjev/clm family law): their stack serves, our Rust measures.**
Server: `iapp-technology/openthai-systemone` @ `5d04bcca0c58bd10e7dac2d3d369d8f760bea6cf`
(Apache-2.0), FastAPI `uvicorn openthai_systemone.server:app` on 127.0.0.1:8000,
weights HF `iapp/OpenThai-SystemOne` (Qwen3.5-0.8B text tower + 256-slot head),
MPS device (fp32), `permutations=1` pinned (the T2.3 determinism law —
`decide_raw` ×2 byte-compare green on every suite). Server env: `.raw/openthai-systemone/.venv`
(uv, python 3.12, torch+transformers 5.17); the `.raw/` clone is research
provenance — removed after the record (the global rule).

## PROVENANCE (the Issue-021 box-state law)

- `scripts/bench_preflight.sh` before the runs:
  **`PROVENANCE: power=AC Power load=3.84 swap=1866.19M canary=137.1us/best5 powermode=2(high)`**
- The board run's own header recorded start load 6.95 (a sibling job was on
  the box at launch; it finished mid-run — end load 3.00) and marks LATENCY
  NOT QUOTABLE at the start state. Read the latencies as
  order-of-magnitude only; the ACCURACY/ECE cells are load-independent.
- Reflex tree at `8569e23` (the T3.1/T3.2 commit) + the noul-wire fix below.

## The one lane defect the board caught — `noul` response wire (fixed in the same unit)

The lane's warmup handshake FAILED on first contact: `answer has no
probabilities`. Root cause, pinned from their source (`types.py` @
`5d04bcca`): `NoulAnswer = {type: "noul", noul: float}` — the p(yes) number
rides a **`noul` field**, no `probabilities` dict and **no `confidence`**
(the research note's `{…, probabilities, confidence, …}` shape is the
REQUEST-side view, not the response). The live noul answer reads
`{"type":"noul","noul":0.0258}`. Fix: the parser reads the `noul` field as
primary (conf falls back to the top side's probability), keeps the
bare-number/one-element `probabilities` reading as documented tolerance;
choice/score require their `probabilities` dict + `confidence` per their
`ChoiceAnswer`/`ScoreAnswer`. 11 lane tests green (the measured-wire primary
+ fallback + loud-error arms).

## T2.6 — EN cross-check (their published card vs our measurement)

| suite | openthai (ours) | their card | readout-ECE | p50 (order-of-mag) |
|---|---|---|---|---|
| xnli_en (300 stratified) | **0.8967** | 89.0 | 0.0447 | 106 ms |
| massive_intent_en (300 stratified) | **0.9200** | 88.3 | 0.0458 | 1630 ms |

- xnli: agreement to 0.3 pt — the lane measures their stack faithfully.
- massive: +3.7 pt above the card, with provenance differences recorded
  (our stratified 300-row slice vs their en-US subset protocol; their
  renormalized probabilities; our instructions). A slice-protocol delta,
  not a lane defect — the same direction as the massive S2 sampling lesson.
- massive's p50 is 15× xnli's: 59 options vs 3 — the option-count scaling
  of their one-forward-per-question design.

## T3.3 — the Thai board (first Thai cells in the arena)

| suite | modelless | laya-multilingual | openthai | chance | their card |
|---|---|---|---|---|---|
| thai_wisesight (400) | 0.1225 | 0.4075 | **0.4750** | 0.25 | 51.6 / ECE 0.353 |
| thai_sib200 (204) | 0.1422 | 0.7843 | **0.8382** | ≈0.143 | — |

- **wisesight**: our openthai read lands 4.1 pt under their card with ECE
  0.3612 vs their published 0.353 — both accuracy AND calibration near the
  card; the delta is the slice protocol (our stratified 400 vs their full
  eval). Hardest suite on the board for EVERY lane (best 0.475).
- **sib200**: the specialist's clean win — openthai 0.8382 vs
  laya-multilingual 0.7843 (+5.4 pt) at ~11× the latency (110 ms vs 10 ms
  p50, unquotable precision — see PROVENANCE).
- **modelless**: at/below chance on both — the PINNED empty-bag law
  (`.docs/02_protocols/thai_posture.md`): pure-Thai clauses empty before
  hashing → zero-vector embed → the distance gate abstains by construction.
  The lane answers honestly at exactly the Phase-1 pin's prediction; real
  modelless Thai capability stays behind Plan 003 T4.2 (deferred with its
  trigger), never a silent guess.
- **laya-multilingual**: 0.4075 / 0.7843 — the one checkpoint that answers
  Thai script (the frozen `ml-thai` G5 row). `laya_checkpoints_for` now
  seats multilingual for `thai_*` suites (english would measure the
  collapse, not the capability).

## Determinism

The openthai lane's pin (`permutations=1`, `decide_raw` ×2 byte-compare) is
green on all four suite-runs — the board's numbers publish under the plan's
law. Modelless bit-identity green (the volatile wall-clock fields only).
Laya observed-repeat check ✓ (reported, not claimed).

## Artifacts

- `074_openthai_thai_board/TABLES.md` + `results.json` — the Thai board
  (BLAKE3 `b1c8939621a7f6c687607a81057f9da2fe1477b8f7753e84456c62062078dcb3`)
- `074_openthai_thai_board/en_crosscheck/TABLES.md` + `results.json` — T2.6
  (BLAKE3 `6562d831043829f3a10c160fdefbf1ce57dcccb2d82ccfc466841df1f30a01f5`)

## Verdicts

- T2.6: **MEASURED, agreement recorded** (xnli 0.3 pt; massive +3.7 pt with
  the provenance caveat). No promotion claim — comparison lane, report-only.
- T3.3: **MEASURED.** The specialist leads Thai (0.475 / 0.838); laya
  multilingual is the best OUR stack answers (0.408 / 0.784); modelless
  abstains by the pinned law. Plan 003 stays research-sake: no engine work
  re-opens on these numbers (T4.2's trigger — an owner product call for
  Thai-capable modelless decisions — did not fire).
- T3.4 (site republish): reflex-site session's half, same standing as 045
  T3's publish.
