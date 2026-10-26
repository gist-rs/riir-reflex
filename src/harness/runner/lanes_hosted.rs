//! The hosted comparison lanes carved from `runner.rs` (issue 098 T1):
//! OpenThai, Drex, D1, Jeva, StartLux, PPLX, D3, and the `TeacherWireLane`
//! family (elephant / mammoth / opendecider) with the Drex per-kind
//! confidence readout. Behavior-preserving carve - the bodies moved
//! verbatim; call sites in `runner.rs` keep the bare names via the glob
//! re-import.

#![allow(clippy::too_many_lines)]

use super::*;
/// The OpenThai comparison lane (Plan 003 Phase 2): their
/// `openthai_systemone` served on loopback, measured over HTTP — the
/// agentjev lane's shape (same cases, their contract via
/// `src/lanes/openthai.rs`, the same metrics tail through
/// [`assemble_laya_lane_result`]). Latency = client round-trip per case.
///
/// Provenance posture (T2.2, the agentjev divergence): no `/api/info` —
/// `GET /healthz` is the liveness probe and the model id comes from the
/// FIRST decide response's `model` field; their
/// `usage.input_tokens` + the abstain-slot count are reported beside the
/// metrics (provenance observations, never folded into probabilities).
///
/// Env: `OPENTHAI_SERVE_URL` (default `http://127.0.0.1:8000`).
pub(super) fn run_openthai_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::openthai::OpenThaiLane;

    let t_start = Instant::now();
    let lane = OpenThaiLane::default();
    lane.health()?;

    // WARMUP (the clm lane's cold-start law): one FIXED throwaway request
    // absorbs the service's cold path AND captures the model id (T2.2 —
    // the response's own `model` field, never a hardcoded name).
    let model;
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let raw = lane
            .decide_raw(&warm)
            .map_err(|e| format!("openthai warmup: {e}"))?;
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("openthai warmup response: {e}"))?;
        model = OpenThaiLane::model_of(&parsed);
        lane.decide(&warm)
            .map_err(|e| format!("openthai warmup: {e}"))?;
    }
    eprintln!("    [openthai] service up: {model}");

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N openthai row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut abstains: usize = 0;
    let mut input_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (outcome, _client_ms) = lane
            .decide(case)
            .map_err(|e| format!("openthai lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        abstains += outcome.abstains;
        input_tokens += outcome.input_tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("openthai determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("openthai determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(outcome.answers.len());
        let mut cpicks = Vec::with_capacity(outcome.answers.len());
        let mut cconfs = Vec::with_capacity(outcome.answers.len());
        for (p, pick, conf) in outcome.answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [openthai] abstain slots: {abstains} · input tokens: {input_tokens}");

    Ok(assemble_laya_lane_result(
        "openthai",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The Drex DLM comparison lane (Issue 073, from `.research/008`): Nace.AI's
/// open-weights 8B diffusion-LM decision model (CC BY-NC weights: MEASUREMENT
/// ONLY — comparison lane, never a product lane, never a distill teacher)
/// served on loopback, measured over HTTP — the openthai lane's shape on the
/// same TypeSafe `/v1/systemone` wire. Same cases, their contract
/// (`src/lanes/drex.rs` pins the mapping), the same metrics tail via
/// [`assemble_laya_lane_result`]. Latency = client round-trip per case;
/// their scorer-side `latency_ms` and billing-style token counts accumulate
/// as provenance lines beside it.
///
/// Env: `DREX_SERVE_URL` (default `http://127.0.0.1:8000`, their Python
/// `serve.py`; the llama.cpp `edlm` fork's 8097 equally valid — the reply's
/// own `model` field discloses which served). The `/health` probe + the
/// warmup capture the model id; never a hardcoded name (the gliner law).
pub(super) fn run_drex_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::drex::DrexLane;

    let t_start = Instant::now();
    let lane = DrexLane::default();
    lane.health()?;

    // WARMUP (the clm lane's cold-start law): one FIXED throwaway request
    // absorbs the service's cold path AND captures the model id (the
    // response's own `model` field — never a hardcoded name).
    let model;
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let raw = lane
            .decide_raw(&warm)
            .map_err(|e| format!("drex warmup: {e}"))?;
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("drex warmup response: {e}"))?;
        model = DrexLane::model_of(&parsed);
        lane.decide(&warm)
            .map_err(|e| format!("drex warmup: {e}"))?;
    }
    eprintln!("    [drex] service up: {model}");

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N drex row against
    // a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut their_confs: Vec<Vec<Option<f64>>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut server_latency_ms: f64 = 0.0;
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (outcome, _client_ms) = lane
            .decide(case)
            .map_err(|e| format!("drex lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        if let Some(w) = outcome.latency_ms {
            server_latency_ms += w;
        }
        input_tokens += outcome.input_tokens;
        output_tokens += outcome.output_tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column. Their own validation file measured GGUF-runner
        // wobble under load (the AgentJev bf16-HTTP-wobble class) — the
        // check is the flag, never an assumption.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("drex determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("drex determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(outcome.answers.len());
        let mut cpicks = Vec::with_capacity(outcome.answers.len());
        let mut cconfs = Vec::with_capacity(outcome.answers.len());
        for (p, pick, conf) in outcome.answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
        their_confs.push(outcome.confidences);
    }
    eprintln!(
        "    [drex] server latency sum: {server_latency_ms:.0} ms · in {input_tokens} tok · out {output_tokens} tok"
    );

    // Issue 076 T2: the per-kind cells over THEIR wire confidence + the
    // split-half conformal floor companion — the stdout line names them so
    // a run's transcript carries the audit, not just results.json.
    let readout = assemble_drex_conf_readout(cases, &picks, &their_confs);
    if let Some(r) = &readout {
        let cell = |c: &Option<crate::lanes::drex::DrexConfCell>| match c {
            Some(c) => format!("n={} ece={:.4} brier={:.4}", c.n, c.ece, c.brier),
            None => "absent (no wire confidence)".to_string(),
        };
        let floor = r.floor_ece.map_or_else(
            || "below the occupancy floor".to_string(),
            |e| format!("ece={e:.4}"),
        );
        eprintln!(
            "    [drex] their-confidence cells: choice {} · score {} · floor {} \
             (cal {} / test {})",
            cell(&r.choice),
            cell(&r.score),
            floor,
            r.floor_n_cal,
            r.floor_n_test
        );
    }

    let mut result = assemble_laya_lane_result(
        "drex",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    );
    result.drex_conf_readout = readout;
    Ok(result)
}

/// Env: `D1_SERVE_URL` (default `http://127.0.0.1:8078`, our reference
/// stdlib server `.raw/d1_server.py` over their in-repo `D1Model`). The
/// `/health` probe + the warmup capture the model id; never a hardcoded
/// name (the gliner law). The mirror of [`run_drex_lane`] on the d1
/// wire (Issue 078) — one shape, one metrics tail, the SAME shared
/// confidence-readout assembly.
pub(super) fn run_d1_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::d1::D1Lane;

    let t_start = Instant::now();
    let lane = D1Lane::default();
    lane.health()?;

    // WARMUP (the cold-start law, the drex lane's shape): one FIXED
    // throwaway request absorbs the service's cold path AND captures the
    // model id (the response's own `model` field — never a hardcoded
    // name).
    let model;
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let raw = lane
            .decide_raw(&warm)
            .map_err(|e| format!("d1 warmup: {e}"))?;
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("d1 warmup response: {e}"))?;
        model = D1Lane::model_of(&parsed);
        lane.decide(&warm).map_err(|e| format!("d1 warmup: {e}"))?;
    }
    eprintln!("    [d1] service up: {model}");

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N d1 row against
    // a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut their_confs: Vec<Vec<Option<f64>>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut server_latency_ms: f64 = 0.0;
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (outcome, _client_ms) = lane
            .decide(case)
            .map_err(|e| format!("d1 lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        if let Some(w) = outcome.latency_ms {
            server_latency_ms += w;
        }
        input_tokens += outcome.input_tokens;
        output_tokens += outcome.output_tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column. Single-pass logit reads promise determinism by
        // construction — the check is the flag, never an assumption.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("d1 determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("d1 determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(outcome.answers.len());
        let mut cpicks = Vec::with_capacity(outcome.answers.len());
        let mut cconfs = Vec::with_capacity(outcome.answers.len());
        for (p, pick, conf) in outcome.answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
        their_confs.push(outcome.confidences);
    }
    eprintln!(
        "    [d1] server latency sum: {server_latency_ms:.0} ms · in {input_tokens} tok · out {output_tokens} tok \
         (out is their ALWAYS-ZERO answer-slot law made visible)"
    );

    // Issue 076 T2's shared machinery (one assemble path for every
    // comparison lane): the per-kind cells over THEIR wire confidence +
    // the split-half conformal floor companion — the stdout line names
    // them so a run's transcript carries the audit. For d1 the wire
    // confidence IS the max probability; the readout adds the per-kind
    // split + the floor companion.
    let readout = assemble_drex_conf_readout(cases, &picks, &their_confs);
    if let Some(r) = &readout {
        let cell = |c: &Option<crate::lanes::drex::DrexConfCell>| match c {
            Some(c) => format!("n={} ece={:.4} brier={:.4}", c.n, c.ece, c.brier),
            None => "absent (no wire confidence)".to_string(),
        };
        let floor = r.floor_ece.map_or_else(
            || "below the occupancy floor".to_string(),
            |e| format!("ece={e:.4}"),
        );
        eprintln!(
            "    [d1] their-confidence cells: choice {} · score {} · floor {} \
             (cal {} / test {})",
            cell(&r.choice),
            cell(&r.score),
            floor,
            r.floor_n_cal,
            r.floor_n_test
        );
    }

    let mut result = assemble_laya_lane_result(
        "d1",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    );
    result.drex_conf_readout = readout;
    Ok(result)
}

/// Env: `JEVA_SERVE_URL` (default `http://127.0.0.1:8095`, OUR reference
/// stdlib server `scripts/jeva_server.py` over their in-repo `jeva.Jeva`
/// — the d1 reference-server law: their class, our listener). The
/// `/health` probe + the warmup capture the model id; never a hardcoded
/// name (the gliner law). The mirror of [`run_d1_lane`] on the jeva wire
/// (Issue 090 B1, feeding riir-train 632 T2) — one shape, one metrics
/// tail, the SAME shared confidence-readout assembly.
pub(super) fn run_jeva_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::jeva::JevaLane;

    let t_start = Instant::now();
    let lane = JevaLane::default();
    lane.health()?;

    // WARMUP (the cold-start law, the drex/d1 shape): one FIXED
    // throwaway request absorbs the service's cold path AND captures
    // the model id (the response's own `model` field — never a
    // hardcoded name).
    let model;
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let raw = lane
            .decide_raw(&warm)
            .map_err(|e| format!("jeva warmup: {e}"))?;
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("jeva warmup response: {e}"))?;
        model = JevaLane::model_of(&parsed);
        lane.decide(&warm)
            .map_err(|e| format!("jeva warmup: {e}"))?;
    }
    eprintln!("    [jeva] service up: {model}");

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N jeva row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut their_confs: Vec<Vec<Option<f64>>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // Plan 001 task 9: the repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut server_latency_ms: f64 = 0.0;
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (outcome, _client_ms) = lane
            .decide(case)
            .map_err(|e| format!("jeva lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        if let Some(w) = outcome.latency_ms {
            server_latency_ms += w;
        }
        input_tokens += outcome.input_tokens;
        output_tokens += outcome.output_tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law): a lane that cannot repeat byte-identically flags
        // its det column. Single-pass logit reads promise determinism by
        // construction — the check is the flag, never an assumption.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("jeva determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("jeva determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(outcome.answers.len());
        let mut cpicks = Vec::with_capacity(outcome.answers.len());
        let mut cconfs = Vec::with_capacity(outcome.answers.len());
        for (p, pick, conf) in outcome.answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
        their_confs.push(outcome.confidences);
    }
    eprintln!(
        "    [jeva] server latency sum: {server_latency_ms:.0} ms · in {input_tokens} tok · out {output_tokens} tok \
         (out is their ALWAYS-ZERO answer-slot law made visible)"
    );

    // Issue 076 T2's shared machinery (one assemble path for every
    // comparison lane): the per-kind cells over THEIR wire confidence +
    // the split-half conformal floor companion — the stdout line names
    // them so a run's transcript carries the audit. Jeva's confidence =
    // max(ρ, max joint) — read as-is (the ECE/Brier cells test it, never
    // assume it).
    let readout = assemble_drex_conf_readout(cases, &picks, &their_confs);
    if let Some(r) = &readout {
        let cell = |c: &Option<crate::lanes::drex::DrexConfCell>| match c {
            Some(c) => format!("n={} ece={:.4} brier={:.4}", c.n, c.ece, c.brier),
            None => "absent (no wire confidence)".to_string(),
        };
        let floor = r.floor_ece.map_or_else(
            || "below the occupancy floor".to_string(),
            |e| format!("ece={e:.4}"),
        );
        eprintln!(
            "    [jeva] their-confidence cells: choice {} · score {} · floor {} \
             (cal {} / test {})",
            cell(&r.choice),
            cell(&r.score),
            floor,
            r.floor_n_cal,
            r.floor_n_test
        );
    }

    let mut result = assemble_laya_lane_result(
        "jeva",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    );
    result.drex_conf_readout = readout;
    Ok(result)
}

/// The StartLux-Decision v1.1 comparison lane (reflex Issue 095, feeding
/// riir-train 635 T3): board #1 (`startlux-models/StartLux-Decision-v1.1-
/// 27B-Instant`; Qwen3.8-27B backbone, weights CC BY-NC 4.0 → DECLARED-NC
/// RESTRICTED — capture-for-measurement + teacher-capture, adoption
/// fail-closed per 635 T0), served through THEIR Apache-2.0 engine
/// `startlux-decide` over llama.cpp on the SAME TypeSafe `/v1/systemone`
/// wire as pplx/d3 (the wire functions are REUSED from the pplx lane —
/// one wire, many oracles), measured over HTTP — the pplx lane's shape
/// verbatim (same cases, the same metrics tail through
/// [`assemble_laya_lane_result`]).
///
/// Env: `STARTLUX_SERVE_URL` (default `http://127.0.0.1:8090`),
/// `STARTLUX_TIMEOUT_MS` (default 300000 — knockout multi-pass rows are
/// slow), `STARTLUX_DEVICE` (the quant-posture disclosure stamp — the
/// lossy-surface law: which quant produced the scores is PINNED and
/// quoted; our 635-T2 posture is Q4_K_M `-ngl 99`), `STARTLUX_API_KEY`
/// (optional bearer). The `/health` handshake GATES the posture: kind
/// "instant" + prompt_attention "bidirectional" + engine.bidir_mode
/// "server" — a causal/off serve misreads the Instant checkpoint (the
/// −26 pt class), and a keys-configured server omits the posture fields
/// (public shape) — both refuse loud naming the boot line. The no-clock
/// law: their body carries `timing_ms` only with `return_details`, which
/// this lane never sets, so the determinism compare is verbatim.
pub(super) fn run_startlux_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::startlux::StartluxLane;

    let t_start = Instant::now();
    let lane = StartluxLane::default();
    let (model, device) = lane.info()?;
    eprintln!("    [startlux] service up: {model} on {device}");

    // WARMUP (the clm lane's cold-start law, the pplx lane's shape): one
    // FIXED throwaway request (never a case's — no cache pollution of
    // measured latencies) absorbs the first-forward path (page-in,
    // llama-server's first non-causal eval).
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm)
            .map_err(|e| format!("startlux warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N startlux row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // The repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut input_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, tokens) = lane
            .decide(case)
            .map_err(|e| format!("startlux lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        input_tokens += tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law) — VERBATIM compare on this wire (the no-clock law).
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("startlux determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("startlux determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [startlux] input tokens summed: {input_tokens}");

    Ok(assemble_laya_lane_result(
        "startlux",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The mammoth/elephant hosted comparison lane (rethink Issue 032 T10 /
/// Research 012): Base Labs' mammoth-1 (the JDI 0.4.1 board-#1, MIT) or
/// elephant-1 on Baseten Model APIs, reached through the operator-run
/// loopback TLS forwarder (the clef A0 law — the hop is part of the row's
/// latency claim and is disclosed), the SAME `/v1/systemone` wire (the
/// wire functions REUSED from the pplx lane; the catalog id rides the
/// model field from `MAMMOTH_MODEL`). MEASUREMENT-ONLY: no RIDT arm — API
/// outputs never enter a capture artifact (the T10.2 law). The spend
/// ceiling (the clef A6 law): free week ≠ unmetered.
/// The teacher-wire lane surface the shared run core consumes: the
/// pplx-family `/v1/systemone` lanes (mammoth, opendecider) differ only in
/// construction, labels, spend-ceiling envs, and the determinism
/// projection — the loop machinery is ONE core (the DRY law; the
/// 2026-10-25 extraction on the opendecider landing, rethink Issue 038
/// T1; the mammoth path is behavior-preserving by construction — the same
/// statements, parameterized).
pub(super) trait TeacherWireLane {
    /// The lane label in errors, progress lines, and the record.
    fn label(&self) -> &'static str;
    /// The handshake: `(model_echo, device)`.
    fn info(&self) -> Result<(String, String), String>;
    /// One decision round: `(answer triples, client_ms, input_tokens)`.
    fn decide(
        &self,
        case: &SuiteCase,
    ) -> Result<(Vec<crate::lanes::pplx::AnswerTriple>, f64, u64), String>;
    /// The verbatim body (the determinism rerun's compare input).
    fn decide_raw(&self, case: &SuiteCase) -> Result<String, String>;
    /// The determinism-compare projection (lane-family policy, static).
    fn determinism_view(raw: &str) -> Result<String, String>;
    /// The spend-ceiling env names + default (the clef A6 law).
    fn spend_env(&self) -> &'static str;
    fn uncapped_env(&self) -> &'static str;
    fn max_cases_default(&self) -> usize;
}

/// The ONE shared loop: warmup, the trim law, the spend ceiling, the
/// decide loop with observed-repeat determinism (first 10 cases; the
/// reruns TRIPLE those cases' spend — disclosed), and the
/// [`assemble_laya_lane_result`] assembly. The per-lane surface is
/// [`TeacherWireLane`].
pub(super) fn run_teacher_wire_lane<L: TeacherWireLane>(
    lane: L,
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    let lane_name = lane.label();

    let t_start = Instant::now();
    let (model, device) = lane.info()?;
    eprintln!("    [{lane_name}] service up: {model} on {device}");

    // WARMUP (the clm lane's cold-start law, the clef shape): one FIXED
    // throwaway request absorbs the hosted route's cold path (a free-week
    // endpoint scales on demand — the first case must not pay it).
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm)
            .map_err(|e| format!("{lane_name} warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N mammoth row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    // The spend ceiling (the clef A6 law): a run above the default smoke
    // cap REFUSES unless explicitly overridden (the hosted lanes' spend
    // law; a local zero-marginal-cost serve records the uncapped posture
    // explicitly instead).
    let max_cases = std::env::var(lane.spend_env())
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| lane.max_cases_default());
    let uncapped = std::env::var(lane.uncapped_env()).is_ok_and(|v| v == "1");
    if cases.len() > max_cases && !uncapped {
        return Err(format!(
            "refusing: {} cases exceeds the {CAP} spend ceiling (active cap {}, default \
             {}) — raise {CAP} or set {UNC}=1 to run uncapped",
            cases.len(),
            max_cases,
            lane.max_cases_default(),
            CAP = lane.spend_env(),
            UNC = lane.uncapped_env(),
        ));
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    let mut determinism_n: usize = 0;
    let mut input_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, tokens) = lane
            .decide(case)
            .map_err(|e| format!("{lane_name} lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        input_tokens += tokens;

        // Observed-repeat determinism check, first 10 cases (the hosted
        // lanes' law — clef does the same): the reruns TRIPLE those cases'
        // spend (1 measured + 2 raw), disclosed here so the ceiling read
        // stays honest.
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("{lane_name} determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("{lane_name} determinism rerun: {e}"))?;
            // The compare projects through the lane's own view: the mammoth
            // family strips the qev serve-meta clock (measured 2026-10-24,
            // sst5 — answers byte-identical, the timing varies per call BY
            // DESIGN); the pplx/opendecider bodies carry no clock — verbatim
            // (the recorded divergence).
            let view1 = L::determinism_view(&raw1)
                .map_err(|e| format!("{lane_name} determinism rerun: {e}"))?;
            let view2 = L::determinism_view(&raw2)
                .map_err(|e| format!("{lane_name} determinism rerun: {e}"))?;
            if view1 != view2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [{lane_name}] input tokens summed: {input_tokens}");

    Ok(assemble_laya_lane_result(
        lane_name,
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

impl TeacherWireLane for crate::lanes::mammoth::MammothLane {
    fn label(&self) -> &'static str {
        "mammoth"
    }
    fn info(&self) -> Result<(String, String), String> {
        self.info()
    }
    fn decide(
        &self,
        case: &SuiteCase,
    ) -> Result<(Vec<crate::lanes::pplx::AnswerTriple>, f64, u64), String> {
        self.decide(case)
    }
    fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        self.decide_raw(case)
    }
    fn determinism_view(raw: &str) -> Result<String, String> {
        // The inherent associated fn (the qev serve-meta clock strip — the
        // full path keeps this unambiguous against the trait method).
        crate::lanes::mammoth::MammothLane::determinism_view(raw)
    }
    fn spend_env(&self) -> &'static str {
        crate::lanes::mammoth::MAX_CASES_ENV
    }
    fn uncapped_env(&self) -> &'static str {
        crate::lanes::mammoth::UNCAPPED_ENV
    }
    fn max_cases_default(&self) -> usize {
        crate::lanes::mammoth::DEFAULT_MAX_CASES
    }
}

impl TeacherWireLane for crate::lanes::elephant::ElephantLane {
    fn label(&self) -> &'static str {
        "elephant"
    }
    fn info(&self) -> Result<(String, String), String> {
        self.info()
    }
    fn decide(
        &self,
        case: &SuiteCase,
    ) -> Result<(Vec<crate::lanes::pplx::AnswerTriple>, f64, u64), String> {
        self.decide(case)
    }
    fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        self.decide_raw(case)
    }
    fn determinism_view(raw: &str) -> Result<String, String> {
        // The inherent associated fn (the reference server's top-level
        // `latency_ms` clock strip — the mammoth serve-meta class, one
        // level up).
        crate::lanes::elephant::ElephantLane::determinism_view(raw)
    }
    fn spend_env(&self) -> &'static str {
        crate::lanes::elephant::MAX_CASES_ENV
    }
    fn uncapped_env(&self) -> &'static str {
        crate::lanes::elephant::UNCAPPED_ENV
    }
    fn max_cases_default(&self) -> usize {
        crate::lanes::elephant::DEFAULT_MAX_CASES
    }
}

/// The elephant lane's thin construction + delegation (the shared core is
/// [`run_teacher_wire_lane`]). The SELF-HOSTED twin of
/// [`run_mammoth_lane`]: same loop, our reference server, no creds gate —
/// and it is ALSO the capture path (`--distill-teacher elephant`, landed
/// with riir-train 640 T2(a)); the comparison lane itself stays
/// measurement over the test splits (the no-cheat law).
pub(super) fn run_elephant_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    let lane = crate::lanes::elephant::ElephantLane::from_env()?;
    run_teacher_wire_lane(lane, suite, laya_max_questions, leak_flags)
}

/// The mammoth lane's thin construction + delegation (the shared core is
/// [`run_teacher_wire_lane`]).
pub(super) fn run_mammoth_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    let lane = crate::lanes::mammoth::MammothLane::from_env()?;
    run_teacher_wire_lane(lane, suite, laya_max_questions, leak_flags)
}

impl TeacherWireLane for crate::lanes::opendecider::OpenDeciderLane {
    fn label(&self) -> &'static str {
        "opendecider"
    }
    fn info(&self) -> Result<(String, String), String> {
        self.info()
    }
    fn decide(
        &self,
        case: &SuiteCase,
    ) -> Result<(Vec<crate::lanes::pplx::AnswerTriple>, f64, u64), String> {
        self.decide(case)
    }
    fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        self.decide_raw(case)
    }
    fn determinism_view(raw: &str) -> Result<String, String> {
        // Verbatim passthrough (their server is not the qev stack; the
        // 038-T1 smoke verifies before any measured read — the mammoth
        // precedent applies if a clock key ever shows up).
        crate::lanes::opendecider::OpenDeciderLane::determinism_view(raw)
    }
    fn spend_env(&self) -> &'static str {
        crate::lanes::opendecider::MAX_CASES_ENV
    }
    fn uncapped_env(&self) -> &'static str {
        crate::lanes::opendecider::UNCAPPED_ENV
    }
    fn max_cases_default(&self) -> usize {
        crate::lanes::opendecider::DEFAULT_MAX_CASES
    }
}

/// The opendecider lane's thin construction + delegation (rethink Issue
/// 038 T1; the P0 admission rig — the gate lives in 038, pre-registered).
pub(super) fn run_opendecider_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    let lane = crate::lanes::opendecider::OpenDeciderLane::from_env()?;
    run_teacher_wire_lane(lane, suite, laya_max_questions, leak_flags)
}
/// `pplx-decider-v1.1-27b` (the JDI 2026-10-07 #1; Apache-2.0, UNGATED —
/// teacher-eligible, the read decides riir-train 623's pick) served
/// through THEIR autojev server, measured over HTTP — the agentjev
/// lane's shape (same cases, the TypeSafe `/v1/systemone` wire via
/// `src/lanes/pplx.rs`, the same metrics tail through
/// [`assemble_laya_lane_result`]).
///
/// Env: `PPLX_SERVE_URL` (default `http://127.0.0.1:8793` — their server
/// binds `PORT`; serve with `PORT=8793` so the drex lane's 8000 stays
/// free), `PPLX_TIMEOUT_MS` (default 300000 — the offload posture is
/// slow per forward), `PPLX_DEVICE` (the posture stamp), `PPLX_API_KEY`
/// (optional bearer). The `/health` handshake captures the model id +
/// checkpoint basename (the serving provenance — never a hardcoded name,
/// the gliner law) and refuses while their lazy first load is still
/// `"loading"`. The no-clock law: their body carries no timing anywhere,
/// so the determinism compare is verbatim — the column measures compute
/// determinism (bf16 device wobble included), never their clock.
pub(super) fn run_pplx_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::pplx::PplxLane;

    let t_start = Instant::now();
    let lane = PplxLane::default();
    let (model, device) = lane.info()?;
    eprintln!("    [pplx] service up: {model} on {device}");

    // WARMUP (the clm lane's cold-start law): one FIXED throwaway request
    // (never a case's — no cache pollution of measured latencies) absorbs
    // the first-forward path (kernel compile / offload page-in).
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm)
            .map_err(|e| format!("pplx warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N pplx row
    // against a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // The repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut input_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, tokens) = lane
            .decide(case)
            .map_err(|e| format!("pplx lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        input_tokens += tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law) — VERBATIM compare on this wire (the no-clock law).
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("pplx determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("pplx determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [pplx] input tokens summed: {input_tokens}");

    Ok(assemble_laya_lane_result(
        "pplx",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The vLLM-SR d3 comparison lane (riir-train 629 T2): the JDI board's d3
/// entrant (Apache-2.0, UNGATED — teacher-eligible; tag v3.1.0), served
/// through their `d3_server.py` on the SAME TypeSafe `/v1/systemone` wire
/// (their server is adapted FROM pplx's autojev server; the wire functions
/// are REUSED from the pplx lane — one wire, many oracles), measured over
/// HTTP — the pplx lane's shape verbatim (same cases, same metrics tail
/// through [`assemble_laya_lane_result`]).
///
/// Env: `D3_SERVE_URL` (default `http://127.0.0.1:8794`), `D3_TIMEOUT_MS`
/// (default 300000 — the offload/quant posture is slow per forward),
/// `D3_DEVICE` (the quant-posture disclosure stamp, e.g. `cuda:0+nf4` —
/// the lossy-surface law: which quant produced the scores is PINNED and
/// quoted beside every number), `D3_API_KEY` (optional bearer). The
/// `/health` handshake captures the served model id (serve version-true
/// with `--name d3-v3.1.0`; their health carries no checkpoint field,
/// unlike pplx's) and refuses while their lazy first load is still
/// `"loading"`. The no-clock law: their body carries no timing anywhere,
/// so the determinism compare is verbatim — the column measures compute
/// determinism (device wobble included), never their clock.
///
/// This is the JDI-panel scoring half of 629 T2: the per-family spot
/// check of the quantized teacher's option-scores against the board's
/// published per-benchmark numbers (the lossy-surface rule) — run over
/// the tranche-1 JDI suites (`winogrande_jdi`, `hellaswag_jdi`,
/// `anli_jdi`, `mmlupro_jdi`, Plan 012) BEFORE trusting the captures.
pub(super) fn run_d3_lane(
    suite: &Suite,
    laya_max_questions: usize,
    leak_flags: Option<&[bool]>,
) -> Result<LaneResult, String> {
    use crate::lanes::d3::D3Lane;

    let t_start = Instant::now();
    let lane = D3Lane::default();
    let (model, device) = lane.info()?;
    eprintln!("    [d3] service up: {model} on {device}");

    // WARMUP (the clm lane's cold-start law, the pplx lane's shape): one
    // FIXED throwaway request (never a case's — no cache pollution of
    // measured latencies) absorbs the first-forward path (kernel compile
    // / offload page-in).
    {
        let warm = SuiteCase {
            id: "warmup".into(),
            state: Value::String(
                "warmup: the lane's cold-path probe (discarded; not a measured case)".into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "warm".into(),
                kind: QKind::Noul,
                instructions: "Is this the warmup?".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        lane.decide(&warm).map_err(|e| format!("d3 warmup: {e}"))?;
    }

    // The trim law (the gliner lane's law): the SAME question cap as the
    // laya lanes, so a capped run never compares a full-N d3 row against
    // a capped laya row.
    let mut cases: &[SuiteCase] = &suite.cases;
    if laya_max_questions > 0 {
        let mut n = 0usize;
        let mut cut = suite.cases.len();
        for (i, c) in suite.cases.iter().enumerate() {
            n += c.questions.len();
            if n >= laya_max_questions {
                cut = i + 1;
                break;
            }
        }
        cases = &suite.cases[..cut];
    }

    let mut probs: Vec<Vec<Vec<f64>>> = Vec::with_capacity(cases.len());
    let mut picks: Vec<Vec<usize>> = Vec::with_capacity(cases.len());
    let mut confs: Vec<Vec<f64>> = Vec::with_capacity(cases.len());
    let mut durs_ms: Vec<u64> = Vec::with_capacity(cases.len());
    let mut determinism_ok: Option<bool> = Some(true);
    // The repeat COUNT (the verdict's support).
    let mut determinism_n: usize = 0;
    let mut input_tokens: u64 = 0;

    for (ci, case) in cases.iter().enumerate() {
        let t0 = Instant::now();
        let (answers, _client_ms, tokens) = lane
            .decide(case)
            .map_err(|e| format!("d3 lane (case {ci}): {e}"))?;
        durs_ms.push(t0.elapsed().as_millis() as u64);
        input_tokens += tokens;

        // Observed-repeat determinism check, first 10 cases (the laya
        // lanes' law) — VERBATIM compare on this wire (the no-clock law).
        if ci < 10 {
            determinism_n += 1;
            let raw1 = lane
                .decide_raw(case)
                .map_err(|e| format!("d3 determinism rerun: {e}"))?;
            let raw2 = lane
                .decide_raw(case)
                .map_err(|e| format!("d3 determinism rerun: {e}"))?;
            if raw1 != raw2 {
                *determinism_ok.get_or_insert(true) = false;
            }
        }

        let mut cprobs = Vec::with_capacity(answers.len());
        let mut cpicks = Vec::with_capacity(answers.len());
        let mut cconfs = Vec::with_capacity(answers.len());
        for (p, pick, conf) in answers {
            cprobs.push(p);
            cpicks.push(pick);
            cconfs.push(conf);
        }
        probs.push(cprobs);
        picks.push(cpicks);
        confs.push(cconfs);
    }
    eprintln!("    [d3] input tokens summed: {input_tokens}");

    Ok(assemble_laya_lane_result(
        "d3",
        &model,
        suite.name,
        cases,
        leak_flags.map(|f| &f[..cases.len()]),
        probs,
        picks,
        confs,
        durs_ms,
        determinism_ok,
        Some(determinism_n),
        t_start.elapsed().as_secs_f64(),
    ))
}

/// The occupancy floor below which a split-half conformal floor is noise:
/// 40 pairs → 20/20 (the modelless lane's dedicated cal window is the G1
/// reference shape; this lane's floor is the cheap split-half companion and
/// says so in its `floor_posture`).
pub(super) const DREX_MIN_FLOOR_PAIRS: usize = 40;

/// Issue 076 T2 (Bench 125's instrumentation, landed on the upstream lane):
/// per-kind ECE/Brier of THEIR wire `confidence` vs gold correctness
/// (choice + score; noul carries no wire confidence → no cell, never a
/// fabricated row) beside the split-half conformal-naive floor over ALL
/// their-conf pairs pooled. Computed through `crate::harness::metrics`
/// only (ece_of / brier_of / conformal_naive_floor / CalibrationPair — no
/// parallel math). `None` when the suite carried no confidence-bearing
/// question at all. The floor reads the CASE ORDER (first half calibrates,
/// second half is audited) — deterministic by construction, and disclosed
/// as a posture rather than pooled with the modelless lane's
/// dedicated-window floor.
pub(super) fn assemble_drex_conf_readout(
    cases: &[SuiteCase],
    picks: &[Vec<usize>],
    their_confs: &[Vec<Option<f64>>],
) -> Option<crate::lanes::drex::DrexConfReadout> {
    use crate::lanes::drex::{DrexConfCell, DrexConfReadout};

    // (confidence, correct) pairs, pooled + per kind (choice/score only).
    let mut pooled: Vec<(f64, bool)> = Vec::new();
    let mut by_kind: BTreeMap<&'static str, Vec<(f64, bool)>> = BTreeMap::new();
    for (ci, case) in cases.iter().enumerate() {
        for (qi, q) in case.questions.iter().enumerate() {
            let Some(conf) = their_confs[ci][qi] else {
                continue;
            };
            let pair = (conf, picks[ci][qi] == case.gold[qi].idx);
            pooled.push(pair);
            match q.kind {
                QKind::Choice => by_kind.entry("choice").or_default().push(pair),
                QKind::Score => by_kind.entry("score").or_default().push(pair),
                // No wire confidence → no cell, never a fabricated row.
                QKind::Noul => continue,
            }
        }
    }
    if pooled.is_empty() {
        return None;
    }
    let cell_of = |pairs: &[(f64, bool)]| DrexConfCell {
        n: pairs.len(),
        ece: ece_of(pairs),
        brier: brier_of(pairs),
    };
    // Split-half conformal floor (Report-the-Floor): the FIRST half is the
    // cal window, the SECOND half the audited test side. Per-kind floors
    // stay absent — split-halfing a kind would halve its window toward the
    // occupancy floor (typed_decisions' choice/score cells sit at a few
    // hundred pairs each) and the pooled floor is the companion the G1 law
    // names.
    let (floor_ece, floor_n_cal, floor_n_test) = if pooled.len() >= DREX_MIN_FLOOR_PAIRS {
        let cut = pooled.len() / 2;
        let cal: Vec<CalibrationPair> = pooled[..cut]
            .iter()
            .map(|&(conf, correct)| CalibrationPair { conf, correct })
            .collect();
        let test_confs: Vec<f64> = pooled[cut..].iter().map(|&(c, _)| c).collect();
        let floored = conformal_naive_floor(&cal, &test_confs);
        let floor_pairs: Vec<(f64, bool)> = floored
            .into_iter()
            .zip(pooled[cut..].iter().map(|&(_, ok)| ok))
            .collect();
        (Some(ece_of(&floor_pairs)), cut, pooled.len() - cut)
    } else {
        (None, 0, 0)
    };
    Some(DrexConfReadout {
        choice: by_kind.get("choice").map(|p| cell_of(p)),
        score: by_kind.get("score").map(|p| cell_of(p)),
        noul: None,
        floor_ece,
        floor_n_cal,
        floor_n_test,
        floor_posture: "split-half conformal (first half cal, second half test) — \
                        this lane has no dedicated cal slice",
    })
}
