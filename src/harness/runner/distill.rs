//! riir-train Issue 576 T3 — the Arm-B TEACHER pass, generalized by
//! riir-train Plan 426 T1: class probabilities over the TRAIN rows of the
//! six Arm-A suites from a SELECTED teacher (`--distill-teacher`, default
//! `laya`), dumped as frozen data for the distillation student
//! (`../riir-train` `instinct_specialist::load_teacher_dump` /
//! `train_arm_b`).
//!
//! Teachers:
//!
//! * `laya` (default) — the G5-proven `RiirAgent::system_one` lane
//!   (feature `laya-riir`); needs the checkpoints locally.
//! * `openthai` — the OpenThai comparison lane (`src/lanes/openthai`, the
//!   Bench 074 wire; their `openthai_systemone` service over loopback
//!   HTTP). No feature needed; the loud refusal without their server is
//!   the agentjev law. The strongest measured massive teacher (0.9200,
//!   Bench 074/075) — Plan 426's V2-qualified distill source.
//! * `bekko` — the Bekko JSONL subprocess oracle (the `--bekko` lane's
//!   script, `scripts/bekko_lane.py`; hotchpotch/bekko-system-one-v0).
//!   No feature needed; the subprocess stays up for the whole suite pass
//!   over the shared [`super::JsonlOracle`] wire. The distill default is
//!   the 400M checkpoint (riir-train Issue 609 — Bench 107: above the
//!   68M on all 9 suites; Issue 608's 68M default followed Bench 103,
//!   where 17M measured negative on the owner's accuracy gate;
//!   `BEKKO_MODEL`/`BEKKO_REVISION` override, the revision default
//!   follows the model choice). License:
//!   MIT (verified 2026-10-02) — the RIDT header carries the license field
//!   on a bekko dump (the attribution law, Issue 608 T5). The measured
//!   xnli teacher (0.6767, Bench 103 — the board's largest gap).
//!
//! Early-exit mode (`harness --distill`), like `--e0`: no eval lane runs,
//! no test row is touched. Per suite:
//!
//! 1. the train envelope goes through the SAME builder the eval lane uses
//!    ([`super::load_rows`] → `stratified_split` → `spec.build`), so the
//!    rendered states/questions are byte-consistent with the arena's own
//!    laya lane;
//! 2. every case is forwarded through `RiirAgent::system_one` (the
//!    G5-proven lane);
//! 3. the answer's probabilities are mapped into the STUDENT-SIDE label
//!    universe (the distinct surviving-doc labels in SORTED order — the
//!    same law as riir-train's `label_universe`), with a per-row gold pin:
//!    the presented key at the gold position must equal the student-side
//!    label string (Name suites: massive/banking77 — direct equality;
//!    FixedInt suites: ag_news / emotion / sst5 / xnli — presented
//!    position i ↔ int label i, the builder's ClassLabel-order invariant).
//!
//! massive_intent_en presents a 20-key PER-ROW option subset; its dump
//! carries the full-universe target vector with 0 for un-presented labels
//! (true zeros: the gold is always presented, so an un-presented label is
//! genuinely not this row's class — the one-vs-all student reads per-class
//! targets, not a distribution).
//!
//! Output: `<out>/<suite>_teacher.bin` (magic `RIDT`, little-endian) +
//! a `.blake3` sidecar. Weights/probabilities are DATA on disk, never
//! committed (`.raw/` is gitignored).

use std::path::Path;

#[cfg(feature = "laya-riir")]
use super::case_questions;
use super::{RunOptions, SUITES, SuiteSpec, git_sha, iso8601_utc, load_rows, percentile_us};
use crate::harness::runner::hostname_refusing_unknown;
use crate::harness::suites::{stratified_split, train_docs};

pub const TEACHER_MAGIC: &[u8; 4] = b"RIDT";
pub const TEACHER_VERSION: u32 = 1;

/// The T3 suite set — the six Arm-A suites (riir-train Issue 576 T2).
/// Everything else registered in [`SUITES`] is a named loud skip.
const T3_SUITES: [&str; 6] = [
    "ag_news",
    "emotion",
    "sst5",
    "massive_intent_en",
    "banking77",
    "xnli_en",
];

/// How a suite's presented option keys map into the student label universe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMap {
    /// Presented keys ARE the student label strings (massive: label_text
    /// verbatim; banking77: the mteb builder's sorted stripped label_text).
    Name,
    /// Presented position i ↔ int label i (the fixed-option suites: the
    /// builder's ClassLabel-order invariant; student labels are int
    /// strings). The per-row gold pin verifies it on real data.
    FixedInt,
}

impl KeyMap {
    /// `None` = not a T3 teacher suite (named loud skip upstream).
    ///
    /// The law: the keymap must match the DOC-LABEL SPELLING the suite's
    /// `train_row_label` rule produces, not the raw dataset column —
    /// ag_news/emotion/xnli_en labels are the OPTION-KEY names (the
    /// Issue-077 content binding in suites.rs), sst5/prompt_injections
    /// fall through to int-strings. Issue 623 T1 (2026-10-11): the first
    /// pplx capture caught the mismatch — name-labeled docs under
    /// FixedInt fail the per-row gold pin on row 0, loud (the
    /// fail-closed pin did its job).
    #[must_use]
    pub fn for_suite(suite: &str) -> Option<Self> {
        match suite {
            "massive_intent_en" | "banking77" | "ag_news" | "emotion" | "emotion_syn"
            | "xnli_en" => Some(Self::Name),
            // Issue 623 (2026-10-10 owner extension): the pplx capture slot
            // rides the same FixedInt law — noul 2-class, int labels, the
            // full 2-class universe presented (the gold pin verifies it).
            // sst5: train_row_label's catch-all arm keeps the int-string
            // spelling, so FixedInt is correct here (proven by the
            // 8544-row clef capture, 2026-10-11).
            "prompt_injections" | "sst5" | "sst5_syn" => Some(Self::FixedInt),
            _ => None,
        }
    }
}

/// The corpus→student label bijection — Issue 623's seam-finding root fix
/// (2026-10-16). The two label LAWS differ per suite: reflex's corpus
/// `train_row_label` names int-ClassLabel suites by OPTION KEY (the
/// Issue-077 content binding) while the instinct student's `suite_row`
/// reads the raw dataset ints; prompt_injections's corpus is int-strings
/// while the student speaks the unified noul pair. The student's join
/// (`train_arm_b`) asserts exact `label_universe` equality, so the first
/// keyed-probs captures refused xnli/pi LOUD and the window's remap
/// script (`plan623_student_universe_remap.py`) bridged offline — the
/// repair. THIS is the law: the WRITER derives the bijection at capture
/// time and emits the dump in the student spelling.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StudentUniverse {
    /// The two label laws agree — nothing to map (massive/banking77:
    /// names both sides; sst5: int-strings both sides).
    Identity,
    /// The int-ClassLabel name-bound suites (ag_news/emotion/xnli_en):
    /// corpus label = the option NAME; student label = the dataset INT
    /// string. The bijection is `train_row_label`'s own law inverted —
    /// `KEYS[int] = name`, so a name maps to its POSITION in the
    /// presented keys (the builder's ClassLabel-order invariant; the
    /// Name gold pin proves presentation order on every row, and the
    /// student re-pins gold against its own rows at load).
    PositionalInt(Vec<String>),
    /// prompt_injections: corpus "0"/"1" vs the student's unified noul
    /// pair "no"/"yes" (gold 1 = injection = yes).
    NoulPair,
}

impl StudentUniverse {
    /// Derive the bijection for a suite. `presented_keys` = case 0's
    /// first-question criteria keys IN PRESENTATION ORDER (the
    /// ClassLabel order); only the PositionalInt arm reads it.
    pub(crate) fn derive(suite: &str, presented_keys: &[String]) -> Result<Self, String> {
        match suite {
            "ag_news" | "emotion" | "emotion_syn" | "xnli_en" => {
                if presented_keys.is_empty() || presented_keys.len() > 4096 {
                    return Err(format!(
                        "{suite}: implausible presented-key count {} for the positional bijection",
                        presented_keys.len()
                    ));
                }
                Ok(Self::PositionalInt(presented_keys.to_vec()))
            }
            "prompt_injections" => Ok(Self::NoulPair),
            _ => Ok(Self::Identity),
        }
    }

    /// One corpus label → its student spelling. Errors name the label —
    /// an absent name means the presented-key order drifted from the
    /// corpus law (the pin class), never a silent passthrough.
    pub(crate) fn translate(&self, corpus: &str) -> Result<String, String> {
        match self {
            Self::Identity => Ok(corpus.to_string()),
            Self::PositionalInt(order) => order
                .iter()
                .position(|k| k == corpus)
                .map(|i| i.to_string())
                .ok_or_else(|| {
                    format!(
                        "corpus label {corpus:?} absent from the presented-key order — \
                         presentation/corpus law drift"
                    )
                }),
            Self::NoulPair => match corpus {
                "0" => Ok("no".to_string()),
                "1" => Ok("yes".to_string()),
                other => Err(format!(
                    "noul-pair corpus label {other:?} is neither \"0\" nor \"1\""
                )),
            },
        }
    }

    /// The full class-universe mapping: translate every corpus class,
    /// verify INJECTIVITY (two corpus labels collapsing onto one student
    /// label would corrupt the join silently — refuse), sort (the
    /// student's `label_universe` law). Returns the student classes and
    /// the corpus→student INDEX permutation for row-level remaps.
    pub(crate) fn apply(
        &self,
        corpus_classes: &[String],
    ) -> Result<(Vec<String>, Vec<usize>), String> {
        let mut student: Vec<String> = Vec::with_capacity(corpus_classes.len());
        for c in corpus_classes {
            let t = self.translate(c)?;
            if student.contains(&t) {
                return Err(format!(
                    "non-injective student mapping: {t:?} twice — refusing the universe remap"
                ));
            }
            student.push(t);
        }
        let mut sorted = student.clone();
        sorted.sort();
        let perm: Vec<usize> = student
            .iter()
            .map(|s| {
                sorted
                    .iter()
                    .position(|x| x == s)
                    .expect("sorted contains every element")
            })
            .collect();
        Ok((sorted, perm))
    }
}

/// The selected distill teacher (Plan 426 T1's seam): everything the
/// per-row loop needs, constructed once per run. The forward returns
/// LABEL-KEYED probabilities — exactly what [`map_targets`] consumes —
/// plus the row's wall milliseconds. `pub(crate)`: the ensemble-gate
/// lane (Plan 426 T2) runs BOTH teachers over one shared case set
/// through this same seam — the fusion can never see teachers answered
/// by different code paths.
pub(crate) enum TeacherForward {
    /// The G5-proven local lane (feature `laya-riir`). The keyed
    /// probabilities come straight off the answer. Boxed: RiirAgent is
    /// orders of magnitude larger than the openthai variant's lane handle
    /// (the large_enum_variant law).
    #[cfg(feature = "laya-riir")]
    Laya {
        agent: Box<crate::laya::riir::RiirAgent>,
        provenance: &'static str,
    },
    /// The OpenThai loopback service (the Bench 074 wire, ungated). The
    /// keyed probabilities are built from their positional answer over
    /// the question's presented keys.
    Openthai {
        lane: crate::lanes::openthai::OpenThaiLane,
        provenance: String,
    },
    /// The Bekko JSONL subprocess oracle (riir-train Issue 608, ungated —
    /// the `--bekko` lane's script over the shared [`super::JsonlOracle`]
    /// wire). The subprocess stays up for the whole suite pass; the keyed
    /// probabilities reuse the openthai keying over the same positional
    /// answer shape (one keying law, two oracles).
    Bekko { oracle: super::JsonlOracle },
    /// The Clef loopback service (riir-train Issue 623 T1, ungated — the
    /// `--clef` lane's client over the same Jev-shaped wire). The keyed
    /// probabilities reuse the openthai keying over the positional answer
    /// (one keying law, many oracles).
    Clef {
        lane: crate::lanes::clef::ClefLane,
        provenance: String,
    },
    /// The pplx-decider loopback service (reflex Issue 082's lane as the
    /// Issue-623 teacher, ungated). Same keyed-probs tail as clef.
    Pplx {
        lane: crate::lanes::pplx::PplxLane,
        provenance: String,
    },
    /// The vLLM-SR d3 loopback service (riir-train Issue 629 T2's B3-arm
    /// teacher, ungated — Apache-2.0 at tag v3.1.0, verified 629 T0). Same
    /// SystemOne wire as pplx (their server is adapted from pplx's);
    /// same keyed-probs tail.
    D3 {
        lane: crate::lanes::d3::D3Lane,
        provenance: String,
    },
    /// The Jeva loopback service (reflex Issue 090 B2's capture rider,
    /// feeding riir-train 632 T2): OUR reference server
    /// (`scripts/jeva_server.py`) over their in-repo `jeva.Jeva`. Weights
    /// license UNDECLARED (632 T0): capture-for-MEASUREMENT allowed,
    /// adoption fail-closed — the RIDT header carries the license note
    /// (the bekko attribution law). Same keyed-probs tail as clef/pplx/d3
    /// (the capture posture is the server's default allow_null=0 — joint
    /// == conditional, no ρ dilution in the teacher targets).
    Jeva {
        lane: crate::lanes::jeva::JevaLane,
        provenance: String,
    },
    /// The StartLux-Decision loopback service (reflex Issue 095's capture
    /// rider, feeding riir-train 635 T3/T4): THEIR Apache-2.0 engine
    /// `startlux-decide` over llama.cpp on the pplx/d3 SystemOne wire.
    /// Weights CC BY-NC 4.0 — DECLARED-NC RESTRICTED (635 T0):
    /// capture-for-MEASUREMENT allowed, adoption fail-closed — the RIDT
    /// header carries the NC-lineage license token (the attribution law).
    /// Same keyed-probs tail as clef/pplx/d3/jeva.
    Startlux {
        lane: crate::lanes::startlux::StartluxLane,
        provenance: String,
    },
    /// The elephant-1 SELF-HOSTED service (riir-train Issue 640 T2(a)/T3,
    /// the capture-capable twin of the hosted mammoth lane, feeding
    /// rethink 032 T10.2's local row): OUR reference stdlib server
    /// (`scripts/elephant_server.py`) over THEIR runtime (OneJev qev
    /// @81ce62f + serve.patch — the checkpoint's block-bidirectional +
    /// slot_lm serve shape). Weights MIT (640 T0 — PERMISSIVE, the first
    /// board-top entrant with no license asterisk), runtime Apache-2.0:
    /// self-hosted local compute, no third party, so the teacher lineage
    /// stays purely self-hosted AND the capture arm is lawful here (the
    /// mammoth contrast). Same keyed-probs tail as clef/pplx/d3/jeva/
    /// startlux (the pplx parser verbatim — qev is the dialect's origin;
    /// probabilities keyed by option label, rounded 6 decimals).
    Elephant {
        lane: crate::lanes::elephant::ElephantLane,
        provenance: String,
    },
    /// The OpenDecider-nano SELF-HOSTED service (rethink Issue 038's P0
    /// ADMITTED candidate, feeding riir-train 640 T4's router column):
    /// their `opendecider serve` (pip `opendecider[serve]`, Jev-compatible
    /// `/v1/systemone`) — ~400M Ettin-encoder-400m + MLP decision head.
    /// Weights Apache-2.0 with LICENSE FILE PRESENT (the pin in 038). THE
    /// BENCH-163 DISCLOSURE RIDES THE HEADER (rule (c)): nano's choice
    /// answers are OPTION-ORDER-SENSITIVE (permuting options moves the
    /// probabilities — the [MASK]-per-option rows carry positional
    /// embeddings), so this column must never be pooled with
    /// order-invariant lanes without that disclosure; captures are taken
    /// at the shipped dataset option order (the same fixed order every
    /// lane and every board entrant reads).
    Opendecider {
        lane: crate::lanes::opendecider::OpenDeciderLane,
        provenance: String,
    },
    /// The mammoth-1 SELF-HOSTED service (riir-train Issue 640 T3b — the
    /// capture twin of the elephant arm over the SAME OneJev qev runtime
    /// @81ce62f + serve.patch, OUR operator serve on loopback): MPS bf16
    /// 27B (≈ 54 GB weights, ~44 GB resident with the serve's MPS cache
    /// hygiene). Weights MIT (640 T0, capture-time re-verified — the
    /// SECOND permissive board-top teacher), runtime Apache-2.0, so the
    /// teacher lineage stays purely self-hosted AND the capture arm is
    /// lawful here (the NC-lineage contrast). The block-bidirectional
    /// mask arms from the checkpoint's own modeling_overrides via
    /// qev.readout (the window's bootcheck reads the `attention` field
    /// off the serve log — a causal serve reads as the wrong model, the
    /// 635 −26 pts class). Same keyed-probs tail as the elephant arm (the
    /// pplx parser verbatim — qev is the dialect's origin).
    Mammoth {
        lane: crate::lanes::mammoth::MammothLane,
        provenance: String,
    },
}

impl TeacherForward {
    /// The teacher's name — the header/record provenance token.
    pub(crate) fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { .. } => "laya",
            Self::Openthai { .. } => "openthai",
            Self::Bekko { .. } => "bekko",
            Self::Clef { .. } => "clef",
            Self::Pplx { .. } => "pplx",
            Self::D3 { .. } => "d3",
            Self::Jeva { .. } => "jeva",
            Self::Startlux { .. } => "startlux",
            Self::Elephant { .. } => "elephant",
            Self::Opendecider { .. } => "opendecider",
            Self::Mammoth { .. } => "mammoth",
        }
    }

    /// Checkpoint (laya), model id (openthai/bekko) — the record's
    /// `checkpoint` field, the teacher-side provenance.
    pub(crate) fn provenance(&self) -> &str {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { provenance, .. } => provenance,
            Self::Openthai { provenance, .. } => provenance,
            Self::Bekko { oracle } => &oracle.model,
            Self::Clef { provenance, .. } | Self::Pplx { provenance, .. } => provenance,
            Self::D3 { provenance, .. } | Self::Jeva { provenance, .. } => provenance,
            Self::Startlux { provenance, .. } => provenance,
            Self::Elephant { provenance, .. } => provenance,
            Self::Opendecider { provenance, .. } => provenance,
            Self::Mammoth { provenance, .. } => provenance,
        }
    }

    /// The license token the RIDT header carries when the teacher is
    /// EXTERNAL (the attribution law, Issue 608 T5 — the artifact names
    /// its teacher's license wherever bekko outputs are recorded; ours
    /// and the Apache-2.0-served openthai read as None here).
    pub(crate) fn license(&self) -> Option<&'static str> {
        match self {
            Self::Bekko { .. } => Some(
                "MIT (hotchpotch/bekko-system-one — Copyright (c) 2026 Yuichi \
                      Tatsumi; verified 2026-10-02)",
            ),
            Self::Jeva { .. } => Some(
                "UNDECLARED (LIQIIIII/Jeva-4B weights; code Apache-2.0 \
                      LiQiiiii/Jeva) — capture-for-measurement posture per \
                      riir-train 632 T0; adoption fail-closed",
            ),
            Self::Startlux { .. } => Some(crate::lanes::startlux::LICENSE_LINE),
            Self::Elephant { .. } => Some(crate::lanes::elephant::LICENSE_LINE),
            Self::Opendecider { .. } => Some(
                "Apache-2.0 (OpenDecider-nano; LICENSE file present, pin in \
                      rethink 038) — OPTION-ORDER-SENSITIVE choice answers \
                      (bench 163 rule (c)): captured at the shipped dataset \
                      option order; never pool with order-invariant lanes \
                      without this disclosure",
            ),
            Self::Mammoth { .. } => Some(
                "MIT (baselabs/mammoth-1; base Qwen3.5 Apache-2.0; runtime \
                      OneJev Apache-2.0) — T0-verified 2026-10-24 + \
                      capture-time re-verified 2026-10-28 (sha 61cec44c \
                      unchanged) per riir-train 640; no LICENSE file ships \
                      (tag+front-matter assertion)",
            ),
            _ => None,
        }
    }

    /// One discarded forward before the loop, so the latency stat is not
    /// cold-skewed (laya's pipeline-compile pre-ramp; openthai's server
    /// cold path) — and, for openthai, the final proof their server is
    /// actually answering plus the model-id provenance capture (the
    /// agentjev law: refuse loud, never half-run).
    pub(crate) fn warmup(&mut self, case: &super::SuiteCase) -> Result<(), String> {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { agent, .. } => agent
                .system_one(&case.state, &case_questions(case))
                .map(|_| ())
                .map_err(|e| format!("laya warmup: {e}")),
            Self::Openthai { lane, provenance } => {
                lane.health()?;
                let raw = lane
                    .decide_raw(case)
                    .map_err(|e| format!("openthai warmup: {e}"))?;
                let parsed: serde_json::Value = serde_json::from_str(&raw)
                    .map_err(|e| format!("openthai warmup parse: {e}"))?;
                *provenance = format!(
                    "openthai:{}",
                    crate::lanes::openthai::OpenThaiLane::model_of(&parsed)
                );
                Ok(())
            }
            // One discarded forward post-handshake: the loud proof the
            // oracle ANSWERS (the handshake only proves it loaded), and
            // the latency stat is not cold-skewed (the seam's law).
            Self::Bekko { oracle } => oracle
                .ask(case)
                .map(|_| ())
                .map_err(|e| format!("bekko warmup: {e}")),
            // The clef handshake (info) IS a full probe round trip over the
            // wire — it fills the model provenance (the envelope's model
            // field, else the CLEF_MODEL stamp); then one discarded forward
            // on the real case shape (the seam's law).
            Self::Clef { lane, provenance } => {
                let (model, _posture) = lane.info().map_err(|e| format!("clef warmup: {e}"))?;
                *provenance = format!("clef:{model}");
                lane.decide(case)
                    .map(|_| ())
                    .map_err(|e| format!("clef warmup: {e}"))
            }
            // pplx's health ran at construct (it fills the provenance);
            // warmup is the loud proof + the cold-path absorb.
            Self::Pplx { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("pplx warmup: {e}")),
            // d3's health ran at construct (the pplx law); warmup is the
            // loud proof + the cold-path absorb.
            Self::D3 { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("d3 warmup: {e}")),
            // The jeva law: health at construct filled the provenance;
            // warmup is the loud proof + the cold-path absorb (their
            // hybrid option cache warms on the first pass).
            Self::Jeva { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("jeva warmup: {e}")),
            // The startlux law: health + the bidirectional posture gate
            // ran at construct (it fills the provenance); warmup is the
            // loud proof + the cold-path absorb (llama-server's first
            // non-causal eval / page-in).
            Self::Startlux { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("startlux warmup: {e}")),
            // The elephant law: health + the attention-posture provenance
            // ran at construct; warmup is the loud proof + the cold-path
            // absorb (their engine's first prefix fork page-ins the KV
            // scratch on mps).
            Self::Elephant { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("elephant warmup: {e}")),
            // The elephant law verbatim: the loud proof + the cold-path
            // absorb (the 27B's first prefix fork page-ins the KV scratch
            // on mps — a bigger absorb than the 9B, same shape).
            Self::Mammoth { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("mammoth warmup: {e}")),
            // The opendecider law: the ONE-probe handshake ran at construct
            // (the clef law; it fills the provenance); warmup is the loud
            // proof + the cold-path absorb (their first encode pages the
            // Ettin backbone).
            Self::Opendecider { lane, .. } => lane
                .decide(case)
                .map(|_| ())
                .map_err(|e| format!("opendecider warmup: {e}")),
        }
    }

    /// One row's keyed probabilities + wall ms. The FIRST question's
    /// answer (the distill join is one gold per row — the single-question
    /// contract the T3 suites speak).
    pub(crate) fn forward(
        &mut self,
        case: &super::SuiteCase,
    ) -> Result<(Vec<(String, f64)>, u64), String> {
        match self {
            #[cfg(feature = "laya-riir")]
            Self::Laya { agent, .. } => {
                let questions = case_questions(case);
                let t0 = std::time::Instant::now();
                let answers = agent
                    .system_one(&case.state, &questions)
                    .map_err(|e| format!("laya forward ({}): {e}", case.id))?;
                let ms = t0.elapsed().as_millis() as u64;
                let a = answers
                    .first()
                    .ok_or_else(|| format!("{}: no answer for its single question", case.id))?;
                Ok((a.probabilities.clone(), ms))
            }
            Self::Openthai { lane, .. } => {
                let (outcome, client_ms) = lane
                    .decide(case)
                    .map_err(|e| format!("openthai forward ({}): {e}", case.id))?;
                let (probs, _pick, _conf) = outcome
                    .answers
                    .first()
                    .ok_or_else(|| format!("{}: no answer for its single question", case.id))?;
                let q = case
                    .questions
                    .first()
                    .ok_or_else(|| format!("{}: no questions for its single gold", case.id))?;
                let keyed =
                    openthai_keyed_probs(q, probs).map_err(|e| format!("{}: {e}", case.id))?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            // The distill join is ONE gold per row (the single-question
            // contract) — the FIRST question's answer, positional p array
            // keyed by the SAME law openthai speaks (one keying, two
            // oracles — [`openthai_keyed_probs`] is the shared home).
            Self::Bekko { oracle } => {
                let t0 = std::time::Instant::now();
                let raw = oracle
                    .ask(case)
                    .map_err(|e| format!("bekko forward ({}): {e}", case.id))?;
                let ms = t0.elapsed().as_millis() as u64;
                let resp: serde_json::Value = serde_json::from_str(raw.trim())
                    .map_err(|e| format!("bekko forward ({}): parse: {e}", case.id))?;
                let q = case
                    .questions
                    .first()
                    .ok_or_else(|| format!("{}: no questions for its single gold", case.id))?;
                let a = resp
                    .get("answers")
                    .and_then(|v| v.as_object())
                    .and_then(|m| m.get(&q.qid))
                    .ok_or_else(|| format!("{}: no bekko answer for qid {}", case.id, q.qid))?;
                let (probs, _pick, _conf) =
                    super::parse_python_answer(q, a).map_err(|e| format!("{}: {e}", case.id))?;
                let keyed =
                    openthai_keyed_probs(q, &probs).map_err(|e| format!("{}: {e}", case.id))?;
                Ok((keyed, ms))
            }
            Self::Clef { lane, .. } => {
                let (answers, client_ms, _server_wall) = lane
                    .decide(case)
                    .map_err(|e| format!("clef forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "clef")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::Pplx { lane, .. } => {
                let (answers, client_ms, _tokens) = lane
                    .decide(case)
                    .map_err(|e| format!("pplx forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "pplx")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::D3 { lane, .. } => {
                let (answers, client_ms, _tokens) = lane
                    .decide(case)
                    .map_err(|e| format!("d3 forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "d3")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::Jeva { lane, .. } => {
                let (outcome, client_ms) = lane
                    .decide(case)
                    .map_err(|e| format!("jeva forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &outcome.answers, "jeva")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::Startlux { lane, .. } => {
                let (answers, client_ms, _tokens) = lane
                    .decide(case)
                    .map_err(|e| format!("startlux forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "startlux")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::Elephant { lane, .. } => {
                let (answers, client_ms, _tokens) = lane
                    .decide(case)
                    .map_err(|e| format!("elephant forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "elephant")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::Mammoth { lane, .. } => {
                let (answers, client_ms, _tokens) = lane
                    .decide(case)
                    .map_err(|e| format!("mammoth forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "mammoth")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
            Self::Opendecider { lane, .. } => {
                let (answers, client_ms, _tokens) = lane
                    .decide(case)
                    .map_err(|e| format!("opendecider forward ({}): {e}", case.id))?;
                let keyed = keyed_first_answer(case, &answers, "opendecider")?;
                let ms = u64::try_from(client_ms.round() as u128).unwrap_or(u64::MAX);
                Ok((keyed, ms))
            }
        }
    }
}

/// The shared keyed-probs tail for the HTTP decision teachers (clef/pplx):
/// the lane's FIRST answer (the single-question contract the T3 suites
/// speak) re-keyed over the question's presented keys — the same law
/// openthai speaks ([`openthai_keyed_probs`] is the shared home; one
/// keying, many oracles).
fn keyed_first_answer(
    case: &super::SuiteCase,
    answers: &[(Vec<f64>, usize, f64)],
    teacher: &str,
) -> Result<Vec<(String, f64)>, String> {
    let (probs, _pick, _conf) = answers
        .first()
        .ok_or_else(|| format!("{}: no answer for its single question", case.id))?;
    let q = case
        .questions
        .first()
        .ok_or_else(|| format!("{}: no questions for its single gold", case.id))?;
    openthai_keyed_probs(q, probs).map_err(|e| format!("{teacher} forward ({}): {e}", case.id))
}
/// Build the label-keyed probability vector one openthai answer speaks,
/// over the question's PRESENTED keys in presentation order (the same
/// iteration [`crate::lanes::openthai::map_answers`] reads positionally):
/// choice → the criteria object's keys; score → the level indices
/// ("0".."k-1", the FixedInt mapping's own spelling); noul → their
/// p(yes) reads as [p_no, p_yes] over the pair, keyed "0"/"1" (the gold
/// idx space). The keys are exactly what [`map_targets`] matches on:
/// Name suites match them to the student labels, FixedInt suites ignore
/// the key string and read the position.
pub(crate) fn openthai_keyed_probs(
    q: &crate::harness::suites::SuiteQuestion,
    probs: &[f64],
) -> Result<Vec<(String, f64)>, String> {
    use crate::harness::suites::QKind;
    let keys: Vec<String> = match q.kind {
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
    };
    if keys.len() != probs.len() {
        return Err(format!(
            "their answer carries {} probabilities vs {} presented keys — wire drift",
            probs.len(),
            keys.len()
        ));
    }
    Ok(keys.into_iter().zip(probs.iter().copied()).collect())
}

/// The bekko distill teacher's model pins (riir-train Issue 609): the
/// 400M checkpoint by default — Bench 107 measured it ABOVE the 68M on
/// all 9 board suites (the family sign flipped: +2.6 pt overall vs the
/// 68M's −6.1). History: Issue 608 defaulted 68M (17M measured NEGATIVE
/// on the owner's accuracy gate; 68M was Bench 103's measured teacher)
/// — that closure was AT the 68M; the 400M re-opens the distill question
/// with better priors, same gates. `BEKKO_MODEL`/`BEKKO_REVISION`
/// overrides are respected; the REVISION default FOLLOWS the model
/// choice (each release has its own; the lane script's own revision
/// default pins the 17M release — loading a known model without its
/// revision would drift, and a FOREIGN model id takes the script's own
/// default). Pure over an env lookup so the default matrix is testable.
fn bekko_teacher_pins_with(get: impl Fn(&str) -> Option<String>) -> (String, String) {
    const M400: &str = "hotchpotch/bekko-system-one-v0-400m";
    const M68: &str = "hotchpotch/bekko-system-one-v0-68m";
    const REV_400M: &str = "4aeb85b9d4042d75d8b8adf6ff7ba9e4629510ba";
    const REV_68M: &str = "6eb1bae2d35066b0d634fabaf8c79beafc6fd9f1";
    const REV_17M: &str = "2147c3d9d00559bf972616c2589eee967e266f3b";
    let model = get("BEKKO_MODEL").unwrap_or_else(|| M400.to_string());
    let revision = get("BEKKO_REVISION").unwrap_or_else(|| match model.as_str() {
        M400 => REV_400M.to_string(),
        M68 => REV_68M.to_string(),
        _ => REV_17M.to_string(),
    });
    (model, revision)
}

fn bekko_teacher_pins() -> (String, String) {
    bekko_teacher_pins_with(|k| std::env::var(k).ok())
}

/// Construct the selected teacher for one distill run (Plan 426 T1).
/// `laya` needs the checkpoints + the `laya-riir` feature; `openthai`
/// health-checks their loopback service here — a server that is down is
/// a LOUD refusal naming the env (the agentjev law), never a half-run.
/// `bekko` spawns the JSONL oracle subprocess (the handshake IS the loud
/// proof the model loaded) at the 68M pins.
pub(crate) fn construct_teacher(name: &str, spec: &SuiteSpec) -> Result<TeacherForward, String> {
    match name {
        "openthai" => {
            let lane = crate::lanes::openthai::OpenThaiLane::default();
            lane.health()?;
            Ok(TeacherForward::Openthai {
                lane,
                // Filled by the warmup's decide (the model id is read
                // off their response — the lane's own law).
                provenance: "openthai:?".to_string(),
            })
        }
        "bekko" => {
            let (model, revision) = bekko_teacher_pins();
            let oracle = super::JsonlOracle::spawn(
                "bekko",
                "BEKKO_LANE_SCRIPT",
                "scripts/bekko_lane.py",
                "BEKKO_PYTHON",
                "BEKKO_PY_DEVICE",
                "cpu",
                "the venv needs torch + transformers + sentence-transformers \
                 (the card's runtime pins; MIT license — verified 2026-10-02)",
                &[
                    ("BEKKO_MODEL", model.as_str()),
                    ("BEKKO_REVISION", revision.as_str()),
                ],
            )?;
            Ok(TeacherForward::Bekko { oracle })
        }
        "clef" => {
            // Env-only at construct (the no-creds gate is the loud one —
            // for the LOCAL rig posture set CLEF_RUN_PATH; the hosted
            // posture keeps its plan-011 account law). The wire is proven
            // + the model provenance filled at warmup.
            let lane = crate::lanes::clef::ClefLane::from_env()?;
            Ok(TeacherForward::Clef {
                lane,
                provenance: "clef:?".to_string(),
            })
        }
        "pplx" => {
            // Health at construct (the openthai law — a server that is
            // down, or still lazy-loading, refuses LOUD here naming the
            // env), and the provenance comes straight off /health.
            let lane = crate::lanes::pplx::PplxLane::default();
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::Pplx { lane, provenance })
        }
        "d3" => {
            // The pplx law verbatim: health at construct, provenance off
            // /health (serve version-true with --name d3-v3.1.0 — their
            // health carries no checkpoint field).
            let lane = crate::lanes::d3::D3Lane::default();
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::D3 { lane, provenance })
        }
        "jeva" => {
            // The pplx law verbatim: health at construct, provenance off
            // /health (model + resolved dtype + allow_null posture). The
            // capture posture is the server's allow_null=0 default (joint
            // == conditional — no ρ dilution in the teacher targets).
            let lane = crate::lanes::jeva::JevaLane::default();
            lane.health()?;
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::Jeva { lane, provenance })
        }
        "startlux" => {
            // The pplx law verbatim, plus the lane's POSTURE gate: health
            // at construct (kind instant + prompt_attention bidirectional
            // + engine.bidir_mode server — a causal/off serve misreads the
            // Instant checkpoint), provenance off /health (their --name,
            // the folder name by default). The quant posture is the
            // STARTLUX_DEVICE stamp — the lossy-surface law (our 635-T2
            // arm: Q4_K_M, -ngl 99).
            let lane = crate::lanes::startlux::StartluxLane::default();
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::Startlux { lane, provenance })
        }
        "elephant" => {
            // The pplx law verbatim, plus the lane's POSTURE disclosure:
            // health at construct, provenance off /health — the resolved
            // attention posture rides it (attention=noncausal is the
            // checkpoint's trained shape; a causal serve reads as the
            // wrong model, the 635 −26 pts class — the provenance makes
            // the drift visible in every record).
            let lane = crate::lanes::elephant::ElephantLane::default();
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::Elephant { lane, provenance })
        }
        "mammoth" => {
            // The elephant law verbatim (riir-train 640 T3b), with the
            // lane's no-creds gate: from_env() refuses LOUD without the
            // token or the explicit MAMMOTH_FORWARDER_AUTH=1 ack (the
            // local rig posture — loopback serve, no credential). The
            // handshake probe at construct proves the wire; the window's
            // bootcheck reads the attention posture off the serve log
            // (qev logs `attention` at engine construction —
            // block_bidirectional is the checkpoint's trained shape; a
            // causal serve reads as the wrong model, the 635 −26 pts
            // class) and records it beside every artifact.
            let lane = crate::lanes::mammoth::MammothLane::from_env()?;
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::Mammoth { lane, provenance })
        }
        "opendecider" => {
            // The clef law: the ONE-probe handshake (info) at construct
            // fills the provenance off their health; the capture runs at
            // the shipped dataset option order — the bench-163
            // order-sensitivity disclosure rides the RIDT header (the
            // license() token above).
            let lane = crate::lanes::opendecider::OpenDeciderLane::from_env()?;
            let (provenance, _device) = lane.info()?;
            Ok(TeacherForward::Opendecider { lane, provenance })
        }
        "laya" => {
            #[cfg(feature = "laya-riir")]
            {
                let ckpt = super::laya_checkpoints_for(spec.name)
                    .first()
                    .copied()
                    .ok_or_else(|| format!("{}: no laya checkpoint", spec.name))?;
                let ck = match ckpt {
                    "english" => crate::laya::config::Checkpoint::English,
                    "multilingual" => crate::laya::config::Checkpoint::Multilingual,
                    "typed" => crate::laya::config::Checkpoint::TypedDecisions,
                    other => return Err(format!("unknown checkpoint {other}")),
                };
                let agent = Box::new(super::load_laya_agent(ckpt, ck)?);
                Ok(TeacherForward::Laya {
                    agent,
                    provenance: ckpt,
                })
            }
            #[cfg(not(feature = "laya-riir"))]
            {
                let _ = spec;
                Err(
                    "--distill-teacher laya needs the `laya-riir` feature — rebuild: cargo \
                     build --release --features laya-riir --bin harness (macOS: \
                     laya-riir-metal)"
                        .to_string(),
                )
            }
        }
        other => Err(format!(
            "unknown --distill-teacher {other:?} (the seam knows 'laya', 'openthai', \
             'bekko', 'clef', 'pplx', 'd3', 'jeva', 'startlux', 'elephant', \
             'opendecider' and 'mammoth')"
        )),
    }
}

fn t3_skip_reason(suite: &str) -> &'static str {
    match suite {
        "typed_decisions" => {
            "not a T3 teacher suite — Bench 051 measured its NB tables never arm \
             (options are state-field values, route terms never activate); \
             distillation there waits for the option-conditioned scorer \
             (riir-instinct Issue 005 E0 note)"
        }
        _ => {
            "not a T3 teacher suite (no train-row join: the synthetic family \
              generates in-process; its students need the synthetic-suite rig first)"
        }
    }
}

/// The distill teacher's row-backed suite set: the six Arm-A suites (576
/// T2) plus `prompt_injections` (Issue 623, 2026-10-10 owner extension —
/// the pplx capture slot; noul 2-class joins the FixedInt law). The
/// synthetic family stays out: it has no train rows for the doc/case
/// join, and its students need the synthetic-suite rig first (T2 scope,
/// riir-train Issue 623).
fn is_distill_suite(name: &str) -> bool {
    T3_SUITES.contains(&name) || EXTENSION_SUITES.contains(&name)
}

/// The row-backed suites beyond the Arm-A six: `prompt_injections` (Issue
/// 623) and the first-party synthetic pair (rethink Research 002 — the ESC
/// re-source; reflex Plan 013 T6 hands their teacher captures to the
/// riir-train re-fit). Each rides its parent's keymap law.
const EXTENSION_SUITES: [&str; 3] = ["prompt_injections", "emotion_syn", "sst5_syn"];

#[derive(Debug, serde::Serialize)]
pub struct DistillSuite {
    pub name: String,
    /// The teacher-side provenance: the laya checkpoint name, or the
    /// openthai model id read off their live response (Plan 426 T1).
    pub checkpoint: String,
    pub n_rows: usize,
    pub n_classes: usize,
    /// Teacher argmax vs gold over the train rows — a DATA-QUALITY sanity
    /// stat for the distillation join, never a published lane number (the
    /// G5 gate governs those).
    pub teacher_accuracy: f64,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub seconds: f64,
    pub path: String,
    /// First 16 hex of the file's BLAKE3 (the sidecar carries the full one).
    pub blake3: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DistillMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub host: String,
    pub device: String,
    pub datasets_dir: String,
    pub limit: usize,
    pub row_rule: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct DistillOutput {
    pub meta: DistillMeta,
    pub suites: Vec<DistillSuite>,
    pub skipped: Vec<String>,
}

/// Argmax over presented (key, prob) pairs, ties to the LOWEST position
/// (the engine argmax law). `pub(crate)`: the synth lane's veto (Plan 426
/// T5) accepts a candidate iff the teacher's argmax key equals the gold
/// label — the same tie law.
pub(crate) fn argmax_lowest_pos(probabilities: &[(String, f64)]) -> usize {
    let mut best = 0usize;
    for (i, (_, p)) in probabilities.iter().enumerate().skip(1) {
        if *p > probabilities[best].1 {
            best = i;
        }
    }
    best
}

/// Map one answer's presented probabilities into the student universe.
/// Returns (gold class, teacher-pick class, q over `classes`).
fn map_targets(
    keymap: KeyMap,
    doc_label: &str,
    gold_pos: usize,
    probabilities: &[(String, f64)],
    classes: &[String],
) -> Result<(usize, usize, Vec<f32>), String> {
    let class_of = |s: &str| {
        classes
            .iter()
            .position(|c| c == s)
            .ok_or_else(|| format!("class {s:?} absent"))
    };
    let mut q = vec![0.0f32; classes.len()];
    let pick_pos = argmax_lowest_pos(probabilities);
    match keymap {
        KeyMap::FixedInt => {
            if probabilities.len() != classes.len() {
                return Err(format!(
                    "presented {} option(s) vs {} class(es) — fixed suites present the \
                     full universe",
                    probabilities.len(),
                    classes.len()
                ));
            }
            if doc_label != gold_pos.to_string() {
                return Err(format!(
                    "gold pin failed: doc label {doc_label:?} != presented gold position \
                     {gold_pos} — builder/dataset order drift"
                ));
            }
            for (i, (_, p)) in probabilities.iter().enumerate() {
                let ci = class_of(&i.to_string())?;
                q[ci] = *p as f32;
            }
            let pick = class_of(&pick_pos.to_string())?;
            Ok((class_of(doc_label)?, pick, q))
        }
        KeyMap::Name => {
            for (k, p) in probabilities {
                let ci = class_of(k)?;
                q[ci] = *p as f32;
            }
            let gold_key = probabilities
                .get(gold_pos)
                .map(|(k, _)| k.as_str())
                .ok_or_else(|| format!("gold position {gold_pos} out of range"))?;
            if gold_key != doc_label {
                return Err(format!(
                    "gold pin failed: presented gold key {gold_key:?} != doc label \
                     {doc_label:?} — join drift"
                ));
            }
            let pick = class_of(&probabilities[pick_pos].0)?;
            Ok((class_of(doc_label)?, pick, q))
        }
    }
}

/// The RIDT bin writer, shared by the distill lane and the T2.5 teacher
/// router (riir-train Issue 629 T2.5) — ONE byte layout, two producers.
/// Layout: `RIDT` magic · u32 version · u32 header_len · header JSON ·
/// per row (gold u32, pick u32, n_classes × f32), all little-endian.
pub(crate) struct TeacherBinWriter {
    bytes: Vec<u8>,
}

impl TeacherBinWriter {
    /// Serializes `header` once; `n_classes` is implied by the header's
    /// `classes` array on read.
    pub(crate) fn new(header: &serde_json::Value) -> Result<Self, String> {
        let header_bytes = serde_json::to_string(header)
            .map_err(|e| format!("header json: {e}"))?
            .into_bytes();
        let mut bytes = Vec::with_capacity(1 << 20);
        bytes.extend_from_slice(TEACHER_MAGIC);
        bytes.extend_from_slice(&TEACHER_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header_bytes);
        Ok(Self { bytes })
    }

    pub(crate) fn push_row(&mut self, gold_c: u32, pick_c: u32, q: &[f32]) {
        self.bytes.extend_from_slice(&gold_c.to_le_bytes());
        self.bytes.extend_from_slice(&pick_c.to_le_bytes());
        for v in q {
            self.bytes.extend_from_slice(&v.to_le_bytes());
        }
    }

    /// The finished artifact bytes (magic-first; blake3 + write at the call
    /// site so every producer shares the sidecar law too).
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// A captured teacher column, read back for routing (Issue 629 T2.5): the
/// header plus the row-aligned gold/pick/q triplets. The per-row gold pins
/// double as the CROSS-CAPTURE ALIGNMENT witness — two captures over the
/// same datasets dir must agree on every row's gold or the join is refused.
pub(crate) struct TeacherColumn {
    pub header: serde_json::Value,
    pub gold: Vec<u32>,
    pub pick: Vec<u32>,
    pub q: Vec<Vec<f32>>,
}

/// Read one `<suite>_teacher.bin` (the RIDT format `TeacherBinWriter`
/// emits). Fails loud on magic/version/truncation/shape drift.
pub(crate) fn read_teacher_bin(bytes: &[u8]) -> Result<TeacherColumn, String> {
    let fail = |why: &str| format!("RIDT read: {why}");
    if bytes.len() < 12 || &bytes[..4] != TEACHER_MAGIC {
        return Err(fail("bad magic (not a RIDT teacher bin)"));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| fail("version bytes"))?);
    if version != TEACHER_VERSION {
        return Err(fail(&format!("unsupported version {version}")));
    }
    let header_len = u32::from_le_bytes(
        bytes[8..12]
            .try_into()
            .map_err(|_| fail("header-len bytes"))?,
    ) as usize;
    let header_end = 12usize
        .checked_add(header_len)
        .ok_or_else(|| fail("header len overflow"))?;
    if bytes.len() < header_end {
        return Err(fail("truncated header"));
    }
    let header: serde_json::Value = serde_json::from_slice(&bytes[12..header_end])
        .map_err(|e| fail(&format!("header json: {e}")))?;
    let classes = header
        .get("classes")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| fail("header carries no classes array"))?;
    let n_classes = classes.len();
    if n_classes == 0 || n_classes > 4096 {
        return Err(fail(&format!("implausible class count {n_classes}")));
    }
    let n_rows = header
        .get("n_rows")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| fail("header carries no n_rows"))? as usize;
    let row_len = 8 + n_classes * 4;
    let body = &bytes[header_end..];
    if body.len()
        != n_rows
            .checked_mul(row_len)
            .ok_or_else(|| fail("row-count overflow"))?
    {
        return Err(fail(&format!(
            "body is {} byte(s), header says {n_rows} row(s) × {row_len} = expected {}",
            body.len(),
            n_rows * row_len
        )));
    }
    let mut gold = Vec::with_capacity(n_rows);
    let mut pick = Vec::with_capacity(n_rows);
    let mut q = Vec::with_capacity(n_rows);
    for r in 0..n_rows {
        let base = header_end + r * row_len;
        let u = |off: usize| -> u32 {
            u32::from_le_bytes(
                bytes[base + off..base + off + 4]
                    .try_into()
                    .expect("row slice"),
            )
        };
        let g = u(0) as usize;
        let p = u(4) as usize;
        if g >= n_classes || p >= n_classes {
            return Err(fail(&format!(
                "row {r}: class index out of range (gold {g}, pick {p})"
            )));
        }
        let mut row_q = Vec::with_capacity(n_classes);
        for c in 0..n_classes {
            let off = 8 + c * 4;
            row_q.push(f32::from_le_bytes(
                bytes[base + off..base + off + 4]
                    .try_into()
                    .expect("q slice"),
            ));
        }
        gold.push(g as u32);
        pick.push(p as u32);
        q.push(row_q);
    }
    Ok(TeacherColumn {
        header,
        gold,
        pick,
        q,
    })
}

/// Distill one suite's train rows through the selected teacher
/// (Plan 426 T1: `--distill-teacher`, default laya — the 576 T3 posture).
fn distill_suite(
    spec: &SuiteSpec,
    dir: &Path,
    out_dir: &Path,
    limit: usize,
    teacher_name: &str,
) -> Result<DistillSuite, String> {
    let t_start = std::time::Instant::now();
    let suite_dir = dir.join(spec.name);
    let train_rows = load_rows(&suite_dir, "train")?;
    // `limit` = a label-stratified cap (validation runs); 0 = the whole
    // split (identity split, the T3 protocol posture).
    let split = stratified_split(&train_rows, spec.name, limit);
    let docs = train_docs(&split.front, spec.name);
    let built = (spec.build)(&split.front, 0);
    if docs.len() != built.cases.len() {
        return Err(format!(
            "row/case join broke: {} doc(s) vs {} case(s) — survivor predicate drift",
            docs.len(),
            built.cases.len()
        ));
    }
    if docs.is_empty() {
        return Err("no train rows survived the label rule".into());
    }
    // Student-side label universe: the distinct surviving-doc labels in
    // SORTED order — riir-train's `label_universe` law ("stable domain
    // order; the artifact carries the names"), which the student asserts
    // as exact equality (order included).
    let mut classes: Vec<String> = Vec::with_capacity(16);
    for d in &docs {
        if !classes.iter().any(|c| c == &d.label) {
            classes.push(d.label.clone());
        }
    }
    classes.sort();
    let n_classes = classes.len();

    let keymap = KeyMap::for_suite(spec.name)
        .ok_or_else(|| format!("{}: no key mapping (not a T3 suite)", spec.name))?;
    // The 623 root-cause law (see [`StudentUniverse`]): emit the dump in
    // the STUDENT label spelling, derived at capture time. The corpus
    // classes stay the pinning space (map_targets runs unchanged); the
    // student classes + index permutation ride every row out.
    let presented_keys: Vec<String> = built
        .cases
        .first()
        .and_then(|c| c.questions.first())
        .and_then(|q| q.criteria.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let student_universe = StudentUniverse::derive(spec.name, &presented_keys)?;
    let (student_classes, student_perm) = student_universe.apply(&classes)?;
    if student_universe == StudentUniverse::Identity {
        eprintln!(
            "  [distill {}] student spelling = corpus (identity map)",
            spec.name
        );
    } else {
        eprintln!(
            "  [distill {}] student spelling remap ACTIVE: {} corpus classes → {} student classes",
            spec.name,
            classes.len(),
            student_classes.len()
        );
    }
    let mut teacher = construct_teacher(teacher_name, spec)?;

    let mut header = serde_json::json!({
        "suite": spec.name,
        "teacher": teacher.name(),
        "checkpoint": teacher.provenance(),
        "git_sha": git_sha(),
        "datasets_dir": dir.display().to_string(),
        "row_rule": "student rows = reflex train_docs over the sorted train-*.json pages; \
                     case i ↔ doc i (same survivor predicate, same order); the student \
                     re-verifies per-row gold labels against its own rows",
        "classes": student_classes,
        "classes_spelling": if student_universe == StudentUniverse::Identity {
            serde_json::json!("corpus (identity)")
        } else {
            let mut map = serde_json::Map::new();
            for (i, c) in classes.iter().enumerate() {
                map.insert(
                    c.clone(),
                    serde_json::Value::String(student_classes[student_perm[i]].clone()),
                );
            }
            serde_json::json!({
                "law": "student universe emitted at capture time (the 623 seam-finding fix)",
                "map": serde_json::Value::Object(map),
            })
        },
        "n_rows": docs.len(),
        "limit": limit,
        "created_utc": iso8601_utc(),
    });
    // The attribution law (Issue 608 T5): an EXTERNAL teacher's license
    // rides the artifact header wherever its outputs are recorded (the
    // student reader tolerates extra header fields — the RidtWriter
    // header_extra precedent).
    if let Some(license) = teacher.license() {
        header["license"] = serde_json::Value::String(license.to_string());
    }
    let mut out_writer = TeacherBinWriter::new(&header)?;

    // One discarded warmup forward: compiles the laya Metal pipelines /
    // warms their server's cold path so the per-row latency stat is not
    // cold-skewed (the run_laya_checkpoint pre-ramp law, in the data-pass
    // posture). For openthai this is also the model-id capture + the
    // final loud proof their server answers.
    if let Some(case) = built.cases.first() {
        teacher.warmup(case)?;
        eprintln!(
            "  [distill {}] warmup done ({}/{})",
            spec.name,
            teacher.name(),
            teacher.provenance()
        );
    }

    let mut durs_ms: Vec<u64> = Vec::with_capacity(docs.len());
    let mut hits = 0usize;
    for (doc, case) in docs.iter().zip(built.cases.iter()) {
        let t0 = std::time::Instant::now();
        let (probabilities, ms) = teacher.forward(case)?;
        durs_ms.push(if ms > 0 {
            ms
        } else {
            t0.elapsed().as_millis() as u64
        });
        // Pinning happens in CORPUS spelling (unchanged law); the row then
        // rides the student permutation out (the 623 capture-time law).
        let (gold_c, pick_c, q) = map_targets(
            keymap,
            &doc.label,
            case.gold[0].idx,
            &probabilities,
            &classes,
        )
        .map_err(|e| format!("{}: {e}", case.id))?;
        let gold_c = student_perm[gold_c];
        let pick_c = student_perm[pick_c];
        let mut q = q;
        let mut sq = vec![0.0f32; q.len()];
        for (ci, p) in q.iter_mut().enumerate() {
            sq[student_perm[ci]] = *p;
        }
        std::mem::swap(&mut q, &mut sq);
        hits += usize::from(pick_c == gold_c);
        out_writer.push_row(gold_c as u32, pick_c as u32, &q);
        if durs_ms.len().is_multiple_of(2000) {
            eprintln!(
                "  [distill {}] {}/{} rows · teacher acc so far {:.3}",
                spec.name,
                durs_ms.len(),
                docs.len(),
                hits as f64 / durs_ms.len() as f64
            );
        }
    }
    let out_bytes = out_writer.into_bytes();

    let (p50, p99, _support) = percentile_us(&durs_ms);
    let teacher_accuracy = hits as f64 / docs.len() as f64;

    std::fs::create_dir_all(out_dir).map_err(|e| format!("create {}: {e}", out_dir.display()))?;
    let path = out_dir.join(format!("{}_teacher.bin", spec.name));
    std::fs::write(&path, &out_bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    let seal = blake3::hash(&out_bytes);
    let sidecar = out_dir.join(format!("{}_teacher.bin.blake3", spec.name));
    std::fs::write(&sidecar, format!("{}\n", seal.to_hex()))
        .map_err(|e| format!("write {}: {e}", sidecar.display()))?;

    eprintln!(
        "  [distill {}] done: {} rows · {} classes · teacher acc {:.4} · p50 {} ms · {:.1}s",
        spec.name,
        docs.len(),
        n_classes,
        teacher_accuracy,
        p50,
        t_start.elapsed().as_secs_f64()
    );
    Ok(DistillSuite {
        name: spec.name.to_string(),
        checkpoint: teacher.provenance().to_string(),
        n_rows: docs.len(),
        n_classes,
        teacher_accuracy,
        latency_p50_ms: p50 as f64,
        latency_p99_ms: p99 as f64,
        seconds: t_start.elapsed().as_secs_f64(),
        path: path.display().to_string(),
        blake3: seal.to_hex()[..16].to_string(),
    })
}

/// Run the teacher pass over the requested suites (empty = all six T3
/// suites). Early-exit: no eval lane, no test row. A suite that fails is a
/// named loud skip — the mode still exits Ok for the suites that ran (the
/// caller prints the skips; the record decides whether they matter).
pub fn run_distill(
    opts: &RunOptions,
    out_dir: &Path,
    limit: usize,
    teacher_name: &str,
) -> Result<DistillOutput, String> {
    let device_env = std::env::var("LAYA_DEVICE").unwrap_or_else(|_| "default".into());
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
        if !is_distill_suite(spec.name) {
            skipped.push(format!("{}: {}", spec.name, t3_skip_reason(spec.name)));
            continue;
        }
        match distill_suite(spec, &opts.datasets_dir, out_dir, limit, teacher_name) {
            Ok(s) => suites.push(s),
            Err(e) => skipped.push(format!("{}: {e}", spec.name)),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "distill: no suite ran ({} skip/failure line(s), first: {})",
            skipped.len(),
            skipped.first().map_or("none", String::as_str)
        ));
    }
    Ok(DistillOutput {
        meta: DistillMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            host: hostname_refusing_unknown(),
            device: if teacher_name == "laya" {
                format!("teacher=laya; laya_device={device_env}")
            } else {
                // The oracle lanes carry their own device env
                // (BEKKO_PY_DEVICE / the openthai service); the per-suite
                // RIDT header's teacher/checkpoint fields carry the
                // model-side provenance.
                format!("teacher={teacher_name}; oracle-lane device env")
            },
            datasets_dir: opts.datasets_dir.display().to_string(),
            limit,
            row_rule: "train rows only (the no-cheat law, riir-reflex Issue 038); the test \
                       split stays reserved for the arena's single read",
        },
        suites,
        skipped,
    })
}

/// The markdown rendering (the record's table block).
#[must_use]
pub fn render_distill_markdown(out: &DistillOutput) -> String {
    let mut s = String::new();
    s.push_str("# Distill teacher pass — teacher probabilities over train rows (riir-train Issue 576 T3 / Plan 426 T1)\n\n");
    s.push_str(&format!(
        "- date {} · sha {} · host {} · device {}\n- datasets {} · limit {}\n- rows: {}\n",
        out.meta.date_utc,
        out.meta.git_sha,
        out.meta.host,
        out.meta.device,
        out.meta.datasets_dir,
        out.meta.limit,
        out.meta.row_rule
    ));
    s.push_str(
        "\n| suite | ckpt | rows | classes | teacher acc | p50 ms | p99 ms | s | blake3 |\n",
    );
    s.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for v in &out.suites {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {:.4} | {:.1} | {:.1} | {:.1} | {} |\n",
            v.name,
            v.checkpoint,
            v.n_rows,
            v.n_classes,
            v.teacher_accuracy,
            v.latency_p50_ms,
            v.latency_p99_ms,
            v.seconds,
            v.blake3
        ));
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

    fn probs(pairs: &[(&str, f64)]) -> Vec<(String, f64)> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    // ── Issue 623's seam-finding root fix: the student-universe law ────

    #[test]
    fn student_universe_dispatches_by_suite() {
        // The name-bound int-ClassLabel suites derive the POSITIONAL
        // bijection from the presented keys; pi is the fixed noul pair;
        // everything else is identity.
        let xnli_keys = [
            "entailment".to_string(),
            "neutral".to_string(),
            "contradiction".to_string(),
        ];
        assert_eq!(
            StudentUniverse::derive("xnli_en", &xnli_keys).unwrap(),
            StudentUniverse::PositionalInt(xnli_keys.to_vec())
        );
        assert_eq!(
            StudentUniverse::derive("ag_news", &xnli_keys).unwrap(),
            StudentUniverse::PositionalInt(xnli_keys.to_vec())
        );
        assert_eq!(
            StudentUniverse::derive("emotion_syn", &xnli_keys).unwrap(),
            StudentUniverse::PositionalInt(xnli_keys.to_vec()),
            "emotion_syn rides emotion's positional law"
        );
        assert_eq!(
            StudentUniverse::derive("prompt_injections", &[]).unwrap(),
            StudentUniverse::NoulPair
        );
        for s in ["massive_intent_en", "banking77", "sst5", "sst5_syn"] {
            assert_eq!(
                StudentUniverse::derive(s, &[]).unwrap(),
                StudentUniverse::Identity,
                "{s}"
            );
        }
        // An implausible presented-key count refuses (never a confident
        // empty bijection).
        assert!(StudentUniverse::derive("xnli_en", &[]).is_err());
    }

    #[test]
    fn positional_int_translates_inverts_the_corpus_law() {
        // train_row_label's own law: corpus name = KEYS[int]. The writer's
        // inverse: name → its position. Round-trips on every class.
        let keys: Vec<String> = ["world", "sports", "business", "sci_tech"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let su = StudentUniverse::derive("ag_news", &keys).unwrap();
        for (i, k) in keys.iter().enumerate() {
            assert_eq!(su.translate(k).unwrap(), i.to_string());
        }
        // A name absent from the presented order is the pin class — loud.
        assert!(su.translate("not_a_key").is_err());
    }

    #[test]
    fn noul_pair_maps_the_fixed_pair() {
        let su = StudentUniverse::derive("prompt_injections", &[]).unwrap();
        assert_eq!(su.translate("0").unwrap(), "no");
        assert_eq!(su.translate("1").unwrap(), "yes");
        assert!(su.translate("2").is_err());
        assert!(su.translate("maybe").is_err());
    }

    #[test]
    fn apply_sorts_and_permutes_the_universe() {
        // xnli shape: corpus classes arrive SORTED by name; the student
        // universe is the int strings, sorted lexically; the permutation
        // carries row-level indices across.
        let keys: Vec<String> = ["entailment", "neutral", "contradiction"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let su = StudentUniverse::derive("xnli_en", &keys).unwrap();
        let corpus: Vec<String> = {
            let mut c = keys.clone();
            c.sort(); // contradiction, entailment, neutral
            c
        };
        let (student, perm) = su.apply(&corpus).unwrap();
        assert_eq!(
            student,
            vec!["0".to_string(), "1".to_string(), "2".to_string()]
        );
        // corpus[0]="contradiction"→int 2→student[2]; corpus[1]="entailment"→0;
        // corpus[2]="neutral"→1.
        assert_eq!(perm, vec![2, 0, 1]);
        // Identity: sorted input, identity permutation.
        let id = StudentUniverse::Identity;
        let (s, p) = id.apply(&["b".to_string(), "a".to_string()]).unwrap();
        assert_eq!(s, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(p, vec![1, 0]);
    }

    #[test]
    fn apply_refuses_non_injective_mappings() {
        // Two corpus names colliding onto one student label — the silent-
        // corruption class — refuses loud.
        let keys: Vec<String> = ["a", "b", "a"].iter().map(|s| (*s).to_string()).collect();
        let su = StudentUniverse::PositionalInt(keys);
        // "a" appears at positions 0 and 2 — translating TWO DIFFERENT
        // corpus classes ("a" and "a") is degenerate input the corpus side
        // never produces; the injectivity wall still holds:
        let e = su.apply(&["a".to_string(), "a".to_string()]).unwrap_err();
        assert!(e.contains("non-injective"), "{e}");
    }

    #[test]
    fn keymap_dispatches_the_t3_suites() {
        assert_eq!(KeyMap::for_suite("massive_intent_en"), Some(KeyMap::Name));
        assert_eq!(KeyMap::for_suite("banking77"), Some(KeyMap::Name));
        // Issue 623 T1 (2026-10-11): the option-KEY-name suites — their
        // train_row_label produces names, so Name is the only join that
        // survives the per-row gold pin (the first pplx capture proved
        // FixedInt REDs on row 0 for these).
        for s in ["ag_news", "emotion", "emotion_syn", "xnli_en"] {
            assert_eq!(KeyMap::for_suite(s), Some(KeyMap::Name), "{s}");
        }
        // The catch-all-arm int-string suites.
        for s in ["sst5", "sst5_syn", "prompt_injections"] {
            assert_eq!(KeyMap::for_suite(s), Some(KeyMap::FixedInt), "{s}");
        }
        assert_eq!(KeyMap::for_suite("typed_decisions"), None);
        assert_eq!(T3_SUITES.len(), 6);
        for s in EXTENSION_SUITES {
            assert!(is_distill_suite(s), "{s}");
            assert!(
                KeyMap::for_suite(s).is_some(),
                "{s}: a capture suite needs a keymap"
            );
        }
    }

    #[test]
    fn fixed_int_maps_by_position_into_the_student_order() {
        // Student universe arrived at label "1" first: classes = [1, 0, 2].
        let classes = vec!["1".to_string(), "0".to_string(), "2".to_string()];
        let p = probs(&[("a", 0.1), ("b", 0.6), ("c", 0.3)]);
        let (gold, pick, q) = map_targets(KeyMap::FixedInt, "2", 2, &p, &classes).expect("map");
        assert_eq!(gold, 2, "doc label \"2\" is classes[2]");
        assert_eq!(
            pick, 0,
            "argmax at position 1 → int label \"1\" = classes[0]"
        );
        // position 0 (p .1) → class "0" → slot 1; position 1 (.6) → "1" → 0; 2 (.3) → "2" → 2.
        assert_eq!(q, vec![0.6, 0.1, 0.3]);
    }

    #[test]
    fn name_maps_by_key_and_pins_gold() {
        let classes: Vec<String> = ["arrange_flight", "check_weather", "send_email"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let p = probs(&[
            ("check_weather", 0.2),
            ("send_email", 0.7),
            ("arrange_flight", 0.1),
        ]);
        let (gold, pick, q) =
            map_targets(KeyMap::Name, "send_email", 1, &p, &classes).expect("map");
        assert_eq!(gold, 2);
        assert_eq!(pick, 2);
        assert_eq!(q, vec![0.1, 0.2, 0.7]);
    }

    #[test]
    fn gold_pin_refuses_drift_in_both_directions() {
        let classes: Vec<String> = ["0", "1", "2"].iter().map(|s| (*s).to_string()).collect();
        let p = probs(&[("a", 0.5), ("b", 0.3), ("c", 0.2)]);
        // FixedInt: doc label disagrees with the presented gold position.
        assert!(map_targets(KeyMap::FixedInt, "1", 2, &p, &classes).is_err());
        // Name: the presented key at gold is not the doc label.
        let pn = probs(&[("x", 0.5), ("y", 0.5)]);
        let cn: Vec<String> = vec!["x".to_string(), "y".to_string()];
        assert!(map_targets(KeyMap::Name, "y", 0, &pn, &cn).is_err());
        // FixedInt width mismatch (presented ≠ classes).
        let p2 = probs(&[("a", 1.0)]);
        assert!(map_targets(KeyMap::FixedInt, "0", 0, &p2, &classes).is_err());
    }

    #[test]
    fn argmax_ties_to_the_lowest_position() {
        let p = probs(&[("a", 0.4), ("b", 0.4), ("c", 0.2)]);
        assert_eq!(argmax_lowest_pos(&p), 0);
        let p2 = probs(&[("a", 0.1), ("b", 0.4), ("c", 0.4)]);
        assert_eq!(argmax_lowest_pos(&p2), 1);
    }

    // ── Plan 426 T1: the openthai keying seam ─────────────────────────

    fn choice_question(criteria: serde_json::Value) -> crate::harness::suites::SuiteQuestion {
        crate::harness::suites::SuiteQuestion {
            qid: "q".into(),
            kind: crate::harness::suites::QKind::Choice,
            instructions: String::new(),
            criteria,
        }
    }

    #[test]
    fn openthai_keying_reads_the_criteria_key_order() {
        let q = choice_question(serde_json::json!({"b_key": 1, "a_key": 2, "c_key": 3}));
        let keyed = openthai_keyed_probs(&q, &[0.2, 0.7, 0.1]).expect("keyed");
        let keys: Vec<&str> = keyed.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            keys,
            vec!["b_key", "a_key", "c_key"],
            "the criteria object's own key order (the same iteration map_answers reads)"
        );
        assert_eq!(keyed[1].1, 0.7);
    }

    #[test]
    fn openthai_keying_scores_and_noul_speak_the_fixed_int_spelling() {
        let mut q = choice_question(serde_json::json!(["l0", "l1", "l2"]));
        q.kind = crate::harness::suites::QKind::Score;
        let keyed = openthai_keyed_probs(&q, &[0.1, 0.6, 0.3]).expect("keyed");
        assert_eq!(
            keyed.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
            vec!["0", "1", "2"],
            "the level-index spelling map_targets' FixedInt arm reads"
        );
        q.kind = crate::harness::suites::QKind::Noul;
        q.criteria = serde_json::Value::Null;
        let keyed = openthai_keyed_probs(&q, &[0.9, 0.1]).expect("keyed");
        assert_eq!(
            keyed
                .iter()
                .map(|(k, v)| (k.as_str(), *v))
                .collect::<Vec<_>>(),
            vec![("0", 0.9), ("1", 0.1)],
            "their p(yes) arrives as [p_no, p_yes] — the gold idx space"
        );
    }

    #[test]
    fn openthai_keying_refuses_width_drift() {
        let q = choice_question(serde_json::json!({"a": 1, "b": 2}));
        assert!(openthai_keyed_probs(&q, &[0.5, 0.3, 0.2]).is_err());
        // A choice question without an object has no keys to speak.
        let mut bad = choice_question(serde_json::json!(["x"]));
        bad.kind = crate::harness::suites::QKind::Choice;
        assert!(openthai_keyed_probs(&bad, &[1.0]).is_err());
    }

    #[test]
    fn bekko_teacher_pins_default_to_400m_and_revision_follows_model() {
        // The default: the 400M teacher at its own release revision
        // (riir-train Issue 609 — Bench 107: the 400M above the 68M on
        // all 9 board suites).
        let (model, revision) = bekko_teacher_pins_with(|_| None);
        assert_eq!(model, "hotchpotch/bekko-system-one-v0-400m");
        assert_eq!(revision, "4aeb85b9d4042d75d8b8adf6ff7ba9e4629510ba");
        // An EXPLICIT 68M model keeps its own release revision — the
        // Issue-608 posture stays reachable by name.
        let (model, revision) = bekko_teacher_pins_with(|k| {
            (k == "BEKKO_MODEL").then(|| "hotchpotch/bekko-system-one-v0-68m".to_string())
        });
        assert_eq!(model, "hotchpotch/bekko-system-one-v0-68m");
        assert_eq!(revision, "6eb1bae2d35066b0d634fabaf8c79beafc6fd9f1");
        // An explicit UNKNOWN model falls back to the 17M revision
        // (the script's own default pin) — never a known model's
        // revision on a foreign model id.
        let (model, revision) =
            bekko_teacher_pins_with(|k| (k == "BEKKO_MODEL").then(|| "other/model".to_string()));
        assert_eq!(model, "other/model");
        assert_eq!(revision, "2147c3d9d00559bf972616c2589eee967e266f3b");
        // An explicit revision override wins over all defaults.
        let (_, revision) = bekko_teacher_pins_with(|k| {
            (k == "BEKKO_REVISION").then(|| "feedfacedeadbeef".to_string())
        });
        assert_eq!(revision, "feedfacedeadbeef");
    }
}
