# Bench 141 — 623 T1 teacher capture: the GATE-RED read, the KeyMap root cause, and the v2 re-capture

**Status:** RECORD (10-11 arc complete; v2 re-capture IN FLIGHT at write time — readout instructions in §5) · date 2026-10-14 · host 4090-windows (shikuwa) · lane: riir-train Issue 623 T1 (the clef-flash + pplx-decider teacher captures over OUR frozen splits).

## 1. What ran (the detached watcher `E:\clef\watch_623_t1.py`, 2026-10-11)

Both teacher captures over `.raw/datasets_t20k` train rows (the no-cheat law, Issue 038), host-stamped `4090-windows`, harness sha `9e517d2`:

- **clef-flash int8** (rig `E:\clef\serve_clef.py` @ 127.0.0.1:8791, bnb int8 ≈ 11.5 GB) → `.raw/distill_teacher_clef` + det ×2 (limit 200):
- **pplx-decider v1.1 NF4** (rig `E:\pplx\serve_nf4.cmd` @ 127.0.0.1:8793, ≈ 13.5 GB) → `.raw/distill_teacher_pplx` + det ×2:

| teacher | suite | rows | classes | teacher acc | p50 ms | seconds | blake3 (16) |
|---|---|---|---|---|---|---|---|
| clef int8 | sst5 | 8544 | 5 | 0.5988 | 1134 | 9861.5 | f583e47aaa322a1d |
| clef int8 | massive_intent_en | 11514 | 60 | **0.9685** | 1146 | 13932.5 | 5538b60f3a4194ec |
| clef int8 | banking77 | 9993 | 77 | 0.9492 | 1649 | 16510.5 | 3c2eafb111732385 |
| pplx NF4 | prompt_injections | 546 | 2 | **0.9817** | 771 | 431.6 | 6811dc8613b3866a |
| pplx NF4 | xnli_en | — | — | **ABSENT** (gold pin, §3) | | | |
| pplx NF4 | emotion | — | — | **ABSENT** (gold pin, §3) | | | |

Determinism: det pairs BYTE-IDENTICAL per suite dumped (clef ×3, pplx prompt_injections; the watcher's det gate, `E:\clef/capture_*_det*.log`).

## 2. The watcher's capture-quality verdict: GATE RED (`E:\clef\capture623_gates.txt`)

Gates = the selecting reads (bench-132 class), law: "a capture below its selecting read is a RED, not a usable teacher":

- clef massive_intent_en 0.9685 ≥ 0.93 → **PASS**
- clef banking77 0.9492 vs 0.95 → **RED** (−0.08 pt ≈ 8 rows of 9993)
- clef sst5 0.5988 vs 0.60 → **RED** (−0.12 pt ≈ 10 rows of 8544)
- pplx prompt_injections 0.9817 ≥ 0.92 → **PASS**
- pplx xnli_en, emotion → **RED (missing from dump)**

## 3. Root cause of the pplx absences — the KeyMap law (FIXED, reflex `724b5df`, 2026-10-12)

The distill seam's `KeyMap::for_suite` mapped `emotion`/`xnli_en`/`ag_news` to `FixedInt` (presented position i ↔ int label i), but those suites' `train_row_label` rule produces OPTION-KEY NAME labels ("sadness", "neutral", …). The per-row gold pin caught it on row 0, loud:

```
emotion:0:  gold pin failed: doc label "sadness"  != presented gold position 0
xnli_en:0:  gold pin failed: doc label "neutral"  != presented gold position 1
```

The fail-closed pin did exactly its job — no mislabeled teacher rows entered any artifact. Fix: reflex `724b5df` (2026-10-12 08:09) — `ag_news`/`emotion`/`xnli_en` → `KeyMap::Name`; `sst5` stays `FixedInt` (proven by the 8544-row clef capture above); `prompt_injections` stays `FixedInt` (proven PASS). The 10-11 pplx dump's prompt_injections half is UNAFFECTED (correct keymap all along).

## 4. The clef hair-width REDs — adjudication (posture delta, not a rig fault)

The 4090 rig is **bnb int8**; the selecting reads came from the **m3 MLX 4-bit** posture (bench 132: banking77 0.9540, sst5 0.6033). Both REDs are the quant posture reading ~0.5 pt below the selecting posture — det pairs byte-identical, so the rig is deterministic and healthy; the int8 surface is measurably slightly weaker on exactly these two suites. Per the gate law these two slots are RED at this posture. Disposition:

- **massive_intent_en slot USABLE** (0.9685 ≥ 0.93, int8).
- **banking77 + sst5 slots → the sanctioned m3 fallback** (the MLX 4-bit bench-132 posture, `623` Owner-gates: "m3 fallback is the clef lane's own host") — an M3-lane re-capture (~680 ms/row ≈ 3.5 h for both slots). Mind the M3's in-flight B2 tranche window (reflex Plan 012) before scheduling.
- A 4090 bnb-NF4 clef rig would be a THIRD posture needing its own spot-check — not sanctioned; don't.

## 5. The v2 re-capture (IN FLIGHT, launched 2026-10-14 04:24 by the 4090 idle lane)

Post-fix pplx re-capture over all three slots into ONE coherent artifact (the 10-11 dir preserved untouched):

- Script `E:\clef\rerun_pplx_v2.py` (this bench's companion, watcher shapes verbatim; canonical copy also at `riir-reflex/.raw/rerun_pplx_v2.py`, gitignored).
- Out dirs: `.raw/distill_teacher_pplx_v2` (+ `_det1`/`_det2`, limit 200); gates: xnli_en ≥ 0.95, emotion ≥ 0.64, prompt_injections ≥ 0.92; det pair byte gate; PLUS an advisory **cross-boot** determinism check (v2 det payloads vs the 10-11 boot's prompt_injections payloads — same quant posture, different process lifetime).
- ETA ≈ 8 h (20000 xnli + 16000 emotion + 546 pi rows at p50 ~771 ms) + rig boot.
- **Readout:** `E:\clef\rerun_pplx_v2_gates.txt` (+ `.done`/`.FAILED` markers; run log `E:\clef\rerun_pplx_v2.log`, per-run `E:\clef\capture_pplxv2_*.log`). A GREEN closes the pplx half of T1's capture-quality gates; a RED on xnli/emotion acc (vs the bench-132 reads that selected pplx for those slots) is a capture-quality finding, not a re-run loop.

## 6. What this does NOT change

- T2 student retrains stay gated (Research 464 §3 owner gate + full capture-quality greens both teachers).
- 629 (snap-beat teacher lane) reuses these captures per its T2; its GPU queue position stays BEHIND 623 T1 (owner call 2026-10-10).
- The 10-11 clef det/int8 artifacts on disk remain the record for that posture (this bench cites them; nothing overwritten).

Refs: riir-train `.issues/623` (the lane; the verdict section lands there same-day), reflex `724b5df` (KeyMap fix), `adc6d48` (the seam), bench 132 (selecting reads), `watch_623_t1.py` (the 10-11 pipeline), Issue 038 (no-cheat law), reflex Research 005/010 (teacher recipes).
