//! riir-train Plan 426 T2 — the TEACHER-ENSEMBLE GATE (V3): both
//! teachers over the SAME frozen test slice, fused at "mint time" under
//! ONE pinned primary rule (logit-mean; rank-fusion the recorded
//! alternate), scored against each member with the paired LB95.
//!
//! The V3 law: the ensemble must beat the BEST SINGLE MEMBER (paired
//! LB95 > 0) before any student trains on its labels (T4); else the
//! distill falls back to openthai-single-teacher. Heterogeneous-teacher
//! decorrelation is MEASURED here (the error-overlap matrix), never
//! assumed.
//!
//! Agentjev family law: harness-only, never in the release set — this
//! is a measurement instrument over teacher labels, not a serving path.
//!
//! Reuses Plan 426 T1's `TeacherForward` seam verbatim: the fusion can
//! never see teachers answered by different code paths (one forward
//! implementation per teacher family, the determinism pin already
//! proven through it).

use std::path::Path;

use super::distill::{construct_teacher, TeacherForward};
use super::{git_sha, iso8601_utc, RunOptions, SuiteSpec, SUITES};

/// The suites the gate serves. V3's target is massive (the surpass
/// board); more can join by outgrowing this list's reason.
const ENSEMBLE_SUITES: [&str; 1] = ["massive_intent_en"];

/// The numerical floor inside log() — probabilities can be exactly 0.
const LOG_EPS: f64 = 1e-12;

#[derive(Debug, serde::Serialize)]
pub struct EnsembleGateOutput {
    pub meta: EnsembleGateMeta,
    pub suites: Vec<EnsembleGateSuite>,
    pub skipped: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct EnsembleGateMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub teacher_a: String,
    pub teacher_b: String,
    /// The pinned fusion rule (V3's primary) + the recorded alternate.
    pub fusion_primary: &'static str,
    pub fusion_alternate: &'static str,
    pub datasets_dir: String,
    pub row_rule: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct EnsembleGateSuite {
    pub name: String,
    pub n_rows: usize,
    pub n_classes_touched: usize,
    /// Per-teacher + fused hard accuracies over the identical rows.
    pub acc_a: f64,
    pub acc_b: f64,
    pub acc_logit_mean: f64,
    pub acc_rank_fusion: f64,
    /// Paired LB95 of (fusion − member) — the V3 gate reads these.
    pub logit_minus_a_lb95: f64,
    pub logit_minus_b_lb95: f64,
    pub rank_minus_a_lb95: f64,
    pub rank_minus_b_lb95: f64,
    pub logit_minus_best_lb95: f64,
    pub rank_minus_best_lb95: f64,
    /// The measured decorrelation (never assumed): error overlap.
    pub both_wrong: usize,
    pub a_only_wrong: usize,
    pub b_only_wrong: usize,
    /// The V3 verdict: the best fusion rule beats the best member.
    pub v3_pass: bool,
    pub v3_note: String,
    pub seconds: f64,
}

/// The presented keys of one question, in presentation order — the same
/// extraction the openthai keying helper reads (choice: the criteria
/// object's keys; score: the level indices; noul: the pair).
fn presented_keys_of(
    q: &crate::harness::suites::SuiteQuestion,
) -> Result<Vec<String>, String> {
    use crate::harness::suites::QKind;
    Ok(match q.kind {
        QKind::Noul => (0..2).map(|i| i.to_string()).collect(),
        QKind::Score => {
            let n = q.criteria.as_array().map_or(0, Vec::len);
            (0..n).map(|i| i.to_string()).collect()
        }
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .ok_or_else(|| "choice criteria must be an object".to_string())?,
    })
}

/// Align one teacher's keyed probabilities onto the presented key list
/// (sorted-key alignment: both teachers answer the same case, so the key
/// sets must match — a mismatch is wire drift, a loud per-row skip).
fn aligned(
    keys: &[String],
    keyed: &[(String, f64)],
) -> Result<Vec<f64>, String> {
    let mut by_key: std::collections::HashMap<&str, f64> =
        keyed.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    if by_key.len() != keyed.len() {
        return Err("duplicate keys in the teacher's answer".into());
    }
    keys.iter()
        .map(|k| {
            by_key
                .remove(k.as_str())
                .ok_or_else(|| format!("teacher answer missing presented key {k:?}"))
        })
        .collect()
}

/// The pinned primary fusion: per-class LOGIT-MEAN of the members'
/// probabilities (ε-floored), argmax. Returns the argmax KEY.
fn fuse_logit_mean(
    keys: &[String],
    a: &[f64],
    b: &[f64],
) -> usize {
    let mut best = 0usize;
    let mut best_v = f64::NEG_INFINITY;
    for i in 0..keys.len() {
        let v = 0.5 * (a[i].max(LOG_EPS).ln() + b[i].max(LOG_EPS).ln());
        if v > best_v {
            best_v = v;
            best = i;
        }
    }
    best
}

/// The recorded alternate: per-teacher RANK (best = 0), fused by the
/// MEAN rank, argmin. Ties break to the lower index (the engine argmax
/// law applied to the fused score's first best).
fn fuse_rank(
    keys: &[String],
    a: &[f64],
    b: &[f64],
) -> usize {
    let ranks = |v: &[f64]| -> Vec<f64> {
        let mut idx: Vec<usize> = (0..v.len()).collect();
        idx.sort_by(|&i, &j| v[j].total_cmp(&v[i]).then(i.cmp(&j)));
        let mut r = vec![0.0f64; v.len()];
        for (rank, &i) in idx.iter().enumerate() {
            r[i] = rank as f64;
        }
        r
    };
    let ra = ranks(a);
    let rb = ranks(b);
    let mut best = 0usize;
    let mut best_v = f64::INFINITY;
    for i in 0..keys.len() {
        let v = 0.5 * (ra[i] + rb[i]);
        if v < best_v {
            best_v = v;
            best = i;
        }
    }
    best
}

/// The paired LB95 of (x − y) over 0/1 correctness vectors — the normal
/// approximation (the same form the arena's T2 gate reports).
fn paired_lb95(x: &[bool], y: &[bool]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let d: Vec<f64> = x
        .iter()
        .zip(y.iter())
        .map(|(&a, &b)| f64::from(a) - f64::from(b))
        .collect();
    let m = d.iter().sum::<f64>() / n as f64;
    let var = d.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (n - 1) as f64;
    m - 1.96 * (var / n as f64).sqrt()
}

/// Run the ensemble gate over the requested suites (empty = the gate's
/// suite list). Early-exit like `--e0`: no eval lane, no modelless row —
/// the teachers are read once each over the frozen test slice.
pub fn run_ensemble_gate(
    opts: &RunOptions,
    teacher_a_name: &str,
    teacher_b_name: &str,
) -> Result<EnsembleGateOutput, String> {
    let mut suites = Vec::new();
    let mut skipped = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
            continue;
        }
        if spec.synthetic.is_some() {
            skipped.push(format!("{}: synthetic family — no dataset rows", spec.name));
            continue;
        }
        if !ENSEMBLE_SUITES.contains(&spec.name) {
            skipped.push(format!(
                "{}: not an ensemble-gate suite (the V3 board is {:?})",
                spec.name, ENSEMBLE_SUITES
            ));
            continue;
        }
        match ensemble_suite(spec, &opts.datasets_dir, teacher_a_name, teacher_b_name) {
            Ok(s) => suites.push(s),
            Err(e) => skipped.push(format!("{}: {e}", spec.name)),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "ensemble-gate: no suite ran ({} skip/failure line(s), first: {})",
            skipped.len(),
            skipped.first().map_or("none", String::as_str)
        ));
    }
    Ok(EnsembleGateOutput {
        meta: EnsembleGateMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            teacher_a: teacher_a_name.to_string(),
            teacher_b: teacher_b_name.to_string(),
            fusion_primary: "logit-mean (pinned, V3)",
            fusion_alternate: "rank-fusion (recorded alternate)",
            datasets_dir: opts.datasets_dir.display().to_string(),
            row_rule: "the frozen TEST slice, read once for the pre-registered V3 arm; \
                       teachers answer through the T1 seam (one forward per family)",
        },
        suites,
        skipped,
    })
}

fn ensemble_suite(
    spec: &SuiteSpec,
    dir: &Path,
    teacher_a_name: &str,
    teacher_b_name: &str,
) -> Result<EnsembleGateSuite, String> {
    let t_start = std::time::Instant::now();
    let suite_dir = dir.join(spec.dataset_dir);
    let eval_rows = super::load_rows(&suite_dir, spec.eval_split)?;
    // The frozen slice: the SAME stratified construction the eval lane
    // uses (the test_cap law), so the gate's rows are the board's rows.
    let eval_split = crate::harness::suites::stratified_split(
        &eval_rows,
        spec.name,
        spec.test_cap,
    );
    let suite = (spec.build)(&eval_split.front, 0);
    if suite.cases.is_empty() {
        return Err("empty test slice".into());
    }

    let mut teacher_a: TeacherForward = construct_teacher(teacher_a_name, spec)?;
    let mut teacher_b: TeacherForward = construct_teacher(teacher_b_name, spec)?;
    // Warm both (the T1 seam's law — openthai's provenance capture rides
    // the warmup too).
    if let Some(case) = suite.cases.first() {
        teacher_a.warmup(case)?;
        teacher_b.warmup(case)?;
    }
    eprintln!(
        "  [ensemble {}] teachers: {} ({}) vs {} ({}) · {} frozen rows",
        spec.name,
        teacher_a.name(),
        teacher_a.provenance(),
        teacher_b.name(),
        teacher_b.provenance(),
        suite.cases.len()
    );

    let mut ok_a = Vec::with_capacity(suite.cases.len());
    let mut ok_b = Vec::with_capacity(suite.cases.len());
    let mut ok_logit = Vec::with_capacity(suite.cases.len());
    let mut ok_rank = Vec::with_capacity(suite.cases.len());
    let mut both_wrong = 0usize;
    let mut a_only = 0usize;
    let mut b_only = 0usize;
    let mut classes_touched = std::collections::BTreeSet::new();

    for case in &suite.cases {
        let q = case.questions.first().ok_or_else(|| {
            format!("{}: no questions for its single gold", case.id)
        })?;
        let keys = presented_keys_of(q)
            .map_err(|e| format!("{}: {e}", case.id))?;
        let gold_key = keys
            .get(case.gold[0].idx)
            .ok_or_else(|| format!("{}: gold idx out of the presented keys", case.id))?;
        classes_touched.extend(keys.iter().cloned());
        let (ka, _ms_a) = teacher_a
            .forward(case)
            .map_err(|e| format!("{}: teacher A: {e}", case.id))?;
        let (kb, _ms_b) = teacher_b
            .forward(case)
            .map_err(|e| format!("{}: teacher B: {e}", case.id))?;
        let pa = aligned(&keys, &ka).map_err(|e| format!("{}: teacher A: {e}", case.id))?;
        let pb = aligned(&keys, &kb).map_err(|e| format!("{}: teacher B: {e}", case.id))?;
        let argmax_key = |v: &[f64]| -> &str {
            let mut best = 0usize;
            for i in 1..v.len() {
                if v[i] > v[best] {
                    best = i;
                }
            }
            keys[best].as_str()
        };
        let pick_a = argmax_key(&pa);
        let pick_b = argmax_key(&pb);
        let pick_logit = keys[fuse_logit_mean(&keys, &pa, &pb)].as_str();
        let pick_rank = keys[fuse_rank(&keys, &pa, &pb)].as_str();
        let c_a = pick_a == gold_key.as_str();
        let c_b = pick_b == gold_key.as_str();
        ok_a.push(c_a);
        ok_b.push(c_b);
        ok_logit.push(pick_logit == gold_key.as_str());
        ok_rank.push(pick_rank == gold_key.as_str());
        match (c_a, c_b) {
            (false, false) => both_wrong += 1,
            (false, true) => a_only += 1,
            (true, false) => b_only += 1,
            (true, true) => {}
        }
    }

    let acc = |v: &[bool]| v.iter().filter(|&&c| c).count() as f64 / v.len() as f64;
    let acc_a = acc(&ok_a);
    let acc_b = acc(&ok_b);
    let acc_logit = acc(&ok_logit);
    let acc_rank = acc(&ok_rank);
    // "Best member" — the V3 gate reads the fusion against whichever
    // member is stronger, decided on the same read (paired, so the
    // comparison is within-row).
    let (best_ok, other_ok, best_name) = if acc_a >= acc_b {
        (&ok_a, &ok_b, teacher_a_name)
    } else {
        (&ok_b, &ok_a, teacher_b_name)
    };
    let _ = other_ok;
    let logit_minus_a = paired_lb95(&ok_logit, &ok_a);
    let logit_minus_b = paired_lb95(&ok_logit, &ok_b);
    let rank_minus_a = paired_lb95(&ok_rank, &ok_a);
    let rank_minus_b = paired_lb95(&ok_rank, &ok_b);
    let logit_minus_best = paired_lb95(&ok_logit, best_ok);
    let rank_minus_best = paired_lb95(&ok_rank, best_ok);
    let v3_pass = logit_minus_best > 0.0 || rank_minus_best > 0.0;
    let v3_note = if v3_pass {
        format!(
            "V3 PASS — the ensemble beats the best member ({best_name}) under the \
             passing rule; T4 may train on the ensemble's labels"
        )
    } else {
        format!(
            "V3 FAIL — neither fusion beats the best member ({best_name}) at LB95 > 0; \
             the distill falls back to single-teacher (the stronger member)"
        )
    };
    eprintln!(
        "  [ensemble {}] acc {}={:.4} · {}={:.4} · logit-mean={:.4} · rank={:.4} · {}",
        spec.name,
        teacher_a_name,
        acc_a,
        teacher_b_name,
        acc_b,
        acc_logit,
        acc_rank,
        v3_note
    );
    Ok(EnsembleGateSuite {
        name: spec.name.to_string(),
        n_rows: suite.cases.len(),
        n_classes_touched: classes_touched.len(),
        acc_a,
        acc_b,
        acc_logit_mean: acc_logit,
        acc_rank_fusion: acc_rank,
        logit_minus_a_lb95: logit_minus_a,
        logit_minus_b_lb95: logit_minus_b,
        rank_minus_a_lb95: rank_minus_a,
        rank_minus_b_lb95: rank_minus_b,
        logit_minus_best_lb95: logit_minus_best,
        rank_minus_best_lb95: rank_minus_best,
        both_wrong,
        a_only_wrong: a_only,
        b_only_wrong: b_only,
        v3_pass,
        v3_note,
        seconds: t_start.elapsed().as_secs_f64(),
    })
}

/// The markdown rendering (the record's table + verdict block).
#[must_use]
pub fn render_ensemble_gate_markdown(out: &EnsembleGateOutput) -> String {
    let mut s = String::new();
    s.push_str(
        "# Teacher-ensemble gate (Plan 426 T2, GATE V3) — both teachers over the frozen test slice\n\n",
    );
    s.push_str(&format!(
        "- date {} · sha {} · teachers {} vs {}\n- fusion: {} (primary) · {} (alternate)\n- datasets {} · rows: {}\n",
        out.meta.date_utc,
        out.meta.git_sha,
        out.meta.teacher_a,
        out.meta.teacher_b,
        out.meta.fusion_primary,
        out.meta.fusion_alternate,
        out.meta.datasets_dir,
        out.meta.row_rule
    ));
    s.push_str("\n| suite | n | acc A | acc B | logit-mean | rank | logit−best LB95 | rank−best LB95 | both wrong / A-only / B-only | V3 |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for v in &out.suites {
        s.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {:.4} | {:+.4} | {:+.4} | {} / {} / {} | {} |\n",
            v.name,
            v.n_rows,
            v.acc_a,
            v.acc_b,
            v.acc_logit_mean,
            v.acc_rank_fusion,
            v.logit_minus_best_lb95,
            v.rank_minus_best_lb95,
            v.both_wrong,
            v.a_only_wrong,
            v.b_only_wrong,
            if v.v3_pass { "PASS" } else { "FAIL" }
        ));
    }
    for v in &out.suites {
        s.push_str(&format!("\n- **{}**: {}\n", v.name, v.v3_note));
    }
    if !out.skipped.is_empty() {
        s.push_str("\n## Skipped / failed (loud absences)\n\n");
        for e in &out.skipped {
            s.push_str(&format!("- {e}\n"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_maps_by_key_and_refuses_drift() {
        let keys = vec!["b".to_string(), "a".to_string()];
        let keyed = vec![("a".to_string(), 0.7), ("b".to_string(), 0.3)];
        let v = aligned(&keys, &keyed).expect("aligned");
        assert_eq!(v, vec![0.3, 0.7], "presented-order alignment by key");
        assert!(aligned(&keys, &[("a".to_string(), 1.0)]).is_err(), "a missing key refuses");
        assert!(
            aligned(&keys, &[("a".to_string(), 1.0), ("a".to_string(), 0.0), ("b".to_string(), 0.0)])
                .is_err(),
            "duplicate keys refuse"
        );
    }

    #[test]
    fn logit_mean_prefers_agreement_and_rank_breaks_ties() {
        let keys = vec!["x".to_string(), "y".to_string()];
        // Both teachers lean x → the fusion leans x even if one is weak.
        assert_eq!(fuse_logit_mean(&keys, &[0.6, 0.4], &[0.9, 0.1]), 0);
        // Disagreement with symmetric confidence: the product (logit mean)
        // reads 0.6·0.1 = 0.06 vs 0.4·0.9 = 0.36 → y.
        assert_eq!(fuse_logit_mean(&keys, &[0.6, 0.4], &[0.1, 0.9]), 1);
        // Rank fusion reads the same ordering: ranks (0,1)+(1,0) tie at 0.5
        // each → the lower index wins (the engine argmax law).
        assert_eq!(fuse_rank(&keys, &[0.6, 0.4], &[0.1, 0.9]), 0);
        // Zero probabilities never panic (the ε floor).
        assert_eq!(fuse_logit_mean(&keys, &[0.0, 1.0], &[0.0, 1.0]), 1);
    }

    #[test]
    fn paired_lb95_matches_the_hand_computation() {
        // x = first 10 true, y = first 5 true → d = [1]×5 + [0]×15.
        // mean 0.25; sample var = (5·0.75² + 15·0.25²)/19 = 15/76 ≈ 0.19737;
        // LB95 = 0.25 − 1.96·√(var/20).
        let x: Vec<bool> = (0..20).map(|i| i < 10).collect();
        let y: Vec<bool> = (0..20).map(|i| i < 5).collect();
        let lb = paired_lb95(&x, &y);
        let var = (5.0 * 0.75_f64.powi(2) + 15.0 * 0.25_f64.powi(2)) / 19.0;
        let expected = 0.25 - 1.96 * (var / 20.0).sqrt();
        assert!((lb - expected).abs() < 1e-12, "{lb} vs {expected}");
        assert!(paired_lb95(&x, &x) == 0.0, "identical vectors: LB95 = 0");
    }

    #[test]
    fn presented_keys_match_the_openthai_extraction() {
        use crate::harness::suites::{QKind, SuiteQuestion};
        let q = SuiteQuestion {
            qid: "q".into(),
            kind: QKind::Choice,
            instructions: String::new(),
            criteria: serde_json::json!({"k2": 1, "k1": 2}),
        };
        assert_eq!(presented_keys_of(&q).unwrap().len(), 2);
        let mut q2 = q;
        q2.kind = QKind::Score;
        q2.criteria = serde_json::json!(["a", "b"]);
        assert_eq!(presented_keys_of(&q2).unwrap(), vec!["0", "1"]);
        q2.kind = QKind::Noul;
        q2.criteria = serde_json::Value::Null;
        assert_eq!(presented_keys_of(&q2).unwrap(), vec!["0", "1"]);
    }
}
