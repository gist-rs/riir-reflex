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
- **The laya lane is SUBSTRATE-SIDE** (Issue 008 T4): the lane + its deps live in
  `../riir-infer`'s `riir-infer-laya`; reflex consumes it via the `src/laya/mod.rs` shim behind the SAME
  `laya-riir` / `laya-riir-metal` features and keeps the CONSUMER-side G5 parity gate + frozen fixtures. Any lane
  dep bump re-runs G5 at both postures before a number is published. `laya-riir-cubecl` (riir-infer Bench 006) is
  opt-in, never in `RELEASE_FEATURES`.
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
`../katgpt-rs/.proposals/014_katgpt_decision_engine_site.md`. PUBLIC (a sanctioned exception to katgpt-rs
Research 003, owner directive 2026-09-22).

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
./scripts/deny_gate.sh                          # cargo-deny advisories gate (riir-refine plan 298 T2.4): membership-pinned ignore list; --scan = report-only; missing cargo-deny = LOUD SKIP; not a guard layer

# The laya lane (weights: LAYA_WEIGHTS_DIR, else LAYA_HOME, else HF download, SHA-256 verified):
cargo test --release --features laya-riir --lib --test laya_riir_parity   # the lane's G5 gate (CPU posture)
# Latency probe (the agent labels the posture):
cargo run --release --features laya-riir --example laya_fixture_timing -- riir english 3
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --example laya_fixture_timing -- riir english 3
# CUDA posture (4090): LAYA_DEVICE=cuda + --features laya-riir-cuda on the same targets. ⚠ REFLEX_BENCH_HOST is
# REQUIRED there (unset label REFUSES at row birth):
# REFLEX_BENCH_HOST=4090-windows LAYA_DEVICE=cuda cargo run --release --features laya-riir-cuda --bin harness -- --out .benchmarks/<bench>_tables

# The katgpt-rs Plan 603 T1.5 harness (datasets first):
scripts/fetch_datasets.sh
cargo run --release --bin harness                       # both-lane tables → .benchmarks/001_phase1_tables/
# Public artifact fetch (riir-ai Plan 623 T6 / Issue 069): BLAKE3 + exact-size verified BEFORE use; protected rows
# refused; CHECK=1 = cache verify only; no manifest = honest no-op:
scripts/fetch_artifacts.sh

# Corpus-cap / accuracy-lever / NLI pair-feature probes (Issue 013, Issue 044, Issue 047 — Benches 003–005, 069, 073;
# all REFUTED or closed measured-negative; kept for re-runs):
cargo run --release --bin harness -- --skip-laya --suites banking77,ag_news --cal-select-cap
cargo run --release --bin harness -- --skip-laya --pair-head-ab --out /tmp/pairhead_ab
cargo run --release --bin harness -- --skip-laya --nli-feature-ab --out /tmp/nli_feature_ab
cargo run --release --bin harness -- --skip-laya --head-select --nb-select --ridge-select \
  --nli-m1 --suites xnli_en_val --datasets-dir .raw/datasets_t20k \
  --out .benchmarks/073_nli_m1_validation

# Option-permutation spread probe (Issue 077 / Bench 128): K deterministic option orderings per lane; gate median
# ≤ 2 pt; a modelless-control red refuses the run. --clef-local rides the in-process ANE arm (never the hosted HTTP arm).
cargo run --release --bin harness -- --perm-probe --out .benchmarks/<N>_perm_spread/<posture>
cargo run --release --bin harness -- --perm-probe --drex --agentjev --suites typed_decisions --out <out>

# Clef lanes (plan 011 hosted `--clef` via the loopback forwarder, spend ceiling CLEF_SMOKE_MAX_CASES; riir-infer
# plan 619 `--clef-local`, feature clef-ane, macOS). Comparison lanes never read as product rows.
# Clef confidence-candidate probe (Issue 094 T3, Bench 158 — RECORDED NEGATIVE; report-only):
cargo run --release --features clef-ane --bin harness -- --clef-conf-probe \
  --clef-conf-max-cases 100 --out .benchmarks/<N>_clef_conf_probe

# Report-only probes — control/identity reds refuse the run (exit 1, record on disk):
cargo run --release --bin harness -- --null-probe --out .benchmarks/<N>_null_probe        # Issue 090 A1–A3, Bench 147
cargo run --release --bin harness -- --rubric-probe --out .benchmarks/<N>_rubric_probe    # Issue 090 C1, Bench 149
cargo run --release --bin harness -- --gate-drift-probe --out .benchmarks/<N>_gate_drift  # Issue 093, Bench 154
# Set-refine A/B (Issue 090 C2 / katgpt-rs Plan 627, opt-in `set_refine`, Bench 150 — MEASURED NEGATIVE, stays opt-in;
# re-open = a different key family, never a γ/τ retune):
cargo run --release --features set_refine --bin harness -- --set-refine-ab --out .benchmarks/150_set_refine_ab_goat
# Label-perm probe (riir-train 657 R8 — seeded Sattolo label-STRING rename over the suite's option-key alphabet;
# options+gold ride σ; the DEFAULT arm relabels corpus+domains too — the invariance arm; --label-perm-keep-corpus
# is the by-name-binding control (renamed keys on the identity corpus → drafter-only). NOT Bench 119 (formatting-flip).
# REPORT-ONLY — no exit-code gate; the paired delta IS the measurement (per-suite/per-seed rows, 3 seeds):
cargo run --release --bin harness -- --label-perm --suites banking77,ag_news,xnli_en,emotion --out .benchmarks/172_label_perm/<posture>

# Harness Warm-tier store (Issue 007 P1, opt-in `corpus_db`; ndb from NDB_BIN else PATH):
NDB_BIN=../riir-neuron-db/target/release/ndb \
  cargo run --release --features corpus_db --bin harness -- \
  --runs-kv --save-corpus emotion,sst5 [--kv-dir .harness/ndb-data]
cargo test --features corpus_db --lib corpus_db -- --nocapture  # golden round-trip SKIPs loud without NDB_BIN
# Near-duplicate leak index (Issue 024, opt-in `slice_leak`; G1 runs scripts/slice_leak_probe.py LIVE;
# SLICE_LEAK_REQUIRE_DATA=1 makes missing .raw/datasets a failure):
cargo test --release --features slice_leak --test slice_leak_oracle -- --nocapture
# E0 evidence density (riir-instinct Issue 005 T1, opt-in `nb_scope`; report-only):
cargo run --release --features nb_scope --bin harness -- --e0 \
  --out .benchmarks/053_e0_evidence_density
# Residual-stability depth probe (Issue 094 T2, opt-in `twt_profile`; report-only):
cargo run --release --features twt_profile --bin harness -- --depth-probe \
  --depth-max-cases 100 --out .benchmarks/depth_probe_n100
# Distill teacher pass (riir-train Issue 576 T3, opt-in `laya-riir`):
cargo run --release --features laya-riir-metal --bin harness -- --distill \
  --datasets-dir .raw/datasets_t20k --distill-out .raw/distill_teacher

# StartLux comparison lane (Issue 095, closed; CC BY-NC 4.0 — DECLARED-NC RESTRICTED, adoption fail-closed):
cargo run --release --bin harness -- --skip-laya --startlux
# mammoth/elephant lane (rethink Issue 032 T10 / Research 012): hosted API reached ONLY through the operator-run
# loopback TLS forwarder; hosted outputs are MEASUREMENT-ONLY (never a capture artifact). THE STANDING TEACHER
# POINTER: mammoth-1 (MIT, board #1 + highest open-weights) is the default Step-4 teacher candidate until a newer
# board displaces it — licence + contamination still gate. API key + ToS re-fetch are OWNER acts.
MAMMOTH_SERVE_URL=http://127.0.0.1:8793 MAMMOTH_API_TOKEN=... \
  cargo run --release --bin harness -- --mammoth --suites typed_decisions --skip-laya
# LOCAL postures (the LOCAL-FIRST law): mammoth via `.raw/mammoth-local/serve_mammoth.py` (:8793), elephant via
# `scripts/elephant_server.py` (:8096, ELEPHANT_FORK_MODE default sequential); a LOCAL row is never pooled with API:
# ELEPHANT_ALLOW_UNCAPPED=1 cargo run --release --bin harness -- --elephant --skip-laya --suites sst5
# Modelless teacher router (riir-train Issue 629 T2.5; sigmoid-gated per-suite blend over captured RIDT, never softmax):
cargo run --release --bin harness -- --teacher-router \
  .raw/distill_teacher_d3,.raw/distill_teacher_pplx_v2,.raw/distill_teacher_clef \
  --router-out .raw/distill_router

# CLM lane (Issue 019, opt-in `clm-lane`; prose law byte-pinned via scripts/clm_goldens.py):
cargo test --features clm-lane --lib lanes::
# GLiNER lane (Issue 029; scripts/gliner_lane.py subprocess oracle; GLINER_PYTHON / GLINER_PY_DEVICE / GLINER_MODEL):
GLINER_PYTHON=.raw/gliner-env/Scripts/python.exe \
  cargo run --release --bin harness -- --gliner --suites banking77 --skip-laya
# AgentJev lane (Issue 025):
AGENTJEV_SERVE_URL=http://127.0.0.1:8149 \
  cargo run --release --bin harness -- --agentjev --suites typed_decisions --skip-laya
# Jev-Mem backend lane (Issue 081; reflex AS the System-One backend via `/v1/systemone`, TYPESAFE_BASE_URL):
.raw/jevmem-env/bin/python scripts/jevmem_demo_e2e.py   # seam + determinism strip
# PAW lanes (Issue 033; one program per QUESTION SHAPE in scripts/paw_specs/; board tier needs
# PAW_COMPILER=paw-ft-bs48-20260530 + PAW_COMPILE_ASYNC=1; preload via scripts/paw_preload.py,
# programasweights==0.4.10 installed with sccache masked from PATH):
PAW_LOCAL_PYTHON=.raw/paw-env/Scripts/python.exe \
  cargo run --release --bin harness -- --paw-local --skip-laya
# Cascade lane (Issue 038 T4′; needs laya-riir; COMBINED posture Bench 066 passes T4′). `--gate-fit-calibrated` is
# the DEFAULT since Issue 056 (`--no-gate-fit-calibrated` restores fit-on-raw); `--mc-ab` (Issue 055) is NEGATIVE:
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --bin harness -- \
  --datasets-dir .raw/datasets_t20k --cascade --cascade-worthiness --cascade-worthiness-margin 0.16 \
  --nb-select --oc-select --ridge-select --gate-fit-selection --gate-distance-only \
  --out .benchmarks/066_gate_rate_axis_levers/both
# cua-s1-forms CoreML arm (Issue 035 / Bench 048, macOS example; scripts/cua_s1_lane.py subprocess):
scripts/bench_preflight.sh
LAYA_DEVICE=metal cargo run --release --features laya-riir-metal \
  --example cua_s1_forms_arena -- --lanes coreml,modelless,laya --n-laya 2437
# openthai lane (fp32 pinned via OPENTHAI_SYSTEMONE_DTYPE; their client default is bf16):
OPENTHAI_PYTHON=.raw/openthai-env/bin/python \
  cargo run --release --bin harness -- --openthai --skip-laya
# Drex DLM lane (Issue 073; CC BY-NC = measurement only; their serve.py on DREX_SERVE_URL). ⛔ FAMILY DEFERRED
# (rethink decision-index-beat law 9): re-runs of owned cells only — no new Drex lanes/captures (Issue 085 license
# gate + not GOAT on the board). Re-read the live board before revisiting.
DREX_SERVE_URL=http://127.0.0.1:8000 \
  cargo run --release --bin harness -- --drex --suites typed_decisions --skip-laya
# LiquidAI d1 lane (Issue 078; measurement only; .raw/d1_server.py on :8078, WINDOWS-SPLIT posture):
D1_SERVE_URL=http://127.0.0.1:8078 \
  cargo run --release --bin harness -- --d1 --suites typed_decisions --skip-laya
# Jeva lane (Issue 090 B1, riir-train 632 T2; license UNDECLARED → measurement + teacher-capture, adoption
# FAIL-CLOSED; macOS serve: JEVA_DEVICE=mps .raw/jeva-env/bin/python scripts/jeva_server.py, Bench 148):
JEVA_SERVE_URL=http://127.0.0.1:8095 \
  cargo run --release --bin harness -- --jeva --suites typed_decisions --skip-laya

# Corpus-synthesis lane (riir-train plan 426 T5): --synth-plan REPORT-ONLY, --synth-corpus writes, --corpus-ab is the
# V5 gate (paired LB95, one frozen test read) + the echo gates on EVERY pass (ECHO = the lane dies). Seated corpus
# stays UNGATED (Benches 120–122). Opt-in riders: --synth-rescue-prefilter (riir-refine Issue 156 T4, AUGMENT never
# replace the veto) and --probe-delta-ab (katgpt-rs Plan 621 T5.1, feature `probe_delta_ab`). Exclusive early-exit modes:
cargo run --release --bin harness -- --synth-plan --help
cargo run --release --bin harness -- --corpus-ab --help
```

- Default features = `["modelless"]`; `--no-default-features` is the tested flag-OFF posture.
- **Repo-birth gate discipline (T1.1e):** every `#![cfg]`-gated target carries its `required-features` row in the
  SAME commit. A whole-file-gated target without its row prints `ok. 0 passed`, exit 0, forever.
- **G5 parity** runs before ANY published laya number; a failed gate marks the lane PROVISIONAL everywhere. Port
  traps: candle's `Tensor::gelu()` is TANH (reference is erf — `gelu_erf()`); the sdpa sliding-window radius is
  `local_attention // 2`.

## Release + serve surface (katgpt-rs Plan 606 Phase 2; release records in HISTORY.md)

- **Build stamp** (`src/build_stamp.rs` + `build.rs`): `--version` prints the COMPILED feature set + `release set:
  STALE — missing …` + the rebuild command. `RELEASE_FEATURES = modelless+laya-riir`.
- **Bin name `reflex`**; package/crate name stays `riir-reflex` (brew/scoop formula name); installers accept both.
- Game heads are MINT-ONLY (boot-fit retired); joined-state protocol: the sentence sequence rides `state` one per
  line (`noul` carries no options by wire law).
- **CORS seam** (`src/serve.rs`): `RIIR_REFLEX_ALLOWED_ORIGIN` allow-list, default CLOSED; `serve_listener_with` is
  the test seam.
- **Release pipeline**: `[profile.dist]` (strip + fat LTO), `scripts/build-release.sh` (dist build + SHA256SUMS +
  licenses), `scripts/binary_leak_scan.sh`, `scripts/install_copy_parity.py`; cross targets via `cargo zigbuild`.
  Dist surface = **gist-rs/reflex** — NEVER this source repo.
- **Arena site** (gist-rs/reflex-site) at <https://reflex.riir.ai> (custom domain dashboard-attached, deliberately not
  in `wrangler.toml`); `/bench/` renders from `data/bench.json` via `scripts/publish_bench.py`.

## Harness (katgpt-rs Plan 603 Phase 1)

- 9 dataset suites + decision-point families over byte-identical questions (`src/harness/`, bin `harness`,
  dispatch-only CI lane `.github/workflows/harness_tables.yml`). The six Issue-004 synthetic families are RETIRED
  (owner call 2026-10-02); `semantic_defects` (Issue 061) stays, gated by `tests/harness_families_gates.rs` +
  `tests/harness_seat_gates.rs` + `tests/fixture_fleet_hygiene.rs`.
- Datasets: HF datasets-server via `scripts/fetch_datasets.sh` (blake3-digested in
  `.docs/02_protocols/dataset_manifest.md`).
- Laws: fused-gate thresholds do NOT transfer (fit per suite at cal ρ=30); the cal slice must not sit inside its own
  reference corpora; ECE bins are left-open (zero-confidence rows read n/a, never perfect).

## No candle (`.issues/006`, owner directive "no candle at all cost")

The candle lane is deleted; the frozen captures in `tests/fixtures/` ARE the reference the riir lane's G5 gate
replays. `libm` stays pinned on numerics grounds; `gemm`/`metal`/`objc2` float freely with a G5 re-run on bump.
Tokenizers v1 reopen trigger: recorded in HISTORY.md.

## The laya-riir lane — the ONE laya backend (`--features laya-riir`, landed `e601295`)

Riir-owned forward on flat `Vec<f32>` tensors (substrate in riir-infer). Bit-parity laws any future port carries:
1. **candle's CPU reduction is SIMD-STRIDED** (NEON `vec_sum`: STEP=32, EPR=4, ARR=8, pairwise tree) — mirrored in
   `riir/ops.rs::candle_vec_sum`; a sequential sum drifts 1e-2-class.
2. **The gelu kernel must BE candle's** — `libm::erff`, not a correct approximation (~1000× over the gate otherwise).

**Metal** (`laya-riir-metal`, `LAYA_DEVICE=metal`; every rung G5-gated; ladder records in HISTORY.md / riir-infer):
- Traps: Tracked hazards are EXPLICIT on macOS; a staging stride must EXCEED the staged tile's row width; the fused
  GLU output `[rows, 2I]` must be written to its own contiguous buffer (`riir/ops.rs::glu_gelu_gate`; 223× otherwise);
  the `Backend::attention_forward` seam is the one op-order home; compare only within position-balanced swapped pairs.
- Measured NEGATIVE (do not re-try blind): wide BK=48, both Wᵀ-staging repairs, per-op commit+wait (19× slower).
- Kill-switches: `LAYA_METAL_FLASH=0`, `LAYA_HEAD_DEFER=0`, `LAYA_METAL_FOLD_RES=0` / `LAYA_METAL_FOLD_GLU=0`,
  `LAYA_METAL_MPS=0` / `LAYA_METAL_MPS_SPLIT=0` (Issue 020, Bench 050). Lesson: price the vendor library call before a
  kernel rewrite.
- **Threading**: `min(available_parallelism, 8)` rayon workers (`RAYON_NUM_THREADS` overrides); tiny per-head gemms
  `Parallelism::None` under 8 MFLOP; numerics never move with worker count (the gate's own assertion).

## Formatting — rustfmt defaults, gated (Issue 099)

Format touched `.rs` with `rustfmt --edition 2024 <file>` before committing.
⛔ **Never `cargo fmt --all` here** — it formats the path-dep siblings too
(`../katgpt-rs`, `../riir-infer`, `../riir-reflexer`), including katgpt-rs's
INTENTIONAL pinned drift, as uncommitted cross-repo edits. Verdict:
`scripts/fmt_gate.py` (guard layer 11; bare rustfmt per tracked file with
`--config skip_children=true` — the `--skip-children` FLAG does not exist on
stable and its rejection exits 1, which the gate refuses as NO VERDICT).
Drift is pinned by MEMBERSHIP on `scripts/fmt_pending_expected.txt` — EMPTY
since `747c8ea`; format new drift, never add a row.

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
`.docs/04_agent_skill/SKILL.md` are published on reflex.riir.ai through
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
