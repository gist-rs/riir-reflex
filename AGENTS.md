# AGENTS.md — riir-reflex

The global `~/.agents/` rules apply; this file documents repo-local context.

## Boundary contract — read `BOUNDARY.md` first

[`BOUNDARY.md`](BOUNDARY.md) is the authoritative per-repo contract: what this
repo **owns**, what it **does not own** (with the correct home for each), the
crate-granular **allowlist** of what it may depend on, links to the cross-repo
rules' one canonical home, and the **drift ledger** of known gaps. On any
conflict with prose in this file, BOUNDARY.md wins.

- **Domain test:** is this **decision-engine serving + comparison +
  contribution** (NOT game runtime, NOT code healing)? NO → it belongs in
  another repo; file there.
- **Read it before** adding any dep, crate, module — the default build stays
  ONE code-level dep (`katgpt-core`); `serde_json` is the HTTP/JSON edge
  only; the laya lane's deps (`tokenizers`/`sha2`/`blake3`/`gemm`/`libm`,
  + `metal`/`objc2` on macOS) join ONLY behind `laya-riir` /
  `laya-riir-metal` (candle was removed entirely — `.issues/006`, owner
  directive), each in the same commit as its G5 parity `[[test]]` row.
- **No Python anywhere** (owner directive): no sidecar, no `uv`, no HF
  transformers — the laya lane is the native-Rust port.
- **Enforcement** is not prose: `../riir-ai/scripts/ci_boundary_contract.sh`
  (run VIA the `boundary-guard` skill, not ad-hoc greps).
- **Found a violation?** File the issue FIRST (`.issues/NNN_boundary_*.md`),
  add the drift row, then fix. Closing the issue removes the row in the same
  commit.

## Role

The katgpt-rs Proposal 014 Phase 1 deliverable (Plan 603): typed decisions over the
LANDED katgpt-rs substrate — `decision_wire` requests (`choice`/`score`/
`noul`, abstention as a first-class answer) answered modellessly:

- hashed-feature embedding → `pick_domain` corpus routing →
  `Lz4FlexDrafter` corpus-is-the-model option scoring → sigmoid
  normalization (never softmax) → `SigmoidGateCalibrator` confidence →
  fused abstain (score + `CorpusDistanceGate`)
- the confidence readout dispatch is INHERITED from katgpt-rs Bench 817's verdict
  (narrow = inverted label entropy, wide = argmax-label-prob) — never
  re-derived
- localhost HTTP edge: one std-only binary, no daemon framework
- the laya comparison lane + honest-metrics harness (opt-in `laya-riir`;
  G5 parity gate ≥ 99.9% top-1 / ≤ 1e-3 p-drift BEFORE any published
  number; published tables are CI-regenerated, never hand-typed)

Spawned by `../katgpt-rs/.proposals/014_katgpt_decision_engine_site.md`,
executed by `../katgpt-rs/.plans/603_reflex_phase1_engine_harness.md`.
Was **Private forever** per katgpt-rs Research 003 — opened public
2026-09-23 as one of the first sanctioned exceptions (Research 003's
dated amendment, owner directive 2026-09-22).

## Sibling-Repo Layout

```
/git/riir-reflex      ← this repo
/git/katgpt-rs        ← katgpt-core (the ONE code-level dep, non-optional)
/git/riir-ai          ← NOT a dep (boundary counter-case: katgpt-rs Proposal 017)
```

Path deps in `Cargo.toml` assume this layout. Move the repo → update the
path dep + this table. Full dependency law: `.docs/sibling_layout.md`.

## Build Commands

```bash
cargo clippy --all-targets -- -D warnings       # default (the product)
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings   # flag-OFF posture
cargo test                                      # semantics gates
cargo bench --bench decision_set_goat           # G2 latency + G4 alloc (release by construction)
./scripts/ci_feature_guard.sh                   # the whole local gate

# The laya lane (opt-in; weights resolve from LAYA_WEIGHTS_DIR, else
# LAYA_HOME, else download from HF with SHA-256 verification):
cargo test --release --features laya-riir --lib --test laya_riir_parity   # the lane's G5 gate (CPU posture)

# Latency probe (the one lane; the agent labels the posture):
cargo run --release --features laya-riir --example laya_fixture_timing -- riir english 3
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --example laya_fixture_timing -- riir english 3

# The Plan 603 T1.5 harness (datasets first: scripts/fetch_datasets.sh):
scripts/fetch_datasets.sh
cargo run --release --bin harness                       # both-lane tables → .benchmarks/001_phase1_tables/
```

- Default features = `["modelless"]` (the engine IS the product — the
  application-crate precedent); `--no-default-features` is the tested
  flag-OFF posture.
- **Repo-birth gate discipline (T1.1e):** every `#![cfg]`-gated target
  carries its `required-features` row in the SAME commit
  (`[[bin]]` + `[[test]] engine_gates` + `[[bench]] decision_set_goat` all
  pin `modelless`; the laya parity `[[test]]` row pins `laya-riir` — the
  candle lane's `laya_parity` row died with the lane, `.issues/006`).
  A whole-file-gated target without its row prints `ok. 0 passed`, exit 0,
  forever.
- **G5 parity LANDED 2026-09-22** — 88/88 forwards, top-1 agreement 1.0 per
  checkpoint, prob drift ≤ 3.1e-6 against the 1e-3 gate (both profiles).
  The gate runs before ANY published laya number; a failed gate marks the
  lane PROVISIONAL everywhere its numbers appear. The port's measured
  traps: candle's `Tensor::gelu()` is the TANH approximation (the
  reference is erf — use `gelu_erf()`), and the sdpa sliding-window mask
  radius is `local_attention // 2` (the attention module's `+1` is
  flash-path bookkeeping only) — getting either wrong showed up as
  length-correlated drift 4 orders of magnitude over the gate.

## Phase 2 status (Plan 606) — LANDED 2026-09-22 (site live on reflex.gist.rs; full 5-target release matrix)

Distribution + arena site live (plan: `../katgpt-rs/.plans/606_reflex_phase2_site_distribution.md`):

- **Build stamp** (`src/build_stamp.rs` + `build.rs`): `--version` prints the COMPILED feature
  set (derived from cargo's own `CARGO_FEATURE_*`, never hand-typed) + `release set: STALE —
  missing …` + the rebuild command when incomplete. `RELEASE_FEATURES = modelless+laya-riir`
  since the candle-free cut (`.issues/006` T4; was `modelless+laya`).
- **Bin name `reflex` (2026-09-23)**: the serve bin target renamed `riir-reflex` → `reflex`
  (owner ask: easy to type) — package/crate name `riir-reflex` unchanged, so
  `brew install riir-reflex` / `scoop install riir-reflex` keep naming the formula while the
  installed COMMAND is `reflex`; `--version` stamps `reflex <ver>`. Ships with the NEXT
  release — until the dist-side follow-ups land, existing archives still carry `riir-reflex`:
  gist-rs/reflex `install.sh`/`install.ps1` (binary filename inside the archive), the tap
  formula's `bin.install` target, the scoop manifest (`bucket/riir-reflex.json` → the exe
  name), and reflex-site's launch-command copy. **ALL DISCHARGED in v0.2.2 (2026-09-23,
  `367766c`+`932a2a3` + tap `d60e40d` + bucket `75d2878` + dist `ba750f4`)** — the v0.2.2
  archives carry `reflex`; the installers accept BOTH spellings (pre-v0.2.2 pins keep
  installing `riir-reflex`), both paths live-verified.
- **v0.2.2 live (2026-09-23)** — the fitted game head + Metal-default laya (Plan 001,
  `.plans/001_game_head_serving.md`): the modelless lane answers Plan 607's Tetris spot
  question from the decoded Bench-881 head (λ=1, 44/120 in+LOO anchors pinned in
  `tests/game_heads_serve.rs`; head digest `00aa6221…c6e`), boot-fitted from the verbatim
  BLAKE3-pinned fixture copy (`assets/game_heads/`); the serve edge tries the head before
  the cosine engine's abstain (grammar-invalid / foreign question / non-noul all fall
  through — `src/game_heads.rs` `respond`). Lanes + flappy stay honest abstains with
  recorded unblock paths (`.issues/011`). Darwin release artifacts carry `laya-riir-metal`
  and the laya lane defaults to Metal on those builds (`LAYA_DEVICE=cpu` opts out; G5
  parity green at BOTH postures — 29 s metal vs 219 s cpu on this box; fresh interleaved
  row p50 78.1 vs 176.8 ms = 2.26×). Release surface: GitHub release v0.2.2 (6 assets,
  leak-scan PASS ×5), host + 4090 windows smoke (byte-identical head answer), brew tap
  resolves 0.2.2 hash-verified + audit clean, scoop bumped, site copy deployed and
  curl-verified.
- **CORS seam** (`src/serve.rs`): `RIIR_REFLEX_ALLOWED_ORIGIN` allow-list gates OPTIONS
  preflight + ACAO echo; default CLOSED (no ACAO — drive-by posture). The arena site prints
  the exact launch command. 7 tcp-level tests both directions (`tests/serve_cors.rs`);
  `serve_listener_with` is the explicit allow-list test seam.
- **Release pipeline**: `[profile.dist]` (strip + fat LTO; benches keep plain `release`),
  `scripts/build-release.sh` (dist build + `--remap-path-prefix $HOME=/build` + package
  root-layout tar.gz + SHA256SUMS; licenses via cargo-about), `scripts/binary_leak_scan.sh`
  (the Plan-105 port), `scripts/install_copy_parity.py` (site vs dist README).
- **v0.1.1 live** on [`gist-rs/reflex`](https://github.com/gist-rs/reflex) — the FULL cargo-heal-matching matrix: macOS aarch64 + x86_64 · linux musl x86_64 + aarch64 · windows x86_64-pc-windows-gnu (the three cross targets built via `cargo zigbuild` from the M3 — `build-release.sh` routes non-host triples through it; no Actions minutes spent). Cross smoke: linux musl ×2 under docker (alpine, both arches) — `--version` stamp complete, serve + healthz + decide live; windows exe on the 4090 — same; leak scan PASS ×3; linux↔mac decide responses byte-identical. v0.1.0 was pulled: its archives nested the binary (found by the G1 clean-install smoke) and the same-name asset replacement did not survive the CDN cache. G1 PASS both mac arches (arm64 native + Rosetta x86_64), brew fetch through the tap formula hash-verified (`homebrew-tap` 8f8e28b). Scoop: `gist-rs/scoop-bucket` carries `bucket/riir-reflex.json` (landed WITH the windows asset, the no-404-manifest rule).
- **Arena site** ([gist-rs/reflex-site](https://github.com/gist-rs/reflex-site)) live at
  **<https://reflex.gist.rs>** (the custom domain — owner attached it via the dashboard
  2026-09-22; deliberately NOT declared in the site's `wrangler.toml`, which records why:
  a config `routes` block would make every future deploy need zone route permission no
  .env token has; the workers.dev URL <https://reflex-site.foxfox.workers.dev> still
  answers): playground over the visitor's localhost engine, `/bench/` rendered from
  `data/bench.json` — GENERATED from the harness output by `scripts/publish_bench.py`
  (sanitizes machine-local meta; the raw results.json carries `/Users/...` paths that
  must never reach the site).

## Phase 1 status (Plan 603) — COMPLETE 2026-09-22

T1.1–T1.8 all landed. The last two:

- **T1.5 harness LANDED 2026-09-22** — 9 dataset suites + the six Issue-004
  harness decision-point families (`src/harness/families.rs`: five modelless
  synthetic + `harness_cache_reuse`, LLM-lane only — loud SKIPPED without
  `laya-riir`) over byte-identical questions (`src/harness/`, bin `harness`,
  record `.benchmarks/001_phase1_harness.md`, regenerated tables
  `.benchmarks/001_phase1_tables/`, dispatch-only CI lane
  `.github/workflows/harness_tables.yml`). Gates: `tests/harness_families_gates.rs`
  (7 tests — counts/disjointness/gold-agreement/determinism/anti-pathology
  floors/**discrimination floor**/cache_reuse loud-skip) +
  `benches/harness_families_goat.rs` (G2 p99 59–79 µs tail support 11/1000,
  G4 alloc-free ×5, canary-first). **Issue-004 T3 COMPLETED 2026-09-23**
  (the issue closed, record in HISTORY.md): the full laya-lane harness run
  landed (uncapped, 15 suites, PASSED) — `harness_cache_reuse` measured at
  acc 0.5000 = exactly chance (the honest negative-ish result; reading +
  box state in Bench 001 Addendum 6), laya columns published as a
  PRE-MOVE BASELINE sha-pinned against the 008 T4 `src/laya` move, and the
  runner now discloses `laya_max_questions` + the baseline posture in
  every TABLES.md header. **The engine option-rank blend (Issue 004
  T7, verdict round 3):** the pre-blend engine ranked options by the LZ4
  drafter delta alone — a 4-byte option string never moves a ~300-byte
  context's compressed length, so every option tied and picks were CONSTANT
  (input-independent; accuracy sat exactly at the gold-0 rate on
  class-balanced fixtures). The fix blends the drafter delta with the
  state-alone cosine to each option's corpus centroid (`ROUTE_SCALE = 8`); a
  discrimination floor (distinct picks ≥ 2, distinct vectors ≥ 2) pins the
  fix in the gates. Post-blend the modelless lane reads ABOVE chance on
  every family (0.375–0.500) and the dataset lane moved too — banking77
  0.040 → 0.446, ag_news 0.258 → 0.510, sst5 0.157 → 0.217; small mixed
  deltas on emotion (−1.3 pt) and prompt_injections (−4.3 pt), recorded
  both ways in the Bench 001 addendum (published, never gated). Datasets:
  HF datasets-server `/rows` JSON (`scripts/fetch_datasets.sh`,
  blake3-digested in `.docs/dataset_manifest.md`); banking77 via the
  `mteb/banking77` mirror (PolyAI is script-based and unservable — recorded
  in Gaps). Metrics port verbatim from `.docs/laya_bench_protocols.md` §5.
  **Two measured lessons the first run paid for:** (a) the fused-gate birth
  thresholds (0.35/0.5) do NOT transfer — they abstained 64–100% on
  real-corpus suites, so the harness FITS both thresholds per suite at the
  cal-slice 30th percentile (the T1.6 arena posture ρ=30%); (b) the
  calibration slice must not sit inside its own reference corpora — a cal
  case scoring cos 1.0 against ITSELF inflated every cal quantile and
  over-armed the gates (the corpus pool now starts AFTER the cal slice). A
  suite where the calibrator learns "always wrong" collapses confidence to
  exactly 0.0 and abstains everything — correct autonomous behavior, and
  the reason some rows read readout-ECE 0.000 (the protocol's ECE bins are
  left-open `(0,1]`, so zero-confidence rows fall in NO bin — read those
  as n/a, never as perfect).
- **T1.8 docs closure LANDED 2026-09-22** — this file, README (results +
  honest reading), the katgpt-rs `decision_wire` catalog note (the
  substrate's consumer), and the `.docs/dataset_manifest.md` banking77
  resolution. The `structured_reads` root promotion line is correctly NOT
  pulled: the engine consumes the drafter/routing/calibration substrate,
  never `structured_read` (the recorded re-arm trigger stays armed).

## The candle lane — REMOVED 2026-09-22 (`.issues/006`, owner directive "no candle at all cost")

The candle reference lane (`laya` / `laya-metal` / `candle-metal` features,
`src/laya/{agent,encoder,head}.rs`, `tests/laya_parity.rs`) is deleted. It
birthed the goldens (Plan 603 T1.4) and took the last candle workload in
repo history — the 005 T5 same-session three-way chart — then died. The
frozen captures in `tests/fixtures/` ARE the reference now; the riir lane's
G5 gate replays against them (candle-independent, verified at the removal).
Historical numbers stay quoted below as FROZEN record, never re-runnable
from this repo.

The former candle-metal posture readings (owner directive 2026-09-22,
interleaved same-box): python torch-MPS **25–31 ms** vs port candle-Metal
**37–46 ms** english/typed, **15–18 vs 21–23 ms** multilingual — torch MPS
~1.3–1.5× faster; the gap was candle-vs-MPSGraph kernel maturity (Bench 002
+ Bench 001 addendum; a first cross-session reading claiming the port
faster was a box-load artifact, retracted with the interleaved A/B).

## The laya-riir lane (owner directive, 2026-09-22) — LANDED at `e601295`

The riir-OWNED forward (`--features laya-riir`) — the ONE laya backend
since `.issues/006`: the model math ripped from the (deleted) candle port
onto our flat-`Vec<f32>` tensor code — **no candle anywhere** ("say our
name, not candle's"). It owns the laya substrate outright
(tokenizer/config/download/render/envelopes — the `any(laya, laya-riir)`
split died with the candle lane). Deps: `gemm` + `libm` — BOTH originally
version-matched to candle's own CPU calls (`.issues/002` + `.issues/003`);
the match RATIONALE died with candle (float freely, G5 re-run on any bump
— `.issues/006` T5) except `libm`, which stays pinned on NUMERICS grounds
(bit-identical gelu).

**G5 parity GREEN at landing** (`tests/laya_riir_parity.rs`, the SAME
corpus + expected capture as the candle lane): top-1 agreement 1.000000
×3 checkpoints, prob drift 1.83e-6 / 1.03e-6 / 3.01e-6 vs the 1e-3 gate
— the candle lane's own drift class. Two bit-parity fixes the first red
run paid for, both worth carrying to any future port of this model:
1. **candle's CPU reduction is SIMD-STRIDED** (NEON `vec_sum`: STEP=32,
   EPR=4, ARR=8 — eight f32x4 accumulators, pairwise tree reduce,
   `vaddvq` horizontal add, scalar leftovers). A sequential scalar sum
   differs by ulps on EVERY LN/softmax, and that bias amplifies through
   22–28 layers into 1e-2-class prob drift (measured: 1.4e-2–3.5e-2 with
   the naive sum). The mirror lives in `riir/ops.rs::candle_vec_sum` —
   scalar f32, same adds, same order, bit-identical, no intrinsics.
2. **The gelu kernel must BE candle's** — `libm::erff`, not a correct
   approximation. The A&S 7.1.26 f64 path (1.5e-7 abs error) measured
   ~1000× over the gate; swapping in the version-matched libm crate
   (the exact kernel candle calls) dropped drift 1000×.

Latency — CPU lane (Bench 001 addendum 2 + the addenda 3–4 threading work):
riir CPU 156.8 ms row p50 (post-threading, back-to-back sweep) ≈ **candle
CPU at parity** (the version-matched gemm working as designed) — the win
is deployment (the prod path builds candle-free).

**The Metal lane (Issue 005, `laya-riir-metal`):** `LAYA_DEVICE=metal` is
HONORED — 14 MSL kernels (one stride-general 16×16-tiled GEMM, candle's
A&S erf gelu verbatim), per-op committed command buffers with candle's
lazy-flush shape (sync only at the three host reads), explicit Tracked
hazards, permanent weight cache + per-pass chain cache. G5 GREEN at the
Metal posture (drift ≤ 5.981e-6 vs 1e-3) — and the gate caught four real
defects en route (weight-cache stale-serving recycled activation
addresses; sync-count eviction vs forward-lifetime slots; untracked
hazards across per-op command buffers — macOS defaults untracked, so
Tracked is EXPLICIT; layer-0's host copy reading stale residual bytes —
now a device-side `copy_into`). The fair all-Metal three-way (Bench 001
addendum 6, same-session interleaved): torch MPS 25.7/25.5/16.2 · candle
Metal 31.1/31.2/18.7 · riir Metal 79.0/78.7/38.5 row p50 — the honest
naive-v1 baseline (2.5× behind candle's MLX simdgroup kernels); the
optimization ladder (persistent activations, fused chains, simdgroup
GEMM) measures against it. A literal per-op commit+wait measured 0.59
ms/dispatch — 19× slower than the lazy shape on the gate corpus.

The GLU trap worth remembering: the fused Wi output is `[rows, 2I]` — the
activation MUST be written to its own contiguous buffer, or the next
matmul reads row 0's gate as row 1's input (measured 223× divergence,
fixed in `riir/ops.rs::glu_gelu_gate`).

**Threading posture (Bench 001 addenda 3–4, both bit-transparent):**
the gemms run `min(available_parallelism, 8)` rayon workers — 8 is the
measured latency optimum on the 12P+4E M3 (E-cores pace every join at
higher counts; `RAYON_NUM_THREADS` overrides verbatim, uncapped), and
the 576 tiny per-head gemms run `Parallelism::None` under 8 MFLOP. The
GLU (the last serial elementwise pass, ~13–14% of the forward wall)
splits its output range across a persistent condvar-parked pool of the
same `num_threads()` workers — bit-identical at any count (G5 drift
byte-identical at every change; pool-vs-serial lib tests pin mid-row
chunk boundaries). Quiet-box e2e: **−9.6% median row p50** (4/4
interleaved pairs, 177.5→161.0 / 181.7→166.0 / 193.5→173.0 /
189.1→169.2 ms), matching the profile-share prediction. Numerics never
move with worker count — that is the gate's own assertion.

## Lint healing — `cargo heal` before manual fixes

Mechanical clippy findings are fixed by the riir-clippy healer FIRST,
manual second (`cargo heal --fix --write --verify <paths>`). Documented
divergence classes stay manual; see the `cargo-heal` skill. The healer's
bench-target guard (`bench-guard:`) declines perf edits under `benches/` —
a bench file's numbers ARE its artifact.

## GOAT gates (bind every promotion in this repo)

- **G1 (calibration):** decision-level ECE/Brier beats its own uncalibrated
  outputs AND the conformal-naive floor (Report-the-Floor, katgpt-rs Plan 340) — a
  lane claiming "calibrated" without flooring fails.
- **G2 (perf):** modelless lane p99 ≤ 1 ms per decision set in-process —
  asserted by `benches/decision_set_goat.rs` (landed at birth).
- **G3 (no regression):** this repo CONSUMES katgpt-rs, never edits it.
- **G4 (alloc):** the zero-alloc core stays alloc-free — asserted with a
  canary-armed counting allocator (landed at birth).
- **G5 (laya parity):** top-1 ≥ 99.9% + p-drift ≤ 1e-3 vs the reference
  checkpoint, per checkpoint, BEFORE any published table cites laya
  numbers; a failed parity gate marks the lane PROVISIONAL everywhere.

## Numbering Discipline

Issue, plan, doc, benchmark, and research numbers are **monotonic and never
reused** — read the target dir's `.highwater`, use `value + 1`, write the
new value back. Applies to `.issues/`, `.plans/`, `.docs/`, `.benchmarks/`,
`.research/`. Before allocating: `ls` the folder AND re-read `.highwater`,
and run the workspace dual-allocation gate (`py
../katgpt-rs/scripts/dual_allocation_gate.py` from this repo's cwd) when a
number matters.

## Branch

`develop` is the working branch AND the default. No feature branches; commit
directly on `develop` per the global rule.

History: `HISTORY.md` (created at the first closed record).
