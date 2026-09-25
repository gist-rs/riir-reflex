# laya_parity_v1.jsonl — the G5 parity fixture corpus (Plan 603 T1.4/T1.5)

**BLAKE3:** `f2a00354ef554f36d99509df160aaf8f35e68ac9058631d283d842ea329d9d26` — the
expected-outputs file must record this hash of the questions it was captured against.

- 36 rows / 37 questions; line 1 is a `_meta` header (protocol + gate rules, machine-readable).
- `checkpoints` names which of the three laya checkpoints the row runs on: 26 forwards per
  english-lane checkpoint, 36 for multilingual (98 total — a seconds-scale capture run).
- Coverage is deliberate (every row targets a parity hazard from
  `.docs/02_protocols/laya_reference_pin.md`): all temperature buckets incl. `choice:11+` (the shipped
  0.1006 the reference clamps to 0.5) and `score:2` (falls back to `temperature[1]`), the
  k=1 single-option forward (the #96 shape), structured/`None`/`0`/`False` criterion
  renders (Python-JSON byte format), dict/list state serialization, `[MASK]`/`<mask>`
  literal neutralization, over-48-token option texts, head-budget pressure, long-state
  truncation at BOTH windows (en 512 / ml 1024 — same rows, different checkpoint windows,
  so the truncation path itself is cross-checkpoint parity), and ten script rows
  (Thai, Hindi, Arabic, Japanese, Chinese, Russian, Greek, Armenian, Hebrew, Korean —
  the workspace-relevant Thai leads) plus the `ml-thai-collapse` row that pins the
  documented english-checkpoint collapse on non-Latin input.
- Expected outputs are **captured from the reference Python stack** (one-time, at pin
  time — the reference side of G5, not lane Python) into
  `tests/fixtures/laya_parity_expected_v1.json`, which records: this file's BLAKE3, the
  checkpoint pins from `.docs/02_protocols/laya_reference_pin.md`, per-row logits, probabilities,
  confidence, and the reference's own temperature resolution per question.
- The Rust parity test (a `[[test]]` row, `required-features = ["laya"]`, same commit as
  the port code) replays both files and asserts **top-1 agreement >= 99.9% + probability
  drift <= 1e-3 per checkpoint**. Bit-identity is NOT claimed (op order differs from
  PyTorch). A failed gate marks the lane PROVISIONAL everywhere its numbers appear.
- Determinism: inference only — no sampling anywhere in the capture; the "fixed seed"
  protocol clause is satisfied trivially (nothing draws).
