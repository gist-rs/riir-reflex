//! Issue 108 — the gold-support ceiling probe (REPORT-ONLY runner half).
//!
//! Harness suite scores are gold-absolute: a lane scoring 0.42 on sst5
//! reads as model failure against a ceiling of 1.0. But benchmark gold
//! itself carries noise — labels the fixture text does not support as
//! typed. This arm machine-adjudicates a stratified sample of the suite's
//! gold under the WRITTEN adjudication contract
//! (`.docs/02_protocols/ceiling_adjudication_contract.md`,
//! `CEILING_CONTRACT_V1`) and reports:
//!
//! 1. **Gold-support ceiling** ĝ with Wilson 95% CI (as-typed support),
//!    with the stratified design effect (deff) beside it.
//! 2. **The fate census** (closed set, one fate per sampled item):
//!    correct/wrong × supported/not_supported/unclear, abstained,
//!    unknown — the `correct ∧ not_supported` cell (lane agrees with
//!    unsupported gold) is the strongest new signal (shared bias /
//!    memorization).
//! 3. **Two ceilings, never pooled**: coverage (the share the gate
//!    SERVES at its fitted operating point) vs support (ĝ).
//! 4. **Ceiling-adjusted score = a per-item JOIN, never a division**:
//!    the lane's accuracy on the `supported` subset only, beside the raw
//!    score (raw stays the headline).
//!
//! Pre-registered laws (all enforced here, not prose):
//!
//! - **The judge-validity gate runs FIRST, in every run**: planted twin
//!   controls (a real item presented with its TRUE gold = the supported
//!   half; the SAME item with a far-class-flipped gold = the flipped
//!   half) interleaved seeded-blind into the judging stream;
//!   **flipped-catch recall ≥ 0.85** or the probe refuses to price gold
//!   (the OntoSmith failure example 29/40 = 0.725 sits below the bar; a
//!   judge below the bar does not price gold, whatever its CI).
//! - **No self-judging**: the judge seat is the CONTRACT's ladder —
//!   Bonsai-27B (ours, never a scored entrant) over a loopback
//!   OpenAI-compatible endpoint, or the Qwen3-8B fallback. Every reflex
//!   lane is disqualified where it sits the suite. The seat arms via
//!   `CEILING_JUDGE_URL` (REQUIRED — the arm refuses loud without it,
//!   never silently prices gold with no judge; loopback only — a remote
//!   judge is an owner act with its own posture stamp).
//! - **Verbatim-quote verification**: a `supported` verdict counts only
//!   with a quote occurring verbatim in the fixture text (whitespace
//!   runs collapsed, case-sensitive, no other edits — the contract's
//!   seam law, option text included); quote-failed supported verdicts
//!   are `unknown`, excluded from every denominator, counts published.
//! - **Blind**: the judge never sees lane scores, gold provenance, or
//!   strata — the prompt names only the text, the option set, and the
//!   label to adjudicate (the as-typed law).
//! - **grammar-not-answers**: adjudication outputs are eval-side only —
//!   this arm writes a report and nothing else.
//!
//! Contract scope V1: the `state.text`-shaped suites (sst5 is the first
//! cell; Score-kind criteria arrays). A suite outside the shape refuses
//! loud — widening is a CONTRACT edit first, never a silent arm.
//!
//! The lane half re-runs the deployed three-engine calibration posture:
//! raw engine → cal pairs → thresholds at the explicit calibrated scale
//! (Issue 056) → fitted engine (`observe` the pairs, thresholds in cfg)
//! → per-item abstain/pick exactly as served.

use serde::Serialize;

use super::*;
use crate::embed::EMBED_DIM;
use crate::engine::{EngineConfig, GateObservation, Posture, Scratch, recommend_fused_gate};
use crate::harness::suites::{QKind, SplitMix64};
use crate::lanes::http_mini;
use serde_json::json;
use std::time::Duration;

// ── the contract's machine projection (ONE commit with the doc) ──

/// The contract of record this arm enforces.
pub const CONTRACT_VERSION: &str = "CEILING_CONTRACT_V1";

/// The judge-validity bar (flipped-control catch recall).
pub const CONTROL_CATCH_BAR: f64 = 0.85;

/// The sampling floor (Wilson half-width ≤ 0.05 at p ≈ 0.5).
pub const MIN_SAMPLE_N: usize = 400;

/// The pre-registered cross-family recheck bar (three-category κ).
pub const CROSS_FAMILY_KAPPA_BAR: f64 = 0.70;

/// Default flipped controls per run (the validity gate's n; the
/// supported half rides the same twins).
pub const DEFAULT_FLIPPED_CONTROLS: usize = 40;

/// Judge request timeout (generous: a local 27B readout).
const JUDGE_TIMEOUT_MS: u64 = 120_000;

/// Env: the judge endpoint (loopback HTTP, OpenAI-compatible
/// `/v1/chat/completions`). Unset → the arm refuses loud.
pub const JUDGE_URL_ENV: &str = "CEILING_JUDGE_URL";
/// Env: the model name the endpoint serves under (default `bonsai`).
pub const JUDGE_MODEL_ENV: &str = "CEILING_JUDGE_MODEL";

#[derive(Debug, Clone)]
pub struct CeilingProbeOptions {
    pub suite: String,
    pub sample_n: usize,
    pub flipped_controls: usize,
    /// The judge seat ("bonsai" | "qwen3-8b" | an explicit model id) —
    /// the identity stamped beside every ceiling number (the contract's
    /// calibration-identity law).
    pub judge_model: String,
    pub seed: u64,
}

impl Default for CeilingProbeOptions {
    fn default() -> Self {
        Self {
            suite: "sst5".to_string(),
            sample_n: MIN_SAMPLE_N,
            flipped_controls: DEFAULT_FLIPPED_CONTROLS,
            judge_model: std::env::var(JUDGE_MODEL_ENV).unwrap_or_else(|_| "bonsai".into()),
            seed: 0x1105_2026_0108,
        }
    }
}

// ───────────────────────────────────────────── report shapes ──

/// The judge's parsed verdict (one sampled item or control).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JudgeVerdict {
    pub verdict: String,
    /// The quote the judge offered (None = no quote field / non-string).
    pub quote: Option<String>,
    /// True when the quote passed verbatim verification (only meaningful
    /// for `supported`).
    pub quote_ok: bool,
}

/// One sampled gold item's complete record (the audit trail).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CeilingItem {
    pub case_id: String,
    /// The gold label AS TYPED (the option string the item's gold index
    /// names — what the judge was asked about, never the judge's answer).
    pub gold_label: String,
    /// The lane answer: "abstained" | "correct" | "wrong".
    pub lane: &'static str,
    /// The adjudicated support reading: "supported" | "not_supported" |
    /// "unclear" | "unknown" (unknown = malformed judge output or a
    /// quote-failed supported verdict — excluded from every denominator).
    pub support: &'static str,
}

/// The closed fate census (exactly one fate per sampled item; `unclear`
/// disclosed separately, never pooled into `not_supported`).
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct FateCensus {
    pub correct_supported: usize,
    pub correct_not_supported: usize,
    pub correct_unclear: usize,
    pub wrong_supported: usize,
    pub wrong_not_supported: usize,
    pub wrong_unclear: usize,
    pub abstained: usize,
    pub unknown: usize,
}

impl FateCensus {
    /// The support denominator (every adjudicated, non-unknown fate).
    pub fn support_denominator(&self) -> usize {
        self.correct_supported
            + self.correct_not_supported
            + self.correct_unclear
            + self.wrong_supported
            + self.wrong_not_supported
            + self.wrong_unclear
    }
    /// Supported count in the denominator (ĝ's numerator).
    pub fn supported_total(&self) -> usize {
        self.correct_supported + self.wrong_supported
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidityGate {
    pub flipped_total: usize,
    /// Flipped controls NOT answered `supported` (the catch).
    pub flipped_caught: usize,
    pub flipped_catch_recall: f64,
    pub supported_total: usize,
    /// Twin supported-halves answered `supported` (with verified quotes).
    pub supported_correct: usize,
    pub supported_catch_rate: f64,
    pub bar: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CeilingReport {
    pub n_support_denominator: usize,
    pub gold_support: f64,
    pub wilson_lo: f64,
    pub wilson_hi: f64,
    pub ci_half_width: f64,
    pub kish_deff: f64,
    /// The share of the sampled front the gate SERVES (coverage ceiling).
    pub coverage_ceiling: f64,
    /// The raw gold-absolute lane accuracy over the whole sample (the
    /// headline — comparability law).
    pub raw_lane_accuracy: f64,
    /// The per-item JOIN: the lane's accuracy on the supported subset.
    pub supported_subset_accuracy: f64,
    pub supported_subset_n: usize,
    /// The shared-bias / memorization cell count, disclosed beside the
    /// adjusted score (the contract's own disclosure law).
    pub correct_not_supported: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CeilingProbeMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub contract_version: &'static str,
    pub suite: String,
    pub judge_model: String,
    pub judge_url_host: String,
    pub sample_n_requested: usize,
    pub sample_n_actual: usize,
    pub flipped_controls: usize,
    pub seed: u64,
    pub score_threshold: f32,
    pub distance_threshold: f32,
    pub rho: f64,
    pub posture_note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CeilingProbeOutput {
    pub meta: CeilingProbeMeta,
    pub validity: ValidityGate,
    pub census: FateCensus,
    pub report: CeilingReport,
    /// The per-item records (eval-side only; grammar-not-answers).
    pub items: Vec<CeilingItem>,
}

// ───────────────────────────────────────────── judge client ──

/// The judge prompt (the contract, verbatim-per-item). Blind: no lane
/// scores, no gold provenance, no strata.
fn judge_messages(context: &str, options: &[String], gold_label: &str) -> Vec<serde_json::Value> {
    let options_block = options
        .iter()
        .map(|o| format!("- {o}"))
        .collect::<Vec<_>>()
        .join("\n");
    vec![
        json!({
            "role": "system",
            "content": "You adjudicate benchmark gold labels. Given a text and a \
             closed option set, decide ONLY whether the text supports the given \
             label AS TYPED — exactly that label, not a nearby reading. Answer \
             with a single JSON object and nothing else:\n\
             {\"verdict\": \"supported\" | \"not_supported\" | \"unclear\", \
             \"quote\": \"...\"}\n\
             Rules: a `supported` verdict MUST carry `quote` — an exact \
             verbatim substring of the text or the options that supports the \
             label as typed. `not_supported` and `unclear` need no quote \
             (\"quote\": null). Never output anything except the JSON object."
        }),
        json!({
            "role": "user",
            "content": format!(
                "Text:\n{text}\n\nOptions:\n{options}\n\nLabel to adjudicate: {gold}\n\nDoes the text support this label as typed? Answer with the JSON object only.",
                text = context,
                options = options_block,
                gold = gold_label
            )
        }),
    ]
}

/// One judge exchange: POST /v1/chat/completions → parse the content as
/// the verdict JSON. HTTP/parsing failures are hard errors (the seat is
/// down → refuse the run); malformed CONTENT is a per-item `unknown`.
fn judge_adjudicate(
    host: &str,
    port: u16,
    model: &str,
    context: &str,
    options: &[String],
    gold_label: &str,
) -> Result<JudgeVerdict, String> {
    let body = json!({
        "model": model,
        "messages": judge_messages(context, options, gold_label),
        "temperature": 0.0_f64,
        "max_tokens": 256,
        "stream": false,
    });
    let payload = serde_json::to_vec(&body).map_err(|e| e.to_string())?;
    let reply = http_mini::request(
        host,
        port,
        "POST",
        "/v1/chat/completions",
        Some(&payload),
        Duration::from_millis(JUDGE_TIMEOUT_MS),
        &[],
    )
    .map_err(|e| format!("judge exchange failed: {e}"))?;
    if reply.0 != 200 {
        return Err(format!(
            "judge endpoint answered HTTP {} — is the judge server up?",
            reply.0
        ));
    }
    let parsed: serde_json::Value =
        serde_json::from_slice(&reply.2).map_err(|e| format!("judge reply unparsable: {e}"))?;
    let content = parsed
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("judge reply carries no choices[0].message.content")?;
    Ok(parse_verdict(content))
}

/// Lenient verdict extraction: the first `{` … last `}` span parsed as
/// JSON; `verdict` + optional `quote`. Anything else is malformed.
fn parse_verdict(content: &str) -> JudgeVerdict {
    let malformed = JudgeVerdict {
        verdict: "malformed".to_string(),
        quote: None,
        quote_ok: false,
    };
    let Some(start) = content.find('{') else {
        return malformed;
    };
    let Some(end) = content.rfind('}') else {
        return malformed;
    };
    if end < start {
        return malformed;
    }
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content[start..=end]) else {
        return malformed;
    };
    let verdict = v
        .get("verdict")
        .and_then(|x| x.as_str())
        .unwrap_or("malformed")
        .trim()
        .to_ascii_lowercase();
    let quote = v.get("quote").and_then(|x| x.as_str()).map(str::to_string);
    JudgeVerdict {
        verdict,
        quote,
        quote_ok: false,
    }
}

/// The contract's quote law: whitespace runs collapse, case-sensitive,
/// exact substring — over the context AND the option text (the
/// `seam_option_text_in_context` ruling).
fn quote_verifies(quote: &str, context: &str, options: &[String]) -> bool {
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let q = norm(quote);
    if q.is_empty() {
        return false;
    }
    if norm(context).contains(&q) {
        return true;
    }
    options.iter().any(|o| norm(o).contains(&q))
}

/// The support reading for one adjudication under the contract.
fn support_reading(v: &JudgeVerdict, context: &str, options: &[String]) -> &'static str {
    match v.verdict.as_str() {
        "supported" => {
            let quoted = v
                .quote
                .as_deref()
                .is_some_and(|q| quote_verifies(q, context, options));
            if quoted {
                "supported"
            } else {
                "unknown" // quote-failed supported → excluded, counted
            }
        }
        "not_supported" => "not_supported",
        "unclear" => "unclear",
        _ => "unknown",
    }
}

// ───────────────────────────────────────────── stats ──

/// The Wilson score interval at 95% (z = 1.959964); n = 0 → (0, 1).
fn wilson95(k: usize, n: usize) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let z = 1.959_964_f64;
    let p = k as f64 / n as f64;
    let z2 = z * z;
    let denom = 1.0 + z2 / n as f64;
    let center = (p + z2 / (2.0 * n as f64)) / denom;
    let half = z / denom * ((p * (1.0 - p) / n as f64 + z2 / (4.0 * n as f64 * n as f64)).sqrt());
    ((center - half).max(0.0), (center + half).min(1.0))
}

/// Kish's approximate design effect over the strata: deff = 1 + cv² of
/// the stratum weight shares (proportional allocation → ≈ 1, disclosed).
fn kish_deff(stratum_sizes: &[usize]) -> f64 {
    let total: usize = stratum_sizes.iter().sum();
    if total == 0 || stratum_sizes.len() < 2 {
        return 1.0;
    }
    let weights: Vec<f64> = stratum_sizes
        .iter()
        .map(|&s| s as f64 / total as f64)
        .collect();
    let mean = 1.0 / weights.len() as f64;
    let var = weights
        .iter()
        .map(|w| {
            let d = w - mean;
            d * d
        })
        .sum::<f64>()
        / weights.len() as f64;
    1.0 + var / (mean * mean)
}

// ───────────────────────────────────────────── fixture text ──

/// The V1 fixture shape: `state.text` (a string) + Score-kind criteria
/// (an array of strings). Anything else refuses loud.
fn fixture_of(case: &SuiteCase) -> Result<(String, Vec<String>, String), String> {
    let context = case
        .state
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            format!(
                "case {}: state carries no `text` string — contract V1 scopes \
                 the state.text shape only",
                case.id
            )
        })?
        .to_string();
    let q = case.questions.first().ok_or_else(|| {
        format!(
            "case {}: no questions — contract V1 needs one scored question",
            case.id
        )
    })?;
    if q.kind != QKind::Score {
        return Err(format!(
            "case {}: question kind is not Score — contract V1 scopes the \
             Score criteria-array shape only",
            case.id
        ));
    }
    let options: Vec<String> = q
        .criteria
        .as_array()
        .ok_or_else(|| {
            format!(
                "case {}: criteria is not an array — contract V1 shape",
                case.id
            )
        })?
        .iter()
        .map(|o| o.as_str().unwrap_or_default().to_string())
        .collect();
    let gold_idx = case
        .gold
        .first()
        .map(|g| g.idx)
        .ok_or_else(|| format!("case {}: no gold — contract V1 shape", case.id))?;
    let gold_label = options.get(gold_idx).cloned().ok_or_else(|| {
        format!(
            "case {}: gold idx {gold_idx} out of the option set — contract V1 shape",
            case.id
        )
    })?;
    Ok((context, options, gold_label))
}

/// The far-class flip for a Score gold: the index maximizing |i − j|
/// under the option order (ties → the lower index — deterministic).
fn far_flip(gold: usize, n_options: usize) -> Option<usize> {
    if n_options < 2 || gold >= n_options {
        return None;
    }
    (0..n_options)
        .filter(|&j| j != gold)
        .max_by_key(|&j| ((j as i64 - gold as i64).abs(), std::cmp::Reverse(j)))
}

// ───────────────────────────────────────────── lane half ──

struct LaneAnswers {
    score_threshold: f32,
    distance_threshold: f32,
    /// Per sampled case, in sample order: the lane answer.
    answers: Vec<&'static str>,
}

/// The deployed three-engine shape at the default posture: raw build →
/// cal pairs + calibrated-scale observations → fitted thresholds →
/// fitted build (`observe` the pairs, thresholds in cfg) → per-item
/// abstain/pick exactly as served.
#[allow(clippy::too_many_lines)]
fn lane_answers<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    sample: &[usize],
    rho: f64,
) -> Result<LaneAnswers, String> {
    let effective_cap = spec.corpus_cap_per_label;
    let raw_cfg = EngineConfig {
        legacy_kn_route: true,
        ..EngineConfig::default()
    };
    let (mut raw_engine, _) = super::build_engine::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        raw_cfg.clone(),
    )?;

    // ── the cal pairs + the calibrated-scale fit observations (the
    //    deployed fit loop, verbatim) ──
    let mut cal_pairs: Vec<(f32, bool)> = Vec::new();
    let mut score_obs: Vec<GateObservation> = Vec::new();
    let mut distance_obs: Vec<GateObservation> = Vec::new();
    {
        let mut sc = Scratch::<EMBED_DIM>::new();
        let max_q = prepared
            .cal_cases
            .iter()
            .map(|c| c.questions.len())
            .max()
            .unwrap_or(1);
        sc.prepare(max_q);
        for (ci, case) in prepared.cal_cases.iter().enumerate() {
            let req = super::engine_request(case, &prepared.cal_state_strs[ci])?;
            raw_engine
                .solve_into(&req, &mut sc)
                .map_err(|e| format!("ceiling cal solve ({}, case {}): {e}", spec.name, case.id))?;
            for (qi, slot) in sc.slots.iter().enumerate() {
                let pick = match case.questions.get(qi).map(|q| q.kind) {
                    Some(QKind::Noul) => (1 - slot.pick) as usize,
                    _ => slot.pick as usize,
                };
                let correct = case.gold.get(qi).is_some_and(|g| g.idx == pick);
                cal_pairs.push((slot.confidence, correct));
            }
            if let (Some(slot), Some(dom)) = (sc.slots.last(), sc.domains.last()) {
                let pick = match case.questions.last().map(|q| q.kind) {
                    Some(QKind::Noul) => (1 - slot.pick) as usize,
                    _ => slot.pick as usize,
                };
                let correct = case.gold.last().is_some_and(|g| g.idx == pick);
                score_obs.push(GateObservation {
                    score: slot.confidence,
                    correct,
                });
                distance_obs.push(GateObservation {
                    score: raw_engine.gate(*dom).abstain_confidence(&sc.q),
                    correct,
                });
            }
        }
    }
    // The Issue-056 calibrator: thresholds fit on the calibrated scale the
    // deployed gate reads.
    let cal_defaults = EngineConfig::default();
    let mut cal = katgpt_core::sigmoid_calibration::SigmoidGateCalibrator::new(
        cal_defaults.cal_capacity,
        cal_defaults.cal_min_obs,
    );
    for (c, ok) in &cal_pairs {
        cal.observe(*c, *ok);
    }
    cal.refit();
    for obs in &mut score_obs {
        obs.score = cal.apply(obs.score);
    }
    let rec = recommend_fused_gate(&score_obs, &distance_obs, Posture::Percentile { rho });
    let (score_threshold, distance_threshold) =
        super::resolve_gate_thresholds(&rec, &raw_cfg, false);

    // ── the fitted engine: thresholds in cfg + observe the cal pairs
    //    (its internal calibrator moves exactly as served) ──
    let fitted_cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        legacy_kn_route: true,
        ..cal_defaults
    };
    let (mut fitted, _) = super::build_engine::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        fitted_cfg,
    )?;
    for (conf, correct) in &cal_pairs {
        fitted.observe(*conf, *correct);
    }

    // ── the sampled front, one lane answer each ──
    let mut answers = Vec::with_capacity(sample.len());
    {
        let mut sc = Scratch::<EMBED_DIM>::new();
        let max_q = sample
            .iter()
            .filter_map(|&i| prepared.suite.cases.get(i))
            .map(|c| c.questions.len())
            .max()
            .unwrap_or(1);
        sc.prepare(max_q);
        for &i in sample {
            let case = &prepared.suite.cases[i];
            let req = super::engine_request(case, &prepared.state_strs[i])?;
            fitted
                .solve_into(&req, &mut sc)
                .map_err(|e| format!("ceiling solve ({}, case {}): {e}", spec.name, case.id))?;
            let Some(slot) = sc.slots.last() else {
                answers.push("wrong");
                continue;
            };
            if slot.abstained {
                answers.push("abstained");
                continue;
            }
            let pick = match case.questions.last().map(|q| q.kind) {
                Some(QKind::Noul) => (1 - slot.pick) as usize,
                _ => slot.pick as usize,
            };
            let correct = case.gold.last().is_some_and(|g| g.idx == pick);
            answers.push(if correct { "correct" } else { "wrong" });
        }
    }
    Ok(LaneAnswers {
        score_threshold,
        distance_threshold,
        answers,
    })
}

fn spec_of(name: &str) -> Result<&'static SuiteSpec, String> {
    SUITES
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("ceiling-probe: suite {name} is not in the registry"))
}

// ───────────────────────────────────────────── judge seat ──

/// The judge endpoint from the env (the seat ladder's arming seam).
/// Pure in the URL so the refusal paths stay testable.
fn judge_endpoint(url: Option<String>) -> Result<(String, u16), String> {
    let url = url.ok_or_else(|| {
        format!(
            "ceiling-probe: the judge seat is not armed — set {JUDGE_URL_ENV} \
             (a LOOPBACK OpenAI-compatible endpoint; the contract's seat \
             ladder names the primary). The probe refuses loud rather than \
             pricing gold with no judge."
        )
    })?;
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("ceiling-probe: judge URL must be http:// (loopback): {url}"))?;
    let host_port = rest.trim_end_matches('/');
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| format!("ceiling-probe: judge URL carries no port: {url}"))?;
    let port: u16 = port
        .parse()
        .map_err(|_| format!("ceiling-probe: judge URL port unparsable: {url}"))?;
    if host != "127.0.0.1" && host != "localhost" && host != "[::1]" {
        return Err(format!(
            "ceiling-probe: judge host must be loopback (got {host}) — a \
             remote judge is an owner act with its own posture stamp"
        ));
    }
    Ok((host.to_string(), port))
}

// ───────────────────────────────────────────── entry ──

/// Run the probe over the contract's V1 shape. REPORT-ONLY: the report is
/// the only artifact (grammar-not-answers — adjudication outputs never
/// enter corpora, retrieval, or training).
#[allow(clippy::too_many_lines)]
pub fn run_ceiling_probe(
    opts: &RunOptions,
    popts: CeilingProbeOptions,
) -> Result<CeilingProbeOutput, String> {
    if popts.sample_n < MIN_SAMPLE_N {
        return Err(format!(
            "ceiling-probe: sample {} < contract floor {MIN_SAMPLE_N} \
             (Wilson half-width ≤ 0.05 needs ≈ 385; never run under)",
            popts.sample_n
        ));
    }
    if popts.flipped_controls == 0 {
        return Err("ceiling-probe: the validity gate needs flipped controls > 0".to_string());
    }
    let (judge_host, judge_port) = judge_endpoint(std::env::var(JUDGE_URL_ENV).ok())?;

    let spec = spec_of(&popts.suite)?;
    if spec.synthetic.is_some() {
        return Err(format!(
            "ceiling-probe: {} is synthetic — contract V1 rides dataset suites",
            popts.suite
        ));
    }
    let prepared = super::prepare(spec, &opts.datasets_dir)?;
    let n_cases = prepared.suite.cases.len();
    if n_cases == 0 {
        return Err(format!("ceiling-probe: {} front is empty", popts.suite));
    }
    // Shape gate on the whole front (fail fast, not mid-sample).
    for case in &prepared.suite.cases {
        fixture_of(case)?;
    }

    // ── the stratified sample (the front is label-stratified; the stride
    //    keeps proportionality) ──
    let sample: Vec<usize> =
        super::perm_probe::stride_indices(n_cases, popts.sample_n.min(n_cases));
    let sample_n = sample.len();
    if sample_n < MIN_SAMPLE_N {
        return Err(format!(
            "ceiling-probe: {} front holds {n_cases} cases — under the \
             contract floor {MIN_SAMPLE_N}; widen the eval cap or pick a \
             bigger suite",
            popts.suite
        ));
    }

    // ── the lane half (fitted-threshold abstain semantics) ──
    macro_rules! dispatch {
        ($n:literal) => {
            lane_answers::<$n>(spec, &prepared, &sample, 0.30)
        };
    }
    let lane = match prepared.labels.len() {
        2 => dispatch!(2),
        3 => dispatch!(3),
        4 => dispatch!(4),
        5 => dispatch!(5),
        6 => dispatch!(6),
        7 => dispatch!(7),
        8 => dispatch!(8),
        10 => dispatch!(10),
        15 => dispatch!(15),
        56 => dispatch!(56),
        59 => dispatch!(59),
        77 => dispatch!(77),
        842 => dispatch!(842),
        n => {
            return Err(format!(
                "{}: no engine instantiation for {n} domains — extend ceiling_probe's dispatch",
                spec.name
            ));
        }
    }?;

    // ── the fixtures for the sample ──
    let mut fixtures: Vec<(String, Vec<String>, String)> = Vec::with_capacity(sample_n);
    for &i in &sample {
        fixtures.push(fixture_of(&prepared.suite.cases[i])?);
    }

    // ── the twin controls (seeded: which sample items twin; the judge
    //    stream interleaves them blind — the judge prompt never reveals
    //    control status) ──
    let mut rng = SplitMix64::new(popts.seed);
    let n_ctrl = popts.flipped_controls.min(sample_n);
    let mut control_positions: Vec<usize> = (0..sample_n).collect();
    // Seeded partial Fisher-Yates over the positions, take the first n_ctrl.
    for k in 0..n_ctrl {
        let j = k + (rng.next_u64() as usize) % (sample_n - k);
        control_positions.swap(k, j);
    }
    control_positions.truncate(n_ctrl);

    // ── the judging stream: controls first (the validity gate runs
    //    FIRST and REFUSES the run before any sample adjudication is
    //    priced), items second ──
    let mut flipped_total = 0usize;
    let mut flipped_caught = 0usize;
    let mut supported_total = 0usize;
    let mut supported_correct = 0usize;
    for &pos in &control_positions {
        let (context, options, gold) = &fixtures[pos];
        let flip = far_flip(
            prepared.suite.cases[sample[pos]]
                .gold
                .first()
                .map(|g| g.idx)
                .unwrap_or(0),
            options.len(),
        )
        .ok_or_else(|| "ceiling-probe: control flip needs ≥ 2 options".to_string())?;
        // Flipped half first (the gate's currency), then its supported twin.
        let flipped_label = &options[flip];
        let v = judge_adjudicate(
            &judge_host,
            judge_port,
            &popts.judge_model,
            context,
            options,
            flipped_label,
        )?;
        let reading = support_reading(&v, context, options);
        flipped_total += 1;
        if reading != "supported" {
            flipped_caught += 1;
        }
        let v = judge_adjudicate(
            &judge_host,
            judge_port,
            &popts.judge_model,
            context,
            options,
            gold,
        )?;
        let reading = support_reading(&v, context, options);
        supported_total += 1;
        if reading == "supported" {
            supported_correct += 1;
        }
    }
    let flipped_catch_recall = flipped_caught as f64 / flipped_total.max(1) as f64;
    let supported_catch_rate = supported_correct as f64 / supported_total.max(1) as f64;
    let validity = ValidityGate {
        flipped_total,
        flipped_caught,
        flipped_catch_recall,
        supported_total,
        supported_correct,
        supported_catch_rate,
        bar: CONTROL_CATCH_BAR,
        pass: flipped_catch_recall >= CONTROL_CATCH_BAR,
    };
    if !validity.pass {
        return Err(format!(
            "ceiling-probe: the judge FAILED its validity gate — flipped \
             catch {flipped_caught}/{flipped_total} = {flipped_catch_recall:.3} \
             < {CONTROL_CATCH_BAR} (supported-half catch {supported_correct}/\
             {supported_total} = {supported_catch_rate:.3}). The judge does \
             not price gold: fix the seat or record the negative — never \
             lower the bar."
        ));
    }

    // ── the sample adjudication (the frozen read) ──
    let mut items: Vec<CeilingItem> = Vec::with_capacity(sample_n);
    let mut census = FateCensus::default();
    let mut strata: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut served = 0usize;
    let mut lane_correct = 0usize;
    let mut supported_subset_correct = 0usize;
    let mut supported_subset_n = 0usize;
    for (k, &pos) in sample.iter().enumerate() {
        let case = &prepared.suite.cases[pos];
        let (context, options, gold) = &fixtures[k];
        let lane_answer = lane.answers[k];
        if lane_answer == "abstained" {
            census.abstained += 1;
            items.push(CeilingItem {
                case_id: case.id.clone(),
                gold_label: gold.clone(),
                lane: lane_answer,
                support: "unsupported_scope",
            });
            continue;
        }
        served += 1;
        if lane_answer == "correct" {
            lane_correct += 1;
        }
        let v = judge_adjudicate(
            &judge_host,
            judge_port,
            &popts.judge_model,
            context,
            options,
            gold,
        )?;
        let reading = support_reading(&v, context, options);
        match (lane_answer, reading) {
            ("correct", "supported") => census.correct_supported += 1,
            ("correct", "not_supported") => census.correct_not_supported += 1,
            ("correct", "unclear") => census.correct_unclear += 1,
            ("wrong", "supported") => census.wrong_supported += 1,
            ("wrong", "not_supported") => census.wrong_not_supported += 1,
            ("wrong", "unclear") => census.wrong_unclear += 1,
            (_, "unknown") => census.unknown += 1,
            _ => census.unknown += 1,
        }
        if reading != "unknown" {
            *strata.entry(gold.clone()).or_default() += 1;
        }
        if reading == "supported" {
            supported_subset_n += 1;
            if lane_answer == "correct" {
                supported_subset_correct += 1;
            }
        }
        items.push(CeilingItem {
            case_id: case.id.clone(),
            gold_label: gold.clone(),
            lane: lane_answer,
            support: reading,
        });
    }

    let denominator = census.support_denominator();
    if denominator == 0 {
        return Err(
            "ceiling-probe: the support denominator is EMPTY — every item \
             abstained or read unknown; nothing to price (loud zero)"
                .to_string(),
        );
    }
    let supported_n = census.supported_total();
    let (wilson_lo, wilson_hi) = wilson95(supported_n, denominator);
    let strata_sizes: Vec<usize> = strata.values().copied().collect();
    let report = CeilingReport {
        n_support_denominator: denominator,
        gold_support: supported_n as f64 / denominator as f64,
        wilson_lo,
        wilson_hi,
        ci_half_width: (wilson_hi - wilson_lo) / 2.0,
        kish_deff: kish_deff(&strata_sizes),
        coverage_ceiling: served as f64 / sample_n as f64,
        raw_lane_accuracy: lane_correct as f64 / sample_n as f64,
        supported_subset_accuracy: if supported_subset_n > 0 {
            supported_subset_correct as f64 / supported_subset_n as f64
        } else {
            f64::NAN
        },
        supported_subset_n,
        correct_not_supported: census.correct_not_supported,
    };

    Ok(CeilingProbeOutput {
        meta: CeilingProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            contract_version: CONTRACT_VERSION,
            suite: popts.suite.clone(),
            judge_model: popts.judge_model.clone(),
            judge_url_host: format!("{judge_host}:{judge_port}"),
            sample_n_requested: popts.sample_n,
            sample_n_actual: sample_n,
            flipped_controls: n_ctrl,
            seed: popts.seed,
            score_threshold: lane.score_threshold,
            distance_threshold: lane.distance_threshold,
            rho: 0.30,
            posture_note: "lane half = the deployed three-engine calibration \
             posture (raw → cal pairs → Issue-056 calibrated-scale thresholds \
             → fitted engine via observe, thresholds in cfg); judge half = \
             the contract's seat ladder over loopback; the frozen read runs \
             ONCE per contract version."
                .to_string(),
        },
        validity,
        census,
        report,
        items,
    })
}

// ───────────────────────────────────────────── render ──

fn pct(x: f64) -> String {
    if x.is_finite() {
        format!("{x:.4}")
    } else {
        "n/a".to_string()
    }
}

/// The markdown report (stdout surface + the .md artifact — one render).
pub fn render_ceiling_probe_markdown(out: &CeilingProbeOutput) -> String {
    let mut md = String::new();
    md.push_str(&format!(
        "# Gold-support ceiling probe — {} ({} under {})\n\n",
        out.meta.suite, out.meta.judge_model, out.meta.contract_version
    ));
    md.push_str(&format!(
        "- judge: {} @ {} (seat ladder primary; no-self-judging holds — the judge is not a scored entrant)\n",
        out.meta.judge_model, out.meta.judge_url_host
    ));
    let v = &out.validity;
    md.push_str(&format!(
        "- validity gate: flipped catch {}/{} = {} (bar {}; {}) · supported-half catch {}/{} = {}\n",
        v.flipped_caught,
        v.flipped_total,
        pct(v.flipped_catch_recall),
        pct(v.bar),
        if v.pass { "PASS" } else { "FAIL" },
        v.supported_correct,
        v.supported_total,
        pct(v.supported_catch_rate),
    ));
    let c = &out.census;
    md.push_str(&format!(
        "- fate census (n = {}): correct supported {} · correct NOT-supported {} (shared-bias cell) · correct unclear {} · wrong supported {} · wrong not-supported {} · wrong unclear {} · abstained {} · unknown {}\n",
        c.support_denominator() + c.abstained + c.unknown,
        c.correct_supported,
        c.correct_not_supported,
        c.correct_unclear,
        c.wrong_supported,
        c.wrong_not_supported,
        c.wrong_unclear,
        c.abstained,
        c.unknown,
    ));
    let r = &out.report;
    md.push_str(&format!(
        "- SUPPORT CEILING ĝ = {} (Wilson 95% [{}, {}], half-width {}; Kish deff {} — n_eff = {})\n",
        pct(r.gold_support),
        pct(r.wilson_lo),
        pct(r.wilson_hi),
        pct(r.ci_half_width),
        pct(r.kish_deff),
        ((r.n_support_denominator as f64) / r.kish_deff) as usize,
    ));
    md.push_str(&format!(
        "- coverage ceiling (gate serves): {} · raw lane accuracy (headline): {}\n",
        pct(r.coverage_ceiling),
        pct(r.raw_lane_accuracy),
    ));
    md.push_str(&format!(
        "- ceiling-adjusted score = per-item JOIN on the supported subset: {} over n {} (raw stays the headline — never a division by ĝ) · correct∧not-supported = {} (the lane agreeing with unsupported gold)\n\n",
        pct(r.supported_subset_accuracy),
        r.supported_subset_n,
        r.correct_not_supported,
    ));
    md.push_str(&format!(
        "population floor: {} sampled · {} adjudicated · thresholds score {}/distance {} at ρ {}\n",
        out.meta.sample_n_actual,
        r.n_support_denominator,
        out.meta.score_threshold,
        out.meta.distance_threshold,
        out.meta.rho,
    ));
    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_verdict_reads_the_json_object() {
        let v = parse_verdict("{\"verdict\": \"supported\", \"quote\": \"great product\"}");
        assert_eq!(v.verdict, "supported");
        assert_eq!(v.quote.as_deref(), Some("great product"));
        // Prose around the object is tolerated (the lenient span).
        let v = parse_verdict("Sure! {\"verdict\": \"not_supported\", \"quote\": null} done");
        assert_eq!(v.verdict, "not_supported");
        assert!(v.quote.is_none());
    }

    #[test]
    fn parse_verdict_malforms_loudly() {
        for junk in [
            "no json at all",
            "{broken",
            "broken}",
            "{\"verdict\": 42}",
            "{\"quote\": \"x\"}", // no verdict field
        ] {
            assert_eq!(parse_verdict(junk).verdict, "malformed");
        }
    }

    #[test]
    fn quote_verification_follows_the_contract() {
        let context = "The  team  delivered a great product on time.";
        let options = vec![
            "very negative".to_string(),
            "positive".to_string(),
            "unclear".to_string(),
        ];
        // Exact hit in the context.
        assert!(quote_verifies("great product", context, &options));
        // Whitespace runs collapse (the seam ruling).
        assert!(quote_verifies("The team delivered", context, &options));
        // Option text counts as fixture text.
        assert!(quote_verifies("very negative", context, &options));
        // Case-sensitive: no hit.
        assert!(!quote_verifies("Great Product", context, &options));
        // Absent span: no hit.
        assert!(!quote_verifies("a terrible product", context, &options));
        // Empty quote: no hit.
        assert!(!quote_verifies("   ", context, &options));
    }

    #[test]
    fn support_reading_enforces_the_quote_law() {
        let context = "a great product";
        let options = vec!["positive".to_string()];
        let ok = JudgeVerdict {
            verdict: "supported".into(),
            quote: Some("great product".into()),
            quote_ok: false,
        };
        assert_eq!(support_reading(&ok, context, &options), "supported");
        let quoteless = JudgeVerdict {
            verdict: "supported".into(),
            quote: None,
            quote_ok: false,
        };
        assert_eq!(support_reading(&quoteless, context, &options), "unknown");
        let bad_quote = JudgeVerdict {
            verdict: "supported".into(),
            quote: Some("no such span".into()),
            quote_ok: false,
        };
        assert_eq!(support_reading(&bad_quote, context, &options), "unknown");
        let unclear = JudgeVerdict {
            verdict: "unclear".into(),
            quote: None,
            quote_ok: false,
        };
        assert_eq!(support_reading(&unclear, context, &options), "unclear");
        let no = JudgeVerdict {
            verdict: "not_supported".into(),
            quote: None,
            quote_ok: false,
        };
        assert_eq!(support_reading(&no, context, &options), "not_supported");
    }

    #[test]
    fn wilson_meets_the_contract_floor_at_the_fat_p() {
        // p ≈ 0.5, n = 400: the half-width must clear ≤ 0.05 — the very
        // bar the contract's sampling floor exists for.
        let (lo, hi) = wilson95(200, 400);
        assert!((hi - lo) / 2.0 <= 0.05, "half-width {}", (hi - lo) / 2.0);
        let (lo, hi) = wilson95(0, 400);
        assert_eq!(lo, 0.0);
        assert!(hi < 0.02);
        let (lo, hi) = wilson95(400, 400);
        assert_eq!(hi, 1.0);
        assert!(lo > 0.99);
        let (lo, hi) = wilson95(1, 0);
        assert_eq!((lo, hi), (0.0, 1.0));
    }

    #[test]
    fn kish_deff_is_one_at_proportional_allocation() {
        // Equal shares → deff 1 (the front's stratified stride is
        // proportional by construction).
        assert!((kish_deff(&[100, 100, 100, 100]) - 1.0).abs() < 1e-9);
        assert!(kish_deff(&[100]) == 1.0);
        assert!(kish_deff(&[]) == 1.0);
        // A dominant stratum raises deff — disclosed, never hidden.
        assert!(kish_deff(&[390, 3, 3, 4]) > 1.5);
    }

    #[test]
    fn far_flip_takes_the_far_class_with_deterministic_ties() {
        // sst5's 5 levels: 0↔4, 1→4 (dist 3), 2→0 (tie 0 vs 4 → the
        // lower index wins), 3→0, 4→0.
        assert_eq!(far_flip(0, 5), Some(4));
        assert_eq!(far_flip(1, 5), Some(4));
        assert_eq!(far_flip(2, 5), Some(0));
        assert_eq!(far_flip(3, 5), Some(0));
        assert_eq!(far_flip(4, 5), Some(0));
        assert_eq!(far_flip(0, 1), None);
        assert_eq!(far_flip(5, 5), None);
    }

    #[test]
    fn judge_endpoint_refuses_the_unarmed_and_remote_seats() {
        assert!(
            judge_endpoint(None)
                .unwrap_err()
                .contains("the judge seat is not armed")
        );
        assert!(
            judge_endpoint(Some("https://api.example.com".into()))
                .unwrap_err()
                .contains("http://")
        );
        assert!(
            judge_endpoint(Some("http://example.com:8080".into()))
                .unwrap_err()
                .contains("loopback")
        );
        assert!(
            judge_endpoint(Some("http://127.0.0.1".into()))
                .unwrap_err()
                .contains("no port")
        );
        let (host, port) = judge_endpoint(Some("http://127.0.0.1:8080/".into())).unwrap();
        assert_eq!((host.as_str(), port), ("127.0.0.1", 8080));
        let (host, port) = judge_endpoint(Some("http://localhost:8090".into())).unwrap();
        assert_eq!((host.as_str(), port), ("localhost", 8090));
    }

    #[test]
    fn census_denominator_and_support_totals_add_up() {
        let c = FateCensus {
            correct_supported: 10,
            correct_not_supported: 5,
            correct_unclear: 3,
            wrong_supported: 4,
            wrong_not_supported: 6,
            wrong_unclear: 2,
            abstained: 40,
            unknown: 7,
        };
        assert_eq!(c.support_denominator(), 30);
        assert_eq!(c.supported_total(), 14);
    }

    #[test]
    fn render_is_deterministic_for_a_fixed_output() {
        let meta = CeilingProbeMeta {
            date_utc: "fixed".into(),
            git_sha: Some("fixed".into()),
            host: "fixed".into(),
            contract_version: CONTRACT_VERSION,
            suite: "sst5".into(),
            judge_model: "bonsai".into(),
            judge_url_host: "127.0.0.1:8080".into(),
            sample_n_requested: 400,
            sample_n_actual: 400,
            flipped_controls: 40,
            seed: 1,
            score_threshold: 0.2,
            distance_threshold: 0.5,
            rho: 0.30,
            posture_note: "fixed".into(),
        };
        let validity = ValidityGate {
            flipped_total: 40,
            flipped_caught: 38,
            flipped_catch_recall: 0.95,
            supported_total: 40,
            supported_correct: 36,
            supported_catch_rate: 0.9,
            bar: CONTROL_CATCH_BAR,
            pass: true,
        };
        let census = FateCensus {
            correct_supported: 150,
            correct_not_supported: 20,
            correct_unclear: 8,
            wrong_supported: 10,
            wrong_not_supported: 60,
            wrong_unclear: 4,
            abstained: 120,
            unknown: 28,
        };
        let (lo, hi) = wilson95(160, 252);
        let report = CeilingReport {
            n_support_denominator: 252,
            gold_support: 160.0 / 252.0,
            wilson_lo: lo,
            wilson_hi: hi,
            ci_half_width: (hi - lo) / 2.0,
            kish_deff: 1.0,
            coverage_ceiling: 280.0 / 400.0,
            raw_lane_accuracy: 170.0 / 400.0,
            supported_subset_accuracy: 150.0 / 160.0,
            supported_subset_n: 160,
            correct_not_supported: 20,
        };
        let mk = || CeilingProbeOutput {
            meta: meta.clone(),
            validity: validity.clone(),
            census: census.clone(),
            report: report.clone(),
            items: vec![CeilingItem {
                case_id: "sst5:0".into(),
                gold_label: "positive".into(),
                lane: "correct",
                support: "supported",
            }],
        };
        assert_eq!(
            render_ceiling_probe_markdown(&mk()),
            render_ceiling_probe_markdown(&mk())
        );
        let md = render_ceiling_probe_markdown(&mk());
        assert!(md.contains("SUPPORT CEILING"));
        assert!(md.contains("PASS"));
        assert!(md.contains("population floor: 400 sampled"));
    }
}
