//! Issue 100 — the inductive-bias lane-seating diagnostics (Research 012,
//! arXiv:2207.08815). Two REPORT-ONLY probes, both off by default (the
//! `SuiteResult.inductive_bias` field is `None` and skipped, so
//! `results.json` is byte-identical with the flags unset):
//!
//! - **T1 `--robust-ab`** — the cross-lane robustness delta. Every armed
//!   lane (modelless + each laya checkpoint + each standard comparison
//!   lane) is re-read on the SAME word-dropout ladder as the echo gates
//!   (`echo_gates::OOD_RUNGS`, the `corrupt_state` stream). The corruption
//!   is applied to the case STATE VALUE's string leaves (seeded per case +
//!   leaf path + rung), so every lane — the modelless serializer and every
//!   encoder's own renderer — reads the identical corrupted text. Per rung:
//!   each lane's paired degradation (corrupted − clean over its own items)
//!   and, per encoder lane, the DELTA-OF-DELTAS
//!   `(enc_corr − enc_clean) − (ml_corr − ml_clean)` over the items both
//!   lanes answered (join key `(case_id, q_idx)`), with a 95% normal CI.
//!   Negative = the encoder degrades MORE than the modelless floor (the
//!   finding-2 prediction on lexically-clustered suites).
//! - **T2 `--label-smooth-ab`** — the label-irregularity axis (finding-1
//!   analog). The modelless corpus's per-label working set (the docs the
//!   engine actually consumes at its effective cap) is graph-smoothed in
//!   the engine's own hashed-bag embedding: each doc's label moves toward
//!   its k nearest neighbours' consensus, neighbours weighted by
//!   `sigmoid(β·(cos − τ))` (never softmax). Re-reading the modelless lane
//!   on the smoothed corpus measures how much accuracy the suite's label
//!   function's LOCAL IRREGULARITY carries. ⛔ The encoder half is
//!   STRUCTURALLY NULL for every frozen encoder lane (laya and the hosted
//!   lanes read no train rows), so it is recorded as such, never measured
//!   as a fabricated 0.
//!
//! G-ISO: harness-only; the serve path is untouched.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::echo_gates::{corrupt_state, state_seed};
use super::{LaneResult, Prepared};
use crate::embed::{EMBED_DIM, Embedder};
use crate::harness::item_dump::ItemOutcome;
use crate::harness::suites::{Suite, SuiteCase, TrainDoc};
use crate::pyjson::serialize_state;

/// T2 mixing strengths α (the neighbourhood weight; 1 − α stays on the
/// doc's own label). Every rung sits ABOVE 0.5: at α ≤ 0.5 the own-label
/// term (1 − α) ≥ α·m(l) for every l, so no doc can move (measured — the
/// first run relabelled 0/256 at α 0.25 and 0.50).
pub(crate) const SMOOTH_ALPHAS: [f64; 3] = [0.60, 0.75, 0.90];
/// T2 neighbourhood size.
pub(crate) const SMOOTH_K: usize = 10;
/// T2 sigmoid sharpness over cosine.
pub(crate) const SMOOTH_BETA: f64 = 20.0;
/// T2 sigmoid centre over cosine.
pub(crate) const SMOOTH_TAU: f64 = 0.30;

// ── corruption (T1) ─────────────────────────────────────────────────────

/// Word-dropout every string leaf of a case state. The seed folds the leaf
/// PATH into the echo-gate key, so two leaves never share a draw stream
/// and the stream is byte-stable across runs/boxes. Keys are never altered.
pub(crate) fn corrupt_value(v: &Value, p: f64, case_id: &str, path: &str) -> Value {
    match v {
        Value::String(s) => Value::String(corrupt_state(
            s,
            p,
            state_seed(&format!("{case_id}/{path}"), p),
        )),
        Value::Array(a) => Value::Array(
            a.iter()
                .enumerate()
                .map(|(i, x)| corrupt_value(x, p, case_id, &format!("{path}[{i}]")))
                .collect(),
        ),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, x)| {
                    (
                        k.clone(),
                        corrupt_value(x, p, case_id, &format!("{path}.{k}")),
                    )
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// The eval suite with every case state corrupted at rung `p`. Questions
/// and gold are untouched — only the state text degrades.
pub(crate) fn corrupt_suite(suite: &Suite, p: f64) -> Suite {
    Suite {
        name: suite.name,
        option_counts_note: suite.option_counts_note,
        cases: suite
            .cases
            .iter()
            .map(|c| SuiteCase {
                id: c.id.clone(),
                state: corrupt_value(&c.state, p, &c.id, ""),
                questions: c.questions.clone(),
                gold: c.gold.clone(),
            })
            .collect(),
    }
}

impl Prepared {
    /// A probe copy reading a different EVAL suite (corpus, cal slice and
    /// labels untouched). `slices` is dropped: the copy is never published.
    pub(super) fn with_eval_suite(&self, suite: Suite) -> Prepared {
        let state_strs = suite
            .cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        Prepared {
            suite,
            train: self.train.clone(),
            state_strs,
            cal_cases: self.cal_cases.clone(),
            cal_state_strs: self.cal_state_strs.clone(),
            labels: self.labels.clone(),
            pool_rows: self.pool_rows.clone(),
            slices: None,
        }
    }

    /// A probe copy reading a different CORPUS (eval + cal untouched).
    pub(super) fn with_train(&self, train: Vec<TrainDoc>) -> Prepared {
        Prepared {
            suite: self.suite.clone(),
            train,
            state_strs: self.state_strs.clone(),
            cal_cases: self.cal_cases.clone(),
            cal_state_strs: self.cal_state_strs.clone(),
            labels: self.labels.clone(),
            pool_rows: self.pool_rows.clone(),
            slices: None,
        }
    }
}

// ── statistics ──────────────────────────────────────────────────────────

/// Mean with a 95% normal CI over per-item values. `n < 2` reads a
/// zero-width interval at the mean (no spread is expressible).
pub(crate) fn mean_ci95(d: &[f64]) -> (f64, f64, f64) {
    let n = d.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let m = d.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m, m, m);
    }
    let var = d.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (n - 1) as f64;
    let h = 1.96 * (var / n as f64).sqrt();
    (m, m - h, m + h)
}

type ItemKey = (String, usize);

fn ok_map(items: &[ItemOutcome]) -> BTreeMap<ItemKey, bool> {
    items
        .iter()
        .map(|it| ((it.case_id.clone(), it.q_idx), it.ok))
        .collect()
}

/// One lane's paired degradation at one rung (its own clean ∩ corrupted
/// items).
#[derive(Debug, Clone, Serialize)]
pub struct LaneDegradation {
    pub lane: String,
    pub n: usize,
    pub acc_clean: f64,
    pub acc_corrupted: f64,
    /// corrupted − clean (≤ 0 = degrades).
    pub delta: f64,
    pub delta_lb95: f64,
    pub delta_ub95: f64,
}

pub(crate) fn lane_degradation(
    lane: &str,
    clean: &[ItemOutcome],
    corrupted: &[ItemOutcome],
) -> LaneDegradation {
    let c = ok_map(clean);
    let x = ok_map(corrupted);
    let mut d = Vec::with_capacity(c.len());
    let (mut a, mut b) = (0usize, 0usize);
    for (k, &okc) in &c {
        if let Some(&okx) = x.get(k) {
            a += usize::from(okc);
            b += usize::from(okx);
            d.push(f64::from(okx) - f64::from(okc));
        }
    }
    let n = d.len();
    let (m, lb, ub) = mean_ci95(&d);
    let denom = n.max(1) as f64;
    LaneDegradation {
        lane: lane.to_string(),
        n,
        acc_clean: a as f64 / denom,
        acc_corrupted: b as f64 / denom,
        delta: m,
        delta_lb95: lb,
        delta_ub95: ub,
    }
}

/// One encoder lane vs the modelless floor at one rung — the
/// delta-of-deltas over the items BOTH lanes answered clean AND corrupted.
#[derive(Debug, Clone, Serialize)]
pub struct CrossLaneDelta {
    pub encoder: String,
    pub n_paired: usize,
    pub modelless_delta: f64,
    pub encoder_delta: f64,
    /// encoder_delta − modelless_delta (< 0 = the encoder degrades more).
    pub delta_of_deltas: f64,
    pub dod_lb95: f64,
    pub dod_ub95: f64,
}

pub(crate) fn cross_lane_delta(
    encoder: &str,
    ml_clean: &[ItemOutcome],
    ml_corr: &[ItemOutcome],
    enc_clean: &[ItemOutcome],
    enc_corr: &[ItemOutcome],
) -> CrossLaneDelta {
    let (mc, mx, ec, ex) = (
        ok_map(ml_clean),
        ok_map(ml_corr),
        ok_map(enc_clean),
        ok_map(enc_corr),
    );
    let mut dm = Vec::new();
    let mut de = Vec::new();
    let mut dod = Vec::new();
    for (k, &e0) in &ec {
        let (Some(&e1), Some(&m0), Some(&m1)) = (ex.get(k), mc.get(k), mx.get(k)) else {
            continue;
        };
        let m = f64::from(m1) - f64::from(m0);
        let e = f64::from(e1) - f64::from(e0);
        dm.push(m);
        de.push(e);
        dod.push(e - m);
    }
    let (dodm, lb, ub) = mean_ci95(&dod);
    CrossLaneDelta {
        encoder: encoder.to_string(),
        n_paired: dod.len(),
        modelless_delta: mean_ci95(&dm).0,
        encoder_delta: mean_ci95(&de).0,
        delta_of_deltas: dodm,
        dod_lb95: lb,
        dod_ub95: ub,
    }
}

/// One rung of T1.
#[derive(Debug, Clone, Serialize)]
pub struct RobustRung {
    pub dropout_p: f64,
    pub lanes: Vec<LaneDegradation>,
    pub cross: Vec<CrossLaneDelta>,
}

// ── label smoothing (T2) ────────────────────────────────────────────────

#[inline]
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// The engine's per-label working set: the first `cap` docs of each label
/// in pool order (exactly what `per_label_corpus_docs` consumes), returned
/// as pool indices in pool order.
pub(crate) fn working_set(train: &[TrainDoc], cap: usize) -> Vec<usize> {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for (i, d) in train.iter().enumerate() {
        let c = seen.entry(d.label.as_str()).or_insert(0);
        if *c < cap {
            *c += 1;
            out.push(i);
        }
    }
    out
}

/// Graph-smooth the labels of `docs` at mixing strength `alpha`. Each doc
/// scores label l as `(1−α)·[l = own] + α·m(l)`, `m` the
/// sigmoid-weighted share of its k nearest neighbours (cosine in the
/// engine's hashed-bag space, self excluded) carrying l; the new label is
/// the arg-max, the doc's OWN label winning every tie (then first-seen
/// order — deterministic). Returns the relabelled docs and the relabel
/// count.
pub(crate) fn smooth_labels(docs: &[TrainDoc], alpha: f64, k: usize) -> (Vec<TrainDoc>, usize) {
    let n = docs.len();
    if n < 2 || alpha <= 0.0 {
        return (docs.to_vec(), 0);
    }
    let emb = Embedder;
    let mut e = vec![0f32; n * EMBED_DIM];
    for (i, d) in docs.iter().enumerate() {
        emb.embed_into(
            d.text.as_bytes(),
            &mut e[i * EMBED_DIM..(i + 1) * EMBED_DIM],
        );
    }
    let mut out = docs.to_vec();
    let mut moved = 0usize;
    let mut sims: Vec<(f32, usize)> = Vec::with_capacity(n);
    let mut mass: Vec<(&str, f64)> = Vec::with_capacity(k + 1);
    for i in 0..n {
        let ei = &e[i * EMBED_DIM..(i + 1) * EMBED_DIM];
        sims.clear();
        for j in 0..n {
            if j == i {
                continue;
            }
            let ej = &e[j * EMBED_DIM..(j + 1) * EMBED_DIM];
            let s: f32 = ei.iter().zip(ej).map(|(a, b)| a * b).sum();
            sims.push((s, j));
        }
        let kk = k.min(sims.len());
        // Strict total order: similarity desc, then index asc (no ties).
        let cmp = |a: &(f32, usize), b: &(f32, usize)| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1));
        sims.select_nth_unstable_by(kk - 1, cmp);
        let nb = &mut sims[..kk];
        nb.sort_unstable_by(cmp);
        mass.clear();
        let own = docs[i].label.as_str();
        mass.push((own, 0.0));
        let mut wsum = 0.0f64;
        for &(s, j) in nb.iter() {
            let w = sigmoid(SMOOTH_BETA * (f64::from(s) - SMOOTH_TAU));
            wsum += w;
            let l = docs[j].label.as_str();
            match mass.iter_mut().find(|(x, _)| *x == l) {
                Some(slot) => slot.1 += w,
                None => mass.push((l, w)),
            }
        }
        if wsum <= 0.0 {
            continue;
        }
        let score = |(l, m): &(&str, f64)| {
            let selfw = if *l == own { 1.0 - alpha } else { 0.0 };
            selfw + alpha * m / wsum
        };
        let mut best = 0usize;
        let mut best_s = score(&mass[0]);
        for (idx, cand) in mass.iter().enumerate().skip(1) {
            let s = score(cand);
            if s > best_s {
                best = idx;
                best_s = s;
            }
        }
        if best != 0 {
            out[i].label = mass[best].0.to_string();
            moved += 1;
        }
    }
    (out, moved)
}

/// The smoothed corpus: the FULL pool with only the working-set docs
/// relabelled in place (pool order kept). Docs outside the working set are
/// untouched, so the uncapped count-table views (nb/oc/ridge) still read
/// the whole pool — the label is the ONE changed variable. Returns
/// (pool, relabelled, working-set size).
pub(crate) fn smoothed_corpus(
    train: &[TrainDoc],
    cap: usize,
    alpha: f64,
) -> (Vec<TrainDoc>, usize, usize) {
    let ws = working_set(train, cap);
    let docs: Vec<TrainDoc> = ws.iter().map(|&i| train[i].clone()).collect();
    let (sm, moved) = smooth_labels(&docs, alpha, SMOOTH_K);
    let mut pool = train.to_vec();
    for (&i, d) in ws.iter().zip(sm) {
        pool[i] = d;
    }
    (pool, moved, ws.len())
}

/// One α of T2.
#[derive(Debug, Clone, Serialize)]
pub struct SmoothRung {
    pub alpha: f64,
    pub working_set: usize,
    pub relabelled: usize,
    pub relabel_rate: f64,
    pub modelless: LaneDegradation,
}

/// T2's block.
#[derive(Debug, Clone, Serialize)]
pub struct LabelSmoothBlock {
    pub effective_cap: usize,
    pub k: usize,
    pub beta: f64,
    pub tau: f64,
    /// The encoder half's disposition — frozen lanes read no train rows,
    /// so their smoothed read equals their clean read by construction.
    pub encoder_half: &'static str,
    pub rungs: Vec<SmoothRung>,
}

/// The per-suite Issue-100 block (`SuiteResult.inductive_bias`).
#[derive(Debug, Clone, Serialize)]
pub struct InductiveBiasBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robust: Option<Vec<RobustRung>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_smooth: Option<LabelSmoothBlock>,
}

/// The encoder lanes a probe pairs against the floor: every laya
/// checkpoint row + the standard comparison rows, by display name.
pub(crate) fn encoder_rows<'a>(
    laya: &'a BTreeMap<String, LaneResult>,
    comparison: impl IntoIterator<Item = (&'static str, Option<&'a LaneResult>)>,
) -> Vec<(String, &'a LaneResult)> {
    let mut v: Vec<(String, &LaneResult)> = laya
        .iter()
        .map(|(ck, r)| (format!("laya/{ck}"), r))
        .collect();
    v.extend(
        comparison
            .into_iter()
            .filter_map(|(n, r)| r.map(|r| (n.to_string(), r))),
    );
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(case: &str, q: usize, ok: bool) -> ItemOutcome {
        ItemOutcome {
            lane: "t",
            model: String::new(),
            case_id: case.to_string(),
            case_idx: 0,
            q_idx: q,
            kind: "choice".into(),
            gold: 0,
            pick: usize::from(!ok),
            ok,
            conf: 0.0,
            conf_cal: None,
            abstain_raw: None,
            abstain_cal: None,
            cause_cal: None,
        }
    }

    #[test]
    fn corrupt_value_touches_only_string_leaves_and_is_seed_stable() {
        let v =
            serde_json::json!({"text": "a b c d e f g h i j", "n": 3, "tags": ["x y z w", true]});
        let a = corrupt_value(&v, 0.5, "c1", "");
        let b = corrupt_value(&v, 0.5, "c1", "");
        assert_eq!(a, b, "same seed → same bytes");
        assert_eq!(a["n"], 3);
        assert_eq!(a["tags"][1], true);
        let kept = a["text"].as_str().unwrap().split_whitespace().count();
        assert!(
            (1..10).contains(&kept),
            "dropout must remove some, keep some: {kept}"
        );
        assert_eq!(corrupt_value(&v, 0.0, "c1", ""), v, "p = 0 is the identity");
    }

    #[test]
    fn mean_ci95_hand_values() {
        let (m, lb, ub) = mean_ci95(&[1.0, 0.0, 1.0, 0.0]);
        let h = 1.96 * ((1.0 / 3.0) / 4.0f64).sqrt();
        assert!(
            (m - 0.5).abs() < 1e-12
                && (lb - (0.5 - h)).abs() < 1e-12
                && (ub - (0.5 + h)).abs() < 1e-12
        );
        assert_eq!(mean_ci95(&[]), (0.0, 0.0, 0.0));
    }

    #[test]
    fn cross_lane_delta_pairs_on_the_join_key_only() {
        // modelless: robust (stays right). encoder: breaks on c1.
        let ml0 = vec![item("c1", 0, true), item("c2", 0, true)];
        let ml1 = vec![item("c1", 0, true), item("c2", 0, true)];
        let e0 = vec![
            item("c1", 0, true),
            item("c2", 0, true),
            item("c3", 0, true),
        ];
        let e1 = vec![
            item("c1", 0, false),
            item("c2", 0, true),
            item("c3", 0, false),
        ];
        let d = cross_lane_delta("enc", &ml0, &ml1, &e0, &e1);
        assert_eq!(d.n_paired, 2, "c3 has no modelless pair");
        assert!((d.delta_of_deltas + 0.5).abs() < 1e-12);
        assert!((d.encoder_delta + 0.5).abs() < 1e-12 && d.modelless_delta == 0.0);
        let l = lane_degradation("enc", &e0, &e1);
        assert_eq!(l.n, 3);
        assert!((l.acc_clean - 1.0).abs() < 1e-12 && (l.acc_corrupted - 1.0 / 3.0).abs() < 1e-12);
    }

    fn doc(l: &str, t: &str) -> TrainDoc {
        TrainDoc {
            label: l.into(),
            text: t.into(),
        }
    }

    #[test]
    fn smoothing_moves_an_island_and_keeps_consistent_docs() {
        // Five near-identical "apple" docs, one mislabelled; two distinct
        // "car" docs keep their label.
        let mut docs: Vec<TrainDoc> = (0..5)
            .map(|i| doc("fruit", &format!("red apple sweet fruit {i}")))
            .collect();
        docs[2].label = "vehicle".into();
        docs.push(doc("vehicle", "fast car engine wheel road"));
        docs.push(doc("vehicle", "fast car engine wheel highway"));
        let (out, moved) = smooth_labels(&docs, 0.75, 4);
        assert_eq!(
            out[2].label, "fruit",
            "the mislabelled island joins its neighbours"
        );
        assert!(moved >= 1);
        let (out0, moved0) = smooth_labels(&docs, 0.0, 4);
        assert_eq!((out0, moved0), (docs.clone(), 0), "α = 0 is the identity");
        let (again, _) = smooth_labels(&docs, 0.75, 4);
        assert_eq!(again, out, "deterministic");
    }

    #[test]
    fn working_set_is_the_engine_cap_in_pool_order() {
        let docs = vec![
            doc("a", "1"),
            doc("b", "2"),
            doc("a", "3"),
            doc("a", "4"),
            doc("b", "5"),
        ];
        assert_eq!(working_set(&docs, 2), vec![0, 1, 2, 4]);
    }

    #[test]
    fn smoothed_corpus_keeps_the_pool_and_alpha_half_is_inert() {
        let docs = vec![
            doc("a", "x y"),
            doc("b", "x y"),
            doc("a", "x y"),
            doc("a", "z w"),
        ];
        let (pool, moved, n) = smoothed_corpus(&docs, 1, 0.9);
        assert_eq!(pool.len(), docs.len(), "out-of-working-set docs are kept");
        assert_eq!(n, 2);
        assert_eq!(&pool[2..], &docs[2..], "only working-set rows may change");
        assert!(moved <= n);
        let mut big: Vec<TrainDoc> = (0..6)
            .map(|i| doc("fruit", &format!("red apple {i}")))
            .collect();
        big[1].label = "x".into();
        assert_eq!(
            smooth_labels(&big, 0.5, 4).1,
            0,
            "α = 0.5: the own term never loses"
        );
    }
}
