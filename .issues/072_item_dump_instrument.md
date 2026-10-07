# Issue 072: `--dump-items` — measurement-only per-item outcome dump (rethink 028 T1's instrument)
**Status:** OPEN — filed 2026-10-07, landed the same day (same commit as the flag); closes when the rethink 028 bench record cites it.

**Kind:** measurement instrument — no serve-path code, no feature flag, no behavior change when off.
**Consumer:** riir-rethink Issue 028 (population-size scaling + gate-realized rung ordering POC) — the T1 leg.

## What landed

- `src/harness/item_dump.rs` — `ItemOutcome` (Serialize only; lane, model, case_id, case_idx, q_idx, kind, gold, pick, ok, conf, conf_cal, abstain_raw, abstain_cal, cause_cal). Ungated + pure: reads materialized eval vectors, never runs an engine.
- `LaneResult.items: Vec<ItemOutcome>` (`#[serde(skip)]`) — ALWAYS built in memory by both lane tails (`run_modelless`, `assemble_laya_lane_result` — the shared tail of ALL laya-family lanes: riir laya, laya-python, clm, gliner, bekko, agentjev, clef, openthai), NEVER serialized into `results.json` (byte-identical with/without the flag).
- `--dump-items` CLI flag (bin) — persists `<out>/items/<suite>.jsonl`, one line per (lane, question).
- `AbstainCause::as_str()` (engine.rs) — the Issue-060 closed-taxonomy keys exposed once for the dump (and any future wire-safe naming).

## Laws

- **Join key `(case_id, q_idx)`** — `SuiteCase::id` is stable per suite build; lanes serving trimmed prefixes / bucket-skipped subsets still join against the full-suite modelless read. `case_idx` is debug-only, NEVER a join key.
- **laya/oracle lanes cannot abstain** (Research-562 flaw) — their records carry `None` gate fields, never a fake false.
- **modelless records carry both postures**: raw gate (`abstain_raw`) and the deployed calibrated gate (`abstain_cal` + `cause_cal` — the ESC lane's escalation trigger).
- **paw lanes are NOT dumped** (own result shape) — a disclosed absence, named by the per-suite writer line.

## Verification

- `cargo clippy --all-targets` clean; `--features laya-riir-metal` clean (both postures compile the touched code).
- Cold-run gate: one suite × `--dump-items --skip-laya` → JSONL written, line count = n_questions, keys match the struct. (Recorded in the rethink 028 bench record when the POC lands.)

## Close condition

rethink 028's bench record cites this issue + the dump files — then fold this file into HISTORY.md per the noise-reduction rule.
