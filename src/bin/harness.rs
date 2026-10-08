//! `harness` — the Plan 603 T1.5 runner CLI.
//!
//! Usage:
//! ```text
//! cargo run --release --bin harness -- [--suites a,b] [--laya-max-questions N]
//!                                      [--skip-laya] [--dump-items] [--laya-python] [--out DIR]
//!                                      [--corpus-cap N] [--cal-select-cap [LIST]]
//!                                      [--head-scale F] [--head-select]
//!                                      [--nb-select] [--oc-select]
//!                                      [--ridge-select] [--genome-select]
//!                                      [--genome-accept-margin F]
//!                                      [--pair-head-ab] [--nli-feature-ab]
//!                                      [--nli-m1]
//!                                      [--mc-ab] [--mc-samples N]
//!                                      [--mc-p-drop P] [--mc-lambda L]
//!                                      (mc_* need --features mc_ensemble; absent
//!                                       build → the flags refuse as unknown)
//!                                      [--density-gate] (needs --features
//!                                       density_gate; absent build → the flag
//!                                       refuses LOUD at run start)
//!                                      [--runs-kv] [--kv-dir DIR] [--save-corpus a,b]
//!                              [--clm] [--gliner] [--agentjev] [--openthai] [--drex]
//!                                      [--d1] [--paw] [--paw-local] [--cascade]
//!                                      [--cascade-worthiness-lcb F]
//!                                      [--gate-fit-selection] [--gate-distance-only]
//!                                      [--no-gate-fit-calibrated] [--no-kn-route]
//!                                      [--drafter-fix <off|per_byte|ncd|shared_prefix|key_only>]
//! ```
//! `--laya-python` adds the ORIGINAL torch reference as a JSONL subprocess
//! oracle lane (measurement-only; needs python3 + torch/transformers and the
//! weights in the shared cache — absent pieces are loud absences, never
//! silent skips).
//! `--clm` adds the CLM comparison lane (Issue 019 T3 / `.issues/027`):
//! the external Contrastive-LM reference over `/v1/systemone` (their stack
//! serving at `CLM_SERVE_URL`, default `http://127.0.0.1:8700`). Needs the
//! `clm-lane` feature; an unreachable server is a loud absence, never a
//! silent skip.
//! `--gliner` adds the GLiNER comparison lane (Issue 029):
//! fastino/GLiNER2.5-Decide as a JSONL subprocess oracle over THEIR gliner2
//! package (`scripts/gliner_lane.py` — the venv needs gliner2 + torch +
//! transformers + peft + accelerate). Env: `GLINER_PYTHON` (the venv
//! python), `GLINER_PY_DEVICE` (default cuda), `GLINER_MODEL`. A missing
//! script/venv is a loud absence, never a silent skip.
//! `--bekko` adds the Bekko comparison lane (Bench 103, owner call
//! 2026-10-01): hotchpotch/bekko-system-one-v0 (17M/68M/400M shared-prefix
//! encoders — the same choice/noul/score decision vocabulary as the wire)
//! as a JSONL subprocess oracle over THEIR `BekkoSentenceTransformer`
//! runtime (`scripts/bekko_lane.py` — the venv needs torch, transformers
//! and sentence-transformers, the card's runtime pins). Env: `BEKKO_PYTHON`
//! (the venv python), `BEKKO_PY_DEVICE` (default cpu — the author's
//! reference posture), `BEKKO_MODEL` + `BEKKO_REVISION` (default the
//! card's 17M release). License: MIT (verified 2026-10-02,
//! `hotchpotch/bekko-system-one` LICENSE — Copyright (c) 2026 Yuichi Tatsumi)
//! — a comparison lane today; teacher/distill use is license-clear with
//! attribution (riir-train Issue 608). A missing script/venv is a loud
//! absence, never a silent skip.
//! `--agentjev` adds the AgentJev comparison lane (Issue 025 amendment 4 /
//! `.issues/027`): their `jev_service` (malevrigns/agent-jev @ a965ca8f,
//! Apache-2.0, not affiliated) answered over HTTP at `AGENTJEV_SERVE_URL`
//! (default `http://127.0.0.1:8149`) — their stack serves, our Rust
//! measures. The lane's owned data point: AgentJev's GOLD-LABEL accuracy
//! on our split (their published 79.25% is teacher-argmax agreement — a
//! different protocol). An unreachable server is a loud absence, never a
//! silent skip.
//! `--clef` adds the Cloudflare Clef comparison lane (plan 011 Phase A,
//! from `.research/005_…`): their Workers-AI-hosted decision models
//! (`Cloudflare/clef` + `clef-flash`, Apache-2.0, not affiliated) answered
//! over the Jev-shaped wire behind an operator-run loopback TLS forwarder
//! (`CLEF_SERVE_URL`, default `http://127.0.0.1:8791` — the hop is part of
//! the row's latency claim and is disclosed in the run posture). Missing
//! creds (`CLEF_ACCOUNT_ID` or `CLEF_RUN_PATH`) refuse at lane
//! construction; the spend ceiling `CLEF_SMOKE_MAX_CASES` (default 50
//! cases; `CLEF_ALLOW_UNCAPPED=1` overrides) bounds undisclosed pricing.
//! Other env: `CLEF_API_TOKEN` (bearer; the forwarder may inject),
//! `CLEF_MODEL` (`clef` default | `clef-flash`), `CLEF_TIMEOUT_MS`.
//! `--drex` adds the Drex DLM comparison lane (Issue 073, from
//! `.research/008`): Nace.AI's open-weights 8B diffusion-LM decision model
//! (`nace-ai/drex-dlm` @ `6c63df2`, Efficient-DLM-8B backbone — repo MIT,
//! weights **CC BY-NC 4.0: MEASUREMENT ONLY**, never a product lane, never
//! a distill teacher) answered over the TypeSafe `/v1/systemone` wire at
//! `DREX_SERVE_URL` (default `http://127.0.0.1:8000`, their Python
//! `serve.py`; their llama.cpp `edlm` fork's 8097 equally valid — the
//! reply's own `model` field discloses which served). Their stack serves,
//! our Rust measures; the owned data points: gold-label accuracy per
//! primitive on our split + the ECE of their `confidence` fields (their
//! card disclaims calibration — Issue 073 T4). An unreachable server is a
//! loud absence, never a silent skip.
//!
//! `--d1` adds the LiquidAI d1 comparison lane (Issue 078, from
//! `.research/009`): d1-3B (`LiquidAI/d1-3B` @ `051bcc4`, LFM2.5-VL-3B
//! backbone — license **`other`/lfm1.0: MEASUREMENT ONLY**, never a
//! product lane) answered over HTTP at `D1_SERVE_URL` (default
//! `http://127.0.0.1:8078` — our reference stdlib server over their
//! in-repo `D1Model`, which ships no HTTP layer). Their official
//! `/decisions/v1/systemone` wire; the served dtype + calibration posture
//! ride the run meta. An unreachable server is a loud absence, never a
//! silent skip.
//! `--paw` adds the PAW comparison lane (Issue 033): ProgramAsWeights
//! (MIT SDK, not affiliated) — one program compiled per specced suite from
//! the committed `scripts/paw_specs/<suite>.txt` (cached by
//! `(suite, compiler, BLAKE3(spec))` in `.raw/paw/programs.json`, so a
//! re-run never recompiles), then one hosted `/api/v1/infer` per question
//! over a `curl` subprocess. `PAW_API_KEY` is OPTIONAL — unset = their
//! anonymous tier (20 compiles/h; programs are PUBLIC on their hub), printed
//! loud. Env: `PAW_API_URL`, `PAW_COMPILER`, `PAW_COMPILE_ASYNC=1`,
//! `PAW_SPECS_DIR`, `PAW_PROGRAM_CACHE`, `PAW_CURL`. Free-text answers are
//! mapped by the exact-match law; unparseable = a counted refusal, never a
//! guess. Suites without a spec are a loud absence.
//! `--paw-local` adds the PAW Posture B lane (Issue 033): the SAME
//! compiled programs answered through their LOCAL llama.cpp runtime
//! (`paw.function(program_id)` over the `programasweights` package as a
//! Python subprocess oracle — the gliner-lane shape). The lane NEVER
//! compiles: the program id comes from the hosted lane's cache, so the
//! local-vs-hosted delta isolates the runtime posture on identical
//! artifacts. Greedy by construction; determinism re-verified per run
//! (observed-repeat, first 10). Env: `PAW_LOCAL_PYTHON` (the venv
//! interpreter, e.g. `.raw/paw-env/Scripts/python.exe`), `PAW_LOCAL_SCRIPT`
//! (default `scripts/paw_local_lane.py`), `PAW_GPU_LAYERS`/`PAW_LOCAL_N_CTX`
//! (their loader's knobs). One-time setup: the venv + a `--paw` run for the
//! program cache (`scripts/paw_preload.py` warms the base download).
//! `--runs-kv` appends ONE Warm-tier row per run via the released `ndb`
//! binary (table `harness_runs`, value = the exact results.json bytes) and
//! `--save-corpus` stores each named suite's dataset as ONE digest-pinned
//! row — both need the `corpus_db` feature + the binary (NDB_BIN or PATH);
//! an explicit flag without either refuses LOUD, never silently skips
//! (Issue 007 P1).
//! `--corpus-cap N` pins every dataset suite's per-label corpus cap
//! (measurement-only, Issue 013 lever 1). `--cal-select-cap [LIST]` is the
//! protocol-clean alternative: cal accuracy is measured at each candidate
//! cap (LIST, or the default ladder 8,16,32,64,128,256,512 — the registry
//! default always joins), the argmax is picked on the CAL SLICE ONLY, and
//! the test split is read once at the selected cap. The two are mutually
//! exclusive (run() refuses the combination).
//! `--cascade` (needs `laya-riir`, mutually exclusive with `--skip-laya`)
//! adds the cascade lane (issue 038 T4′): the modelless answers stand;
//! the calibrated fused gate's abstains escalate to each served riir-laya
//! checkpoint. Accuracy AND the escalation rate per suite (the rate is
//! the latency claim); LLM-only families get an honest absence.
//! `--cascade-worthiness` (needs `--cascade`, issue 042 lever 3) gates
//! the escalation per suite by a cal-slice probe: the checkpoint answers
//! the cal questions the calibrated gate abstained on, and the suite's
//! escalation stays armed only where it reads ≥ the forced modelless
//! picks (`--cascade-worthiness-margin <F64>`, default 0.16 — the
//! Bench-063/066 measured bar, promoted per the issue's own trigger; 0.0
//! restores arm-at-parity). A negative
//! probe disarms the suite's escalation (disclosed in the row); missing
//! cal records / thin support stay armed with the named reason.
//!
//! `--cascade-worthiness-lcb <F64>` (needs `--cascade-worthiness`,
//! issue 046 lever 4) adds a support-aware arm leg: the suite also arms
//! where the probe delta's one-sided-95% lower bound ≥ F — the
//! Bench-070 verified answer to the fused family's ag_news/massive
//! magnitude tie (support separates what the margin provably cannot;
//! conservative on paired data; the Bench-063 preregistered floor is
//! 0.05).
//!
//! `--gate-fit-selection` (issue 042 lever 1) fits the fused-gate
//! thresholds on the STRATIFIED selection slice instead of the train-tail
//! cal slice — the shared held-out instrument, with the probe corpus
//! excluding the fit docs so the observed geometry matches the test side.
//! `--gate-distance-only` (issue 042 lever 2) disables the gate's score
//! axis (threshold 0.0): abstain/escalation runs on the corpus-distance
//! axis alone, at its fitted rho=30% threshold — the transfer-stable
//! axis (Bench 061). Both are gate postures, composable with the cascade
//! lane; default off = the shipped T1.6 fused fit.
//! `--e0` (needs `nb_scope`) runs the riir-instinct Issue-005 E0
//! evidence-density measurement INSTEAD of the lanes: per dataset suite, on
//! the stratified selection slice, the distribution of seen-token counts
//! over the deployed count tables + the rumor fraction — report only, no
//! gold, no test-row eval. Writes `e0.json` + `E0.md` into `--out`.
//! Writes `results.json` + `TABLES.md` into `--out`
//! (default `.benchmarks/001_phase1_tables/`). Datasets come from
//! `.raw/datasets/` (scripts/fetch_datasets.sh). Exit 0 iff every requested
//! suite ran BOTH lanes clean — absences are printed loud and listed in the
//! tables, never silently dropped (the frontier-report law).

use riir_reflex::harness::runner::{self, DEFAULT_DATASETS_DIR, RunOptions};

/// Harness stack size. The runner keeps several `DecisionEngine<N, 256>`
/// values live per suite (probe, raw, cal, fitted, plus issue 038's
/// transductive build), each carrying `N` experts by value. On the
/// 77-domain banking77 that overflowed the 1 MiB Windows main-thread stack
/// (measured on the 4090, 2026-09-26) while the 8 MiB macOS default fit.
/// One explicit size for every platform instead of per-OS luck.
const HARNESS_STACK_BYTES: usize = 64 << 20;

fn main() {
    let worker = std::thread::Builder::new()
        .name("harness".to_string())
        .stack_size(HARNESS_STACK_BYTES)
        .spawn(harness_main)
        .expect("spawn the harness thread");
    if worker.join().is_err() {
        // The panic message already printed on the worker thread.
        std::process::exit(101);
    }
}

fn harness_main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = RunOptions {
        datasets_dir: std::env::current_dir()
            .map(|d| d.join(DEFAULT_DATASETS_DIR))
            .unwrap_or_else(|_| DEFAULT_DATASETS_DIR.into()),
        suites: Vec::new(),
        laya_max_questions: 0,
        skip_laya: false,
        dump_items: false,
        cascade: false,
        cascade_worthiness: false,
        // Issue 042 T3 (a), promoted per the issue's own trigger (a second
        // independent lane run reproduced 10/10 at 0.16 — byte-identical
        // probe deltas, recorded in the 042 HISTORY row): the measured
        // arm bar sits in the (0.150, 0.288] gap that separated every
        // probe family measured (Bench 063 fused, Bench 066 distance +
        // combined). An explicit --cascade-worthiness-margin overrides.
        cascade_worthiness_margin: 0.16,
        // Issue 046 lever 4: the support-aware arm leg — off by default
        // (byte-identical with the lever-3 lane).
        cascade_worthiness_lcb: None,
        gate_fit_selection: false,
        gate_distance_only: false,
        // Issue 056 direction 0 — PROMOTED default-on 2026-09-30 (verdict
        // round 2, the units-bug adjudication): the gate-fit probe observes
        // on the SAME scale the deployed gate applies. At a monotone fit the
        // percentile-coherent threshold reproduces the raw gate's fitted
        // target exactly, so this can never lose to the pre-calibration
        // baseline. `--no-gate-fit-calibrated` restores the old
        // fit-on-raw/apply-on-calibrated posture for measurement.
        gate_fit_calibrated: true,
        // Issue 079 — the legacy `k == N` index-alignment route binding is
        // the incumbent default; `--no-kn-route` runs the content-bound
        // posture (route terms arm only via by-name resolution) so both
        // sides of the bench-128 position-binding finding are measurable
        // in one binary.
        kn_route: true,
        // Issue 079 — the drafter-only corrections are OFF by default (the
        // shipped scores, byte-identical); `--drafter-fix <mode>` arms one
        // for the content-bound posture's tuning axis.
        drafter_fix: riir_reflex::engine::DrafterFix::Off,
        laya_python: false,
        clm: false,
        gliner: false,
        bekko: false,
        agentjev: false,
        clef: false,
        openthai: false,
        drex: false,
        d1: false,
        paw: false,
        paw_local: false,
        corpus_cap_override: 0,
        cal_select_caps: Vec::new(),
        pair_head_ab: false,
        nli_feature_ab: false,
        nli_m1: false,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: false,
        #[cfg(feature = "mc_ensemble")]
        mc_samples: 0,
        #[cfg(feature = "mc_ensemble")]
        mc_p_drop: None,
        #[cfg(feature = "mc_ensemble")]
        mc_lambda: None,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        oc_select: false,
        ridge_select: false,
        genome_select: false,
        genome_accept_margin: 0.05,
        density_gate: false,
    };
    let mut out_dir = std::path::PathBuf::from(".benchmarks/001_phase1_tables");
    let mut runs_kv = false;
    let mut kv_dir: Option<std::path::PathBuf> = None;
    let mut save_corpus: Vec<String> = Vec::new();
    let mut e0 = false;
    // Issue 077 — the option-permutation spread probe (exclusive early-exit
    // mode; the comparison lanes reuse --drex/--agentjev).
    let mut perm_probe = false;
    let mut perm_k = 0usize;
    let mut perm_max_cases = 0usize;
    let mut distill = false;
    // The distill mode's knobs (Plan 426 T1: the mode is no longer
    // laya-only — the openthai teacher runs on a default-features build,
    // so the parsing is ungated; the LAYA teacher still refuses without
    // the feature, loud).
    let mut distill_teacher = "laya".to_string();
    let mut ensemble_gate = false;
    let mut ensemble_out = std::path::PathBuf::from(".raw/ensemble_gate");
    let mut distill_out = std::path::PathBuf::from(".raw/distill_teacher");
    let mut distill_limit = 0usize;
    // Plan 426 T5: the synth + corpus-AB modes (exclusive early-exit like
    // --e0/--distill).
    let mut synth_corpus = false;
    let mut synth_plan = false;
    let mut synth_density_pilot = false;
    let mut corpus_ab: Option<std::path::PathBuf> = None;
    let mut synth_out = std::path::PathBuf::from(".raw/corpus_synth");
    let mut synth_teacher = "openthai".to_string();
    let mut synth_max = 2048usize;
    let mut synth_per_label = 128usize;
    let mut synth_span = 4usize;
    let mut synth_extra_cap = 128usize;
    let mut synth_density_gate: Option<runner::DensityRung> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--suites" => {
                i += 1;
                opts.suites = args
                    .get(i)
                    .unwrap_or_else(|| die("--suites needs a comma list"))
                    .split(',')
                    .map(str::to_string)
                    .collect();
            }
            "--laya-max-questions" => {
                i += 1;
                opts.laya_max_questions = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--laya-max-questions needs a number"));
            }
            "--skip-laya" => opts.skip_laya = true,
            "--dump-items" => opts.dump_items = true,
            "--cascade" => opts.cascade = true,
            "--cascade-worthiness" => opts.cascade_worthiness = true,
            "--gate-fit-selection" => opts.gate_fit_selection = true,
            "--gate-distance-only" => opts.gate_distance_only = true,
            "--gate-fit-calibrated" => opts.gate_fit_calibrated = true,
            "--no-gate-fit-calibrated" => opts.gate_fit_calibrated = false,
            "--no-kn-route" => opts.kn_route = false,
            "--drafter-fix" => {
                i += 1;
                let mode = args
                    .get(i)
                    .unwrap_or_else(|| die("--drafter-fix needs a mode"))
                    .clone();
                opts.drafter_fix = riir_reflex::engine::DrafterFix::from_spelling(&mode)
                    .unwrap_or_else(|| {
                        die("unknown --drafter-fix mode (expected off|per_byte|ncd|shared_prefix|key_only)")
                    });
            }
            "--cascade-worthiness-margin" => {
                i += 1;
                opts.cascade_worthiness_margin = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--cascade-worthiness-margin needs a number"));
            }
            "--cascade-worthiness-lcb" => {
                i += 1;
                opts.cascade_worthiness_lcb = Some(
                    args.get(i)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or_else(|| die("--cascade-worthiness-lcb needs a number")),
                );
            }
            "--pair-head-ab" => opts.pair_head_ab = true,
            "--nli-feature-ab" => opts.nli_feature_ab = true,
            "--nli-m1" => opts.nli_m1 = true,
            #[cfg(feature = "mc_ensemble")]
            "--mc-ab" => opts.mc_ab = true,
            #[cfg(feature = "mc_ensemble")]
            "--mc-samples" => {
                i += 1;
                opts.mc_samples = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--mc-samples needs a number (0 = default 8)"));
            }
            #[cfg(feature = "mc_ensemble")]
            "--mc-p-drop" => {
                i += 1;
                let p: f32 = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--mc-p-drop needs a fraction in (0, 0.5]"));
                if !(p > 0.0 && p <= 0.5) {
                    die("--mc-p-drop must lie in (0, 0.5]");
                }
                opts.mc_p_drop = Some(p);
            }
            #[cfg(feature = "mc_ensemble")]
            "--mc-lambda" => {
                i += 1;
                let l: f32 = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--mc-lambda needs a number >= 0"));
                if l < 0.0 {
                    die("--mc-lambda must be >= 0");
                }
                opts.mc_lambda = Some(l);
            }
            "--head-scale" => {
                i += 1;
                opts.head_scale = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--head-scale needs a number (0 = off)"));
            }
            "--head-select" => opts.head_select = true,
            "--nb-select" => opts.nb_select = true,
            "--oc-select" => opts.oc_select = true,
            "--ridge-select" => opts.ridge_select = true,
            "--genome-select" => opts.genome_select = true,
            // Issue 066: the fused-abstain density-half A/B (report-only;
            // needs the density_gate feature — run() refuses loud without
            // it, never a silent skip).
            "--density-gate" => opts.density_gate = true,
            "--genome-accept-margin" => {
                i += 1;
                opts.genome_accept_margin = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--genome-accept-margin needs a fraction (e.g. 0.03)"));
            }
            "--e0" => e0 = true,
            "--perm-probe" => perm_probe = true,
            "--perm-k" => {
                i += 1;
                perm_k = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--perm-k needs a number (orderings incl. identity)"));
            }
            "--perm-max-cases" => {
                i += 1;
                perm_max_cases = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--perm-max-cases needs a number"));
            }
            "--distill" => distill = true,
            "--ensemble-gate" => ensemble_gate = true,
            "--ensemble-out" => {
                i += 1;
                ensemble_out = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--ensemble-out needs a path"));
            }
            "--distill-teacher" => {
                i += 1;
                distill_teacher = args
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| die("--distill-teacher needs a name (laya | openthai | bekko)"));
            }
            "--distill-out" => {
                i += 1;
                distill_out = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--distill-out needs a path"));
            }
            "--synth-corpus" => synth_corpus = true,
            "--synth-plan" => synth_plan = true,
            "--synth-density-pilot" => synth_density_pilot = true,
            "--synth-teacher" => {
                i += 1;
                synth_teacher = args
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| die("--synth-teacher needs a name (laya | openthai | bekko)"));
            }
            "--synth-max" => {
                i += 1;
                synth_max = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .filter(|&v| v > 0)
                    .unwrap_or_else(|| die("--synth-max needs a number > 0"));
            }
            "--synth-per-label" => {
                i += 1;
                synth_per_label = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .filter(|&v| v > 0)
                    .unwrap_or_else(|| die("--synth-per-label needs a number > 0"));
            }
            "--synth-span" => {
                i += 1;
                synth_span = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .filter(|&v| v > 0)
                    .unwrap_or_else(|| die("--synth-span needs a number > 0 (tokens)"));
            }
            "--synth-out" => {
                i += 1;
                synth_out = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--synth-out needs a path"));
            }
            "--synth-extra-cap" => {
                i += 1;
                synth_extra_cap = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .filter(|&v| v > 0)
                    .unwrap_or_else(|| die("--synth-extra-cap needs a number > 0"));
            }
            "--synth-density-gate" => {
                i += 1;
                let tok = args
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| die("--synth-density-gate needs a rung (p50 | p75 | p90)"));
                synth_density_gate = Some(runner::DensityRung::from_token(&tok).unwrap_or_else(|| {
                    die("--synth-density-gate needs a rung (p50 | p75 | p90)")
                }));
            }
            "--corpus-ab" => {
                i += 1;
                corpus_ab = Some(
                    args.get(i)
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| die("--corpus-ab needs the synth artifact path")),
                );
            }
            "--limit" => {
                i += 1;
                distill_limit = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--limit needs a number (0 = the whole split)"));
            }
            "--laya-python" => opts.laya_python = true,
            "--clm" => opts.clm = true,
            "--gliner" => opts.gliner = true,
            "--bekko" => opts.bekko = true,
            "--agentjev" => opts.agentjev = true,
            "--clef" => opts.clef = true,
            "--openthai" => opts.openthai = true,
            "--drex" => opts.drex = true,
            "--d1" => opts.d1 = true,
            "--paw" => opts.paw = true,
            "--paw-local" => opts.paw_local = true,
            "--corpus-cap" => {
                i += 1;
                opts.corpus_cap_override = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| die("--corpus-cap needs a number (0 = registry default)"));
            }
            "--cal-select-cap" => {
                // Optional comma list; the bare flag = the default ladder.
                // A following non-numeric arg (e.g. --skip-laya) stays put.
                let list = args.get(i + 1).filter(|v| {
                    !v.is_empty() && v.split(',').all(|p| p.trim().parse::<usize>().is_ok())
                });
                let parsed = match list {
                    Some(v) => {
                        i += 1;
                        runner::parse_cal_select_caps(Some(v))
                    }
                    None => runner::parse_cal_select_caps(None),
                };
                opts.cal_select_caps = parsed.unwrap_or_else(|e| die(&e));
            }
            "--out" => {
                i += 1;
                out_dir = args
                    .get(i)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| die("--out needs a path"));
            }
            "--datasets-dir" => {
                // Issue 038 T2: read a non-canonical dataset pull (e.g. a
                // TRAIN_CAP=20000 fetch into its own dir). Disclosed in the
                // run meta's datasets line like the default.
                i += 1;
                opts.datasets_dir = args
                    .get(i)
                    .map(Into::into)
                    .unwrap_or_else(|| die("--datasets-dir needs a path"));
            }
            "--runs-kv" => runs_kv = true,
            "--kv-dir" => {
                i += 1;
                kv_dir = Some(
                    args.get(i)
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| die("--kv-dir needs a path")),
                );
            }
            "--save-corpus" => {
                i += 1;
                save_corpus = args
                    .get(i)
                    .unwrap_or_else(|| die("--save-corpus needs a comma list"))
                    .split(',')
                    .map(str::to_string)
                    .collect();
            }
            other => die(&format!("unknown flag {other}")),
        }
        i += 1;
    }

    // The kv flags are EXPLICIT ops — a compile-time-missing feature is a
    // loud refusal naming the rebuild, never an unknown-flag error.
    if (runs_kv || !save_corpus.is_empty()) && !cfg!(feature = "corpus_db") {
        die(
            "--runs-kv / --save-corpus need the `corpus_db` feature — rebuild: \
             cargo build --release --features corpus_db --bin harness",
        );
    }
    #[cfg(not(feature = "corpus_db"))]
    let _ = kv_dir;

    // Issue 005 (riir-instinct) E0: an EXCLUSIVE early-exit measurement
    // mode — evidence density on the stratified selection slice, report
    // only, before any lane machinery runs. Needs `nb_scope` (it reads the
    // count tables' fit-time seen set); a missing feature is a loud refusal
    // naming the rebuild (the build-stamp law).
    #[cfg(feature = "nb_scope")]
    if e0 {
        println!(
            "harness --e0: datasets {} · suites {:?}",
            opts.datasets_dir.display(),
            opts.suites
        );
        let out = match runner::run_e0(&opts) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            die(&format!("create {}: {e}", out_dir.display()));
        }
        let json_path = out_dir.join("e0.json");
        let md_path = out_dir.join("E0.md");
        let json = serde_json::to_string_pretty(&out).expect("e0 serialize");
        let md = runner::render_e0_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --e0: absence: {e}");
        }
        println!(
            "harness --e0: PASSED — {} suite(s) measured, {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    #[cfg(not(feature = "nb_scope"))]
    if e0 {
        die(
            "--e0 needs the `nb_scope` feature — rebuild: cargo build --release \
             --features nb_scope --bin harness",
        );
    }
    if e0 && distill {
        die("--e0 and --distill are both exclusive early-exit modes — pass one");
    }
    // Issue 077 — the option-PERMUTATION spread probe: an exclusive
    // early-exit measurement mode. The comparison lanes ride the existing
    // --drex / --d1 / --agentjev flags (their construction refuses loud
    // when a server is unreachable — the comparison-lane law); the laya
    // lane runs whenever its feature is compiled unless --skip-laya. The
    // modelless control reds the RUN (exit 1) — byte-identity across
    // orderings is an engine invariant; a control red is a finding or a
    // harness bug, never a quiet pass.
    if perm_probe {
        if perm_k != 0 && perm_k < 2 {
            die("--perm-k needs >= 2 orderings (the identity + at least one shuffle)");
        }
        let popts = runner::PermProbeOptions {
            k: if perm_k == 0 {
                runner::DEFAULT_PERM_K
            } else {
                perm_k
            },
            max_cases: if perm_max_cases == 0 {
                runner::DEFAULT_PERM_MAX_CASES
            } else {
                perm_max_cases
            },
            drex: opts.drex,
            d1: opts.d1,
            agentjev: opts.agentjev,
        };
        println!(
            "harness --perm-probe: datasets {} · suites {:?} · K {} orderings · \
             ≤{} case(s)/suite · lanes: modelless (control){}{}{}",
            opts.datasets_dir.display(),
            opts.suites,
            popts.k,
            popts.max_cases,
            if popts.drex { " +drex" } else { "" },
            if popts.d1 { " +d1" } else { "" },
            if popts.agentjev { " +agentjev" } else { "" },
        );
        let out = match runner::run_perm_probe(&opts, &popts) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            die(&format!("create {}: {e}", out_dir.display()));
        }
        let json_path = out_dir.join("perm_probe.json");
        let md_path = out_dir.join("PERM_PROBE.md");
        let json = serde_json::to_string_pretty(&out).expect("perm probe serialize");
        let md = runner::render_perm_probe_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        let control_red = out.suites.iter().any(|s| {
            s.lanes
                .iter()
                .any(|l| l.control && l.verdict == runner::SpreadVerdict::Red)
        });
        if control_red {
            die(
                "--perm-probe: the modelless CONTROL red — the top pick moved \
                 on an untied slot or the median swing left the L1 normalizer's \
                 fp envelope. This is a finding or a harness bug; the record is \
                 on disk (never a quiet pass).",
            );
        }
        println!(
            "harness --perm-probe: DONE — {} suite(s), {} RED lane cell(s) \
             disclosed; wrote {} + {}",
            out.suites.len(),
            out.red_cells,
            json_path.display(),
            md_path.display()
        );
        return;
    }
    {
        // The exclusive early-exit modes (one per run — the e0/distill law
        // generalized as the family grew).
        let modes: [(&str, bool); 8] = [
            ("--e0", e0),
            ("--perm-probe", perm_probe),
            ("--distill", distill),
            ("--ensemble-gate", ensemble_gate),
            ("--synth-corpus", synth_corpus),
            ("--synth-plan", synth_plan),
            ("--synth-density-pilot", synth_density_pilot),
            ("--corpus-ab", corpus_ab.is_some()),
        ];
        let active: Vec<&str> = modes
            .iter()
            .filter(|(_, on)| *on)
            .map(|(n, _)| *n)
            .collect();
        if active.len() > 1 {
            die(&format!(
                "{} are all exclusive early-exit modes — pass one",
                active.join(" + ")
            ));
        }
    }

    // riir-train Issue 576 T3: the Arm-B TEACHER pass — laya probabilities
    // over the train rows of the six Arm-A suites, dumped as frozen data
    // (magic `RIDT` + a `.blake3` sidecar per suite) for the distillation
    // student in ../riir-train. Early-exit like --e0: no eval lane, no test
    // row. Needs `laya-riir` (the lane); a missing feature is a loud refusal
    // naming the rebuild (the build-stamp law).
    if distill {
        // Plan 426 T1: the openthai teacher is a plain lane (no feature);
        // Issue 608: the bekko teacher likewise (the JSONL oracle — no
        // feature, the venv + script are its requirements). The laya
        // teacher keeps its loud feature refusal (the same message,
        // unchanged behavior for the existing posture).
        if distill_teacher == "laya" && !cfg!(feature = "laya-riir") {
            die(
                "--distill-teacher laya needs the `laya-riir` feature — rebuild: cargo build \
                 --release --features laya-riir-metal --bin harness (macOS; elsewhere: \
                 laya-riir)",
            );
        }
        #[cfg(feature = "laya-riir")]
        let run = runner::run_distill(&opts, &distill_out, distill_limit, &distill_teacher);
        #[cfg(not(feature = "laya-riir"))]
        let run = if matches!(distill_teacher.as_str(), "openthai" | "bekko") {
            runner::run_distill(&opts, &distill_out, distill_limit, &distill_teacher)
        } else {
            // Unreachable — the laya posture died above; kept for the
            // compiler's feature-less arm.
            unreachable!("laya without the feature refused above")
        };
        println!(
            "harness --distill: teacher {} · datasets {} · suites {:?} · limit {} · out {}",
            distill_teacher,
            opts.datasets_dir.display(),
            opts.suites,
            distill_limit,
            distill_out.display()
        );
        let out = match run {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&distill_out) {
            die(&format!("create {}: {e}", distill_out.display()));
        }
        let json_path = distill_out.join("distill.json");
        let md_path = distill_out.join("DISTILL.md");
        let json = serde_json::to_string_pretty(&out).expect("distill serialize");
        let md = runner::render_distill_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --distill: absence: {e}");
        }
        println!(
            "harness --distill: PASSED — {} suite(s) dumped, {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    if ensemble_gate {
        // Plan 426 T2 — the V3 gate: openthai ⊕ laya over the frozen test
        // slice, the pinned logit-mean primary + the rank alternate, the
        // paired LB95 against each member and the best one. The teacher
        // pair is FIXED here (the gate's question is about THESE two
        // heterogenous families; a third member is T3's owner-gated lane).
        println!(
            "harness --ensemble-gate: teachers openthai ⊕ laya · datasets {} · suites {:?} · out {}",
            opts.datasets_dir.display(),
            opts.suites,
            ensemble_out.display()
        );
        let out = match runner::run_ensemble_gate(&opts, "openthai", "laya") {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&ensemble_out) {
            die(&format!("create {}: {e}", ensemble_out.display()));
        }
        let json_path = ensemble_out.join("ensemble_gate.json");
        let md_path = ensemble_out.join("ENSEMBLE_GATE.md");
        let json = serde_json::to_string_pretty(&out).expect("ensemble serialize");
        let md = runner::render_ensemble_gate_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --ensemble-gate: absence: {e}");
        }
        println!(
            "harness --ensemble-gate: DONE — {} suite(s), {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    if synth_corpus || synth_plan {
        // Plan 426 T5 — the coverage-directed corpus synthesis lane.
        let laya_teacher = synth_teacher == "laya";
        if laya_teacher && !cfg!(feature = "laya-riir") {
            die(
                "--synth-teacher laya needs the `laya-riir` feature — rebuild: cargo build \
                 --release --features laya-riir-metal --bin harness (macOS; elsewhere: \
                 laya-riir)",
            );
        }
        let sopts = runner::SynthOptions {
            teacher: synth_teacher.clone(),
            max_accepted: synth_max,
            max_per_label: synth_per_label,
            max_span_len: synth_span,
            out_dir: synth_out.clone(),
            density_gate: synth_density_gate,
        };
        if synth_plan {
            println!(
                "harness --synth-plan: teacher {} · datasets {} · suites {:?}",
                synth_teacher,
                opts.datasets_dir.display(),
                opts.suites
            );
            let md = match runner::run_synth_plan(&opts, &sopts) {
                Ok(m) => m,
                Err(e) => die(&e),
            };
            print!("{md}");
            println!("harness --synth-plan: DONE — nothing written (report-only)");
            return;
        }
        println!(
            "harness --synth-corpus: teacher {} · max {} / {} per label · span ≤ {} · out {}",
            synth_teacher,
            synth_max,
            synth_per_label,
            synth_span,
            synth_out.display()
        );
        let out = match runner::run_synth_corpus(&opts, &sopts) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&synth_out) {
            die(&format!("create {}: {e}", synth_out.display()));
        }
        let json_path = synth_out.join("synth_report.json");
        let md_path = synth_out.join("SYNTH.md");
        let json = serde_json::to_string_pretty(&out).expect("synth serialize");
        let md = runner::render_synth_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --synth-corpus: absence: {e}");
        }
        println!(
            "harness --synth-corpus: DONE — {} suite(s), {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    if synth_density_pilot {
        // Issue 064 pilot — the learner-density ascent-leg MEASUREMENT
        // (report-only, no artifact, no teacher): score the transplant
        // candidates with the vMF corpus-density proxy over the engine's
        // own embedding and read the minimal-deviation accept ladder + the
        // <5% kill gate. Shares the synth family's knobs; only the span
        // bound affects the measurement.
        let sopts = runner::SynthOptions {
            teacher: synth_teacher.clone(),
            max_accepted: synth_max,
            max_per_label: synth_per_label,
            max_span_len: synth_span,
            out_dir: synth_out.clone(),
            density_gate: synth_density_gate,
        };
        println!(
            "harness --synth-density-pilot: datasets {} · suites {:?} · span ≤ {} · out {}",
            opts.datasets_dir.display(),
            opts.suites,
            synth_span,
            synth_out.display()
        );
        let out = match runner::run_density_pilot(&opts, &sopts) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&synth_out) {
            die(&format!("create {}: {e}", synth_out.display()));
        }
        let json_path = synth_out.join("density_pilot.json");
        let md_path = synth_out.join("DENSITY_PILOT.md");
        let json = serde_json::to_string_pretty(&out).expect("density pilot serialize");
        let md = runner::render_density_pilot_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        for e in &out.skipped {
            eprintln!("harness --synth-density-pilot: absence: {e}");
        }
        println!(
            "harness --synth-density-pilot: DONE — {} suite(s), {} absence(s); wrote {} + {}",
            out.suites.len(),
            out.skipped.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }
    if let Some(ab_path) = corpus_ab.clone() {
        // Plan 426 T5 — the V5 gate: gold-only vs +synth over ONE frozen
        // test read (paired LB95 + the aliveness anchor).
        println!(
            "harness --corpus-ab: artifact {} · datasets {} · extra-cap {}",
            ab_path.display(),
            opts.datasets_dir.display(),
            synth_extra_cap
        );
        let out = match runner::run_corpus_ab(&opts, &ab_path, synth_extra_cap) {
            Ok(r) => r,
            Err(e) => die(&e),
        };
        if let Err(e) = std::fs::create_dir_all(&synth_out) {
            die(&format!("create {}: {e}", synth_out.display()));
        }
        let json_path = synth_out.join("corpus_ab.json");
        let md_path = synth_out.join("CORPUS_AB.md");
        let json = serde_json::to_string_pretty(&out).expect("corpus-ab serialize");
        let md = runner::render_corpus_ab_markdown(&out);
        if let Err(e) = std::fs::write(&json_path, json) {
            die(&format!("write {}: {e}", json_path.display()));
        }
        if let Err(e) = std::fs::write(&md_path, &md) {
            die(&format!("write {}: {e}", md_path.display()));
        }
        print!("{md}");
        println!(
            "harness --corpus-ab: DONE — {} suite(s); wrote {} + {}",
            out.suites.len(),
            json_path.display(),
            md_path.display()
        );
        return;
    }

    // The clm flag is an EXPLICIT lane request — a compile-time-missing
    // feature is a loud refusal naming the rebuild (the build-stamp law),
    // never a silent column of absences.
    if opts.clm && !cfg!(feature = "clm-lane") {
        die(
            "--clm needs the `clm-lane` feature — rebuild: cargo build --release \
             --features clm-lane --bin harness (and their stack serving at \
             CLM_SERVE_URL, default http://127.0.0.1:8700 — .issues/027)",
        );
    }

    println!(
        "harness: datasets {} · suites {:?}",
        opts.datasets_dir.display(),
        opts.suites
    );
    let (output, errors) = match runner::run(&opts) {
        Ok(r) => r,
        Err(e) => die(&e),
    };

    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        die(&format!("create {}: {e}", out_dir.display()));
    }
    let json_path = out_dir.join("results.json");
    let md_path = out_dir.join("TABLES.md");
    let json = serde_json::to_string_pretty(&output).expect("results serialize");
    let md = runner::render_markdown(&output, &errors);
    if let Err(e) = std::fs::write(&json_path, json) {
        die(&format!("write {}: {e}", json_path.display()));
    }
    if let Err(e) = std::fs::write(&md_path, md) {
        die(&format!("write {}: {e}", md_path.display()));
    }
    println!("harness: wrote {}", json_path.display());
    println!("harness: wrote {}", md_path.display());

    // Issue 072 (`--dump-items`; rethink 028 T1): persist the per-item
    // records every lane materialized. One JSONL file per suite; one line
    // per (lane, question); join key `(case_id, q_idx)`. The paw lanes
    // keep their own result shape and are NOT dumped (a disclosed
    // absence — the lane list line below names what was written).
    if opts.dump_items {
        let items_dir = out_dir.join("items");
        if let Err(e) = std::fs::create_dir_all(&items_dir) {
            die(&format!("create {}: {e}", items_dir.display()));
        }
        let mut total_lines = 0usize;
        for suite in &output.suites {
            let path = items_dir.join(format!("{}.jsonl", suite.name));
            let mut buf = String::new();
            let mut lanes = 0usize;
            let mut write_lane = |lane: &runner::LaneResult, buf: &mut String| {
                for it in &lane.items {
                    buf.push_str(
                        &serde_json::to_string(it).expect("item outcome serialize"),
                    );
                    buf.push('\n');
                }
                lanes += 1;
            };
            if let Some(lane) = &suite.modelless {
                write_lane(lane, &mut buf);
            }
            for lane in suite.laya.values() {
                write_lane(lane, &mut buf);
            }
            for lane in [
                &suite.clm,
                &suite.gliner,
                &suite.bekko,
                &suite.agentjev,
                &suite.clef,
                &suite.openthai,
                &suite.drex,
                &suite.d1,
            ]
            .into_iter()
            .flatten()
            {
                write_lane(lane, &mut buf);
            }
            if let Err(e) = std::fs::write(&path, &buf) {
                die(&format!("write {}: {e}", path.display()));
            }
            let lines = buf.lines().count();
            total_lines += lines;
            println!(
                "harness: wrote {} ({} lane(s), {} item line(s))",
                path.display(),
                lanes,
                lines
            );
        }
        println!("harness: --dump-items total: {total_lines} item line(s)");
    }

    // Issue 007 P1: ONE run-history row per run — the exact results.json
    // bytes under a sortable date/sha/time key. Explicit-flag failures die
    // AFTER the files are on disk (the run happened; the persistence failed
    // — both facts are loud).
    #[cfg(feature = "corpus_db")]
    if runs_kv && let Err(e) = write_run_row(&output, &json_path, kv_dir.as_deref()) {
        die(&e);
    }
    #[cfg(feature = "corpus_db")]
    for name in &save_corpus {
        if let Err(e) = save_corpus_row(&opts.datasets_dir, name, kv_dir.as_deref()) {
            die(&e);
        }
    }

    if !errors.is_empty() {
        eprintln!("harness: {} absence(s)/error(s):", errors.len());
        for e in &errors {
            eprintln!("  - {e}");
        }
    }
    // Clean run = every suite present with BOTH lanes. Absences make the
    // run LOUD, never a silent green.
    if errors.is_empty() {
        println!(
            "harness: PASSED — {} suite(s), no absences",
            output.suites.len()
        );
    } else {
        println!(
            "harness: PASSED WITH ABSENCES — {} suite(s), {} absence(s) listed above and in TABLES.md",
            output.suites.len(),
            errors.len()
        );
    }
}

#[cfg(feature = "corpus_db")]
fn write_run_row(
    output: &riir_reflex::harness::runner::RunOutput,
    json_path: &std::path::Path,
    kv_dir: Option<&std::path::Path>,
) -> Result<(), String> {
    use riir_reflex::harness::corpus_db::{NdbRunStore, RUNS_TABLE};
    let kv_path = kv_dir
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(default_kv_dir);
    let store = NdbRunStore::resolve(&kv_path).map_err(|e| format!("--runs-kv: {e}"))?;
    let version = store.probe().map_err(|e| format!("--runs-kv: {e}"))?;
    let bytes = std::fs::read(json_path)
        .map_err(|e| format!("--runs-kv: read {}: {e}", json_path.display()))?;
    let key = format!(
        "{}/{}/{}",
        output.meta.date_utc,
        &output.meta.git_sha[..output.meta.git_sha.len().min(8)],
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let ack = store
        .put_row(RUNS_TABLE, &key, &bytes)
        .map_err(|e| format!("--runs-kv: {e}"))?;
    println!(
        "harness: runs.kv row {key} ({} bytes, ndb {}, store {})",
        ack.bytes,
        version.version,
        store.store_dir().display()
    );
    Ok(())
}

#[cfg(feature = "corpus_db")]
fn save_corpus_row(
    datasets_dir: &std::path::Path,
    name: &str,
    kv_dir: Option<&std::path::Path>,
) -> Result<(), String> {
    use riir_reflex::harness::corpus_db::NdbRunStore;
    let kv_path = kv_dir
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(default_kv_dir);
    let store = NdbRunStore::resolve(&kv_path).map_err(|e| format!("--save-corpus {name}: {e}"))?;
    let envelope = runner::load_suite_envelope(datasets_dir, name)
        .map_err(|e| format!("--save-corpus {name}: {e}"))?;
    let blob = serde_json::to_vec(&envelope)
        .map_err(|e| format!("--save-corpus {name}: serialize: {e}"))?;
    let (ack, digest) = store
        .put_corpus(name, &blob)
        .map_err(|e| format!("--save-corpus {name}: {e}"))?;
    // Read-back verification: the stored row must digest back to the pin.
    let read_back = store
        .get_corpus(name, &digest)
        .map_err(|e| format!("--save-corpus {name}: verify: {e}"))?;
    if read_back != blob {
        return Err(format!(
            "--save-corpus {name}: read-back digest mismatch — the stored row is \
             not the corpus that was written; refusing to report success"
        ));
    }
    println!(
        "harness: corpus row {}/{} ({} bytes, blake3 {}…, store {})",
        name,
        ack.store_key,
        ack.bytes,
        &digest[..16],
        store.store_dir().display()
    );
    Ok(())
}

#[cfg(feature = "corpus_db")]
fn default_kv_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(".harness/ndb-data")
}

fn die(msg: &str) -> ! {
    eprintln!("harness: {msg}");
    std::process::exit(2);
}
