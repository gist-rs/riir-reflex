# AGENTS.md — riir-reflex

The global `~/.agents/` rules apply; this file documents repo-local context.

## Boundary contract — read `BOUNDARY.md` first

[`BOUNDARY.md`](BOUNDARY.md) is the authoritative per-repo contract (owns /
does-not-own / crate allowlist / drift ledger); on any conflict with prose,
BOUNDARY.md wins.

- **Domain test:** decision-engine serving + comparison + contribution (NOT
  game runtime, NOT code healing)? NO → another repo; file there.
- Default build stays ONE foreign code-level dep (`katgpt-core`); `serde_json`
  is the HTTP/JSON edge only. Read BOUNDARY.md before any dep/crate/module.
- **The laya lane is SUBSTRATE-SIDE** (Issue 008 T4, 2026-09-24): the lane +
  its deps (`tokenizers`/`sha2`/`gemm`/`libm`, + macOS target-scoped
  `metal`/`objc2`) live in `../riir-infer`'s `riir-infer-laya` crate; reflex
  consumes it via the `src/laya/mod.rs` `pub use` shim behind the SAME
  `laya-riir` / `laya-riir-metal` feature names (public API unchanged) and
  keeps the CONSUMER-side G5 parity gate + frozen fixture captures. The dep is
  non-optional (the ungated Python-JSON writer moved with the lane) but
  compiles to just that writer until a forwarding feature lights the lane. Any
  lane dep bump re-runs G5 at both postures before a number is published.
  Third, opt-in posture: `laya-riir-cubecl` (riir-infer Bench 006: 5–8×
  slower than Metal, beats CPU on short sequences) — never in
  `RELEASE_FEATURES`.
- **No candle anywhere** (`.issues/006`, owner directive). **No Python
  anywhere** in the shipped binary (owner directive) — Python appears only as
  measurement-lane subprocess oracles.
- Enforcement is not prose: `../riir-ai/scripts/ci_boundary_contract.sh` via
  the `boundary-guard` skill. Found a violation? File
  `.issues/NNN_boundary_*.md` FIRST, add the drift row, then fix; closing the
  issue removes the row in the same commit.

## Role

katgpt-rs Proposal 014 Phase 1 deliverable (Plan 603): typed `decision_wire`
requests (`choice`/`score`/`noul`, abstention first-class) answered
modellessly — hashed-feature embedding → `pick_domain` corpus routing →
`Lz4FlexDrafter` corpus-is-the-model option scoring → sigmoid normalization
(never softmax) → `SigmoidGateCalibrator` confidence → fused abstain (score +
`CorpusDistanceGate`). The confidence readout dispatch is INHERITED from
katgpt-rs Bench 817's verdict — never re-derived. One std-only binary,
localhost HTTP edge, no daemon framework. Spawned by
`../katgpt-rs/.proposals/014_katgpt_decision_engine_site.md`, executed by
`../katgpt-rs/.plans/603_reflex_phase1_engine_harness.md`. Was Private forever
per katgpt-rs Research 003 — opened public 2026-09-23 as a sanctioned
exception (Research 003's dated amendment, owner directive 2026-09-22).

## Head vessels (instinct Proposal 001 T4/T7 — the laws that bind THIS repo)

Arena game heads (tetris / lanes / flappy) are SIGNED artifacts, not compiled
bytes. Canonical text:
`../riir-instinct/.proposals/001_arsenal_cognition_vessel_protocol.md`.

1. **A1 — bytes are runtime, capability is compile-time.** This repo selects
   `vessel_public_read` ONLY (default-on); the HOSTED-ONLY reader has no
   selectable path here — the moat. Head weights ride signed `.vessel` files
   loaded whole at boot; nothing compiled in, nothing re-fitted at serve time
   (the serve binary carries no fixture bytes — the fit runs at MINT time).
2. **A8 — no runtime minting.** `reflex mint-heads` is the OFFLINE mint front
   (fixture-digest-gated, deterministic — same fixtures + key →
   byte-identical vessels); the serve path only loads + verifies (strict
   ed25519 + BLAKE3, monotonic apply). A tampered/drifting vessel refuses
   loud; never repaired in place.
3. **A10 — moat.** The demo heads are public BY DESIGN (the arena is the
   public product); no GAME-IP content may ever ride a PUBLIC-RELEASE vessel
   through this repo. Hosted-only minting stays in riir-train, and its
   vessels cannot even be READ by a build of this repo.

Trust anchoring: compiled pin table first (`reflexer_vessel::default_pins`,
EMPTY until the first release artifact ships), then the operator wildcard
`RIIR_REFLEX_HEADS_PUBKEY`. A heads dir carrying vessels with NO anchor is a
config gap: exit 2 naming the env.

## Sibling-Repo Layout

```
/git/riir-reflex      ← this repo
/git/katgpt-rs        ← katgpt-core (the ONE foreign code-level dep, non-optional)
/git/riir-infer       ← the laya lane substrate (`crates/riir-infer-laya`, path dep —
                        always resolved: the ungated pyjson writer moved there; the
                        lane itself lights up behind laya-riir / laya-riir-metal)
/git/riir-reflexer    ← the vessel FORMAT repo (`crates/reflexer-vessel`, optional path dep
                        activated by `vessel_public_read`, default-on — the public-class
                        reader/writer the head lane verifies through; the manifest is
                        load-bearing for every cargo command either way)
/git/riir-ai          ← NOT a dep (boundary counter-case: katgpt-rs Proposal 017)
```

Path deps in `Cargo.toml` assume this layout. Move the repo → update the path
dep + this table. Full dependency law: `.docs/01_orientation/sibling_layout.md`.

## Build Commands

```bash
cargo clippy --all-targets -- -D warnings       # default (the product)
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings   # flag-OFF posture
cargo test                                      # semantics gates
cargo bench --bench decision_set_goat           # G2 latency + G4 alloc (release by construction)
./scripts/ci_feature_guard.sh                   # the whole local gate

# The laya lane (weights resolve from LAYA_WEIGHTS_DIR, else LAYA_HOME, else
# HF download with SHA-256 verification):
cargo test --release --features laya-riir --lib --test laya_riir_parity   # the lane's G5 gate (CPU posture)

# Latency probe (the agent labels the posture):
cargo run --release --features laya-riir --example laya_fixture_timing -- riir english 3
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --example laya_fixture_timing -- riir english 3
# CUDA posture (4090; G5 + timing, typed 13.8 ms row p50):
# LAYA_DEVICE=cuda cargo test --release --features laya-riir-cuda --test laya_riir_parity
# LAYA_DEVICE=cuda cargo run --release --features laya-riir-cuda --example laya_fixture_timing -- riir typed 5
# Full harness at the CUDA posture (Bench 082: 15/15 PASSED, typed acc
# byte-identical cross-host 0.7445 at 164 ms p50). ⚠ REFLEX_BENCH_HOST is
# REQUIRED there (unset label REFUSES at row birth — the phantom-host sentinel):
# REFLEX_BENCH_HOST=4090-windows LAYA_DEVICE=cuda cargo run --release --features laya-riir-cuda --bin harness -- --out .benchmarks/<bench>_tables

# The katgpt-rs Plan 603 T1.5 harness (datasets first):
scripts/fetch_datasets.sh
cargo run --release --bin harness                       # both-lane tables → .benchmarks/001_phase1_tables/

# The public artifact fetch lane (riir-ai Plan 623 T6 / Issue 069): public-class
# assets from hf://gist-rs/<repo>-artifacts into artifacts/cache/, BLAKE3 +
# exact-size verified BEFORE use; protected rows refused; CHECK=1 = cache
# verify only. No manifest = honest no-op (committed fixtures are the default):
scripts/fetch_artifacts.sh

# Corpus-cap levers (Issue 013; Bench 004: ag_news default 64 CONFIRMED,
# banking77 stays 40; banking77 selection TRANSFERS post-Issue-023 but was
# declined on perf/sec):
cargo run --release --bin harness -- --skip-laya --suites banking77,ag_news --cal-select-cap

# Accuracy-lever probes (Issue 013, all REFUTED — Bench 003/004/005; re-read
# post-Issue-023 confirms: banking77 net 0, massive too spread):
cargo run --release --bin harness -- --skip-laya --pair-head-ab --out /tmp/pairhead_ab

# NLI pair-feature head A/B (Issue 044 T3, xnli-shaped suites only): the
# blend's +5.67 pt FAILED the G1 floor; the G1-constrained reopen (Bench 069)
# was UNSATISFIABLE; Issue 047 / Bench 073 closed xnli measured-negative on
# the fresh validation slice — the gap stands ACCEPTED (raw max-prob AUROC
# 0.6542 is the honest confidence surface there):
cargo run --release --bin harness -- --skip-laya --nli-feature-ab --out /tmp/nli_feature_ab

# Issue 047 M1 validation-reopen lane (CLOSED, Bench 073; reads
# xnli_en_val ONLY — never the spent test split):
cargo run --release --bin harness -- --skip-laya --head-select --nb-select --ridge-select \
  --nli-m1 --suites xnli_en_val --datasets-dir .raw/datasets_t20k \
  --out .benchmarks/073_nli_m1_validation

# Option-permutation spread probe (Issue 077 / Bench 128): every probed lane
# answers each choice question under K deterministic option orderings through
# its own decide path; gate median ≤ 2 pt; the modelless control holds
# ties-only flips + the L1 fp envelope (a control red refuses the run).
# Comparison lanes ride --drex / --d1 / --agentjev (servers up, loud refusals);
# laya rides its feature + G5 parity FIRST (`--test laya_riir_parity`).
cargo run --release --bin harness -- --perm-probe --out .benchmarks/<N>_perm_spread/<posture>
cargo run --release --bin harness -- --perm-probe --drex --agentjev --suites typed_decisions --out <out>

# Harness Warm-tier store (Issue 007 P1, opt-in `corpus_db`; ndb from NDB_BIN
# else PATH — build: (cd ../riir-neuron-db && cargo build --release -p neuron-db-cli)):
NDB_BIN=../riir-neuron-db/target/release/ndb \
  cargo run --release --features corpus_db --bin harness -- \
  --runs-kv --save-corpus emotion,sst5 [--kv-dir .harness/ndb-data]
cargo test --features corpus_db --lib corpus_db -- --nocapture  # golden round-trip SKIPs loud without NDB_BIN

# Near-duplicate leak index (Issue 024, opt-in `slice_leak`; G1 runs
# scripts/slice_leak_probe.py LIVE — Rust counts must match; UNSEEN without
# .raw/datasets, SLICE_LEAK_REQUIRE_DATA=1 makes that a failure):
cargo test --release --features slice_leak --test slice_leak_oracle -- --nocapture

# E0 evidence density (riir-instinct Issue 005 T1, opt-in `nb_scope`; report-only):
cargo run --release --features nb_scope --bin harness -- --e0 \
  --out .benchmarks/053_e0_evidence_density

# Distill teacher pass (riir-train Issue 576 T3, opt-in `laya-riir`;
# student = riir-train examples/instinct_arm_b, Bench 609):
cargo run --release --features laya-riir-metal --bin harness -- --distill \
  --datasets-dir .raw/datasets_t20k --distill-out .raw/distill_teacher

# CLM comparison lane (Issue 019, opt-in `clm-lane`; prose law byte-pinned to
# sha `cca045ff` via scripts/clm_goldens.py; Bench 059 = the T-Rex re-run):
cargo test --features clm-lane --lib lanes::

# GLiNER lane (Issue 029, `--gliner`, no feature gate; scripts/gliner_lane.py
# subprocess oracle; env GLINER_PYTHON / GLINER_PY_DEVICE / GLINER_MODEL):
GLINER_PYTHON=.raw/gliner-env/Scripts/python.exe \
  cargo run --release --bin harness -- --gliner --suites banking77 --skip-laya

# AgentJev lane (Issue 025 / .issues/027, `--agentjev`; their jev_service @
# a965ca8f over HTTP; typed_decisions gold-label 0.7715 vs our 0.7445):
AGENTJEV_SERVE_URL=http://127.0.0.1:8149 \
  cargo run --release --bin harness -- --agentjev --suites typed_decisions --skip-laya

# PAW lanes (Issue 033, `--paw` hosted / `--paw-local`; one program per
# QUESTION SHAPE — scripts/paw_specs/<suite>[.<qid>].txt; per-shape resolver
# is paw::resolve_shapes, BOTH lanes). Board tier = PAW_COMPILER=paw-ft-bs48-20260530
# + PAW_COMPILE_ASYNC=1 (NOT optional). Local setup:
#   uv venv .raw/paw-env
#   PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin \
#     uv pip install --python .raw/paw-env/bin/python programasweights==0.4.10   # mask sccache (0.13.0 breaks the build)
#   .raw/paw-env/bin/python scripts/paw_preload.py   # 594 MB base + bundles (detached, resumable)
PAW_LOCAL_PYTHON=.raw/paw-env/Scripts/python.exe \
  cargo run --release --bin harness -- --paw-local --skip-laya

# Cascade lane (Issue 038 T4′ / Bench 061; needs laya-riir, mutually exclusive
# with --skip-laya). Verdict: shipped fused gate NEGATIVE (issue 042); the
# COMBINED posture (Bench 066: --gate-fit-selection --gate-distance-only,
# margin 0.16 — Bench 063, 10/10 PASS) passes the issue's full T4′
# acceptance. Lever 4 `--cascade-worthiness-lcb` (Bench 070) is OPT-IN; the
# combined-posture LCB leg is a PROVABLE no-op (Bench 071).
# `--gate-fit-calibrated` is the DEFAULT posture since Issue 056 (katgpt-rs
# Issues 909/910/911 Platt-solver repairs; `--no-gate-fit-calibrated` restores
# fit-on-raw). `--mc-ab` (Issue 055, Bench 092) is the recorded NEGATIVE.
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --bin harness -- \
  --datasets-dir .raw/datasets_t20k --cascade --cascade-worthiness --cascade-worthiness-margin 0.16 \
  --nb-select --oc-select --ridge-select --gate-fit-selection --gate-distance-only \
  --out .benchmarks/066_gate_rate_axis_levers/both

# cua-s1-forms CoreML arm (Issue 035 / Bench 048, macOS EXAMPLE — never in
# the default run; scripts/cua_s1_lane.py subprocess; coreml reads
# 24,359/24,370 == their published result):
scripts/bench_preflight.sh
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal \
  --example cua_s1_forms_arena -- --lanes coreml,modelless,laya --n-laya 2437

# openthai comparison lane (Bench 074 lineage, `--openthai`; fp32 numerics
# pinned via OPENTHAI_SYSTEMONE_DTYPE — their client default is bf16;
# Benches 084/085/086 = 4090 + M3 full fill, option-count scaling law
# 1.2×→17.2×):
OPENTHAI_PYTHON=.raw/openthai-env/bin/python \
  cargo run --release --bin harness -- --openthai --skip-laya

# Drex DLM comparison lane (Issue 073, `--drex`; CC BY-NC weights =
# measurement only): their serve.py on DREX_SERVE_URL (default 127.0.0.1:8000;
# the llama.cpp edlm fork's 8097 equally valid — the reply's model field
# discloses which). Serve: .raw/drex-env (torch cu124 + transformers 5.19)
# + .raw/drex-model; the owned cells: gold-label accuracy per primitive +
# the ECE/Brier of their confidence fields (T4 readout_brier).
DREX_SERVE_URL=http://127.0.0.1:8000 \
  cargo run --release --bin harness -- --drex --suites typed_decisions --skip-laya

# LiquidAI d1 comparison lane (Issue 078, `--d1`; license other/lfm1.0 =
# measurement only): .raw/d1_server.py (their in-repo D1Model behind a
# stdlib listener, port 8078) on D1_SERVE_URL; fp16 · calibration=None ·
# WINDOWS-SPLIT posture (their one-pass tree needs flash kernels no Windows
# torch wheel compiles — verified through 2.11+cu128). Board cells: bench 129
# (typed 0.6510 / xnli 0.8167 / massive 0.9067; ECE 0.037–0.051 raw;
# perm-probe RED ×3/6).
D1_SERVE_URL=http://127.0.0.1:8078 \
  cargo run --release --bin harness -- --d1 --suites typed_decisions --skip-laya

# Corpus-synthesis lane (riir-train plan 426 T5, `8426cef`): sealed SYNT v2
# artifact + blake3 sidecar, openthai agreement VETO. --synth-plan is
# REPORT-ONLY; --synth-corpus writes; --corpus-ab is the V5 gate (paired LB95,
# one frozen test read). Issue 064 ascent leg: --synth-density-pilot (report)
# + --synth-density-gate <p50|p75|p90>; Bench 120/121/122 verdict: UNGATED
# beats every gate strength — the seated corpus stays the ungated artifact.
# EVERY --corpus-ab pass computes the echo gates (abstention-entropy KL ≤ 0.05;
# OOD word-dropout ladder, gate rung .20): a clean V5 PASS with a negative
# gate-rung LB95 reads ECHO = the lane dies. Exclusive early-exit modes:
cargo run --release --bin harness -- --synth-plan --help
cargo run --release --bin harness -- --corpus-ab --help
```

- Default features = `["modelless"]`; `--no-default-features` is the tested
  flag-OFF posture.
- **Repo-birth gate discipline (T1.1e):** every `#![cfg]`-gated target carries
  its `required-features` row in the SAME commit (`[[bin]]`, `[[test]]
  engine_gates`, `[[bench]] decision_set_goat` pin `modelless`; the laya
  parity `[[test]]` row pins `laya-riir`). A whole-file-gated target without
  its row prints `ok. 0 passed`, exit 0, forever.
- **G5 parity LANDED 2026-09-22** — 88/88 forwards, top-1 agreement 1.0 per
  checkpoint, drift ≤ 3.1e-6 vs the 1e-3 gate (both profiles). The gate runs
  before ANY published laya number; a failed gate marks the lane PROVISIONAL
  everywhere. Port traps: candle's `Tensor::gelu()` is the TANH approximation
  (reference is erf — `gelu_erf()`); the sdpa sliding-window mask radius is
  `local_attention // 2` — either wrong = length-correlated drift 4 orders
  over the gate.

## Phase 2 status (katgpt-rs Plan 606) — LANDED 2026-09-22

- **Build stamp** (`src/build_stamp.rs` + `build.rs`): `--version` prints the
  COMPILED feature set + `release set: STALE — missing …` + the rebuild
  command when incomplete. `RELEASE_FEATURES = modelless+laya-riir` since the
  candle-free cut.
- **Bin name `reflex` (2026-09-23)**: serve bin renamed `riir-reflex` →
  `reflex` (owner ask); package/crate name unchanged (`brew install
  riir-reflex` / scoop keep the formula name). ALL dist-side follow-ups
  DISCHARGED in v0.2.2 (`367766c`+`932a2a3` + tap `d60e40d` + bucket `75d2878`
  + dist `ba750f4`) — archives carry `reflex`; installers accept both
  spellings.
- **v0.2.2 (2026-09-23)**: fitted Tetris head (decoded katgpt-rs Bench-881
  arm, λ=1, head digest `00aa6221…c6e`, boot-fit from `assets/game_heads/`)
  + Metal-default laya on darwin (`LAYA_DEVICE=cpu` opts out; G5 green both
  postures; interleaved p50 78.1 vs 176.8 ms = 2.26×).
- **v0.2.3 (2026-09-24)**: THREE-BOARD release (issue 011 closed) — lanes at
  katgpt-rs Bench 880's lossless decoded arm (λ 0.01, 84/100, digest prefix
  `7d3f1d8e`) + flappy at katgpt-rs Bench 882's v3 decoded arm (λ 1, 96/100,
  FULL digest `c93d36dc…e3c5`). Joined-state protocol: the sentence sequence
  rides `state` one per line (`noul` carries no options by wire law). Carries
  the Metal BK64+xwide lane (`a51ea42`) and the Issue-015 fix (`a3215da`).
- **v0.2.4 (2026-10-03)**: Issue 063 first-corpus lane (`RIIR_REFLEX_CORPUS`,
  `/healthz` corpus disclosure) + Issue 062 wire id fix; game heads MINT-ONLY
  (boot-fit retired). Gates 9/9, G2 p99 44 µs, leak scan ×5.
- **CORS seam** (`src/serve.rs`): `RIIR_REFLEX_ALLOWED_ORIGIN` allow-list,
  default CLOSED; `serve_listener_with` is the test seam.
- **Release pipeline**: `[profile.dist]` (strip + fat LTO),
  `scripts/build-release.sh` (dist build + SHA256SUMS + licenses),
  `scripts/binary_leak_scan.sh`, `scripts/install_copy_parity.py`. Cross
  targets via `cargo zigbuild` from the M3; dist surface = **gist-rs/reflex**
  (NEVER this source repo — the v0.2.3 wrong-repo lesson). v0.1.1 live on
  gist-rs/reflex: macOS aarch64+x86_64, linux musl ×2, windows gnu.
- **Arena site** (gist-rs/reflex-site) live at <https://reflex.gist.rs>
  (custom domain, dashboard-attached — deliberately not in `wrangler.toml`);
  `/bench/` rendered from `data/bench.json` via `scripts/publish_bench.py`
  (sanitizes machine-local meta).

## Phase 1 status (katgpt-rs Plan 603) — COMPLETE 2026-09-22

- **T1.5 harness**: 9 dataset suites + decision-point families over
  byte-identical questions (`src/harness/`, bin `harness`, dispatch-only CI
  lane `.github/workflows/harness_tables.yml`). ⛔ The six Issue-004 families
  are RETIRED (owner call, 2026-10-02): home-made synthetic evals the
  modelless engine reads AT CHANCE on honest wide populations; records in git
  history + `.benchmarks/`. `semantic_defects` (Issue 061, 102 cases) STAYS;
  gates: `tests/harness_families_gates.rs` + `tests/harness_seat_gates.rs` +
  `tests/fixture_fleet_hygiene.rs`. The Issue-004 T7 option-rank blend stands
  on the dataset lane (banking77 0.040 → 0.446, ag_news 0.258 → 0.510).
  Datasets: HF datasets-server via `scripts/fetch_datasets.sh` (blake3-digested
  in `.docs/02_protocols/dataset_manifest.md`; banking77 via `mteb/banking77`).
  Measured lessons: fused-gate birth thresholds do NOT transfer (fit per suite
  at cal ρ=30); the cal slice must not sit inside its own reference corpora;
  ECE bins are left-open so zero-confidence rows read n/a, never perfect.
- **T1.8 docs closure**: this file, README, the katgpt-rs `decision_wire`
  catalog note, `.docs/02_protocols/dataset_manifest.md` banking77 resolution.
  `structured_reads` root promotion correctly NOT pulled (re-arm trigger armed).

## The candle lane — REMOVED 2026-09-22 (`.issues/006`, owner directive "no candle at all cost")

Deleted: `laya`/`laya-metal`/`candle-metal` features, `candle-core` dep,
`src/laya/{agent,encoder,head}.rs`, `tests/laya_parity.rs`. The frozen
captures in `tests/fixtures/` ARE the reference; the riir lane's G5 gate
replays against them. Tokenizers v1 reopen trigger (recorded in HISTORY.md):
1.0.0 STABLE — the pinned GPT-2-family vocabs lack 14 ByteLevel byte atoms
that v1 validates up front (0.22 tolerates lazily). `libm` stays pinned on
numerics grounds; `gemm`/`metal`/`objc2` float freely with a G5 re-run on
bump. FROZEN record: torch MPS 25–31 ms vs candle-Metal 37–46 ms
english/typed (1.3–1.5×) — kernel-maturity gap.

## The laya-riir lane (owner directive, 2026-09-22) — LANDED at `e601295`

The riir-OWNED forward (`--features laya-riir`) — the ONE laya backend: model
math on our flat-`Vec<f32>` tensor code, owns the laya substrate outright.
CPU lane at candle parity (~156.8 ms row p50 post-threading) — the win is
deployment (prod path builds candle-free). Two bit-parity lessons any future
port must carry:
1. **candle's CPU reduction is SIMD-STRIDED** (NEON `vec_sum`: STEP=32, EPR=4,
   ARR=8, pairwise tree reduce). A sequential scalar sum drifts 1e-2-class
   through 22–28 layers; the mirror lives in `riir/ops.rs::candle_vec_sum`
   (scalar f32, same adds, same order, bit-identical).
2. **The gelu kernel must BE candle's** — `libm::erff`, not a correct
   approximation (the A&S f64 path measured ~1000× over the gate).

**Metal ladder** (`laya-riir-metal`, `LAYA_DEVICE=metal` honored; each rung
G5-gated):
- v1 (Issue 005): 14 MSL kernels, per-op CBs, lazy-flush — naive baseline
  79.0/78.7/38.5 row p50 vs torch MPS 25.7/25.5/16.2; the gate caught 4 real
  defects (stale weight-cache serving, sync-count eviction, untracked
  hazards — Tracked is EXPLICIT on macOS — layer-0 stale host copy).
- Pass 1 (2026-09-24): pass-scoped CB (commit at the 3 host reads), all-heads
  batched attention, simdgroup GEMM, row-parallel softmax/LN →
  **28.3/28.3/12.2** (beats MPS on multilingual, matches candle).
- Pass 2 `4ef290c`: narrow/wide sgemm split by `m`; trap: a staging stride
  must EXCEED the staged tile's row width (overlap corrupts half the output);
  fused flash-attn built+measured+REVERTED (real seqs need BQ≥32; the
  `Backend::attention_forward` seam STAYS — the one op-order home).
- Pass 3 `a51ea42`: BK 32→64 + xwide `sgemm_xwide` (m≥256 && n≥2048) —
  ag_news 34→29 ms BEATS the python oracle; traps: the xwide n-floor is
  MEASURED (staging intensity only pays under over-subscription); compare
  only within position-balanced swapped pairs.
- Pass 4: fused `flash_attn` (BQ=32, window-predicated key ranges,
  two-pass normalize) — banking77 75–76 vs python 70 (gap 1.2× → ~1.07×);
  traps: scores-tile live-frag bound, per-row window predicate lives in row
  threads, `l_reg` published before drain. Kill-switch `LAYA_METAL_FLASH=0`.
- Measured NEGATIVE, on record so nobody re-tries blind: wide BK=48 (barriers
  not the binding constraint), both Wᵀ-staging repairs (uncoalesced element
  form stays), per-op commit+wait (19× slower than lazy).
- **GLU trap**: the fused Wi output is `[rows, 2I]` — the activation MUST be
  written to its own contiguous buffer (`riir/ops.rs::glu_gelu_gate`;
  measured 223× divergence otherwise).
- **typed-trio + T12** (riir-infer `be46033`/`40d15dd`): the encoder is 90.1%
  of case GPU (not the head); T12 deferred head reads PROMOTED default-on
  2026-09-26 (typed 5-q median 0.984, 24/24 wins) — kill-switch
  `LAYA_HEAD_DEFER=0`. Split-K epilogue folds default-on (riir-infer
  `53334f9`) — kill-switches `LAYA_METAL_FOLD_RES=0` / `LAYA_METAL_FOLD_GLU=0`;
  the narrow (non-split) sgemm stays UN-fused.
- **T13/T13b (Issue 020 CLOSED, Bench 050, riir-infer `b0de034`+`5e18da4`)**:
  unsplit batch-1 GEMMs dispatch Apple's `MPSMatrixMultiplication`
  (bit-identical, 0.575–0.741× whole forward); paired A/B **9/9 p50 AND 9/9
  p99 wins** vs torch MPS. Kill-switches `LAYA_METAL_MPS=0` /
  `LAYA_METAL_MPS_SPLIT=0`. Lesson: five refutations among OUR kernels said
  nothing about the vendor's — price the library call before a rewrite.
- **Threading posture** (Bench 001 addenda 3–4): `min(available_parallelism, 8)`
  rayon workers (measured optimum on the 12P+4E M3; `RAYON_NUM_THREADS`
  overrides), tiny per-head gemms `Parallelism::None` under 8 MFLOP, GLU on a
  condvar-parked pool — quiet-box e2e −9.6% median row p50; numerics never
  move with worker count (the gate's own assertion).

## Lint healing — `cargo refine` before manual fixes

Mechanical clippy findings are fixed by the riir-refine healer FIRST, manual
second (`cargo refine --fix --write --verify <paths>`; feature-gated code
needs `--verify-args "--features <set>"`). Documented divergence classes stay
manual — see the `cargo-refine` skill. The bench-target guard (`bench-guard:`)
declines perf edits under `benches/` — a bench file's numbers ARE its artifact.

## GOAT gates (bind every promotion in this repo)

- **G1 (calibration):** decision-level ECE/Brier beats its own uncalibrated
  outputs AND the conformal-naive floor (Report-the-Floor, katgpt-rs Plan
  340) — a lane claiming "calibrated" without flooring fails.
- **G2 (perf):** modelless lane p99 ≤ 1 ms per decision set in-process —
  `benches/decision_set_goat.rs` (landed at birth).
- **G3 (no regression):** this repo CONSUMES katgpt-rs, never edits it.
- **G4 (alloc):** the zero-alloc core stays alloc-free — canary-armed counting
  allocator (landed at birth).
- **G5 (laya parity):** top-1 ≥ 99.9% + p-drift ≤ 1e-3 vs the reference
  checkpoint, per checkpoint, BEFORE any published table cites laya numbers;
  a failed gate marks the lane PROVISIONAL everywhere.
- **Box state is part of every latency claim (Issue 021).** Run
  `scripts/bench_preflight.sh` before any published or quoted latency number
  and quote its `PROVENANCE:` line. It refuses on battery, Low Power Mode
  (`powermode=1`; 2 = High Power, 0 = Automatic), under `SETTLE_MIN` minutes
  since plug-in, or over `MAX_LOAD`. On a Windows box the sibling gate is
  `scripts/bench_preflight.ps1` (Issue 065 T3(b)); `box_state::capture()`
  stamps the same verdict in-process, so 4090-hosted harness runs are never
  UNJUDGED.

## Documentation Shape

A **numbered-folder `.docs/` book** (fleet format — `katgpt-rs/.docs/` is the
reference): numbered folders (`01_orientation` · `02_protocols` ·
`03_decision_flow` · `04_agent_skill` · `05_resources`), bare slugs inside, a
`README.md` index per folder, `.docs/README.md` top index. Add a doc =
`slug.md` in the right folder + one index line. Gate-enforced:
`scripts/docs_shape_gate.py` (layer 8 of `scripts/ci_feature_guard.sh`).

**Site mirror law.** `.docs/03_decision_flow/decision_flow.svg` and
`.docs/04_agent_skill/SKILL.md` are published on reflex.gist.rs through
MIRRORS in `../reflex-site`. The `.docs/` copies are the **source of truth**;
after editing either, run `python3 ../reflex-site/scripts/sync_mirror.py`,
then commit **both** repos. The guard's mirror layer (layer 9,
`sync_mirror.py --check`) fails on drift and skips loud without the sibling
checkout — a skip is a deferral, never a green.

**Bench republish.** After any bench-affecting landing, re-publish the site:
the home-page TL;DR + `/bench/` tables ALL render from `data/bench.json`.
Wrapper: `../reflex-site/scripts/republish_bench.sh`.

## Numbering Discipline

Issue, plan, doc, benchmark, and research numbers are **monotonic and never
reused** — read the target dir's `.highwater`, use `value + 1`, write the new
value back (`.issues/`, `.plans/`, `.docs/`, `.benchmarks/`, `.research/`).
Before allocating: `ls` the folder AND re-read `.highwater`, and run the
workspace dual-allocation gate (`py ../katgpt-rs/scripts/dual_allocation_gate.py`
from this repo's cwd) when a number matters.

## Branch

`develop` is the working branch AND the default. No feature branches; commit
directly on `develop` per the global rule.

History: `HISTORY.md` (created at the first closed record).
