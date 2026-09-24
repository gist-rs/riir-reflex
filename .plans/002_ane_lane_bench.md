# Plan 002 — the ANE lane bench: laya on the Apple Neural Engine, measured on OUR box, published to reflex.gist.rs/bench

**Status:** IN PROGRESS — P0 PARTIAL 2026-09-24: the conversion harness (`scripts/ane_convert.py`) landed with the BC1S discipline + refusal path; the weight-mapped encoder trace is NOT wired, so NO artifacts exist yet (the plan's first draft over-claimed P0 green — retracted in the P0 record; do not bench from anything until T0.3's placement verify passes). P1 (Rust ANE lane) UNBLOCKED 2026-09-24 — the harness sibling WIP landed (runner.rs/suites.rs committed, worktree clean; the old blocker note was stale, Claude verdict r1). P2 (site publish) rides P1.

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

## P0 — offline conversion + compute-plan verify (M3, PARTIAL — honest state)

- [x] T0.1 `scripts/ane_convert.py` — the one-time offline conversion
      harness (uv-run coremltools): the BC1S scaffold, the BLAKE3 manifest,
      the compute-plan verify posture, and the refusal path that forbids
      shipping stub-weight artifacts. Env smoke green (coremltools 9.x via
      uv on the M3). **The weight-mapped encoder stack is NOT wired** — see
      the P0 execution record; T0.2–T0.5 wait on it.
- [ ] T0.2 Weight-mapped trace of the ModernBERT encoder (22-layer
      alternating full/sliding attention, mask constant-folded per bucket)
      + convert `multilingual` at buckets L64 + L128 — the winning model
      per issue 017's table.
- [ ] T0.3 Compute-plan verify per artifact: 100%-ANE / 0-transitions on
      the M3, printed into the conversion log and pinned in the artifact
      manifest (the tool refuses otherwise).
- [ ] T0.4 Convert `english` + `typed` at the same buckets (completes the
      three-model set the harness tables already carry).
- [ ] T0.5 Numerical smoke: ANE forward vs the Metal lane's probabilities on
      8 fixture prompts per model — top-1 agreement + max prob err OBSERVED
      (expected 0.008–0.013 class; NOT gated), near-ties listed.

## P1 — the `laya-riir-ane` bench lane (M3; BLOCKED on harness sibling WIP)

- [ ] T1.1 `laya-riir-ane` feature (opt-in, macOS-only, never wasm32) +
      objc2-core-ml runtime: load per-bucket artifact, compute-plan verify at
      load (refuse otherwise), fixed-shape FP16 in/out, logits → the
      EXISTING calibration head unchanged. Lands in the SAME commit as its
      `[[test]]` gate row (repo law).
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

## P0 execution record (2026-09-24, the M3, in-session)

- Env: `uv run --with coremltools` — coremltools 9.x imports on the M3
  (macOS 26.6.2); the `_MLComputePlanRemoteProxy` load failures in its
  banner are the PYTHON-side proxy only and do not by themselves measure
  ANE placement — placement verification must go through the compiled
  artifact's own compute-plan load (the T0.3 arm, which refuses on <100%).
- Tool: `scripts/ane_convert.py` — the harness landed (env smoke passes,
  the BC1S scaffold + manifest + refusal path run), but the WEIGHT-MAPPED
  encoder stack is NOT wired: converting ModernBERT's 22-layer alternating
  full/sliding attention stack faithfully (the exact op set the ANE
  accepts, mask constant-folded per bucket) is real work, and a stub
  conversion would produce silent-garbage artifacts that issue 017's
  no-silent-fallback doctrine forbids shipping. The tool REFUSES loudly
  rather than emitting them (measured: RC + the SystemExit message).
- Honest status: **P0 is PARTIAL** — T0.1 done (tool + discipline), T0.2
  through T0.5 NOT done (no artifacts exist; `assets/ane/` is empty). The
  P0 record above the line originally claimed green — RETRACTED; this
  paragraph is the record of record. The remaining P0 work (weight-mapped
  trace, per-bucket conversion, placement verify, 8-prompt numerical smoke
  vs the Metal lane) is a bounded half-day of focused work and does NOT
  belong half-faked in a shared-repo session: it is filed as the M3 side
  of the ANE bench work and continues in a dedicated pass.
- Box state at the smoke: M3 Max, AC power, load ~2 (sibling agents
  active), 47 GiB free.
