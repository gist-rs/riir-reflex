# Bench 129 — the LiquidAI d1 comparison lane (issue 078): third vendor entrant on the wire, the calibration claim SURVIVES the open weights, and the order-sensitivity the vendor's own recipe predicted

**Status:** COMPLETE — T1–T5 measured (`--d1` lane + T3 cells + T4 perm probe); T6 deferred per the issue. Commits: see HISTORY.md row.

## What ran

Issue 078's `--d1` comparison lane (`src/lanes/d1.rs`): LiquidAI's open-weights **d1-3B**
(`LiquidAI/d1-3B` @ `051bcc4`, LFM2.5-VL-3B backbone, license `other`/`lfm1.0`) answered over
their official `/decisions/v1/systemone` wire (pinned from their in-repo `api.py` /
`prompt.py` / `runner.py`), our Rust harness measuring on the same cases, splits, caps, and
metrics tail as every other lane.

- **License check (the issue's law, now discharged):** `lfm1.0` = LFM Open License v1.0 —
  commercial use conditioned under the $10M-revenue Threshold (§5); measurement-only
  research use licensed; redistribution needs license copy + notices (§4). Not Apache/MIT,
  and not an NC cage — the blog's "without restrictions" oversold it; the lane stays
  measurement-only, weights gitignored under `.raw/d1-model/`.
- **The reference serving posture:** their repo ships `modeling_d1.D1Model` (Python, `system_one`,
  `trust_remote_code=True`, transformers ≥5.14) with NO HTTP layer — our stdlib-listener
  wrapper (`.raw/d1_server.py`, the Drex-lane shape) serves their class **unmodified** on
  loopback:8078 (`D1_SERVE_URL`), fp16, calibration=None, port fresh in the lane family.
- **The WINDOWS-SPLIT posture (a finding in itself):** their one-pass multi-question tree
  (`hybrid.py`) calls the private flash-attention ops, which are **NOT COMPILED into Windows
  torch wheels** — the `USE_FLASH_ATTENTION was not enabled for build` guard verified failing
  on `torch 2.6.0+cu124` (the shared drex-env) AND a throwaway `torch 2.11.0+cu128` venv
  (deleted after; the guard is a build flag, not a version gap). The server therefore answers
  multi-question requests one question per `system_one` pass — the vendor's single-question
  reference path, code unmodified ("mathematically each row alone", their docstring). What
  differs from the fused one-pass posture: kernel batch shapes (their own caveat: kernel
  shapes move low-order bits in fp16) and the state re-encoded per question (their
  input_tokens accounting for single-question rows already counts the full prompt per row, so
  the token tally matches their reference accounting for the requests as issued). The dtype +
  calibration + posture columns ride `/health`, the boot log, and the run meta — never
  assumed.
- **Cells:** typed_decisions + massive_intent_en + xnli_en (T3's three; their card publishes
  MASSIVE 87.3 / XNLI 85.0 on the same public datasets — our splits + caps apply, so every
  cell is CROSSWALK, never a board claim). ECE/Brier of the per-option probabilities AND of
  their `confidence` field — which COINCIDE for d1 (their `answer()` emits `confidence` =
  max probability), disclosed rather than duplicated: the hard-metrics ECE(maxp) and the
  their-confidence cells measure the same axis; the readout adds the per-kind split + the
  split-half conformal floor companion. Determinism = the observed-repeat check (first 10
  cases ×2, byte-identical under the latency-tail strip). Latency = client round-trip,
  box-state stamped.
- **T4:** the issue-077 option-permutation probe mounted on d1 (their recipe names option
  shuffling as a top training lever — "their model is the probe's most interesting subject").

Run 1 + perm: `.benchmarks/129_d1_lane/{run1,perm}/`. Preflight
`PROVENANCE: power=AC scheme=high load=3.36 swap=3463MB gpu=21 %, 7182 MiB canary=skipped`
(the 7182 MiB is the d1 server's own resident model; no other compute consumer). Run box
state: start AC/high/load 1.44/7182 MiB → end AC/high/2.88/7506 MiB — latency QUOTABLE
(harness stamp, Issue 021). GPU exclusivity: the d1 server IS the GPU consumer; nothing else
compute-bearing ran beside.

## T3 cells (run1, 4090-windows, release, fp16/windows-split posture)

| suite | lane | n | acc | macro F1 | ECE(maxp) | Brier | p50 | p99 (support) | det |
|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | d1 | 2000 q | **0.6510** | 0.6280 | **0.0373** | 0.4623 | 191.0 ms | 311.0 ms (5) | ✓ |
| typed_decisions | modelless (same run) | 2000 q | 0.3345 | 0.2456 | 0.0488 | 0.6895 | 0.909 ms | 2.700 ms (5) | ✓ |
| xnli_en | d1 | 300 | **0.8167** | 0.8176 | 0.0505 | 0.2502 | 24.0 ms | 39.0 ms (4) | ✓ |
| xnli_en | modelless | 300 | 0.3400 | 0.2052 | 0.0055 | 0.6676 | 0.076 ms | 0.125 ms (4) | ✓ |
| massive_intent_en | d1 | 300 | **0.9067** | 0.9026 | 0.0397 | 0.1428 | 29.0 ms | 37.0 ms (4) | ✓ |
| massive_intent_en | modelless | 300 | 0.6100 | 0.6035 | 0.5454 | 0.9258 | 0.090 ms | 0.158 ms (4) | ✓ |

typed_decisions per-type (d1): choice 600 **0.6033** (ece 0.0599) · noul 600 **0.8050**
(ece 0.0829) · score 800 **0.5713** (ece 0.0523). Their-confidence cells (== max-prob axis,
the coincidence disclosed): choice n=600 ece 0.0599 / brier 0.2010 · score n=800 ece 0.0498
/ brier 0.2315 · split-half floor ece 0.1509 (cal 700 / test 700). xnli choice n=300 ece
0.0505 (floor 0.4205); massive choice n=300 ece 0.0397 (floor 0.3412).

### The findings

1. **The board cells.** massive **0.9067** / xnli **0.8167** on OUR splits vs their card's
   87.3 / 85.0 on theirs (≤1000 rows, their protocol) — direction consistent, population
   different (ours: 300-stratified caps, our option render, our split seed); crosswalk
   discipline (T5) forbids reading either number against the other. typed_decisions
   **0.6510** places d1 THIRD among the measured specialists — AgentJev 0.7715 (bench 039,
   gold-label) > laya-typed 0.7445 > **d1 0.6510** > modelless 0.3345 — while d1 costs
   ~0.2–0.5× laya's latency class on single-question suites (24–29 ms p50 vs laya's
   1312 ms at its published baseline posture; different box, disclosure only).
2. **The calibration claim SURVIVES the open weights.** The card says "calibrated"; the open
   weights ship NO temperature artifact (`config.json` carries none;
   `D1Model.engine` passes `calibration=None`) — and the RAW SOFTMAX posture still reads
   ECE 0.037–0.051 across all three suites, beating its own split-half conformal floor
   everywhere (0.15–0.42 at these window sizes). Whatever the hosted tier ships, the
   un-artifacted open weights are already well calibrated — the vendor's claim holds at the
   posture we could test, and the per-type temperatures (if ever shipped) can only be
   measured as a follow-up cell.
3. **Windows is a second-class platform for their serving shape** — the tree path needs
   flash kernels that no official Windows wheel carries (verified through torch 2.11+cu128).
   The split posture is the honest workaround, priced: typed's 5-question cases pay ~5×
   state re-encode (191 ms p50 vs 24–29 ms single-question suites).
4. **T4 — d1 IS order-sensitive on 3 of 6 probed suites** (K=5 orderings × ≤40 cases, the
   Bench-128 posture): typed 7.70 pt median / 13.3% flips RED · emotion 4.92 pt / 12.5% RED
   · banking77 5.58 pt / 10.3% RED · ag_news 0.51 pt PASS (0 flips) · xnli 1.76 pt PASS ·
   massive 0.99 pt PASS (1 flip). Their recipe's "shuffling answer options" training lever
   bought robustness on the wide-4/3-option suites but NOT on the 6–7-option or typed
   suites — order sensitivity is a MEASURED axis for the current d1-3B release, exactly what
   Research 009 predicted the probe would expose. Canary cell: d1 flips it on every suite
   (11.6 pt — the probe provably fires on this lane).
5. **The typed modelless control reads RED (4.70 pt / 68.3% flips) — the standing Issue-079
   finding**, not a regression: the `k == N` route binding is position-coupled until the
   owner picks the fix posture (the 077 fix cleaned ag_news/emotion/xnli; typed is 079's
   subject, owner-gated). The control red refuses the probe's RUN verdict by design; the
   record stands as disclosure, the same shape as Bench 128.

## Verdict

- **Lane: LANDED** (T1–T4 complete; T5 discipline carried in every cell; T6 deferred per the
  issue — the seven-benchmark panel, SQuAD2.0-as-noul, and the hosted `d1:free` posture all
  stay closed behind an owner ask).
- **d1-3B: a real entrant, not a board threat.** Best-in-family calibration (ECE ≤ 0.051 raw,
  no artifact), competitive single-question accuracy + latency, but third on typed and
  measurably order-biased on half the probed suites. The abstention primitive stays ours
  alone; their confidence threshold-at-0.5 guidance is exactly the shape our fused
  abstention gate replaces.
- **The fusion candidate** (Research 009's actionable 2 + 3): nothing here moves the
  modelless engine — the per-type temperature idea is COARSER than laya's per-(kind,count)
  refit; the transferable artifact is calibration-shipped-as-versioned-file, which is what
  `SigmoidGateCalibrator` + the freeze/thaw law already do.

## Provenance

- Model: `LiquidAI/d1-3B` @ `051bcc4` (HF snapshot, `.raw/d1-model/`, gitignored).
- Serving: `.raw/d1_server.py` + `.raw/boot_d1.cmd` (port 8078; `.raw/kill_port_8078.ps1`);
  stack transformers 5.19.0 / torch 2.6.0+cu124 / CUDA 12.4 (the shared drex-env — the
  flash-guard negative was verified on a separate throwaway 2.11.0+cu128 venv, then deleted).
- Posture: fp16 · calibration=none · windows-split (one question per pass) — carried in
  `/health`, the boot log, and `RunMeta.d1_lane`.
- Host: 4090-windows, AC power, High mode, release profile; preflight PASSED (line quoted
  above). GPU exclusivity held (the server was the only compute consumer).
- Suites: canonical `.raw/datasets` (slice digests in results.json: typed
  fnv1a64-287d…, xnli 6142…, massive 328f… — identical populations to the recorded lanes).
