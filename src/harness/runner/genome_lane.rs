//! Issue 038 T5 — the joint blend-genome selection (`--genome-select`),
//! the reflexer-style self-evolve scaffolding over the score blend: the
//! scoring-time scales `{route, head, nb(+α,+view), oc, ridge}` as one
//! genome line, hill-climbed on the stratified selection slice with a
//! margin-gated acceptance, test read once at the end.
//!
//! WHY: the shipped posture was composed by four one-at-a-time greedy
//! ladders in a fixed order (head → nb → oc → ridge), each selected at
//! the then-current posture — greedy order effects are exactly the
//! interaction class the joint walk fixes (e.g. emotion selected ridge@8
//! at the then-current nb posture; the walk can find nb@lower + ridge@8
//! beating it on the same slice).
//!
//! COST SHAPE: the search builds ONCE per (α, view) with every lever's
//! tables FITTED (positive placeholder scales force the fits), then moves
//! scales at scoring time via [`DecisionEngine::set_blend_scales`] — a
//! coordinate eval is one sel-slice scoring pass, not a rebuild.
//!
//! PROTOCOL (the shared no-cheat law): the walk reads the selection slice
//! only; the walk end must clear the SEED by [`HEAD_SELECT_MARGIN`] (the
//! lanes' own promotion bar) or the posture is HELD — the published
//! posture stands and the final engine is the plain rebuild at the
//! untouched config (byte-identical by construction). λ stays fixed
//! ([`super::ridge_lane::RIDGE_LAMBDA`], build-time) and the noul
//! polarity stays fixed (already cal-selected) — the genome refines the
//! scale coordinates plus the discrete α/view table shape.

use super::*;
#[cfg(feature = "nb_scope")]
use crate::nb_scope::{NbAlpha, NbView};

/// A candidate value for one genome coordinate: a blend scale or a
/// discrete table-shape name (α / view).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LadderValue {
    Scale(f32),
    Discrete(&'static str),
}

impl std::fmt::Display for LadderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scale(s) => write!(f, "{s}"),
            Self::Discrete(name) => write!(f, "{name}"),
        }
    }
}

/// One genome point: the coordinate values plus the selection-slice
/// accuracy that scored it.
#[derive(Debug, Clone, Serialize)]
pub struct GenomePoint {
    pub route_scale: f32,
    pub head_scale: f32,
    /// Count-table scale + smoothing α + event view (issue 038 T1/T3).
    #[cfg(feature = "nb_scope")]
    pub nb_scale: f32,
    #[cfg(feature = "nb_scope")]
    pub nb_alpha: &'static str,
    #[cfg(feature = "nb_scope")]
    pub nb_view: &'static str,
    /// Option-conditioned scale (issue 038 T7b).
    #[cfg(feature = "option_cond")]
    pub oc_scale: f32,
    /// NBSVM ridge scale (issue 038 T7a).
    #[cfg(feature = "nb_ridge")]
    pub ridge_scale: f32,
    pub cal_acc: f64,
}

impl GenomePoint {
    /// The route/head skeleton; the lever scales are filled per-feature
    /// by the constructor below (the fields are cfg-gated, so tests and
    /// the lane both start here).
    fn skeleton(route_scale: f32, head_scale: f32) -> Self {
        Self {
            route_scale,
            head_scale,
            #[cfg(feature = "nb_scope")]
            nb_scale: 0.0,
            #[cfg(feature = "nb_scope")]
            nb_alpha: "observed-laplace",
            #[cfg(feature = "nb_scope")]
            nb_view: "bag",
            #[cfg(feature = "option_cond")]
            oc_scale: 0.0,
            #[cfg(feature = "nb_ridge")]
            ridge_scale: 0.0,
            cal_acc: 0.0,
        }
    }

    /// The five blend scales in [`DecisionEngine::set_blend_scales`]
    /// order, feature-neutral (a compiled-out lever contributes 0 — a
    /// legal scale there, never a silent fire).
    fn scales(&self) -> (f32, f32, f32, f32, f32) {
        #[cfg(feature = "nb_scope")]
        let nb = self.nb_scale;
        #[cfg(not(feature = "nb_scope"))]
        let nb = 0.0f32;
        #[cfg(feature = "option_cond")]
        let oc = self.oc_scale;
        #[cfg(not(feature = "option_cond"))]
        let oc = 0.0f32;
        #[cfg(feature = "nb_ridge")]
        let ridge = self.ridge_scale;
        #[cfg(not(feature = "nb_ridge"))]
        let ridge = 0.0f32;
        (self.route_scale, self.head_scale, nb, oc, ridge)
    }
}

/// A genome coordinate: identity + its candidate ladder. The ladders span
/// the shipped regimes plus the headroom the round-2 extension lesson
/// asked for (a saturated top rung means the ladder was too short).
struct CoordSpec {
    id: CoordId,
    ladder: Vec<LadderValue>,
}

/// Coordinate identities; `get`/`set` are the only writers of a
/// [`GenomePoint`] during the walk (the pure [`descend`] core needs no
/// engine — the lane's eval closure does).
#[derive(Debug, Clone, Copy)]
enum CoordId {
    Route,
    Head,
    #[cfg(feature = "nb_scope")]
    NbScale,
    #[cfg(feature = "nb_scope")]
    NbAlpha,
    #[cfg(feature = "nb_scope")]
    NbView,
    #[cfg(feature = "option_cond")]
    Oc,
    #[cfg(feature = "nb_ridge")]
    Ridge,
}

impl CoordId {
    fn name(self) -> &'static str {
        match self {
            Self::Route => "route",
            Self::Head => "head",
            #[cfg(feature = "nb_scope")]
            Self::NbScale => "nb",
            #[cfg(feature = "nb_scope")]
            Self::NbAlpha => "nb_alpha",
            #[cfg(feature = "nb_scope")]
            Self::NbView => "nb_view",
            #[cfg(feature = "option_cond")]
            Self::Oc => "oc",
            #[cfg(feature = "nb_ridge")]
            Self::Ridge => "ridge",
        }
    }

    fn get(self, p: &GenomePoint) -> LadderValue {
        match self {
            Self::Route => LadderValue::Scale(p.route_scale),
            Self::Head => LadderValue::Scale(p.head_scale),
            #[cfg(feature = "nb_scope")]
            Self::NbScale => LadderValue::Scale(p.nb_scale),
            #[cfg(feature = "nb_scope")]
            Self::NbAlpha => LadderValue::Discrete(p.nb_alpha),
            #[cfg(feature = "nb_scope")]
            Self::NbView => LadderValue::Discrete(p.nb_view),
            #[cfg(feature = "option_cond")]
            Self::Oc => LadderValue::Scale(p.oc_scale),
            #[cfg(feature = "nb_ridge")]
            Self::Ridge => LadderValue::Scale(p.ridge_scale),
        }
    }

    /// A ladder value is written back verbatim; a scale coordinate's
    /// ladder only ever carries [`LadderValue::Scale`] (constructor
    /// invariant), so the `if let` cannot silently drop a discrete value.
    fn set(self, p: &mut GenomePoint, v: LadderValue) {
        match self {
            Self::Route => {
                if let LadderValue::Scale(s) = v {
                    p.route_scale = s;
                }
            }
            Self::Head => {
                if let LadderValue::Scale(s) = v {
                    p.head_scale = s;
                }
            }
            #[cfg(feature = "nb_scope")]
            Self::NbScale => {
                if let LadderValue::Scale(s) = v {
                    p.nb_scale = s;
                }
            }
            #[cfg(feature = "nb_scope")]
            Self::NbAlpha => {
                if let LadderValue::Discrete(name) = v {
                    p.nb_alpha = name;
                }
            }
            #[cfg(feature = "nb_scope")]
            Self::NbView => {
                if let LadderValue::Discrete(name) = v {
                    p.nb_view = name;
                }
            }
            #[cfg(feature = "option_cond")]
            Self::Oc => {
                if let LadderValue::Scale(s) = v {
                    p.oc_scale = s;
                }
            }
            #[cfg(feature = "nb_ridge")]
            Self::Ridge => {
                if let LadderValue::Scale(s) = v {
                    p.ridge_scale = s;
                }
            }
        }
    }
}

/// One accepted walk move (the audit trail: what moved, when, and the
/// slice accuracy that paid for it).
#[derive(Debug, Clone, Serialize)]
pub struct GenomeMove {
    pub coord: &'static str,
    pub pass: usize,
    pub from: String,
    pub to: String,
    pub cal_acc: f64,
}

/// The cal-selected joint genome for a suite: the seed (the composed
/// greedy posture), the walk end, whether the walk end cleared the
/// promotion bar (HELD = the published posture stands), and the moves.
#[derive(Debug, Clone, Serialize)]
pub struct GenomeSelection {
    pub held: bool,
    /// The acceptance rule this walk used (disclosed per run; the house
    /// arming bar 0.05 by default, the refinement rounds pass their own).
    pub accept_margin: f64,
    pub passes: usize,
    pub seed: GenomePoint,
    pub selected: GenomePoint,
    pub moves: Vec<GenomeMove>,
}

/// A candidate must beat the coordinate's current best by this much to
/// move — ≈2 questions on a 200-case slice, the noise floor for a MOVE
/// (the final acceptance vs the seed carries the stricter promotion bar).
const GENOME_MOVE_MARGIN: f64 = 0.01;

/// Coordinate-descent passes are bounded; the walk converges in 1-2 on
/// these ladders (measured on the first run — raise only with evidence).
const GENOME_MAX_PASSES: usize = 2;

const G_ROUTE_LADDER: [LadderValue; 5] = [
    LadderValue::Scale(2.0),
    LadderValue::Scale(4.0),
    LadderValue::Scale(8.0),
    LadderValue::Scale(16.0),
    LadderValue::Scale(32.0),
];
const G_HEAD_LADDER: [LadderValue; 5] = [
    LadderValue::Scale(0.0),
    LadderValue::Scale(0.25),
    LadderValue::Scale(0.5),
    LadderValue::Scale(1.0),
    LadderValue::Scale(2.0),
];
#[cfg(feature = "nb_scope")]
const G_NB_LADDER: [LadderValue; 6] = [
    LadderValue::Scale(0.0),
    LadderValue::Scale(1.0),
    LadderValue::Scale(4.0),
    LadderValue::Scale(16.0),
    LadderValue::Scale(32.0),
    LadderValue::Scale(64.0),
];
#[cfg(feature = "nb_scope")]
const G_ALPHA_LADDER: [LadderValue; 2] = [
    LadderValue::Discrete("observed-laplace"),
    LadderValue::Discrete("fixed-1"),
];
#[cfg(feature = "nb_scope")]
const G_VIEW_LADDER: [LadderValue; 2] = [
    LadderValue::Discrete("bag"),
    LadderValue::Discrete("pair"),
];
#[cfg(feature = "option_cond")]
const G_OC_LADDER: [LadderValue; 7] = [
    LadderValue::Scale(0.0),
    LadderValue::Scale(0.25),
    LadderValue::Scale(0.5),
    LadderValue::Scale(1.0),
    LadderValue::Scale(2.0),
    LadderValue::Scale(4.0),
    LadderValue::Scale(8.0),
];
#[cfg(feature = "nb_ridge")]
const G_RIDGE_LADDER: [LadderValue; 7] = [
    LadderValue::Scale(0.0),
    LadderValue::Scale(0.5),
    LadderValue::Scale(1.0),
    LadderValue::Scale(2.0),
    LadderValue::Scale(4.0),
    LadderValue::Scale(8.0),
    LadderValue::Scale(16.0),
];

/// The pure walk core: coordinate descent over `coords`, a candidate
/// moves only when it clears the coordinate's starting accuracy by
/// `move_margin` AND is the argmax among the clearing candidates (the
/// lanes' own promotion shape, applied per coordinate); ties hold the
/// current value. Returns the walk end, the accepted moves and the pass
/// count. No engine, no I/O — fully deterministic and unit-testable.
fn descend(
    mut cur: GenomePoint,
    coords: &[CoordSpec],
    mut eval: impl FnMut(&GenomePoint) -> Result<f64, String>,
    move_margin: f64,
    max_passes: usize,
) -> Result<(GenomePoint, Vec<GenomeMove>, usize), String> {
    let mut moves = Vec::new();
    let mut passes = 0usize;
    for pass in 0..max_passes {
        let mut moved = false;
        for c in coords {
            let cur_val = c.id.get(&cur);
            let floor = cur.cal_acc;
            let mut best_val = cur_val;
            let mut best_acc = floor;
            for &cand in &c.ladder {
                if cand == cur_val {
                    continue;
                }
                let mut trial = cur.clone();
                c.id.set(&mut trial, cand);
                let acc = eval(&trial)?;
                if acc >= floor + move_margin && acc > best_acc {
                    best_acc = acc;
                    best_val = cand;
                }
            }
            if best_val != cur_val {
                moves.push(GenomeMove {
                    coord: c.id.name(),
                    pass,
                    from: cur_val.to_string(),
                    to: best_val.to_string(),
                    cal_acc: best_acc,
                });
                c.id.set(&mut cur, best_val);
                cur.cal_acc = best_acc;
                moved = true;
            }
        }
        passes += 1;
        if !moved {
            break;
        }
    }
    Ok((cur, moves, passes))
}

/// The α name a selected [`NbAlpha`] maps to (the inverse of
/// [`super::nb_lane::alpha_of`]).
#[cfg(feature = "nb_scope")]
fn alpha_name(a: NbAlpha) -> &'static str {
    if matches!(a, NbAlpha::Fixed(_)) {
        "fixed-1"
    } else {
        "observed-laplace"
    }
}

/// The view name a selected [`NbView`] maps to.
#[cfg(feature = "nb_scope")]
fn view_name(v: NbView) -> &'static str {
    if matches!(v, NbView::Pair) {
        "pair"
    } else {
        "bag"
    }
}

/// Cal-side JOINT selection of the blend genome at the already-selected
/// greedy posture (the seed). Same slice, same promotion bar as the
/// per-lever lanes; test read once at the walk end.
#[cfg(feature = "nb_scope")]
#[allow(clippy::too_many_lines)]
pub(super) fn build_genome_selection<const N: usize>(
    inp: &ModellessInput<'_>,
    effective_cap: usize,
    base_cfg: &EngineConfig,
    accept_margin: f64,
    #[cfg(feature = "option_cond")] events_for_pool: impl Fn(&[TrainDoc]) -> Vec<crate::option_cond::OcEvent>,
    #[cfg(not(feature = "option_cond"))] _events_for_pool: (),
) -> Result<GenomeSelection, String> {
    let spec = inp.spec;
    let SelSlice {
        cases,
        state_strs,
        pool,
    } = selection_slice(inp, "genome-select")?;
    #[cfg(feature = "option_cond")]
    let oc_events = events_for_pool(&pool);

    let pair_suite = cases.first().is_some_and(|c| {
        c.state
            .as_object()
            .is_some_and(|m| m.values().filter(|v| v.is_string()).count() >= 2)
    });
    let mut coords: Vec<CoordSpec> = vec![
        CoordSpec {
            id: CoordId::Route,
            ladder: G_ROUTE_LADDER.to_vec(),
        },
        CoordSpec {
            id: CoordId::Head,
            ladder: G_HEAD_LADDER.to_vec(),
        },
    ];
    #[cfg(feature = "nb_scope")]
    {
        coords.push(CoordSpec {
            id: CoordId::NbScale,
            ladder: G_NB_LADDER.to_vec(),
        });
        coords.push(CoordSpec {
            id: CoordId::NbAlpha,
            ladder: G_ALPHA_LADDER.to_vec(),
        });
        coords.push(CoordSpec {
            id: CoordId::NbView,
            ladder: G_VIEW_LADDER
                .iter()
                .copied()
                .filter(|v| pair_suite || *v != LadderValue::Discrete("pair"))
                .collect(),
        });
    }
    // No events → the (qid, option) tables cannot be fitted → the oc
    // coordinate cannot fire; it stays out of the walk entirely.
    #[cfg(feature = "option_cond")]
    if !oc_events.is_empty() {
        coords.push(CoordSpec {
            id: CoordId::Oc,
            ladder: G_OC_LADDER.to_vec(),
        });
    }
    #[cfg(feature = "nb_ridge")]
    coords.push(CoordSpec {
        id: CoordId::Ridge,
        ladder: G_RIDGE_LADDER.to_vec(),
    });

    let mut seed = GenomePoint::skeleton(base_cfg.route_scale, base_cfg.head_scale);
    #[cfg(feature = "nb_scope")]
    {
        seed.nb_scale = base_cfg.nb_scale;
        seed.nb_alpha = alpha_name(base_cfg.nb_alpha);
        seed.nb_view = view_name(base_cfg.nb_view);
    }
    #[cfg(feature = "option_cond")]
    {
        seed.oc_scale = base_cfg.oc_scale;
    }
    #[cfg(feature = "nb_ridge")]
    {
        seed.ridge_scale = base_cfg.ridge_scale;
    }

    // The FIT-universe build: positive placeholder scales force every
    // lever's fit once; the scales the search scores are set per-eval via
    // set_blend_scales. The discrete (α, view) shape rebuilds (they
    // parameterize the table fit itself); λ and the noul polarity ride
    // the seed config unchanged. The oc placeholder is 0 when the suite
    // carries no events — a plain build refuses an armed oc scale (the
    // fail-closed gate) and there is nothing to fit.
    #[cfg(feature = "option_cond")]
    let oc_placeholder: f32 = if oc_events.is_empty() { 0.0 } else { 1.0 };
    let fit_cfg = EngineConfig {
        head_scale: 1.0,
        #[cfg(feature = "nb_scope")]
        nb_scale: 1.0,
        #[cfg(feature = "option_cond")]
        oc_scale: oc_placeholder,
        #[cfg(feature = "nb_ridge")]
        ridge_scale: 1.0,
        score_threshold: 2.0,
        distance_threshold: 2.0,
        ..base_cfg.clone()
    };
    #[cfg(feature = "nb_scope")]
    let mut last_discrete: Option<(&'static str, &'static str)> = None;
    let mut engine_opt: Option<DecisionEngine<N, EMBED_DIM>> = None;

    let mut eval = |p: &GenomePoint| -> Result<f64, String> {
        #[cfg(feature = "nb_scope")]
        let want = (p.nb_alpha, p.nb_view);
        #[cfg(feature = "nb_scope")]
        if engine_opt.is_none() || last_discrete != Some(want) {
            let cfg = EngineConfig {
                nb_alpha: super::nb_lane::alpha_of(want.0),
                nb_view: super::nb_lane::view_of(want.1),
                ..fit_cfg.clone()
            };
            // The oc placeholder scale needs the event-carrying build when
            // the suite carries events (plain `build_engine` refuses an
            // armed oc scale — the fail-closed gate).
            #[cfg(feature = "option_cond")]
            let built = if !oc_events.is_empty() {
                build_engine_oc_with::<N>(
                    spec.name,
                    &pool,
                    inp.labels,
                    effective_cap,
                    cfg,
                    &[],
                    &oc_events,
                )
            } else {
                build_engine::<N>(spec.name, &pool, inp.labels, effective_cap, cfg)
            };
            #[cfg(not(feature = "option_cond"))]
            let built = build_engine::<N>(spec.name, &pool, inp.labels, effective_cap, cfg);
            let (engine, _) = built?;
            engine_opt = Some(engine);
            last_discrete = Some(want);
        }
        let engine = engine_opt.as_mut().expect("built above");
        let (route, head, nb, oc, ridge) = p.scales();
        engine
            .set_blend_scales(route, head, nb, oc, ridge)
            .map_err(|e| e.to_string())?;
        let (ev, _) = eval_engine(engine, &cases, &state_strs, false)?;
        Ok(hard_metrics(&ev.forced_rows(&cases)).accuracy)
    };

    seed.cal_acc = eval(&seed)?;
    #[cfg(feature = "nb_scope")]
    let nb_desc = format!(
        " nb {} α {} view {}",
        seed.nb_scale, seed.nb_alpha, seed.nb_view
    );
    #[cfg(not(feature = "nb_scope"))]
    let nb_desc = String::new();
    #[cfg(feature = "option_cond")]
    let oc_desc = format!(" oc {}", seed.oc_scale);
    #[cfg(not(feature = "option_cond"))]
    let oc_desc = String::new();
    #[cfg(feature = "nb_ridge")]
    let ridge_desc = format!(" ridge {}", seed.ridge_scale);
    #[cfg(not(feature = "nb_ridge"))]
    let ridge_desc = String::new();
    eprintln!(
        "    genome-select: seed (route {} head {}{}{}{}) → sel-slice acc {:.4}",
        seed.route_scale,
        seed.head_scale,
        nb_desc,
        oc_desc,
        ridge_desc,
        seed.cal_acc
    );

    let (walk_end, moves, passes) = descend(
        seed.clone(),
        &coords,
        &mut eval,
        GENOME_MOVE_MARGIN,
        GENOME_MAX_PASSES,
    )?;
    for m in &moves {
        eprintln!(
            "    genome-select: pass {} {} {} → {} (sel-slice acc {:.4})",
            m.pass, m.coord, m.from, m.to, m.cal_acc
        );
    }
    let held = walk_end.cal_acc < seed.cal_acc + accept_margin;
    let selected = if held {
        eprintln!(
            "    genome-select: HELD — the walk clears the seed by less than the \
             {:+.0} pt acceptance bar; the composed posture stands (byte-identical)",
            accept_margin * 100.0
        );
        seed.clone()
    } else {
        eprintln!(
            "    genome-select: ACCEPTED — the suite's row below is the single test \
             read, at this genome"
        );
        walk_end
    };
    Ok(GenomeSelection {
        held,
        accept_margin,
        passes,
        moves,
        seed,
        selected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A concave surface over (route, head) with the peak ON the ladder
    /// (route 16, head 1.0): the walk must find both rungs, across two
    /// passes (route first, then head), and hold on pass 2.
    #[test]
    fn walk_finds_the_ladder_optimum() {
        let surface = |p: &GenomePoint| -> Result<f64, String> {
            Ok((0.5 - ((p.route_scale - 16.0) / 16.0).powi(2)
                - ((p.head_scale - 1.0) / 1.0).powi(2)) as f64)
        };
        let coords = vec![
            CoordSpec {
                id: CoordId::Route,
                ladder: G_ROUTE_LADDER.to_vec(),
            },
            CoordSpec {
                id: CoordId::Head,
                ladder: G_HEAD_LADDER.to_vec(),
            },
        ];
        let mut seed = GenomePoint::skeleton(2.0, 0.0);
        seed.cal_acc = surface(&seed).unwrap();
        let (end, moves, passes) =
            descend(seed, &coords, surface, GENOME_MOVE_MARGIN, GENOME_MAX_PASSES).unwrap();
        assert_eq!(end.route_scale, 16.0);
        assert_eq!(end.head_scale, 1.0);
        assert_eq!(moves.len(), 2, "one move per coordinate");
        assert_eq!(passes, 2, "pass 1 moves both, pass 2 holds");
        assert!((end.cal_acc - 0.5).abs() < 1e-9, "the peak value");
    }

    /// A sub-margin bump must NOT move the coordinate (the noise floor):
    /// the surface improves by 0.005 (< GENOME_MOVE_MARGIN) at route 4
    /// and stays flat elsewhere — the walk holds the seed exactly.
    #[test]
    fn sub_margin_improvement_holds() {
        let surface = |p: &GenomePoint| -> Result<f64, String> {
            let bonus = if p.route_scale == 4.0 { 0.005 } else { 0.0 };
            Ok(0.5 + bonus)
        };
        let coords = vec![CoordSpec {
            id: CoordId::Route,
            ladder: G_ROUTE_LADDER.to_vec(),
        }];
        let mut seed = GenomePoint::skeleton(8.0, 0.0);
        seed.cal_acc = 0.5;
        let (end, moves, _) =
            descend(seed, &coords, surface, GENOME_MOVE_MARGIN, GENOME_MAX_PASSES).unwrap();
        assert_eq!(end.route_scale, 8.0, "the 0.005 bump is under the move margin");
        assert!(moves.is_empty());
    }

    /// A later candidate that clears the floor by MORE than an earlier
    /// one wins the coordinate (argmax among the clearing candidates —
    /// the lanes' own shape), and ties hold the current value.
    #[test]
    fn argmax_among_clearing_candidates() {
        // route 32 is the best (+0.30), route 16 clears less (+0.10).
        let surface = |p: &GenomePoint| -> Result<f64, String> {
            let bonus = match p.route_scale {
                16.0 => 0.10,
                32.0 => 0.30,
                _ => 0.0,
            };
            Ok(0.5 + bonus)
        };
        let coords = vec![CoordSpec {
            id: CoordId::Route,
            ladder: G_ROUTE_LADDER.to_vec(),
        }];
        let mut seed = GenomePoint::skeleton(8.0, 0.0);
        seed.cal_acc = 0.5;
        let (end, moves, _) =
            descend(seed, &coords, surface, GENOME_MOVE_MARGIN, GENOME_MAX_PASSES).unwrap();
        assert_eq!(end.route_scale, 32.0);
        assert_eq!(moves.len(), 1);
        assert_eq!(end.cal_acc, 0.8);
    }
}
