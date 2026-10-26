# Issue 098 — harness runner.rs carve: the 12,097-line monolith vs the repo's own 20-module convention

**Status:** Open — filed 2026-10-26 (doc-sync unit #46 idle window, 4090 box) from a
code-smell hunt; zero behavior change intended, pure structure. No plan opens until a
session can run the build battery (this box was mid-bout at filing time; the work is
mechanical and consumer-free).
**Claimed:** zed-amo-idle (riir-refine cycle), 2026-10-26T12:10Z, expires 2026-10-28T12:10Z

Facts (measured at filing, `develop` `0eb4c3f`):

- `src/harness/runner.rs` is **12,097 lines / 179 fns** — ~6× over the workspace's
  3200-line guidance and ~6× the next-largest file (`suites.rs` 2,487).
- The carve convention already exists and works: **20 `runner/*.rs` submodules**
  (`nb_lane`, `genome_lane`, `oc_lane`, `ridge_lane`, `nli_lane`, `nli_m1`, `e0`,
  `distill`, `router`, `ensemble`, `synth`, `corpus_ab`, `perm_probe`, `null_probe`,
  `depth_probe`, `clef_conf_probe`, `rubric_probe`, `density_pilot`, `echo_gates`,
  `gate_drift`) were extracted lane-by-plane as lanes landed. `cb221da` shows the
  SOLID form: extract ONE core (`TeacherWireLane` trait + `run_teacher_wire_lane`),
  delegate behavior-preserving, lib tests green.
- What remains in the monolith (line ranges at filing):
  - **~17 inline comparison-lane bodies, ~4122–7279 (~3.1k lines)**: `run_clm_lane`,
    `run_jsonl_oracle_lane`, `run_gliner_lane`, `run_bekko_lane`, `run_agentjev_lane`,
    `run_clef_lane`, `run_openthai_lane`, `run_drex_lane`, `run_d1_lane`,
    `run_jeva_lane`, `run_startlux_lane`, `run_teacher_wire_lane` + the
    `TeacherWireLane` impls (`run_elephant_lane`, `run_mammoth_lane`,
    `run_opendecider_lane`), `run_pplx_lane`, `run_d3_lane`, plus the laya
    load/pick/assemble block (~4122–4961). These are exactly the class the 20
    existing modules carved.
  - **`run()` ~8634–10264 (~1.6k lines)** — the run loop; plus the report
    formatters trailing it (`fmt4`/`fmt_opt`/`fmt_gate_fit`, ~10264+).
  - `prepare` (~7279), `suite_lanes` (7568 — NOT a smell: one-line lookup over the
    `SUITES` table; verified before filing), shared core types
    (`SuiteSpec`/`LaneResult`/`SuiteResult`/`RunMeta`/`RunOutput`), engine building +
    posture fitting (~1987–4122: `build_engine*`, `engine_request_into`,
    `fit_posture_inner`, `build_selection_measurement*`, `readout_report_on_cal`).

## Tasks

- [x] **T1 — carve the comparison-lane bodies (mechanical, do first).** DONE 2026-10-26
  (zed-amo-idle): `runner/lanes_wire.rs` (1,688 lines — the laya block + CLM twins +
  jsonl-oracle + GLiNER/Bekko/AgentJev/clef) and `runner/lanes_hosted.rs` (1,446 —
  OpenThai/Drex/D1/Jeva/StartLux/PPLX/D3 + the TeacherWireLane family + the Drex
  per-kind confidence readout); runner.rs 12,098 → 8,974. Bodies moved verbatim
  (column-0 items `pub(super)`); bare-name call sites kept via the glob re-imports +
  two explicit re-exports (`parse_python_answer` — the external `tests/harness_units.rs`
  path — and `JsonlOracle` for distill's `super::` paths); three `super::{cascade,
  item_dump, suites}` references (harness-level modules, not runner-level) re-pointed
  `crate::harness::`. Gates: `cargo test --lib` 480/0 (floor 472) + `--all-features`
  551/0 + clippy `--all-targets -D warnings` at default/all-features/no-default — all
  green; rustfmt clean.
- [x] **T2 — carve the report tail.** DONE 2026-10-26 (zed-amo-idle): the `// ── table
  rendering` block (fmt4 / fmt_opt / fmt_gate_fit / jdi_crosswalk_applies /
  render_markdown, 1,177 lines) → `runner/report.rs`; runner.rs 8,973 → 7,799.
  `render_markdown` re-exported (`pub use self::report::render_markdown;`) so the
  `harness` bin's `runner::render_markdown` path is unchanged; two harness-level
  `super::{box_state, modal_floor}` references re-pointed `crate::harness::`. Same
  gates green: lib tests 480/0 + all-features 551/0; clippy ×3 postures 0 errors;
  rustfmt clean.
- [ ] **T3 — split `run()` into phase fns (highest risk, last).** The ~1.6k-line loop
  decomposes along its own seams: per-suite preparation call, lane orchestration,
  leak/echo gate wiring, summary assembly. Split in-place first (private fns in
  runner.rs), move to `runner/run_phases.rs` only if runner.rs stays over ~3200
  after T1+T2 (projected ~5.5k → after T3 ~4k; a further carve is a separate call,
  NOT this issue — the guidance is "as possible", and each move buys real
  reviewability only at file granularity).
- [-] **T4 (deferred — separate decision).** `engine.rs` (2,402) and `suites.rs`
  (2,487) sit near the 2048 soft guidance but under 3200; leave unless a T1-style
  natural seam shows up. The measurement-lane `unwrap/expect` density
  (`lanes/*.rs`, 6–59 per file) is the measurement-oracle convention (fail loud in
  a lane), not a defect; `serve.rs`'s 12 are mutex-poisoning fail-fast + self-built
  hex decode + one compile-time fixture — verified benign at filing.
- [x] **T0.** This filing + the smell sweep that grounded it (runner.rs structure,
  serve/engine unwrap census, allow-attr census — 34, all comment-guarded or
  deliberate).

Gate: pure refactor — byte-identical behavior asserted by the existing suite
(lib tests + clippy ×3 postures + rustfmt); no bench owed (no perf claim).
