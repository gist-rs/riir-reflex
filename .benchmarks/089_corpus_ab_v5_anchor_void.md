# Bench 089 — plan 426 T5 corpus A/B: **V5 NOT ANSWERED — the instrument voided itself by its own anchor**

**Status:** MEASURED 2026-09-29, reflex @ `cfa132a` tree, harness binary `/tmp/rfx_t5/release/harness` (fingerprint `cc0838e` — a dirty-tree build from the T5 landing window; the A/B reports it, the re-run after the fix must quote a clean HEAD build). **The V5 verdict is VOID by the lane's own aliveness law** — recorded here as the instrument finding + the default-posture datum, never as a V5 answer.

## What ran (the T5 lane, complete)

- **Dump** (T1 seam): openthai teacher over the full massive train split — 11514/11514 rows, teacher acc **0.9548**, artifact `.raw/distill_teacher_openthai/massive_intent_en_teacher.bin` + blake3 sidecar (digest `49b16a89…`), `DISTILL.md` written. Determinism pin carried from T1.
- **Synthesis + veto**: 212,912 candidates → budget ≤2048 accepted/128 per label → **3523 teacher forwards, 2048 accepted / 1475 vetoed (54.2% acceptance)**, 14,278 s of teacher wall (~4 h at p50 4070 ms/call). Per-label E0 rumor weighting visible in the allocation (rumor-1.0 labels at the 128 cap, rumor-0 labels at floor). Artifact `massive_intent_en_synth.jsonl` (2048 rows) + sidecar, `SYNTH.md`.
- **A/B** (the V5 gate): one frozen test read, two engine builds differing only in corpus → **gold 0.6100 → synth 0.6100, paired LB95 −0.0382**, flips 17/17, determinism true, latency gold p50 102 µs / synth p50 99 µs (V6: sub-ms holds).

## Why the verdict is VOID

The A/B's own WARN: **arm A reads 0.6100 vs the published 0.7800 anchor** — "the instrument is NOT reproducing the anchor; investigate before reading the verdict." The lane's docstring law: "A drift there means the instrument is mis-built, and the verdict is void before it is read."

**Mechanism (read off the code, `src/harness/runner/corpus_ab.rs` + `src/engine.rs`):** the A/B's arm A builds `EngineConfig::default()` — whose levers are all **0.0** (`head_scale: 0.0, nb_scale: 0.0, ridge_scale: 0.0`, `src/engine.rs` defaults) — over the capped gold pool. The published massive 0.7800 is the **seat posture** (the arena run this morning printed it: `cap 48 · head 0.00 · nb 4.00 · ridge 0.00`), where the count-table lever is **cal-select-armed at nb_scale 4.0** and its tables read every pool doc UNcapped. The A/B builds a different, weaker engine than the published one — the docstring's claim ("exactly the published posture") is not what the code does. The anchor did its job.

**Why the paired LB95 at the default posture does not transfer:** at the published posture the corpus enters twice — the capped drafter pool AND the uncapped count tables; synth rows would flow into both. At the default posture only the drafter pool sees them. A wash at one operating point (+17/−17 flips) is evidence about that point only.

## The recorded datum at the default posture (NOT a V5 answer)

gold 0.6100 → +synth 0.6100; LB95 (synth−gold) −0.0382; per-label nets scatter ±2 with no direction; V6 sub-ms holds (99 µs p50). The synth artifact itself (54% teacher acceptance, E0-targeted allocation) is sound and stays recorded; only the gate read is void.

## The fix (filed, next session)

Arm A must build the SEAT posture — the per-suite cal-select levers (for massive: nb_scale 4.0) and the uncapped-table rule — not `EngineConfig::default()`. Then the anchor must read 0.7800 EXACTLY before any verdict is read. The A/B flags need to accept/consume the seat-select posture (the `--head-select/--nb-select/--ridge-select/--oc-select` path exists for the runner; the corpus_ab lane ignores it today).

## Session wrap (the owner's stop order)

This bench closes the session: T4 measured (V4 FAIL, instinct bench 023), T5 lane ran end-to-end (dump → veto → A/B) with the gate voided by its own anchor. The openthai server is DOWN (killed 10:02 after the veto completed — no further teacher forwards needed). No process of this session remains.
