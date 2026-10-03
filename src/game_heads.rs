//! The fitted game heads over HTTP — the arena's modelless lane plays.
//!
//! Plan 607's corpus-fitted heads (katgpt-rs Benches 878/880/881/882) read a
//! closed-grammar game sentence and answer the pinned yes/no question with
//! the fitted P(clean). All THREE arena heads are served (issue 011
//! closed):
//!
//! - **Tetris** — Bench 881's decoded arm (λ=1, in-corpus 44/120, LOO
//!   44/120): the spot features are fully self-contained (five fill
//!   ordinals decoded from ONE sentence), so the request's `state` is the
//!   option sentence itself.
//! - **Lanes** — Bench 880's lanes arm decoded (λ=0.01, in-corpus 84/100,
//!   LOO 84/100; the decoded arm is EXACTLY lossless — identical head
//!   digest `7d3f1d8e…`). The head's cross-lane feature columns (6–7) read
//!   the OTHER lanes, so one lane sentence cannot reproduce it: the
//!   **joined-state protocol** — the request's `state` carries the three
//!   lane sentences one per line (pinned [left, middle, right] order, the
//!   wire order = the lane order), and the request carries exactly three
//!   noul questions; answer *i* is lane *i*'s P(safe).
//! - **Flappy** — Bench 882's v3 decoded arm (λ=1, in-corpus 96/100, LOO
//!   96/100, full head digest `c93d36dc…`): the head row reconstructs the
//!   structured units from the v3 option sentence (band + offset + neutral
//!   post-motion) PLUS the state's pre-rel/v/h, so the request's `state`
//!   carries TWO lines — the state context sentence, then the option
//!   sentence — with exactly one noul question; the answer is that
//!   option's P(clean). (One request per option; the site's per-option
//!   worker shape is preserved.)
//!
//! The line-split convention is the wire's only new shape: closed-grammar
//! sentences never contain newlines, `noul` questions legally carry no
//! options (`WireError::NoulCarriesOptions`), and the request's question
//! count/order carries the answer mapping.
//!
//! Provenance and drift discipline:
//! - the fixtures are verbatim copies of katgpt-rs `tests/fixtures/` (the
//!   laya oracle over the T0b/T5 state dumps), BLAKE3-pinned below and
//!   asserted in `tests/game_heads_serve.rs`; the UNSERVED tetris v3/v4
//!   oracle fixtures are pinned the same way test-side (the issue-031
//!   four-hash: every pin is verified against bytes, never length-only);
//! - each head is FITTED from its fixture by the published recipe
//!   (standardize → λ by state-level LOO MSE over the pinned grid → final
//!   fit). SINCE THE VESSEL EXTRACTION (instinct Proposal 001 T4), the
//!   fit runs at MINT time (`reflex mint-heads`, or the tests) — the
//!   serve binary carries no fixture bytes and fits nothing: it loads
//!   the signed PUBLIC-RELEASE head vessels from `RIIR_REFLEX_HEADS_DIR`
//!   (feature `vessel_public_read`, default-on) and installs them whole;
//!   the tests pin each fit's bit-determinism plus the published
//!   agreement anchors, and the vessel lane pins byte-identical answers;
//! - grammar-invalid sentences, foreign questions and unknown inputs are
//!   LOUD refusals here: they return `None` and the request falls through
//!   to the cosine engine, which abstains off-corpus as before.
//!
//! Parse-precision note (katgpt-rs Bench 890 §G3): a future head fit whose
//! result must land on the katgpt-rs-side head bytes needs the same
//! `serde_json/float_roundtrip` feature katgpt-rs fits under — the DEFAULT
//! parser rounds differently and moves the tetris structured-head digest
//! (`65409c14…` vs `b3c91ee0…`), agreement numbers unaffected.
//!
//! This is boot + edge surface (one parse, one fit, one decode per
//! request) — the engine core's G4 alloc-free window is untouched.

use katgpt_core::decision_wire::{
    Answer, Calibration, DecisionRequest, DecisionResponse, Lane, QuestionKind, Routing,
};
use katgpt_core::state_option_scoring::head::{FittedHead, HeadFitter};
use katgpt_core::template_decode::{DecodeError, Grammar, Seg, Template, Vocab};

/// The three oracle fixtures' file names under `assets/game_heads/` —
/// the MINT INPUTS (repo files; never compiled into the serve binary
/// since the vessel extraction). The mint command and the tests read
/// them from disk; the serve binary never sees them.
pub const TETRIS_FIXTURE_FILE: &str = "tetris_oracle_laya_en_v3.jsonl";
pub const LANES_FIXTURE_FILE: &str = "lanes_oracle_laya_en_v1.jsonl";
pub const FLAPPY_FIXTURE_FILE: &str = "flappy_oracle_laya_en_v3.jsonl";

/// BLAKE3 of the tetris fixture — equals the published v3 fixture pin
/// `12035ebf…6e804` (katgpt-rs tetris_oracle_v3_README). Asserted against
/// the on-disk bytes at mint time and in the tests.
pub const TETRIS_FIXTURE_BLAKE3: &str =
    "12035ebf43d0293c7ec00e716e72ee6a21686cc41a222938a81d0abd9316e804";

/// The lanes oracle fixture file (Plan 607 T5 / Bench 880), verbatim from
/// katgpt-rs `tests/fixtures/`. BLAKE3 of it equals the published Bench 880
/// pin `6a6d02af…4f600`.
pub const LANES_FIXTURE_BLAKE3: &str =
    "6a6d02af05b529749ddac3bf962344c2e22565b467f28a5eecd37031d0a4f600";

/// BLAKE3 of the flappy v3 fixture — equals the published Bench 882 pin
/// `88ac82bf…51fba`.
pub const FLAPPY_V3_FIXTURE_BLAKE3: &str =
    "88ac82bfd2d50fb9f3448d57242d93f8fd9fd02ce101b5c1f97c249206f51fba";

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
    Seg::Lit(""), // separator only — the two fills are adjacent in the sentence
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

// ── the laya-flappy-v3 grammar ─────────────────────────────────────────
// Verbatim from katgpt-rs examples/common/grammar_tables.rs (Bench 882's
// decoded arm): the option sentence carries band + offset + post-motion;
// the STATE sentence (grammar unchanged across v1/v2/v3) carries the
// pre-rel band, motion and gap width. Vocabulary order = contract.

/// `PosBand` order (Below..Above).
static FLAPPY_POS: [&str; 7] = [
    "sinks below the gap",
    "squeezes through the bottom of the gap",
    "glides through the lower half of the gap",
    "glides through the middle of the gap",
    "glides through the upper half of the gap",
    "squeezes through the top of the gap",
    "flies above the gap",
];
/// Fine post_rel relative to the gap center, clamped ±2.
static FLAPPY_OFFSET: [&str; 5] = [
    "under the center",
    "just under the center",
    "at the center",
    "just over the center",
    "over the center",
];
/// Neutral post-motion clause — bijective on the clamped −2..=2 domain.
static FLAPPY_PMOT: [&str; 5] = [
    "drifting down two steps",
    "drifting down one step",
    "holding this height",
    "drifting up one step",
    "drifting up two steps",
];
/// `RelBand` order (WellAbove..WellBelow).
static FLAPPY_REL: [&str; 5] = [
    "well above",
    "a little above",
    "level with",
    "a little below",
    "well below",
];
/// The bird's CURRENT motion — state sentence only.
static FLAPPY_MOT: [&str; 5] = [
    "falling fast",
    "falling",
    "flying level",
    "rising",
    "climbing fast",
];
/// `gap_clause(h)`: 2 → narrow, else wide (bijective on {2, 3}).
static FLAPPY_GAP: [&str; 2] = ["narrow", "wide"];

static T_FLAPPY_OPTION_V3: [Seg; 7] = [
    Seg::Lit("The bird "),
    Seg::Slot(0), // FLAPPY_POS
    Seg::Lit(", "),
    Seg::Slot(1), // FLAPPY_OFFSET
    Seg::Lit(", "),
    Seg::Slot(2), // FLAPPY_PMOT
    Seg::Lit("."),
];
static T_FLAPPY_STATE: [Seg; 7] = [
    Seg::Lit("The bird is "),
    Seg::Slot(0), // FLAPPY_REL
    Seg::Lit(" the gap center, "),
    Seg::Slot(1), // FLAPPY_MOT
    Seg::Lit(". The gap is "),
    Seg::Slot(2), // FLAPPY_GAP
    Seg::Lit(". The pipe is just ahead."),
];

/// The `laya-flappy-v3` per-option grammar: 3 slots (band, offset,
/// post-motion).
fn flappy_option_v3() -> Grammar {
    static VOCABS: [Vocab; 3] = [&FLAPPY_POS, &FLAPPY_OFFSET, &FLAPPY_PMOT];
    static TEMPLATES: [Template; 1] = [Template(&T_FLAPPY_OPTION_V3)];
    Grammar::new(&VOCABS, &TEMPLATES)
}

/// The `laya-flappy-v3` state grammar (unchanged across versions): 3 slots
/// (rel band, motion, gap width).
fn flappy_state() -> Grammar {
    static VOCABS: [Vocab; 3] = [&FLAPPY_REL, &FLAPPY_MOT, &FLAPPY_GAP];
    static TEMPLATES: [Template; 1] = [Template(&T_FLAPPY_STATE)];
    Grammar::new(&VOCABS, &TEMPLATES)
}

/// Decode a flappy v3 option sentence → (band, offset, post-motion) fills.
fn decode_flappy_option_v3(g: &Grammar, sentence: &str) -> Result<[u8; 3], DecodeError> {
    let m = g.decode(sentence)?;
    debug_assert_eq!(m.template, 0);
    debug_assert_eq!(m.n_slots, 3);
    Ok([m.fills[0], m.fills[1], m.fills[2]])
}

/// Decode a flappy state sentence → (rel-band fill, exact v, exact h).
/// v/h are EXACT on the pinned domain (the fixture's own note).
fn decode_flappy_state(g: &Grammar, sentence: &str) -> Result<(u8, i32, i32), DecodeError> {
    let m = g.decode(sentence)?;
    debug_assert_eq!(m.template, 0);
    debug_assert_eq!(m.n_slots, 3);
    let v = m.fills[1] as i32 - 2;
    let h = if m.fills[2] == 0 { 2 } else { 3 };
    Ok((m.fills[0], v, h))
}

/// The flappy v3 decoded design width: the STRUCTURED-UNITS reconstruction
/// (Bench 882's measured design — raw fill ordinals measured 51/100, BELOW
/// constant-pick; this reconstruction measured Δ0 = the structured arm's
/// 96/100).
pub const FLAPPY_V3_DECODED_F: usize = 8;
/// Design width: F standardized features + intercept.
pub const FLAPPY_V3_D: usize = FLAPPY_V3_DECODED_F + 1;

/// The decoded feature row (Bench 882's reconstruction law — verbatim from
/// `flappy_v3_decoded_features`): [post_rel, post_abs_rel, post_v, pre_rel,
/// pre_v, in_gap, edge_margin, gap_half]. Exact on every rendered
/// combination except the documented collapses (crash tails at ±(h+1);
/// |pre_rel| ≥ 2 at ±2).
pub fn flappy_v3_decoded_features(
    post: [u8; 3],
    rel: u8,
    v: i32,
    h: i32,
) -> [f64; FLAPPY_V3_DECODED_F] {
    let (band, offset) = (post[0], post[1]);
    let hh = h;
    let post_rel: i32 = match band {
        0 => -(hh + 1), // Below: tail → boundary
        1 => -hh,       // SqueezeBottom: exact
        2 => {
            if offset == 1 {
                -1
            } else {
                -2
            }
        } // Lower: just-under exact
        3 => 0,         // Middle: exact
        4 => {
            if offset == 3 {
                1
            } else {
                2
            }
        } // Upper: just-over exact
        5 => hh,        // SqueezeTop: exact
        _ => hh + 1,    // Above: tail → boundary
    };
    let pre_rel: i32 = match rel {
        0 => 2,
        1 => 1,
        2 => 0,
        3 => -1,
        _ => -2,
    };
    let post_v = post[2] as i32 - 2;
    let abs = post_rel.abs();
    [
        post_rel as f64,
        abs as f64,
        post_v as f64,
        pre_rel as f64,
        v as f64,
        (abs <= hh) as u8 as f64,
        (hh - abs) as f64,
        h as f64,
    ]
}

// ── the laya-lanes-v1 grammar ──────────────────────────────────────
// Verbatim from katgpt-rs examples/common/grammar_tables.rs (Bench 880's
// lanes arm; the decoded arm is EXACTLY lossless — Bench 881). Two
// templates (clear / blocked); the lane-name fill is contract.

/// `LANE_NAMES` order.
static LANES_LANE: [&str; 3] = ["left", "middle", "right"];
static LANES_DIST: [&str; 2] = ["close", "far"];
/// One noun per obstacle class — the wording pin (`lanes_sim::obstacle_noun`).
static LANES_NOUN: [&str; 3] = ["a barrier", "a train", "a rock"];

static T_LANES_CLEAR: [Seg; 3] = [
    Seg::Lit("The "),
    Seg::Slot(0),
    Seg::Lit(" lane is clear ahead."),
];
static T_LANES_BLOCKED: [Seg; 7] = [
    Seg::Lit("The "),
    Seg::Slot(0), // LANES_LANE
    Seg::Lit(" lane is blocked "),
    Seg::Slot(1), // LANES_DIST
    Seg::Lit(" ahead by "),
    Seg::Slot(2), // LANES_NOUN
    Seg::Lit("."),
];

/// The `laya-lanes-v1` per-lane grammar: two templates (clear / blocked).
fn lanes_option() -> Grammar {
    static VOCABS: [Vocab; 3] = [&LANES_LANE, &LANES_DIST, &LANES_NOUN];
    static TEMPLATES: [Template; 2] = [Template(&T_LANES_CLEAR), Template(&T_LANES_BLOCKED)];
    Grammar::new(&VOCABS, &TEMPLATES)
}

/// One lane's decoded obstacle: kind ordinal (0 = clear, then noun order)
/// and the distance fill when blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneDecoded {
    pub kind: u8,
    pub dist: Option<u8>,
    /// The lane-name fill — which lane this sentence describes.
    pub lane: u8,
}

/// Decode one lanes option sentence.
fn decode_lanes_option(g: &Grammar, sentence: &str) -> Result<LaneDecoded, DecodeError> {
    let m = g.decode(sentence)?;
    let lane = m.fills[0];
    match m.template {
        0 => Ok(LaneDecoded {
            kind: 0,
            dist: None,
            lane,
        }),
        1 => Ok(LaneDecoded {
            kind: 1 + m.fills[2],
            dist: Some(m.fills[1]),
            lane,
        }),
        t => unreachable!("lanes grammar has 2 templates, decoded {t}"),
    }
}

/// The lanes decoded feature width — EXACTLY the structured row (all eight
/// columns are recoverable from the three sentences; the lossless anchor
/// of the whole arm, Bench 881).
pub const LANES_DECODED_F: usize = 8;
/// Design width: F standardized features + intercept.
pub const LANES_D: usize = LANES_DECODED_F + 1;

/// The decoded feature row for one lane given ALL THREE lanes' decodes
/// (verbatim from `lanes_decoded_features`): lane-local kind/dist bits plus
/// the cross-lane context columns (6–7) — the reason the serving path
/// needs the joined state.
pub fn lanes_decoded_features(d: &[LaneDecoded; 3], lane: usize) -> [f64; LANES_DECODED_F] {
    let l = d[lane];
    let blocked = l.kind != 0;
    let dist_close = l.dist == Some(0);
    let dist_far = l.dist == Some(1);
    [
        blocked as u8 as f64,
        (blocked && dist_close) as u8 as f64,
        (blocked && dist_far) as u8 as f64,
        (l.kind == 1) as u8 as f64,
        (l.kind == 2) as u8 as f64,
        (l.kind == 3) as u8 as f64,
        d.iter()
            .enumerate()
            .filter(|&(i, x)| i != lane && x.kind != 0)
            .count() as f64,
        d.iter().filter(|x| x.kind == 0).count() as f64,
    ]
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
/// std == 0 → the column carries no signal, map to 0). Verbatim from the
/// published recipe (`micro_fit::Standardizer<F>`).
#[derive(Debug, Clone, PartialEq)]
pub struct Standardizer<const F: usize> {
    pub mean: [f64; F],
    pub inv_std: [f64; F],
}

impl<const F: usize> Standardizer<F> {
    /// Fit over the corpus raws (fixed column order — part of the recipe).
    pub fn fit(rows: &[[f64; F]]) -> Self {
        let n = rows.len().max(1) as f64;
        let mut mean = [0.0; F];
        for r in rows {
            for (m, x) in mean.iter_mut().zip(r.iter()) {
                *m += x;
            }
        }
        for m in mean.iter_mut() {
            *m /= n;
        }
        let mut var = [0.0; F];
        for r in rows {
            for (v, (x, m)) in var.iter_mut().zip(r.iter().zip(mean.iter())) {
                let d = x - m;
                *v += d * d;
            }
        }
        let mut inv_std = [0.0; F];
        for i in 0..F {
            let s = (var[i] / n).sqrt();
            inv_std[i] = if s > 0.0 { 1.0 / s } else { 0.0 };
        }
        Self { mean, inv_std }
    }

    /// A live option's design row: standardized features + intercept. The
    /// design width is a METHOD-level const parameter (an impl-level `D`
    /// would be unconstrained — E0207), inferred at every call site.
    pub fn design<const D: usize>(&self, raw: &[f64; F]) -> [f64; D] {
        let mut row = [0.0; D];
        for i in 0..F {
            row[i] = (raw[i] - self.mean[i]) * self.inv_std[i];
        }
        row[F] = 1.0;
        row
    }
}

// ── fixture shape (the oracle records) ─────────────────────────────
// The tetris fixture's option rows also carry rot/col/row/cells (the T0b
// dump's structured payload) — ignored here: the serving path reads only
// the sentences, the oracle reads, and the argmaxes.
#[derive(serde::Deserialize)]
struct TetrisFixtureOption {
    #[allow(dead_code)]
    rot: usize,
    #[allow(dead_code)]
    col: usize,
    sentence: String,
    p_clean: Option<f64>,
}

#[derive(serde::Deserialize)]
struct TetrisFixtureState {
    #[allow(dead_code)]
    state_id: String,
    options: Vec<TetrisFixtureOption>,
    /// The oracle's decision (lowest-index tie-break) — the agreement
    /// anchor's reference.
    #[serde(default)]
    argmax: usize,
}

/// The parsed corpus: everything the fit + the agreement replay need.
pub struct HeadCorpus<const D: usize> {
    /// The pinned question (the fixture `_meta` record's field).
    pub question: String,
    /// Design rows in fixture order (state 0's options, state 1's, …).
    pub rows: Vec<[f64; D]>,
    /// The oracle's per-option read, row-aligned with [`Self::rows`].
    pub targets: Vec<f64>,
    /// `offsets[s]..offsets[s+1]` is state s's row range (the LOO unit).
    pub offsets: Vec<usize>,
    /// The oracle's per-state decision (lowest-index tie-break).
    pub argmaxes: Vec<usize>,
}

/// The tetris corpus at its published width.
pub type TetrisCorpus = HeadCorpus<TETRIS_D>;

/// Parse a jsonl oracle fixture line into `(state_id, options, argmax)`.
/// `_meta` records yield `None`. Panics on any drift — the fixtures are
/// digest-pinned data; a corrupt copy is a broken build and must die at
/// boot, not serve nonsense.
/// The fixture's per-state record: options (sentence + p_clean + the
/// structured `features` cross-check row) + the oracle's argmax + the
/// state context sentence (flappy's head needs it; absent elsewhere).
struct FixtureStateRecord {
    state_sentence: Option<String>,
    options: Vec<(String, f64, Option<Vec<f64>>)>,
    argmax: usize,
}

fn parse_fixture_state(v: serde_json::Value, ln: usize, name: &'static str) -> FixtureStateRecord {
    #[derive(serde::Deserialize)]
    struct FixtureOption {
        sentence: String,
        p_clean: Option<f64>,
        #[serde(default)]
        features: Option<Vec<f64>>,
    }
    #[derive(serde::Deserialize)]
    struct FixtureState {
        #[serde(default)]
        state_sentence: Option<String>,
        options: Vec<FixtureOption>,
        #[serde(default)]
        argmax: usize,
    }
    let s: FixtureState =
        serde_json::from_value(v).unwrap_or_else(|e| panic!("{name} fixture line {}: {e}", ln + 1));
    FixtureStateRecord {
        state_sentence: s.state_sentence,
        options: s
            .options
            .into_iter()
            .map(|o| {
                let p = o.p_clean.unwrap_or_else(|| {
                    panic!(
                        "{name} fixture at line {}: p_clean missing — the corpus needs \
                         the oracle's per-option read",
                        ln + 1
                    )
                });
                (o.sentence, p, o.features)
            })
            .collect(),
        argmax: s.argmax,
    }
}

/// Cross-check a decoded feature row against the fixture's structured
/// `features` row — the free drift detector. LANES ONLY: the lanes decoded
/// row EQUALS the structured one on every row (Bench 881's losslessness —
/// replayed per row here). Flappy's fixture carries the TRUE structured
/// geometry while the reconstruction lawfully collapses the documented
/// tails, so no equality check exists there (its pin is the full digest).
fn assert_features_match(name: &str, ln: usize, i: usize, raw: &[f64], fixture: &Option<Vec<f64>>) {
    if let Some(fx) = fixture {
        assert_eq!(
            raw,
            fx.as_slice(),
            "{name} fixture line {} option {i}: decoded features != fixture features",
            ln + 1
        );
    }
}

/// Parse the tetris fixture text, decode every option sentence through the
/// grammar and standardize by the corpus stats. Panics on any drift.
pub fn parse_corpus(fixture: &str) -> (TetrisCorpus, Standardizer<TETRIS_DECODED_F>) {
    let g = tetris_spot();
    let mut question = String::new();
    let mut raws: Vec<[f64; TETRIS_DECODED_F]> = Vec::new();
    let mut targets: Vec<f64> = Vec::new();
    let mut offsets = vec![0usize];
    let mut argmaxes = Vec::new();

    for (ln, line) in fixture.lines().enumerate() {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("tetris fixture line {}: {e}", ln + 1));
        if v["state_id"] == "_meta" {
            question = v["question"].as_str().unwrap_or_default().to_string();
            continue;
        }
        let s: TetrisFixtureState = serde_json::from_value(v)
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
                panic!(
                    "tetris fixture line {}: option sentence refused: {e:?}",
                    ln + 1
                )
            });
            raws.push(tetris_decoded_features(&fills));
            targets.push(p);
        }
        offsets.push(raws.len());
        argmaxes.push(s.argmax);
    }
    let stdizer = Standardizer::fit(&raws);
    let rows = raws.iter().map(|r| stdizer.design(r)).collect();
    (
        TetrisCorpus {
            question,
            rows,
            targets,
            offsets,
            argmaxes,
        },
        stdizer,
    )
}

/// Parse the lanes fixture (Bench 880): decode ALL THREE lane sentences of
/// each state first — the head's cross-lane columns (6–7) read them — then
/// build each lane's 8-column row and cross-check it against the fixture's
/// structured `features` (the losslessness proof, replayed per row). The
/// lane-name fill must equal the option's position (the pinned slot
/// order). Panics on any drift.
pub fn parse_lanes_corpus(fixture: &str) -> (HeadCorpus<LANES_D>, Standardizer<LANES_DECODED_F>) {
    let g = lanes_option();
    let mut question = String::new();
    let mut raws: Vec<[f64; LANES_DECODED_F]> = Vec::new();
    let mut targets: Vec<f64> = Vec::new();
    let mut offsets = vec![0usize];
    let mut argmaxes = Vec::new();

    for (ln, line) in fixture.lines().enumerate() {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("lanes fixture line {}: {e}", ln + 1));
        if v["state_id"] == "_meta" {
            question = v["question"].as_str().unwrap_or_default().to_string();
            continue;
        }
        let s = parse_fixture_state(v, ln, "lanes");
        assert_eq!(
            s.options.len(),
            3,
            "lanes fixture line {}: every state carries exactly 3 lanes",
            ln + 1
        );
        let lanes: Vec<LaneDecoded> = s
            .options
            .iter()
            .enumerate()
            .map(|(i, (sentence, _, _))| {
                let d = decode_lanes_option(&g, sentence)
                    .unwrap_or_else(|e| panic!("lanes fixture line {} option {i}: {e:?}", ln + 1));
                assert_eq!(
                    d.lane,
                    i as u8,
                    "lanes fixture line {} option {i}: lane slot order drifted",
                    ln + 1
                );
                d
            })
            .collect();
        let lanes: [LaneDecoded; 3] = lanes.try_into().expect("checked 3 above");
        for (i, (_, p, fx)) in s.options.iter().enumerate() {
            let raw = lanes_decoded_features(&lanes, i);
            assert_features_match("lanes", ln, i, &raw, fx);
            raws.push(raw);
            targets.push(*p);
        }
        offsets.push(raws.len());
        argmaxes.push(s.argmax);
    }
    let stdizer = Standardizer::fit(&raws);
    let rows = raws.iter().map(|r| stdizer.design(r)).collect();
    (
        HeadCorpus {
            question,
            rows,
            targets,
            offsets,
            argmaxes,
        },
        stdizer,
    )
}

/// Parse the flappy v3 fixture (Bench 882): decode each state's state
/// sentence once (pre-rel band + exact v/h), each option sentence
/// (band + offset + post-motion), and reconstruct the STRUCTURED UNITS per
/// option. NO equality cross-check against the fixture's `features` here —
/// that column carries the STRUCTURED TRUE geometry, while the
/// reconstruction lawfully collapses the documented tails (post_rel at
/// ±(h+1); |pre_rel| ≥ 2 clamped). That difference is exactly why Bench
/// 882 pins TWO digests at one 96/100 agreement (structured `dc6bcf73…`,
/// decoded `c93d36dc…`): the reconstruction is pinned by the FULL decoded
/// digest instead. Panics on any decode drift.
pub fn parse_flappy_v3_corpus(
    fixture: &str,
) -> (HeadCorpus<FLAPPY_V3_D>, Standardizer<FLAPPY_V3_DECODED_F>) {
    let go = flappy_option_v3();
    let gs = flappy_state();
    let mut question = String::new();
    let mut raws: Vec<[f64; FLAPPY_V3_DECODED_F]> = Vec::new();
    let mut targets: Vec<f64> = Vec::new();
    let mut offsets = vec![0usize];
    let mut argmaxes = Vec::new();

    for (ln, line) in fixture.lines().enumerate() {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("flappy fixture line {}: {e}", ln + 1));
        if v["state_id"] == "_meta" {
            question = v["question"].as_str().unwrap_or_default().to_string();
            continue;
        }
        let s = parse_fixture_state(v, ln, "flappy");
        let state_sentence = s
            .state_sentence
            .as_deref()
            .unwrap_or_else(|| panic!("flappy fixture line {}: state_sentence missing", ln + 1));
        let (rel, v_vel, h) = decode_flappy_state(&gs, state_sentence)
            .unwrap_or_else(|e| panic!("flappy fixture line {}: state refused: {e:?}", ln + 1));
        for (i, (sentence, p, _fx)) in s.options.iter().enumerate() {
            let post = decode_flappy_option_v3(&go, sentence)
                .unwrap_or_else(|e| panic!("flappy fixture line {} option {i}: {e:?}", ln + 1));
            raws.push(flappy_v3_decoded_features(post, rel, v_vel, h));
            targets.push(*p);
        }
        offsets.push(raws.len());
        argmaxes.push(s.argmax);
    }
    let stdizer = Standardizer::fit(&raws);
    let rows = raws.iter().map(|r| stdizer.design(r)).collect();
    (
        HeadCorpus {
            question,
            rows,
            targets,
            offsets,
            argmaxes,
        },
        stdizer,
    )
}

/// State-level LOO λ selection over the pinned grid (the published
/// criterion: MSE on the held-out state's options — sibling options never
/// leak; the agreement number is never consulted). Returns
/// (chosen λ, per-state LOO picks at the chosen λ).
pub fn loo_select<const D: usize>(
    fitter: &mut HeadFitter<D>,
    corpus: &HeadCorpus<D>,
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
            let mut train: Vec<[f64; D]> = Vec::with_capacity(corpus.rows.len() - (b - a));
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
pub fn head_digest<const D: usize>(h: &FittedHead<D>) -> blake3::Hash {
    head_digest_weights(h.weights())
}

/// [`head_digest`] over raw weights — the VESSEL path's digest: the
/// loaded weights are digested through the exact same bytes the fit-side
/// digest hashes, so a loaded head and its fitted original carry the SAME
/// digest (the acceptance gate asserts the published pins against it).
pub fn head_digest_weights<const D: usize>(weights: &[f64; D]) -> blake3::Hash {
    let mut bytes = Vec::with_capacity(D * 8);
    for w in weights {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    blake3::hash(&bytes)
}

/// Every oracle fixture as `(name, given text's BLAKE3 hex, pinned BLAKE3
/// hex)` — the cross-repo data contract made checkable. The pins were
/// documentation until the bytes were hashed (katgpt-rs Issue 884): a
/// re-copied fixture whose pin was not bumped, or a bumped pin whose
/// fixture was not re-copied, reds in the tests (and the mint refuses
/// before fitting). The fixture TEXTS are passed in — the serve binary
/// carries none.
pub fn fixture_pins<'a>(
    tetris: &str,
    lanes: &str,
    flappy: &str,
) -> [(&'a str, String, &'static str); 3] {
    let hex = |s: &str| blake3::hash(s.as_bytes()).to_hex().to_string();
    [
        ("tetris", hex(tetris), TETRIS_FIXTURE_BLAKE3),
        ("lanes", hex(lanes), LANES_FIXTURE_BLAKE3),
        ("flappy_v3", hex(flappy), FLAPPY_V3_FIXTURE_BLAKE3),
    ]
}

// ── the serving struct ──────────────────────────────────────────────

/// One fitted head + its standardizer + the published-fit metadata. Built
/// once at mint time (from the fit) or load time (from the signed vessel
/// payload); scoring is one decode + one design + one dot product.
///
/// The weights are held DIRECTLY ([f64; D]) rather than as a
/// `FittedHead<D>`: the substrate type has no public constructor from
/// weights (fit-only), and the vessel path must install the minted
/// weights WHOLE — never re-fit. The score fold below is the substrate's
/// sequential f64 fold verbatim, and the vessel-vs-fitted answer-identity
/// gate (`tests/game_heads_vessels.rs`) pins the two paths byte-equal.
struct FittedGameHead<const F: usize, const D: usize> {
    weights: [f64; D],
    stdizer: Standardizer<F>,
    question: String,
    lambda: f64,
    digest: blake3::Hash,
    n_options: usize,
}

impl<const F: usize, const D: usize> FittedGameHead<F, D> {
    /// The published recipe: LOO λ selection over the pinned grid, then the
    /// final fit at the chosen λ. Panics only via the parse (fixture drift).
    fn build(corpus: HeadCorpus<D>, stdizer: Standardizer<F>) -> Self {
        let n_options = corpus.rows.len();
        let mut fitter = HeadFitter::<D>::new();
        let (lambda, _loo_picks) = loo_select(&mut fitter, &corpus);
        let head = fitter.fit_into(&corpus.rows, &corpus.targets, lambda);
        let weights = *head.weights();
        let digest = head_digest(&head);
        Self {
            weights,
            stdizer,
            question: corpus.question,
            lambda,
            digest,
            n_options,
        }
    }

    /// `w·x` — the substrate's sequential f64 fold VERBATIM (no SIMD
    /// reordering): the serving path must score a loaded head exactly as
    /// `FittedHead::score` scores the fitted one.
    fn score_row(&self, x: &[f64; D]) -> f64 {
        let mut s = 0.0f64;
        for (wi, &xi) in self.weights.iter().zip(x.iter()) {
            s = wi.mul_add(xi, s);
        }
        s
    }

    /// Score one decoded feature row: design → head score, clamped [0, 1].
    fn score_raw(&self, raw: &[f64; F]) -> f64 {
        let row: [f64; D] = self.stdizer.design(raw);
        self.score_row(&row).clamp(0.0, 1.0)
    }
}

/// The fitted Tetris head + its grammar. The request's `state` is the spot
/// sentence itself (the features are self-contained).
struct TetrisHead {
    grammar: Grammar,
    fit: FittedGameHead<TETRIS_DECODED_F, TETRIS_D>,
}

impl TetrisHead {
    fn build(fixture: &str) -> Self {
        let (corpus, stdizer) = parse_corpus(fixture);
        Self {
            grammar: tetris_spot(),
            fit: FittedGameHead::build(corpus, stdizer),
        }
    }

    fn score(&self, sentence: &str) -> Option<f64> {
        let fills = decode_tetris_spot(&self.grammar, sentence).ok()?;
        Some(self.fit.score_raw(&tetris_decoded_features(&fills)))
    }
}

/// The fitted lanes head + its grammar. The request's `state` is the
/// joined-state protocol's THREE lane sentences (one per line, pinned
/// [left, middle, right] order) and the request carries exactly three
/// noul questions; answer *i* is lane *i*'s P(safe).
struct LanesHead {
    grammar: Grammar,
    fit: FittedGameHead<LANES_DECODED_F, LANES_D>,
}

impl LanesHead {
    fn build(fixture: &str) -> Self {
        let (corpus, stdizer) = parse_lanes_corpus(fixture);
        Self {
            grammar: lanes_option(),
            fit: FittedGameHead::build(corpus, stdizer),
        }
    }

    /// Split the joined state (one sentence per line), decode all three
    /// lanes, score each lane's 8-column row.
    fn score_turn(&self, state: &str) -> Option<[f64; 3]> {
        let lines: Vec<&str> = state.lines().collect();
        if lines.len() != 3 {
            return None;
        }
        let mut lanes = [LaneDecoded {
            kind: 0,
            dist: None,
            lane: 0,
        }; 3];
        for (i, sentence) in lines.iter().enumerate() {
            let d = decode_lanes_option(&self.grammar, sentence).ok()?;
            // The protocol pins position = lane; a sentence naming a
            // different lane than its line is a malformed turn — refuse.
            if d.lane as usize != i {
                return None;
            }
            lanes[i] = d;
        }
        Some([
            self.fit.score_raw(&lanes_decoded_features(&lanes, 0)),
            self.fit.score_raw(&lanes_decoded_features(&lanes, 1)),
            self.fit.score_raw(&lanes_decoded_features(&lanes, 2)),
        ])
    }
}

/// The fitted flappy v3 head + its two grammars. The request's `state` is
/// TWO lines — the state context sentence, then the option sentence — with
/// exactly one noul question; the answer is that option's P(clean).
struct FlappyHead {
    option_grammar: Grammar,
    state_grammar: Grammar,
    fit: FittedGameHead<FLAPPY_V3_DECODED_F, FLAPPY_V3_D>,
}

impl FlappyHead {
    fn build(fixture: &str) -> Self {
        let (corpus, stdizer) = parse_flappy_v3_corpus(fixture);
        Self {
            option_grammar: flappy_option_v3(),
            state_grammar: flappy_state(),
            fit: FittedGameHead::build(corpus, stdizer),
        }
    }

    /// Score one (state, option) sentence pair.
    fn score_pair(&self, state_sentence: &str, option_sentence: &str) -> Option<f64> {
        let (rel, v, h) = decode_flappy_state(&self.state_grammar, state_sentence).ok()?;
        let post = decode_flappy_option_v3(&self.option_grammar, option_sentence).ok()?;
        Some(
            self.fit
                .score_raw(&flappy_v3_decoded_features(post, rel, v, h)),
        )
    }
}

/// The arena's game heads. Since the vessel extraction (instinct Proposal
/// 001 T4 — A1: bytes are runtime, capability is compile-time) each head
/// is an `Option`: PRESENT when fitted at mint time or loaded from its
/// signed PUBLIC-RELEASE vessel, ABSENT otherwise — and an absent lane is
/// a LOUD serving posture (the boot line names the env + the mint
/// command), never a silent one.
pub struct GameHeads {
    tetris: Option<TetrisHead>,
    lanes: Option<LanesHead>,
    flappy: Option<FlappyHead>,
}

/// The vessel file names inside the heads dir (`RIIR_REFLEX_HEADS_DIR`).
pub const HEAD_VESSEL_FILES: [&str; 3] = ["tetris.vessel", "lanes.vessel", "flappy.vessel"];

/// A loud-absent `GameHeads`: every lane absent. `respond` answers `None`
/// for everything (the requests fall through to the abstaining engine).
pub fn absent_heads_message(dir_env: &str) -> String {
    format!(
        "game heads: ABSENT — the arena game boards will abstain. Set {dir_env} to a minted \
         heads dir; mint with: reflex mint-heads --fixtures assets/game_heads --out <dir> \
         --key-id <u32> --key <64-hex-seed> (then trust it: RIIR_REFLEX_HEADS_PUBKEY=<the \
         verifying key hex `reflexer sign` / `reflex mint-heads` prints>)",
        dir_env = dir_env
    )
}

fn head_response(req: &DecisionRequest, ps: &[f64], reason: &str) -> DecisionResponse {
    let answers = req
        .questions
        .iter()
        .zip(ps.iter())
        .map(|(q, p)| Answer::noul(q.id.clone(), *p >= 0.5, *p as f32, *p as f32))
        .collect();
    DecisionResponse {
        answers,
        routing: Routing {
            lane: Lane::Modelless,
            reason: Some(reason.to_string()),
        },
        calibration: Calibration::none(),
    }
}

impl GameHeads {
    /// Fit all three heads from the given fixture texts (the published
    /// recipes) — the MINT-time constructor (the mint subcommand, the
    /// tests). Panics on any fixture drift — the data is digest-pinned; a
    /// panic here means a broken mint, loud by design. The SERVE binary
    /// never calls this: it loads vessels (`from_vessel_dir`).
    pub fn build(tetris_fixture: &str, lanes_fixture: &str, flappy_fixture: &str) -> Self {
        Self {
            tetris: Some(TetrisHead::build(tetris_fixture)),
            lanes: Some(LanesHead::build(lanes_fixture)),
            flappy: Some(FlappyHead::build(flappy_fixture)),
        }
    }

    /// The loud-absent posture: every lane absent (`respond` falls through
    /// for everything).
    pub fn absent() -> Self {
        Self { tetris: None, lanes: None, flappy: None }
    }

    pub fn has_tetris(&self) -> bool {
        self.tetris.is_some()
    }

    pub fn has_lanes(&self) -> bool {
        self.lanes.is_some()
    }

    pub fn has_flappy(&self) -> bool {
        self.flappy.is_some()
    }

    #[cfg(feature = "vessel_public_read")]
    pub fn has_any(&self) -> bool {
        self.tetris.is_some() || self.lanes.is_some() || self.flappy.is_some()
    }

    /// The tetris head's blake3 (f64 LE weights) — the determinism anchor.
    /// Panics when the tetris lane is absent (callers gate on
    /// [`Self::has_tetris`]; the tests and the boot log always do).
    pub fn digest_hex(&self) -> String {
        self.tetris.as_ref().expect("tetris head absent").fit.digest.to_hex().to_string()
    }

    /// The tetris head's chosen λ (the LOO-selected ridge, honesty).
    pub fn lambda(&self) -> f64 {
        self.tetris.as_ref().expect("tetris head absent").fit.lambda
    }

    /// The tetris corpus size (options), for the boot log.
    pub fn n_options(&self) -> usize {
        self.tetris.as_ref().expect("tetris head absent").fit.n_options
    }

    /// The tetris head's pinned question (from the fixture's `_meta`).
    pub fn question(&self) -> &str {
        &self.tetris.as_ref().expect("tetris head absent").fit.question
    }

    /// The tetris grammar, for the round-trip tests.
    pub fn grammar(&self) -> &Grammar {
        &self.tetris.as_ref().expect("tetris head absent").grammar
    }

    /// The lanes fit's published anchors (λ, digest hex, corpus size).
    pub fn lanes_fit(&self) -> (f64, String, usize) {
        let fit = &self.lanes.as_ref().expect("lanes head absent").fit;
        (fit.lambda, fit.digest.to_hex().to_string(), fit.n_options)
    }

    /// The flappy v3 fit's published anchors (λ, digest hex, corpus size).
    pub fn flappy_fit(&self) -> (f64, String, usize) {
        let fit = &self.flappy.as_ref().expect("flappy head absent").fit;
        (fit.lambda, fit.digest.to_hex().to_string(), fit.n_options)
    }

    /// The lanes head's pinned question (for the tests' request builders).
    pub fn lanes_question(&self) -> &str {
        &self.lanes.as_ref().expect("lanes head absent").fit.question
    }

    /// The flappy head's pinned question (for the tests' request builders).
    pub fn flappy_question(&self) -> &str {
        &self.flappy.as_ref().expect("flappy head absent").fit.question
    }

    /// The flappy v3 grammars, for the round-trip tests
    /// (option grammar, state grammar).
    pub fn flappy_grammars(&self) -> (&Grammar, &Grammar) {
        let flappy = self.flappy.as_ref().expect("flappy head absent");
        (&flappy.option_grammar, &flappy.state_grammar)
    }

    /// The lanes grammar, for the round-trip tests.
    pub fn lanes_grammar(&self) -> &Grammar {
        &self.lanes.as_ref().expect("lanes head absent").grammar
    }

    /// Score one tetris spot sentence (the tests' seam).
    pub fn score(&self, sentence: &str) -> Option<f64> {
        self.tetris.as_ref()?.score(sentence)
    }

    /// Score one lanes turn from its joined state (the tests' seam).
    pub fn score_lanes(&self, joined: &str) -> Option<[f64; 3]> {
        self.lanes.as_ref()?.score_turn(joined)
    }

    /// Score one flappy (state, option) pair (the tests' seam).
    pub fn score_flappy(&self, state: &str, option: &str) -> Option<f64> {
        self.flappy.as_ref()?.score_pair(state, option)
    }

    /// Decode a sentence to its fill vector (the round-trip tests' seam —
    /// re-render with [`Grammar::render`] and compare byte-identical).
    pub fn decode_ok_for_test(g: &Grammar, sentence: &str) -> Vec<u8> {
        let m = g.decode(sentence).expect("fixture sentence must decode");
        m.fills[..m.n_slots].to_vec()
    }

    /// Answer a `/decide` request when it matches one of the PRESENT game
    /// heads' pinned protocol shapes; `None` otherwise (fall through —
    /// the cosine engine abstains off-corpus as before). The prompts are
    /// pairwise distinct, so at most one shape can match.
    pub fn respond(&self, req: &DecisionRequest) -> Option<DecisionResponse> {
        let all_noul_with = |prompt: &str| {
            !req.questions.is_empty()
                && req
                    .questions
                    .iter()
                    .all(|q| q.kind == QuestionKind::Noul && q.prompt == prompt)
        };
        // Tetris: the state IS the spot sentence; every question shares
        // its p (the site sends one question per option).
        if let Some(tetris) = &self.tetris
            && all_noul_with(&tetris.fit.question)
            && let Some(p) = tetris.score(&req.state)
        {
            let ps = vec![p; req.questions.len()];
            return Some(head_response(
                req,
                &ps,
                "game-head/tetris (corpus-fitted, decoded arm)",
            ));
        }
        // Lanes: the joined-state protocol — three lane sentences, one per
        // line, exactly three questions; answer i is lane i's P(safe).
        if let Some(lanes) = &self.lanes
            && req.questions.len() == 3
            && all_noul_with(&lanes.fit.question)
            && let Some(ps) = lanes.score_turn(&req.state)
        {
            return Some(head_response(
                req,
                &ps,
                "game-head/lanes (corpus-fitted, lossless decoded arm)",
            ));
        }
        // Flappy: the (state, option) pair — exactly two lines, exactly
        // one question; the answer is that option's P(clean).
        let lines: Vec<&str> = req.state.lines().collect();
        if let Some(flappy) = &self.flappy
            && req.questions.len() == 1
            && all_noul_with(&flappy.fit.question)
            && lines.len() == 2
            && let Some(p) = flappy.score_pair(lines[0], lines[1])
        {
            return Some(head_response(
                req,
                &[p],
                "game-head/flappy (corpus-fitted, v3 decoded arm)",
            ));
        }
        None
    }
}

// ── the PUBLIC-RELEASE vessel lane (instinct Proposal 001 T4) ─────────
// Heads are signed artifacts, not compiled bytes (A1: vessels never
// committed, never compiled in). The MINT path (the `reflex mint-heads`
// subcommand + the tests) fits from the fixture files, encodes the
// canonical head payload below, and signs it through reflexer-vessel's
// public writer. The SERVE path lazy-loads the vessels from
// RIIR_REFLEX_HEADS_DIR: open → strict ed25519 + blake3 → install the
// head WHOLE — the fit happened at mint time; nothing here re-fits.
#[cfg(feature = "vessel_public_read")]
pub mod head_vessels {
    use super::*;
    use reflexer_vessel::{self as vessel, PinTable, VerifiedVessel};
    use std::path::Path;

    fn hex32(b: &[u8; 32]) -> String {
        let mut s = String::with_capacity(64);
        for byte in b {
            use std::fmt::Write as _;
            let _ = write!(s, "{byte:02x}");
        }
        s
    }

    /// The canonical head-payload magic + version (a HEAD payload, not
    /// the vessel wrapper — the vessel's own format version is v1).
    pub const HEAD_PAYLOAD_MAGIC: [u8; 4] = *b"RFXH";
    pub const HEAD_PAYLOAD_VERSION: u32 = 1;

    /// Every way a head vessel refuses to install. The vessel-layer
    /// errors pass through verbatim; the payload-layer refusals are this
    /// module's own fail-closed taxonomy.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum HeadVesselError {
        Vessel(vessel::VesselError),
        BadPayload(String),
    }

    impl std::fmt::Display for HeadVesselError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Vessel(e) => write!(f, "vessel refused: {e}"),
                Self::BadPayload(why) => write!(f, "head payload refused: {why}"),
            }
        }
    }

    impl std::error::Error for HeadVesselError {}

    /// The canonical head payload — the fitted head, serialized. Layout
    /// (all integers LE, all floats f64 LE; EXACT length, no padding):
    ///
    /// ```text
    /// 0   4   magic   b"RFXH"
    /// 4   4   version u32    (= 1)
    /// 8   4   F       u32    (standardized feature width)
    /// 12  4   D       u32    (design width = F + 1)
    /// 16  8   lambda  f64
    /// 24  8   n_opts  u64
    /// 32  4   qlen    u32
    /// 36  qlen    question UTF-8 bytes
    /// ..  F*8 mean
    /// ..  F*8 inv_std
    /// ..  D*8 weights
    /// ```
    /// The digest is NOT serialized — it is recomputed from the weights
    /// on load through [`super::head_digest_weights`], the same bytes the
    /// fit-side digest hashes, and pinned against the published anchors.
    pub fn encode_head_payload<const F: usize, const D: usize>(
        head: &FittedGameHeadFields<'_, F, D>,
    ) -> Vec<u8> {
        let q = head.question.as_bytes();
        assert!(q.len() <= u32::MAX as usize, "question too long");
        let mut out = Vec::with_capacity(36 + q.len() + 16 * F + 8 * D);
        out.extend_from_slice(&HEAD_PAYLOAD_MAGIC);
        out.extend_from_slice(&HEAD_PAYLOAD_VERSION.to_le_bytes());
        out.extend_from_slice(&(F as u32).to_le_bytes());
        out.extend_from_slice(&(D as u32).to_le_bytes());
        out.extend_from_slice(&head.lambda.to_le_bytes());
        out.extend_from_slice(&(head.n_options as u64).to_le_bytes());
        out.extend_from_slice(&(q.len() as u32).to_le_bytes());
        out.extend_from_slice(q);
        for v in &head.stdizer.mean {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for v in &head.stdizer.inv_std {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for w in head.weights {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    fn rd_u32(b: &[u8], at: usize) -> Result<u32, HeadVesselError> {
        b.get(at..at + 4)
            .map(|s| u32::from_le_bytes(s.try_into().expect("len")))
            .ok_or_else(|| HeadVesselError::BadPayload("truncated payload header".into()))
    }

    fn rd_f64s(b: &[u8], at: usize, n: usize) -> Result<(Vec<f64>, usize), HeadVesselError> {
        let end = at + n * 8;
        if b.len() < end {
            return Err(HeadVesselError::BadPayload("truncated payload body".into()));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let s: [u8; 8] = b[at + i * 8..at + (i + 1) * 8].try_into().expect("len");
            out.push(f64::from_le_bytes(s));
        }
        Ok((out, end))
    }

    /// Decode + FULLY validate a head payload against the expected widths.
    /// Every field is checked; the length must be EXACT (no trailing
    /// bytes) — an unverified or malformed payload never reaches a head.
    pub fn decode_head_payload<const F: usize, const D: usize>(
        payload: &[u8],
    ) -> Result<(String, f64, usize, Standardizer<F>, [f64; D]), HeadVesselError> {
        if payload.len() < 36 || payload[0..4] != HEAD_PAYLOAD_MAGIC {
            return Err(HeadVesselError::BadPayload("not a head payload (magic)".into()));
        }
        if rd_u32(payload, 4)? != HEAD_PAYLOAD_VERSION {
            return Err(HeadVesselError::BadPayload(format!(
                "unknown head payload version {}",
                rd_u32(payload, 4)?
            )));
        }
        let f = rd_u32(payload, 8)? as usize;
        let d = rd_u32(payload, 12)? as usize;
        if f != F || d != D {
            return Err(HeadVesselError::BadPayload(format!(
                "head payload widths F={f} D={d} do not match this lane (F={F} D={D})"
            )));
        }
        let lambda = f64::from_le_bytes(
            payload[16..24]
                .try_into()
                .map_err(|_| HeadVesselError::BadPayload("truncated".into()))?,
        );
        let n_options = u64::from_le_bytes(
            payload[24..32]
                .try_into()
                .map_err(|_| HeadVesselError::BadPayload("truncated".into()))?,
        ) as usize;
        let qlen = rd_u32(payload, 32)? as usize;
        let mut at = 36 + qlen;
        if payload.len() < at {
            return Err(HeadVesselError::BadPayload("question overruns payload".into()));
        }
        let question = String::from_utf8(payload[36..at].to_vec())
            .map_err(|_| HeadVesselError::BadPayload("question is not UTF-8".into()))?;
        let (mean, next) = rd_f64s(payload, at, F)?;
        at = next;
        let (inv_std, next) = rd_f64s(payload, at, F)?;
        at = next;
        let (weights_vec, next) = rd_f64s(payload, at, D)?;
        at = next;
        if at != payload.len() {
            return Err(HeadVesselError::BadPayload(format!(
                "payload has {} trailing bytes",
                payload.len() - at
            )));
        }
        let mut mean_arr = [0.0; F];
        mean_arr.copy_from_slice(&mean);
        let mut inv_std_arr = [0.0; F];
        inv_std_arr.copy_from_slice(&inv_std);
        let mut weights = [0.0; D];
        weights.copy_from_slice(&weights_vec);
        if !lambda.is_finite() {
            return Err(HeadVesselError::BadPayload("lambda is not finite".into()));
        }
        Ok((question, lambda, n_options, Standardizer { mean: mean_arr, inv_std: inv_std_arr }, weights))
    }

    /// The mint-side shape: one fitted head's fields, borrowable for
    /// encode. (A constructor over the private `FittedGameHead` — the
    /// heads stay private to the module; minting reads them through this.)
    pub struct FittedGameHeadFields<'a, const F: usize, const D: usize> {
        pub question: &'a str,
        pub lambda: f64,
        pub n_options: usize,
        pub stdizer: &'a Standardizer<F>,
        pub weights: &'a [f64; D],
    }

    /// Mint ONE head vessel (the `reflex mint-heads` core, callable from
    /// the tests): fit fields → canonical payload → reflexer's public
    /// writer. Returns the vessel bytes + the commitment (the pin).
    pub fn mint_head_vessel<const F: usize, const D: usize>(
        key: &vessel::ed25519_dalek::SigningKey,
        key_id: u32,
        artifact_version: u64,
        fields: FittedGameHeadFields<'_, F, D>,
    ) -> Result<vessel::writer::MintedVessel, vessel::VesselError> {
        let payload = encode_head_payload(&fields);
        vessel::writer::sign_public(key, key_id, artifact_version, [0u8; 32], &payload)
    }

    /// One minted head's report row (the pin the mint prints and the
    /// release notes record).
    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    pub struct MintedHead {
        pub name: &'static str,
        pub file: &'static str,
        pub commitment_hex: String,
        pub head_digest_hex: String,
        pub lambda: f64,
        pub n_options: usize,
    }

    /// Mint ALL THREE heads: read the fixture files from `fixtures_dir`,
    /// verify each against its published BLAKE3 pin (a drifted fixture is
    /// refused BEFORE any fitting — the mint never bakes unverified data
    /// into an artifact), fit by the published recipes, sign through the
    /// format repo's public writer, and write `<out_dir>/{tetris,lanes,
    /// flappy}.vessel`. Deterministic: the same fixtures + key + version
    /// produce byte-identical vessels, so re-running a mint refreshes
    /// nothing and moves no pin.
    pub fn mint_all(
        fixtures_dir: &Path,
        out_dir: &Path,
        key: &vessel::ed25519_dalek::SigningKey,
        key_id: u32,
        artifact_version: u64,
    ) -> Result<Vec<MintedHead>, String> {
        let read_fixture = |name: &str, pin: &str| -> Result<String, String> {
            let p = fixtures_dir.join(name);
            let bytes = std::fs::read(&p)
                .map_err(|e| format!("fixture {}: {e} (pass --fixtures <assets/game_heads>)", p.display()))?;
            let got = blake3::hash(&bytes).to_hex().to_string();
            if got != pin {
                return Err(format!(
                    "fixture {} BLAKE3 {} != pinned {} — refusing to mint from drifted data",
                    p.display(),
                    got,
                    pin
                ));
            }
            String::from_utf8(bytes).map_err(|_| format!("fixture {}: not UTF-8", p.display()))
        };
        let tetris_text = read_fixture(super::TETRIS_FIXTURE_FILE, super::TETRIS_FIXTURE_BLAKE3)?;
        let lanes_text = read_fixture(super::LANES_FIXTURE_FILE, super::LANES_FIXTURE_BLAKE3)?;
        let flappy_text = read_fixture(super::FLAPPY_FIXTURE_FILE, super::FLAPPY_V3_FIXTURE_BLAKE3)?;

        let heads = GameHeads::build(&tetris_text, &lanes_text, &flappy_text);

        std::fs::create_dir_all(out_dir)
            .map_err(|e| format!("create {}: {e}", out_dir.display()))?;

        let mut report = Vec::with_capacity(3);

        // Tetris
        let t = heads.tetris.as_ref().expect("build fits all three");
        let minted = mint_head_vessel(
            key,
            key_id,
            artifact_version,
            FittedGameHeadFields {
                question: &t.fit.question,
                lambda: t.fit.lambda,
                n_options: t.fit.n_options,
                stdizer: &t.fit.stdizer,
                weights: &t.fit.weights,
            },
        )
        .map_err(|e| format!("tetris mint refused: {e}"))?;
        report.push(MintedHead {
            name: "tetris",
            file: HEAD_VESSEL_FILES[0],
            commitment_hex: hex32(&minted.commitment),
            head_digest_hex: t.fit.digest.to_hex().to_string(),
            lambda: t.fit.lambda,
            n_options: t.fit.n_options,
        });
        std::fs::write(out_dir.join(HEAD_VESSEL_FILES[0]), &minted.bytes)
            .map_err(|e| format!("write {}: {e}", out_dir.join(HEAD_VESSEL_FILES[0]).display()))?;

        // Lanes
        let l = heads.lanes.as_ref().expect("build fits all three");
        let minted = mint_head_vessel(
            key,
            key_id,
            artifact_version,
            FittedGameHeadFields {
                question: &l.fit.question,
                lambda: l.fit.lambda,
                n_options: l.fit.n_options,
                stdizer: &l.fit.stdizer,
                weights: &l.fit.weights,
            },
        )
        .map_err(|e| format!("lanes mint refused: {e}"))?;
        report.push(MintedHead {
            name: "lanes",
            file: HEAD_VESSEL_FILES[1],
            commitment_hex: hex32(&minted.commitment),
            head_digest_hex: l.fit.digest.to_hex().to_string(),
            lambda: l.fit.lambda,
            n_options: l.fit.n_options,
        });
        std::fs::write(out_dir.join(HEAD_VESSEL_FILES[1]), &minted.bytes)
            .map_err(|e| format!("write {}: {e}", out_dir.join(HEAD_VESSEL_FILES[1]).display()))?;

        // Flappy
        let f = heads.flappy.as_ref().expect("build fits all three");
        let minted = mint_head_vessel(
            key,
            key_id,
            artifact_version,
            FittedGameHeadFields {
                question: &f.fit.question,
                lambda: f.fit.lambda,
                n_options: f.fit.n_options,
                stdizer: &f.fit.stdizer,
                weights: &f.fit.weights,
            },
        )
        .map_err(|e| format!("flappy mint refused: {e}"))?;
        report.push(MintedHead {
            name: "flappy",
            file: HEAD_VESSEL_FILES[2],
            commitment_hex: hex32(&minted.commitment),
            head_digest_hex: f.fit.digest.to_hex().to_string(),
            lambda: f.fit.lambda,
            n_options: f.fit.n_options,
        });
        std::fs::write(out_dir.join(HEAD_VESSEL_FILES[2]), &minted.bytes)
            .map_err(|e| format!("write {}: {e}", out_dir.join(HEAD_VESSEL_FILES[2]).display()))?;

        Ok(report)
    }

    fn decode_head_fitted<const F: usize, const D: usize>(
        verified: &VerifiedVessel,
    ) -> Result<FittedGameHead<F, D>, HeadVesselError> {
        let (question, lambda, n_options, stdizer, weights) = decode_head_payload::<F, D>(verified.payload())?;
        Ok(FittedGameHead {
            weights,
            stdizer,
            question,
            lambda,
            digest: head_digest_weights(&weights),
            n_options,
        })
    }

    impl GameHeads {
        /// Load the head vessels from `dir` (the `RIIR_REFLEX_HEADS_DIR`
        /// shape): `tetris.vessel` / `lanes.vessel` / `flappy.vessel`. A
        /// MISSING file leaves that lane absent (the boot says so, loud);
        /// a PRESENT file that fails verify/parse is a HARD error — a
        /// vessel that asked to be a head and failed must never be
        /// papered over. Every installed head carries the exact weights
        /// the mint signed: nothing is re-fitted.
        pub fn from_vessel_dir(dir: &Path, pins: &PinTable) -> Result<Self, HeadVesselError> {
            let load = |name: &str| -> Result<Option<VerifiedVessel>, HeadVesselError> {
                let p = dir.join(name);
                if !p.exists() {
                    return Ok(None);
                }
                vessel::open(&p, pins)
                    .map(Some)
                    .map_err(HeadVesselError::Vessel)
            };
            let tetris = load(HEAD_VESSEL_FILES[0])?;
            let lanes = load(HEAD_VESSEL_FILES[1])?;
            let flappy = load(HEAD_VESSEL_FILES[2])?;
            Ok(Self {
                tetris: tetris
                    .as_ref()
                    .map(decode_head_fitted::<TETRIS_DECODED_F, TETRIS_D>)
                    .transpose()?
                    .map(|fit| TetrisHead { grammar: tetris_spot(), fit }),
                lanes: lanes
                    .as_ref()
                    .map(decode_head_fitted::<LANES_DECODED_F, LANES_D>)
                    .transpose()?
                    .map(|fit| LanesHead { grammar: lanes_option(), fit }),
                flappy: flappy
                    .as_ref()
                    .map(decode_head_fitted::<FLAPPY_V3_DECODED_F, FLAPPY_V3_D>)
                    .transpose()?
                    .map(|fit| FlappyHead {
                        option_grammar: flappy_option_v3(),
                        state_grammar: flappy_state(),
                        fit,
                    }),
            })
        }
    }
}
