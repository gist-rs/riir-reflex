# Bench 111 — Plan 435 T5 tables: the laya round-1 fine-tuned checkpoint measured on OUR board

**Status:** RECORD — produced by riir-train Plan 435 / Bench 620 (Issue 607 owner GO,
2026-10-02); the round's narrative + gates live THERE. Reflex-side number allocated
for the table dirs (this repo's counter; no reflex lane protocol changed).

- `111_plan435_typed_round1_tables/` — the typed_decisions suite, CUDA laya lane:
  **typed 0.6910** (round1 mounted as `typed`, `LAYA_ALLOW_UNPINNED_WEIGHTS=1`)
  vs the published **0.7445** (Bench 082) — **−5.35 pt on the serving cell**;
  english 0.3575 / multilingual 0.3490 reproduce Bench 082 EXACTLY (same split,
  same posture — the delta is real, not drift). Determinism ✓ all lanes.
- `111_plan435_s1mb_base_tables/` + `111_plan435_s1mb_round1_tables/` — the
  our-metric s1mb suites (Plan 010 converter). ⚠ The harness's s1mb suites route
  to the **english** checkpoint (`laya_checkpoints_for`), which the round does not
  touch — both runs read byte-identical noul 0.6964 / score 0.2590 (and choice is
  absent: a 151-option clinc case exceeds the Rust lane's head budget — the same
  wall both runs). The round1 s1mb evidence is the S1MB-protocol read in
  riir-train Bench 620 (task avg 16.94 → 51.18), not these cells.
