//! The fitted game head over HTTP — the arena's modelless lane plays.
//!
//! Plan 607's corpus-fitted heads (katgpt-rs Benches 878/881/882) read a
//! closed-grammar game sentence and answer the pinned yes/no question with
//! the fitted P(clean). This module serves the **decoded Tetris head** —
//! Bench 881's decoded arm, the measured-best tetris reader (λ=1,
//! in-corpus 44/120, LOO 44/120 vs the structured 36/120) — so the
//! out-of-the-box engine plays the arena's Tetris board instead of
//! abstaining into a random spot.
//!
//! Why Tetris only (per-game analysis, `.plans/001_game_head_serving.md`):
//! the spot features are fully self-contained (five fill ordinals decoded
//! from ONE sentence), so serving needs nothing but the sentence the wire
//! already carries. The lanes head reads the OTHER lanes (feature columns
//! 6–7) and the flappy head needs the v3 render (the site still renders
//! v2) — both stay on the honest abstain until their unblock paths land
//! (issue 011).
//!
//! Provenance and drift discipline:
//! - the fixture is a verbatim copy of katgpt-rs `tests/fixtures/`
//!   (the laya oracle over the T0b state dumps), BLAKE3-pinned below and
//!   asserted in `tests/game_heads_serve.rs`;
//! - the head is FITTED AT BOOT from that fixture by the published recipe
//!   (standardize → λ by state-level LOO MSE over the pinned grid → final
//!   fit) — no weight artifact exists to drift, and the test pins the
//!   fit's bit-determinism plus the published agreement anchors;
//! - grammar-invalid sentences, foreign questions and unknown inputs are
//!   LOUD refusals here: they return `None` and the request falls through
//!   to the cosine engine, which abstains off-corpus as before.
//!
//! This is boot + edge surface (one parse, one fit, one decode per
//! request) — the engine core's G4 alloc-free window is untouched.

use katgpt_core::decision_wire::{
    Answer, Calibration, DecisionRequest, DecisionResponse, Lane, QuestionKind, Routing,
};
use katgpt_core::state_option_scoring::head::{FittedHead, HeadFitter};
use katgpt_core::template_decode::{DecodeError, Grammar, Seg, Template, Vocab};

/// The tetris oracle fixture, verbatim from katgpt-rs `tests/fixtures/`
/// (Plan 607 T0b + T3). Pinned by BLAKE3 in the serve tests.
const TETRIS_FIXTURE: &str = include_str!("../assets/game_heads/tetris_oracle_laya_en_v2.jsonl");

/// BLAKE3 of [`TETRIS_FIXTURE`] — the cross-repo data contract.
pub const TETRIS_FIXTURE_BLAKE3: &str =
    "f32c8577bca50726618d2bb4fb27c904148161d650f16a59c01676a97fa540bb";

// ── the laya-tetris-v2 grammar ───────────────────────────────────────────
// Ported from katgpt-rs examples/common/grammar_tables.rs (Plan 607 T2):
// the fill ORDINAL is the decoded feature value, so vocabulary order is
// contract. The corpus round-trip test (every fixture sentence decodes and
// re-renders byte-identically) is the port's drift detector.

/// holes_delta class: exact 0/1/2, then the render's bands.
static TETRIS_HOLES: [&str; 5] = [
    "leaves no holes",
    "leaves one hole",
    "leaves two holes",
    "leaves a few holes",
    "leaves many holes",
];
/// `SideBand` clause order (LeftEdge..RightEdge).
static TETRIS_SIDE: [&str; 5] = [
    "on the left edge",
    "on the left side",
    "in the middle",
    "on the right side",
    "on the right edge",
];
/// `BumpBand` order (Flat, Small, Tall, Gap).
static TETRIS_SURFACE: [&str; 4] = [
    "sits flat on the surface",
    "makes a small bump on top",
    "makes a tall step on top",
    "fills a deep gap",
];
/// `HeightBand` order (Low, Medium, Tall) — post-placement bands
/// (0–6 / 7–11 / 12+).
static TETRIS_HEIGHT: [&str; 3] = [
    "the stack stays low",
    "the stack stands medium",
    "the stack grows tall",
];
/// lines_cleared: "" = no clear (the render appends nothing).
static TETRIS_CLEARS: [&str; 5] = [
    "",
    ", and clears a line",
    ", and clears two lines",
    ", and clears three lines",
    ", and clears four lines",
];

static T_TETRIS_SPOT: [Seg; 11] = [
    Seg::Lit("The piece "),
    Seg::Slot(0), // TETRIS_HOLES
    Seg::Lit(" under it "),
    Seg::Slot(1), // TETRIS_SIDE
    Seg::Lit(", "),
    Seg::Slot(2), // TETRIS_SURFACE
    Seg::Lit(", and "),
    Seg::Slot(3), // TETRIS_HEIGHT
    Seg::Lit(""),  // separator only — the two fills are adjacent in the sentence
    Seg::Slot(4), // TETRIS_CLEARS ("" renders as nothing)
    Seg::Lit("."),
];

/// The `laya-tetris-v2` per-spot option grammar: 5 slots (holes class,
/// side band, surface band, height band, clears class).
fn tetris_spot() -> Grammar {
    static VOCABS: [Vocab; 5] = [
        &TETRIS_HOLES,
        &TETRIS_SIDE,
        &TETRIS_SURFACE,
        &TETRIS_HEIGHT,
        &TETRIS_CLEARS,
    ];
    static TEMPLATES: [Template; 1] = [Template(&T_TETRIS_SPOT)];
    Grammar::new(&VOCABS, &TEMPLATES)
}

/// Decode a spot sentence → (holes, side, surface, height, clears) fills.
fn decode_tetris_spot(g: &Grammar, sentence: &str) -> Result<[u8; 5], DecodeError> {
    let m = g.decode(sentence)?;
    debug_assert_eq!(m.template, 0);
    debug_assert_eq!(m.n_slots, 5);
    Ok([m.fills[0], m.fills[1], m.fills[2], m.fills[3], m.fills[4]])
}

/// The decoded design width: the five CLASS ordinals (Bench 881's decoded
/// arm — the classes ARE the render's vocabulary).
pub const TETRIS_DECODED_F: usize = 5;
/// Design width: F standardized features + intercept.
pub const TETRIS_D: usize = TETRIS_DECODED_F + 1;

/// The decoded feature row (pure — order pinned with the grammar).
pub fn tetris_decoded_features(fills: &[u8; 5]) -> [f64; TETRIS_DECODED_F] {
    [
        fills[0] as f64,
        fills[1] as f64,
        fills[2] as f64,
        fills[3] as f64,
        fills[4] as f64,
    ]
}

// ── the published fit recipe ─────────────────────────────────────────────
// katgpt-rs examples/common/micro_fit.rs (tetris_03's recipe, T2's
// width-generic form) at the decoded width. The tests assert the published
// anchors, so any recipe drift reds instead of silently re-fitting.

/// Pinned λ grid (standardized scale). Selection criterion: state-level
/// LOO MSE — corpus-only, deterministic, never the agreement number.
pub const RIDGE_GRID: [f64; 4] = [1e-3, 1e-2, 1e-1, 1.0];

/// Corpus-side standardization stats (mean/inv-std in fixed column order;
/// std == 0 → the column carries no signal, map to 0).
#[derive(Debug, Clone, PartialEq)]
pub struct Standardizer {
    pub mean: [f64; TETRIS_DECODED_F],
    pub inv_std: [f64; TETRIS_DECODED_F],
}

impl Standardizer {
    /// Fit over the corpus raws (fixed column order — part of the recipe).
    pub fn fit(rows: &[[f64; TETRIS_DECODED_F]]) -> Self {
        let n = rows.len().max(1) as f64;
        let mut mean = [0.0; TETRIS_DECODED_F];
        for r in rows {
            for (m, x) in mean.iter_mut().zip(r.iter()) {
                *m += x;
            }
        }
        for m in mean.iter_mut() {
            *m /= n;
        }
        let mut var = [0.0; TETRIS_DECODED_F];
        for r in rows {
            for (v, (x, m)) in var.iter_mut().zip(r.iter().zip(mean.iter())) {
                let d = x - m;
                *v += d * d;
            }
        }
        let mut inv_std = [0.0; TETRIS_DECODED_F];
        for i in 0..TETRIS_DECODED_F {
            let s = (var[i] / n).sqrt();
            inv_std[i] = if s > 0.0 { 1.0 / s } else { 0.0 };
        }
        Self { mean, inv_std }
    }

    /// A live option's design row: standardized features + intercept.
    pub fn design(&self, raw: &[f64; TETRIS_DECODED_F]) -> [f64; TETRIS_D] {
        let mut row = [0.0; TETRIS_D];
        for i in 0..TETRIS_DECODED_F {
            row[i] = (raw[i] - self.mean[i]) * self.inv_std[i];
        }
        row[TETRIS_DECODED_F] = 1.0;
        row
    }
}

// ── fixture shape (the T0b oracle records) ───────────────────────────────

#[derive(serde::Deserialize)]
struct FixtureOption {
    #[allow(dead_code)]
    rot: usize,
    #[allow(dead_code)]
    col: usize,
    sentence: String,
    p_clean: Option<f64>,
}

#[derive(serde::Deserialize)]
struct FixtureState {
    #[allow(dead_code)]
    state_id: String,
    options: Vec<FixtureOption>,
    /// The oracle's decision (lowest-index tie-break) — the agreement
    /// anchor's reference.
    #[serde(default)]
    argmax: usize,
}

/// The parsed corpus: everything the fit + the agreement replay need.
pub struct TetrisCorpus {
    /// The pinned question (the fixture `_meta` record's field).
    pub question: String,
    /// Design rows in fixture order (state 0's options, state 1's, …).
    pub rows: Vec<[f64; TETRIS_D]>,
    /// The oracle's per-option read, row-aligned with [`Self::rows`].
    pub targets: Vec<f64>,
    /// `offsets[s]..offsets[s+1]` is state s's row range (the LOO unit).
    pub offsets: Vec<usize>,
    /// The oracle's per-state decision (lowest-index tie-break).
    pub argmaxes: Vec<usize>,
}

/// Parse the fixture, decode every option sentence through the grammar and
/// standardize by the corpus stats. `_meta` records are skipped. Panics on
/// any drift — the fixture is digest-pinned data; a corrupt copy is a
/// broken build and must die at boot, not serve nonsense.
pub fn parse_corpus() -> TetrisCorpus {
    let g = tetris_spot();
    let mut question = String::new();
    let mut raws: Vec<[f64; TETRIS_DECODED_F]> = Vec::new();
    let mut targets: Vec<f64> = Vec::new();
    let mut offsets = vec![0usize];
    let mut argmaxes = Vec::new();

    for (ln, line) in TETRIS_FIXTURE.lines().enumerate() {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("tetris fixture line {}: {e}", ln + 1));
        if v["state_id"] == "_meta" {
            question = v["question"].as_str().unwrap_or_default().to_string();
            continue;
        }
        let s: FixtureState = serde_json::from_value(v)
            .unwrap_or_else(|e| panic!("tetris fixture line {}: {e}", ln + 1));
        for o in &s.options {
            let p = o.p_clean.unwrap_or_else(|| {
                panic!(
                    "tetris fixture state at line {}: p_clean missing — the corpus needs \
                     the oracle's per-option read",
                    ln + 1
                )
            });
            let fills = decode_tetris_spot(&g, &o.sentence).unwrap_or_else(|e| {
                panic!("tetris fixture line {}: option sentence refused: {e:?}", ln + 1)
            });
            raws.push(tetris_decoded_features(&fills));
            targets.push(p);
        }
        offsets.push(raws.len());
        argmaxes.push(s.argmax);
    }
    let stdizer = Standardizer::fit(&raws);
    let rows = raws.iter().map(|r| stdizer.design(r)).collect();
    TetrisCorpus {
        question,
        rows,
        targets,
        offsets,
        argmaxes,
    }
}

/// State-level LOO λ selection over the pinned grid (the published
/// criterion: MSE on the held-out state's options — sibling options never
/// leak; the agreement number is never consulted). Returns
/// (chosen λ, per-state LOO picks at the chosen λ).
pub fn loo_select(
    fitter: &mut HeadFitter<TETRIS_D>,
    corpus: &TetrisCorpus,
) -> (f64, Vec<usize>) {
    let n_states = corpus.argmaxes.len();
    let mut chosen = RIDGE_GRID[0];
    let mut chosen_mse = f64::INFINITY;
    let mut chosen_picks = vec![0usize; n_states];
    let ranges: Vec<(usize, usize)> = (0..n_states)
        .map(|s| (corpus.offsets[s], corpus.offsets[s + 1]))
        .collect();
    for &lam in &RIDGE_GRID {
        let mut sq = 0.0f64;
        let mut picks = vec![0usize; n_states];
        for ((_, &(a, b)), slot) in ranges.iter().enumerate().zip(picks.iter_mut()) {
            let mut train: Vec<[f64; TETRIS_D]> = Vec::with_capacity(corpus.rows.len() - (b - a));
            train.extend_from_slice(&corpus.rows[..a]);
            train.extend_from_slice(&corpus.rows[b..]);
            let mut ty: Vec<f64> = Vec::with_capacity(corpus.targets.len() - (b - a));
            ty.extend_from_slice(&corpus.targets[..a]);
            ty.extend_from_slice(&corpus.targets[b..]);
            let head = fitter.fit_into(&train, &ty, lam);
            let mut best_pred = f64::NEG_INFINITY;
            let mut bi = 0usize;
            for (j, row) in corpus.rows[a..b].iter().enumerate() {
                let p = head.score(row);
                let e = p - corpus.targets[a + j];
                sq += e * e;
                if p > best_pred {
                    best_pred = p;
                    bi = j;
                }
            }
            *slot = bi;
        }
        let mse = sq / corpus.targets.len() as f64;
        if mse < chosen_mse {
            chosen_mse = mse;
            chosen = lam;
            chosen_picks = picks;
        }
    }
    (chosen, chosen_picks)
}

/// BLAKE3 over the head weights (f64 LE) — the determinism anchor.
pub fn head_digest(h: &FittedHead<TETRIS_D>) -> blake3::Hash {
    let mut bytes = Vec::with_capacity(TETRIS_D * 8);
    for w in h.weights() {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    blake3::hash(&bytes)
}

// ── the serving struct ───────────────────────────────────────────────────

/// The fitted Tetris head + its grammar + the pinned question. Built once
/// at boot; scoring is one decode + one dot product.
pub struct GameHeads {
    grammar: Grammar,
    head: FittedHead<TETRIS_D>,
    stdizer: Standardizer,
    question: String,
    lambda: f64,
    digest: blake3::Hash,
    n_options: usize,
}

impl GameHeads {
    /// Fit the head from the embedded fixture (the published recipe).
    /// Panics on any fixture drift — the data is compile-time and
    /// digest-pinned; a panic here means a broken build, loud by design.
    pub fn build() -> Self {
        let corpus = parse_corpus();
        let n_options = corpus.rows.len();
        let mut fitter = HeadFitter::<TETRIS_D>::new();
        let (lambda, _loo_picks) = loo_select(&mut fitter, &corpus);
        let head = fitter.fit_into(&corpus.rows, &corpus.targets, lambda);
        let digest = head_digest(&head);
        // The standardizer the corpus rows were designed with — refit over
        // the same raws the parse saw. The corpus rows carry the design
        // (standardized + intercept), so recover the raws by re-walking
        // the fixture: identical bytes → identical stats, and the digest
        // test pins the whole pipeline end to end.
        let stdizer = {
            let g = tetris_spot();
            let mut raws = Vec::with_capacity(n_options);
            for line in TETRIS_FIXTURE.lines() {
                let v: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if v["state_id"] == "_meta" {
                    continue;
                }
                let s: FixtureState = match serde_json::from_value(v) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                for o in &s.options {
                    if let Ok(fills) = decode_tetris_spot(&g, &o.sentence) {
                        raws.push(tetris_decoded_features(&fills));
                    }
                }
            }
            Standardizer::fit(&raws)
        };
        Self {
            grammar: tetris_spot(),
            head,
            stdizer,
            question: corpus.question,
            lambda,
            digest,
            n_options,
        }
    }

    /// The head's blake3 (f64 LE weights) — the determinism anchor.
    pub fn digest_hex(&self) -> String {
        self.digest.to_hex().to_string()
    }

    /// The chosen λ (the LOO-selected ridge, reported for honesty).
    pub fn lambda(&self) -> f64 {
        self.lambda
    }

    /// Corpus size (options), for the boot log.
    pub fn n_options(&self) -> usize {
        self.n_options
    }

    /// The pinned question (from the fixture's `_meta`).
    pub fn question(&self) -> &str {
        &self.question
    }

    /// The grammar, for the round-trip tests.
    pub fn grammar(&self) -> &Grammar {
        &self.grammar
    }

    /// Decode a sentence to its fill vector (the round-trip tests' seam —
    /// re-render with [`Grammar::render`] and compare byte-identical).
    pub fn decode_ok_for_test(g: &Grammar, sentence: &str) -> Vec<u8> {
        let m = g.decode(sentence).expect("fixture sentence must decode");
        m.fills[..m.n_slots].to_vec()
    }

    /// Score one sentence: decode → features → design → head score,
    /// clamped to [0, 1]. `None` when the sentence is not a well-formed
    /// spot sentence (the caller falls through to the cosine engine).
    pub fn score(&self, sentence: &str) -> Option<f64> {
        let fills = decode_tetris_spot(&self.grammar, sentence).ok()?;
        let raw = tetris_decoded_features(&fills);
        let row = self.stdizer.design(&raw);
        Some(self.head.score(&row).clamp(0.0, 1.0))
    }

    /// Answer a `/decide` request when it is a Tetris spot question.
    /// Every question must be `noul` with the pinned prompt; anything
    /// else returns `None` (fall through — the head's P(clean) semantic
    /// is pinned to that question, never reused under another).
    pub fn respond(&self, req: &DecisionRequest) -> Option<DecisionResponse> {
        if req
            .questions
            .iter()
            .any(|q| q.kind != QuestionKind::Noul || q.prompt != self.question)
        {
            return None;
        }
        let p = self.score(&req.state)?;
        let answers = req
            .questions
            .iter()
            .map(|q| Answer::noul(q.id.clone(), p >= 0.5, p as f32, p as f32))
            .collect();
        Some(DecisionResponse {
            answers,
            routing: Routing {
                lane: Lane::Modelless,
                reason: Some("game-head/tetris (corpus-fitted, Bench 881 decoded arm)".into()),
            },
            calibration: Calibration::none(),
        })
    }
}
