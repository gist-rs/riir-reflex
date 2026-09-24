# Plan 002 — the ANE lane bench: laya on the Apple Neural Engine, measured on OUR box, published to reflex.gist.rs/bench

**Status:** IN PROGRESS — P0 COMPLETE 2026-09-24: T0.1–T0.5 all done; six BC1S FP16 artifacts (3 models × L64/L128) exist locally, each compute-plan-verified 100%-ANE / 0-transitions on the M3 (T0.3 gate), and the 8-prompt-per-model numerical smoke vs the frozen G5 goldens reads 24/24 top-1 agreement with max prob err 0.0077–0.0200 (OBSERVED, never gated). Artifacts are LOCAL ONLY (gitignored, ~3.0 GB); the committed record is `assets/ane/manifest.json` (BLAKE3 digest-pinned) + `assets/ane/conversion_log.md`. P1 (Rust ANE lane) UNBLOCKED — the harness sibling WIP landed and the P1 runtime can now bind the artifact shapes the manifest pins (embeddings [1,L,d] fp16 + pad_bias [1,1,1,L] fp16 → hidden_state [1,L,d] fp16). P2 (site publish) rides P1.

Executes the bench half of `.issues/017_ane_lane_reference_design.md` (the
owner directive 2026-09-24: "add plan to add ane lane bench to
reflex.gist.rs/bench and do that first" — this directive is also the owner
ratification of issue 017's gate 1 at the feasibility + bench scope, with the
release-archive question for `.mlpackage` artifacts RULED 2026-09-24 at P3:
download-on-demand, digest-pinned (`LAYA_ANE_ARTIFACTS_DIR` + the BLAKE3
manifest — Claude verdict r1).

The bench page (`reflex.gist.rs/bench/`) currently carries the modelless vs
laya-rust vs laya-python-oracle comparison over the 9 dataset suites. The ANE
lane adds the third device axis for the laya lane: same cases, same
protocols, the encoder forwarded through a CoreML `.mlpackage` on the ANE —
so the table's latency column answers "CPU vs Metal vs ANE" for exactly the
single-question short-prompt class reflex serves.

## The qualified shape (from issue 017 — restated as build constraints)

- **BC1S graphs only**: fixed batch=1, one `.mlpackage` per length bucket,
  masked attention. The flexible/enumerated CoreML graph is numerically WRONG
  on the ANE (their audit: hard decision mismatches, prob err 1.0) and
  runs 100% CPU on this OS anyway. Buckets {64, 128} for the bench lane
  (their measured L64/L128 rows are the winning class; L256 rides later).
- **Compute-plan verify at conversion AND load**: 100% of ops on the ANE,
  0 device transitions — else the artifact is rejected, loudly.
- **No silent fallback**: the bench reports device placement per run; an ANE
  failure is a FAILED row, never a CPU number wearing an ANE label.
- **Decision-level parity, not p-drift**: the ANE arm will fail the Metal
  lane's 1e-3 p-drift bar by nature of FP16 ANE math (their measured
  0.0077–0.0128 prob err). The bench therefore reports top-1 agreement +
  near-tie-band list vs the Metal lane on the same cases, and gates on
  decision agreement only. p-drift is OBSERVED data, never a gate.
- **No Python at serving time**: coremltools is the OFFLINE conversion step
  (this plan's P0, a `scripts/` helper); the runtime is native Rust
  (objc2-core-ml) — Python never runs in the serve path or the bench
  runtime path.
- **Serialized** (Issue 015 containment): the ANE timing runs never share a
  process with a live Metal instance.

## P0 — offline conversion + compute-plan verify (M3, COMPLETE 2026-09-24)

- [x] T0.1 `scripts/ane_convert.py` — the one-time offline conversion
      harness (uv-run coremltools): the BC1S scaffold, the BLAKE3 manifest,
      the compute-plan verify posture, and the refusal path that forbids
      shipping stub-weight artifacts. Env smoke green (coremltools 9.x via
      uv on the M3; Python 3.12 pinned — see the record for why). **The
      weight-mapped encoder stack is NOT wired** — see the P0 execution
      record; T0.2–T0.5 wait on it.
- [x] T0.2 Weight-mapped trace of the ModernBERT encoder (22-layer
      alternating full/sliding attention for multilingual, 28-layer for
      english/typed; mask constant-folded per bucket) + convert
      `multilingual` at buckets L64 + L128 — the winning model per issue
      017's table. **DONE 2026-09-24** — both artifacts 100%-ANE / 0
      transitions (see the record for the embedding-gather finding that
      shaped the artifact I/O).
- [x] T0.3 Compute-plan verify per artifact: 100%-ANE / 0-transitions on
      the M3, printed into the conversion log and pinned in the artifact
      manifest (the tool refuses otherwise). **DONE 2026-09-24 — all six
      artifacts PASS; one earlier artifact shape (embedding gather inside
      the graph) was REFUSED by the gate at 1373/1374 and re-converted
      with the gather host-side.**
- [x] T0.4 Convert `english` + `typed` at the same buckets (completes the
      three-model set the harness tables already carry). **DONE
      2026-09-24** — all four artifacts 100%-ANE / 0 transitions.
- [x] T0.5 Numerical smoke: ANE forward vs the frozen G5 goldens'
      probabilities on 8 fixture prompts per model — top-1 agreement + max
      prob err OBSERVED, near-ties listed. **DONE 2026-09-24 — 24/24
      top-1 agreement; max prob err multilingual 0.0200 / english 0.0077 /
      typed 0.0146; zero decision flips, zero near-tie flips; structural
      parity (markers/seq_len/bucket/temperature) exact on all 24.**

## P1 — the `laya-riir-ane` bench lane (M3; UNBLOCKED — P0's artifact shapes are pinned in the manifest)

- [ ] T1.1 `laya-riir-ane` feature (opt-in, macOS-only, never wasm32) +
      objc2-core-ml runtime: load per-bucket artifact, compute-plan verify at
      load (refuse otherwise), fixed-shape FP16 in/out, logits → the
      EXISTING calibration head unchanged. Lands in the SAME commit as its
      `[[test]]` gate row (repo law). Artifact I/O is pinned by the manifest
      (P0's shape): inputs `embeddings` [1,L,d] fp16 (host-side token
      gather — the vocab-sized gather op does not place on the ANE) +
      `pad_bias` [1,1,1,L] fp16, output `hidden_state` [1,L,d] fp16
      (per-bucket output name in the manifest; the runtime slices [:n] and
      feeds the existing head).
- [ ] T1.2 G5-ANE parity gate: top-1 agreement vs the frozen goldens +
      near-tie-band report + decision-level bar calibrated on OUR goldens
      (0.02 class); p-drift published as observation.
- [ ] T1.3 `laya_fixture_timing --device ane` posture + the harness timing
      table rows (ANE column beside CPU/Metal), SERIALIZED. Position-balanced
      rounds; box state (load, RAM, power) recorded beside the numbers.
- [ ] T1.4 BOUNDARY.md allowlist rows for `objc2-core-ml` +
      `objc2-foundation` ride T1.1's commit (the boundary contract).

## P2 — publish to reflex.gist.rs/bench

- [ ] T2.1 Re-run the harness timing tables with the ANE column (the same
      `results.json` shape; new `device: ane` rows sanitized by
      `publish_bench.py` — extend its sanitizer for the artifact digests if
      they appear in meta).
- [ ] T2.2 `python3 ../reflex-site/scripts/publish_bench.py
      .benchmarks/001_phase1_tables/results.json ../reflex-site` + commit +
      `npx wrangler deploy` in the site repo (manual deploy per the
      free-tier rule; no secrets in the repo).
- [ ] T2.3 The bench page's lane legend gains the ANE row explanation
      (decision-level gate wording — near-ties listed, never hidden).

## Non-goals

- No serve.rs routing wiring in this plan (that is issue 017's later task —
  the bench measures the device; the router integration is separate).
- No heterogeneous two-queue serving (deferred in 017 behind Issue 015).
- No Windows/4090 angle — ANE is Apple silicon only; the 4090 bench lane is
  the separate bench-on-4090 issue.

## P0 execution record (2026-09-24, the M3, in-session) — T0.1–T0.5 COMPLETE

- Toolchain: `uv run --python 3.12 --with coremltools --with torch==2.7.0 --with safetensors
  --with numpy --with blake3 --with tokenizers==0.22.0 scripts/ane_convert.py …`. Python is
  PINNED 3.12: coremltools 9.0's compiled CoreML bindings (`libcoremlpython` — the
  compute-plan API the T0.3 gate is built on) ship only for ≤3.13 interpreters; on the box
  default 3.14 the `_MLComputePlanRemoteProxy` silently fails to load and placement
  verification would be BLIND while looking armed. torch pinned 2.7.0 (newest coremltools-tested).
- **Artifact shape (the finding that shaped it):** the first conversion put the token→embedding
  gather inside the graph and the T0.3 gate REFUSED it — `ios16.gather` is the one op of 1374
  the ANE compiler will not place (1373/1374, 1 transition). The gather moved HOST-side: the
  artifact input is the pre-gathered fp16 `embeddings` [1,L,d] (bit-exact row copy from the
  safetensors table; the P1 runtime holds that table resident anyway) + `pad_bias` [1,1,1,L]
  fp16 (host-built from n: 0 attend / −1e4 masked — the fp16-safe stand-in for the lane's
  f32::MIN; identical softmax outcome, documented in the tool). Re-converted: 100%/0 on all
  six. No silent fallback anywhere — the refusal was loud, recorded, and the artifact shape
  changed because of it.
- **T0.3 placement verdicts (gate = ComputeUnit.CPU_AND_NE over the COMPILED artifact):**
  multilingual L64 1373/1373 ANE · L128 1387/1387; english L64 1745/1745 · L128 1763/1763;
  typed L64 1745/1745 · L128 1763/1763 — every one 0 device transitions. L128 = L64 + one
  `slide_bias` add per sliding layer (14 ml / 18 en/td): the sliding-window mask is
  constant-folded per bucket and built only when window < L−1 (the lane's own mask-skip rule —
  at L64 the window covers the bucket). Full digests in `assets/ane/manifest.json`; the
  conversion narrative is `assets/ane/conversion_log.md`.
- **T0.5 numbers (ANE encoder artifact + fp32 torch mirror of the lane's decision head vs the
  frozen G5 goldens — the goldens are the reference since the candle lane died; the Metal lane
  was NOT run, so the serialized/issue-015 posture never came up):** selection rule = per model,
  first 4 golden rows (file order) with n ≤ 64 + first 4 with 64 < n ≤ 128 — both buckets and
  the padding path exercised, choice/score/noul all covered. Structural parity (markers /
  seq_len / bucket / temperature vs the goldens) EXACT on all 24 forwards — which also proves
  the Python render/tokenize ports byte-identical (`build_sequence`/`render_options`/py-JSON).
  Results: **top-1 agreement 24/24; max prob err multilingual 0.0200, english 0.0077, typed
  0.0146** (issue 017's expected class 0.008–0.013 — english sits inside it, typed at its edge,
  multilingual's 0.0200 is one question, `ch3-dict-crit#q0`, golden margin 0.3751 — nowhere
  near a tie). ZERO decision flips, ZERO near-tie flips (band: golden margin < 0.04), max act
  err 0.0000 everywhere. All numbers OBSERVED data, never gated — the decision-level contract
  (issue 017 §5) is what this smoke reports.
- Honest debugging note: a draft invocation carried an unsplit `--buckets` string that produced
  two bogus `L6`/`L4` artifacts (sequence length 6 and 4) during the gather debugging; they
  passed nothing, were never manifest-pinned, and were deleted before any commit. No shipped
  artifact derives from that run.
- Provenance note: the smoke's head mirror mirrors the lane's decision head, and the encoder
  mirror mirrors the lane's encoder forward, as this working tree held them
  (`src/laya/riir/{head,encoder,ops}.rs`); mid-session the sibling session LANDED the T4 move
  of the laya lane substrate-side (`04531ae` → `riir-infer-laya`, consumed via the `pub use`
  shim, same feature names) — the math is unchanged by the move, and this work touched NO
  Rust code either side of it. The mirrors are smoke-side transcriptions of the pinned math
  (G5-parity-locked, `±1e-3` vs the same goldens), not of a file path.
- Artifact release posture: binaries LOCAL ONLY (3.0 GB total, gitignored —
  `assets/ane/**/*.mlpackage/`); the committed record is `assets/ane/manifest.json`
  (BLAKE3-dir digests, shapes, placement verdicts, weight-source SHA-256) +
  `assets/ane/conversion_log.md` (per-run record). Download-on-demand + digest-pin binds at
  P3 exactly as issue 017 ruled.
- Box state: M3 Max (aarch64), macOS 26.6.2; conversions + smoke ran on BATTERY power at
  loadavg 7–36 (sibling agents compiling concurrently), ~30 GiB free RAM. Placement and
  parity verdicts are correctness facts; NO latency number is recorded in P0 — the ANE
  timing bench is P1 (serialized, position-balanced, box state beside every number).
- Historical: the first P0 record above the line originally claimed green with no artifacts —
  RETRACTED same day; the retraction paragraph stood until this session landed the real
  artifacts. The lesson stands: artifacts exist or they do not; the manifest is the proof.
