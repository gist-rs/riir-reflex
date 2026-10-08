# Issue 078 — `--d1` comparison lane (LiquidAI d1-3B)

**Status:** CLOSED 2026-10-08 — T1–T5 complete, T6 deferred per task list. Measured:
`.benchmarks/129_d1_lane.md` (run1 + perm). Lane: `src/lanes/d1.rs` + `--d1` + the
windows-split reference server (`.raw/d1_server.py`, port 8078). T4 mounted on issue 077's
probe; the lane needed no new eval machinery — same TypeSafe wire, pure transport +
posture.

## Context

LiquidAI open-weighted the d1 decision-model family (blog 2026-10-07): d1-3B (3.1B,
Decision Index v0.2.1 **48.57**, their table) and d1-omni-600M (587M bidirectional encoder
— the laya scale class). d1 speaks the TypeSafe-Jev wire natively
(`POST /decisions/v1/systemone`, `state` + `questions{noul|choice|score}`, zero output
tokens, multi-question one pass, `system_one_batch` packing). Their card publishes cells
on MASSIVE intent (**87.3**) and XNLI (**85.0**) — two of OUR harness suites — so the lane
gives a direct same-benchmark third-party check. Their text answers carry per-type
temperatures (config.json) — a calibration claim our G1 axis can settle. Web sweep
(2026-10-08): no independent third-party coverage yet — early third-party read.

License law: **HF tag `license: other`, `license_name: lfm1.0`** — the blog's "without
restrictions" is marketing. Measurement-only posture is safe; read `LICENSE` before ANY
local weight redistribution or product use (the Drex CC BY-NC lesson).

## Tasks

- [x] **T1 — the lane** (`src/lanes/d1.rs`, mirror of `drex.rs`/`agentjev.rs`; same
      TypeSafe wire): `--d1` harness flag + `SuiteResult.d1` + `RunMeta.d1_lane` posture;
      std-only `http_mini` transport; `D1_SERVE_URL` (default loopback, port named in the
      runbook). Reference server posture: their in-repo Python (`modeling_d1.D1Model`,
      `trust_remote_code=True`, transformers ≥5.14) behind a plaintext loopback listener —
      the Drex-lane shape (`DREX_SERVE_URL=http://127.0.0.1:8000` precedent). Hosted
      `d1:free` is HTTPS → loopback TLS-terminating forwarder (operator-run, disclosed hop)
      or owner-gated `ureq` (BOUNDARY change) — the Research-005 Clef transport finding
      verbatim; both owner-gated, neither the default. **Done:** `.raw/d1_server.py` (port
      8078); the hosted postures stay owner-gated/closed.
- [x] **T2 — lane laws:** loud refusal without the server (never half-run); fixed-throwaway
      warmup; determinism rerun probe from day one (det ✗ is a row, not a blocker); the
      laya trim cap; **dtype-posture column mandatory** (vendor table: fp16 = 0 top-answer
      flips vs fp32; bf16 flips 0.8% text / 1.7% audio — prefer fp16/fp32 serving, record
      the SERVED dtype beside every cell). **Done:** all landed; served dtype = fp16,
      carried in `/health` + boot log + `RunMeta.d1_lane`. NEW LAW from the bring-up: the
      one-pass tree needs flash kernels no Windows torch wheel compiles (verified through
      2.11+cu128) — the windows-split posture is disclosed beside every cell.
- [x] **T3 — owned cells:** per-primitive gold-label accuracy on **typed_decisions +
      massive_intent_en + xnli_en** (d1 published 87.3 MASSIVE / 85.0 XNLI on the same
      public datasets — direct check; our splits + caps apply, crosswalk discipline below);
      **ECE/Brier of the per-option `probabilities` AND of their `confidence` field,
      reported separately** (their per-type-temperature calibration claim = the G1 test —
      Research-005 risk-6 posture; Report-the-Floor applies to any calibrated claim);
      round-trip latency with `scripts/bench_preflight.sh` posture quoted (hosted vs
      on-card never compared). **Done:** bench 129. The two ECE axes COINCIDE for d1 (their
      `confidence` IS the max probability) — disclosed, not duplicated; the shared
      `assemble_drex_conf_readout` path carries the per-kind cells + floor. The raw-softmax
      posture reads ECE 0.037–0.051 and beats its floor everywhere — the calibration claim
      SURVIVES at the open-weights posture (no temperature artifact shipped).
- [x] **T4 — mount the option-permutation probe (issue 077):** d1's own recipe names option
      shuffling — their model is the probe's most interesting subject. **Done:** bench 129
      perm run — d1 RED on 3 of 6 suites (typed 7.70 pt / emotion 4.92 / banking77 5.58
      medians), PASS on ag_news/xnli/massive; canary flips on every suite (the probe
      provably fires).
- [x] **T5 — crosswalk + edition discipline:** our-splits cells are CROSSWALK, never board
      claims (Research-005 honesty law — chance levels computed from the label distribution
      actually evaluated); every published cell carries edition+split+hardware/serving
      posture columns (Decision Index v0.2.1 public split ≠ v0.3 private vision split —
      Research-008 edition law); d1's own table is cited "their table" (subset — Jev
      57.91 / Drex 1.5 58.28 rows absent). **Done:** bench 129 carries the columns;
      license terms now READ (LFM Open License v1.0 — $10M-revenue Threshold on commercial
      use; measurement-only research licensed; not NC, not Apache/MIT).
- [x] **T6 — [-] deferred:** SQuAD2.0-as-noul external abstention-anchor suite; the wider
      seven-benchmark panel (SQuAD2.0/CivilComments/BoolQ/PubMedQA/PAWS-X) as harness
      suites; hosted-`d1:free` lane posture if an owner ever arms it.

## Board context cells to land beside

laya-typed **0.7445** · AgentJev **0.7715** det✗ · rethink typed **0.7550** · the drex
lane's closed verdicts (det ✓ at three postures, not-calibrated — Benches 125/126) are the
calibration-verdict template for d1's.
