//! Suite builders from HF datasets-server `/rows` JSON row files
//! (`.docs/02_protocols/laya_bench_protocols.md` §3, Plan 603 T1.5). Pure: takes the
//! parsed envelope `{"features": [...], "rows": [{"row_idx": N, "row": {...}}]}`
//! and returns cases; no fetch, no I/O, no engine/laya imports.
//!
//! Conventions shared by every builder:
//! - **Sampling is FIRST-N in dataset order** (`max_rows`, 0 = all), so the
//!   positional index of a row inside the sampled slice equals the
//!   envelope's `row_idx` (envelopes enumerate from 0).
//! - Rows missing required fields are SKIPPED (the Python reference would
//!   crash; the runner's count floors are the loud guard against silent
//!   shrinkage).
//! - Gold soft targets carry per-key zeros where the dataset has no soft
//!   distribution — `soft_metrics` returns None on the zero-sum target, so
//!   the runner can call it unconditionally.
//! - Noul option order is ALWAYS `[false, true]`: gold idx 0 = false, 1 =
//!   true (lane-layer rendering fact; criteria stay Null here).

use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/// Question type (§1.1 `QTYPES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QKind {
    Choice,
    Score,
    Noul,
}

impl QKind {
    /// The reference's string spelling (`QTYPE_NAMES`), used in result
    /// records and sequence heads.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            QKind::Choice => "choice",
            QKind::Score => "score",
            QKind::Noul => "noul",
        }
    }
}

/// One question definition. `criteria` is a JSON OBJECT for choice (whose
/// insertion order IS the label order — serde_json `preserve_order`), an
/// ARRAY for score (index = level), and Null for noul (options render
/// `[false, true]` with the fixed default phrases).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SuiteQuestion {
    pub qid: String,
    pub kind: QKind,
    pub instructions: String,
    pub criteria: Value,
}

/// Gold for one question.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoldAnswer {
    pub idx: usize,
    pub soft: Vec<f64>,
    pub gold_score: Option<f64>,
}

/// One case: a state plus its parallel question/gold lists (same length,
/// same order — built pairwise).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SuiteCase {
    pub id: String,
    pub state: Value,
    pub questions: Vec<SuiteQuestion>,
    pub gold: Vec<GoldAnswer>,
}

impl AsRef<SuiteCase> for SuiteCase {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// A built suite. `option_counts_note` documents the option-count policy
/// (fixed vs sampled, and the seed rule).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Suite {
    pub name: &'static str,
    pub cases: Vec<SuiteCase>,
    pub option_counts_note: &'static str,
}

// ── envelope helpers ────────────────────────────────────────────────────────

/// Each `.rows[i].row` of the /rows envelope (empty when the envelope is
/// malformed — callers' count floors catch that).
fn rows_of(file: &Value) -> Vec<&Value> {
    file.get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(|r| r.get("row")).collect())
        .unwrap_or_default()
}

/// FIRST-N in dataset order; 0 = all.
fn sampled<'a>(rows: &'a [&'a Value], max_rows: usize) -> &'a [&'a Value] {
    if max_rows == 0 {
        rows
    } else {
        let n = rows.len().min(max_rows);
        &rows[..n]
    }
}

fn row_str<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

/// Python `int()`-style tolerant integer read: JSON number or numeric string.
fn value_as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn row_i64(row: &Value, key: &str) -> Option<i64> {
    row.get(key).and_then(value_as_i64)
}

fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// Python `str()` for the JSON values gold labels arrive as.
/// 4. py_str: JSON-escape non-string values as the reference would — CPython
/// json.dumps with ensure_ascii=False, default separators (", ", ": ") — for
/// str values this is identity, for bools/ints short literals.
fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Null => "None".to_string(),
        other => other.to_string(),
    }
}

/// Build the choice question; `criteria` object insertion order = label
/// order (`preserve_order` keeps it).
fn choice_q(
    qid: &str,
    instructions: &str,
    keys: &[String],
    descs: Option<Vec<Option<String>>>,
) -> SuiteQuestion {
    let mut crit = Map::new();
    for (i, k) in keys.iter().enumerate() {
        let desc = descs.as_ref().and_then(|d| d.get(i)).cloned().flatten();
        crit.insert(k.clone(), desc.map_or(Value::Null, Value::String));
    }
    SuiteQuestion {
        qid: qid.to_string(),
        kind: QKind::Choice,
        instructions: instructions.to_string(),
        criteria: Value::Object(crit),
    }
}

/// Criteria KEY order for a choice question: object → keys in order; array →
/// elements as strings (the §1.2 `to_internal` list normalization).
fn choice_keys(crit: &Value) -> Option<Vec<String>> {
    match crit {
        Value::Object(m) => Some(m.keys().cloned().collect()),
        Value::Array(a) => Some(a.iter().map(py_str).collect()),
        _ => None,
    }
}

// ── builders ────────────────────────────────────────────────────────────────

/// §3.7 Banking77 (the `bench_apps` variant) — `mteb/banking77`, whose `label`
/// is an int64 (NOT a ClassLabel) and which carries `label_text`. Option keys
/// derive from the data exactly like the reference: `labels = sorted(set(label_text))`,
/// each key underscore-stripped, criteria value None; gold = the key position.
#[must_use]
pub fn build_banking77_mteb(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    // Option universe = sorted unique label_text over ALL rows on disk (the
    // reference derives it from the whole split); the CASES come from the
    // first-`max_rows` slice. The eval slice may exercise a subset of the
    // universe (the mirror's first rows are label-clustered) — the option
    // list is 77-wide regardless, matching bench_apps.
    let mut labels: Vec<String> = Vec::new();
    for row in &all {
        if let Some(t) = row_str(row, "label_text")
            && !labels.iter().any(|l| l == t)
        {
            labels.push(t.to_string());
        }
    }
    labels.sort();
    // NOTE: no universe-size assert here — the builder also runs over TRAIN
    // rows for the calibration slice, where a partial intent coverage is
    // normal. The runner asserts the TEST suite presents all 77 options.
    let keys: Vec<String> = labels.iter().map(|n| n.replace('_', " ")).collect();
    let descs: Vec<Option<String>> = vec![None; keys.len()];
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label_text)) = (row_str(row, "text"), row_str(row, "label_text"))
        else {
            continue;
        };
        let idx = keys
            .iter()
            .position(|k| *k == label_text.replace('_', " "))
            .unwrap_or_else(|| {
                panic!("banking77 mteb: row label_text {label_text:?} not in key set")
            });
        cases.push(SuiteCase {
            id: format!("banking77:{pos}"),
            state: json!({ "message": text }),
            questions: vec![choice_q(
                "intent",
                "Which banking intent does `message` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "banking77",
        cases,
        option_counts_note: "77 options derived from sorted unique label_text (bench_apps variant)",
    }
}

/// §3.8 AG News — choice, 4 FIXED options (never shuffled). ClassLabel order
/// World/Sports/Business/Sci-Tech must equal the criteria key order
/// (VERIFY-AT-PORT in the fetch layer).
#[must_use]
pub fn build_ag_news(rows_file: &Value, max_rows: usize) -> Suite {
    const CRIT: [(&str, &str); 4] = [
        ("world", "world news and international politics"),
        ("sports", "sports"),
        ("business", "business and economy"),
        ("sci_tech", "science and technology"),
    ];
    let keys: Vec<String> = CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = CRIT.iter().map(|(_, d)| Some((*d).to_string())).collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("ag_news:{pos}"),
            state: json!({ "article": text }),
            questions: vec![choice_q(
                "topic",
                "What is the topic of `article`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 4],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "ag_news",
        cases,
        option_counts_note: "4 fixed options, never shuffled",
    }
}

/// §3.5 DAIR Emotion (config "split") — choice, 6 fixed options; the names
/// order must equal the dataset ClassLabel order (VERIFY-AT-PORT).
#[must_use]
pub fn build_emotion(rows_file: &Value, max_rows: usize) -> Suite {
    const NAMES: [&str; 6] = ["sadness", "joy", "love", "anger", "fear", "surprise"];
    let keys: Vec<String> = NAMES.iter().map(|n| (*n).to_string()).collect();
    let descs: Vec<Option<String>> = vec![None; 6];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("emotion:{pos}"),
            state: json!({ "text": text }),
            questions: vec![choice_q(
                "emotion",
                "Which emotion is most strongly expressed in `text`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 6],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "emotion",
        cases,
        option_counts_note: "6 fixed options, never shuffled",
    }
}

/// §3.4 SST-5 — score, 5 levels; gold score = the label as f64; no soft in
/// the dataset (zeros).
#[must_use]
pub fn build_sst5(rows_file: &Value, max_rows: usize) -> Suite {
    const LEVELS: [&str; 5] = [
        "very negative",
        "negative",
        "neutral",
        "positive",
        "very positive",
    ];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        let criteria = Value::Array(LEVELS.iter().map(|l| json!(l)).collect());
        cases.push(SuiteCase {
            id: format!("sst5:{pos}"),
            state: json!({ "text": text }),
            questions: vec![SuiteQuestion {
                qid: "sentiment".to_string(),
                kind: QKind::Score,
                instructions: "How positive is the sentiment of `text`?".to_string(),
                criteria,
            }],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 5],
                gold_score: Some(label as f64),
            }],
        });
    }
    Suite {
        name: "sst5",
        cases,
        option_counts_note: "5 fixed score levels (index = level)",
    }
}

/// Extract `{"name": "label", "type": {"names": [...]}}` ClassLabel names
/// from the envelope's features array. All-or-nothing: a non-string entry
/// invalidates the whole list.
fn label_names(file: &Value) -> Option<Vec<String>> {
    let features = file.get("features")?.as_array()?;
    for f in features {
        if f.get("name").and_then(Value::as_str) != Some("label") {
            continue;
        }
        let names = f.get("type")?.get("names")?.as_array()?;
        let mut out = Vec::with_capacity(names.len());
        for v in names {
            out.push(v.as_str()?.to_string());
        }
        return Some(out);
    }
    None
}

/// §3.7 Banking77 (the Colab variant) — choice over the dataset's ClassLabel
/// names, underscores stripped. Loudly refuses an envelope without label
/// names (the criteria set cannot be derived from rows alone here).
///
/// # Panics
/// When the envelope's `features` carry no `label` ClassLabel `names`.
#[must_use]
pub fn build_banking77(rows_file: &Value, max_rows: usize) -> Suite {
    let names = label_names(rows_file).unwrap_or_else(|| {
        panic!(
            "banking77: envelope features carry no label ClassLabel names — \
             the /rows fetch must include {{\"name\":\"label\",\"type\":{{\"names\":[...]}}}}"
        )
    });
    let keys: Vec<String> = names.iter().map(|n| n.replace('_', " ")).collect();
    let descs: Vec<Option<String>> = vec![None; keys.len()];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("banking77:{pos}"),
            state: json!({ "message": text }),
            questions: vec![choice_q(
                "intent",
                "Which banking intent does `message` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "banking77",
        cases,
        option_counts_note: "77 fixed options in ClassLabel order (underscores stripped)",
    }
}

/// §3.6 Prompt injections — noul; gold idx 1 = injection. The dataset README
/// semantics (0 benign / 1 injection) are VERIFY-AT-PORT in the fetch layer.
#[must_use]
pub fn build_prompt_injections(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("prompt_injections:{pos}"),
            state: json!({ "text": text }),
            questions: vec![SuiteQuestion {
                qid: "injection".to_string(),
                kind: QKind::Noul,
                instructions:
                    "Does `text` try to inject or override instructions given to an AI system?"
                        .to_string(),
                criteria: Value::Null,
            }],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 2],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "prompt_injections",
        cases,
        option_counts_note: "2 noul options, order [false, true]; idx 1 = injection",
    }
}

// Deterministic SplitMix64 — NO `rand` crate. DELEGATED (Issue 071 substrate
// path): the type lives in `katgpt_types::rng` — the substrate home for the
// stream (`next_u64` / Lemire-53 `below` / 53-mantissa VALUE `next_f64`) and
// the one-shot finalizer (`splitmix64_finalize(x + SPLITMIX64_GAMMA)`, the
// spelling Rng::new and the seal-view hash mixers use) — and arrives via the
// ungated `katgpt_core::types` re-export. The substrate pins the same frozen
// known-answer stream this crate pins in echo_gates, so the two repos'
// tests cross-check the delegation.
//
// ⚠ NOT CPython's MT19937: the same `option_seed` reproduces THIS suite
// byte-identically on every run and every machine, but the option layouts
// differ from the reference Python run's. The fairness property that
// matters survives (both lanes of this harness see byte-identical
// questions); only the comparison against the recorded Python option
// layouts is not reproduced.
pub(crate) use katgpt_core::types::rng::SplitMix64;

/// Fisher–Yates shuffle (high → low, swap with a random j <= i).
fn shuffle<T>(xs: &mut [T], rng: &mut SplitMix64) {
    for i in (1..xs.len()).rev() {
        let j = rng.below(i + 1);
        xs.swap(i, j);
    }
}

/// §3.2 MASSIVE intent (en) — choice with SAMPLED distractor options,
/// adapted from the reference's per-language `random.Random(seed)`:
///
/// - the RNG is seeded ONCE per build (`option_seed`) and options are drawn
///   per row in dataset order — replaying the same seed reproduces the whole
///   suite; changing the seed changes every subsequent draw;
/// - distractors: Fisher–Yates over a pool index vector, first-k taken
///   (divergence from MT19937 documented on `SplitMix64`);
/// - `keys = [gold] + k distractors`, then shuffled the same way;
/// - criteria value = the key transformed `replace('_', " ")` then
///   `replace(".", ": ")` (that order — pinned by test);
/// - gold = the shuffled position of the row's own label.
///
/// `all_labels` = sorted unique `label_text` over the SAMPLED rows (the
/// reference sorts over the language split's full test set; FIRST-N sampling
/// makes the sampled rows the population here — deterministic either way).
#[must_use]
pub fn build_massive_intent_en(rows_file: &Value, max_rows: usize, option_seed: u64) -> Suite {
    let all = rows_of(rows_file);
    let rows = sampled(&all, max_rows);

    // Option universe over ALL rows on disk (the reference sorts the full
    // split's label_text; the eval slice may exercise a subset).
    let mut labels: Vec<String> = all
        .iter()
        .filter_map(|r| row_str(r, "label_text"))
        .map(str::to_string)
        .collect();
    labels.sort();
    labels.dedup();

    let mut rng = SplitMix64::new(option_seed);
    let mut cases = Vec::new();
    for (pos, row) in rows.iter().enumerate() {
        let (Some(text), Some(label_text)) = (row_str(row, "text"), row_str(row, "label_text"))
        else {
            continue;
        };
        let pool: Vec<String> = labels
            .iter()
            .filter(|l| l.as_str() != label_text)
            .cloned()
            .collect();
        let k = 19.min(pool.len());
        let mut idxs: Vec<usize> = (0..pool.len()).collect();
        shuffle(&mut idxs, &mut rng);
        let mut keys: Vec<String> = std::iter::once(label_text.to_string())
            .chain(idxs.into_iter().take(k).map(|i| pool[i].clone()))
            .collect();
        shuffle(&mut keys, &mut rng);
        let gold = keys
            .iter()
            .position(|k| k.as_str() == label_text)
            .expect("gold label is keys[0] before the shuffle");
        let mut criteria = Map::new();
        for key in &keys {
            criteria.insert(
                key.clone(),
                Value::String(key.replace('_', " ").replace('.', ": ")),
            );
        }
        cases.push(SuiteCase {
            id: format!("massive_intent_en:{pos}"),
            state: json!({ "utterance": text }),
            questions: vec![SuiteQuestion {
                qid: "intent".to_string(),
                kind: QKind::Choice,
                instructions: "What is the user asking for in `utterance`?".to_string(),
                criteria: Value::Object(criteria),
            }],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "massive_intent_en",
        cases,
        option_counts_note: "1 gold + min(19, pool) sampled distractors; SplitMix64 option_seed \
                             drawn once per build in row order (NOT MT19937 — see SplitMix64)",
    }
}

/// §3.3 XNLI (en) — choice, 3 FIXED options in ClassLabel order
/// [entailment, neutral, contradiction] (the fetch layer verifies the
/// dataset's `features.label.names`).
#[must_use]
pub fn build_xnli_en(rows_file: &Value, max_rows: usize) -> Suite {
    const NLI_CRIT: [(&str, &str); 3] = [
        ("entailment", "the premise implies the hypothesis is true"),
        (
            "neutral",
            "the premise neither implies nor contradicts the hypothesis",
        ),
        (
            "contradiction",
            "the premise implies the hypothesis is false",
        ),
    ];
    let keys: Vec<String> = NLI_CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = NLI_CRIT
        .iter()
        .map(|(_, d)| Some((*d).to_string()))
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(premise), Some(hypothesis), Some(label)) = (
            row_str(row, "premise"),
            row_str(row, "hypothesis"),
            row_i64(row, "label"),
        ) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("xnli_en:{pos}"),
            // key order premise then hypothesis — the serialized state bytes
            // depend on it (§1.3)
            state: json!({ "premise": premise, "hypothesis": hypothesis }),
            questions: vec![choice_q(
                "relation",
                "What is the relationship between `premise` and `hypothesis`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 3],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "xnli_en",
        cases,
        option_counts_note: "3 fixed options in ClassLabel order [entailment, neutral, \
                             contradiction] — fetch layer verifies features.label.names",
    }
}

// ── Thai probe suites (Plan 003 T3.1/T3.2 — opt-in only, never a default-run
// member: the G-ISO-2 non-contamination law, `named_only: true` in the
// registry) ─────────────────────────────────────────────────────────────────

/// Plan 003 T3.2 — `thai_wisesight` (pythainlp/wisesight_sentiment, config
/// `wisesight_sentiment`): 4-class Thai sentiment choice. Fixed options in
/// ClassLabel order [pos, neu, neg, q] — the fetch layer verifies
/// `features.category.names` (the ag_news VERIFY-AT-PORT law; the verified
/// order is pinned in dataset_manifest.md). Gold = the int `category`.
#[must_use]
pub fn build_thai_wisesight(rows_file: &Value, max_rows: usize) -> Suite {
    const CRIT: [(&str, &str); 4] = [
        ("pos", "positive sentiment"),
        ("neu", "neutral statement"),
        ("neg", "negative sentiment"),
        ("q", "a question"),
    ];
    let keys: Vec<String> = CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = CRIT
        .iter()
        .map(|(_, d)| Some((*d).to_string()))
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "texts"), row_i64(row, "category")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("thai_wisesight:{pos}"),
            state: json!({ "text": text }),
            questions: vec![choice_q(
                "sentiment",
                "What sentiment does `text` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 4],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "thai_wisesight",
        cases,
        option_counts_note: "4 fixed options in ClassLabel order [pos, neu, neg, q] — \
                             fetch layer verifies features.category.names",
    }
}

/// Plan 003 T3.2 — `thai_sib200` (Davlan/sib200, config `tha_Thai`): 7-way
/// Thai topical choice. `category` arrives as a STRING (no ClassLabel), so
/// the option universe derives from the data exactly like banking77:
/// `sorted(unique(category))` over ALL rows on disk; gold = the key position.
/// The engine's domains are the same string keys (the registry's default
/// option-key-union branch), which the train docs' category strings bind to
/// directly.
#[must_use]
pub fn build_thai_sib200(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut labels: Vec<String> = Vec::new();
    for row in &all {
        if let Some(c) = row_str(row, "category")
            && !labels.iter().any(|l| l == c)
        {
            labels.push(c.to_string());
        }
    }
    labels.sort();
    let descs: Vec<Option<String>> = vec![None; labels.len()];
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(category)) = (row_str(row, "text"), row_str(row, "category")) else {
            continue;
        };
        let idx = labels.iter().position(|l| l == category).unwrap_or_else(|| {
            panic!("thai_sib200: row category {category:?} not in key set")
        });
        cases.push(SuiteCase {
            id: format!("thai_sib200:{pos}"),
            state: json!({ "text": text }),
            questions: vec![choice_q(
                "topic",
                "Which topic does `text` belong to?",
                &labels,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; labels.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "thai_sib200",
        cases,
        option_counts_note: "topic options derived from sorted unique category strings \
                             (SIB-200; the whole 204-row test split presents all 7)",
    }
}

// ── typed-decisions (§3.1) ──────────────────────────────────────────────────

/// A dataset column that arrives as a JSON string (or, tolerantly, already
/// parsed): parse the string, Null on malformed (the row is then skipped —
/// the reference would crash; the runner's count floors guard).
fn parse_json_string_or_value(v: &Value) -> Value {
    match v {
        Value::String(s) => serde_json::from_str(s).unwrap_or(Value::Null),
        other => other.clone(),
    }
}

fn qdef_kind(qdef: &Value) -> Option<QKind> {
    match qdef.get("type").and_then(Value::as_str)? {
        "choice" => Some(QKind::Choice),
        "score" => Some(QKind::Score),
        "noul" => Some(QKind::Noul),
        _ => None,
    }
}

/// §3.1 gold mapping for one question (notebook cell 12 verbatim).
/// `probs = g.get("probabilities", {})` may be ABSENT — every per-key read
/// defaults to 0.0 (or the documented noul fallbacks).
fn gold_answer(kind: QKind, qdef: &Value, g: &Value) -> Option<GoldAnswer> {
    let probs = g.get("probabilities").and_then(Value::as_object);
    let prob_of = |key: &str| -> f64 {
        probs
            .and_then(|p| p.get(key))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    match kind {
        QKind::Choice => {
            let keys = choice_keys(qdef.get("criteria")?)?;
            let label = py_str(g.get("label")?);
            // position of str(gold.label) in the criteria KEY order
            let idx = keys.iter().position(|k| *k == label)?;
            let soft = keys.iter().map(|k| prob_of(k)).collect();
            Some(GoldAnswer {
                idx,
                soft,
                gold_score: None,
            })
        }
        QKind::Noul => {
            let is_true = py_str(g.get("label")?).to_lowercase() == "true";
            // p_true = probabilities["true"] else gold.noul else 0.5
            let p_true = probs
                .and_then(|p| p.get("true"))
                .and_then(Value::as_f64)
                .or_else(|| g.get("noul").and_then(Value::as_f64))
                .unwrap_or(0.5);
            Some(GoldAnswer {
                idx: usize::from(is_true),
                soft: vec![1.0 - p_true, p_true],
                gold_score: None,
            })
        }
        QKind::Score => {
            let n = qdef.get("criteria")?.as_array()?.len();
            let label = value_as_i64(g.get("label")?)?;
            let idx = usize::try_from(label).ok()?;
            let soft: Vec<f64> = (0..n).map(|i| prob_of(&i.to_string())).collect();
            // gold_score = float(gold.score) else float(label)
            let gold_score = g
                .get("score")
                .and_then(value_as_f64)
                .or_else(|| g.get("label").and_then(value_as_f64));
            Some(GoldAnswer {
                idx,
                soft,
                gold_score,
            })
        }
    }
}

/// §3.1 typed-decisions — ALL rows carry the case's question set; questions
/// iterate in the parsed dict's INSERTION order (`preserve_order`), and
/// questions/gold are built pairwise so the parallel vectors stay aligned.
/// Per-row divergences from the Python (documented): a malformed
/// `questions`/`gold` string skips the row; a question with no gold entry, a
/// non-string instruction, or an unresolvable gold mapping skips that
/// question only.
#[must_use]
pub fn build_typed_decisions(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let Some(workflow) = row_str(row, "workflow") else {
            continue;
        };
        // state: JSON string → parse; parse failure → the raw string AS the
        // state (spec `try: json.loads except: pass`)
        let state = match row.get("state") {
            Some(Value::String(s)) => {
                serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.clone()))
            }
            Some(other) => other.clone(),
            None => continue,
        };
        let questions_v = row.get("questions").map(parse_json_string_or_value);
        let gold_v = row.get("gold").map(parse_json_string_or_value);
        let (Some(qmap), Some(gmap)) = (
            questions_v.as_ref().and_then(Value::as_object),
            gold_v.as_ref().and_then(Value::as_object),
        ) else {
            continue;
        };

        let mut questions = Vec::new();
        let mut gold = Vec::new();
        for (qid, qdef) in qmap {
            let Some(g) = gmap.get(qid) else {
                continue;
            };
            let Some(kind) = qdef_kind(qdef) else {
                continue;
            };
            let Some(instructions) = qdef.get("instructions").and_then(Value::as_str) else {
                continue;
            };
            let Some(answer) = gold_answer(kind, qdef, g) else {
                continue;
            };
            // criteria: dict for choice (normalize a list per §1.2), array
            // for score, Null for noul (its descriptions are dropped in this
            // slice — the lane layer renders the fixed [false, true] text)
            let criteria = match (kind, qdef.get("criteria")) {
                (QKind::Noul, _) => Value::Null,
                (QKind::Choice, Some(Value::Array(a))) => {
                    let mut m = Map::new();
                    for v in a {
                        m.insert(py_str(v), Value::Null);
                    }
                    Value::Object(m)
                }
                (_, Some(c)) => c.clone(),
                (_, None) => Value::Null,
            };
            questions.push(SuiteQuestion {
                qid: qid.clone(),
                kind,
                instructions: instructions.to_string(),
                criteria,
            });
            gold.push(answer);
        }
        if questions.is_empty() {
            continue;
        }
        cases.push(SuiteCase {
            id: format!("{workflow}:{pos}"),
            state,
            questions,
            gold,
        });
    }
    Suite {
        name: "typed_decisions",
        cases,
        option_counts_note: "variable per row (choice/score/noul mixed); question order = \
                             parsed insertion order; choice criteria object order IS the \
                             label order",
    }
}

/// The option-conditioned fit events (issue 038 T7b): for every scorable
/// train question, `(question id, gold option key, raw state string)` —
/// the same parse rules as [`build_typed_decisions`] (per-question skips;
/// the row-level workflow filter is deliberately NOT applied: a train row
/// that never becomes a case is still train data for the tables, which is
/// exactly what the probe validated). The option spelling is the engine's
/// own: choice keys verbatim, score levels via the same `Value → String`
/// rule [`engine_request`](super::runner::engine_request) uses, noul as
/// the internal `[yes, no]` candidates (gold `true` → `yes`).
#[cfg(feature = "option_cond")]
#[must_use]
pub fn typed_gold_events(rows_file: &Value) -> Vec<(String, String, String)> {
    let all = rows_of(rows_file);
    let mut out = Vec::new();
    for row in &all {
        let Some(state_raw) = row_str(row, "state") else {
            continue;
        };
        let questions_v = row.get("questions").map(parse_json_string_or_value);
        let gold_v = row.get("gold").map(parse_json_string_or_value);
        let (Some(qmap), Some(gmap)) = (
            questions_v.as_ref().and_then(Value::as_object),
            gold_v.as_ref().and_then(Value::as_object),
        ) else {
            continue;
        };
        for (qid, qdef) in qmap {
            let Some(g) = gmap.get(qid) else {
                continue;
            };
            let Some(kind) = qdef_kind(qdef) else {
                continue;
            };
            let Some(answer) = gold_answer(kind, qdef, g) else {
                continue;
            };
            let option = match kind {
                QKind::Noul => {
                    if answer.idx == 1 {
                        "yes"
                    } else {
                        "no"
                    }
                    .to_string()
                }
                QKind::Choice => {
                    let Some(crit) = qdef.get("criteria") else {
                        continue;
                    };
                    let Some(keys) = choice_keys(crit) else {
                        continue;
                    };
                    match keys.get(answer.idx) {
                        Some(k) => k.clone(),
                        None => continue,
                    }
                }
                QKind::Score => {
                    let Some(levels) = qdef.get("criteria").and_then(Value::as_array) else {
                        continue;
                    };
                    match levels.get(answer.idx) {
                        Some(Value::String(s)) => s.clone(),
                        Some(other) => other.to_string(),
                        None => continue,
                    }
                }
            };
            out.push((qid.clone(), option, state_raw.to_string()));
        }
    }
    out
}

// ── corpus helpers (modelless lane train rows) ──────────────────────────────

/// One train row for the modelless lane's corpus.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrainDoc {
    pub label: String,
    pub text: String,
}

/// The per-row LABEL rule of [`train_docs`] — one rule, two consumers: the
/// corpus builder and the cap-selection stratification (which must label
/// rows exactly as the corpora would, or the round-robin buckets the wrong
/// rows).
#[must_use]
pub fn train_row_label(suite: &str, row: &Value) -> Option<String> {
    match suite {
        "massive_intent_en" => row_str(row, "label_text").map(str::to_string),
        // Plan 003 Thai probe suites: the label column is `category` on
        // both — an int ClassLabel on wisesight, a plain string on sib200.
        "thai_wisesight" => row_i64(row, "category").map(|l| l.to_string()),
        "thai_sib200" => row_str(row, "category").map(str::to_string),
        "typed_decisions" => row_str(row, "workflow").map(str::to_string),
        // Plan 010 S1MB: the corpus label = the gold KEY string at the
        // row's gold position (choice: the option key; score: the level-
        // index string; noul: "false"/"true") — the same spelling the
        // option-key union (choice/score) and the noul-exempt branch read.
        "s1mb_choice" | "s1mb_noul" | "s1mb_score" => {
            let (Some(keys), Some(gold_idx)) = (
                row.get("question").and_then(|q| q.get("keys")).and_then(Value::as_array),
                row.get("question").and_then(|q| q.get("gold_idx")).and_then(Value::as_u64),
            ) else {
                return None;
            };
            keys.get(gold_idx as usize)
                .and_then(Value::as_str)
                .map(str::to_string)
        }
        // mteb/banking77 carries label_text (the option-key source — the
        // STRIPPED form, so the corpora bind to the engine's option-key
        // domains); int label as the fallback for a PolyAI-shaped row.
        "banking77" => match row_str(row, "label_text") {
            Some(t) => Some(t.replace('_', " ")),
            None => row_i64(row, "label").map(|l| l.to_string()),
        },
        _ => row_i64(row, "label").map(|l| l.to_string()),
    }
}

/// The label-stratified cap-selection slices (Issue 013 lever-1 protocol
/// plumb). The registry cal slice — the FIRST `cal_cap` train rows — is
/// label-clustered by construction: the mirrors store rows label-grouped,
/// so the first 200 rows span 2/4 labels for ag_news and 2/32 for
/// banking77 (measured 2026-09-24), and cal ACCURACY on it reads
/// chance-level, which cannot rank corpus caps (the first selection run
/// picked cap 8 for ag_news — the sweep's worst test reading). The
/// selection front instead round-robins one row per label per round over
/// the rows BEYOND the cal prefix (`pool_from..`), in the engine's label
/// order, up to `budget` rows — deterministic (fixed order, no RNG).
///
/// Returns the front envelope (the selection cases' source) plus the whole
/// train envelope with the front FIRST: the builders derive their option
/// universe from ALL rows on disk, so passing the permuted envelope keeps
/// the selection questions' option set identical to the registry cal
/// slice's. The rows in the front are members of the corpus pool region,
/// so the caller must exclude their texts from the selection corpora
/// (content-exclusion) — a selection case must never score against its
/// own text.
pub struct SelectionSlices {
    /// The stratified front envelope (≤ `budget` rows).
    pub front: Value,
    /// front rows first, then every other train row in original order.
    pub permuted: Value,
    /// Rows in the front (= the `max_rows` the builders must see).
    pub n_picks: usize,
}

#[must_use]
pub fn stratified_selection_slices(
    train_rows_file: &Value,
    suite: &str,
    labels: &[String],
    pool_from: usize,
    budget: usize,
) -> SelectionSlices {
    let rows = rows_of(train_rows_file);
    // Per-label row indices in pool order. Rows whose label falls outside
    // the engine's label universe (or fails the label rule) never pick —
    // the corpora cannot contain them either.
    let mut by_label: Vec<Vec<usize>> = vec![Vec::new(); labels.len()];
    for (ri, row) in rows.iter().enumerate().skip(pool_from) {
        if let Some(l) = train_row_label(suite, row)
            && let Some(li) = labels.iter().position(|x| *x == l)
        {
            by_label[li].push(ri);
        }
    }
    let picks = round_robin_picks(&by_label, budget);
    let wrap = |idxs: &[usize]| wrap_rows(&rows, idxs);
    let mut rest: Vec<usize> = (0..rows.len()).filter(|ri| !picks.contains(ri)).collect();
    let mut permuted_idx = picks.clone();
    permuted_idx.append(&mut rest);
    SelectionSlices {
        front: wrap(&picks),
        permuted: wrap(&permuted_idx),
        n_picks: picks.len(),
    }
}

/// The round-robin label-cyclic index picks over per-label index lists —
/// the stratified-front core shared by the Issue-013 selection slices and
/// the Issue-039 test/cal splits. Deterministic by construction: fixed
/// label order, dataset order within each label, no RNG.
fn round_robin_picks(by_label: &[Vec<usize>], budget: usize) -> Vec<usize> {
    let mut cursors = vec![0usize; by_label.len()];
    let mut picks: Vec<usize> = Vec::with_capacity(budget);
    if budget > 0 {
        loop {
            let mut took_any = false;
            let mut exhausted = true;
            for (li, idxs) in by_label.iter().enumerate() {
                if cursors[li] < idxs.len() {
                    exhausted = false;
                    picks.push(idxs[cursors[li]]);
                    cursors[li] += 1;
                    took_any = true;
                    if picks.len() >= budget {
                        break;
                    }
                }
            }
            if picks.len() >= budget || !took_any || exhausted {
                break;
            }
        }
    }
    picks
}

/// Envelope re-wrap of picked row indices (row_idx re-enumerated 0..n —
/// the builders read `.rows[i].row`, never the original index).
fn wrap_rows(rows: &[&Value], idxs: &[usize]) -> Value {
    let wrapped: Vec<Value> = idxs
        .iter()
        .enumerate()
        .map(|(i, &ri)| serde_json::json!({ "row_idx": i, "row": rows[ri] }))
        .collect();
    serde_json::json!({ "rows": wrapped })
}

/// A label-stratified split of a row envelope (Issue 039 T2).
#[derive(Debug, Clone, PartialEq)]
pub struct StratSplit {
    /// The stratified front envelope (≤ `budget` rows, round-robin over
    /// the label universe; the identity envelope when `budget` is 0 or
    /// covers every row).
    pub front: Value,
    /// The complement, in original envelope order (the Issue-039 corpus
    /// pool's source: cal rows excluded by CONSTRUCTION, not position).
    pub rest: Value,
    /// Rows in the front (= the case count the builders produce).
    pub n_picks: usize,
}

/// Label-stratified split of a row envelope: `front` = round-robin one row
/// per label per round over the WHOLE split (first-appearance label order,
/// dataset order within each label), up to `budget` rows; `rest` = every
/// row not picked, in original order.
///
/// Why: the first-N test sample is a label PREFIX on label-sorted mirrors —
/// banking77's 500 first rows spanned 13 of 77 labels, massive's 300 spanned
/// 30 of 60 (measured, Issue 039) — so every lane scored a non-representative
/// slice and the questions' remaining options carried only their self-doc
/// fallback corpus. The round-robin front spans every label proportionally.
/// Deterministic by construction (no RNG, no seed to record).
///
/// Identity law: `budget == 0` (= all rows) or `budget >= rows.len()`
/// returns the WHOLE envelope in original order — byte-identical to the
/// first-N law it replaces, so uncapped suites never move.
#[must_use]
pub fn stratified_split(rows_file: &Value, suite: &str, budget: usize) -> StratSplit {
    let rows = rows_of(rows_file);
    if budget == 0 || rows.len() <= budget {
        let all: Vec<usize> = (0..rows.len()).collect();
        return StratSplit {
            n_picks: rows.len(),
            front: wrap_rows(&rows, &all),
            rest: serde_json::json!({ "rows": [] }),
        };
    }
    // Bucket by the row's own label, first appearance = label order. Rows
    // failing the label rule never build cases (the builders skip them), so
    // they stay in `rest` and never spend front budget.
    let mut label_order: Vec<String> = Vec::new();
    let mut label_idx: HashMap<String, usize> = HashMap::new();
    let mut by_label: Vec<Vec<usize>> = Vec::new();
    for (ri, row) in rows.iter().enumerate() {
        if let Some(l) = train_row_label(suite, row) {
            let li = match label_idx.get(&l) {
                Some(&li) => li,
                None => {
                    label_order.push(l.clone());
                    by_label.push(Vec::new());
                    label_idx.insert(l, by_label.len() - 1);
                    by_label.len() - 1
                }
            };
            by_label[li].push(ri);
        }
    }
    let picks = round_robin_picks(&by_label, budget);
    if picks.is_empty() {
        // No labeled rows at all: the round-robin has nothing to stratify —
        // the identity split (never produce an empty front over rows that
        // exist; the builders' own count floors govern what they skip).
        let all: Vec<usize> = (0..rows.len()).collect();
        return StratSplit {
            n_picks: rows.len(),
            front: wrap_rows(&rows, &all),
            rest: serde_json::json!({ "rows": [] }),
        };
    }
    let rest: Vec<usize> = (0..rows.len()).filter(|ri| !picks.contains(ri)).collect();
    StratSplit {
        n_picks: picks.len(),
        front: wrap_rows(&rows, &picks),
        rest: wrap_rows(&rows, &rest),
    }
}

/// Train rows per suite. Label/text rules:
/// - `ag_news` / `emotion` / `sst5` / `banking77` / `prompt_injections` /
///   `thai_sib200`:
///   label = the per-suite [`train_row_label`] rule, text = the `text` field;
/// - `thai_wisesight`: label = `int(category).to_string()`, text = the
///   `texts` field (the dataset's own column spelling);
/// - `massive_intent_en`: label = `label_text`, text = `text`;
/// - `xnli_en`: label = `int(label).to_string()`, text = premise + "\n" +
///   hypothesis;
/// - `typed_decisions`: label = `workflow`, text = the raw state string AS
///   STORED (no reparse — the compression scorer works on the stored bytes).
///
/// An unknown suite name → empty vec (the caller decides whether that is an
/// error). Keyed on the `Suite::name` spellings above.
#[must_use]
pub fn train_docs(train_rows_file: &Value, suite: &str) -> Vec<TrainDoc> {
    let rows = rows_of(train_rows_file);
    match suite {
        "ag_news" | "emotion" | "sst5" | "prompt_injections" | "thai_sib200" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        // wisesight's text column is `texts`, not `text` — its own arm.
        "thai_wisesight" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "texts")?.to_string(),
                })
            })
            .collect(),
        "banking77" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        "massive_intent_en" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        "xnli_en" | "xnli_en_val" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: format!("{}\n{}", row_str(r, "premise")?, row_str(r, "hypothesis")?),
                })
            })
            .collect(),
        "typed_decisions" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "state")?.to_string(),
                })
            })
            .collect(),
        // Plan 010 S1MB: the same shape as typed_decisions — the corpus
        // text is the RAW state_json string, identical to what
        // serialize_state returns for the built case (zero drift).
        "s1mb_choice" | "s1mb_noul" | "s1mb_score" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "state")?.to_string(),
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Plan 010 — the S1MB lane (our-lanes scope, riir-train Issue 607): the
/// GENERIC builder over `scripts/s1mb_fetch_convert.py`'s converter rows.
/// One row = ONE judgment (case × decision, flattened at the converter) —
/// the universal single-question shape every lane consumes unchanged. The
/// state rides as the RAW state_json string (the typed_decisions row shape):
/// serialize_state passes it through verbatim, so corpus text == query state
/// text with zero re-serialization drift. Gold was mapped BY ID at the
/// converter (criteria order and target-ids order are independent in S1MB);
/// the builder trusts the row's gold_idx — the converter's zero-skip run is
/// the contract (26,269/26,269 mapped).
fn build_s1mb_kind(rows_file: &Value, max_rows: usize, name: &'static str, kind_want: &str) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let Some(state) = row.get("state").and_then(Value::as_str) else {
            continue;
        };
        let Some(q) = row.get("question") else { continue };
        let Some(qid) = q.get("qid").and_then(Value::as_str) else { continue };
        let Some(kind) = q.get("kind").and_then(Value::as_str) else { continue };
        if kind != kind_want {
            eprintln!("  [{name}] row {pos}: kind {kind:?} != {kind_want:?} — skipped");
            continue;
        }
        let Some(instructions) = q.get("instructions").and_then(Value::as_str) else {
            continue;
        };
        let Some(gold_idx) = q.get("gold_idx").and_then(Value::as_u64) else { continue };
        let Some(case_id) = row.get("case_id").and_then(Value::as_str) else { continue };
        let kind = match kind {
            "choice" => QKind::Choice,
            "score" => QKind::Score,
            _ => QKind::Noul,
        };
        let soft: Vec<f64> = q
            .get("soft")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
            .unwrap_or_default();
        let gold_score = q.get("gold_score").and_then(Value::as_f64);
        let criteria = q.get("criteria").cloned().unwrap_or(Value::Null);
        cases.push(SuiteCase {
            id: format!("{case_id}:{qid}"),
            state: Value::String(state.to_string()),
            questions: vec![SuiteQuestion {
                qid: qid.to_string(),
                kind,
                instructions: instructions.to_string(),
                criteria,
            }],
            gold: vec![GoldAnswer {
                idx: gold_idx as usize,
                soft,
                gold_score,
            }],
        });
    }
    Suite {
        name,
        cases,
        option_counts_note: "S1MB: variable per-question candidate sets (106 subsets, flattened one judgment per case); gold mapped by id at the converter",
    }
}

/// The three registry rows' entry points (Suite.name is &'static — the
/// kind dispatch rides the wrappers, one per converter suite dir).
#[must_use]
pub fn build_s1mb_choice(rows_file: &Value, max_rows: usize) -> Suite {
    build_s1mb_kind(rows_file, max_rows, "s1mb_choice", "choice")
}

#[must_use]
pub fn build_s1mb_noul(rows_file: &Value, max_rows: usize) -> Suite {
    build_s1mb_kind(rows_file, max_rows, "s1mb_noul", "noul")
}

#[must_use]
pub fn build_s1mb_score(rows_file: &Value, max_rows: usize) -> Suite {
    build_s1mb_kind(rows_file, max_rows, "s1mb_score", "score")
}

#[cfg(test)]
mod selection_slice_tests {
    //! Issue 013 lever-1 protocol plumb — the stratified selection front.

    use super::stratified_selection_slices;

    /// ag_news-shaped envelope: rows is a bare array of `{label, text}`.
    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        let wrapped: Vec<serde_json::Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| serde_json::json!({ "row_idx": i, "row": r }))
            .collect();
        serde_json::json!({ "rows": wrapped })
    }

    fn grouped_rows(labels: usize, per_label: usize) -> Vec<serde_json::Value> {
        let mut rows = Vec::new();
        for label in 0..labels {
            for _ in 0..per_label {
                rows.push(serde_json::json!({
                    "label": label,
                    "text": format!("t{label}_{})", rows.len()),
                }));
            }
        }
        rows
    }

    fn front_labels(s: &super::SelectionSlices) -> Vec<String> {
        s.front
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .unwrap()
            .iter()
            .map(|r| {
                r.get("row")
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_i64()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn round_robin_interleaves_labels_in_order() {
        let env = envelope(grouped_rows(3, 4));
        let labels: Vec<String> = (0..3).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "ag_news", &labels, 0, 6);
        assert_eq!(s.n_picks, 6);
        assert_eq!(front_labels(&s), ["0", "1", "2", "0", "1", "2"]);
    }

    #[test]
    fn pool_from_skips_the_cal_prefix() {
        // rows 0..4 are the (label-clustered) cal prefix: label 0's WHOLE
        // block — the banking77 situation (the prefix holds labels the
        // pool region never sees again).
        let env = envelope(grouped_rows(3, 4));
        let labels: Vec<String> = (0..3).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "ag_news", &labels, 4, 9);
        // Budget 9 is unfillable: the pool region carries only 8 rows
        // (labels 1 and 2, 4 each) — the front takes all 8 and never a
        // prefix row.
        assert_eq!(s.n_picks, 8);
        let seq = front_labels(&s);
        assert!(!seq.iter().any(|l| l == "0"));
        assert_eq!(seq.iter().filter(|l| *l == "1").count(), 4);
        assert_eq!(seq.iter().filter(|l| *l == "2").count(), 4);
    }

    #[test]
    fn rows_outside_the_label_universe_never_pick() {
        let mut rows = grouped_rows(2, 2);
        rows.push(serde_json::json!({ "label": 9, "text": "t9_0" }));
        let env = envelope(rows);
        let labels: Vec<String> = (0..2).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "ag_news", &labels, 0, 10);
        assert_eq!(s.n_picks, 4); // the label-9 row is unpickable
        assert!(!front_labels(&s).iter().any(|l| l == "9"));
    }

    #[test]
    fn permuted_envelope_preserves_every_row() {
        let env = envelope(grouped_rows(3, 4));
        let labels: Vec<String> = (0..3).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "ag_news", &labels, 0, 5);
        let permuted = s.permuted.get("rows").unwrap().as_array().unwrap();
        assert_eq!(permuted.len(), 12); // every row survives the permutation
        // The front IS the permuted envelope's head.
        let head: Vec<&serde_json::Value> = permuted[..s.n_picks]
            .iter()
            .map(|r| r.get("row").unwrap())
            .collect();
        let front: Vec<&serde_json::Value> = s
            .front
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.get("row").unwrap())
            .collect();
        assert_eq!(head, front);
    }
}

#[cfg(test)]
mod stratified_split_tests {
    //! Issue 039 T2 — the label-stratified test/cal split.

    use super::stratified_split;

    /// ag_news-shaped envelope: rows is a bare array of `{label, text}`.
    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        let wrapped: Vec<serde_json::Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| serde_json::json!({ "row_idx": i, "row": r }))
            .collect();
        serde_json::json!({ "rows": wrapped })
    }

    /// Label-GROUPED rows (the mirror shape): `per` rows of each of
    /// `labels` labels, label blocks in first-appearance order.
    fn grouped_rows(labels: usize, per: usize) -> Vec<serde_json::Value> {
        (0..labels)
            .flat_map(|l| (0..per).map(move |i| serde_json::json!({ "label": l, "text": format!("t{l}_{i}") })))
            .collect()
    }

    fn front_texts(s: &super::StratSplit) -> Vec<String> {
        s.front
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.get("row").unwrap().get("text").unwrap().as_str().unwrap().to_string())
            .collect()
    }

    fn rest_texts(s: &super::StratSplit) -> Vec<String> {
        s.rest
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.get("row").unwrap().get("text").unwrap().as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn front_spans_every_label_round_robin() {
        // 3 label blocks × 4 rows, budget 6 → 2 rows per label, cyclic.
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 6);
        assert_eq!(s.n_picks, 6);
        assert_eq!(front_texts(&s), ["t0_0", "t1_0", "t2_0", "t0_1", "t1_1", "t2_1"]);
    }

    #[test]
    fn front_and_rest_partition_the_envelope_in_order() {
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 6);
        let mut all = front_texts(&s);
        let mut rest = rest_texts(&s);
        all.append(&mut rest);
        all.sort();
        let mut want: Vec<String> = (0..3)
            .flat_map(|l| (0..4).map(move |i| format!("t{l}_{i}")))
            .collect();
        want.sort();
        assert_eq!(all, want);
        // rest preserves original (label-grouped) order.
        assert_eq!(rest_texts(&s), ["t0_2", "t0_3", "t1_2", "t1_3", "t2_2", "t2_3"]);
    }

    #[test]
    fn budget_zero_is_the_identity_split() {
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 0);
        assert_eq!(s.n_picks, 12);
        assert_eq!(front_texts(&s).len(), 12);
        assert_eq!(rest_texts(&s).len(), 0);
    }

    #[test]
    fn budget_ge_rows_is_the_identity_split() {
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 12);
        assert_eq!(s.n_picks, 12);
        // original order, not round-robin — the first-N law preserved.
        assert_eq!(front_texts(&s)[0], "t0_0");
        assert_eq!(front_texts(&s)[1], "t0_1");
    }

    #[test]
    fn unbalanced_labels_get_proportional_picks() {
        // banking77 shape: many labels, budget spanning all with a remainder
        // — every label picks at least once, no label more than one extra.
        let s = stratified_split(&envelope(grouped_rows(7, 2)), "ag_news", 10);
        assert_eq!(s.n_picks, 10);
        let mut per_label: std::collections::HashMap<char, usize> =
            std::collections::HashMap::new();
        for t in front_texts(&s) {
            *per_label.entry(t.chars().nth(1).unwrap()).or_insert(0) += 1;
        }
        assert_eq!(per_label.len(), 7);
        assert!(per_label.values().all(|&n| (1..=2).contains(&n)));
    }

    #[test]
    fn unlabeled_rows_never_pick_and_stay_in_rest() {
        let mut rows = grouped_rows(2, 2);
        rows.push(serde_json::json!({ "text": "no_label" }));
        let s = stratified_split(&envelope(rows), "ag_news", 2);
        assert_eq!(s.n_picks, 2);
        assert!(!front_texts(&s).iter().any(|t| t == "no_label"));
        assert!(rest_texts(&s).contains(&"no_label".to_string()));
    }

    #[test]
    fn massive_label_rule_buckets_by_label_text() {
        // label_text rows (massive/label_text rule), first-appearance order.
        let rows = vec![
            serde_json::json!({ "label_text": "audio", "text": "a0" }),
            serde_json::json!({ "label_text": "play", "text": "p0" }),
            serde_json::json!({ "label_text": "audio", "text": "a1" }),
            serde_json::json!({ "label_text": "play", "text": "p1" }),
        ];
        let s = stratified_split(&envelope(rows), "massive_intent_en", 2);
        assert_eq!(front_texts(&s), ["a0", "p0"]);
    }
}

#[cfg(test)]
mod thai_builder_tests {
    //! Plan 003 T3.2 — the Thai probe-suite builders. Plan 010: also hosts
    //! the S1MB generic-builder tests (same module shape: builder + row
    //! envelope fixtures).

    use super::{QKind, build_s1mb_choice, build_s1mb_noul, build_s1mb_score};
    use super::{build_thai_sib200, build_thai_wisesight, train_docs};

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        let wrapped: Vec<serde_json::Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| serde_json::json!({ "row_idx": i, "row": r }))
            .collect();
        serde_json::json!({ "rows": wrapped })
    }

    /// The FETCHED ClassLabel order (dataset_manifest.md — verified at
    /// fetch 2026-09-28): the builder's fixed criteria MUST match it.
    #[test]
    fn wisesight_fixed_options_match_the_classlabel_order() {
        let env = envelope(vec![serde_json::json!({ "texts": "สวัสดี", "category": 0 })]);
        let s = build_thai_wisesight(&env, 0);
        let crit = s.cases[0].questions[0].criteria.as_object().unwrap();
        let keys: Vec<&String> = crit.keys().collect();
        assert_eq!(keys, ["pos", "neu", "neg", "q"]);
    }

    #[test]
    fn wisesight_gold_is_the_int_category() {
        let env = envelope(vec![
            serde_json::json!({ "texts": "ดีมาก", "category": 0 }),
            serde_json::json!({ "texts": "เฉยๆ", "category": 1 }),
            serde_json::json!({ "texts": "แย่", "category": 2 }),
            serde_json::json!({ "texts": "ช่วยหน่อย", "category": 3 }),
        ]);
        let s = build_thai_wisesight(&env, 0);
        assert_eq!(s.cases.len(), 4);
        assert_eq!(s.cases[0].state["text"], "ดีมาก");
        assert_eq!(s.cases[3].gold[0].idx, 3);
        // state carries the `texts` column under the standard `text` key.
        assert_eq!(s.cases[0].id, "thai_wisesight:0");
    }

    #[test]
    fn wisesight_train_docs_read_the_texts_column() {
        let env = envelope(vec![
            serde_json::json!({ "texts": "a", "category": 1 }),
            serde_json::json!({ "texts": "b", "category": 2 }),
        ]);
        let docs = train_docs(&env, "thai_wisesight");
        assert_eq!(docs.len(), 2);
        assert_eq!((docs[0].label.as_str(), docs[0].text.as_str()), ("1", "a"));
        assert_eq!((docs[1].label.as_str(), docs[1].text.as_str()), ("2", "b"));
    }

    #[test]
    fn sib200_option_universe_is_sorted_unique_categories() {
        // Rows out of sorted order on purpose: the universe must come out
        // sorted regardless of first-appearance order.
        let env = envelope(vec![
            serde_json::json!({ "index_id": 1, "category": "travel", "text": "t1" }),
            serde_json::json!({ "index_id": 2, "category": "health", "text": "t2" }),
            serde_json::json!({ "index_id": 3, "category": "travel", "text": "t3" }),
        ]);
        let s = build_thai_sib200(&env, 0);
        let crit = s.cases[0].questions[0].criteria.as_object().unwrap();
        let keys: Vec<&String> = crit.keys().collect();
        assert_eq!(keys, ["health", "travel"]);
        // gold = the key position of the row's own category.
        assert_eq!(s.cases[0].gold[0].idx, 1); // travel
        assert_eq!(s.cases[1].gold[0].idx, 0); // health
        assert_eq!(s.cases[0].id, "thai_sib200:0");
    }

    #[test]
    fn sib200_train_docs_carry_the_category_string() {
        let env = envelope(vec![
            serde_json::json!({ "index_id": 1, "category": "politics", "text": "x" }),
            serde_json::json!({ "index_id": 2, "category": "sports", "text": "y" }),
        ]);
        let docs = train_docs(&env, "thai_sib200");
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].label, "politics");
        assert_eq!(docs[1].label, "sports");
    }

    // ── S1MB (Plan 010): the generic converter-row builder ─────────────

    fn s1mb_row(subset: &str, q: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "subset": subset,
            "group": "domain",
            "case_id": format!("{subset}:hash"),
            "state": "{\"prompt\": \"x\"}",
            "question": q,
        })
    }

    #[test]
    fn s1mb_choice_builder_maps_gold_by_presented_position() {
        let env = envelope(vec![s1mb_row(
            "arc",
            serde_json::json!({
                "qid": "answer", "kind": "choice", "instructions": "pick",
                "criteria": {"B": "desc-b", "C": null, "A": null, "D": null},
                "keys": ["B", "C", "A", "D"],
                "gold_idx": 2, "gold_score": null,
                "soft": [0.0, 0.0, 1.0, 0.0],
            }),
        )]);
        let s = build_s1mb_choice(&env, 0);
        assert_eq!(s.name, "s1mb_choice");
        assert_eq!(s.cases.len(), 1);
        let c = &s.cases[0];
        assert_eq!(c.id, "arc:hash:answer");
        // The state rides verbatim (Value::String) — serialize_state passthrough.
        assert_eq!(c.state.as_str(), Some("{\"prompt\": \"x\"}"));
        let q = &c.questions[0];
        assert_eq!(q.kind, QKind::Choice);
        let keys: Vec<_> = q.criteria.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["B", "C", "A", "D"]);
        assert_eq!(c.gold[0].idx, 2);
        assert_eq!(c.gold[0].soft, vec![0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn s1mb_score_builder_carries_numeric_levels_and_gold_score() {
        let env = envelope(vec![s1mb_row(
            "s1mb-generalization-diverse-score",
            serde_json::json!({
                "qid": "rate", "kind": "score", "instructions": "rate",
                "criteria": [0.0, 0.3333333333333333, 0.6666666666666666, 1.0],
                "keys": ["0", "1", "2", "3"],
                "gold_idx": 2, "gold_score": 0.6666666666666666,
                "soft": [0.0, 0.0, 1.0, 0.0],
            }),
        )]);
        let s = build_s1mb_score(&env, 0);
        let q = &s.cases[0].questions[0];
        assert_eq!(q.kind, QKind::Score);
        assert_eq!(q.criteria.as_array().unwrap().len(), 4);
        assert_eq!(s.cases[0].gold[0].gold_score, Some(0.6666666666666666));
        assert_eq!(s.cases[0].gold[0].idx, 2);
    }

    #[test]
    fn s1mb_noul_builder_maps_false_true_and_the_label_arm_keys_the_corpus() {
        let env = envelope(vec![
            s1mb_row(
                "babi_nli",
                serde_json::json!({
                    "qid": "support", "kind": "noul", "instructions": "Does it support?",
                    "criteria": null, "keys": ["false", "true"],
                    "gold_idx": 1, "gold_score": null, "soft": [0.0, 1.0],
                }),
            ),
            s1mb_row(
                "babi_nli",
                serde_json::json!({
                    "qid": "support", "kind": "noul", "instructions": "Does it support?",
                    "criteria": null, "keys": ["false", "true"],
                    "gold_idx": 0, "gold_score": null, "soft": [1.0, 0.0],
                }),
            ),
        ]);
        let s = build_s1mb_noul(&env, 0);
        assert_eq!(s.cases[0].questions[0].kind, QKind::Noul);
        assert!(s.cases[0].questions[0].criteria.is_null());
        assert_eq!(s.cases[0].gold[0].idx, 1);
        assert_eq!(s.cases[1].gold[0].idx, 0);
        // The corpus label = the gold key string ("true"/"false") — the
        // noul-exempt branch rides train-doc labels, so the arm must speak
        // the same spelling the gold positions do.
        let docs = train_docs(&env, "s1mb_noul");
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].label, "true");
        assert_eq!(docs[1].label, "false");
        // The choice corpus label = the gold KEY string (the option-key
        // union branch — default — must meet it).
        let env2 = envelope(vec![s1mb_row(
            "arc",
            serde_json::json!({
                "qid": "answer", "kind": "choice", "instructions": "pick",
                "criteria": {"B": "d", "C": null, "A": null, "D": null},
                "keys": ["B", "C", "A", "D"],
                "gold_idx": 0, "gold_score": null, "soft": [1.0, 0.0, 0.0, 0.0],
            }),
        )]);
        let docs2 = train_docs(&env2, "s1mb_choice");
        assert_eq!(docs2[0].label, "B");
        assert_eq!(docs2[0].text, "{\"prompt\": \"x\"}");
    }
}
